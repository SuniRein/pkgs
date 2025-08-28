use std::fs;
use std::path::{Path, PathBuf};

use googletest::Result;

use crate::fs::create_symlink;

pub struct TempDir(pub tempfile::TempDir);

impl TempDir {
    pub fn new() -> Result<Self> {
        Ok(TempDir(tempfile::tempdir()?))
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    pub fn join(&self, subpath: impl AsRef<Path>) -> PathBuf {
        self.path().join(subpath)
    }

    pub fn file(self, path: impl AsRef<str>, content: impl AsRef<str>) -> Result<Self> {
        fs::write(self.join(path.as_ref()), content.as_ref())?;
        Ok(self)
    }

    pub fn dir(self, path: impl AsRef<str>) -> Result<Self> {
        fs::create_dir_all(self.join(path.as_ref()))?;
        Ok(self)
    }

    pub fn symlink(self, dst: impl AsRef<str>, src: impl AsRef<str>) -> Result<Self> {
        create_symlink(self.join(src.as_ref()), self.join(dst.as_ref()))?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;
    use crate::test_utils::matchers::is_symlink_for;

    #[gtest]
    fn it_works() -> Result<()> {
        let td = TempDir::new()?
            .dir("b/c")?
            .file("b/c/d", "a file")?
            .symlink("d", "b/c")?;

        expect_pred!(td.join("b/c/d").is_file());
        expect_eq!(fs::read_to_string(td.join("b/c/d"))?, "a file");

        expect_pred!(td.join("b/c").is_dir());

        expect_that!(td.join("d"), is_symlink_for(td.join("b/c")));

        Ok(())
    }
}
