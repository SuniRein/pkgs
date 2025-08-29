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

    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Fail to load {module}: {source}")]
    LoadModuleError { source: LoadError, module: String },

    #[error("Fail to unload {module}: {source}")]
    UnloadModuleError { source: UnloadError, module: String },

    #[error("No action to rollback")]
    NoActionToRollback,
}

#[derive(Debug, Error)]
#[error("Io error while {action}: {source}")]
pub struct IoError {
    pub source: io::Error,
    pub action: String,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("package directory for '{0}' not found")]
    PkgDirNotFound(String),

    #[error("source '{0}' does not exist")]
    SrcNotExists(String),

    #[error("'{dst}' for '{src}' already exists")]
    DstAlreadyExists { src: String, dst: PathBuf },

    #[error("'{dst}' for '{src}' found in trace file but not a symlink")]
    DstNotSymlink { src: String, dst: PathBuf },
}

#[derive(Debug, Error)]
pub enum UnloadError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("'{dst}' for '{src}' does not exist")]
    DstNotFound { src: String, dst: PathBuf },

    #[error("'{dst}' for '{src}' found in trace file but not a symlink")]
    DstNotSymlink { src: String, dst: PathBuf },
}

impl RunnerError {
    pub fn unwrap_load(self) -> LoadError {
        match self {
            RunnerError::LoadModuleError { source, .. } => source,
            _ => panic!("Called unwrap_load on a non-LoadModuleError variant"),
        }
    }

    pub fn unwrap_unload(self) -> UnloadError {
        match self {
            RunnerError::UnloadModuleError { source, .. } => source,
            _ => panic!("Called unwrap_unload on a non-UnloadModuleError variant"),
        }
    }
}
