use std::fmt::{self, Display};
use std::ops::Deref;

use tracing::{error, info};

use crate::compose::ToRunnableComposeContainers;
use crate::runner::Runner;

/// A running compose containers
///
/// If not detached, on drop stop the containers
#[derive(Debug, Clone)]
pub struct ComposeContainers<I>
where
    I: ToRunnableComposeContainers,
{
    pub(crate) runner: Runner,
    pub(crate) name: String,
    pub(crate) images: I,
    pub(crate) file: I::AsPath,
    pub(crate) detached: bool,
}

impl<I> Deref for ComposeContainers<I>
where
    I: ToRunnableComposeContainers,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.images
    }
}

impl<I> ComposeContainers<I>
where
    I: ToRunnableComposeContainers,
{
    /// Detach the container
    ///
    /// A detached container won't be stopped during the drop.
    pub fn detach(&mut self) {
        self.detached = true;
    }
}

impl<I> Drop for ComposeContainers<I>
where
    I: ToRunnableComposeContainers,
{
    fn drop(&mut self) {
        let name = &self.name;
        if self.detached {
            info!(%name, "Detached compose containers {self} is NOT stopped");
            return;
        }

        info!(%name, "🚮 Stopping compose containers");
        if let Err(e) = self.runner.compose_stop(&self.name, self.file.as_ref()) {
            error!(%name, "Fail to stop compose containers {self} because {e}");
        }
    }
}

impl<I> Display for ComposeContainers<I>
where
    I: ToRunnableComposeContainers,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
