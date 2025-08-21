mod read;

use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub packages: HashMap<String, Package>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub kind: PackageType,

    #[serde(default)]
    pub maps: HashMap<String, String>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageType {
    #[default]
    Local
}
