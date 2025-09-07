mod de_map_as_vec;
mod error;
mod named_package;
mod read;
mod var;

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

use de_map_as_vec::deserialize_map_as_vec;

pub use error::{PkgsParseError, VarsBuildError, VarsParseError};
pub use named_package::NamedPackage;
pub use read::ConfigError;
pub use var::VarMap;

fn empty_map() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_map_as_vec")]
    #[schemars(default = "empty_map", with = "BTreeMap<String, String>")]
    pub vars: Vec<(String, String)>,

    pub packages: BTreeMap<String, Package>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Package {
    #[serde(default)]
    pub kind: PackageType,

    #[serde(default, deserialize_with = "deserialize_map_as_vec")]
    #[schemars(default = "empty_map", with = "BTreeMap<String, String>")]
    pub vars: Vec<(String, String)>,

    #[serde(default, deserialize_with = "deserialize_map_as_vec")]
    #[schemars(default = "empty_map", with = "BTreeMap<String, String>")]
    pub maps: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageType {
    #[default]
    Local,
}
