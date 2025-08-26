use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogMessage {
    LoadModule(String),
    UnloadModule(String),

    CreateDir(PathBuf),
    CreateFile(PathBuf),
    CreateSymlink { src: PathBuf, dst: PathBuf },
    RemoveSymlink { src: PathBuf, dst: PathBuf },
}
