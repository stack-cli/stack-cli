use crate::error::Error;
use crate::operator::crd::RealtimeConfig;
use crate::services::deployment;
use crate::services::jwt_secrets;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Secret;
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use rand::{distr::Alphanumeric, Rng};
use serde_json::json;

pub const REALTIME_NAME: &str = "realtime";
pub const REALTIME_IMAGE: &str = "supabase/realtime:v2.47.2";
pub const REALTIME_PORT: u16 = 4000;
const REALTIME_SECRET_NAME: &str = "realtime-secrets";
const REALTIME_SECRET_KEY_BASE_KEY: &str = "secret-key-base";
const REALTIME_DB_ENC_KEY: &str = "db-enc-key";
const DB_ENC_KEY_LEN: usize = 16;
const REALTIME_INIT_IMAGE: &str = "postgres:16-alpine";

pub async fn deploy(
    client: Client,
    namespace: &str,
    _config: Option<&RealtimeConfig>,
) -> Result<(), Error> {
    jwt_secrets::ensure_secret(client.clone(), namespace).await?;
    ensure_secret(client.clone(), namespace).await?;

    let env = vec![
        json!({"name": "PORT", "value": REALTIME_PORT.to_string()}),
        json!({"name": "DB_HOST", "value": "stack-db-cluster-rw"}),
        json!({"name": "DB_PORT", "value": "5432"}),
        json!({"name": "DB_NAME", "value": "stack-app"}),
        json!({
            "name": "DB_USER",
            "valueFrom": {
                "secretKeyRef": {
                    "name": "db-owner",
                    "key": "username"
                }
            }
        }),
        json!({
            "name": "DB_PASSWORD",
            "valueFrom": {
                "secretKeyRef": {
                    "name": "db-owner",
                    "key": "password"
                }
            }
        }),
        json!({
            "name": "DB_ENC_KEY",
            "valueFrom": {
                "secretKeyRef": {
                    "name": REALTIME_SECRET_NAME,
                    "key": REALTIME_DB_ENC_KEY
                }
            }
        }),
        json!({"name": "DB_AFTER_CONNECT_QUERY", "value": "SET search_path TO _realtime"}),
        json!({
            "name": "API_JWT_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": jwt_secrets::JWT_AUTH_SECRET_NAME,
                    "key": jwt_secrets::JWT_SECRET_KEY
                }
            }
        }),
        json!({
            "name": "SECRET_KEY_BASE",
            "valueFrom": {
                "secretKeyRef": {
                    "name": REALTIME_SECRET_NAME,
                    "key": REALTIME_SECRET_KEY_BASE_KEY
                }
            }
        }),
        json!({"name": "ERL_AFLAGS", "value": "-proto_dist inet_tcp"}),
        json!({"name": "RLIMIT_NOFILE", "value": "1048576"}),
        json!({"name": "DNS_NODES", "value": "''"}),
        json!({"name": "APP_NAME", "value": "realtime"}),
        json!({"name": "RUN_JANITOR", "value": "true"}),
        json!({"name": "JANITOR_INTERVAL", "value": "60000"}),
        json!({"name": "LOG_LEVEL", "value": "info"}),
        json!({"name": "SEED_SELF_HOST", "value": "true"}),
    ];

    let init_container = deployment::InitContainer {
        image_name: REALTIME_INIT_IMAGE.to_string(),
        env: vec![
            json!({"name": "PGHOST", "value": "stack-db-cluster-rw"}),
            json!({"name": "PGPORT", "value": "5432"}),
            json!({"name": "PGDATABASE", "value": "stack-app"}),
            json!({
                "name": "PGUSER",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": "db-owner",
                        "key": "username"
                    }
                }
            }),
            json!({
                "name": "PGPASSWORD",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": "db-owner",
                        "key": "password"
                    }
                }
            }),
        ],
        command: Some(deployment::Command {
            command: vec!["/bin/sh".to_string(), "-c".to_string()],
            args: vec![
                "psql -v ON_ERROR_STOP=1 -c 'CREATE SCHEMA IF NOT EXISTS _realtime;' -c 'ALTER SCHEMA _realtime OWNER TO \"db-owner\";'"
                    .to_string(),
            ],
        }),
    };

    deployment::deployment(
        client,
        deployment::ServiceDeployment {
            name: REALTIME_NAME.to_string(),
            image_name: REALTIME_IMAGE.to_string(),
            replicas: 1,
            port: REALTIME_PORT,
            env,
            init_containers: vec![init_container],
            command: None,
            volume_mounts: vec![],
            volumes: vec![],
        },
        namespace,
        false,
    )
    .await
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(REALTIME_NAME).await.is_ok() {
        deployments
            .delete(REALTIME_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<k8s_openapi::api::core::v1::Service> =
        Api::namespaced(client.clone(), namespace);
    if services.get(REALTIME_NAME).await.is_ok() {
        services
            .delete(REALTIME_NAME, &DeleteParams::default())
            .await?;
    }

    let secrets: Api<Secret> = Api::namespaced(client, namespace);
    if secrets.get(REALTIME_SECRET_NAME).await.is_ok() {
        secrets
            .delete(REALTIME_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

async fn ensure_secret(client: Client, namespace: &str) -> Result<(), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let existing = secret_api.get(REALTIME_SECRET_NAME).await.ok();

    let secret_key_base = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, REALTIME_SECRET_KEY_BASE_KEY))
        .unwrap_or_else(random_token);
    let db_enc_key = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, REALTIME_DB_ENC_KEY))
        .filter(|value| value.len() == DB_ENC_KEY_LEN)
        .unwrap_or_else(|| random_token_len(DB_ENC_KEY_LEN));

    let secret_manifest = json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": REALTIME_SECRET_NAME,
            "namespace": namespace
        },
        "stringData": {
            REALTIME_SECRET_KEY_BASE_KEY: secret_key_base,
            REALTIME_DB_ENC_KEY: db_enc_key
        }
    });

    secret_api
        .patch(
            REALTIME_SECRET_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;

    Ok(())
}

fn random_token() -> String {
    random_token_len(32)
}

fn random_token_len(len: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(len)
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
