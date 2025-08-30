mod error;
mod name_package;
mod read;
mod var;

use std::collections::BTreeMap;

use serde::Deserialize;
use tuple_vec_map;

pub use error::{PkgsParseError, VarsBuildError, VarsParseError};
pub use name_package::NamedPackage;
pub use read::ConfigError;
pub use var::VarMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default, with = "tuple_vec_map")]
    pub vars: Vec<(String, String)>,
    pub packages: BTreeMap<String, Package>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub kind: PackageType,

    #[serde(default)]
    pub maps: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageType {
    #[default]
    Local,
}
