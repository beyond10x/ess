//! Outcome shapes beyond `ess/14` (`docs/design/outcome-shapes.md`): `unknown_instance:`
//! (beyond10x/ess#145), `deletes:` (#151), `into:` (#150), `accepts: nothing` (#144) and a
//! system's `preconditions:` (#152).
//!
//! Each is admitted under source format `ess/15` and refused below it with
//! `unsupported_format_version`, at the key the author wrote. A document using none of them means
//! what it meant before, and compiles to the same bytes.
//!
//! Three passes, one per question: [`validate_outcome`] and [`validate_command`] are what one
//! command can check alone and run wherever its shape is checked; [`validate`] needs the whole
//! specification — the format, the entity a creation lands in, and the commands and actors a
//! precondition names.

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::node::Node;

use super::{
    CommandSpec, Effect, InstanceSurface, Outcome, OutcomeCondition, OutcomeName, Subject,
};
use crate::system::FormatVersion;
use crate::{Specification, TypeRef};

/// What an `accepts:` key may say. One word, so a later form has somewhere to go.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Accepts {
    /// The request is accepted and nothing observable changes.
    Nothing,
}

/// A `creates:` subject with the state `into:` names, or the refusal of an `into:` beside anything
/// else.
pub(super) fn creation_state(
    name: &OutcomeName,
    subject: Option<Subject>,
    into: Option<crate::entity::StateName>,
) -> Result<Option<Subject>, ValidationErrors> {
    let Some(state) = into else {
        return Ok(subject);
    };
    match subject {
        Some(mut subject) if subject.effect == Effect::Creates => {
            subject.into = Some(state);
            Ok(Some(subject))
        }
        _ => Err(ValidationError::new(
            ValidationCode::ConflictingDeclaration,
            format!("outcomes.{name}.into"),
            format!(
                "outcome `{name}` declares `into: {state}` without `creates:`; only a creation \
                 has a first state to name"
            ),
        )
        .with_hint("drop `into:`, or make this the branch that creates the instance")
        .into()),
    }
}

/// The rules one outcome can check alone, for the three outcome-level constructs.
pub(super) fn validate_outcome(outcome: &Outcome, at: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let conflict = |key: &str, message: String, hint: &str| {
        ValidationError::at(
            at.clone().key(key),
            ValidationCode::ConflictingDeclaration,
            message,
        )
        .with_hint(hint.to_owned())
    };
    if outcome.accepts_nothing {
        let admitted_condition = matches!(
            outcome.condition,
            OutcomeCondition::When(_) | OutcomeCondition::Otherwise
        );
        if outcome.subject.is_some()
            || !outcome.emits.is_empty()
            || outcome.error.is_some()
            || !outcome.sets.is_empty()
            || !outcome.payload.is_empty()
            || outcome.replays.is_some()
            || !admitted_condition
        {
            errors.push(conflict(
                "accepts",
                format!(
                    "outcome `{}` declares `accepts: nothing` beside a subject, an event, an \
                     error, an assignment, a replay or a condition other than `when:`; an \
                     accepted request that changes nothing carries none of them",
                    outcome.name
                ),
                "keep `accepts: nothing` alone, under `when:` or as the default; a command with \
                 a subject says `preserves:`",
            ));
        }
    }
    if outcome
        .subject
        .as_ref()
        .is_some_and(|subject| subject.effect == Effect::Deletes)
        && !outcome.sets.is_empty()
    {
        errors.push(conflict(
            "deletes",
            format!(
                "outcome `{}` deletes its subject and sets its fields; a removed row holds \
                 nothing to set",
                outcome.name
            ),
            "drop `sets:`, or split the branch into an update and a deletion",
        ));
    }
    if outcome.condition == OutcomeCondition::UnknownInstance
        && (outcome.subject.is_some()
            || !outcome.emits.is_empty()
            || !outcome.sets.is_empty()
            || !outcome.payload.is_empty()
            || outcome.replays.is_some())
    {
        errors.push(conflict(
            "unknown_instance",
            format!(
                "outcome `{}` is `unknown_instance` and declares an effect; an identity no record \
                 carries has nothing to act on",
                outcome.name
            ),
            "keep the error it reports, or `refuses: false`, and nothing else",
        ));
    }
    errors
}

