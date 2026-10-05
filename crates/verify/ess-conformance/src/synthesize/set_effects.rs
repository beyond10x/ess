//! Set effects over filtered instances (ess/16, beyond10x/ess#167, #175,
//! `docs/design/set-effects-over-filtered-instances.md`), witnessed on rows arranged on both sides
//! of the filter.
//!
//! * **`instances:`** gets its own `/outcome/` scenario ([`set_scenario`]): three rows the filter
//!   selects, one row per conjunct of the filter that fails that conjunct alone ([`misses`]), and
//!   — for a `moves:` — one selected row resting outside the transition's `from` states; then the
//!   command, the event with `{count: changed}` equal to the rows it had to change, and every row
//!   read back: the changed ones in their new state with what `sets:` wrote, the others as they
//!   were arranged. The command is then sent again under an input the filter selects no row by
//!   ([`zero_match`]): accepted, a count of 0, and every row as the first call left it.
//! * **`affects:`** adds a segment to the branch's own scenario ([`affects_segment`]): a subject
//!   under a fresh name, three rows each entry's filter selects and one per conjunct it leaves
//!   out, `subject.<field>` conjuncts included, the command, then the subject as the branch leaves
//!   it, the selected rows with what the entry's `sets:` wrote, and the others as they were. An
//!   entry that moves its rows (ess/22, beyond10x/ess#229) is witnessed as an `instances:` move
//!   ([`affect_rows`]): its changed rows rest in the move's `from` states and are read back in its
//!   arrival state, and one row the filter selects rests outside them and is read back unmoved.
//!
//! Every row is read from a view that publishes the entity's identity, unfiltered, whole and
//! immediate, and that publishes the state a set move leaves and every field the effect writes
//! ([`observed`]); where no such view exists the scenario is refused by name rather than filed
//! without the observation. Every step used is an existing one, so a suite carrying either construct keeps
//! the format its other steps select.
//!
//! The claims are about every stored row: the count and the unchanged rows hold on a target no
//! other scenario writes to at the same time, which is what §8 requires of a shared one.

use ess_compiler::ir::{ResolvedAffect, ResolvedSetSubject};
use ess_domain::view::Consistency;
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Truth};

use super::caller::{InvocationModels, InvocationPhase};
use crate::witness::{Distinction, WitnessGap};

use super::{
    arrange_toward_bound, arrange_toward_filter, determined_payload, fact_value,
    has_subject_guards, insert, instance_name, lifecycle_state, not_emitted, prepare_in, reach,
    reach_in_state, require, settled, shown, subject_fact, supply, ActorRef, Arrangement,
    AssertionStyle, BTreeMap, BTreeSet, CommandRef, ConformanceScenario, ConformanceSuite,
    Determined, EntityHandle, EntityRef, EssIr, EssSemanticRef, EventRef, Focus, InstanceName,
    Node, OutcomeRef, QualifiedName, Refusal, RefusalCause, ResolvedCommand, ResolvedEffect,
    ResolvedOutcome, ResolvedView, ScenarioId, ScenarioStep, ScenarioValue, StateName,
    ViewExpectation, ViewRef,
};

/// How many rows the filter must select, so a target that changes one of them is caught.
const MATCHING: usize = 3;

/// Every set outcome's scenario, and every `affects:` segment.
pub(super) fn set_effects(
    models: &InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for command in models.acting.commands().values() {
        if !focus.takes(&command.name) {
            continue;
        }
        for outcome in &command.outcomes {
            let id = ScenarioId::Outcome {
                outcome: OutcomeRef::new(
                    CommandRef::new(command.name.clone()),
                    outcome.name.clone(),
                ),
            };
            if let Some(set) = &outcome.instances {
                match set_scenario(models, command, outcome, set, actors) {
                    Ok(scenario) => insert(suite, id.clone(), scenario, refusals),
                    Err(cause) => refusals.push(Refusal::about(&id, cause)),
                }
            }
            if outcome.affects.is_empty() {
                continue;
            }
            let Some(scenario) = suite.scenarios.get(&id) else {
                // The branch's own scenario was refused, and says why.
                continue;
            };
            let mut taken = captured(&scenario.steps);
            match affects_segment(models, command, outcome, actors, &mut taken) {
                Ok((steps, source)) => {
                    let scenario = suite
                        .scenarios
                        .get_mut(&id)
                        .expect("checked to be filed above");
                    scenario.steps.extend(steps);
                    scenario.source.extend(source);
                }
                Err(cause) => {
                    suite.scenarios.remove(&id);
                    refusals.push(Refusal::about(&id, cause));
                }
            }
        }
    }
}

/// Every instance name a list of steps captured.
fn captured(steps: &[ScenarioStep]) -> BTreeSet<InstanceName> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CaptureInstance { instance, .. } => Some(instance.clone()),
            _ => None,
        })
        .collect()
}

/// The lowest further distinction whose instance name nothing in the scenario holds yet, claimed.
fn next(ir: &EssIr, entity: &EntityHandle, taken: &mut BTreeSet<InstanceName>) -> Distinction {
    let name = &ir.entity(entity).name;
    (1..=taken.len() + 1)
        .map(Distinction::further)
        .find(|distinction| taken.insert(instance_name(name, *distinction)))
        .unwrap_or(Distinction::UNKNOWN)
}

