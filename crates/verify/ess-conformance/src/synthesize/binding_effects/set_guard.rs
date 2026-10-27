//! The branch of a set guard a binding's invocation takes, decided by arranging the rows it
//! selects before the trigger (a downstream adopter's report of 2026-10-26).
//!
//! As for a `via:` guard ([`related_decided`](super::related_decided)): the invoked command acts
//! on no existing row of its own, and every branch it can take unforced is the default or a set
//! guard, `when_related: {entity:, where:, …}`, with no input guard of its own. Each input a
//! selector reads is mapped from an event field the trigger fills from an input of its own, each
//! step unconverted or across a conversion that hands the value on as it is. Then the rows are the
//! scenario's to choose: they are arranged as the branch's own scenario arranges them
//! ([`row_set::prepare`](super::super::row_set::prepare)), the trigger is sent the input they
//! were arranged against, and the branch is decided again over the rows the arrangement and the
//! trigger leave ([`row_set::decided_after`](super::super::row_set::decided_after)), so a trigger
//! that changes them is followed, never assumed away.
//!
//! The accepting branch is the first, in declaration order, the rows so arranged decide. Where
//! none is, the binding is refused as [`BindingGap::BranchSetUnarranged`], naming the set guard,
//! with a remedy the specification can follow: the help of [`BindingGap::BranchUndecided`] asks
//! to declare the other branches `external:`, which a default branch cannot be.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedCommand, ResolvedCondition, ResolvedOutcome,
    ResolvedPayloadValue, ResolvedRowSelection,
};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::super::{captured, row_set, BindingGap, Run};
use super::{command_of, mapped_unchanged, payload_followed, Prepared};
use crate::scenario::{ActorRef, CommandRef, ScenarioStep, ScenarioValue};

/// The selections whose set guards decide `invoked`'s branch, where they alone decide it: every
/// branch it takes unforced is the default or a set guard with no input guard of its own, at least
/// one is a set guard, and no branch reads a related row through `via:`. `None` for every other
/// command, which keeps the arrangement and the gap it had.
fn guards(invoked: &ResolvedCommand) -> Option<Vec<&ResolvedRowSelection>> {
    if super::super::related_guard::uses(invoked) {
        return None;
    }
    let mut selections: Vec<&ResolvedRowSelection> = Vec::new();
    for outcome in &invoked.outcomes {
        if outcome.test_strategy == TestStrategy::InjectFault {
            continue;
        }
        if outcome.replays.is_some() {
            return None;
        }
        match &outcome.condition {
            ResolvedCondition::Otherwise => {}
            ResolvedCondition::RelatedSet {
                selection,
                input: None,
                ..
            } => {
                if !selections.contains(&selection) {
                    selections.push(selection);
                }
            }
            _ => return None,
        }
    }
    (!selections.is_empty()).then_some(selections)
}

/// The trigger of `binding` with the rows its invoked command's set guards select arranged before
/// it, so the invocation reaches an accepting branch; or, where that cannot be done, the trigger
/// alone with the gap [`BindingGap::BranchSetUnarranged`]. `None` for a command whose branch set
/// guards alone do not decide ([`guards`]).
pub(super) fn decided<'a>(
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Prepared<'a>> {
    let selections = guards(invoked)?;
    let arranged = read_through(ir, binding, published_by, &selections).and_then(|read| {
        invoked
            .outcomes
            .iter()
            .filter(|outcome| {
                outcome.error.is_none() && outcome.test_strategy != TestStrategy::InjectFault
            })
            .find_map(|accepting| {
                arranged_for(
                    ir,
                    (invoked, accepting),
                    published_by,
                    &read,
                    trigger,
                    actors,
                )
            })
    });
    Some(arranged.unwrap_or_else(|| Prepared {
        setup: trigger.setup.clone(),
        invoke: trigger.invoke.clone(),
        capture: Vec::new(),
        reached: Err(unarranged(invoked, &selections)),
        row: None,
        source: BTreeSet::new(),
    }))
}

