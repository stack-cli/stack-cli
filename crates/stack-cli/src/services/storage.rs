use crate::error::Error;
use crate::operator::crd::StorageConfig;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::{Secret, Service};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use serde_json::json;

pub const STORAGE_NAME: &str = "storage";
pub const DEFAULT_STORAGE_IMAGE: &str = "supabase/storage-api:v1.33.0";
pub const DEFAULT_STORAGE_PORT: u16 = 5000;
const STORAGE_AUTH_SECRET_NAME: &str = "storage-auth";
const STORAGE_AUTH_SECRET_KEY: &str = "auth-jwt-secret";
const STORAGE_AUTH_SECRET_VALUE: &str = "f023d3db-39dc-4ac9-87b2-b2be72e9162b";
const STORAGE_S3_SECRET_NAME: &str = "storage-s3";
const STORAGE_S3_BUCKET_KEY: &str = "STORAGE_S3_BUCKET";
const STORAGE_S3_ENDPOINT_KEY: &str = "STORAGE_S3_ENDPOINT";
const STORAGE_S3_REGION_KEY: &str = "STORAGE_S3_REGION";
const STORAGE_S3_FORCE_PATH_STYLE_KEY: &str = "STORAGE_S3_FORCE_PATH_STYLE";
const AWS_ACCESS_KEY_ID_KEY: &str = "AWS_ACCESS_KEY_ID";
const AWS_SECRET_ACCESS_KEY_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const S3_PROTOCOL_ACCESS_KEY_ID_KEY: &str = "S3_PROTOCOL_ACCESS_KEY_ID";
const S3_PROTOCOL_ACCESS_KEY_SECRET_KEY: &str = "S3_PROTOCOL_ACCESS_KEY_SECRET";
const DEFAULT_AWS_ACCESS_KEY_ID: &str = "supa-storage";
const DEFAULT_AWS_SECRET_ACCESS_KEY: &str = "secret1234";
const DEFAULT_S3_PROTOCOL_ACCESS_KEY_ID: &str = "625729a08b95bf1b7ff351a663f3a23c";
const DEFAULT_S3_PROTOCOL_ACCESS_KEY_SECRET: &str =
    "850181e4652dd023b7a98c58ae0d2d34bd487ee0cc3254aed6eda37307425907";
const DEFAULT_S3_BUCKET: &str = "supa-storage-bucket";
const DEFAULT_S3_ENDPOINT: &str = "http://minio:9000";
const DEFAULT_S3_REGION: &str = "us-east-1";
const DEFAULT_S3_FORCE_PATH_STYLE: &str = "true";
const MINIO_NAME: &str = "minio";
const MINIO_IMAGE: &str = "minio/minio:latest";
const MINIO_PORT: u16 = 9000;

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&StorageConfig>,
) -> Result<(), Error> {
    let secret_name = config
        .and_then(|c| c.s3_secret_name.as_ref())
        .map(String::from)
        .unwrap_or_else(|| STORAGE_S3_SECRET_NAME.to_string());
    let install_minio = config.map_or(true, |c| {
        c.install_minio.unwrap_or(c.s3_secret_name.is_none())
    });

    ensure_secret(client.clone(), namespace).await?;
    if config.is_none() || config.and_then(|c| c.s3_secret_name.as_ref()).is_none() {
        ensure_s3_secret(client.clone(), namespace, &secret_name).await?;
    }

    if install_minio {
        deploy_minio(client.clone(), namespace).await?;
    }

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
                    "name": secret_name,
                    "key": STORAGE_S3_BUCKET_KEY
                }
            }
        }),
        json!({
            "name": "STORAGE_S3_ENDPOINT",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": STORAGE_S3_ENDPOINT_KEY
                }
            }
        }),
        json!({
            "name": "STORAGE_S3_FORCE_PATH_STYLE",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": STORAGE_S3_FORCE_PATH_STYLE_KEY
                }
            }
        }),
        json!({
            "name": "STORAGE_S3_REGION",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": STORAGE_S3_REGION_KEY
                }
            }
        }),
        json!({"name": "FILE_SIZE_LIMIT", "value": "52428800"}),
        json!({
            "name": "AWS_ACCESS_KEY_ID",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": AWS_ACCESS_KEY_ID_KEY
                }
            }
        }),
        json!({
            "name": "AWS_SECRET_ACCESS_KEY",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": AWS_SECRET_ACCESS_KEY_KEY
                }
            }
        }),
        json!({
            "name": "S3_PROTOCOL_ACCESS_KEY_ID",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": S3_PROTOCOL_ACCESS_KEY_ID_KEY
                }
            }
        }),
        json!({
            "name": "S3_PROTOCOL_ACCESS_KEY_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": S3_PROTOCOL_ACCESS_KEY_SECRET_KEY
                }
            }
        }),
        json!({"name": "STORAGE_FILE_LOCAL_STORAGE_PATH", "value": "/var/lib/storage"}),
        json!({
            "name": "AUTH_JWT_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": STORAGE_AUTH_SECRET_NAME,
                    "key": STORAGE_AUTH_SECRET_KEY
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
            init_container: None,
            command: None,
            volume_mounts,
            volumes,
        },
        namespace,
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
    if secrets.get(STORAGE_AUTH_SECRET_NAME).await.is_ok() {
        secrets
            .delete(STORAGE_AUTH_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

async fn ensure_secret(client: Client, namespace: &str) -> Result<(), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": STORAGE_AUTH_SECRET_NAME,
            "namespace": namespace
        },
        "stringData": {
            STORAGE_AUTH_SECRET_KEY: STORAGE_AUTH_SECRET_VALUE
        }
    });

    secret_api
        .patch(
            STORAGE_AUTH_SECRET_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;

    Ok(())
}

async fn ensure_s3_secret(client: Client, namespace: &str, secret_name: &str) -> Result<(), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": secret_name,
            "namespace": namespace
        },
        "stringData": {
            STORAGE_S3_BUCKET_KEY: DEFAULT_S3_BUCKET,
            STORAGE_S3_ENDPOINT_KEY: DEFAULT_S3_ENDPOINT,
            STORAGE_S3_REGION_KEY: DEFAULT_S3_REGION,
            STORAGE_S3_FORCE_PATH_STYLE_KEY: DEFAULT_S3_FORCE_PATH_STYLE,
            AWS_ACCESS_KEY_ID_KEY: DEFAULT_AWS_ACCESS_KEY_ID,
            AWS_SECRET_ACCESS_KEY_KEY: DEFAULT_AWS_SECRET_ACCESS_KEY,
            S3_PROTOCOL_ACCESS_KEY_ID_KEY: DEFAULT_S3_PROTOCOL_ACCESS_KEY_ID,
            S3_PROTOCOL_ACCESS_KEY_SECRET_KEY: DEFAULT_S3_PROTOCOL_ACCESS_KEY_SECRET
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

async fn deploy_minio(client: Client, namespace: &str) -> Result<(), Error> {
    let env = vec![
        json!({"name": "MINIO_ROOT_USER", "value": DEFAULT_AWS_ACCESS_KEY_ID}),
        json!({"name": "MINIO_ROOT_PASSWORD", "value": DEFAULT_AWS_SECRET_ACCESS_KEY}),
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
    )
    .await
}
