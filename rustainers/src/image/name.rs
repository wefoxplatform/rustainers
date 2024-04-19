use std::borrow::Cow;
use std::fmt::{self, Display};
use std::str::FromStr;

use super::ImageNameError;

/// An image name.
///
/// It contains the name, and optionally a tag or a digest.
///
/// # Example
///
/// Create an constant image
///
/// ```rust
/// # use rustainers::ImageName;
/// const POSTGRES_IMAGE: &ImageName = &ImageName::new("postgres");
///```
///
/// Parse an image name:
///
/// ```rust
/// # use rustainers::ImageName;
/// let image = "minio/minio".parse::<ImageName>().unwrap();
/// assert_eq!(image.to_string(), "minio/minio");
///
/// let image_with_tag = "postgres:15.2".parse::<ImageName>().unwrap();
/// assert_eq!(image_with_tag.to_string(), "postgres:15.2");
///
/// let image_with_digest = "redis@sha256:1f9f545dd3d396ee72ca4588d31168341247e46b7e70fabca82f88a809d407a8".parse::<ImageName>().unwrap();
/// assert_eq!(image_with_digest.to_string(), "redis@sha256:1f9f545dd3d396ee72ca4588d31168341247e46b7e70fabca82f88a809d407a8");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageName {
    /// The repository,
    /// it can include the registry if necessary
    repository: Cow<'static, str>,

    /// The image tag
    tag: Option<Cow<'static, str>>,

    /// The digest,
    /// it should include the `sha256:` prefix
    digest: Option<Cow<'static, str>>,
}

impl ImageName {
    /// Create a new image name
    #[must_use]
    pub const fn new(repository: &'static str) -> Self {
        Self {
            repository: Cow::Borrowed(repository),
            tag: None,
            digest: None,
        }
    }

    /// Create a new image with a tag
    #[must_use]
    pub const fn new_with_tag(repository: &'static str, tag: &'static str) -> Self {
        Self {
            repository: Cow::Borrowed(repository),
            tag: Some(Cow::Borrowed(tag)),
            digest: None,
        }
    }

    /// Create a new image with a digest
    #[must_use]
    pub const fn new_with_digest(repository: &'static str, digest: &'static str) -> Self {
        Self {
            repository: Cow::Borrowed(repository),
            tag: None,
            digest: Some(Cow::Borrowed(digest)),
        }
    }

    /// Set the image tag
    pub fn set_tag(&mut self, tag: impl Into<String>) {
        self.tag = Some(Cow::Owned(tag.into()));
    }

    /// Set the image digest
    pub fn set_digest(&mut self, digest: impl Into<String>) {
        self.digest = Some(Cow::Owned(digest.into()));
    }
}

impl Display for ImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repository)?;

        if let Some(tag) = &self.tag {
            write!(f, ":{tag}")?;
        }
        if let Some(digest) = &self.digest {
            write!(f, "@{digest}")?;
        }

        Ok(())
    }
}

impl FromStr for ImageName {
    type Err = ImageNameError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        if str.is_empty() {
            return Err(ImageNameError::EmptyName);
        }

        let result = if let Some((name, rest)) = str.split_once(':') {
            if let Some((tag, digest)) = rest.split_once('@') {
                Self {
                    repository: Cow::Owned(String::from(name)),
                    tag: Some(Cow::Owned(String::from(tag))),
                    digest: Some(Cow::Owned(String::from(digest))),
                }
            } else {
                Self {
                    repository: Cow::Owned(String::from(name)),
                    tag: Some(Cow::Owned(String::from(rest))),
                    digest: None,
                }
            }
        } else if let Some((name, digest)) = str.split_once('@') {
            Self {
                repository: Cow::Owned(String::from(name)),
                tag: None,
                digest: Some(Cow::Owned(String::from(digest))),
            }
        } else {
            Self {
                repository: Cow::Owned(String::from(str)),
                tag: None,
                digest: None,
            }
        };
        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::let_assert;

    use super::*;

    #[test]
    fn should_not_parse_empty_name() {
        let result = "".parse::<ImageName>();
        let_assert!(Err(ImageNameError::EmptyName) = result);
    }
}
