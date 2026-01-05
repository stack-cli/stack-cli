use crate::error::Error;
use crate::operator::crd::RestConfig;
use crate::services::deployment;
use crate::services::jwt_secrets;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::DeleteParams;
use kube::{Api, Client};
use serde_json::json;

pub const REST_NAME: &str = "rest";
pub const DEFAULT_REST_IMAGE: &str = "postgrest/postgrest:v14.1";
pub const DEFAULT_REST_PORT: u16 = 3000;
const DEFAULT_DB_SCHEMAS: &str = "public";
const DEFAULT_JWT_EXPIRY: &str = "3600";

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&RestConfig>,
) -> Result<(), Error> {
    jwt_secrets::ensure_secret(client.clone(), namespace).await?;

    let db_schemas = config
        .and_then(|c| c.db_schemas.clone())
        .unwrap_or_else(|| DEFAULT_DB_SCHEMAS.to_string());
    let jwt_expiry = config
        .and_then(|c| c.jwt_expiry.clone())
        .unwrap_or_else(|| DEFAULT_JWT_EXPIRY.to_string());

    let env = vec![
        json!({
            "name": "PGRST_DB_URI",
            "valueFrom": {
                "secretKeyRef": {
                    "name": "database-urls",
                    "key": "authenticator-url"
                }
            }
        }),
        json!({"name": "PGRST_DB_SCHEMAS", "value": db_schemas}),
        json!({"name": "PGRST_DB_ANON_ROLE", "value": "anon"}),
        json!({
            "name": "PGRST_JWT_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": jwt_secrets::JWT_AUTH_SECRET_NAME,
                    "key": jwt_secrets::JWT_SECRET_KEY
                }
            }
        }),
        json!({"name": "PGRST_DB_USE_LEGACY_GUCS", "value": "false"}),
        json!({
            "name": "PGRST_APP_SETTINGS_JWT_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": jwt_secrets::JWT_AUTH_SECRET_NAME,
                    "key": jwt_secrets::JWT_SECRET_KEY
                }
            }
        }),
        json!({
            "name": "PGRST_APP_SETTINGS_JWT_EXP",
            "value": jwt_expiry
        }),
    ];

    let command = deployment::Command {
        command: vec!["postgrest".to_string()],
        args: vec![],
    };

    deployment::deployment(
        client,
        deployment::ServiceDeployment {
            name: REST_NAME.to_string(),
            image_name: DEFAULT_REST_IMAGE.to_string(),
            replicas: 1,
            port: DEFAULT_REST_PORT,
            env,
            init_container: None,
            command: Some(command),
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
    if deployments.get(REST_NAME).await.is_ok() {
        deployments.delete(REST_NAME, &DeleteParams::default()).await?;
    }

    let services: Api<Service> = Api::namespaced(client, namespace);
    if services.get(REST_NAME).await.is_ok() {
        services.delete(REST_NAME, &DeleteParams::default()).await?;
    }

    Ok(())
}
