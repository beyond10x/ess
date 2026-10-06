//! The row a `when_related:` guard reads (source format `ess/18`, beyond10x/ess#211).
//!
//! A command guarded by a row of another entity — the one whose identity an input field carries —
//! is reached by arranging that row, or by arranging its absence, and pointing the input at it. So
//! every branch of such a command is arranged here:
//!
//! * the `exists: false` branch is sent an identity no row carries — a fresh one, as an
//!   `unknown_instance:` scenario sends — beside two rows of the entity that carry others, so an
//!   implementation answering "some row exists" rather than "this row exists" fails it;
//! * every other branch is sent the identity of a row the scenario creates, between two decoys, and
//!   is selected by that row's stored fields crossed with the input: the predicate branches on the
//!   side of their predicate that selects them, the default on the side that selects none. The
//!   input is searched as a `when_subject` predicate's is, with each comparison between the row and
//!   the input grounded on the row's value, so a predicate is witnessed true in its own scenario and
//!   false in its siblings'.
//!
//! A missing row decides every predicate `Unknown`, and selects only `exists: false` — never a
//! predicate branch and never the default.
//!
//! A refusal naming no subject, beside a branch acting on an existing row, is sent for a row that
//! branch could act on. A driver running such a command for another scenario arranges the related
//! row first ([`drive`]). Every other family that would send the command — a boundary, an unknown
//! identity, an illegal move — has no row to point it at, and is refused at [`super::reach`] with
//! the strategy named, rather than sent for a row it did not arrange.
//!
//! A command reading several rows through its input (ess/22, beyond10x/ess#283) is arranged one row
//! at a time: each scenario around the row its branch reads ([`projection`]), every other row
//! present beside it with nothing selected that the precedence order answers before the branch
//! ([`beside`], [`Around`]), and, for a refusal, the overlaps the order decides sent first
//! ([`overlaps`]).

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedInstance,
    ResolvedOutcome, ResolvedPayloadValue, ResolvedRelatedTest, ResolvedRelatedVia,
};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_primitives::facts::FactPath;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

use super::{
    arrange_first, candidates, decides, flatten, fresh_identity, prepare_in, state_default,
    subject_fact, ActorRef, Arrangement, Distinction, RefusalCause, Setup,
};
use crate::witness::MAX_CANDIDATES;

pub(super) mod stored;

/// The step between the blocks of distinctions the related rows are arranged at: the number
/// [`super::related`] uses, for the reason it gives (every enum of up to twelve variants reads a
/// decoy one away from the row as another variant).
const BLOCK: usize = 27_720;

/// The block a scenario's own related rows start at, and the one a driver's start at: apart, so an
/// arrangement that drives such a command before the branch under test names no row twice.
const OWN: usize = 5;
const DRIVEN: usize = 7;

/// The block a related row of an entity already being arranged is created in, one level deep
/// ([`nested`]): apart from both, so it names no row a scenario or a driver arranges.
const NESTED: usize = 9;

/// The block the companions of a named row are created in (beyond10x/ess#270, [`surround`]): apart
/// from every other, so they name no row a scenario, a driver or a nested arrangement does.
const COMPANION: usize = 11;

/// The blocks the rows a scenario names beside the one it is arranged around are created in, one
/// per further row (beyond10x/ess#283, [`beside`]): a scenario's own from `BESIDE`, a driver's from
/// `BESIDE_DRIVEN`, and the rows of the overlaps a scenario sends ([`overlaps`]) from `OVERLAP`,
/// each four blocks wide and apart from every block above.
const BESIDE: usize = 13;
const BESIDE_DRIVEN: usize = 17;
const OVERLAP: usize = 21;

/// Input for a related-row overlap and the arranged identities it references.
type BoundInput = (
    BTreeMap<String, Node>,
    BTreeMap<String, crate::scenario::InstanceName>,
);

/// The distinction the related row of the `distinction`th witness is arranged at, in block `base`:
/// four apart, so no row and no decoy either side of it (one away) is another witness's, and every
/// further instance a driver arranges reads a related row of its own (beyond10x/ess#211).
fn block_start(base: usize, distinction: Distinction) -> usize {
    BLOCK * base + 4 * distinction.get()
}

/// The further witness an input is searched at when the one at `distinction` grounds nothing: `1`
/// for the plain witness, and past every further instance's own number otherwise, so a further
/// instance's retry never sends another instance's input.
fn retry(distinction: Distinction) -> Distinction {
    match distinction.get() {
        0 => Distinction::further(1),
        nth => Distinction::further(MAX_CANDIDATES * nth + 1),
    }
}

/// One further related row a branch is witnessed on: every predicate of the first list false on
/// it, every one of the second true.
pub(super) type Goal = (Vec<Predicate>, Vec<Predicate>);

/// Whether the related row `outcome`'s guard reads is the row its creation is owned by, named
/// through the same input field (`PostEntry` into an `Account` that owns `Entry`, guarded on
/// `input.account_id`): then that row is arranged once, as the related row, and not a second time
/// as the owner — which would bind the field the guard needs to point at before it could.
pub(super) fn owner_is_related(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> bool {
    focus(ir, command, outcome)
        .is_some_and(|(via, related)| owns_through(ir, outcome, via, related))
}

/// Whether `outcome` creates a row owned by a row of `related`, its owner link set from the input
/// `via` names unchanged.
fn owns_through(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    via: &ResolvedRelatedVia,
    related: &EntityHandle,
) -> bool {
    let Some(subject) = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Creates)
    else {
        return false;
    };
    let Some(belongs) = ir.owner_of(&subject.entity) else {
        return false;
    };
    belongs.owner == *related
        && outcome.sets.iter().any(|set| {
            set.target == belongs.via
                && set.conversion.is_none()
                && matches!(&set.value, ResolvedPayloadValue::InputField { field, .. } if field == via.field())
        })
}

/// Whether the document's format arranges a related row of an entity already being arranged, one
/// level deep ([`nested`]): from `ess/20`, the format whose `state` operand in `when_related` needs
/// it (beyond10x/ess#229). Below it such a run stops, as it did before that format existed.
pub(super) fn nests(ir: &EssIr) -> bool {
    ir.format().major() >= ess_domain::system::FormatVersion::V20.major()
}

/// Whether this command's related guards read a row of `entity`.
pub(super) fn reads_entity(command: &ResolvedCommand, entity: &EntityHandle) -> bool {
    rows(command).iter().any(|(_, related)| *related == entity)
}

/// Whether any branch of this command reads a related row.
pub(super) fn uses(command: &ResolvedCommand) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, ResolvedCondition::Related { .. }))
}

/// Whether this strategy arranges the scenario for `outcome`: every branch of a command reading a
/// related row that the row decides, which is all of them but an injected fault and a replay.
pub(super) fn routes(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    uses(command)
        && outcome.replays.is_none()
        && !matches!(
            outcome.condition,
            ResolvedCondition::External { .. }
                | ResolvedCondition::ExternalWhen { .. }
                // The command's own identity, checked first: the existence family arranges it.
                | ResolvedCondition::ExistingInstance
        )
}

/// Whether `outcome` is an external branch of a command reading a related row through its input
/// (beyond10x/ess#464), which [`routes`] leaves out — its boundaries and further witnesses are its
/// siblings' — but whose own scenario, and every run driving a row through it, is sent naming a
/// present row no `when_related` branch claims ([`prepare_at_in`], [`drive`]).
pub(super) fn arranges_external(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    uses(command)
        && stored::field(command).is_none()
        && outcome.replays.is_none()
        && is_external(outcome)
}

/// Whether `outcome` is an external branch of a command whose `when_related` guard reads a stored
/// reference of the addressed row (ess/22, beyond10x/ess#304), sent by the plain witness `setup`
/// for a row whose reference is not known to be left out (beyond10x/ess#464). Which row such a
/// reference names is not arranged for an external branch, so a missing one — or no addressed row
/// at all, for a branch naming no subject of its own — would answer before it.
pub(super) fn stored_external(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    setup: &Setup,
) -> bool {
    is_external(outcome)
        && outcome.replays.is_none()
        && stored::field(command).is_some_and(|(field, _)| {
            setup.settled.get(field).is_none_or(|held| {
                !matches!(
                    held.value,
                    crate::scenario::ScenarioValue::Literal { value: Node::Null }
                )
            })
        })
}

/// The refusal of a [`stored_external`] branch, naming the `when_related` branches over the
/// stored reference.
pub(super) fn stored_external_refused(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> RefusalCause {
    let field = stored::field(command).map_or("", |(field, _)| field);
    let claiming: Vec<String> = command
        .outcomes
        .iter()
        .filter_map(|branch| match &branch.condition {
            ResolvedCondition::Related { test, .. } => Some(match test {
                ResolvedRelatedTest::Absent => format!("`{}` (exists: false)", branch.name),
                ResolvedRelatedTest::Holds { predicate } => {
                    format!("`{}` ({predicate})", branch.name)
                }
            }),
            _ => None,
        })
        .collect();
    RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{}` forced on a row whose stored reference `subject.{field}` no `when_related` \
             branch claims: none of {}; such a reference is not arranged for an external branch",
            outcome.name,
            claiming.join(", ")
        ),
        tried: 0,
    }
}

/// Whether `outcome` is decided by a provider.
fn is_external(outcome: &ResolvedOutcome) -> bool {
    matches!(
        outcome.condition,
        ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
    )
}

/// Whether `input`, or an arranged row `bound` names, points the command's related guards at a
/// row: a reference sent, or one an arrangement bound. A reference left out reads no row, and
/// selects no `when_related` branch.
pub(super) fn reads_row(
    command: &ResolvedCommand,
    input: &BTreeMap<String, Node>,
    bound: &BTreeMap<String, crate::scenario::InstanceName>,
) -> bool {
    rows(command).iter().any(|(via, _)| {
        bound.contains_key(via.field())
            || ess_compiler::ir::read_input(input, via.field())
                .is_some_and(|value| !matches!(value, Node::Null))
    })
}

/// The refusal of an external branch for which no present related row and input leave the
/// branch to answer (beyond10x/ess#464), naming the `when_related` branches that claim it. A cause
/// that is no unmet guard — a row that cannot be arranged at all — is its own.
pub(super) fn external_unwitnessed(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    cause: RefusalCause,
) -> RefusalCause {
    let RefusalCause::GuardUnsatisfiable { tried, .. } = cause else {
        return cause;
    };
    let claiming: Vec<String> = command
        .outcomes
        .iter()
        .filter_map(|branch| match &branch.condition {
            ResolvedCondition::Related {
                via,
                test: ResolvedRelatedTest::Holds { predicate },
                input,
                ..
            } => Some(match input {
                Some(guard) => format!("`{}` ({via}: {predicate} and {guard})", branch.name),
                None => format!("`{}` ({via}: {predicate})", branch.name),
            }),
            _ => None,
        })
        .collect();
    RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{}` forced beside a present related row on which no `when_related` branch claims \
             the input: none of {}",
            outcome.name,
            claiming.join(", ")
        ),
        tried,
    }
}

