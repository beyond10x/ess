//! Closed collection profiles: observed membership is distinct from omitted content.

use serde::Serialize;

/// A complete collection of the declared topology subset in exactly one namespace.
///
/// All seventeen supported kinds must be present. Namespaced collections cover this namespace;
/// Namespace is the one named object; Nodes are only those referenced by the collected Pods.
/// Values, literal environment entries, probe definitions and unknown API fields are unobserved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "profile", rename_all = "snake_case")]
pub enum CollectionCoverage {
    /// Structural namespace inventory, with value-bearing payloads deliberately omitted.
    NamespaceTopology {
        /// The exact namespace requested from the API.
        namespace: String,
    },
}

impl TryFrom<crate::raw::RawCollectionCoverage> for CollectionCoverage {
    type Error = crate::ValidationErrors;

    fn try_from(raw: crate::raw::RawCollectionCoverage) -> Result<Self, Self::Error> {
        let crate::raw::RawCollectionCoverage::NamespaceTopology { namespace } = raw;
        let coverage = Self::NamespaceTopology { namespace };
        if coverage.is_valid() {
            Ok(coverage)
        } else {
            let mut errors = crate::ValidationErrors::new();
            errors.refuse(
                crate::InfraCode::UnsupportedFormat,
                "coverage",
                "invalid collection namespace",
            );
            Err(errors)
        }
    }
}

impl CollectionCoverage {
    /// The exact namespace this collection covers.
    pub fn namespace(&self) -> &str {
        match self {
            Self::NamespaceTopology { namespace } => namespace,
        }
    }

    /// Whether the namespace can be used as one literal Kubernetes namespace argument.
    pub fn is_valid(&self) -> bool {
        let name = self.namespace();
        !name.is_empty()
            && name.len() <= 63
            && name.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && name.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && name
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    }

    /// Stable coverage explanation for reports; empty omitted fields are never absence claims.
    pub const fn limitation(&self) -> &'static str {
        "namespace topology only: configuration and Secret keys/values, literal environment entries, probes and unmodeled API fields are unobserved; nodes are referenced nodes only"
    }
}
