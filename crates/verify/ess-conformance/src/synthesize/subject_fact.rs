//! Bounded arrangement of a row to a goal over its stored fields.
//!
//! One strategy for both `when_subject:` shapes — `{field, equals}` (ess/6) runs here as the
//! one-leaf predicate `field == equals` — rewritten in the four parts
//! `docs/design/cross-record-and-stored-field-guards.md` ("Conformance: arranging a row to a goal")
//! names:
//!
//! 1. **Goal-directed input choice through `sets:` mappings.** Every guarded branch's stored-field
//!    predicate is translated through a driver's `sets:` input mappings onto that driver's input,
//!    and the witness search of [`candidates`] is asked for inputs over the translation — so the
//!    creating command is sent `weight_kg: 21` because the guard compares with `20`. A driver's own
//!    guards still have to select its outcome. A literal `sets:` is a fixed point; a field written
//!    through a conversion, or from nothing the search can set, is not arrangeable and the branch
//!    is refused with `ESS-SYNTH-001` naming it.
//! 2. **Goal values from the guard's own literals**, by the rule [`candidates`] already applies to
//!    input: a number at `n`, `n + 1`, `n - 1`, `0` and `-1`, an enum at each variant, a
//!    `Timestamp` at the instant written and a second either side.
//! 3. **A typed fact source over the arranged row**: the determined values are bound against the
//!    entity's declared fields and evaluated by the one evaluator validation and the runners use,
//!    so an `Integer` compares as a number and a `Timestamp` by its instant.
//! 4. **A bounded search** whose node is the lifecycle state crossed with how every guarded
//!    branch's predicate — and each of its leaves — decides over the row. Two arrangements that
//!    decide everything alike are one node; the cap of 64 nodes stays.
//!
//! Where several arranged rows reach the branch at one depth, the one whose values satisfy the
//! most leaves of the guards, and then sit on the most of their literals, is taken: a success
//! beside `service == Express and weight_kg > 20` is witnessed with an Express parcel of 20 kg, the
//! boundary, so an implementation that refuses every Express parcel fails it.
use super::{
    absorb, candidates, created, decides, flatten, has_subject_guards, invoke, invoke_with,
    map_paths, not_emitted, require, shows, state_default, supply, when, ActorRef, Arrangement,
    AssertionStyle, BTreeMap, BTreeSet, CommandRef, Distinction, Driver, EntityHandle, EntityRef,
    EntitySpec, EssIr, EssSemanticRef, FactPath, InstanceNeed, OutcomeRef, Predicate,
    QualifiedName, RefusalCause, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedInstance, ResolvedOutcome, ResolvedPayloadValue, ResolvedSubject, ScenarioStep,
    ScenarioValue, Setup, Truth, Unreachable, ViewExpectation, ViewRef, WitnessGap,
};
use ess_domain::command::OutcomeName;
use ess_domain::entity::Cardinality;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand};

use crate::synthesis_seeds::SeedRecord;

/// The most search nodes one arrangement visits before it refuses.
const MAX_NODES: usize = 64;

/// Whether any branch of this command reads the existing subject's stored fields.
pub(super) fn uses(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectField { .. } | ResolvedCondition::SubjectPredicate { .. }
        )
    })
}

/// Whether any branch of this command uses the ess/9 predicate form.
pub(super) fn uses_predicate(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectPredicate { .. }
        )
    })
}

/// What a branch requires of the stored fields, as one predicate over them.
fn stored(condition: &ResolvedCondition) -> Option<Predicate> {
    match condition {
        ResolvedCondition::SubjectPredicate { predicate, .. } => Some(predicate.clone()),
        ResolvedCondition::SubjectField { field, equals, .. } => Some(Predicate::Compare {
            kind: ess_primitives::predicate::CompareKind::Value,
            left: Operand::Fact(FactPath::new(field).ok()?),
            op: CompareOp::Eq,
            right: Operand::Literal(ess_primitives::facts::FactValue::text(equals.clone())),
        }),
        ResolvedCondition::When { .. }
        | ResolvedCondition::SubjectState { .. }
        | ResolvedCondition::StateChange { .. }
        | ResolvedCondition::Otherwise
        | ResolvedCondition::ExternalWhen { .. }
        | ResolvedCondition::External { .. }
        | ResolvedCondition::Related { .. }
        | ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance
        | ResolvedCondition::InputAbsent
        | ResolvedCondition::ExistingInstance => None,
    }
}

/// What a branch requires of the input.
fn input_guard(condition: &ResolvedCondition) -> Option<&Predicate> {
    match condition {
        ResolvedCondition::When { predicate } => Some(predicate),
        ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
        ResolvedCondition::SubjectPredicate { input, .. } => input.as_ref(),
        _ => None,
    }
}

/// The existing subject a subject-fact command reads: the one its subject-bearing branches name.
pub(super) fn common(command: &ResolvedCommand) -> Option<&ResolvedSubject> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }))
}

/// Whether this strategy arranges the scenario for `outcome`.
///
/// Every branch of a command reading stored fields that an arranged row decides: the guarded ones,
/// the default and any input-guarded sibling — including a refusal that names no subject of its
/// own and reads the command's. Faults, wrong-state refusals and replays keep their own families.
pub(super) fn routes(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    uses(command)
        && outcome.replays.is_none()
        && !matches!(
            outcome.condition,
            ResolvedCondition::External { .. }
                | ResolvedCondition::ExternalWhen { .. }
                | ResolvedCondition::WrongState
        )
        && (outcome.subject.is_some() || common(command).is_some())
        && !reads_identity(command, outcome)
}

/// Whether `outcome` is an input-guarded refusal whose guard reads the identity field that names
/// the subject (beyond10x/ess#178): `id-required: ticket_id == ""`.
///
/// No arranged row can be sent for it, because sending the row replaces the value its own guard
/// admits with the row's identity. It is sent as a plain invocation, with the input its guard
/// admits, and is taken before any row would be read.
pub(super) fn reads_identity(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    let Some(guard) = super::is_input_guarded_refusal(outcome)
        .then(|| when(outcome))
        .flatten()
    else {
        return false;
    };
    let Some(ResolvedInstance::Supplied { field }) =
        common(command).map(|subject| &subject.instance)
    else {
        return false;
    };
    guard
        .fact_paths()
        .iter()
        .any(|path| path.namespace() == field.name)
}

/// The subject this outcome's scenario arranges: its own, or the one its siblings name.
pub(super) fn reading<'a>(
    command: &'a ResolvedCommand,
    outcome: &'a ResolvedOutcome,
) -> Option<&'a ResolvedSubject> {
    outcome
        .subject
        .as_ref()
        .filter(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }))
        .or_else(|| common(command))
}

/// The branches that compete for selection: everything but the default and the families decided
/// elsewhere.
///
/// An `unknown_instance:` answer is one of those families: it is taken for an identity no row
/// carries, never for a row the arrangement built, and counting it beside the guarded branches
/// made every row select two of them.
fn guarded(command: &ResolvedCommand) -> impl Iterator<Item = &ResolvedOutcome> {
    command.outcomes.iter().filter(|branch| {
        !state_default(branch)
            && !matches!(
                branch.condition,
                ResolvedCondition::External { .. }
                    | ResolvedCondition::ExternalWhen { .. }
                    | ResolvedCondition::WrongState
                    | ResolvedCondition::UnknownInstance
                    | ResolvedCondition::InputAbsent
                    | ResolvedCondition::ExistingInstance
            )
    })
}

/// Every guarded branch's stored-field predicate, in declaration order.
fn hints(command: &ResolvedCommand) -> Vec<Predicate> {
    guarded(command)
        .filter_map(|branch| stored(&branch.condition))
        .collect()
}

/// The stored fields any guarded branch of this command reads, in name order.
pub(super) fn guarded_fields(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
) -> BTreeSet<String> {
    read_fields(ir, entity, &hints(command))
}

/// The stored fields one predicate reads.
pub(super) fn read_by(
    ir: &EssIr,
    entity: &EntityHandle,
    predicate: &Predicate,
) -> BTreeSet<String> {
    read_fields(ir, entity, std::slice::from_ref(predicate))
}

/// The stored fields these predicates read, in name order.
fn read_fields(ir: &EssIr, entity: &EntityHandle, predicates: &[Predicate]) -> BTreeSet<String> {
    let declared = ir.entity(entity);
    predicates
        .iter()
        .flat_map(Predicate::fact_paths)
        .map(|path| path.namespace().to_owned())
        .filter(|root| declared.fields.iter().any(|field| &field.name == root))
        .collect()
}

fn missing(entity: &EntityHandle, field: &str, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: format!("{entity}.{field}"),
        type_ref: entity.to_string(),
        reason,
    })
}

/// How one stored-field predicate decides over the arranged row.
///
/// Only the values the arrangement determined as literals are bound, and bound against the
/// entity's declared fields, so each is read at its declared type. A predicate reading a field the
/// arrangement did not determine is `Unknown` — never decided against an absent binding, because
/// the row does hold *something* there and the specification does not say what — except an
/// `Optional` field `unwritten` names, which no step since the row's creation wrote: that one holds
/// nothing, and is bound as absent (beyond10x/ess#239).
///
/// `held` is the lifecycle state the row rests in, which a predicate reading `state` decides over
/// (ess/18, beyond10x/ess#204); without it such a predicate is `Unknown`, for the same reason.
pub(super) fn row_truth(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    unwritten: &BTreeSet<String>,
    held: Option<&super::StateName>,
    predicate: &Predicate,
) -> Truth {
    row_truth_with(ir, entity, settled, unwritten, held, predicate, None)
}

/// Whether a stored-field predicate reads the held lifecycle state as `state` (ess/18,
/// beyond10x/ess#204): the bare path, which no declared field can shadow.
pub(super) fn reads_held_state(ir: &EssIr, entity: &EntityHandle, predicate: &Predicate) -> bool {
    !ir.entity(entity)
        .fields
        .iter()
        .any(|field| field.name == EntitySpec::STATE)
        && ess_domain::command::subject_fact::reads_state(predicate)
}

/// The rows a wrong state `held` of `command` is answered on by guarded branches reading `state`
/// (ess/18, beyond10x/ess#204): for each such branch whose stored predicate is not false with
/// `state` bound to `held` alone, in declaration order, a row of `entity` arranged in `held` under a
/// distinction of its own, observed there, and sent an input the branch is selected by on it.
///
/// Guarded branches select before `wrong_state:` applies (the #192 ruling), so these rows belong in
/// the state's `<entity>/state/<S>/refuses/<command>` scenario beside the plain wrong-state row,
/// which is witnessed only where some input still reaches it. Every step asserts the branch taken
/// and its error or its absence; the branch's effects are its own outcome scenario's.
///
/// `Ok` with no steps where no guarded branch reads `state`. A branch that may be selected in `held`
/// and that no bounded arrangement reaches refuses the whole with that cause: the state is never
/// left silently unwitnessed.
pub(super) fn state_answered_rows(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    held: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    let Ok(path) = FactPath::new(EntitySpec::STATE) else {
        return Ok((steps, source));
    };
    let mut facts = ess_primitives::facts::FactStore::new();
    facts.set_if_absent(
        path,
        ess_primitives::facts::FactValue::text(held.to_string()),
    );
    let hints = hints(command);
    let command_ref = CommandRef::new(command.name.clone());
    let mut rows = 0;
    for branch in guarded(command) {
        let Some(predicate) = stored(&branch.condition) else {
            continue;
        };
        if !reads_held_state(ir, entity, &predicate)
            || predicate.evaluate(&facts) == Truth::False
            || reading(command, branch).is_none_or(|subject| &subject.entity != entity)
        {
            continue;
        }
        rows += 1;
        let (mut arrangement, input) = search(
            ir,
            entity,
            actors,
            &hints,
            Distinction::further(rows),
            "state",
            |node| {
                if &node.state != held {
                    return Ok(None);
                }
                reach_at(ir, command, branch, entity, node)
            },
        )?;
        let (observed, view) =
            observe_fields(ir, entity, &read_by(ir, entity, &predicate), &arrangement)?;
        models.mark(
            super::caller::InvocationPhase::Arrange,
            &mut arrangement.steps,
        );
        steps.extend(arrangement.steps);
        steps.extend(observed);
        source.extend(arrangement.source);
        source.insert(view.into());
        let outcome_ref = OutcomeRef::new(command_ref.clone(), branch.name.clone());
        let supplied = supply(
            ir,
            command,
            &input,
            reading(command, branch),
            Some(&arrangement.instance),
            &BTreeMap::new(),
        );
        steps.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: supplied.clone(),
        });
        steps.push(ScenarioStep::ExpectOutcome {
            outcome: outcome_ref.clone(),
        });
        match &branch.error {
            Some(error) => {
                steps.push(super::expect_error(
                    ir,
                    branch,
                    error,
                    &supplied,
                    &arrangement.settled,
                ));
                steps.push(ScenarioStep::ExpectNoEvents);
            }
            None => steps.push(ScenarioStep::ExpectNoError),
        }
        source.insert(outcome_ref.into());
    }
    models.mark(super::caller::InvocationPhase::Act, &mut steps);
    Ok((steps, source))
}

/// [`row_truth`], with the command's input bound under `input.` for a predicate that compares the
/// row with it (beyond10x/ess#157). Without an input such a comparison is `Unknown`, which is what
/// the search and the boundary goals see: the row alone does not decide it.
pub(super) fn row_truth_with(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    unwritten: &BTreeSet<String>,
    held: Option<&super::StateName>,
    predicate: &Predicate,
    input: Option<(&ResolvedCommand, &BTreeMap<String, Node>)>,
) -> Truth {
    evaluate_row(
        ir, entity, settled, unwritten, held, predicate, input, false,
    )
}

/// [`row_truth_with`] for a branch's whole stored guard, read the way the target selects a branch:
/// only true takes it (beyond10x/ess#234).
///
/// A guard that stays `Unknown` only because a fact it reads is absent from a row the arrangement
/// knows whole — an `Optional` field no step wrote ([`row_truth`], #239), a member of a struct
/// held there, or an `Optional` input member the witness leaves out — is the specification's own
/// unknown, which every target reads as not taken: it is `False` here. An `Unknown` the arrangement
/// caused — a field it did not determine, an ordering no scale decides — stays `Unknown`. Never
/// for a leaf or conjunct of a guard: under `not` the specification's unknown stays unknown.
pub(super) fn guard_truth_with(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    unwritten: &BTreeSet<String>,
    held: Option<&super::StateName>,
    predicate: &Predicate,
    input: Option<(&ResolvedCommand, &BTreeMap<String, Node>)>,
) -> Truth {
    evaluate_row(ir, entity, settled, unwritten, held, predicate, input, true)
}

/// Whether every leaf of `predicate` that `facts` leave `Unknown` reads a fact they hold absent:
/// a `null` or left-out value under a root they bind, never an `input.` path without an input.
fn unknown_by_absence(predicate: &Predicate, facts: &RowAndInput<'_>) -> bool {
    let mut found = Vec::new();
    leaves(predicate, &mut found);
    let absent = |path: &&FactPath| {
        (facts.input.is_some() || input_path(path).is_none())
            && ess_primitives::facts::FactSource::fact(facts, path).is_none()
            && !ess_primitives::facts::FactSource::present(facts, path)
    };
    found
        .iter()
        .all(|leaf| leaf.evaluate(facts) != Truth::Unknown || leaf.fact_paths().iter().any(absent))
}

#[allow(clippy::too_many_arguments)]
fn evaluate_row(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    unwritten: &BTreeSet<String>,
    held: Option<&super::StateName>,
    predicate: &Predicate,
    input: Option<(&ResolvedCommand, &BTreeMap<String, Node>)>,
    as_guard: bool,
) -> Truth {
    let declared = ir.entity(entity);
    let mut fields = declared.fields.clone();
    // A stored instant the predicate orders against the current time is held only as a
    // `now_offset` its creator was sent, read at the reference instant the guards are decided at
    // (ess/22, A3); a literal one would be decided at the reference and not at a run, so it is
    // left undetermined, as a generated one is.
    let now_roots: BTreeSet<&str> = crate::now_offset::now_compared(predicate)
        .into_iter()
        .filter(|(_, whole)| *whole)
        .filter_map(|(path, _)| {
            declared
                .fields
                .iter()
                .find(|field| field.name == path.namespace())
                .map(|field| field.name.as_str())
        })
        .collect();
    let mut values: BTreeMap<String, Node> = settled
        .iter()
        .filter_map(|(name, determined)| match &determined.value {
            ScenarioValue::NowOffset { seconds } => crate::now_offset::reference()
                .plus_seconds(*seconds)
                .map(|instant| (name.clone(), Node::Text(instant.to_rfc3339()))),
            ScenarioValue::Literal { value }
                if now_roots.contains(name.as_str()) && *value != Node::Null =>
            {
                None
            }
            other => other
                .as_literal()
                .map(|value| (name.clone(), value.clone())),
        })
        .collect();
    // An `Optional` field no step of the arrangement wrote holds nothing (beyond10x/ess#239): the
    // row as its creator left it, which `{exists: false}` and `not defined()` select.
    for field in unwritten {
        if !settled.contains_key(field) {
            values.insert(field.clone(), Node::Null);
        }
    }
    // A link to the owner compared with an input naming an arranged owner is bound as that
    // owner's token, which the input carries too (beyond10x/ess#193).
    if let Some((command, sent)) = input {
        values.extend(link_facts(ir, command, entity, settled, predicate, sent));
    }
    for path in predicate.fact_paths() {
        let root = path.namespace();
        if declared.fields.iter().any(|field| field.name == root) && !values.contains_key(root) {
            return Truth::Unknown;
        }
    }
    // The held state is bound as `state` at the lifecycle's own type, beside the stored fields.
    if reads_held_state(ir, entity, predicate) {
        let Some(held) = held else {
            return Truth::Unknown;
        };
        fields.push(declared.state_field());
        values.insert(EntitySpec::STATE.to_owned(), Node::Text(held.to_string()));
    }
    let Ok(store) = crate::input::bind(ir, &fields, &values, crate::input::Completeness::Partial)
    else {
        return Truth::Unknown;
    };
    let row = crate::input::TypedFacts::new(ir, &fields, store);
    let input = match input {
        Some((command, values)) if reads_input(ir, entity, predicate) => {
            match flatten(ir, command, values) {
                Ok(facts) => Some(facts),
                Err(_) => return Truth::Unknown,
            }
        }
        _ => None,
    };
    let facts = RowAndInput { row, input };
    match predicate.evaluate(&facts) {
        Truth::Unknown if as_guard && unknown_by_absence(predicate, &facts) => Truth::False,
        truth => truth,
    }
}

/// The distinction the second owner a link comparison names is arranged under: past every further
/// instance an arrangement or a view's companion rows number (those stop at
/// [`MAX_CANDIDATES`](super::MAX_CANDIDATES)), so its name is never one they capture as well.
const OTHER_OWNER: Distinction = Distinction::further(super::MAX_CANDIDATES + 1);

/// Every `==` or `!=` a guarded branch's stored predicate writes between the row's link to its
/// owner and an input field of the owner's identity type (beyond10x/ess#193): the input field, and
/// the link field it is compared with.
///
/// `account_id != input.account_id` asks whether the caller names the owner the row was filed
/// under. Neither side is a value the specification spells — the owner is the instance the
/// arrangement created, and the caller names one — so the input is sent as an arranged instance:
/// the row's own owner, or a second one arranged beside it ([`linked_inputs`]).
pub(super) fn links(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let Some(owned) = ir.owner_of(entity) else {
        return out;
    };
    let Some(link) = ir
        .entity(entity)
        .fields
        .iter()
        .find(|field| field.name == owned.via)
    else {
        return out;
    };
    let mut found = Vec::new();
    for hint in hints(command).iter().chain(&related_over(command, entity)) {
        leaves(hint, &mut found);
    }
    for leaf in found.iter().filter(|leaf| reads_input(ir, entity, leaf)) {
        let Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq | CompareOp::Ne,
            right: Operand::Fact(right),
            ..
        } = leaf
        else {
            continue;
        };
        for (row, other) in [(left, right), (right, left)] {
            let Some(sent) = input_path(other).filter(|sent| sent.segments().len() == 1) else {
                continue;
            };
            let typed = command
                .input
                .iter()
                .find(|field| field.name == sent.namespace())
                .is_some_and(|field| field.type_ref.required() == link.type_ref.required());
            if typed && row.segments().len() == 1 && row.namespace() == link.name {
                out.insert((sent.namespace().to_owned(), link.name.clone()));
            }
        }
    }
    out
}

/// Every predicate a `when_related:` branch of `command` holds of a row of `entity`, in declaration
/// order (beyond10x/ess#271): the related row is compared with the input as a subject row is, so a
/// link comparison over it is one [`links`] names too. Empty for every command reading no related
/// row of `entity`, which keeps the candidates, and so the suites, it had.
fn related_over(command: &ResolvedCommand, entity: &EntityHandle) -> Vec<Predicate> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                entity: read,
                test: ess_compiler::ir::ResolvedRelatedTest::Holds { predicate },
                ..
            } if read == entity => Some(predicate.clone()),
            _ => None,
        })
        .collect()
}

/// Whether any of `predicates` holds a link comparison of `command` ([`links`]): an `==` or `!=`
/// between the row's link to its owner and the input naming an owner. Such a predicate is decided
/// only with the input bound, so every search toward it runs over [`linked_inputs`] (beyond10x/ess
/// #193). A goal over it past [`MAX_BOUNDARIES`] is refused, and so is one the bounded search did not
/// reach where an input naming an arranged owner left it undecided ([`goal_input`]); one every
/// candidate decided and no bounded row meets adds no row, as for every other guard.
fn compares_link(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    predicates: &[Predicate],
) -> bool {
    let pairs = links(ir, command, entity);
    if pairs.is_empty() {
        return false;
    }
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    let pair = |row: &FactPath, other: &FactPath| {
        pairs.iter().any(|(sent, field)| {
            row.segments().len() == 1
                && row.namespace() == field
                && input_path(other).is_some_and(|rest| rest.segments() == [sent.clone()])
        })
    };
    found.iter().any(|leaf| {
        matches!(
            leaf,
            Predicate::Compare {
                left: Operand::Fact(left),
                op: CompareOp::Eq | CompareOp::Ne,
                right: Operand::Fact(right),
             .. } if pair(left, right) || pair(right, left)
        )
    })
}

/// For each of `inputs`, whether an undecided goal on it counts as one the search could not decide:
/// an input naming an arranged owner ([`linked`]), or any input where none of them names one. A
/// plain candidate beside inputs that name an owner leaves a link comparison undecided by
/// construction, and says nothing about whether the goal's row exists.
fn naming_owner(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    inputs: &[BTreeMap<String, Node>],
) -> Vec<bool> {
    let names: Vec<bool> = inputs
        .iter()
        .map(|input| !linked(ir, command, entity, &arrangement.settled, input).is_empty())
        .collect();
    let any = names.contains(&true);
    names.into_iter().map(|named| named || !any).collect()
}

/// The value an input field naming `instance` is chosen at while the search decides a link
/// comparison: a token of the instance at the field's type, which no two instances share. It is
/// never sent — [`prepare`] sends the instance itself in its place, through `Setup::bound`.
pub(super) fn token(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    instance: &super::InstanceName,
) -> Option<Node> {
    let path = FactPath::new(field).ok()?;
    let declared = |primitive| crate::input::declared_as(ir, &command.input, &path, primitive);
    let text = format!("instance:{instance}");
    if declared(ess_domain::types::Primitive::Uuid) {
        Some(Node::Text(crate::witness::uuid_of(&text)))
    } else if declared(ess_domain::types::Primitive::String) {
        Some(Node::Text(text))
    } else {
        None
    }
}

/// The owner a link comparison's other side names, and what it is called: an instance of the
/// entity that owns `entity`, arranged under [`OTHER_OWNER`].
fn other_owner(ir: &EssIr, entity: &EntityHandle) -> Option<(EntityHandle, super::InstanceName)> {
    let owned = ir.owner_of(entity)?;
    let name = super::instance_name(&ir.entity(&owned.owner).name, OTHER_OWNER);
    Some((owned.owner, name))
}

/// The arranged owner each link input of `input` names, by input field: the row's own owner, or
/// the [`other_owner`], where the input carries its [`token`]. A field carrying anything else — a
/// literal nobody assigned — names no owner, and the comparison stays undecided.
fn linked(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    input: &BTreeMap<String, Node>,
) -> BTreeMap<String, super::InstanceName> {
    let other = other_owner(ir, entity).map(|(_, name)| name);
    let mut out = BTreeMap::new();
    for (sent, field) in links(ir, command, entity) {
        let Some(ScenarioValue::Instance { instance: own }) =
            settled.get(&field).map(|held| &held.value)
        else {
            continue;
        };
        let Some(value) = input.get(&sent) else {
            continue;
        };
        let known: Vec<(&super::InstanceName, Option<Node>)> = std::iter::once(own)
            .chain(other.iter().filter(|other| *other != own))
            .map(|name| (name, token(ir, command, &sent, name)))
            .collect();
        // Two owners whose tokens coincide cannot be told apart, so neither is named.
        if let [(_, first), (_, second)] = known.as_slice() {
            if first == second {
                continue;
            }
        }
        if let Some((name, _)) = known
            .iter()
            .find(|(_, token)| token.as_ref() == Some(value))
        {
            out.insert(sent, (*name).clone());
        }
    }
    out
}

/// The link fields [`row_truth_with`] binds for `predicate` and this input: each link whose input
/// names an arranged owner, at the row's own owner's [`token`] — where `predicate` reads the link
/// and that input only in `==` or `!=` between the two, or the link in `defined()`. Anything else
/// asked of either — an ordering, a literal, a text test — is a question about the identity's
/// value, which no token answers, and leaves the predicate `Unknown` as before.
fn link_facts(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicate: &Predicate,
    input: &BTreeMap<String, Node>,
) -> BTreeMap<String, Node> {
    let named = linked(ir, command, entity, settled, input);
    let mut out = BTreeMap::new();
    for (sent, field) in links(ir, command, entity) {
        let Some(ScenarioValue::Instance { instance: own }) =
            settled.get(&field).map(|held| &held.value)
        else {
            continue;
        };
        if !named.contains_key(&sent) || !only_compared(ir, entity, predicate, &field, &sent) {
            continue;
        }
        if let Some(token) = token(ir, command, &sent, own) {
            out.insert(field, token);
        }
    }
    out
}

