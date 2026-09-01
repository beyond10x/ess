//! Standalone ESS conformance report primitives.

use std::fmt;

use crate::error::ParseError;

/// Digest of a resolved ESS specification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct SpecDigest(String);

impl SpecDigest {
    /// Shortest accepted legacy digest.
    pub const MIN_LENGTH: usize = 16;
    /// Full SHA-256 length, and the longest accepted digest.
    pub const MAX_LENGTH: usize = 64;

    /// Creates a lowercase hexadecimal digest.
    pub fn new(value: impl Into<String>) -> Result<Self, ParseError> {
        let value = value.into();
        let valid = value.len() >= Self::MIN_LENGTH
            && value.len() <= Self::MAX_LENGTH
            && value.chars().all(|ch| ch.is_ascii_hexdigit())
            && !value.chars().any(|ch| ch.is_ascii_uppercase());
        if !valid {
            return Err(ParseError::reference(
                "specification digest",
                &value,
                format!(
                    "expected {} to {} lower-case hexadecimal characters",
                    Self::MIN_LENGTH,
                    Self::MAX_LENGTH
                ),
            ));
        }
        Ok(Self(value))
    }

    /// Returns the digest as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpecDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SpecDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for SpecDigest {
    fn schema_name() -> String {
        "SpecDigest".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(format!(
            "^[0-9a-f]{{{},{}}}$",
            Self::MIN_LENGTH,
            Self::MAX_LENGTH
        ));
        schema.into()
    }
}
