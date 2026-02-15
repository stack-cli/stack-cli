use crate::cli::manifest;
use crate::services::jwt_secrets;
use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::{Pod, Secret};
use kube::api::{ListParams, LogParams};
use kube::{Api, Client, ResourceExt};

fn decode_secret_field(secret: &Secret, key: &str) -> Option<String> {
    if let Some(data) = &secret.data {
        if let Some(value) = data.get(key) {
            if let Ok(decoded) = String::from_utf8(value.0.clone()) {
                if !decoded.is_empty() {
                    return Some(decoded);
                }
            }
        }
    }
    if let Some(string_data) = &secret.string_data {
        if let Some(value) = string_data.get(key) {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }
    }
    None
}

fn extract_cloudflare_url(logs: &str) -> Option<String> {
    for line in logs.lines().rev() {
        if let Some(start) = line.find("https://") {
            let slice = &line[start..];
            let end = slice
                .find(|c: char| c.is_whitespace())
                .unwrap_or(slice.len());
            let mut url = slice[..end].trim_end_matches('.').to_string();
            if url.ends_with('"') {
                url.pop();
            }
            if url.ends_with('"') {
                url.pop();
            }
            if !url.is_empty() {
                return Some(url);
            }
        }
    }
    None
}

pub async fn status(args: &crate::cli::StatusArgs) -> Result<()> {
    println!("🔌 Connecting to the cluster...");
    let client = Client::try_default().await?;
    println!("✅ Connected");

    let (stack_app, _) = manifest::load_stackapp(&args.manifest, args.profile.as_deref())?;

    let namespace = stack_app
        .namespace()
        .ok_or_else(|| anyhow!("StackApp manifest is missing metadata.namespace"))?;

    let keycloak_secret_api: Api<Secret> =
        Api::namespaced(client.clone(), args.keycloak_namespace.as_str());
    match keycloak_secret_api.get("keycloak-initial-admin").await {
        Ok(admin_secret) => {
            let username =
                decode_secret_field(&admin_secret, "username").unwrap_or_else(|| "admin".into());
            let password = decode_secret_field(&admin_secret, "password")
                .unwrap_or_else(|| "<unknown>".into());

            println!("🛡️ Keycloak Admin");
            println!("   Username: {}", username);
            println!("   Password: {}", password);
        }
        Err(_) => {
            println!(
                "🛡️ Keycloak Admin: (not found in namespace '{}' - run `stack init --install-keycloak` to enable)",
                args.keycloak_namespace
            );
        }
    }

    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace.as_str());
    let pod_list = pods
        .list(&ListParams::default().labels("app=cloudflared"))
        .await
        .context("Unable to list cloudflared pods")?;

    if let Some(pod) = pod_list.items.first() {
        let pod_name = pod.name_any();
        let logs = pods
            .logs(
                &pod_name,
                &LogParams {
                    tail_lines: Some(200),
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_default();
        if let Some(url) = extract_cloudflare_url(&logs) {
            let base = url.trim_end_matches('/');
            println!("☁️ Cloudflare URL: {}", base);
            println!("   Keycloak login: {}/realms/{}", base, namespace);
        } else {
            println!("☁️ Cloudflare URL: (not found in recent logs – is the tunnel running?)");
        }
    } else {
        println!(
            "☁️ Cloudflare deployment not found in namespace '{}'",
            namespace
        );
    }

    let jwt_secret_api: Api<Secret> = Api::namespaced(client.clone(), namespace.as_str());
    if let Ok(jwt_secret) = jwt_secret_api.get(jwt_secrets::JWT_AUTH_SECRET_NAME).await {
        let anon_jwt =
            decode_secret_field(&jwt_secret, jwt_secrets::JWT_ANON_TOKEN_KEY).unwrap_or_default();
        let service_role_jwt =
            decode_secret_field(&jwt_secret, jwt_secrets::JWT_SERVICE_ROLE_TOKEN_KEY)
                .unwrap_or_default();

        println!("🔑 JWTs");
        println!("   Anon: {}", anon_jwt);
        println!("   Service role: {}", service_role_jwt);
    } else {
        println!("🔑 JWTs: (jwt-auth secret not found)");
    }

    Ok(())
}
