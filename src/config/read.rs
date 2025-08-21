use std::path::{PathBuf, Path};
use std::fs;
use std::io;

use thiserror::Error;
use toml::de::Error as TomlDeError;

use super::Config;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] TomlDeError),

    #[error("unsupported file format: {0}")]
    UnsupportedFileFormat(PathBuf)
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;

        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => Self::from_toml(&content).map_err(ConfigError::Parse),
            _ => Err(ConfigError::UnsupportedFileFormat(path.to_path_buf())),
        }
    }

    pub fn from_toml(content: &str) -> Result<Self, TomlDeError> {
        toml::from_str(content)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use googletest::prelude::*;
    use indoc::indoc;

    use super::*;
    use crate::config::PackageType;

    const TOML_CONTENT: &str = indoc! {r#"
        [packages.yazi]
        type = "local"

        [packages.yazi.maps]
        yazi = "${CONFIG_DIR}/yazi"
        "yazi.nu" = "${NU_AUTOLOAD}/yazi.nu"

        [packages.kitty.maps]
        kitty = "${CONFIG_DIR}/kitty"
        "kitty.desktop" = "${DESKTOP_DIR}/kitty.desktop"

        [packages."empty maps"]
    "#};

    fn expect_map(map: &HashMap<String, String>, key: &str, value: &str) {
        expect_that!(map, has_entry(key.to_string(), eq(&value.to_string())));
    }

    mod toml_parse {
        use super::*;

        #[gtest]
        fn it_works() {
            let config: Config = Config::from_toml(TOML_CONTENT).unwrap();

            expect_eq!(config.packages.len(), 3);

            expect_eq!(config.packages["yazi"].kind, PackageType::Local);
            expect_eq!(config.packages["yazi"].maps.len(), 2);
            expect_map(&config.packages["yazi"].maps, "yazi", "${CONFIG_DIR}/yazi");
            expect_map(&config.packages["yazi"].maps, "yazi.nu", "${NU_AUTOLOAD}/yazi.nu");

            expect_eq!(config.packages["kitty"].kind, PackageType::Local);
            expect_eq!(config.packages["kitty"].maps.len(), 2);
            expect_map(&config.packages["kitty"].maps, "kitty", "${CONFIG_DIR}/kitty");
            expect_map(&config.packages["kitty"].maps, "kitty.desktop", "${DESKTOP_DIR}/kitty.desktop");

            expect_eq!(config.packages["empty maps"].kind, PackageType::Local);
            expect_that!(config.packages["empty maps"].maps, is_empty());
        }
    }

    mod read {
        use tempfile::NamedTempFile;

        use super::*;

        fn setup(suffix: &str, content: &str) -> NamedTempFile {
            let file = NamedTempFile::with_suffix(suffix).unwrap();
            fs::write(file.path(), content).unwrap();
            file
        }

        #[gtest]
        fn it_works() {
            let file = setup(".toml", TOML_CONTENT);

            let config = Config::read(file.path()).unwrap();
            expect_eq!(config.packages.len(), 3);
        }

        #[gtest]
        fn parse_error() {
            let file = setup(".toml", "invalid toml content");

            let err = Config::read(file.path()).unwrap_err();
            expect_that!(err, pat!(ConfigError::Parse(_)));
        }

        #[gtest]
        fn unsupported_file_format() {
            let file = setup(".ini", "");

            let err = Config::read(file.path()).unwrap_err();
            expect_that!(err, pat!(ConfigError::UnsupportedFileFormat(_)));
        }
    }
}
