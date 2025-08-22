use std::io::{self, Write};

use super::LogMessage;

pub trait LoggerOutput {
    fn log(&mut self, message: &LogMessage);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NullOutput;

impl LoggerOutput for NullOutput {
    fn log(&mut self, _message: &LogMessage) {}
}

pub struct WriterOutput<W: Write> {
    writer: W,
}

impl<W: Write> WriterOutput<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> LoggerOutput for WriterOutput<W> {
    fn log(&mut self, message: &LogMessage) {
        let message = match message {
            LogMessage::LoadModule(module) => format!("Load Module {module}"),
            LogMessage::CreateDir(path) => format!("Create Directory {}", path.to_string_lossy()),
            LogMessage::CreateFile(path) => format!("Create File {}", path.to_string_lossy()),
            LogMessage::CreateSymlink { src, dst } => format!(
                "Create Symlink {} -> {}",
                dst.to_string_lossy(),
                src.to_string_lossy()
            ),
        };

        // ignore errors on write
        let _ = writeln!(self.writer, "{message}").inspect_err(|err| {
            eprintln!("Warning! Write log output to writer failed: {err}");
        });
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;

    use googletest::prelude::*;

    use super::*;

    #[gtest]
    fn writer_output() -> Result<()> {
        let mut buf = Cursor::new(Vec::new());
        let mut output = WriterOutput::new(&mut buf);
        output.log(&LogMessage::CreateDir(PathBuf::from("test_dir")));
        output.log(&LogMessage::CreateSymlink {
            src: PathBuf::from("src"),
            dst: PathBuf::from("dst"),
        });
        output.flush()?;

        let content = String::from_utf8(buf.into_inner())?;
        expect_eq!(
            content,
            "Create Directory test_dir\nCreate Symlink dst -> src\n"
        );

        Ok(())
    }
}
