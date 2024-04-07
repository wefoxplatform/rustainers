use std::fmt::{self, Debug, Display};
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tracing::info;

use crate::{Container, Network, RunnableContainer, ToRunnableContainer, VolumeName};

mod docker;
pub use self::docker::Docker;

mod nerdctl;
pub use self::nerdctl::Nerdctl;

mod podman;
pub use self::podman::Podman;

mod error;
pub use self::error::*;

mod inner;
pub(crate) use self::inner::*;

mod options;
pub use self::options::*;

mod runner_network;
pub use self::runner_network::*;

/// The test containers runner
///
/// Use the [`Runner::auto`], [`Runner::docker`], [`Runner::podman`], [`Runner::nerdctl`] functions
///  to create your runner
// Note: we do not derive Copy to avoid a future breaking-change if add another implementation
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Runner {
    /// Docker
    Docker(Docker),

    /// Podman
    Podman(Podman),

    /// Nerdctl
    Nerdctl(Nerdctl),
}

impl Display for Runner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Docker(runner) => write!(f, "{runner}"),
            Self::Podman(runner) => write!(f, "{runner}"),
            Self::Nerdctl(runner) => write!(f, "{runner}"),
        }
    }
}

impl Runner {
    /// Find an available runner
    ///
    /// # Errors
    ///
    /// Fail if no runner is available
    ///
    pub fn auto() -> Result<Self, RunnerError> {
        // Check Docker
        if let Ok(runner) = Self::docker() {
            info!("🐳 Using docker");
            return Ok(runner);
        }

        // Check Podman
        if let Ok(runner) = Self::podman() {
            info!("Using podman");
            return Ok(runner);
        }

        // Check nerdctl
        if let Ok(runner) = Self::nerdctl() {
            info!("Using nerdctl");
            return Ok(runner);
        }

        // Fallback
        Err(RunnerError::NoRunnerAvailable)
    }

    /// Create a docker runner
    ///
    /// # Errors
    ///
    /// Fail if the docker command is not found
    /// Fail if the docker command version is unsupported
    pub fn docker() -> Result<Self, RunnerError> {
        let runner = docker::create()?;
        Ok(Self::Docker(runner))
    }

    /// Create a podman runner
    ///
    /// # Errors
    ///
    /// Fail if the podman command is not found
    /// Fail if the podman command version is unsupported
    pub fn podman() -> Result<Self, RunnerError> {
        let runner = podman::create()?;
        Ok(Self::Podman(runner))
    }

    /// Create a nerdctl runner
    ///
    /// # Errors
    ///
    /// Fail if the nerdctl command is not found
    /// Fail if the nerdctl command version is unsupported
    pub fn nerdctl() -> Result<Self, RunnerError> {
        let runner = nerdctl::create()?;
        Ok(Self::Nerdctl(runner))
    }
}

impl Runner {
    /// Start a runnable container
    ///
    /// The default [`RunOption`] is used
    ///
    /// # Errors
    ///
    /// Fail if we cannot launch the container
    pub async fn start<I>(&self, image: I) -> Result<Container<I>, RunnerError>
    where
        I: ToRunnableContainer,
    {
        let options = RunOption::default();
        self.start_with_options(image, options).await
    }

