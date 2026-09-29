//! The delivery context an externally delivered event arrives with (beyond10x/ess#195, `ess/18`).
//!
//! An event delivered per account on a channel such as `accounts/{account_id}/messages` names the
//! message and not the account: the recipient is the subscription the event arrived on. That value
//! is not in the payload, and no `event.<channel>` pseudo-field stands in for it
//! (`docs/design/binding-mapping-bounded-accessor.md`). The binding declares it instead, under
//! `when:` beside the event:
//!
//! ```yaml
//! when:
//!   event: demo.inbox.MessageReceived
//!   context_authority: account-messages
//!   context_fields:
//!     - {name: account_id, type: demo.inbox.AccountId}
//! mapping:
//!   account_id: context.account_id
//! ```
//!
//! `context_fields` is a typed record separate from the payload, and `context.<field>` reads one of
//! its fields. `context_authority` names the external channel whose authority binds that record — a
//! binding-local name, never a credential — because naming a field does not establish who supplies
//! it. The host binds the context from the channel the occurrence arrived on, validates it against
//! the declared types, and supplies it with that occurrence; a redelivery of the occurrence carries
//! the context it was first delivered with.
//!
//! | rule | code |
//! |---|---|
//! | `context_fields` or `context.<field>` below `ess/18` | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//! | `context_fields` without `context_authority`, or the other way round | [`MissingDeclaration`](ValidationCode::MissingDeclaration) |
//! | `context_fields: []` | [`EmptyDeclaration`](ValidationCode::EmptyDeclaration) |
//! | a context field named twice, or not a field name | [`DuplicateDeclaration`](ValidationCode::DuplicateDeclaration) |
//! | a delivery context beside `periodic:` | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | a delivery context on an event a command or an escalation of this specification publishes | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | `context.<field>` on a binding that declares no delivery context | [`UnobservableFact`](ValidationCode::UnobservableFact) |
//! | `context.<field>` naming no declared context field | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | a context field whose type does not reach the input it fills | [`TypeMismatch`](ValidationCode::TypeMismatch) |
//!
//! # Only for an event an external channel delivers
//!
//! An event some command of this specification emits, or some binding escalates into, arrives from
//! inside the system: it has no channel, so nothing could bind a context to it, and a context the
//! model declared for it would be a value no implementation can supply. The context is therefore
//! admitted only for an event nothing in the specification publishes — which is the event the
//! conformance suite delivers itself, from outside, with a context of its choosing.
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use super::{BindingName, MappingSource};
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::Field;

/// An event delivered by an external channel, with the typed context that channel binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalEvent {
    /// The event.
    pub event: QualifiedName,
    /// The binding-local name of the external channel whose authority binds the context.
    ///
    /// A declaration, not proof of authentication: no credential is serialized here.
    pub authority: BindingName,
    /// The typed context each occurrence arrives with, separate from its payload.
    pub context_fields: Vec<Field>,
}

impl ExternalEvent {
    /// The prefix a mapping reads a context field by.
    pub const PREFIX: &'static str = "context.";

    /// The declared context field with this name.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.context_fields.iter().find(|field| field.name == name)
    }

    /// Everything checkable without the rest of the specification.
    pub fn validate(&self, at: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let path = format!("{at}.context_fields");
        if self.context_fields.is_empty() {
            errors.push(
                ValidationError::new(
                    ValidationCode::EmptyDeclaration,
                    &path,
                    "`context_fields` declares no field, so the delivery context carries nothing",
                )
                .with_hint("declare the fields the channel binds, or drop `context_fields`"),
            );
        }
        errors.extend(super::periodic::field_table(
            &self.context_fields,
            &path,
            "delivery context",
        ));
        errors
    }
}

/// The format gate, primitive admission and the external-channel rule, for every binding of the
/// specification, including one assembled in code.
pub fn validate_specification(spec: &crate::Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let format = spec.system().format;
    let admitted = format.major() >= FormatVersion::V18.major();
    for binding in spec.bindings().values() {
        let Some(external) = binding.cause.external() else {
            if !admitted {
                for (target, source) in &binding.mapping {
                    if matches!(source, MappingSource::DeliveryContext { .. }) {
                        errors.push(
                            ValidationError::new(
                                ValidationCode::UnsupportedFormatVersion,
                                format!("binding.{}.mapping.{target}", binding.name),
                                format!(
                                    "`{source}` reads a delivery context, which requires \
                                     specification format ess/18"
                                ),
                            )
                            .with_hint("declare `format: ess/18`"),
                        );
                    }
                }
            }
            continue;
        };
        let at = format!("binding.{}.when", binding.name);
        if !admitted {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("{at}.context_fields"),
                    "a delivery context requires specification format ess/18",
                )
                .with_hint("declare `format: ess/18`"),
            );
        }
        for field in &external.context_fields {
            crate::primitive_admission::reference(
                &field.type_ref,
                Some(format),
                &format!("{at}.context_fields.{}", field.name),
                &mut errors,
            );
        }
        for publisher in publishers(spec, &external.event) {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("{at}.context_fields"),
                    format!(
                        "a delivery context is admitted only for an event an external channel \
                         delivers, and `{}` is published by {publisher}",
                        external.event
                    ),
                )
                .with_hint(
                    "an event the system publishes arrives on no channel, so nothing can bind a \
                     context to it; carry the value in the payload, or react to the event the \
                     external channel delivers",
                ),
            );
        }
    }
    errors
}

/// Everything in the specification that publishes `event`, spelt for a diagnostic.
fn publishers(spec: &crate::Specification, event: &QualifiedName) -> Vec<String> {
    let mut found = Vec::new();
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            if outcome.emits.contains(event) {
                found.push(format!("`{}` (outcome `{}`)", command.name, outcome.name));
            }
        }
    }
    for binding in spec.bindings().values() {
        if binding.escalation.as_ref() == Some(event) {
            found.push(format!("the escalation of binding `{}`", binding.name));
        }
    }
    found
}
