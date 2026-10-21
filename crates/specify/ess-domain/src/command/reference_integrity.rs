//! A stored reference whose row cannot be missing needs no `exists: false` branch (`ess/29`,
//! beyond10x/ess#509, `docs/design/generic-language-features.md` § G2, "Enforced references").
//!
//! A command reading the row a stored field of its addressed subject names, through
//! `when_related: {via: <field>, predicate: …}`, is refused as `non_exhaustive_branches` when no
//! branch answers a missing row ([`super::related_guard`]). From `ess/29` the refusal is lifted
//! where the specification itself proves that row present whenever the reference is, which takes
//! two facts, both read off the commands:
//!
//! 1. **Every write of the field checks the row.** Each branch that creates a row of the subject's
//!    entity sets the field, and each branch that writes it — creating, updating or moving the
//!    addressed row, or every row an `instances:` or `affects:` selection names — writes either
//!    nothing (`{cleared: true}`, the field's own value) or an input its command refuses first, with
//!    an `exists: false` branch over that input naming the same entity and carrying no `when:`.
//! 2. **No branch removes a row of the referenced entity.** No `deletes:` subject, no
//!    `instances: {deletes}` and no `affects:` entry with `deletes: true` names it. A binding runs a
//!    command, so the command is what is named.
//!
//! The proof is derived, not declared: no key is authored, and a specification that later gains a
//! deleting command or an unchecked writer is refused at the reading command, the refusal naming
//! each branch that defeats the proof. Below `ess/29` the refusal stands, its hint naming
//! `ess/29` where the proof holds.
use super::{related_guard, related_value, CommandSpec, Effect, OutcomeCondition, PayloadSource};
use super::{RelatedTest, RelatedVia};
use crate::entity::EntitySpec;
use crate::spec::Specification;
use crate::system::FormatVersion;
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// `errors`, the command's own refusals, with its missing-row refusal lifted where the row its
/// stored reference names cannot be missing (`ess/29`), or its hint naming why it can.
pub(crate) fn lift(
    spec: &Specification,
    command: &CommandSpec,
    errors: ValidationErrors,
) -> ValidationErrors {
    let message = related_guard::missing_row_message(command);
    let refused = |error: &ess_primitives::error::ValidationError| {
        error.code == ValidationCode::NonExhaustiveBranches && error.message == message
    };
    if !errors.as_slice().iter().any(refused) {
        return errors;
    }
    let Some(proof) = proof(spec, command) else {
        return errors;
    };
    let admitted = spec.system().format.major() >= FormatVersion::V29.major();
    let mut out = ValidationErrors::new();
    for mut error in errors {
        if refused(&error) {
            match (&proof, admitted) {
                (Ok(()), true) => continue,
                (Ok(()), false) => {
                    let hint = error.hint.take().unwrap_or_default();
                    error.hint = Some(format!(
                        "{hint}, or declare `format: ess/29`, which needs none here: every \
                         write of the reference checks the row, and no command deletes one"
                    ));
                }
                (Err(defeats), true) => {
                    let hint = error.hint.take().unwrap_or_default();
                    error.hint = Some(format!(
                        "{hint}; the row can be missing because {}",
                        defeats.join(", and ")
                    ));
                }
                (Err(_), false) => {}
            }
        }
        out.push(error);
    }
    out
}

/// Whether the one row `command` reads through a stored field of its addressed subject is present
/// whenever the field is: `Ok` where it is, `Err` with each branch that defeats it, in declaration
/// order; `None` where the command reads no single stored reference to one entity.
fn proof(spec: &Specification, command: &CommandSpec) -> Option<Result<(), Vec<String>>> {
    if related_guard::reads_several_rows(command) {
        return None;
    }
    let RelatedVia::Subject(field) = related_guard::read_via(command)? else {
        return None;
    };
    let subject = related_guard::addressed_subject(spec, command)?;
    let related_value::Referenced::Entity(target) =
        related_guard::related_entity_via(spec, command, &RelatedVia::Subject(field.clone()))
    else {
        return None;
    };
    let mut defeats = Vec::new();
    for other in spec.commands().values() {
        defeats.extend(writes(spec, other, subject, field, target));
        defeats.extend(deletes(other, target));
    }
    Some(if defeats.is_empty() {
        Ok(())
    } else {
        Err(defeats)
    })
}

