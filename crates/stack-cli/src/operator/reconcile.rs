use super::crd::{EnvVar, SecretEnvVar, ServiceSpec, StackApp, StackAppSpec};
use super::finalizer;
use crate::error::Error;
use crate::services::{
    cloudflare, database, deployment, document_engine, jwt_secrets, keycloak, nginx, oauth2_proxy,
    postgrest, realtime, selenium, storage, mailhog,
};
use k8s_openapi::api::{
    apps::v1::Deployment as KubeDeployment,
    core::v1::{ConfigMap, Service},
};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client, Resource, ResourceExt};
use kube_runtime::controller::Action;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};

const DEFAULT_DB_DISK_SIZE_GB: i32 = 20;
const DB_NODEPORT_SERVICE_NAME: &str = "postgres-development";
const APP_NODEPORT_SERVICE_NAME: &str = "nginx-development";
const REST_NODEPORT_SERVICE_NAME: &str = "rest-development";
const SELENIUM_NODEPORT_SERVICE_NAME: &str = "selenium-development";
const MAILHOG_NODEPORT_SERVICE_NAME: &str = "mailhog-development";
const WEB_APP_REPLICAS: i32 = 1;
const CLOUDFLARE_DEPLOYMENT_NAME: &str = "cloudflared";
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
        for name in app.spec.services.extra.keys() {
            let deployments: Api<KubeDeployment> =
                Api::namespaced(client.clone(), namespace.as_str());
            if deployments.get(name).await.is_ok() {
                deployments
                    .delete(name, &DeleteParams::default())
                    .await?;
            }
            delete_service_if_exists(&client, &namespace, name).await?;
        }
        delete_application_resources(&client, &namespace, &name).await?;
        oauth2_proxy::delete(client.clone(), &namespace).await?;
        nginx::delete_nginx(client.clone(), &namespace).await?;
        keycloak::delete(client.clone(), &namespace).await?;
        storage::delete(client.clone(), &namespace).await?;
        postgrest::delete(client.clone(), &namespace).await?;
        realtime::delete(client.clone(), &namespace).await?;
        document_engine::delete(client.clone(), &namespace).await?;
        selenium::delete(client.clone(), &namespace).await?;
        mailhog::delete(client.clone(), &namespace).await?;
        jwt_secrets::delete(client.clone(), &namespace).await?;
        delete_cloudflare_resources(&client, &namespace).await?;
        database::delete(client.clone(), &namespace, &name).await?;
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
        &name,
        DEFAULT_DB_DISK_SIZE_GB,
        &insecure_override_passwords,
    )
    .await?;

    if let Some(storage_spec) = app.spec.components.storage.as_ref() {
        storage::deploy(client.clone(), &namespace, &name, Some(storage_spec)).await?;
    } else {
        storage::delete(client.clone(), &namespace).await?;
    }

    if let Some(rest_spec) = app.spec.components.rest.as_ref() {
        postgrest::deploy(client.clone(), &namespace, Some(rest_spec)).await?;
    } else {
        postgrest::delete(client.clone(), &namespace).await?;
    }

    if let Some(realtime_spec) = app.spec.components.realtime.as_ref() {
        realtime::deploy(client.clone(), &namespace, &name, Some(realtime_spec)).await?;
    } else {
        realtime::delete(client.clone(), &namespace).await?;
    }

    if let Some(document_engine_spec) = app.spec.components.document_engine.as_ref() {
        document_engine::deploy(client.clone(), &namespace, Some(document_engine_spec)).await?;
    } else {
        document_engine::delete(client.clone(), &namespace).await?;
    }

    if let Some(selenium_spec) = app.spec.components.selenium.as_ref() {
        selenium::deploy(client.clone(), &namespace, Some(selenium_spec)).await?;
    } else {
        selenium::delete(client.clone(), &namespace).await?;
    }

    if let Some(mailhog_spec) = app.spec.components.mailhog.as_ref() {
        mailhog::deploy(client.clone(), &namespace, Some(mailhog_spec)).await?;
    } else {
        mailhog::delete(client.clone(), &namespace).await?;
    }

    let auth_hostname = app
        .spec
        .components
        .auth
        .as_ref()
        .and_then(|auth| auth.hostname_url.clone());

    let include_storage = app.spec.components.storage.is_some();
    let include_rest = app.spec.components.rest.is_some();
    let include_realtime = app.spec.components.realtime.is_some();
    let include_document_engine = app.spec.components.document_engine.is_some();

    let web_port = app.spec.services.web.port.ok_or_else(|| {
        Error::Other("spec.services.web.port is required for the web service".to_string())
    })?;

    if let Some(hostname_url) = auth_hostname {
        let realm_config =
            oauth2_proxy::ensure_secret(client.clone(), &namespace, &hostname_url).await?;
        keycloak::ensure_realm(client.clone(), &realm_config).await?;
        oauth2_proxy::deploy(
            client.clone(),
            &namespace,
            &hostname_url,
            web_port,
            &name,
        )
        .await?;
        let allow_admin = app
            .spec
            .components
            .auth
            .as_ref()
            .and_then(|auth| auth.expose_admin)
            .unwrap_or(false);
        nginx::deploy_nginx(
            &client,
            &namespace,
            nginx::NginxMode::Oidc { allow_admin },
            web_port,
            &name,
            include_storage,
            include_rest,
            include_realtime,
            include_document_engine,
        )
        .await?;
    } else {
        cleanup_auth_resources(client.clone(), &namespace).await?;
        jwt_secrets::ensure_secret(client.clone(), &namespace).await?;
        let jwt_value = jwt_secrets::get_token(
            client.clone(),
            &namespace,
            jwt_secrets::JWT_ANON_TOKEN_KEY,
        )
        .await?
        .unwrap_or_else(|| "1".to_string());
        nginx::deploy_nginx(
            &client,
            &namespace,
            nginx::NginxMode::StaticJwt {
                token: jwt_value.clone(),
            },
            web_port,
            &name,
            include_storage,
            include_rest,
            include_realtime,
            include_document_engine,
        )
        .await?;
    }

    if let Some(cloudflare_config) = app.spec.components.cloudflare.as_ref() {
        cloudflare::deploy(
            &client,
            &namespace,
            cloudflare_config.secret_name.as_deref(),
        )
        .await?;
    } else {
        delete_cloudflare_resources(&client, &namespace).await?;
    }

    deploy_web_app(&client, &namespace, &app.spec, &name, web_port).await?;
    deploy_extra_services(&client, &namespace, &app.spec.services.extra, &name).await?;
    let db_cluster_name = database::cluster_resource_name(&name);
    ensure_optional_nodeports(&client, &namespace, &app.spec, &db_cluster_name).await?;

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
    app_name: &str,
    web_port: u16,
) -> Result<(), Error> {
    let hostname_env = spec
        .components
        .auth
        .as_ref()
        .and_then(|a| a.hostname_url.clone())
        .unwrap_or_default();

    let mut env = vec![json!({"name": "HOSTNAME_URL", "value": hostname_env})];

    append_db_envs(
        &mut env,
        &spec.services.web.database_url,
        &spec.services.web.migrations_database_url,
        &spec.services.web.readonly_database_url,
    );

    env.push(json!({
        "name": "WEB_IMAGE",
        "value": spec.services.web.image.clone()
    }));

    append_env_from_spec(
        &mut env,
        &spec.services.web.env,
        &spec.services.web.secret_env,
    );

    let init_containers = spec.services.web.init.as_ref().map(|init| {
        let mut init_env = Vec::new();
        append_db_envs(
            &mut init_env,
            &init.database_url,
            &init.migrations_database_url,
            &init.readonly_database_url,
        );
        append_env_from_spec(&mut init_env, &init.env, &init.secret_env);

        deployment::InitContainer {
            image_name: init.image.clone(),
            env: init_env,
            command: None,
        }
    });

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: app_name.to_string(),
            image_name: spec.services.web.image.clone(),
            replicas: WEB_APP_REPLICAS,
            port: Some(web_port),
            env,
            init_containers: init_containers.into_iter().collect(),
            command: None,
            volume_mounts: vec![],
            volumes: vec![],
        },
        namespace,
        spec.components.ingress.is_some(),
        true,
    )
    .await
}

