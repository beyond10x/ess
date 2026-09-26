//! Explicit held-state authority and the shared bounded state/input partition proof.
use super::{finite, CommandSpec, Effect, InstanceSurface, Outcome, OutcomeCondition};
use crate::{
    entity::{EntitySpec, StateName},
    expression::DomainEnvironment,
    spec::Specification,
    types::TypeRegistry,
};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use std::collections::BTreeSet;

/// Whether this command needs a held-state selection authority.
pub fn uses(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| outcome.condition.reads_held_state())
}

/// Whether this command names a held state literally, with `when_subject_state:`.
///
/// Separate from [`uses_state_changes`] because the two arrived in different source formats, and
/// the gate that refuses a construct to an older document has to be able to tell them apart.
pub fn uses_subject_state(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::SubjectState { .. }))
}

/// Whether this command tests a move against the state already held, with `when_state_changes:`.
pub fn uses_state_changes(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::StateChange { .. }))
}

/// The held states one branch's condition admits, or `None` when it reads no held state.
///
/// A literal guard admits the one state it names — whether or not the branch's move starts there,
/// because that disagreement is [`validate_move`]'s to report under the key the author wrote.
/// `when_state_changes:` admits a side of its own transition's `from` set, partitioned by whether
/// each state is the one the move arrives at: that derivation is the whole construct, and having it
/// in one function is what stops the partition prover and a consumer computing it differently.
///
/// `None` for a `moves:` naming a transition this entity does not declare. That is an undeclared
/// reference, already refused where references are resolved, and a second diagnostic calling the
/// branch unreachable would send the author to the wrong line.
fn admitted(outcome: &Outcome, entity: &EntitySpec) -> Option<BTreeSet<StateName>> {
    match &outcome.condition {
        OutcomeCondition::SubjectState { state, .. } => Some([state.clone()].into()),
        OutcomeCondition::StateChange { changes, .. } => {
            let transition = declared_move(outcome, entity)?;
            Some(
                transition
                    .from
                    .iter()
                    .filter(|from| (**from != transition.to) == *changes)
                    .cloned()
                    .collect(),
            )
        }
        OutcomeCondition::When(_)
        | OutcomeCondition::Otherwise
        | OutcomeCondition::SubjectField { .. }
        | OutcomeCondition::SubjectPredicate { .. }
        | OutcomeCondition::External { .. }
        | OutcomeCondition::ExternalWhen { .. }
        | OutcomeCondition::WrongState => None,
    }
}

/// The transition this branch takes, as this entity's lifecycle declares it.
fn declared_move<'a>(
    outcome: &Outcome,
    entity: &'a EntitySpec,
) -> Option<&'a crate::entity::Transition> {
    let Effect::Moves { transition } = &outcome.subject.as_ref()?.effect else {
        return None;
    };
    entity
        .states
        .transitions
        .iter()
        .find(|declared| declared.name == *transition)
}

/// Local declaration checks, also used before a registry or entity map exists.
pub fn validate_shape(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let mut identity = None;
    let mut defaults = 0;
    for outcome in &command.outcomes {
        if matches!(
            outcome.condition,
            OutcomeCondition::External { .. } | OutcomeCondition::ExternalWhen { .. }
        ) {
            continue;
        }
        if outcome.is_unconditional() {
            defaults += 1;
        }
        if outcome.is_wrong_state() {
            errors.push(ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                "explicit subject-state guards cannot also declare wrong_state precedence",
            ));
            continue;
        }
        if outcome.is_unconditional() && outcome.error.is_some() && outcome.subject.is_none() {
            continue;
        }
        let Some(subject) = command
            .selection_subject(outcome)
            .filter(|subject| subject.surface() == InstanceSurface::CommandInput)
        else {
            errors.push(ValidationError::at(command.site().key("outcomes").named(outcome.name.as_str()), ValidationCode::UnobservableFact,
                "every input-selected branch of a subject-state command must name an existing moves or updates subject"));
            continue;
        };
        let current = (&subject.entity, &subject.instance);
        if identity.is_some_and(|previous| previous != current) {
            errors.push(ValidationError::at(command.site().key("outcomes").named(outcome.name.as_str()), ValidationCode::ConflictingDeclaration,
                "subject-state branches must select against the same entity and command-input identity field"));
        } else {
            identity = Some(current);
        }
    }
    if defaults > 1 {
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::ConflictingDeclaration,
            "a subject-state command has at most one genuine default",
        ));
    }
    errors
}

/// Validate subject names and complete finite state/input partitions using actual entities.
pub fn validate(spec: &Specification, types: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for command in spec.commands().values().filter(|command| uses(command)) {
        errors.extend(validate_shape(command));
        let Some(subject) = command
            .outcomes
            .iter()
            .filter(|outcome| {
                !matches!(
                    outcome.condition,
                    OutcomeCondition::External { .. } | OutcomeCondition::ExternalWhen { .. }
                )
            })
            .find_map(|outcome| command.selection_subject(outcome))
        else {
            continue;
        };
        let Some(entity) = spec.entities().get(&subject.entity) else {
            errors.push(ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::UndeclaredReference,
                format!("subject-state entity `{}` is not declared", subject.entity),
            ));
            continue;
        };
        for outcome in &command.outcomes {
            match &outcome.condition {
                OutcomeCondition::SubjectState { state, .. } => {
                    errors.extend(validate_move(command, outcome, entity, state));
                    if !entity.states.states.contains(state) {
                        errors.push(ValidationError::at(
                            command
                                .site()
                                .key("outcomes")
                                .named(outcome.name.as_str())
                                .key("when_subject_state"),
                            ValidationCode::UnknownState,
                            format!("`{state}` is not a declared state of `{}`", entity.name),
                        ));
                    }
                }
                OutcomeCondition::StateChange { changes, .. } => {
                    errors.extend(validate_state_change(command, outcome, entity, *changes));
                }
                OutcomeCondition::When(_)
                | OutcomeCondition::Otherwise
                | OutcomeCondition::SubjectField { .. }
                | OutcomeCondition::SubjectPredicate { .. }
                | OutcomeCondition::External { .. }
                | OutcomeCondition::ExternalWhen { .. }
                | OutcomeCondition::WrongState => {}
            }
        }
        errors.extend(validate_partition(command, entity, types));
    }
    errors
}

