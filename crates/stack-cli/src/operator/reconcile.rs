use super::crd::{EnvVar, SecretEnvVar, StackApp, StackAppSpec};
use super::finalizer;
use crate::error::Error;
use crate::services::application::APPLICATION_NAME;
use crate::services::{database, deployment, keycloak, nginx, oauth2_proxy, storage};
use k8s_openapi::api::{
    apps::v1::Deployment as KubeDeployment,
    core::v1::{ConfigMap, Secret, Service},
};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client, Resource, ResourceExt};
use kube_runtime::controller::Action;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};

const DEFAULT_DB_DISK_SIZE_GB: i32 = 20;
const DB_NODEPORT_SERVICE_NAME: &str = "postgres-development";
const APP_NODEPORT_SERVICE_NAME: &str = "nginx-development";
const STORAGE_NODEPORT_SERVICE_NAME: &str = "storage-development";
const STACK_DB_CLUSTER_NAME: &str = "stack-db-cluster";
const WEB_APP_REPLICAS: i32 = 1;
const CLOUDFLARE_DEPLOYMENT_NAME: &str = "cloudflared";
const CLOUDFLARE_SECRET_NAME: &str = "cloudflare-credentials";
const CLOUDFLARE_CONFIG_NAME: &str = "cloudflared";

/// Context injected with each `reconcile` and `on_error` method invocation.
pub struct ContextData {
    /// Kubernetes client to make Kubernetes API requests with. Required for K8S resource management.
    client: Client,
}

impl ContextData {
    // Constructs a new instance of ContextData.
    //
    // # Arguments:
    // - `client`: A Kubernetes client to make Kubernetes REST API requests with.
    // Resources will be created and deleted with this client.
    pub fn new(client: Client) -> Self {
        ContextData { client }
    }
}

pub async fn reconcile(app: Arc<StackApp>, context: Arc<ContextData>) -> Result<Action, Error> {
    let client: Client = context.client.clone(); // The `Client` is shared -> a clone from the reference is obtained

    let namespace: String = app.namespace().unwrap_or("default".to_string());
    let name = app.name_any();

    if app.meta().deletion_timestamp.is_some() {
        delete_application_resources(&client, &namespace).await?;
        oauth2_proxy::delete(client.clone(), &namespace).await?;
        nginx::delete_nginx(client.clone(), &namespace).await?;
        keycloak::delete(client.clone(), &namespace).await?;
        storage::delete(client.clone(), &namespace).await?;
        delete_cloudflare_resources(&client, &namespace).await?;
        database::delete(client.clone(), &namespace).await?;
        finalizer::delete(client, &name, &namespace).await?;
        return Ok(Action::await_change());
    }

    finalizer::add(client.clone(), &name, &namespace).await?;

    let insecure_override_passwords = app
        .spec
        .components
        .db
        .as_ref()
        .and_then(|db| db.danger_override_password.clone());
    database::deploy(
        client.clone(),
        &namespace,
        DEFAULT_DB_DISK_SIZE_GB,
        &insecure_override_passwords,
    )
    .await?;

    if let Some(storage_spec) = app.spec.components.storage.as_ref() {
        storage::deploy(client.clone(), &namespace, Some(storage_spec)).await?;
    } else {
        storage::delete(client.clone(), &namespace).await?;
    }

    let auth_hostname = app
        .spec
        .components
        .auth
        .as_ref()
        .and_then(|auth| auth.hostname_url.clone());
    let jwt_value = app
        .spec
        .components
        .auth
        .as_ref()
        .and_then(|auth| auth.danger_override_jwt.clone())
        .unwrap_or_else(|| "1".to_string());

    if let Some(hostname_url) = auth_hostname {
        let realm_config =
            oauth2_proxy::ensure_secret(client.clone(), &namespace, &hostname_url).await?;
        keycloak::ensure_realm(client.clone(), &realm_config).await?;
        oauth2_proxy::deploy(
            client.clone(),
            &namespace,
            &hostname_url,
            app.spec.services.web.port,
        )
        .await?;
        nginx::deploy_nginx(
            &client,
            &namespace,
            nginx::NginxMode::Oidc,
            app.spec.services.web.port,
        )
        .await?;
    } else {
        cleanup_auth_resources(client.clone(), &namespace).await?;
        nginx::deploy_nginx(
            &client,
            &namespace,
            nginx::NginxMode::StaticJwt {
                token: jwt_value.clone(),
            },
            app.spec.services.web.port,
        )
        .await?;
    }

    deploy_web_app(&client, &namespace, &app.spec).await?;
    ensure_optional_nodeports(&client, &namespace, &app.spec).await?;

    Ok(Action::requeue(Duration::from_secs(10)))
}

