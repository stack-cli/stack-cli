use anyhow::{Context, Result};
use dagger_sdk::{File, Query};

const OPERATOR_ARTIFACT_PATH: &str = "artifacts/operator/stack-operator.tar";

pub async fn build_container(client: &Query, operator_binary: &File) -> Result<()> {
    let operator_container = client
        .container()
        .with_file("/stack-linux", operator_binary.clone())
        .with_entrypoint(vec!["/stack-linux".to_string(), "operator".to_string()]);

    operator_container
        .export(OPERATOR_ARTIFACT_PATH)
        .await
        .context("failed to export stack operator container")?;

    Ok(())
}
