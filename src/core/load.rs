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
    DstAlreadyExists { src: PathBuf, dst: PathBuf },
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

    let pkg_dir = root.join(&trace.directory);

    for (src, dst) in package.maps() {
        let src_path = pkg_dir.join(src);
        if !src_path.exists() {
            return Err(LoadError::SrcNotExists(src.to_string()));
        }

        let dst_path = PathBuf::from(&dst);
        if dst_path.exists() {
            return Err(LoadError::DstAlreadyExists {
                src: src_path,
                dst: dst_path,
            });
        }

        if let Some(parent) = dst_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
            logger.create_dir(parent);
        }

        let src_abs = src_path.canonicalize()?;
        create_symlink(&src_abs, dst)?;
        logger.create_symlink(&src_abs, &dst_path);

        let dst_abs = dst_path.canonicalize()?;
        trace
            .maps
            .insert(src.into(), dst_abs.to_str().unwrap().to_string());
    }

    Ok(trace)
}

fn load_with_trace<O: LoggerOutput>(
    _root: &Path,
    _package: &NamedPackage,
    _trace: &PkgTrace,
    _output: &mut Logger<O>,
) -> Result<PkgTrace, LoadError> {
    todo!();
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;
    use crate::test_utils::{TempDir, matchers::is_symlink_for};

    mod load_without_trace {
        use std::collections::HashMap;

        use super::*;
        use crate::config::{Package, PackageType};
        use crate::logger::{LogMessage, null_logger};

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

        #[gtest]
        fn it_works() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);

            let trace = load(td.0.path(), &pkg, None, &mut null_logger())?;

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
                dst_file.canonicalize()?.to_str().unwrap(),
                "src_file should map to the absolute path of dst_file in trace"
            );
            expect_eq!(
                trace.maps["src_dir"],
                dst_dir.canonicalize()?.to_str().unwrap(),
                "src_dir should map to the absolute path of dst_dir in trace"
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
            expect_that!(result, pat!(LoadError::SrcNotExists(_)));

            Ok(())
        }

        #[gtest]
        fn dst_already_exists() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            fs::create_dir_all(td.join(DST_FILE_PATH))?;

            let result = load(td.path(), &pkg, None, &mut null_logger()).unwrap_err();
            expect_that!(result, pat!(LoadError::DstAlreadyExists { .. }));

            Ok(())
        }
    }
}
