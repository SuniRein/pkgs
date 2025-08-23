use std::path::Path;

use super::{LogMessage, LoggerOutput};

pub struct Logger<O: LoggerOutput> {
    output: O,
    messages: Vec<LogMessage>,
}

impl<O: LoggerOutput> Logger<O> {
    pub fn new(output: O) -> Self {
        Self {
            output,
            messages: vec![],
        }
    }

    pub fn messages(&self) -> &[LogMessage] {
        &self.messages
    }

    pub fn log(&mut self, message: LogMessage) {
        self.output.log(&message);
        self.messages.push(message);
    }

    pub fn load_module(&mut self, module: impl AsRef<str>) {
        self.log(LogMessage::LoadModule(module.as_ref().into()));
    }

    pub fn create_dir(&mut self, path: impl AsRef<Path>) {
        self.log(LogMessage::CreateDir(path.as_ref().into()));
    }

    pub fn create_file(&mut self, path: impl AsRef<Path>) {
        self.log(LogMessage::CreateFile(path.as_ref().into()));
    }

    pub fn create_symlink(&mut self, src: impl AsRef<Path>, dst: impl AsRef<Path>) {
        self.log(LogMessage::CreateSymlink {
            src: src.as_ref().into(),
            dst: dst.as_ref().into(),
        });
    }

    pub fn remove_symlink(&mut self, src: impl AsRef<Path>, dst: impl AsRef<Path>) {
        self.log(LogMessage::RemoveSymlink {
            src: src.as_ref().into(),
            dst: dst.as_ref().into(),
        });
    }
}
