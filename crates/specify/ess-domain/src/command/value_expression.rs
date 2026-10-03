//! Value expressions in `payload:` and `sets:` (source format `ess/14`,
//! `docs/design/value-expressions.md`).
//!
//! One pass over every outcome, after assembly, because each rule needs more than one declaration
//! in hand: `{subject: …}` needs the entity the outcome acts on, `{input: …, else: …}` the
//! command's input, and a nested mapping the struct its target resolves to. The sources that
//! existed before `ess/14` keep their own checks in `validate_payloads` and `validate_sets`; this
//! pass checks the new ones, and the leaves of a nested mapping, which those two never see.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::Number;

use super::related_value::{
    input_carrier, referenced_entity, subject_field_from_input, Referenced,
};
use super::{
    literal_representation, scalar_representation, CommandSpec, Effect, Outcome, PayloadSource,
    RelatedVia, Resolved, ScalarKind,
};
use crate::binding::{is_field_name, representation, Representation, Resolution};

use crate::system::FormatVersion;
use crate::types::{Field, Primitive, TypeRef};
use crate::Specification;

/// How deep a nested mapping may go before it is refused, whatever the type allows.
const MAX_DEPTH: usize = 32;

/// Where a source is written, because two of the new sources are admitted in one place only.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Payload,
    Sets,
}

/// Everything one outcome's sources are checked against.
struct Context<'a> {
    spec: &'a Specification,
    command: &'a CommandSpec,
    outcome: &'a Outcome,
    resolved: Resolved<'a>,
    /// The entity the outcome acts on, and whether it existed before the outcome ran.
    subject: Option<(&'a crate::entity::EntitySpec, bool)>,
}

/// Checks every `ess/14` source, the leaves of nested mappings, and `subject.<field>` literals.
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let types = &spec.system().types;
    let inhabitation = crate::system::Inhabitation::of(types);
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let subject = outcome
                .subject
                .as_ref()
                .map(|subject| {
                    let existing = matches!(subject.effect, Effect::Moves { .. } | Effect::Updates);
                    (&subject.entity, existing)
                })
                // The rows a set subject changes exist before it (ess/16, `set_effects`).
                .or(outcome
                    .set_effects
                    .instances
                    .as_ref()
                    .map(|set| (&set.entity, true)))
                .and_then(|(entity, existing)| {
                    spec.entities().get(entity).map(|entity| (entity, existing))
                });
            let context = Context {
                spec,
                command,
                outcome,
                resolved: Resolved {
                    types,
                    conversions: spec.conversions(),
                    inhabitation: &inhabitation,
                },
                subject,
            };
            let site = command.site().key("outcomes").named(outcome.name.as_str());
            for (event_name, fields) in &outcome.payload {
                let Some(event) = spec.events().get(event_name) else {
                    continue;
                };
                for (target, source) in fields {
                    let Some(filled) = event.field(target) else {
                        continue;
                    };
                    let at = site
                        .clone()
                        .key("payload")
                        .named(event_name.to_string())
                        .named(target);
                    check(
                        &context,
                        &at,
                        Place::Payload,
                        (filled, filled),
                        source,
                        0,
                        &mut errors,
                    );
                    errors.extend(fallback_literal(
                        &context,
                        &Filled::Payload { at: &at, event },
                        filled,
                        source,
                    ));
                }
            }
            errors.extend(error_payload(&context, &site));
            if let Some((entity, _)) = context.subject {
                for (target, source) in &outcome.sets {
                    let held = if entity.identity.name == *target {
                        Some(&entity.identity)
                    } else {
                        entity.field(target)
                    };
                    let Some(held) = held else {
                        continue;
                    };
                    let at = site.clone().key("sets").named(target);
                    check(
                        &context,
                        &at,
                        Place::Sets,
                        (held, held),
                        source,
                        0,
                        &mut errors,
                    );
                    errors.extend(fallback_literal(
                        &context,
                        &Filled::Sets { entity },
                        held,
                        source,
                    ));
                }
            }
        }
    }
    // The caller (ess/16, #168): actor attributes, and every guard that reads one.
    errors.extend(super::caller_value::validate(spec));
    // Set effects (ess/16, #167, #175): `instances:`, `affects:` and `{count: changed}`.
    errors.extend(super::set_effects::validate(spec));
    errors
}

