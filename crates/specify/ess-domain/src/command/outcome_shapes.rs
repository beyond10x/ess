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
    if creates_unknown(outcome) {
        errors.extend(validate_creating_unknown(outcome, at));
    } else if outcome.condition == OutcomeCondition::UnknownInstance
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
    errors.extend(validate_existing(outcome, at));
    errors
}

// ---- selection by existence (ess/16, beyond10x/ess#164) ----------------------------------------

/// `true` for the create-or-update form's creating half: a `creates:` branch marked
/// `unknown_instance: true`, taken when no record carries the identity the input names.
pub(crate) fn creates_unknown(outcome: &Outcome) -> bool {
    outcome.condition == OutcomeCondition::UnknownInstance
        && outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.effect == Effect::Creates)
}

/// The input field a creating branch takes its new identity from: the `input.` source its payload
/// declares for the event field `instance:` names. `None` for a branch that creates nothing, and
/// for an identity the implementation generates — one no caller could name a second time.
pub(crate) fn created_identity(outcome: &Outcome) -> Option<&str> {
    let subject = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == Effect::Creates)?;
    outcome
        .payload
        .values()
        .filter_map(|fields| fields.get(&subject.instance))
        .find_map(|source| match source {
            super::PayloadSource::InputField { field } => Some(field.as_str()),
            _ => None,
        })
}

/// [`created_identity`], or the optional input `{input: f, else: {generated: true}}` reads: a caller
/// that sends `f` names the identity, and can name it twice (the #164 follow-up's optional id). What
/// `existing_instance:` needs; create-or-update keeps the stricter rule, because its update reads
/// the same field as a required `instance:`.
pub(crate) fn supplied_identity(outcome: &Outcome) -> Option<&str> {
    created_identity(outcome).or_else(|| {
        let subject = outcome
            .subject
            .as_ref()
            .filter(|subject| subject.effect == Effect::Creates)?;
        outcome
            .payload
            .values()
            .filter_map(|fields| fields.get(&subject.instance))
            .find_map(|source| match source {
                super::PayloadSource::InputOrGenerated {
                    field,
                    otherwise: None,
                } => Some(field.as_str()),
                _ => None,
            })
    })
}

/// The creating half of create-or-update, alone: it creates, and reports nothing. The branch is an
/// acceptance, so an `error:` or `refuses: false` would say a second thing about it.
fn validate_creating_unknown(outcome: &Outcome, at: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if outcome.replays.is_some() || outcome.error.is_some() || !outcome.refuses {
        errors.push(
            ValidationError::at(
                at.clone().key("unknown_instance"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{}` is `unknown_instance` and creates the record; a creation taken \
                     for an identity no record carries reports no error, declares no `refuses:` \
                     and replays nothing",
                    outcome.name
                ),
            )
            .with_hint(
                "drop `error:`/`refuses:`/`replays:`; a refusal for an unknown identity is a \
                 separate `unknown_instance:` branch on a command that does not create",
            ),
        );
    }
    errors
}

/// The refusal half of create-or-refuse, alone: it names an error and changes nothing.
fn validate_existing(outcome: &Outcome, at: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if outcome.condition != OutcomeCondition::ExistingInstance {
        return errors;
    }
    if outcome.subject.is_some()
        || !outcome.emits.is_empty()
        || !outcome.sets.is_empty()
        || !outcome.payload.is_empty()
        || outcome.replays.is_some()
        || outcome.accepts_nothing
    {
        errors.push(
            ValidationError::at(
                at.clone().key("existing_instance"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{}` is `existing_instance` and declares an effect; the branch taken \
                     for an identity a record already carries is refused and changes nothing",
                    outcome.name
                ),
            )
            .with_hint(
                "keep the error it reports, and nothing else; to update the existing record \
                 instead, mark the creating branch `unknown_instance: true` beside an `updates:`",
            ),
        );
    }
    if outcome.error.is_none() {
        errors.push(
            ValidationError::at(
                at.clone().key("error"),
                ValidationCode::MissingDeclaration,
                format!(
                    "outcome `{}` is the branch taken for an identity a record already carries, \
                     and names no error; the error is the only thing this branch can say",
                    outcome.name
                ),
            )
            .with_hint("give it `error:`, naming what the command reports for a duplicate"),
        );
    }
    errors
}

