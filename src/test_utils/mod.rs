mod file;
pub mod matchers;

use std::path::Path;

use crate::logger::NullOutput;
use crate::runner::Runner;

pub use file::TempDir;

pub fn null_runner() -> Runner<NullOutput> {
    Runner::new(Path::new(""), NullOutput)
}

pub fn common_runner(cwd: impl AsRef<Path>) -> Runner<NullOutput> {
    Runner::new(cwd.as_ref(), NullOutput)
}