async fn deploy_extra_services(
    client: &Client,
    namespace: &str,
    services: &std::collections::BTreeMap<String, ServiceSpec>,
    app_name: &str,
) -> Result<(), Error> {
    let reserved = [
        app_name,
        nginx::NGINX_NAME,
        postgrest::REST_NAME,
        realtime::REALTIME_NAME,
        storage::STORAGE_NAME,
        document_engine::DOCUMENT_ENGINE_NAME,
        selenium::SELENIUM_NAME,
        mailhog::MAILHOG_NAME,
        "oauth2-proxy",
        "cloudflared",
        "minio",
    ];
    let mut seen = std::collections::HashSet::new();

    for (name, service) in services {
        if name.trim().is_empty() {
            return Err(Error::Other("extra service name cannot be empty".to_string()));
        }
        if reserved.contains(&name.as_str()) {
            return Err(Error::Other(format!(
                "extra service name '{}' is reserved",
                name
            )));
        }
        if !seen.insert(name.as_str()) {
            return Err(Error::Other(format!(
                "duplicate extra service name '{}'",
                name
            )));
        }

        let mut env = Vec::new();

        append_db_envs(
            &mut env,
            &service.database_url,
            &service.migrations_database_url,
            &service.readonly_database_url,
        );

        append_env_from_spec(&mut env, &service.env, &service.secret_env);

        let init_containers = service.init.as_ref().map(|init| {
            let mut init_env = Vec::new();
            append_db_envs(
                &mut init_env,
                &init.database_url,
                &init.migrations_database_url,
                &init.readonly_database_url,
            );
            append_env_from_spec(&mut init_env, &init.env, &init.secret_env);

            deployment::InitContainer {
                image_name: init.image.clone(),
                env: init_env,
                command: None,
            }
        });

        deployment::deployment(
            client.clone(),
            deployment::ServiceDeployment {
                name: name.clone(),
                image_name: service.image.clone(),
                replicas: WEB_APP_REPLICAS,
                port: service.port,
                env,
                init_containers: init_containers.into_iter().collect(),
                command: None,
                volume_mounts: vec![],
                volumes: vec![],
            },
            namespace,
            false,
            false,
        )
        .await?;
    }

    Ok(())
}

