use std::fmt::{Debug, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{Id, IdError};

/// A container id
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContainerId(Id);

impl From<ContainerId> for String {
    fn from(value: ContainerId) -> Self {
        String::from(value.0)
    }
}

impl FromStr for ContainerId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>().map(Self)
    }
}

impl Debug for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ContainerId")
            .field(&String::from(*self))
            .finish()
    }
}

impl Display for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