/// `true` when `outcome` acts on an existing instance the caller names in its input: the branches
/// an identity no record carries can reach.
pub(crate) fn names_existing(outcome: &Outcome) -> bool {
    outcome.subject.as_ref().is_some_and(|subject| {
        matches!(
            subject.effect,
            Effect::Moves { .. } | Effect::Updates | Effect::Deletes
        ) && subject.surface() == InstanceSurface::CommandInput
    })
}

/// At most one `unknown_instance:` branch per command, and only where an unknown identity can
/// arrive.
pub(super) fn validate_command(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let marked: Vec<&Outcome> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.condition == OutcomeCondition::UnknownInstance)
        .collect();
    if marked.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all `unknown_instance`, so `{}` declares more than one \
                     answer for an identity no record carries",
                    super::join(marked.iter().map(|outcome| &outcome.name)),
                    command.name
                ),
            )
            .with_hint("keep one"),
        );
    }
    if let Some(first) = marked.first() {
        if !command.outcomes.iter().any(names_existing) {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes").named(first.name.as_str()),
                    ValidationCode::UnreachableBranch,
                    format!(
                        "outcome `{}` answers an identity no record carries, and no branch of \
                         `{}` acts on an instance its input names, so no such identity reaches it",
                        first.name, command.name
                    ),
                )
                .with_hint(
                    "declare it on a command whose `moves:`, `updates:` or `deletes:` branch \
                     reads `instance:` from input",
                ),
            );
        }
    }
    errors
}

/// Everything that needs the whole specification: the format gate for all five constructs, the
/// state an `into:` names, and each precondition against the commands and actors it names.
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let format = spec.system().format;
    let old = format.major() < FormatVersion::V15.major();
    let gate = |at: ConstructRef, construct: &str| {
        ValidationError::at(
            at,
            ValidationCode::UnsupportedFormatVersion,
            format!("{construct} requires specification format ess/15"),
        )
        .with_hint("write `format: ess/15` on the source that declares the system")
    };
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let at = command.site().key("outcomes").named(outcome.name.as_str());
            if old && outcome.condition == OutcomeCondition::UnknownInstance {
                errors.push(gate(
                    at.clone().key("unknown_instance"),
                    "`unknown_instance:`",
                ));
            }
            if old && outcome.accepts_nothing {
                errors.push(gate(at.clone().key("accepts"), "`accepts: nothing`"));
            }
            let Some(subject) = &outcome.subject else {
                continue;
            };
            if old && subject.effect == Effect::Deletes {
                errors.push(gate(at.clone().key("deletes"), "`deletes:`"));
            }
            let Some(state) = &subject.into else {
                continue;
            };
            if old {
                errors.push(gate(at.clone().key("into"), "`into:`"));
            }
            if let Some(entity) = spec.entities().get(&subject.entity) {
                if !entity.states.states.contains(state) {
                    errors.push(
                        ValidationError::at(
                            at.key("into"),
                            ValidationCode::UnknownState,
                            format!(
                                "outcome `{}` creates `{}` into `{state}`, which its lifecycle \
                                 does not declare",
                                outcome.name, subject.entity
                            ),
                        )
                        .with_hint(format!(
                            "declared states: {}",
                            super::join(entity.states.states.iter())
                        )),
                    );
                }
            }
        }
    }
    let preconditions = &spec.system().preconditions;
    if old && !preconditions.is_empty() {
        errors.push(
            ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                "system.preconditions",
                "`preconditions:` requires specification format ess/15",
            )
            .with_hint("write `format: ess/15` on the source that declares the system"),
        );
    }
    for (index, precondition) in preconditions.iter().enumerate() {
        errors.extend(validate_precondition(spec, index, precondition));
    }
    errors
}

