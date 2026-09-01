//! Standalone ESS conformance report primitives.

use std::fmt;

use crate::error::ParseError;
use crate::verification::{VerificationStatus, Verifier};

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

/// Result body of a standalone ESS conformance report.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct EssConformanceResult {
    /// Human-readable specification and version.
    pub specification: String,
    /// Exact resolved specification digest.
    pub spec_digest: SpecDigest,
    /// Implementation identity.
    pub implementation: String,
    /// Overall verification outcome.
    pub status: VerificationStatus,
    /// Number of scenarios executed.
    #[serde(default)]
    pub scenarios_total: usize,
    /// Number of scenarios that did not pass.
    #[serde(default)]
    pub scenarios_failed: usize,
    /// Conformance suite version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite_version: Option<String>,
    /// Legacy compiler stamp accepted on read and never written.
    #[serde(default, skip_serializing)]
    pub compiler_version: Option<String>,
    /// Legacy generator stamp accepted on read and never written.
    #[serde(default, skip_serializing)]
    pub generator_version: Option<String>,
    /// Actionable failing scenario identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed_scenarios: Vec<String>,
}

impl EssConformanceResult {
    /// Returns whether this report concerns `digest`.
    pub fn attests(&self, digest: &SpecDigest) -> bool {
        &self.spec_digest == digest
    }
}

/// Standalone report body, tagged with the stable ESS conformance kind.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evidence {
    /// ESS conformance output.
    EssConformance(EssConformanceResult),
}

/// Producer identity written by the ESS conformance runner.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "producer", rename_all = "snake_case")]
pub enum Producer {
    /// An agent-authored report, accepted for compatibility but never minted by ESS.
    Agent {
        /// Agent identifier.
        id: String,
    },
    /// A verifier-authored report.
    Verifier {
        /// Verifier class.
        verifier: Verifier,
    },
}

impl Producer {
    /// Returns whether an agent produced the report.
    pub fn is_agent(&self) -> bool {
        matches!(self, Self::Agent { .. })
    }
}

/// How the standalone report was obtained.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Command that produced the report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Digest of raw output, when independently recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    /// Input locations used by the run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,
}
