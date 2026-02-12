use crate::error::Error;
use crate::operator::crd::RedisConfig;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::{PersistentVolumeClaim, Secret, Service};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use rand::{distr::Alphanumeric, Rng};
use serde_json::json;

pub const REDIS_NAME: &str = "redis";
pub const DEFAULT_REDIS_IMAGE: &str = "redis:7-alpine";
pub const DEFAULT_REDIS_PORT: u16 = 6379;
const REDIS_PASSWORD_SECRET_NAME: &str = "redis-auth";
const REDIS_PASSWORD_KEY: &str = "password";
const REDIS_URLS_SECRET_NAME: &str = "redis-urls";
const REDIS_URL_KEY: &str = "redis-url";
const REDIS_PVC_NAME: &str = "redis-data";
const DEFAULT_REDIS_SIZE: &str = "1Gi";

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&RedisConfig>,
) -> Result<(), Error> {
    let image_name = config
        .and_then(|cfg| cfg.image.clone())
        .unwrap_or_else(|| DEFAULT_REDIS_IMAGE.to_string());
    let port = config
        .and_then(|cfg| cfg.port)
        .unwrap_or(DEFAULT_REDIS_PORT);
    let persistence_enabled = config.and_then(|cfg| cfg.persistence).unwrap_or(true);
    let size = config
        .and_then(|cfg| cfg.size.clone())
        .unwrap_or_else(|| DEFAULT_REDIS_SIZE.to_string());
    let custom_password_secret = config.and_then(|cfg| cfg.password_secret_name.clone());
    let password_secret_name = custom_password_secret
        .clone()
        .unwrap_or_else(|| REDIS_PASSWORD_SECRET_NAME.to_string());

    let password = if custom_password_secret.is_some() {
        read_password_from_secret(client.clone(), namespace, &password_secret_name).await?
    } else {
        ensure_password_secret(client.clone(), namespace, &password_secret_name).await?
    };

    ensure_redis_url_secret(client.clone(), namespace, port, &password).await?;

    let (volume_mounts, volumes) = if persistence_enabled {
        ensure_redis_pvc(client.clone(), namespace, &size).await?;
        (
            vec![json!({
                "name": REDIS_PVC_NAME,
                "mountPath": "/data"
            })],
            vec![json!({
                "name": REDIS_PVC_NAME,
                "persistentVolumeClaim": {
                    "claimName": REDIS_PVC_NAME
                }
            })],
        )
    } else {
        (
            vec![json!({
                "name": REDIS_PVC_NAME,
                "mountPath": "/data"
            })],
            vec![json!({
                "name": REDIS_PVC_NAME,
                "emptyDir": {}
            })],
        )
    };

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: REDIS_NAME.to_string(),
            image_name,
            replicas: 1,
            port: Some(port),
            env: vec![json!({
                "name": "REDIS_PASSWORD",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": password_secret_name,
                        "key": REDIS_PASSWORD_KEY
                    }
                }
            })],
            init_containers: vec![],
            command: Some(deployment::Command {
                command: vec!["/bin/sh".to_string(), "-c".to_string()],
                args: vec![format!(
                    "redis-server --port {port} --appendonly {} --requirepass \"$REDIS_PASSWORD\"",
                    if persistence_enabled { "yes" } else { "no" }
                )],
            }),
            volume_mounts,
            volumes,
        },
        namespace,
        false,
        false,
    )
    .await
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(REDIS_NAME).await.is_ok() {
        deployments
            .delete(REDIS_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    if services.get(REDIS_NAME).await.is_ok() {
        services
            .delete(REDIS_NAME, &DeleteParams::default())
            .await?;
    }

    let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
    if secrets.get(REDIS_URLS_SECRET_NAME).await.is_ok() {
        secrets
            .delete(REDIS_URLS_SECRET_NAME, &DeleteParams::default())
            .await?;
    }
    if secrets.get(REDIS_PASSWORD_SECRET_NAME).await.is_ok() {
        secrets
            .delete(REDIS_PASSWORD_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client, namespace);
    if pvcs.get(REDIS_PVC_NAME).await.is_ok() {
        pvcs.delete(REDIS_PVC_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

async fn ensure_redis_pvc(client: Client, namespace: &str, size: &str) -> Result<(), Error> {
    let pvc_manifest = json!({
        "apiVersion": "v1",
        "kind": "PersistentVolumeClaim",
        "metadata": {
            "name": REDIS_PVC_NAME,
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
            REDIS_PVC_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(pvc_manifest),
        )
        .await?;
    Ok(())
}

async fn ensure_password_secret(
    client: Client,
    namespace: &str,
    secret_name: &str,
) -> Result<String, Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let existing_secret = secret_api.get(secret_name).await.ok();
    let password = existing_secret
        .as_ref()
        .and_then(|secret| read_secret_field(secret, REDIS_PASSWORD_KEY))
        .unwrap_or_else(random_token);

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": secret_name,
            "namespace": namespace
        },
        "stringData": {
            REDIS_PASSWORD_KEY: password
        }
    });

    secret_api
        .patch(
            secret_name,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;

    Ok(password)
}

async fn read_password_from_secret(
    client: Client,
    namespace: &str,
    secret_name: &str,
) -> Result<String, Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let secret = secret_api.get(secret_name).await.map_err(|_| {
        Error::Other(format!(
            "Redis password secret '{}' not found in namespace '{}'",
            secret_name, namespace
        ))
    })?;

    read_secret_field(&secret, REDIS_PASSWORD_KEY).ok_or_else(|| {
        Error::Other(format!(
            "Redis password secret '{}' is missing key '{}'",
            secret_name, REDIS_PASSWORD_KEY
        ))
    })
}

async fn ensure_redis_url_secret(
    client: Client,
    namespace: &str,
    port: u16,
    password: &str,
) -> Result<(), Error> {
    let redis_url = format!("redis://:{}@{}:{}/0", password, REDIS_NAME, port);
    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": REDIS_URLS_SECRET_NAME,
            "namespace": namespace
        },
        "stringData": {
            REDIS_URL_KEY: redis_url
        }
    });

    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    secret_api
        .patch(
            REDIS_URLS_SECRET_NAME,
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
