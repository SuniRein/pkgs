use super::{Runner, RunnerError};
use crate::logger::{LogMessage, LoggerOutput};

impl<O: LoggerOutput> Runner<O> {
    pub fn rollback(&mut self) -> Result<(), RunnerError> {
        let Some(actions) = self.last_action() else {
            return Err(RunnerError::NoActionToRollback);
        };

        let (head, actions) = actions.split_first().unwrap();
        match head {
            LogMessage::LoadModule(module) => self.logger.rollback_load_module(module),
            LogMessage::UnloadModule(module) => self.logger.rollback_unload_module(module),
            _ => unreachable!(),
        }

        for action in actions.iter().rev() {
            match action {
                LogMessage::LoadModule(_)
                | LogMessage::UnloadModule(_)
                | LogMessage::RollbackLoadModule(_)
                | LogMessage::RollbackUnloadModule(_) => unreachable!(),

                LogMessage::CreateDir(path) => self.remove_dir(path)?,
                LogMessage::CreateSymlink { src, dst } => self.remove_symlink(src, dst)?,

                LogMessage::RemoveDir(path) => self.create_dir(path)?,
                LogMessage::RemoveSymlink { src, dst } => self.create_symlink(src, dst)?,
            }
        }

        Ok(())
    }

    fn last_action(&self) -> Option<Vec<LogMessage>> {
        let msgs = self.messages();
        for i in (0..msgs.len()).rev() {
            match &msgs[i] {
                LogMessage::LoadModule(_) | LogMessage::UnloadModule(_) => {
                    return Some(msgs[i..].to_vec());
                }
                LogMessage::RollbackLoadModule(_) | LogMessage::RollbackUnloadModule(_) => {
                    return None;
                }
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::test_utils::prelude::*;

    #[gtest]
    fn nothing_to_rollback() -> Result<()> {
        let (_td, _pkg, mut runner) = common_local_pkg()?;
        let err = runner.rollback().unwrap_err();
        expect_that!(err, pat!(RunnerError::NoActionToRollback));
        Ok(())
    }

    #[gtest]
    fn rollback_twice() -> Result<()> {
        let (_td, pkg, mut runner) = common_local_pkg()?;
        runner.load_module(&pkg, None)?;
        runner.rollback()?;

        let err = runner.rollback().unwrap_err();
        expect_that!(err, pat!(RunnerError::NoActionToRollback));

        Ok(())
    }

    mod rollback_load_module {
        use super::*;

        #[gtest]
        fn after_success() -> Result<()> {
            let (td, pkg, mut runner) = common_local_pkg()?;

            runner.load_module(&pkg, None)?;
            let msgs = runner.messages()[1..].to_vec();
            let rollback_begin = runner.messages().len();

            runner.rollback()?;
            let rollback_msgs = runner.messages()[rollback_begin..].to_vec();

            expect_that!(
                rollback_msgs[0],
                pat!(LogMessage::RollbackLoadModule("test_package"))
            );
            expect_eq!(rollback_msgs.len(), msgs.len() + 1);

            expect_pred!(!td.join(DST_DIR_PATH).exists());
            expect_pred!(!td.join(DST_FILE_PATH).exists());

            expect_that!(
                rollback_msgs,
                superset_of([
                    &LogMessage::RemoveSymlink {
                        src: td.join(SRC_FILE_PATH).canonicalize()?,
                        dst: td.join(DST_FILE_PATH)
                    },
                    &LogMessage::RemoveDir(td.join("./test_pkg")),
                ])
            );
            expect_that!(
                rollback_msgs,
                superset_of([
                    &LogMessage::RemoveSymlink {
                        src: td.join(SRC_DIR_PATH).canonicalize()?,
                        dst: td.join(DST_DIR_PATH)
                    },
                    &LogMessage::RemoveDir(td.join("./test_a/test_b")),
                ])
            );

            Ok(())
        }

        #[gtest]
        fn after_failure() -> Result<()> {
            let (td, pkg, mut runner) = common_local_pkg()?;
            fs::remove_file(td.join(SRC_FILE_PATH))?;

            let _ = runner.load_module(&pkg, None).unwrap_err();
            let msgs = runner.messages()[1..].to_vec();
            let rollback_begin = runner.messages().len();
            let load_src_dir = td.join(DST_DIR_PATH).exists();

            runner.rollback()?;
            let rollback_msgs = runner.messages()[rollback_begin..].to_vec();

            expect_that!(
                rollback_msgs[0],
                pat!(LogMessage::RollbackLoadModule("test_package"))
            );
            expect_eq!(rollback_msgs.len(), msgs.len() + 1);

            expect_pred!(!td.join(DST_DIR_PATH).exists());
            expect_pred!(!td.join(DST_FILE_PATH).exists());

            if load_src_dir {
                expect_that!(
                    rollback_msgs,
                    superset_of([
                        &LogMessage::RemoveSymlink {
                            src: td.join(SRC_DIR_PATH).canonicalize()?,
                            dst: td.join(DST_DIR_PATH)
                        },
                        &LogMessage::RemoveDir(td.join("./test_a/test_b")),
                    ])
                );
            }

            Ok(())
        }

        #[gtest]
        fn only_rollback_last_loading() -> Result<()> {
            let (td, mut pkg, mut runner) = common_local_pkg()?;
            let trace = runner.load_module(&pkg, None)?;

            let new_src_file = "test_package/new_src_file";
            let td = td.file(new_src_file, "")?;
            pkg.package.maps.insert(
                String::from("new_src_file"),
                td.join("new_dst_file").to_string_lossy().into(),
            );

            let mut runner = common_runner(td.path());
            runner.load_module(&pkg, Some(&trace))?;

            let msgs = runner.messages()[1..].to_vec();
            let rollback_begin = runner.messages().len();

            runner.rollback()?;
            let rollback_msgs = runner.messages()[rollback_begin..].to_vec();

            expect_that!(
                rollback_msgs[0],
                pat!(LogMessage::RollbackLoadModule("test_package"))
            );
            expect_eq!(rollback_msgs.len(), msgs.len() + 1);

            expect_pred!(td.join(DST_DIR_PATH).exists());
            expect_pred!(td.join(DST_FILE_PATH).exists());
            expect_pred!(!td.join("new_dst_file").exists());

            expect_that!(
                rollback_msgs,
                superset_of([&LogMessage::RemoveSymlink {
                    src: td.join(new_src_file).canonicalize()?,
                    dst: td.join("new_dst_file")
                },])
            );

            Ok(())
        }
    }

    mod rollback_unload_module {
        use super::*;

        #[gtest]
        fn after_success() -> Result<()> {
            let (td, pkg, mut runner) = common_local_pkg()?;
            let trace = runner.load_module(&pkg, None)?;

            let mut runner = common_runner(td.path());
            runner.unload_module("test_package", &trace)?;

            let msgs = runner.messages()[1..].to_vec();
            let rollback_begin = runner.messages().len();

            runner.rollback()?;
            let rollback_msgs = runner.messages()[rollback_begin..].to_vec();

            expect_that!(
                rollback_msgs[0],
                pat!(LogMessage::RollbackUnloadModule("test_package"))
            );
            expect_eq!(rollback_msgs.len(), msgs.len() + 1);

            expect_pred!(td.join(DST_DIR_PATH).exists());
            expect_pred!(td.join(DST_FILE_PATH).exists());

            expect_that!(
                rollback_msgs,
                contains(pat!(LogMessage::CreateSymlink {
                    src: &td.join(SRC_DIR_PATH),
                    dst: &td.join(DST_DIR_PATH),
                }))
            );
            expect_that!(
                rollback_msgs,
                contains(pat!(LogMessage::CreateSymlink {
                    src: &td.join(SRC_FILE_PATH),
                    dst: &td.join(DST_FILE_PATH),
                }))
            );

            Ok(())
        }

        #[gtest]
        fn after_failure() -> Result<()> {
            let (td, pkg, mut runner) = common_local_pkg()?;
            let trace = runner.load_module(&pkg, None)?;
            fs::remove_file(td.join(DST_FILE_PATH))?;

            let mut runner = common_runner(td.path());
            let _ = runner.unload_module("test_package", &trace).unwrap_err();

            let msgs = runner.messages()[1..].to_vec();
            let rollback_begin = runner.messages().len();
            let unload_src_dir = runner.messages().contains(&LogMessage::RemoveSymlink {
                src: td.join(SRC_DIR_PATH),
                dst: td.join(DST_DIR_PATH),
            });

            runner.rollback()?;
            let rollback_msgs = runner.messages()[rollback_begin..].to_vec();

            expect_that!(
                rollback_msgs[0],
                pat!(LogMessage::RollbackUnloadModule("test_package"))
            );
            expect_eq!(rollback_msgs.len(), msgs.len() + 1);

            expect_pred!(td.join(DST_DIR_PATH).exists());

            if unload_src_dir {
                expect_that!(
                    rollback_msgs,
                    contains(pat!(LogMessage::CreateSymlink {
                        src: &td.join(SRC_DIR_PATH),
                        dst: &td.join(DST_DIR_PATH),
                    }))
                );
            }

            Ok(())
        }
    }
}
