mod file;
pub mod matchers;

use crate::logger::NullOutput;
use crate::runner::Runner;

pub use file::TempDir;

pub fn null_runner() -> Runner<NullOutput> {
    Runner::new(NullOutput::default())
}
