use std::fmt::{Debug, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{Id, IdError};

/// An image id
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ImageId(Id);

impl From<ImageId> for String {
    fn from(value: ImageId) -> Self {
        String::from(value.0)
    }
}

impl FromStr for ImageId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>().map(Self)
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