/// Whether every leaf of `predicate` reading the row's `field` or `input.<sent>` is an `==` or `!=`
/// between exactly those two, or `defined(field)`.
fn only_compared(
    ir: &EssIr,
    entity: &EntityHandle,
    predicate: &Predicate,
    field: &str,
    sent: &str,
) -> bool {
    let mut found = Vec::new();
    leaves(predicate, &mut found);
    let row = |path: &FactPath| path.segments().len() == 1 && path.namespace() == field;
    let input =
        |path: &FactPath| input_path(path).is_some_and(|rest| rest.segments() == [sent.to_owned()]);
    let touches = |path: &FactPath| {
        path.namespace() == field
            || (reads_input(ir, entity, predicate)
                && input_path(path).is_some_and(|rest| rest.namespace() == sent))
    };
    found.iter().all(|leaf| match leaf {
        Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq | CompareOp::Ne,
            right: Operand::Fact(right),
            ..
        } if (row(left) && input(right)) || (input(left) && row(right)) => true,
        Predicate::Defined(path) if row(path) => true,
        other => !other.fact_paths().into_iter().any(&touches),
    })
}

/// Whether a stored-field predicate compares the row with the command's input: a path rooted at
/// `input.`, where the entity declares no field of that name (beyond10x/ess#157).
fn reads_input(ir: &EssIr, entity: &EntityHandle, predicate: &Predicate) -> bool {
    let namespace = ess_domain::command::subject_fact::INPUT_NAMESPACE;
    !ir.entity(entity)
        .fields
        .iter()
        .any(|field| field.name == namespace)
        && predicate
            .fact_paths()
            .iter()
            .any(|path| path.namespace() == namespace && path.segments().len() > 1)
}

/// The path under `input.` a stored-field predicate reads, as the input path it names.
fn input_path(path: &FactPath) -> Option<FactPath> {
    let (root, rest) = path.segments().split_first()?;
    (root == ess_domain::command::subject_fact::INPUT_NAMESPACE && !rest.is_empty())
        .then(|| FactPath::from_segments(rest))
}

/// The arranged row, and the input a scenario sends read under `input.`: the one fact source a
/// stored-field predicate over both is evaluated against, each half at its declared types.
struct RowAndInput<'a> {
    row: crate::input::TypedFacts<'a>,
    input: Option<crate::InputFacts<'a>>,
}

impl RowAndInput<'_> {
    fn split(&self, path: &FactPath) -> Option<(&crate::InputFacts<'_>, FactPath)> {
        let input = self.input.as_ref()?;
        input_path(path).map(|rest| (input, rest))
    }
}

impl ess_primitives::facts::FactSource for RowAndInput<'_> {
    fn fact(&self, path: &FactPath) -> Option<ess_primitives::facts::FactValue> {
        match self.split(path) {
            Some((input, rest)) => input.fact(&rest),
            None => self.row.fact(path),
        }
    }

    /// `defined()` over an `Optional` struct, list or map reads the presence the row or the input
    /// recorded for it, which no fact carries (beyond10x/ess#176).
    fn present(&self, path: &FactPath) -> bool {
        match self.split(path) {
            Some((input, rest)) => input.present(&rest),
            None => self.row.present(path),
        }
    }

    fn scales(&self) -> &ess_primitives::facts::Scales {
        self.row.scales()
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        match self.split(path) {
            Some((input, rest)) => input.orders_as_instant(&rest),
            None => self.row.orders_as_instant(path),
        }
    }

    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        match self.split(path) {
            Some((input, rest)) => input.orders_text_by_bytes(&rest),
            None => self.row.orders_text_by_bytes(path),
        }
    }

    /// The reference instant every guard of the decision is decided at, a stored row's ordering
    /// against `now` included (ess/22, A3): the instant each `now_offset` a row holds is read
    /// from, as an input guard's is ([`crate::now_offset::reference`]).
    fn now(&self) -> Option<ess_primitives::time::Rfc3339Instant> {
        Some(crate::now_offset::reference())
    }
}

/// `predicate` with every comparison between the row and the input replaced by `Always`: what the
/// arranging branches' input search is handed, because an `input.` path names nothing of theirs.
fn without_input(ir: &EssIr, entity: &EntityHandle, predicate: &Predicate) -> Predicate {
    match predicate {
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| without_input(ir, entity, child))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| without_input(ir, entity, child))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(without_input(ir, entity, inner))),
        leaf if reads_input(ir, entity, leaf) => Predicate::Always,
        other => other.clone(),
    }
}

/// Every comparison between the row and the input, grounded on this arrangement: the stored side
/// replaced by the value the row holds, the input side by the input path it names. Handed to the
/// candidate search beside the command's own guards, its literals are the values the input is
/// tried at — the row's value, and by rule 3 its neighbours — so one candidate names the stored
/// value and another does not, and the guard is witnessed both ways (beyond10x/ess#157). A
/// comparison inside a quantifier over a stored list or map is grounded once per element the row
/// holds ([`ground_leaf`], beyond10x/ess#240).
pub(super) fn grounded(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicates: &[Predicate],
) -> Vec<Predicate> {
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    let mut out = Vec::new();
    for leaf in found.iter().filter(|leaf| reads_input(ir, entity, leaf)) {
        ground_leaf(settled, &[], leaf, &mut out);
    }
    out
}

/// The value the row holds at `path`, reading a quantifier's binder as the element it is bound to
/// (innermost first): a struct member by name, a list element by its ordinal.
fn held_node(
    settled: &BTreeMap<String, super::Determined>,
    bound: &[(&str, &Node)],
    path: &FactPath,
) -> Option<Node> {
    let (root, rest) = path.segments().split_first()?;
    let mut node = match bound.iter().rev().find(|(name, _)| name == root) {
        Some((_, element)) => *element,
        None => settled.get(root)?.value.as_literal()?,
    };
    for segment in rest {
        node = match node {
            Node::Map(entries) => entries.get(segment)?,
            Node::Seq(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(node.clone())
}

/// A byte length of an input stays the byte length of that input; one of a stored text is the number
/// the row's text measures, or grounds nothing where the row holds none (decision 11).
fn ground_derived(
    settled: &BTreeMap<String, super::Determined>,
    bound: &[(&str, &Node)],
    derived: &ess_primitives::predicate::Derived,
) -> Option<Operand> {
    let parent = derived.parent();
    match input_path(parent) {
        Some(rest) if !bound.iter().any(|(name, _)| *name == parent.namespace()) => {
            Some(Operand::Derived(derived.with_parent(rest)))
        }
        _ => derived
            .value_with(&|path| {
                held_node(settled, bound, path)
                    .as_ref()
                    .and_then(super::fact_value)
            })
            .map(Operand::Literal),
    }
}

/// One leaf of a row/input comparison, grounded on the row: a comparison with its stored side
/// replaced by the value held there; a quantifier over a stored collection once per element the row
/// holds, its binder read as that element (beyond10x/ess#240). A map's elements are its values in
/// key order — what the quantifier binds — and a list's its elements, so `exists r in
/// redirect_uris: r == input.application` over a row holding `{k: v}` grounds `v == application`,
/// and the input is tried at `v`, which satisfies it, and at its neighbours, which do not. An empty
/// collection grounds nothing: the input cannot move a quantifier over it.
fn ground_leaf(
    settled: &BTreeMap<String, super::Determined>,
    bound: &[(&str, &Node)],
    leaf: &Predicate,
    out: &mut Vec<Predicate>,
) {
    let side = |operand: &Operand| -> Option<Operand> {
        match operand {
            Operand::Fact(path) if !bound.iter().any(|(name, _)| *name == path.namespace()) => {
                match input_path(path) {
                    Some(rest) => Some(Operand::Fact(rest)),
                    None => held_node(settled, bound, path)
                        .as_ref()
                        .and_then(super::fact_value)
                        .map(Operand::Literal),
                }
            }
            Operand::Fact(path) => held_node(settled, bound, path)
                .as_ref()
                .and_then(super::fact_value)
                .map(Operand::Literal),
            // An offset of an input stays an offset of that input; one of a stored value is the
            // value it names (A2), or grounds nothing where the row holds no such value.
            Operand::Offset(offset) => {
                let base = &offset.base;
                match input_path(base) {
                    Some(rest) if !bound.iter().any(|(name, _)| *name == base.namespace()) => {
                        Some(Operand::Offset(ess_primitives::predicate::OffsetOperand {
                            base: rest,
                            direction: offset.direction,
                            magnitude: offset.magnitude,
                        }))
                    }
                    _ => held_node(settled, bound, base)
                        .as_ref()
                        .and_then(super::fact_value)
                        .and_then(|value| offset.value_at(&value))
                        .map(Operand::Literal),
                }
            }
            Operand::Derived(derived) => ground_derived(settled, bound, derived),
            Operand::Literal(value) => Some(Operand::Literal(value.clone())),
        }
    };
    match leaf {
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } => {
            if let (Some(left), Some(right)) = (side(left), side(right)) {
                // A held value against an offset of an input (`upper == input.to + 5`) is the input
                // against the held value moved back (`to == upper - 5`), with the operator turned
                // round: one input leaf against a literal, whose boundary the ladders try exactly
                // and a unit either side (A2). Where the moved value is past what the type holds,
                // nothing is grounded.
                let (left, op, right) = match (left, right) {
                    (Operand::Literal(held), Operand::Offset(offset)) => {
                        let Some(bound) = offset.reversed(offset.base.clone()).value_at(&held)
                        else {
                            return;
                        };
                        (
                            Operand::Fact(offset.base.clone()),
                            turned(*op),
                            Operand::Literal(bound),
                        )
                    }
                    (left, right) => (left, *op, right),
                };
                // Only a comparison the input takes part in steers the input.
                if matches!(left, Operand::Fact(_))
                    || matches!(right, Operand::Fact(_) | Operand::Offset(_))
                {
                    out.push(Predicate::Compare {
                        kind: *kind,
                        left,
                        op,
                        right,
                    });
                }
            }
        }
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let elements = match held_node(settled, bound, &quantified.over) {
                Some(Node::Map(entries)) => entries.into_values().collect(),
                Some(Node::Seq(items)) => items,
                _ => Vec::new(),
            };
            let mut body = Vec::new();
            leaves(&quantified.body, &mut body);
            for element in &elements {
                let mut inner = bound.to_vec();
                inner.push((quantified.bind.as_str(), element));
                for leaf in &body {
                    ground_leaf(settled, &inner, leaf, out);
                }
            }
        }
        _ => {}
    }
}

/// `op` with its two sides swapped: `a < b` is `b > a`.
fn turned(op: CompareOp) -> CompareOp {
    match op {
        CompareOp::Lt => CompareOp::Gt,
        CompareOp::Le => CompareOp::Ge,
        CompareOp::Gt => CompareOp::Lt,
        CompareOp::Ge => CompareOp::Le,
        other => other,
    }
}

/// The distinction the second and third entries of a spread collection are built at, past every
/// further instance an arrangement numbers ([`MAX_CANDIDATES`](super::MAX_CANDIDATES)) and the
/// second owner ([`OTHER_OWNER`]), so an entry never repeats a value another row of the scenario
/// holds. A row at distinction `d` takes `SPREAD + 2d` and `SPREAD + 2d + 1`.
const SPREAD: usize = 2 * (super::MAX_CANDIDATES + 2);

/// The quantifiers of `predicates` whose collection is stored on the row and whose body compares
/// an element with the command's input (beyond10x/ess#240): the leaves [`spread`] arranges several
/// entries for and [`witnesses_elements`] holds a scenario to.
fn elementwise(ir: &EssIr, entity: &EntityHandle, predicates: &[Predicate]) -> Vec<Predicate> {
    let declared = ir.entity(entity);
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    found.retain(|leaf| match leaf {
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            input_path(&quantified.over).is_none()
                && declared
                    .fields
                    .iter()
                    .any(|field| field.name == quantified.over.namespace())
                && reads_input(ir, entity, leaf)
        }
        _ => false,
    });
    found
}

/// Whether any of `predicates` is an [`elementwise`] quantifier.
pub(super) fn has_elementwise(ir: &EssIr, entity: &EntityHandle, predicates: &[Predicate]) -> bool {
    !elementwise(ir, entity, predicates).is_empty()
}

/// Whether the row `node` and `input` witness every [`elementwise`] quantifier of `predicates`
/// element by element (beyond10x/ess#240).
///
/// A quantifier one element decides — an `exists` that holds, a `forall` that fails — must read
/// otherwise over the collection's first element alone, over its last alone, and as the other
/// quantifier, so a target reading only the first value, only the last, or `forall` for `exists`
/// (and the reverse) answers this scenario wrongly. One the whole collection decides — an `exists`
/// that fails, a `forall` that holds — must hold two or more elements, so it is not decided by one
/// value standing in for all of them. A quantifier the row does not decide is not held to either.
pub(super) fn witnesses_elements(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    predicates: &[Predicate],
    node: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> bool {
    let truth = |settled: &BTreeMap<String, super::Determined>, leaf: &Predicate| {
        row_truth_with(
            ir,
            entity,
            settled,
            &node.unwritten,
            Some(&node.state),
            leaf,
            Some((command, input)),
        )
    };
    elementwise(ir, entity, predicates).iter().all(|leaf| {
        let (Predicate::Forall(quantified) | Predicate::Exists(quantified)) = leaf else {
            return true;
        };
        let exists = matches!(leaf, Predicate::Exists(_));
        let held = truth(&node.settled, leaf);
        let count = match held_node(&node.settled, &[], &quantified.over) {
            Some(Node::Map(entries)) => entries.len(),
            Some(Node::Seq(items)) => items.len(),
            _ => return true,
        };
        let one_decides = match held {
            Truth::True => exists,
            Truth::False => !exists,
            Truth::Unknown => return true,
        };
        if !one_decides {
            return count >= 2;
        }
        let other = if exists {
            Predicate::Forall(quantified.clone())
        } else {
            Predicate::Exists(quantified.clone())
        };
        let differs = |truth: Truth| truth != held && truth != Truth::Unknown;
        differs(truth(&node.settled, &other))
            && [false, true].into_iter().all(|last| {
                one_element(&node.settled, &quantified.over, last)
                    .is_some_and(|settled| differs(truth(&settled, leaf)))
            })
    })
}

/// `settled` with the stored collection at `path` cut to its first element, or its `last` — a map
/// to its first or last entry in key order, the order a quantifier walks it in.
fn one_element(
    settled: &BTreeMap<String, super::Determined>,
    path: &FactPath,
    last: bool,
) -> Option<BTreeMap<String, super::Determined>> {
    let (root, rest) = path.segments().split_first()?;
    let mut out = settled.clone();
    let held = out.get_mut(root)?;
    let mut value = held.value.as_literal()?.clone();
    let mut at = &mut value;
    for segment in rest {
        at = match at {
            Node::Map(members) => members.get_mut(segment)?,
            _ => return None,
        };
    }
    match at {
        Node::Map(entries) => {
            let kept = if last {
                entries.pop_last()?
            } else {
                entries.pop_first()?
            };
            *entries = BTreeMap::from([kept]);
        }
        Node::Seq(items) => {
            let kept = if last {
                items.pop()?
            } else {
                items.first()?.clone()
            };
            *items = vec![kept];
        }
        _ => return None,
    }
    held.value = ScenarioValue::Literal { value };
    Some(out)
}

/// Inputs for `driver` that write each stored collection an [`elementwise`] quantifier of `hints`
/// reads with several entries rather than the witness's one (beyond10x/ess#240): the witness input
/// at `distinction`, with the collection's input set to three entries in three shapes — every value
/// distinct, every value the first, and the first value either side of another. Over those an
/// `exists` finds a row where the one matching value is neither first nor last, a `forall` one where
/// several values all satisfy it and one where the one failing value sits in the middle, which is
/// what [`witnesses_elements`] asks of the row a scenario is arranged on.
///
/// The further entries are the witnesses at [`SPREAD`], so no value repeats one another row holds.
/// Empty where no such quantifier reads a field `driver` writes from its input, or where the input
/// cannot hold two distinct entries (a map keyed by a `Boolean` holds two at most).
fn spread(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    hints: &[Predicate],
    distinction: Distinction,
) -> Vec<BTreeMap<String, Node>> {
    let quantifiers = elementwise(ir, entity, hints);
    if quantifiers.is_empty() {
        return Vec::new();
    }
    let mapping = mapped(driver.outcome);
    let guards: Vec<&Predicate> = driver
        .command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    let at = |distinction: Distinction| {
        candidates(ir, driver.command, &guards, distinction)
            .ok()?
            .into_iter()
            .next()
    };
    let Some(base) = at(distinction) else {
        return Vec::new();
    };
    let further = [
        at(Distinction::further(SPREAD + 2 * distinction.get())),
        at(Distinction::further(SPREAD + 2 * distinction.get() + 1)),
    ];
    let mut shapes = vec![base.clone(), base.clone(), base.clone()];
    let mut written = BTreeSet::new();
    for leaf in &quantifiers {
        let (Predicate::Forall(quantified) | Predicate::Exists(quantified)) = leaf else {
            continue;
        };
        let Some(sent) = mapping.get(quantified.over.namespace()) else {
            continue;
        };
        let mut path: Vec<String> = sent.split('.').map(str::to_owned).collect();
        path.extend(quantified.over.segments()[1..].iter().cloned());
        if !written.insert(path.clone()) {
            continue;
        }
        let witnesses = std::iter::once(Some(&base))
            .chain(further.iter().map(Option::as_ref))
            .flatten()
            .filter_map(|input| node_at(input, &path));
        let Some(collections) = spread_collection(witnesses) else {
            continue;
        };
        for (shape, collection) in shapes.iter_mut().zip(collections) {
            set_at(shape, &path, collection);
        }
    }
    if written.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<BTreeMap<String, Node>> = Vec::new();
    for shape in shapes {
        if shape != base && !out.contains(&shape) {
            out.push(shape);
        }
    }
    out
}

/// The three collections [`spread`] writes, from the first entry of each witness collection: every
/// value distinct, every value the first, and the first value either side of the second. `None`
/// where the witnesses give fewer than two distinct entries — a map's keys collide, or every value
/// is one value.
fn spread_collection<'n>(witnesses: impl Iterator<Item = &'n Node>) -> Option<[Node; 3]> {
    let mut keys = BTreeSet::new();
    let mut values = Vec::new();
    let mut map = false;
    for collection in witnesses {
        match collection {
            Node::Map(entries) => {
                map = true;
                if let Some((key, value)) = entries.first_key_value() {
                    if keys.insert(key.clone()) {
                        values.push(value.clone());
                    }
                }
            }
            Node::Seq(items) => values.extend(items.first().cloned()),
            _ => {}
        }
    }
    let (first, middle) = match values.as_slice() {
        [first, middle, ..] if values.iter().any(|value| value != first) => {
            (first.clone(), middle.clone())
        }
        _ => return None,
    };
    let odd = (0..values.len())
        .map(|index| {
            if index == 1 {
                middle.clone()
            } else {
                first.clone()
            }
        })
        .collect();
    let layouts = [values.clone(), vec![first; values.len()], odd];
    Some(layouts.map(|layout| {
        if map {
            Node::Map(keys.iter().cloned().zip(layout).collect())
        } else {
            Node::Seq(layout)
        }
    }))
}

/// The node at `path` in a command input, through struct members.
fn node_at<'a>(input: &'a BTreeMap<String, Node>, path: &[String]) -> Option<&'a Node> {
    let (root, rest) = path.split_first()?;
    let mut node = input.get(root)?;
    for segment in rest {
        node = match node {
            Node::Map(members) => members.get(segment)?,
            _ => return None,
        };
    }
    Some(node)
}

/// `input` with the node at `path` replaced, where every step to it exists.
fn set_at(input: &mut BTreeMap<String, Node>, path: &[String], value: Node) {
    let Some((root, rest)) = path.split_first() else {
        return;
    };
    let Some(mut node) = input.get_mut(root) else {
        return;
    };
    for segment in rest {
        node = match node {
            Node::Map(members) => match members.get_mut(segment) {
                Some(member) => member,
                None => return,
            },
            _ => return,
        };
    }
    *node = value;
}

/// The inputs tried against `arrangement` for one command reading stored fields: the witness
/// search over the command's own guards, and over every row/input comparison grounded on the row.
fn inputs_for(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let grounded = grounded(ir, entity, &arrangement.settled, &hints(command));
    let mut guards: Vec<&Predicate> = command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    guards.extend(grounded.iter());
    let mut inputs =
        candidates(ir, command, &guards, Distinction::PLAIN).map_err(RefusalCause::NoWitness)?;
    // The row was arranged from plain witnesses, so a plain input may carry the very value the row
    // holds on every candidate — a text has no neighbour rule 3 could offer. A further witness
    // starts every leaf from another base value, which is the input a comparison with the row
    // needs for its other side. Only where such a comparison exists: every other command keeps the
    // candidates, and so the suites, it had.
    if !grounded.is_empty() {
        if let Ok(further) = candidates(ir, command, &guards, Distinction::further(1)) {
            inputs.extend(further);
        }
    }
    Ok(inputs)
}

/// [`inputs_for`], and each of them once more for every link comparison ([`links`]) on a row whose
/// link holds an arranged owner (beyond10x/ess#193): sent naming that owner, and — for each link
/// in turn — naming the [`other_owner`] instead. So an `!=` and an `==` between the link and the
/// input are each decided both ways, and a branch on either side is reached.
///
/// Only [`prepare`] offers these, because only it sends the owner named in their place: every other
/// arrangement keeps the candidates, and so the suites, it had.
fn linked_inputs(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let inputs = inputs_for(ir, command, entity, arrangement)?;
    Ok(naming_owners(
        ir,
        command,
        entity,
        arrangement,
        inputs,
        &BTreeMap::new(),
    ))
}

/// `inputs`, and each of them once more for every link comparison ([`links`]) on a row whose link
/// holds an arranged owner: sent naming that owner, and — for each link in turn — naming the
/// [`other_owner`] instead ([`linked_inputs`]). A caller offering these sends the owner each names
/// through `Setup::bound` ([`bind_links`]); the related-row search does (beyond10x/ess#271).
///
/// A link input `pins` holds is already bound to an arranged owner, and is sent naming that one
/// whatever the search chose ([`bind_pinned`]). So it is offered only on the side that owner is
/// on: the row's own owner's token where the row was filed under the pinned owner, else the
/// [`other_owner`]'s, which stands for the pinned owner — both differ from the row's own.
pub(super) fn naming_owners(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    mut inputs: Vec<BTreeMap<String, Node>>,
    pins: &BTreeMap<String, super::InstanceName>,
) -> Vec<BTreeMap<String, Node>> {
    let owned: Vec<(String, &super::InstanceName)> = links(ir, command, entity)
        .into_iter()
        .filter_map(
            |(sent, field)| match &arrangement.settled.get(&field)?.value {
                ScenarioValue::Instance { instance } => Some((sent, instance)),
                _ => None,
            },
        )
        .collect();
    if owned.is_empty() {
        return inputs;
    }
    let other = other_owner(ir, entity).map(|(_, name)| name);
    let mut more = Vec::new();
    for input in &inputs {
        let mut same = input.clone();
        for (sent, own) in &owned {
            let named = match pins.get(sent) {
                Some(held) if held != *own => other.as_ref(),
                _ => Some(*own),
            };
            if let Some(token) = named.and_then(|named| token(ir, command, sent, named)) {
                same.insert(sent.clone(), token);
            }
        }
        more.push(same.clone());
        for (sent, _) in owned.iter().filter(|(sent, _)| !pins.contains_key(sent)) {
            if let Some(token) = other
                .as_ref()
                .and_then(|other| token(ir, command, sent, other))
            {
                let mut apart = same.clone();
                apart.insert(sent.clone(), token);
                more.push(apart);
            }
        }
    }
    // A token that is not a value of the field's type (a text newtype narrower than the token) is
    // not a candidate: the comparison then stays undecided and is refused as it was.
    inputs.extend(
        more.into_iter()
            .filter(|input| flatten(ir, command, input).is_ok()),
    );
    inputs
}