/// Whether the forced external `outcome` answers `command` sent `input` for the present related
/// row `row` (beyond10x/ess#464): its own guard holds; no input-guarded refusal and no accepting
/// `when:` declared before it claims the input; and no `when_related` predicate branch holds on
/// the row with its input guard, whatever its declaration order, one the row leaves undecided
/// counting as holding.
fn leaves_external(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    row: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> Result<bool, RefusalCause> {
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    if let ResolvedCondition::ExternalWhen { predicate, .. } = &outcome.condition {
        if !decides(&facts, &[predicate], true)? {
            return Ok(false);
        }
    }
    let refusals: Vec<&Predicate> = super::sibling_refusals(command, outcome)
        .filter_map(super::when)
        .collect();
    if !decides(&facts, &refusals, false)?
        || super::claimed_by(&facts, &super::earlier_accepting(command, outcome))
    {
        return Ok(false);
    }
    for branch in &command.outcomes {
        let ResolvedCondition::Related {
            test: ResolvedRelatedTest::Holds { predicate },
            ..
        } = &branch.condition
        else {
            continue;
        };
        match subject_fact::guard_truth_with(
            ir,
            entity,
            &row.settled,
            &row.unwritten,
            Some(&row.state),
            predicate,
            Some((command, input)),
        ) {
            Truth::False => continue,
            Truth::Unknown => return Ok(false),
            Truth::True => {}
        }
        let refuted = match input_guard(branch) {
            Some(guard) => decides(&facts, &[guard], false)?,
            None => false,
        };
        if !refuted {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether the present row `row`, crossed with `input`, leaves `outcome` to answer: the branch
/// [`selects`] picks there — or, for an external branch, which no row selects, the one no sibling
/// claims ([`leaves_external`]).
fn chooses(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    row: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> bool {
    if is_external(outcome) {
        return leaves_external(ir, command, outcome, entity, row, input).unwrap_or(false);
    }
    selects(ir, command, entity, Some(row), input)
        .ok()
        .flatten()
        .is_some_and(|branch| branch.name == outcome.name)
}

/// The refusal for a family that has no related row to send the command for.
pub(super) fn unarranged() -> RefusalCause {
    RefusalCause::StrategyWithoutGuard {
        strategy: TestStrategy::ArrangeRelatedRow,
    }
}

/// The input field the command's related guards read and the entity whose row it names, for an
/// arrangement outside this module that chooses the row's surroundings (an aggregate view's rows,
/// beyond10x/ess#272).
pub(super) fn reads(command: &ResolvedCommand) -> Option<(&str, &EntityHandle)> {
    read(command).map(|(via, entity)| (via.field(), entity))
}

/// The input field the command's related guards read and the entity whose row it names.
fn read(command: &ResolvedCommand) -> Option<(&ResolvedRelatedVia, &EntityHandle)> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related { via, entity, .. } => Some((via, entity)),
            _ => None,
        })
}

/// Every related row the command's guards read — the `via` naming it and its entity — in the
/// order first declared: one, but on a command reading several rows through its input (ess/22,
/// beyond10x/ess#283).
fn rows(command: &ResolvedCommand) -> Vec<(&ResolvedRelatedVia, &EntityHandle)> {
    let mut rows: Vec<(&ResolvedRelatedVia, &EntityHandle)> = Vec::new();
    for outcome in &command.outcomes {
        if let ResolvedCondition::Related { via, entity, .. } = &outcome.condition {
            if !rows.iter().any(|(read, _)| read.field() == via.field()) {
                rows.push((via, entity));
            }
        }
    }
    rows
}

/// Whether the command's guards read more than one related row (ess/22, beyond10x/ess#283). Its
/// scenarios are then arranged around one row at a time ([`projection`]), with every other row
/// arranged beside it so that its guards select nothing ([`beside`]).
pub(super) fn several(command: &ResolvedCommand) -> bool {
    rows(command).len() > 1
}

/// The row a scenario of `outcome` is arranged around: the one its own guard reads; for a branch
/// reading none, the first whose row it copies a value from or files its creation under, else the
/// first declared.
fn focus<'c>(
    ir: &EssIr,
    command: &'c ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Option<(&'c ResolvedRelatedVia, &'c EntityHandle)> {
    let rows = rows(command);
    if let ResolvedCondition::Related { via, .. } = &outcome.condition {
        return rows
            .into_iter()
            .find(|(read, _)| read.field() == via.field());
    }
    rows.iter()
        .copied()
        .find(|(via, entity)| {
            !super::related::guarded_fields(ir, outcome, (via.field(), entity)).is_empty()
                || owns_through(ir, outcome, via, entity)
        })
        .or_else(|| rows.first().copied())
}

/// `command` as the guards over the row `field` names read it: every branch reading another row
/// left out. Where every other row is arranged so that nothing answering before the branch under
/// test is selected there ([`answers_before`]), this command selects that branch where the whole
/// one does.
fn projection(command: &ResolvedCommand, field: &str) -> ResolvedCommand {
    let mut projected = command.clone();
    projected
        .outcomes
        .retain(|outcome| match &outcome.condition {
            ResolvedCondition::Related { via, .. } => via.field() == field,
            _ => true,
        });
    projected
}

/// The branch of `projected` standing for `outcome`, where it kept one.
fn kept<'p>(
    projected: &'p ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Result<&'p ResolvedOutcome, RefusalCause> {
    projected
        .outcomes
        .iter()
        .find(|branch| branch.name == outcome.name)
        .ok_or_else(unarranged)
}

/// Every predicate a branch of the command holds of the row `field` names, in declaration order.
fn predicates_over(command: &ResolvedCommand, field: &str) -> Vec<Predicate> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                via,
                test: ResolvedRelatedTest::Holds { predicate },
                ..
            } if via.field() == field => Some(predicate.clone()),
            _ => None,
        })
        .collect()
}

/// A row a scenario names beside the one it is arranged around (beyond10x/ess#283): one of its own
/// entity on which no branch the precedence order answers before the branch under test is selected.
pub(super) struct Beside<'c> {
    via: &'c ResolvedRelatedVia,
    entity: &'c EntityHandle,
    row: Arrangement,
}

/// Whether a branch over another row, holding, answers before `outcome` in the precedence order
/// (beyond10x/ess#283). Nothing does before a missing row's `exists: false`, an input-guarded
/// refusal, `wrong_state` or `existing_instance`, all answered before any present row's predicate;
/// before a present-related predicate refusal, a refusal over another row declared earlier does;
/// before anything else, every branch over another row does — a refusal answers first, and two
/// acceptances are one request selecting two branches.
fn answers_before(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    branch: &ResolvedOutcome,
) -> bool {
    let declared = |of: &ResolvedOutcome| {
        command
            .outcomes
            .iter()
            .position(|declared| declared.name == of.name)
    };
    match &outcome.condition {
        ResolvedCondition::Related {
            test: ResolvedRelatedTest::Absent,
            ..
        }
        | ResolvedCondition::WrongState
        | ResolvedCondition::ExistingInstance => false,
        _ if super::is_input_guarded_refusal(outcome) => false,
        ResolvedCondition::Related {
            test: ResolvedRelatedTest::Holds { .. },
            ..
        } if outcome.error.is_some() => {
            branch.error.is_some() && declared(branch) < declared(outcome)
        }
        _ => true,
    }
}

/// Whether the row `row` of `entity`, named through `field`, leaves `outcome` to answer: whether no
/// branch over it that [`answers_before`] `outcome` holds there, crossed with `input` where one is
/// chosen. `Unknown` where a predicate or its input guard is not decided — without an input, every
/// one that reads it.
fn leaves_to(
    ir: &EssIr,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    (field, entity, row): (&str, &EntityHandle, &Arrangement),
    input: Option<&BTreeMap<String, Node>>,
) -> Result<Truth, RefusalCause> {
    let facts = input
        .map(|input| flatten(ir, command, input).map_err(RefusalCause::WitnessRejected))
        .transpose()?;
    let mut leaves = Truth::True;
    for branch in &command.outcomes {
        let ResolvedCondition::Related {
            via,
            test: ResolvedRelatedTest::Holds { predicate },
            ..
        } = &branch.condition
        else {
            continue;
        };
        if via.field() != field || !answers_before(command, outcome, branch) {
            continue;
        }
        let held = subject_fact::guard_truth_with(
            ir,
            entity,
            &row.settled,
            &row.unwritten,
            Some(&row.state),
            predicate,
            input.map(|input| (command, input)),
        );
        let guarded = match (input_guard(branch), &facts) {
            (None, _) => Truth::True,
            (Some(guard), Some(facts)) => {
                if decides(facts, &[guard], true)? {
                    Truth::True
                } else {
                    Truth::False
                }
            }
            (Some(_), None) => Truth::Unknown,
        };
        leaves = leaves.and(held.and(guarded).not());
    }
    Ok(leaves)
}

/// For every row the command's guards read but the one `focus` names, a row of its entity that
/// leaves `outcome` to answer ([`leaves_to`]) whatever the input — or, where not `strict`, one no
/// branch answering before it is known to hold on without the input, which the input search then
/// decides ([`Around`]) — arranged in the blocks from `base` at `distinction`. Refused where no
/// bounded arrangement holds such a row.
fn beside<'c>(
    ir: &EssIr,
    (command, outcome): (&'c ResolvedCommand, &ResolvedOutcome),
    focus: &str,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    (base, strict): (usize, bool),
) -> Result<Vec<Beside<'c>>, RefusalCause> {
    let mut arranged = Vec::new();
    for (index, (via, entity)) in rows(command)
        .into_iter()
        .filter(|(via, _)| via.field() != focus)
        .enumerate()
    {
        if index >= 4 {
            return Err(unarranged());
        }
        let predicates = predicates_over(command, via.field());
        let at = block_start(base + index, distinction);
        let truth = |node: &Arrangement| {
            leaves_to(ir, (command, outcome), (via.field(), entity, node), None)
                .unwrap_or(Truth::False)
        };
        let row = search_rows(ir, entity, actors, (at, None), &predicates, |node| {
            Ok(truth(node) == Truth::True)
        })
        .or_else(|| row_at(ir, entity, actors, at, &[]).filter(|node| truth(node) == Truth::True))
        .or_else(|| {
            (!strict)
                .then(|| {
                    search_rows(ir, entity, actors, (at, None), &predicates, |node| {
                        Ok(truth(node) != Truth::False)
                    })
                })
                .flatten()
        })
        .ok_or_else(|| RefusalCause::GuardUnsatisfiable {
            predicate: format!(
                "a row of `{}` for `{via}` of `{}` on which no branch answering before `{}` is \
                 selected, beside the row the scenario is arranged around",
                entity.name(),
                command.name,
                outcome.name
            ),
            tried: 1,
        })?;
        arranged.push(Beside { via, entity, row });
    }
    Ok(arranged)
}

/// The rows arranged beside the one a scenario is arranged around, as the input search for that
/// row reads them (beyond10x/ess#283): `outcome` is the branch under test in the whole `command`.
#[derive(Clone, Copy)]
struct Around<'a> {
    command: &'a ResolvedCommand,
    outcome: &'a ResolvedOutcome,
    rows: &'a [Beside<'a>],
}

impl Around<'_> {
    /// Every comparison a predicate over a row beside makes with the input, grounded on that row
    /// ([`subject_fact::grounded`]): literals the input search tries, so an input on which no such
    /// branch holds is among the candidates.
    fn hints(&self, ir: &EssIr) -> Vec<Predicate> {
        self.rows
            .iter()
            .flat_map(|row| {
                subject_fact::grounded(
                    ir,
                    row.entity,
                    &row.row.settled,
                    &predicates_over(self.command, row.via.field()),
                )
            })
            .collect()
    }

    /// Whether, sent with `input`, every row beside leaves the branch under test to answer.
    fn admits(&self, ir: &EssIr, input: &BTreeMap<String, Node>) -> bool {
        self.rows.iter().all(|row| {
            leaves_to(
                ir,
                (self.command, self.outcome),
                (row.via.field(), row.entity, &row.row),
                Some(input),
            )
            .is_ok_and(|truth| truth == Truth::True)
        })
    }
}

/// Whether, sent with `input`, the row `beside` names leaves `outcome` to answer.
fn passes(
    ir: &EssIr,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    beside: &Beside<'_>,
    input: &BTreeMap<String, Node>,
) -> Result<bool, RefusalCause> {
    leaves_to(
        ir,
        (command, outcome),
        (beside.via.field(), beside.entity, &beside.row),
        Some(input),
    )
    .map(|truth| truth == Truth::True)
}

/// The row whose predicate `goal` is a boundary of: the first branch over a row whose
/// [`boundary_goals`] name it.
fn goal_row<'c>(
    ir: &EssIr,
    command: &'c ResolvedCommand,
    goal: &Goal,
) -> Option<(&'c ResolvedRelatedVia, &'c EntityHandle)> {
    command.outcomes.iter().find_map(|branch| {
        let ResolvedCondition::Related { via, .. } = &branch.condition else {
            return None;
        };
        boundary_goals(ir, command, branch)
            .iter()
            .any(|(known, _)| known == goal)
            .then(|| {
                rows(command)
                    .into_iter()
                    .find(|(read, _)| read.field() == via.field())
            })
            .flatten()
    })
}

/// A branch's input guard: its `when:`, or the `when:` beside its `when_related:`.
fn input_guard(outcome: &ResolvedOutcome) -> Option<&Predicate> {
    match &outcome.condition {
        ResolvedCondition::When { predicate } => Some(predicate),
        ResolvedCondition::Related { input, .. } => input.as_ref(),
        _ => None,
    }
}

