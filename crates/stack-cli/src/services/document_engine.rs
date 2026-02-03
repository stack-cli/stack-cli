use crate::error::Error;
use crate::operator::crd::DocumentEngineConfig;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::DeleteParams;
use kube::{Api, Client};

pub const DOCUMENT_ENGINE_NAME: &str = "document-engine";
pub const DOCUMENT_ENGINE_IMAGE: &str = "ghcr.io/kreuzberg-dev/kreuzberg:4.1.0";
pub const DOCUMENT_ENGINE_PORT: u16 = 8000;

pub async fn deploy(
    client: Client,
    namespace: &str,
    _config: Option<&DocumentEngineConfig>,
) -> Result<(), Error> {
    deployment::deployment(
        client,
        deployment::ServiceDeployment {
            name: DOCUMENT_ENGINE_NAME.to_string(),
            image_name: DOCUMENT_ENGINE_IMAGE.to_string(),
            replicas: 1,
            port: Some(DOCUMENT_ENGINE_PORT),
            env: vec![],
            init_containers: vec![],
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
    if deployments.get(DOCUMENT_ENGINE_NAME).await.is_ok() {
        deployments
            .delete(DOCUMENT_ENGINE_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client, namespace);
    if services.get(DOCUMENT_ENGINE_NAME).await.is_ok() {
        services
            .delete(DOCUMENT_ENGINE_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}