/// The `payload:` block keyed by the error an outcome reports (ess/19,
/// `story:error-payload-sources`), checked entry by entry by the rules an event's payload is held
/// to: the field must be one the error declares, and the source's type the field's.
///
/// `{subject: …}` reads the row the refusal is answered for, which only a branch answered after
/// the row is read has: `wrong_state:`, a held-state or stored-field guard, or a branch acting on
/// an existing subject of its own. An input-guarded refusal answers before any row is read
/// (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order"), and an unknown
/// identity has none.
fn error_payload(context: &Context<'_>, site: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let outcome = context.outcome;
    let Some(name) = &outcome.error else {
        return errors;
    };
    if outcome.error_payload.is_empty() {
        return errors;
    }
    let Some(error) = context.spec.errors().get(name) else {
        return errors;
    };
    // An error's fields are filled as an event's are, so they are checked by the one rule: the
    // error, seen as the record the branch fills.
    let carried = super::EventSpec {
        name: error.name.clone(),
        fields: error.fields.clone(),
        naming: error.naming.clone(),
    };
    let row = super::subject_fact::error_subject(context.command, outcome)
        .and_then(|subject| context.spec.entities().get(&subject.entity))
        .map(|entity| (entity, true));
    let context = Context {
        spec: context.spec,
        command: context.command,
        outcome,
        resolved: context.resolved,
        subject: row,
    };
    for (target, source) in &outcome.error_payload {
        let at = site
            .clone()
            .key("payload")
            .named(name.to_string())
            .named(target);
        errors.extend(super::check_payload_entry(
            &at,
            (context.command, outcome),
            &carried,
            target,
            source,
            context.resolved,
        ));
        let Some(filled) = carried.field(target) else {
            continue;
        };
        check(
            &context,
            &at,
            Place::Payload,
            (filled, filled),
            source,
            0,
            &mut errors,
        );
        errors.extend(fallback_literal(
            &context,
            &Filled::Payload {
                at: &at,
                event: &carried,
            },
            filled,
            source,
        ));
    }
    errors
}

/// The `sets:` of one `affects:` entry (ess/16, #175), checked as a subject's own over `entity`,
/// whose rows exist before the outcome.
pub(super) fn validate_affect(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &Outcome,
    entity: &crate::entity::EntitySpec,
    sets: &std::collections::BTreeMap<String, PayloadSource>,
    at: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let types = &spec.system().types;
    let inhabitation = crate::system::Inhabitation::of(types);
    let context = Context {
        spec,
        command,
        outcome,
        resolved: Resolved {
            types,
            conversions: spec.conversions(),
            inhabitation: &inhabitation,
        },
        subject: Some((entity, true)),
    };
    for (target, source) in sets {
        let held = if entity.identity.name == *target {
            Some(&entity.identity)
        } else {
            entity.field(target)
        };
        let Some(held) = held else {
            continue;
        };
        let site = at.clone().key("sets").named(target);
        check(
            &context,
            &site,
            Place::Sets,
            (held, held),
            source,
            0,
            &mut errors,
        );
        errors.extend(fallback_literal(
            &context,
            &Filled::Sets { entity },
            held,
            source,
        ));
    }
    errors
}

