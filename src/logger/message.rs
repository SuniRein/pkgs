use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogMessage {
    LoadModule(String),
    UnloadModule(String),

    RollbackLoadModule(String),
    RollbackUnloadModule(String),

    CreateDir(PathBuf),
    CreateSymlink { src: PathBuf, dst: PathBuf },

    RemoveDir(PathBuf),
    RemoveSymlink { src: PathBuf, dst: PathBuf },
}
