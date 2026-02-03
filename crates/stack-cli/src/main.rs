mod cli;
mod error;
mod operator;
mod services;
use anyhow::Result;
use clap::Parser;

const MANAGER: &str = "stack-operator";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match &cli.command {
        cli::Commands::Deploy(deployer) => {
            cli::deploy::deploy(deployer).await?;
        }
        cli::Commands::Init(initializer) => {
            cli::init::init(initializer).await?;
        }
        cli::Commands::Operator(args) => {
            operator::operator(args.once).await?;
        }
        cli::Commands::Status(args) => {
            cli::status::status(args).await?;
        }
        cli::Commands::Secrets(args) => {
            cli::secrets::secrets(args).await?;
        }
        cli::Commands::Cloudflare(args) => {
            cli::cloudflare::cloudflare(args).await?;
        }
    }

    Ok(())
}