/// One source against the field it fills. `depth` counts nested mappings; a top-level source that
/// existed before `ess/14` is checked elsewhere and only its `subject.` spelling is read here.
fn check(
    context: &Context<'_>,
    at: &ConstructRef,
    place: Place,
    targets: (&Field, &Field),
    source: &PayloadSource,
    depth: usize,
    errors: &mut ValidationErrors,
) {
    let (root, target) = targets;
    let format = context.spec.system().format;
    if source.needs_value_expressions() && format.major() < FormatVersion::V14.major() {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedFormatVersion,
            format!(
                "a {} source requires specification format {}",
                kind(source),
                if matches!(
                    source,
                    PayloadSource::RelatedField { .. } | PayloadSource::CallerAttribute { .. }
                ) {
                    "ess/16"
                } else {
                    "ess/14"
                }
            ),
        ));
        return;
    }
    match source {
        PayloadSource::Literal { value } => {
            if let Some(field) = value
                .strip_prefix("subject.")
                .filter(|rest| is_field_name(rest))
            {
                errors.push(
                    ValidationError::at(
                        at.clone(),
                        ValidationCode::MisspelledReference,
                        format!(
                            "`{value}` reads as the literal text `{value}`, not as the addressed \
                             entity's field"
                        ),
                    )
                    .with_hint(format!(
                        "write `{{subject: {field}}}` (format ess/14) to read the entity's value \
                         before this outcome"
                    )),
                );
            } else if depth > 0 {
                refuse_literal(context, at, place, target, value, None, errors);
            }
        }
        PayloadSource::Scalar { value, scalar } if depth > 0 => {
            refuse_literal(context, at, place, target, value, Some(*scalar), errors);
        }
        PayloadSource::InputField { field } if depth > 0 => {
            check_read(context, at, target, field, false, errors);
        }
        PayloadSource::ResponseField { field } if depth > 0 => {
            check_read(context, at, target, field, true, errors);
        }
        PayloadSource::Cleared if depth > 0 => errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            "`{cleared: true}` clears a whole entity field, not a field inside a nested mapping",
        )),
        PayloadSource::SubjectField { field } => check_subject(context, at, target, field, errors),
        PayloadSource::RelatedField { via, field } => {
            check_related(context, at, target, via, field, errors);
        }
        PayloadSource::CallerAttribute { attribute } => {
            super::caller_value::check_source(
                context.spec,
                context.command,
                at,
                target,
                attribute,
                errors,
            );
        }
        PayloadSource::Increment { by, scalar } => {
            check_increment(context, at, place, (root, target), by, *scalar, errors);
        }
        PayloadSource::InputOrGenerated { field, otherwise } => {
            check_fallback(
                context,
                at,
                (place, depth),
                target,
                field,
                otherwise.as_deref(),
                errors,
            );
        }
        PayloadSource::Struct { fields } => {
            check_struct(context, at, place, (root, target), fields, depth, errors);
        }
        PayloadSource::ChangedCount if depth > 0 || place == Place::Sets => {
            errors.push(super::set_effects::count_elsewhere(at));
        }
        PayloadSource::ChangedCount
        | PayloadSource::Scalar { .. }
        | PayloadSource::InputField { .. }
        | PayloadSource::ResponseField { .. }
        | PayloadSource::Cleared
        | PayloadSource::Generated => {}
    }
}

fn kind(source: &PayloadSource) -> &'static str {
    match source {
        PayloadSource::SubjectField { .. } => "`{subject: …}`",
        PayloadSource::Increment { .. } => "`{increment: …}`",
        PayloadSource::InputOrGenerated { .. } => "`{input: …, else: …}`",
        PayloadSource::Struct { .. } => "nested mapping",
        PayloadSource::RelatedField { .. } => "`{related: …}`",
        PayloadSource::CallerAttribute { .. } => "`{caller: …}`",
        PayloadSource::ChangedCount => "`{count: changed}`",
        _ => "payload",
    }
}

/// The type of the field a `{related: …}` source's `via` reads, and the subject field whose
/// relation may say which entity it names; or a refusal saying why there is no such field.
///
/// A subject field is read as it was before the outcome, so an existing subject's. On `creates:`
/// there is no row before, but a field the branch sets from its input holds that input, so it is
/// admitted there and read as the input (`subject_field_from_input`, which counts the identity the
/// branch fills from its input too). An input is carried by the subject field the branch sets from
/// it, or by the identity it names the instance by, where there is one (`input_carrier`).
fn related_via<'a>(
    context: &Context<'a>,
    at: &ConstructRef,
    via: &RelatedVia,
    errors: &mut ValidationErrors,
) -> Option<(&'a TypeRef, Option<(&'a crate::entity::EntitySpec, String)>)> {
    let command = context.command;
    match via {
        RelatedVia::Subject(name) => {
            let created = context
                .subject
                .filter(|(_, existing)| !existing)
                .filter(|(entity, _)| {
                    subject_field_from_input(context.outcome, entity, name).is_some()
                })
                .and_then(|(entity, _)| entity.field(name).or(Some(&entity.identity)))
                .filter(|held| held.name == *name);
            let held = if let Some(held) = created {
                held
            } else {
                existing_subject_field(context, at, "`{related: …}`", name, errors)?
            };
            Some((
                &held.type_ref,
                context.subject.map(|(entity, _)| (entity, name.clone())),
            ))
        }
        RelatedVia::Input(name) => {
            let Some(read) = command.input_field(name) else {
                errors.push(ValidationError::at(
                    at.clone(),
                    ValidationCode::UndeclaredReference,
                    format!("`{name}` is not an input of `{}`", command.name),
                ));
                return None;
            };
            Some((
                &read.type_ref,
                input_carrier(
                    context.outcome,
                    context.subject.map(|(entity, _)| entity),
                    name,
                ),
            ))
        }
    }
}