fn validate_partition(
    command: &CommandSpec,
    entity: &EntitySpec,
    types: &TypeRegistry,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let guarded: Vec<_> = command
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
    let guards: Vec<_> = guarded
        .iter()
        .map(|outcome| finite::StateGuard {
            states: admitted(outcome, entity),
            predicate: outcome.condition.predicate(),
        })
        .collect();
    let default = command.default_outcome();
    let Some(cases) = finite::analyze_with_states(
        &DomainEnvironment::new(types, &command.input),
        &guards,
        &entity.states.states,
    ) else {
        if default.is_none() || command.has_state_refusal() {
            errors.push(ValidationError::at(command.site().key("outcomes"), ValidationCode::NonExhaustiveBranches,
                    "subject-state/input coverage is open, unsupported, or exceeds 64 joint assignments; declare a genuine default"));
        } else {
            // A default closes input coverage, but an unavailable partition cannot
            // prove which held states an ordinary input guard or default excludes.
            // State-qualified moves were checked against their explicit state above;
            // externally selected outcomes keep their separate authority.
            for outcome in command.outcomes.iter().filter(|outcome| {
                matches!(
                    outcome.condition,
                    OutcomeCondition::When(_)
                        | OutcomeCondition::Otherwise
                        | OutcomeCondition::SubjectField { .. }
                        | OutcomeCondition::SubjectPredicate { .. }
                )
            }) {
                for state in &entity.states.states {
                    errors.extend(validate_move(command, outcome, entity, state));
                }
            }
        }
        return errors;
    };
    for case in cases {
        let selected: Vec<_> = if case.input.selected.is_empty() {
            default.into_iter().collect()
        } else {
            case.input
                .selected
                .iter()
                .map(|index| guarded[*index])
                .collect()
        };
        let assignment = case
            .input
            .values
            .iter()
            .map(|(path, value)| format!("{path} = {value}"))
            .collect::<Vec<_>>()
            .join(", ");
        if selected.len() != 1 {
            errors.push(ValidationError::at(
                command.site().key("outcomes"),
                if selected.is_empty() {
                    ValidationCode::NonExhaustiveBranches
                } else {
                    ValidationCode::ConflictingDeclaration
                },
                format!(
                    "held state {} and input [{assignment}] select {} branches: {}",
                    case.state,
                    selected.len(),
                    selected
                        .iter()
                        .map(|outcome| outcome.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
            continue;
        }
        errors.extend(validate_move(command, selected[0], entity, &case.state));
    }
    errors
}

/// A `when_state_changes:` answer that no held state satisfies is an unreachable branch.
///
/// The condition picks a side of its own transition's `from` set, so an empty side is a statement
/// about the lifecycle rather than about the input: a move that starts only where it arrives never
/// changes the state, and a move that does not start where it arrives never restates one. Both are
/// worth refusing by name, because the repair is different from the one an author reaches for —
/// the branch that answers a restatement is usually the default beside this one, not this move.
fn validate_state_change(
    command: &CommandSpec,
    outcome: &Outcome,
    entity: &EntitySpec,
    changes: bool,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let Some(transition) = declared_move(outcome, entity) else {
        return errors;
    };
    if admitted(outcome, entity).is_some_and(|states| !states.is_empty()) {
        return errors;
    }
    let (because, hint) = if changes {
        (
            format!(
                "`{}` starts only in `{}`, the state it arrives at, so it never changes one",
                transition.name, transition.to
            ),
            "drop the key: every push this branch answers restates the state already held",
        )
    } else {
        (
            format!(
                "`{}` does not start in `{}`, the state it arrives at, so it never restates one",
                transition.name, transition.to
            ),
            "a push that restates the state already held is answered by the default branch \
             beside this one rather than by this move",
        )
    };
    errors.push(
        ValidationError::at(
            command
                .site()
                .key("outcomes")
                .named(outcome.name.as_str())
                .key("when_state_changes"),
            ValidationCode::UnreachableBranch,
            format!(
                "`when_state_changes: {changes}` admits no held state of `{}`: {because}",
                entity.name
            ),
        )
        .with_hint(hint),
    );
    errors
}

fn validate_move(
    command: &CommandSpec,
    outcome: &Outcome,
    entity: &EntitySpec,
    state: &StateName,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(subject) = &outcome.subject {
        if let Effect::Moves { transition } = &subject.effect {
            if entity
                .states
                .transitions
                .iter()
                .find(|declared| declared.name == *transition)
                .is_some_and(|transition| !transition.from.contains(state))
            {
                errors.push(ValidationError::at(command.site().key("outcomes").named(outcome.name.as_str()).key("when_subject_state"), ValidationCode::ConflictingDeclaration,
                    format!("held-state guard `{state}` cannot take move `{transition}`, which does not start there")));
            }
        }
    }
    errors
}
