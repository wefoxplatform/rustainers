use std::fmt::{self, Debug, Display};

use tracing::info;

use crate::{Container, RunnableContainer, ToRunnableContainer};

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

/// The test containers runner
///
/// Use the [`Runner::auto`], [`Runner::docker`], [`Runner::podman`], [`Runner::nerdctl`] functions
///  to create your runner
// Note: we do not derive Copy to avoid a future breaking-change if add another implementation
#[derive(Debug, Clone)]
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
            detached: false,
        })
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
    ) -> Result<(), RunnerError>
    where
        S: Into<String>,
        I: ToRunnableContainer,
    {
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
