use super::{
    apply,
    init::{ensure_namespace, ensure_stackapp_crd},
    manifest,
};
use anyhow::{anyhow, Context, Result};
use kube::{Client, ResourceExt};

pub async fn deploy(deployer: &crate::cli::Deployer) -> Result<()> {
    println!("🔌 Connecting to the cluster...");
    let client = Client::try_default().await?;
    println!("✅ Connected");

    let (stack_app, manifest_raw) =
        manifest::load_stackapp(&deployer.manifest, deployer.profile.as_deref())?;

    let namespace = stack_app
        .namespace()
        .ok_or_else(|| anyhow!("StackApp manifest is missing metadata.namespace"))?;

    let app_name = stack_app.name_any();

    ensure_stackapp_crd(&client).await?;
    ensure_namespace(&client, &namespace).await?;

    apply::apply(&client, &manifest_raw, None)
        .await
        .context("Failed to apply StackApp manifest")?;

    println!(
        "🚀 Applied StackApp `{}` in namespace `{}`",
        app_name, namespace
    );

    Ok(())
}
