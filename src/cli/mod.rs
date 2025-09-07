mod error;
mod modules;

use clap::{Parser, Subcommand};

use modules::Modules;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Load modules
    Load {
        /// The modules to load
        #[command(flatten)]
        modules: Modules,
    },

    /// Unload modules
    Unload {
        /// The modules to unload
        #[command(flatten)]
        modules: Modules,
    },

    /// List available modules
    List {
        /// List one module per line
        #[arg(short('1'), long)]
        oneline: bool,
    },

    /// Generate json schema for configuration file
    Schema,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::prelude::*;

    #[gtest]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
