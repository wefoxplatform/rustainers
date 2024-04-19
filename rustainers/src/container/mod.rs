use std::fmt::{self, Debug, Display};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tracing::{error, info};

use crate::runner::Runner;
use crate::ImageReference;

mod id;
pub use self::id::*;

mod health_check;
pub use self::health_check::*;

mod runnable;
pub use self::runnable::*;

mod process;
pub(crate) use self::process::ContainerProcess;

mod wait_condition;
pub use self::wait_condition::*;

mod network;
pub use self::network::*;

mod health;
pub(crate) use self::health::ContainerHealth;

mod state;
pub use self::state::*;

mod volume;
pub use self::volume::*;

/// A running container
///
/// It implements [`std::ops::Deref`] for the image.
///
/// When it's dropped, by default it's stopping the container,
/// but you can choose to keep alive this container by calling [`Container::detach`](Self::detach)
#[derive(Debug)]
pub struct Container<I>
where
    I: ToRunnableContainer,
{
    pub(crate) runner: Runner,
    pub(crate) id: ContainerId,
    pub(crate) image: I,
    pub(crate) image_ref: ImageReference,

    pub(crate) detached: Arc<AtomicBool>,
}

impl<I> Container<I>
where
    I: ToRunnableContainer,
{
    /// The container id
    pub fn id(&self) -> ContainerId {
        self.id
    }

    /// Detach the container
    ///
    /// A detached container won't be stopped during the drop.
    pub fn detach(&self) {
        self.detached.store(true, Ordering::Release);
    }
}

impl<I> Deref for Container<I>
where
    I: ToRunnableContainer,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl<I> Drop for Container<I>
where
    I: ToRunnableContainer,
{
    fn drop(&mut self) {
        let detached = self.detached.load(Ordering::Acquire);
        if detached {
            info!("Detached container {self} is NOT stopped");
            return;
        }

        info!("🚮 Stopping container");
        if let Err(err) = self.runner.stop(self) {
            error!("Fail to stop the container {self} because {err}");
        }
    }
}

impl<I> Display for Container<I>
where
    I: ToRunnableContainer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.image_ref, self.id)
    }
}
