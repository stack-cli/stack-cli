use crate::cli::apply;
use crate::error::Error;
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
    Oidc,
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
        proxy_set_header Sec-WebSocket-Protocol $http_sec_websocket_protocol;
        proxy_pass_header Sec-WebSocket-Protocol;
        add_header Sec-WebSocket-Protocol $http_sec_websocket_protocol always;
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

fn auth_proxy_block(proto_var: &str) -> String {
    format!(
        r#"
    location = /auth {{
        return 301 /auth/v1/;
    }}

    location = /auth/ {{
        return 301 /auth/v1/;
    }}

    location ^~ /auth/v1/ {{
        if ($request_method = OPTIONS) {{
            add_header Access-Control-Allow-Origin $http_origin always;
            add_header Access-Control-Allow-Credentials "true" always;
            add_header Access-Control-Allow-Methods "GET, POST, PUT, PATCH, DELETE, OPTIONS" always;
            add_header Access-Control-Allow-Headers "authorization, apikey, content-type, x-client-info, x-auth-jwt, x-supabase-api-version, x-supabase-client" always;
            add_header Access-Control-Max-Age 86400 always;
            add_header Vary Origin always;
            return 204;
        }}

        proxy_pass http://auth:9999/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto {proto_var};
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header Authorization $http_authorization;
        proxy_set_header X-Auth-JWT $http_x_auth_jwt;
    }}
"#,
        proto_var = proto_var
    )
}

fn storage_proxy_block(proto_var: &str) -> String {
    format!(
        r#"
    location = /storage/v1 {{
        return 301 /storage/v1/;
    }}

    location ^~ /storage/v1/ {{
        if ($request_method = OPTIONS) {{
            add_header Access-Control-Allow-Origin $http_origin always;
            add_header Access-Control-Allow-Credentials "true" always;
            add_header Access-Control-Allow-Methods "GET, POST, PUT, PATCH, DELETE, OPTIONS" always;
            add_header Access-Control-Allow-Headers "authorization, apikey, content-type, x-client-info, x-upsert, x-auth-jwt, x-supabase-api-version, x-supabase-client" always;
            add_header Access-Control-Max-Age 86400 always;
            add_header Vary Origin always;
            return 204;
        }}

        proxy_pass http://storage:5000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto {proto_var};
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header Authorization $http_authorization;
        proxy_set_header X-Auth-JWT $http_x_auth_jwt;
    }}
"#,
        proto_var = proto_var
    )
}

// The web user interface
pub async fn deploy_nginx(
    client: &Client,
    namespace: &str,
    mode: NginxMode,
    upstream_port: u16,
    app_name: &str,
    include_auth: bool,
    include_storage: bool,
    include_rest: bool,
    include_realtime: bool,
    include_document_engine: bool,
) -> Result<(), Error> {
    let env = vec![];

    let image_name = "nginx:1.27.2".to_string();

    let storage_block = if include_storage {
        storage_proxy_block("$forwarded_proto")
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
        let rest_block = proxy_block(
            "/realtime/v1/api",
            "realtime",
            4000,
            "$forwarded_proto",
            "/api/",
        );

        format!(
            r#"{ws_block}
{rest_block}"#
        )
        .replace(
            "proxy_set_header Host $host;",
            "proxy_set_header Host realtime-dev;",
        )
        .replace(
            "proxy_set_header X-Forwarded-Host $host;",
            "proxy_set_header X-Forwarded-Host realtime-dev;",
        )
    } else {
        String::new()
    };
    let document_engine_block = if include_document_engine {
        proxy_block(
            "/document-engine",
            "document-engine",
            8000,
            "$forwarded_proto",
            "/",
        )
    } else {
        String::new()
    };

    let auth_block = if include_auth {
        let proto_var = match mode {
            NginxMode::Oidc => "$forwarded_proto",
            NginxMode::StaticJwt { .. } => "$scheme",
        };
        auth_proxy_block(proto_var)
    } else {
        String::new()
    };

    let config_body = match mode {
        NginxMode::Oidc => {
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
{auth_block}
{rest_block}
{realtime_block}
{document_engine_block}

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
                auth_block = auth_block,
                storage_block = storage_block,
                rest_block = rest_block,
                realtime_block = realtime_block,
                document_engine_block = document_engine_block
            )
        }
        NginxMode::StaticJwt { token } => {
            let escaped_token = token.replace('"', "\\\"");
            let storage_block = if include_storage {
                storage_proxy_block("$scheme")
            } else {
                String::new()
            };
            let rest_block = if include_rest {
                proxy_block("/rest/v1", "rest", 3000, "$scheme", "/")
            } else {
                String::new()
            };
            let realtime_block = if include_realtime {
                let ws_block =
                    websocket_block("/realtime/v1", "realtime", 4000, "$scheme", "/socket/");
                let rest_block =
                    proxy_block("/realtime/v1/api", "realtime", 4000, "$scheme", "/api/");

                format!(
                    r#"{ws_block}
{rest_block}"#
                )
                .replace(
                    "proxy_set_header Host $host;",
                    "proxy_set_header Host realtime-dev;",
                )
                .replace(
                    "proxy_set_header X-Forwarded-Host $host;",
                    "proxy_set_header X-Forwarded-Host realtime-dev;",
                )
            } else {
                String::new()
            };
            let document_engine_block = if include_document_engine {
                proxy_block("/document-engine", "document-engine", 8000, "$scheme", "/")
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
{auth_block}
{rest_block}
{realtime_block}
{document_engine_block}

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
                app = app_name,
                port = upstream_port,
                token = escaped_token,
                auth_block = auth_block,
                storage_block = storage_block,
                rest_block = rest_block,
                realtime_block = realtime_block,
                document_engine_block = document_engine_block
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
            port: Some(NGINX_PORT),
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
        false,
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
