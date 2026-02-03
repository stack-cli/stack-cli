use crate::error::Error;
use crate::operator::crd::SupabaseAuthConfig;
use crate::services::{database, deployment, jwt_secrets};
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::DeleteParams;
use kube::{Api, Client};
use serde_json::json;

pub const AUTH_NAME: &str = "auth";
pub const AUTH_IMAGE: &str = "supabase/gotrue:v2.185.0";
pub const AUTH_PORT: u16 = 9999;
const AUTH_INIT_IMAGE: &str = "postgres:16-alpine";

pub async fn deploy(
    client: Client,
    namespace: &str,
    app_name: &str,
    config: &SupabaseAuthConfig,
) -> Result<(), Error> {
    jwt_secrets::ensure_secret(client.clone(), namespace).await?;

    let env = vec![
        json!({"name": "GOTRUE_API_PORT", "value": AUTH_PORT.to_string()}),
        json!({"name": "GOTRUE_DB_DRIVER", "value": "postgres"}),
        json!({"name": "API_EXTERNAL_URL", "value": config.api_external_url.clone()}),
        json!({"name": "GOTRUE_SITE_URL", "value": config.gotrue_site_url.clone()}),
        json!({"name": "GOTRUE_JWT_ADMIN_ROLES", "value": "service_role"}),
        json!({"name": "GOTRUE_JWT_AUD", "value": "authenticated"}),
        json!({"name": "GOTRUE_JWT_DEFAULT_GROUP_NAME", "value": "authenticated"}),
        json!({
            "name": "GOTRUE_DB_DATABASE_URL",
            "valueFrom": {
                "secretKeyRef": {
                    "name": "database-urls",
                    "key": "migrations-url"
                }
            }
        }),
        json!({
            "name": "GOTRUE_JWT_SECRET",
            "valueFrom": {
                "secretKeyRef": {
                    "name": jwt_secrets::JWT_AUTH_SECRET_NAME,
                    "key": jwt_secrets::JWT_SECRET_KEY
                }
            }
        }),
    ];

    let init_container = deployment::InitContainer {
        image_name: AUTH_INIT_IMAGE.to_string(),
        env: vec![
            json!({"name": "PGHOST", "value": database::cluster_rw_service_name(app_name)}),
            json!({"name": "PGPORT", "value": "5432"}),
            json!({"name": "PGDATABASE", "value": database::database_name(app_name)}),
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
                "psql -v ON_ERROR_STOP=1 -c 'CREATE SCHEMA IF NOT EXISTS auth;' -c 'ALTER SCHEMA auth OWNER TO \"db-owner\";'"
                    .to_string(),
            ],
        }),
    };

    deployment::deployment(
        client,
        deployment::ServiceDeployment {
            name: AUTH_NAME.to_string(),
            image_name: AUTH_IMAGE.to_string(),
            replicas: 1,
            port: Some(AUTH_PORT),
            env,
            init_containers: vec![init_container],
            command: None,
            volume_mounts: vec![],
            volumes: vec![],
        },
        namespace,
        false,
        false,
    )
    .await
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(AUTH_NAME).await.is_ok() {
        deployments
            .delete(AUTH_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client, namespace);
    if services.get(AUTH_NAME).await.is_ok() {
        services.delete(AUTH_NAME, &DeleteParams::default()).await?;
    }

    Ok(())
}
