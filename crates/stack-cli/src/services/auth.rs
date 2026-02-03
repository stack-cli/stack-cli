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
const AUTH_ADMIN_USER: &str = "supabase_auth_admin";
const AUTH_ADMIN_PASSWORD: &str = "testpassword";

pub async fn deploy(
    client: Client,
    namespace: &str,
    app_name: &str,
    config: &SupabaseAuthConfig,
) -> Result<(), Error> {
    jwt_secrets::ensure_secret(client.clone(), namespace).await?;

    let cluster_rw_service = database::cluster_rw_service_name(app_name);
    let db_name = database::database_name(app_name);

    let env = vec![
        json!({"name": "GOTRUE_API_PORT", "value": AUTH_PORT.to_string()}),
        json!({"name": "GOTRUE_DB_DRIVER", "value": "postgres"}),
        json!({"name": "API_EXTERNAL_URL", "value": config.api_external_url.clone()}),
        json!({"name": "GOTRUE_SITE_URL", "value": config.site_url.clone()}),
        json!({"name": "GOTRUE_JWT_ADMIN_ROLES", "value": "service_role"}),
        json!({"name": "GOTRUE_JWT_AUD", "value": "authenticated"}),
        json!({"name": "GOTRUE_JWT_DEFAULT_GROUP_NAME", "value": "authenticated"}),
        json!({
            "name": "GOTRUE_DB_DATABASE_URL",
            "value": format!(
                "postgres://{}:{}@{}:5432/{}?sslmode=disable",
                AUTH_ADMIN_USER,
                AUTH_ADMIN_PASSWORD,
                cluster_rw_service,
                db_name
            )
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
            json!({"name": "PGHOST", "value": cluster_rw_service}),
            json!({"name": "PGPORT", "value": "5432"}),
            json!({"name": "PGDATABASE", "value": db_name}),
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
                format!(
                    "psql -v ON_ERROR_STOP=1 -c \"DO \\$\\$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{user}') THEN CREATE USER {user} NOINHERIT CREATEROLE LOGIN NOREPLICATION PASSWORD '{password}'; END IF; END \\$\\$;\" \
                     -c \"CREATE SCHEMA IF NOT EXISTS auth AUTHORIZATION {user};\" \
                     -c \"GRANT CREATE ON DATABASE \\\"{db}\\\" TO {user};\" \
                     -c \"ALTER USER {user} SET search_path = 'auth';\"",
                    user = AUTH_ADMIN_USER,
                    password = AUTH_ADMIN_PASSWORD,
                    db = db_name
                ),
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
