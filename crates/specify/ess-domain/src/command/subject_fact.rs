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

/// The existing row a refusal's `payload:` reads `{subject: …}` from (ess/19,
/// `story:error-payload-sources`), where the refusal is answered after that row is read: its own
/// existing subject, else for `wrong_state:` and a held-state or stored-field guard the subject its
/// siblings name. An input-guarded refusal answers before any row is read
/// (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order") and an unknown
/// identity has none, so both are `None`.
pub fn error_subject<'a>(command: &'a CommandSpec, outcome: &'a Outcome) -> Option<&'a Subject> {
    if let Some(subject) = &outcome.subject {
        return (!matches!(subject.effect, Effect::Creates)).then_some(subject);
    }
    let reads_row = matches!(
        outcome.condition,
        OutcomeCondition::WrongState
            | OutcomeCondition::SubjectState { .. }
            | OutcomeCondition::StateChange { .. }
            | OutcomeCondition::SubjectField { .. }
            | OutcomeCondition::SubjectPredicate { .. }
    );
    reads_row.then(|| common_subject(command)).flatten()
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
                    errors.extend(check(command, entity, types, predicate, &site));
                    if reads_state(predicate) && admits_state(types) {
                        errors.extend(state_moves(outcome, entity, predicate, &site));
                    }
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

/// A branch that reads the held state in its `when_subject` predicate and takes a move must be
/// able to take it in every state the predicate may select it in (ess/18, beyond10x/ess#204).
///
/// The author named the state, so a state the predicate leaves open that the move does not start
/// from is a contradiction in the branch itself, reported where it is written — the same rule
/// `when_subject_state:` is held to. Each declared state is bound as `state` alone; where the
/// predicate is then not decided false, the branch can be selected there.
fn state_moves(
    outcome: &Outcome,
    entity: &EntitySpec,
    predicate: &ess_primitives::predicate::Predicate,
    site: &ess_primitives::error::ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let Some(Effect::Moves { transition }) =
        outcome.subject.as_ref().map(|subject| &subject.effect)
    else {
        return errors;
    };
    let Some(declared) = entity
        .states
        .transitions
        .iter()
        .find(|declared| declared.name == *transition)
    else {
        return errors;
    };
    let Ok(path) = ess_primitives::facts::FactPath::new(EntitySpec::STATE) else {
        return errors;
    };
    for state in entity
        .states
        .states
        .iter()
        .filter(|state| !declared.from.contains(*state))
    {
        let mut facts = ess_primitives::facts::FactStore::new();
        facts.set_if_absent(
            path.clone(),
            ess_primitives::facts::FactValue::text(state.as_str()),
        );
        if predicate.evaluate(&facts) != ess_primitives::predicate::Truth::False {
            errors.push(
                ValidationError::at(
                    site.clone(),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "`{predicate}` may select `{}` while the subject is `{state}`, and move \
                         `{transition}` does not start there",
                        outcome.name
                    ),
                )
                .with_hint(format!(
                    "narrow the predicate to the states `{transition}` starts from: {}",
                    super::join(declared.from.iter())
                )),
            );
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

/// The namespace a stored-field predicate reads the command's input under (beyond10x/ess#157).
pub const INPUT_NAMESPACE: &str = "input";

/// Whether a stored-field predicate of a command reading `entity` names the command's input.
///
/// A path rooted at `input.` with a field after it does, unless the entity declares a stored field
/// named `input`: that field keeps being read as itself, so a document that validated before
/// `ess/15` means under it what it meant. A bare `input` names no field, and keeps the
/// `unobservable_fact` refusal it always had rather than being told to upgrade.
pub fn reads_input(entity: &EntitySpec, predicate: &ess_primitives::predicate::Predicate) -> bool {
    !declares_input_field(entity)
        && predicate
            .fact_paths()
            .iter()
            .any(|path| path.namespace() == INPUT_NAMESPACE && path.segments().len() > 1)
}

/// Whether a stored-field predicate reads the held lifecycle state as `state` (ess/18,
/// beyond10x/ess#204).
///
/// The bare path `state`, the pseudo-field entity invariants already read. An entity cannot
/// declare a stored field of that name — it would shadow the lifecycle in every invariant — so the
/// path is never a stored field.
pub fn reads_state(predicate: &ess_primitives::predicate::Predicate) -> bool {
    predicate
        .fact_paths()
        .iter()
        .any(|path| path.namespace() == EntitySpec::STATE && path.segments().len() == 1)
}

/// The fields a `when_subject` predicate reads, in declaration order: the entity's stored fields,
/// then, where the format admits it (ess/18), the held state as `state`, typed by the lifecycle's
/// own synthesised enum so the partition treats it as the closed set it is.
///
/// A `when_related` predicate reads the related row through the same list, from `ess/20`
/// (beyond10x/ess#229).
pub(crate) fn readable_fields(entity: &EntitySpec, admits_state: bool) -> Vec<crate::types::Field> {
    let mut fields = entity.fields.clone();
    if admits_state
        && !entity
            .fields
            .iter()
            .any(|field| field.name == EntitySpec::STATE)
    {
        fields.push(crate::types::Field::new(
            EntitySpec::STATE,
            TypeRef::Named(entity.name.child(EntitySpec::STATE_TYPE)),
        ));
    }
    fields
}

/// Whether this build reads `state` in a `when_subject` predicate at the document's format.
fn admits_state(types: &TypeRegistry) -> bool {
    admits_state_from(types, crate::system::FormatVersion::V18)
}

/// Whether the document's format is `since` or later, where `state` is read in a predicate over a
/// row: `ess/18` for the addressed subject, `ess/20` for a related row.
pub(super) fn admits_state_from(types: &TypeRegistry, since: crate::system::FormatVersion) -> bool {
    types
        .format()
        .is_none_or(|format| format.major() >= since.major())
}

fn declares_input_field(entity: &EntitySpec) -> bool {
    entity
        .fields
        .iter()
        .any(|field| field.name == INPUT_NAMESPACE)
}

/// The expression checker, over the entity's declared fields and — from `ess/15` — the command's
/// input under `input.` (beyond10x/ess#157).
///
/// The environment `entity.rs` builds for invariants, minus the identity, and — below `ess/18` —
/// minus the `state` pseudo-field, which from `ess/18` reads the held lifecycle state
/// (beyond10x/ess#204). Below it `state` is refused with the format it needs, as an `input.`
/// operand is below `ess/15`. The checker's own diagnostics are
/// kept — `unobservable_fact` for a root the entity does not declare, `type_mismatch` and
/// `undeclared_reference` for a literal or an input field the command does not declare — and sited
/// at the key the author wrote. Below `ess/15` an `input.` operand is refused with the format it
/// needs, not with `unobservable_fact`, because the document is not wrong, only its header is.
fn check(
    command: &CommandSpec,
    entity: &EntitySpec,
    types: &TypeRegistry,
    predicate: &ess_primitives::predicate::Predicate,
    site: &ess_primitives::error::ConstructRef,
) -> ValidationErrors {
    let owner = site.render();
    let mut errors = ValidationErrors::new();
    // A predicate reading the caller (ess/16) is checked whole by `caller_value`, which has the
    // actors in hand.
    if super::caller_value::reads_caller(predicate, &entity.fields) {
        return errors;
    }
    let admits_input = types
        .format()
        .is_none_or(|format| format.major() >= crate::system::FormatVersion::V15.major());
    if reads_input(entity, predicate) && !admits_input {
        errors.push(
            ValidationError::at(
                site.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "an `input.` operand in a `when_subject` predicate requires specification format \
                 ess/15",
            )
            .with_hint("declare `format: ess/15`, or compare the stored field with a literal"),
        );
        return errors;
    }
    if reads_state(predicate) && !admits_state(types) {
        errors.push(
            ValidationError::at(
                site.clone(),
                ValidationCode::UnsupportedFormatVersion,
                "reading the held lifecycle state as `state` in a `when_subject` predicate \
                 requires specification format ess/18",
            )
            .with_hint("declare `format: ess/18`, or guard the state with `when_subject_state:`"),
        );
        return errors;
    }
    let readable = readable_fields(entity, admits_state(types));
    // The decision reads the row with the one instant it reads its input with (ess/22, A3).
    let mut environment = DomainEnvironment::new(types, &readable).with_stored_current_time();
    if admits_input && !declares_input_field(entity) {
        environment = environment.with_input(&command.input);
    }
    let checked = crate::expression::check_predicate(&environment, predicate, &owner);
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
                    "stored fields: {}; the input is read as `input.<field>` (ess/15) or in \
                     `when:`, and the lifecycle state in `when_subject_state:`",
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
                        | OutcomeCondition::UnknownInstance
                        | OutcomeCondition::InputAbsent
                        | OutcomeCondition::ExistingInstance
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
    let readable = readable_fields(entity, admits_state(types));
    let analyze = if default.is_some() {
        finite::analyze_enum_fields
    } else {
        finite::analyze_with_fields
    };
    let Some(cases) = analyze(
        &DomainEnvironment::new(types, &readable),
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
        let assignment =
            |values: &std::collections::BTreeMap<_, ess_primitives::facts::FactValue>| {
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