/// The one entity a `{related: …}` source's `via`, of type `via_type`, names, or a refusal saying
/// why there is none: a reference that may be absent or is several, a type that is no entity's
/// identity, or one several entities share with no relation to decide.
fn related_entity<'a>(
    context: &Context<'a>,
    at: &ConstructRef,
    via: &RelatedVia,
    via_type: &TypeRef,
    carrier: Option<(&crate::entity::EntitySpec, &str)>,
    errors: &mut ValidationErrors,
) -> Option<&'a crate::entity::EntitySpec> {
    if !matches!(via_type, TypeRef::Primitive(_) | TypeRef::Named(_)) {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{via}` is `{via_type}`, and `{{related: …}}` follows one reference that is \
                     always there"
                ),
            )
            .with_hint("read a required field typed as the other entity's identity"),
        );
        return None;
    }
    let entities = match referenced_entity(context.spec, via_type, carrier) {
        Referenced::Entity(entity) => return Some(entity),
        Referenced::NoEntity => {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!("`{via}` is `{via_type}`, which is no entity's identity"),
            ));
            return None;
        }
        Referenced::Ambiguous(entities) => entities,
    };
    let hint = match (via, context.subject) {
        (RelatedVia::Subject(name), Some((subject, _))) => format!(
            "say which: declare `relations: [{{name: …, kind: references, target: <entity>, \
             cardinality: one, via: {name}}}]` on `{}`, or, where that entity owns `{}`, an \
             `owns` relation on it with `via: {name}`",
            subject.name, subject.name
        ),
        (RelatedVia::Input(name), Some((subject, existing))) => {
            let setter = context
                .outcome
                .sets
                .iter()
                .find_map(|(target, source)| match source {
                    PayloadSource::InputField { field } if field == name => Some(target.as_str()),
                    _ => None,
                });
            let row = if existing {
                "the row before the outcome"
            } else {
                "a `creates:` branch has no row before it, so a field it sets from the input"
            };
            match setter {
                Some(target) => format!(
                    "read the identity through a field of the subject that a `references` \
                     relation carries — {row}: declare `relations: [{{name: …, kind: \
                     references, target: <entity>, cardinality: one, via: {target}}}]` on `{}`, \
                     then `input.{name}` or `via: {target}` names that entity",
                    subject.name
                ),
                None => format!(
                    "read the identity through a field of the subject that a `references` \
                     relation carries — {row}: set a field of `{}` from `input.{name}` and \
                     declare that relation on it",
                    subject.name
                ),
            }
        }
        _ => "read the identity through a field of the subject that a `references` relation \
              carries; this branch acts on no entity, so give the entities distinct identity \
              types instead"
            .to_owned(),
    };
    errors.push(
        ValidationError::at(
            at.clone(),
            ValidationCode::ConflictingDeclaration,
            format!(
                "`{via}` is `{via_type}`, the identity of {}",
                entities
                    .iter()
                    .map(|entity| format!("`{}`", entity.name))
                    .collect::<Vec<_>>()
                    .join(" and ")
            ),
        )
        .with_hint(hint),
    );
    None
}

/// `{related: {via, field}}` (`ess/16`, beyond10x/ess#166): `field` of the row `via` names.
///
/// `via` is a field of the subject before the outcome (so an existing subject, as for
/// `{subject: …}`) or of the input, typed as exactly one entity's identity — never `Optional` or a
/// collection, because the source reads one row that is always there. A `references` relation the
/// subject declares on `via` says which entity where the type alone names several.
fn check_related(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    via: &RelatedVia,
    field: &str,
    errors: &mut ValidationErrors,
) {
    if context.spec.system().format.major() < FormatVersion::V16.major() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a `{related: …}` source requires specification format ess/16",
            )
            .with_hint("write `format: ess/16` on the source that declares the system"),
        );
        return;
    }
    if !is_field_name(via.field()) || !is_field_name(field) {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UndeclaredReference,
            "`{related: …}` names one field in `via:` (a field of the subject, or \
             `input.<field>`) and one field in `field:`",
        ));
        return;
    }
    let Some((via_type, carrier)) = related_via(context, at, via, errors) else {
        return;
    };
    let carrier = carrier
        .as_ref()
        .map(|(entity, field)| (*entity, field.as_str()));
    let Some(entity) = related_entity(context, at, via, via_type, carrier, errors) else {
        return;
    };
    let read = if entity.identity.name == field {
        Some(&entity.identity)
    } else {
        entity.field(field)
    };
    let Some(read) = read else {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!("`{field}` is not a field of `{}`", entity.name),
            )
            .with_hint(format!(
                "`{}` holds: {}",
                entity.name,
                std::iter::once(entity.identity.name.as_str())
                    .chain(entity.fields.iter().map(|field| field.name.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
        return;
    };
    if !context
        .resolved
        .conversions
        .permits(&read.type_ref, &target.type_ref)
    {
        errors.push(mismatch(
            at,
            &format!("`{}.{field}`", entity.name),
            &read.type_ref,
            target,
        ));
    }
}

/// The subject's field, before this outcome.
fn check_subject(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    field: &str,
    errors: &mut ValidationErrors,
) {
    let Some(held) = existing_subject_field(context, at, "`{subject: …}`", field, errors) else {
        return;
    };
    if !context
        .resolved
        .conversions
        .permits(&held.type_ref, &target.type_ref)
    {
        errors.push(mismatch(
            at,
            &format!("the subject's `{field}`"),
            &held.type_ref,
            target,
        ));
    }
}

/// The entity field an outcome on an existing subject reads, or a refusal saying why there is none.
fn existing_subject_field<'a>(
    context: &Context<'a>,
    at: &ConstructRef,
    what: &str,
    field: &str,
    errors: &mut ValidationErrors,
) -> Option<&'a Field> {
    let outcome = context.outcome;
    let Some((entity, existing)) = context.subject else {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UndeclaredReference,
            format!(
                "{what} reads the entity an outcome acts on, and outcome `{}` acts on no entity",
                outcome.name
            ),
        ));
        return None;
    };
    if !existing {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "{what} reads `{}` as it was before outcome `{}`, and a `creates:` outcome \
                     has no row before it",
                    entity.name, outcome.name
                ),
            )
            .with_hint("read the value from the command's input instead"),
        );
        return None;
    }
    let held = if entity.identity.name == field {
        Some(&entity.identity)
    } else {
        entity.field(field)
    };
    if held.is_none() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!("`{field}` is not a field of `{}`", entity.name),
            )
            .with_hint(format!(
                "`{}` holds: {}",
                entity.name,
                std::iter::once(entity.identity.name.as_str())
                    .chain(entity.fields.iter().map(|field| field.name.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
    }
    held
}

/// `{increment: n}`: a `sets:` source over a required number of an existing subject.
fn check_increment(
    context: &Context<'_>,
    at: &ConstructRef,
    place: Place,
    targets: (&Field, &Field),
    by: &str,
    scalar: ScalarKind,
    errors: &mut ValidationErrors,
) {
    let (root, target) = targets;
    if place != Place::Sets {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::ConflictingDeclaration,
                "`{increment: …}` changes a stored field and is a `sets:` source only",
            )
            .with_hint("to publish the new value, set it and read it with `{subject: …}` later"),
        );
        return;
    }
    if existing_subject_field(context, at, "`{increment: …}`", &root.name, errors).is_none() {
        return;
    }
    if target.type_ref.is_optional() {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!(
                "`{}` is `{}`, and an absent value has nothing to add to",
                target.name, target.type_ref
            ),
        ));
        return;
    }
    let primitive = match representation(
        &target.type_ref,
        context.resolved.types,
        context.resolved.inhabitation,
    ) {
        Resolution::Established(Representation::Primitive(primitive)) => primitive,
        Resolution::Undeclared | Resolution::Uninhabited => return,
        _ => Primitive::String,
    };
    let admitted = match primitive {
        Primitive::Integer => {
            scalar == ScalarKind::Integer
                && by
                    .parse::<i64>()
                    .is_ok_and(|n| n != 0 && n.to_string() == by)
        }
        Primitive::Decimal => {
            Number::decimal_literal(by).is_some_and(|n| n.get() != 0.0)
                || (scalar == ScalarKind::Integer && by.parse::<i64>().is_ok_and(|n| n != 0))
        }
        _ => {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{}` is `{}`, and only an `Integer` or a `Decimal` field can be incremented",
                    target.name, target.type_ref
                ),
            ));
            return;
        }
    };
    if !admitted {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{by}` is not a non-zero amount of `{}`, which is `{primitive}` underneath",
                    target.name
                ),
            )
            .with_hint(match primitive {
                Primitive::Integer => "an `Integer` field takes a whole number such as `1` or `-1`",
                _ => "a `Decimal` field takes a decimal such as `0.5` or `-1`",
            }),
        );
    }
}

/// `{input: f, else: {generated: true}}`: an optional input, or a minted value; or
/// `{input: f, else: <literal>}` (`ess/16`, #163): an optional input, or the literal, which is held
/// to the rule a literal written in the target's place is.
fn check_fallback(
    context: &Context<'_>,
    at: &ConstructRef,
    (place, depth): (Place, usize),
    target: &Field,
    field: &str,
    otherwise: Option<&PayloadSource>,
    errors: &mut ValidationErrors,
) {
    if let Some(literal) = otherwise {
        let format = context.spec.system().format;
        if format.major() < FormatVersion::V16.major() {
            errors.push(
                ValidationError::at(
                    at.clone(),
                    ValidationCode::UnsupportedFormatVersion,
                    "a literal after `else:` requires specification format ess/16",
                )
                .with_hint(
                    "write `format: ess/16` on the source that declares the system, or \
                     `else: {generated: true}`",
                ),
            );
            return;
        }
        // The rule a literal written in the target's place gets. A top-level fallback is read at
        // depth 0, the `subject.<field>` misspelling only, because [`fallback_literal`] runs the
        // bare payload or `sets:` literal's own checks on it, as `validate_payloads` and
        // `validate_sets` do on a bare one; inside a nested mapping the leaf rule at depth 1
        // checks both the misspelling and the literal's type against the target.
        let depth = usize::from(depth > 0);
        check(context, at, place, (target, target), literal, depth, errors);
    }
    let command = context.command;
    let Some(read) = command.input_field(field) else {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UndeclaredReference,
            format!("`{field}` is not an input of `{}`", command.name),
        ));
        return;
    };
    if !read.type_ref.is_optional() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{}.{field}` is `{}`, which is always sent, so `else:` never applies",
                    command.name, read.type_ref
                ),
            )
            .with_hint(format!("write `input.{field}`")),
        );
        return;
    }
    let present = read.type_ref.required();
    let conversions = context.resolved.conversions;
    if !conversions.permits(present, target.type_ref.required())
        && !conversions.permits(present, &target.type_ref)
    {
        errors.push(mismatch(
            at,
            &format!("`{}.{field}` when present", command.name),
            present,
            target,
        ));
    }
}

