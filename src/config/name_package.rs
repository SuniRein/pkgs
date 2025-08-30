use std::collections::BTreeMap;

use crate::config::{Package, PackageType};

#[derive(Debug)]
pub struct NamedPackage {
    name: String,
    package: Package,
}

impl NamedPackage {
    pub fn new(name: &str, package: Package) -> Self {
        Self {
            name: name.to_string(),
            package,
        }
    }

    pub fn get_directory(&self) -> String {
        match self.kind() {
            PackageType::Local => self.name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> PackageType {
        self.package.kind
    }

    pub fn maps(&self) -> &BTreeMap<String, String> {
        &self.package.maps
    }

    #[cfg(test)]
    pub fn insert_map(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        self.package
            .maps
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
    }

    #[cfg(test)]
    pub fn remove_map(&mut self, key: impl AsRef<str>) {
        self.package.maps.remove(key.as_ref());
    }
}
