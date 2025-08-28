use std::path::PathBuf;

use super::{LoadError, Runner, RunnerError};
use crate::core::NamedPackage;
use crate::logger::LoggerOutput;
use crate::trace::PkgTrace;

impl<O: LoggerOutput> Runner<O> {
    pub fn load_module(
        &mut self,
        package: &NamedPackage,
        trace: Option<&PkgTrace>,
    ) -> Result<PkgTrace, RunnerError> {
        self.logger.load_module(&package.name);

        let result = if let Some(trace) = trace {
            self.load_with_trace(package, trace)
        } else {
            self.load_directly(package)
        };

        result.map_err(|e| RunnerError::LoadModuleError {
            source: e,
            module: package.name.clone(),
        })
    }

    fn load_directly(&mut self, package: &NamedPackage) -> Result<PkgTrace, LoadError> {
        let mut trace = PkgTrace::new(package.get_directory());

        let pkg_dir = self.cwd.join(&trace.directory).canonicalize()?;

        for (src, dst) in package.maps() {
            let src_path = pkg_dir.join(src);
            if !src_path.exists() {
                return Err(LoadError::SrcNotExists(src.to_string()));
            }

            let dst_path = PathBuf::from(&dst);
            if dst_path.exists() {
                return Err(LoadError::DstAlreadyExists {
                    src: src.clone(),
                    dst: dst_path,
                });
            }

            if let Some(parent) = dst_path.parent()
                && !parent.exists()
            {
                self.create_dir(parent)?;
            }

            self.create_symlink(&src_path, &dst_path)?;

            trace.maps.insert(src.into(), dst.into());
        }

        Ok(trace)
    }

    fn load_with_trace(
        &mut self,
        package: &NamedPackage,
        old_trace: &PkgTrace,
    ) -> Result<PkgTrace, LoadError> {
        let directory = package.get_directory();
        if directory != old_trace.directory {
            return self.load_with_pkg_dir_changed(package, old_trace);
        }

        let mut trace = PkgTrace::new(directory);

        let pkg_dir = self.cwd.join(&trace.directory).canonicalize()?;

        for (src, dst) in package.maps() {
            let src_path = pkg_dir.join(src);
            if !src_path.exists() {
                return Err(LoadError::SrcNotExists(src.to_string()));
            }

            let dst_path = PathBuf::from(&dst);

            if let Some(dst_in_trace) = old_trace.maps.get(src) {
                let dst_in_trace = PathBuf::from(dst_in_trace);
                if dst_in_trace.exists() {
                    if !dst_in_trace.is_symlink() {
                        return Err(LoadError::DstNotSymlink(dst_in_trace));
                    }

                    if dst_path == dst_in_trace {
                        trace.maps.insert(src.into(), dst.into());
                        continue;
                    }

                    self.remove_symlink(&src_path, dst_in_trace)?;
                }
            }

            if dst_path.exists() {
                return Err(LoadError::DstAlreadyExists {
                    src: src.clone(),
                    dst: dst_path,
                });
            }

            if let Some(parent) = dst_path.parent()
                && !parent.exists()
            {
                self.create_dir(parent)?;
            }

            self.create_symlink(&src_path, dst)?;

            trace.maps.insert(src.into(), dst.into());
        }

        for (src, dst) in &old_trace.maps {
            let dst_path = PathBuf::from(&dst);

            if dst_path.exists() && !trace.maps.contains_key(src) {
                if !dst_path.is_symlink() {
                    return Err(LoadError::DstNotSymlink(dst_path));
                }
                self.remove_symlink(pkg_dir.join(src), dst)?;
            }
        }

        Ok(trace)
    }

    fn load_with_pkg_dir_changed(
        &mut self,
        _package: &NamedPackage,
        _old_trace: &PkgTrace,
    ) -> Result<PkgTrace, LoadError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use googletest::prelude::*;

    use super::*;
    use crate::config::{Package, PackageType};
    use crate::logger::{LogMessage, NullOutput};
    use crate::test_utils::{TempDir, common_runner, matchers::is_symlink_for};

    const SRC_FILE_PATH: &str = "test_package/src_file";
    const SRC_DIR_PATH: &str = "test_package/src_dir";

    const DST_FILE_PATH: &str = "./test_pkg/dst_file";
    const DST_DIR_PATH: &str = "./test_a/test_b/dst_dir";

    fn setup() -> Result<(TempDir, NamedPackage, Runner<NullOutput>)> {
        let td = TempDir::new()?
            .dir(SRC_DIR_PATH)?
            .file(SRC_FILE_PATH, "test_content")?;

        let dst_file_path = td.join(DST_FILE_PATH).to_str().unwrap().to_string();
        let dst_dir_path = td.join(DST_DIR_PATH).to_str().unwrap().to_string();

        let pkgs = NamedPackage::new(
            "test_package",
            Package {
                kind: PackageType::Local,
                maps: BTreeMap::from([
                    ("src_file".into(), dst_file_path),
                    ("src_dir".into(), dst_dir_path),
                ]),
            },
        );

        let runner = common_runner(td.path());

        Ok((td, pkgs, runner))
    }