/// `existing_instance:` composes with no other condition and says nothing but that the record
/// exists.
pub(super) fn existing_alone(raw: &super::RawOutcome) -> Result<(), ValidationErrors> {
    if raw.when.is_some()
        || raw.when_subject.is_some()
        || raw.when_subject_state.is_some()
        || raw.when_state_changes.is_some()
        || raw.external.is_some()
        || raw.wrong_state
        || raw.unknown_instance
        || raw.input_absent
    {
        return Err(super::outcome_conflict(
            &raw.name,
            "existing_instance",
            format!(
                "outcome `{}` declares `existing_instance` beside another condition; the branch \
                 an identity a record already carries takes is decided by nothing else",
                raw.name
            ),
            "keep `existing_instance: true` alone, naming its `error:`",
        ));
    }
    Ok(())
}

/// The command-level rules of both forms: a creating `unknown_instance:` pairs with a branch
/// acting on the record its own identity names; [`validate_existing_pair`] checks the other form.
fn validate_existence_pair(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = |outcome: &Outcome| command.site().key("outcomes").named(outcome.name.as_str());
    for creating in command
        .outcomes
        .iter()
        .filter(|outcome| creates_unknown(outcome))
    {
        if !command.outcomes.iter().any(names_existing) {
            // Reported by `validate_command` as an unreachable branch.
            continue;
        }
        let Some(subject) = &creating.subject else {
            continue;
        };
        let Some(identity) = created_identity(creating) else {
            errors.push(
                ValidationError::at(
                    at(creating).key("unknown_instance"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` creates `{}` for an identity no record carries, and its \
                         payload does not take that identity from the input; an identity the \
                         implementation generates is never one a caller names again, so whether \
                         a record exists cannot select this branch",
                        creating.name, subject.entity
                    ),
                )
                .with_hint(format!(
                    "publish `{}` from the input field the updating branch reads as `instance:`",
                    subject.instance
                )),
            );
            continue;
        };
        // The update half is a `moves:` or an `updates:`; a `deletes:` sibling would make the pair a
        // create-or-delete toggle, which is not the construct.
        let paired = command.outcomes.iter().any(|sibling| {
            names_existing(sibling)
                && sibling.subject.as_ref().is_some_and(|acting| {
                    matches!(acting.effect, Effect::Moves { .. } | Effect::Updates)
                        && acting.entity == subject.entity
                        && acting.instance == identity
                })
        });
        if !paired {
            errors.push(
                ValidationError::at(
                    at(creating).key("unknown_instance"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` creates `{}` from input `{identity}` when no record carries \
                         it, and no branch of `{}` acts on the `{}` that input names, so nothing \
                         answers the call when one does",
                        creating.name, subject.entity, command.name, subject.entity
                    ),
                )
                .with_hint(format!(
                    "declare the `updates:` or `moves:` branch on `{}` with `instance: {identity}`",
                    subject.entity
                )),
            );
        }
    }
    errors.extend(validate_existing_pair(command));
    errors
}

