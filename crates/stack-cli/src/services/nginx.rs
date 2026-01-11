use crate::cli::apply;
use crate::error::Error;
use crate::services::application::APPLICATION_NAME;
use k8s_openapi::api::{
    apps::v1::Deployment as KubeDeployment,
    core::v1::{ConfigMap, Service},
};
use kube::api::DeleteParams;
use kube::{Api, Client};
use serde_json::json;

use super::deployment;

pub const NGINX_NAME: &str = "nginx";
pub const NGINX_PORT: u16 = 80;

pub enum NginxMode {
    Oidc { allow_admin: bool },
    StaticJwt { token: String },
}

fn proxy_block(
    path: &str,
    service: &str,
    port: u16,
    proto_var: &str,
    upstream_path: &str,
) -> String {
    format!(
        r#"
    location = {path} {{
        return 301 {path}/;
    }}

    location ^~ {path}/ {{
        proxy_pass http://{service}:{port}{upstream_path};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto {proto_var};
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header Authorization $http_authorization;
        proxy_set_header X-Auth-JWT $http_x_auth_jwt;
    }}
"#,
        path = path,
        service = service,
        port = port,
        proto_var = proto_var,
        upstream_path = upstream_path
    )
}

fn websocket_block(
    path: &str,
    service: &str,
    port: u16,
    proto_var: &str,
    upstream_path: &str,
) -> String {
    format!(
        r#"
    location = {path} {{
        return 301 {path}/;
    }}

    location ^~ {path}/ {{
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_pass http://{service}:{port}{upstream_path};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto {proto_var};
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header Authorization $http_authorization;
        proxy_set_header X-Auth-JWT $http_x_auth_jwt;
    }}
"#,
        path = path,
        service = service,
        port = port,
        proto_var = proto_var,
        upstream_path = upstream_path
    )
}

