use crate::error::Error;
use crate::operator::crd::MailhogConfig;
use crate::services::deployment;
use k8s_openapi::api::apps::v1::Deployment as KubeDeployment;
use k8s_openapi::api::core::v1::Service;
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use serde_json::json;

pub const MAILHOG_NAME: &str = "mailhog";
pub const DEFAULT_MAILHOG_IMAGE: &str = "mailhog/mailhog";
pub const DEFAULT_SMTP_PORT: u16 = 1025;
pub const DEFAULT_WEB_PORT: u16 = 8025;

pub async fn deploy(
    client: Client,
    namespace: &str,
    config: Option<&MailhogConfig>,
) -> Result<(), Error> {
    let image = config
        .and_then(|cfg| cfg.image.clone())
        .unwrap_or_else(|| DEFAULT_MAILHOG_IMAGE.to_string());
    let smtp_port = config
        .and_then(|cfg| cfg.smtp_port)
        .unwrap_or(DEFAULT_SMTP_PORT);
    let web_port = config
        .and_then(|cfg| cfg.web_port)
        .unwrap_or(DEFAULT_WEB_PORT);
    let allow_from_anywhere = config
        .map(|cfg| cfg.expose_smtp_port.is_some() || cfg.expose_web_port.is_some())
        .unwrap_or(false);

    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: MAILHOG_NAME.to_string(),
            image_name: image,
            replicas: 1,
            port: smtp_port,
            env: vec![],
            init_containers: vec![],
            command: None,
            volume_mounts: vec![],
            volumes: vec![],
        },
        namespace,
        allow_from_anywhere,
        false,
    )
    .await?;

    let service = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": MAILHOG_NAME,
            "namespace": namespace
        },
        "spec": {
            "type": "ClusterIP",
            "selector": {
                "app": MAILHOG_NAME
            },
            "ports": [
                {
                    "protocol": "TCP",
                    "port": smtp_port,
                    "targetPort": smtp_port,
                    "name": "smtp"
                },
                {
                    "protocol": "TCP",
                    "port": web_port,
                    "targetPort": web_port,
                    "name": "web"
                }
            ]
        }
    });

    let services: Api<Service> = Api::namespaced(client, namespace);
    services
        .patch(
            MAILHOG_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(service),
        )
        .await?;

    Ok(())
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(MAILHOG_NAME).await.is_ok() {
        deployments
            .delete(MAILHOG_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client, namespace);
    if services.get(MAILHOG_NAME).await.is_ok() {
        services
            .delete(MAILHOG_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}