    mod load_without_trace {
        use super::*;

        #[gtest]
        fn it_works() -> Result<()> {
            let (td, pkg, mut runner) = setup()?;

            let trace = runner.load_module(&pkg, None)?;

            let dst_file = td.join(DST_FILE_PATH);
            let dst_dir = td.join(DST_DIR_PATH);

            expect_that!(
                dst_file,
                is_symlink_for(td.join(SRC_FILE_PATH).canonicalize()?),
                "dst_file should point to the absolute path of src_file"
            );

            expect_that!(
                dst_dir,
                is_symlink_for(td.join(SRC_DIR_PATH).canonicalize()?),
                "dst_dir should point to the absolute path of src_dir"
            );

            expect_eq!(trace.directory, "test_package");
            expect_eq!(trace.maps.len(), 2);
            expect_eq!(
                trace.maps["src_file"],
                td.join(DST_FILE_PATH).to_str().unwrap()
            );
            expect_eq!(
                trace.maps["src_dir"],
                td.join(DST_DIR_PATH).to_str().unwrap()
            );

            Ok(())
        }

        #[gtest]
        fn runner_output() -> Result<()> {
            let (td, pkg, mut runner) = setup()?;
            runner.load_module(&pkg, None)?;

            let messages = runner.messages();
            expect_eq!(messages.len(), 5);
            expect_that!(messages[0], pat!(LogMessage::LoadModule("test_package")));
            expect_that!(
                messages,
                superset_of([
                    &LogMessage::CreateDir(td.join("./test_pkg")),
                    &LogMessage::CreateSymlink {
                        src: td.join(SRC_FILE_PATH).canonicalize()?,
                        dst: td.join(DST_FILE_PATH)
                    }
                ])
            );
            expect_that!(
                messages,
                superset_of([
                    &LogMessage::CreateDir(td.join("./test_a/test_b")),
                    &LogMessage::CreateSymlink {
                        src: td.join(SRC_DIR_PATH).canonicalize()?,
                        dst: td.join(DST_DIR_PATH)
                    }
                ])
            );

            Ok(())
        }

        #[gtest]
        fn src_not_exists() -> Result<()> {
            let (td, pkg, mut runner) = setup()?;
            fs::remove_file(td.join(SRC_FILE_PATH))?;

            let result = runner.load_module(&pkg, None).unwrap_err().unwrap_load();
            expect_that!(result, pat!(LoadError::SrcNotExists("src_file")));

            Ok(())
        }

        #[gtest]
        fn dst_already_exists() -> Result<()> {
            let (td, pkg, mut runner) = setup()?;
            fs::create_dir_all(td.join(DST_FILE_PATH))?;

            let result = runner.load_module(&pkg, None).unwrap_err().unwrap_load();
            expect_that!(
                result,
                pat!(LoadError::DstAlreadyExists {
                    src: "src_file",
                    dst: &td.join(DST_FILE_PATH)
                })
            );

            Ok(())
        }
    }

    mod load_with_trace_without_dir_changed {
        use super::*;

        fn setup() -> Result<(TempDir, NamedPackage, PkgTrace)> {
            let (td, pkg, mut runner) = super::setup()?;
            let trace = runner.load_module(&pkg, None)?;
            Ok((td, pkg, trace))
        }

        #[gtest]
        fn no_changed() -> Result<()> {
            let (td, pkg, trace) = setup()?;
            let mut runner = common_runner(td.path());
            let new_trace = runner.load_module(&pkg, Some(&trace))?;

            expect_eq!(new_trace, trace);
            expect_eq!(runner.messages().len(), 1);
            expect_that!(
                runner.messages()[0],
                pat!(LogMessage::LoadModule("test_package"))
            );

            Ok(())
        }

        #[gtest]
        fn just_update() -> Result<()> {
            let (td, mut pkg, trace) = setup()?;
            pkg.package.maps.insert(
                "src_file".into(),
                td.join("new_dest_file").to_string_lossy().into(),
            );

            let mut runner = common_runner(td.path());
            let new_trace = runner.load_module(&pkg, Some(&trace))?;

            expect_eq!(new_trace.directory, trace.directory);
            expect_eq!(new_trace.maps.len(), trace.maps.len());
            expect_eq!(new_trace.maps["src_dir"], trace.maps["src_dir"]);
            expect_eq!(
                new_trace.maps["src_file"],
                td.join("new_dest_file").to_str().unwrap()
            );

            expect_that!(
                runner.messages(),
                superset_of([
                    &LogMessage::RemoveSymlink {
                        src: td.join(SRC_FILE_PATH).canonicalize()?,
                        dst: td.join(DST_FILE_PATH)
                    },
                    &LogMessage::CreateSymlink {
                        src: td.join(SRC_FILE_PATH).canonicalize()?,
                        dst: td.join("new_dest_file")
                    },
                ])
            );

            Ok(())
        }

