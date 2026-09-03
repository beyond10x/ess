use crate::Identifier;
use std::fmt;

/// The lowering stage which owns an obligation or refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Source-to-artifact transformations.
    Build,
    /// Physical-realization to deployable-runtime mapping.
    Runtime,
    /// Immutable artifact publication.
    Release,
    /// Multi-system constraint resolution.
    Composition,
    /// Target-environment binding and deployment lowering.
    Deployment,
}

/// Stable validation and refusal categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    /// Persisted format marker is not supported by this reader.
    UnsupportedFormat,
    /// One stable identity was declared more than once.
    DuplicateIdentifier,
    /// A reference does not resolve in its declared input set.
    UnknownReference,
    /// An explicitly declared dependency graph is cyclic.
    DependencyCycle,
    /// A typed value violates its semantic constraints.
    InvalidValue,
    /// A remote or base input lacks immutable identity.
    UnpinnedInput,
    /// A build step requests a secret the build interface did not declare.
    UndeclaredSecret,
    /// A stage lacks a required named artifact output.
    MissingOutput,
    /// A semantic component has no unambiguous runtime realization.
    MissingComponent,
    /// A semantic component is realized more than once.
    DuplicateComponent,
    /// A release lacks required provenance, SBOM, signature, or conformance evidence.
    MissingEvidence,
    /// Claimed content identity differs from the supplied canonical input.
    DigestMismatch,
    /// No released system satisfies a stack requirement.
    UnsatisfiedConstraint,
    /// A required environment coordinate is absent.
    MissingBinding,
    /// A required authority or service-account binding is absent.
    AuthorityUnbound,
    /// Credential bytes appeared in a format that may only carry secret references.
    SecretValueForbidden,
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedFormat => "unsupported_format",
            Self::DuplicateIdentifier => "duplicate_identifier",
            Self::UnknownReference => "unknown_reference",
            Self::DependencyCycle => "dependency_cycle",
            Self::InvalidValue => "invalid_value",
            Self::UnpinnedInput => "unpinned_input",
            Self::UndeclaredSecret => "undeclared_secret",
            Self::MissingOutput => "missing_output",
            Self::MissingComponent => "missing_component",
            Self::DuplicateComponent => "duplicate_component",
            Self::MissingEvidence => "missing_evidence",
            Self::DigestMismatch => "digest_mismatch",
            Self::UnsatisfiedConstraint => "unsatisfied_constraint",
            Self::MissingBinding => "missing_binding",
            Self::AuthorityUnbound => "authority_unbound",
            Self::SecretValueForbidden => "secret_value_forbidden",
        })
    }
}

/// One deterministic, repair-oriented diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Diagnostic {
    stage: Stage,
    code: DiagnosticCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<Identifier>,
    detail: String,
}

impl Diagnostic {
    pub(crate) fn new(
        stage: Stage,
        code: DiagnosticCode,
        subject: Option<Identifier>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            stage,
            code,
            subject,
            detail: detail.into(),
        }
    }

    /// The stage which must resolve the failure.
    pub const fn stage(&self) -> Stage {
        self.stage
    }

    /// Stable machine-readable category.
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Affected local identity, when one exists.
    pub fn subject(&self) -> Option<&Identifier> {
        self.subject.as_ref()
    }

    /// Repair-oriented explanation.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Every diagnostic from one deterministic compilation attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub(crate) fn from(mut diagnostics: Vec<Diagnostic>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self(diagnostics)
    }

    /// The complete canonical diagnostic list.
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.0
    }

    /// Whether a category was reported.
    pub fn contains(&self, code: DiagnosticCode) -> bool {
        self.0.iter().any(|diagnostic| diagnostic.code == code)
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            write!(
                formatter,
                "[{}:{:?}] {}",
                diagnostic.code, diagnostic.stage, diagnostic.detail
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}
