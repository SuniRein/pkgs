use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("module '{0}' not found")]
    ModuleNotFound(String),
}