/// The views a row of `entity` is read back from: row-level, publishing the identity at its own
/// type, unfiltered, unparameterised, unpaged and immediate — a view that holds every row.
fn observing<'i>(ir: &'i EssIr, entity: &EntityHandle) -> Vec<&'i ResolvedView> {
    let identity = &ir.entity(entity).identity;
    ir.views()
        .values()
        .filter(|view| {
            !view.is_aggregate()
                && view.source == *entity
                && view
                    .field(&identity.name)
                    .is_some_and(|field| field.type_ref == identity.type_ref)
                && view.filter.is_none()
                && view.params.is_empty()
                && view.paging.is_none()
                && view.consistency == Consistency::ReadYourWrites
                && view.assertion_style == AssertionStyle::Expect
        })
        .collect()
}

/// [`observing`], narrowed to the views that publish what the effect changes — the state a set
/// move leaves where `moves` is set, and every field `sets` writes, each at the entity's type —
/// or a refusal where none does: a row read back where the change cannot be seen is no
/// observation, and a scenario built on it would pass a target changing the wrong rows.
fn observed<'i>(
    ir: &'i EssIr,
    entity: &EntityHandle,
    at: String,
    (moves, sets): (bool, &[ess_compiler::ir::ResolvedPayloadField]),
) -> Result<Vec<&'i ResolvedView>, RefusalCause> {
    if !moves && sets.is_empty() {
        return Err(gap(
            at,
            entity.to_string(),
            "is updated by a set effect that writes no field, so no read can tell a changed row \
             from one left alone",
        ));
    }
    let initial = &ir.entity(entity).lifecycle.initial;
    let publishes = |view: &&ResolvedView| {
        (!moves || lifecycle_state(ir, entity, view, initial).is_some())
            && sets.iter().all(|set| {
                view.field(&set.target)
                    .is_some_and(|field| field.type_ref == set.target_type)
            })
    };
    let views: Vec<&ResolvedView> = observing(ir, entity)
        .into_iter()
        .filter(publishes)
        .collect();
    if views.is_empty() {
        return Err(gap(
            at,
            entity.to_string(),
            "is published by no immediate, unfiltered, unparameterised, unpaged view carrying \
             its identity, the state a set move leaves and every field the effect writes, so \
             the rows it changes and the rows it leaves cannot be told apart when read back",
        ));
    }
    Ok(views)
}

fn gap(path: String, type_ref: String, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path,
        type_ref,
        reason,
    })
}

/// The value a path under `input.` names in the input a scenario sends.
fn input_value(path: &FactPath, input: &BTreeMap<String, Node>) -> Option<FactValue> {
    let (root, rest) = path.segments().split_first()?;
    if root != ess_domain::command::subject_fact::INPUT_NAMESPACE {
        return None;
    }
    let (first, rest) = rest.split_first()?;
    let mut node = input.get(first)?;
    for segment in rest {
        let Node::Map(entries) = node else {
            return None;
        };
        node = entries.get(segment)?;
    }
    fact_value(node)
}

/// The value a path under `subject.` names in what the subject held before the outcome.
fn subject_value(path: &FactPath, before: &BTreeMap<String, Determined>) -> Option<FactValue> {
    let (root, rest) = path.segments().split_first()?;
    if root != ess_domain::command::set_effects::SUBJECT_NAMESPACE {
        return None;
    }
    let (first, rest) = rest.split_first()?;
    let mut node = before.get(first)?.value.as_literal()?;
    for segment in rest {
        let Node::Map(entries) = node else {
            return None;
        };
        node = entries.get(segment)?;
    }
    fact_value(node)
}

/// `filter` with every comparison operand `read` answers written in as a literal; the other
/// operands are kept.
fn written_in(filter: &Predicate, read: &dyn Fn(&FactPath) -> Option<FactValue>) -> Predicate {
    let operand = |it: &Operand| match it {
        Operand::Fact(path) => read(path).map_or_else(|| it.clone(), Operand::Literal),
        Operand::Offset(offset) => read(&offset.base)
            .and_then(|base| offset.value_at(&base))
            .map_or_else(|| it.clone(), Operand::Literal),
        Operand::Literal(_) => it.clone(),
    };
    match filter {
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| written_in(child, read))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| written_in(child, read))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(written_in(inner, read))),
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } => Predicate::Compare {
            kind: *kind,
            left: operand(left),
            op: *op,
            right: operand(right),
        },
        other => other.clone(),
    }
}

/// What a set effect leaves in one row: what the row held, with what `sets:` writes over it.
fn after(
    ir: &EssIr,
    sets: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Determined> {
    let mut out = before.clone();
    out.extend(settled(ir, sets, supplied, before));
    out
}

/// Whether every field `sets:` determines reads differently on `row` after the effect: what makes
/// a target that skips the row, or changes one it should not, visible on it.
fn visibly_changed(
    ir: &EssIr,
    sets: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    row: &BTreeMap<String, Determined>,
) -> bool {
    settled(ir, sets, supplied, row)
        .iter()
        .all(|(field, written)| {
            row.get(field).is_some_and(|held| {
                held.value.as_literal().is_some() && held.value != written.value
            })
        })
}

/// What one branch's filter selects rows by, and what it does to them.
struct Selection<'a> {
    ir: &'a EssIr,
    command: &'a ResolvedCommand,
    entity: &'a EntityHandle,
    /// The filter, with the subject's values written in.
    filter: Predicate,
    /// The same, with the input's values written in too: what the arranging search is steered by.
    closed: Predicate,
    input: &'a BTreeMap<String, Node>,
    supplied: &'a BTreeMap<String, ScenarioValue>,
    /// An outcome carrying the `sets:` the selected rows take.
    sets: &'a ResolvedOutcome,
    /// Subject identity operands keep the captured name, never a guessed identity literal.
    symbols: BTreeMap<FactPath, InstanceName>,
    /// A second real instance of the subject, for the identity conjunct's nonmatching witness.
    other: Option<InstanceName>,
}

