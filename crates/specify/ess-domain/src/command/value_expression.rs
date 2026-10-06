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
                    paths: super::input_path::admitted(Some(spec.system().format)),
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
            errors.extend(identity_paths(&context, &site));
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

/// Decision 8 of the design's final review (`ess/22`, A4): a dotted input path supplies the
/// identity a creating branch names its instance by only when nothing on its route — the last
/// segment included — may be absent. Existence then reads it as it reads a top-level input.
fn identity_paths(context: &Context<'_>, site: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let outcome = context.outcome;
    let Some(subject) = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == Effect::Creates && context.resolved.paths)
    else {
        return errors;
    };
    let mut sources: Vec<(ConstructRef, &PayloadSource)> = outcome
        .payload
        .iter()
        .filter_map(|(event, fields)| {
            fields.get(&subject.instance).map(|source| {
                (
                    site.clone()
                        .key("payload")
                        .named(event.to_string())
                        .named(&subject.instance),
                    source,
                )
            })
        })
        .collect();
    if let Some(entity) = context.spec.entities().get(&subject.entity) {
        if let Some(source) = outcome.sets.get(&entity.identity.name) {
            sources.push((
                site.clone().key("sets").named(&entity.identity.name),
                source,
            ));
        }
    }
    for (at, source) in sources {
        let (PayloadSource::InputField { field } | PayloadSource::InputOrGenerated { field, .. }) =
            source
        else {
            continue;
        };
        if !super::input_path::is_path(field) {
            continue;
        }
        let resolved = super::input_path::resolve(context.command, context.resolved.types, field);
        if resolved.is_ok_and(|path| path.may_be_absent()) {
            errors.push(super::input_path::optional_route(&at, field, "an identity"));
        }
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
            paths: super::input_path::admitted(Some(spec.system().format)),
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
        PayloadSource::RelatedField { .. } | PayloadSource::RelatedSelection { .. } => {
            check_related(context, at, target, source, errors);
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
        PayloadSource::RelatedSelection { .. } => "`{related: {entity, where, field}}`",
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
            // Decision 8 (`ess/22`, A4): a creating branch reads the address from the input path
            // that fills the field, and a path supplies one only when its whole route is
            // required.
            let carried = context
                .subject
                .filter(|_| created.is_some())
                .and_then(|(entity, _)| subject_field_from_input(context.outcome, entity, name))
                .filter(|path| super::input_path::is_path(path));
            if let Some(path) = carried {
                let resolved = super::input_path::resolve(command, context.resolved.types, path);
                if resolved.is_ok_and(|path| path.may_be_absent()) {
                    errors.push(super::input_path::optional_route(
                        at,
                        path,
                        "the address of another row",
                    ));
                    return None;
                }
            }
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

/// One reference a `{related: …}` source follows: its `via`, or (ess/22, beyond10x/ess#285) the
/// field of the row `via` names that a chained `via:` names second.
#[derive(Clone, Copy)]
enum Reference<'r> {
    Via(&'r RelatedVia),
    Hop {
        entity: &'r crate::entity::EntitySpec,
        field: &'r str,
    },
}

impl std::fmt::Display for Reference<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Via(via) => write!(f, "`{via}`"),
            Self::Hop { entity, field } => write!(f, "`{}.{field}`", entity.name),
        }
    }
}

/// The one entity a reference of type `via_type` names, and whether the reference may be absent;
/// or a refusal saying why there is none: a reference that is several, one that may be absent
/// below `ess/22`, a type that is no entity's identity, or one several entities share with no
/// relation to decide.
///
/// From `ess/22` (beyond10x/ess#285) a reference may be `Optional<…>` of an identity: the row is
/// read where it is present, and the value is absent where it is not.
fn related_entity<'a>(
    context: &Context<'a>,
    at: &ConstructRef,
    reference: Reference<'_>,
    via_type: &TypeRef,
    carrier: Option<(&crate::entity::EntitySpec, &str)>,
    errors: &mut ValidationErrors,
) -> Option<(&'a crate::entity::EntitySpec, bool)> {
    let (identity_type, absent) = reference_identity(context, at, reference, via_type, errors)?;
    let entities = match referenced_entity(context.spec, identity_type, carrier) {
        Referenced::Entity(entity) => return Some((entity, absent)),
        Referenced::NoEntity => {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!("{reference} is `{via_type}`, which is no entity's identity"),
            ));
            return None;
        }
        Referenced::Ambiguous(entities) => entities,
    };
    errors.push(ambiguous(context, at, reference, via_type, &entities));
    None
}

