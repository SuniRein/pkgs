use std::io;
use std::path::PathBuf;

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

    #[error("Fail to load {module}: {source}")]
    LoadModuleError { source: LoadError, module: String },
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("source '{0}' does not exist")]
    SrcNotExists(String),

    #[error("'{dst}' for '{src}' already exists")]
    DstAlreadyExists { src: String, dst: PathBuf },

    #[error("destination '{0}' found in trace file but not a symlink")]
    DstNotSymlink(PathBuf),
}

impl RunnerError {
    pub fn unwrap_load(self) -> LoadError {
        match self {
            RunnerError::LoadModuleError { source, .. } => source,
            _ => panic!("Called unwrap_load on a non-LoadModuleError variant"),
        }
    }
}