/// Actions to be taken when a reconciliation fails - for whatever reason.
/// Prints out the error to `stderr` and requeues the resource for another reconciliation after
/// five seconds.
///
/// # Arguments
/// - `resource`: The erroneous resource.
/// - `error`: A reference to the `kube::Error` that occurred during reconciliation.
/// - `_context`: Unused argument. Context Data "injected" automatically by kube-rs.
pub fn on_error(resource: Arc<StackApp>, error: &Error, _context: Arc<ContextData>) -> Action {
    eprintln!("Reconciliation error:\n{:?}.\n{:?}", error, resource);
    Action::requeue(Duration::from_secs(5))
}

async fn deploy_web_app(
    client: &Client,
    namespace: &str,
    spec: &StackAppSpec,
) -> Result<(), Error> {
    let hostname_env = spec
        .components
        .auth
        .as_ref()
        .and_then(|a| a.hostname_url.clone())
        .unwrap_or_default();

    let mut env = vec![json!({"name": "HOSTNAME_URL", "value": hostname_env})];

    if let Some(db_env_name) = spec.services.web.database_url.clone() {
        env.push(json!({
            "name": db_env_name,
            "valueFrom": {
                "secretKeyRef": {
                    "name": "database-urls",
                    "key": "application-url"
                }
            }
        }));
    }

    if let Some(superuser_env_name) = spec.services.web.migrations_database_url.clone() {
        env.push(json!({
            "name": superuser_env_name,
            "valueFrom": {
                "secretKeyRef": {
                    "name": "database-urls",
                    "key": "migrations-url"
                }
            }
        }));
    }

    if let Some(readonly_env_name) = spec.services.web.readonly_database_url.clone() {
        env.push(json!({
            "name": readonly_env_name,
            "valueFrom": {
                "secretKeyRef": {
                    "name": "database-urls",
                    "key": "readonly-url"
                }
            }
        }));
    }

    env.push(json!({
        "name": "WEB_IMAGE",
        "value": spec.services.web.image.clone()
    }));

    append_env_from_spec(
        &mut env,
        &spec.services.web.env,
        &spec.services.web.secret_env,
    );

    let init_container = spec
        .services
        .web
        .init
        .as_ref()
        .map(|init| deployment::InitContainer {
            image_name: init.image.clone(),
            env: build_env(&init.env, &init.secret_env),
            command: None,
        });

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: APPLICATION_NAME.to_string(),
            image_name: spec.services.web.image.clone(),
            replicas: WEB_APP_REPLICAS,
            port: spec.services.web.port,
            env,
            init_container,
            command: None,
            volume_mounts: vec![],
            volumes: vec![],
        },
        namespace,
        spec.services.web.expose_app_port.is_some(),
    )
    .await
}