/// Every predicate a branch of the command holds of the related row, in declaration order.
fn predicates(command: &ResolvedCommand) -> Vec<Predicate> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                test: ResolvedRelatedTest::Holds { predicate },
                ..
            } => Some(predicate.clone()),
            _ => None,
        })
        .collect()
}

/// The command's `exists: false` branch, where it declares one.
fn absent_branch(command: &ResolvedCommand) -> Option<&ResolvedOutcome> {
    command.outcomes.iter().find(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::Related {
                test: ResolvedRelatedTest::Absent,
                ..
            }
        )
    })
}

/// Which branch the command selects for this input, with the related row `row` present — or absent,
/// where it is `None` — if exactly one does.
///
/// A missing row is answered by the `exists: false` branch before any other (the #211 ruling). With
/// the row present, a predicate it does not decide (a field the arrangement did not determine)
/// selects nothing and is not guessed at, and an input-guarded refusal is taken before any branch it
/// overlaps (beyond10x/ess#178), as everywhere else. `existing_instance:` is answered before either
/// and is not asked here: the command's own identity is the existence family's to arrange.
fn selects<'c>(
    ir: &EssIr,
    command: &'c ResolvedCommand,
    entity: &EntityHandle,
    row: Option<&Arrangement>,
    input: &BTreeMap<String, Node>,
) -> Result<Option<&'c ResolvedOutcome>, RefusalCause> {
    let Some(row) = row else {
        return Ok(absent_branch(command));
    };
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    let mut selected = Vec::new();
    for branch in command.outcomes.iter().filter(|branch| {
        !state_default(branch)
            && !matches!(
                branch.condition,
                ResolvedCondition::External { .. }
                    | ResolvedCondition::ExternalWhen { .. }
                    | ResolvedCondition::ExistingInstance
                    // Selected from the addressed row's held state, before this row is read.
                    | ResolvedCondition::WrongState
            )
    }) {
        if let ResolvedCondition::Related { test, .. } = &branch.condition {
            match test {
                ResolvedRelatedTest::Absent => continue,
                ResolvedRelatedTest::Holds { predicate } => match subject_fact::guard_truth_with(
                    ir,
                    entity,
                    &row.settled,
                    &row.unwritten,
                    Some(&row.state),
                    predicate,
                    Some((command, input)),
                ) {
                    Truth::True => {}
                    Truth::False => continue,
                    Truth::Unknown => return Ok(None),
                },
            }
        }
        if let Some(guard) = input_guard(branch) {
            if !decides(&facts, &[guard], true)? {
                continue;
            }
        }
        selected.push(branch);
    }
    // Of the input refusals these facts select, the first declared answers (the precedence order,
    // `docs/design/cross-record-and-stored-field-guards.md`); `selected` keeps declaration order.
    if let Some(first) = selected
        .iter()
        .copied()
        .find(|branch| super::is_input_guarded_refusal(branch))
    {
        selected = vec![first];
    } else if let Some(first) = orders_present_related_refusal(ir, command)
        .then(|| {
            selected.iter().copied().find(|branch| {
                branch.error.is_some()
                    && matches!(
                        branch.condition,
                        ResolvedCondition::Related {
                            test: ResolvedRelatedTest::Holds { .. },
                            ..
                        }
                    )
            })
        })
        .flatten()
    {
        // From ess/22 the present-related predicate refusal answers before every accepting branch.
        // Validation keeps two selected related refusals ambiguous, so this first match does not
        // introduce a declaration-order tie-break between them.
        selected = vec![first];
    }
    Ok(match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    })
}

pub(super) fn orders_present_related_refusal(ir: &EssIr, command: &ResolvedCommand) -> bool {
    // A stored reference is read after the addressed row's existence and held state, and its
    // present-related refusal answers before every accepting branch (beyond10x/ess#304).
    if stored::field(command).is_some() {
        return true;
    }
    // Several rows: the first declared present-related refusal whose predicate holds answers
    // across them, after the addressed row's existence and held state (beyond10x/ess#283).
    if several(command) {
        return true;
    }
    ir.format().major() >= ess_domain::system::FormatVersion::V22.major()
        && command
            .outcomes
            .iter()
            .any(|branch| matches!(branch.condition, ResolvedCondition::WrongState))
        && command.outcomes.iter().any(|branch| {
            branch.error.is_some()
                && matches!(
                    branch.condition,
                    ResolvedCondition::Related {
                        test: ResolvedRelatedTest::Holds { .. },
                        ..
                    }
                )
        })
}

/// Arrange a present related row that selects the predicate refusal, while the addressed
/// subject is already in the wrong state. This is the overlap the wrong-state scenario must send
/// to distinguish the ess/22 order from a target that reads the related row first.
pub(super) fn wrong_state_overlap(
    ir: &EssIr,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    addressed: &mut Arrangement,
) -> Result<BoundInput, RefusalCause> {
    // The held state answers before a stored reference is read: the subject is sent as its
    // arrangement left it (beyond10x/ess#304).
    if stored::field(command).is_some() {
        return Ok((stored::wrong_state_input(ir, command)?, BTreeMap::new()));
    }
    // Several rows (beyond10x/ess#283): the row of the first present-related refusal arranged to
    // select it, as for one row, and every other row present beside it — a missing one would be
    // answered before the held state.
    if several(command) {
        let refusing = command.outcomes.iter().find(|outcome| {
            outcome.error.is_some()
                && matches!(
                    outcome.condition,
                    ResolvedCondition::Related {
                        test: ResolvedRelatedTest::Holds { .. },
                        ..
                    }
                )
        });
        // Without a present-related refusal every row is arranged beside: none is selected.
        let (focused, (input, mut bound)) =
            match refusing.and_then(|refusal| focus(ir, command, refusal)) {
                Some((via, _)) => {
                    let projected = projection(command, via.field());
                    (
                        via.field(),
                        wrong_state_overlap(ir, &projected, actors, addressed)?,
                    )
                }
                None => (
                    "",
                    (
                        plain_input(ir, command, Distinction::PLAIN)?,
                        BTreeMap::new(),
                    ),
                ),
            };
        // The held state answers before any present row's predicate: any row it can read will do.
        let held = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::WrongState)
            .or_else(|| command.outcomes.iter().find(|outcome| is_absent(outcome)))
            .ok_or_else(unarranged)?;
        let others = beside(
            ir,
            (command, held),
            focused,
            actors,
            Distinction::PLAIN,
            (BESIDE, true),
        )?;
        for other in others {
            addressed.steps.extend(other.row.steps);
            addressed.source.extend(other.row.source);
            addressed.source.insert(entity_ref(other.entity));
            bound.insert(other.via.field().to_owned(), other.row.instance);
        }
        return Ok((input, bound));
    }
    let refusal = command
        .outcomes
        .iter()
        .find(|outcome| {
            outcome.error.is_some()
                && matches!(
                    outcome.condition,
                    ResolvedCondition::Related {
                        test: ResolvedRelatedTest::Holds { .. },
                        ..
                    }
                )
        })
        .ok_or_else(unarranged)?;
    let (via, entity) = read(command).ok_or_else(unarranged)?;
    let (row, _, input) = with_row(
        ir,
        command,
        refusal,
        entity,
        actors,
        (OWN, Distinction::PLAIN, &[]),
        None,
        None,
        None,
        (&BTreeMap::new(), &addressed.steps),
    )?;
    addressed.steps.extend(row.steps);
    addressed.source.extend(row.source);
    addressed.source.insert(entity_ref(entity));
    let bound = BTreeMap::from([(via.field().to_owned(), row.instance)]);
    Ok((input, bound))
}

/// The first branch of the command acting on an existing row the input names, and that input.
fn acting(command: &ResolvedCommand) -> Option<(&ResolvedOutcome, &str)> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.subject.as_ref()?.instance {
            ResolvedInstance::Supplied { field } if outcome.replays.is_none() => {
                Some((outcome, field.name.as_str()))
            }
            _ => None,
        })
}

/// Whether `outcome` is the `exists: false` branch.
fn is_absent(outcome: &ResolvedOutcome) -> bool {
    matches!(
        outcome.condition,
        ResolvedCondition::Related {
            test: ResolvedRelatedTest::Absent,
            ..
        }
    )
}

/// One run of a branch of a command reading a related row as an arranging act — a driver bringing
/// its subject into existence or along its lifecycle for another scenario: the related row the
/// branch needs arranged first, under a block of its own, and the input pointed at it, or at an
/// identity no row carries where the branch is the `exists: false` one.
///
/// `input`, where the caller chose it, is sent as it is but for the field `via` names; otherwise
/// one is searched for. `arranging` names the entities whose rows are being arranged around this
/// run, the entity it creates or moves included: a related row of one of them would arrange itself
/// again, so the run stops there and the caller tries its next creator.
///
/// `distinction` is the caller's: the input searched for and the related row arranged are that
/// witness's, so a further instance created through this command carries an identity and a related
/// row of its own rather than the plain instance's.
///
/// `known` are the steps the caller ran to arrange what `bound` names: an owner they create and
/// file no row of the related entity under is one a `cardinality: one` relation still admits a row
/// under (beyond10x/ess#271, [`super::holds_none`]).
// One argument per thing an arranging run is told, and the witness it is for.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive(
    ir: &EssIr,
    driver: &super::Driver<'_>,
    instance: Option<&crate::scenario::InstanceName>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    (bound, known): (
        &BTreeMap<String, crate::scenario::InstanceName>,
        &[super::ScenarioStep],
    ),
    input: Option<&BTreeMap<String, Node>>,
    arranging: &[&EntityHandle],
) -> Result<super::Invocation, RefusalCause> {
    // A stored reference is read from the row being driven, which [`stored::step`] is given.
    if stored::field(driver.command).is_some() {
        return Err(unarranged());
    }
    // Several rows (beyond10x/ess#283): the run is driven around one of them, every other row
    // arranged beside it first so that nothing answering before the driven branch can hold there,
    // whatever the input.
    if several(driver.command) {
        let (via, _) = focus(ir, driver.command, driver.outcome).ok_or_else(unarranged)?;
        let projected = projection(driver.command, via.field());
        let around = super::Driver {
            command: &projected,
            outcome: kept(&projected, driver.outcome)?,
            effect: driver.effect,
        };
        let others = beside(
            ir,
            (driver.command, driver.outcome),
            via.field(),
            actors,
            distinction,
            (BESIDE_DRIVEN, true),
        )?;
        let mut bound = bound.clone();
        for other in &others {
            if bound.contains_key(other.via.field()) {
                return Err(unarranged());
            }
            bound.insert(other.via.field().to_owned(), other.row.instance.clone());
        }
        let mut invocation = drive(
            ir,
            &around,
            instance,
            actors,
            distinction,
            (&bound, known),
            input,
            arranging,
        )?;
        let mut steps = Vec::new();
        for other in others {
            steps.extend(other.row.steps);
            invocation.source.extend(other.row.source);
            invocation.source.insert(entity_ref(other.entity));
        }
        steps.append(&mut invocation.steps);
        invocation.steps = steps;
        return Ok(invocation);
    }
    let (via, entity) = read(driver.command).ok_or_else(unarranged)?;
    let field = via.field();
    let names_subject = driver.outcome.subject.as_ref().is_some_and(|subject| {
        matches!(&subject.instance, ResolvedInstance::Supplied { field: named } if named.name == field)
    });
    // Below `ess/20` a related row of an entity already being arranged stops the run, before
    // any branch is looked at, as it always did: a document without an `ess/20` construct
    // synthesizes the suite it did (beyond10x/ess#229, adversary pass 2).
    if names_subject || bound.contains_key(field) || (!nests(ir) && arranging.contains(&entity)) {
        return Err(unarranged());
    }
    // From `ess/20`, a missing row arranges nothing, so the `exists: false` branch is sent
    // whatever is being arranged around it: no row of `entity` is created, and nothing recurses.
    if is_absent(driver.outcome) {
        let input = match input {
            Some(chosen) => {
                let mut chosen = chosen.clone();
                let fresh = fresh_identity(ir, driver.command, field, Some(&chosen))?;
                chosen.insert(field.to_owned(), fresh);
                chosen
            }
            None => without_row(ir, driver.command, entity, field, distinction)?,
        };
        return Ok(super::invoke_with(
            ir, driver, instance, actors, bound, &input,
        ));
    }
    // A related row of an entity already being arranged — the mover that brings a candidate to
    // `Accepted` reading a parent candidate — is arranged one level deep, where it rests in its
    // initial state, and never searched for along its lifecycle (beyond10x/ess#229, adversary pass
    // 1).
    // Reached from `ess/20` only: below it the run stopped above.
    if arranging.contains(&entity) {
        // Where no row one level deep selects the branch — the row would need a related row of
        // its own again — an Optional reference the branch is answered without is left out
        // (ess/22, beyond10x/ess#304): no row is read, so none is arranged.
        return nested(
            ir,
            driver,
            instance,
            actors,
            distinction,
            bound,
            input,
            arranging,
        )
        .or_else(|cause| {
            without_reference(ir, driver, instance, actors, distinction, bound, input).ok_or(cause)
        });
    }
    // An owner the caller already bound for a link input — the one `created_owned` arranged for the
    // subject this run creates — is the owner the run is sent naming: the related row is arranged
    // against it, never against a second owner that would replace it (beyond10x/ess#271).
    searched(
        ir,
        driver,
        instance,
        actors,
        distinction,
        (bound, known),
        input,
        arranging,
    )
    .map(|(invocation, _)| invocation)
}