/// The identity type a reference of `via_type` carries, and whether it may be absent; or a
/// refusal: a collection, or below `ess/22` an `Optional<…>`.
fn reference_identity<'t>(
    context: &Context<'_>,
    at: &ConstructRef,
    reference: Reference<'_>,
    via_type: &'t TypeRef,
    errors: &mut ValidationErrors,
) -> Option<(&'t TypeRef, bool)> {
    let optional_admitted = context.spec.system().format.major() >= FormatVersion::V22.major();
    Some(match via_type {
        TypeRef::Primitive(_) | TypeRef::Named(_) => (via_type, false),
        TypeRef::Optional(inner)
            if optional_admitted
                && matches!(**inner, TypeRef::Primitive(_) | TypeRef::Named(_)) =>
        {
            (&**inner, true)
        }
        TypeRef::Optional(inner)
            if matches!(**inner, TypeRef::Primitive(_) | TypeRef::Named(_)) =>
        {
            errors.push(
                ValidationError::at(
                    at.clone(),
                    ValidationCode::TypeMismatch,
                    format!(
                        "{reference} is `{via_type}`, and below specification format ess/22 \
                         `{{related: …}}` follows one reference that is always there"
                    ),
                )
                .with_hint(
                    "declare `format: ess/22` to read through an Optional reference — the value \
                     is then absent where the reference is — or read a required field typed as \
                     the other entity's identity",
                ),
            );
            return None;
        }
        _ => {
            errors.push(
                ValidationError::at(
                    at.clone(),
                    ValidationCode::TypeMismatch,
                    format!(
                        "{reference} is `{via_type}`, and `{{related: …}}` follows one reference \
                         that is always there{}",
                        if optional_admitted {
                            " or, from ess/22, one that may be absent"
                        } else {
                            ""
                        }
                    ),
                )
                .with_hint("read a field typed as the other entity's identity"),
            );
            return None;
        }
    })
}

