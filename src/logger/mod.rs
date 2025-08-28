#[allow(clippy::module_inception)]
mod logger;
mod message;
mod output;

pub use logger::Logger;
pub use message::LogMessage;
pub use output::{LoggerOutput, NullOutput, WriterOutput};
