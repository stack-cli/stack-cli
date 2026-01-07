pub mod apply;
pub mod init;
pub mod install;
pub mod secrets;
pub mod status;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub struct Initializer {
    /// Install Keycloak operator and shared realm
    #[arg(long, default_value_t = true)]
    pub install_keycloak: bool,
    /// Namespace for the operator
    #[arg(long, default_value = "stack-system")]
    pub operator_namespace: String,
    /// Skip installing the operator
    #[arg(long, default_value_t = false)]
    pub no_operator: bool,
}

#[derive(Parser)]
pub struct Installer {
    /// Path to a StackApp manifest to apply
    #[arg(long)]
    manifest: PathBuf,
}

#[derive(Parser)]
pub struct OperatorArgs {
    /// Run a single reconciliation tick then exit
    #[arg(long, default_value_t = false)]
    pub once: bool,
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Path to a StackApp manifest to read namespace from
    #[arg(long)]
    pub manifest: PathBuf,
    /// Namespace where the shared Keycloak installation lives
    #[arg(long, default_value = "keycloak")]
    pub keycloak_namespace: String,
}

#[derive(Parser)]
pub struct SecretsArgs {
    /// Path to a StackApp manifest to read namespace from
    #[arg(long)]
    pub manifest: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install an application into Kubernetes
    Install(Installer),
    /// Install the required operators into Kubernetes
    Init(Initializer),
    /// Run the Stack Kubernetes Operator
    Operator(OperatorArgs),
    /// Show platform connection details (Keycloak credentials, Cloudflare URL)
    Status(StatusArgs),
    /// Print namespace secrets as KEY=VALUE lines for .env files
    Secrets(SecretsArgs),
}