/// Where a top-level literal after `else:` is written: an event's payload field, or the addressed
/// entity's field in `sets:`.
enum Filled<'a> {
    Payload {
        at: &'a ConstructRef,
        event: &'a super::EventSpec,
    },
    Sets {
        entity: &'a crate::entity::EntitySpec,
    },
}

/// The literal after `else:` in a top-level payload or `sets:` field, checked by the rule a bare
/// literal written there is held to — `check_payload_literal` or `sets_literal` — and refused under
/// the same code, the same owner and the same path, so the two refusals read the same.
///
/// Two things differ, both hints, because the bare one's repair is not one `else:` admits: a
/// misspelled reference is repaired by a literal or `{generated: true}` (`else:` reads no input),
/// and a quoted spelling keeps the `{input, else}` around it. Nothing below `ess/16`, where the
/// fallback itself is refused.
fn fallback_literal(
    context: &Context<'_>,
    filled: &Filled<'_>,
    target: &Field,
    source: &PayloadSource,
) -> ValidationErrors {
    let PayloadSource::InputOrGenerated {
        field,
        otherwise: Some(literal),
    } = source
    else {
        return ValidationErrors::new();
    };
    if context.spec.system().format.major() < FormatVersion::V16.major() {
        return ValidationErrors::new();
    }
    let (value, scalar) = match literal.as_ref() {
        PayloadSource::Literal { value } => (value, None),
        PayloadSource::Scalar { value, scalar } => (value, Some(*scalar)),
        _ => return ValidationErrors::new(),
    };
    let command = context.command;
    let errors = match filled {
        Filled::Payload { at, event } => super::check_payload_literal(
            at,
            command,
            event,
            &target.name,
            target,
            value,
            scalar,
            context.resolved,
        ),
        Filled::Sets { entity } => {
            let Some(refusal) = super::sets_literal(
                &entity.name,
                &target.name,
                target,
                literal,
                command,
                context.resolved,
            ) else {
                return ValidationErrors::new();
            };
            let at = format!(
                "commands.{}.outcomes.{}.sets.{}",
                command.name, context.outcome.name, target.name
            );
            ValidationErrors::from(
                ValidationError::new(ValidationCode::TypeMismatch, at, refusal.reason)
                    .with_hint(refusal.hint),
            )
        }
    };
    let mut out = ValidationErrors::new();
    for mut error in errors {
        error.hint = error.hint.map(|hint| {
            if error.code == ValidationCode::MisspelledReference {
                format!(
                    "`else:` reads no input: write a literal `{}` admits, or \
                     `else: {{generated: true}}`",
                    target.name
                )
            } else {
                within_fallback(&hint, &target.name, field)
            }
        });
        out.push(error);
    }
    out
}

