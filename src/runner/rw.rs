use super::{Runner, RunnerError};
use crate::config::Config;
use crate::logger::LoggerOutput;
use crate::meta::TOML_CONFIG_FILE;

impl<O: LoggerOutput> Runner<O> {
    pub fn read_config(&self) -> Result<Config, RunnerError> {
        let toml_path = self.cwd.join(TOML_CONFIG_FILE);
        if !toml_path.exists() {
            return Err(RunnerError::ConfigNotFound);
        }

        let config = Config::read(&self.cwd.join(TOML_CONFIG_FILE))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use indoc::indoc;

    use super::*;
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
}