async fn ensure_optional_nodeports(
    client: &Client,
    namespace: &str,
    spec: &StackAppSpec,
    db_cluster_name: &str,
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
                "cnpg.io/cluster": db_cluster_name,
                "role": "primary"
            }),
            5432,
            node_port,
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, DB_NODEPORT_SERVICE_NAME).await?;
    }

    let auth_node_port = spec
        .components
        .auth
        .as_ref()
        .and_then(|auth_config| auth_config.expose_auth_port);
    if let Some(node_port) = auth_node_port {
        ensure_nodeport_service(
            client,
            namespace,
            APP_NODEPORT_SERVICE_NAME,
            json!({ "app": nginx::NGINX_NAME }),
            nginx::NGINX_PORT,
            node_port,
        )
        .await?;
    } else if let Some(node_port) = spec
        .components
        .ingress
        .as_ref()
        .and_then(|ingress_config| ingress_config.port)
    {
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
        .rest
        .as_ref()
        .and_then(|rest_config| rest_config.expose_rest_port)
    {
        ensure_nodeport_service(
            client,
            namespace,
            REST_NODEPORT_SERVICE_NAME,
            json!({ "app": postgrest::REST_NAME }),
            postgrest::DEFAULT_REST_PORT,
            node_port,
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, REST_NODEPORT_SERVICE_NAME).await?;
    }

    let selenium_config = spec.components.selenium.as_ref();
    let webdriver_nodeport = selenium_config.and_then(|cfg| cfg.expose_webdriver_port);
    let vnc_nodeport = selenium_config.and_then(|cfg| cfg.expose_vnc_port);
    if webdriver_nodeport.is_some() || vnc_nodeport.is_some() {
        let webdriver_port = selenium_config
            .and_then(|cfg| cfg.port)
            .unwrap_or(selenium::DEFAULT_SELENIUM_PORT);
        let vnc_port = selenium_config
            .and_then(|cfg| cfg.vnc_port)
            .unwrap_or(selenium::DEFAULT_SELENIUM_VNC_PORT);

        ensure_nodeport_service_multi(
            client,
            namespace,
            SELENIUM_NODEPORT_SERVICE_NAME,
            json!({ "app": selenium::SELENIUM_NAME }),
            &[
                NodePortSpec {
                    name: "webdriver",
                    port: webdriver_port,
                    node_port: webdriver_nodeport,
                },
                NodePortSpec {
                    name: "vnc",
                    port: vnc_port,
                    node_port: vnc_nodeport,
                },
            ],
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, SELENIUM_NODEPORT_SERVICE_NAME).await?;
    }

    let mailhog_config = spec.components.mailhog.as_ref();
    let smtp_nodeport = mailhog_config.and_then(|cfg| cfg.expose_smtp_port);
    let web_nodeport = mailhog_config.and_then(|cfg| cfg.expose_web_port);
    if smtp_nodeport.is_some() || web_nodeport.is_some() {
        let smtp_port = mailhog_config
            .and_then(|cfg| cfg.smtp_port)
            .unwrap_or(mailhog::DEFAULT_SMTP_PORT);
        let web_port = mailhog_config
            .and_then(|cfg| cfg.web_port)
            .unwrap_or(mailhog::DEFAULT_WEB_PORT);

        ensure_nodeport_service_multi(
            client,
            namespace,
            MAILHOG_NODEPORT_SERVICE_NAME,
            json!({ "app": mailhog::MAILHOG_NAME }),
            &[
                NodePortSpec {
                    name: "smtp",
                    port: smtp_port,
                    node_port: smtp_nodeport,
                },
                NodePortSpec {
                    name: "web",
                    port: web_port,
                    node_port: web_nodeport,
                },
            ],
        )
        .await?;
    } else {
        delete_service_if_exists(client, namespace, MAILHOG_NODEPORT_SERVICE_NAME).await?;
    }

    Ok(())
}

