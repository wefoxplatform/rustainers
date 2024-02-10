use std::path::{Path, PathBuf};

use tracing::info;

use crate::images::Alpine;
use crate::runner::{RunOption, Runner, RunnerError};
use crate::{Volume, VolumeName};

/// Copy errors
#[derive(Debug, thiserror::Error)]
pub enum CopyError {
    /// A runner error
    #[error(transparent)]
    RunnerError(#[from] RunnerError),

    /// Path does not exist
    #[error("Path {0:?} doest not exists in host")]
    PathNotExists(PathBuf),

    /// Path without parent
    #[error("Path {0:?} doest not have parent")]
    PathWithoutParent(PathBuf),

    /// Path without Name
    #[error("Path {0:?} doest not have a name")]
    PathWithoutName(PathBuf),
}

impl Runner {
    /// Copy a file or a folder into a volume
    ///
    /// # Errors
    ///
    /// Fail if the path does not exists
    /// Fail if the path does not have a parent
    /// Fail if the path does not have a name
    /// Fail if the we cannot launch the copy into the containers
    #[tracing::instrument(skip(self, path), fields(path = ?path.as_ref()))]
    pub async fn copy_to_volume(
        &self,
        volume: VolumeName,
        path: impl AsRef<Path>,
    ) -> Result<(), CopyError> {
        let path = path.as_ref();
        // Check the path
        if !path.exists() {
            return Err(CopyError::PathNotExists(path.to_path_buf()));
        }
        let Some(parent) = path.parent() else {
            return Err(CopyError::PathWithoutParent(path.to_path_buf()));
        };
        let Some(file_name) = path.file_name().and_then(|it| it.to_str()) else {
            return Err(CopyError::PathWithoutName(path.to_path_buf()));
        };

        // The copy command
        let source = format!("/source/{file_name}");
        let cmds = if path.is_dir() {
            vec!["cp", "-R", &source, "/dest"]
        } else {
            vec!["cp", &source, "/dest"]
        };

        // Run the copy
        let options = RunOption::builder()
            .with_volumes([
                Volume::bind_mount(parent, "/source"),
                Volume::container_volume(volume.clone(), "/dest"),
            ])
            .with_command(cmds)
            .build();
        let _container = self.start_with_options(Alpine, options).await?;
        info!("{path:?} copied into {volume}");

        Ok(())
    }
}
