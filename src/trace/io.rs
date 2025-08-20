use std::path::Path;
use std::io;
use std::fs;

use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;
use thiserror::Error;

use super::Trace;

#[derive(Debug, Error)]
pub enum TraceIoError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Parse(#[from] TomlDeError),

    #[error(transparent)]
    Serialize(#[from] TomlSerError),
}

impl Trace {
    pub fn read_from_file(path: &Path) -> Result<Self, TraceIoError> {
        let content = fs::read_to_string(path)?;
        let trace = toml::from_str(&content)?;
        Ok(trace)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), TraceIoError> {
        let content = toml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap};
    use std::io::Write;

    use tempfile::NamedTempFile;
    use googletest::prelude::*;

    use super::*;
    use crate::trace::{Trace, PkgTrace, TraceSrc, TraceDst};

    #[gtest]
    fn write_and_read_trace() {
        let trace = Trace {packages: BTreeMap::from([
            ("pkg1".to_string(), PkgTrace {
                directory: "dir1".to_string(),
                maps: BTreeMap::from([
                    (TraceSrc("src1".to_string()), TraceDst("dst1".to_string())),
                    (TraceSrc("src2".to_string()), TraceDst("dst2".to_string())),
                ]),
            }),
            ("pkg2".to_string(), PkgTrace {
                directory: "dir2".to_string(),
                maps: BTreeMap::from([
                    (TraceSrc("src3".to_string()), TraceDst("dst3".to_string())),
                ]),
            }),
        ]) };

        let file = NamedTempFile::new().unwrap();
        trace.write_to_file(file.path()).unwrap();

        assert_eq!(trace, Trace::read_from_file(file.path()).unwrap());
    }

    #[gtest]
    fn read_non_existent_file() {
        let result = Trace::read_from_file(Path::new("no_such_file.toml")).unwrap_err();
        assert_that!(result, pat!(TraceIoError::Io(_)));
    }

    #[test]
    fn read_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid = [this is not toml]").unwrap();

        let result = Trace::read_from_file(file.path()).unwrap_err();
        assert_that!(result, pat!(TraceIoError::Parse(_)));
    }
}

