use crate::cli::init::ensure_namespace;
use crate::cli::manifest;
use crate::services::cloudflare::{
    self, SECRET_INGRESS_TARGET_KEY, SECRET_TOKEN_KEY, SECRET_TUNNEL_NAME_KEY,
};
use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, ResourceExt};

pub async fn cloudflare(args: &crate::cli::CloudflareArgs) -> Result<()> {
    println!("🔌 Connecting to the cluster...");
    let client = Client::try_default().await?;
    println!("✅ Connected");

    let (stack_app, _) = manifest::load_stackapp(&args.manifest, None)?;

    let namespace = stack_app
        .namespace()
        .ok_or_else(|| anyhow!("StackApp manifest is missing metadata.namespace"))?;

    ensure_namespace(&client, &namespace).await?;

    match args.tunnel_name.as_deref() {
        Some(tunnel_name) => {
            let token = args
                .token
                .as_ref()
                .ok_or_else(|| anyhow!("--token is required when using --tunnel-name"))?;
            let secret_name = format!("cloudflare-{}", tunnel_name);

            let mut string_data = serde_json::Map::new();
            string_data.insert(SECRET_TOKEN_KEY.to_string(), token.clone().into());
            string_data.insert(
                SECRET_TUNNEL_NAME_KEY.to_string(),
                tunnel_name.to_string().into(),
            );
            if let Some(ingress_target) = args.ingress_target.as_ref() {
                string_data.insert(
                    SECRET_INGRESS_TARGET_KEY.to_string(),
                    ingress_target.clone().into(),
                );
            }

            let secret_manifest = serde_json::json!({
                "apiVersion": "v1",
                "kind": "Secret",
                "metadata": {
                    "name": secret_name,
                    "namespace": namespace
                },
                "type": "Opaque",
                "stringData": string_data
            });

            let secrets_api: Api<Secret> = Api::namespaced(client.clone(), namespace.as_str());
            secrets_api
                .patch(
                    &secret_name,
                    &PatchParams::apply(crate::MANAGER).force(),
                    &Patch::Apply(secret_manifest),
                )
                .await
                .context("Failed to apply Cloudflare secret")?;

            cloudflare::deploy(&client, &namespace, Some(&secret_name))
                .await
                .context("Failed to deploy Cloudflare resources")?;

            println!(
                "☁️ Cloudflare deployment applied in namespace '{}' with secret '{}'",
                namespace, secret_name
            );
        }
        None => {
            if args.token.is_some() {
                return Err(anyhow!("--token requires --tunnel-name"));
            }

            cloudflare::deploy(&client, &namespace, None)
                .await
                .context("Failed to deploy Cloudflare resources")?;

            println!(
                "☁️ Cloudflare quick tunnel deployment applied in namespace '{}'",
                namespace
            );
        }
    }

    Ok(())
}
