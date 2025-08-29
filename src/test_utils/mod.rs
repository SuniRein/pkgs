mod file;
mod matchers;
mod package;

use std::path::Path;

use crate::logger::NullOutput;
use crate::runner::Runner;

pub fn common_runner(cwd: impl AsRef<Path>) -> Runner<NullOutput> {
    Runner::new(cwd.as_ref(), NullOutput)
}

pub mod prelude {
    pub use std::collections::BTreeMap;
    pub use std::path::PathBuf;

    pub use googletest::prelude::*;

    pub use super::common_runner;
    pub use super::file::TempDir;
    pub use super::matchers::*;
    pub use super::package::*;

    pub use crate::config::{NamedPackage, PackageType};
    pub use crate::logger::{LogMessage, NullOutput};
    pub use crate::runner::{LoadError, RunnerError, UnloadError};
    pub use crate::trace::PkgTrace;
}
