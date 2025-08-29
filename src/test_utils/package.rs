use std::collections::BTreeMap;

use googletest::Result;

use super::common_runner;
use super::file::TempDir;
use crate::config::{NamedPackage, Package, PackageType};
use crate::logger::NullOutput;
use crate::runner::Runner;

pub const SRC_FILE_PATH: &str = "test_package/src_file";
pub const SRC_DIR_PATH: &str = "test_package/src_dir";

pub const DST_FILE_PATH: &str = "./test_pkg/dst_file";
pub const DST_DIR_PATH: &str = "./test_a/test_b/dst_dir";

pub fn common_local_pkg() -> Result<(TempDir, NamedPackage, Runner<NullOutput>)> {
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
