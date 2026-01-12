use crate::cli::manifest;
use crate::services::jwt_secrets;
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client, ResourceExt};
use std::collections::BTreeMap;
use url::Url;

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

fn to_env_key(key: &str) -> String {
    key.to_ascii_uppercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn rewrite_database_url(value: &str, host: Option<&str>, port: Option<u16>) -> Result<String> {
    if host.is_none() && port.is_none() {
        return Ok(value.to_string());
    }

    let mut url = Url::parse(value)
        .map_err(|err| anyhow!("failed to parse database url '{}': {}", value, err))?;

    if let Some(host) = host {
        url.set_host(Some(host))
            .map_err(|err| anyhow!("failed to set database host '{}': {}", host, err))?;
    }

    if let Some(port) = port {
        url.set_port(Some(port))
            .map_err(|_| anyhow!("failed to set database port {}", port))?;
    }

    Ok(url.to_string())
}

pub async fn secrets(args: &crate::cli::SecretsArgs) -> Result<()> {
    let client = Client::try_default().await?;

    let (stack_app, _) = manifest::load_stackapp(&args.manifest, args.profile.as_deref())?;

    let namespace = stack_app
        .namespace()
        .ok_or_else(|| anyhow!("StackApp manifest is missing metadata.namespace"))?;

    let secrets_api: Api<Secret> = Api::namespaced(client, namespace.as_str());
    let mut env_vars: BTreeMap<String, String> = BTreeMap::new();

    if let Ok(database_urls) = secrets_api.get("database-urls").await {
        let keys = [
            "application-url",
            "migrations-url",
            "readonly-url",
            "authenticator-url",
        ];
        for key in keys {
            if let Some(value) = decode_secret_field(&database_urls, key) {
                env_vars.insert(to_env_key(key), value);
            }
        }
    }

    if args.db_host.is_some() || args.db_port.is_some() {
        let keys = [
            to_env_key("application-url"),
            to_env_key("migrations-url"),
            to_env_key("readonly-url"),
            to_env_key("authenticator-url"),
        ];
        for key in keys {
            if let Some(value) = env_vars.get(&key).cloned() {
                let rewritten = rewrite_database_url(
                    &value,
                    args.db_host.as_deref(),
                    args.db_port,
                )?;
                env_vars.insert(key, rewritten);
            }
        }
    }

    if let Ok(jwt_secret) = secrets_api.get(jwt_secrets::JWT_AUTH_SECRET_NAME).await {
        let jwt_keys = [
            jwt_secrets::JWT_ANON_TOKEN_KEY,
            jwt_secrets::JWT_SERVICE_ROLE_TOKEN_KEY,
        ];
        for key in jwt_keys {
            if let Some(value) = decode_secret_field(&jwt_secret, key) {
                env_vars.insert(to_env_key(key), value);
            }
        }
    }

    for (key, value) in env_vars {
        println!("{}={}", key, value);
    }

    Ok(())
}