impl Selection<'_> {
    fn truth_of(&self, predicate: &Predicate, row: &Arrangement) -> Truth {
        match predicate {
            Predicate::All(children) => {
                return children.iter().fold(Truth::True, |truth, child| {
                    truth.and(self.truth_of(child, row))
                })
            }
            Predicate::Any(children) => {
                return children.iter().fold(Truth::False, |truth, child| {
                    truth.or(self.truth_of(child, row))
                })
            }
            Predicate::Not(inner) => return self.truth_of(inner, row).not(),
            Predicate::Compare {
                left, op, right, ..
            } if self.symbolic(predicate) => {
                let instance = |operand: &Operand| {
                    let Operand::Fact(path) = operand else {
                        return None;
                    };
                    self.symbols.get(path).or_else(|| {
                        if path.segments().len() != 1 {
                            return None;
                        }
                        match &row.settled.get(path.namespace())?.value {
                            ScenarioValue::Instance { instance } => Some(instance),
                            _ => None,
                        }
                    })
                };
                return match (instance(left), instance(right), op) {
                    (Some(left), Some(right), CompareOp::Eq) => Truth::from_bool(left == right),
                    (Some(left), Some(right), CompareOp::Ne) => Truth::from_bool(left != right),
                    _ => Truth::Unknown,
                };
            }
            _ => {}
        }
        subject_fact::row_truth_with(
            self.ir,
            self.entity,
            &row.settled,
            &row.unwritten,
            Some(&row.state),
            predicate,
            Some((self.command, self.input)),
        )
    }

    fn symbolic(&self, predicate: &Predicate) -> bool {
        predicate
            .fact_paths()
            .iter()
            .any(|path| self.symbols.contains_key(*path))
    }

    /// Discharge each symbolic equality with an actual captured instance. Ordinary conjuncts
    /// remain for the existing typed witness search. Unsupported symbolic expressions refuse
    /// through the caller's usual no-witness diagnostic rather than fabricating an ID.
    fn bindings(
        &self,
        predicate: &Predicate,
        positive: bool,
    ) -> Option<BTreeMap<String, InstanceName>> {
        if !self.symbolic(predicate) {
            return Some(BTreeMap::new());
        }
        match predicate {
            Predicate::All(children) if positive => {
                let mut out = BTreeMap::new();
                for child in children {
                    for (field, instance) in self.bindings(child, true)? {
                        if out
                            .insert(field, instance.clone())
                            .is_some_and(|held| held != instance)
                        {
                            return None;
                        }
                    }
                }
                Some(out)
            }
            Predicate::Not(inner) => self.bindings(inner, !positive),
            Predicate::Compare {
                left: Operand::Fact(left),
                op,
                right: Operand::Fact(right),
                ..
            } if matches!(op, CompareOp::Eq | CompareOp::Ne) => {
                let (instance, field) = self
                    .symbols
                    .get(left)
                    .map(|instance| (instance, right))
                    .or_else(|| self.symbols.get(right).map(|instance| (instance, left)))?;
                if field.segments().len() != 1 {
                    return None;
                }
                let instance = if (*op == CompareOp::Eq) == positive {
                    instance
                } else {
                    self.other.as_ref()?
                };
                Some(
                    [(field.to_string(), instance.clone())]
                        .into_iter()
                        .collect(),
                )
            }
            _ => None,
        }
    }

    /// Erase only discharged symbolic leaves from the steering predicate. This is not the
    /// acceptance test: `truth_of` still evaluates the complete predicate on the resulting row.
    fn steering(&self, predicate: &Predicate) -> Option<Predicate> {
        if !self.symbolic(predicate) {
            return Some(predicate.clone());
        }
        match predicate {
            Predicate::All(children) => Some(Predicate::All(
                children
                    .iter()
                    .filter_map(|child| self.steering(child))
                    .collect(),
            )),
            Predicate::Not(inner) => self
                .steering(inner)
                .map(|inner| Predicate::Not(Box::new(inner))),
            _ => None,
        }
    }

    /// One further row the filter selects, resting in a state `rests` admits, and whose `sets:`
    /// fields the effect would visibly change where `visible` asks for it.
    fn row(
        &self,
        actors: &BTreeMap<QualifiedName, ActorRef>,
        taken: &mut BTreeSet<InstanceName>,
        rests: &dyn Fn(&StateName) -> bool,
        visible: bool,
    ) -> Option<Arrangement> {
        let goal = (self.filter.clone(), self.closed.clone());
        self.row_meeting(actors, taken, &goal, rests, visible)
    }

    /// One further row on which `goal` — a predicate over the filter's own terms, and the same with
    /// the input written in, which steers the search — holds, as [`Self::row`] otherwise.
    fn row_meeting(
        &self,
        actors: &BTreeMap<QualifiedName, ActorRef>,
        taken: &mut BTreeSet<InstanceName>,
        (goal, steer): &(Predicate, Predicate),
        rests: &dyn Fn(&StateName) -> bool,
        visible: bool,
    ) -> Option<Arrangement> {
        // A row arranged by sending the command under test would carry that command's own effect
        // (its set or secondary rows, its subject) into the scenario before it is witnessed, so
        // such an arrangement is never a witness; another path, or none, is taken instead.
        let under_test = CommandRef::new(self.command.name.clone());
        let accept = |row: &Arrangement| {
            self.truth_of(goal, row) == Truth::True
                && rests(&row.state)
                && (!visible || visibly_changed(self.ir, self.sets, self.supplied, &row.settled))
                && !row.steps.iter().any(|step| {
                    matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                        if *command == under_test)
                })
        };
        let distinction = next(self.ir, self.entity, taken);
        if self.symbolic(goal) {
            let bound = self.bindings(goal, true)?;
            let steering = self.steering(steer).unwrap_or(Predicate::Always);
            return arrange_toward_bound(
                self.ir,
                self.entity,
                &steering,
                actors,
                distinction,
                None,
                &bound,
                &accept,
            );
        }
        arrange_toward_filter(self.ir, self.entity, steer, actors, distinction, &accept).or_else(
            || {
                // A filter over the state alone steers no creating input; the plain search over
                // the same filter still offers every witness it knows.
                arrange_toward_filter(
                    self.ir,
                    self.entity,
                    &Predicate::Always,
                    actors,
                    distinction,
                    &accept,
                )
            },
        )
    }
}

