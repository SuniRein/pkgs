use std::path::PathBuf;

use super::{Runner, RunnerError};
use crate::config::Config;
use crate::logger::LoggerOutput;
use crate::meta::{PKGS_DIR, TOML_CONFIG_FILE};

impl<O: LoggerOutput> Runner<O> {
    pub fn read_config(&self) -> Result<Config, RunnerError> {
        let toml_path = self.cwd.join(TOML_CONFIG_FILE);
        if !toml_path.exists() {
            return Err(RunnerError::ConfigNotFound);
        }

        let config = Config::read(&self.cwd.join(TOML_CONFIG_FILE))?;
        Ok(config)
    }

    pub fn create_pkgs_dir(&mut self) -> Result<PathBuf, RunnerError> {
        let pkgs_dir = self.cwd.join(PKGS_DIR).to_path_buf();
        if !pkgs_dir.exists() {
            self.create_dir(&pkgs_dir)?;
            return Ok(pkgs_dir);
        }
        if !pkgs_dir.is_dir() {
            return Err(RunnerError::PkgsDirNotADir);
        }
        Ok(pkgs_dir)
    }

    pub fn get_pkgs_dir(&self) -> Result<PathBuf, RunnerError> {
        let pkgs_dir = self.cwd.join(PKGS_DIR).to_path_buf();
        if !pkgs_dir.exists() {
            return Err(RunnerError::PkgsDirNotFound);
        }
        if !pkgs_dir.is_dir() {
            return Err(RunnerError::PkgsDirNotADir);
        }
        Ok(pkgs_dir)
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use indoc::indoc;

    use super::*;
    use crate::logger::LogMessage;
    use crate::test_utils::{TempDir, common_runner};

    mod read_config {
        use super::*;
        use crate::config::PackageType;

        #[gtest]
        fn read_toml() -> Result<()> {
            let td = TempDir::new()?.file(
                TOML_CONFIG_FILE,
                indoc! {r#"
                    [packages.test.maps]
                    src_file = "dst_file"
                "#},
            )?;
            let runner = common_runner(td.path());
            let config = runner.read_config()?;

            expect_eq!(config.packages.len(), 1);

            let test_pkg = &config.packages["test"];
            expect_eq!(test_pkg.kind, PackageType::Local);
            expect_eq!(test_pkg.maps["src_file"], "dst_file");

            Ok(())
        }

        #[gtest]
        fn config_not_found() -> Result<()> {
            let td = TempDir::new()?;
            let runner = common_runner(td.path());
            let error = runner.read_config().unwrap_err();

            expect_that!(error, pat!(RunnerError::ConfigNotFound));

            Ok(())
        }

        #[gtest]
        fn wrong_config_file_format() -> Result<()> {
            let td = TempDir::new()?.file(TOML_CONFIG_FILE, "invalid file content")?;
            let runner = common_runner(td.path());
            let error = runner.read_config().unwrap_err();

            expect_that!(error, pat!(RunnerError::ConfigReadError(_)));

            Ok(())
        }
    }

    mod create_pkgs_dir {

        use super::*;

        #[gtest]
        fn create_if_not_exist() -> Result<()> {
            let td = TempDir::new()?;
            let mut runner = common_runner(td.path());
            let pkgs_dir = runner.create_pkgs_dir()?;

            expect_eq!(pkgs_dir, td.join(PKGS_DIR));
            expect_pred!(pkgs_dir.is_dir());
            expect_eq!(
                runner.messages()[0],
                LogMessage::CreateDir(td.join(PKGS_DIR))
            );

            Ok(())
        }

        #[gtest]
        fn do_nothing_if_exist() -> Result<()> {
            let td = TempDir::new()?.dir(PKGS_DIR)?;
            let mut runner = common_runner(td.path());
            let pkgs_dir = runner.create_pkgs_dir()?;

            expect_eq!(pkgs_dir, td.join(PKGS_DIR));
            expect_pred!(pkgs_dir.is_dir());

            Ok(())
        }

        #[gtest]
        fn error_if_pkgs_not_a_dir() -> Result<()> {
            let td = TempDir::new()?.file(PKGS_DIR, "")?;
            let mut runner = common_runner(td.path());
            let err = runner.create_pkgs_dir().unwrap_err();

            expect_that!(err, pat!(RunnerError::PkgsDirNotADir));

            Ok(())
        }
    }

    mod get_pkgs_dir {
        use super::*;

        #[gtest]
        fn it_works() -> Result<()> {
            let td = TempDir::new()?.dir(PKGS_DIR)?;
            let runner = common_runner(td.path());
            let pkgs_dir = runner.get_pkgs_dir()?;

            expect_eq!(pkgs_dir, td.join(PKGS_DIR));

            Ok(())
        }

        #[gtest]
        fn not_found() -> Result<()> {
            let td = TempDir::new()?;
            let runner = common_runner(td.path());
            let err = runner.get_pkgs_dir().unwrap_err();

            expect_that!(err, pat!(RunnerError::PkgsDirNotFound));

            Ok(())
        }

        #[gtest]
        fn not_a_dir() -> Result<()> {
            let td = TempDir::new()?.file(PKGS_DIR, "")?;
            let runner = common_runner(td.path());
            let err = runner.get_pkgs_dir().unwrap_err();

            expect_that!(err, pat!(RunnerError::PkgsDirNotADir));

            Ok(())
        }
    }
}
