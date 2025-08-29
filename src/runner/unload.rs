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
    use std::fs;

    use super::*;
    use crate::test_utils::prelude::*;

    fn setup() -> Result<(TempDir, PkgTrace, Runner<NullOutput>)> {
        let (td, pkg, mut runner) = common_local_pkg()?;
        let trace = runner.load_module(&pkg, None)?;
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