/// The rows a set effect is witnessed on: those it must change, and those it must leave.
struct Rows {
    changed: Vec<Arrangement>,
    kept: Vec<Arrangement>,
}

/// [`MATCHING`] rows `selection` selects in a state `movable` admits, one per [`misses`] it does
/// not select, and — where `outside` is given — one it selects resting outside those states. Each
/// row is one the effect's `sets:` would visibly change, so writing a row it must leave is caught.
fn rows(
    selection: &Selection<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    movable: &dyn Fn(&StateName) -> bool,
    outside: Option<Outside<'_>>,
    visible: bool,
) -> Result<Rows, &'static str> {
    let mut changed = Vec::new();
    for _ in 0..MATCHING {
        changed.push(selection.row(actors, taken, movable, visible).ok_or(
            "has no three rows the filter selects that the arranging commands can produce",
        )?);
    }
    let mut kept = Vec::new();
    for miss in misses(&selection.filter)
        .into_iter()
        .zip(misses(&selection.closed))
    {
        kept.push(
            selection
                .row_meeting(actors, taken, &miss, movable, visible)
                .ok_or(
                    "has no row leaving out exactly one conjunct of the filter that the \
                     arranging commands can produce",
                )?,
        );
    }
    if let Some((outside, to)) = outside {
        // Optional: a lifecycle whose other states no arrangement reaches has no row to skip. A
        // row resting where the move arrives would not show a wrong move by its state, so another
        // state is tried first, and that one only where its `sets:` fields would show it.
        let elsewhere = |state: &StateName| outside(state) && Some(state) != to;
        let skipped = selection
            .row(actors, taken, &elsewhere, false)
            .or_else(|| selection.row(actors, taken, outside, true));
        if let Some(skipped) = skipped {
            kept.push(skipped);
        }
    }
    Ok(Rows { changed, kept })
}

/// The field match naming `row` in `view`, with its state and every value `settled` determined
/// that the view publishes at the entity's type.
fn row_fields(
    ir: &EssIr,
    entity: &EntityHandle,
    view: &ResolvedView,
    instance: &InstanceName,
    state: &StateName,
    settled: &BTreeMap<String, Determined>,
) -> BTreeMap<String, ScenarioValue> {
    let mut fields: BTreeMap<String, ScenarioValue> = [(
        ir.entity(entity).identity.name.clone(),
        ScenarioValue::instance(instance.clone()),
    )]
    .into_iter()
    .collect();
    fields.extend(lifecycle_state(ir, entity, view, state));
    shown(view, fields, settled)
}

/// Every row read back after the command: changed rows as the effect leaves them, the others as
/// they were arranged.
#[allow(clippy::too_many_arguments)]
fn read_back(
    ir: &EssIr,
    entity: &EntityHandle,
    views: &[&ResolvedView],
    rows: &Rows,
    to: Option<&StateName>,
    sets: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
    for view in views {
        let name = ViewRef::new(view.name.clone());
        let mut reads = Vec::new();
        for row in &rows.changed {
            let state = to.unwrap_or(&row.state);
            let left = after(ir, sets, supplied, &row.settled);
            let fields = row_fields(ir, entity, view, &row.instance, state, &left);
            require(
                view,
                &name,
                BTreeMap::new(),
                ViewExpectation::Contains { fields },
                &mut reads,
            );
        }
        for row in &rows.kept {
            let fields = row_fields(ir, entity, view, &row.instance, &row.state, &row.settled);
            require(
                view,
                &name,
                BTreeMap::new(),
                ViewExpectation::Contains { fields },
                &mut reads,
            );
        }
        steps.extend(reads);
        source.insert(name.into());
    }
}

/// The invocation of the branch under test, and the answer it must give.
fn invocation(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    supplied: BTreeMap<String, ScenarioValue>,
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
    let start = steps.len();
    let command_ref = CommandRef::new(command.name.clone());
    let branch = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    steps.push(ScenarioStep::ExecuteCommand {
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supplied,
        caller: BTreeMap::new(),
    });
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: branch.clone(),
    });
    models.mark(InvocationPhase::Act, &mut steps[start..]);
    source.insert(command_ref.into());
    source.insert(branch.into());
    if let Some(actor) = actors.get(&command.name) {
        source.insert(actor.clone().into());
    }
}

/// Where a set move's rows must not change: the states outside its `from`, and its arrival state.
type Outside<'a> = (&'a dyn Fn(&StateName) -> bool, Option<&'a StateName>);

