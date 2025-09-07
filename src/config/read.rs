use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_yaml_ng::Error as YamlDeError;
use thiserror::Error;
use toml::de::Error as TomlDeError;

use super::Config;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] TomlDeError),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] YamlDeError),

    #[error("unsupported file format: {0}")]
    UnsupportedFileFormat(PathBuf),
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;

        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => Self::from_toml(&content).map_err(ConfigError::TomlParse),
            Some("yaml") | Some("yml") => Self::from_yaml(&content).map_err(ConfigError::YamlParse),
            _ => Err(ConfigError::UnsupportedFileFormat(path.to_path_buf())),
        }
    }

    pub fn from_toml(content: &str) -> Result<Self, TomlDeError> {
        toml::from_str(content)
    }

    pub fn from_yaml(content: &str) -> Result<Self, YamlDeError> {
        serde_yaml_ng::from_str(content)
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

    const YAML_CONTENT: &str = indoc! {r#"
        vars:
          CONFIG_DIR: ${HOME}/.config
          DESKTOP_DIR: ${HOME}/.local/share/applications
          NU_AUTOLOAD: ${HOME}/.config/nu/autoload
          A_VAR: a value with ${CONFIG} inside

        packages:
          yazi:
            type: local
            maps:
              yazi: ${CONFIG_DIR}/yazi
              "yazi.nu": ${NU_AUTOLOAD}/yazi.nu

          kitty:
            maps:
              kitty: ${CONFIG_DIR}/kitty
              kitty.desktop: ${DESKTOP_DIR}/kitty.desktop

          empty maps: {}
    "#};

    mod parse {
        use super::*;

        #[gtest]
        fn toml_parse() {
            let config: Config = Config::from_toml(TOML_CONTENT).unwrap();
            validate_config(config);
        }

        #[gtest]
        fn yaml_parse() {
            let config: Config = Config::from_yaml(YAML_CONTENT).unwrap();
            validate_config(config);
        }

        fn validate_config(config: Config) {
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
            expect_eq!(
                config.packages["yazi"].maps,
                [
                    ("yazi".into(), "${CONFIG_DIR}/yazi".into()),
                    ("yazi.nu".into(), "${NU_AUTOLOAD}/yazi.nu".into())
                ]
            );

            expect_eq!(config.packages["kitty"].kind, PackageType::Local);
            expect_eq!(
                config.packages["kitty"].maps,
                [
                    ("kitty".into(), "${CONFIG_DIR}/kitty".into()),
                    (
                        "kitty.desktop".into(),
                        "${DESKTOP_DIR}/kitty.desktop".into()
                    )
                ]
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
        fn read_toml() {
            let file = setup(".toml", TOML_CONTENT);
            let config = Config::read(file.path()).unwrap();
            expect_eq!(config.packages.len(), 3);
        }

        #[gtest]
        fn read_yaml() {
            let file = setup(".yaml", YAML_CONTENT);
            let config = Config::read(file.path()).unwrap();
            expect_eq!(config.packages.len(), 3);
        }

        #[gtest]
        fn read_yml() {
            let file = setup(".yml", YAML_CONTENT);
            let config = Config::read(file.path()).unwrap();
            expect_eq!(config.packages.len(), 3);
        }

        #[gtest]
        fn parse_error() {
            let file = setup(".toml", "invalid toml content");

            let err = Config::read(file.path()).unwrap_err();
            expect_that!(err, pat!(ConfigError::TomlParse(_)));
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
