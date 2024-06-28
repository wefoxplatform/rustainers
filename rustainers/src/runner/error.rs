use std::env::VarError;
use std::path::PathBuf;

use crate::cmd::CommandError;
use crate::version::Version;
use crate::{ContainerId, IdError, Network, Port, RunnableContainer, VolumeError, WaitStrategy};

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

    /// Fail to create a network
    #[error("Fail to create network '{name}' because {source}\nrunner: {runner}")]
    CreateNetworkError {
        /// The runner
        runner: Runner,
        /// The network name
        name: String,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to create a volume
    #[error("Fail to create volume'{name}' because {source}\nrunner: {runner}")]
    CreateVolumeError {
        /// The runner
        runner: Runner,
        /// The volume name
        name: String,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to retrieve container IP in a specific network
    #[error("Fail to inspect container {container} networks because {source}\nrunner: {runner}")]
    InspectNetworkError {
        /// The runner
        runner: Runner,

        /// The container,
        container: Box<ContainerId>,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to retrieve container IP in a specific network
    #[error("Fail to list network because {source}\nrunner: {runner}")]
    ListNetworkError {
        /// The runner
        runner: Runner,

        /// The source error
        source: Box<ContainerError>,
    },

    /// Fail to retrieve network IP
    #[error("Fail to retrieve network named '{network}' IP for container {container} because {source}\nrunner: {runner}")]
    FindNetworkIpError {
        /// The runner
        runner: Runner,
        /// The network
        network: Box<Network>,
        /// The container,
        container: Box<ContainerId>,
        /// The source error
        source: Box<ContainerError>,
    },

    /// Expected a network name
    #[error("Fail to retrieve container {container} IP because we expect a network with name, got {network}")]
    ExpectedNetworkNameError {
        /// The runner
        runner: Runner,
        /// The network
        network: Box<Network>,
        /// The container,
        container: ContainerId,
    },

    /// No IP found
    #[error("No IP found for container {container} and network {network}")]
    NoNetworkIp {
        /// The runner
        runner: Runner,
        /// The network
        network: Box<Network>,
        /// The container,
        container: ContainerId,
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

    /// Fail to retrieve host ip address
    #[error("Can not fetch host because {source}\nrunner: {runner}")]
    HostIpError {
        /// The runner
        runner: Runner,
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

    /// Different runner
    #[error("The operation need to be done with the same runner\ncurrent: {runner}\ncontainer runner: {container_runner}")]
    DifferentRunner {
        /// The current runner
        runner: Runner,
        /// The container runner
        container_runner: Box<Runner>,
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

    /// The container cannot reach wait condition
    #[error("Container {0} cannot reach wait condition {1}")]
    WaitConditionUnreachable(ContainerId, WaitStrategy),

    /// Fail to run error
    #[error(transparent)]
    CommandError(#[from] CommandError),

    /// Id error
    #[error(transparent)]
    IdError(#[from] IdError),

    /// Volume error
    #[error(transparent)]
    VolumeError(#[from] VolumeError),

    /// Environment variable error
    #[error(transparent)]
    EnvVarError(#[from] VarError),

    /// No gateway error
    #[error("No gateway")]
    NoGateway,

    /// No network error
    #[error("No host network")]
    NoNetwork,
}
