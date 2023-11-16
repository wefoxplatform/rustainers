use std::path::PathBuf;

use crate::cmd::CommandError;
use crate::version::Version;
use crate::{ContainerId, IdError, Port, RunnableContainer};

use super::Runner;

/// Runner errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RunnerError {
    /// Command not available
    #[error("Command '{0}' not available")]
    CommandNotAvailable(String),

    /// Unsupported version
    #[error("{command} version {current} expected to be ≥ that {minimal}")]
    UnsupportedVersion {
        /// The command
        command: String,
        /// The current version
        current: Version,
        /// The minimal version
        minimal: Version,
    },

    /// Unable to find an available runner
    #[error("No runner available")]
    NoRunnerAvailable,

    /// Fail to start a container
    #[error(
        "Fail to start container because {source}\nrunner: {runner}\ncontainer: {container:#?}"
    )]
    StartError {
        /// The runner
        runner: Runner,
        /// The runnable container
        container: Box<RunnableContainer>,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to exec a container
    #[error("Fail to execute command in container {id} because {source}\nrunner: {runner}")]
    ExecError {
        /// The runner
        runner: Runner,
        /// The container id
        id: ContainerId,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to stop a container
    #[error("Fail to stop container {id} because {source}\nrunner: {runner}")]
    StopError {
        /// The runner
        runner: Runner,
        /// The container id
        id: ContainerId,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to run compose
    #[error("Fail run compose in {path:?} because {source}\nrunner: {runner}")]
    ComposeError {
        /// The runner
        runner: Runner,
        /// The path containing the compose file
        path: PathBuf,
        /// The source error
        source: Box<crate::compose::ComposeError>,
    },
}

/// Errors that could happen during creation of a container
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ContainerError {
    /// Port not found
    #[error("Cannot find host port for {id} and container port {container_port}")]
    PortNotFound {
        /// The container id
        id: ContainerId,
        /// The container port
        container_port: Port,
    },

    /// Fail to start a container
    #[error("Container '{0}' cannot be started")]
    ContainerCannotBeStarted(ContainerId),

    /// Fail to resume a container
    #[error("Container '{0}' cannot be resumed (unpause)")]
    ContainerCannotBeResumed(ContainerId),

    /// Invalid container state
    #[error("Container {0} state {1:?} is unexpected")]
    InvalidContainerState(ContainerId, String),

    /// The container is not healthy
    #[error("Container {0} is unhealthy")]
    UnhealthyContainer(ContainerId),

    /// Invalid container health
    #[error("Container {0} does not have a health check")]
    UnknownContainerHealth(ContainerId),

    /// Fail to remove a container
    #[error("Container '{0}' cannot be removed")]
    ContainerCannotBeRemoved(ContainerId),

    /// Fail to run error
    #[error(transparent)]
    CommandError(#[from] CommandError),

    /// Id error
    #[error(transparent)]
    IdError(#[from] IdError),
}
