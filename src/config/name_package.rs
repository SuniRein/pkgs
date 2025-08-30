use std::collections::BTreeMap;

use super::{Config, PkgsParseError, VarMap, VarsParseError};
use crate::config::{Package, PackageType};

impl Config {
    pub fn get(&self, name: &str) -> Result<NamedPackage, PkgsParseError> {
        Ok(NamedPackage::try_new(
            name,
            self.packages[name].clone(),
            VarMap::try_new(&self.vars)?, // PERF: varmap will be built multiple times here
        )?)
    }
}

#[derive(Debug)]
pub struct NamedPackage {
    name: String,
    kind: PackageType,
    maps: BTreeMap<String, String>,
}

impl NamedPackage {
    pub fn try_new(name: &str, package: Package, vars: VarMap) -> Result<Self, VarsParseError> {
        let mut maps = BTreeMap::new();
        for (k, v) in package.maps {
            maps.insert(k, vars.parse(&v)?);
        }

        Ok(Self {
            name: name.to_string(),
            kind: package.kind,
            maps,
        })
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
        self.kind
    }

    pub fn maps(&self) -> &BTreeMap<String, String> {
        &self.maps
    }

    #[cfg(test)]
    pub fn insert_map(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        self.maps
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
    }

    #[cfg(test)]
    pub fn remove_map(&mut self, key: impl AsRef<str>) {
        self.maps.remove(key.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fs::home_dir, test_utils::prelude::*};

    fn setup() -> Config {
        let vars = vec![
            ("APP_DIR".to_string(), "${HOME}/myapp".to_string()),
            ("MY_VAR1".to_string(), "hello".to_string()),
            ("MY_VAR2".to_string(), "${MY_VAR1}_world".to_string()),
        ];

        let packages = BTreeMap::from_iter([(
            "test_pkg".to_string(),
            Package {
                kind: PackageType::Local,
                maps: BTreeMap::from_iter([
                    ("app_dir".to_string(), "${APP_DIR}".to_string()),
                    ("path".to_string(), "/usr/local/${MY_VAR2}".to_string()),
                    ("config".to_string(), "${MY_VAR1}_config".to_string()),
                ]),
            },
        )]);

        Config { vars, packages }
    }

    #[gtest]
    fn it_works() -> Result<()> {
        let config = setup();

        let pkg = config.get("test_pkg")?;
        expect_eq!(pkg.name(), "test_pkg");
        expect_eq!(pkg.kind(), PackageType::Local);
        expect_eq!(pkg.get_directory(), "test_pkg");

        expect_eq!(pkg.maps().len(), 3);
        expect_eq!(
            pkg.maps()["app_dir"],
            home_dir().join("myapp").to_str().unwrap()
        );
        expect_eq!(pkg.maps()["path"], "/usr/local/hello_world");
        expect_eq!(pkg.maps()["config"], "hello_config");

        Ok(())
    }

    #[gtest]
    fn unknown_var_when_build() -> Result<()> {
        let mut config = setup();
        config
            .vars
            .push(("MY_VAR3".to_string(), "${UNKNOWN}".to_string()));

        let err = config.get("test_pkg").unwrap_err();
        expect_that!(err, pat!(PkgsParseError::VarsBuild(_)));

        Ok(())
    }

    #[gtest]
    fn unknown_var_when_parse() -> Result<()> {
        let mut config = setup();
        config
            .packages
            .get_mut("test_pkg")
            .unwrap()
            .maps
            .insert("bad".to_string(), "${UNKNOWN}".to_string());

        let err = config.get("test_pkg").unwrap_err();
        expect_that!(err, pat!(PkgsParseError::VarsParse(_)));

        Ok(())
    }
}
