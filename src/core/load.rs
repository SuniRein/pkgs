use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::utils::create_symlink;
use super::NamedPackage;
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

pub fn load(
    root: &Path,
    package: &NamedPackage,
    trace: Option<&PkgTrace>,
) -> Result<PkgTrace, LoadError> {
    if let Some(trace) = trace {
        load_with_trace(root, package, trace)
    } else {
        load_directly(root, package)
    }
}

fn load_directly(root: &Path, package: &NamedPackage) -> Result<PkgTrace, LoadError> {
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
        }

        let src_abs = src_path.canonicalize()?;
        create_symlink(&src_abs, dst)?;

        let dst_abs = dst_path.canonicalize()?;
        trace
            .maps
            .insert(src.into(), dst_abs.to_str().unwrap().to_string());
    }

    Ok(trace)
}

fn load_with_trace(
    _root: &Path,
    _package: &NamedPackage,
    _trace: &PkgTrace,
) -> Result<PkgTrace, LoadError> {
    todo!();
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    mod load_without_trace {
        use std::collections::HashMap;

        use tempfile::TempDir;

        use super::*;
        use crate::config::{Package, PackageType};

        const SRC_FILE_PATH: &str = "test_package/src_file";
        const SRC_DIR_PATH: &str = "test_package/src_dir";

        const DST_FILE_PATH: &str = "./test_pkg/dst_file";
        const DST_DIR_PATH: &str = "./test_a/test_b/dst_dir";

        fn setup_pkg(td: &TempDir) -> NamedPackage {
            let dst_file_path = td.path().join(DST_FILE_PATH).to_str().unwrap().to_string();
            let dst_dir_path = td.path().join(DST_DIR_PATH).to_str().unwrap().to_string();

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
            let td = TempDir::new()?;
            let pkg_dir = td.path().join("test_package");
            fs::create_dir_all(&pkg_dir)?;
            fs::create_dir_all(pkg_dir.join("src_dir"))?;
            fs::write(pkg_dir.join("src_file"), "test content")?;
            Ok(td)
        }

        #[gtest]
        fn it_works() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);

            let trace = load(td.path(), &pkg, None)?;

            let dst_file = td.path().join(DST_FILE_PATH);
            let dst_dir = td.path().join(DST_DIR_PATH);

            expect_pred!(dst_file.exists());
            expect_pred!(dst_file.is_symlink());
            expect_eq!(
                dst_file.read_link()?,
                td.path().join(SRC_FILE_PATH).canonicalize()?,
                "dst_file should point to the absolute path of src_file"
            );

            expect_pred!(dst_dir.exists());
            expect_pred!(dst_dir.is_symlink());
            expect_eq!(
                dst_dir.read_link()?,
                td.path().join(SRC_DIR_PATH).canonicalize()?,
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
        fn src_not_exists() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            fs::remove_file(td.path().join(SRC_FILE_PATH))?;

            let result = load(td.path(), &pkg, None).unwrap_err();
            expect_that!(result, pat!(LoadError::SrcNotExists(_)));

            Ok(())
        }

        #[gtest]
        fn dst_already_exists() -> Result<()> {
            let td = setup_dir()?;
            let pkg = setup_pkg(&td);
            fs::create_dir_all(td.path().join(DST_FILE_PATH))?;

            let result = load(td.path(), &pkg, None).unwrap_err();
            expect_that!(result, pat!(LoadError::DstAlreadyExists { .. }));

            Ok(())
        }
    }
}
