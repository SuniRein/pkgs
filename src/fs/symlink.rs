use std::io;
use std::path::Path;

pub fn create_symlink<S: AsRef<Path>, D: AsRef<Path>>(src: S, dst: D) -> io::Result<()> {
    if !src.as_ref().exists() {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source path '{}' does not exist", src.as_ref().display()),
        ))?
    }

    let src = src.as_ref().canonicalize().unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(src, dst)
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::{symlink_dir, symlink_file};

        if src.is_dir() {
            symlink_dir(src, dst)
        } else {
            symlink_file(src, dst)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::{self, Write};

    use super::*;
    use crate::test_utils::prelude::*;

    #[gtest]
    fn create_symlink_file_unix() -> Result<()> {
        let td = TempDir::new()?;
        let src = td.join("src_file");
        let mut f = File::create(&src)?;
        writeln!(f, "hello")?;
        let dst = td.join("dst_file_link");

        create_symlink(&src, &dst)?;
        expect_true!(dst.exists());
        expect_true!(dst.is_symlink());

        let target = fs::read_link(&dst)?;
        assert_eq!(target.canonicalize()?, src.canonicalize()?);
        Ok(())
    }

    #[gtest]
    fn create_symlink_dir_unix() -> Result<()> {
        let td = TempDir::new()?;
        let src = td.join("src_dir");
        fs::create_dir(&src)?;
        let dst = td.join("dst_dir_link");

        create_symlink(&src, &dst)?;
        expect_true!(dst.exists());
        expect_true!(dst.is_symlink());

        let target = fs::read_link(&dst)?;
        assert_eq!(target.canonicalize()?, src.canonicalize()?);
        Ok(())
    }

    #[gtest]
    fn create_symlink_nonexistent_src_unix() -> Result<()> {
        let td = TempDir::new()?;
        let src = td.join("no_such_src");
        let dst = td.join("dst_nonexistent_link");

        let result = create_symlink(&src, &dst).unwrap_err();
        expect_eq!(result.kind(), io::ErrorKind::NotFound);
        Ok(())
    }
}
