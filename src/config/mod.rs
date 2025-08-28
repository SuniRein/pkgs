mod name_package;
mod read;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use read::ConfigError;

pub use name_package::NamedPackage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub packages: BTreeMap<String, Package>,
}

impl Config {
    pub fn get(&self, name: &str) -> NamedPackage {
        NamedPackage::new(name, self.packages[name].clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub kind: PackageType,

    #[serde(default)]
    pub maps: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageType {
    #[default]
    Local,
}
