use anyhow::{Context, Result};
use dagger_sdk::{Container, Query, SecretId};
use std::collections::HashSet;

pub struct PublishCredentials {
    pub username: String,
    pub secret_id: SecretId,
}

pub async fn load_credentials(
    client: &Query,
    username_envs: &[&str],
    token_envs: &[&str],
    context_label: &str,
) -> Result<Option<PublishCredentials>> {
    let username = first_env(username_envs);
    let token = first_env(token_envs);

    match (username, token) {
        (Some(username), Some(token)) => {
            let secret = client.set_secret("ghcr-token", token);
            let secret_id = secret
                .id()
                .await
                .context("failed to register GHCR token as Dagger secret")?;
            println!("Using GHCR username `{username}` for {context_label} publication");
            Ok(Some(PublishCredentials {
                username,
                secret_id,
            }))
        }
        (None, Some(_)) => {
            println!(
                "GHCR username not provided (tried: {} )",
                username_envs.join(", ")
            );
            Ok(None)
        }
        (Some(_), None) => {
            println!(
                "GHCR token not provided (tried: {} )",
                token_envs.join(", ")
            );
            Ok(None)
        }
        (None, None) => {
            println!(
                "GHCR username not provided (tried: {} )",
                username_envs.join(", ")
            );
            println!(
                "GHCR token not provided (tried: {} )",
                token_envs.join(", ")
            );
            Ok(None)
        }
    }
}

pub async fn publish_container_tags(
    container: &Container,
    registry: &str,
    repository: &str,
    tags: &[String],
    credentials: Option<&PublishCredentials>,
    image_label: &str,
) -> Result<()> {
    println!(
        "Publishing {image_label} images to {registry}/{repository} with tags: {}",
        tags.join(", ")
    );

    for tag in tags {
        let address = format!("{registry}/{repository}:{tag}");
        let publish_container = if let Some(creds) = credentials {
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
            .with_context(|| format!("failed to publish {image_label} image {address}"))?;
        println!("Published {image_label} image: {digest}");
    }

    Ok(())
}

pub fn collect_sanitized_tags(candidates: &[Option<String>], split_commas: bool) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tags = Vec::new();

    for candidate in candidates.iter().flatten() {
        if split_commas {
            for raw in candidate.split(',') {
                if let Some(tag) = sanitize_tag(raw) {
                    if seen.insert(tag.clone()) {
                        tags.push(tag);
                    }
                }
            }
        } else if let Some(tag) = sanitize_tag(candidate) {
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

fn first_env(keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Ok(value) = std::env::var(key) {
            return Some(value);
        }
    }
    None
}
