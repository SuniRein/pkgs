use std::collections::HashMap;

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

    pub fn maps(&self) -> &HashMap<String, String> {
        &self.package.maps
    }
}
