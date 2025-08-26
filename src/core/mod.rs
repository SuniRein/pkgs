mod load;
mod unload;

mod named_package;
pub mod utils;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Failed to create symlink: {0}")]
    SymlinkCreationError(String),
}

pub use load::load;
pub use unload::unload;

pub use named_package::NamedPackage;
