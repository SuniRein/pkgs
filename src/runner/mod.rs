mod error;
mod rw;

mod load;
mod unload;

pub use error::{IoError, LoadError, RunnerError, UnloadError};

use std::fs;
use std::path::{Path, PathBuf};

use crate::logger::{LogMessage, Logger, LoggerOutput};

pub struct Runner<O: LoggerOutput> {
    cwd: PathBuf,
    logger: Logger<O>,
}

impl<O: LoggerOutput> Runner<O> {
    pub fn new(cwd: &Path, output: O) -> Self {
        if !cwd.is_absolute() {
            panic!("cwd must be an absolute path");
        }

        Self {
            cwd: cwd.to_path_buf(),
            logger: Logger::new(output),
        }
    }

    pub fn messages(&self) -> &[LogMessage] {
        self.logger.messages()
    }

    pub fn absolute_path_from(&self, path: impl AsRef<Path>) -> PathBuf {
        if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.cwd.join(path)
        }
    }

    pub fn create_dir(&mut self, path: impl AsRef<Path>) -> Result<(), IoError> {
        fs::create_dir_all(&path).map_err(|source| IoError {
            source,
            action: format!("create dir '{}'", path.as_ref().display()),
        })?;
        self.logger.create_dir(path);
        Ok(())
    }

    pub fn create_symlink(
        &mut self,
        src: impl AsRef<Path>,
        dst: impl AsRef<Path>,
    ) -> Result<(), IoError> {
        crate::fs::create_symlink(&src, &dst).map_err(|source| IoError {
            source,
            action: format!(
                "create symlink '{}' for '{}'",
                dst.as_ref().display(),
                src.as_ref().display()
            ),
        })?;
        self.logger.create_symlink(src, dst);
        Ok(())
    }

    pub fn remove_symlink(
        &mut self,
        src: impl AsRef<Path>,
        dst: impl AsRef<Path>,
    ) -> Result<(), IoError> {
        fs::remove_file(&dst).map_err(|source| IoError {
            source,
            action: format!(
                "remove symlink '{}' for '{}'",
                dst.as_ref().display(),
                src.as_ref().display()
            ),
        })?;
        self.logger.remove_symlink(src, dst);
        Ok(())
    }
}