/// [`reach_at`] over [`linked_inputs`]. Only an input `accept` takes of the row is offered.
fn reach_linked(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    accept: &dyn Fn(&Arrangement, &BTreeMap<String, Node>) -> bool,
    order: Order,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    for input in linked_inputs(ir, command, entity, arrangement)? {
        if selects(ir, command, entity, arrangement, &input, order)?
            .is_some_and(|branch| branch.name == outcome.name)
            && accept(arrangement, &input)
        {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// The owners the chosen `input` names through a link comparison, as `Setup::bound` sends them, and
/// the second owner arranged where it names that one and `present` says this scenario has not
/// arranged it yet (beyond10x/ess#193). An owner that cannot be arranged refuses the branch, naming
/// it: its side of the guard would otherwise go unwitnessed.
///
/// The second owner is given a row of `entity` of its own ([`row_under`]), so the owner the refused
/// side names holds rows too, just not this one: a target asking whether the named owner holds
/// *any* row, rather than this row, fails. Not under a `cardinality: one` owner relation where
/// `outcome`, the branch sent, writes the link from an input naming the second owner
/// ([`files_under`]): the branch then files this row under that owner, which would hold two, a
/// state the relation says no owner reaches.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_links(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    arrangement: &mut Arrangement,
    input: &BTreeMap<String, Node>,
    present: &mut bool,
) -> Result<BTreeMap<String, super::InstanceName>, RefusalCause> {
    let bound = linked(ir, command, entity, &arrangement.settled, input);
    let crowded = ir.owner_of(entity).is_some_and(|owned| {
        owned.relation.cardinality == Cardinality::One
            && other_owner(ir, entity)
                .is_some_and(|(_, other)| files_under(outcome, owned.via, &bound, &other))
    });
    if let Some((owner, other)) = other_owner(ir, entity)
        .filter(|(_, other)| !*present && bound.values().any(|named| named == other))
    {
        let initial = ir.entity(&owner).lifecycle.initial.clone();
        let arranged = super::arrange_first(
            ir,
            &owner,
            std::slice::from_ref(&initial),
            actors,
            OTHER_OWNER,
            &[entity],
        )
        .map_err(|reason| RefusalCause::InstanceRequired {
            entity: EntityRef::from(&owner),
            need: InstanceNeed::InState { state: initial },
            reason,
        })?;
        assert_eq!(
            arranged.instance, other,
            "an owner is named after its entity and distinction"
        );
        arrangement.steps.extend(arranged.steps);
        arrangement.source.extend(arranged.source);
        if let Some(row) = (!crowded)
            .then(|| row_under(ir, entity, actors, &arranged.instance, &arranged.state))
            .flatten()
        {
            arrangement.steps.extend(row.steps);
            arrangement.source.extend(row.source);
        }
        *present = true;
    }
    Ok(bound)
}

/// The link inputs of `command` over rows of `entity` ([`links`]) that `bound` already binds to an
/// arranged owner — the subject's own identity, or the owner of a subject being created — by field
/// (beyond10x/ess#271). A search deciding a link comparison sends these naming that owner: choosing
/// another would send one owner while the row was arranged against a second.
pub(super) fn link_pins(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    bound: &BTreeMap<String, super::InstanceName>,
) -> BTreeMap<String, super::InstanceName> {
    links(ir, command, entity)
        .into_iter()
        .filter_map(|(sent, _)| Some((sent.clone(), bound.get(&sent)?.clone())))
        .collect()
}

/// [`bind_links`] where `pins` holds link inputs already bound ([`link_pins`]): a pinned input the
/// search chose naming the [`other_owner`] is sent naming the pinned owner instead, which differs
/// from the row's own just as the other owner does, and the other owner is then not arranged for
/// it. The pinned owner is given a row of `entity` of its own ([`row_under`]), as the other owner
/// would have been, so a target asking whether the named owner holds *any* row fails. A pinned
/// input the search chose naming the row's own owner names the pinned one only where the row was
/// filed under it; the caller searched for such a row ([`naming_owners`]), and anything else is
/// refused as unarranged, never sent.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_pinned(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    arrangement: &mut Arrangement,
    input: &BTreeMap<String, Node>,
    pins: &BTreeMap<String, super::InstanceName>,
) -> Result<BTreeMap<String, super::InstanceName>, RefusalCause> {
    let other = other_owner(ir, entity).map(|(_, name)| name);
    let chosen = linked(ir, command, entity, &arrangement.settled, input);
    // The other owner is arranged only for a link no pin speaks for; its row is then named as the
    // pinned owner's would be, so the pinned owner is given none.
    let elsewhere = chosen
        .iter()
        .any(|(sent, named)| !pins.contains_key(sent) && Some(named) == other.as_ref());
    let mut present = !elsewhere;
    let mut bound = bind_links(
        ir,
        command,
        outcome,
        entity,
        actors,
        arrangement,
        input,
        &mut present,
    )?;
    let own = super::owner_of_row(ir, entity, &arrangement.settled).map(|(_, own)| own.clone());
    // The initial state of an owner that may hold several rows: only such a one is given a row of
    // its own beside the one the scenario reads.
    let shared = ir
        .owner_of(entity)
        .filter(|owned| owned.relation.cardinality == Cardinality::Many)
        .map(|owned| ir.entity(&owned.owner).lifecycle.initial.clone());
    let mut given = elsewhere;
    for (sent, held) in pins {
        let apart = own.as_ref() != Some(held);
        match chosen.get(sent) {
            Some(named) if apart && Some(named) == own.as_ref() => {
                return Err(super::related_guard::unarranged());
            }
            Some(named) if apart && !given && Some(named) == other.as_ref() => {
                if let Some(row) = shared
                    .as_ref()
                    .and_then(|state| row_under(ir, entity, actors, held, state))
                {
                    arrangement.steps.extend(row.steps);
                    arrangement.source.extend(row.source);
                    given = true;
                }
            }
            _ => {}
        }
        bound.insert(sent.clone(), held.clone());
    }
    Ok(bound)
}

/// Whether `outcome` writes the link field `via` from an input that `bound` sends as `owner`: the
/// branch files the row it names under that owner.
fn files_under(
    outcome: &ResolvedOutcome,
    via: &str,
    bound: &BTreeMap<String, super::InstanceName>,
    owner: &super::InstanceName,
) -> bool {
    outcome.sets.iter().any(|set| {
        set.target == via
            && matches!(
                &set.value,
                ResolvedPayloadValue::InputField { field, .. }
                    if bound.get(field) == Some(owner)
            )
    })
}

/// One row of `entity` created under the arranged `owner`, named under [`OTHER_OWNER`], by the
/// first creating branch that names the owner from its input and can be run — `None` where none
/// can. The owner holds no row before it, so a `cardinality: one` relation admits this one; the
/// caller ([`bind_links`]) does not ask for it where the branch it sends would then file a second
/// row under the same owner.
fn row_under(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    owner: &super::InstanceName,
    owner_state: &super::StateName,
) -> Option<Arrangement> {
    let via = ir.owner_of(entity)?.via;
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .find_map(|creator| {
            let field = creator
                .outcome
                .sets
                .iter()
                .find_map(|set| match &set.value {
                    // A path (ess/22, A4) reads inside a literal, which holds no owner's reference.
                    ResolvedPayloadValue::InputField { field, .. }
                        if set.target == via
                            && set.conversion.is_none()
                            && !ess_domain::command::input_path::is_path(field) =>
                    {
                        Some(field.clone())
                    }
                    _ => None,
                })?;
            let under = (
                field,
                Arrangement {
                    instance: owner.clone(),
                    state: owner_state.clone(),
                    steps: Vec::new(),
                    source: BTreeSet::new(),
                    settled: BTreeMap::new(),
                    unwritten: BTreeSet::new(),
                },
            );
            super::created_owned(
                ir,
                entity,
                creator,
                actors,
                OTHER_OWNER,
                &[],
                Some(&under),
                None,
            )
            .ok()
        })
}

/// How [`selects`] answers a row and input on which several guarded branches hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Order {
    /// No branch: the search goes on to a row and input on which exactly one is selected, so a
    /// witness never depends on the order a target reads its branches in. Every search runs this
    /// way first.
    Unique,
    /// The first declared, as the precedence order states (`docs/design/cross-record-and-stored-
    /// field-guards.md`, "The precedence order", steps 2 and 5; beyond10x/ess#217): only where no
    /// row and input selects the branch alone — `held-for-promotion: result == Healthy` beside a
    /// `when_subject:`, declared before `promoted: result == Healthy` (beyond10x/ess#278).
    FirstDeclared,
}

/// Which branch this command selects for the row `arrangement` holds and this input: the only one
/// selected, or under [`Order::FirstDeclared`] the first declared of several.
///
/// A row whose state no move of the command starts from is the wrong-state family's
/// ([`refusal_witness`]) and is not answered here, although its stored fields do select a guarded
/// branch there. An `Unknown` stored fact selects no branch, and never the default.
fn selects<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    input: &BTreeMap<String, Node>,
    order: Order,
) -> Result<Option<&'a ResolvedOutcome>, RefusalCause> {
    let wrong = ir
        .wrong_states(command)
        .get(&entity)
        .is_some_and(|states| states.contains(&arrangement.state));
    // A guarded branch is selected before `wrong_state` applies (beyond10x/ess#192), and one
    // whose predicate reads `state` (ess/18, #204) may name a state no move starts from: such a
    // row is answered here when that branch is the one it selects. Every other wrong-state row is
    // the wrong-state family's, as before.
    let reads_state = |branch: &ResolvedOutcome| {
        stored(&branch.condition).is_some_and(|predicate| reads_held_state(ir, entity, &predicate))
    };
    if wrong && !guarded(command).any(reads_state) {
        return Ok(None);
    }
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    let mut selected = Vec::new();
    for branch in guarded(command) {
        if let Some(predicate) = stored(&branch.condition) {
            match guard_truth_with(
                ir,
                entity,
                &arrangement.settled,
                &arrangement.unwritten,
                Some(&arrangement.state),
                &predicate,
                Some((command, input)),
            ) {
                Truth::True => {}
                Truth::False => continue,
                Truth::Unknown => return Ok(None),
            }
        }
        if let Some(guard) = input_guard(&branch.condition) {
            if !decides(&facts, &[guard], true)? {
                continue;
            }
        }
        selected.push(branch);
    }
    // An input-guarded refusal is taken before any accepting branch it overlaps (beyond10x/ess
    // #178), so where one is selected the accepting branches beside it are not.
    if selected
        .iter()
        .any(|branch| super::is_input_guarded_refusal(branch))
    {
        selected.retain(|branch| super::is_input_guarded_refusal(branch));
    }
    // `selected` is in declaration order; under [`Order::FirstDeclared`] the first answers.
    let pick = match (selected.as_slice(), order) {
        ([], _) => command.outcomes.iter().find(|branch| state_default(branch)),
        ([only], _) => Some(*only),
        ([first, ..], Order::FirstDeclared) => Some(*first),
        (_, Order::Unique) => None,
    };
    if wrong && !pick.is_some_and(reads_state) {
        return Ok(None);
    }
    Ok(pick.filter(|branch| {
        branch
            .subject
            .as_ref()
            .and_then(|subject| subject.effect.transition())
            .is_none_or(|transition| transition.from.contains(&arrangement.state))
    }))
}

/// Whether an input alone selects `outcome` of a command that reads no stored field.
pub(super) fn input_selects(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
) -> Result<bool, RefusalCause> {
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    let mut selected = Vec::new();
    for branch in guarded(command) {
        let Some(guard) = when(branch) else {
            continue;
        };
        if decides(&facts, &[guard], true)? {
            selected.push(branch);
        }
    }
    if selected
        .iter()
        .any(|branch| super::is_input_guarded_refusal(branch))
    {
        selected.retain(|branch| super::is_input_guarded_refusal(branch));
    }
    let pick = match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    };
    Ok(pick.is_some_and(|branch| branch.name == outcome.name))
}

/// The row and input a wrong-state scenario sends a command reading stored fields
/// (beyond10x/ess#173, #192): an input the moving branch `outcome`'s own input guard admits, where
/// it declares one, sent to a row in `state` on which no sibling branch is selected.
///
/// A guarded branch is selected in any state before `wrong_state` applies — the stored fields do
/// select there (`docs/design/cross-record-and-stored-field-guards.md`, *Wrong state*; Entity
/// Runtime orders guarded branches before the wrong-state one). So every sibling is missed:
///
/// * a branch guarded by its input alone — a plain `when:`, an input-guarded refusal (#178) — only
///   through its input;
/// * a branch guarded by the stored row alone — `held: when_subject: history == Pending` — only
///   through the row, which must decide its guard false;
/// * a branch that needs both — a `when:` beside a `when_subject:` — through either: its input half
///   refuted, or, where no candidate refutes every input half, its subject half decided false by
///   the row. `already-confirmed: history == Confirmed, token != ""` beside `token-required: token
///   == ""` is missed by `token != ""` on a row whose `history` is not `Confirmed`.
///
/// A sibling that moves the subject along a transition not starting from `state` needs no
/// refuting: selected or not, the move is the wrong-state answer.
///
/// A candidate refuting every input half is preferred, so a witness that already did stays the one
/// it was. The row is `arrangement` where it serves, and otherwise the first row in `state` the
/// declared drivers leave that does — found by the search [`prepare`] arranges subject-fact
/// branches with, steered by the command's own stored guards. Where no row serves, the scenario is
/// refused (ESS-SYNTH-003, naming the guards) rather than written to depend on evaluation order.
/// Every stored guard the witness relies on the row for is observed before the command, as
/// [`prepare`] observes it: the row is a fact the scenario is about, not one it assumes.
#[allow(clippy::too_many_arguments)]
pub(super) fn refusal_witness(
    ir: &EssIr,
    entity: &EntityHandle,
    state: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    arrangement: Arrangement,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<(Arrangement, BTreeMap<String, Node>), RefusalCause> {
    let (mut arrangement, input, relied) =
        match refusal_input(ir, entity, &arrangement, command, outcome, distinction) {
            Ok((input, relied)) => (arrangement, input, relied),
            Err(cause) => {
                let hints = hints(command);
                let found = search(
                    ir,
                    entity,
                    actors,
                    &hints,
                    Distinction::PLAIN,
                    "state",
                    |node| {
                        if &node.state != state {
                            return Ok(None);
                        }
                        Ok(refusal_input(ir, entity, node, command, outcome, distinction).ok())
                    },
                );
                let Ok((row, (input, relied))) = found else {
                    return Err(cause);
                };
                (row, input, relied)
            }
        };
    if !relied.is_empty() {
        let fields = read_fields(ir, entity, &relied);
        let (steps, view) = observe_fields(ir, entity, &fields, &arrangement)?;
        arrangement.steps.extend(steps);
        arrangement.source.insert(view.into());
    }
    Ok((arrangement, input))
}

/// The input [`refusal_witness`] sends to `arrangement`'s row, and the stored guards of the
/// siblings it misses through that row rather than through its input: every row-only sibling's,
/// and a mixed sibling's where its input half holds.
///
/// Where no candidate serves that way, a moving branch with a stored guard is tried as selected
/// first on the row, which an accepting sibling declared after it then need not be refuted against
/// (beyond10x/ess#278).
#[allow(clippy::too_many_lines)]
fn refusal_input(
    ir: &EssIr,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<(BTreeMap<String, Node>, Vec<Predicate>), RefusalCause> {
    let own: Vec<&Predicate> = input_guard(&outcome.condition).into_iter().collect();
    let branches: Vec<&ResolvedOutcome> = command
        .outcomes
        .iter()
        .filter(|other| other.name != outcome.name && !state_default(other))
        .filter(|other| input_guard(&other.condition).is_some())
        .collect();
    let siblings: Vec<&Predicate> = branches
        .iter()
        .filter_map(|other| input_guard(&other.condition))
        .collect();
    // A sibling selected by the stored row alone — `held: when_subject: history == Pending` — is
    // taken in any state before `wrong_state` applies, and no input refutes it: only the row can.
    // A sibling moving the subject along a transition that does not start from the row's state is
    // answered by `wrong_state` whether or not its guard selects it — Entity Runtime lowers no
    // held-state guard onto a stored-field branch, and a move from a state it does not leave is the
    // wrong-state answer — so the row need not refute it (`rushed: urgent and not fast` on the row
    // `rush` itself left in `Rushed`).
    let answered_by_state = |other: &ResolvedOutcome| {
        other
            .subject
            .as_ref()
            .and_then(|own| own.effect.transition())
            .is_some_and(|transition| !transition.from.contains(&arrangement.state))
    };
    let row_only: Vec<&ResolvedOutcome> = guarded(command)
        .filter(|other| other.name != outcome.name && input_guard(&other.condition).is_none())
        .filter(|other| stored(&other.condition).is_some() && !answered_by_state(other))
        .collect();
    // What the row must refute, where the input leaves it to the row: every row-only sibling's
    // stored guard, and a mixed sibling's where its input half holds.
    let halves: Vec<Predicate> = row_only
        .iter()
        .chain(&branches)
        .filter_map(|other| stored(&other.condition))
        .collect();
    // The row the stored guards read is the command's subject; a row of another entity decides
    // none of them.
    let on_row = common(command).is_some_and(|subject| &subject.entity == entity);
    let falsified = |branch: &ResolvedOutcome, input: &BTreeMap<String, Node>| {
        stored(&branch.condition).filter(|predicate| {
            on_row
                && guard_truth_with(
                    ir,
                    entity,
                    &arrangement.settled,
                    &arrangement.unwritten,
                    Some(&arrangement.state),
                    predicate,
                    Some((command, input)),
                ) == Truth::False
        })
    };
    let row_guards: &[Predicate] = if on_row { &halves } else { &[] };
    let inputs = refusal_candidates(ir, entity, arrangement, command, row_guards, distinction)?;
    let mut through_row = None;
    let mut lost_on_row = false;
    'candidates: for input in &inputs {
        let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
        if !decides(&facts, &own, true)? {
            continue;
        }
        let mut relied = Vec::new();
        for branch in &row_only {
            let Some(predicate) = falsified(branch, input) else {
                lost_on_row = true;
                continue 'candidates;
            };
            relied.push(predicate);
        }
        if decides(&facts, &siblings, false)? {
            return Ok((input.clone(), relied));
        }
        if through_row.is_some() {
            continue;
        }
        for branch in &branches {
            let Some(guard) = input_guard(&branch.condition) else {
                continue;
            };
            if decides(&facts, &[guard], false)? {
                continue;
            }
            if stored(&branch.condition).is_some() && answered_by_state(branch) {
                continue;
            }
            let Some(predicate) = falsified(branch, input) else {
                lost_on_row |= stored(&branch.condition).is_some();
                continue 'candidates;
            };
            relied.push(predicate);
        }
        through_row = Some((input.clone(), relied));
    }
    if let Some(found) = through_row {
        return Ok(found);
    }
    // Only where no candidate served above: `outcome` selected on this row answers before every
    // accepting sibling declared after it (beyond10x/ess#278), so such a sibling whose input guard
    // the candidate cannot refute — `promoted: result == Healthy` after `held-for-promotion:
    // result == Healthy` beside `when_subject:` — need not be refuted at all.
    //
    // A guarded `outcome` is selected where its input guard holds, which every candidate here
    // satisfies, and its stored guard, where it declares one, holds on the row; it then answers
    // with the wrong-state answer only where its own move does not start from the row's state. Every
    // input-guarded refusal is still refuted, since it is taken before any accepting branch
    // whatever the order (beyond10x/ess#178), and so is every sibling declared before `outcome`.
    let position = |branch: &ResolvedOutcome| {
        command
            .outcomes
            .iter()
            .position(|other| other.name == branch.name)
    };
    let answered_after =
        |other: &ResolvedOutcome| other.error.is_none() && position(other) > position(outcome);
    let own_stored = stored(&outcome.condition);
    // Only a branch with a stored guard: Entity Runtime lowers no held-state guard onto a
    // stored-field branch, so such a branch selected on a row in a state its move does not start
    // from is the wrong-state answer, as a sibling's is above (`answered_by_state`). A branch with a
    // stored guard is never the default, and [`refusal_witness`] is asked only for a state no move
    // of the command starts from, so `outcome` is guarded and its move does not start from the
    // row's state wherever this runs.
    let first_declared =
        on_row && own_stored.is_some() && branches.iter().any(|b| answered_after(b));
    if first_declared {
        debug_assert!(guarded(command).any(|branch| branch.name == outcome.name));
        debug_assert!(answered_by_state(outcome));
        'first_declared: for input in &inputs {
            let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
            if !decides(&facts, &own, true)? {
                continue;
            }
            let mut relied = Vec::new();
            if let Some(predicate) = &own_stored {
                if guard_truth_with(
                    ir,
                    entity,
                    &arrangement.settled,
                    &arrangement.unwritten,
                    Some(&arrangement.state),
                    predicate,
                    Some((command, input)),
                ) != Truth::True
                {
                    continue;
                }
                relied.push(predicate.clone());
            }
            for branch in &row_only {
                let Some(predicate) = falsified(branch, input) else {
                    continue 'first_declared;
                };
                relied.push(predicate);
            }
            for branch in &branches {
                let Some(guard) = input_guard(&branch.condition) else {
                    continue;
                };
                if decides(&facts, &[guard], false)? || answered_after(branch) {
                    continue;
                }
                if stored(&branch.condition).is_some() && answered_by_state(branch) {
                    continue;
                }
                let Some(predicate) = falsified(branch, input) else {
                    continue 'first_declared;
                };
                relied.push(predicate);
            }
            return Ok((input.clone(), relied));
        }
    }
    // A sibling whose input guard admits exactly what one of `own` admits is selected by every
    // candidate satisfying `own`, so it is no guard the input could refute. Declared after
    // `outcome` and accepting, the search above passed it over where `outcome` was selected on the
    // row; otherwise — declared before, or a refusal — it is what blocked every candidate.
    let twin = |branch: &ResolvedOutcome| {
        input_guard(&branch.condition)
            .is_some_and(|guard| own.iter().any(|mine| admit_alike(ir, command, mine, guard)))
    };
    // A twin whose stored guard the row refutes for every candidate was passed over by the search,
    // so it took nothing. Of the rest, one cause is named: an input-guarded refusal answers before
    // any accepting branch whatever the order, so the first refusal twin, and only where there is
    // none the first accepting twin declared before `outcome`.
    let passed_over = |branch: &ResolvedOutcome| {
        !inputs.is_empty()
            && inputs
                .iter()
                .all(|input| falsified(branch, input).is_some())
    };
    let earlier: Vec<&ResolvedOutcome> = branches
        .iter()
        .copied()
        .filter(|branch| twin(branch) && !answered_after(branch) && !passed_over(branch))
        .collect();
    let cause: Option<(&OutcomeName, &Predicate)> = earlier
        .iter()
        .find(|branch| branch.error.is_some())
        .or_else(|| earlier.first())
        .and_then(|branch| Some((&branch.name, input_guard(&branch.condition)?)));
    let later = branches
        .iter()
        .any(|branch| twin(branch) && answered_after(branch));
    let listed: Vec<&Predicate> = siblings
        .iter()
        .copied()
        .filter(|sibling| {
            !own.iter()
                .any(|mine| admit_alike(ir, command, mine, sibling))
        })
        .collect();
    let holding = own_stored
        .as_ref()
        .filter(|_| first_declared && later && earlier.is_empty());
    Err(refusal_unsatisfied(
        entity,
        (&own, &siblings, &listed),
        cause,
        holding,
        lost_on_row.then_some(halves.as_slice()),
        inputs.len().min(super::MAX_CANDIDATES),
    ))
}

/// Whether two input guards admit the same inputs — `result in [Healthy]` and `result == Healthy`
/// — answered `false` wherever that is not shown:
///
/// 1. equal once a one-value `in [x]` is written `== x` ([`one_value_equality`]);
/// 2. never, where either reads the bare truthiness of a leaf that is not a `Boolean`: a text is
///    falsy at values (`"false"`) no candidate is drawn at, so no candidate set shows it alike;
/// 3. decided alike at every value of the leaves they read, where every one is a top-level input
///    of a `Boolean` or enum type, or an `Optional` of one, absent included ([`finite_alike`]);
/// 4. decided alike by every candidate over both, where those candidates cover every region the
///    guards' literals divide the input into ([`crate::witness::exhausts`]).
fn admit_alike(ir: &EssIr, command: &ResolvedCommand, left: &Predicate, right: &Predicate) -> bool {
    let (left, right) = (&one_value_equality(left), &one_value_equality(right));
    if left == right {
        return true;
    }
    let mut truthy = Vec::new();
    truthy_paths(left, &mut truthy);
    truthy_paths(right, &mut truthy);
    if truthy.iter().any(|path| {
        !finite_leaf(ir, command, path).is_some_and(|values| {
            values
                .iter()
                .all(|value| matches!(value, Some(Node::Bool(_))))
        })
    }) {
        return false;
    }
    if let Some(alike) = finite_alike(ir, command, left, right) {
        return alike;
    }
    let guards = [left, right];
    if !crate::witness::exhausts(ir, command, &guards) {
        return false;
    }
    let Ok(mut inputs) = candidates(ir, command, &guards, Distinction::PLAIN) else {
        return false;
    };
    if let Ok(Some(between)) =
        crate::witness::candidates_between(ir, command, &guards, Distinction::PLAIN)
    {
        inputs.extend(between);
    }
    !inputs.is_empty()
        && inputs.iter().all(|input| {
            let Ok(facts) = flatten(ir, command, input) else {
                return false;
            };
            match (
                decides(&facts, &[left], true),
                decides(&facts, &[right], true),
            ) {
                (Ok(one), Ok(other)) => one == other,
                _ => false,
            }
        })
}

/// `guard` with every one-value `in [x]` written `== x`, so two spellings of one comparison compare
/// equal.
fn one_value_equality(guard: &Predicate) -> Predicate {
    match guard {
        Predicate::AnyOf { path, values } if values.len() == 1 => Predicate::Compare {
            kind: ess_primitives::predicate::CompareKind::Value,
            left: Operand::Fact(path.clone()),
            op: CompareOp::Eq,
            right: Operand::Literal(values[0].clone()),
        },
        Predicate::All(children) => {
            Predicate::All(children.iter().map(one_value_equality).collect())
        }
        Predicate::Any(children) => {
            Predicate::Any(children.iter().map(one_value_equality).collect())
        }
        Predicate::Not(inner) => Predicate::Not(Box::new(one_value_equality(inner))),
        other => other.clone(),
    }
}

/// Every leaf `guard` reads the bare truthiness of.
fn truthy_paths<'p>(guard: &'p Predicate, out: &mut Vec<&'p FactPath>) {
    match guard {
        Predicate::Truthy(path) => out.push(path),
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                truthy_paths(child, out);
            }
        }
        Predicate::Not(inner) => truthy_paths(inner, out),
        _ => {}
    }
}

/// Every value a top-level input leaf can take — `None` for absent where it is `Optional` — where
/// its type is a `Boolean` or an enum, or an `Optional` of one; otherwise `None`.
fn finite_leaf(
    ir: &EssIr,
    command: &ResolvedCommand,
    path: &FactPath,
) -> Option<Vec<Option<Node>>> {
    use ess_compiler::ir::{ResolvedBody, ResolvedTypeRef};
    let [name] = path.segments() else {
        return None;
    };
    let field = command.input.iter().find(|field| &field.name == name)?;
    let (type_ref, optional) = match &field.type_ref {
        ResolvedTypeRef::Optional { of } => (of.as_ref(), true),
        other => (other, false),
    };
    let mut values: Vec<Option<Node>> = match type_ref {
        ResolvedTypeRef::Primitive {
            name: ess_domain::types::Primitive::Boolean,
        } => vec![Some(Node::Bool(true)), Some(Node::Bool(false))],
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Enum { variants } => variants
                .iter()
                .map(|variant| Some(Node::Text(variant.name.clone())))
                .collect(),
            _ => return None,
        },
        _ => return None,
    };
    if optional {
        values.push(None);
    }
    Some(values)
}

