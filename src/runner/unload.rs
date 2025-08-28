use std::path::PathBuf;

use super::{Runner, RunnerError, UnloadError};
use crate::logger::LoggerOutput;
use crate::trace::PkgTrace;

impl<O: LoggerOutput> Runner<O> {
    pub fn unload_module(&mut self, name: &str, trace: &PkgTrace) -> Result<(), RunnerError> {
        self.logger.unload_module(name);
        self.unload_module_inner(trace)
            .map_err(|e| RunnerError::UnloadModuleError {
                source: e,
                module: name.to_string(),
            })
    }

    fn unload_module_inner(&mut self, trace: &PkgTrace) -> Result<(), UnloadError> {
        let pkg_dir = self.cwd.join(&trace.directory);

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
            self.remove_symlink(src_path, dst_path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use googletest::prelude::*;

    use super::*;
    use crate::config::{Package, PackageType};
    use crate::core::NamedPackage;
    use crate::logger::{LogMessage, NullOutput};
    use crate::test_utils::{TempDir, common_runner};

    const SRC_FILE_PATH: &str = "test_package/src_file";
    const SRC_DIR_PATH: &str = "test_package/src_dir";

    const DST_FILE_PATH: &str = "./test_pkg/dst_file";
    const DST_DIR_PATH: &str = "./test_a/test_b/dst_dir";

    fn setup() -> Result<(TempDir, PkgTrace, Runner<NullOutput>)> {
        let td = TempDir::new()?
            .dir(SRC_DIR_PATH)?
            .file(SRC_FILE_PATH, "test_content")?;

        let dst_file_path = td.join(DST_FILE_PATH).to_str().unwrap().to_string();
        let dst_dir_path = td.join(DST_DIR_PATH).to_str().unwrap().to_string();

        let pkg = NamedPackage::new(
            "test_package",
            Package {
                kind: PackageType::Local,
                maps: BTreeMap::from([
                    ("src_file".into(), dst_file_path),
                    ("src_dir".into(), dst_dir_path),
                ]),
            },
        );

        let trace = (common_runner(td.path())).load_module(&pkg, None)?;

        let runner = common_runner(td.path());

        Ok((td, trace, runner))
    }

    #[gtest]
    fn it_works() -> Result<()> {
        let (td, trace, mut runner) = setup()?;
        runner.unload_module("test_package", &trace)?;

        expect_pred!(!td.join(DST_FILE_PATH).exists());
        expect_pred!(!td.join(DST_DIR_PATH).exists());

        let messages = runner.messages();
        expect_eq!(messages.len(), 3);
        expect_that!(messages[0], pat!(LogMessage::UnloadModule("test_package")));
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
        let (td, trace, mut runner) = setup()?;
        fs::remove_file(td.join(DST_FILE_PATH))?;

        let err = runner
            .unload_module("test_package", &trace)
            .unwrap_err()
            .unwrap_unload();
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
        let (td, trace, mut runner) = setup()?;
        fs::remove_file(td.join(DST_FILE_PATH))?;
        fs::write(td.join(DST_FILE_PATH), "not_a_symlink")?;

        let err = runner
            .unload_module("test_package", &trace)
            .unwrap_err()
            .unwrap_unload();
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
