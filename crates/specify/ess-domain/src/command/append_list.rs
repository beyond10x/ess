//! `sets: {items: {append: input.item}}`: an outcome storing the list its subject held before it,
//! absent read as empty, with one input element last (source format `ess/31`).
//!
//! A `sets:` source over a `List<T>` or `Optional<List<T>>` field of an existing subject, as
//! `{increment: n}` is over a number: refused on `creates:`, in a payload, inside a nested mapping
//! and on a set effect. The element is `input.<path>` of type `T` exactly; an `Optional` read has
//! nothing to append, and a declared conversion is not applied. Other element sources (the subject,
//! a related row, the caller, a literal) are not admitted in this cut.
//!
//! Below `ess/31` the mapping `{append: …}` is the nested mapping it always was, a struct with a
//! member `append` ([`read_below_ess_31`]); over a `List<…>` field it is refused naming `ess/31`.
use std::collections::BTreeMap;

use super::{CommandSpec, PayloadSource};
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::{Field, TypeRef};
use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};

/// The key of the value source: `{append: input.<path>}`.
pub const APPEND: &str = "append";

/// How a diagnostic names the source.
pub const WRITTEN: &str = "`{append: …}`";

/// `{append: input.<path>}` as written.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawAppendSource {
    /// The input element appended last, written `input.<field>` or from `ess/22` `input.<path>`.
    append: String,
}

impl RawAppendSource {
    /// The mapping `{append: input.<path>}`, and nothing else.
    pub(super) fn recognise(entries: &[(String, super::RawPayloadSource)]) -> Option<Self> {
        match entries {
            [(key, super::RawPayloadSource::Text(read))]
                if key == APPEND
                    && read
                        .strip_prefix(PayloadSource::INPUT_PREFIX)
                        .is_some_and(|path| path.split('.').all(crate::binding::is_field_name)) =>
            {
                Some(Self {
                    append: read.clone(),
                })
            }
            _ => None,
        }
    }

    /// The source it is.
    pub(super) fn into_source(self) -> PayloadSource {
        let field = self
            .append
            .strip_prefix(PayloadSource::INPUT_PREFIX)
            .unwrap_or(&self.append)
            .to_owned();
        PayloadSource::Append { field }
    }

    /// The written form of an append of `field`.
    pub(super) fn of(field: &str) -> Self {
        Self {
            append: format!("{}{field}", PayloadSource::INPUT_PREFIX),
        }
    }
}

/// Reads `{append: input.<path>}` back as the nested mapping it was below `ess/31`.
pub fn read_below_ess_31(
    format: FormatVersion,
    commands: &mut BTreeMap<QualifiedName, CommandSpec>,
) {
    if format.major() >= FormatVersion::V31.major() {
        return;
    }
    for outcome in commands
        .values_mut()
        .flat_map(|command| command.outcomes.iter_mut())
    {
        let affects = outcome
            .set_effects
            .affects
            .iter_mut()
            .flat_map(|affect| affect.sets.values_mut());
        for source in outcome
            .payload
            .values_mut()
            .flat_map(|fields| fields.values_mut())
            .chain(outcome.error_payload.values_mut())
            .chain(outcome.sets.values_mut())
            .chain(affects)
        {
            nested_append(source);
        }
    }
}

fn nested_append(source: &mut PayloadSource) {
    match source {
        PayloadSource::Append { field } => {
            *source = PayloadSource::Struct {
                fields: vec![super::PayloadField {
                    target: APPEND.to_owned(),
                    source: PayloadSource::InputField {
                        field: field.clone(),
                    },
                }],
            };
        }
        PayloadSource::Struct { fields } => {
            for leaf in fields {
                nested_append(&mut leaf.source);
            }
        }
        PayloadSource::InputOrGenerated {
            otherwise: Some(otherwise),
            ..
        } => nested_append(otherwise),
        _ => {}
    }
}

/// The element type of a `List<T>` or `Optional<List<T>>` field.
pub fn element(target: &TypeRef) -> Option<&TypeRef> {
    match target {
        TypeRef::List(element) => Some(element),
        TypeRef::Optional(inner) => match inner.as_ref() {
            TypeRef::List(element) => Some(element),
            _ => None,
        },
        _ => None,
    }
}

/// A nested mapping whose one member is `append`, filling a `List<…>` field: below `ess/31` the
/// source it would be from `ess/31`, refused naming that format; from `ess/31` an `{append: …}`
/// whose element is no `input.<path>`, refused naming the one element it takes.
pub(super) fn misread_mapping(
    format: FormatVersion,
    at: &ConstructRef,
    target: &Field,
    fields: &[super::PayloadField],
) -> Option<ValidationError> {
    let [only] = fields else {
        return None;
    };
    if only.target != APPEND || element(&target.type_ref).is_none() {
        return None;
    }
    Some(if format.major() < FormatVersion::V31.major() {
        ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedFormatVersion,
            format!(
                "`{}` is `{}`, and {WRITTEN} appends to a stored list from specification format \
                 ess/31",
                target.name, target.type_ref
            ),
        )
        .with_hint("declare `format: ess/31`")
    } else {
        ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!(
                "{WRITTEN} appends an element read from the input, and `{}` is no `input.<path>`",
                only.source
            ),
        )
        .with_hint("write `{append: input.<path>}`, the input field holding the element")
    })
}

/// A refusal of `{append: …}` written anywhere but a whole `sets:` entry.
pub(super) fn elsewhere(at: &ConstructRef, nested: bool) -> ValidationError {
    if nested {
        ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!(
                "{WRITTEN} appends to a whole entity field, not to a field inside a nested mapping"
            ),
        )
    } else {
        ValidationError::at(
            at.clone(),
            ValidationCode::ConflictingDeclaration,
            format!("{WRITTEN} changes a stored list and is a `sets:` source only"),
        )
        .with_hint("to publish the element, read it with `input.<field>`")
    }
}

/// The element `input.<field>` against the list `target` holds: `T` exactly for a `List<T>` or an
/// `Optional<List<T>>`. `read` is the input as the command declares it, or `None` when the
/// refusal of the read was already reported.
pub(super) fn check_element(
    at: &ConstructRef,
    (entity, target): (&QualifiedName, &Field),
    (field, read): (&str, &Field),
    errors: &mut ValidationErrors,
) {
    let Some(element) = element(&target.type_ref) else {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{entity}.{}` is `{}`, and {WRITTEN} appends to a `List<…>` or an \
                     `Optional<List<…>>`",
                    target.name, target.type_ref
                ),
            )
            .with_hint("set the field from `input.<field>`, or declare it a `List<…>`"),
        );
        return;
    };
    if &read.type_ref != element {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`input.{field}` has type `{}`, and `{entity}.{}` is `{}`, whose element is \
                     `{element}`; {WRITTEN} appends an element of exactly that type",
                    read.type_ref, target.name, target.type_ref
                ),
            )
            .with_hint(if read.type_ref.is_optional() {
                "an absent element has nothing to append; declare the input required".to_owned()
            } else {
                format!("declare `{field}` as `{element}`")
            }),
        );
    }
}