// The web user interface
pub async fn deploy_nginx(
    client: &Client,
    namespace: &str,
    mode: NginxMode,
    upstream_port: u16,
    include_storage: bool,
    include_rest: bool,
    include_realtime: bool,
) -> Result<(), Error> {
    let env = vec![];

    let image_name = "nginx:1.27.2".to_string();

    let storage_block = if include_storage {
        proxy_block("/storage/v1", "storage", 5000, "$forwarded_proto", "/")
    } else {
        String::new()
    };
    let rest_block = if include_rest {
        proxy_block("/rest/v1", "rest", 3000, "$forwarded_proto", "/")
    } else {
        String::new()
    };
    let realtime_block = if include_realtime {
        let ws_block = websocket_block(
            "/realtime/v1",
            "realtime",
            4000,
            "$forwarded_proto",
            "/socket/",
        );
        let rest_block = proxy_block("/realtime/v1/api", "realtime", 4000, "$forwarded_proto", "/api/");

        format!(
            r#"{ws_block}
{rest_block}"#
        )
        .replace("proxy_set_header Host $host;", "proxy_set_header Host realtime-dev;")
        .replace(
            "proxy_set_header X-Forwarded-Host $host;",
            "proxy_set_header X-Forwarded-Host realtime-dev;",
        )
    } else {
        String::new()
    };

    let config_body = match mode {
        NginxMode::Oidc { allow_admin } => {
            let admin_block = if allow_admin {
                String::new()
            } else {
                r#"
    location ^~ /oidc/admin {
        return 404;
    }
"#
                .to_string()
            };
            format!(
                r#"
server {{
    listen 80;

    # Increase buffer sizes to handle large headers
    proxy_buffer_size   128k;
    proxy_buffers       4 256k;
    proxy_busy_buffers_size 256k;
    set $forwarded_proto $scheme;
    if ($http_x_forwarded_proto != "") {{
        set $forwarded_proto $http_x_forwarded_proto;
    }}

{admin_block}
    location = /oidc {{
        return 301 /oidc/;
    }}

    location ^~ /oidc/ {{
        proxy_pass http://keycloak-service:8080/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $forwarded_proto;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-Prefix /oidc;
        proxy_redirect ~^http://keycloak-service\.keycloak\.svc\.cluster\.local:8080/(.*)$ $scheme://$host/oidc/$1;
    }}

{storage_block}
{rest_block}
{realtime_block}

    location / {{
        proxy_pass http://oauth2-proxy:7900;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
        proxy_redirect ~^http://keycloak-service\.keycloak\.svc\.cluster\.local:8080/(.*)$ $scheme://$host/$1;
    }}
}}
"#,
                admin_block = admin_block
                , storage_block = storage_block
                , rest_block = rest_block
                , realtime_block = realtime_block
            )
        }
        NginxMode::StaticJwt { token } => {
            let escaped_token = token.replace('"', "\\\"");
            let storage_block = if include_storage {
                proxy_block("/storage/v1", "storage", 5000, "$scheme", "/")
            } else {
                String::new()
            };
            let rest_block = if include_rest {
                proxy_block("/rest/v1", "rest", 3000, "$scheme", "/")
            } else {
                String::new()
            };
            let realtime_block = if include_realtime {
                let ws_block = websocket_block("/realtime/v1", "realtime", 4000, "$scheme", "/socket/");
                let rest_block = proxy_block("/realtime/v1/api", "realtime", 4000, "$scheme", "/api/");

                format!(
                    r#"{ws_block}
{rest_block}"#
                )
                .replace("proxy_set_header Host $host;", "proxy_set_header Host realtime-dev;")
                .replace(
                    "proxy_set_header X-Forwarded-Host $host;",
                    "proxy_set_header X-Forwarded-Host realtime-dev;",
                )
            } else {
                String::new()
            };
            format!(
                r#"
server {{
    listen 80;

    proxy_buffer_size   128k;
    proxy_buffers       4 256k;
    proxy_busy_buffers_size 256k;

{storage_block}
{rest_block}
{realtime_block}

    location / {{
        proxy_pass http://{app}:{port};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header Authorization "Bearer {token}";
        proxy_set_header X-Auth-JWT "{token}";
    }}
}}
"#,
                app = APPLICATION_NAME,
                port = upstream_port,
                token = escaped_token
                , storage_block = storage_block
                , rest_block = rest_block
                , realtime_block = realtime_block
            )
        }
    };

    // Put the nginx config into a ConfigMap
    let config_map = serde_json::json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": NGINX_NAME,
            "namespace": namespace
        },
        "data": {
            "default.conf": config_body,
        }
    });

    apply::apply(client, &config_map.to_string(), Some(namespace))
        .await
        .map_err(Error::from)?;

    // Application with the migrations as a sidecar
    deployment::deployment(
        client.clone(),
        deployment::ServiceDeployment {
            name: NGINX_NAME.to_string(),
            image_name,
            replicas: 1,
            port: NGINX_PORT,
            env,
            command: None,
            init_containers: vec![],
            volume_mounts: vec![json!({"name": NGINX_NAME, "mountPath": "/etc/nginx/conf.d"})],
            volumes: vec![json!({"name": NGINX_NAME,
                "configMap": {
                    "name": NGINX_NAME
                }
            })],
        },
        namespace,
        true,
    )
    .await?;

    Ok(())
}

pub async fn delete_nginx(client: Client, namespace: &str) -> Result<(), Error> {
    let deployments: Api<KubeDeployment> = Api::namespaced(client.clone(), namespace);
    if deployments.get(NGINX_NAME).await.is_ok() {
        deployments
            .delete(NGINX_NAME, &DeleteParams::default())
            .await?;
    }

    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    if services.get(NGINX_NAME).await.is_ok() {
        services
            .delete(NGINX_NAME, &DeleteParams::default())
            .await?;
    }

    let configs: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    if configs.get(NGINX_NAME).await.is_ok() {
        configs.delete(NGINX_NAME, &DeleteParams::default()).await?;
    }

    Ok(())
}
