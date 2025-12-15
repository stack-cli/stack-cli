use crate::error::Error;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::DeleteParams;
use kube::{Api, Client};
use serde_json::json;

pub const STORAGE_NAME: &str = "storage";
pub const DEFAULT_STORAGE_IMAGE: &str = "supabase/storage-api:v1.33.0";
pub const DEFAULT_STORAGE_PORT: u16 = 5000;

pub async fn deploy(client: Client, namespace: &str) -> Result<(), Error> {
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
        json!({
            "name": "PGRST_JWT_SECRET",
            "value": ""
        }),
        json!({"name": "STORAGE_BACKEND", "value": "file"}),
        json!({"name": "FILE_SIZE_LIMIT", "value": "52428800"}),
        json!({"name": "STORAGE_FILE_LOCAL_STORAGE_PATH", "value": "/var/lib/storage"}),
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

    let services: Api<Service> = Api::namespaced(client, namespace);
    if services.get(STORAGE_NAME).await.is_ok() {
        services
            .delete(STORAGE_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}
