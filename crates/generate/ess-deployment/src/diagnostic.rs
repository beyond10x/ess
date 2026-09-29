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

impl DiagnosticCode {
    /// Every category, in declaration order, with what it means and how to repair it.
    ///
    /// `tests/diagnostic_catalogue.rs` fails when a variant is missing here, and
    /// `website/docs/reference/diagnostics.md` is rendered from it by `cargo xtask diagnostics`.
    pub const CATALOGUE: &'static [ess_compiler::diagnostic::CatalogueEntry<Self>] = &[
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnsupportedFormat,
            meaning: "The document's format marker is not one this build reads.",
            repair: "Write a format this build reads, or run a build that reads this one.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DuplicateIdentifier,
            meaning: "One identity is declared more than once.",
            repair: "Remove or rename the repeated declaration.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnknownReference,
            meaning: "A reference does not resolve in the inputs it is resolved against.",
            repair: "Correct the name, or supply the input that declares it.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DependencyCycle,
            meaning: "The declared dependencies form a cycle.",
            repair: "Remove one dependency so the graph has an order.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::InvalidValue,
            meaning: "A value is not valid for what it declares.",
            repair: "Write the value in the form the message describes.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnpinnedInput,
            meaning: "A remote or base input has no immutable identity, such as a digest.",
            repair: "Pin the input by digest.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UndeclaredSecret,
            meaning: "A build step asks for a secret the build interface does not declare.",
            repair: "Declare the secret in the build interface, or stop asking for it.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::MissingOutput,
            meaning: "A stage does not produce a named artifact it is required to.",
            repair: "Declare the output the message names.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::MissingComponent,
            meaning: "A component of the specification has no runtime realization, or more than \
                      one candidate for it.",
            repair: "Realize the component exactly once in the runtime document.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DuplicateComponent,
            meaning: "A component is realized more than once.",
            repair: "Keep one realization of the component.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::MissingEvidence,
            meaning: "A release lacks provenance, an SBOM, a signature or conformance evidence it \
                      requires.",
            repair: "Supply the evidence the message names before releasing.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DigestMismatch,
            meaning: "A claimed content digest differs from the digest of the input supplied.",
            repair: "Supply the input the digest names, or update the digest to the input's.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnsatisfiedConstraint,
            meaning: "No released system satisfies a requirement of the stack.",
            repair: "Relax the requirement, or release a system that satisfies it.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::MissingBinding,
            meaning: "A required environment coordinate is not bound.",
            repair: "Bind the coordinate the message names in the environment.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::AuthorityUnbound,
            meaning: "A required authority or service-account binding is absent.",
            repair: "Bind the authority or service account the message names in the environment.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::SecretValueForbidden,
            meaning: "Credential bytes appear where only a reference to a secret is allowed.",
            repair: "Replace the value with a reference to the secret that holds it.",
        },
    ];

    /// This category's entry in [`Self::CATALOGUE`].
    pub fn catalogue_entry(self) -> &'static ess_compiler::diagnostic::CatalogueEntry<Self> {
        Self::CATALOGUE
            .iter()
            .find(|entry| entry.key == self)
            .expect("every category is catalogued")
    }
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
