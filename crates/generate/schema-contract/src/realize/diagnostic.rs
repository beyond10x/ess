//! Shared normalization and structural-target diagnostics.

use serde::Serialize;
use std::fmt;

/// One source-located statement about a type projection's boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    /// Input schema JSON Pointer, not a generated declaration line.
    pub pointer: String,
    /// Stable rule name or the source keyword requiring interpretation.
    pub rule: String,
    /// The unimplemented guarantee or reason for refusal.
    pub detail: String,
}

/// All observed refusals; no partial successful plan is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused(pub Vec<Finding>);

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            writeln!(f, "{}: {}: {}", item.pointer, item.rule, item.detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for Refused {}

pub(super) fn path(parent: &str, segment: &str) -> String {
    format!("{parent}/{}", segment.replace('~', "~0").replace('/', "~1"))
}

pub(super) fn finding(pointer: &str, rule: &str, detail: &str) -> Finding {
    Finding {
        pointer: pointer.to_owned(),
        rule: rule.to_owned(),
        detail: detail.to_owned(),
    }
}
