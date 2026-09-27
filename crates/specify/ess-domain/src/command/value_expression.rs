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

use super::{
    literal_representation, scalar_representation, CommandSpec, Effect, Outcome, PayloadSource,
    Resolved, ScalarKind,
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
            let subject = outcome.subject.as_ref().and_then(|subject| {
                let existing = matches!(subject.effect, Effect::Moves { .. } | Effect::Updates);
                spec.entities()
                    .get(&subject.entity)
                    .map(|entity| (entity, existing))
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
                        filled,
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
                    check(&context, &at, Place::Sets, held, source, 0, &mut errors);
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
    errors
}

/// One source against the field it fills. `depth` counts nested mappings; a top-level source that
/// existed before `ess/14` is checked elsewhere and only its `subject.` spelling is read here.
fn check(
    context: &Context<'_>,
    at: &ConstructRef,
    place: Place,
    target: &Field,
    source: &PayloadSource,
    depth: usize,
    errors: &mut ValidationErrors,
) {
    let format = context.spec.system().format;
    if source.needs_value_expressions() && format.major() < FormatVersion::V14.major() {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedFormatVersion,
            format!(
                "a {} source requires specification format ess/14",
                kind(source)
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
        PayloadSource::Increment { by, scalar } => {
            check_increment(context, at, place, target, by, *scalar, errors);
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
            check_struct(context, at, place, target, fields, depth, errors);
        }
        PayloadSource::Scalar { .. }
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
        _ => "payload",
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
    target: &Field,
    by: &str,
    scalar: ScalarKind,
    errors: &mut ValidationErrors,
) {
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
    if existing_subject_field(context, at, "`{increment: …}`", &target.name, errors).is_none() {
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
        check(context, at, place, target, literal, depth, errors);
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

/// A nested mapping: every field of the struct the target resolves to, one source each.
fn check_struct(
    context: &Context<'_>,
    at: &ConstructRef,
    place: Place,
    target: &Field,
    fields: &[super::PayloadField],
    depth: usize,
    errors: &mut ValidationErrors,
) {
    if depth >= MAX_DEPTH {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!("nested mappings stop at {MAX_DEPTH} levels"),
        ));
        return;
    }
    let Some(declared) = context.resolved.types.struct_fields(&target.type_ref) else {
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
            inner,
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
