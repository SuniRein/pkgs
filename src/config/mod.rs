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
    pub maps: HashMap<MappingSrc, MappingDst>
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageType {
    #[default]
    Local
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappingSrc(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappingDst(pub String);