/// One precondition: a declared command, a declared actor permitted to invoke it, and an input
/// that names only its fields, supplies every required one and types each literal.
fn validate_precondition(
    spec: &Specification,
    index: usize,
    precondition: &crate::system::Precondition,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = format!("system.preconditions[{index}]");
    let Some(command) = spec.commands().get(&precondition.command) else {
        errors.push(
            ValidationError::new(
                ValidationCode::UndeclaredReference,
                format!("{at}.command"),
                format!(
                    "precondition {index} invokes `{}`, which is not a declared command",
                    precondition.command
                ),
            )
            .with_hint("name a command a domain declares"),
        );
        return errors;
    };
    if let Some(actor) = &precondition.actor {
        errors.extend(precondition_actor(spec, index, actor, command));
    }
    let input_errors = precondition_input(spec, index, precondition, command);
    if input_errors.is_empty() {
        if let Err((code, reason)) = precondition_branch(command, &precondition.input) {
            errors.push(
                ValidationError::new(
                    code,
                    format!("system.preconditions[{index}].input"),
                    format!(
                        "precondition {index} invokes `{}`, and {reason}",
                        command.name
                    ),
                )
                .with_hint(
                    "give it an input that selects exactly one branch reporting no error: a \
                     precondition must succeed, or every scenario would run in a world it did not \
                     make",
                ),
            );
        }
    }
    errors.extend(input_errors);
    errors
}

/// The actor a precondition runs as is declared, and may invoke its command.
fn precondition_actor(
    spec: &Specification,
    index: usize,
    actor: &crate::QualifiedName,
    command: &CommandSpec,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = format!("system.preconditions[{index}].as");
    match spec.actors().get(actor) {
        None => errors.push(ValidationError::new(
            ValidationCode::UndeclaredReference,
            at,
            format!("precondition {index} runs as `{actor}`, which is not a declared actor"),
        )),
        Some(declared) if !declared.may.contains(&command.name) => {
            errors.push(ValidationError::new(
                ValidationCode::UndeclaredReference,
                at,
                format!(
                    "precondition {index} runs `{}` as `{actor}`, which `{actor}` may not invoke",
                    command.name
                ),
            ));
        }
        Some(_) => {}
    }
    errors
}

/// A precondition's input names only its command's fields, supplies every required one, and
/// types each literal; a `{fixture: name}` value reads the command's own fixture input.
fn precondition_input(
    spec: &Specification,
    index: usize,
    precondition: &crate::system::Precondition,
    command: &CommandSpec,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = format!("system.preconditions[{index}]");
    let declared: BTreeSet<&str> = command
        .input
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    for (field, value) in &precondition.input {
        let Some(input) = command.input_field(field) else {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("{at}.input.{field}"),
                    format!(
                        "precondition {index} supplies `{field}`, which `{}` does not take",
                        command.name
                    ),
                )
                .with_hint(format!("its input: {}", super::join(declared.iter()))),
            );
            continue;
        };
        if let Some(fixture) = fixture_of(value) {
            if command
                .fixture_inputs
                .get(field)
                .map(super::fixture_inputs::FixtureName::as_str)
                != Some(fixture)
            {
                errors.push(ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("{at}.input.{field}"),
                    format!(
                        "precondition {index} reads `{field}` from fixture `{fixture}`, and `{}` \
                         declares no such fixture input for it",
                        command.name
                    ),
                ));
            }
            continue;
        }
        if let Err((code, message, hint)) =
            super::example_admitted(&spec.system().types, &input.type_ref, value)
        {
            errors.push(
                ValidationError::new(
                    code,
                    format!("{at}.input.{field}"),
                    format!("precondition {index}, input `{field}`: {message}"),
                )
                .with_hint(hint),
            );
        }
    }
    for field in &command.input {
        let supplied = precondition.input.contains_key(&field.name)
            || command.fixture_inputs.contains_key(&field.name)
            || matches!(field.type_ref, TypeRef::Optional(_));
        if !supplied {
            errors.push(
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    format!("{at}.input.{}", field.name),
                    format!(
                        "precondition {index} invokes `{}` without `{}`, which it requires",
                        command.name, field.name
                    ),
                )
                .with_hint("supply it, or declare it a fixture input of the command"),
            );
        }
    }
    errors
}

