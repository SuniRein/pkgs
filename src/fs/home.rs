use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    std::env::home_dir().expect("Failed to get home directory")
}
