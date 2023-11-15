use std::fmt::{Debug, Display};

use super::{ImageId, ImageName};

/// An image reference
///
/// An image can be reference by a name, or by and id
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageReference {
    /// An image id
    Id(ImageId),

    /// An image name
    Name(ImageName),
}

impl From<ImageId> for ImageReference {
    fn from(value: ImageId) -> Self {
        Self::Id(value)
    }
}

impl From<ImageName> for ImageReference {
    fn from(value: ImageName) -> Self {
        Self::Name(value)
    }
}

impl Display for ImageReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => write!(f, "{id}"),
            Self::Name(name) => write!(f, "{name}"),
        }
    }
}
