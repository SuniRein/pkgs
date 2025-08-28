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
        /// the modules to load
        #[command(flatten)]
        modules: Modules,
    },

    /// Unload modules
    Unload {
        /// the modules to unload
        #[command(flatten)]
        modules: Modules,
    },

    /// List available modules
    List,
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    #[gtest]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