/// An `existing_instance:` pairs with a creation whose identity the caller supplies, and with
/// nothing that acts on an existing record.
fn validate_existing_pair(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = |outcome: &Outcome| command.site().key("outcomes").named(outcome.name.as_str());
    let existing: Vec<&Outcome> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.condition == OutcomeCondition::ExistingInstance)
        .collect();
    if existing.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all `existing_instance`, so `{}` declares more than one \
                     answer for an identity a record already carries",
                    super::join(existing.iter().map(|outcome| &outcome.name)),
                    command.name
                ),
            )
            .with_hint("keep one"),
        );
    }
    if let Some(first) = existing.first() {
        if let Some(acting) = command
            .outcomes
            .iter()
            .find(|outcome| names_existing(outcome) || outcome.is_wrong_state())
        {
            errors.push(
                ValidationError::at(
                    at(first).key("existing_instance"),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "outcome `{}` refuses an identity a record already carries, and `{}` \
                         answers the existing record too, so two branches claim one call",
                        first.name, acting.name
                    ),
                )
                .with_hint(
                    "keep `existing_instance:` on a command that only creates; to update the \
                     existing record, mark the creation `unknown_instance: true` instead",
                ),
            );
        } else if !command
            .outcomes
            .iter()
            .any(|outcome| supplied_identity(outcome).is_some())
        {
            errors.push(
                ValidationError::at(
                    at(first).key("existing_instance"),
                    ValidationCode::UnreachableBranch,
                    format!(
                        "outcome `{}` answers an identity a record already carries, and no branch \
                         of `{}` creates a record from an identity its input supplies, so no call \
                         can name an existing one",
                        first.name, command.name
                    ),
                )
                .with_hint(
                    "declare it beside a `creates:` whose payload publishes `instance:` from an \
                     `input.` field",
                ),
            );
        }
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
    errors.extend(validate_existence_pair(command));
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
    let before_16 = format.major() < FormatVersion::V16.major();
    let gate_16 = |at: ConstructRef, construct: &str| {
        ValidationError::at(
            at,
            ValidationCode::UnsupportedFormatVersion,
            format!("{construct} requires specification format ess/16"),
        )
        .with_hint("write `format: ess/16` on the source that declares the system")
    };
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let at = command.site().key("outcomes").named(outcome.name.as_str());
            if before_16 && creates_unknown(outcome) {
                errors.push(gate_16(
                    at.clone().key("unknown_instance"),
                    "`unknown_instance:` on a `creates:` branch",
                ));
            }
            if before_16 && outcome.condition == OutcomeCondition::ExistingInstance {
                errors.push(gate_16(
                    at.clone().key("existing_instance"),
                    "`existing_instance:`",
                ));
            }
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
    // The identity write of an `updates:` (ess/23), checked beside the pair it may not join.
    errors.extend(super::identity_write::validate(spec));
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
        let branch = precondition_branch(&spec.system().types, command, &precondition.input);
        if let Ok(selected) = &branch {
            if let Some(refusal) = precondition_row(spec, command, selected, &precondition.input) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("system.preconditions[{index}].input"),
                        format!(
                            "precondition {index} invokes `{}`, and {refusal}",
                            command.name
                        ),
                    )
                    .with_hint(
                        "give it an input the row it creates can hold: a precondition must \
                         succeed, and no correct implementation stores a row its invariants \
                         forbid",
                    ),
                );
            }
        }
        if let Err((code, reason)) = branch {
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

/// Why the row a precondition's selected branch creates breaks its entity's invariants, if it does
/// (beyond10x/ess#205).
///
/// The row holds what the precondition writes: its identity from the `instance:` input, each field
/// the branch `sets:` from an input the precondition supplies as a literal, and the state it lands
/// in. A field set from anywhere else — generated, a fixture, a literal in the outcome — is not
/// known here, so an invariant reading one is left to the run; one these facts make false is
/// refused.
fn precondition_row(
    spec: &Specification,
    command: &CommandSpec,
    selected: &super::Outcome,
    input: &BTreeMap<String, Node>,
) -> Option<String> {
    let subject = selected.subject.as_ref()?;
    if subject.effect != super::Effect::Creates {
        return None;
    }
    let entity = spec.entities().get(&subject.entity)?;
    let types = &spec.system().types;
    let row = RowSources {
        types,
        command,
        input,
    };
    let mut values: Vec<(&str, &TypeRef, Node)> = Vec::new();
    if let Sent::Literal(identity) = row.sent(&subject.instance) {
        values.push((
            entity.identity.name.as_str(),
            &entity.identity.type_ref,
            identity.clone(),
        ));
    }
    for (target, source) in &selected.sets {
        let Some(declared) = entity
            .fields
            .iter()
            .find(|candidate| &candidate.name == target)
        else {
            continue;
        };
        if let Some(value) = row.value(&declared.type_ref, source, 0) {
            values.push((target.as_str(), &declared.type_ref, value));
        }
    }
    let members: Vec<(&str, &TypeRef, &Node)> = values
        .iter()
        .map(|(name, declared, value)| (*name, *declared, value))
        .collect();
    let mut facts = super::LiteralFacts::of(types, &members);
    let state = subject.into.as_ref().unwrap_or(&entity.states.initial);
    if let Ok(path) = ess_primitives::facts::FactPath::new("state") {
        facts.set(path, ess_primitives::facts::FactValue::text(state.as_str()));
    }
    entity.invariants.iter().find_map(|invariant| {
        (facts.truth(invariant) == ess_primitives::predicate::Truth::False).then(|| {
            format!(
                "the `{}` it creates breaks the invariant `{invariant}` of `{}`",
                subject.entity, entity.name
            )
        })
    })
}

/// What a precondition sends for one input.
enum Sent<'v> {
    /// A literal value.
    Literal(&'v Node),
    /// Nothing, or `null`: an optional input's fallback applies.
    Omitted,
    /// A fixture, whose value is not known here.
    Fixture,
}

/// The values a precondition's created row holds, read from its `sets:` sources and its literal
/// input: what the interpreter stores, where the document alone says what that is.
struct RowSources<'s> {
    types: &'s crate::types::TypeRegistry,
    command: &'s CommandSpec,
    input: &'s BTreeMap<String, Node>,
}

