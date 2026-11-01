//! Rows a binding rewrites before the send (story
//! `an-outcome-arrangement-follows-the-bindings-its-steps-set-off`).
//!
//! A binding is part of the running system: a step that publishes its event invokes its command
//! there, before or after the scenario's next step. A row searched for a `when_related:` guard is
//! decided on what its own steps left in it ([`Arrangement::settled`]); where a step of the
//! scenario sets off a binding whose command writes a field the guard reads, the row the command
//! is sent for need not hold those values any more, and a target that runs the binding at once —
//! the model interpreter does — answers another branch. Synthesis does not follow what such a
//! binding leaves, so a row that selects the branch only through such a step is passed over, and
//! where no other row selects it the branch is refused naming the binding ([`refused`]).
//!
//! Read as `refusal_policy::rewritten` reads a chain: every branch at every hop, and every binding
//! on an emitted event whatever its condition, so the answer errs toward "rewritten"; each command
//! is visited once. Only a write the guard can read counts: a field one of the command's related
//! predicates reads, the lifecycle state where one reads `state`, or a removal of the row.

use std::collections::{BTreeSet, VecDeque};

use ess_compiler::ir::{EntityHandle, EssIr, ResolvedCommand, ResolvedEffect, ResolvedOutcome};
use ess_domain::name::QualifiedName;
use ess_primitives::predicate::Predicate;

use super::super::{Arrangement, RefusalCause, ScenarioStep};

/// One step of the scenario that sets off a binding writing what a related guard reads.
#[derive(Debug, Clone)]
pub(super) struct Rewrite {
    /// The command the step sends.
    step: QualifiedName,
    /// The binding it sets off, directly or down a chain.
    binding: String,
    /// The command that binding invokes.
    writer: QualifiedName,
    /// What that command does to rows of the guard's entity.
    what: String,
}

/// What the related predicates of `command` read of a row of `entity`: the declared fields their
/// facts name, and `state` where one reads the lifecycle state.
pub(super) fn read(
    ir: &EssIr,
    entity: &EntityHandle,
    predicates: &[Predicate],
) -> BTreeSet<String> {
    let declared = ir.entity(entity);
    predicates
        .iter()
        .flat_map(Predicate::fact_paths)
        .map(|path| path.namespace().to_owned())
        .filter(|root| {
            root == ess_domain::entity::EntitySpec::STATE
                || declared.fields.iter().any(|field| &field.name == root)
        })
        .collect()
}

/// The first step of `row`'s own steps, then of `before`, that sets off a binding writing what
/// `read` names on rows of `entity` ([`Rewrite`]); `None` where none does, or where the guard reads
/// nothing of the row.
pub(super) fn rewriting(
    ir: &EssIr,
    entity: &EntityHandle,
    read: &BTreeSet<String>,
    row: &Arrangement,
    before: &[&[ScenarioStep]],
) -> Option<Rewrite> {
    if read.is_empty() || ir.bindings().is_empty() {
        return None;
    }
    std::iter::once(row.steps.as_slice())
        .chain(before.iter().copied())
        .flatten()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => {
                set_off(ir, entity, read, command.name())
            }
            _ => None,
        })
}

/// The first binding sending `start` sets off, at any hop, whose command writes what `read` names
/// on rows of `entity`. `start`'s own branches are the arrangement and are not counted.
fn set_off(
    ir: &EssIr,
    entity: &EntityHandle,
    read: &BTreeSet<String>,
    start: &QualifiedName,
) -> Option<Rewrite> {
    let first = ir.commands().get(start)?;
    let mut visited = BTreeSet::from([start.clone()]);
    let mut pending: VecDeque<(&ess_compiler::ir::ResolvedBinding, &ResolvedCommand)> =
        bound(ir, first).collect();
    while let Some((binding, invoked)) = pending.pop_front() {
        if !visited.insert(invoked.name.clone()) {
            continue;
        }
        if let Some(what) = invoked
            .outcomes
            .iter()
            .find_map(|outcome| writes(outcome, entity, read))
        {
            return Some(Rewrite {
                step: start.clone(),
                binding: binding.name.to_string(),
                writer: invoked.name.clone(),
                what,
            });
        }
        pending.extend(bound(ir, invoked));
    }
    None
}

