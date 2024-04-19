use std::fmt::{self, Display};
use std::ops::Deref;

use tracing::{error, info};

use crate::compose::ToRunnableComposeContainers;
use crate::runner::Runner;

/// A running compose containers
///
/// It implements [`std::ops::Deref`] for the images.
///
/// When it's dropped, by default it's stopping containers,
/// but you can choose to keep alive those containers by calling [`ComposeContainers::detach`](Self::detach)
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
        if let Err(err) = self.runner.compose_stop(&self.name, self.file.as_ref()) {
            error!(%name, "Fail to stop compose containers {self} because {err}");
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