/// Each branch of `command` that leaves `field` of a `subject` row naming an identity it did not
/// check a `target` row carries.
fn writes(
    spec: &Specification,
    command: &CommandSpec,
    subject: &EntitySpec,
    field: &str,
    target: &EntitySpec,
) -> Vec<String> {
    let mut defeats = Vec::new();
    for outcome in &command.outcomes {
        let branch = format!("`{}/{}`", command.name, outcome.name);
        let own = outcome
            .subject
            .as_ref()
            .filter(|named| named.entity == subject.name);
        let selected = outcome
            .set_effects
            .instances
            .as_ref()
            .filter(|named| named.entity == subject.name);
        if let Some(own) = own {
            if own.effect == Effect::Creates && !outcome.sets.contains_key(field) {
                defeats.push(format!(
                    "{branch} creates a `{}` row without setting `{field}`",
                    subject.name
                ));
                continue;
            }
        }
        if own.is_some() || selected.is_some() {
            if let Some(source) = outcome.sets.get(field) {
                if !checked(spec, command, outcome, source, (field, target), true) {
                    defeats.push(unchecked(&branch, field, target));
                    continue;
                }
            }
        }
        for affect in &outcome.set_effects.affects {
            if affect.entity != subject.name {
                continue;
            }
            let read = affect
                .each
                .as_ref()
                .is_some_and(|each| each.reads.contains_key(field));
            let written = affect.sets.get(field).is_some_and(|source| {
                !checked(spec, command, outcome, source, (field, target), false)
            });
            if read || written {
                defeats.push(unchecked(&branch, field, target));
                break;
            }
        }
    }
    defeats
}

fn unchecked(branch: &str, field: &str, target: &EntitySpec) -> String {
    format!(
        "{branch} writes `{field}` without first refusing an identity no `{}` row carries",
        target.name
    )
}

/// Whether `source`, written to `field` by `outcome` of `command`, names a `target` row whenever it
/// names anything: nothing, the field's own value (`own` only, where the row written is the
/// addressed one), or an input the command refuses first where no `target` row carries it.
fn checked(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &super::Outcome,
    source: &PayloadSource,
    (field, target): (&str, &EntitySpec),
    own: bool,
) -> bool {
    match source {
        PayloadSource::Cleared => true,
        PayloadSource::SubjectField { field: read } => own && read == field,
        PayloadSource::InputField { field: input } => {
            !input.contains('.')
                && !refuses_missing(outcome, input)
                && command
                    .outcomes
                    .iter()
                    .any(|other| refuses_missing(other, input))
                && matches!(
                    related_guard::related_entity(spec, command, input),
                    related_value::Referenced::Entity(named) if named.name == target.name
                )
        }
        _ => false,
    }
}

/// Whether `outcome` is the `exists: false` branch over the row `input` names, alone and with no
/// input guard, so a missing row takes it whatever else the request carries.
fn refuses_missing(outcome: &super::Outcome, input: &str) -> bool {
    matches!(
        &outcome.condition,
        OutcomeCondition::Related {
            via: RelatedVia::Input(read),
            test: RelatedTest::Absent,
            input: None,
            also,
        } if read == input && also.is_empty()
    )
}

/// Each branch of `command` that removes a `target` row.
fn deletes(command: &CommandSpec, target: &EntitySpec) -> Vec<String> {
    command
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome
                .subject
                .as_ref()
                .is_some_and(|named| named.entity == target.name && named.effect == Effect::Deletes)
                || outcome.set_effects.instances.as_ref().is_some_and(|named| {
                    named.entity == target.name && named.effect == Effect::Deletes
                })
                || outcome
                    .set_effects
                    .affects
                    .iter()
                    .any(|affect| affect.entity == target.name && affect.deletes)
        })
        .map(|outcome| {
            format!(
                "`{}/{}` deletes `{}` rows",
                command.name, outcome.name, target.name
            )
        })
        .collect()
}