/// The refusal of a reference whose type several entities are identified by, with no relation to
/// say which, and a hint naming a remedy that validates where the reference is read.
fn ambiguous(
    context: &Context<'_>,
    at: &ConstructRef,
    reference: Reference<'_>,
    via_type: &TypeRef,
    entities: &[&crate::entity::EntitySpec],
) -> ValidationError {
    let named = entities
        .iter()
        .map(|entity| format!("`{}`", entity.name))
        .collect::<Vec<_>>()
        .join(" and ");
    let via = match reference {
        Reference::Via(via) => via,
        Reference::Hop { entity, field } => {
            return ValidationError::at(
                at.clone(),
                ValidationCode::ConflictingDeclaration,
                format!("{reference} is `{via_type}`, the identity of {named}"),
            )
            .with_hint(format!(
                "say which: declare `relations: [{{name: …, kind: references, target: \
                 <entity>, cardinality: one, via: {field}}}]` on `{}`",
                entity.name
            ));
        }
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
    ValidationError::at(
        at.clone(),
        ValidationCode::ConflictingDeclaration,
        format!("`{via}` is `{via_type}`, the identity of {named}"),
    )
    .with_hint(hint)
}

/// `{related: {via, field}}` (`ess/16`, beyond10x/ess#166): `field` of the row `via` names.
///
/// `via` is a field of the subject before the outcome (so an existing subject, as for
/// `{subject: …}`) or of the input, typed as exactly one entity's identity — below `ess/22` never
/// `Optional`, and never a collection, because the source reads one row. A `references` relation
/// the subject declares on `via` says which entity where the type alone names several.
///
/// From `ess/22` (beyond10x/ess#285) `via` may be `Optional<…>` of an identity, and `through`
/// names one further reference: a field of the row `via` names, typed the same way, which the
/// relation that row's entity declares on it decides as the subject's does. Where any reference
/// may be absent, so may the value, and the target must admit that.
fn check_related(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    source: &PayloadSource,
    errors: &mut ValidationErrors,
) {
    if matches!(source, PayloadSource::RelatedSelection { .. }) {
        check_selection(context, at, target, source, errors);
        return;
    }
    let PayloadSource::RelatedField {
        via,
        through,
        field,
    } = source
    else {
        return;
    };
    if let Some(refused) = related_shape_refused(context, at, via, through, field) {
        errors.push(refused);
        return;
    }
    let Some((via_type, carrier)) = related_via(context, at, via, errors) else {
        return;
    };
    let carrier = carrier
        .as_ref()
        .map(|(entity, field)| (*entity, field.as_str()));
    let Some((mut entity, mut absent)) =
        related_entity(context, at, Reference::Via(via), via_type, carrier, errors)
    else {
        return;
    };
    for hop in through {
        let Some(held) = entity_field(entity, hop) else {
            errors.push(not_a_field(at, entity, hop));
            return;
        };
        let reference = Reference::Hop { entity, field: hop };
        let Some((next, may_be_absent)) = related_entity(
            context,
            at,
            reference,
            &held.type_ref,
            Some((entity, hop.as_str())),
            errors,
        ) else {
            return;
        };
        entity = next;
        absent |= may_be_absent;
    }
    let Some(read) = entity_field(entity, field) else {
        errors.push(not_a_field(at, entity, field));
        return;
    };
    let value_type = if absent && !matches!(read.type_ref, TypeRef::Optional(_)) {
        TypeRef::Optional(Box::new(read.type_ref.clone()))
    } else {
        read.type_ref.clone()
    };
    let conversions = &context.resolved.conversions;
    if conversions.permits(&value_type, &target.type_ref) {
        return;
    }
    if absent && conversions.permits(&read.type_ref, &target.type_ref) {
        errors.push(absent_into_required(at, entity, field, target));
        return;
    }
    errors.push(mismatch(
        at,
        &format!("`{}.{field}`", entity.name),
        &value_type,
        target,
    ));
}

/// The refusal of a `{related: …}` source's shape where the format or the names refuse it: below
/// `ess/16`; a chained `via:` below `ess/22` (beyond10x/ess#285); a name that is not one field.
fn related_shape_refused(
    context: &Context<'_>,
    at: &ConstructRef,
    via: &RelatedVia,
    through: &[String],
    field: &str,
) -> Option<ValidationError> {
    let format = context.spec.system().format;
    if format.major() < FormatVersion::V16.major() {
        return Some(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a `{related: …}` source requires specification format ess/16",
            )
            .with_hint("write `format: ess/16` on the source that declares the system"),
        );
    }
    if !through.is_empty() && format.major() < FormatVersion::V22.major() {
        return Some(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a chained `via:` — `{related: {via: [<field>, <field>], …}}`, read across two \
                 references — requires specification format ess/22",
            )
            .with_hint(
                "declare `format: ess/22`, or copy the value onto the row the first reference \
                 names and read it in one hop",
            ),
        );
    }
    if !is_field_name(via.field())
        || !is_field_name(field)
        || through.iter().any(|hop| !is_field_name(hop))
    {
        // Below ess/22 the sentence it always was; from ess/22 it names the chained form too.
        let named = if format.major() >= FormatVersion::V22.major() {
            "`{related: …}` names one field in `via:` (a field of the subject, or \
             `input.<field>`, or a list of that and one field of the row it names) and one field \
             in `field:`"
        } else {
            "`{related: …}` names one field in `via:` (a field of the subject, or \
             `input.<field>`) and one field in `field:`"
        };
        return Some(ValidationError::at(
            at.clone(),
            ValidationCode::UndeclaredReference,
            named,
        ));
    }
    None
}

