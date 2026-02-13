use crate::publish;
use anyhow::{anyhow, Context, Result};
use dagger_sdk::{Container, File, Query};
use std::env;

const OPERATOR_ARTIFACT_PATH: &str = "artifacts/operator/stack-operator.tar";
const DEFAULT_REGISTRY: &str = "ghcr.io";
const DEFAULT_REPOSITORY: &str = "stack-cli/stack-operator";

pub async fn package_and_publish(client: &Query, operator_binary: &File) -> Result<()> {
    let container = client
        .container()
        .with_file("/k8s-operator", operator_binary.clone())
        .with_entrypoint(vec!["./k8s-operator", "operator"]);

    container
        .export(OPERATOR_ARTIFACT_PATH)
        .await
        .context("failed to export stack operator container")?;

    publish_image(client, &container).await?;

    Ok(())
}

async fn publish_image(client: &Query, container: &Container) -> Result<()> {
    let registry = env::var("STACK_OPERATOR_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.into());
    let repository =
        env::var("STACK_OPERATOR_REPOSITORY").unwrap_or_else(|_| DEFAULT_REPOSITORY.into());
    let tags = collect_image_tags();

    if tags.is_empty() {
        println!("No operator image tags provided; skipping publish step.");
        return Ok(());
    }

    let require_publish = env::var("CI").is_ok();
    let credentials = publish::load_credentials(
        client,
        &["GHCR_USERNAME", "GITHUB_ACTOR"],
        &["GHCR_TOKEN", "GITHUB_TOKEN"],
        "operator",
    )
    .await?;

    if credentials.is_none() && require_publish {
        return Err(anyhow!(
            "publishing operator images requires GHCR credentials (`GHCR_USERNAME`/`GITHUB_ACTOR` and `GHCR_TOKEN`/`GITHUB_TOKEN`)"
        ));
    }

    publish::publish_container_tags(
        container,
        &registry,
        &repository,
        &tags,
        credentials.as_ref(),
        "operator",
    )
    .await
}

fn collect_image_tags() -> Vec<String> {
    publish::collect_sanitized_tags(
        &[
            env::var("STACK_VERSION").ok(),
            env::var("GITHUB_REF_NAME").ok(),
            env::var("GITHUB_SHA")
                .ok()
                .map(|sha| sha.chars().take(7).collect::<String>())
                .map(|sha| format!("sha-{sha}")),
        ],
        false,
    )
}
