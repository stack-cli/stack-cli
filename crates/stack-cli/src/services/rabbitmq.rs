use crate::error::Error;
use crate::operator::crd::RabbitMqConfig;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::{PersistentVolumeClaim, Secret, Service};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use rand::{distr::Alphanumeric, Rng};
use serde_json::json;

pub const RABBITMQ_NAME: &str = "rabbitmq";
pub const RABBITMQ_MANAGEMENT_SERVICE_NAME: &str = "rabbitmq-management";
pub const DEFAULT_RABBITMQ_IMAGE: &str = "rabbitmq:3-management-alpine";
pub const DEFAULT_RABBITMQ_PORT: u16 = 5672;
pub const DEFAULT_RABBITMQ_MANAGEMENT_PORT: u16 = 15672;
const RABBITMQ_AUTH_SECRET_NAME: &str = "rabbitmq-auth";
const RABBITMQ_USERNAME_KEY: &str = "username";
const RABBITMQ_PASSWORD_KEY: &str = "password";
const RABBITMQ_URLS_SECRET_NAME: &str = "rabbitmq-urls";
const RABBITMQ_AMQP_URL_KEY: &str = "amqp-url";
const RABBITMQ_MANAGEMENT_URL_KEY: &str = "management-url";
const RABBITMQ_PVC_NAME: &str = "rabbitmq-data";
const DEFAULT_RABBITMQ_SIZE: &str = "5Gi";

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&RabbitMqConfig>,
) -> Result<(), Error> {
    let image_name = config
        .and_then(|cfg| cfg.image.clone())
        .unwrap_or_else(|| DEFAULT_RABBITMQ_IMAGE.to_string());
    let amqp_port = config
        .and_then(|cfg| cfg.port)
        .unwrap_or(DEFAULT_RABBITMQ_PORT);
    let management_port = config
        .and_then(|cfg| cfg.management_port)
        .unwrap_or(DEFAULT_RABBITMQ_MANAGEMENT_PORT);
    let persistence_enabled = config.and_then(|cfg| cfg.persistence).unwrap_or(true);
    let size = config
        .and_then(|cfg| cfg.size.clone())
        .unwrap_or_else(|| DEFAULT_RABBITMQ_SIZE.to_string());

    let custom_credentials_secret = config.and_then(|cfg| cfg.credentials_secret_name.clone());
    let credentials_secret_name = custom_credentials_secret
        .clone()
        .unwrap_or_else(|| RABBITMQ_AUTH_SECRET_NAME.to_string());

    let (username, password) = if custom_credentials_secret.is_some() {
        read_credentials_from_secret(client.clone(), namespace, &credentials_secret_name).await?
    } else {
        ensure_credentials_secret(client.clone(), namespace, &credentials_secret_name).await?
    };

    ensure_rabbitmq_url_secret(
        client.clone(),
        namespace,
        amqp_port,
        management_port,
        &username,
        &password,
    )
    .await?;

    let (volume_mounts, volumes) = if persistence_enabled {
        ensure_rabbitmq_pvc(client.clone(), namespace, &size).await?;
        (
            vec![json!({
                "name": RABBITMQ_PVC_NAME,
                "mountPath": "/var/lib/rabbitmq"
            })],
            vec![json!({
                "name": RABBITMQ_PVC_NAME,
                "persistentVolumeClaim": {
                    "claimName": RABBITMQ_PVC_NAME
                }
            })],
        )
    } else {
        (
            vec![json!({
                "name": RABBITMQ_PVC_NAME,
                "mountPath": "/var/lib/rabbitmq"
            })],
            vec![json!({
                "name": RABBITMQ_PVC_NAME,
                "emptyDir": {}
            })],
        )
    };

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: RABBITMQ_NAME.to_string(),
            image_name,
            replicas: 1,
            port: Some(amqp_port),
            env: vec![
                json!({
                    "name": "RABBITMQ_DEFAULT_USER",
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": credentials_secret_name,
                            "key": RABBITMQ_USERNAME_KEY
                        }
                    }
                }),
                json!({
                    "name": "RABBITMQ_DEFAULT_PASS",
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": credentials_secret_name,
                            "key": RABBITMQ_PASSWORD_KEY
                        }
                    }
                }),
            ],
            init_containers: vec![],
            command: None,
            volume_mounts,
            volumes,
        },
        namespace,
        false,
        false,
    )
    .await?;

    ensure_management_service(client, namespace, management_port).await?;

    Ok(())
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(RABBITMQ_NAME).await.is_ok() {
        deployments
            .delete(RABBITMQ_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    if services.get(RABBITMQ_NAME).await.is_ok() {
        services
            .delete(RABBITMQ_NAME, &DeleteParams::default())
            .await?;
    }
    if services.get(RABBITMQ_MANAGEMENT_SERVICE_NAME).await.is_ok() {
        services
            .delete(RABBITMQ_MANAGEMENT_SERVICE_NAME, &DeleteParams::default())
            .await?;
    }

    let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
    if secrets.get(RABBITMQ_URLS_SECRET_NAME).await.is_ok() {
        secrets
            .delete(RABBITMQ_URLS_SECRET_NAME, &DeleteParams::default())
            .await?;
    }
    if secrets.get(RABBITMQ_AUTH_SECRET_NAME).await.is_ok() {
        secrets
            .delete(RABBITMQ_AUTH_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client, namespace);
    if pvcs.get(RABBITMQ_PVC_NAME).await.is_ok() {
        pvcs.delete(RABBITMQ_PVC_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

async fn ensure_management_service(
    client: Client,
    namespace: &str,
    management_port: u16,
) -> Result<(), Error> {
    let service_manifest = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": RABBITMQ_MANAGEMENT_SERVICE_NAME,
            "namespace": namespace
        },
        "spec": {
            "type": "ClusterIP",
            "selector": {
                "app": RABBITMQ_NAME
            },
            "ports": [
                {
                    "protocol": "TCP",
                    "port": management_port,
                    "targetPort": management_port
                }
            ]
        }
    });

    let service_api: Api<Service> = Api::namespaced(client, namespace);
    service_api
        .patch(
            RABBITMQ_MANAGEMENT_SERVICE_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(service_manifest),
        )
        .await?;
    Ok(())
}

async fn ensure_rabbitmq_pvc(client: Client, namespace: &str, size: &str) -> Result<(), Error> {
    let pvc_manifest = json!({
        "apiVersion": "v1",
        "kind": "PersistentVolumeClaim",
        "metadata": {
            "name": RABBITMQ_PVC_NAME,
            "namespace": namespace
        },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": {
                "requests": {
                    "storage": size
                }
            }
        }
    });

    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client, namespace);
    pvc_api
        .patch(
            RABBITMQ_PVC_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(pvc_manifest),
        )
        .await?;
    Ok(())
}

