//! An outcome for a request whose input is absent as a whole: `input_absent: true` (ess/16,
//! beyond10x/ess#170, `docs/design/outcome-shapes.md`).
//!
//! A marker beside `wrong_state:` and `unknown_instance:`. It is taken when the command arrives
//! with no input document at all — an absent request body, which an implementation may answer
//! differently from `{}` and from a body that lacks a field — and it names the error it reports.
//! The command's fields keep their types, so declaring it changes no other branch's contract.
//!
//! Under `ess/16` the same module refuses the guard authors wrote before the marker existed: one
//! that cannot hold because every way it could hold needs an input that is not `Optional` to be
//! absent (`not defined(f)`, `missing(f)`). Validation admitted such a branch and synthesis could
//! never witness it (the validate/synthesize disagreement #170 reports beside the marker). Below
//! `ess/16` the old behaviour stands.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use std::collections::BTreeSet;

use ess_primitives::facts::FactPath;
use ess_primitives::predicate::Predicate;

use super::{CommandSpec, Outcome, OutcomeCondition, RawOutcome};
use crate::system::FormatVersion;
use crate::{Specification, TypeRef, TypeRegistry};

/// The marker composes with no other condition and says nothing but that the input is absent.
pub(super) fn alone(raw: &RawOutcome) -> Result<(), ValidationErrors> {
    if raw.when.is_some()
        || raw.when_subject.is_some()
        || raw.when_subject_state.is_some()
        || raw.when_state_changes.is_some()
        || raw.external.is_some()
        || raw.wrong_state
        || raw.unknown_instance
    {
        return Err(super::outcome_conflict(
            &raw.name,
            "input_absent",
            format!(
                "outcome `{}` declares `input_absent` beside another condition; a request with no \
                 input has no field, state or identity another condition could read",
                raw.name
            ),
            "keep `input_absent: true` alone, naming its `error:`",
        ));
    }
    Ok(())
}

/// The rules one `input_absent:` branch can check alone: it names an error and does nothing else.
pub(super) fn validate_outcome(outcome: &Outcome, at: &ConstructRef) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if outcome.condition != OutcomeCondition::InputAbsent {
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
                at.clone().key("input_absent"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcome `{}` is `input_absent` and declares an effect; a request with no \
                     input is answered before anything is read or changed",
                    outcome.name
                ),
            )
            .with_hint("keep the error it reports, and nothing else"),
        );
    }
    if outcome.error.is_none() {
        errors.push(
            ValidationError::at(
                at.clone().key("error"),
                ValidationCode::MissingDeclaration,
                format!(
                    "outcome `{}` is the branch taken for a request with no input, and names no \
                     error; the error is the only thing this branch can say",
                    outcome.name
                ),
            )
            .with_hint("give it `error:`, naming what the command reports for a missing body"),
        );
    }
    errors
}

/// At most one `input_absent:` branch per command, and only on a command that takes input.
pub(super) fn validate_command(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let marked: Vec<&Outcome> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.condition == OutcomeCondition::InputAbsent)
        .collect();
    if marked.len() > 1 {
        errors.push(
            ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "outcomes {} are all `input_absent`, so `{}` declares more than one answer \
                     for a request with no input",
                    super::join(marked.iter().map(|outcome| &outcome.name)),
                    command.name
                ),
            )
            .with_hint("keep one"),
        );
    }
    if let Some(first) = marked.first() {
        if command.input.is_empty() {
            errors.push(
                ValidationError::at(
                    command.site().key("outcomes").named(first.name.as_str()),
                    ValidationCode::UnreachableBranch,
                    format!(
                        "outcome `{}` answers a request with no input, and `{}` takes no input, \
                         so every request it receives is one",
                        first.name, command.name
                    ),
                )
                .with_hint("declare input on the command, or drop the branch"),
            );
        }
    }
    errors
}

