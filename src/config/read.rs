use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    UnsupportedFileFormat(PathBuf),
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
    use indoc::indoc;

    use super::*;
    use crate::test_utils::prelude::*;

    const TOML_CONTENT: &str = indoc! {r#"
        [vars]
        CONFIG_DIR = "${HOME}/.config"
        DESKTOP_DIR = "${HOME}/.local/share/applications"
        NU_AUTOLOAD = "${HOME}/.config/nu/autoload"
        A_VAR = "a value with ${CONFIG} inside"

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

    fn expect_map(map: &BTreeMap<String, String>, key: &str, value: &str) {
        expect_that!(map.get(key), some(eq(value)));
    }

    mod toml_parse {
        use super::*;

        #[gtest]
        fn it_works() {
            let config: Config = Config::from_toml(TOML_CONTENT).unwrap();

            let vars = config.vars;
            expect_eq!(
                vars,
                [
                    ("CONFIG_DIR".into(), "${HOME}/.config".into()),
                    (
                        "DESKTOP_DIR".into(),
                        "${HOME}/.local/share/applications".into()
                    ),
                    ("NU_AUTOLOAD".into(), "${HOME}/.config/nu/autoload".into()),
                    ("A_VAR".into(), "a value with ${CONFIG} inside".into()), // preserve order
                ]
            );

            expect_eq!(config.packages.len(), 3);

            expect_eq!(config.packages["yazi"].kind, PackageType::Local);
            expect_eq!(config.packages["yazi"].maps.len(), 2);
            expect_map(&config.packages["yazi"].maps, "yazi", "${CONFIG_DIR}/yazi");
            expect_map(
                &config.packages["yazi"].maps,
                "yazi.nu",
                "${NU_AUTOLOAD}/yazi.nu",
            );

            expect_eq!(config.packages["kitty"].kind, PackageType::Local);
            expect_eq!(config.packages["kitty"].maps.len(), 2);
            expect_map(
                &config.packages["kitty"].maps,
                "kitty",
                "${CONFIG_DIR}/kitty",
            );
            expect_map(
                &config.packages["kitty"].maps,
                "kitty.desktop",
                "${DESKTOP_DIR}/kitty.desktop",
            );

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

        #[gtest]
        fn invalid_path() {
            let file = NamedTempFile::new().unwrap();
            let path = file.path().to_path_buf();
            drop(file);

            let err = Config::read(&path).unwrap_err();
            expect_that!(err, pat!(ConfigError::Io(_)));
        }
    }
}