    /// Start a runnable container with option
    ///
    /// # Errors
    ///
    /// Fail if we cannot launch the container
    pub async fn start_with_options<I>(
        &self,
        image: I,
        options: RunOption,
    ) -> Result<Container<I>, RunnerError>
    where
        I: ToRunnableContainer,
    {
        let mut container = image.to_runnable(RunnableContainer::builder());
        let image_ref = container.image.clone();

        let id = match self {
            Self::Docker(runner) => runner.start_container(&mut container, options).await,
            Self::Podman(runner) => runner.start_container(&mut container, options).await,
            Self::Nerdctl(runner) => runner.start_container(&mut container, options).await,
        }
        .map_err(|source| RunnerError::StartError {
            runner: self.clone(),
            container: Box::new(container),
            source: Box::new(source),
        })?;

        Ok(Container {
            runner: self.clone(),
            image,
            image_ref,
            id,
            detached: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Create a network
    ///
    /// # Errors
    ///
    /// Could fail if we cannot execute the command
    pub async fn create_network(&self, name: impl Into<String>) -> Result<Network, RunnerError> {
        let name = name.into();
        match self {
            Self::Docker(runner) => runner.create_network(&name).await,
            Self::Podman(runner) => runner.create_network(&name).await,
            Self::Nerdctl(runner) => runner.create_network(&name).await,
        }
        .map_err(|source| RunnerError::CreateNetworkError {
            runner: self.clone(),
            name: name.clone(),
            source: Box::new(source),
        })?;

        Ok(Network::Custom(name))
    }

    /// List networks
    ///
    /// # Errors
    ///
    /// Could fail if we cannot execute the command
    pub async fn list_networks(&self) -> Result<Vec<RunnerNetwork>, RunnerError> {
        let result = match self {
            Self::Docker(runner) => runner.list_networks().await,
            Self::Podman(runner) => runner.list_networks().await,
            Self::Nerdctl(runner) => runner.list_networks().await,
        }
        .map_err(|source| RunnerError::ListNetworkError {
            runner: self.clone(),
            source: Box::new(source),
        })?;

        Ok(result)
    }

    /// Create a container volume
    ///
    /// # Errors
    ///
    /// Could fail if we cannot execute the command
    pub async fn create_volume(&self, name: impl Into<String>) -> Result<VolumeName, RunnerError> {
        let name = name.into();
        match self {
            Self::Docker(runner) => runner.create_volume(&name).await,
            Self::Podman(runner) => runner.create_volume(&name).await,
            Self::Nerdctl(runner) => runner.create_volume(&name).await,
        }
        .map_err(|source| RunnerError::CreateVolumeError {
            runner: self.clone(),
            name: name.clone(),
            source: Box::new(source),
        })?;

        Ok(VolumeName(name))
    }

    fn guard_runner<I>(&self, container: &Container<I>) -> Result<(), RunnerError>
    where
        I: ToRunnableContainer,
    {
        if &container.runner != self {
            return Err(RunnerError::DifferentRunner {
                runner: self.clone(),
                container_runner: Box::new(container.runner.clone()),
            });
        }
        Ok(())
    }

    /// Get the container IP for a custom network
    ///
    /// # Errors
    ///
    /// Fail if the network is not custom
    /// Fail if the IP is not found
    /// Could fail if we cannot execute the inspect command
    pub async fn network_ip<I>(
        &self,
        container: &Container<I>,
        network: &Network,
    ) -> Result<Ipv4Addr, RunnerError>
    where
        I: ToRunnableContainer,
    {
        self.guard_runner(container)?;

        let id = container.id;
        let Some(net) = network.name() else {
            return Err(RunnerError::ExpectedNetworkNameError {
                runner: self.clone(),
                network: Box::new(network.clone()),
                container: id,
            });
        };

        let container_network = match self {
            Self::Docker(runner) => runner.network_ip(id, net).await,
            Self::Podman(runner) => runner.network_ip(id, net).await,
            Self::Nerdctl(runner) => runner.network_ip(id, net).await,
        }
        .map_err(|source| RunnerError::FindNetworkIpError {
            runner: self.clone(),
            network: Box::new(network.clone()),
            container: Box::new(id),
            source: Box::new(source),
        })?;

        let Some(ip) = container_network.ip_address else {
            return Err(RunnerError::NoNetworkIp {
                runner: self.clone(),
                network: Box::new(network.clone()),
                container: id,
            });
        };
        Ok(ip.0)
    }

    /// Execute a command into the container
    ///
    /// # Errors
    ///
    /// Could fail if we cannot execute the command
    pub async fn exec<I, S>(
        &self,
        container: &Container<I>,
        exec_command: impl IntoIterator<Item = S> + Debug,
    ) -> Result<String, RunnerError>
    where
        S: Into<String>,
        I: ToRunnableContainer,
    {
        self.guard_runner(container)?;

        let id = container.id;
        let exec_command = exec_command.into_iter().map(Into::into).collect();
        match self {
            Self::Docker(runner) => runner.exec(id, exec_command).await,
            Self::Podman(runner) => runner.exec(id, exec_command).await,
            Self::Nerdctl(runner) => runner.exec(id, exec_command).await,
        }
        .map_err(|source| RunnerError::ExecError {
            runner: self.clone(),
            id,
            source: Box::new(source),
        })
    }

    /// Stop the container
    ///
    /// This method is call during the [`crate::Container`] drop if it's not detached
    ///
    /// # Errors
    ///
    /// Fail if we cannot launch the container
    pub fn stop<I>(&self, container: &Container<I>) -> Result<(), RunnerError>
    where
        I: ToRunnableContainer,
    {
        self.guard_runner(container)?;

        let id = container.id;
        match self {
            Self::Docker(runner) => runner.stop(id),
            Self::Podman(runner) => runner.stop(id),
            Self::Nerdctl(runner) => runner.stop(id),
        }
        .map_err(|source| RunnerError::StopError {
            runner: self.clone(),
            id,
            source: Box::new(source),
        })
    }
}
