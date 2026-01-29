use crate::error::Error;
use crate::operator::crd::SeleniumConfig;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use serde_json::json;

pub const SELENIUM_NAME: &str = "selenium";
pub const DEFAULT_SELENIUM_IMAGE: &str = "selenium/standalone-chrome";
pub const DEFAULT_SELENIUM_PORT: u16 = 4444;
pub const DEFAULT_SELENIUM_VNC_PORT: u16 = 7900;
const DEFAULT_SHM_SIZE: &str = "2Gi";

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&SeleniumConfig>,
) -> Result<(), Error> {
    let image = config
        .and_then(|cfg| cfg.image.clone())
        .unwrap_or_else(|| DEFAULT_SELENIUM_IMAGE.to_string());
    let port = config
        .and_then(|cfg| cfg.port)
        .unwrap_or(DEFAULT_SELENIUM_PORT);
    let vnc_port = config
        .and_then(|cfg| cfg.vnc_port)
        .unwrap_or(DEFAULT_SELENIUM_VNC_PORT);
    let shm_size = config
        .and_then(|cfg| cfg.shm_size.clone())
        .unwrap_or_else(|| DEFAULT_SHM_SIZE.to_string());

    let volume_mounts = vec![json!({
        "name": "dshm",
        "mountPath": "/dev/shm"
    })];
    let volumes = vec![json!({
        "name": "dshm",
        "emptyDir": {
            "medium": "Memory",
            "sizeLimit": shm_size
        }
    })];

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: SELENIUM_NAME.to_string(),
            image_name: image,
            replicas: 1,
            port,
            env: vec![],
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

    // Ensure the service exposes both ports (webdriver + VNC).
    let service = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": SELENIUM_NAME,
            "namespace": namespace
        },
        "spec": {
            "type": "ClusterIP",
            "selector": {
                "app": SELENIUM_NAME
            },
            "ports": [
                {
                    "protocol": "TCP",
                    "port": port,
                    "targetPort": port,
                    "name": "webdriver"
                },
                {
                    "protocol": "TCP",
                    "port": vnc_port,
                    "targetPort": vnc_port,
                    "name": "vnc"
                }
            ]
        }
    });

    let services: Api<Service> = Api::namespaced(client, namespace);
    services
        .patch(
            SELENIUM_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(service),
        )
        .await?;

    Ok(())
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(SELENIUM_NAME).await.is_ok() {
        deployments
            .delete(SELENIUM_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client, namespace);
    if services.get(SELENIUM_NAME).await.is_ok() {
        services
            .delete(SELENIUM_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}