/// Every binding an event some branch of `command` emits sets off, with the command it invokes.
fn bound<'ir>(
    ir: &'ir EssIr,
    command: &'ir ResolvedCommand,
) -> impl Iterator<Item = (&'ir ess_compiler::ir::ResolvedBinding, &'ir ResolvedCommand)> {
    let emitted: BTreeSet<_> = command
        .outcomes
        .iter()
        .flat_map(|outcome| outcome.emits.iter())
        .collect();
    ir.bindings()
        .values()
        .filter(move |binding| {
            binding
                .cause
                .event()
                .is_some_and(|event| emitted.contains(event))
        })
        .map(|binding| (binding, ir.command(&binding.command)))
}

/// The first of `targets` that `read` names, as the phrase [`writes`] gives it.
fn first_read<'t>(
    read: &BTreeSet<String>,
    mut targets: impl Iterator<Item = &'t str>,
) -> Option<String> {
    targets
        .find(|target| read.contains(*target))
        .map(|target| format!("writes `{target}`"))
}

/// What `outcome` does to rows of `entity` that a guard reading `read` sees, as a phrase; `None`
/// for a refusal, a creation (a row of its own) and a write of nothing `read` names.
fn writes(
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    read: &BTreeSet<String>,
) -> Option<String> {
    if outcome.error.is_some() {
        return None;
    }
    let state = read.contains(ess_domain::entity::EntitySpec::STATE);
    let effect = |effect: &ResolvedEffect| match effect {
        ResolvedEffect::Deletes => Some("removes the row".to_owned()),
        ResolvedEffect::Moves { transition } if state => {
            Some(format!("moves the row to `{}`", transition.to))
        }
        _ => None,
    };
    let on_entity = outcome
        .subject
        .as_ref()
        .filter(|subject| &subject.entity == entity)
        .map(|subject| &subject.effect)
        .filter(|effect| !matches!(effect, ResolvedEffect::Creates))
        .or_else(|| {
            outcome
                .instances
                .as_ref()
                .filter(|set| &set.entity == entity)
                .map(|set| &set.effect)
        });
    if let Some(found) = on_entity.and_then(|on| {
        effect(on).or_else(|| first_read(read, outcome.sets.iter().map(|set| set.target.as_str())))
    }) {
        return Some(found);
    }
    outcome
        .affects
        .iter()
        .filter(|affect| &affect.entity == entity)
        .find_map(|affect| {
            if affect.deletes {
                return Some("removes the row".to_owned());
            }
            if let Some(transition) = affect.moves.as_ref().filter(|_| state) {
                return Some(format!("moves the row to `{}`", transition.to));
            }
            first_read(
                read,
                affect.sets.iter().map(|set| set.target.as_str()).chain(
                    affect
                        .each
                        .iter()
                        .flat_map(|each| each.reads.iter().map(|read| read.target.as_str())),
                ),
            )
        })
}

/// The refusal of `outcome` of `command` where the search for its related row of `entity` failed
/// with `cause` after passing over a row only `rewrite` kept from selecting it: the binding named,
/// and the candidates the search tried where it counted them.
pub(super) fn refused(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    rewrite: &Rewrite,
    cause: &RefusalCause,
) -> RefusalCause {
    let (tried, cut) = match cause {
        RefusalCause::GuardUnsatisfiable { tried, cut, .. } => (*tried, *cut),
        _ => (1, 0),
    };
    RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "a row of `{entity}` selecting `{}` of `{}` that no binding rewrites before the send: \
             the row that selects it is arranged through `{}`, which sets off `{}`, and its `{}` \
             {}, which the guard reads; synthesis does not follow what a binding leaves in an \
             arranged row",
            outcome.name, command.name, rewrite.step, rewrite.binding, rewrite.writer, rewrite.what
        ),
        tried,
        cut,
    }
}