/// The one branch a precondition's literal input selects, which must report no error.
///
/// Decided here, once, so synthesis and the generated explorers expect the same branch: the
/// `when:` guards are evaluated over the literal input; exactly one holding guard selects its
/// branch, none selects the default. A branch decided outside the input (`external:`), by the held
/// subject, or for an unknown identity is not a precondition's to take, and a guard reading an
/// input the precondition leaves to a fixture is undecided — both are refused rather than guessed.
///
/// # Errors
///
/// The code and the reason, where the input selects no branch, several, or a refusal.
pub fn precondition_branch<'c>(
    command: &'c CommandSpec,
    input: &BTreeMap<String, Node>,
) -> Result<&'c Outcome, (ValidationCode, String)> {
    if command.outcomes.iter().any(|outcome| {
        outcome.condition.reads_held_state() || outcome.condition.reads_subject_fact()
    }) {
        return Err((
            ValidationCode::UnobservableFact,
            "its branches are selected by the existing subject, which a precondition cannot \
             observe before it runs"
                .to_owned(),
        ));
    }
    let mut facts = ess_primitives::facts::FactStore::new();
    for (field, value) in input {
        let Ok(path) = ess_primitives::facts::FactPath::new(field) else {
            continue;
        };
        match value {
            Node::Bool(flag) => facts.set(path, *flag),
            Node::Number(number) => facts.set(path, *number),
            Node::Text(text) => facts.set(path, text.as_str()),
            Node::Null | Node::Seq(_) | Node::Map(_) => {}
        }
    }
    let mut holding = Vec::new();
    for outcome in &command.outcomes {
        let OutcomeCondition::When(predicate) = &outcome.condition else {
            continue;
        };
        match predicate.evaluate(&facts) {
            ess_primitives::predicate::Truth::True => holding.push(outcome),
            ess_primitives::predicate::Truth::False => {}
            ess_primitives::predicate::Truth::Unknown => {
                return Err((
                    ValidationCode::UnobservableFact,
                    format!(
                        "the guard of `{}` is undecided by its literal input",
                        outcome.name
                    ),
                ))
            }
        }
    }
    let selected = match holding.as_slice() {
        [one] => *one,
        [] => command.default_outcome().ok_or_else(|| {
            (
                ValidationCode::NonExhaustiveBranches,
                "its input selects no branch".to_owned(),
            )
        })?,
        several => {
            return Err((
                ValidationCode::ConflictingDeclaration,
                format!(
                    "its input selects {} at once",
                    super::join(several.iter().map(|outcome| &outcome.name))
                ),
            ))
        }
    };
    if let Some(error) = &selected.error {
        return Err((
            ValidationCode::ConflictingDeclaration,
            format!(
                "its input selects `{}`, which refuses with `{error}`",
                selected.name
            ),
        ));
    }
    Ok(selected)
}

/// The fixture a precondition input reads, when it is written `{fixture: name}`.
pub fn fixture_of(value: &Node) -> Option<&str> {
    let Node::Map(entries) = value else {
        return None;
    };
    match entries.iter().collect::<Vec<_>>().as_slice() {
        [(key, Node::Text(name))] if key.as_str() == "fixture" => Some(name.as_str()),
        _ => None,
    }
}
