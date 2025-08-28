use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::logger::LoggerOutput;
use crate::runner::Runner;
use crate::trace::PkgTrace;

#[derive(Debug, Error)]
pub enum UnloadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("'{dst}' for '{src}' does not exist")]
    DstNotFound { src: String, dst: PathBuf },

    #[error("'{dst}' for '{src}' found in trace file but not a symlink")]
    DstNotSymlink { src: String, dst: PathBuf },
}

pub fn unload<O: LoggerOutput>(
    root: &Path,
    trace: &PkgTrace,
    runner: &mut Runner<O>,
) -> Result<(), UnloadError> {
    let pkg_dir = root.join(&trace.directory);

    for (src, dst) in &trace.maps {
        let dst_path = PathBuf::from(dst);
        if !dst_path.exists() {
            return Err(UnloadError::DstNotFound {
                src: src.clone(),
                dst: dst_path,
            });
        }
        if !dst_path.is_symlink() {
            return Err(UnloadError::DstNotSymlink {
                src: src.clone(),
                dst: dst_path,
            });
        }

        let src_path = pkg_dir.join(src);
        runner.remove_symlink(src_path, dst_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use googletest::prelude::*;

    use super::*;
    use crate::config::{Package, PackageType};
    use crate::core::{NamedPackage, load};
    use crate::logger::LogMessage;
    use crate::test_utils::{TempDir, null_runner};

    const SRC_FILE_PATH: &str = "test_package/src_file";
    const SRC_DIR_PATH: &str = "test_package/src_dir";

    const DST_FILE_PATH: &str = "./test_pkg/dst_file";
    const DST_DIR_PATH: &str = "./test_a/test_b/dst_dir";

    fn setup() -> Result<(TempDir, PkgTrace)> {
        let td = TempDir::new()?
            .dir(SRC_DIR_PATH)?
            .file(SRC_FILE_PATH, "test_content")?;

        let dst_file_path = td.join(DST_FILE_PATH).to_str().unwrap().to_string();
        let dst_dir_path = td.join(DST_DIR_PATH).to_str().unwrap().to_string();

        let pkg = NamedPackage::new(
            "test_package",
            Package {
                kind: PackageType::Local,
                maps: HashMap::from([
                    ("src_file".into(), dst_file_path),
                    ("src_dir".into(), dst_dir_path),
                ]),
            },
        );

        let trace = load(td.path(), &pkg, None, &mut null_runner())?;
        Ok((td, trace))
    }

    #[gtest]
    fn it_works() -> Result<()> {
        let (td, trace) = setup()?;
        let mut runner = null_runner();
        unload(td.path(), &trace, &mut runner)?;

        expect_pred!(!td.join(DST_FILE_PATH).exists());
        expect_pred!(!td.join(DST_DIR_PATH).exists());

        let messages = runner.messages();
        expect_eq!(messages.len(), 2);
        expect_that!(
            messages,
            contains(pat!(LogMessage::RemoveSymlink {
                src: &td.join(SRC_DIR_PATH),
                dst: &td.join(DST_DIR_PATH),
            }))
        );
        expect_that!(
            messages,
            contains(pat!(LogMessage::RemoveSymlink {
                src: &td.join(SRC_FILE_PATH),
                dst: &td.join(DST_FILE_PATH),
            }))
        );

        Ok(())
    }

    #[gtest]
    fn dst_not_exists() -> Result<()> {
        let (td, trace) = setup()?;
        fs::remove_file(td.join(DST_FILE_PATH))?;

        let err = unload(td.path(), &trace, &mut null_runner()).unwrap_err();
        expect_that!(
            err,
            pat!(UnloadError::DstNotFound {
                src: "src_file",
                dst: &td.join(DST_FILE_PATH)
            })
        );

        Ok(())
    }

    #[gtest]
    fn dst_is_not_symlink() -> Result<()> {
        let (td, trace) = setup()?;
        fs::remove_file(td.join(DST_FILE_PATH))?;
        fs::write(td.join(DST_FILE_PATH), "not_a_symlink")?;

        let err = unload(td.path(), &trace, &mut null_runner()).unwrap_err();
        expect_that!(
            err,
            pat!(UnloadError::DstNotSymlink {
                src: "src_file",
                dst: &td.join(DST_FILE_PATH)
            })
        );

        Ok(())
    }
}
