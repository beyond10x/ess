//! Standalone ESS verification vocabulary.

use std::fmt;

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
