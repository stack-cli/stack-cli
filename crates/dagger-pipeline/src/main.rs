mod args;
mod cli;

use anyhow::{Context, Result};
use args::{Args, CliTarget, Command};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = dagger_sdk::connect()
        .await
        .context("failed to connect to dagger engine")?;
    let repo = client.host().directory(".");

    match args.command {
        Command::Build { target } => {
            if let Some(target) = target {
                cli::build_cli(&client, &repo, target).await?;
            } else {
                for target in CliTarget::all() {
                    cli::build_cli(&client, &repo, target).await?;
                }
            }
        }
    }

    Ok(())
}
