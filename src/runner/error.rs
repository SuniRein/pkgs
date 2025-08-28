use thiserror::Error;

use crate::config::ConfigError;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("No configuration file found in the current directory.")]
    ConfigNotFound,

    #[error(transparent)]
    ConfigReadError(#[from] ConfigError),
}