impl RowSources<'_> {
    /// What the precondition sends for `path`, an input field or a dotted path into one.
    fn sent(&self, path: &str) -> Sent<'_> {
        let mut segments = path.split('.');
        let Some(field) = segments.next() else {
            return Sent::Omitted;
        };
        let Some(mut value) = self.input.get(field) else {
            return Sent::Omitted;
        };
        if fixture_of(self.command, field, value).is_some() {
            return Sent::Fixture;
        }
        for segment in segments {
            match value {
                Node::Map(entries) => match entries.get(segment) {
                    Some(inner) => value = inner,
                    None => return Sent::Omitted,
                },
                _ => return Sent::Omitted,
            }
        }
        if matches!(value, Node::Null) {
            Sent::Omitted
        } else {
            Sent::Literal(value)
        }
    }

    /// The value `source` writes into a member of type `target`, or `None` where it is decided
    /// outside the document: generated, a fixture, the caller, another record.
    fn value(&self, target: &TypeRef, source: &super::PayloadSource, depth: usize) -> Option<Node> {
        use super::PayloadSource;
        if depth > crate::types::MAX_TYPE_DEPTH {
            return None;
        }
        match source {
            PayloadSource::InputField { field } => match self.sent(field) {
                Sent::Literal(value) => Some(value.clone()),
                Sent::Omitted | Sent::Fixture => None,
            },
            PayloadSource::Literal { value } | PayloadSource::Scalar { value, .. } => {
                literal_node(self.types, target, value)
            }
            PayloadSource::InputOrGenerated { field, otherwise } => match self.sent(field) {
                Sent::Literal(value) => Some(value.clone()),
                Sent::Fixture => None,
                Sent::Omitted => otherwise
                    .as_deref()
                    .and_then(|fallback| self.value(target, fallback, depth + 1)),
            },
            PayloadSource::Struct { fields } => {
                let layers = self.types.newtype_layers(target);
                let TypeRef::Named(name) = &layers.terminal else {
                    return None;
                };
                let Some(crate::types::TypeBody::Struct {
                    fields: declared, ..
                }) = self.types.get(name).map(|named| &named.body)
                else {
                    return None;
                };
                let mut members = BTreeMap::new();
                for member in fields {
                    let Some(typed) = declared.iter().find(|field| field.name == member.target)
                    else {
                        continue;
                    };
                    if let Some(value) = self.value(&typed.type_ref, &member.source, depth + 1) {
                        members.insert(member.target.clone(), value);
                    }
                }
                Some(Node::Map(members))
            }
            _ => None,
        }
    }
}