async fn ensure_optional_nodeports(
    client: &Client,
    namespace: &str,
    spec: &StackAppSpec,
) -> Result<(), Error> {
    if let Some(node_port) = spec
        .components
        .db
        .as_ref()
        .and_then(|db_config| db_config.expose_db_port)
    {
        ensure_nodeport_service(
            client,
            namespace,
            DB_NODEPORT_SERVICE_NAME,
            json!({
                "cnpg.io/cluster": STACK_DB_CLUSTER_NAME,
                "role": "primary"
            }),
            5432,
            node_port,
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, DB_NODEPORT_SERVICE_NAME).await?;
    }

    if let Some(node_port) = spec.services.web.expose_app_port {
        ensure_nodeport_service(
            client,
            namespace,
            APP_NODEPORT_SERVICE_NAME,
            json!({ "app": nginx::NGINX_NAME }),
            nginx::NGINX_PORT,
            node_port,
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, APP_NODEPORT_SERVICE_NAME).await?;
    }

    if let Some(node_port) = spec
        .components
        .storage
        .as_ref()
        .and_then(|storage_config| storage_config.expose_storage_port)
    {
        ensure_nodeport_service(
            client,
            namespace,
            STORAGE_NODEPORT_SERVICE_NAME,
            json!({ "app": storage::STORAGE_NAME }),
            storage::DEFAULT_STORAGE_PORT,
            node_port,
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, STORAGE_NODEPORT_SERVICE_NAME).await?;
    }

    Ok(())
}

async fn delete_application_resources(client: &Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(APPLICATION_NAME).await.is_ok() {
        deployments
            .delete(APPLICATION_NAME, &DeleteParams::default())
            .await?;
    }

    delete_service_if_exists(client, namespace, APPLICATION_NAME).await?;
    delete_service_if_exists(client, namespace, APP_NODEPORT_SERVICE_NAME).await?;
    delete_service_if_exists(client, namespace, DB_NODEPORT_SERVICE_NAME).await?;
    delete_service_if_exists(client, namespace, STORAGE_NODEPORT_SERVICE_NAME).await?;

    Ok(())
}

async fn delete_cloudflare_resources(client: &Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(CLOUDFLARE_DEPLOYMENT_NAME).await.is_ok() {
        deployments
            .delete(CLOUDFLARE_DEPLOYMENT_NAME, &DeleteParams::default())
            .await?;
    }

    let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
    if secrets.get(CLOUDFLARE_SECRET_NAME).await.is_ok() {
        secrets
            .delete(CLOUDFLARE_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    let configs: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    if configs.get(CLOUDFLARE_CONFIG_NAME).await.is_ok() {
        configs
            .delete(CLOUDFLARE_CONFIG_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

async fn ensure_nodeport_service(
    client: &Client,
    namespace: &str,
    name: &str,
    selector: Value,
    target_port: u16,
    node_port: u16,
) -> Result<(), Error> {
    let service = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": name,
            "namespace": namespace
        },
        "spec": {
            "type": "NodePort",
            "selector": selector,
            "ports": [
                {
                    "port": target_port,
                    "targetPort": target_port,
                    "nodePort": node_port
                }
            ]
        }
    });

    let service_api: Api<Service> = Api::namespaced(client.clone(), namespace);
    service_api
        .patch(
            name,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(service),
        )
        .await?;

    Ok(())
}

async fn delete_service_if_exists(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<(), Error> {
    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    if services.get(name).await.is_ok() {
        services.delete(name, &DeleteParams::default()).await?;
    }

    Ok(())
}

async fn cleanup_auth_resources(client: Client, namespace: &str) -> Result<(), Error> {
    oauth2_proxy::delete(client.clone(), namespace).await?;
    keycloak::delete(client, namespace).await?;
    Ok(())
}

fn append_env_from_spec(
    env: &mut Vec<Value>,
    env_vars: &[EnvVar],
    secret_env_vars: &[SecretEnvVar],
) {
    for env_var in env_vars {
        env.push(json!({
            "name": env_var.name,
            "value": env_var.value
        }));
    }

    for env_var in secret_env_vars {
        env.push(json!({
            "name": env_var.name,
            "valueFrom": {
                "secretKeyRef": {
                    "name": env_var.secret_name,
                    "key": env_var.secret_key
                }
            }
        }));
    }
}

fn build_env(env_vars: &[EnvVar], secret_env_vars: &[SecretEnvVar]) -> Vec<Value> {
    let mut env = Vec::new();
    append_env_from_spec(&mut env, env_vars, secret_env_vars);
    env
}
