use crate::publish;
use anyhow::{anyhow, Context, Result};
use dagger_sdk::{Container, Directory, Query};
use std::env;

const BASE_IMAGE: &str = "node:22-bookworm";
const RUNTIME_IMAGE: &str = "node:22-bookworm-slim";
const DEMO_APP_DIR: &str = "examples/react-supabase-next";
const DEMO_ARTIFACT_PATH: &str = "artifacts/demo-app/react-supabase-next.tar";
const DEFAULT_REGISTRY: &str = "ghcr.io";
const DEFAULT_REPOSITORY: &str = "stack-cli/react-supabase-next";

pub async fn build_and_publish(client: &Query, repo: &Directory) -> Result<()> {
    let app_dir = repo.directory(DEMO_APP_DIR);

    let builder = client
        .container()
        .from(BASE_IMAGE)
        .with_directory("/app", app_dir)
        .with_workdir("/app")
        .with_env_variable("NEXT_TELEMETRY_DISABLED", "1")
        .with_exec(vec!["corepack", "enable"])
        .with_exec(vec!["pnpm", "install", "--frozen-lockfile"])
        .with_exec(vec!["pnpm", "build"]);

    let container = client
        .container()
        .from(RUNTIME_IMAGE)
        .with_workdir("/app")
        .with_env_variable("NODE_ENV", "production")
        .with_env_variable("NEXT_TELEMETRY_DISABLED", "1")
        .with_env_variable("HOSTNAME", "0.0.0.0")
        .with_env_variable("PORT", "8080")
        .with_directory("/app", builder.directory("/app/.next/standalone"))
        .with_directory("/app/.next/static", builder.directory("/app/.next/static"))
        .with_entrypoint(vec!["node", "server.js"]);

    if should_export_artifact() {
        println!("Exporting demo app image artifact to {DEMO_ARTIFACT_PATH}...");
        container
            .clone()
            .export(DEMO_ARTIFACT_PATH)
            .await
            .context("failed to export demo app container")?;
        println!("Exported demo app image artifact.");
    } else {
        println!("Skipping demo app image artifact export (set STACK_DEMO_APP_EXPORT_ARTIFACT=1 to enable).");
    }

    publish_image(client, &container).await
}

async fn publish_image(client: &Query, container: &Container) -> Result<()> {
    if !is_main_ref() {
        println!("Skipping demo app publish: branch is not main");
        return Ok(());
    }

    let registry = env::var("STACK_DEMO_APP_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.into());
    let repository =
        env::var("STACK_DEMO_APP_REPOSITORY").unwrap_or_else(|_| DEFAULT_REPOSITORY.into());
    let tags = collect_image_tags();

    if tags.is_empty() {
        println!("No demo app image tags provided; skipping publish step.");
        return Ok(());
    }

    let credentials = publish::load_credentials(
        client,
        &["GHCR_USERNAME", "GITHUB_ACTOR"],
        &["GHCR_TOKEN", "GITHUB_TOKEN"],
        "demo app",
    )
    .await?;

    if credentials.is_none() {
        return Err(anyhow!(
            "publishing demo app images requires GHCR credentials (`GHCR_USERNAME`/`GITHUB_ACTOR` and `GHCR_TOKEN`/`GITHUB_TOKEN`)"
        ));
    }

    publish::publish_container_tags(
        container,
        &registry,
        &repository,
        &tags,
        credentials.as_ref(),
        "demo app",
    )
    .await
}

fn is_main_ref() -> bool {
    env::var("GITHUB_REF_NAME")
        .map(|v| v == "main")
        .unwrap_or(false)
}

fn collect_image_tags() -> Vec<String> {
    publish::collect_sanitized_tags(
        &[
            env::var("STACK_DEMO_APP_TAGS").ok(),
            env::var("STACK_VERSION").ok(),
            env::var("GITHUB_SHA")
                .ok()
                .map(|sha| sha.chars().take(7).collect::<String>())
                .map(|sha| format!("sha-{sha}")),
        ],
        true,
    )
}

fn should_export_artifact() -> bool {
    matches!(
        env::var("STACK_DEMO_APP_EXPORT_ARTIFACT").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}
