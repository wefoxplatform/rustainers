use std::fs::Permissions;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::fs;
use tracing::{info, warn};
use typed_builder::TypedBuilder;
use ulid::Ulid;

use super::TempDirError;

/// A temporary file into a temporary director
///
/// # Example
///
/// ```rust, no_run
/// # use rustainers::compose::TemporaryFile;
/// let tmp_file = TemporaryFile::builder()
///     .with_path("docker-compose.yaml")
///     .with_content(r#"..."#)
///     .build();
/// ```
#[derive(Debug, TypedBuilder)]
#[builder(field_defaults(setter(prefix = "with_")))]
pub struct TemporaryFile {
    #[builder(setter(transform = |path: impl AsRef<Path>| path.as_ref().to_path_buf()))]
    path: PathBuf,

    #[builder(setter(transform = |content: impl AsRef<[u8]>| content.as_ref().to_vec()))]
    content: Vec<u8>,

    #[builder(default, setter(strip_option))]
    permissions: Option<Permissions>,
}

/// A temporary directory
///
/// The temporary directory is created with the [`std::env::temp_dir`] function.
///
/// The directory is removed during the drop.
/// You can opt-in to keep the folder if you call [`detach`](Self::detach)
///
/// # Example
///
/// ```rust, no_run
/// # use rustainers::compose::{TemporaryDirectory, TemporaryFile};
/// # async fn build() {
/// let temp_dir = TemporaryDirectory::with_files(
///     "componse-nginx",
///     [TemporaryFile::builder()
///         .with_path("docker-compose.yaml")
///         .with_content(r#"..."#)
///         .build()],
/// ).await.unwrap();
/// # }
/// ```
///
/// See [existings compose images](https://github.com/wefoxplatform/rustainers/tree/main/rustainers/src/compose/images/)
/// for real usages.
#[derive(Debug, Clone)]
pub struct TemporaryDirectory(PathBuf, Arc<AtomicBool>);

impl TemporaryDirectory {
    /// Create a new empty temporary directory
    ///
    /// # Errors
    ///
    /// Fail if the directory cannot be created
    pub async fn new(prefix: &str) -> Result<Self, TempDirError> {
        let mktemp = || {
            let mut tmp = std::env::temp_dir();
            let folder = format!("tc_{prefix}_{}", Ulid::new());
            tmp.push(&folder);
            tmp
        };

        let mut tmp = mktemp();
        loop {
            if !tmp.exists() {
                break;
            }
            warn!("Oops, try {tmp:?} temp. dir. but it already exists, retry");
            tmp = mktemp();
        }

        Self::mkdirp(&tmp).await?;
        Ok(Self(tmp, Arc::new(AtomicBool::new(false))))
    }

    /// Create a new empty temporary directory with some files
    ///
    /// # Errors
    ///
    /// Fail if we cannot create the directory
    /// Fail if we cannot create a file
    #[allow(clippy::missing_panics_doc)]
    pub async fn with_files(
        prefix: &str,
        files: impl IntoIterator<Item = impl Into<TemporaryFile>>,
    ) -> Result<Self, TempDirError> {
        let result = Self::new(prefix).await?;
        let root_dir = result.as_ref().to_path_buf();
        Self::mkdirp(&root_dir).await?;

        for temp_file in files {
            let temp_file: TemporaryFile = temp_file.into();
            // Ensure the path is not absolute (otherwise we could write)
            if temp_file.path.is_absolute() {
                return Err(TempDirError::CannotCreateAbsoluteTempFile(
                    temp_file.path.clone(),
                ));
            }
            let mut file = root_dir.clone();
            file.push(&temp_file.path);

            if file.exists() {
                return Err(TempDirError::CannotOverrideTempFile(temp_file.path.clone()));
            }
            fs::write(&file, temp_file.content)
                .await
                .map_err(|source| TempDirError::CannotWriteFile {
                    file: file.clone(),
                    source,
                })?;

            if let Some(perm) = temp_file.permissions {
                fs::set_permissions(&file, perm).await.map_err(|source| {
                    TempDirError::CannotSetPermission {
                        file: file.clone(),
                        source,
                    }
                })?;
            }
        }

        Ok(result)
    }

    async fn mkdirp(dir: &Path) -> Result<(), TempDirError> {
        fs::create_dir_all(dir)
            .await
            .map_err(|source| TempDirError::CannotCreateDir {
                dir: dir.to_path_buf(),
                source,
            })?;
        info!("Temporary directory {dir:?} created");

        Ok(())
    }

    // TODO create from an existing path, with keeping the permissions

    /// Detach the temp. directory.
    ///
    /// A detached [`TemporaryDirectory`] is not removed during drop.
    pub fn detach(&self) {
        self.1.store(true, Ordering::Release);
    }
}

impl AsRef<Path> for TemporaryDirectory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let detached = self.1.load(Ordering::Acquire);
        if !detached && self.0.exists() {
            if let Err(err) = std::fs::remove_dir_all(&self.0) {
                warn!("Fail to clean up temporary dir {:?} because {err}", self.0);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use std::mem;

    use assert2::check;

    use super::*;

    #[tokio::test]
    async fn should_create_dir() {
        _ = tracing_subscriber::fmt::try_init();

        let plop = TemporaryDirectory::new("plop").await.expect("temp. dir.");

        // Check directory exists
        let path = plop.as_ref().to_path_buf();
        assert!(path.exists());
        assert!(path.is_dir());

        // Check the directory is remove on drop
        mem::drop(plop);
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn should_create_dir_with_file() {
        _ = tracing_subscriber::fmt::try_init();
        let content = "plop";

        let plop = TemporaryDirectory::with_files(
            "plop",
            [TemporaryFile::builder()
                .with_path("plop.txt")
                .with_content(content)
                .build()],
        )
        .await
        .expect("temp. dir.");

        // Check directory exists
        let path = plop.as_ref().to_path_buf();
        assert!(path.exists());
        assert!(path.is_dir());

        // Check file exists
        let mut child = path.clone();
        child.push("plop.txt");
        assert!(child.exists());
        assert!(child.is_file());
        let child_content = fs::read_to_string(child).await.expect("file content");
        check!(child_content == "plop");

        // Check the directory is remove on drop
        mem::drop(plop);
        assert!(!path.exists());
    }
}
