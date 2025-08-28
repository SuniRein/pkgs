mod read;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub packages: BTreeMap<String, Package>,
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
