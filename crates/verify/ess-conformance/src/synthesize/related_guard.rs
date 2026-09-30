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

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedInstance,
    ResolvedOutcome, ResolvedPayloadValue, ResolvedRelatedTest, ResolvedRelatedVia,
};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

use super::{
    arrange_first, candidates, decides, flatten, fresh_identity, prepare_in, state_default,
    subject_fact, ActorRef, Arrangement, Distinction, RefusalCause, Setup,
};
use crate::witness::MAX_CANDIDATES;

/// The step between the blocks of distinctions the related rows are arranged at: the number
/// [`super::related`] uses, for the reason it gives (every enum of up to twelve variants reads a
/// decoy one away from the row as another variant).
const BLOCK: usize = 27_720;

/// The block a scenario's own related rows start at, and the one a driver's start at: apart, so an
/// arrangement that drives such a command before the branch under test names no row twice.
const OWN: usize = 5;
const DRIVEN: usize = 7;

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
    let Some((via, related)) = read(command) else {
        return false;
    };
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

/// The refusal for a family that has no related row to send the command for.
pub(super) fn unarranged() -> RefusalCause {
    RefusalCause::StrategyWithoutGuard {
        strategy: TestStrategy::ArrangeRelatedRow,
    }
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
    }
    Ok(match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    })
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
    let (via, entity) = read(driver.command).ok_or_else(unarranged)?;
    let field = via.field();
    let names_subject = driver.outcome.subject.as_ref().is_some_and(|subject| {
        matches!(&subject.instance, ResolvedInstance::Supplied { field: named } if named.name == field)
    });
    if names_subject || bound.contains_key(field) || arranging.contains(&entity) {
        return Err(unarranged());
    }
    if is_absent(driver.outcome) {
        let input = match input {
            Some(chosen) => {
                let mut chosen = chosen.clone();
                chosen.insert(field.to_owned(), fresh_identity(ir, driver.command, field)?);
                chosen
            }
            None => without_row(ir, driver.command, entity, field, distinction)?,
        };
        return Ok(super::invoke_with(
            ir, driver, instance, actors, bound, &input,
        ));
    }
    // An owner the caller already bound for a link input — the one `created_owned` arranged for the
    // subject this run creates — is the owner the run is sent naming: the related row is arranged
    // against it, never against a second owner that would replace it (beyond10x/ess#271).
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
    let mut steps = row.steps;
    steps.append(&mut invocation.steps);
    invocation.steps = steps;
    invocation.source.extend(row.source);
    Ok(invocation)
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
pub(super) fn prepare(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    prepare_at(ir, command, outcome, actors, Distinction::PLAIN, None)
}

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
    if let Some((_, entity)) = read(command) {
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

/// [`prepare`] for a further witness: the subject arranged at `distinction`, the related row in the
/// scenario's own block at that witness's place, and — where `goal` names one — the row searched
/// for until every predicate of the goal reads as it says, crossed with the input sent.
pub(super) fn prepare_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    goal: Option<&Goal>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let (via, entity) = read(command).ok_or_else(unarranged)?;
    let field = via.field();
    // A missing row reads no predicate, so no boundary of one is witnessed on it.
    if goal.is_some() && is_absent(outcome) {
        return Err(unarranged());
    }
    let mut setup = match (&outcome.subject, acting(command)) {
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
            super::related::arrange(
                ir,
                outcome,
                actors,
                distinction,
                Setup {
                    after: Some(born.clone()),
                    ..Setup::none()
                },
            )?
        }
        _ => prepare_in(ir, outcome, actors, None, distinction)?,
    };
    let names_subject = outcome.subject.as_ref().is_some_and(|subject| {
        matches!(&subject.instance, ResolvedInstance::Supplied { field: named } if named.name == field)
    });
    if names_subject || setup.bound.contains_key(field) {
        return Err(unarranged());
    }
    let mut steps = Vec::new();
    let mut pinned = BTreeMap::new();
    let input = if is_absent(outcome) {
        let mut each = without_row_each(ir, command, entity, field, distinction)?.into_iter();
        let input = each.next().ok_or_else(unarranged)?;
        // Rows of the entity exist, each carrying an identity other than the one sent.
        let first = block_start(OWN, distinction);
        for at in [first - 1, first + 1] {
            if let Some(decoy) = row_at(ir, entity, actors, at, &[]) {
                steps.extend(decoy.steps);
                setup.source.extend(decoy.source);
            }
        }
        // Every other input guard's input is sent for the missing row too, before the scenario's
        // own send: each is answered by this refusal, so a target answering that guard's branch
        // before reading the row fails. Only for a refusal, which changes nothing, so the sends
        // leave the row missing for the next.
        send_each_without_row(ir, command, outcome, actors, &setup.bound, each, &mut steps);
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
            (&pinned, &setup.steps),
        )?;
        let owners = subject_fact::bind_pinned(
            ir, command, outcome, entity, actors, &mut row, &input, &pinned,
        )?;
        setup.bound.extend(owners);
        for at in [first - 1, first + 1] {
            if at == first + 1 {
                steps.extend(row.steps.iter().cloned());
                setup.source.extend(row.source.iter().cloned());
            }
            if let Some(decoy) = decoy(ir, command, outcome, entity, actors, at, &input) {
                steps.extend(decoy.steps);
                setup.source.extend(decoy.source);
            }
        }
        observe_read(ir, command, entity, &row, &mut steps, &mut setup.source);
        setup.bound.insert(field.to_owned(), row.instance.clone());
        input
    };
    // The related rows go ahead of the branch's own arrangement, but for rows filed under an owner
    // that arrangement brings into being (`pinned`): those follow it.
    if pinned.is_empty() {
        std::mem::swap(&mut steps, &mut setup.steps);
    }
    setup.steps.append(&mut steps);
    setup.source.insert(entity_ref(entity));
    Ok((setup, input))
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

/// A row beside the one the input names, under the further witness `at`: one on which the same
/// input selects another branch, where the stored-row search finds one, so an implementation
/// reading it in place of the named row answers otherwise — else the plain row there.
fn decoy(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
    input: &BTreeMap<String, Node>,
) -> Option<Arrangement> {
    let predicates = predicates(command);
    subject_fact::search_within(
        ir,
        entity,
        actors,
        &predicates,
        Distinction::further(at),
        "related decoy",
        &[],
        |node| {
            Ok(selects(ir, command, entity, Some(node), input)?
                .is_some_and(|branch| branch.name != outcome.name)
                .then_some(()))
        },
    )
    .map(|(row, ())| row)
    .ok()
    .or_else(|| row_at(ir, entity, actors, at, &[]))
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
        let supplied = super::supply(command, &other, None, None, bound);
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
    let fresh = fresh_identity(ir, command, field)?;
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
    (pins, known): (
        &BTreeMap<String, crate::scenario::InstanceName>,
        &[super::ScenarioStep],
    ),
) -> Result<(Arrangement, usize, BTreeMap<String, Node>), RefusalCause> {
    let guards: Vec<&Predicate> = command.outcomes.iter().filter_map(input_guard).collect();
    let predicates = predicates(command);
    let first = block_start(base, distinction);
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
                && selects(ir, command, entity, Some(node), input)
                    .ok()
                    .flatten()
                    .is_some_and(|branch| branch.name == outcome.name)
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
    let fresh = fresh_identity(ir, command, via.field())?;
    input.insert(via.field().to_owned(), fresh);
    Ok(())
}