/// Whether two guards over finite top-level input leaves only ([`finite_leaf`]) decide alike at
/// every combination of those leaves' values, absence included, on one candidate's other fields.
/// `None` where they read another leaf, the combinations exceed the candidate bound, or a
/// combination cannot be decided.
fn finite_alike(
    ir: &EssIr,
    command: &ResolvedCommand,
    left: &Predicate,
    right: &Predicate,
) -> Option<bool> {
    let mut paths = left.fact_paths();
    for path in right.fact_paths() {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    let leaves: Vec<(&FactPath, Vec<Option<Node>>)> = paths
        .into_iter()
        .map(|path| Some((path, finite_leaf(ir, command, path)?)))
        .collect::<Option<_>>()?;
    let combinations = leaves.iter().try_fold(1_usize, |product, (_, values)| {
        product.checked_mul(values.len())
    })?;
    if leaves.is_empty() || combinations > super::MAX_CANDIDATES {
        return None;
    }
    let base = candidates(ir, command, &[left, right], Distinction::PLAIN)
        .ok()?
        .into_iter()
        .next()?;
    for index in 0..combinations {
        let mut input = base.clone();
        let mut rest = index;
        for (path, values) in &leaves {
            let name = &path.segments()[0];
            match &values[rest % values.len()] {
                Some(value) => {
                    input.insert(name.clone(), value.clone());
                }
                None => {
                    input.remove(name);
                }
            }
            rest /= values.len();
        }
        let facts = flatten(ir, command, &input).ok()?;
        let one = decides(&facts, &[left], true).ok()?;
        let other = decides(&facts, &[right], true).ok()?;
        if one != other {
            return Some(false);
        }
    }
    Some(true)
}

/// The candidates [`refusal_input`] tries: over every branch's input guard, and over every stored
/// guard in `row_guards` that compares the row with the input, grounded on this row. Such a guard
/// — `fence != input.publication.expected_fence` — is refuted by an input naming the value the row
/// holds, which no plain witness does (beyond10x/ess#234); grounded, its literals are the values
/// that input is tried at, as [`inputs_for`] tries them.
fn refusal_candidates(
    ir: &EssIr,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    command: &ResolvedCommand,
    row_guards: &[Predicate],
    distinction: Distinction,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let grounded = grounded(ir, entity, &arrangement.settled, row_guards);
    let guards: Vec<&Predicate> = command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .chain(&grounded)
        .collect();
    candidates(ir, command, &guards, distinction).map_err(RefusalCause::NoWitness)
}

/// ESS-SYNTH-003 for [`refusal_input`]: the input guards no candidate satisfied, and — where some
/// candidate was lost only to the row — the stored guards the row had to refute.
///
/// A sibling guard admitting what one of `own` admits is not among the input guards named after
/// `none of:` (`listed` leaves it out): it holds of every candidate that satisfies `own`, so
/// `c and none of: c, …` would state a contradiction rather than what was searched for
/// (beyond10x/ess#278). Instead:
///
/// * the one twin selected first — the first refusal twin, or else the first accepting twin
///   declared before the branch that the row did not refute — is named as what took every such
///   input (`cause`);
/// * where only twins declared after it were passed over, and only on a row selecting the branch
///   first, the diagnostic names that row (`holding`, the branch's stored guard).
fn refusal_unsatisfied(
    entity: &EntityHandle,
    (own, siblings, listed): (&[&Predicate], &[&Predicate], &[&Predicate]),
    cause: Option<(&OutcomeName, &Predicate)>,
    holding: Option<&Predicate>,
    halves: Option<&[Predicate]>,
    tried: usize,
) -> RefusalCause {
    let mut named = own.to_vec();
    named.extend(siblings.iter().copied());
    let mut rendered = match (own.is_empty(), listed.is_empty()) {
        (true, true) => String::new(),
        (false, true) => super::rendered(own, true),
        (true, false) => super::rendered(listed, false),
        (false, false) => format!(
            "{} and {}",
            super::rendered(own, true),
            super::rendered(listed, false)
        ),
    };
    if let Some((name, guard)) = cause {
        rendered = format!("{rendered}, every such input taken first by `{name}` ({guard})");
    }
    if let Some(predicate) = holding {
        named.push(predicate);
        let row = format!("a `{entity}` row in this state holding {predicate}");
        rendered = if rendered.is_empty() {
            row
        } else {
            format!("{rendered}, on {row}")
        };
    }
    if let Some(halves) = halves {
        named.extend(halves.iter());
        let row = format!(
            "a `{entity}` row in this state refuting every one of: {}",
            halves
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        rendered = if rendered.is_empty() {
            row
        } else {
            format!("{rendered}, on {row}")
        };
    }
    super::unsatisfied(&named, rendered, tried)
}

/// The stored fields `outcome`'s `sets:` fills from an input field, and the input field each reads.
///
/// A conversion says two types may meet and not what it computes, so a field crossing one is not a
/// field an input can be chosen for; a literal is a fixed point the search takes as it is.
fn mapped(outcome: &ResolvedOutcome) -> BTreeMap<&str, &str> {
    outcome
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
        .filter_map(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. } => {
                Some((set.target.as_str(), field.as_str()))
            }
            _ => None,
        })
        .collect()
}

/// The inputs `creator` creates a row of `entity` with, chosen toward the stored-field guards of
/// every command moving it: a row created with one of them is one such a move can be taken on
/// without searching for another row (beyond10x/ess#279). Empty where no move reads stored fields,
/// or the creating branch maps none of the fields they read.
pub(super) fn toward_moves(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
) -> Vec<BTreeMap<String, Node>> {
    let all = ir.drivers();
    let mut read: Vec<Predicate> = Vec::new();
    for driver in all.get(entity).map_or(&[][..], Vec::as_slice) {
        if driver.effect.transition().is_none() || !uses(driver.command) {
            continue;
        }
        for hint in hints(driver.command) {
            if !read.contains(&hint) {
                read.push(hint);
            }
        }
    }
    if read.is_empty() {
        return Vec::new();
    }
    hinted(ir, entity, creator, &read)
        .ok()
        .flatten()
        .unwrap_or_default()
}

/// The candidate inputs for one arranging branch, varied toward the stored-field goal.
///
/// Every hint is translated through the branch's `sets:` mappings onto its own input, and the
/// translations are handed to the witness search beside the command's own guards: the literals the
/// guard writes become the values the mapped input fields are tried at. `None` when the branch
/// maps none of the fields the hints read, because then no choice of its input moves the row.
fn hinted(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    hints: &[Predicate],
) -> Result<Option<Vec<BTreeMap<String, Node>>>, RefusalCause> {
    let mapping = mapped(driver.outcome);
    // A comparison with the command's input (ess/15) is decided by the input the branch under
    // test sends, not by the row an arranging branch leaves, so it steers nothing here.
    let hints: Vec<Predicate> = hints
        .iter()
        .map(|hint| without_input(ir, entity, hint))
        .collect();
    let translated: Vec<Predicate> = hints
        .iter()
        .filter(|hint| {
            hint.fact_paths()
                .iter()
                .any(|path| mapping.contains_key(path.namespace()))
        })
        .map(|hint| {
            map_paths(
                hint,
                &|path: &FactPath| match mapping.get(path.namespace()) {
                    Some(input) => {
                        // A path (ess/22, A4) is its segments.
                        let mut segments: Vec<String> =
                            input.split('.').map(str::to_owned).collect();
                        segments.extend(path.segments()[1..].iter().cloned());
                        FactPath::from_segments(segments)
                    }
                    None => path.clone(),
                },
            )
        })
        .collect();
    if translated.is_empty() {
        return Ok(None);
    }
    let mut guards: Vec<&Predicate> = driver
        .command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    guards.extend(translated.iter());
    candidates(ir, driver.command, &guards, Distinction::PLAIN)
        .map(Some)
        .map_err(RefusalCause::NoWitness)
}

/// Every leaf of a predicate: the comparisons, tests and quantifiers its connectives join.
fn leaves(predicate: &Predicate, out: &mut Vec<Predicate>) {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                leaves(child, out);
            }
        }
        Predicate::Not(inner) => leaves(inner, out),
        Predicate::Always | Predicate::Never => {}
        other => out.push(other.clone()),
    }
}

/// A comparison leaf rewritten as equality with its own literal: true where the row sits on the
/// boundary the guard writes.
fn on_literal(leaf: &Predicate) -> Option<Predicate> {
    let Predicate::Compare { left, right, .. } = leaf else {
        return None;
    };
    match (left, right) {
        (Operand::Fact(_), Operand::Literal(_)) | (Operand::Literal(_), Operand::Fact(_)) => {
            Some(Predicate::Compare {
                kind: ess_primitives::predicate::CompareKind::Value,
                left: left.clone(),
                op: CompareOp::Eq,
                right: right.clone(),
            })
        }
        _ => None,
    }
}

/// How far from a literal its guard compares a stored counter with the search follows the
/// counter's value, in the counter's own units (beyond10x/ess#226).
const COUNTER_REACH: i64 = 16;

/// What a refusal says where a counter the guards read lay beyond [`COUNTER_REACH`] of every
/// literal they compare it with on some row the search left.
fn beyond_reach(fields: &[String]) -> String {
    format!(
        "a stored counter ({}) is followed only within {COUNTER_REACH} of each literal its guard \
         compares it with",
        fields
            .iter()
            .map(|field| format!("`{field}`"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Whether a refusal is the search's own past [`COUNTER_REACH`].
pub(super) fn is_beyond_reach(cause: &RefusalCause) -> bool {
    matches!(cause, RefusalCause::GuardUnsatisfiable { predicate, .. }
        if predicate.contains("is followed only within"))
}

/// One stored counter of an entity (beyond10x/ess#226): the amounts an arranging branch moves it by
/// with `{increment: n}` (none zero), and the number literals a branch sets it to outright — `None`
/// where some branch sets it to anything else (an input, a generated value), so where a run of it
/// starts is not known.
#[derive(Debug, Clone, Default)]
pub(super) struct Counter {
    amounts: Vec<Number>,
    starts: Option<Vec<Number>>,
}

/// The stored counters of one entity: every field an arranging branch moves with `{increment: n}`.
pub(super) fn counters(ir: &EssIr, entity: &EntityHandle) -> BTreeMap<String, Counter> {
    let all = ir.drivers();
    let mut out: BTreeMap<String, Counter> = BTreeMap::new();
    for driver in all.get(entity).map_or(&[][..], Vec::as_slice) {
        for set in &driver.outcome.sets {
            let counter = out.entry(set.target.clone()).or_insert_with(|| Counter {
                amounts: Vec::new(),
                starts: Some(Vec::new()),
            });
            match &set.value {
                ResolvedPayloadValue::Increment { by } => {
                    if let Some(by) = Number::decimal_literal(by).filter(|by| by.get() != 0.0) {
                        if !counter.amounts.contains(&by) {
                            counter.amounts.push(by);
                        }
                    }
                }
                ResolvedPayloadValue::Literal { value } => {
                    match (Number::decimal_literal(value), counter.starts.as_mut()) {
                        (Some(start), Some(starts)) if !starts.contains(&start) => {
                            starts.push(start);
                        }
                        (Some(_), _) => {}
                        (None, _) => counter.starts = None,
                    }
                }
                _ => counter.starts = None,
            }
        }
    }
    out.retain(|_, counter| !counter.amounts.is_empty());
    out
}

/// The most values [`reachable`] enumerates before it gives up and the limit's sides are taken one
/// smallest step either side of it.
const MAX_REACHABLE: usize = 4096;

/// Every value `counter` holds on some run that lies within [`COUNTER_REACH`] of `literal`, or
/// between a start and that window, one widest step either side: the starts, and each start moved
/// by any sequence of its amounts, never leaving that span. `None` where the starts are not known
/// or any required span arithmetic is unrepresentable, or the span holds more than
/// [`MAX_REACHABLE`] values.
fn reachable(counter: &Counter, literal: Number) -> Option<BTreeSet<Number>> {
    let starts = counter.starts.as_ref()?;
    if starts.is_empty() {
        return Some(BTreeSet::new());
    }
    let reach = Number::from(COUNTER_REACH);
    let widest = counter
        .amounts
        .iter()
        .map(|by| Some((*by).max(negated(*by)?)))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max()?;
    let lower_window = literal.checked_add(negated(reach)?)?;
    let upper_window = literal.checked_add(reach)?;
    let low = starts
        .iter()
        .copied()
        .chain([lower_window])
        .min()?
        .checked_add(negated(widest)?)?;
    let high = starts
        .iter()
        .copied()
        .chain([upper_window])
        .max()?
        .checked_add(widest)?;
    let mut seen = BTreeSet::new();
    let mut frontier = starts.clone();
    while let Some(value) = frontier.pop() {
        if value < low || value > high || !seen.insert(value) {
            continue;
        }
        if seen.len() > MAX_REACHABLE {
            return None;
        }
        frontier.extend(
            counter
                .amounts
                .iter()
                .filter_map(|by| value.checked_add(*by)),
        );
    }
    Some(seen)
}

/// How far `value` lies from `literal`, or `None` where the difference overflows.
fn distance(value: Number, literal: Number) -> Option<Number> {
    let apart = value.checked_add(negated(literal)?)?;
    Some(apart.max(negated(apart)?))
}

/// The side of a counter limit a row is pinned at: the value, how the comparison decides there,
/// and — where the value lies past [`COUNTER_REACH`] — why no row the search follows holds it.
type Edge = (Number, bool, Option<String>);

/// One side of a limit: the value pinned there and why it lies past the reach, or `None` where no
/// run holds a value on that side.
type Pinned = Option<(Number, Option<String>)>;

/// The counter values on either side of a limit (beyond10x/ess#226), with how the comparison
/// decides there: on each side the value nearest the limit that a run of the counter holds
/// ([`reachable`]). `retries >= 3` moved by one from 0: `2` false and `3` true; moved by two: `2`
/// false and `4` true; `retries == 3` moved by one: `2` and `4` false, `3` true. A side no run holds
/// a value on (`retries >= 0` from 0, below it) has no edge; a side whose nearest value lies past
/// [`COUNTER_REACH`] carries a cause naming the step and the limit. Where the starts are unknown the
/// edges are one smallest step either side of the literal, as any value may be held.
fn limit_edges(field: &str, op: CompareOp, literal: Number, counter: &Counter) -> Vec<Edge> {
    let Some(values) = reachable(counter, literal) else {
        let Some(step) = counter
            .amounts
            .iter()
            .filter_map(|by| Some((*by).max(negated(*by)?)))
            .min()
        else {
            return Vec::new();
        };
        return stepped_edges(op, literal, step)
            .into_iter()
            .map(|(value, decided)| (value, decided, None))
            .collect();
    };
    let reach = Number::from(COUNTER_REACH);
    let pin = |found: Option<&Number>, side: &str| -> Pinned {
        let value = *found?;
        if distance(value, literal).is_some_and(|apart| apart <= reach) {
            return Some((value, None));
        }
        Some((
            value,
            Some(format!(
                "`{field}` moves by {} from {}, so the nearest value it holds {side} the limit \
             {literal} is {value}; {}",
                list(&counter.amounts),
                list(counter.starts.as_deref().unwrap_or_default()),
                beyond_reach(&[field.to_owned()]),
            )),
        ))
    };
    let below = |strict: bool| {
        pin(
            values
                .iter()
                .rev()
                .find(|value| **value < literal || (!strict && **value == literal)),
            if strict { "below" } else { "at or below" },
        )
    };
    let above = |strict: bool| {
        pin(
            values
                .iter()
                .find(|value| **value > literal || (!strict && **value == literal)),
            if strict { "above" } else { "at or above" },
        )
    };
    let sides: Vec<(Pinned, bool)> = match op {
        CompareOp::Ge => vec![(below(true), false), (above(false), true)],
        CompareOp::Gt => vec![(below(false), false), (above(true), true)],
        CompareOp::Le => vec![(below(false), true), (above(true), false)],
        CompareOp::Lt => vec![(below(true), true), (above(false), false)],
        CompareOp::Eq | CompareOp::Ne if values.contains(&literal) => {
            let at = op == CompareOp::Eq;
            vec![
                (below(true), !at),
                (Some((literal, None)), at),
                (above(true), !at),
            ]
        }
        CompareOp::Eq | CompareOp::Ne => Vec::new(),
    };
    sides
        .into_iter()
        .filter_map(|(pinned, decided)| pinned.map(|(value, past)| (value, decided, past)))
        .collect()
}

/// Numbers as a list for a refusal.
fn list(values: &[Number]) -> String {
    values
        .iter()
        .map(|value| value.exact_text())
        .collect::<Vec<_>>()
        .join(" or ")
}

/// A comparison leaf between a stored counter and a number literal, as `field op literal`.
fn counter_leaf(
    leaf: &Predicate,
    counters: &BTreeMap<String, Counter>,
) -> Option<(String, CompareOp, Number)> {
    let Predicate::Compare {
        left, op, right, ..
    } = leaf
    else {
        return None;
    };
    let (path, op, value) = match (left, right) {
        (Operand::Fact(path), Operand::Literal(value)) => (path, *op, value),
        (Operand::Literal(value), Operand::Fact(path)) => (
            path,
            match op {
                CompareOp::Lt => CompareOp::Gt,
                CompareOp::Le => CompareOp::Ge,
                CompareOp::Gt => CompareOp::Lt,
                CompareOp::Ge => CompareOp::Le,
                other => *other,
            },
            value,
        ),
        _ => return None,
    };
    (path.segments().len() == 1 && counters.contains_key(path.namespace()))
        .then_some(())
        .and(value.as_number())
        .map(|number| (path.namespace().to_owned(), op, number))
}

/// `value` with its sign turned.
fn negated(value: Number) -> Option<Number> {
    let text = value.exact_text();
    Number::decimal_literal(
        text.strip_prefix('-')
            .map_or_else(|| format!("-{text}"), str::to_owned)
            .as_str(),
    )
}

/// Whether `value op literal` holds.
fn compares(value: Number, op: CompareOp, literal: Number) -> bool {
    match op {
        CompareOp::Eq => value == literal,
        CompareOp::Ne => value != literal,
        CompareOp::Lt => value < literal,
        CompareOp::Le => value <= literal,
        CompareOp::Gt => value > literal,
        CompareOp::Ge => value >= literal,
    }
}

/// [`limit_edges`] where any value may be held: of the literal and one step either side of it,
/// every value whose neighbour decides the comparison the other way, with how it decides there.
fn stepped_edges(op: CompareOp, literal: Number, step: Number) -> Vec<(Number, bool)> {
    let values: Vec<Number> = [
        negated(step).and_then(|down| literal.checked_add(down)),
        Some(literal),
        literal.checked_add(step),
    ]
    .into_iter()
    .flatten()
    .collect();
    let truths: Vec<bool> = values
        .iter()
        .map(|value| compares(*value, op, literal))
        .collect();
    (0..values.len())
        .filter(|at| {
            (*at > 0 && truths[at - 1] != truths[*at])
                || truths.get(at + 1).is_some_and(|next| *next != truths[*at])
        })
        .map(|at| (values[at], truths[at]))
        .collect()
}

/// The stored counters the hints compare with number literals, and the literals: the values the
/// search follows a counter at, where the rows every raise leaves would otherwise decide the guards
/// alike and be one node (beyond10x/ess#226). Each field carries its literals and the amounts it is
/// moved by.
#[derive(Debug, Default)]
struct Follow {
    fields: Vec<(String, Vec<Number>, Vec<Number>)>,
}

impl Follow {
    fn new(ir: &EssIr, entity: &EntityHandle, hints: &[Predicate]) -> Self {
        let counters = counters(ir, entity);
        if counters.is_empty() {
            return Self::default();
        }
        let mut literals: BTreeMap<String, Vec<Number>> = BTreeMap::new();
        for hint in hints {
            let mut all = Vec::new();
            leaves(hint, &mut all);
            for leaf in &all {
                if let Some((field, _, literal)) = counter_leaf(leaf, &counters) {
                    let held = literals.entry(field).or_default();
                    if !held.contains(&literal) {
                        held.push(literal);
                    }
                }
            }
        }
        Self {
            fields: literals
                .into_iter()
                .map(|(field, literals)| {
                    let amounts = counters
                        .get(&field)
                        .map(|counter| counter.amounts.clone())
                        .unwrap_or_default();
                    (field, literals, amounts)
                })
                .collect(),
        }
    }

    /// Whether no counter is followed.
    fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// The followed counters, in name order.
    fn names(&self) -> Vec<String> {
        self.fields
            .iter()
            .map(|(field, ..)| field.clone())
            .collect()
    }

    /// Each followed counter's value on the row, where it lies within [`COUNTER_REACH`] of a
    /// literal — and whether some counter did not while a raise could still move it toward one.
    ///
    /// Only such a row stands for values the search needed and did not follow: a counter past every
    /// literal in the direction every amount moves it only moves farther off, so the rows the
    /// search collapses there hold no value any limit is decided at (beyond10x/ess#226).
    fn key(&self, settled: &BTreeMap<String, super::Determined>) -> (Vec<Option<String>>, bool) {
        let reach = Number::from(COUNTER_REACH);
        let zero = Number::from(0_i64);
        let mut beyond = false;
        let key = self
            .fields
            .iter()
            .map(|(field, literals, amounts)| {
                let ScenarioValue::Literal {
                    value: Node::Number(value),
                } = &settled.get(field)?.value
                else {
                    return None;
                };
                let near = literals
                    .iter()
                    .any(|literal| distance(*value, *literal).is_some_and(|apart| apart <= reach));
                let toward = literals.iter().any(|literal| {
                    amounts.iter().any(|by| {
                        (*value < *literal && *by > zero) || (*value > *literal && *by < zero)
                    })
                });
                beyond |= !near && toward;
                near.then(|| value.exact_text())
            })
            .collect();
        (key, beyond)
    }

    /// Carries the value every `{increment: n}` of `driver` leaves in a followed counter, read
    /// against the row before the act: an arranging act otherwise determines nothing there.
    fn raise(
        &self,
        ir: &EssIr,
        driver: &Driver<'_>,
        before: &BTreeMap<String, super::Determined>,
        after: &mut BTreeMap<String, super::Determined>,
    ) {
        if self.fields.is_empty() {
            return;
        }
        let raised = super::settled(ir, driver.outcome, &BTreeMap::new(), before);
        for set in &driver.outcome.sets {
            if matches!(set.value, ResolvedPayloadValue::Increment { .. })
                && self.fields.iter().any(|(field, ..)| field == &set.target)
            {
                if let Some(value) = raised.get(&set.target) {
                    after.insert(set.target.clone(), value.clone());
                }
            }
        }
    }
}

/// How the row decides every hint and every leaf: the search node, and the ranking of goals.
///
/// `counters` is each followed counter's value ([`Follow`]), so the rows a repeated raise leaves on
/// the way to a limit are nodes of their own.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Profile {
    state: String,
    hints: Vec<Decided>,
    leaves: Vec<Decided>,
    on_literal: Vec<bool>,
    counters: Vec<Option<String>>,
    /// For each [`elementwise`] quantifier, which of the collection's values repeat an earlier one:
    /// rows [`spread`] writes decide every hint alike and differ only here (beyond10x/ess#240).
    shapes: Vec<Vec<usize>>,
    /// For each `distinct` over a stored list ([`stored_distincts`]), whether the row holds two or
    /// more elements there: a decisive row and a vacuous one decide every hint alike and differ
    /// only here (`docs/design/expression-family-source22.md`, `distinct`).
    decisive: Vec<bool>,
}

/// One three-valued answer, ordered so a search node can be keyed on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Decided {
    Unknown,
    False,
    True,
}

fn code(truth: Truth) -> Decided {
    match truth {
        Truth::True => Decided::True,
        Truth::False => Decided::False,
        Truth::Unknown => Decided::Unknown,
    }
}

fn profile(
    ir: &EssIr,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    hints: &[Predicate],
    follow: &Follow,
) -> Profile {
    let mut all = Vec::new();
    for hint in hints {
        leaves(hint, &mut all);
    }
    let truth = |predicate: &Predicate| {
        row_truth(
            ir,
            entity,
            &arrangement.settled,
            &arrangement.unwritten,
            Some(&arrangement.state),
            predicate,
        )
    };
    Profile {
        state: arrangement.state.to_string(),
        hints: hints.iter().map(|hint| code(truth(hint))).collect(),
        leaves: all.iter().map(|leaf| code(truth(leaf))).collect(),
        on_literal: all
            .iter()
            .map(|leaf| on_literal(leaf).is_some_and(|eq| truth(&eq) == Truth::True))
            .collect(),
        counters: follow.key(&arrangement.settled).0,
        shapes: elementwise(ir, entity, hints)
            .iter()
            .filter_map(|leaf| match leaf {
                Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                    held_node(&arrangement.settled, &[], &quantified.over)
                }
                _ => None,
            })
            .map(|collection| {
                let values: Vec<Node> = match collection {
                    Node::Map(entries) => entries.into_values().collect(),
                    Node::Seq(items) => items,
                    _ => Vec::new(),
                };
                values
                    .iter()
                    .map(|value| values.iter().position(|other| other == value).unwrap_or(0))
                    .collect()
            })
            .collect(),
        decisive: stored_distincts(ir, entity, hints)
            .iter()
            .map(|over| {
                matches!(
                    held_node(&arrangement.settled, &[], over),
                    Some(Node::Seq(items)) if items.len() >= 2
                )
            })
            .collect(),
    }
}

/// The stored lists a `distinct` among `hints` reads outside any quantifier, in leaf order.
fn stored_distincts(ir: &EssIr, entity: &EntityHandle, hints: &[Predicate]) -> Vec<FactPath> {
    let declared = ir.entity(entity);
    let mut found = Vec::new();
    for hint in hints {
        for (distinct, scope) in hint.distincts() {
            if scope.is_empty()
                && input_path(&distinct.over).is_none()
                && declared
                    .fields
                    .iter()
                    .any(|field| field.name == distinct.over.namespace())
            {
                found.push(distinct.over.clone());
            }
        }
    }
    found
}

/// Refuses a row that reaches its branch with a stored list a `distinct` among `hints` reads
/// holding fewer than two elements, where no row of the same depth held more: a `distinct` over
/// none or one element holds vacuously, and a target that compares nothing passes it
/// (`docs/design/expression-family-source22.md`, `distinct`). The named no-witness refusal, so the
/// branch is reported rather than witnessed by a list that decides nothing.
fn vacuous(
    ir: &EssIr,
    entity: &EntityHandle,
    row: &Arrangement,
    hints: &[Predicate],
) -> Result<(), RefusalCause> {
    for over in stored_distincts(ir, entity, hints) {
        if !matches!(
            held_node(&row.settled, &[], &over),
            Some(Node::Seq(items)) if items.len() >= 2
        ) {
            return Err(RefusalCause::NoWitness(WitnessGap {
                path: over.to_string(),
                type_ref: entity.to_string(),
                reason: "holds fewer than two elements in every row the arrangement leaves for \
                         this branch, so a `distinct` over it would be decided only vacuously",
            }));
        }
    }
    Ok(())
}

/// How close a row sits to the guards: satisfied leaves first, then leaves on their own literal,
/// then stored lists a `distinct` reads that hold two or more elements, so the branch a `distinct`
/// lets through is arranged decisively rather than over a list of none or one.
fn score(profile: &Profile) -> (usize, usize, usize) {
    (
        profile
            .leaves
            .iter()
            .filter(|truth| **truth == Decided::True)
            .count(),
        profile.on_literal.iter().filter(|on| **on).count(),
        profile.decisive.iter().filter(|held| **held).count(),
    )
}

/// The rows the creating branch can leave: its plain witness first, then one per input chosen
/// toward the hints.
fn creations(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    (arranging, under): (&[&EntityHandle], Under<'_>),
) -> Result<Vec<Arrangement>, RefusalCause> {
    let required = |reason| RefusalCause::InstanceRequired {
        entity: EntityRef::from(entity),
        need: InstanceNeed::Updates,
        reason,
    };
    // Under an owner the scenario already holds ([`search_under`]); a creator that cannot file the
    // row there leaves none. An owner known to hold no row yet takes one whatever the relation's
    // cardinality; any other only under `cardinality: many`.
    let owner = match under {
        Some((owner, empty)) => {
            let filed = if empty {
                super::filed_under(ir, entity, creator, owner)
            } else {
                super::under_owner(ir, entity, creator, owner)
            };
            match filed {
                Some(owner) => Some(owner),
                None => return Ok(Vec::new()),
            }
        }
        None => None,
    };
    let created = |ir: &EssIr,
                   entity: &EntityHandle,
                   creator: &Driver<'_>,
                   actors: &BTreeMap<QualifiedName, ActorRef>,
                   distinction: Distinction,
                   arranging: &[&EntityHandle],
                   input: Option<&BTreeMap<String, Node>>| match &owner {
        Some(owner) => super::created_owned(
            ir,
            entity,
            creator,
            actors,
            distinction,
            arranging,
            Some(owner),
            input,
        ),
        None => created(ir, entity, creator, actors, distinction, arranging, input),
    };
    let mut out =
        vec![created(ir, entity, creator, actors, distinction, arranging, None).map_err(required)?];
    if has_subject_guards(creator.command)
        || creator.outcome.test_strategy == ess_domain::command::TestStrategy::InjectFault
    {
        return Ok(out);
    }
    // And the rows whose stored collections hold several entries, where a quantifier over one
    // compares its elements with the input (beyond10x/ess#240).
    let mut inputs = hinted(ir, entity, creator, hints)?.unwrap_or_default();
    inputs.extend(spread(ir, entity, creator, hints, distinction));
    for input in inputs {
        if input_selects(ir, creator.command, creator.outcome, &input)? {
            if let Ok(arrangement) = created(
                ir,
                entity,
                creator,
                actors,
                distinction,
                arranging,
                Some(&input),
            ) {
                out.push(arrangement);
            }
        }
    }
    if under.is_none() && !super::related_guard::routes(creator.command, creator.outcome) {
        out.extend(copied_creations(
            ir,
            entity,
            creator,
            actors,
            hints,
            distinction,
            arranging,
        ));
    }
    Ok(out)
}

/// The copies one creating branch's `sets:` makes through one input naming a related row: the
/// entity that row is of, and each target field with the related field it copies.
type Copies<'a> = (&'a EntityHandle, Vec<(&'a str, &'a str)>);

/// Every `{related: {via: input.f, field: g}}` of `outcome`'s `sets:`, by the input `f`. A chained
/// read (ess/22, beyond10x/ess#285) names a row of another entity than the one it copies from, and
/// is not one of them.
fn copies_by_input(outcome: &ResolvedOutcome) -> BTreeMap<&str, Copies<'_>> {
    let mut out: BTreeMap<&str, Copies<'_>> = BTreeMap::new();
    for set in outcome.sets.iter().filter(|set| set.conversion.is_none()) {
        if let ResolvedPayloadValue::RelatedField {
            via: ess_compiler::ir::ResolvedRelatedVia::Input { field: via, .. },
            through,
            entity: related,
            field,
            ..
        } = &set.value
        {
            if !through.is_empty() {
                continue;
            }
            out.entry(via.as_str())
                .or_insert_with(|| (related, Vec::new()))
                .1
                .push((set.target.as_str(), field.as_str()));
        }
    }
    out
}

