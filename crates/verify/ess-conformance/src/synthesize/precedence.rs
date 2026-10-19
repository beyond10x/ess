//! The one question every witness search asks of a command's precedence plan
//! (`docs/design/selection-plan.md`, `story:synthesis-reads-selection-plan`): which branches answer
//! before this one.
//!
//! A witness for a branch refutes every branch that would answer first, so each search needs that
//! set. It is read here from [`PrecedencePlan`], the one derivation of the precedence order, and the
//! searches filter it for the kind of branch they refute: an input-guarded refusal, an accepting
//! `when:`, a branch the held state selects. None of them orders branches by matching on condition
//! kinds. The plan reads the current phase order, so a test that exchanges phases with
//! [`with_phase_order`](ess_domain::command::precedence::with_phase_order) moves every search's
//! answer with it.
//!
//! The answers come in declaration order, which is the order every search walked before it read the
//! plan; the plan decides membership.
//!
//! Every plan is built in the document's own source format: a command's format moves a
//! `when_related:` predicate refusal between the present-related and the accepting phase
//! (`Composition::new`), and from the one rule (`ess_domain::command::precedence::ONE_RULE`) a
//! missing row the input names and `existing_instance:` beside a related row too.
use std::collections::BTreeSet;

use ess_compiler::ir::{PrecedencePlan, ResolvedCommand, ResolvedOutcome};
use ess_domain::command::precedence::Phase;
use ess_domain::command::OutcomeName;
use ess_domain::system::FormatVersion;

/// Every branch of `command`, declared in source format `format`, the precedence plan reads before
/// `outcome`, in declaration order.
///
/// Where `command` declares no branch named as `outcome` (a clone that dropped it), every other
/// branch.
pub(crate) fn answers_before<'c>(
    format: FormatVersion,
    command: &'c ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<&'c ResolvedOutcome> {
    let plan = PrecedencePlan::new(command, format);
    let before: BTreeSet<&OutcomeName> = plan
        .iter()
        .map(|(_, branch)| &branch.name)
        .take_while(|name| **name != outcome.name)
        .collect();
    command
        .outcomes
        .iter()
        .filter(|branch| branch.name != outcome.name && before.contains(&branch.name))
        .collect()
}

/// Every branch of `command`, declared in source format `format`, the precedence plan reads after
/// `outcome`, in declaration order; none where `command` declares no branch named as `outcome`.
pub(crate) fn answers_after<'c>(
    format: FormatVersion,
    command: &'c ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<&'c ResolvedOutcome> {
    let plan = PrecedencePlan::new(command, format);
    let after: BTreeSet<&OutcomeName> = plan
        .iter()
        .map(|(_, branch)| &branch.name)
        .skip_while(|name| **name != outcome.name)
        .skip(1)
        .collect();
    command
        .outcomes
        .iter()
        .filter(|branch| after.contains(&branch.name))
        .collect()
}

/// Of `branches`, all of `command`, the first the plan reads in a phase before both the held state
/// and the accepting branches, where one is: the branch that answers before every held-state and
/// accepting branch an input and a held state select alike.
///
/// The held-state and accepting phases are the two the interpreter once read in one
/// declaration-order pass; since beyond10x/ess#486 validation lets them be read in either order with
/// one answer, so a witness among them is one no other of them claims, and only a phase read before
/// both decides by order.
pub(crate) fn first_before_held_and_accepting<'c>(
    format: FormatVersion,
    command: &ResolvedCommand,
    branches: &[&'c ResolvedOutcome],
) -> Option<&'c ResolvedOutcome> {
    let plan = PrecedencePlan::new(command, format);
    let pair = Phase::HeldState.position().min(Phase::Accepting.position());
    let first = plan
        .iter()
        .filter(|(phase, _)| phase.position() < pair)
        .find_map(|(_, planned)| {
            branches
                .iter()
                .copied()
                .find(|branch| branch.name == planned.name)
        });
    first
}
