use crate::error::Error;
use crate::operator::crd::StorageConfig;
use crate::services::deployment;
use crate::services::jwt_secrets;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::{Secret, Service};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use rand::{distributions::Alphanumeric, Rng};
use serde_json::json;

pub const STORAGE_NAME: &str = "storage";
pub const DEFAULT_STORAGE_IMAGE: &str = "supabase/storage-api:v1.33.0";
pub const DEFAULT_STORAGE_PORT: u16 = 5000;
const STORAGE_S3_SECRET_NAME: &str = "storage-s3";
const STORAGE_S3_BUCKET_KEY: &str = "STORAGE_S3_BUCKET";
const STORAGE_S3_ENDPOINT_KEY: &str = "STORAGE_S3_ENDPOINT";
const STORAGE_S3_REGION_KEY: &str = "STORAGE_S3_REGION";
const STORAGE_S3_FORCE_PATH_STYLE_KEY: &str = "STORAGE_S3_FORCE_PATH_STYLE";
const AWS_ACCESS_KEY_ID_KEY: &str = "AWS_ACCESS_KEY_ID";
const AWS_SECRET_ACCESS_KEY_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const S3_PROTOCOL_ACCESS_KEY_ID_KEY: &str = "S3_PROTOCOL_ACCESS_KEY_ID";
const S3_PROTOCOL_ACCESS_KEY_SECRET_KEY: &str = "S3_PROTOCOL_ACCESS_KEY_SECRET";
const DEFAULT_S3_BUCKET: &str = "supa-storage-bucket";
const DEFAULT_S3_ENDPOINT: &str = "http://minio:9000";
const DEFAULT_S3_REGION: &str = "us-east-1";
const DEFAULT_S3_FORCE_PATH_STYLE: &str = "true";
const MINIO_NAME: &str = "minio";
const MINIO_IMAGE: &str = "minio/minio:latest";
const MINIO_PORT: u16 = 9000;
const MINIO_MC_IMAGE: &str = "minio/mc:latest";

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&StorageConfig>,
) -> Result<(), Error> {
    let secret_name = config
        .and_then(|c| c.s3_secret_name.as_ref())
        .map(String::from)
        .unwrap_or_else(|| STORAGE_S3_SECRET_NAME.to_string());
    let secret_name_env = secret_name.clone();
    let install_minio = config.map_or(true, |c| {
        c.install_minio.unwrap_or(c.s3_secret_name.is_none())
    });

    let override_jwt = config.and_then(|c| c.danger_override_jwt_secret.clone());
    jwt_secrets::ensure_secret(client.clone(), namespace, override_jwt).await?;
    if config.is_none() || config.and_then(|c| c.s3_secret_name.as_ref()).is_none() {
        ensure_s3_secret(client.clone(), namespace, &secret_name).await?;
    }

    if install_minio {
        deploy_minio(client.clone(), namespace, &secret_name).await?;
    }

    let minio_init = if install_minio {
        Some(deployment::InitContainer {
            image_name: MINIO_MC_IMAGE.to_string(),
            env: vec![
                json!({
                    "name": STORAGE_S3_BUCKET_KEY,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_name_env,
                            "key": STORAGE_S3_BUCKET_KEY
                        }
                    }
                }),
                json!({
                    "name": STORAGE_S3_ENDPOINT_KEY,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_name_env,
                            "key": STORAGE_S3_ENDPOINT_KEY
                        }
                    }
                }),
                json!({
                    "name": AWS_ACCESS_KEY_ID_KEY,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_name_env,
                            "key": AWS_ACCESS_KEY_ID_KEY
                        }
                    }
                }),
                json!({
                    "name": AWS_SECRET_ACCESS_KEY_KEY,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_name_env,
                            "key": AWS_SECRET_ACCESS_KEY_KEY
                        }
                    }
                }),
            ],
            command: Some(deployment::Command {
                command: vec!["/bin/sh".to_string(), "-c".to_string()],
                args: vec![r#"set -e
mc alias set supa-minio "$STORAGE_S3_ENDPOINT" "$AWS_ACCESS_KEY_ID" "$AWS_SECRET_ACCESS_KEY"
mc mb --ignore-existing supa-minio/"$STORAGE_S3_BUCKET"
mc mb --ignore-existing supa-minio/warehouse--table-s3
mc anonymous set download supa-minio/warehouse--table-s3 || true
"#
                .to_string()],
            }),
        })
    } else {
        None
    };

    let env = vec![
        json!({
            "name": "PORT",
            "value": DEFAULT_STORAGE_PORT.to_string()
        }),
        json!({
            "name": "DATABASE_URL",
            "valueFrom": {
                "secretKeyRef": {
                    "name": "database-urls",
                    "key": "migrations-url"
                }
            }
        }),
        json!({"name": "STORAGE_BACKEND", "value": "s3"}),
        json!({
            "name": "STORAGE_S3_BUCKET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": STORAGE_S3_BUCKET_KEY
                }
            }
        }),
        json!({
            "name": "STORAGE_S3_ENDPOINT",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": STORAGE_S3_ENDPOINT_KEY
                }
            }
        }),
        json!({
            "name": "STORAGE_S3_FORCE_PATH_STYLE",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": STORAGE_S3_FORCE_PATH_STYLE_KEY
                }
            }
        }),
        json!({
            "name": "STORAGE_S3_REGION",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": STORAGE_S3_REGION_KEY
                }
            }
        }),
        json!({"name": "FILE_SIZE_LIMIT", "value": "52428800"}),
        json!({
            "name": "AWS_ACCESS_KEY_ID",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": AWS_ACCESS_KEY_ID_KEY
                }
            }
        }),
        json!({
            "name": "AWS_SECRET_ACCESS_KEY",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": AWS_SECRET_ACCESS_KEY_KEY
                }
            }
        }),
        json!({
            "name": "S3_PROTOCOL_ACCESS_KEY_ID",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": S3_PROTOCOL_ACCESS_KEY_ID_KEY
                }
            }
        }),
        json!({
            "name": "S3_PROTOCOL_ACCESS_KEY_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name_env,
                    "key": S3_PROTOCOL_ACCESS_KEY_SECRET_KEY
                }
            }
        }),
        json!({"name": "DB_INSTALL_ROLES", "value": "true"}),
        json!({"name": "STORAGE_FILE_LOCAL_STORAGE_PATH", "value": "/var/lib/storage"}),
        json!({
            "name": "AUTH_JWT_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": jwt_secrets::JWT_AUTH_SECRET_NAME,
                    "key": jwt_secrets::JWT_SECRET_KEY
                }
            }
        }),
        json!({"name": "AUTH_JWT_ALGORITHM", "value": "HS256"}),
    ];

    let volume_mounts = vec![json!({
        "name": "storage-data",
        "mountPath": "/var/lib/storage"
    })];

    let volumes = vec![json!({
        "name": "storage-data",
        "emptyDir": {}
    })];

    deployment::deployment(
        client,
        deployment::ServiceDeployment {
            name: STORAGE_NAME.to_string(),
            image_name: DEFAULT_STORAGE_IMAGE.to_string(),
            replicas: 1,
            port: DEFAULT_STORAGE_PORT,
            env,
            init_container: minio_init,
            command: None,
            volume_mounts,
            volumes,
        },
        namespace,
        true,
    )
    .await
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(STORAGE_NAME).await.is_ok() {
        deployments
            .delete(STORAGE_NAME, &DeleteParams::default())
            .await?;
    }
    if deployments.get(MINIO_NAME).await.is_ok() {
        deployments
            .delete(MINIO_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    if services.get(STORAGE_NAME).await.is_ok() {
        services
            .delete(STORAGE_NAME, &DeleteParams::default())
            .await?;
    }
    if services.get(MINIO_NAME).await.is_ok() {
        services
            .delete(MINIO_NAME, &DeleteParams::default())
            .await?;
    }

    let secrets: Api<Secret> = Api::namespaced(client, namespace);
    if secrets
        .get(jwt_secrets::JWT_AUTH_SECRET_NAME)
        .await
        .is_ok()
    {
        secrets
            .delete(jwt_secrets::JWT_AUTH_SECRET_NAME, &DeleteParams::default())
            .await?;
    }
    if secrets.get(STORAGE_S3_SECRET_NAME).await.is_ok() {
        secrets
            .delete(STORAGE_S3_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

async fn ensure_s3_secret(client: Client, namespace: &str, secret_name: &str) -> Result<(), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let existing = secret_api.get(secret_name).await.ok();

    let bucket = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, STORAGE_S3_BUCKET_KEY))
        .unwrap_or_else(|| DEFAULT_S3_BUCKET.to_string());
    let endpoint = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, STORAGE_S3_ENDPOINT_KEY))
        .unwrap_or_else(|| DEFAULT_S3_ENDPOINT.to_string());
    let region = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, STORAGE_S3_REGION_KEY))
        .unwrap_or_else(|| DEFAULT_S3_REGION.to_string());
    let force_path_style = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, STORAGE_S3_FORCE_PATH_STYLE_KEY))
        .unwrap_or_else(|| DEFAULT_S3_FORCE_PATH_STYLE.to_string());
    let access_key_id = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, AWS_ACCESS_KEY_ID_KEY))
        .unwrap_or_else(random_token);
    let secret_access_key = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, AWS_SECRET_ACCESS_KEY_KEY))
        .unwrap_or_else(random_token);
    let protocol_access_key_id = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, S3_PROTOCOL_ACCESS_KEY_ID_KEY))
        .unwrap_or_else(random_token);
    let protocol_access_key_secret = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, S3_PROTOCOL_ACCESS_KEY_SECRET_KEY))
        .unwrap_or_else(random_token);

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": secret_name,
            "namespace": namespace
        },
        "stringData": {
            STORAGE_S3_BUCKET_KEY: bucket,
            STORAGE_S3_ENDPOINT_KEY: endpoint,
            STORAGE_S3_REGION_KEY: region,
            STORAGE_S3_FORCE_PATH_STYLE_KEY: force_path_style,
            AWS_ACCESS_KEY_ID_KEY: access_key_id,
            AWS_SECRET_ACCESS_KEY_KEY: secret_access_key,
            S3_PROTOCOL_ACCESS_KEY_ID_KEY: protocol_access_key_id,
            S3_PROTOCOL_ACCESS_KEY_SECRET_KEY: protocol_access_key_secret
        }
    });

    secret_api
        .patch(
            secret_name,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;

    Ok(())
}

async fn deploy_minio(client: Client, namespace: &str, secret_name: &str) -> Result<(), Error> {
    let env = vec![
        json!({
            "name": "MINIO_ROOT_USER",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": AWS_ACCESS_KEY_ID_KEY
                }
            }
        }),
        json!({
            "name": "MINIO_ROOT_PASSWORD",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": AWS_SECRET_ACCESS_KEY_KEY
                }
            }
        }),
        json!({"name": "MINIO_DOMAIN", "value": "minio"}),
    ];

    let command = deployment::Command {
        command: vec!["minio".to_string()],
        args: vec![
            "server".to_string(),
            "--console-address".to_string(),
            ":9001".to_string(),
            "/data".to_string(),
        ],
    };

    let volume_mounts = vec![json!({
        "name": "minio-data",
        "mountPath": "/data"
    })];

    let volumes = vec![json!({
        "name": "minio-data",
        "emptyDir": {}
    })];

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: MINIO_NAME.to_string(),
            image_name: MINIO_IMAGE.to_string(),
            replicas: 1,
            port: MINIO_PORT,
            env,
            init_container: None,
            command: Some(command),
            volume_mounts,
            volumes,
        },
        namespace,
        true,
    )
    .await
}

fn random_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
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
