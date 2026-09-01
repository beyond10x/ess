//! Opaque identifiers used by standalone ESS conformance execution.

use std::fmt;

use crate::error::ParseError;

/// Identifier shared by operations belonging to one conformance activity.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// The greatest accepted identifier length, preserving the pre-extraction contract.
    pub const MAX_LENGTH: usize = 200;

    /// Creates an identifier from a non-empty ASCII loose identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, ParseError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= Self::MAX_LENGTH
            && value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '-' | '_'))
            && value
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric());
        if !valid {
            return Err(ParseError::identifier(
                "correlation",
                &value,
                "must be a non-empty ASCII identifier using letters, digits, `.`, `/`, `-` or `_`"
                    .to_owned(),
            ));
        }
        Ok(Self(value))
    }

    /// Returns the identifier as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CorrelationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CorrelationId({})", self.0)
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for CorrelationId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for CorrelationId {
    fn schema_name() -> String {
        "CorrelationId".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some("^[A-Za-z0-9][A-Za-z0-9._/-]*$".to_owned());
        schema.string().max_length = Some(200);
        schema.into()
    }
}
