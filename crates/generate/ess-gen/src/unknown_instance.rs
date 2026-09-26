//! Where a generated seam needs a second spelling of a command's `wrong_state` branch.
//!
//! The unknown-instance rule (`docs/design/typed-literals-and-unknown-instances.md`, section 2):
//! a command whose `instance:` names no record answers its `wrong_state` outcome. That outcome's
//! error usually describes the instance — `VisitStateConflict.state` is the state the visit is
//! really in — and an instance that does not exist has nothing to describe. A seam whose only
//! `wrong_state` spelling demands those fields cannot give the answer the rule requires, so every
//! projection of a command surface asks this one question and gets one answer
//! (`docs/design/unknown-instance-seams.md`).

use ess_compiler::ir::{
    ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedInstance, ResolvedOutcome,
};
use ess_compiler::EssIr;

/// The `wrong_state` outcome a command answers for an instance no record carries, when the
/// declared spelling of that outcome cannot say it.
///
/// `Some` exactly when all three hold:
///
/// 1. a branch acts on an existing instance the caller names — `moves:` or `updates:` with
///    `instance:` read from input — so an identity naming no record can reach the command;
/// 2. the command declares a `wrong_state` outcome, which is the rule's answer;
/// 3. that outcome reports an error carrying at least one field, which the declared spelling
///    requires and an absent instance cannot supply.
///
/// `None` everywhere else, and every projection of such a command keeps its bytes: where (1) fails
/// the case cannot arise, where (2) fails the specification declares no answer, and where (3)
/// fails the declared spelling already says it.
pub fn unknown_instance_answer<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
) -> Option<&'a ResolvedOutcome> {
    let reachable = command.outcomes.iter().any(|outcome| {
        outcome.subject.as_ref().is_some_and(|subject| {
            matches!(
                subject.effect,
                ResolvedEffect::Moves { .. } | ResolvedEffect::Updates
            ) && matches!(subject.instance, ResolvedInstance::Supplied { .. })
        })
    });
    if !reachable {
        return None;
    }
    let declared = command
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::WrongState)?;
    let error = declared.error.as_ref()?;
    (!ir.error(error).fields.is_empty()).then_some(declared)
}