/// A bare literal's repair, quoting `label: '0'`, rewritten as the fallback's own:
/// `label: {input: label, else: '0'}`. A hint that spells no `target: …` is left as it is.
fn within_fallback(hint: &str, target: &str, field: &str) -> String {
    let opening = format!("`{target}: ");
    let Some(start) = hint.find(&opening) else {
        return hint.to_owned();
    };
    let value_at = start + opening.len();
    let Some(length) = hint[value_at..].find('`') else {
        return hint.to_owned();
    };
    format!(
        "{}{target}: {{input: {field}, else: {}}}{}",
        &hint[..=start],
        &hint[value_at..value_at + length],
        &hint[value_at + length..]
    )
}

/// The refusal for a nested mapping over a non-struct target that was meant as `{related: …}`:
/// the exact shape below `ess/16` (read there as a nested mapping, `related_value::read_below_ess_16`)
/// is refused as the format it needs, and any other mapping keyed `related` says the one shape
/// that is a source.
fn related_mapping(
    context: &Context<'_>,
    at: &ConstructRef,
    fields: &[super::PayloadField],
) -> Option<ValidationError> {
    let [written] = fields else {
        return fields
            .iter()
            .any(|field| field.target == "related")
            .then(|| {
                ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                "`{related: …}` is written alone: a mapping with other keys beside `related` is \
                 a nested mapping",
            )
            .with_hint("`{related: …}` takes exactly `{via: <field>, field: <field>}`")
            });
    };
    if written.target != "related" {
        return None;
    }
    let exact = matches!(&written.source, PayloadSource::Struct { fields: inner }
    if inner.len() == 2 && inner.iter().all(|leaf| {
        matches!(leaf.target.as_str(), "via" | "field")
            && matches!(leaf.source, PayloadSource::InputField { .. } | PayloadSource::Literal { .. })
    }));
    if exact && context.spec.system().format.major() < FormatVersion::V16.major() {
        return Some(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a `{related: …}` source requires specification format ess/16",
            )
            .with_hint("write `format: ess/16` on the source that declares the system"),
        );
    }
    Some(
        ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            "a mapping under `related` that is not exactly `{via, field}` is a nested mapping",
        )
        .with_hint("`{related: …}` takes exactly `{via: <field>, field: <field>}`"),
    )
}

