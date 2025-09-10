use std::collections::HashMap;

use super::{VarsBuildError, VarsParseError};
use crate::fs::home_dir;

#[derive(Debug)]
pub struct VarMap {
    map: HashMap<String, String>,
}

impl VarMap {
    fn default_vars() -> HashMap<String, String> {
        HashMap::from([("HOME".into(), home_dir().to_string_lossy().into())])
    }

    pub fn try_new(vars: &[(String, String)]) -> Result<Self, VarsBuildError> {
        let mut ret = Self::default();
        for (var, value) in vars {
            ret.add_var(var, value)?;
        }
        Ok(ret)
    }

    pub fn extends(&mut self, vars: &[(String, String)]) -> Result<(), VarsBuildError> {
        for (var, value) in vars {
            self.add_var(var, value)?;
        }
        Ok(())
    }

    fn add_var(&mut self, var: &str, value: &str) -> Result<(), VarsBuildError> {
        let value = self.parse(value).map_err(|kind| VarsBuildError {
            var: var.to_string(),
            kind,
        })?;
        self.map.insert(var.to_string(), value);
        Ok(())
    }

    pub fn map(&self) -> &HashMap<String, String> {
        &self.map
    }

    pub fn parse(&self, input: &str) -> Result<String, VarsParseError> {
        let mut result = String::with_capacity(input.len());
        let mut rest = input;
        let mut cursor = 0;

        while let Some(i) = rest.find("${") {
            result.push_str(&rest[..i]);

            let after = i + 2;
            let Some(end_rel) = rest[after..].find('}') else {
                return Err(VarsParseError::UnclosedBrace(cursor + i));
            };

            let name = &rest[after..after + end_rel];
            if name.is_empty() {
                return Err(VarsParseError::EmptyVarName(cursor + i));
            }

            match self.map().get(name) {
                Some(value) => result.push_str(value),
                None => return Err(VarsParseError::UnknowndVar(name.to_string(), cursor + i)),
            }

            rest = &rest[after + end_rel + 1..];
            cursor += after + end_rel + 1;
        }
        result.push_str(rest);

        Ok(result)
    }
}

impl Default for VarMap {
    fn default() -> Self {
        Self {
            map: Self::default_vars(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::prelude::*;

    fn setup() -> Result<VarMap> {
        let custom = vec![
            ("CONFIG_DIR".into(), "${HOME}/.config".into()),
            ("MY_VAR1".into(), "hello".into()),
            ("MY_VAR2".into(), "${MY_VAR1}_world".into()),
        ];
        Ok(VarMap::try_new(&custom)?)
    }

    #[gtest]
    fn default_vars() -> Result<()> {
        let var_map = VarMap::try_new(&[])?;
        expect_eq!(var_map.map()["HOME"], home_dir().to_string_lossy());
        Ok(())
    }

    mod extend_vars {
        use super::*;

        #[gtest]
        fn add_new() -> Result<()> {
            let mut var_map = setup()?;
            let len = var_map.map().len();

            var_map.extends(&[("NEW_VAR".into(), "new_value".into())])?;
            expect_eq!(var_map.map().len(), len + 1);
            expect_eq!(var_map.map()["NEW_VAR"], "new_value");

            Ok(())
        }

        #[gtest]
        fn override_existing() -> Result<()> {
            let mut var_map = setup()?;
            let len = var_map.map().len();

            var_map.extends(&[("MY_VAR1".into(), "${MY_VAR1}_new".into())])?;
            expect_eq!(var_map.map().len(), len);
            expect_eq!(var_map.map()["MY_VAR1"], "hello_new");
            expect_eq!(var_map.map()["MY_VAR2"], "hello_world");

            Ok(())
        }
    }

    #[gtest]
    fn custom_vars() -> Result<()> {
        let var_map = setup()?;
        expect_eq!(var_map.map()["HOME"], home_dir().to_string_lossy());
        expect_eq!(
            var_map.map()["CONFIG_DIR"],
            home_dir().join(".config").to_string_lossy()
        );
        expect_eq!(var_map.map()["MY_VAR1"], "hello");
        expect_eq!(var_map.map()["MY_VAR2"], "hello_world");

        Ok(())
    }

    #[gtest]
    fn custom_vars_with_wrong_order() -> Result<()> {
        let custom = vec![
            ("MY_VAR2".into(), "${MY_VAR1}_world".into()),
            ("MY_VAR1".into(), "hello".into()),
        ];

        let err = VarMap::try_new(&custom).unwrap_err();
        expect_that!(
            err.kind,
            pat!(VarsParseError::UnknowndVar("MY_VAR1", &0_usize))
        );
        expect_eq!(err.var, "MY_VAR2");

        Ok(())
    }

    mod parse {
        use super::*;

        #[gtest]
        fn common_parse() -> Result<()> {
            let var_map = setup()?;
            expect_eq!(
                var_map.parse("${CONFIG_DIR}/test")?,
                home_dir().join(".config/test").to_string_lossy()
            );
            expect_eq!(
                var_map.parse("/tmp/${MY_VAR1}/${MY_VAR2}/${MY_VAR1}")?,
                "/tmp/hello/hello_world/hello"
            );

            Ok(())
        }

        #[gtest]
        fn unclosed_brace() -> Result<()> {
            let var_map = setup()?;
            let err = var_map
                .parse("/tmp/${MY_VAR1}/${MY_VAR2/hello")
                .unwrap_err();
            expect_that!(err, pat!(VarsParseError::UnclosedBrace(&16_usize)));
            Ok(())
        }

        #[gtest]
        fn empty_var() -> Result<()> {
            let var_map = setup()?;
            let err = var_map.parse("/tmp/${MY_VAR1}/${MY_VAR2}/${}").unwrap_err();
            expect_that!(err, pat!(VarsParseError::EmptyVarName(&27_usize)));
            Ok(())
        }

        #[gtest]
        fn unknowd_var() -> Result<()> {
            let var_map = setup()?;
            let err = var_map
                .parse("/tmp/${MY_VAR1}/${MY_VAR2}/${MY_VAR3}")
                .unwrap_err();
            expect_that!(err, pat!(VarsParseError::UnknowndVar("MY_VAR3", &27_usize)));
            Ok(())
        }
    }
}
