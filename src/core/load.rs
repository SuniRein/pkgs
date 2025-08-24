use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::NamedPackage;
use super::utils::create_symlink;
use crate::logger::{Logger, LoggerOutput};
use crate::trace::PkgTrace;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("source '{0}' does not exist")]
    SrcNotExists(String),

    #[error("'{dst}' for '{src}' already exists")]
    DstAlreadyExists { src: String, dst: PathBuf },

    #[error("destination '{0}' found in trace file but not a symlink")]
    DstNotSymlink(PathBuf),
}

pub fn load<O: LoggerOutput>(
    root: &Path,
    package: &NamedPackage,
    trace: Option<&PkgTrace>,
    logger: &mut Logger<O>,
) -> Result<PkgTrace, LoadError> {
    logger.load_module(package.name());

    if let Some(trace) = trace {
        load_with_trace(root, package, trace, logger)
    } else {
        load_directly(root, package, logger)
    }
}

fn load_directly<O: LoggerOutput>(
    root: &Path,
    package: &NamedPackage,
    logger: &mut Logger<O>,
) -> Result<PkgTrace, LoadError> {
    let mut trace = PkgTrace {
        directory: package.get_directory(),
        maps: BTreeMap::new(),
    };

    let pkg_dir = root.join(&trace.directory).canonicalize()?;

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
            fs::create_dir_all(parent)?;
            logger.create_dir(parent);
        }

        create_symlink(&src_path, dst)?;
        logger.create_symlink(&src_path, &dst_path);

        trace.maps.insert(src.into(), dst.into());
    }

    Ok(trace)
}

fn load_with_trace<O: LoggerOutput>(
    root: &Path,
    package: &NamedPackage,
    old_trace: &PkgTrace,
    logger: &mut Logger<O>,
) -> Result<PkgTrace, LoadError> {
    let directory = package.get_directory();
    if directory != old_trace.directory {
        return load_with_pkg_dir_changed(root, package, old_trace, logger);
    }

    let mut trace = PkgTrace {
        directory,
        maps: BTreeMap::new(),
    };

    let pkg_dir = root.join(&trace.directory).canonicalize()?;

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

                fs::remove_file(&dst_in_trace)?;
                logger.remove_symlink(&src_path, dst_in_trace);
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
            fs::create_dir_all(parent)?;
            logger.create_dir(parent);
        }

        create_symlink(&src_path, dst)?;
        logger.create_symlink(&src_path, dst);

        trace.maps.insert(src.into(), dst.into());
    }

    for (src, dst) in &old_trace.maps {
        let dst_path = PathBuf::from(&dst);

        if dst_path.exists() && !trace.maps.contains_key(src) {
            if !dst_path.is_symlink() {
                return Err(LoadError::DstNotSymlink(dst_path));
            }
            fs::remove_file(dst)?;
            logger.remove_symlink(pkg_dir.join(src), dst);
        }
    }

    Ok(trace)
}

