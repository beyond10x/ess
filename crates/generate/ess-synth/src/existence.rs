//! Outcomes selected by whether the addressed record exists (ess/16, beyond10x/ess#164,
//! `docs/design/outcome-shapes.md`), refused by every code target as `Json` is.
//!
//! Both forms — a creating branch marked `unknown_instance: true` beside the branch that updates
//! the record, and an `existing_instance:` refusal beside a creation — select a branch by a lookup
//! of the store before the input is dispatched. The generated seams and explorers select a branch
//! from the decoded input and the model's own state machine, and neither holds the question "does a
//! record carry this identity" as something a branch can be selected by. Each such branch is named
//! rather than emitted with a selection nobody chose.

use ess_compiler::ir::{ResolvedCondition, ResolvedEffect, ResolvedOutcome};

use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::{SynthesisPlan, Target};

/// Why the branch has no representation.
const DETAIL: &str =
    "this target cannot select a branch by whether a record carries the addressed identity";

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

/// Every branch selected by existence, named at the key its author wrote.
pub(crate) fn refuse(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
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
