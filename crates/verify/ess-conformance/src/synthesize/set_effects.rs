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
//! * **Deletion** (ess/23, beyond10x/ess#452): a `deletes:` set subject, or a deleting `affects:`
//!   entry, is witnessed on the same rows, and each row it removes is read absent by its identity
//!   ([`require_absent`], `deletes:`'s `expect_subject_absent`) from every view that holds every
//!   row ([`observed_removal`]); the rows it leaves are read as arranged.
//!
//! Every row is read from a view that publishes the entity's identity, unfiltered, whole and
//! immediate, and that publishes the state a set move leaves and every field the effect writes
//! ([`observed`]); where no such view exists the scenario is refused by name rather than filed
//! without the observation. Every step used is an existing one, so a suite carrying either construct keeps
//! the format its other steps select.
//!
//! The claims are about every stored row: the count and the unchanged rows hold on a target no
//! other scenario writes to at the same time, which is what §8 requires of a shared one.

use ess_compiler::ir::{ResolvedAffect, ResolvedSetSubject, ResolvedTypeRef};
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
        refused_writes_none(models.arrangement, command, suite);
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

/// [`observing`], for a set effect that removes its rows (ess/23, beyond10x/ess#452): a removed row
/// is read absent by its identity and a kept one as arranged, so any view holding every row of the
/// entity with its identity observes both; or a refusal where none does.
fn observed_removal<'i>(
    ir: &'i EssIr,
    entity: &EntityHandle,
    at: String,
) -> Result<Vec<&'i ResolvedView>, RefusalCause> {
    let views = observing(ir, entity);
    if views.is_empty() {
        return Err(gap(
            at,
            entity.to_string(),
            "is removed by a set effect and published by no immediate, unfiltered, \
             unparameterised, unpaged view carrying its identity, so a removed row and a kept \
             one cannot be told apart when read back",
        ));
    }
    Ok(views)
}

/// The read requiring `instance` of `entity` absent from `view` after the command: `deletes:`'s
/// absence check (`expect_subject_absent`, `ess-conformance/22`), after one query of the view.
fn require_absent(
    ir: &EssIr,
    entity: &EntityHandle,
    name: &ViewRef,
    instance: &InstanceName,
    reads: &mut Vec<ScenarioStep>,
) {
    if !reads
        .iter()
        .any(|step| matches!(step, ScenarioStep::QueryView { view, .. } if view == name))
    {
        reads.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
    }
    reads.push(ScenarioStep::ExpectSubjectAbsent {
        view: name.clone(),
        subject: [(
            ir.entity(entity).identity.name.clone(),
            ScenarioValue::instance(instance.clone()),
        )]
        .into_iter()
        .collect(),
    });
}

fn gap(path: String, type_ref: String, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path,
        type_ref,
        reason,
    })
}

