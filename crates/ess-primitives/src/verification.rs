//! Standalone ESS verification vocabulary.

use std::fmt;

use crate::error::ParseError;

/// A verifier class understood by a standalone ESS report.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verifier {
    /// The runner that checks an implementation against an ESS conformance suite.
    ConformanceRunner,
}

impl Verifier {
    /// Returns the stable wire spelling.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ConformanceRunner => "conformance-runner",
        }
    }

    /// Parses a verifier spelling.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        match value {
            "conformance-runner" => Ok(Self::ConformanceRunner),
            _ => Err(ParseError::identifier(
                "ESS verifier",
                value,
                "expected `conformance-runner`".to_owned(),
            )),
        }
    }
}

impl fmt::Display for Verifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl serde::Serialize for Verifier {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Verifier {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for Verifier {
    fn schema_name() -> String {
        "Verifier".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.enum_values = Some(vec![serde_json::Value::String(
            "conformance-runner".to_owned(),
        )]);
        schema.into()
    }
}

/// Outcome of a standalone ESS verification.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// The specification held.
    Passed,
    /// The implementation contradicted the specification.
    Failed,
    /// The runner could not decide.
    Inconclusive,
    /// The runner did not execute.
    Skipped,
}

impl VerificationStatus {
    /// Returns `true` only for a passing result.
    pub fn is_pass(self) -> bool {
        self == Self::Passed
    }
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Inconclusive => "inconclusive",
            Self::Skipped => "skipped",
        })
    }
}
