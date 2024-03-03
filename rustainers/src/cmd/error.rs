use std::error::Error;
use std::fmt::{self, Display};
use std::process::Output;

use crate::io::ReadLinesError;

#[derive(Debug)]
#[non_exhaustive]
pub enum CommandError {
    /// Command run but fail
    CommandFail {
        /// The command
        command: String,
        /// The command output
        output: Output,
    },

    /// Command fail to run
    CommandProcessError {
        /// The command
        command: String,
        /// The source
        source: std::io::Error,
    },

    /// I/O error
    IoError {
        /// The command
        command: String,
        /// The source
        source: std::io::Error,
    },

    /// A serde error
    SerdeError {
        /// The command
        command: String,
        /// The output
        output: Output,
        /// The source
        source: serde_json::Error,
    },

    /// Command run but fail
    CommandWatchFail {
        /// The command
        command: String,
        // The source
        source: ReadLinesError,
    },
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandFail { command, output } => {
                writeln!(f, "Fail to execute command\n{command}")?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                writeln!(f, "------ stdout ------\n{stdout}")?;
                let stderr = String::from_utf8_lossy(&output.stderr);
                write!(f, "------ stderr ------\n{stderr}")
            }
            Self::CommandProcessError { command, .. } => {
                write!(f, "Fail to execute command\n{command}")
            }
            Self::IoError { command, source } => {
                writeln!(f, "IO error: {source} during")?;
                writeln!(f, "{command}")
            }
            Self::SerdeError {
                command,
                output,
                source,
            } => {
                writeln!(f, "Serde error: {source} during")?;
                writeln!(f, "{command}")?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                writeln!(f, "------ stdout ------\n{stdout}")?;
                let stderr = String::from_utf8_lossy(&output.stderr);
                write!(f, "------ stderr ------\n{stderr}")
            }
            Self::CommandWatchFail { command, source } => {
                writeln!(f, "Read lines error: {source} during\n{command}")
            }
        }
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandFail { .. } => None,
            Self::CommandProcessError { source, .. } | Self::IoError { source, .. } => Some(source),
            Self::CommandWatchFail { source, .. } => Some(source),
            Self::SerdeError { source, .. } => Some(source),
        }
    }
}
