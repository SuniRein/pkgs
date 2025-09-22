mod de_map_as_vec;
mod de_pkg_type;
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
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_map_as_vec")]
    #[schemars(default = "empty_map", with = "BTreeMap<String, String>")]
    pub vars: Vec<(String, String)>,

    pub packages: BTreeMap<String, Package>,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub kind: PackageType,
    pub vars: Vec<(String, String)>,
    pub maps: Vec<(String, String)>,
}

#[derive(Debug, Clone, JsonSchema, Default, PartialEq, Eq)]
pub enum PackageType {
    #[default]
    Local,
    Git(GitPkg),
}

#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GitPkg {
    url: String,
}

impl PackageType {
    pub fn unwrap_git(&self) -> &GitPkg {
        if let Self::Git(pkg) = self {
            pkg
        } else {
            panic!("not a git package kind")
        }
    }
}
