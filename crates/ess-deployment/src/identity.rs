use std::fmt;
use std::str::FromStr;

/// A stable lowercase identifier used inside deployment documents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct Identifier(String);

impl Identifier {
    /// Parses a lowercase, path-safe identifier.
    pub fn new(value: impl AsRef<str>) -> Result<Self, IdentifierError> {
        let value = value.as_ref();
        let mut previous_separator = false;
        let valid = !value.is_empty()
            && value
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_lowercase())
            && value.chars().all(|character| {
                let separator = matches!(character, '.' | '_' | '-');
                let accepted = character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || (separator && !previous_separator);
                previous_separator = separator;
                accepted
            })
            && !previous_separator;
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(IdentifierError(value.to_owned()))
        }
    }

    /// The validated spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A malformed deployment identifier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid identifier {0:?}: expected lowercase segments separated by '.', '_' or '-'")]
pub struct IdentifierError(String);

/// An exact SHA-256 content digest including its algorithm prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct Digest(String);

impl Digest {
    /// Parses `sha256:` followed by 64 lowercase hexadecimal characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DigestError> {
        let value = value.as_ref();
        let Some(hex) = value.strip_prefix("sha256:") else {
            return Err(DigestError(value.to_owned()));
        };
        if hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value.to_owned()))
        } else {
            Err(DigestError(value.to_owned()))
        }
    }

    /// Hashes canonical bytes.
    pub(crate) fn of_bytes(bytes: &[u8]) -> Self {
        use sha2::{Digest as _, Sha256};
        let hash = Sha256::digest(bytes);
        let mut value = String::with_capacity(71);
        value.push_str("sha256:");
        for byte in hash {
            use fmt::Write as _;
            write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Self(value)
    }

    /// The exact digest spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Digest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> serde::Deserialize<'de> for Digest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A malformed content digest.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid digest {0:?}: expected sha256:<64 lowercase hexadecimal characters>")]
pub struct DigestError(String);

pub(crate) fn canonical_json(value: &impl serde::Serialize) -> String {
    let mut json = serde_json::to_string_pretty(value).expect("typed ESS IR always serializes");
    json.push('\n');
    json
}