/// The rows a set subject is witnessed on, and the state its changed rows arrive in, if it moves.
fn set_rows<'s>(
    selection: &Selection<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    set: &'s ResolvedSetSubject,
) -> Result<(Rows, Option<&'s StateName>), &'static str> {
    let (from, to) = match &set.effect {
        ResolvedEffect::Moves { transition } => (Some(&transition.from), Some(&transition.to)),
        _ => (None, None),
    };
    let movable = |state: &StateName| from.is_none_or(|from| from.contains(state));
    let outside = |state: &StateName| from.is_some_and(|from| !from.contains(state));
    let skipped: Option<Outside<'_>> = from.map(|_| (&outside as &dyn Fn(&StateName) -> bool, to));
    rows(
        selection,
        actors,
        &mut BTreeSet::new(),
        &movable,
        skipped,
        true,
    )
    .or_else(|reason| {
        // A move is observed by the state it leaves; where no row can show its `sets:` changing,
        // the state alone still separates the changed rows from the others.
        if to.is_some() {
            rows(
                selection,
                actors,
                &mut BTreeSet::new(),
                &movable,
                skipped,
                false,
            )
        } else {
            Err(reason)
        }
    })
    .map(|rows| (rows, to))
}

/// What the branch's own answer must be: its error, its events — `{count: changed}` equal to the
/// rows it changed — its typed response, and no other event.
fn answer(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    changed: usize,
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) -> Result<(), RefusalCause> {
    if let Some(error) = &outcome.error {
        steps.push(super::expect_error(
            ir,
            outcome,
            error,
            supplied,
            &BTreeMap::new(),
        ));
    }
    let emitted: Vec<EventRef> = outcome.emits.iter().map(EventRef::from).collect();
    let changed = i64::try_from(changed).unwrap_or(i64::MAX);
    for event in &emitted {
        let mut payload = determined_payload(ir, outcome, event, supplied, &BTreeMap::new());
        for field in outcome
            .payload
            .iter()
            .filter(|payload| EventRef::from(&payload.event) == *event)
            .flat_map(|payload| &payload.fields)
            .filter(|field| field.value == ess_compiler::ir::ResolvedPayloadValue::ChangedCount)
        {
            payload.insert(field.target.clone(), Node::Number(Number::from(changed)));
        }
        steps.push(ScenarioStep::ExpectEvent {
            event: event.clone(),
            payload,
            shape: crate::response::event_shape(ir, event, outcome),
        });
        source.insert(event.clone().into());
    }
    let observations =
        crate::response::Observation::of(ir, command, outcome).map_err(|reason| {
            gap(
                format!("{}.response: {reason}", command.name),
                "command response".into(),
                "typed response observation cannot execute this contract",
            )
        })?;
    steps.extend(
        observations
            .into_iter()
            .map(|response| ScenarioStep::ExpectResponsePayload { response }),
    );
    let absent = not_emitted(ir, &emitted);
    for event in &absent {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(absent.into_iter().map(EssSemanticRef::from));
    Ok(())
}

/// The `instances:` scenario.
fn set_scenario(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    set: &ResolvedSetSubject,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = models.arrangement;
    let entity = &set.entity;
    let at = || format!("{}.instances", outcome.name);
    let moves = matches!(set.effect, ResolvedEffect::Moves { .. });
    let views = observed(ir, entity, at(), (moves, &outcome.sets))?;
    let input = reach(ir, command, outcome, Distinction::PLAIN)?;
    let supplied = supply(command, &input, None, None, &BTreeMap::new());
    let selection = Selection {
        ir,
        command,
        entity,
        filter: set.filter.clone(),
        closed: written_in(&set.filter, &|path| input_value(path, &input)),
        input: &input,
        supplied: &supplied,
        sets: outcome,
        symbols: BTreeMap::new(),
        other: None,
    };
    let (arranged, to) = set_rows(&selection, actors, set)
        .map_err(|reason| gap(at(), entity.to_string(), reason))?;

    let mut steps = Vec::new();
    let mut source: BTreeSet<EssSemanticRef> = BTreeSet::new();
    for row in arranged.changed.iter().chain(&arranged.kept) {
        steps.extend(row.steps.iter().cloned());
        source.extend(row.source.iter().cloned());
    }
    source.insert(EntityRef::from(entity).into());
    models.mark(InvocationPhase::Arrange, &mut steps);
    invocation(
        models,
        command,
        outcome,
        actors,
        supplied.clone(),
        &mut steps,
        &mut source,
    );
    answer(
        ir,
        command,
        outcome,
        &supplied,
        arranged.changed.len(),
        &mut steps,
        &mut source,
    )?;
    read_back(
        ir,
        entity,
        &views,
        &arranged,
        to,
        outcome,
        &supplied,
        &mut steps,
        &mut source,
    );
    let left = left_by(ir, outcome, &supplied, &arranged, to);
    if let Some(none) = matching_none(ir, command, outcome, &set.filter, entity, &left) {
        zero_match(
            models,
            command,
            outcome,
            actors,
            (&views, entity),
            &none,
            left,
            (&mut steps, &mut source),
        )?;
    }
    let text = format!(
        "`{}` {} every `{}` its filter selects and no other row, and reports how many",
        command.name,
        set.effect.verb(),
        ir.entity(entity).name
    );
    Ok(ConformanceScenario::new(
        super::clipped(&text),
        steps,
        source,
    ))
}

/// One `affects:` entry's rows, read back after the command.
struct Entry<'o> {
    affect: &'o ResolvedAffect,
    sets: ResolvedOutcome,
    views: Vec<&'o ResolvedView>,
    rows: Rows,
    /// The entry's filter with the subject's values written in, and the same with the input's.
    filter: Predicate,
    closed: Predicate,
}

/// What one arranged row holds after the command: its state and its determined fields.
type Left = (StateName, BTreeMap<String, Determined>);

/// What every arranged row of every `affects:` entry holds after the command: what each entry, in
/// the order written, does to it (beyond10x/ess#229). Each entry over the row's entity whose filter
/// selects the row as arranged writes its `sets:` and — where it moves and the row rests in the
/// move's `from` states — takes its move; the others leave it. One row two entries select thus
/// gets one expectation, the one the interpreter answers. Per entry, its changed rows then its
/// kept rows, as [`Rows`] holds them; or a refusal where an entry cannot tell whether it selects a
/// row another entry arranged.
#[allow(clippy::too_many_arguments)] // The selection context each entry is re-read under.
fn combined(
    ir: &EssIr,
    command: &ResolvedCommand,
    entries: &[Entry<'_>],
    input: &BTreeMap<String, Node>,
    supplied: &BTreeMap<String, ScenarioValue>,
    symbols: &BTreeMap<FactPath, InstanceName>,
    other: Option<&InstanceName>,
    at: &dyn Fn(usize) -> String,
) -> Result<Vec<Vec<Left>>, RefusalCause> {
    let selections: Vec<Selection<'_>> = entries
        .iter()
        .map(|entry| Selection {
            ir,
            command,
            entity: &entry.affect.entity,
            filter: entry.filter.clone(),
            closed: entry.closed.clone(),
            input,
            supplied,
            sets: &entry.sets,
            symbols: symbols.clone(),
            other: other.cloned(),
        })
        .collect();
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut left = Vec::new();
        for row in entry.rows.changed.iter().chain(&entry.rows.kept) {
            let mut state = row.state.clone();
            let mut settled = row.settled.clone();
            for (index, (other, selection)) in entries.iter().zip(&selections).enumerate() {
                if other.affect.entity != entry.affect.entity {
                    continue;
                }
                match selection.truth_of(&selection.filter, row) {
                    Truth::True => {}
                    Truth::False => continue,
                    Truth::Unknown => {
                        return Err(gap(
                            at(index),
                            other.affect.entity.to_string(),
                            "cannot tell whether this entry selects a row another entry over the \
                             same entity arranges, so what the rows hold after the command is \
                             unknown",
                        ))
                    }
                }
                if let Some(transition) = &other.affect.moves {
                    if !transition.from.contains(&row.state) {
                        continue;
                    }
                    state = transition.to.clone();
                }
                settled = after(ir, &other.sets, supplied, &settled);
            }
            left.push((state, settled));
        }
        out.push(left);
    }
    Ok(out)
}

/// Every row of one entry read back from each of its views as [`combined`] leaves it: the same
/// expectations [`read_back`] writes for an entry no other entry touches.
fn read_back_left(
    ir: &EssIr,
    entry: &Entry<'_>,
    left: &[Left],
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
    let entity = &entry.affect.entity;
    for view in &entry.views {
        let name = ViewRef::new(view.name.clone());
        let mut reads = Vec::new();
        for (row, (state, settled)) in entry.rows.changed.iter().chain(&entry.rows.kept).zip(left) {
            let fields = row_fields(ir, entity, view, &row.instance, state, settled);
            require(
                view,
                &name,
                BTreeMap::new(),
                ViewExpectation::Contains { fields },
                &mut reads,
            );
        }
        steps.extend(reads);
        source.insert(name.into());
    }
}

/// The rows one `affects:` entry is witnessed on (ess/22, beyond10x/ess#229). An entry that only
/// sets fields is witnessed as it always was; a moving one as an `instances:` move is ([`set_rows`]):
/// its changed rows rest in a `from` state other than the move's arrival wherever the arranging
/// commands reach one, and one row the filter selects rests outside them, which the move skips.
/// Where only the arrival state itself is reached, the move is seen through what `sets:` writes,
/// and an entry writing nothing is refused by name.
fn affect_rows(
    selection: &Selection<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    affect: &ResolvedAffect,
) -> Result<Rows, &'static str> {
    let Some(transition) = &affect.moves else {
        return rows(selection, actors, taken, &|_| true, None, true);
    };
    let (from, to) = (&transition.from, &transition.to);
    // A row resting where the move arrives reads the same whether or not the move was taken, so
    // the changed rows rest in a `from` state other than `to` wherever one is arranged.
    let leaves = |state: &StateName| from.contains(state) && state != to;
    let movable = |state: &StateName| from.contains(state);
    let outside = |state: &StateName| !from.contains(state);
    let skipped: Option<Outside<'_>> = Some((&outside, Some(to)));
    let held = taken.clone();
    let mut attempt = |rests: &dyn Fn(&StateName) -> bool, visible: bool| {
        taken.clone_from(&held);
        rows(selection, actors, taken, rests, skipped, visible)
    };
    attempt(&leaves, true)
        // A move is observed by the state it leaves; where no row can show its `sets:` changing,
        // the state alone still separates the changed rows from the others.
        .or_else(|_| attempt(&leaves, false))
        .or_else(|reason| {
            // Only `to` itself is arranged among the `from` states: the move is seen only through
            // what `sets:` writes, and with nothing written it is not seen at all.
            if affect.sets.is_empty() {
                Err(
                    "is moved by its entry only from the state the move arrives in, as far as \
                     the arranging commands reach, and the entry writes no field, so a target \
                     that skips the move reads the same",
                )
            } else {
                attempt(&movable, true).map_err(|_| reason)
            }
        })
}