/// [`drive`] for a creating branch an aggregate view's rows are created through, where rows given
/// one value of the input the guard reads share one related row (beyond10x/ess#272): the run, and
/// the related row it arranged — its steps already the run's — for [`drive_on`] to send a later
/// row against. Refused where [`drive`] would not search for the row: a branch sent naming its
/// own subject or a bound row through the guard's input, the `exists: false` branch, and a related
/// row of an entity `arranging` names.
// One argument per thing an arranging run is told, as [`drive`] takes them.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_sharing(
    ir: &EssIr,
    driver: &super::Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    (bound, known): (
        &BTreeMap<String, crate::scenario::InstanceName>,
        &[super::ScenarioStep],
    ),
    input: &BTreeMap<String, Node>,
    arranging: &[&EntityHandle],
) -> Result<(super::Invocation, Arrangement), RefusalCause> {
    // The aggregate arranges one related row itself; the rows beside it are not its to arrange
    // (beyond10x/ess#283).
    if several(driver.command) {
        return Err(unarranged());
    }
    let (via, entity) = read(driver.command).ok_or_else(unarranged)?;
    let names_subject = driver.outcome.subject.as_ref().is_some_and(|subject| {
        matches!(&subject.instance, ResolvedInstance::Supplied { field: named } if named.name == via.field())
    });
    if names_subject
        || bound.contains_key(via.field())
        || is_absent(driver.outcome)
        || arranging.contains(&entity)
    {
        return Err(unarranged());
    }
    searched(
        ir,
        driver,
        None,
        actors,
        distinction,
        (bound, known),
        Some(input),
        arranging,
    )
}

/// The tail of [`drive`]: the related row searched for and arranged, the run sent naming it, and
/// that row with its steps moved into the run's.
// One argument per thing an arranging run is told, as [`drive`] takes them.
#[allow(clippy::too_many_arguments)]
fn searched(
    ir: &EssIr,
    driver: &super::Driver<'_>,
    instance: Option<&crate::scenario::InstanceName>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    (bound, known): (
        &BTreeMap<String, crate::scenario::InstanceName>,
        &[super::ScenarioStep],
    ),
    input: Option<&BTreeMap<String, Node>>,
    arranging: &[&EntityHandle],
) -> Result<(super::Invocation, Arrangement), RefusalCause> {
    let (via, entity) = read(driver.command).ok_or_else(unarranged)?;
    let field = via.field();
    let pins = pins(ir, driver.command, driver.outcome, entity, instance, bound);
    let (mut row, _, input) = with_row(
        ir,
        driver.command,
        driver.outcome,
        entity,
        actors,
        (DRIVEN, distinction, arranging),
        input,
        None,
        None,
        (&pins, known),
    )?;
    let owners = subject_fact::bind_pinned(
        ir,
        driver.command,
        driver.outcome,
        entity,
        actors,
        &mut row,
        &input,
        &pins,
    )?;
    let mut bound = bound.clone();
    bound.extend(owners);
    bound.insert(field.to_owned(), row.instance.clone());
    let mut invocation = super::invoke_with(ir, driver, instance, actors, &bound, &input);
    let mut steps = std::mem::take(&mut row.steps);
    steps.append(&mut invocation.steps);
    invocation.steps = steps;
    invocation.source.extend(row.source.iter().cloned());
    Ok((invocation, row))
}

/// [`drive`] on a related row the caller arranged itself, holding values it chose — an aggregate
/// view's row copying its group key from the row its guard reads (beyond10x/ess#272): `input` sent
/// as it is but for the field the guard reads, which names `row`. Refused where that row, crossed
/// with `input`, does not select `driver`'s branch, rather than sent for a branch it does not reach.
/// The row's own steps are the caller's to run, before this one.
pub(super) fn drive_on(
    ir: &EssIr,
    driver: &super::Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    bound: &BTreeMap<String, crate::scenario::InstanceName>,
    row: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> Result<super::Invocation, RefusalCause> {
    if several(driver.command) {
        return Err(unarranged());
    }
    let (via, entity) = read(driver.command).ok_or_else(unarranged)?;
    let field = via.field();
    let selected = selects(ir, driver.command, entity, Some(row), input)?;
    if selected.is_none_or(|branch| branch.name != driver.outcome.name) {
        return Err(RefusalCause::GuardUnsatisfiable {
            predicate: format!(
                "the row of `{}` the arrangement named for `input.{field}` of `{}` does not \
                 select `{}`",
                entity.name(),
                driver.command.name,
                driver.outcome.name
            ),
            tried: 1,
        });
    }
    let mut bound = bound.clone();
    bound.insert(field.to_owned(), row.instance.clone());
    Ok(super::invoke_with(ir, driver, None, actors, &bound, input))
}

/// [`drive`] where the related row is of an entity `arranging` already names: one level deep, a
/// fresh row where its lifecycle starts, and the run sent only where that row selects its branch.
///
/// A row searched for along its lifecycle would be moved by the same commands that are arranging
/// the outer row, and could need a related row of its own again, without end; the initial state
/// needs no move. Inside that nested row's own arrangement the entity is named twice, and a
/// further related row of it is refused, naming why. Where the fresh row does not select the
/// branch — "accept only below an accepted parent" — the refusal says so: the row the guard needs
/// is one this bound does not arrange.
// One argument per thing an arranging run is told, as [`drive`] takes them.
#[allow(clippy::too_many_arguments)]
fn nested(
    ir: &EssIr,
    driver: &super::Driver<'_>,
    instance: Option<&crate::scenario::InstanceName>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    bound: &BTreeMap<String, crate::scenario::InstanceName>,
    chosen: Option<&BTreeMap<String, Node>>,
    arranging: &[&EntityHandle],
) -> Result<super::Invocation, RefusalCause> {
    let (via, entity) = read(driver.command).ok_or_else(unarranged)?;
    let field = via.field();
    let refused = |why: &str, tried: usize| RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "a related row of `{}` for `input.{field}` of `{}`, inside an arrangement of `{}`: \
             {why}",
            entity.name(),
            driver.command.name,
            entity.name()
        ),
        tried,
    };
    if arranging.iter().filter(|held| **held == entity).count() > 1 {
        return Err(refused(
            "already nested once; a related row of an entity being arranged is arranged one \
             level deep only",
            0,
        ));
    }
    let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
    let row = row_at(ir, entity, actors, block_start(NESTED, distinction), &chain)
        .ok_or_else(|| refused("no row of it can be created where its lifecycle starts", 0))?;
    let inputs = if let Some(chosen) = chosen {
        vec![chosen.clone()]
    } else {
        let predicates = predicates(driver.command);
        let grounded = subject_fact::grounded(ir, entity, &row.settled, &predicates);
        let mut searched: Vec<&Predicate> = driver
            .command
            .outcomes
            .iter()
            .filter_map(input_guard)
            .collect();
        searched.extend(grounded.iter());
        candidates(ir, driver.command, &searched, distinction).map_err(RefusalCause::NoWitness)?
    };
    let tried = inputs.len();
    let input = inputs
        .into_iter()
        .find(|input| {
            selects(ir, driver.command, entity, Some(&row), input)
                .ok()
                .flatten()
                .is_some_and(|branch| branch.name == driver.outcome.name)
        })
        .ok_or_else(|| {
            refused(
                &format!(
                    "a fresh row in state `{}` does not select `{}`, and a row in any other state \
                     would be moved by the arrangement it is nested in",
                    row.state, driver.outcome.name
                ),
                tried,
            )
        })?;
    let mut bound = bound.clone();
    bound.insert(field.to_owned(), row.instance.clone());
    let mut invocation = super::invoke_with(ir, driver, instance, actors, &bound, &input);
    let mut steps = row.steps;
    steps.append(&mut invocation.steps);
    invocation.steps = steps;
    invocation.source.extend(row.source);
    Ok(invocation)
}

/// [`drive`] with the Optional input reference the command's related guards read left out (ess/22,
/// beyond10x/ess#304), where that selects the branch: no row is read, so none is arranged. `None`
/// for a required reference, a related branch, and a branch the absent reference does not select —
/// with the caller's input as it is but for the reference, where the caller chose one.
fn without_reference(
    ir: &EssIr,
    driver: &super::Driver<'_>,
    instance: Option<&crate::scenario::InstanceName>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    bound: &BTreeMap<String, crate::scenario::InstanceName>,
    chosen: Option<&BTreeMap<String, Node>>,
) -> Option<super::Invocation> {
    let (ResolvedRelatedVia::Input { field, .. }, _) = read(driver.command)? else {
        return None;
    };
    if !optional(driver.command) {
        return None;
    }
    let input = match chosen {
        Some(chosen) => {
            let mut chosen = chosen.clone();
            chosen.remove(field);
            selects_absent(ir, driver.command, &chosen)
                .ok()
                .flatten()
                .is_some_and(|branch| branch.name == driver.outcome.name)
                .then_some(chosen)?
        }
        None => absent_input(ir, driver.command, driver.outcome, distinction).ok()?,
    };
    Some(super::invoke_with(
        ir, driver, instance, actors, bound, &input,
    ))
}

/// The link inputs over rows of `entity` ([`subject_fact::link_pins`]) the send of `outcome`
/// already names an arranged owner through: `bound`, and the field the branch names its subject
/// by, where `instance` is that subject — the one `super::supply` sends before anything `bound`
/// says (beyond10x/ess#271).
fn pins(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    instance: Option<&crate::scenario::InstanceName>,
    bound: &BTreeMap<String, crate::scenario::InstanceName>,
) -> BTreeMap<String, crate::scenario::InstanceName> {
    let mut held = bound.clone();
    if let (Some(subject), Some(instance)) = (&outcome.subject, instance) {
        if let ResolvedInstance::Supplied { field } = &subject.instance {
            held.insert(field.name.clone(), instance.clone());
        }
    }
    subject_fact::link_pins(ir, command, entity, &held)
}

/// A row of the related entity where its lifecycle starts, under the further witness `at`, inside
/// an arrangement of `arranging`.
fn row_at(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
    arranging: &[&EntityHandle],
) -> Option<Arrangement> {
    let initial = ir.entity(entity).lifecycle.initial.clone();
    arrange_first(
        ir,
        entity,
        std::slice::from_ref(&initial),
        actors,
        Distinction::further(at),
        arranging,
    )
    .ok()
}

