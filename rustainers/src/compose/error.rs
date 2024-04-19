use std::path::PathBuf;

use crate::runner::ContainerError;

use super::ComposeService;

/// A compose error
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ComposeError {
    /// Unsupported compose command
    #[error("Compose command is unsupported for {0}")]
    UnsupportedComposeCommand(String),

    /// Compose file missing
    #[error("Compose file missing {0:?}")]
    ComposeFileMissing(PathBuf),

    /// Bad compose file missing
    #[error("Bad compose file {0:?}")]
    BadComposeFile(PathBuf),

    /// Cannot launch compose containers
    #[error("Cannot launch compose containers {0:?}")]
    ComposeContainerCannotBeStarted(String),

    /// Custom health forbidden in compose
    #[error("Cannot use a custom health check with compose service {0}")]
    NoCustomHealthCheckInCompose(ComposeService),

    /// A temporary directory error
    #[error(transparent)]
    TempDirError(#[from] TempDirError),

    /// A container error
    #[error(transparent)]
    ContainerError(#[from] ContainerError),

    /// Fail to run command error
    #[error(transparent)]
    CommandError(#[from] crate::cmd::CommandError),

    /// Cannot extract compose service state
    #[error("Cannot extract compose service state because {source}\njson: {json}")]
    CannotParseComposeServiceState {
        /// The json
        json: String,
        /// The source
        source: serde_json::Error,
    },

    /// Missing compose version
    #[error("Missing compose version")]
    MissingComposeVersion,
}

/// A temporary directory error
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TempDirError {
    /// Cannot create a temp. file from absolute path
    #[error("Cannot create a temporary file from absolute path {0:?}")]
    CannotCreateAbsoluteTempFile(PathBuf),

    /// Cannot override a temp. file from absolute path
    #[error("Cannot override a temporary file from absolute path {0:?}")]
    CannotOverrideTempFile(PathBuf),

    /// Cannot create directory
    #[error("Cannot create dir {dir:?} because {source}")]
    CannotCreateDir {
        /// The dir to create
        dir: PathBuf,
        /// The source
        source: std::io::Error,
    },

    /// Cannot write file
    #[error("Cannot write {file:?} because {source}")]
    CannotWriteFile {
        /// The file to write
        file: PathBuf,
        /// The source
        source: std::io::Error,
    },

    /// Cannot set permission
    #[error("Cannot write {file:?} because {source}")]
    CannotSetPermission {
        /// The file to write
        file: PathBuf,
        /// The source
        source: std::io::Error,
    },
}