/// The value a path under `input.` names in the input a scenario sends.
pub(super) fn input_value(path: &FactPath, input: &BTreeMap<String, Node>) -> Option<FactValue> {
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
pub(super) fn subject_value(
    path: &FactPath,
    before: &BTreeMap<String, Determined>,
) -> Option<FactValue> {
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
pub(super) fn written_in(
    filter: &Predicate,
    read: &dyn Fn(&FactPath) -> Option<FactValue>,
) -> Predicate {
    let operand = |it: &Operand| match it {
        Operand::Fact(path) => read(path).map_or_else(|| it.clone(), Operand::Literal),
        Operand::Offset(offset) => read(&offset.base)
            .and_then(|base| offset.value_at(&base))
            .map_or_else(|| it.clone(), Operand::Literal),
        Operand::Derived(derived) => derived
            .value_with(read)
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
        // An arrangement is the row it arranges and nothing else: each act it sends is folded
        // into that row's expected state. An act taking a branch with a set effect (`instances:`
        // or `affects:`) also changes rows the arrangement does not account for, which the
        // scenario then reads back as arranged, so such an arrangement is never a witness and
        // another path, or none, is taken. Sending the command under test is otherwise allowed:
        // a branch of it that changes only the row it names is accounted for like any other act.
        let accept = |row: &Arrangement| {
            self.truth_of(goal, row) == Truth::True
                && rests(&row.state)
                && (!visible || visibly_changed(self.ir, self.sets, self.supplied, &row.settled))
                && !reaches_other_rows(self.ir, &row.steps)
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

/// Every row read back after the command: changed rows as the effect leaves them — absent where
/// it removes them (ess/23, beyond10x/ess#452) — the others as they were arranged.
#[allow(clippy::too_many_arguments)]
fn read_back(
    ir: &EssIr,
    entity: &EntityHandle,
    views: &[&ResolvedView],
    rows: &Rows,
    (to, removes): (Option<&StateName>, bool),
    sets: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
    for view in views {
        let name = ViewRef::new(view.name.clone());
        let mut reads = Vec::new();
        for row in rows.changed.iter().filter(|_| removes) {
            require_absent(ir, entity, &name, &row.instance, &mut reads);
        }
        for row in rows.changed.iter().filter(|_| !removes) {
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
    let removes = set.effect == ResolvedEffect::Deletes;
    let views = if removes {
        observed_removal(ir, entity, at())?
    } else {
        observed(ir, entity, at(), (moves, &outcome.sets))?
    };
    let input = reach(ir, command, outcome, Distinction::PLAIN)?;
    let supplied = supply(ir, command, &input, None, None, &BTreeMap::new());
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
        (to, removes),
        outcome,
        &supplied,
        &mut steps,
        &mut source,
    );
    let left = left_by(ir, outcome, &supplied, &arranged, (to, removes));
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
    // A deleting entry (ess/23, beyond10x/ess#452) stands alone over its entity, so each row it
    // changes is one it removes, read absent; the rows it leaves are read as every entry leaves them.
    let removed = if entry.affect.deletes {
        entry.rows.changed.len()
    } else {
        0
    };
    for view in &entry.views {
        let name = ViewRef::new(view.name.clone());
        let mut reads = Vec::new();
        for row in &entry.rows.changed[..removed] {
            require_absent(ir, entity, &name, &row.instance, &mut reads);
        }
        for (row, (state, settled)) in entry
            .rows
            .changed
            .iter()
            .chain(&entry.rows.kept)
            .zip(left)
            .skip(removed)
        {
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

/// Each `affects:` filter with the subject's values written in, beside the entry's position, or a
/// refusal naming a subject field the arrangement leaves undetermined.
fn subject_filters<'o>(
    outcome: &'o ResolvedOutcome,
    subject: &EntityHandle,
    before: &BTreeMap<String, Determined>,
    symbols: &BTreeMap<FactPath, InstanceName>,
) -> Result<Vec<(usize, &'o ResolvedAffect, Predicate)>, RefusalCause> {
    let mut out = Vec::new();
    for (index, affect) in outcome.affects.iter().enumerate() {
        // An `each:` entry (ess/23, beyond10x/ess#459) selects no row; [`each_segment`] has it.
        let Some(filter) = &affect.filter else {
            continue;
        };
        let filter = written_in(filter, &|path| subject_value(path, before));
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
        out.push((index, affect, filter));
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
        outcome.affects.iter().any(|affect| {
            affect
                .filter
                .as_ref()
                .is_some_and(|filter| filter.fact_paths().contains(&path))
        })
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
    let mut input = match (&setup.before, has_subject_guards(command)) {
        (Some(held), true) => reach_in_state(ir, command, outcome, held, Distinction::PLAIN)?,
        _ => reach(ir, command, outcome, Distinction::PLAIN)?,
    };
    // One row per element of an input list (ess/23, beyond10x/ess#459): the command under test
    // sends the elements naming the held row and a new one, after a first call put the held row
    // and a decoy in place.
    let each = each_lists(ir, command, outcome, setup.before.as_ref(), &input)?;
    if let Some(each) = &each {
        input.insert(each.each.list.clone(), each.second.clone());
    }
    let supplied = supply(
        ir,
        command,
        &input,
        Some(subject),
        Some(&instance),
        &setup.bound,
    );

    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    let first = match &each {
        Some(each) => Some(first_call(
            models,
            (command, outcome),
            actors,
            taken,
            each,
            &input,
            (&mut steps, &mut source),
        )?),
        None => None,
    };
    steps.extend(setup.steps.iter().cloned());
    source.extend(setup.source.iter().cloned());
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
    for (index, affect, filter) in filters {
        let sets = with_sets(outcome, affect);
        let views = if affect.deletes {
            observed_removal(ir, &affect.entity, at(index))?
        } else {
            observed(
                ir,
                &affect.entity,
                at(index),
                (affect.moves.is_some(), &affect.sets),
            )?
        };
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
    if let (Some(each), Some(first)) = (&each, &first) {
        each_reads(
            ir,
            outcome,
            each,
            (first, &supplied),
            &mut steps,
            &mut source,
        )?;
    }
    Ok((steps, source))
}

/// The one `each:` entry of a branch (ess/23, beyond10x/ess#459) and the two lists its scenario
/// sends: `first`, the held row's element and a decoy's, and `second`, an element naming the held
/// row with other values and one naming an identity no row holds.
struct EachLists<'o> {
    index: usize,
    affect: &'o ResolvedAffect,
    each: &'o ess_compiler::ir::ResolvedEach,
    first: Node,
    second: Node,
    /// The members of the decoy's element in `first`, of the held row's in `second`, and of the new
    /// row's.
    decoy: BTreeMap<String, Node>,
    again: BTreeMap<String, Node>,
    new: BTreeMap<String, Node>,
}

/// How many sets of four distinctions [`each_lists`] tries before refusing.
const EACH_TRIES: usize = 8;

/// The one `each:` entry of a branch and its position, or `None` for a branch with none; or a
/// refusal where the branch has more than one, or where another entry writes the same entity.
fn one_each(
    outcome: &ResolvedOutcome,
) -> Result<Option<(usize, &ResolvedAffect, &ess_compiler::ir::ResolvedEach)>, RefusalCause> {
    let at = |index: usize| format!("{}.affects[{index}]", outcome.name);
    let mut found = outcome
        .affects
        .iter()
        .enumerate()
        .filter_map(|(index, affect)| affect.each.as_ref().map(|each| (index, affect, each)));
    let Some((index, affect, each)) = found.next() else {
        return Ok(None);
    };
    if found.next().is_some() {
        return Err(gap(
            at(index),
            affect.entity.to_string(),
            "is written by more than one `each:` entry of the branch, and this cut witnesses one",
        ));
    }
    if outcome
        .affects
        .iter()
        .enumerate()
        .any(|(other, entry)| other != index && entry.entity == affect.entity)
    {
        return Err(gap(
            at(index),
            affect.entity.to_string(),
            "is written by an `each:` entry and another entry of the branch, so which rows each \
             leaves as what is not decided in this cut",
        ));
    }
    Ok(Some((index, affect, each)))
}

/// The lists an `each:` entry's scenario sends, or `None` for a branch with no `each:` entry; or a
/// refusal where the branch has more than one, where another entry writes the same entity, where
/// the list is not one of structs, or where no four elements the witness offers keep the three
/// identities apart, change every member the entry reads on the held row, and reach the branch.
fn each_lists<'o>(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &'o ResolvedOutcome,
    held: Option<&StateName>,
    input: &BTreeMap<String, Node>,
) -> Result<Option<EachLists<'o>>, RefusalCause> {
    let at = |index: usize| format!("{}.affects[{index}]", outcome.name);
    let Some((index, affect, each)) = one_each(outcome)? else {
        return Ok(None);
    };
    let members = command
        .input
        .iter()
        .find(|field| field.name == each.list)
        .and_then(|field| match &field.type_ref {
            ResolvedTypeRef::List { of } => of.declared(),
            _ => None,
        })
        .and_then(|element| match &ir.named_type(element).body {
            ess_compiler::ir::ResolvedBody::Struct { fields, .. } => Some(fields),
            _ => None,
        })
        .ok_or_else(|| {
            gap(
                at(index),
                each.list.clone(),
                "is not a list of structs the witness can build elements of",
            )
        })?;
    let identity = |element: &BTreeMap<String, Node>| element.get(&each.member).cloned();
    for attempt in 0..EACH_TRIES {
        let element = |nth: usize| {
            crate::witness::fields(ir, members, Distinction::further(11 + 4 * attempt + nth)).ok()
        };
        // The second call's element for the held row sits at the distinction next to the first's:
        // adjacent distinctions differ for every scalar kind, a `Boolean` and an enum of an even
        // number of variants included, where two of one parity would not.
        let (Some(held_row), Some(mut again), Some(decoy), Some(new)) =
            (element(0), element(1), element(2), element(3))
        else {
            continue;
        };
        let (Some(held_id), Some(decoy_id), Some(new_id)) =
            (identity(&held_row), identity(&decoy), identity(&new))
        else {
            continue;
        };
        if held_id == decoy_id || held_id == new_id || decoy_id == new_id {
            continue;
        }
        again.insert(each.member.clone(), held_id);
        // The held row shows the second call on every field the element writes.
        if each
            .reads
            .iter()
            .any(|read| held_row.get(&read.member) == again.get(&read.member))
        {
            continue;
        }
        let list = |items: [&BTreeMap<String, Node>; 2]| {
            Node::Seq(items.into_iter().cloned().map(Node::Map).collect())
        };
        let (first, second) = (list([&held_row, &decoy]), list([&again, &new]));
        let reaches = |list: &Node| {
            let mut sent = input.clone();
            sent.insert(each.list.clone(), list.clone());
            super::selects_branch(ir, command, outcome, held, &sent).is_ok_and(|reached| reached)
        };
        if !(reaches(&first) && reaches(&second)) {
            continue;
        }
        return Ok(Some(EachLists {
            index,
            affect,
            each,
            first,
            second,
            decoy,
            again,
            new,
        }));
    }
    Err(gap(
        at(index),
        each.list.clone(),
        "offers no four elements that keep three identities apart, change every member the \
         entry reads on a held row, and reach the branch",
    ))
}

/// The first call of an `each:` segment (ess/23, beyond10x/ess#459): a second subject is
/// arranged, and the command is sent for it with the held row's element and the decoy's, which
/// puts both rows in place without any other command creating the entity. Returns what the call
/// supplied, which the decoy is read back under.
fn first_call(
    models: &InvocationModels<'_>,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    each: &EachLists<'_>,
    input: &BTreeMap<String, Node>,
    (steps, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<First, RefusalCause> {
    let ir = models.arrangement;
    let subject = outcome
        .subject
        .as_ref()
        .expect("affects_segment checked the subject");
    let distinction = next(ir, &subject.entity, taken);
    let setup = prepare_in(ir, outcome, actors, None, distinction)?;
    let instance = setup.instance.clone().ok_or_else(|| {
        gap(
            format!("{}.affects[{}]", outcome.name, each.index),
            subject.entity.to_string(),
            "has no second subject row an arrangement names, to put the held row in place",
        )
    })?;
    let mut sent = input.clone();
    sent.insert(each.each.list.clone(), each.first.clone());
    let supplied = supply(
        ir,
        command,
        &sent,
        Some(subject),
        Some(&instance),
        &setup.bound,
    );
    steps.extend(setup.steps);
    source.extend(setup.source);
    send((command, outcome), actors, supplied.clone(), steps, source);
    let carried = each
        .again
        .get(&each.each.member)
        .and_then(|held| carry(ir, command, each, held, actors))
        .map(|carry| {
            steps.extend(carry.steps);
            source.extend(carry.source);
            carry.kept
        })
        .unwrap_or_default();
    empty_call(
        models,
        (command, outcome),
        actors,
        taken,
        each,
        input,
        (steps, source),
    )?;
    Ok(First { supplied, carried })
}

/// What the arrangement of an `each:` segment leaves for the reads: what the first call supplied,
/// which the decoy is read under, and the fields another command wrote on the held row that the
/// entry does not write, which an update carries (beyond10x/ess#459).
struct First {
    supplied: BTreeMap<String, ScenarioValue>,
    carried: BTreeMap<String, Determined>,
}

/// The command sent, and the branch required of it.
fn send(
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    supplied: BTreeMap<String, ScenarioValue>,
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
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
    source.insert(command_ref.into());
    source.insert(branch.into());
}

/// A call of another command putting a field the `each:` entry does not write on the held row
/// (beyond10x/ess#459): "updated if held" carries it, so a target replacing the row loses it. The
/// first accepting `updates:` branch over the entity, naming its row by a supplied identity and
/// changing no other row, that writes such a field with a value the scenario decides and that the
/// held identity reaches; `None` where there is none, and then no field the entry leaves can tell
/// a replacement from an update.
fn carry(
    ir: &EssIr,
    under_test: &ResolvedCommand,
    each: &EachLists<'_>,
    held: &Node,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Carry> {
    let entity = &each.affect.entity;
    let initial = &ir.entity(entity).lifecycle.initial;
    let written: BTreeSet<&str> = each
        .affect
        .sets
        .iter()
        .map(|set| set.target.as_str())
        .chain(each.each.reads.iter().map(|read| read.target.as_str()))
        .collect();
    for command in ir.commands().values() {
        if command.name == under_test.name {
            continue;
        }
        for outcome in &command.outcomes {
            let Some(subject) = &outcome.subject else {
                continue;
            };
            let ess_compiler::ir::ResolvedInstance::Supplied { field } = &subject.instance else {
                continue;
            };
            if outcome.error.is_some()
                || outcome.instances.is_some()
                || !outcome.affects.is_empty()
                || subject.entity != *entity
                || subject.effect != ResolvedEffect::Updates
            {
                continue;
            }
            let Ok(mut input) = reach(ir, command, outcome, Distinction::PLAIN) else {
                continue;
            };
            input.insert(field.name.clone(), held.clone());
            if !super::selects_branch(ir, command, outcome, Some(initial), &input)
                .is_ok_and(|reached| reached)
            {
                continue;
            }
            let supplied = supply(ir, command, &input, None, None, &BTreeMap::new());
            let kept: BTreeMap<String, Determined> =
                settled(ir, outcome, &supplied, &BTreeMap::new())
                    .into_iter()
                    .filter(|(target, determined)| {
                        !written.contains(target.as_str())
                            && determined.value.as_literal().is_some()
                    })
                    .collect();
            if kept.is_empty() {
                continue;
            }
            let (mut steps, mut source) = (Vec::new(), BTreeSet::new());
            send(
                (command, outcome),
                actors,
                supplied,
                &mut steps,
                &mut source,
            );
            return Some(Carry {
                steps,
                source,
                kept,
            });
        }
    }
    None
}

/// The call [`carry`] found, and the fields it leaves on the held row.
struct Carry {
    steps: Vec<ScenarioStep>,
    source: BTreeSet<EssSemanticRef>,
    kept: BTreeMap<String, Determined>,
}

/// The command sent with an empty list, for a further subject, requiring the accepting branch
/// (beyond10x/ess#459): an empty list is an accepted answer and writes nothing, which the reads
/// after the command under test then hold of every element row. Left out where the branch's guard
/// does not admit an empty list.
fn empty_call(
    models: &InvocationModels<'_>,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    each: &EachLists<'_>,
    input: &BTreeMap<String, Node>,
    (steps, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<(), RefusalCause> {
    let ir = models.arrangement;
    let subject = outcome
        .subject
        .as_ref()
        .expect("affects_segment checked the subject");
    let mut sent = input.clone();
    sent.insert(each.each.list.clone(), Node::Seq(Vec::new()));
    let distinction = next(ir, &subject.entity, taken);
    let setup = prepare_in(ir, outcome, actors, None, distinction)?;
    let Some(instance) = setup.instance.clone() else {
        return Ok(());
    };
    if !super::selects_branch(ir, command, outcome, setup.before.as_ref(), &sent)
        .is_ok_and(|reached| reached)
    {
        return Ok(());
    }
    let supplied = supply(
        ir,
        command,
        &sent,
        Some(subject),
        Some(&instance),
        &setup.bound,
    );
    steps.extend(setup.steps);
    source.extend(setup.source);
    send((command, outcome), actors, supplied, steps, source);
    Ok(())
}

/// What one element leaves in the row it names: the entry's `sets:` under `supplied`, and every
/// member the entry reads.
fn element_row(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    each: &EachLists<'_>,
    supplied: &BTreeMap<String, ScenarioValue>,
    element: &BTreeMap<String, Node>,
) -> BTreeMap<String, Determined> {
    let mut row = settled(
        ir,
        &with_sets(outcome, each.affect),
        supplied,
        &BTreeMap::new(),
    );
    for read in &each.each.reads {
        if let Some(value) = element.get(&read.member) {
            row.insert(
                read.target.clone(),
                Determined {
                    value: ScenarioValue::literal(value.clone()),
                    type_ref: read.target_type.clone(),
                },
            );
        }
    }
    row
}

/// The reads of an `each:` segment after the command (ess/23, beyond10x/ess#459), from every view
/// holding every row of the entity and publishing every field the entry writes: the held row with
/// the second call's element and every field [`carry`] put on it, the new row with its own, both in `initial`, and the decoy as the
/// first call left it; then a snapshot by each element's identity of the second call, which selects
/// exactly one row. Or a refusal where no view publishes them.
fn each_reads(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    each: &EachLists<'_>,
    (first, second): (&First, &BTreeMap<String, ScenarioValue>),
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) -> Result<(), RefusalCause> {
    let entity = &each.affect.entity;
    let declared = ir.entity(entity);
    let written: Vec<(&String, &ResolvedTypeRef)> = each
        .affect
        .sets
        .iter()
        .map(|set| (&set.target, &set.target_type))
        .chain(
            each.each
                .reads
                .iter()
                .map(|read| (&read.target, &read.target_type)),
        )
        .collect();
    let views: Vec<&ResolvedView> = observing(ir, entity)
        .into_iter()
        .filter(|view| {
            written.iter().all(|(target, type_ref)| {
                view.field(target)
                    .is_some_and(|field| field.type_ref == **type_ref)
            })
        })
        .collect();
    if views.is_empty() {
        return Err(gap(
            format!("{}.affects[{}]", outcome.name, each.index),
            entity.to_string(),
            "is written by an `each:` entry and published by no immediate, unfiltered, \
             unparameterised, unpaged view carrying its identity and every field the entry \
             writes, so a row it updated, one it created and one it left cannot be told apart",
        ));
    }
    let identity = &declared.identity.name;
    let initial = &declared.lifecycle.initial;
    // The held row carries what another command wrote and the entry does not write.
    let mut held = first.carried.clone();
    held.extend(element_row(ir, outcome, each, second, &each.again));
    let rows = [
        (&each.again, held),
        (&each.new, element_row(ir, outcome, each, second, &each.new)),
        (
            &each.decoy,
            element_row(ir, outcome, each, &first.supplied, &each.decoy),
        ),
    ];
    for view in views {
        let name = ViewRef::new(view.name.clone());
        let mut reads = Vec::new();
        for (element, settled) in &rows {
            let Some(id) = element.get(&each.each.member) else {
                continue;
            };
            let mut fields: BTreeMap<String, ScenarioValue> =
                [(identity.clone(), ScenarioValue::literal(id.clone()))]
                    .into_iter()
                    .collect();
            fields.extend(lifecycle_state(ir, entity, view, initial));
            let fields = shown(view, fields, settled);
            require(
                view,
                &name,
                BTreeMap::new(),
                ViewExpectation::Contains { fields },
                &mut reads,
            );
        }
        // One row per identity: a snapshot selects exactly one row or fails.
        for element in [&each.again, &each.new] {
            if let Some(id) = element.get(&each.each.member) {
                reads.push(ScenarioStep::SnapshotSubject {
                    view: name.clone(),
                    subject: [(identity.clone(), ScenarioValue::literal(id.clone()))]
                        .into_iter()
                        .collect(),
                });
            }
        }
        steps.extend(reads);
        source.insert(name.into());
    }
    source.insert(EntityRef::from(entity).into());
    Ok(())
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
pub(super) fn misses(filter: &Predicate) -> Vec<Predicate> {
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
/// what `sets:` wrote, the others as they were. A removed row (ess/23, beyond10x/ess#452) is left
/// nowhere, so only the others remain.
fn left_by(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    rows: &Rows,
    (to, removes): (Option<&StateName>, bool),
) -> Vec<Arrangement> {
    let changed = rows
        .changed
        .iter()
        .filter(|_| !removes)
        .map(|row| Arrangement {
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
    let supplied = supply(ir, command, input, None, None, &BTreeMap::new());
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
        ir,
        entity,
        views,
        &unchanged,
        (None, false),
        outcome,
        &supplied,
        steps,
        source,
    );
    Ok(())
}

/// Whether `steps` take a branch with a set effect (`instances:` or `affects:`): an act whose
/// effect reaches rows beside the one it names, which an arrangement of one row does not account
/// for.
fn reaches_other_rows(ir: &EssIr, steps: &[ScenarioStep]) -> bool {
    steps.iter().any(|step| {
        let ScenarioStep::ExpectOutcome { outcome } = step else {
            return false;
        };
        ir.commands()
            .values()
            .filter(|command| command.name.to_string() == outcome.command.to_string())
            .flat_map(|command| &command.outcomes)
            .any(|branch| {
                branch.name == outcome.outcome
                    && (branch.instances.is_some() || !branch.affects.is_empty())
            })
    })
}

/// Appends to each refusal scenario of a command with an `each:` entry (ess/23, beyond10x/ess#459)
/// the reads of the entry's entity absent for every identity the refused call's list named: a
/// refused run writes none of the rows. An identity an earlier accepted call of the scenario already
/// sent is left out, since that call may have put its row in place; so is a list holding a reference.
fn refused_writes_none(ir: &EssIr, command: &ResolvedCommand, suite: &mut ConformanceSuite) {
    let entries: Vec<(&ResolvedAffect, &ess_compiler::ir::ResolvedEach)> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_none())
        .flat_map(|outcome| &outcome.affects)
        .filter_map(|affect| affect.each.as_ref().map(|each| (affect, each)))
        .collect();
    if entries.is_empty() {
        return;
    }
    let command_ref = CommandRef::new(command.name.clone());
    for refusal in command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_some())
    {
        let id = ScenarioId::Outcome {
            outcome: OutcomeRef::new(command_ref.clone(), refusal.name.clone()),
        };
        let Some(scenario) = suite.scenarios.get_mut(&id) else {
            continue;
        };
        let Some(at) = scenario.steps.iter().rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. } if *command == command_ref)
        }) else {
            continue;
        };
        let ScenarioStep::ExecuteCommand { input, .. } = &scenario.steps[at] else {
            continue;
        };
        let mut reads = Vec::new();
        let mut views = BTreeSet::new();
        for (affect, each) in &entries {
            let Some(ScenarioValue::Literal {
                value: Node::Seq(items),
            }) = input.get(&each.list)
            else {
                continue;
            };
            let mut named: Vec<&Node> = Vec::new();
            for item in items {
                let Node::Map(members) = item else {
                    continue;
                };
                let Some(identity) = members.get(&each.member) else {
                    continue;
                };
                // An earlier call that a refusal answered put no row in place.
                let earlier = scenario.steps[..at]
                    .iter()
                    .enumerate()
                    .any(|(index, step)| {
                        matches!(step, ScenarioStep::ExecuteCommand { input, .. }
                        if input.values().any(|value| mentions(value, identity)))
                            && !refused_at(ir, scenario.steps.get(index + 1))
                    });
                if !earlier && !named.contains(&identity) {
                    named.push(identity);
                }
            }
            let identity_name = &ir.entity(&affect.entity).identity.name;
            for view in observing(ir, &affect.entity) {
                let name = ViewRef::new(view.name.clone());
                for identity in &named {
                    if !reads
                        .iter()
                        .any(|step| matches!(step, ScenarioStep::QueryView { view, .. } if *view == name))
                    {
                        reads.push(ScenarioStep::QueryView {
                            view: name.clone(),
                            params: BTreeMap::new(),
                        });
                    }
                    reads.push(ScenarioStep::ExpectSubjectAbsent {
                        view: name.clone(),
                        subject: [(
                            identity_name.clone(),
                            ScenarioValue::literal((*identity).clone()),
                        )]
                        .into_iter()
                        .collect(),
                    });
                    views.insert(name.clone());
                }
            }
        }
        scenario.steps.extend(reads);
        scenario
            .source
            .extend(views.into_iter().map(EssSemanticRef::from));
    }
}

/// Whether `step` requires a branch reporting an error: the answer to a call that changed nothing.
fn refused_at(ir: &EssIr, step: Option<&ScenarioStep>) -> bool {
    let Some(ScenarioStep::ExpectOutcome { outcome }) = step else {
        return false;
    };
    ir.commands()
        .values()
        .filter(|command| command.name.to_string() == outcome.command.to_string())
        .flat_map(|command| &command.outcomes)
        .any(|branch| branch.name == outcome.outcome && branch.error.is_some())
}

/// Whether `value` holds `node` anywhere: as itself, or inside a list or mapping.
fn mentions(value: &ScenarioValue, node: &Node) -> bool {
    fn within(held: &Node, node: &Node) -> bool {
        held == node
            || match held {
                Node::Seq(items) => items.iter().any(|item| within(item, node)),
                Node::Map(members) => members.values().any(|member| within(member, node)),
                _ => false,
            }
    }
    match value {
        ScenarioValue::Literal { value } => within(value, node),
        ScenarioValue::List { items } => items.iter().any(|item| mentions(item, node)),
        ScenarioValue::Members { members } => members.values().any(|member| mentions(member, node)),
        _ => false,
    }
}
