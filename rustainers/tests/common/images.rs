use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::bail;
use rustainers::runner::{RunOption, Runner};
use rustainers::{
    ContainerStatus, ExposedPort, HealthCheck, ImageName, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer, Volume, VolumeName, WaitStrategy,
};
use tracing::info;

// A web server only accessible when sharing the same network
#[derive(Debug)]
pub struct InternalWebServer;

impl ToRunnableContainer for InternalWebServer {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("nginx"))
            .with_wait_strategy(
                HealthCheck::builder()
                    .with_command("curl -sf http://localhost") //DevSkim: ignore DS137138
                    .build(),
            )
            .build()
    }
}

// Curl
#[derive(Debug)]
struct Curl {
    url: String,
}

pub async fn curl(
    runner: &Runner,
    url: impl Into<String>,
    options: RunOption,
) -> anyhow::Result<()> {
    let url = url.into();
    let image = Curl { url };
    let _container = runner.start_with_options(image, options).await?;

    Ok(())
}

impl ToRunnableContainer for Curl {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("curlimages/curl"))
            .with_wait_strategy(WaitStrategy::State(ContainerStatus::Exited))
            .with_command(["-fsv", "--connect-timeout", "1", &self.url])
            .build()
    }
}

// A web server accessible from host
#[derive(Debug)]
pub struct WebServer(ExposedPort);

impl Default for WebServer {
    fn default() -> Self {
        Self(ExposedPort::new(80))
    }
}

impl ToRunnableContainer for WebServer {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("nginx"))
            .with_port_mappings([self.0.clone()])
            .with_wait_strategy(
                HealthCheck::builder()
                    .with_command("curl -sf http://localhost") //DevSkim: ignore DS137138
                    .build(),
            )
            .build()
    }
}

impl WebServer {
    // Container path that contains the static HTML pages
    pub const STATIC_HTML: &'static str = "/usr/share/nginx/html";

    /// Get the text content
    pub async fn get(&self, path: &str) -> anyhow::Result<String> {
        let port = self.0.host_port().await?;
        let url = format!("http://localhost:{port}/{}", path.trim_start_matches('/')); //DevSkim: ignore DS137138
        let out = Command::new("curl").arg(&url).output()?;
        let result = String::from_utf8_lossy(&out.stdout);
        Ok(result.to_string())
    }
}

// A helper image to copy file into a named volume
#[derive(Debug)]
struct CopyFile {
    parent: PathBuf,
    file_name: String,
}

impl CopyFile {
    fn build(path: &Path) -> anyhow::Result<Self> {
        // TODO maybe relaxe this constraint to allow copy a folder ?
        if !path.is_file() {
            bail!("Expected {path:?} to be a file");
        }
        if !path.exists() {
            bail!("Expected {path:?} to exists");
        }
        let Some(parent) = path.parent() else {
            bail!("Expected {path:?} to have a parent");
        };
        let Some(file_name) = path.file_name().and_then(|it| it.to_str()) else {
            bail!("Expected {path:?} to have a file name");
        };

        let parent = parent.to_path_buf();
        let file_name = file_name.to_string();
        let image = Self { parent, file_name };
        dbg!(&image);
        Ok(image)
    }
}

impl ToRunnableContainer for CopyFile {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        let source = format!("/source/{}", self.file_name);
        builder
            .with_image(ImageName::new("alpine"))
            .with_wait_strategy(WaitStrategy::None)
            .with_command(["cp", &source, "/dest"])
            .build()
    }
}

// TODO maybe create a more general copy_to_volume function in the lib
// ```sh
// docker container create --name dummy -v myvolume:/root hello-world
// docker cp c:\myfolder\myfile.txt dummy:/root/myfile.txt
// docker rm dummy
/// ```

pub async fn copy_file_to_volume(
    runner: &Runner,
    volume: VolumeName,
    file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let file = file.as_ref();
    let image = CopyFile::build(file)?;
    let parent = image.parent.clone();

    let options = RunOption::builder()
        .with_volumes([
            Volume::bind_mount(parent, "/source"),
            Volume::container_volume(volume.clone(), "/dest"),
        ])
        .build();
    let _container = runner.start_with_options(image, options).await?;
    info!("File {file:?} copied into {volume}");

    Ok(())
}

#[derive(Debug)]
pub struct Alpine;

impl ToRunnableContainer for Alpine {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("alpine"))
            .with_wait_strategy(WaitStrategy::None)
            // keep the container alive
            .with_command(["tail", "-f", "/dev/null"])
            .build()
    }
}
