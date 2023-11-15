use std::time::Duration;

use crate::{
    ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const POSTGRES_IMAGE: &ImageName = &ImageName::new("postgres");

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
/// let url = container.url()?;
/// // ...
/// # Ok(())
/// # }
///```
#[derive(Debug, Clone)]
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
        let Self {
            mut image,
            user,
            password,
            db,
            port,
        } = self;
        image.set_tag(tag);
        Self {
            image,
            user,
            password,
            db,
            port,
        }
    }

    /// Set the image digest
    #[must_use]
    pub fn with_digest(self, digest: impl Into<String>) -> Self {
        let Self {
            mut image,
            user,
            password,
            db,
            port,
        } = self;
        image.set_digest(digest);
        Self {
            image,
            user,
            password,
            db,
            port,
        }
    }

    /// Set the database user
    #[must_use]
    pub fn with_user(self, user: impl Into<String>) -> Self {
        let Self {
            image,
            password,
            db,
            port,
            ..
        } = self;
        let user = user.into();
        Self {
            image,
            user,
            password,
            db,
            port,
        }
    }

    /// Set the database password
    #[must_use]
    pub fn with_password(self, password: impl Into<String>) -> Self {
        let Self {
            image,
            user,
            db,
            port,
            ..
        } = self;
        let password = password.into();
        Self {
            image,
            user,
            password,
            db,
            port,
        }
    }

    /// Set the database db
    #[must_use]
    pub fn with_db(self, db: impl Into<String>) -> Self {
        let Self {
            image,
            user,
            password,
            port,
            ..
        } = self;
        let db = db.into();
        Self {
            image,
            user,
            password,
            db,
            port,
        }
    }
}

impl Postgres {
    /// Get connection URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub fn url(&self) -> Result<String, PortError> {
        let user = &self.user;
        let password = &self.password;
        let port = self.port.host_port()?;
        let database = &self.db;
        let url = format!("postgresql://{user}:{password}@localhost:{port}/{database}");
        Ok(url)
    }

    /// Get connection string
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub fn config(&self) -> Result<String, PortError> {
        let user = &self.user;
        let password = &self.password;
        let port = self.port.host_port()?;
        let database = &self.db;
        let config =
            format!("host=localhost user={user} password={password} port={port} dbname={database}");
        Ok(config)
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
            .with_port_mappings([self.port])
            .build()
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::check;

    use super::*;

    #[test]
    fn should_build_config() {
        let image = Postgres {
            port: ExposedPort::fixed(PORT, Port::new(5432)),
            ..Default::default()
        };
        let result = image.config().unwrap();
        check!(result == "host=localhost user=postgres password=passwd port=5432 dbname=postgres");
    }

    #[test]
    fn should_build_url() {
        let image = Postgres {
            port: ExposedPort::fixed(PORT, Port::new(5432)),
            ..Default::default()
        };
        let result = image.url().unwrap();
        check!(result == "postgresql://postgres:passwd@localhost:5432/postgres");
    }
}