/// The hints that read a copied field, rewritten over the related row's field the copy reads, with
/// every comparison with the command's input dropped ([`without_input`]).
fn through_copies(
    ir: &EssIr,
    entity: &EntityHandle,
    hints: &[Predicate],
    copies: &[(&str, &str)],
) -> Vec<Predicate> {
    let onto: BTreeMap<&str, &str> = copies.iter().copied().collect();
    hints
        .iter()
        .map(|hint| without_input(ir, entity, hint))
        .filter(|hint| {
            hint.fact_paths()
                .iter()
                .any(|path| onto.contains_key(path.namespace()))
        })
        .map(|hint| {
            map_paths(&hint, &|path: &FactPath| match onto.get(path.namespace()) {
                Some(field) => {
                    let mut segments: Vec<String> = field.split('.').map(str::to_owned).collect();
                    segments.extend(path.segments()[1..].iter().cloned());
                    FactPath::from_segments(segments)
                }
                None => path.clone(),
            })
        })
        .collect()
}

/// The block the related rows a creation copies a field from are arranged at ([`copied_creations`]):
/// past every block [`super::related_guard`] and [`super::related`] number, so its rows name no row
/// either arranges.
const COPIED: usize = 27_720 * 13;

/// The rows a creating branch leaves where its `sets:` copies a field the hints read from a related
/// row its input names (`{related: {via: input.f, field: g}}`, beyond10x/ess#307): that row is
/// arranged first, at each value the related entity's own creation can be steered to toward the
/// hints read through the copy, and the creation is sent naming it, so the copied field holds the
/// value the row was given. Empty where the branch copies no field the hints read, where the
/// related entity is already being arranged, or where the related row is the creation's owner.
fn copied_creations(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Vec<Arrangement> {
    let read: BTreeSet<String> = hints
        .iter()
        .flat_map(Predicate::fact_paths)
        .map(|path| path.namespace().to_owned())
        .collect();
    let mut out = Vec::new();
    for (via, (related, copies)) in copies_by_input(creator.outcome) {
        if !copies.iter().any(|(target, _)| read.contains(*target))
            || related == entity
            || arranging.contains(&related)
            || ir
                .owner_of(entity)
                .is_some_and(|owned| owned.owner == *related)
        {
            continue;
        }
        let translated = through_copies(ir, entity, hints, &copies);
        let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
        let at = Distinction::further(COPIED + distinction.get());
        let all = ir.drivers();
        let rows: Vec<Arrangement> = all
            .get(related)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
            .filter_map(|row_creator| {
                creations(
                    ir,
                    related,
                    row_creator,
                    actors,
                    &translated,
                    at,
                    (&chain, None),
                )
                .ok()
            })
            .flatten()
            .collect();
        let owner =
            super::arrange_owner(ir, creator.outcome, entity, actors, distinction, arranging);
        for row in rows {
            let Ok(mut arrangement) = super::created_by(
                ir,
                entity,
                creator,
                distinction,
                owner.as_ref(),
                |bound, _| {
                    let input = super::reach(ir, creator.command, creator.outcome, distinction)?;
                    let mut bound = bound.clone();
                    bound.insert(via.to_owned(), row.instance.clone());
                    let mut invocation = invoke_with(ir, creator, None, actors, &bound, &input);
                    let mut steps = row.steps.clone();
                    steps.append(&mut invocation.steps);
                    invocation.steps = steps;
                    invocation.source.extend(row.source.iter().cloned());
                    Ok::<_, RefusalCause>(invocation)
                },
            ) else {
                continue;
            };
            settle_copies(
                ir,
                creator.outcome,
                (related, &copies),
                &row,
                &mut arrangement,
            );
            out.push(arrangement);
        }
    }
    out
}

/// Settles on `arrangement` each field the creation copied from `row`: the row's identity as its
/// instance, an `Optional` field no step of the row wrote as `null`, and any other as the row
/// settled it. A field the row did not determine stays undetermined.
fn settle_copies(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    (related, copies): (&EntityHandle, &[(&str, &str)]),
    row: &Arrangement,
    arrangement: &mut Arrangement,
) {
    let identity = &ir.entity(related).identity;
    for (target, field) in copies {
        let held = if identity.name == *field {
            Some(ScenarioValue::instance(row.instance.clone()))
        } else if row.unwritten.contains(*field) && !row.settled.contains_key(*field) {
            Some(ScenarioValue::Literal { value: Node::Null })
        } else {
            row.settled.get(*field).map(|held| held.value.clone())
        };
        let (Some(value), Some(set)) =
            (held, outcome.sets.iter().find(|set| set.target == *target))
        else {
            continue;
        };
        arrangement.settled.insert(
            (*target).to_owned(),
            super::Determined {
                value,
                type_ref: set.target_type.clone(),
            },
        );
    }
}

/// The row after `driver` runs on `arrangement` with this input, carrying the counters `follow`
/// names past the raise.
#[allow(clippy::too_many_arguments)]
fn advanced(
    ir: &EssIr,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
    before: Vec<ScenarioStep>,
    no_error: bool,
    follow: &Follow,
) -> Arrangement {
    let mut next = arrangement.clone();
    next.steps.extend(before);
    let invoked = invoke_with(
        ir,
        driver,
        Some(&arrangement.instance),
        actors,
        &BTreeMap::new(),
        input,
    );
    next.steps.extend(invoked.steps);
    if no_error {
        next.steps.push(ScenarioStep::ExpectNoError);
    }
    next.source.extend(invoked.source);
    next.absorb(driver.outcome, invoked.settled);
    follow.raise(ir, driver, &arrangement.settled, &mut next.settled);
    if let Some(transition) = driver.effect.transition() {
        next.state = transition.to.clone();
    }
    next
}

/// Every row one arranging branch can leave from `arrangement`, toward the hints.
///
/// A branch of a command that itself reads stored fields is taken only with an input the row
/// selects it for, and the facts it reads are observed first. Any other branch is run with its
/// plain witness, and — where its `sets:` maps a field the hints read — with each input chosen
/// toward them that its own guards still select it for.
#[allow(clippy::too_many_arguments)]
fn successors(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    arranging: &[&EntityHandle],
    follow: &Follow,
) -> Vec<Arrangement> {
    let mut out = Vec::new();
    // A move reading a related row through a stored field of this row (ess/22,
    // beyond10x/ess#304) is sent with that reference left out, or naming a row arranged for it.
    if super::related_guard::stored::field(driver.command).is_some() {
        if let Ok(mut next) = super::related_guard::stored::step(
            ir,
            driver,
            arrangement,
            actors,
            Distinction::PLAIN,
            arranging,
        ) {
            follow.raise(ir, driver, &arrangement.settled, &mut next.settled);
            out.push(next);
        }
        return out;
    }
    if uses(driver.command) {
        let own = self::hints(driver.command);
        let fields = read_fields(ir, entity, &own);
        let Ok((observed, _)) = observe_fields(ir, entity, &fields, arrangement) else {
            return out;
        };
        let mut inputs = inputs_for(ir, driver.command, entity, arrangement).unwrap_or_default();
        if let Ok(Some(more)) = hinted(ir, entity, driver, hints) {
            inputs.extend(more);
        }
        for input in inputs {
            if selects(
                ir,
                driver.command,
                entity,
                arrangement,
                &input,
                Order::Unique,
            )
            .ok()
            .flatten()
            .is_some_and(|branch| branch.name == driver.outcome.name)
            {
                out.push(advanced(
                    ir,
                    driver,
                    arrangement,
                    actors,
                    &input,
                    observed.clone(),
                    true,
                    follow,
                ));
            }
        }
        return out;
    }
    let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
    if let Ok(invoked) = invoke(
        ir,
        driver,
        Some(&arrangement.instance),
        Some(&arrangement.state),
        actors,
        Distinction::PLAIN,
        &BTreeMap::new(),
        &chain,
    ) {
        let mut next = arrangement.clone();
        next.steps.extend(invoked.steps);
        next.source.extend(invoked.source);
        next.absorb(driver.outcome, invoked.settled);
        follow.raise(ir, driver, &arrangement.settled, &mut next.settled);
        if let Some(transition) = driver.effect.transition() {
            next.state = transition.to.clone();
        }
        out.push(next);
    }
    if has_subject_guards(driver.command)
        || driver.outcome.test_strategy == ess_domain::command::TestStrategy::InjectFault
        // A related-row command (ess/18) is run only with the row it reads arranged, above.
        || super::related_guard::uses(driver.command)
    {
        return out;
    }
    // A move writing a stored collection a quantifier compares with the input is also offered
    // with several entries (beyond10x/ess#240), as a creation is.
    let mut inputs = hinted(ir, entity, driver, hints)
        .ok()
        .flatten()
        .unwrap_or_default();
    inputs.extend(spread(ir, entity, driver, hints, Distinction::PLAIN));
    for input in inputs {
        if input_selects(ir, driver.command, driver.outcome, &input).unwrap_or(false) {
            out.push(advanced(
                ir,
                driver,
                arrangement,
                actors,
                &input,
                Vec::new(),
                false,
                follow,
            ));
        }
    }
    out
}

/// Search the rows the declared drivers can leave for one the goal accepts.
///
/// Every creating branch is a place to start (beyond10x/ess#198): each is searched in the order
/// [`EssIr::drivers`] yields them — command name, then the command's branches as declared, the
/// IR keeping commands by name — within its own budget of [`MAX_NODES`], and the first whose rows
/// reach the goal wins. So a branch only a later creation's row selects is witnessed through that
/// creation, and a model whose first creation already reaches the goal is arranged exactly as
/// before. A creation that leaves no row, or none the goal accepts, gives way to the next, and the
/// refusal is the first creation's cause where every one fails.
fn search<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    search_within(ir, entity, actors, hints, distinction, field, &[], goal)
}