fn load_with_pkg_dir_changed<O: LoggerOutput>(
    _root: &Path,
    _package: &NamedPackage,
    _old_trace: &PkgTrace,
    _logger: &mut Logger<O>,
) -> Result<PkgTrace, LoadError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use googletest::prelude::*;

    use super::*;
    use crate::config::{Package, PackageType};
    use crate::logger::{LogMessage, null_logger};
    use crate::test_utils::{TempDir, matchers::is_symlink_for};

    const SRC_FILE_PATH: &str = "test_package/src_file";
    const SRC_DIR_PATH: &str = "test_package/src_dir";

    const DST_FILE_PATH: &str = "./test_pkg/dst_file";
    const DST_DIR_PATH: &str = "./test_a/test_b/dst_dir";

    fn setup_pkg(td: &TempDir) -> NamedPackage {
        let dst_file_path = td.join(DST_FILE_PATH).to_str().unwrap().to_string();
        let dst_dir_path = td.join(DST_DIR_PATH).to_str().unwrap().to_string();

        let package = Package {
            kind: PackageType::Local,
            maps: HashMap::from([
                ("src_file".into(), dst_file_path),
                ("src_dir".into(), dst_dir_path),
            ]),
        };
        NamedPackage::new("test_package", package)
    }

    fn setup_dir() -> Result<TempDir> {
        TempDir::new()?
            .dir(SRC_DIR_PATH)?
            .file(SRC_FILE_PATH, "test_content")
    }

    mod load_without_trace {
        use super::*;

        #[gtest]
        fn it_works() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);

            let trace = load(td.path(), &pkg, None, &mut null_logger())?;

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
        fn logger_output() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            let mut logger = null_logger();
            load(td.path(), &pkg, None, &mut logger)?;

            let messages = logger.messages();
            expect_eq!(messages.len(), 5);
            expect_eq!(messages[0], LogMessage::LoadModule("test_package".into()));
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
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            fs::remove_file(td.join(SRC_FILE_PATH))?;

            let result = load(td.path(), &pkg, None, &mut null_logger()).unwrap_err();
            expect_that!(result, pat!(LoadError::SrcNotExists("src_file")));

            Ok(())
        }

        #[gtest]
        fn dst_already_exists() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            fs::create_dir_all(td.join(DST_FILE_PATH))?;

            let result = load(td.path(), &pkg, None, &mut null_logger()).unwrap_err();
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
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            let trace = load(td.path(), &pkg, None, &mut null_logger())?;
            Ok((td, pkg, trace))
        }

        #[gtest]
        fn no_changed() -> Result<()> {
            let (td, pkg, trace) = setup()?;
            let mut logger = null_logger();
            let new_trace = load(td.path(), &pkg, Some(&trace), &mut logger)?;

            expect_eq!(new_trace, trace);
            expect_eq!(
                logger.messages(),
                &[LogMessage::LoadModule("test_package".to_string())]
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

            let mut logger = null_logger();
            let new_trace = load(td.path(), &pkg, Some(&trace), &mut logger)?;

            expect_eq!(new_trace.directory, trace.directory);
            expect_eq!(new_trace.maps.len(), trace.maps.len());
            expect_eq!(new_trace.maps["src_dir"], trace.maps["src_dir"]);
            expect_eq!(
                new_trace.maps["src_file"],
                td.join("new_dest_file").to_str().unwrap()
            );

            expect_that!(
                logger.messages(),
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

            let mut logger = null_logger();
            let new_trace = load(td.path(), &pkg, Some(&trace), &mut logger)?;

            expect_eq!(new_trace.directory, trace.directory);
            expect_eq!(new_trace.maps.len(), trace.maps.len() + 1);
            expect_eq!(new_trace.maps["src_dir"], trace.maps["src_dir"]);
            expect_eq!(new_trace.maps["src_file"], trace.maps["src_file"]);
            expect_eq!(
                new_trace.maps["new_src_file"],
                new_dst_path.to_str().unwrap(),
            );

            expect_that!(
                logger.messages(),
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

            let mut logger = null_logger();
            let new_trace = load(td.path(), &pkg, Some(&trace), &mut logger)?;

            expect_eq!(new_trace.directory, trace.directory);
            expect_eq!(new_trace.maps.len(), trace.maps.len() - 1);
            expect_eq!(new_trace.maps["src_dir"], trace.maps["src_dir"]);
            expect_eq!(new_trace.maps.get("src_file"), None);

            expect_that!(
                logger.messages(),
                superset_of([&LogMessage::RemoveSymlink {
                    src: td.join(SRC_FILE_PATH),
                    dst: td.join(DST_FILE_PATH)
                },])
            );

            Ok(())
        }

        #[gtest]
        fn src_not_exists() -> Result<()> {
            let (td, pkg, trace) = setup()?;
            fs::remove_file(td.join(SRC_FILE_PATH))?;

            let result = load(td.path(), &pkg, Some(&trace), &mut null_logger()).unwrap_err();
            expect_that!(result, pat!(LoadError::SrcNotExists("src_file")));

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

            let result = load(td.path(), &pkg, Some(&trace), &mut null_logger()).unwrap_err();
            expect_that!(
                result,
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

            let result = load(td.path(), &pkg, Some(&trace), &mut null_logger()).unwrap_err();
            expect_that!(
                result,
                pat!(LoadError::DstNotSymlink(&td.join(DST_FILE_PATH)))
            );

            Ok(())
        }
    }
}
