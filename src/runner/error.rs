use std::io;

use thiserror::Error;

use crate::config::ConfigError;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("No configuration file found in the current directory.")]
    ConfigNotFound,

    #[error(transparent)]
    ConfigReadError(#[from] ConfigError),

    #[error("'.pkgs' directory already exists but is not a directory.")]
    PkgsDirNotADir,

    #[error("Package directory '.pkgs' not found.")]
    PkgsDirNotFound,

    #[error("Io error while {action}: {source}")]
    Io {
        source: io::Error,
        action: &'static str,
    },
}