/// Each `affects:` filter with the subject's values written in, or a refusal naming a subject field
/// the arrangement leaves undetermined.
fn subject_filters<'o>(
    outcome: &'o ResolvedOutcome,
    subject: &EntityHandle,
    before: &BTreeMap<String, Determined>,
    symbols: &BTreeMap<FactPath, InstanceName>,
) -> Result<Vec<(&'o ResolvedAffect, Predicate)>, RefusalCause> {
    let mut out = Vec::new();
    for (index, affect) in outcome.affects.iter().enumerate() {
        let filter = written_in(&affect.filter, &|path| subject_value(path, before));
        if let Some(path) = filter.fact_paths().into_iter().find(|path| {
            path.namespace() == ess_domain::command::set_effects::SUBJECT_NAMESPACE
                && !symbols.contains_key(*path)
        }) {
            return Err(gap(
                format!("{}.affects[{index}]: {path}", outcome.name),
                subject.to_string(),
                "reads a subject field the arrangement does not determine, so which rows the \
                 filter selects is unknown",
            ));
        }
        out.push((affect, filter));
    }
    Ok(out)
}

/// The reads requiring the subject as the branch leaves it, from every view observing its entity.
fn subject_reads(
    ir: &EssIr,
    subject: &EntityHandle,
    instance: &InstanceName,
    state: Option<&StateName>,
    left: &BTreeMap<String, Determined>,
    source: &mut BTreeSet<EssSemanticRef>,
) -> Vec<ScenarioStep> {
    let mut reads = Vec::new();
    let Some(state) = state else {
        return reads;
    };
    for view in observing(ir, subject) {
        let name = ViewRef::new(view.name.clone());
        let fields = row_fields(ir, subject, view, instance, state, left);
        require(
            view,
            &name,
            BTreeMap::new(),
            ViewExpectation::Contains { fields },
            &mut reads,
        );
        source.insert(name.into());
    }
    reads
}