/// The setup and input that reach `outcome` of a command reading a related row: the branch's own
/// subject arranged as for any branch, the related row (or its absence) arranged ahead of it with a
/// decoy either side, and the input pointed at the row — or at an identity no row carries.
///
/// Refused where no row and input the search tries select the branch, and where the branch's own
/// arrangement already binds the input `via` names, which then names two rows at once — but for
/// the row the creation is owned by, which is arranged once, as the related row
/// ([`owner_is_related`]).
/// Every further related row `outcome`'s own predicate is witnessed on, one per connective child:
/// for a conjunction, each conjunct refuted alone with every other held, where the command answers
/// with another branch; for a disjunction — the predicate's, or one among its conjuncts — each
/// disjunct held alone with every other refuted and the remaining conjuncts held, where it answers
/// with this branch. One row on each side of the whole predicate shows only that *some* child is
/// read; these show each one is (the rule `when_subject` follows, beyond10x/ess#155 and #204).
///
/// After them, the far side of every counter limit the predicate compares a stored counter of the
/// related row with (beyond10x/ess#226, [`subject_fact::limit_goals`]): the nearest row a run holds
/// short of the limit, answered by whichever branch the command takes there. The branch's own
/// witness is the first row the search reaches that holds the predicate, which for a counter moved
/// toward its limit is the nearest row at or past it; so a target whose limit is off by one either
/// way fails. Each goal is paired with what it is ([`subject_fact::Further`]).
pub(super) fn boundary_goals(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<(Goal, subject_fact::Further)> {
    let ResolvedCondition::Related {
        test: ResolvedRelatedTest::Holds { predicate },
        ..
    } = &outcome.condition
    else {
        return Vec::new();
    };
    let conjuncts: Vec<&Predicate> = match predicate {
        Predicate::All(children) => children.iter().collect(),
        other => vec![other],
    };
    let mut goals = Vec::new();
    if conjuncts.len() >= 2 {
        goals.extend(isolating(&conjuncts, false));
    }
    for (at, conjunct) in conjuncts.iter().enumerate() {
        let Predicate::Any(disjuncts) = conjunct else {
            continue;
        };
        let rest: Vec<Predicate> = conjuncts
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != at)
            .map(|(_, other)| (*other).clone())
            .collect();
        if disjuncts.len() < 2 {
            continue;
        }
        for (refuted, mut held) in isolating(&disjuncts.iter().collect::<Vec<_>>(), true) {
            held.extend(rest.iter().cloned());
            goals.push((refuted, held));
        }
    }
    let mut goals: Vec<(Goal, subject_fact::Further)> = goals
        .into_iter()
        .map(|goal| (goal, subject_fact::Further::Plain))
        .collect();
    if let Some((_, entity)) = focus(ir, command, outcome) {
        let counters = subject_fact::counters(ir, entity);
        for (goal, kind) in subject_fact::limit_goals(&counters, predicate, false) {
            if !goals.iter().any(|(known, _)| known == &goal) {
                goals.push((goal, kind));
            }
        }
    }
    goals
}

/// A boundary no arrangement reached, as a refusal states it.
pub(super) fn describe_goal(refuted: &[Predicate], held: &[Predicate]) -> String {
    let list = |predicates: &[Predicate]| {
        predicates
            .iter()
            .map(|predicate| format!("`{predicate}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "a related row on which {} is false and {} is true",
        list(refuted),
        if held.is_empty() {
            "nothing else".to_owned()
        } else {
            list(held)
        }
    )
}

/// One goal per child: that child `alone` (true for a disjunct, false for a conjunct), every other
/// the opposite.
fn isolating(children: &[&Predicate], alone: bool) -> Vec<Goal> {
    (0..children.len())
        .map(|index| {
            let (mut falses, mut trues) = (Vec::new(), Vec::new());
            for (other, child) in children.iter().enumerate() {
                if (other == index) == alone {
                    trues.push((*child).clone());
                } else {
                    falses.push((*child).clone());
                }
            }
            (falses, trues)
        })
        .collect()
}

/// [`prepare_at_in`] for a further witness: the subject arranged at `distinction`, the related row in the
/// scenario's own block at that witness's place, and — where `goal` names one — the row searched
/// for until every predicate of the goal reads as it says, crossed with the input sent.
pub(super) fn prepare_at_in(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    goal: Option<&Goal>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    arranged(models, command, outcome, actors, distinction, goal)
        .map(|(setup, input, _)| (setup, input))
}

/// [`arranged_at`], on a command reading several related rows (beyond10x/ess#283) around the row
/// `outcome`'s own guard reads — or the one `goal` is a boundary of — with every other row
// arranged beside it so that nothing answering before `outcome` is selected there by the input sent
/// ([`Around`]), and, for a refusal over a row, the overlaps the precedence order decides sent first
/// ([`overlaps`]).
fn arranged(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    goal: Option<&Goal>,
) -> Result<(Setup, BTreeMap<String, Node>, bool), RefusalCause> {
    if !several(command) {
        return arranged_at(models, command, outcome, actors, distinction, goal, None);
    }
    let ir = models.arrangement;
    let (via, _) = match goal {
        Some(goal) => goal_row(ir, command, goal),
        None => focus(ir, command, outcome),
    }
    .ok_or_else(unarranged)?;
    let projected = projection(command, via.field());
    let own = kept(&projected, outcome)?;
    let others = beside(
        ir,
        (command, outcome),
        via.field(),
        actors,
        distinction,
        (BESIDE, false),
    )?;
    let around = Around {
        command,
        outcome,
        rows: &others,
    };
    let (mut setup, input, lonely) = arranged_at(
        models,
        &projected,
        own,
        actors,
        distinction,
        goal,
        Some(around),
    )?;
    for other in &others {
        if !passes(ir, (command, outcome), other, &input)? {
            return Err(RefusalCause::GuardUnsatisfiable {
                predicate: format!(
                    "a row of `{}` for `{}` of `{}` on which no branch over it is selected by the \
                     input that selects `{}`",
                    other.entity.name(),
                    other.via,
                    command.name,
                    outcome.name
                ),
                tried: 1,
            });
        }
    }
    if goal.is_none() {
        overlaps(
            models,
            command,
            outcome,
            actors,
            (distinction, via.field()),
            &mut setup,
            &input,
        )?;
    }
    Ok((setup, input, lonely))
}

/// Whether `outcome`'s own scenario ([`prepare_at_in`]) copies a field from the row its guard reads and
/// arranges no second row the guard accepts holding another value there ([`companion`],
/// beyond10x/ess#270): a target copying from "a row the guard accepts" rather than the row named
/// is then not failed by it. The fields copied, where it is; else the copied fields whose type has
/// one value, which no second row can hold otherwise (beyond10x/ess#287); none for a branch copying
/// nothing from that row, without arranging anything.
pub(super) fn unaccompanied(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Vec<String> {
    let Some((via, entity)) = focus(ir, command, outcome) else {
        return Vec::new();
    };
    let copied = super::related::guarded_fields(ir, outcome, (via.field(), entity));
    if !routes(command, outcome) || is_absent(outcome) || copied.is_empty() {
        return Vec::new();
    }
    let lonely = arranged(
        &super::caller::InvocationModels::plain(ir),
        command,
        outcome,
        actors,
        Distinction::PLAIN,
        None,
    )
    .is_ok_and(|(_, _, lonely)| lonely);
    if lonely {
        return copied.into_iter().map(str::to_owned).collect();
    }
    // A field whose type has one value — the one row of a singleton entity a row references — is
    // held alike by every row, so no companion holds another value there (beyond10x/ess#287).
    let related = ir.entity(entity);
    copied
        .into_iter()
        .filter(|copied| {
            related
                .fields
                .iter()
                .chain([&related.identity])
                .find(|field| field.name == *copied)
                .is_some_and(|field| super::singleton::has_one_value(ir, &field.type_ref))
        })
        .map(str::to_owned)
        .collect()
}

/// Whether the command's related guards read through an Optional input (ess/22, beyond10x/ess#304),
/// so a request may leave the reference absent.
pub(super) fn optional(command: &ResolvedCommand) -> bool {
    // On a command reading several rows, every reference may be left absent only where each is
    // Optional (beyond10x/ess#283).
    let rows = rows(command);
    !rows.is_empty() && rows.iter().all(|(via, _)| via.type_ref().is_optional())
}

/// Which branch the command selects for this input with the Optional reference absent, if exactly
/// one does: no row is read and no `when_related` branch is selected, so only the branches that read
/// no related row answer — the first input-guarded refusal whose guard holds, else the one accepting
/// branch whose guard holds, else the default. Held state and the command's own identity are
/// answered before it, as for a present reference, and are not asked here.
fn selects_absent<'c>(
    ir: &EssIr,
    command: &'c ResolvedCommand,
    input: &BTreeMap<String, Node>,
) -> Result<Option<&'c ResolvedOutcome>, RefusalCause> {
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    let mut selected = Vec::new();
    for branch in command.outcomes.iter().filter(|branch| {
        !state_default(branch)
            && !matches!(
                branch.condition,
                ResolvedCondition::External { .. }
                    | ResolvedCondition::ExternalWhen { .. }
                    | ResolvedCondition::ExistingInstance
                    | ResolvedCondition::WrongState
                    | ResolvedCondition::Related { .. }
            )
    }) {
        if let Some(guard) = input_guard(branch) {
            if !decides(&facts, &[guard], true)? {
                continue;
            }
        }
        selected.push(branch);
    }
    if let Some(first) = selected
        .iter()
        .copied()
        .find(|branch| super::is_input_guarded_refusal(branch))
    {
        selected = vec![first];
    }
    Ok(match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    })
}

/// An input that reaches `outcome` with the Optional reference the command's related guards read
/// left out (ess/22, beyond10x/ess#304). Refused for a command whose reference is required, and
/// where no candidate input reaches the branch without it.
pub(super) fn absent_input(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    read(command)
        .filter(|(via, _)| optional(command) || matches!(via, ResolvedRelatedVia::Subject { .. }))
        .ok_or_else(unarranged)?;
    if matches!(outcome.condition, ResolvedCondition::Related { .. }) {
        return Err(unarranged());
    }
    // Every input guard of the command, as [`with_row`] searches with: an input-guarded refusal
    // beside the branch must be refuted by the input sent, which a search over the branch's own
    // guard alone does not look for (beyond10x/ess#304 adversary pass 1).
    let guards: Vec<&Predicate> = command.outcomes.iter().filter_map(input_guard).collect();
    for mut input in
        candidates(ir, command, &guards, distinction).map_err(RefusalCause::NoWitness)?
    {
        // Every reference left out: on a command reading several rows, each is Optional
        // ([`optional`], beyond10x/ess#283).
        for (via, _) in rows(command) {
            input.remove(via.field());
        }
        if selects_absent(ir, command, &input)?.is_some_and(|branch| branch.name == outcome.name) {
            return Ok(input);
        }
    }
    Err(unarranged())
}