/// A guard that cannot hold because every way it could hold needs a required input to be absent
/// (ess/16).
///
/// `not defined(f)` or `missing(f)` over an input `f` that is not `Optional` never holds. A guard
/// is refused only when it *as a whole* cannot hold: negation is pushed inward by De Morgan (under
/// `not`, `all` reads as `any` and `any` as `all`), a conjunction cannot hold when one conjunct
/// cannot, and a disjunction cannot hold only when every disjunct cannot. `any: [text == "x",
/// missing(text)]` holds whenever `text == "x"`, so it is admitted with its dead disjunct; `not:
/// {all: [defined(text), text == "x"]}` is `text != "x"` and is admitted too.
///
/// Refused rather than admitted as a branch synthesis cannot witness. The hint names the marker,
/// because the behaviour such a guard usually tries to state — a request that arrived without its
/// body — is what `input_absent:` says.
fn validate_guard(
    types: &TypeRegistry,
    command: &CommandSpec,
    outcome: &Outcome,
    at: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let Some(predicate) = outcome.condition.predicate() else {
        return errors;
    };
    let required = |path: &FactPath| required_path(types, command, path);
    let Some(absent) = cannot_hold(predicate, false, &required) else {
        return errors;
    };
    let fields: BTreeSet<(String, String)> = absent.into_iter().collect();
    let named = super::join(fields.iter().map(|(field, ty)| format!("{field}: {ty}")));
    let first = fields
        .iter()
        .next()
        .map_or_else(String::new, |(field, _)| field.clone());
    errors.push(
        ValidationError::at(
            at.clone().key("when"),
            ValidationCode::TypeMismatch,
            format!(
                "outcome `{}` is guarded on a required input being absent, and `{}` requires {named}, \
                 so the guard never holds and no request can take the branch",
                outcome.name, command.name
            ),
        )
        .with_hint(format!(
            "for a request that arrives with no input at all, write `input_absent: true` with the \
             `error:` it reports (ess/16); for a field a caller may leave out, declare `{first}` \
             `Optional`"
        )),
    );
    errors
}

/// The dotted path and declared type of `path` when every segment of it is required: an input
/// that is not `Optional`, then, through each struct it names (newtypes of a struct included), a
/// field that is not `Optional`. `None` for anything else — an `Optional` segment, a path into a
/// list, map, enum or union, or a name that is not an input.
fn required_path(
    types: &TypeRegistry,
    command: &CommandSpec,
    path: &FactPath,
) -> Option<(String, String)> {
    let (first, rest) = path.segments().split_first()?;
    let mut current = &command.input_field(first)?.type_ref;
    if matches!(current, TypeRef::Optional(_)) {
        return None;
    }
    for segment in rest {
        let field = types
            .struct_fields(current)?
            .iter()
            .find(|field| &field.name == segment)?;
        if matches!(field.type_ref, TypeRef::Optional(_)) {
            return None;
        }
        current = &field.type_ref;
    }
    Some((path.segments().join("."), current.to_string()))
}

/// `Some(fields)` when `predicate`, read under `negated`, cannot hold because every way it could
/// hold needs one of `fields` — required inputs — to be absent; `None` when it may hold.
///
/// Quantifier bodies are not entered, since a bound name could shadow an input, and every other
/// leaf is taken as able to hold.
fn cannot_hold<F>(
    predicate: &Predicate,
    negated: bool,
    required: &F,
) -> Option<Vec<(String, String)>>
where
    F: Fn(&FactPath) -> Option<(String, String)>,
{
    match predicate {
        Predicate::Defined(path) if negated => required(path).map(|field| vec![field]),
        Predicate::Not(inner) => cannot_hold(inner, !negated, required),
        Predicate::All(children) | Predicate::Any(children) => {
            let conjunction = matches!(predicate, Predicate::All(_)) != negated;
            if conjunction {
                children
                    .iter()
                    .find_map(|child| cannot_hold(child, negated, required))
            } else if children.is_empty() {
                None
            } else {
                children.iter().try_fold(Vec::new(), |mut all, child| {
                    all.extend(cannot_hold(child, negated, required)?);
                    Some(all)
                })
            }
        }
        _ => None,
    }
}

/// The format gate — `input_absent:` requires `ess/16` — and, under `ess/16`, the refusal of a
/// guard that cannot hold for want of a required input. Below `ess/16` such a guard keeps the
/// meaning it had: admitted.
pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let current = spec.system().format.major() >= FormatVersion::V16.major();
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let at = command.site().key("outcomes").named(outcome.name.as_str());
            if current {
                errors.extend(validate_guard(&spec.system().types, command, outcome, &at));
            } else if outcome.condition == OutcomeCondition::InputAbsent {
                errors.push(
                    ValidationError::at(
                        at.key("input_absent"),
                        ValidationCode::UnsupportedFormatVersion,
                        "`input_absent:` requires specification format ess/16",
                    )
                    .with_hint("write `format: ess/16` on the source that declares the system"),
                );
            }
        }
    }
    errors
}