async fn delete_application_resources(
    client: &Client,
    namespace: &str,
    app_name: &str,
) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(app_name).await.is_ok() {
        deployments.delete(app_name, &DeleteParams::default()).await?;
    }

    delete_service_if_exists(client, namespace, app_name).await?;
    delete_service_if_exists(client, namespace, APP_NODEPORT_SERVICE_NAME).await?;
    delete_service_if_exists(client, namespace, DB_NODEPORT_SERVICE_NAME).await?;
    delete_service_if_exists(client, namespace, REST_NODEPORT_SERVICE_NAME).await?;
    delete_service_if_exists(client, namespace, SELENIUM_NODEPORT_SERVICE_NAME).await?;
    delete_service_if_exists(client, namespace, MAILHOG_NODEPORT_SERVICE_NAME).await?;

    Ok(())
}

async fn delete_cloudflare_resources(client: &Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(CLOUDFLARE_DEPLOYMENT_NAME).await.is_ok() {
        deployments
            .delete(CLOUDFLARE_DEPLOYMENT_NAME, &DeleteParams::default())
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

struct NodePortSpec<'a> {
    name: &'a str,
    port: u16,
    node_port: Option<u16>,
}

async fn ensure_nodeport_service_multi(
    client: &Client,
    namespace: &str,
    name: &str,
    selector: Value,
    ports: &[NodePortSpec<'_>],
) -> Result<(), Error> {
    let ports_value: Vec<Value> = ports
        .iter()
        .map(|port| {
            let mut entry = json!({
                "name": port.name,
                "port": port.port,
                "targetPort": port.port
            });
            if let Some(node_port) = port.node_port {
                entry["nodePort"] = json!(node_port);
            }
            entry
        })
        .collect();

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
            "ports": ports_value
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

fn append_db_envs(
    env: &mut Vec<Value>,
    database_url: &Option<String>,
    migrations_database_url: &Option<String>,
    readonly_database_url: &Option<String>,
) {
    if let Some(db_env_name) = database_url.clone() {
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

    if let Some(superuser_env_name) = migrations_database_url.clone() {
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

    if let Some(readonly_env_name) = readonly_database_url.clone() {
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
}
