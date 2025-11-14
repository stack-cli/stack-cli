use anyhow::{anyhow, Context, Result};
use dagger_sdk::{Container, File, Query, SecretId};
use std::{collections::HashSet, env};

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
    let credentials = load_credentials(client).await?;

    if credentials.is_none() && require_publish {
        return Err(anyhow!(
            "publishing operator images requires GHCR credentials (`GHCR_USERNAME`/`GITHUB_ACTOR` and `GHCR_TOKEN`/`GITHUB_TOKEN`)"
        ));
    }

    println!(
        "Publishing operator images to {registry}/{repository} with tags: {}",
        tags.join(", ")
    );

    for tag in tags {
        let address = format!("{registry}/{repository}:{tag}");
        let publish_container = if let Some(creds) = &credentials {
            container.clone().with_registry_auth(
                address.clone(),
                creds.username.clone(),
                creds.secret_id.clone(),
            )
        } else {
            container.clone()
        };

        let digest = publish_container
            .publish(address.clone())
            .await
            .with_context(|| format!("failed to publish operator image {address}"))?;
        println!("Published operator image: {digest}");
    }

    Ok(())
}

async fn load_credentials(client: &Query) -> Result<Option<PublishCredentials>> {
    let username = env::var("GHCR_USERNAME").or_else(|_| env::var("GITHUB_ACTOR"));
    let token = env::var("GHCR_TOKEN").or_else(|_| env::var("GITHUB_TOKEN"));

    match (username, token) {
        (Ok(username), Ok(token)) => {
            let secret = client.set_secret("ghcr-token", token);
            let secret_id = secret
                .id()
                .await
                .context("failed to register GHCR token as Dagger secret")?;
            println!("Using GHCR username `{username}` for image publication");
            Ok(Some(PublishCredentials {
                username,
                secret_id,
            }))
        }
        (Err(user_err), Ok(_)) => {
            println!("GHCR username not provided (`GHCR_USERNAME` / `GITHUB_ACTOR`): {user_err}");
            Ok(None)
        }
        (Ok(_), Err(token_err)) => {
            println!("GHCR token not provided (`GHCR_TOKEN` / `GITHUB_TOKEN`): {token_err}");
            Ok(None)
        }
        (Err(user_err), Err(token_err)) => {
            println!("GHCR username not provided (`GHCR_USERNAME` / `GITHUB_ACTOR`): {user_err}");
            println!("GHCR token not provided (`GHCR_TOKEN` / `GITHUB_TOKEN`): {token_err}");
            Ok(None)
        }
    }
}

fn collect_image_tags() -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tags = Vec::new();

    for candidate in [
        env::var("STACK_VERSION").ok(),
        env::var("GITHUB_REF_NAME").ok(),
        env::var("GITHUB_SHA")
            .ok()
            .map(|sha| sha.chars().take(7).collect::<String>())
            .map(|sha| format!("sha-{sha}")),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(tag) = sanitize_tag(&candidate) {
            if seen.insert(tag.clone()) {
                tags.push(tag);
            }
        }
    }

    tags
}

fn sanitize_tag(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut sanitized = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        match ch {
            'a'..='z' | '0'..='9' | '.' | '-' | '_' => sanitized.push(ch),
            'A'..='Z' => sanitized.push(ch.to_ascii_lowercase()),
            _ => sanitized.push('-'),
        }
    }

    let sanitized = sanitized.trim_matches(['-', '_', '.']).to_string();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

struct PublishCredentials {
    username: String,
    secret_id: SecretId,
}
