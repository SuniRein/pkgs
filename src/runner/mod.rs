mod error;
mod rw;

mod load;
mod unload;

pub use error::{LoadError, RunnerError, UnloadError};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::logger::{LogMessage, Logger, LoggerOutput};

pub struct Runner<O: LoggerOutput> {
    cwd: PathBuf,
    logger: Logger<O>,
}

impl<O: LoggerOutput> Runner<O> {
    pub fn new(cwd: &Path, output: O) -> Self {
        Self {
            cwd: cwd.to_path_buf(),
            logger: Logger::new(output),
        }
    }

    pub fn messages(&self) -> &[LogMessage] {
        self.logger.messages()
    }

    pub fn create_dir(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&path)?;
        self.logger.create_dir(path);
        Ok(())
    }

    pub fn create_symlink(
        &mut self,
        src: impl AsRef<Path>,
        dst: impl AsRef<Path>,
    ) -> io::Result<()> {
        crate::fs::create_symlink(&src, &dst)?;
        self.logger.create_symlink(src, dst);
        Ok(())
    }

    pub fn remove_symlink(
        &mut self,
        src: impl AsRef<Path>,
        dst: impl AsRef<Path>,
    ) -> io::Result<()> {
        fs::remove_file(&dst)?;
        self.logger.remove_symlink(src, dst);
        Ok(())
    }
}
