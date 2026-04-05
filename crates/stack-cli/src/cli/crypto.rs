use crate::crypto;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct CryptoArgs {
    #[command(subcommand)]
    pub command: CryptoCommands,
}

#[derive(Subcommand)]
pub enum CryptoCommands {
    /// Encrypt a plaintext value using the age public key in STACK_CLI_AGE_PUBLIC_KEY
    Encrypt {
        /// Plaintext value to encrypt
        #[arg(long)]
        value: String,
    },
    /// Decrypt an ENC[AGE,...] value using STACK_CLI_AGE_SECRET_KEY
    Decrypt {
        /// Encrypted value in ENC[AGE,...] format
        #[arg(long)]
        value: String,
    },
    /// Generate an age x25519 key pair
    GenerateKey {
        /// Optional passphrase to protect the generated private key
        #[arg(long)]
        password: Option<String>,
    },
}

pub async fn crypto(args: &CryptoArgs) -> Result<()> {
    match &args.command {
        CryptoCommands::Encrypt { value } => {
            let public_key = std::env::var("STACK_CLI_AGE_PUBLIC_KEY")
                .context("STACK_CLI_AGE_PUBLIC_KEY is required")?;
            let encrypted = crypto::encrypt_with_public_key(value, &public_key)?;
            println!("{}", encrypted);
        }
        CryptoCommands::Decrypt { value } => {
            let secret_key = std::env::var("STACK_CLI_AGE_SECRET_KEY")
                .context("STACK_CLI_AGE_SECRET_KEY is required")?;
            let decrypted = crypto::decrypt_with_secret_key(value, &secret_key)?;
            println!("{}", decrypted);
        }
        CryptoCommands::GenerateKey { password } => {
            let (public_key, private_key) = crypto::generate_key_pair(password.as_deref())?;
            println!("STACK_CLI_AGE_PUBLIC_KEY={}", public_key);
            println!("STACK_CLI_AGE_SECRET_KEY={}", private_key);
            if password.is_some() {
                println!("# also set STACK_CLI_AGE_KEY_PASSWORD to the same passphrase");
            }
        }
    }

    Ok(())
}
