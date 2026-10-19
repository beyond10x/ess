//! A minted value stored present, `{generated: true, present: true}`, and a `sets:` value read
//! from the outcome's own payload, `{payload: <Event>.<field>}` (source format `ess/27`,
//! beyond10x/ess#545, `docs/design/minted-value-sources.md`).
//!
//! Neither shape is all source keywords, so each is recognised by its exact shape, as
//! `{caller: …}` is: `present` and `payload` are no source keywords, and any other mapping using
//! those keys is a nested mapping, as it was before `ess/27`. A document below `ess/27` whose struct
//! happens to have one of these shapes is read back as that nested mapping by
//! [`read_below_ess_27`], so no earlier document changes meaning.

use std::collections::BTreeMap;

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};

use super::{CommandSpec, Outcome, PayloadField, PayloadSource, RawPayloadSource};
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::{is_assignable, Field};
use crate::Specification;

/// The key a payload read is written under: `{payload: <Event>.<field>}`.
pub const PAYLOAD_KEY: &str = "payload";
/// The key beside `generated` that requires the minted value present.
pub const PRESENT_KEY: &str = "present";
/// The keyword a mint is written with.
const GENERATED_KEY: &str = "generated";

/// `{generated: true, present: true}`, written as exactly these two keys.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPresentMint {
    /// `true`: the implementation mints the value.
    generated: bool,
    /// `true`: the minted value is present.
    present: bool,
}

impl RawPresentMint {
    /// The mapping `{generated: true, present: true}`, in either order, and nothing else.
    pub(super) fn recognise(entries: &[(String, RawPayloadSource)]) -> Option<Self> {
        let [(first, RawPayloadSource::Boolean(true)), (second, RawPayloadSource::Boolean(true))] =
            entries
        else {
            return None;
        };
        let keys = [first.as_str(), second.as_str()];
        (keys == [GENERATED_KEY, PRESENT_KEY] || keys == [PRESENT_KEY, GENERATED_KEY]).then_some(
            Self {
                generated: true,
                present: true,
            },
        )
    }

    /// The written form of a present mint.
    pub(super) fn of() -> Self {
        Self {
            generated: true,
            present: true,
        }
    }
}

/// `{payload: <Event>.<field>}`: the value this outcome publishes in `<field>` of `<Event>`.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPayloadRead {
    /// The qualified event name, a dot, and the field: `demo.routing.EntryAgentSelected.seq`.
    payload: String,
}

impl RawPayloadRead {
    /// The mapping `{payload: <qualified event>.<field>}`, and nothing else.
    pub(super) fn recognise(entries: &[(String, RawPayloadSource)]) -> Option<Self> {
        let [(key, RawPayloadSource::Text(text))] = entries else {
            return None;
        };
        (key == PAYLOAD_KEY && split(text).is_some()).then(|| Self {
            payload: text.clone(),
        })
    }

    /// The written form of a payload read.
    pub(super) fn of(event: &QualifiedName, field: &str) -> Self {
        Self {
            payload: format!("{event}.{field}"),
        }
    }

    /// The source it is.
    pub(super) fn into_source(self) -> PayloadSource {
        let (event, field) = split(&self.payload).expect("recognised only where it splits");
        PayloadSource::PayloadRead { event, field }
    }
}

/// `<qualified event>.<field>` split at its last dot, where both halves are well formed.
fn split(text: &str) -> Option<(QualifiedName, String)> {
    let (event, field) = text.rsplit_once('.')?;
    let event = event.parse::<QualifiedName>().ok()?;
    crate::binding::is_field_name(field).then(|| (event, field.to_owned()))
}

/// Reads every present mint and payload read of a document below `ess/27` back as the nested
/// mapping it was before that format existed.
pub fn read_below_ess_27(
    format: FormatVersion,
    commands: &mut BTreeMap<QualifiedName, CommandSpec>,
) {
    if format.major() >= FormatVersion::V27.major() {
        return;
    }
    for outcome in commands
        .values_mut()
        .flat_map(|command| command.outcomes.iter_mut())
    {
        for source in outcome
            .payload
            .values_mut()
            .flat_map(|fields| fields.values_mut())
            .chain(outcome.sets.values_mut())
            .chain(outcome.error_payload.values_mut())
            .chain(
                outcome
                    .set_effects
                    .affects
                    .iter_mut()
                    .flat_map(|affect| affect.sets.values_mut()),
            )
        {
            nested(source);
        }
    }
}

fn nested(source: &mut PayloadSource) {
    match source {
        PayloadSource::Generated { present: true } => {
            *source = PayloadSource::Struct {
                fields: [GENERATED_KEY, PRESENT_KEY]
                    .into_iter()
                    .map(|key| PayloadField {
                        target: key.to_owned(),
                        source: PayloadSource::Scalar {
                            value: "true".to_owned(),
                            scalar: super::ScalarKind::Boolean,
                        },
                    })
                    .collect(),
            };
        }
        PayloadSource::PayloadRead { event, field } => {
            *source = PayloadSource::Struct {
                fields: vec![PayloadField {
                    target: PAYLOAD_KEY.to_owned(),
                    source: PayloadSource::parse(&format!("{event}.{field}")),
                }],
            };
        }
        PayloadSource::Struct { fields } => {
            for leaf in fields {
                nested(&mut leaf.source);
            }
        }
        _ => {}
    }
}

