use std::num::ParseIntError;

/// Version parsing errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VersionError {
    /// Empty string
    #[error("Empty string")]
    Empty,

    /// Require major & minor version
    #[error("Require at least <major>.<minor>")]
    RequireMajorMinor,

    /// Invalid major version
    #[error("Invalid major version because {0}")]
    InvalidMajorVersion(ParseIntError),

    /// Invalid minor version
    #[error("Invalid minor version because {0}")]
    InvalidMinorVersion(ParseIntError),

    /// Invalid patch version
    #[error("Invalid patch version because {0}")]
    InvalidPatchVersion(ParseIntError),
}

/// Id errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IdError {
    /// Empty
    #[error("Id is empty")]
    Empty,

    /// Invalid id
    #[error("Id '{value}' is invalid because {source}")]
    InvalidId {
        /// The invalid value
        value: String,
        /// The source
        source: hex::FromHexError,
    },

    /// Too long
    #[error("Id '{0}' is too long (maximum length is 64)")]
    TooLong(String),
}