/// [`search`] for a further row of a scenario, under the first further distinction from `first`
/// whose arrangement binds no instance name in `taken` — the names the scenario's earlier steps
/// already bind (beyond10x/ess#193). The row the scenario's own subject was arranged on may already
/// sit on a further distinction ([`prepare`] asks `1..=FRESH_WITNESSES` for a row leaving fewer
/// writes unchanged), and a further row captured under that name again would rebind it, so the
/// closing observation of the scenario's own row would read another one.
///
/// The ordinal is searched rather than counted from a known offset, as `arrange_unbound` in the
/// parent module does, because the numbers taken are chosen by code that does not know about this
/// search. The inner result is [`search`]'s own, which the caller answers as it answers any search;
/// the outer error is a row for which every name up to [`MAX_CANDIDATES`](super::MAX_CANDIDATES)
/// is taken, which is refused whatever the goal, never skipped.
#[allow(clippy::too_many_arguments)]
fn search_unbound<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    first: usize,
    field: &str,
    taken: &BTreeSet<super::InstanceName>,
    mut goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<Result<(Arrangement, T), RefusalCause>, RefusalCause> {
    let name = &ir.entity(entity).name;
    for nth in first..=super::MAX_CANDIDATES {
        let distinction = Distinction::further(nth);
        if taken.contains(&super::instance_name(name, distinction)) {
            continue;
        }
        match search(ir, entity, actors, hints, distinction, field, &mut goal) {
            Ok(found) if !super::bound_instances(&found.0.steps).is_disjoint(taken) => {}
            found => return Ok(found),
        }
    }
    Err(RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{entity}` {field} row under an instance name no earlier step of the scenario binds"
        ),
        tried: 0,
    })
}

/// [`search`], inside an arrangement of the entities `arranging` names: a creator or a driver that
/// would need one of them again (a related row of its own entity, ess/18) stops at the cycle, and
/// the next creator is tried.
#[allow(clippy::too_many_arguments)]
pub(super) fn search_within<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    arranging: &[&EntityHandle],
    goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    search_under(
        ir,
        entity,
        actors,
        hints,
        (distinction, field),
        arranging,
        None,
        goal,
    )
}

/// Where a search files its rows ([`search_under`]): under an owner the scenario already arranged,
/// and whether that owner is known to hold no row of the entity yet ([`super::holds_none`]).
pub(super) type Under<'a> = Option<(&'a super::InstanceName, bool)>;

/// [`search_within`], every row created under `under` where it names an owner the scenario
/// already arranged ([`super::under_owner`]), and not under an owner of its own: a creator that
/// cannot file the row there — the relation holds one row per owner and the owner is not known to
/// hold none, or the creator does not name the owner from its input — leaves no row, and the
/// search refuses as it would for none (beyond10x/ess#271).
#[allow(clippy::too_many_arguments)]
pub(super) fn search_under<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    (distinction, field): (Distinction, &str),
    arranging: &[&EntityHandle],
    under: Under<'_>,
    mut goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let mut first: Option<RefusalCause> = None;
    for creator in drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    {
        match search_from(
            ir,
            entity,
            drivers,
            creator,
            actors,
            hints,
            distinction,
            field,
            (arranging, under),
            &mut goal,
        ) {
            Ok(found) => return Ok(found),
            // The first creator's cause is kept, except that a later search stopped by the counter
            // bound replaces one that was not: a side of a limit one creation could have reached
            // past the bound is refused naming it, not read as a value no run holds (#226).
            Err(cause) => match &first {
                Some(kept) if !is_beyond_reach(kept) && is_beyond_reach(&cause) => {
                    first = Some(cause);
                }
                Some(_) => {}
                None => first = Some(cause),
            },
        }
    }
    Err(first.unwrap_or(RefusalCause::InstanceRequired {
        entity: EntityRef::from(entity),
        need: InstanceNeed::Updates,
        reason: Unreachable::NothingCreates,
    }))
}

/// [`search`] from the rows one creating branch can leave.
///
/// Breadth-first. At each depth every new node is offered to the goal, and of those it accepts the
/// one closest to the guards wins — ties to the earlier, so the choice is a function of the model
/// (§37).
#[allow(clippy::too_many_arguments)]
fn search_from<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    drivers: &[Driver<'_>],
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    (arranging, under): (&[&EntityHandle], Under<'_>),
    goal: &mut impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let level = creations(
        ir,
        entity,
        creator,
        actors,
        hints,
        distinction,
        (arranging, under),
    )?;
    search_rows(
        ir, entity, drivers, actors, hints, field, arranging, level, goal,
    )
}

/// The bounded search shared by fresh creations and an already observed row.
#[allow(clippy::too_many_arguments)]
fn search_rows<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    drivers: &[Driver<'_>],
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    field: &str,
    arranging: &[&EntityHandle],
    mut level: Vec<Arrangement>,
    goal: &mut impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let follow = Follow::new(ir, entity, hints);
    // Rows beyond the followed counter bound share a node, as on the creation search.
    let mut beyond = false;
    let mut nested = false;
    let mut seen = BTreeSet::new();
    let mut first: Option<RefusalCause> = None;
    loop {
        let mut fresh = Vec::new();
        for node in level {
            beyond |= follow.key(&node.settled).1;
            if seen.insert(profile(ir, entity, &node, hints, &follow)) {
                fresh.push(node);
            }
            if seen.len() > MAX_NODES {
                // Rows a followed counter tells apart fill the budget: no field lacks a value, so
                // this is the counter's bound, `ESS-SYNTH-003`, not a type without a finite value
                // (beyond10x/ess#226).
                if !follow.is_empty() {
                    return Err(RefusalCause::GuardUnsatisfiable {
                        predicate: format!(
                            "`{entity}` stored {field} selecting this branch: the rows the \
                             followed counters leave exceed the search budget of {MAX_NODES} \
                             rows; {}",
                            beyond_reach(&follow.names())
                        ),
                        tried: seen.len(),
                    });
                }
                return Err(missing(
                    entity,
                    field,
                    "subject fact arrangement exceeds 64 lifecycle/fact combinations",
                ));
            }
        }
        if fresh.is_empty() {
            break;
        }
        let mut best: Option<((usize, usize, usize), usize, T)> = None;
        for (index, node) in fresh.iter().enumerate() {
            match goal(node) {
                Ok(Some(found)) => {
                    let rank = score(&profile(ir, entity, node, hints, &follow));
                    if best.as_ref().is_none_or(|(held, ..)| rank > *held) {
                        best = Some((rank, index, found));
                    }
                }
                Ok(None) => {}
                Err(cause) => {
                    first.get_or_insert(cause);
                }
            }
        }
        if let Some((_, index, found)) = best {
            vacuous(ir, entity, &fresh[index], hints)?;
            return Ok((fresh.swap_remove(index), found));
        }
        let mut next = Vec::new();
        for node in &fresh {
            for driver in drivers {
                if matches!(
                    driver.effect,
                    ResolvedEffect::Creates | ResolvedEffect::Preserves
                ) || driver
                    .effect
                    .transition()
                    .is_some_and(|transition| !transition.from.contains(&node.state))
                {
                    continue;
                }
                let moved = successors(ir, entity, driver, node, actors, hints, arranging, &follow);
                // A move reading a related row of this entity, which left no row: the one-level
                // bound on such a row ([`super::related_guard`]) is what stopped it (#229).
                nested |=
                    moved.is_empty() && super::related_guard::reads_entity(driver.command, entity);
                next.extend(moved);
            }
        }
        level = next;
    }
    // Every field the guards read can be set (`unarrangeable` checked that first), so what failed
    // is the search for values that decide the guard: a guard the candidate values cannot
    // satisfy, not a type without a value — `ESS-SYNTH-003`, with its repair, or `ESS-SYNTH-018`
    // where the guard's `.count` boundary lies past what a witness is built with.
    Err(first.unwrap_or_else(|| {
        super::unsatisfied(
            &hints.iter().collect::<Vec<_>>(),
            format!(
                "`{entity}` stored {field} selecting this branch, over the rows {} bounded \
                 arrangements left{}{}",
                seen.len(),
                if beyond {
                    format!("; {}", beyond_reach(&follow.names()))
                } else {
                    String::new()
                },
                nested_bound(entity, nested && super::related_guard::nests(ir)),
            ),
            seen.len(),
        )
    }))
}

/// Delete the actual row a create-or-refuse segment already stored. No creation or deletion is
/// traversed: the former would replace the row being tested, and the latter leaves no row to
/// advance. The goal invokes a declared deletion through the ordinary guarded successor builder.
pub(super) fn delete_existing(
    ir: &EssIr,
    entity: &EntityHandle,
    mut arrangement: Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Option<Arrangement>, RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let deleting: Vec<_> = drivers
        .iter()
        .filter(|driver| *driver.effect == ResolvedEffect::Deletes)
        .copied()
        .collect();
    if deleting.is_empty() {
        return Ok(None);
    }
    let moving: Vec<_> = drivers
        .iter()
        .filter(|driver| {
            !matches!(
                driver.effect,
                ResolvedEffect::Creates | ResolvedEffect::Deletes
            )
        })
        .copied()
        .collect();
    let hints: Vec<_> = drivers
        .iter()
        .flat_map(|driver| hints(driver.command))
        .collect();
    let follow = Follow::new(ir, entity, &hints);
    // The creation and the duplicate refusal have already run in the enclosing segment.
    arrangement.steps.clear();
    search_rows(
        ir,
        entity,
        &moving,
        actors,
        &hints,
        "deletion after duplicate creation",
        &[],
        vec![arrangement],
        &mut |row| {
            Ok(deleting.iter().find_map(|driver| {
                successors(ir, entity, driver, row, actors, &hints, &[], &follow)
                    .into_iter()
                    .next()
            }))
        },
    )
    .map(|(_, deleted)| Some(deleted))
}

/// What a search says where a move toward the goal read a related row of the searched entity
/// itself and left no row: the one-level bound on such a row stopped it (beyond10x/ess#229).
fn nested_bound(entity: &EntityHandle, nested: bool) -> String {
    if nested {
        format!(
            "; a move toward it reads a related row of `{entity}` itself, which is arranged one \
             level deep and only where its lifecycle starts"
        )
    } else {
        String::new()
    }
}

/// The first input that selects `outcome` for the row `arrangement` holds.
fn reach_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    for input in inputs_for(ir, command, entity, arrangement)? {
        if selects(ir, command, entity, arrangement, &input, Order::Unique)?
            .is_some_and(|branch| branch.name == outcome.name)
        {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// An input the input-guarded refusal `outcome` is taken for on `arrangement`'s row, decided by the
/// input alone: the refusal answers before the held state is read, so the row's stored guards —
/// decided or not — select nothing beside it (beyond10x/ess#234). `None` on a row in a state the
/// command refuses, which is the wrong-state family's, and where no candidate selects `outcome`
/// and no other input-guarded refusal.
fn refusal_first(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    if ir
        .wrong_states(command)
        .get(entity)
        .is_some_and(|states| states.contains(&arrangement.state))
    {
        return Ok(None);
    }
    for input in inputs_for(ir, command, entity, arrangement)? {
        let facts = flatten(ir, command, &input).map_err(RefusalCause::WitnessRejected)?;
        let mut taken = Vec::new();
        for branch in command
            .outcomes
            .iter()
            .filter(|branch| super::is_input_guarded_refusal(branch))
        {
            if let Some(guard) = input_guard(&branch.condition) {
                if decides(&facts, &[guard], true)? {
                    taken.push(&branch.name);
                }
            }
        }
        if taken == [&outcome.name] {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// The stored fields the goal reads that the arrangement cannot set, and why — or `None`.
///
/// Refused rather than searched for: a guarded field that no arranging branch writes from an input,
/// no literal fixes and no creator leaves absent is a field the search has no way to move.
fn unarrangeable(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
) -> Option<RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    for field in fields {
        let written = drivers.iter().any(|driver| {
            driver.outcome.sets.iter().any(|set| {
                &set.target == field
                    && set.conversion.is_none()
                    && !matches!(set.value, ResolvedPayloadValue::Generated)
            })
        });
        // A creator that leaves an `Optional` field unwritten arranges it too: absent
        // (beyond10x/ess#239, #234).
        let left_absent = drivers.iter().any(|driver| {
            matches!(driver.effect, ResolvedEffect::Creates)
                && super::unwritten_by(ir, entity, driver.outcome).contains(field)
        });
        if !written && !left_absent {
            return Some(missing(
                entity,
                field,
                "no arranging branch sets this stored field from an input or a literal without a \
                 conversion, so no row can be arranged to the guard",
            ));
        }
    }
    None
}

/// The refusal for a stored instant one of `predicates` orders against the current time that no
/// arranging branch can carry (ess/22, `docs/design/expression-family-source22.md`, A3), naming the
/// stored path; `None` where every such instant can be carried.
///
/// Synthesis decides a row's guards at the fixed reference instant, and only a value sent to the
/// creator as a `now_offset` keeps that decision at a run: it travels through the creator's
/// `sets:` into the row, resolved from the moment the run sends it. So such an instant is arranged
/// only as a whole field some arranging branch writes from a whole input field without a
/// conversion. One the implementation generates, a literal, a converted value, a member inside a
/// structure and an element a quantifier binds cannot be, and are refused rather than decided at
/// the reference: no clock value is fabricated.
///
/// `creating` counts only the branches that create the row: a stored instant every creator leaves
/// to the implementation or to a literal may still be carried by a later branch writing it from its
/// input, and where no arrangement reaches the branch that way the search's refusal is restated as
/// this one.
pub(super) fn now_uncarried(
    ir: &EssIr,
    entity: &EntityHandle,
    predicates: &[Predicate],
    creating: bool,
) -> Option<RefusalCause> {
    let declared = ir.entity(entity);
    let all = ir.drivers();
    let drivers: Vec<&Driver<'_>> = all
        .get(entity)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|driver| !creating || matches!(driver.effect, ResolvedEffect::Creates))
        .collect();
    for predicate in predicates {
        for (path, whole) in crate::now_offset::now_compared(predicate) {
            let root = path.namespace();
            if !declared.fields.iter().any(|field| field.name == root) {
                continue;
            }
            let carried = whole
                && drivers.iter().any(|driver| {
                    driver.outcome.sets.iter().any(|set| {
                        set.target == root
                            && set.conversion.is_none()
                            && matches!(set.value, ResolvedPayloadValue::InputField { .. })
                    })
                });
            if !carried {
                return Some(RefusalCause::NoWitness(WitnessGap {
                    path: format!("{}.{path}", declared.name),
                    type_ref: "Timestamp".into(),
                    reason: "a stored instant ordered against the current time is arranged only \
                             as a whole field an arranging branch writes from a whole input \
                             field, sent as a now_offset; this one is generated, a literal, \
                             converted or inside a value, and no clock value is fabricated",
                }));
            }
        }
    }
    None
}

/// [`now_uncarried`] over every arranging branch, as the refusal; otherwise over the creators only,
/// as the refusal a failed search is restated as.
fn now_refusal(
    ir: &EssIr,
    entity: &EntityHandle,
    predicates: &[Predicate],
) -> Result<Option<RefusalCause>, RefusalCause> {
    match now_uncarried(ir, entity, predicates, false) {
        Some(refusal) => Err(refusal),
        None => Ok(now_uncarried(ir, entity, predicates, true)),
    }
}

/// Records, for a row no input selects `outcome` on, every input whose own stored and input guards
/// hold of that row, and which sibling input-guarded refusal claimed it (beyond10x/ess#178). A row
/// whose state no move of the command starts from is the wrong-state family's and is skipped.
fn shadowed_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    shadow: &mut super::Shadow,
) -> Result<(), RefusalCause> {
    if outcome.error.is_some()
        || ir
            .wrong_states(command)
            .get(&entity)
            .is_some_and(|states| states.contains(&arrangement.state))
    {
        return Ok(());
    }
    let row_guard = stored(&outcome.condition);
    let own_input: Vec<&Predicate> = input_guard(&outcome.condition).into_iter().collect();
    for input in inputs_for(ir, command, entity, arrangement)? {
        if let Some(predicate) = &row_guard {
            if row_truth_with(
                ir,
                entity,
                &arrangement.settled,
                &arrangement.unwritten,
                Some(&arrangement.state),
                predicate,
                Some((command, &input)),
            ) != Truth::True
            {
                continue;
            }
        }
        let facts = flatten(ir, command, &input).map_err(RefusalCause::WitnessRejected)?;
        if decides(&facts, &own_input, true)? {
            shadow.record(command, outcome, &facts)?;
        }
    }
    Ok(())
}

/// The stored-row search's refusal, restated as the shadow it is where every input the branch's
/// own guards admit on the rows searched was claimed by a sibling input-guarded refusal.
fn shadowed(
    outcome: &ResolvedOutcome,
    shadow: &super::Shadow,
    cause: RefusalCause,
) -> RefusalCause {
    let RefusalCause::GuardUnsatisfiable { tried, .. } = &cause else {
        return cause;
    };
    let row_guard = stored(&outcome.condition);
    let mut guards: Vec<&Predicate> = row_guard.iter().collect();
    guards.extend(input_guard(&outcome.condition));
    match shadow.rendered(&guards) {
        Some(predicate) => RefusalCause::GuardUnsatisfiable {
            predicate,
            tried: *tried,
        },
        None => cause,
    }
}

/// A row and the input a scenario sends it.
type Arranged = (Arrangement, BTreeMap<String, Node>);

/// The row [`prepare`] starts from, the input that selects the branch on it, and how it was found.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Found {
    /// The row and input witness every [`elementwise`] quantifier of the guards element by element
    /// ([`witnesses_elements`], beyond10x/ess#240); every later refinement keeps that.
    Elementwise,
    /// The plain search: the row decides the branch's stored guards and the input selects it.
    Plain,
    /// An input-guarded refusal taken before the row is read (beyond10x/ess#234).
    BeforeRow,
    /// The plain search under [`Order::FirstDeclared`], where no row and input selects the branch
    /// alone (beyond10x/ess#278); every later refinement keeps that order.
    FirstDeclared,
}

impl Found {
    /// The order every refinement of a witness found this way selects under.
    fn order(self) -> Order {
        if self == Self::FirstDeclared {
            Order::FirstDeclared
        } else {
            Order::Unique
        }
    }
}

/// The row [`prepare`] arranges for `outcome` and the input that selects it there.
///
/// Where the command has an [`elementwise`] quantifier, the search first offers only inputs that
/// witness it element by element, and the plain search runs only where no bounded arrangement holds
/// one — a quantifier whose collection no input writes, or whose values the witnesses cannot spread.
///
/// An input-guarded refusal answers before the held state is read, so a row whose stored fields no
/// arranging step determined — an optional struct left absent — still takes it
/// (beyond10x/ess#234). That row is searched for only where no row decided every stored guard, so a
/// witness found there is the one it always was.
fn arranged_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    label: &str,
) -> Result<(Arranged, Found), RefusalCause> {
    if has_elementwise(ir, entity, hints) {
        let strict = |node: &Arrangement, input: &BTreeMap<String, Node>| {
            witnesses_elements(ir, command, entity, hints, node, input)
        };
        if let Ok(found) = search(
            ir,
            entity,
            actors,
            hints,
            Distinction::PLAIN,
            label,
            |node| reach_linked(ir, command, outcome, entity, node, &strict, Order::Unique),
        ) {
            return Ok((found, Found::Elementwise));
        }
    }
    let mut shadow = super::Shadow::default();
    let searched = search(
        ir,
        entity,
        actors,
        hints,
        Distinction::PLAIN,
        label,
        |node| {
            let found = reach_linked(
                ir,
                command,
                outcome,
                entity,
                node,
                &|_, _| true,
                Order::Unique,
            )?;
            if found.is_none() {
                shadowed_at(ir, command, outcome, entity, node, &mut shadow)?;
            }
            Ok(found)
        },
    )
    .map_err(|cause| shadowed(outcome, &shadow, cause));
    match searched {
        Err(cause) if super::is_input_guarded_refusal(outcome) => search(
            ir,
            entity,
            actors,
            hints,
            Distinction::PLAIN,
            label,
            |node| refusal_first(ir, command, outcome, entity, node),
        )
        .map(|found| (found, Found::BeforeRow))
        .map_err(|_| cause),
        // Only where no row and input selects the branch alone: the first declared of several
        // answers (beyond10x/ess#278), so a branch declared before a sibling it cannot be told
        // apart from by its input is witnessed on a row its stored guard admits. A branch the
        // search found before is found as it was.
        Err(cause) => search(
            ir,
            entity,
            actors,
            hints,
            Distinction::PLAIN,
            label,
            |node| {
                reach_linked(
                    ir,
                    command,
                    outcome,
                    entity,
                    node,
                    &|_, _| true,
                    Order::FirstDeclared,
                )
            },
        )
        .map(|found| (found, Found::FirstDeclared))
        .map_err(|_| cause),
        searched => searched.map(|found| (found, Found::Plain)),
    }
}

/// Require the arranged row before the command runs ([`prepare`]): every guarded field, or — for a
/// branch taken before the row is read — only what the arrangement determined of it, and nothing
/// where it determined none of them.
fn observe_prepared(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: BTreeSet<String>,
    before_row: bool,
    arrangement: &mut Arrangement,
) -> Result<(), RefusalCause> {
    let observed: BTreeSet<String> = if before_row {
        fields
            .into_iter()
            .filter(|field| arrangement.settled.contains_key(field))
            .collect()
    } else {
        fields
    };
    if !before_row || !observed.is_empty() {
        let (steps, view) = observe_fields(ir, entity, &observed, arrangement)?;
        arrangement.steps.extend(steps);
        arrangement.source.insert(view.into());
    }
    Ok(())
}

/// Arrange the row the branch under test is selected for, and the input that selects it.
pub(super) fn prepare(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let subject = reading(command, outcome).ok_or(RefusalCause::StrategyWithoutGuard {
        strategy: outcome.test_strategy,
    })?;
    let entity = &subject.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    let label = fields.iter().cloned().collect::<Vec<_>>().join(",");
    if let Some(refusal) = unarrangeable(ir, entity, &fields) {
        return Err(refusal);
    }
    // A stored instant ordered against `now` no arranging branch carries is refused by name; one
    // no creator carries names the search's refusal where no later branch reached it (A3).
    let named = now_refusal(ir, entity, &hints)?;
    // Where a quantifier over a stored collection compares its elements with the input, the row
    // and input witness it element by element wherever some arrangement does ([`arranged_row`]),
    // and every refinement below keeps that.
    let ((arrangement, input), found) =
        arranged_row(ir, command, outcome, entity, actors, &hints, &label)
            .map_err(|cause| named.unwrap_or(cause))?;
    let order = found.order();
    let strict = |node: &Arrangement, input: &BTreeMap<String, Node>| {
        witnesses_elements(ir, command, entity, &hints, node, input)
    };
    let any = |_: &Arrangement, _: &BTreeMap<String, Node>| true;
    let accept: &dyn Fn(&Arrangement, &BTreeMap<String, Node>) -> bool =
        if found == Found::Elementwise {
            &strict
        } else {
            &any
        };
    // A row the branch's writes would leave unchanged proves nothing about them (beyond10x/ess#161).
    // So where the plain row leaves some write unchanged, the search is asked again — under the
    // plain witness and then further ones — for a row that leaves fewer unchanged, and keeps the
    // fewest: the criterion [`arranged`](super::arranged) applies on the plain path. A write no row
    // can change (a literal every arrangement already holds) does not stop the others being changed.
    let mut unchanged = super::unchanged_writes(ir, outcome, &input, &arrangement.settled);
    let (mut arrangement, input) = {
        let mut best = (arrangement, input);
        for distinction in std::iter::once(Distinction::PLAIN)
            .chain((1..=super::FRESH_WITNESSES).map(Distinction::further))
        {
            if unchanged == 0 {
                break;
            }
            let bound = unchanged;
            let Ok((row, input)) =
                search(ir, entity, actors, &hints, distinction, &label, |node| {
                    Ok(
                        reach_linked(ir, command, outcome, entity, node, accept, order)?.filter(
                            |input| {
                                super::unchanged_writes(ir, outcome, input, &node.settled) < bound
                            },
                        ),
                    )
                })
            else {
                continue;
            };
            unchanged = super::unchanged_writes(ir, outcome, &input, &row.settled);
            best = (row, input);
        }
        best
    };
    // The pair a `sets-retarget` mutant joins is sent apart on the chosen row too, where the row
    // still selects the branch and no more of its writes are left unchanged (beyond10x/ess#202).
    let keeps = |next: &BTreeMap<String, Node>| {
        selects(ir, command, entity, &arrangement, next, order)
            .ok()
            .flatten()
            .is_some_and(|branch| branch.name == outcome.name)
            && super::unchanged_writes(ir, outcome, next, &arrangement.settled) <= unchanged
            && accept(&arrangement, next)
    };
    let (input, _) = super::sources_apart(ir, command, outcome, input, &keeps);
    let bound = bind_links(
        ir,
        command,
        outcome,
        entity,
        actors,
        &mut arrangement,
        &input,
        &mut false,
    )?;
    observe_prepared(
        ir,
        entity,
        fields,
        found == Found::BeforeRow,
        &mut arrangement,
    )?;
    // A stored-field guard changes how the row is arranged, not whether deletion leaves one.
    let after = match outcome.subject.as_ref().map(|own| &own.effect) {
        Some(ResolvedEffect::Deletes) => None,
        effect => Some(effect.and_then(ResolvedEffect::transition).map_or_else(
            || arrangement.state.clone(),
            |transition| transition.to.clone(),
        )),
    };
    Ok((
        Setup {
            steps: arrangement.steps,
            instance: Some(arrangement.instance),
            bound,
            source: arrangement.source,
            after,
            before: Some(arrangement.state),
            settled: arrangement.settled,
        },
        input,
    ))
}

/// The row after one route step through a branch of a command reading stored fields, where the
/// row the route built selects that branch.
///
/// Used by the ordinary lifecycle arrangement, which takes the plain witness at every step: this
/// checks the claim that witness makes rather than assuming it, and adds nothing to the steps a
/// route that was already right produced.
pub(super) fn step(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Arrangement> {
    let inputs = inputs_for(ir, driver.command, entity, arrangement).ok()?;
    inputs.into_iter().find_map(|input| {
        selects(
            ir,
            driver.command,
            entity,
            arrangement,
            &input,
            Order::Unique,
        )
        .ok()
        .flatten()
        .filter(|branch| branch.name == driver.outcome.name)
        .map(|_| {
            advanced(
                ir,
                driver,
                arrangement,
                actors,
                &input,
                Vec::new(),
                false,
                &Follow::default(),
            )
        })
    })
}

/// A row resting in `target`, reached through branches every row on the way selects.
///
/// Arranged under `distinction`, so a further instance searched for here keeps the name its caller
/// chose it apart by (beyond10x/ess#199).
pub(super) fn reach_state(
    ir: &EssIr,
    entity: &EntityHandle,
    target: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Result<Arrangement, RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let mut hints = Vec::new();
    for driver in drivers.iter().filter(|driver| uses(driver.command)) {
        for hint in self::hints(driver.command) {
            if !hints.contains(&hint) {
                hints.push(hint);
            }
        }
    }
    search(ir, entity, actors, &hints, distinction, "state", |node| {
        Ok((&node.state == target).then_some(()))
    })
    .map(|(arrangement, ())| arrangement)
}

/// Which assertion styles a stored-field observation may be made through.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// Only an `expect` view: the observation must see the row at the moment it is made.
    Immediate,
    /// An `expect` view where one qualifies, and an `eventually` view where none does.
    Settling,
}

/// The unfiltered, parameterless view a stored-field arrangement is observed through: it projects
/// the identity, `state` and every field named, each at the entity's declared type.
///
/// An immediate view is preferred. Where `reading` allows it and no immediate view qualifies, an
/// `eventual` one does (beyond10x/ess#172). [`require`] puts that requirement in an `eventually`
/// block, which waits until the projection shows the identity, the state and every fact named at
/// the values the arrangement's last step left, instead of racing it. What that proves is that the
/// implementation applied those values; no step of the scenario writes the row between that
/// observation and the command, so they are what the command reads. It proves nothing about a
/// row that did not move: a projection that is behind shows the old row too, which is why a
/// refusal's unchanged row is read with [`Reading::Immediate`] only ([`observe_unchanged`]).
fn observer<'ir>(
    ir: &'ir EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    reading: Reading,
) -> Option<&'ir ess_compiler::ir::ResolvedView> {
    let declared = ir.entity(entity);
    let projects = |view: &&ess_compiler::ir::ResolvedView| {
        !view.is_aggregate()
            && view.source == *entity
            && super::paging::read_whole(view)
            && view.filter.is_none()
            && view
                .field(&declared.identity.name)
                .is_some_and(|f| f.type_ref == declared.identity.type_ref)
            && view
                .field(EntitySpec::STATE)
                .is_some_and(|f| f.type_ref == declared.state_field().type_ref)
            && fields.iter().all(|name| {
                declared
                    .fields
                    .iter()
                    .find(|field| &field.name == name)
                    .is_some_and(|field| {
                        view.field(name)
                            .is_some_and(|shown| shown.type_ref == field.type_ref)
                    })
            })
    };
    let styled = |style: AssertionStyle| {
        ir.views()
            .values()
            .filter(projects)
            .find(|view| view.assertion_style == style)
    };
    styled(AssertionStyle::Expect).or_else(|| {
        (reading == Reading::Settling)
            .then(|| styled(AssertionStyle::Eventually))
            .flatten()
    })
}

/// Observe one stored fact on the arranged row. Kept for the replay family, which observes the
/// facts one at a time, and keeps its immediate witness: the eventual fallback of
/// [`observe_fields`] belongs to subject-fact selection (beyond10x/ess#172), and a replay's
/// observation has its own profile.
pub(super) fn observe(
    ir: &EssIr,
    entity: &EntityHandle,
    field: &str,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let fields = BTreeSet::from([field.to_owned()]);
    let row = expected_row(ir, entity, &fields, arrangement)?;
    let view = observer(ir, entity, &fields, Reading::Immediate).ok_or_else(|| {
        missing(
            entity,
            field,
            "subject fact selection requires an immediate unfiltered identity/state/fact view",
        )
    })?;
    Ok(required(view, row))
}

/// Require the arranged row, with every guarded field at the value the arrangement determined,
/// before the command runs: the facts the scenario is about are observed, not assumed.
///
/// Also the row a moving or updating branch leaves: it arrives at a state or values it did not
/// hold, so an `eventually` block waiting for them does not pass on a projection that has not
/// caught up.
pub(super) fn observe_fields(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let label = fields.iter().cloned().collect::<Vec<_>>().join(",");
    let row = expected_row(ir, entity, fields, arrangement)?;
    let view = observer(ir, entity, fields, Reading::Settling).ok_or_else(|| {
        missing(
            entity,
            &label,
            "subject fact selection requires an immediate unfiltered identity/state/fact view, \
             or an `eventual` one it waits for, and no view projects them",
        )
    })?;
    Ok(required(view, row))
}

/// Require the row a refusal left where it was, through an immediate view only — or `None`.
///
/// Nothing moved, so there is nothing for an `eventually` block to wait for: a projection that
/// has not caught up with a wrong change still shows the arranged row, and the check would pass on
/// exactly the implementation it exists to catch. Where no immediate view projects the identity,
/// the state and the fields named, the unchanged-row check is omitted rather than asserted
/// through an `eventual` one; the scenario still asserts the refusal's outcome, its error and
/// that no event was published. The same reason keeps [`absent`] on immediate views.
fn observe_unchanged(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<Option<(Vec<ScenarioStep>, ViewRef)>, RefusalCause> {
    let row = expected_row(ir, entity, fields, arrangement)?;
    Ok(observer(ir, entity, fields, Reading::Immediate).map(|view| required(view, row)))
}

/// The row an observation of the arrangement requires: its identity, its state and every field
/// named at the value the arrangement determined.
fn expected_row(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<BTreeMap<String, ScenarioValue>, RefusalCause> {
    let declared = ir.entity(entity);
    let mut row = BTreeMap::from([
        (
            declared.identity.name.clone(),
            ScenarioValue::instance(arrangement.instance.clone()),
        ),
        (
            EntitySpec::STATE.to_owned(),
            ScenarioValue::literal(Node::Text(arrangement.state.to_string())),
        ),
    ]);
    for field in fields {
        // An `Optional` field no writer can have reached holds nothing (beyond10x/ess#239). A row
        // may leave an absent field out or carry it as `null`, and the runner compares a top-level
        // field exactly, so neither form can be required here: the command's answer witnesses it.
        if !arrangement.settled.contains_key(field) && arrangement.unwritten.contains(field) {
            continue;
        }
        let Some(value) = arrangement.settled.get(field) else {
            return Err(missing(
                entity,
                field,
                "subject fact has no determined arrangement value",
            ));
        };
        row.insert(field.clone(), value.value.clone());
    }
    Ok(row)
}

/// The steps requiring `row` of `view`, in the block its consistency decides.
fn required(
    view: &ess_compiler::ir::ResolvedView,
    row: BTreeMap<String, ScenarioValue>,
) -> (Vec<ScenarioStep>, ViewRef) {
    let name = ViewRef::new(view.name.clone());
    let mut steps = Vec::new();
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Contains { fields: row },
        &mut steps,
    );
    (steps, name)
}

/// The absent-subject witness: the command sent once for an identity no row carries.
///
/// The specification declares no outcome for it, so nothing is asserted about the error. What it
/// does say is that a subject-fact branch reads a row that exists: so no declared event is
/// published and no row appears under that identity.
pub(super) fn absent(
    ir: &EssIr,
    command: &ResolvedCommand,
    subject: &ResolvedSubject,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ResolvedInstance::Supplied { field } = &subject.instance else {
        return Err(RefusalCause::StrategyWithoutGuard {
            strategy: ess_domain::command::TestStrategy::ObserveSubjectFact,
        });
    };
    let input = candidates(ir, command, &[], Distinction::PLAIN)
        .map_err(RefusalCause::NoWitness)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            missing(
                &subject.entity,
                &field.name,
                "the command has no witness input",
            )
        })?;
    let identity = input.get(&field.name).cloned().ok_or_else(|| {
        missing(
            &subject.entity,
            &field.name,
            "the witness names no identity",
        )
    })?;
    let view =
        observer(ir, &subject.entity, &BTreeSet::new(), Reading::Immediate).ok_or_else(|| {
            missing(
                &subject.entity,
                &field.name,
                "subject fact selection requires an immediate unfiltered identity/state/fact view",
            )
        })?;
    let command_ref = CommandRef::new(command.name.clone());
    let mut steps = vec![ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supply(ir, command, &input, None, None, &BTreeMap::new()),
    }];
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    let name = ViewRef::new(view.name.clone());
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Excludes {
            fields: BTreeMap::from([(
                ir.entity(&subject.entity).identity.name.clone(),
                ScenarioValue::literal(identity),
            )]),
        },
        &mut steps,
    );
    let mut source: BTreeSet<EssSemanticRef> = forbidden.into_iter().map(Into::into).collect();
    source.insert(command_ref.into());
    source.insert(name.into());
    Ok((steps, source))
}

/// What a routed branch's scenario observes around the command, beyond the arrangement.
///
/// A branch naming no subject of its own is opened by the [`absent`] witness, and after it the
/// arranged row is required again exactly as it was observed: a refusal changes nothing. A branch
/// that moves or updates the row it read is observed afterwards in the state it arrives at, with
/// the stored fields as the branch left them. Returns the steps that belong after the branch's own
/// assertions; the absent-subject steps are spliced into the arrangement. The [`boundaries`] a
/// default is further witnessed against come last, with the further rows refused on their own under
/// the scenario while it stands (a side of a counter limit, beyond10x/ess#226).
pub(super) fn around(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Result<(Vec<ScenarioStep>, Vec<RefusalCause>), RefusalCause> {
    let ir = models.arrangement;
    // Whether the scenario already arranged the second owner a link comparison names (#193).
    let mut present = reading(command, outcome)
        .and_then(|subject| other_owner(ir, &subject.entity))
        .is_some_and(|(_, other)| setup.bound.values().any(|named| named == &other));
    // Every instance name the arrangement already binds, which no further row may bind again.
    let mut taken = super::bound_instances(&setup.steps);
    taken.extend(setup.instance.iter().cloned());
    let mut refused = Vec::new();
    let (further, source) = boundaries(
        models,
        command,
        outcome,
        actors,
        (&setup.settled, setup.before.as_ref()),
        (&mut present, &mut taken),
        (&setup.steps, &mut refused),
    )?;
    let (overlapping, overlap_source) =
        overlaps(models, command, outcome, actors, (&mut present, &mut taken))?;
    let mut steps = around_row(models, command, outcome, actors, setup, supplied)?;
    models.mark(super::caller::InvocationPhase::Act, &mut steps);
    steps.extend(further);
    steps.extend(overlapping);
    setup.source.extend(source);
    setup.source.extend(overlap_source);
    Ok((steps, refused))
}

/// Whether `outcome` moves or updates the row it names.
fn moves_row(outcome: &ResolvedOutcome) -> bool {
    outcome.subject.as_ref().is_some_and(|own| {
        matches!(
            own.effect,
            ResolvedEffect::Moves { .. } | ResolvedEffect::Updates
        )
    })
}

/// Whether the row after the command differs from the arranged one in its state or in a field
/// named: only then does an `eventually` block after the command wait for something a projection
/// that is behind does not show. A move back to the state it left, or an update writing the values
/// the row already held, is read as a refusal's row is, through [`observe_unchanged`]; the generic
/// view assertion (`view_expectations`) drops its `eventually` block for the same row.
fn leaves_changed(
    before_state: Option<&super::StateName>,
    before: &BTreeMap<String, super::Determined>,
    after: &Arrangement,
    fields: &BTreeSet<String>,
) -> bool {
    before_state.is_some_and(|state| state != &after.state)
        || fields.iter().any(|field| {
            before.get(field).map(|held| &held.value)
                != after.settled.get(field).map(|held| &held.value)
        })
}

fn around_row(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Result<Vec<ScenarioStep>, RefusalCause> {
    let ir = models.arrangement;
    let Some(subject) = reading(command, outcome) else {
        return Ok(Vec::new());
    };
    // Only for the ess/9 predicate form: an ess/6 `{field, equals}` command keeps the suite it
    // generated before this construct, and its moving branch is observed as it always was.
    let changes = uses_predicate(command) && moves_row(outcome);
    if outcome.subject.is_none() {
        let (mut steps, source) = absent(ir, command, subject, actors)?;
        models.mark(super::caller::InvocationPhase::Act, &mut steps);
        setup.steps.splice(0..0, steps);
        setup.source.extend(source);
    } else if !changes {
        return Ok(Vec::new());
    }
    let (Some(instance), Some(state)) = (&setup.instance, &setup.after) else {
        return Ok(Vec::new());
    };
    let mut left = setup.settled.clone();
    if changes {
        absorb(
            &mut left,
            outcome,
            super::settled(ir, outcome, supplied, &setup.settled),
        );
    }
    let fields = guarded_fields(ir, command, &subject.entity)
        .into_iter()
        .filter(|field| left.contains_key(field))
        .collect();
    let arrangement = Arrangement {
        instance: instance.clone(),
        state: state.clone(),
        steps: Vec::new(),
        source: BTreeSet::new(),
        settled: left,
        unwritten: BTreeSet::new(),
    };
    // A refusal left the row where it was, and only an immediate read can say so.
    let observation = if changes
        && leaves_changed(setup.before.as_ref(), &setup.settled, &arrangement, &fields)
    {
        Some(observe_fields(ir, &subject.entity, &fields, &arrangement)?)
    } else {
        observe_unchanged(ir, &subject.entity, &fields, &arrangement)?
    };
    let Some((observed, view)) = observation else {
        return Ok(Vec::new());
    };
    setup.source.insert(view.into());
    Ok(observed)
}

/// What one further row of [`boundaries`] is (beyond10x/ess#226).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Further {
    /// A row isolating one child of a guard ([`goals_for`]): witnessed as this branch, or adding
    /// none where no bounded row meets it.
    Plain,
    /// One side of a counter limit: witnessed as whichever branch the command answers there.
    Limit,
    /// One side of a counter limit whose nearest value a run holds lies past [`COUNTER_REACH`]:
    /// refused under the scenario with this cause, never searched.
    Past(String),
}

/// The comparison that holds exactly where `op` does not.
fn negated_op(op: CompareOp) -> CompareOp {
    match op {
        CompareOp::Eq => CompareOp::Ne,
        CompareOp::Ne => CompareOp::Eq,
        CompareOp::Lt => CompareOp::Ge,
        CompareOp::Ge => CompareOp::Lt,
        CompareOp::Le => CompareOp::Gt,
        CompareOp::Gt => CompareOp::Le,
    }
}

/// `predicate` with every `not:` pushed down through `all:` and `any:` onto its comparisons of a
/// stored counter with a number literal, each turned into its opposite comparison: `{not: retries <
/// 3}` is `retries >= 3`, so its limit has the rows its positive form has. Any other leaf under a
/// `not:` keeps it. A predicate with no `not:` is returned as it is.
fn positive(
    predicate: &Predicate,
    counters: &BTreeMap<String, Counter>,
    negate: bool,
) -> Predicate {
    match predicate {
        Predicate::Not(inner) => positive(inner, counters, !negate),
        Predicate::All(children) => {
            let children = children
                .iter()
                .map(|child| positive(child, counters, negate))
                .collect();
            if negate {
                Predicate::Any(children)
            } else {
                Predicate::All(children)
            }
        }
        Predicate::Any(children) => {
            let children = children
                .iter()
                .map(|child| positive(child, counters, negate))
                .collect();
            if negate {
                Predicate::All(children)
            } else {
                Predicate::Any(children)
            }
        }
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } if negate && counter_leaf(predicate, counters).is_some() => Predicate::Compare {
            kind: *kind,
            left: left.clone(),
            op: negated_op(*op),
            right: right.clone(),
        },
        other if negate => Predicate::Not(Box::new(other.clone())),
        other => other.clone(),
    }
}

/// The rows either side of every counter limit in `predicate` (beyond10x/ess#226): for each
/// comparison of a stored counter with a number literal — a top-level conjunct, or a disjunct of a
/// top-level `any:`, once every `not:` is pushed onto its comparisons ([`positive`]) — one row per
/// value at the limit's edge ([`limit_edges`]), the counter pinned at the nearest value a run holds
/// on that side. A side whose nearest value lies past [`COUNTER_REACH`] is [`Further::Past`].
///
/// `holds` picks the side. `true` asks for the rows the comparison decides the predicate on: the
/// comparison held, every other disjunct beside it refuted and every other conjunct held (the
/// guarded branch's own). `false` asks for the rows it is refuted on: the conjunct it sits in
/// refuted — the comparison, or its whole `any:` — and every other conjunct held (the branch
/// answering beside it). `retries >= 3` gives `retries == 3` with it held, or `retries == 2` with
/// it refuted.
pub(super) fn limit_goals(
    counters: &BTreeMap<String, Counter>,
    predicate: &Predicate,
    holds: bool,
) -> Vec<(Goal, Further)> {
    let predicate = positive(predicate, counters, false);
    let mut conjuncts: Vec<&Predicate> = Vec::new();
    let mut pending = vec![&predicate];
    while let Some(next) = pending.pop() {
        match next {
            Predicate::All(children) => pending.extend(children.iter().rev()),
            other => conjuncts.push(other),
        }
    }
    let mut goals: Vec<(Goal, Further)> = Vec::new();
    for (at, conjunct) in conjuncts.iter().enumerate() {
        let rest: Vec<Predicate> = conjuncts
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != at)
            .map(|(_, other)| (*other).clone())
            .collect();
        // Each comparison this conjunct holds, with the disjuncts beside it.
        let disjuncts: Vec<&Predicate> = match conjunct {
            Predicate::Any(children) => children.iter().collect(),
            other => vec![other],
        };
        for (nth, leaf) in disjuncts.iter().enumerate() {
            let Some((field, op, literal)) = counter_leaf(leaf, counters) else {
                continue;
            };
            let Some(counter) = counters.get(&field) else {
                continue;
            };
            let Ok(path) = FactPath::new(&field) else {
                continue;
            };
            for (value, decided, past) in limit_edges(&field, op, literal, counter) {
                if decided != holds {
                    continue;
                }
                let pin = Predicate::Compare {
                    kind: ess_primitives::predicate::CompareKind::Value,
                    left: Operand::Fact(path.clone()),
                    op: CompareOp::Eq,
                    right: Operand::Literal(ess_primitives::facts::FactValue::Number(value)),
                };
                let mut held = rest.clone();
                let goal = if holds {
                    held.push((*leaf).clone());
                    held.push(pin);
                    let others = disjuncts
                        .iter()
                        .enumerate()
                        .filter(|(other, _)| *other != nth)
                        .map(|(_, other)| (*other).clone())
                        .collect();
                    (others, held)
                } else {
                    held.push(pin);
                    (vec![(*conjunct).clone()], held)
                };
                if !goals.iter().any(|(known, _)| known == &goal) {
                    goals.push((goal, past.map_or(Further::Limit, Further::Past)));
                }
            }
        }
    }
    goals
}

/// The rows either side of each counter limit this branch is witnessed on (beyond10x/ess#226): its
/// own predicate's rows at the limit, and — for the default — each guarded sibling's rows short of
/// it, skipping a row the ordinary witness already is. Each is witnessed as whichever branch the
/// command answers there ([`boundaries`]), so a sibling's side row another guarded sibling answers
/// (`blocked: 3..4` below `over: >= 5`, at `4`) asserts that sibling and never refuses the default.
fn counter_goals(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<(Goal, Further)> {
    let counters = counters(ir, entity);
    if counters.is_empty() {
        return Vec::new();
    }
    let default = state_default(outcome);
    let mut goals: Vec<(Goal, Further)> = Vec::new();
    for branch in guarded(command) {
        let Some(predicate) = stored(&branch.condition) else {
            continue;
        };
        let own = branch.name == outcome.name;
        if !own && !default {
            continue;
        }
        for (goal, kind) in limit_goals(&counters, &predicate, own) {
            let truth = |child: &Predicate| {
                row_truth(
                    ir,
                    entity,
                    witnessed,
                    &BTreeSet::new(),
                    witnessed_state,
                    child,
                )
            };
            let met = goal.0.iter().all(|child| truth(child) == Truth::False)
                && goal.1.iter().all(|child| truth(child) == Truth::True);
            if !met && !goals.iter().any(|(known, _)| known == &goal) {
                goals.push((goal, kind));
            }
        }
    }
    goals
}

/// Every further row [`boundaries`] witnesses a branch on — [`goals_for`]'s, then each side of a
/// counter limit ([`counter_goals`]) not already among them — each paired with what it is.
fn further_goals(
    ir: &EssIr,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    entity: &EntityHandle,
    hints: &[Predicate],
    (witnessed, witnessed_state): (
        &BTreeMap<String, super::Determined>,
        Option<&super::StateName>,
    ),
) -> Vec<(Goal, Further)> {
    let mut goals: Vec<(Goal, Further)> =
        goals_for(ir, outcome, entity, hints, witnessed, witnessed_state)
            .into_iter()
            .map(|goal| (goal, Further::Plain))
            .collect();
    for (goal, kind) in counter_goals(ir, command, outcome, entity, witnessed, witnessed_state) {
        if !goals.iter().any(|(known, _)| known == &goal) {
            goals.push((goal, kind));
        }
    }
    goals
}

/// The refusal for a further row [`boundaries`] did not witness, or `None` where it adds no row.
///
/// `cause` is `None` for a goal past [`MAX_BOUNDARIES`], which was not searched, and otherwise the
/// search's refusal with whether an input naming an arranged owner left the goal undecided. A goal
/// isolating a link comparison is refused where it was not searched or was left undecided
/// (beyond10x/ess#193). A side of a counter limit is refused where it was not searched, where the
/// search went past [`COUNTER_REACH`], or where it stopped for any cause but having left every row
/// it could; one no row the model leaves holds — `retries == 4` beside a refusal at 3 — adds none,
/// as a goal every candidate decided does for every other guard.
fn left_unwitnessed(
    (entity, command, outcome): (&EntityHandle, &ResolvedCommand, &ResolvedOutcome),
    goal: (&[Predicate], &[Predicate]),
    (linked, limit): (bool, bool),
    cause: Option<(&RefusalCause, bool)>,
) -> Option<RefusalCause> {
    match cause {
        None if linked => Some(unreached(entity, command, outcome, goal, None)),
        None if limit => Some(limit_unreached(entity, command, outcome, goal, None)),
        Some((cause, true)) if linked => {
            Some(unreached(entity, command, outcome, goal, Some(cause)))
        }
        Some((cause, _))
            if limit
                && (is_beyond_reach(cause)
                    || !matches!(cause, RefusalCause::GuardUnsatisfiable { .. })) =>
        {
            Some(limit_unreached(entity, command, outcome, goal, Some(cause)))
        }
        _ => None,
    }
}

/// `ESS-SYNTH-003` for a row either side of a counter limit the bounded search did not reach
/// (beyond10x/ess#226): the goal, the branch it is for, and why.
fn limit_unreached(
    entity: &EntityHandle,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    (refuted, held): (&[Predicate], &[Predicate]),
    cause: Option<&RefusalCause>,
) -> RefusalCause {
    let render = |predicates: &[Predicate]| {
        predicates
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let (why, tried) = match cause {
        None => (
            "past the most further rows one branch is witnessed on".to_owned(),
            0,
        ),
        Some(RefusalCause::GuardUnsatisfiable { predicate, tried }) => (predicate.clone(), *tried),
        Some(other) => (other.to_string(), 0),
    };
    RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{entity}` row for `{}/{}` with [{}] false and [{}] true, one side of a stored \
             counter's limit: {why}",
            command.name,
            outcome.name,
            render(refuted),
            render(held),
        ),
        tried,
    }
}

/// For every conjunctive stored-field guard, one goal per conjunct: that conjunct refuted and every
/// other one satisfied — skipping a goal the ordinary witness row already meets.
fn conjunct_goals(
    ir: &EssIr,
    entity: &EntityHandle,
    hints: &[Predicate],
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    let mut goals = Vec::new();
    for hint in hints {
        if let Predicate::All(conjuncts) = hint {
            goals.extend(isolating(
                ir,
                entity,
                conjuncts,
                false,
                witnessed,
                witnessed_state,
            ));
            // `defined(flag) && flag == true` cannot isolate absence while also holding the
            // comparison true. Keep the independent conjuncts (including command input), omit
            // only comparisons reading this Optional field, and let full branch selection check
            // the resulting row. Present false and absent remain distinct witnesses (#307).
            for child in conjuncts {
                let Predicate::Defined(path) = child else {
                    continue;
                };
                if path.segments().len() != 1
                    || !ir
                        .entity(entity)
                        .fields
                        .iter()
                        .any(|field| field.name == path.namespace() && field.type_ref.is_optional())
                    || row_truth(
                        ir,
                        entity,
                        witnessed,
                        &BTreeSet::new(),
                        witnessed_state,
                        child,
                    ) == Truth::False
                {
                    continue;
                }
                let dependent = |predicate: &&Predicate| {
                    matches!(predicate, Predicate::Compare { .. })
                        && predicate
                            .fact_paths()
                            .iter()
                            .any(|read| read.segments().starts_with(path.segments()))
                };
                if conjuncts.iter().any(|predicate| dependent(&predicate)) {
                    let held = conjuncts
                        .iter()
                        .filter(|predicate| *predicate != child && !dependent(predicate))
                        .cloned()
                        .collect();
                    goals.push((vec![child.clone()], held));
                }
            }
        }
    }
    goals
}

/// The further rows one branch is witnessed on: the default's, one per conjunct of a guarded
/// sibling; a guarded branch's own, one per disjunct of its predicate (beyond10x/ess#155).
fn goals_for(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    hints: &[Predicate],
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    if state_default(outcome) {
        if outcome.subject.is_none() {
            return Vec::new();
        }
        return conjunct_goals(ir, entity, hints, witnessed, witnessed_state);
    }
    stored(&outcome.condition)
        .map(|own| disjunct_goals(ir, entity, &own, witnessed, witnessed_state))
        .unwrap_or_default()
}

/// One row a further witness is arranged on: every predicate of the first list false on it, every
/// one of the second true.
pub(super) type Goal = (Vec<Predicate>, Vec<Predicate>);

/// For a disjunctive stored-field guard of the branch itself — its predicate an `any`, or an `any`
/// among its top-level conjuncts — one goal per disjunct: that disjunct satisfied and every other one
/// refuted, with the guard's remaining conjuncts satisfied (beyond10x/ess#155). A row satisfying
/// every disjunct at once is also a row of the `all` a connective mutant writes.
fn disjunct_goals(
    ir: &EssIr,
    entity: &EntityHandle,
    own: &Predicate,
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    let conjuncts: Vec<&Predicate> = match own {
        Predicate::All(children) => children.iter().collect(),
        other => vec![other],
    };
    let mut goals = Vec::new();
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
        for (refuted, mut held) in
            isolating(ir, entity, disjuncts, true, witnessed, witnessed_state)
        {
            held.extend(rest.iter().cloned());
            goals.push((refuted, held));
        }
    }
    goals
}

/// One goal per child of a connective with two or more children: that child `alone` (true for a
/// disjunct, false for a conjunct) and every other the opposite — skipping a goal the ordinary
/// witness row already meets, because a second copy proves nothing more.
fn isolating(
    ir: &EssIr,
    entity: &EntityHandle,
    children: &[Predicate],
    alone: bool,
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    if children.len() < 2 {
        return Vec::new();
    }
    let wanted = |value: bool| if value { Truth::True } else { Truth::False };
    let mut goals = Vec::new();
    for index in 0..children.len() {
        if children.iter().enumerate().all(|(other, child)| {
            row_truth(
                ir,
                entity,
                witnessed,
                &BTreeSet::new(),
                witnessed_state,
                child,
            ) == wanted((other == index) == alone)
        }) {
            continue;
        }
        let (mut falses, mut trues) = (Vec::new(), Vec::new());
        for (other, child) in children.iter().enumerate() {
            if (other == index) == alone {
                trues.push(child.clone());
            } else {
                falses.push(child.clone());
            }
        }
        goals.push((falses, trues));
    }
    goals
}

/// The most further rows the default of one command is witnessed against.
const MAX_BOUNDARIES: usize = 8;

/// Further rows the default branch is witnessed against, one per conjunct of a guarded sibling.
///
/// One row refuting a conjunctive guard shows only that *some* conjunct is read: `Express` at
/// `20` refutes `service == Express and weight_kg > 20` through the weight, and an implementation
/// refusing every parcel over 20 kg, whatever its service, passes it. So for each sibling whose
/// stored-field predicate is a conjunction of two or more conjuncts, the default is witnessed once
/// more per conjunct on a row that refutes **exactly that one** and satisfies the rest — `Standard`
/// at `21`, `Express` at `20` — each on its own instance. Bounded by [`MAX_BOUNDARIES`]; a conjunct
/// no bounded arrangement refutes alone adds no row, because the ordinary witness still stands.
///
/// The ess/9 predicate form only: an ess/6 `{field, equals}` guard is one leaf, and its suites keep
/// their bytes.
///
/// A goal holding a link comparison ([`compares_link`]) is decided with the input bound, over
/// [`linked_inputs`], and sent the owners that input names ([`bind_links`]) (beyond10x/ess#193).
/// The branch is refused with `ESS-SYNTH-003` naming the goal where the goal lies past
/// [`MAX_BOUNDARIES`], or where the bounded search did not reach it and an input naming an arranged
/// owner left it undecided: the search could not say whether a row meets it. A goal every
/// candidate decided and no bounded row meets adds no row, as for every other guard.
///
/// A side of a counter limit ([`Further::Limit`], beyond10x/ess#226) is sent as whichever branch
/// the command answers on its row, and one [`left_unwitnessed`] refuses — or whose nearest value
/// lies past the search's reach ([`Further::Past`]) — is pushed onto `refused`: that row alone is
/// refused, under the scenario, and the branch's own witness stands.
#[allow(clippy::too_many_lines)]
pub(super) fn boundaries(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (witnessed, witnessed_state): (
        &BTreeMap<String, super::Determined>,
        Option<&super::StateName>,
    ),
    (present, taken): (&mut bool, &mut BTreeSet<super::InstanceName>),
    (known, refused): (&[ScenarioStep], &mut Vec<RefusalCause>),
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    // A branch naming no subject of its own — a refusal — reads the row its siblings name.
    let Some(read) = reading(command, outcome) else {
        return Ok((steps, source));
    };
    if !uses_predicate(command) {
        return Ok((steps, source));
    }
    let entity = &read.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    let goals = further_goals(
        ir,
        (command, outcome),
        entity,
        &hints,
        (witnessed, witnessed_state),
    );
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    // A boundary may isolate a stored flag while holding an input conjunct true. Decide the
    // complete goal with that input bound, including ordinary scalar inputs, not only links.
    // `selects` reads every branch, so a goal can also read input through a sibling.
    let decided_with_input = !links(ir, command, entity).is_empty()
        || hints.iter().any(|hint| reads_input(ir, entity, hint));
    let mut rows = 0;
    // The absent-row witness a branch naming no subject of its own is opened with: a literal
    // identity the scenario sends, which no seeded row may carry (beyond10x/ess#413).
    let opened = if models.seeds.is_empty() || outcome.subject.is_some() {
        Vec::new()
    } else {
        absent(ir, command, read, actors).map_or_else(|_| Vec::new(), |(opened, _)| opened)
    };
    for ((refuted, held), kind) in goals {
        let limit = kind != Further::Plain;
        let linked = compares_link(
            ir,
            command,
            entity,
            &refuted.iter().chain(&held).cloned().collect::<Vec<_>>(),
        );
        let undecided = std::cell::Cell::new(false);
        let mut decide = |node: &Arrangement| {
            if !decided_with_input {
                let truth = |predicate: &Predicate| {
                    row_truth(
                        ir,
                        entity,
                        &node.settled,
                        &node.unwritten,
                        Some(&node.state),
                        predicate,
                    )
                };
                if refuted.iter().any(|child| truth(child) != Truth::False)
                    || held.iter().any(|child| truth(child) != Truth::True)
                {
                    return Ok(None);
                }
                if limit {
                    return answer_at(ir, command, entity, node);
                }
                return Ok(
                    reach_at(ir, command, outcome, entity, node)?.map(|input| (input, outcome))
                );
            }
            let mut unsure = undecided.get();
            let found = goal_input(
                ir,
                (command, outcome),
                (entity, node),
                (linked_inputs(ir, command, entity, node)?, &|_| Ok(true)),
                (&refuted, &held),
                &mut unsure,
            );
            // Kept whatever the goal answered, as the flag it replaces was.
            undecided.set(unsure);
            Ok(found?.map(|input| (input, outcome)))
        };
        // A side of a counter limit whose nearest value lies past the search's reach is not
        // searched; an explicitly admitted seed row is offered before it is refused (#413).
        if let Further::Past(why) = &kind {
            let refusal = limit_unreached(
                entity,
                command,
                outcome,
                (&refuted, &held),
                Some(&RefusalCause::GuardUnsatisfiable {
                    predicate: why.clone(),
                    tried: 0,
                }),
            );
            // Past the bound a goal adds no row, seeded or not.
            if rows >= MAX_BOUNDARIES {
                refused.push(refusal);
                continue;
            }
            let known = [known, &opened, &steps].concat();
            match seeded(
                models,
                command,
                (entity, read),
                (&known, &free_instance(ir, entity, rows + 1, taken), 0),
                &mut decide,
            ) {
                Ok((arrangement, (input, answering))) => {
                    rows += 1;
                    if answering.name != outcome.name {
                        source.insert(
                            OutcomeRef::new(command_ref.clone(), answering.name.clone()).into(),
                        );
                    }
                    send_for_row(
                        models,
                        command,
                        answering,
                        actors,
                        (read, &fields),
                        arrangement,
                        (&input, (&mut *present, &mut *taken)),
                        (&mut steps, &mut source),
                    )?;
                }
                Err(notes) => refused.push(annotated(refusal, &notes)),
            }
            continue;
        }
        if rows >= MAX_BOUNDARIES {
            // Past the bound a goal adds no row; one isolating a link comparison was not searched,
            // so whether a row meets it is unknown, and it is refused rather than left unwitnessed.
            match left_unwitnessed(
                (entity, command, outcome),
                (&refuted, &held),
                (linked, limit),
                None,
            ) {
                Some(refusal) if limit => refused.push(refusal),
                Some(refusal) => return Err(refusal),
                None => {}
            }
            continue;
        }
        let found = search_unbound(
            ir,
            entity,
            actors,
            &hints,
            rows + 1,
            "boundary",
            taken,
            &mut decide,
        )?;
        let (arrangement, (input, answering)) = match found {
            Ok(found) => found,
            Err(cause) => {
                // A side of a counter limit is refused on its own; any other goal refuses the
                // branch. Only a side actually refused is offered an explicitly admitted seed row.
                match left_unwitnessed(
                    (entity, command, outcome),
                    (&refuted, &held),
                    (linked, limit),
                    Some((&cause, undecided.get())),
                ) {
                    Some(refusal) if limit => {
                        let known = [known, &opened, &steps].concat();
                        match seeded(
                            models,
                            command,
                            (entity, read),
                            (
                                &known,
                                &free_instance(ir, entity, rows + 1, taken),
                                spent(&cause),
                            ),
                            &mut decide,
                        ) {
                            Ok(found) => found,
                            Err(notes) => {
                                refused.push(annotated(refusal, &notes));
                                continue;
                            }
                        }
                    }
                    Some(refusal) => return Err(refusal),
                    None => continue,
                }
            }
        };
        rows += 1;
        if answering.name != outcome.name {
            source.insert(OutcomeRef::new(command_ref.clone(), answering.name.clone()).into());
        }
        send_for_row(
            models,
            command,
            answering,
            actors,
            (read, &fields),
            arrangement,
            (&input, (&mut *present, &mut *taken)),
            (&mut steps, &mut source),
        )?;
    }
    if rows > 0 {
        source.insert(command_ref.into());
        source.insert(outcome_ref.into());
    }
    Ok((steps, source))
}

/// An input and the branch it is answered with.
type Answer<'a> = (BTreeMap<String, Node>, &'a ResolvedOutcome);

/// The first input and the branch it selects for the row `arrangement` holds, of every branch this
/// strategy arranges: the branch a further row asserts where it is one side of a counter limit,
/// whichever answers there (beyond10x/ess#226).
fn answer_at<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Option<Answer<'a>>, RefusalCause> {
    for input in inputs_for(ir, command, entity, arrangement)? {
        if let Some(branch) = selects(ir, command, entity, arrangement, &input, Order::Unique)?
            .filter(|branch| routes(command, branch))
        {
            return Ok(Some((input, branch)));
        }
    }
    Ok(None)
}

/// Which candidate inputs a further row's search may send at all, before its row is decided.
type Admits<'a> = &'a dyn Fn(&BTreeMap<String, Node>) -> Result<bool, RefusalCause>;

/// The first of `inputs` that `admits` and that sends `outcome` on `node`'s row with every predicate
/// of `refuted` false and every one of `held` true, each decided with that input bound
/// (beyond10x/ess#193). `undecided` is set where an input naming an arranged owner
/// ([`naming_owner`]) left a goal predicate `Unknown` and none decided wrong: the row may be the
/// goal's, and the search could not say.
fn goal_input(
    ir: &EssIr,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    (entity, node): (&EntityHandle, &Arrangement),
    (inputs, admits): (Vec<BTreeMap<String, Node>>, Admits<'_>),
    (refuted, held): (&[Predicate], &[Predicate]),
    undecided: &mut bool,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    let naming = naming_owner(ir, command, entity, node, &inputs);
    for (input, names) in inputs.into_iter().zip(naming) {
        if !admits(&input)? {
            continue;
        }
        let truth = |predicate: &Predicate| {
            row_truth_with(
                ir,
                entity,
                &node.settled,
                &node.unwritten,
                Some(&node.state),
                predicate,
                Some((command, &input)),
            )
        };
        let falses: Vec<Truth> = refuted.iter().map(truth).collect();
        let trues: Vec<Truth> = held.iter().map(truth).collect();
        if falses.contains(&Truth::True) || trues.contains(&Truth::False) {
            continue;
        }
        if falses.contains(&Truth::Unknown) || trues.contains(&Truth::Unknown) {
            *undecided |= names;
            continue;
        }
        if selects(ir, command, entity, node, &input, Order::Unique)?
            .is_some_and(|branch| branch.name == outcome.name)
        {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// `ESS-SYNTH-003` for a further row a guard comparing a link with an input needs and no bounded
/// arrangement gives (beyond10x/ess#193): the goal, as the predicates that must be false and true
/// on it, and the branch it is for. `cause` is the search's own refusal; `None` where the goal lies
/// past [`MAX_BOUNDARIES`] and was not searched.
fn unreached(
    entity: &EntityHandle,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    (refuted, held): (&[Predicate], &[Predicate]),
    cause: Option<&RefusalCause>,
) -> RefusalCause {
    let (why, tried) = match cause {
        None => ("past the most further rows one branch is witnessed on", 0),
        Some(RefusalCause::GuardUnsatisfiable { tried, .. }) => {
            ("over the rows bounded arrangements left", *tried)
        }
        Some(_) => ("over the rows bounded arrangements left", 0),
    };
    let render = |predicates: &[Predicate]| {
        predicates
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{entity}` row for `{}/{}` with [{}] false and [{}] true, a guard comparing the \
             owner link with an input, {why}",
            command.name,
            outcome.name,
            render(refuted),
            render(held),
        ),
        tried,
    }
}

/// The branch sent once more for a further arranged row, with what it requires, and the row
/// observed again afterwards: as the branch left it, or unchanged. The owners a link comparison's
/// input names are arranged and sent first ([`bind_links`], with `present`), and every instance
/// name the row binds is added to `taken`, so no later further row of the scenario binds it again.
#[allow(clippy::too_many_arguments)]
fn send_for_row(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (read, fields): (&ResolvedSubject, &BTreeSet<String>),
    mut arrangement: Arrangement,
    (input, (present, taken)): (
        &BTreeMap<String, Node>,
        (&mut bool, &mut BTreeSet<super::InstanceName>),
    ),
    (steps, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<(), RefusalCause> {
    let ir = models.arrangement;
    let entity = &read.entity;
    let bound = &bind_links(
        ir,
        command,
        outcome,
        entity,
        actors,
        &mut arrangement,
        input,
        present,
    )?;
    taken.extend(super::bound_instances(&arrangement.steps));
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    let (observed, view) = observe_fields(ir, entity, fields, &arrangement)?;
    arrangement.steps.extend(observed);
    models.mark(
        super::caller::InvocationPhase::Arrange,
        &mut arrangement.steps,
    );
    steps.append(&mut arrangement.steps);
    source.append(&mut arrangement.source);
    source.insert(view.into());
    let supplied = supply(
        ir,
        command,
        input,
        Some(read),
        Some(&arrangement.instance),
        bound,
    );
    steps.push(ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref,
        actor: actors.get(&command.name).cloned(),
        input: supplied.clone(),
    });
    let sent = steps.len() - 1;
    models.mark(super::caller::InvocationPhase::Act, &mut steps[sent..]);
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: outcome_ref,
    });
    match &outcome.error {
        Some(error) => steps.push(super::expect_error(
            ir,
            outcome,
            error,
            &supplied,
            &arrangement.settled,
        )),
        None => steps.push(ScenarioStep::ExpectNoError),
    }
    let held = (arrangement.state.clone(), arrangement.settled.clone());
    let mut left = arrangement.settled.clone();
    absorb(
        &mut left,
        outcome,
        super::settled(ir, outcome, &supplied, &arrangement.settled),
    );
    if let Some(transition) = outcome
        .subject
        .as_ref()
        .and_then(|own| own.effect.transition())
    {
        arrangement.state = transition.to.clone();
    }
    arrangement.settled = left;
    arrangement.unwritten = super::still_unwritten(&arrangement.unwritten, outcome);
    let kept = fields
        .iter()
        .filter(|field| arrangement.settled.contains_key(*field))
        .cloned()
        .collect();
    if leaves_changed(Some(&held.0), &held.1, &arrangement, &kept) {
        let (after, _) = observe_fields(ir, entity, &kept, &arrangement)?;
        steps.extend(after);
    } else if let Some((after, _)) = observe_unchanged(ir, entity, &kept, &arrangement)? {
        steps.extend(after);
    }
    Ok(())
}

