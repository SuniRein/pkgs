use std::iter::IntoIterator;

use clap::Args;

use super::error::CliError;

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct Modules {
    /// Names of targeted modules
    modules: Vec<String>,

    /// Target all modules
    #[arg(long)]
    all: bool,
}

impl Modules {
    pub fn get<I, S>(&self, available: I) -> Result<Vec<String>, CliError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let available = available
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        if self.all {
            return Ok(available);
        }

        for module in &self.modules {
            if !available.iter().any(|m| m == module) {
                return Err(CliError::ModuleNotFound(module.clone()));
            }
        }
        Ok(self.modules.clone())
    }
}

#[cfg(test)]
mod tests {
    use clap::{Parser, error::ErrorKind};

    use super::*;
    use crate::test_utils::prelude::*;

    #[derive(Debug, Parser)]
    struct TestCli {
        #[command(flatten)]
        modules: Modules,
    }

    const PKGS: &[&str] = &["mod1", "mod2", "mod3"];

    #[gtest]
    fn parse_one_module() -> Result<()> {
        let cli = TestCli::try_parse_from(["test", "mod2"])?;
        let modules = cli.modules.get(PKGS)?;
        expect_eq!(modules, vec!["mod2"]);
        Ok(())
    }

    #[gtest]
    fn parse_multiple_modules() -> Result<()> {
        let cli = TestCli::parse_from(["test", "mod1", "mod2"]);
        let modules = cli.modules.get(PKGS)?;
        expect_eq!(modules, vec!["mod1", "mod2"]);
        Ok(())
    }

    #[gtest]
    fn parse_all_flag() -> Result<()> {
        let cli = TestCli::parse_from(["test", "--all"]);
        let modules = cli.modules.get(PKGS)?;
        expect_eq!(modules, vec!["mod1", "mod2", "mod3"]);
        Ok(())
    }

    #[gtest]
    fn all_flag_and_module_could_not_be_specified_simultaneously() -> Result<()> {
        let err = TestCli::try_parse_from(["test", "--all", "mod1"]).unwrap_err();
        expect_that!(err.kind(), pat!(ErrorKind::ArgumentConflict));
        Ok(())
    }

    #[gtest]
    fn modules_is_required() -> Result<()> {
        let err = TestCli::try_parse_from(["test"]).unwrap_err();
        expect_that!(err.kind(), pat!(ErrorKind::MissingRequiredArgument));
        Ok(())
    }

    #[gtest]
    fn invalid_module_could_not_be_parsed() -> Result<()> {
        let cli = TestCli::parse_from(["test", "mod1", "mod4"]);
        let e = cli.modules.get(PKGS).unwrap_err();
        expect_that!(e, pat!(CliError::ModuleNotFound("mod4")));
        Ok(())
    }
}
