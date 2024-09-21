#![doc = include_str!("./doc.md")]

mod postgres;
use indexmap::IndexMap;

use crate::ContainerStatus;
use crate::ExposedPort;
use crate::ImageReference;
use crate::RunnableContainer;
use crate::RunnableContainerBuilder;
use crate::ToRunnableContainer;
use crate::WaitStrategy;

pub use self::postgres::*;

mod minio;
pub use self::minio::*;

mod redis;
pub use self::redis::*;

mod mongo;
pub use self::mongo::*;

mod alpine;
pub use self::alpine::*;

mod mosquitto;
pub use self::mosquitto::*;

mod nats;
pub use self::nats::*;

/// A Generic Image
#[derive(Debug)]
pub struct GenericImage(RunnableContainer);

impl GenericImage {
    /// Build a new generic image
    pub fn new(image: impl Into<ImageReference>) -> Self {
        let result = RunnableContainer {
            image: image.into(),
            container_name: None,
            command: vec![],
            env: IndexMap::default(),
            wait_strategy: WaitStrategy::State(ContainerStatus::Running),
            port_mappings: vec![],
        };
        Self(result)
    }

    /// Set the container name
    pub fn set_container_name(&mut self, name: impl Into<String>) {
        self.0.container_name = Some(name.into());
    }

    /// Set the command
    pub fn set_command<I, S>(&mut self, cmd: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.command = cmd.into_iter().map(Into::into).collect();
    }

    /// Add an environment variable
    pub fn add_env_var(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.env.insert(name.into(), value.into());
    }

    /// Set the wait strategy
    pub fn set_wait_strategy(&mut self, wait_strategy: impl Into<WaitStrategy>) {
        self.0.wait_strategy = wait_strategy.into();
    }

    /// Add a port to publish
    pub fn add_port_mapping(&mut self, container_port: u16) {
        let port = ExposedPort::new(container_port);
        self.0.port_mappings.push(port);
    }
}

impl ToRunnableContainer for GenericImage {
    fn to_runnable(&self, _builder: RunnableContainerBuilder) -> RunnableContainer {
        RunnableContainer {
            image: self.0.image.clone(),
            container_name: self.0.container_name.clone(),
            command: self.0.command.clone(),
            env: self.0.env.clone(),
            wait_strategy: self.0.wait_strategy.clone(),
            port_mappings: self.0.port_mappings.clone(),
        }
    }
}
