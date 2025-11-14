mod args;
mod cli;
mod operator;

use anyhow::{Context, Result};
use args::{Args, CliTarget, Command};
use clap::Parser;
use eyre::Report;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    dagger_sdk::connect(|client| async move {
        run(client, args)
            .await
            .map_err(|err| Report::msg(err.to_string()))
    })
    .await
    .context("failed to connect to dagger engine")?;

    Ok(())
}

async fn run(client: dagger_sdk::Query, args: Args) -> Result<()> {
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
