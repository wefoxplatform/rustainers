use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use path_absolutize::Absolutize;

use crate::VolumeError;

/// A Docker Volume name
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolumeName(pub(crate) String);

impl FromStr for VolumeName {
    type Err = VolumeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(VolumeError::EmptyVolumeName);
        }
        Ok(Self(s.to_string()))
    }
}

impl Display for VolumeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A volume
///
/// See [Docker reference](https://docs.docker.com/storage/volumes/)
///
/// # Examples
///
/// ```rust
/// # use rustainers::{Volume, VolumeName};
/// // Create a bind mount volumes
/// let v1 = Volume::from(("./data", "/etc/var/data"));
/// let v2 = Volume::bind_mount("./data", "/etc/var/data");
/// assert_eq!(v1, v2);
///
/// // Create a container volume
/// let name = "my-vol".parse::<VolumeName>().expect("a valid volume name");
/// let v1 = Volume::from((name.clone(), "/etc/var/data"));
/// let v2 = Volume::container_volume(name, "/etc/var/data");
/// assert_eq!(v1, v2);
///
/// // Create a tmpfs volume (in-memory)
/// let v = Volume::tmpfs("/etc/var/data");
/// assert!(matches!(v, Volume::Tmpfs{..}));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Volume {
    /// A containervolume
    ///
    /// See <https://docs.docker.com/storage/volumes/>
    ContainerVolume {
        /// The name
        name: VolumeName,
        /// The container path (target)
        container: PathBuf,
        /// If read-only
        readonly: bool,
    },

    /// A bind mount
    ///
    /// See <https://docs.docker.com/storage/bind-mounts/>
    BindMount {
        /// The container path (target)
        container: PathBuf,
        /// The host path (source)
        host: PathBuf,
        /// If read-only
        readonly: bool,
    },

    /// In-Memory mount
    ///
    /// ⚠️ WARNING, this is not supported by all platform
    /// See <https://docs.docker.com/storage/tmpfs>
    Tmpfs {
        /// The container path (target)
        container: PathBuf,
    },
}

impl Volume {
    /// Create a container volume
    pub fn container_volume(name: VolumeName, container: impl Into<PathBuf>) -> Self {
        let container = container.into();
        Self::ContainerVolume {
            name,
            container,
            readonly: false,
        }
    }

    /// Create a bind mount volume
    pub fn bind_mount(host: impl Into<PathBuf>, container: impl Into<PathBuf>) -> Self {
        let host = host.into();
        let container = container.into();
        Self::BindMount {
            host,
            container,
            readonly: false,
        }
    }

    /// Create a tmpfs volume
    pub fn tmpfs(container: impl Into<PathBuf>) -> Self {
        let container = container.into();
        Self::Tmpfs { container }
    }

    /// Make the volume readonly
    ///
    /// ⚠️ WARNING this is not supported by `Tmpfs`
    pub fn read_only(&mut self) {
        match self {
            Self::ContainerVolume { readonly, .. } | Self::BindMount { readonly, .. } => {
                *readonly = true;
            }
            Self::Tmpfs { .. } => {}
        }
    }

    pub(crate) fn mount_arg(&self) -> Result<String, VolumeError> {
        let arg = match self {
            Self::BindMount {
                container,
                host,
                readonly,
            } => {
                let src = host.absolutize()?;
                let trg = container.absolutize()?;
                let ro = if *readonly { ",readonly" } else { "" };
                format!(
                    "type=bind,source={},target={}{ro}",
                    src.to_string_lossy(),
                    trg.to_string_lossy()
                )
            }
            Self::ContainerVolume {
                name,
                container,
                readonly,
            } => {
                let trg = container.absolutize()?;
                let ro = if *readonly { ",readonly" } else { "" };
                format!(
                    "type=volume,source={name},target={}{ro}",
                    trg.to_string_lossy()
                )
            }
            Self::Tmpfs { container } => {
                let trg = container.absolutize()?;
                format!("type=tmpfs,target={}", trg.to_string_lossy())
            }
        };
        Ok(arg)
    }
}

impl<P, Q> From<(P, Q)> for Volume
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    fn from(value: (P, Q)) -> Self {
        let host = value.0.as_ref();
        let container = value.1.as_ref();
        Self::bind_mount(host, container)
    }
}

impl<P> From<(VolumeName, P)> for Volume
where
    P: AsRef<Path>,
{
    fn from(value: (VolumeName, P)) -> Self {
        let container = value.1.as_ref();
        Self::container_volume(value.0, container)
    }
}
