//! Outcomes selected by whether the addressed record exists (ess/16, beyond10x/ess#164,
//! `docs/design/outcome-shapes.md`), refused by every code target without a storage port.
//!
//! Both forms — a creating branch marked `unknown_instance: true` beside the branch that updates
//! the record, and an `existing_instance:` refusal beside a creation — select a branch by a lookup
//! of the store before the input is dispatched. The Rust and Go targets generate that lookup over
//! the storage port their generated behaviour already reads (`rust::behaviour`, beyond10x/ess#310;
//! `go::behaviour`, beyond10x/ess#314), and where the plan keeps such a command an obligation the
//! implementor's behaviour selects it, as it decides every other branch of an owed command. The Web
//! and Clap targets carry no storage
//! port: their seams and explorers select a branch from the decoded input and the model's own state
//! machine, and neither holds the question "does a record carry this identity" as something a
//! branch can be selected by. There each such branch is named rather than emitted with a selection
//! nobody chose.

use ess_compiler::ir::{ResolvedCondition, ResolvedEffect, ResolvedOutcome};

use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::{SynthesisPlan, Target};

/// Why the branch has no representation.
const DETAIL: &str = "this target cannot select a branch by whether a record carries the \
                      addressed identity; the Rust target selects it over its storage port";

/// The key an author wrote to select this branch by existence, or `None` for any other branch.
fn marker(outcome: &ResolvedOutcome) -> Option<&'static str> {
    match outcome.condition {
        ResolvedCondition::ExistingInstance => Some("existing_instance"),
        ResolvedCondition::UnknownInstance
            if outcome
                .subject
                .as_ref()
                .is_some_and(|subject| subject.effect == ResolvedEffect::Creates) =>
        {
            Some("unknown_instance")
        }
        _ => None,
    }
}

/// Every branch selected by existence, named at the key its author wrote, on a target that cannot
/// select it. The Rust target can, and is never refused here.
pub(crate) fn refuse(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    if target == Target::Rust {
        return Ok(());
    }
    let mut causes = Vec::new();
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            let Some(key) = marker(outcome) else {
                continue;
            };
            causes.push(TargetFailureCause::new(
                TargetFailureCode::MissingRepresentation,
                vec![format!(
                    "commands.{}.outcomes.{}.{key}",
                    command.name, outcome.name
                )],
                DETAIL.to_owned(),
            ));
        }
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}
