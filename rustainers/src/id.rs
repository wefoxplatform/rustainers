use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;

use hex::{decode, encode, FromHex};

use crate::IdError;

/// An id for image or a container.
///
/// The id is a SHA-252 represented as 32 bytes array.
/// Therefore this type is [`Copy`].
///
/// Note because some version of Docker CLI return truncated value,
/// we need to store the size of the id.
///
/// `PartialEq`, `Eq` and `Hash` implementation is based on all fields (size included)
///
/// Most usage of this type is done with the string representation.
///
/// Note that the [`Display`] view truncate the id,
/// to have the full [`String`] you need to use the [`Into`] or [`From`] implementation.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id([u8; 32], usize);

impl Id {
    /// Is ids are the same, they could have different size
    #[allow(clippy::indexing_slicing)]
    pub fn same(&self, other: &Self) -> bool {
        let size = self.1.min(other.1);
        self.0[..size] == other.0[..size]
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        let Id(data, size) = value;
        debug_assert!(size <= data.len());
        #[allow(clippy::indexing_slicing)]
        encode(&data[..size])
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        if str.is_empty() {
            return Err(IdError::Empty);
        }
        if str.len() > 64 {
            return Err(IdError::TooLong(String::from(str)));
        }

        if str.len() == 64 {
            let data = <[u8; 32]>::from_hex(str).map_err(|source| IdError::InvalidId {
                value: String::from(str),
                source,
            })?;
            Ok(Self(data, 32))
        } else {
            let mut data = [0; 32];
            let bytes = decode(str).map_err(|source| IdError::InvalidId {
                value: String::from(str),
                source,
            })?;
            let size = bytes.len();
            debug_assert!(size < data.len());
            #[allow(clippy::indexing_slicing)]
            for (i, b) in bytes.iter().enumerate() {
                data[i] = *b;
            }

            Ok(Self(data, size))
        }
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&String::from(*self)).finish()
    }
}

impl Display for Id {
    #[allow(clippy::indexing_slicing)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = String::from(*self);
        let truncate = self.1.min(6) * 2;
        debug_assert!(truncate <= str.len());
        write!(f, "{}", &str[..truncate])
    }
}

mod image_id_serde {
    use serde::de::Visitor;
    use serde::{Deserialize, Serialize};

    use super::Id;

    impl Serialize for Id {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let str = String::from(*self);
            serializer.serialize_str(&str)
        }
    }

    struct IdVisitor;

    impl<'de> Visitor<'de> for IdVisitor {
        type Value = Id;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "Expected an hex-encoded 32 bits (length 64)")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse().map_err(E::custom)
        }
    }

    impl<'de> Deserialize<'de> for Id {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(IdVisitor)
        }
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "c94f6f8d4ef25b80584b9457ca24b964032681895b3a6fd7cd24fd40fad4895e",
        "c94f6f8d4ef2"
    )]
    #[case(
        "637ceb59b7a01df4466442fc5bb30bcf0ce3428289b00bbc02f62ddaa3e6bd8d",
        "637ceb59b7a0"
    )]
    #[case("637ceb59b7a0", "637ceb59b7a0")]
    #[case("637c", "637c")]
    fn should_parse_id(#[case] str: &str, #[case] short: &str) {
        let result = str.parse::<Id>();
        let_assert!(Ok(id) = result);
        check!(id.to_string() == short);
    }

    #[test]
    fn should_compare_prefix() {
        let id0 = "c94f6f8d4ef2".parse::<Id>().expect("valid id");
        let id1 = "c94f6f8d4ef25b80584b9457ca24b964032681895b3a6fd7cd24fd40fad4895e"
            .parse::<Id>()
            .expect("valid id");
        check!(id0.same(&id1) == true, "same prefix");

        let id0 = "c94f6f8d4ef200".parse::<Id>().expect("valid id");
        let id1 = "c94f6f8d4ef25b80584b9457ca24b964032681895b3a6fd7cd24fd40fad4895e"
            .parse::<Id>()
            .expect("valid id");
        check!(id0.same(&id1) == false, "different prefix");
    }

    #[rstest]
    #[case::normal("\"c94f6f8d4ef25b80584b9457ca24b964032681895b3a6fd7cd24fd40fad4895e\"")]
    #[case::short("\"637ceb59b7a0\"")]
    fn should_serde(#[case] str: &str) {
        let result = serde_json::from_str::<Id>(str);
        let_assert!(Ok(id) = result);
        let result = serde_json::to_string(&id);
        let_assert!(Ok(json) = result);
        check!(json == str);
    }

    #[rstest]
    #[case::empty("")]
    #[case::invalid("X94f6f8d4ef25b80584b9457ca24b964032681895b3a6fd7cd24fd40fad4895e")]
    #[case::too_long("794f6f8d4ef25b80584b9457ca24b964032681895b3a6fd7cd24fd40fad4895e0000")]
    fn should_not_parse_image_id(#[case] str: &str) {
        let result = str.parse::<Id>();
        let_assert!(Err(_) = result);
    }
}