/// The refusal of a value read through a reference that may be absent into a target that cannot be
/// left absent (ess/22, beyond10x/ess#285).
fn absent_into_required(
    at: &ConstructRef,
    entity: &crate::entity::EntitySpec,
    field: &str,
    target: &Field,
) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::TypeMismatch,
        format!(
            "`{}.{field}` is read through a reference that may be absent, and then the value is \
             absent too; `{}` requires `{}`, which cannot be left absent",
            entity.name, target.name, target.type_ref
        ),
    )
    .with_hint(format!(
        "declare `{}` as `Optional<{}>`, or read through references that are always there",
        target.name, target.type_ref
    ))
}

/// The field `name` of `entity`, its identity included.
fn entity_field<'e>(entity: &'e crate::entity::EntitySpec, name: &str) -> Option<&'e Field> {
    if entity.identity.name == name {
        Some(&entity.identity)
    } else {
        entity.field(name)
    }
}

/// `name` is no field of `entity`, with the fields it holds as the hint.
fn not_a_field(
    at: &ConstructRef,
    entity: &crate::entity::EntitySpec,
    name: &str,
) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UndeclaredReference,
        format!("`{name}` is not a field of `{}`", entity.name),
    )
    .with_hint(format!(
        "`{}` holds: {}",
        entity.name,
        std::iter::once(entity.identity.name.as_str())
            .chain(entity.fields.iter().map(|field| field.name.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// The subject's field, before this outcome.
fn check_subject(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    field: &str,
    errors: &mut ValidationErrors,
) {
    if field == crate::entity::EntitySpec::STATE {
        check_subject_state(context, at, target, errors);
        return;
    }
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

/// `{subject: state}` (ess/23, beyond10x/ess#458): the lifecycle state the row held immediately
/// before the outcome, spelled as `when_subject`, `when_related` and invariants read it. Admitted
/// wherever `{subject: …}` is — an existing row is read — and typed as the entity's own `State`,
/// which no declared field can be called; below `ess/23` it is refused naming `ess/23`.
fn check_subject_state(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    errors: &mut ValidationErrors,
) {
    if context.spec.system().format.major() < FormatVersion::V23.major() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "`{subject: state}`, the state the record held before the outcome, requires \
                 specification format ess/23",
            )
            .with_hint(
                "declare `format: ess/23`, or fill the field from the input or a literal state",
            ),
        );
        return;
    }
    // The row the refusal or the outcome reads, by the rule every `{subject: …}` follows; the
    // identity stands in for the row, since `state` is no declared field.
    let Some((entity, _)) = context.subject else {
        existing_subject_field(context, at, "`{subject: …}`", "state", errors);
        return;
    };
    if existing_subject_field(context, at, "`{subject: …}`", &entity.identity.name, errors)
        .is_none()
    {
        return;
    }
    let state = TypeRef::Named(entity.name.child(crate::entity::EntitySpec::STATE_TYPE));
    if !context
        .resolved
        .conversions
        .permits(&state, &target.type_ref)
    {
        errors.push(mismatch(at, "the subject's `state`", &state, target));
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
    // From `ess/22` (A4) the input read may be a path, and the fallback another input path.
    let fallback_read = match otherwise {
        Some(PayloadSource::InputField { field }) => Some(field.as_str()),
        _ => None,
    };
    if !context.resolved.paths {
        let below = if super::input_path::is_path(field) {
            Some("`{input: <path>, else: …}`")
        } else {
            fallback_read.map(|_| "an input after `else:`")
        };
        if let Some(what) = below {
            errors.push(super::input_path::below_ess_22(at, what));
            return;
        }
    }
    if let Some(literal) = otherwise.filter(|_| fallback_read.is_none()) {
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
    let read = match command.read_input((context.resolved.types, context.resolved.paths), field) {
        Ok(read) => read,
        Err(Some(unresolved)) => {
            errors.push(unresolved.refusal(at, command, field));
            return;
        }
        Err(None) => {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!("`{field}` is not an input of `{}`", command.name),
            ));
            return;
        }
    };
    if let Some(other) = fallback_read {
        check_input_fallback(context, at, target, other, errors);
    }
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

/// `else: input.<path>` (`ess/22`, A4): the fallback reads another input, which must be present
/// whenever the request is valid — required along its whole route — because `else:` promises a
/// value; and its type must fill the target as a plain `input.` source would.
fn check_input_fallback(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    field: &str,
    errors: &mut ValidationErrors,
) {
    let command = context.command;
    let read = match command.read_input((context.resolved.types, context.resolved.paths), field) {
        Ok(read) => read,
        Err(Some(unresolved)) => {
            errors.push(unresolved.refusal(at, command, field));
            return;
        }
        Err(None) => {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!("`{field}` is not an input of `{}`", command.name),
            ));
            return;
        }
    };
    // A newtype declared over an `Optional` may be absent as surely as the `Optional` itself.
    if context
        .resolved
        .types
        .newtype_layers(&read.type_ref)
        .optional
    {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::TypeMismatch,
                format!(
                    "`input.{field}` is `{}`, which may be absent, and `else:` promises a value",
                    read.type_ref
                ),
            )
            .with_hint(
                "fall back to an input that is required along its whole route, or to a literal",
            ),
        );
        return;
    }
    let conversions = context.resolved.conversions;
    if !conversions.permits(&read.type_ref, target.type_ref.required())
        && !conversions.permits(&read.type_ref, &target.type_ref)
    {
        errors.push(mismatch(
            at,
            &format!("`input.{field}`"),
            &read.type_ref,
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
        command
            .response
            .iter()
            .find(|read| read.name == field)
            .cloned()
    } else {
        // From `ess/22` a leaf may read a member of a struct input (A4).
        match command.read_input((context.resolved.types, context.resolved.paths), field) {
            Ok(read) => Some(read),
            Err(Some(unresolved)) => {
                errors.push(unresolved.refusal(at, command, field));
                return;
            }
            Err(None) => None,
        }
    };
    let Some(read) = read.as_ref() else {
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

/// `{related: {entity, where, field}}` (ess/22, beyond10x/ess#299): a declared entity, a selector
/// typed over its candidate row, the input and the addressed subject — as a row-set guard's is
/// ([`super::row_set`]) — and a field of that entity whose type the target takes.
fn check_selection(
    context: &Context<'_>,
    at: &ConstructRef,
    target: &Field,
    source: &PayloadSource,
    errors: &mut ValidationErrors,
) {
    let PayloadSource::RelatedSelection {
        selection,
        field,
        legacy,
    } = source
    else {
        return;
    };
    let spec = context.spec;
    if spec.system().format.major() < FormatVersion::V22.major() {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "a filtered read — `{related: {entity, where, field}}` — requires specification \
                 format ess/22",
            )
            .with_hint("declare `format: ess/22`"),
        );
        return;
    }
    if let Some(why) = &legacy.predicate {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!(
                "the `where:` of `{{related: {{entity, where, field}}}}` is no predicate: {why}"
            ),
        ));
        return;
    }
    let Some(entity) = spec.entities().get(&selection.entity) else {
        errors.push(super::row_set::undeclared(spec, at, &selection.entity));
        return;
    };
    if selection.filter == ess_primitives::predicate::Predicate::Always {
        errors.push(
            ValidationError::at(
                at.clone(),
                ValidationCode::EmptyDeclaration,
                format!(
                    "the filtered read selects every row of `{}`: its `where` selects every row",
                    entity.name
                ),
            )
            .with_hint(
                "select by a field the input scopes, as `where: tenant_id == input.tenant_id`",
            ),
        );
        return;
    }
    let registry = spec.types_with_lifecycles(&mut ValidationErrors::new());
    let checked = super::row_set::check(
        spec,
        context.command,
        entity,
        &registry,
        &selection.filter,
        at,
    );
    if !checked.is_empty() {
        errors.extend(checked);
        return;
    }
    let Some(read) = entity_field(entity, field) else {
        errors.push(not_a_field(at, entity, field));
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
