use std::fmt::Debug;
use std::path::{Path, PathBuf};

use googletest::{
    description::Description,
    matcher::{Matcher, MatcherBase, MatcherResult},
};

pub fn is_symlink_for(src: impl AsRef<Path>) -> IsSymlinkForMatcher {
    IsSymlinkForMatcher {
        src: src.as_ref().to_path_buf(),
    }
}

#[derive(MatcherBase)]
pub struct IsSymlinkForMatcher {
    src: PathBuf,
}

impl<T> Matcher<T> for IsSymlinkForMatcher
where
    T: Debug + Copy + AsRef<Path>,
{
    fn matches(&self, actual: T) -> MatcherResult {
        let actual = actual.as_ref();
        if actual.is_symlink() && actual.read_link().is_ok_and(|link| link == self.src) {
            MatcherResult::Match
        } else {
            MatcherResult::NoMatch
        }
    }

    fn describe(&self, matcher_result: MatcherResult) -> Description {
        format!(
            "{} symlink for '{}'",
            match matcher_result {
                MatcherResult::Match => "is",
                MatcherResult::NoMatch => "isn't",
            },
            self.src.to_string_lossy()
        )
        .into()
    }

    fn explain_match(&self, actual: T) -> Description {
        let actual = actual.as_ref();
        if !actual.exists() {
            return "which doesn't exist".into();
        }
        if !actual.is_symlink() {
            return "which isn't a symlink".into();
        }

        let link = actual.read_link().unwrap();
        if link == self.src {
            "which is a symlink pointing to the targeted path".into()
        } else {
            format!(
                "which is a symlink but points to a different path '{}'",
                link.to_string_lossy()
            )
            .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use googletest::prelude::*;
    use indoc::formatdoc;

    use super::*;
    use crate::fs::create_symlink;

    #[gtest]
    fn it_works() -> Result<()> {
        let td = tempfile::tempdir()?;

        let src = td.path().join("src");
        fs::create_dir(td.path().join("src"))?;

        let dst = td.path().join("dst");
        create_symlink(&src, &dst)?;

        verify_that!(dst, is_symlink_for(src))
    }

    #[gtest]
    fn negative() -> Result<()> {
        let td = tempfile::tempdir()?;

        let src = td.path().join("src");
        fs::create_dir(&src)?;

        let dst = td.path().join("dst");
        create_symlink(&src, &dst)?;

        let result = verify_that!(dst, not(is_symlink_for(&src)));
        verify_that!(
            result,
            err(displays_as(contains_substring(formatdoc! {r#"
                Value of: dst
                Expected: isn't symlink for '{}'
                Actual: "{}",
                  which is a symlink pointing to the targeted path
            "#,
                src.to_string_lossy(),
                dst.to_string_lossy(),
            })))
        )
    }

    #[gtest]
    fn mismatch_the_same_path_with_different_name() -> Result<()> {
        let td = tempfile::tempdir()?;

        let src = td.path().join("src");
        fs::create_dir(td.path().join("src"))?;

        let dst = td.path().join("dst");
        create_symlink(&src, &dst)?;

        let result = verify_that!(dst, is_symlink_for(src.join("../src")));
        verify_that!(
            result,
            err(displays_as(contains_substring(formatdoc! {r#"
                Value of: dst
                Expected: is symlink for '{}'
                Actual: "{}",
                  which is a symlink but points to a different path '{}'
            "#,
                src.join("../src").to_string_lossy(),
                dst.to_string_lossy(),
                src.to_string_lossy(),
            })))
        )
    }

    #[gtest]
    fn mismatch_path_not_exists() -> Result<()> {
        let td = tempfile::tempdir()?;

        let src = td.path().join("src");
        fs::create_dir(&src)?;

        let dst = td.path().join("non_existent_dst");

        let result = verify_that!(dst, is_symlink_for(&src));
        verify_that!(
            result,
            err(displays_as(contains_substring(formatdoc! {r#"
                Value of: dst
                Expected: is symlink for '{}'
                Actual: "{}",
                  which doesn't exist
            "#,
                src.to_string_lossy(),
                dst.to_string_lossy(),
            })))
        )
    }

    #[gtest]
    fn mismatch_path_is_not_a_symlink() -> Result<()> {
        let td = tempfile::tempdir()?;

        let src = td.path().join("src");
        fs::create_dir(&src)?;

        let dst = td.path().join("dst");
        fs::create_dir(&dst)?;

        let result = verify_that!(dst, is_symlink_for(&src));
        verify_that!(
            result,
            err(displays_as(contains_substring(formatdoc! {r#"
                Value of: dst
                Expected: is symlink for '{}'
                Actual: "{}",
                  which isn't a symlink
            "#,
                src.to_string_lossy(),
                dst.to_string_lossy(),
            })))
        )
    }
}