/// `{generated: true, present: true}` over `target`: refused where the target is required, which a
/// mint fills present already.
pub(super) fn check_present(at: &ConstructRef, target: &Field) -> Option<ValidationError> {
    if target.type_ref.is_optional() {
        return None;
    }
    Some(
        ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!(
                "`{}` is `{}`, which a mint fills present already; `present: true` says a minted \
                 value of an `Optional` field is present",
                target.name, target.type_ref
            ),
        )
        .with_hint("write `{generated: true}`"),
    )
}

/// Why a nested mapping refused over a field that is no struct is one of this module's shapes
/// written where it is not one: below `ess/27`, or with `present:` other than `true`.
pub(super) fn misread_mapping(
    format: FormatVersion,
    at: &ConstructRef,
    target: &Field,
    fields: &[PayloadField],
) -> Option<ValidationError> {
    let keys: Vec<&str> = fields.iter().map(|field| field.target.as_str()).collect();
    let present = matches!(
        keys.as_slice(),
        [GENERATED_KEY, PRESENT_KEY] | [PRESENT_KEY, GENERATED_KEY]
    );
    let payload = keys == [PAYLOAD_KEY];
    if !present && !payload {
        return None;
    }
    let shape = if present {
        "`{generated: true, present: true}`"
    } else {
        "`{payload: <Event>.<field>}`"
    };
    if format.major() < FormatVersion::V27.major() {
        return Some(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                format!(
                    "{shape} is a source from specification format ess/27; below it this mapping \
                     fills a struct's fields, and `{}` is `{}`",
                    target.name, target.type_ref
                ),
            )
            .with_hint("declare `format: ess/27`"),
        );
    }
    Some(
        ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            if present {
                format!(
                    "`{}` is `{}`, and `generated:` and `present:` beside it each take `true`",
                    target.name, target.type_ref
                )
            } else {
                format!(
                    "`{}` is `{}`, and `payload:` names `<qualified event>.<field>`",
                    target.name, target.type_ref
                )
            },
        )
        .with_hint(
            "write `{generated: true, present: true}` for a present mint, or \
             `{generated: true}` where the value may be absent",
        ),
    )
}

/// Where a payload read is written: in `sets:` only.
pub(super) fn read_elsewhere(at: &ConstructRef) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::TypeMismatch,
        "`{payload: <Event>.<field>}` stores a value this outcome publishes, and is a `sets:` \
         source only",
    )
    .with_hint(
        "give the field the source the payload field has; one mint feeds a payload field and a \
         `sets:` entry that reads it",
    )
}

/// `{payload: <event>.<field>}` over the stored `target`: the event is one this outcome fills
/// under `payload:`, the field one it gives a source there, and its type assignable to the
/// target's without a conversion.
pub(super) fn check_read(
    spec: &Specification,
    outcome: &Outcome,
    at: &ConstructRef,
    target: &Field,
    (event, field): (&QualifiedName, &str),
    errors: &mut ValidationErrors,
) {
    let filled = outcome
        .payload
        .get(event)
        .filter(|_| outcome.emits.contains(event));
    let Some(filled) = filled else {
        let named: Vec<String> = outcome.payload.keys().map(ToString::to_string).collect();
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!(
                    "`{event}` is not an event this outcome emits and fills under `payload:`, so \
                     it publishes no `{event}.{field}` to store"
                ),
            )
            .with_hint(if named.is_empty() {
                "this outcome fills no event under `payload:`".to_owned()
            } else {
                format!("this outcome fills: {}", named.join(", "))
            }),
        );
        return;
    };
    let Some(declared) = spec
        .events()
        .get(event)
        .and_then(|declared| declared.field(field))
    else {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UndeclaredReference,
            format!("`{field}` is not a field `{event}` carries"),
        ));
        return;
    };
    if !filled.contains_key(field) {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!(
                    "this outcome's `payload:` gives `{event}.{field}` no source, so there is no \
                     published value to store"
                ),
            )
            .with_hint(format!(
                "give `{field}` a source under `payload: {{{event}: …}}`, `{{generated: true}}` for \
                 a value the implementation mints"
            )),
        );
        return;
    }
    if !is_assignable(&declared.type_ref, &target.type_ref) {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{event}.{field}` has type `{}`, and `{}` requires `{}`; a stored payload \
                     value is the published value, so no conversion applies",
                    declared.type_ref, target.name, target.type_ref
                ),
            )
            .with_hint("make the two types agree"),
        );
    }
}