async fn ensure_credentials_secret(
    client: Client,
    namespace: &str,
    secret_name: &str,
) -> Result<(String, String), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let existing_secret = secret_api.get(secret_name).await.ok();
    let username = existing_secret
        .as_ref()
        .and_then(|secret| read_secret_field(secret, RABBITMQ_USERNAME_KEY))
        .unwrap_or_else(|| "stack".to_string());
    let password = existing_secret
        .as_ref()
        .and_then(|secret| read_secret_field(secret, RABBITMQ_PASSWORD_KEY))
        .unwrap_or_else(random_token);

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": secret_name,
            "namespace": namespace
        },
        "stringData": {
            RABBITMQ_USERNAME_KEY: username,
            RABBITMQ_PASSWORD_KEY: password
        }
    });

    secret_api
        .patch(
            secret_name,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;

    Ok((username, password))
}

async fn read_credentials_from_secret(
    client: Client,
    namespace: &str,
    secret_name: &str,
) -> Result<(String, String), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let secret = secret_api.get(secret_name).await.map_err(|_| {
        Error::Other(format!(
            "RabbitMQ credentials secret '{}' not found in namespace '{}'",
            secret_name, namespace
        ))
    })?;

    let username = read_secret_field(&secret, RABBITMQ_USERNAME_KEY).ok_or_else(|| {
        Error::Other(format!(
            "RabbitMQ credentials secret '{}' is missing key '{}'",
            secret_name, RABBITMQ_USERNAME_KEY
        ))
    })?;
    let password = read_secret_field(&secret, RABBITMQ_PASSWORD_KEY).ok_or_else(|| {
        Error::Other(format!(
            "RabbitMQ credentials secret '{}' is missing key '{}'",
            secret_name, RABBITMQ_PASSWORD_KEY
        ))
    })?;

    Ok((username, password))
}

async fn ensure_rabbitmq_url_secret(
    client: Client,
    namespace: &str,
    amqp_port: u16,
    management_port: u16,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    let amqp_url = format!(
        "amqp://{}:{}@{}:{}/",
        username, password, RABBITMQ_NAME, amqp_port
    );
    let management_url = format!(
        "http://{}:{}@{}:{}/",
        username, password, RABBITMQ_MANAGEMENT_SERVICE_NAME, management_port
    );

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": RABBITMQ_URLS_SECRET_NAME,
            "namespace": namespace
        },
        "stringData": {
            RABBITMQ_AMQP_URL_KEY: amqp_url,
            RABBITMQ_MANAGEMENT_URL_KEY: management_url
        }
    });

    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    secret_api
        .patch(
            RABBITMQ_URLS_SECRET_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;
    Ok(())
}

fn random_token() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn read_secret_field(secret: &Secret, key: &str) -> Option<String> {
    if let Some(data) = &secret.data {
        if let Some(value) = data.get(key) {
            if let Ok(val) = String::from_utf8(value.0.clone()) {
                return Some(val);
            }
        }
    }

    secret
        .string_data
        .as_ref()
        .and_then(|map| map.get(key).cloned())
}