/// Further rows an input-guarded refusal sent for an arranged row is witnessed on where it
/// overlaps an accepting branch (beyond10x/ess#178): one per accepting sibling with an input half,
/// on a row that sibling's stored guard admits, with an input both input guards admit and every
/// other input-guarded refusal refutes. The refusal is required there, and the row is observed
/// unchanged, so a target that reads the accepting branch's guards first fails.
///
/// The row-free half of the rule — a refusal over the identity, and every command reading no
/// stored field — is `overlap_inputs` in the parent module.
fn overlaps(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (present, taken): (&mut bool, &mut BTreeSet<super::InstanceName>),
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    let Some(own) = super::is_input_guarded_refusal(outcome)
        .then(|| when(outcome))
        .flatten()
    else {
        return Ok((steps, source));
    };
    let Some(read) = reading(command, outcome) else {
        return Ok((steps, source));
    };
    let entity = &read.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    let refusals: Vec<&Predicate> = super::sibling_refusals(command, outcome)
        .filter_map(when)
        .collect();
    // As in `boundaries`: a command comparing a link with an input is decided with the input bound
    // (beyond10x/ess#193). A row of an accepting guard holding that comparison is refused past the
    // bound, or where the bounded search missed it with an input naming an arranged owner leaving
    // it undecided; one every candidate decided and no bounded row meets adds no row.
    let decided_with_input = !links(ir, command, entity).is_empty();
    let mut rows = 0;
    for accepting in &command.outcomes {
        let Some(guard) = super::accepting_input_half(accepting) else {
            continue;
        };
        let row_guard = stored(&accepting.condition);
        let linked = row_guard.as_ref().is_some_and(|predicate| {
            compares_link(ir, command, entity, std::slice::from_ref(predicate))
        });
        if rows >= MAX_BOUNDARIES {
            if linked {
                return Err(unreached(
                    entity,
                    command,
                    outcome,
                    (&[], row_guard.as_slice()),
                    None,
                ));
            }
            continue;
        }
        // Numbered from past every row `boundaries` can arrange, and past every name the scenario
        // already binds, so no instance name is bound twice.
        let mut undecided = false;
        let admits = |input: &BTreeMap<String, Node>| {
            let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
            Ok(decides(&facts, &[own, guard], true)? && decides(&facts, &refusals, false)?)
        };
        let found = search_unbound(
            ir,
            entity,
            actors,
            &hints,
            MAX_BOUNDARIES + rows + 1,
            "overlap",
            taken,
            |node| {
                let inputs = if decided_with_input {
                    linked_inputs(ir, command, entity, node)?
                } else {
                    inputs_for(ir, command, entity, node)?
                };
                goal_input(
                    ir,
                    (command, outcome),
                    (entity, node),
                    (inputs, &admits),
                    (&[], row_guard.as_slice()),
                    &mut undecided,
                )
            },
        )?;
        let (arrangement, input) = match found {
            Ok(found) => found,
            Err(cause) if linked && undecided => {
                return Err(unreached(
                    entity,
                    command,
                    outcome,
                    (&[], row_guard.as_slice()),
                    Some(&cause),
                ));
            }
            Err(_) => continue,
        };
        rows += 1;
        send_for_row(
            models,
            command,
            outcome,
            actors,
            (read, &fields),
            arrangement,
            (&input, (&mut *present, &mut *taken)),
            (&mut steps, &mut source),
        )?;
    }
    if rows > 0 {
        let command_ref = CommandRef::new(command.name.clone());
        source.insert(OutcomeRef::new(command_ref.clone(), outcome.name.clone()).into());
        source.insert(command_ref.into());
    }
    Ok((steps, source))
}

