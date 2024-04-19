use std::fmt::{Debug, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{Id, IdError};

/// An image id
///
/// The id is a SHA-252 represented as 32 bytes array.
/// Therefore this type is [`Copy`].
///
/// Note because some version of Docker CLI return truncated value,
/// we need to store the size of the id.
///
/// Most usage of this type is done with the string representation.
///
/// Note that the [`Display`] view truncate the id,
/// to have the full [`String`] you need to use the [`Into`] or [`From`] implementation.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ImageId(Id);

impl From<ImageId> for String {
    fn from(value: ImageId) -> Self {
        String::from(value.0)
    }
}

impl FromStr for ImageId {
    type Err = IdError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        str.parse::<Id>().map(Self)
    }
}

impl Debug for ImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ImageId")
            .field(&String::from(*self))
            .finish()
    }
}

impl Display for ImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