struct AffectInputs {
    symbols: BTreeMap<FactPath, InstanceName>,
    other: Option<Arrangement>,
    literals: BTreeMap<String, Node>,
}

/// Bind identity operands and arrange a real alternative identity. These names are references
/// to captures, not surrogate literal IDs; ordinary inputs retain the existing typed search.
fn affect_inputs(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    instance: &InstanceName,
    supplied: &BTreeMap<String, ScenarioValue>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
) -> Result<AffectInputs, RefusalCause> {
    let subject = &outcome
        .subject
        .as_ref()
        .expect("affects requires a subject")
        .entity;
    let identity = &ir.entity(subject).identity.name;
    let mut symbols = BTreeMap::new();
    symbols.insert(
        FactPath::from_segments(["subject".to_owned(), identity.clone()]),
        instance.clone(),
    );
    for (field, value) in supplied {
        if matches!(value, ScenarioValue::Instance { instance: held } if held == instance) {
            symbols.insert(
                FactPath::from_segments(["input".to_owned(), field.clone()]),
                instance.clone(),
            );
        }
    }
    symbols.retain(|path, _| {
        outcome
            .affects
            .iter()
            .any(|affect| affect.filter.fact_paths().contains(&path))
    });
    let other = if symbols.is_empty() {
        None
    } else {
        let distinction = next(ir, subject, taken);
        let initial = &ir.entity(subject).lifecycle.initial;
        Some(super::arrange(ir, subject, initial, actors, distinction, &[])
            .map_err(|_| gap(format!("{}.affects", outcome.name), subject.to_string(), "cannot arrange a second subject to witness the identity filter's excluded rows"))?)
    };
    let literals = supplied
        .iter()
        .filter_map(|(field, value)| {
            value
                .as_literal()
                .map(|value| (field.clone(), value.clone()))
        })
        .collect();
    Ok(AffectInputs {
        symbols,
        other,
        literals,
    })
}