pub(super) struct Preservation {
    pub before: Vec<ScenarioStep>,
    pub after: Vec<ScenarioStep>,
    pub source: BTreeSet<EssSemanticRef>,
    /// The subject fields no view let the observation cover, in name order; empty for a complete
    /// one (beyond10x/ess#132).
    pub unobserved: Vec<String>,
}

/// Every declared field must be observed. Unknown generated values are captured from
/// the implementation before the command, never filled from the expected outcome.
pub(super) fn preservation(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .expect("preservation has a subject");
    preserve_subject(ir, subject, setup)
}

pub(super) fn preserve_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    preserve(ir, subject, setup, Observation::Legacy)
}

pub(super) fn preserve_complete_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    preserve(ir, subject, setup, Observation::Complete)
}

/// [`preserve_complete_subject`] for a wrong-state refusal, which falls back to what the declared
/// views publish where no immediate views cover every subject field (beyond10x/ess#132), and
/// names the rest in [`Preservation::unobserved`].
pub(super) fn preserve_refused_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    preserve(ir, subject, setup, Observation::CompleteOrPublished)
}

/// How much of the subject a preservation has to observe.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Observation {
    /// Snapshot and comparison over immediate views covering every field (before `ess/7`).
    Legacy,
    /// The complete typed snapshot over immediate views covering every field.
    Complete,
    /// [`Self::Complete`], or else whatever the declared views publish.
    CompleteOrPublished,
}

fn preserve(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
    observation: Observation,
) -> Result<Preservation, RefusalCause> {
    let complete = observation != Observation::Legacy;
    let entity = ir.entity(&subject.entity);
    let instance = setup
        .instance
        .as_ref()
        .expect("preservation arranged an existing subject");
    let mut required: BTreeSet<String> = entity
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect();
    required.insert(entity.identity.name.clone());
    required.insert(EntitySpec::STATE.into());
    let mut before = Vec::new();
    let mut after = Vec::new();
    let mut source = BTreeSet::new();
    for view in ir.views().values().filter(|view| {
        !view.is_aggregate()
            && view.source == subject.entity
            && super::paging::read_whole(view)
            && setup.after.as_ref().is_some_and(|state| {
                shows(ir, view, state, &setup.settled, &BTreeMap::new()) == Ok(true)
            })
            && view.assertion_style == AssertionStyle::Expect
            && view
                .field(&entity.identity.name)
                .is_some_and(|field| field.type_ref == entity.identity.type_ref)
    }) {
        let name = ViewRef::new(view.name.clone());
        for field in &view.fields {
            let exact = if field.name == entity.identity.name {
                field.type_ref == entity.identity.type_ref
            } else if field.name == EntitySpec::STATE {
                field.type_ref == entity.state_field().type_ref
            } else {
                entity.fields.iter().any(|declared| {
                    declared.name == field.name && declared.type_ref == field.type_ref
                })
            };
            if exact {
                required.remove(&field.name);
            }
        }
        before.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        let selected = [(
            entity.identity.name.clone(),
            ScenarioValue::instance(instance.clone()),
        )]
        .into_iter()
        .collect();
        before.push(if complete {
            let shape = crate::subject::SubjectShape::of(ir, view, &entity.identity.name).map_err(
                |reason| {
                    RefusalCause::NoWitness(WitnessGap {
                        path: format!("{name}: {reason}"),
                        type_ref: "complete subject observation".into(),
                        reason: "complete subject requires a finite exact typed observer",
                    })
                },
            )?;
            ScenarioStep::SnapshotCompleteSubject {
                view: name.clone(),
                subject: selected,
                shape,
            }
        } else {
            ScenarioStep::SnapshotSubject {
                view: name.clone(),
                subject: selected,
            }
        });
        after.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        after.push(if complete {
            ScenarioStep::ExpectCompleteSubjectUnchanged { view: name.clone() }
        } else {
            ScenarioStep::ExpectSubjectUnchanged { view: name.clone() }
        });
        source.insert(name.into());
    }
    let observed = Preservation {
        before,
        after,
        source,
        unobserved: Vec::new(),
    };
    covered(ir, subject, setup, observation, observed, required)
}

/// The observation, where the immediate views covered every subject field. Where they fall short,
/// a refusal's observation keeps what they and the `eventual` views observe, with the rest named as
/// unobserved (beyond10x/ess#132); anything else — or nothing observing the row at all — refuses.
fn covered(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
    observation: Observation,
    mut observed: Preservation,
    mut required: BTreeSet<String>,
) -> Result<Preservation, RefusalCause> {
    if required.is_empty() {
        return Ok(observed);
    }
    if observation != Observation::CompleteOrPublished {
        return Err(missing(
            &subject.entity,
            &required.into_iter().collect::<Vec<_>>().join(","),
            "preservation requires immediate identity views covering every subject field",
        ));
    }
    let instance = setup
        .instance
        .as_ref()
        .expect("preservation arranged an existing subject");
    let partial = eventual_observation(ir, subject, setup, instance, &mut required);
    if observed.before.is_empty() && partial.after.is_empty() {
        return Err(missing(
            &subject.entity,
            &required.into_iter().collect::<Vec<_>>().join(","),
            "a refused subject is observed through a view of the entity projecting its \
             identity, and none shows this row",
        ));
    }
    observed.after.extend(partial.after);
    observed.source.extend(partial.source);
    observed.unobserved = required.into_iter().collect();
    Ok(observed)
}

/// A complete refusal's observation where no set of immediate views covers every subject field
/// (beyond10x/ess#132): the fields the declared views do publish, each required after the refusal
/// to hold what the arrangement left there.
///
/// An `eventual` identity/state view is the honest declaration for an entity whose other fields the
/// implementation cannot read back, and requiring a view that claims a consistent store of every
/// field would make the specification false. So each `eventual` view that shows the arranged row is
/// required, in its own block, to hold that row with its identity, its state and every field the
/// arrangement settled and the view projects at the entity's type — and nothing more, so the
/// scenario's own steps record exactly which part of the subject was observed. The immediate views
/// that did qualify keep their complete snapshot and comparison beside it. Empty where no eventual
/// view shows the row; with no immediate view either there is nothing to observe, and the caller's
/// refusal stands.
///
/// What an `eventual` read proves is weaker, and [`Note::PartialObservation`](super::Note) says so:
/// a refusal leaves the row where it was, so there is no later state to poll for, and a projection
/// that has not yet caught up with a wrong change still shows the arranged row on its first read.
/// The check catches a wrong change the projection has already applied, and nothing sooner.
fn eventual_observation(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
    instance: &super::InstanceName,
    unobserved: &mut BTreeSet<String>,
) -> Preservation {
    let entity = ir.entity(&subject.entity);
    let mut after = Vec::new();
    let mut source = BTreeSet::new();
    let Some(state) = setup.after.as_ref() else {
        return Preservation {
            before: Vec::new(),
            after,
            source,
            unobserved: Vec::new(),
        };
    };
    for view in ir.views().values().filter(|view| {
        !view.is_aggregate()
            && view.source == subject.entity
            && super::paging::read_whole(view)
            && view.assertion_style == AssertionStyle::Eventually
            && shows(ir, view, state, &setup.settled, &BTreeMap::new()) == Ok(true)
            && view
                .field(&entity.identity.name)
                .is_some_and(|field| field.type_ref == entity.identity.type_ref)
    }) {
        let mut row = BTreeMap::from([(
            entity.identity.name.clone(),
            ScenarioValue::instance(instance.clone()),
        )]);
        if view
            .field(EntitySpec::STATE)
            .is_some_and(|field| field.type_ref == entity.state_field().type_ref)
        {
            row.insert(
                EntitySpec::STATE.to_owned(),
                ScenarioValue::literal(Node::Text(state.to_string())),
            );
        }
        for field in &view.fields {
            let Some(determined) = setup.settled.get(&field.name) else {
                continue;
            };
            if determined.type_ref == field.type_ref
                && entity
                    .fields
                    .iter()
                    .any(|declared| declared.name == field.name)
            {
                row.insert(field.name.clone(), determined.value.clone());
            }
        }
        for observed in row.keys() {
            unobserved.remove(observed);
        }
        let name = ViewRef::new(view.name.clone());
        require(
            view,
            &name,
            BTreeMap::new(),
            ViewExpectation::Contains { fields: row },
            &mut after,
        );
        source.insert(name.into());
    }
    Preservation {
        before: Vec::new(),
        after,
        source,
        unobserved: Vec::new(),
    }
}

// ---- explicit synthesis seeds (beyond10x/ess#413, `docs/design/synthesis-seeds.md`) -----------

/// A seeded arrangement and the input that selects the branch on it, or why each admitted row of
/// the entity could not be offered.
pub(super) type Seeded = Result<(Setup, BTreeMap<String, Node>), Vec<String>>;

/// [`prepare`] from an explicitly admitted seed row, where no bounded arrangement selects the
/// branch: the first eligible row, in admitted order, on which an input grounded from the row
/// itself selects it. The row is established, observed, and then sent the command, exactly as an
/// arranged row is. `Err` carries why rows of the entity could not be offered.
pub(super) fn prepare_seeded(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    ordinary: &RefusalCause,
) -> Result<Seeded, RefusalCause> {
    let ir = models.arrangement;
    let Some(subject) = reading(command, outcome) else {
        return Ok(Err(Vec::new()));
    };
    let entity = &subject.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    // The absent-row witness this branch is opened with, where it names no subject of its own.
    let opened = if outcome.subject.is_none() {
        absent(ir, command, subject, actors).map_or_else(|_| Vec::new(), |(opened, _)| opened)
    } else {
        Vec::new()
    };
    let instance = super::instance_name(&ir.entity(entity).name, Distinction::PLAIN);
    let found = seeded(
        models,
        command,
        (entity, subject),
        (&opened, &instance, spent(ordinary)),
        &mut |node: &Arrangement| {
            if let Some(input) = reach_linked(
                ir,
                command,
                outcome,
                entity,
                node,
                &|_, _| true,
                Order::Unique,
            )? {
                return Ok(Some((input, false)));
            }
            if super::is_input_guarded_refusal(outcome) {
                return Ok(
                    refusal_first(ir, command, outcome, entity, node)?.map(|input| (input, true))
                );
            }
            Ok(None)
        },
    );
    let (mut arrangement, (input, before_row)) = match found {
        Ok(found) => found,
        Err(notes) => return Ok(Err(notes)),
    };
    let bound = bind_links(
        ir,
        command,
        outcome,
        entity,
        actors,
        &mut arrangement,
        &input,
        &mut false,
    )?;
    observe_prepared(ir, entity, fields, before_row, &mut arrangement)?;
    let after = match outcome.subject.as_ref().map(|own| &own.effect) {
        Some(ResolvedEffect::Deletes) => None,
        effect => Some(effect.and_then(ResolvedEffect::transition).map_or_else(
            || arrangement.state.clone(),
            |transition| transition.to.clone(),
        )),
    };
    Ok(Ok((
        Setup {
            steps: arrangement.steps,
            instance: Some(arrangement.instance),
            bound,
            source: arrangement.source,
            after,
            before: Some(arrangement.state),
            settled: arrangement.settled,
        },
        input,
    )))
}

/// The rows the bounded search of an obligation visited before it was refused: what its seed
/// attempts leave of the arrangement budget ([`MAX_NODES`]) they share with that search.
pub(super) fn spent(cause: &RefusalCause) -> usize {
    match cause {
        RefusalCause::GuardUnsatisfiable { tried, .. } => *tried,
        _ => 0,
    }
}

/// The first admitted seed row of `entity`, in admitted order, that `goal` accepts, established as
/// `instance` — or why each row of the entity that could not be offered was not. A row the goal
/// merely does not hold adds no note: it answers another obligation. Each row offered to `goal` is
/// one node of the arrangement budget the obligation's own search already `spent` some of; no seed
/// is offered once the budget is gone.
fn seeded<T>(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    (entity, subject): (&EntityHandle, &ResolvedSubject),
    (known, instance, spent): (&[ScenarioStep], &super::InstanceName, usize),
    goal: &mut impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), Vec<String>> {
    let ir = models.arrangement;
    let declared = EntityRef::from(entity);
    let mut notes = Vec::new();
    let mut budget = MAX_NODES.saturating_sub(spent);
    for seed in models
        .seeds
        .rows()
        .iter()
        .filter(|seed| seed.entity == declared)
    {
        let name = format!("`{}#{}`", seed.source.as_str(), seed.instance);
        let why = seed_unsupported(ir, command, entity, subject)
            .map(str::to_owned)
            .or_else(|| {
                collides(known, &declared, &seed.identity).then(|| {
                    "its identity collides with an identity the scenario already establishes, \
                     sends or observes"
                        .to_owned()
                })
            });
        if let Some(why) = why {
            notes.push(format!("synthesis seed {name} not applied: {why}"));
            continue;
        }
        if budget == 0 {
            notes.push(format!(
                "synthesis seed {name} not applied: the arrangement budget of {MAX_NODES} rows \
                 this obligation shares with its bounded search is spent"
            ));
            break;
        }
        budget -= 1;
        let node = seed_arrangement(ir, entity, seed, instance.clone());
        match goal(&node) {
            Ok(Some(found)) => return Ok((node, found)),
            Ok(None) => {}
            Err(cause) => notes.push(format!("synthesis seed {name} not applied: {cause}")),
        }
    }
    Err(notes)
}

/// Why no seed row can be offered for `command` reading `subject`: a row that belongs to an owner,
/// a command comparing a link to one, or a row the command does not name by an input. Seeds
/// establish one independent row; related and owned arrangements stay ordinary or refused.
fn seed_unsupported(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    subject: &ResolvedSubject,
) -> Option<&'static str> {
    if ir.owner_of(entity).is_some() {
        return Some("the row belongs to an owner, which a seed does not establish");
    }
    if !links(ir, command, entity).is_empty() {
        return Some("the command compares a link to an owner, which a seed does not establish");
    }
    if !matches!(subject.instance, ResolvedInstance::Supplied { .. }) {
        return Some("the command does not name the row it reads by an input");
    }
    None
}

/// Whether `identity` of `entity` is already established by a step the scenario holds, or is sent
/// or observed anywhere in one as a value: a seeded row never shares an identity with another row
/// or an absence witness of the same scenario, and is never renamed to avoid one.
fn collides(known: &[ScenarioStep], entity: &EntityRef, identity: &Node) -> bool {
    fn holds(value: &serde_json::Value, wanted: &serde_json::Value) -> bool {
        value == wanted
            || match value {
                serde_json::Value::Array(items) => items.iter().any(|item| holds(item, wanted)),
                serde_json::Value::Object(members) => {
                    members.values().any(|member| holds(member, wanted))
                }
                _ => false,
            }
    }
    let Ok(wanted) = serde_json::to_value(identity) else {
        return true;
    };
    known.iter().any(|step| match step {
        ScenarioStep::EstablishEntity {
            entity: other,
            identity: held,
            ..
        } => other == entity && held == identity,
        other => serde_json::to_value(other).is_ok_and(|value| holds(&value, &wanted)),
    })
}

/// The row an admitted seed is, as an arrangement: established by one `establish_entity` step,
/// every literal field settled at its declared type, and every Optional field it leaves out absent.
fn seed_arrangement(
    ir: &EssIr,
    entity: &EntityHandle,
    seed: &SeedRecord,
    instance: super::InstanceName,
) -> Arrangement {
    let declared = ir.entity(entity);
    let mut types = BTreeSet::new();
    for field in declared
        .fields
        .iter()
        .chain(std::iter::once(&declared.identity))
    {
        super::reachable_types(ir, &field.type_ref, &mut types);
    }
    let mut source = BTreeSet::from([EssSemanticRef::from(EntityRef::from(entity))]);
    source.extend(types.into_iter().map(EssSemanticRef::from));
    Arrangement {
        instance: instance.clone(),
        state: seed.state.clone(),
        steps: vec![ScenarioStep::EstablishEntity {
            instance,
            entity: seed.entity.clone(),
            identity: seed.identity.clone(),
            fields: seed.fields.clone(),
            state: seed.state.clone(),
        }],
        source,
        settled: declared
            .fields
            .iter()
            .filter_map(|field| {
                seed.fields.get(&field.name).map(|value| {
                    (
                        field.name.clone(),
                        super::Determined {
                            value: ScenarioValue::literal(value.clone()),
                            type_ref: field.type_ref.clone(),
                        },
                    )
                })
            })
            .collect(),
        unwritten: declared
            .fields
            .iter()
            .filter(|field| field.type_ref.is_optional() && !seed.fields.contains_key(&field.name))
            .map(|field| field.name.clone())
            .collect(),
    }
}

/// The first further instance name from `first` no step of the scenario binds yet: where a seeded
/// further row is established, numbered as [`search_unbound`] numbers its rows.
fn free_instance(
    ir: &EssIr,
    entity: &EntityHandle,
    first: usize,
    taken: &BTreeSet<super::InstanceName>,
) -> super::InstanceName {
    let name = &ir.entity(entity).name;
    // Of `taken.len() + 1` distinct names, at least one is not taken.
    (first..=first + taken.len())
        .map(|nth| super::instance_name(name, Distinction::further(nth)))
        .find(|instance| !taken.contains(instance))
        .expect("one of more names than are taken is free")
}

/// `refusal`, saying why each admitted seed row of its entity was not applied.
pub(super) fn annotated(refusal: RefusalCause, notes: &[String]) -> RefusalCause {
    match refusal {
        RefusalCause::GuardUnsatisfiable { predicate, tried } if !notes.is_empty() => {
            RefusalCause::GuardUnsatisfiable {
                predicate: format!("{predicate}; {}", notes.join("; ")),
                tried,
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod counter413a_arithmetic_completeness {
    use super::{reachable, BTreeSet, Counter, Number};

    fn counter(amounts: &[i64]) -> Counter {
        Counter {
            starts: Some(vec![Number::from(0_i64)]),
            amounts: amounts.iter().copied().map(Number::from).collect(),
        }
    }

    #[test]
    fn max_literal_cannot_drop_the_required_upper_window() {
        let result = reachable(&counter(&[1]), Number::from(i64::MAX));
        assert!(
            result.is_none(),
            "overflowing upper window must be incomplete, not a complete tiny set: {result:?}"
        );
    }

    #[test]
    fn min_literal_cannot_drop_the_required_lower_window() {
        let result = reachable(&counter(&[-1]), Number::from(i64::MIN));
        assert!(
            result.is_none(),
            "overflowing lower window must be incomplete, not a complete tiny set: {result:?}"
        );
    }

    #[test]
    fn representable_windows_with_unrepresentable_padding_remain_incomplete() {
        for (literal, step) in [(i64::MAX - 16, 1), (i64::MIN + 16, -1)] {
            let result = reachable(&counter(&[step]), Number::from(literal));
            assert!(
                result.is_none(),
                "widest-step padding at {literal} must remain checked: {result:?}"
            );
        }
    }

    #[test]
    fn an_unrepresentable_negative_magnitude_cannot_be_omitted_from_the_widest_step() {
        // The positive sibling makes filter_map's silent omission observable: it cannot stand in
        // for the larger negative step whose magnitude does not fit the signed arithmetic.
        let result = reachable(&counter(&[i64::MIN, 1]), Number::from(0_i64));
        assert!(
            result.is_none(),
            "every step's required magnitude must be representable: {result:?}"
        );
    }

    #[test]
    fn finite_two_and_empty_start_completeness_are_unchanged() {
        let result = reachable(&counter(&[1]), Number::from(2_i64));
        let expected: BTreeSet<_> = (0_i64..=19).map(Number::from).collect();
        assert_eq!(result, Some(expected));
        let mut empty = counter(&[i64::MIN, 1]);
        empty.starts = Some(Vec::new());
        assert_eq!(
            reachable(&empty, Number::from(i64::MAX)),
            Some(BTreeSet::new())
        );
    }
}

#[cfg(test)]
mod seed_budget {
    use super::{reading, seeded, spent, Arrangement, RefusalCause, MAX_NODES};
    use crate::authored::Source;
    use crate::synthesize::caller::InvocationModels;
    use crate::synthesize::{AdmittedSeeds, SeedSelection};

    const MODEL: &str = "format: ess/20
system: counter
version: v1
domain: counter.model
entities:
  - name: counter.model.Counter
    identity: {name: id, type: Uuid}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
commands:
  - name: counter.model.Create
    outcomes:
      - name: created
        creates: counter.model.Counter
        instance: id
        sets: {revision: 0}
        emits: [counter.model.Created]
        payload:
          counter.model.Created: {id: {generated: true}}
  - name: counter.model.Authorize
    input:
      - {name: id, type: Uuid}
    outcomes:
      - name: exhausted
        when_subject: {predicate: revision >= 100}
        error: counter.model.Exhausted
      - name: authorized
        updates: counter.model.Counter
        instance: id
        sets: {revision: {increment: 1}}
        emits: [counter.model.Authorized]
        payload:
          counter.model.Authorized: {id: input.id}
errors:
  - {name: counter.model.Exhausted, fields: []}
events:
  - name: counter.model.Created
    fields: [{name: id, type: Uuid}]
  - name: counter.model.Authorized
    fields: [{name: id, type: Uuid}]
views:
  - name: counter.model.Counters
    source: counter.model.Counter
    consistency: read_your_writes
    fields:
      - {name: id, type: Uuid}
      - {name: revision, type: Integer}
";

    const SEED: &str = "type: ess-scenario/2
domain: counter.model
scenario: counter-row
summary: One counter row.
arrange:
  - instance: row
    entity: counter.model.Counter
    setup:
      identity: 00000000-0000-4000-8000-00000000e001
      fields: {revision: 7}
      state: Active
assert:
  - view: counter.model.Counters
    contains: {id: {$instance: row}, revision: 7}
";

    /// Seed attempts share the arrangement budget their obligation's bounded search spent part of
    /// (beyond10x/ess#413, `docs/design/synthesis-seeds.md`): with the budget left, a row is
    /// offered; with it gone, none is, and the refusal says why.
    #[test]
    fn a_seed_attempt_is_one_node_of_the_budget_its_search_spent() {
        use ess_compiler::{resolve::compile, source::SourceMap};
        use ess_domain::{spec::RawSpecFile, system::Source as File, Specification};
        let spec = Specification::assemble([(
            File::new("counter.yaml"),
            RawSpecFile::parse(MODEL).unwrap(),
        )])
        .unwrap_or_else(|errors| panic!("{errors}"));
        let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
        let seeds = AdmittedSeeds::compile(
            &ir,
            &[SeedSelection {
                source: Source::new("row.yaml", SEED),
                instance: crate::InstanceName::new("row").unwrap(),
            }],
        )
        .unwrap_or_else(|error| panic!("{error}"));
        let models = InvocationModels::seeded(&ir, &seeds);
        let command = ir
            .commands()
            .values()
            .find(|command| command.name.to_string() == "counter.model.Authorize")
            .unwrap();
        let outcome = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.to_string() == "authorized")
            .unwrap();
        let subject = reading(command, outcome).unwrap();
        let instance = crate::InstanceName::new("counter").unwrap();
        let mut offered = 0;
        let mut accept = |_: &Arrangement| -> Result<Option<()>, RefusalCause> {
            offered += 1;
            Ok(Some(()))
        };
        let left = seeded(
            &models,
            command,
            (&subject.entity, subject),
            (&[], &instance, MAX_NODES - 1),
            &mut accept,
        );
        assert!(left.is_ok());
        let spent_all = seeded(
            &models,
            command,
            (&subject.entity, subject),
            (&[], &instance, MAX_NODES),
            &mut accept,
        );
        let Err(notes) = spent_all else {
            panic!("a seed was offered past the budget")
        };
        assert_eq!(offered, 1, "no row is offered once the budget is spent");
        assert!(notes[0].contains("budget"), "{notes:?}");
        assert_eq!(
            spent(&RefusalCause::GuardUnsatisfiable {
                predicate: String::new(),
                tried: 9
            }),
            9
        );
    }
}
