//! Typed admission of facts on an existing subject: one enum field equal to a variant (ess/6), or
//! a predicate over the subject's declared stored fields (ess/9).
//!
//! `docs/design/cross-record-and-stored-field-guards.md` is the binding design for the second. Both
//! shapes read the same subject at the same point — the row the command addresses, immediately
//! before selection — and both enter one joint partition here: the stored fields the guards name,
//! crossed with the input fields the `when:` guards name.
use super::{finite, CommandSpec, Effect, InstanceSurface, Outcome, OutcomeCondition, Subject};
use crate::{
    entity::EntitySpec,
    expression::DomainEnvironment,
    spec::Specification,
    types::{TypeBody, TypeRef, TypeRegistry},
};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use std::fmt::Write as _;

/// Whether any branch of this command reads the existing subject's stored fields.
pub fn uses(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| outcome.condition.reads_subject_fact())
}

/// Whether any branch of this command uses the ess/9 predicate form.
pub fn uses_predicate(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::SubjectPredicate { .. }))
}

/// The existing subject a subject-fact command reads: the one its subject-bearing branches name.
///
/// A refusal guarded by the predicate form names no subject of its own and reads this one. Which
/// branch supplies it does not matter where the command is valid — every subject-bearing branch
/// must name the same entity and identity, which [`validate`] checks — so the first declared one
/// answers, and a command whose only subject-bearing branch creates has none.
pub fn common_subject(command: &CommandSpec) -> Option<&Subject> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find(|subject| subject.surface() == InstanceSurface::CommandInput)
}

/// The subject one branch reads its stored fields from: its own, or the command's common one.
pub fn reading_subject<'a>(command: &'a CommandSpec, outcome: &'a Outcome) -> Option<&'a Subject> {
    outcome
        .subject
        .as_ref()
        .filter(|subject| subject.surface() == InstanceSurface::CommandInput)
        .or_else(|| common_subject(command))
}

