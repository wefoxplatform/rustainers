use std::fmt::{self, Display};
use std::str::FromStr;

use crate::VersionError;

/// A version with the `<major>.<minor>.<patch>-<rest>` form
///
/// This is a relaxed [semver version](https://semver.org/) because
///
/// * the patch could be omit
/// * there are no constraints on the rest part, we remove it
/// * allow the 'v' prefix
///
/// As we do not need additional semver version (build, pre-release),
/// we could implement [`Copy`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: Option<u64>,
}

impl Version {
    pub const fn new(major: u64, minor: u64) -> Self {
        Self {
            major,
            minor,
            patch: None,
        }
    }
}

fn extract_simple_version(str: &str) -> Result<Version, VersionError> {
    let Some((major, rest)) = str.split_once('.') else {
        return Err(VersionError::RequireMajorMinor);
    };
    let major = major.parse().map_err(VersionError::InvalidMajorVersion)?;

    let (minor, patch) = if let Some((minor, patch)) = rest.split_once('.') {
        let minor = minor.parse().map_err(VersionError::InvalidMinorVersion)?;
        let patch = patch.parse().map_err(VersionError::InvalidPatchVersion)?;
        (minor, Some(patch))
    } else {
        let minor = rest.parse().map_err(VersionError::InvalidMinorVersion)?;
        (minor, None)
    };

    let result = Version {
        major,
        minor,
        patch,
    };
    Ok(result)
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let str = str.trim_start_matches('v');
        if str.is_empty() {
            return Err(VersionError::Empty);
        }

        if let Some(idx) = str.find(['-', '+']) {
            let (version, _) = str.split_at(idx);
            extract_simple_version(version)
        } else {
            extract_simple_version(str)
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            major,
            minor,
            patch,
        } = self;
        write!(f, "{major}.{minor}")?;
        if let Some(patch) = patch {
            write!(f, ".{patch}")?;
        }
        Ok(())
    }
}

mod serde_version {
    use std::fmt;

    use serde::de::Visitor;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::Version;

    impl Serialize for Version {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let version = self.to_string();
            serializer.serialize_str(&version)
        }
    }

    struct VersionVisitor;

    impl Visitor<'_> for VersionVisitor {
        type Value = Version;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an version with the '<major>.<minor>.<patch>+<build>' pattern")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse().map_err(E::custom)
        }
    }

    impl<'de> Deserialize<'de> for Version {
        fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(VersionVisitor)
        }
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};
    use rstest::rstest;

    use super::*;

    fn version(major: u64, minor: u64, patch: Option<u64>) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }

    #[rstest]
    #[case("v1.2.3+plop", version(1, 2, Some(3)))]
    #[case("1.2.3+plop", version(1, 2, Some(3)))]
    #[case("1.2.3-plop", version(1, 2, Some(3)))]
    #[case("1.2.3", version(1, 2, Some(3)))]
    #[case("1.2+plop", version(1, 2, None))]
    #[case("1.2", version(1, 2, None))]
    #[case(
        "11011.246546.465465-asd+asdasd~asdasd",
        version(11_011, 246_546, Some(465_465))
    )]
    fn should_parse(#[case] input: &str, #[case] expected: Version) {
        // Check parsing
        let result = input.parse::<Version>();
        let_assert!(Ok(version) = result);

        // Check expected
        check!(version == expected);
    }

    #[rstest]
    #[case("")]
    #[case("w1.2.3")]
    #[case("1.2.3.4")]
    #[case("1.x")]
    #[case("1.")]
    #[case("1.a")]
    #[case("1.1.x")]
    #[case("1.1.0 alpha")]
    #[case("1.-1.0")]
    fn should_not_parse(#[case] input: &str) {
        // Check parsing
        let result = input.parse::<Version>();
        let_assert!(Err(_) = result);
    }

    #[rstest]
    #[case(version(1, 2, Some(3)))]
    #[case(version(1, 2, Some(3)))]
    #[case(version(1, 2, Some(3)))]
    #[case(version(1, 2, None))]
    #[case(version(1, 2, None))]
    #[case(version(1_1011, 246_546, Some(465_465)))]
    fn should_serde(#[case] value: Version) {
        let result = serde_json::to_string(&value);
        let_assert!(Ok(json) = result);

        let result = serde_json::from_str::<Version>(&json);
        let_assert!(Ok(version) = result);
        check!(version == value);
    }

    #[rstest]
    #[case::major("10.2.1", "1.2.2")]
    #[case::minor("1.20.1", "1.2.2")]
    #[case::patch("1.2.4", "1.2.3")]
    #[case::with_patch("1.2.0", "1.2")]
    fn should_compare(#[case] rhs: &str, #[case] lhs: &str) {
        let rhs = rhs.parse::<Version>().expect("valid version");
        let lhs = lhs.parse::<Version>().expect("valid version");

        // equals
        check!(rhs == rhs);
        check!(lhs == lhs);

        // greater
        check!(rhs > lhs);

        // lower
        check!(lhs < rhs);
    }
}
