use std::time::Duration;

use crate::{
    Container, ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const POSTGRES_IMAGE: &ImageName = &ImageName::new("docker.io/postgres");

const PORT: Port = Port(5432);

/// The default postgres user
const POSTGRES_USER: &str = "postgres";

/// The default postgres password
const POSTGRES_PASSWORD: &str = "passwd";

/// The default postgres database
const POSTGRES_DATABASE: &str = POSTGRES_USER;

/// A `PostgreSQL` image
///
/// # Example
///
/// ```rust, no_run
/// # async fn run() -> anyhow::Result<()> {
/// use rustainers::images::Postgres;
///
/// let default_image = Postgres::default();
///
/// let custom_image = Postgres::default()
///        .with_tag("15.2")
///        .with_db("plop");
///
/// # let runner = rustainers::runner::Runner::auto()?;
/// // ...
/// let container = runner.start(default_image).await?;
/// let url = container.url().await?;
/// // ...
/// # Ok(())
/// # }
///```
#[derive(Debug)]
pub struct Postgres {
    image: ImageName,
    user: String,
    password: String,
    db: String,
    port: ExposedPort,
}

impl Postgres {
    /// Set the image tag
    #[must_use]
    pub fn with_tag(self, tag: impl Into<String>) -> Self {
        let Self { mut image, .. } = self;
        image.set_tag(tag);
        Self { image, ..self }
    }

    /// Set the image digest
    #[must_use]
    pub fn with_digest(self, digest: impl Into<String>) -> Self {
        let Self { mut image, .. } = self;
        image.set_digest(digest);
        Self { image, ..self }
    }

    /// Set the database user
    #[must_use]
    pub fn with_user(self, user: impl Into<String>) -> Self {
        let user = user.into();
        Self { user, ..self }
    }

    /// Set the database password
    #[must_use]
    pub fn with_password(self, password: impl Into<String>) -> Self {
        let password = password.into();
        Self { password, ..self }
    }

    /// Set the database db
    #[must_use]
    pub fn with_db(self, db: impl Into<String>) -> Self {
        let db = db.into();
        Self { db, ..self }
    }

    /// Set the port mapping
    #[must_use]
    pub fn with_port(mut self, port: ExposedPort) -> Self {
        self.port = port;
        self
    }
}

impl Default for Postgres {
    fn default() -> Self {
        Self {
            image: POSTGRES_IMAGE.clone(),
            user: String::from(POSTGRES_USER),
            password: String::from(POSTGRES_PASSWORD),
            db: String::from(POSTGRES_DATABASE),
            port: ExposedPort::new(PORT),
        }
    }
}

impl Container<Postgres> {
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn config(&self) -> Result<String, PortError> {
        let user = &self.user;
        let password = &self.password;
        let host_ip = self.runner.container_host_ip().await?;
        let port = self.port.host_port().await?;
        let database = &self.db;
        let config =
            format!("host={host_ip} user={user} password={password} port={port} dbname={database}");
        Ok(config)
    }

    /// Get connection URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn url(&self) -> Result<String, PortError> {
        let user = &self.user;
        let password = &self.password;
        let port = self.port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let database = &self.db;
        let url = format!("postgresql://{user}:{password}@{host_ip}:{port}/{database}");
        Ok(url)
    }
}
impl ToRunnableContainer for Postgres {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_wait_strategy({
                let db = &self.db;
                let user = &self.user;
                HealthCheck::builder()
                    .with_command(format!("pg_isready --dbname={db} --username={user}"))
                    .with_interval(Duration::from_millis(250))
                    .build()
            })
            .with_env([
                ("POSTGRES_USER", &self.user),
                ("POSTGRES_PASSWORD", &self.password),
                ("POSTGRES_DB", &self.db),
            ])
            .with_port_mappings([self.port.clone()])
            .build()
    }
}