/// Local declaration checks, which need no registry and no entity.
///
/// The partition waits for [`validate`], where the entity's field types are known; until then a
/// command reading stored facts is checked for what its own branches say: one default at most, one
/// wrong-state branch at most, a branch a scenario can reach, and a subject for every predicate
/// that names none.
pub fn validate_shape(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let unconditional: Vec<_> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.is_unconditional())
        .map(|outcome| &outcome.name)
        .collect();
    if unconditional.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all unconditional, so the result of `{}` is not determined \
                     by its input or its subject",
                    super::join(unconditional.iter()),
                    command.name
                ),
            )
            .with_hint("give all but one of them a `when` or a `when_subject`"),
        );
    }
    if common_subject(command).is_none() {
        for outcome in command.outcomes.iter().filter(|outcome| {
            matches!(outcome.condition, OutcomeCondition::SubjectPredicate { .. })
                && outcome.subject.is_none()
        }) {
            errors.push(
                ValidationError::at(
                    command
                        .site()
                        .key("outcomes")
                        .named(outcome.name.as_str())
                        .key("when_subject"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` reads the stored fields of the subject its siblings name, \
                         and no branch of `{}` names an existing subject",
                        outcome.name, command.name
                    ),
                )
                .with_hint(
                    "declare the branch that moves, updates or preserves the subject beside it; a \
                     command whose only subject is the one it creates has no stored row to read",
                ),
            );
        }
    }
    let reachable = command
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome.is_testable_from_input()
                || outcome.test_strategy() == super::TestStrategy::ObserveSubjectFact
        })
        .count();
    errors.extend(command.validate_reachable_and_wrong_state(reachable));
    errors
}

/// Check the facts against the entity that owns them, and partition the stored fields jointly
/// with the input.
pub fn validate(spec: &Specification, types: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for command in spec.commands().values().filter(|command| uses(command)) {
        let Some(common) = common_subject(command) else {
            // Reported by `validate_shape`, under the key the author wrote.
            continue;
        };
        for outcome in &command.outcomes {
            if !outcome.condition.reads_subject_fact() {
                continue;
            }
            let site = command
                .site()
                .key("outcomes")
                .named(outcome.name.as_str())
                .key("when_subject");
            let subject = reading_subject(command, outcome).unwrap_or(common);
            if (&subject.entity, &subject.instance) != (&common.entity, &common.instance) {
                errors.push(ValidationError::at(
                    site.clone(),
                    ValidationCode::ConflictingDeclaration,
                    "subject fact branches must share one entity, identity",
                ));
                continue;
            }
            let Some(entity) = spec.entities().get(&subject.entity) else {
                // An undeclared subject entity is refused where references are resolved.
                continue;
            };
            match &outcome.condition {
                OutcomeCondition::SubjectField { field, equals, .. } => {
                    if !declares_variant(entity, types, field, equals) {
                        errors.push(ValidationError::at(
                            site,
                            ValidationCode::UndeclaredReference,
                            "subject fact must name a declared enum field and one of its variants",
                        ));
                    }
                }
                OutcomeCondition::SubjectPredicate { predicate, .. } => {
                    errors.extend(check(entity, types, predicate, &site));
                }
                _ => {}
            }
        }
        if command
            .outcomes
            .iter()
            .any(|outcome| outcome.condition.reads_held_state())
        {
            errors.push(ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                "subject fact and lifecycle guards cannot be combined in one command",
            ));
            continue;
        }
        let mut shared = true;
        for subject in command
            .outcomes
            .iter()
            .filter_map(|outcome| outcome.subject.as_ref())
        {
            if subject.effect == Effect::Creates
                || subject.entity != common.entity
                || subject.instance != common.instance
            {
                shared = false;
                errors.push(ValidationError::at(
                    command.site().key("outcomes"),
                    ValidationCode::ConflictingDeclaration,
                    "subject fact selection requires one existing subject identity",
                ));
            }
        }
        if let (true, Some(entity)) = (shared, spec.entities().get(&common.entity)) {
            errors.extend(validate_partition(command, entity, types));
        }
    }
    errors
}

/// Whether `field` is a declared enum field of `entity` and `equals` one of its variants.
fn declares_variant(entity: &EntitySpec, types: &TypeRegistry, field: &str, equals: &str) -> bool {
    entity
        .fields
        .iter()
        .find(|candidate| candidate.name == field)
        .and_then(|candidate| match &candidate.type_ref {
            TypeRef::Named(name) => types.get(name),
            _ => None,
        })
        .is_some_and(|named| match &named.body {
            TypeBody::Enum { variants } => variants.iter().any(|variant| variant.name() == equals),
            _ => false,
        })
}

/// The expression checker, over the entity's declared fields and nothing else.
///
/// The environment `entity.rs` builds for invariants, minus the `state` pseudo-field: the lifecycle
/// stays with `when_subject_state:` and `when_state_changes:`. The checker's own diagnostics are
/// kept — `unobservable_fact` for a root the entity does not declare, `type_mismatch` and
/// `undeclared_reference` for a literal — and sited at the key the author wrote.
fn check(
    entity: &EntitySpec,
    types: &TypeRegistry,
    predicate: &ess_primitives::predicate::Predicate,
    site: &ess_primitives::error::ConstructRef,
) -> ValidationErrors {
    let owner = site.render();
    let environment = DomainEnvironment::new(types, &entity.fields);
    let checked = crate::expression::check_predicate(&environment, predicate, &owner);
    let mut errors = ValidationErrors::new();
    for error in &checked.errors {
        let mut diagnostic = error.validation_error();
        if let Some(path) = &error.path {
            if error.segment.as_deref() == Some(path.namespace())
                && !entity
                    .fields
                    .iter()
                    .any(|field| field.name == path.namespace())
            {
                write!(
                    diagnostic.message,
                    "; `when_subject` reads the stored fields of `{}`, and `{}` is not one",
                    entity.name,
                    path.namespace()
                )
                .expect("writing to a String");
                diagnostic.hint = Some(format!(
                    "stored fields: {}; the input stays in `when:` and the lifecycle state in \
                     `when_subject_state:`",
                    super::join(entity.fields.iter().map(|field| &field.name))
                ));
            }
        }
        errors.push(diagnostic);
    }
    errors
}

/// The joint stored-field × input partition, under the finite prover's caps.
///
/// Closed domains are what the prover admits — equality and membership against enum variants over
/// non-optional paths — and an open one, `weight_kg > 20`, makes it decline; the command then needs
/// a genuine default, as an open input guard does. Where it proves, every joint assignment must
/// select exactly one branch, the default counting where none of the guarded ones does.
fn validate_partition(
    command: &CommandSpec,
    entity: &EntitySpec,
    types: &TypeRegistry,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let guarded: Vec<&Outcome> = command
        .outcomes
        .iter()
        .filter(|outcome| {
            !outcome.is_unconditional()
                && !matches!(
                    outcome.condition,
                    OutcomeCondition::External { .. }
                        | OutcomeCondition::ExternalWhen { .. }
                        | OutcomeCondition::WrongState
                )
        })
        .collect();
    let stored: Vec<_> = guarded
        .iter()
        .map(|outcome| outcome.condition.subject_predicate())
        .collect();
    let guards: Vec<_> = guarded
        .iter()
        .zip(&stored)
        .map(|(outcome, fields)| finite::FieldGuard {
            fields: fields.as_ref(),
            input: outcome.condition.predicate(),
        })
        .collect();
    let default = command.default_outcome();
    let Some(cases) = finite::analyze_with_fields(
        &DomainEnvironment::new(types, &entity.fields),
        &DomainEnvironment::new(types, &command.input),
        &guards,
    ) else {
        if default.is_none() {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes"),
                    ValidationCode::NonExhaustiveBranches,
                    "subject-fact/input coverage is open, unsupported, or exceeds 64 joint \
                     assignments; declare a genuine default",
                )
                .with_hint(
                    "drop the guard from the branch that answers every other stored row — usually \
                     the success beside the refusal",
                ),
            );
        }
        return errors;
    };
    for case in cases {
        let selected: Vec<&Outcome> = if case.selected.is_empty() {
            default.into_iter().collect()
        } else {
            case.selected.iter().map(|index| guarded[*index]).collect()
        };
        if selected.len() == 1 {
            continue;
        }
        let assignment = |values: &std::collections::BTreeMap<_, String>| {
            values
                .iter()
                .map(|(path, value)| format!("{path} = {value}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            if selected.is_empty() {
                ValidationCode::NonExhaustiveBranches
            } else {
                ValidationCode::ConflictingDeclaration
            },
            format!(
                "stored [{}] and input [{}] select {} branches: {}",
                assignment(&case.fields),
                assignment(&case.input),
                selected.len(),
                selected
                    .iter()
                    .map(|outcome| outcome.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    errors
}
