use std::fs;

use googletest::Result;

use crate::core::utils::create_symlink;

pub enum TempDirChild {
    File(String, String),
    Dir(String),
    Symlink(String, String),
}

pub fn td_file(path: impl AsRef<str>, content: impl AsRef<str>) -> TempDirChild {
    TempDirChild::File(path.as_ref().to_string(), content.as_ref().to_string())
}

pub fn td_dir(path: impl AsRef<str>) -> TempDirChild {
    TempDirChild::Dir(path.as_ref().to_string())
}

pub fn td_symlink(dst: impl AsRef<str>, src: impl AsRef<str>) -> TempDirChild {
    TempDirChild::Symlink(dst.as_ref().to_string(), src.as_ref().to_string())
}

pub struct TempDir(pub tempfile::TempDir);

impl TempDir {
    pub fn join(&self, subpath: impl AsRef<Path>) -> PathBuf {
        self.0.path().join(subpath)
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

pub fn temp_dir(children: &[TempDirChild]) -> Result<TempDir> {
    let td = tempfile::tempdir()?;
    for child in children {
        use TempDirChild::*;
        match child {
            File(path, content) => fs::write(td.path().join(path), content)?,
            Dir(path) => fs::create_dir_all(td.path().join(path))?,
            Symlink(dst, src) => create_symlink(td.path().join(src), td.path().join(dst))?,
        }
    }
    Ok(TempDir(td))
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;
    use crate::test_utils::matchers::is_symlink_for;

    #[gtest]
    fn it_works() -> Result<()> {
        let td = temp_dir(&[
            td_dir("b/c"),
            td_file("b/c/d", "a file"),
            td_symlink("d", "b/c"),
        ])?;

        expect_pred!(td.join("b/c/d").is_file());
        expect_eq!(fs::read_to_string(td.join("b/c/d"))?, "a file");

        expect_pred!(td.join("b/c").is_dir());

        expect_that!(td.join("d"), is_symlink_for(td.join("b/c")));

        Ok(())
    }
}