/// The `affects:` segment appended to the branch's own scenario.
#[allow(clippy::too_many_lines)] // One complete set-effect witness, including invocation authority.
fn affects_segment(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let at = |index: usize| format!("{}.affects[{index}]", outcome.name);
    let subject = outcome.subject.as_ref().ok_or_else(|| {
        gap(
            at(0),
            command.name.to_string(),
            "is declared beside no subject, so `subject.` reads nothing",
        )
    })?;
    let distinction = next(ir, &subject.entity, taken);
    let setup = prepare_in(ir, outcome, actors, None, distinction)?;
    let instance = setup.instance.clone().ok_or_else(|| {
        gap(
            at(0),
            subject.entity.to_string(),
            "has no subject row an arrangement names",
        )
    })?;
    let input = match (&setup.before, has_subject_guards(command)) {
        (Some(held), true) => reach_in_state(ir, command, outcome, held, Distinction::PLAIN)?,
        _ => reach(ir, command, outcome, Distinction::PLAIN)?,
    };
    let supplied = supply(
        command,
        &input,
        Some(subject),
        Some(&instance),
        &setup.bound,
    );

    let mut steps = setup.steps.clone();
    let mut source = setup.source.clone();
    let left = after(ir, outcome, &supplied, &setup.settled);
    let reads = subject_reads(
        ir,
        &subject.entity,
        &instance,
        setup.after.as_ref(),
        &left,
        &mut source,
    );
    let mut entries = Vec::new();
    let operands = affect_inputs(ir, outcome, &instance, &supplied, actors, taken)?;
    let other = operands.other.as_ref().map(|row| row.instance.clone());
    if let Some(other) = operands.other {
        steps.extend(other.steps);
        source.extend(other.source);
    }
    let filters = subject_filters(outcome, &subject.entity, &setup.settled, &operands.symbols)?;
    for (index, (affect, filter)) in filters.into_iter().enumerate() {
        let sets = with_sets(outcome, affect);
        let views = observed(
            ir,
            &affect.entity,
            at(index),
            (affect.moves.is_some(), &affect.sets),
        )?;
        let selection = Selection {
            ir,
            command,
            entity: &affect.entity,
            closed: written_in(&filter, &|path| input_value(path, &operands.literals)),
            filter,
            input: &input,
            supplied: &supplied,
            sets: &sets,
            symbols: operands.symbols.clone(),
            other: other.clone(),
        };
        let rows = affect_rows(&selection, actors, taken, affect)
            .map_err(|reason| gap(at(index), affect.entity.to_string(), reason))?;
        let (filter, closed) = (selection.filter.clone(), selection.closed.clone());
        for row in rows.changed.iter().chain(&rows.kept) {
            steps.extend(row.steps.iter().cloned());
            source.extend(row.source.iter().cloned());
        }
        source.insert(EntityRef::from(&affect.entity).into());
        entries.push(Entry {
            affect,
            sets,
            views,
            rows,
            filter,
            closed,
        });
    }
    models.mark(InvocationPhase::Arrange, &mut steps);
    invocation(
        models,
        command,
        outcome,
        actors,
        supplied.clone(),
        &mut steps,
        &mut source,
    );
    steps.extend(reads);
    let left = combined(
        ir,
        command,
        &entries,
        &input,
        &supplied,
        &operands.symbols,
        other.as_ref(),
        &at,
    )?;
    for (entry, left) in entries.iter().zip(&left) {
        read_back_left(ir, entry, left, &mut steps, &mut source);
    }
    Ok((steps, source))
}

/// The branch with an `affects:` entry's `sets:` in place of its own: what [`settled`] reads.
fn with_sets(outcome: &ResolvedOutcome, affect: &ResolvedAffect) -> ResolvedOutcome {
    ResolvedOutcome {
        sets: affect.sets.clone(),
        ..outcome.clone()
    }
}

/// The predicates a row the filter leaves out is arranged to meet: for a conjunction, one per
/// conjunct — that conjunct false and every other true — so a target dropping any one conjunct
/// changes a row it must leave; for anything else, the whole filter false.
fn misses(filter: &Predicate) -> Vec<Predicate> {
    let not = |predicate: &Predicate| Predicate::Not(Box::new(predicate.clone()));
    match filter {
        Predicate::All(conjuncts) if conjuncts.len() > 1 => (0..conjuncts.len())
            .map(|missed| {
                Predicate::All(
                    conjuncts
                        .iter()
                        .enumerate()
                        .map(|(at, conjunct)| {
                            if at == missed {
                                not(conjunct)
                            } else {
                                conjunct.clone()
                            }
                        })
                        .collect(),
                )
            })
            .collect(),
        other => vec![not(other)],
    }
}

/// Every arranged row as the branch under test leaves it: the changed ones in their new state with
/// what `sets:` wrote, the others as they were.
fn left_by(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    rows: &Rows,
    to: Option<&StateName>,
) -> Vec<Arrangement> {
    let changed = rows.changed.iter().map(|row| Arrangement {
        state: to.unwrap_or(&row.state).clone(),
        settled: after(ir, outcome, supplied, &row.settled),
        unwritten: super::still_unwritten(&row.unwritten, outcome),
        ..row.clone()
    });
    changed.chain(rows.kept.iter().cloned()).collect()
}

/// An input reaching the branch under which the filter selects none of `rows`: the zero-match
/// call, or `None` where no witness the search offers leaves every row out.
fn matching_none(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    filter: &Predicate,
    entity: &EntityHandle,
    rows: &[Arrangement],
) -> Option<BTreeMap<String, Node>> {
    (1..=crate::witness::MAX_CANDIDATES)
        .filter_map(|further| reach(ir, command, outcome, Distinction::further(further)).ok())
        .find(|input| {
            rows.iter().all(|row| {
                subject_fact::row_truth_with(
                    ir,
                    entity,
                    &row.settled,
                    &row.unwritten,
                    Some(&row.state),
                    filter,
                    Some((command, input)),
                ) == Truth::False
            })
        })
}

/// The zero-match segment: the command again, under an input the filter selects no row by. It is
/// accepted, `{count: changed}` is 0, and every row reads as the first call left it.
#[allow(clippy::too_many_arguments)]
fn zero_match(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (views, entity): (&[&ResolvedView], &EntityHandle),
    input: &BTreeMap<String, Node>,
    left: Vec<Arrangement>,
    (steps, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<(), RefusalCause> {
    let ir = models.arrangement;
    let supplied = supply(command, input, None, None, &BTreeMap::new());
    invocation(
        models,
        command,
        outcome,
        actors,
        supplied.clone(),
        steps,
        source,
    );
    answer(ir, command, outcome, &supplied, 0, steps, source)?;
    let unchanged = Rows {
        changed: Vec::new(),
        kept: left,
    };
    read_back(
        ir, entity, views, &unchanged, None, outcome, &supplied, steps, source,
    );
    Ok(())
}