/// Whether `outcome` is a branch an absent Optional reference selects (ess/22, beyond10x/ess#304):
/// its scenario then carries the absent witness ([`prepare_absent_in`]).
pub(super) fn absent_selects(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> bool {
    if several(command) {
        return routes(command, outcome) && absent_row(ir, command, outcome).is_some();
    }
    routes(command, outcome)
        && optional(command)
        && absent_input(ir, command, outcome, Distinction::PLAIN).is_ok()
}

/// On a command reading several rows (beyond10x/ess#283), the first Optional reference whose
/// absence — every other row arranged beside — selects `outcome`: the row its absent witness leaves
/// out.
fn absent_row<'c>(
    ir: &EssIr,
    command: &'c ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Option<&'c ResolvedRelatedVia> {
    rows(command)
        .into_iter()
        .filter(|(via, _)| via.type_ref().is_optional())
        .find(|(via, _)| {
            let projected = projection(command, via.field());
            kept(&projected, outcome)
                .is_ok_and(|own| absent_input(ir, &projected, own, Distinction::PLAIN).is_ok())
        })
        .map(|(via, _)| via)
}

/// The setup and input of the absent witness for `outcome` (ess/22, beyond10x/ess#304): the
/// branch's own subject arranged at `distinction`, between two related rows each selecting a
/// present-related refusal where one is found, and the command sent with the Optional reference
/// left out. A target that treats absence as a missing row answers `exists: false`; one that reads
/// some row of the entity answers a refusal; neither answers `outcome`.
pub(super) fn prepare_absent_in(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let ir = models.arrangement;
    if stored::field(command).is_some() {
        return stored::absent_at(models, command, outcome, actors, distinction);
    }
    // Several rows (beyond10x/ess#283): the first Optional reference whose absence selects the
    // branch is left out, every other row arranged beside it.
    if several(command) {
        let via = absent_row(ir, command, outcome).ok_or_else(unarranged)?;
        let projected = projection(command, via.field());
        let own = kept(&projected, outcome)?;
        let others = beside(
            ir,
            (command, outcome),
            via.field(),
            actors,
            distinction,
            (BESIDE, false),
        )?;
        let (setup, input) = absent_at(models, &projected, own, actors, distinction, &others)?;
        for other in &others {
            if !passes(ir, (command, outcome), other, &input)? {
                return Err(unarranged());
            }
        }
        return Ok((setup, input));
    }
    absent_at(models, command, outcome, actors, distinction, &[])
}

/// The tail of [`prepare_absent_in`], with the rows `others` names arranged beside the reference
/// left out ([`beside`]).
fn absent_at(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    others: &[Beside<'_>],
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let ir = models.arrangement;
    let (via, entity) = read(command).ok_or_else(unarranged)?;
    let field = via.field();
    let input = absent_input(ir, command, outcome, distinction)?;
    let mut setup = own_arrangement(
        ir,
        command,
        outcome,
        actors,
        distinction,
        (field, entity),
        others,
    )?;
    if setup.bound.contains_key(field) {
        return Err(unarranged());
    }
    place_beside(ir, outcome, &mut setup, others);
    let mut steps = Vec::new();
    if !super::singleton::is_singleton(ir, entity) {
        let refusing = |node: &Arrangement| {
            selects(ir, command, entity, Some(node), &input).map(|branch| {
                branch.is_some_and(|branch| {
                    branch.error.is_some()
                        && matches!(
                            branch.condition,
                            ResolvedCondition::Related {
                                test: ResolvedRelatedTest::Holds { .. },
                                ..
                            }
                        )
                })
            })
        };
        let first = block_start(OWN, distinction);
        for at in [first - 1, first + 1] {
            let decoy = search_rows(
                ir,
                entity,
                actors,
                (at, None),
                &predicates(command),
                refusing,
            )
            .or_else(|| row_at(ir, entity, actors, at, &[]));
            if let Some(decoy) = decoy {
                steps.extend(decoy.steps);
                setup.source.extend(decoy.source);
            }
        }
    }
    std::mem::swap(&mut steps, &mut setup.steps);
    setup.steps.append(&mut steps);
    setup.source.insert(entity_ref(entity));
    prepend_beside(&mut setup, others);
    Ok((setup, input))
}

/// [`prepare_at_in`], and whether the scenario is [`unaccompanied`].
fn arranged_at(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    goal: Option<&Goal>,
    around: Option<Around<'_>>,
) -> Result<(Setup, BTreeMap<String, Node>, bool), RefusalCause> {
    let ir = models.arrangement;
    if stored::field(command).is_some() {
        return stored::arranged_at(models, command, outcome, actors, distinction, goal);
    }
    let others = around.map_or(&[][..], |around| around.rows);
    let (via, entity) = read(command).ok_or_else(unarranged)?;
    let field = via.field();
    // A missing row reads no predicate, so no boundary of one is witnessed on it.
    if goal.is_some() && is_absent(outcome) {
        return Err(unarranged());
    }
    let mut setup = own_arrangement(
        ir,
        command,
        outcome,
        actors,
        distinction,
        (field, entity),
        others,
    )?;
    let names_subject = outcome.subject.as_ref().is_some_and(|subject| {
        matches!(&subject.instance, ResolvedInstance::Supplied { field: named } if named.name == field)
    });
    if names_subject || setup.bound.contains_key(field) {
        return Err(unarranged());
    }
    place_beside(ir, outcome, &mut setup, others);
    let mut steps = Vec::new();
    let mut pinned = BTreeMap::new();
    let mut lonely = false;
    let input = if is_absent(outcome) {
        let mut each = without_row_each(ir, command, entity, field, distinction)?.into_iter();
        let input = each.next().ok_or_else(unarranged)?;
        // Rows of the entity exist, each carrying an identity other than the one sent — but for a
        // singleton entity, which has no identity but the one (beyond10x/ess#287).
        let first = block_start(OWN, distinction);
        let decoys = if super::singleton::is_singleton(ir, entity) {
            Vec::new()
        } else {
            vec![first - 1, first + 1]
        };
        for at in decoys {
            if let Some(decoy) = row_at(ir, entity, actors, at, &[]) {
                steps.extend(decoy.steps);
                setup.source.extend(decoy.source);
            }
        }
        // Every other input guard's input is sent for the missing row too, before the scenario's
        // own send: each is answered by this refusal, so a target answering that guard's branch
        // before reading the row fails. Only for a refusal, which changes nothing, so the sends
        // leave the row missing for the next.
        models.mark(super::caller::InvocationPhase::Arrange, &mut steps);
        send_each_without_row(ir, command, outcome, actors, &setup.bound, each, &mut steps);
        models.mark(super::caller::InvocationPhase::Act, &mut steps);
        input
    } else {
        // The owners the branch's own arrangement already names for a link input — its subject,
        // or the subject's owner — are the ones it is sent naming: the related row is arranged
        // against them, and never against a second owner the send would not name (#271).
        pinned = pins(
            ir,
            command,
            outcome,
            entity,
            setup.instance.as_ref(),
            &setup.bound,
        );
        let (mut row, first, input) = with_row(
            ir,
            command,
            outcome,
            entity,
            actors,
            (OWN, distinction, &[]),
            None,
            goal,
            around,
            (&pinned, &setup.steps),
        )?;
        let owners = subject_fact::bind_pinned(
            ir, command, outcome, entity, actors, &mut row, &input, &pinned,
        )?;
        setup.bound.extend(owners);
        lonely = surround(
            ir,
            (command, outcome, entity),
            actors,
            (first, &input),
            (
                &row,
                &super::related::guarded_fields(ir, outcome, (field, entity)),
            ),
            (&mut steps, &mut setup.source),
        );
        observe_read(ir, command, entity, &row, &mut steps, &mut setup.source);
        setup.bound.insert(field.to_owned(), row.instance.clone());
        super::related::settle(ir, outcome, &mut setup, (field, entity), &row);
        input
    };
    // The related rows go ahead of the branch's own arrangement, but for rows filed under an owner
    // that arrangement brings into being (`pinned`): those follow it.
    if pinned.is_empty() {
        std::mem::swap(&mut steps, &mut setup.steps);
    }
    setup.steps.append(&mut steps);
    setup.source.insert(entity_ref(entity));
    prepend_beside(&mut setup, others);
    Ok((setup, input, lonely))
}

/// The branch's own arrangement, ahead of the related row [`prepare_at_in`] arranges for its guard.
///
/// A `{related: …}` value read through the input the guard reads names the guard's own row
/// (`guarded`): it is left out here, arranged once by [`prepare_at_in`], and its values carried from
/// it (beyond10x/ess#270).
fn own_arrangement(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    guarded: super::related::Guarded<'_>,
    others: &[Beside<'_>],
) -> Result<Setup, RefusalCause> {
    // The rows arranged beside the guarded one (beyond10x/ess#283) are the guards' to arrange too.
    let each: Vec<super::related::Guarded<'_>> = std::iter::once(guarded)
        .chain(others.iter().map(|other| (other.via.field(), other.entity)))
        .collect();
    let except = |setup: Setup| {
        if others.is_empty() {
            super::related::arrange_except(ir, outcome, actors, distinction, setup, Some(guarded))
        } else {
            super::related::arrange_except_each(ir, outcome, actors, distinction, setup, &each)
        }
    };
    let setup = match (&outcome.subject, acting(command)) {
        // A branch naming no subject of its own, beside one acting on an existing row, is sent for
        // a row that branch could act on: the related row is then the only thing that refuses it.
        (None, Some((sibling, named))) => {
            let arranged = prepare_in(ir, sibling, actors, None, distinction)?;
            let instance = arranged.instance.ok_or_else(unarranged)?;
            Setup {
                steps: arranged.steps,
                source: arranged.source,
                bound: BTreeMap::from([(named.to_owned(), instance)]),
                ..Setup::none()
            }
        }
        // A creation owned by the related row: the row is arranged below, once, as the related
        // row, and the owner field points at it — or, on the `exists: false` branch, at none.
        (Some(subject), _) if owner_is_related(ir, command, outcome) => {
            let born = subject
                .into
                .as_ref()
                .unwrap_or(&ir.entity(&subject.entity).lifecycle.initial);
            except(Setup {
                after: Some(born.clone()),
                ..Setup::none()
            })?
        }
        _ => {
            let setup = super::prepare_subject(ir, outcome, actors, None, distinction)?;
            except(setup)?
        }
    };
    Ok(setup)
}

/// The rows `others` names, bound to the inputs naming them and their values the branch copies
/// settled ([`super::related::settle`]), before anything the scenario sends is built.
fn place_beside(ir: &EssIr, outcome: &ResolvedOutcome, setup: &mut Setup, others: &[Beside<'_>]) {
    for other in others {
        setup
            .bound
            .insert(other.via.field().to_owned(), other.row.instance.clone());
        super::related::settle(
            ir,
            outcome,
            setup,
            (other.via.field(), other.entity),
            &other.row,
        );
    }
}

/// The steps creating the rows `others` names, ahead of everything else the scenario arranges.
fn prepend_beside(setup: &mut Setup, others: &[Beside<'_>]) {
    if others.is_empty() {
        return;
    }
    let mut steps = Vec::new();
    for other in others {
        steps.extend(other.row.steps.iter().cloned());
        setup.source.extend(other.row.source.iter().cloned());
        setup.source.insert(entity_ref(other.entity));
    }
    steps.append(&mut setup.steps);
    setup.steps = steps;
}

/// The overlaps the precedence order decides for a refusal over one of several related rows
/// (beyond10x/ess#283), each sent before the scenario's own send and answered by `outcome`, so a
/// target reading the rows in another order fails it. For every other row:
///
/// * where `outcome` is an `exists: false` branch declared before that row's, the row missing too:
///   the first declared missing row answers;
/// * a row selecting one of its own branches `outcome` still answers before — for `exists: false`,
///   any of them, since a missing row answers before every predicate; for a predicate refusal, an
///   accepting branch or a refusal declared after it.
///
/// Only for a refusal, which changes nothing, so every send leaves the scenario as it was. A row
/// no bounded arrangement selects that way with the scenario's input adds no send.
fn overlaps(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (distinction, focus): (Distinction, &str),
    setup: &mut Setup,
    input: &BTreeMap<String, Node>,
) -> Result<(), RefusalCause> {
    let ir = models.arrangement;
    if outcome.error.is_none() {
        return Ok(());
    }
    let ResolvedCondition::Related { test, .. } = &outcome.condition else {
        return Ok(());
    };
    let missing = matches!(test, ResolvedRelatedTest::Absent);
    let declared = |branch: &ResolvedOutcome| {
        command
            .outcomes
            .iter()
            .position(|declared| declared.name == branch.name)
    };
    let mut steps = Vec::new();
    let mut sends: Vec<(
        BTreeMap<String, Node>,
        BTreeMap<String, crate::scenario::InstanceName>,
    )> = Vec::new();
    for (index, (via, entity)) in rows(command)
        .into_iter()
        .filter(|(via, _)| via.field() != focus)
        .enumerate()
    {
        let over = |branch: &&ResolvedOutcome| {
            matches!(&branch.condition, ResolvedCondition::Related { via: read, .. }
                if read.field() == via.field())
        };
        let later_absent = command
            .outcomes
            .iter()
            .filter(over)
            .find(|branch| is_absent(branch))
            .is_some_and(|branch| declared(branch) > declared(outcome));
        if missing && later_absent {
            let mut sent = input.clone();
            sent.insert(
                via.field().to_owned(),
                fresh_identity(ir, command, via.field(), Some(input))?,
            );
            let mut bound = setup.bound.clone();
            bound.remove(via.field());
            sends.push((sent, bound));
        }
        let projected = projection(command, via.field());
        let answered_after: Vec<&ResolvedOutcome> = command
            .outcomes
            .iter()
            .filter(over)
            .filter(|branch| !is_absent(branch))
            .filter(|branch| {
                missing || branch.error.is_none() || declared(branch) > declared(outcome)
            })
            .collect();
        let at = block_start(OVERLAP + index.min(3), distinction);
        let predicates = predicates_over(command, via.field());
        let found = answered_after.iter().find_map(|selected| {
            search_rows(ir, entity, actors, (at, None), &predicates, |node| {
                Ok(selects(ir, &projected, entity, Some(node), input)?
                    .is_some_and(|branch| branch.name == selected.name))
            })
        });
        if let Some(row) = found {
            steps.extend(row.steps);
            setup.source.extend(row.source);
            let mut bound = setup.bound.clone();
            bound.insert(via.field().to_owned(), row.instance);
            sends.push((input.clone(), bound));
        }
    }
    if missing {
        sends.extend(earlier_absent(
            command,
            outcome,
            focus,
            (input, &setup.bound),
        ));
    }
    if sends.is_empty() {
        return Ok(());
    }
    models.mark(super::caller::InvocationPhase::Arrange, &mut steps);
    for (sent, bound) in sends {
        send_each_without_row(
            ir,
            command,
            outcome,
            actors,
            &bound,
            std::iter::once(sent),
            &mut steps,
        );
    }
    models.mark(super::caller::InvocationPhase::Act, &mut steps);
    setup.steps.append(&mut steps);
    Ok(())
}

/// The send of the `exists: false` branch `outcome` over the row `focus` names with every Optional
/// reference whose `exists: false` is declared earlier left out (beyond10x/ess#283): an absent
/// reference reads no row, and the read carries on past it to the missing one. `None` where no
/// such reference is declared.
fn earlier_absent(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    focus: &str,
    (input, bound): (
        &BTreeMap<String, Node>,
        &BTreeMap<String, crate::scenario::InstanceName>,
    ),
) -> Option<(
    BTreeMap<String, Node>,
    BTreeMap<String, crate::scenario::InstanceName>,
)> {
    let declared = |of: &ResolvedOutcome| {
        command
            .outcomes
            .iter()
            .position(|declared| declared.name == of.name)
    };
    let earlier: Vec<&str> = rows(command)
        .into_iter()
        .filter(|(via, _)| via.field() != focus && via.type_ref().is_optional())
        .filter(|(via, _)| {
            command
                .outcomes
                .iter()
                .find(|branch| {
                    is_absent(branch)
                        && matches!(&branch.condition, ResolvedCondition::Related { via: read, .. }
                            if read.field() == via.field())
                })
                .is_some_and(|branch| declared(branch) < declared(outcome))
        })
        .map(|(via, _)| via.field())
        .collect();
    if earlier.is_empty() {
        return None;
    }
    let mut sent = input.clone();
    let mut bound = bound.clone();
    for field in earlier {
        sent.remove(field);
        bound.remove(field);
    }
    Some((sent, bound))
}

/// The row's fields the guards read, observed before the command where a view shows them: the row
/// is a fact the scenario is about, not one it assumes.
fn observe_read(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    row: &Arrangement,
    steps: &mut Vec<super::ScenarioStep>,
    source: &mut BTreeSet<crate::scenario::EssSemanticRef>,
) {
    let read_fields: BTreeSet<String> = predicates(command)
        .iter()
        .flat_map(|predicate| subject_fact::read_by(ir, entity, predicate))
        .collect();
    if let Ok((observed, view)) = subject_fact::observe_fields(ir, entity, &read_fields, row) {
        steps.extend(observed);
        source.insert(view.into());
    }
}

/// The named row `row` between its decoys, at `first` in its block, pushed onto `steps`: a decoy
/// one before it and one after, each selecting another branch where one is found — and, where the
/// branch copies fields from the named row (`copied`, beyond10x/ess#270), a companion the guard
/// accepts too created before all of them and another after all of them ([`companion`]), so the
/// named row is neither the first nor the last row the guard accepts. Whether the branch copies a
/// field and neither companion was found.
fn surround(
    ir: &EssIr,
    (command, outcome, entity): (&ResolvedCommand, &ResolvedOutcome, &EntityHandle),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (first, input): (usize, &BTreeMap<String, Node>),
    (row, copied): (&Arrangement, &[&str]),
    (steps, source): (
        &mut Vec<super::ScenarioStep>,
        &mut BTreeSet<crate::scenario::EssSemanticRef>,
    ),
) -> bool {
    // A singleton entity has no row beside the named one, so it is alone (beyond10x/ess#287).
    if super::singleton::is_singleton(ir, entity) {
        steps.extend(row.steps.iter().cloned());
        source.extend(row.source.iter().cloned());
        return false;
    }
    // The companions' witnesses lie in a block of their own, one either side of its start: no
    // other row's, and one away from a multiple of `BLOCK`, so an enum reads another variant there.
    let base = first + BLOCK * (COMPANION - OWN);
    let branch = (command, outcome, entity);
    let [before, after] = [(base - 1, Side::Below), (base + 1, Side::Above)]
        .map(|(at, side)| companion(ir, branch, actors, at, input, (row, copied), side));
    let lonely = !copied.is_empty() && before.is_none() && after.is_none();
    let owner = super::owner_of_row(ir, entity, &row.settled).map(|(_, owner)| owner.clone());
    let mut named = row.steps.clone();
    if let Some(before) = before {
        // A companion filed under the named row's owner follows the step creating that owner.
        let captured = owner.as_ref().and_then(|owner| {
            named.iter().position(|step| {
                matches!(step, super::ScenarioStep::CaptureInstance { instance, .. }
                    if instance == owner)
            })
        });
        match captured {
            Some(at) => {
                let after = at + 1;
                named.splice(after..after, before.steps);
            }
            None => steps.extend(before.steps),
        }
        source.extend(before.source);
    }
    for at in [first - 1, first + 1] {
        if at == first + 1 {
            steps.append(&mut named);
            source.extend(row.source.iter().cloned());
        }
        if let Some(decoy) = decoy(ir, branch, actors, at, input, (row, copied)) {
            steps.extend(decoy.steps);
            source.extend(decoy.source);
        }
    }
    if let Some(after) = after {
        steps.extend(after.steps);
        source.extend(after.source);
    }
    lonely
}

/// A row beside the one the input names, under the further witness `at`: one on which the same
/// input selects another branch, where the stored-row search finds one, so an implementation
/// reading it in place of the named row answers otherwise — else the plain row there.
///
/// `copied` are the fields the branch copies from the named row (`{related: …}` through the
/// guard's input, beyond10x/ess#270): a row that also holds another value in each is searched for
/// first, so an implementation copying from it publishes another value.
fn decoy(
    ir: &EssIr,
    (command, outcome, entity): (&ResolvedCommand, &ResolvedOutcome, &EntityHandle),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
    input: &BTreeMap<String, Node>,
    (named, copied): (&Arrangement, &[&str]),
) -> Option<Arrangement> {
    let predicates = predicates(command);
    let other = |node: &Arrangement| {
        selects(ir, command, entity, Some(node), input)
            .map(|branch| branch.is_some_and(|branch| branch.name != outcome.name))
    };
    let apart = (!copied.is_empty())
        .then(|| {
            let steers = plain_steers(ir, entity, actors, at, named, copied);
            search_rows(
                ir,
                entity,
                actors,
                (at, None),
                &steered(&predicates, &steers),
                |node| Ok(other(node)? && copied.iter().all(|f| differs(node, named, f))),
            )
        })
        .flatten();
    apart
        .or_else(|| search_rows(ir, entity, actors, (at, None), &predicates, other))
        .or_else(|| row_at(ir, entity, actors, at, &[]))
}

/// The side of the named row's value a companion's copied values are sought on.
#[derive(Debug, Clone, Copy)]
enum Side {
    Below,
    Above,
}

/// A value next to `value` on `side` of it, for a value with an order: a text one character
/// shorter (a prefix sorts first) or with its last character repeated, a number one less or one
/// more. `None` for a value with no order to be on a side of — an enum variant, a flag — or no
/// next value.
fn beyond(value: &Node, side: Side) -> Option<Node> {
    match (value, side) {
        (Node::Text(text), Side::Below) => {
            let mut shorter = text.clone();
            shorter.pop()?;
            Some(Node::Text(shorter))
        }
        (Node::Text(text), Side::Above) => {
            let last = text.chars().last()?;
            Some(Node::Text(format!("{text}{last}")))
        }
        (Node::Number(number), side) => {
            let step = ess_primitives::facts::Number::decimal_literal(match side {
                Side::Below => "-1",
                Side::Above => "1",
            })?;
            number.checked_add(step).map(Node::Number)
        }
        _ => None,
    }
}

/// A second row the guard accepts beside the one the input names, under the further witness `at`
/// (beyond10x/ess#270): one on which the same input selects `outcome` too, holding another value
/// than the named row in the fields the branch copies from it (`copied`), so an implementation
/// copying from a row the guard accepts other than the one named publishes another value.
///
/// Searched for in three passes, each first under the named row's owner, where it has one — a row
/// filed elsewhere would answer a guard comparing the row's owner otherwise — then anywhere: every
/// copied field holding the value next to the named row's on `side` of it ([`beyond`]) where the
/// field is ordered, and another value where not, so the named row's value lies strictly between
/// two companions'; then every copied field holding another value; then at least one — a field
/// the guard pins to one value differs in no row it accepts. `None` where the branch copies
/// nothing, or no pass finds a row.
fn companion(
    ir: &EssIr,
    (command, outcome, entity): (&ResolvedCommand, &ResolvedOutcome, &EntityHandle),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
    input: &BTreeMap<String, Node>,
    (named, copied): (&Arrangement, &[&str]),
    side: Side,
) -> Option<Arrangement> {
    if copied.is_empty() {
        return None;
    }
    let predicates = predicates(command);
    let plain = plain_steers(ir, entity, actors, at, named, copied);
    let targets: Vec<(&str, Option<Node>)> = copied
        .iter()
        .map(|field| {
            let target = literal(named, field).and_then(|value| beyond(&value, side));
            (*field, target)
        })
        .collect();
    let ordered: Vec<(&str, Node)> = targets
        .iter()
        .filter_map(|(field, target)| {
            target
                .clone()
                .or_else(|| {
                    plain
                        .iter()
                        .find(|(held, _)| held == field)
                        .map(|(_, v)| v.clone())
                })
                .map(|value| (*field, value))
        })
        .collect();
    let on_side = |node: &Arrangement| {
        targets.iter().all(|(field, target)| match target {
            Some(target) => literal(node, field).as_ref() == Some(target),
            None => differs(node, named, field),
        })
    };
    let every = |node: &Arrangement| copied.iter().all(|field| differs(node, named, field));
    let any = |node: &Arrangement| copied.iter().any(|field| differs(node, named, field));
    let passes: [Pass<'_>; 3] = [(&ordered, &on_side), (&plain, &every), (&plain, &any)];
    let owner = super::owner_of_row(ir, entity, &named.settled).map(|(_, owner)| owner);
    let unders: Vec<subject_fact::Under<'_>> = owner
        .into_iter()
        .map(|owner| Some((owner, false)))
        .chain([None])
        .collect();
    for (steers, accepts) in passes {
        let hints = steered(&predicates, steers);
        for under in &unders {
            let found = search_rows(ir, entity, actors, (at, *under), &hints, |node| {
                Ok(selects(ir, command, entity, Some(node), input)?
                    .is_some_and(|branch| branch.name == outcome.name)
                    && accepts(node))
            });
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

/// One pass of the [`companion`] search: the values its rows are steered toward, and what it takes.
type Pass<'a> = (&'a [(&'a str, Node)], &'a dyn Fn(&Arrangement) -> bool);

/// What `row` holds in `field`, where it is a literal.
fn literal(row: &Arrangement, field: &str) -> Option<Node> {
    match &row.settled.get(field)?.value {
        crate::scenario::ScenarioValue::Literal { value } => Some(value.clone()),
        _ => None,
    }
}

/// Whether `node` holds another value in `field` than `named`.
fn differs(node: &Arrangement, named: &Arrangement, field: &str) -> bool {
    let held = |row: &Arrangement| row.settled.get(field).map(|held| held.value.clone());
    held(node) != held(named)
}

/// The value the plain row at the further witness `at` holds in each of `copied` where it differs
/// from what `named` holds there: the value a row apart from the named one is steered toward.
fn plain_steers<'f>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
    named: &Arrangement,
    copied: &[&'f str],
) -> Vec<(&'f str, Node)> {
    let Some(plain) = row_at(ir, entity, actors, at, &[]) else {
        return Vec::new();
    };
    copied
        .iter()
        .filter_map(|field| {
            let other = literal(&plain, field)?;
            (Some(&other) != literal(named, field).as_ref()).then_some((*field, other))
        })
        .collect()
}

/// The command's predicates, then each side of every predicate joined with each field holding its
/// value in `steers`, then those values alone: the stored-row search tries the literals its hints
/// name, so a row on either side of the guard is tried with them.
fn steered(predicates: &[Predicate], steers: &[(&str, Node)]) -> Vec<Predicate> {
    let steers: Vec<Predicate> = steers
        .iter()
        .filter_map(|(field, value)| {
            Some(Predicate::AnyOf {
                path: FactPath::new(field).ok()?,
                values: vec![super::fact_value(value)?],
            })
        })
        .collect();
    let mut hints = predicates.to_vec();
    if steers.is_empty() {
        return hints;
    }
    for predicate in predicates {
        for side in [
            predicate.clone(),
            Predicate::Not(Box::new(predicate.clone())),
        ] {
            hints.push(Predicate::All(
                std::iter::once(side)
                    .chain(steers.iter().cloned())
                    .collect(),
            ));
        }
    }
    hints.extend(steers);
    hints
}

/// The first row of `entity` the stored-row search reaches under the further witness `at`, filed
/// under `under` where it names an owner, that `accepts` takes.
fn search_rows(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (at, under): (usize, subject_fact::Under<'_>),
    hints: &[Predicate],
    accepts: impl Fn(&Arrangement) -> Result<bool, RefusalCause>,
) -> Option<Arrangement> {
    subject_fact::search_under(
        ir,
        entity,
        actors,
        hints,
        (Distinction::further(at), "related decoy"),
        &[],
        under,
        |node| Ok(accepts(node)?.then_some(())),
    )
    .map(|(row, ())| row)
    .ok()
}

/// The input that reaches the `exists: false` branch: a fresh identity for `field`, and — where the
/// command declares an input-guarded refusal, else an accepting branch guarded by its input — an
/// input that guard takes, so the witness holds the precedence order: on a missing row,
/// `exists: false` answers before an input refusal and before any accepting branch, whatever the
/// input. A refusal is preferred because it is the nearest step after `exists: false`.
fn without_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    field: &str,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    without_row_each(ir, command, entity, field, distinction)?
        .into_iter()
        .next()
        .ok_or_else(unarranged)
}

/// Sends each of `others` for the missing row, each answered by the `exists: false` refusal
/// `outcome` (beyond10x/ess#227). Only for a refusal, which changes nothing, so every send leaves
/// the row missing for the next.
fn send_each_without_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    bound: &BTreeMap<String, crate::scenario::InstanceName>,
    others: impl Iterator<Item = BTreeMap<String, Node>>,
    steps: &mut Vec<super::ScenarioStep>,
) {
    let Some(error) = &outcome.error else {
        return;
    };
    let command_ref = super::CommandRef::new(command.name.clone());
    for other in others {
        let supplied = super::supply(ir, command, &other, None, None, bound);
        let expected = super::expect_error(ir, outcome, error, &supplied, &BTreeMap::new());
        steps.push(super::ScenarioStep::ExecuteCommand {
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: supplied,
            caller: BTreeMap::new(),
        });
        steps.push(super::ScenarioStep::ExpectOutcome {
            outcome: super::OutcomeRef::new(command_ref.clone(), outcome.name.clone()),
        });
        steps.push(expected);
    }
}

/// [`without_row`] for every input guard of the command: one input per input-guarded refusal and
/// per accepting branch guarded by its input, each taken by that guard, in that order, without
/// repeats — so the `exists: false` scenario sends the missing row with each of them, and a target
/// answering any one of those branches before reading the row fails (beyond10x/ess#227). Where no
/// guard is satisfied, the plain witness.
fn without_row_each(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    field: &str,
    distinction: Distinction,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let fresh = fresh_identity(ir, command, field, None)?;
    let refusals = command
        .outcomes
        .iter()
        .filter(|outcome| super::is_input_guarded_refusal(outcome))
        .filter_map(input_guard);
    let accepting: Vec<&Predicate> = refusals
        .chain(
            command
                .outcomes
                .iter()
                .filter(|outcome| outcome.error.is_none())
                .filter_map(input_guard),
        )
        .collect();
    let mut inputs: Vec<BTreeMap<String, Node>> = Vec::new();
    for guard in &accepting {
        let found =
            candidates(ir, command, &[*guard], distinction).map_err(RefusalCause::NoWitness)?;
        for input in found {
            let facts = flatten(ir, command, &input).map_err(RefusalCause::WitnessRejected)?;
            if decides(&facts, &[*guard], true)? {
                if !inputs.contains(&input) {
                    inputs.push(input);
                }
                break;
            }
        }
    }
    if inputs.is_empty() {
        inputs.extend(
            candidates(ir, command, &[], distinction)
                .map_err(RefusalCause::NoWitness)?
                .into_iter()
                .take(1),
        );
    }
    if inputs.is_empty() {
        return Err(RefusalCause::GuardUnsatisfiable {
            predicate: format!("no row of `{}` for `input.{field}`", entity.name()),
            tried: 0,
        });
    }
    Ok(inputs
        .into_iter()
        .map(|mut input| {
            input.insert(field.to_owned(), fresh.clone());
            input
        })
        .collect())
}

/// A row of the related entity and an input that together select `outcome`, and the distinction
/// the row was arranged at.
///
/// The row is searched for as a `when_subject` branch's is (`subject_fact::search_within`): every
/// creator, and the moves after it, steered toward the command's predicates over the row, so each
/// side of a predicate over a stored field alone — `plan == Basic` — is reached on a row that holds
/// it. For each row the input is searched with every row/input comparison grounded on the row's
/// value; where the caller chose the input, only that one is tried. `(base, distinction,
/// arranging)` names the block of distinctions the row is arranged in, the witness it is for — its
/// place in the block and the distinction its input is searched at — and the entities being
/// arranged around it. A `goal` further requires every predicate it names to read as it says on
/// the row, crossed with the input.
// One argument per thing the search is told (the goal joined the seven, beyond10x/ess#211).
#[allow(clippy::too_many_arguments)]
fn with_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (base, distinction, arranging): (usize, Distinction, &[&EntityHandle]),
    chosen: Option<&BTreeMap<String, Node>>,
    goal: Option<&Goal>,
    around: Option<Around<'_>>,
    (pins, known): (
        &BTreeMap<String, crate::scenario::InstanceName>,
        &[super::ScenarioStep],
    ),
) -> Result<(Arrangement, usize, BTreeMap<String, Node>), RefusalCause> {
    let mut guards: Vec<&Predicate> = command.outcomes.iter().filter_map(input_guard).collect();
    // An external branch's own guard, so an input it admits is among the candidates
    // (beyond10x/ess#464).
    if let ResolvedCondition::ExternalWhen { predicate, .. } = &outcome.condition {
        guards.push(predicate);
    }
    let predicates = predicates(command);
    // A related row's instant ordered against the current time is arranged only through its
    // creator's input, as a `now_offset` (ess/22, A3); one nothing can carry is refused by name.
    if let Some(refusal) = subject_fact::now_uncarried(ir, entity, &predicates, false) {
        return Err(refusal);
    }
    let first = block_start(base, distinction);
    // The rows beside this one (beyond10x/ess#283): their comparisons with the input are tried
    // too, and an input is taken only where each leaves the branch to answer.
    let beside_hints = around.map(|around| around.hints(ir)).unwrap_or_default();
    let meets = |node: &Arrangement, input: &BTreeMap<String, Node>| {
        goal.is_none_or(|(falses, trues)| {
            let truth = |predicate: &Predicate| {
                subject_fact::row_truth_with(
                    ir,
                    entity,
                    &node.settled,
                    &node.unwritten,
                    Some(&node.state),
                    predicate,
                    Some((command, input)),
                )
            };
            falses.iter().all(|child| truth(child) == Truth::False)
                && trues.iter().all(|child| truth(child) == Truth::True)
        })
    };
    // The first input tried on a row that selects `outcome` there — and, `strict`, witnesses every
    // quantifier over a stored collection element by element.
    let selecting = |node: &Arrangement,
                     strict: bool|
     -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
        let inputs = if let Some(chosen) = chosen {
            vec![chosen.clone()]
        } else {
            let grounded = subject_fact::grounded(ir, entity, &node.settled, &predicates);
            let mut searched = guards.clone();
            searched.extend(grounded.iter());
            searched.extend(beside_hints.iter());
            let mut inputs =
                candidates(ir, command, &searched, distinction).map_err(RefusalCause::NoWitness)?;
            if !grounded.is_empty() {
                if let Ok(further) = candidates(ir, command, &searched, retry(distinction)) {
                    inputs.extend(further);
                }
            }
            // A predicate comparing the row's link to its owner with an input naming an owner is
            // decided only on an input naming an arranged owner: the row's own, or a second one
            // (beyond10x/ess#271). The caller sends the owner named through `Setup::bound`, and a
            // link input already bound (`pins`) names that owner whatever is chosen here.
            subject_fact::naming_owners(ir, command, entity, node, inputs, pins)
        };
        Ok(inputs.into_iter().find(|input| {
            meets(node, input)
                && around.is_none_or(|around| around.admits(ir, input))
                && chooses(ir, command, outcome, entity, node, input)
                && (!strict
                    || subject_fact::witnesses_elements(
                        ir,
                        command,
                        entity,
                        &predicates,
                        node,
                        input,
                    ))
        }))
    };
    let search = |strict: bool, under: subject_fact::Under<'_>| {
        subject_fact::search_under(
            ir,
            entity,
            actors,
            &predicates,
            (Distinction::further(first), "related row"),
            arranging,
            under,
            |node| selecting(node, strict),
        )
    };
    // Where a quantifier over a stored collection of the row compares its elements with the input,
    // a row and input that witness it element by element are searched for first
    // (`subject_fact::witnesses_elements`, beyond10x/ess#240), and the plain search runs only where
    // none does. Every other command searches once, as it always did.
    let elementwise = subject_fact::has_elementwise(ir, entity, &predicates);
    let run = |under: subject_fact::Under<'_>| {
        if elementwise {
            search(true, under).or_else(|_| search(false, under))
        } else {
            search(false, under)
        }
    };
    // A link input already bound to an owner (`pins`) is sent naming that owner, so the side of a
    // link comparison where it names the row's own is reached only on a row filed under it: where
    // the row under an owner of its own selects no branch here, it is searched for there
    // (beyond10x/ess#271), and refused naming the first search's cause where that fails too.
    let (row, input) = match run(None) {
        Ok(found) => found,
        Err(cause) => match pins.values().next() {
            Some(held) => {
                run(Some((held, super::holds_none(ir, entity, known, held)))).map_err(|_| cause)?
            }
            None => return Err(cause),
        },
    };
    Ok((row, first, input))
}

fn entity_ref(entity: &EntityHandle) -> crate::scenario::EssSemanticRef {
    crate::scenario::EntityRef::from(entity).into()
}

/// A plain input for a command reading a related row, where a family other than this one chooses
/// the rest of the scenario — the existence family sending a creation twice for one identity. The
/// related row is arranged where the input is sent through [`drive`]; [`point_at_missing`] points
/// it at none instead.
pub(super) fn plain_input(
    ir: &EssIr,
    command: &ResolvedCommand,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    candidates(ir, command, &[], distinction)
        .map_err(RefusalCause::NoWitness)?
        .into_iter()
        .next()
        .ok_or_else(unarranged)
}

/// `input` with the field the command's related guards read set to an identity no row carries.
///
/// The existence family's second call, for an identity a record already carries: the command's
/// own identity is checked before the related row is read, so a target that reads the missing row
/// first answers `exists: false` where `existing_instance:` is declared (beyond10x/ess#211).
pub(super) fn point_at_missing(
    ir: &EssIr,
    command: &ResolvedCommand,
    input: &mut BTreeMap<String, Node>,
) -> Result<(), RefusalCause> {
    let (via, _) = read(command).ok_or_else(unarranged)?;
    let fresh = fresh_identity(ir, command, via.field(), Some(&*input))?;
    input.insert(via.field().to_owned(), fresh);
    Ok(())
}
