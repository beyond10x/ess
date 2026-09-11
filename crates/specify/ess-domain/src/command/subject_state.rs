//! Explicit held-state authority and the shared bounded state/input partition proof.
use super::{finite, CommandSpec, Effect, InstanceSurface, Outcome, OutcomeCondition};
use crate::{
    entity::{EntitySpec, StateName},
    expression::DomainEnvironment,
    spec::Specification,
    types::TypeRegistry,
};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

/// Whether this command needs a held-state selection authority.
pub fn uses(command: &CommandSpec) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::SubjectState { .. }))
}

/// Local declaration checks, also used before a registry or entity map exists.
pub fn validate_shape(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let mut identity = None;
    let mut defaults = 0;
    for outcome in &command.outcomes {
        if matches!(outcome.condition, OutcomeCondition::External { .. }) {
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
        let Some(subject) = outcome
            .subject
            .as_ref()
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
            .filter(|outcome| !matches!(outcome.condition, OutcomeCondition::External { .. }))
            .find_map(|outcome| outcome.subject.as_ref())
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
            let OutcomeCondition::SubjectState { state, .. } = &outcome.condition else {
                continue;
            };
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
                    OutcomeCondition::External { .. } | OutcomeCondition::WrongState
                )
        })
        .collect();
    let guards: Vec<_> = guarded
        .iter()
        .map(|outcome| finite::StateGuard {
            state: match &outcome.condition {
                OutcomeCondition::SubjectState { state, .. } => Some(state),
                _ => None,
            },
            predicate: outcome.condition.predicate(),
        })
        .collect();
    let default = command.default_outcome();
    let Some(cases) = finite::analyze_with_states(
        &DomainEnvironment::new(types, &command.input),
        &guards,
        &entity.states.states,
    ) else {
        if default.is_none() {
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
                    OutcomeCondition::When(_) | OutcomeCondition::Otherwise
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