        #[gtest]
        fn add_new() -> Result<()> {
            let (td, mut pkg, trace) = setup()?;
            let td = td.file("test_package/new_src_file", "")?;
            let new_dst_path = td.join("nonexistent_parent/new_dest_file");
            pkg.package
                .maps
                .insert("new_src_file".into(), new_dst_path.to_string_lossy().into());

            let mut runner = common_runner(td.path());
            let new_trace = runner.load_module(&pkg, Some(&trace))?;

            expect_eq!(new_trace.directory, trace.directory);
            expect_eq!(new_trace.maps.len(), trace.maps.len() + 1);
            expect_eq!(new_trace.maps["src_dir"], trace.maps["src_dir"]);
            expect_eq!(new_trace.maps["src_file"], trace.maps["src_file"]);
            expect_eq!(
                new_trace.maps["new_src_file"],
                new_dst_path.to_str().unwrap(),
            );

            expect_that!(
                runner.messages(),
                superset_of([&LogMessage::CreateSymlink {
                    src: td.join("test_package/new_src_file"),
                    dst: new_dst_path
                },])
            );

            Ok(())
        }

        #[gtest]
        fn remove_old() -> Result<()> {
            let (td, mut pkg, trace) = setup()?;
            pkg.package.maps.remove("src_file");

            let mut runner = common_runner(td.path());
            let new_trace = runner.load_module(&pkg, Some(&trace))?;

            expect_eq!(new_trace.directory, trace.directory);
            expect_eq!(new_trace.maps.len(), trace.maps.len() - 1);
            expect_eq!(new_trace.maps["src_dir"], trace.maps["src_dir"]);
            expect_eq!(new_trace.maps.get("src_file"), None);

            expect_that!(
                runner.messages(),
                superset_of([&LogMessage::RemoveSymlink {
                    src: td.join(SRC_FILE_PATH),
                    dst: td.join(DST_FILE_PATH)
                },])
            );

            Ok(())
        }

        #[gtest]
        fn remove_old_but_dst_not_a_symlink() -> Result<()> {
            let (td, mut pkg, trace) = setup()?;
            pkg.package.maps.remove("src_file");

            fs::remove_file(td.join(DST_FILE_PATH))?;
            fs::write(td.join(DST_FILE_PATH), "")?;

            let mut runner = common_runner(td.path());
            let err = runner
                .load_module(&pkg, Some(&trace))
                .unwrap_err()
                .unwrap_load();
            expect_that!(err, pat!(LoadError::DstNotSymlink(&td.join(DST_FILE_PATH))));

            Ok(())
        }

        #[gtest]
        fn src_not_exists() -> Result<()> {
            let (td, pkg, trace) = setup()?;
            fs::remove_file(td.join(SRC_FILE_PATH))?;

            let mut runner = common_runner(td.path());
            let err = runner
                .load_module(&pkg, Some(&trace))
                .unwrap_err()
                .unwrap_load();
            expect_that!(err, pat!(LoadError::SrcNotExists("src_file")));

            Ok(())
        }

        #[gtest]
        fn dst_exists_but_not_in_trace() -> Result<()> {
            let (td, mut pkg, trace) = setup()?;
            let td = td
                .file("test_package/new_src_file", "")?
                .file("new_dest_file", "")?;
            pkg.package.maps.insert(
                "new_src_file".into(),
                td.join("new_dest_file").to_string_lossy().into(),
            );

            let mut runner = common_runner(td.path());
            let err = runner
                .load_module(&pkg, Some(&trace))
                .unwrap_err()
                .unwrap_load();
            expect_that!(
                err,
                pat!(LoadError::DstAlreadyExists {
                    src: "new_src_file",
                    dst: &td.join("new_dest_file")
                })
            );

            Ok(())
        }

        #[gtest]
        fn dst_in_trace_but_not_a_symlink() -> Result<()> {
            let (td, pkg, trace) = setup()?;
            fs::remove_file(td.join(DST_FILE_PATH))?;
            fs::write(td.join(DST_FILE_PATH), "")?;

            let mut runner = common_runner(td.path());
            let err = runner
                .load_module(&pkg, Some(&trace))
                .unwrap_err()
                .unwrap_load();
            expect_that!(err, pat!(LoadError::DstNotSymlink(&td.join(DST_FILE_PATH))));

            Ok(())
        }

        #[gtest]
        fn dst_in_trace_but_not_exists() -> Result<()> {
            let (td, pkg, trace) = setup()?;
            fs::remove_file(td.join(DST_FILE_PATH))?;

            let mut runner = common_runner(td.path());
            let new_trace = runner.load_module(&pkg, Some(&trace))?;

            expect_eq!(new_trace, trace);
            expect_that!(
                runner.messages(),
                superset_of([&LogMessage::CreateSymlink {
                    src: td.join(SRC_FILE_PATH),
                    dst: td.join(DST_FILE_PATH)
                }])
            );

            Ok(())
        }
    }
}