/// The value a literal's text is when read as `target`: the spelling the conformance reader
/// (`ess-conformance` `input.rs`, `primitive_literal`) gives it, or `None` where it has none.
fn literal_node(types: &crate::types::TypeRegistry, target: &TypeRef, text: &str) -> Option<Node> {
    use crate::types::Primitive;
    match &types.newtype_layers(target).terminal {
        TypeRef::Primitive(primitive) => {
            let value = match primitive {
                Primitive::Boolean => match text {
                    "true" => Node::Bool(true),
                    "false" => Node::Bool(false),
                    _ => return None,
                },
                Primitive::Integer => {
                    let number = text.parse::<i64>().ok()?;
                    if number.to_string() != text {
                        return None;
                    }
                    Node::Number(number.into())
                }
                Primitive::Decimal => {
                    Node::Number(ess_primitives::facts::Number::decimal_literal(text)?)
                }
                Primitive::Binary64 | Primitive::Json => return None,
                Primitive::String
                | Primitive::Timestamp
                | Primitive::Duration
                | Primitive::Uuid
                | Primitive::Bytes => Node::Text(text.to_owned()),
            };
            primitive.admits(&value).map(|_| value)
        }
        TypeRef::Named(name) => match types.get(name).map(|named| &named.body) {
            Some(crate::types::TypeBody::Enum { .. }) => Some(Node::Text(text.to_owned())),
            _ => None,
        },
        _ => None,
    }
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
        let undeclared_fixture = |fixture: &str| {
            ValidationError::new(
                ValidationCode::UndeclaredReference,
                format!("{at}.input.{field}"),
                format!(
                    "precondition {index} reads `{field}` from fixture `{fixture}`, and `{}` \
                     declares no such fixture input for it",
                    command.name
                ),
            )
        };
        if let Some(fixture) = fixture_of(command, field, value) {
            if command
                .fixture_inputs
                .get(field)
                .map(super::fixture_inputs::FixtureName::as_str)
                != Some(fixture)
            {
                errors.push(undeclared_fixture(fixture));
            }
            continue;
        }
        if let Err((code, message, hint)) =
            super::precondition_literal_admitted(&spec.system().types, &input.type_ref, value)
        {
            // Written as a fixture reference for an input with no fixture, and no value of its
            // type either: the reference is what was meant, so its refusal is the one reported.
            if let Some(fixture) = written_fixture(value) {
                errors.push(undeclared_fixture(fixture));
                continue;
            }
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
            || spec.system().types.newtype_layers(&field.type_ref).optional;
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
    types: &crate::types::TypeRegistry,
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
    if command.outcomes.iter().any(|outcome| {
        creates_unknown(outcome) || outcome.condition == OutcomeCondition::ExistingInstance
    }) {
        return Err((
            ValidationCode::UnobservableFact,
            "its branches are selected by whether a record carries the identity it names, which \
             a precondition cannot observe before it runs"
                .to_owned(),
        ));
    }
    // Every literal input as the interpreter reads it (beyond10x/ess#205): scalar leaves, a list's
    // `.count` and elements, a struct's members, and presence for `defined()`. An input left to a
    // fixture binds nothing, so a guard reading it is undecided.
    let members: Vec<(&str, &TypeRef, &Node)> = input
        .iter()
        .filter(|(field, value)| fixture_of(command, field, value).is_none())
        .filter_map(|(field, value)| {
            command
                .input_field(field)
                .map(|declared| (field.as_str(), &declared.type_ref, value))
        })
        .collect();
    let facts = super::LiteralFacts::of(types, &members);
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

/// The fixture a precondition's input `field` reads: written `{fixture: name}` for an input its
/// command declares a fixture input for.
///
/// Anywhere else the same map is a literal of the input's type (beyond10x/ess#205): a struct with a
/// field named `fixture`, or a map with that key.
pub fn fixture_of<'v>(command: &CommandSpec, field: &str, value: &'v Node) -> Option<&'v str> {
    written_fixture(value).filter(|_| command.fixture_inputs.contains_key(field))
}

/// A value written `{fixture: name}`, whichever input it sits on.
fn written_fixture(value: &Node) -> Option<&str> {
    let Node::Map(entries) = value else {
        return None;
    };
    match entries.iter().collect::<Vec<_>>().as_slice() {
        [(key, Node::Text(name))] if key.as_str() == "fixture" => Some(name.as_str()),
        _ => None,
    }
}