/// Each input the selectors read, with the trigger's input it is filled from: mapped from an event
/// field the trigger fills from an input of its own, each step unconverted or across a conversion
/// that hands the value on as it is. `None` where any is filled another way, or a selector reads
/// a value of the invoked command's own payload.
fn read_through<'s>(
    ir: &EssIr,
    binding: &ResolvedBinding,
    published_by: &ResolvedOutcome,
    selections: &[&'s ResolvedRowSelection],
) -> Option<Vec<(&'s str, String)>> {
    let event = binding.cause.event()?;
    let mut read: Vec<(&str, String)> = Vec::new();
    for selection in selections {
        if !selection.reads.is_empty() {
            return None;
        }
        for field in row_set::input_reads(&selection.filter) {
            if read.iter().any(|(known, _)| *known == field) {
                continue;
            }
            let carried = mapped_unchanged(ir, binding, field)?;
            match payload_followed(ir, published_by, event, carried)? {
                (ResolvedPayloadValue::InputField { field: from, .. }, false)
                    if !ess_domain::command::input_path::is_path(from) =>
                {
                    read.push((field, from.clone()));
                }
                _ => return None,
            }
        }
    }
    Some(read)
}

/// The trigger with the rows `accepting`'s own scenario arranges before it, sent the input they
/// were arranged against, where the rows the steps leave take the invocation to `accepting`.
fn arranged_for<'a>(
    ir: &EssIr,
    (invoked, accepting): (&ResolvedCommand, &'a ResolvedOutcome),
    published_by: &ResolvedOutcome,
    read: &[(&str, String)],
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Prepared<'a>> {
    let models = super::super::caller::InvocationModels::plain(ir);
    let (setup, input, gaps) = row_set::prepare(&models, invoked, accepting, actors).ok()?;
    // Only the rows are the scenario's to choose: a row the input names is a value the event fills.
    if !gaps.is_empty() || setup.instance.is_some() || !setup.bound.is_empty() {
        return None;
    }
    let taken = captured(trigger);
    let collides = setup.steps.iter().any(|step| {
        matches!(step, ScenarioStep::CaptureInstance { instance, .. } if taken.contains(instance))
    });
    if collides {
        return None;
    }

    // The trigger is sent each value the rows were arranged against, where that keeps the branch
    // its own guards choose.
    let publisher = command_of(ir, published_by)?;
    let mut invoke = trigger.invoke.clone();
    let sent = invoke.iter_mut().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. }
            if command.name() == &publisher.name =>
        {
            Some(input)
        }
        _ => None,
    })?;
    let mut literals: BTreeMap<String, Node> = sent
        .iter()
        .filter_map(|(field, value)| {
            value
                .as_literal()
                .map(|value| (field.clone(), value.clone()))
        })
        .collect();
    let mut supplied = BTreeMap::new();
    let mut steered: BTreeMap<&str, &Node> = BTreeMap::new();
    for (field, from) in read {
        let value = input.get(*field)?;
        // Two inputs filled from one input of the trigger carry one value.
        if steered
            .insert(from, value)
            .is_some_and(|other| other != value)
        {
            return None;
        }
        if literals.get(from) != Some(value) {
            if !super::super::keeps_branch(ir, publisher, &literals, from, value) {
                return None;
            }
            literals.insert(from.clone(), value.clone());
            sent.insert(from.clone(), ScenarioValue::literal(value.clone()));
        }
        supplied.insert((*field).to_owned(), ScenarioValue::literal(value.clone()));
    }

    let mut before = trigger.setup.clone();
    before.extend(setup.steps);
    let mut steps = before.clone();
    steps.extend(invoke.iter().cloned());
    row_set::decided_after(ir, &steps, invoked, accepting, &supplied).ok()?;
    Some(Prepared {
        setup: before,
        invoke,
        capture: Vec::new(),
        reached: Ok(accepting),
        row: None,
        source: setup.source,
    })
}

/// [`BindingGap::BranchSetUnarranged`] for `invoked`, naming its first set guard.
fn unarranged(invoked: &ResolvedCommand, selections: &[&ResolvedRowSelection]) -> BindingGap {
    let guard = selections.first().map_or_else(String::new, |selection| {
        format!("`{}` where {}", selection.entity.name(), selection.filter)
    });
    BindingGap::BranchSetUnarranged {
        command: CommandRef::new(invoked.name.clone()),
        branches: invoked
            .outcomes
            .iter()
            .filter(|outcome| outcome.test_strategy != TestStrategy::InjectFault)
            .map(|outcome| outcome.name.clone())
            .collect(),
        guard,
    }
}
