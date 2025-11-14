use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about = "Build Stack CLI binaries using Dagger")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build the Stack CLI binaries
    Build {
        /// Specific target to build (defaults to all targets)
        #[arg(value_enum)]
        target: Option<CliTarget>,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliTarget {
    Linux,
    Macos,
    Windows,
}

impl CliTarget {
    pub fn all() -> [Self; 3] {
        [Self::Linux, Self::Macos, Self::Windows]
    }
}