/// A nested mapping: every field of the struct the target resolves to, one source each.
fn check_struct(
    context: &Context<'_>,
    at: &ConstructRef,
    place: Place,
    targets: (&Field, &Field),
    fields: &[super::PayloadField],
    depth: usize,
    errors: &mut ValidationErrors,
) {
    let (root, target) = targets;
    if depth >= MAX_DEPTH {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!("nested mappings stop at {MAX_DEPTH} levels"),
        ));
        return;
    }
    let Some(declared) = context.resolved.types.struct_fields(&target.type_ref) else {
        if let Some(refusal) = related_mapping(context, at, fields) {
            errors.push(refusal);
            return;
        }
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`{}` is `{}`, and a nested mapping fills a struct's fields",
                    target.name, target.type_ref
                ),
            )
            .with_hint(
                "a mapping whose keys are all source keywords is a source; any other key makes \
                 it a nested mapping",
            ),
        );
        return;
    };
    for field in fields {
        let Some(inner) = declared
            .iter()
            .find(|declared| declared.name == field.target)
        else {
            errors.push(
                ValidationError::at(
                    at.clone().named(&field.target),
                    ValidationCode::UndeclaredReference,
                    format!("`{}` is not a field of `{}`", field.target, target.type_ref),
                )
                .with_hint(format!(
                    "`{}` has: {}",
                    target.type_ref,
                    declared
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
            continue;
        };
        check(
            context,
            &at.clone().named(&field.target),
            place,
            (root, inner),
            &field.source,
            depth + 1,
            errors,
        );
    }
    let missing: Vec<&str> = declared
        .iter()
        .filter(|declared| !fields.iter().any(|field| field.target == declared.name))
        .map(|declared| declared.name.as_str())
        .collect();
    if !missing.is_empty() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::EmptyDeclaration,
                format!(
                    "the nested mapping for `{}` gives no source for {}",
                    target.name,
                    missing
                        .iter()
                        .map(|name| format!("`{name}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
            .with_hint(
                "give each one a source; `{generated: true}` leaves it to the implementation",
            ),
        );
    }
}

/// A leaf literal inside a nested mapping, by the rule a top-level literal is held to.
fn refuse_literal(
    context: &Context<'_>,
    at: &ConstructRef,
    place: Place,
    target: &Field,
    value: &str,
    scalar: Option<ScalarKind>,
    errors: &mut ValidationErrors,
) {
    let place = match place {
        Place::Payload => "payload",
        Place::Sets => "`sets:` entry",
    };
    let owner = &context.command.name;
    let refusal = match scalar {
        Some(scalar) => scalar_representation(
            owner,
            &target.name,
            target,
            value,
            scalar,
            place,
            context.command,
            context.resolved,
        ),
        None => literal_representation(
            owner,
            &target.name,
            target,
            value,
            place,
            context.command,
            context.resolved,
        ),
    };
    if let Some(refusal) = refusal {
        errors.push(
            ValidationError::at(at.clone(), ValidationCode::TypeMismatch, refusal.reason)
                .with_hint(refusal.hint),
        );
    }
}

/// A leaf `input.<field>` or `{response: <field>}` inside a nested mapping.
fn check_read(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    field: &str,
    response: bool,
    errors: &mut ValidationErrors,
) {
    let command = context.command;
    let read = if response {
        command.response.iter().find(|read| read.name == field)
    } else {
        command.input_field(field)
    };
    let Some(read) = read else {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UndeclaredReference,
            format!(
                "`{field}` has no declared source field on `{}`",
                command.name
            ),
        ));
        return;
    };
    // A leaf reads the input as a top-level entry does: narrowed from `ess/16` (#169), with the
    // declared type and its declared crossings first.
    let conversions = context.resolved.conversions;
    let admitted = if response {
        conversions.permits(&read.type_ref, &target.type_ref)
    } else {
        let format = Some(context.spec.system().format);
        command.admits_input_read(context.outcome, read, &target.type_ref, conversions, format)
    };
    if !admitted {
        errors.push(mismatch(
            at,
            &format!("`{}.{field}`", command.name),
            &read.type_ref,
            target,
        ));
    }
}

fn mismatch(at: &ConstructRef, source: &str, from: &TypeRef, target: &Field) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::TypeMismatch,
        format!(
            "{source} has type `{from}`, and `{}` requires `{}`; no conversion is declared",
            target.name, target.type_ref
        ),
    )
    .with_hint(format!(
        "declare the crossing — `conversions: [{{from: {from}, to: {}, because: …}}]` — or make \
         the two types agree",
        target.type_ref
    ))
}
