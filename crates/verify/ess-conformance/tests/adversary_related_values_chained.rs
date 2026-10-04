//! Adversary cases for `{related: …}` through an Optional reference and across two references
//! (ess/22, beyond10x/ess#285, pass 1).
//!
//! Each case drives the synthesized suite of a model the validator admits against the native
//! interpreter or a hand-written target, and asserts what the feature promises: a correct target
//! passes, and a target that reads the wrong value fails the branch's own scenario.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const CHAINED: &str = include_str!("fixtures/related-values-chained.yaml");
const BOOKED: &str = "demo.costs.Book/outcome/booked";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("costs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture holds {from:?}");
    text.replace(from, to)
}

/// Every scenario of `suite` that did not pass against `target`, by id, with its detail.
fn failing_against(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let mut failed: Vec<String> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect();
    failed.sort();
    failed
}

fn interpreter_failures(text: &str, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(text)),
        )
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| format!("{result:#?}"))
        .collect()
}

// ---- models -------------------------------------------------------------------------------

/// The objective's initiative can be changed after the objective was set.
fn with_retarget() -> String {
    let text = replaced(
        CHAINED,
        "  - name: demo.costs.CostBooked\n",
        "  - name: demo.costs.ObjectiveRetargeted\n    fields:\n      - {name: objective_id, type: \
         demo.costs.ObjectiveId}\n  - name: demo.costs.CostBooked\n",
    );
    let text = replaced(
        &text,
        "may: [demo.costs.StartInitiative, demo.costs.SetObjective, demo.costs.Book]",
        "may: [demo.costs.StartInitiative, demo.costs.SetObjective, demo.costs.Retarget, \
         demo.costs.Book]",
    );
    replaced(
        &text,
        "  - name: demo.costs.Book\n    input:",
        "  - name: demo.costs.Retarget
    input:
      - {name: objective_id, type: demo.costs.ObjectiveId}
      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}
    outcomes:
      - name: retargeted
        updates: demo.costs.Objective
        instance: objective_id
        emits: [demo.costs.ObjectiveRetargeted]
        payload: {demo.costs.ObjectiveRetargeted: {objective_id: input.objective_id}}
        sets:
          initiative_id: input.initiative_id
  - name: demo.costs.Book
    input:",
    )
}

/// The copied value is published on the event only: the entry does not store it.
fn event_only() -> String {
    replaced(
        CHAINED,
        "          outcome_id: {related: {via: [objective_id, initiative_id], field: outcome_id}}\n          cents",
        "          cents",
    )
}

/// The entry's objective may itself be absent: both references of the chain are Optional.
fn optional_objective() -> String {
    replaced(
        CHAINED,
        "{name: objective_id, type: demo.costs.ObjectiveId}\n      - {name: initiative_id",
        "{name: objective_id, type: 'Optional<demo.costs.ObjectiveId>'}\n      - {name: initiative_id",
    )
}

/// Two reads through one Optional input: in one hop, and chained on to the initiative's sponsor.
const SAME_INPUT_TWO_PATHS: &str = "format: ess/22
system: demo
version: v1
domain: demo.costs
types:
  - {name: demo.costs.InitiativeId, kind: newtype, of: String}
  - {name: demo.costs.SponsorId, kind: newtype, of: String}
  - {name: demo.costs.EntryId, kind: newtype, of: String}
  - {name: demo.costs.OutcomeId, kind: newtype, of: String}
entities:
  - name: demo.costs.Sponsor
    identity: {name: sponsor_id, type: demo.costs.SponsorId}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.costs.Initiative
    identity: {name: initiative_id, type: demo.costs.InitiativeId}
    fields:
      - {name: outcome_id, type: demo.costs.OutcomeId}
      - {name: sponsor_id, type: demo.costs.SponsorId}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.costs.CostEntry
    identity: {name: entry_id, type: demo.costs.EntryId}
    fields:
      - {name: outcome_id, type: 'Optional<demo.costs.OutcomeId>'}
      - {name: sponsor, type: 'Optional<String>'}
      - {name: cents, type: Integer}
    lifecycle: {initial: Booked, states: [Booked], terminal: [Booked]}
events:
  - name: demo.costs.SponsorAdded
    fields:
      - {name: sponsor_id, type: demo.costs.SponsorId}
  - name: demo.costs.InitiativeStarted
    fields:
      - {name: initiative_id, type: demo.costs.InitiativeId}
  - name: demo.costs.CostBooked
    fields:
      - {name: entry_id, type: demo.costs.EntryId}
actors:
  - name: demo.costs.Controller
    may: [demo.costs.AddSponsor, demo.costs.StartInitiative, demo.costs.Book]
commands:
  - name: demo.costs.AddSponsor
    input:
      - {name: name, type: String}
    outcomes:
      - name: added
        creates: demo.costs.Sponsor
        instance: sponsor_id
        emits: [demo.costs.SponsorAdded]
        payload:
          demo.costs.SponsorAdded: {sponsor_id: {generated: true}}
        sets:
          name: input.name
  - name: demo.costs.StartInitiative
    input:
      - {name: outcome_id, type: demo.costs.OutcomeId}
      - {name: sponsor_id, type: demo.costs.SponsorId}
    outcomes:
      - name: started
        creates: demo.costs.Initiative
        instance: initiative_id
        emits: [demo.costs.InitiativeStarted]
        payload:
          demo.costs.InitiativeStarted: {initiative_id: {generated: true}}
        sets:
          outcome_id: input.outcome_id
          sponsor_id: input.sponsor_id
  - name: demo.costs.Book
    input:
      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}
      - {name: cents, type: Integer}
    outcomes:
      - name: booked
        creates: demo.costs.CostEntry
        instance: entry_id
        emits: [demo.costs.CostBooked]
        payload:
          demo.costs.CostBooked: {entry_id: {generated: true}}
        sets:
          outcome_id: {related: {via: input.initiative_id, field: outcome_id}}
          sponsor: {related: {via: [input.initiative_id, sponsor_id], field: name}}
          cents: input.cents
views:
  - name: demo.costs.CostEntries
    source: demo.costs.CostEntry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: demo.costs.EntryId}
      - {name: outcome_id, type: 'Optional<demo.costs.OutcomeId>'}
      - {name: sponsor, type: 'Optional<String>'}
      - {name: cents, type: Integer}
";

/// An `updates:` branch copying through the entry's own stored Optional reference.
fn rebook_through_stored_optional() -> String {
    let text = replaced(
        CHAINED,
        "  - name: demo.costs.CostBooked\n",
        "  - name: demo.costs.CostRebooked\n    fields:\n      - {name: entry_id, type: \
         demo.costs.EntryId}\n  - name: demo.costs.CostBooked\n",
    );
    let text = replaced(
        &text,
        "demo.costs.SetObjective, demo.costs.Book]",
        "demo.costs.SetObjective, demo.costs.Book, demo.costs.Rebook]",
    );
    replaced(
        &text,
        "views:\n",
        "  - name: demo.costs.Rebook
    input:
      - {name: entry_id, type: demo.costs.EntryId}
    outcomes:
      - name: rebooked
        updates: demo.costs.CostEntry
        instance: entry_id
        emits: [demo.costs.CostRebooked]
        payload: {demo.costs.CostRebooked: {entry_id: input.entry_id}}
        sets:
          outcome_id: {related: {via: initiative_id, field: outcome_id}}
views:
",
    )
}

// ---- a hand-written target -----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Reads the outcome of the initiative the references name, as they are now.
    Correct,
    /// Writes the absent value on the row, and publishes the latest initiative's outcome on the
    /// event where a reference is absent.
    EventIgnoresAbsence,
    /// Follows the objective's initiative as it was when the objective was set, not as it is now.
    SnapshotMiddle,
    /// Handles an absent objective, but copies the latest initiative's outcome where the
    /// objective names no initiative.
    IgnoresHopAbsence,
    /// Reads the latest objective's initiative, whichever objective the entry names.
    LatestObjective,
    /// Reads the first objective's initiative, whichever objective the entry names.
    FirstObjective,
    /// Copies the value it read first in the scenario for every later booking.
    CachesFirstRead,
}

type Row = BTreeMap<String, Node>;

struct Objective {
    id: Node,
    set_with: Option<Node>,
    now: Option<Node>,
}

#[derive(Default)]
struct World {
    initiatives: Vec<(Node, Node)>,
    objectives: Vec<Objective>,
    entries: Vec<Row>,
    /// The first value read in the scenario, once read: at most one entry.
    cached: Vec<Option<Node>>,
}

struct Costs {
    mode: Mode,
    /// Whether the model stores the copied value on the entry.
    stores: bool,
    world: RefCell<World>,
    minted: Cell<u32>,
}

impl Costs {
    fn new(mode: Mode, stores: bool) -> Self {
        Self {
            mode,
            stores,
            world: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn present(input: &Row, name: &str) -> Option<Node> {
        input
            .get(name)
            .filter(|value| **value != Node::Null)
            .cloned()
    }

    /// The correct value, `Err` for a missing row.
    fn correct(world: &World, input: &Row) -> Result<Option<Node>, ()> {
        let Some(objective) = Self::present(input, "objective_id") else {
            return Ok(None);
        };
        let objective = world
            .objectives
            .iter()
            .find(|held| held.id == objective)
            .ok_or(())?;
        let Some(initiative) = &objective.now else {
            return Ok(None);
        };
        Self::outcome_of(world, initiative).map(Some)
    }

    fn outcome_of(world: &World, initiative: &Node) -> Result<Node, ()> {
        world
            .initiatives
            .iter()
            .find(|(held, _)| held == initiative)
            .map(|(_, outcome)| outcome.clone())
            .ok_or(())
    }

    fn latest(world: &World) -> Option<Node> {
        world.initiatives.last().map(|(_, outcome)| outcome.clone())
    }

    /// What the row and the event carry, in that order.
    fn copied(&self, world: &mut World, input: &Row) -> Result<(Option<Node>, Option<Node>), ()> {
        let value = match self.mode {
            Mode::Correct | Mode::EventIgnoresAbsence => Self::correct(world, input)?,
            Mode::CachesFirstRead => {
                if let Some(cached) = world.cached.first() {
                    cached.clone()
                } else {
                    let value = Self::correct(world, input)?;
                    world.cached.push(value.clone());
                    value
                }
            }
            Mode::SnapshotMiddle | Mode::IgnoresHopAbsence => {
                let Some(objective) = Self::present(input, "objective_id") else {
                    return Ok((None, None));
                };
                let objective = world
                    .objectives
                    .iter()
                    .find(|held| held.id == objective)
                    .ok_or(())?;
                let initiative = if self.mode == Mode::SnapshotMiddle {
                    &objective.set_with
                } else {
                    &objective.now
                };
                match initiative {
                    Some(initiative) => Some(Self::outcome_of(world, initiative)?),
                    None if self.mode == Mode::IgnoresHopAbsence => Self::latest(world),
                    None => None,
                }
            }
            Mode::LatestObjective | Mode::FirstObjective => {
                if Self::present(input, "objective_id").is_none() {
                    return Ok((None, None));
                }
                let objective = if self.mode == Mode::LatestObjective {
                    world.objectives.last()
                } else {
                    world.objectives.first()
                }
                .ok_or(())?;
                match &objective.now {
                    Some(initiative) => Some(Self::outcome_of(world, initiative)?),
                    None => None,
                }
            }
        };
        let event = match (self.mode, &value) {
            (Mode::EventIgnoresAbsence, None) => Self::latest(world),
            _ => value.clone(),
        };
        Ok((value, event))
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Costs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("costs-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.world.replace(World::default());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let mut world = self.world.borrow_mut();
        let input = &request.input;
        let result = match command.to_string().as_str() {
            "demo.costs.StartInitiative" => {
                let id = Node::Text(format!("initiative-{n}"));
                world
                    .initiatives
                    .push((id.clone(), input["outcome_id"].clone()));
                SemanticCommandResult::took(outcome(&command, "started")).emitting(
                    ObservedEvent::new("demo.costs.InitiativeStarted".parse().unwrap())
                        .with("initiative_id", id),
                )
            }
            "demo.costs.SetObjective" => {
                let id = Node::Text(format!("objective-{n}"));
                let initiative = Self::present(input, "initiative_id");
                world.objectives.push(Objective {
                    id: id.clone(),
                    set_with: initiative.clone(),
                    now: initiative,
                });
                SemanticCommandResult::took(outcome(&command, "set")).emitting(
                    ObservedEvent::new("demo.costs.ObjectiveSet".parse().unwrap())
                        .with("objective_id", id),
                )
            }
            "demo.costs.Retarget" => {
                let id = input["objective_id"].clone();
                let initiative = Self::present(input, "initiative_id");
                let Some(objective) = world.objectives.iter_mut().find(|held| held.id == id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                objective.now = initiative;
                SemanticCommandResult::took(outcome(&command, "retargeted")).emitting(
                    ObservedEvent::new("demo.costs.ObjectiveRetargeted".parse().unwrap())
                        .with("objective_id", id),
                )
            }
            "demo.costs.Book" => {
                let Ok((row, event)) = self.copied(&mut world, input) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let id = Node::Text(format!("entry-{n}"));
                let stored = if self.stores {
                    row.unwrap_or(Node::Null)
                } else {
                    Node::Null
                };
                world.entries.push(Row::from([
                    ("entry_id".to_owned(), id.clone()),
                    ("outcome_id".to_owned(), stored),
                    ("cents".to_owned(), input["cents"].clone()),
                ]));
                SemanticCommandResult::took(outcome(&command, "booked")).emitting(
                    ObservedEvent::new("demo.costs.CostBooked".parse().unwrap())
                        .with("entry_id", id)
                        .with("outcome_id", event.unwrap_or(Node::Null)),
                )
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let world = self.world.borrow();
        match request.view.to_string().as_str() {
            "demo.costs.CostPerOutcome" => {
                let mut groups: Vec<(Node, i64)> = Vec::new();
                for entry in &world.entries {
                    let key = entry["outcome_id"].clone();
                    let Node::Number(cents) = &entry["cents"] else {
                        panic!("cents: {entry:?}")
                    };
                    let cents = cents.as_i64().expect("whole cents");
                    match groups.iter_mut().find(|(held, _)| *held == key) {
                        Some((_, total)) => *total += cents,
                        None => groups.push((key, cents)),
                    }
                }
                Ok(SemanticViewResult::of(groups.into_iter().map(
                    |(key, total)| {
                        Row::from([
                            ("outcome_id".to_owned(), key),
                            ("total".to_owned(), Node::Number(total.into())),
                        ])
                    },
                )))
            }
            "demo.costs.CostEntries" => Ok(SemanticViewResult::of(world.entries.iter().cloned())),
            other => Err(TargetError::unsupported("view", other)),
        }
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// The suite of `text`, its `Correct` target's failures (which must be none), and `mode`'s.
fn faulty_against(text: &str, mode: Mode, stores: bool) -> (Vec<String>, Vec<String>) {
    let suite = synthesis(text).suite;
    assert!(
        suite.scenarios.keys().any(|id| id.to_string() == BOOKED),
        "the booking scenario is synthesized"
    );
    (
        failing_against(&suite, &Costs::new(Mode::Correct, stores)),
        failing_against(&suite, &Costs::new(mode, stores)),
    )
}

// ---- cases --------------------------------------------------------------------------------

/// The suite witnesses an absent reference on the #285 reduction itself, but the run that leaves
/// the objective's initiative absent asserts nothing about the event's copied value: a target that
/// writes the row right and publishes some other initiative's outcome on the event passes.
#[test]
fn adversary_285_an_event_copying_a_value_through_an_absent_reference_is_asserted() {
    let (correct, faulty) = faulty_against(CHAINED, Mode::EventIgnoresAbsence, true);
    assert_eq!(correct, Vec::<String>::new(), "the correct target passes");
    assert!(
        faulty.iter().any(|id| id == BOOKED),
        "a target publishing a present outcome for an absent reference fails {BOOKED}: \
         failing {faulty:?}"
    );
}

/// Where the copied value is published and not stored, the absent witness asserts nothing about
/// it at all: a target ignoring absence passes every scenario.
#[test]
fn adversary_285_an_event_only_copy_through_an_absent_reference_is_witnessed() {
    let (correct, faulty) = faulty_against(&event_only(), Mode::EventIgnoresAbsence, false);
    assert_eq!(correct, Vec::<String>::new(), "the correct target passes");
    assert!(
        faulty.iter().any(|id| id == BOOKED),
        "a target publishing a present outcome for an absent reference fails {BOOKED}: \
         failing {faulty:?}"
    );
}

/// The source reads the middle row as it is at the branch. A target that follows the objective's
/// initiative as it was when the objective was set — though the model lets `Retarget` change it —
/// fails the booking scenario, as a one-hop read's snapshot fails its own (`related::changed`).
#[test]
fn adversary_285_a_target_snapshotting_the_middle_reference_fails() {
    let (correct, faulty) = faulty_against(&with_retarget(), Mode::SnapshotMiddle, true);
    assert_eq!(correct, Vec::<String>::new(), "the correct target passes");
    assert!(
        faulty.iter().any(|id| id == BOOKED),
        "a target reading the objective's initiative as first set fails {BOOKED}: failing \
         {faulty:?}"
    );
}

/// With both references Optional, each absence is a path of its own. A target that handles an
/// absent objective but copies some initiative's outcome where the objective names none fails the
/// booking scenario.
#[test]
fn adversary_285_the_second_references_absence_is_witnessed_when_the_first_is_optional_too() {
    let (correct, faulty) = faulty_against(&optional_objective(), Mode::IgnoresHopAbsence, true);
    assert_eq!(correct, Vec::<String>::new(), "the correct target passes");
    assert!(
        faulty.iter().any(|id| id == BOOKED),
        "a target ignoring the objective's absent initiative fails {BOOKED}: failing {faulty:?}"
    );
}

/// Two reads through one Optional input — in one hop, and chained — are both absent where the
/// input is left out. The absent run leaves the input out for the first, and the native
/// interpreter, which reads both as absent, passes the suite.
#[test]
fn adversary_285_two_reads_through_one_optional_input_agree_with_the_interpreter() {
    let result = synthesis(SAME_INPUT_TWO_PATHS);
    let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    let failed = interpreter_failures(SAME_INPUT_TWO_PATHS, &result.suite);
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the native interpreter passes the suite (refusals: {refusals:?})"
    );
}

/// An `updates:` branch copying through the subject's stored Optional reference is admitted from
/// ess/22, and its absent witness can be arranged — create the entry without the reference — so
/// the branch is not refused.
#[test]
fn adversary_285_a_stored_optional_reference_on_an_update_is_not_refused() {
    let text = rebook_through_stored_optional();
    let result = synthesis(&text);
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("Rebook"))
        .collect();
    let scenario = result
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "demo.costs.Rebook/outcome/rebooked");
    assert_eq!(
        refused,
        Vec::<String>::new(),
        "Rebook is witnessed (its scenario is in the suite: {scenario})"
    );
}

/// The middle row's creating event carries the reference the chained arrangement rewrites, and
/// the entry copies the initiative's own identity, grouped by an aggregate: the native interpreter
/// passes the suite of each.
#[test]
fn adversary_285_rewritten_middle_rows_and_copied_identities_agree_with_the_interpreter() {
    let carried = replaced(
        CHAINED,
        "  - name: demo.costs.ObjectiveSet\n    fields:\n      - {name: objective_id, type: \
         demo.costs.ObjectiveId}\n",
        "  - name: demo.costs.ObjectiveSet\n    fields:\n      - {name: objective_id, type: \
         demo.costs.ObjectiveId}\n      - {name: initiative_id, type: \
         'Optional<demo.costs.InitiativeId>'}\n",
    );
    let carried = replaced(
        &carried,
        "demo.costs.ObjectiveSet: {objective_id: {generated: true}}",
        "demo.costs.ObjectiveSet: {objective_id: {generated: true}, initiative_id: \
         input.initiative_id}",
    );
    let identity = CHAINED.replace(
        "{related: {via: [objective_id, initiative_id], field: outcome_id}}",
        "{related: {via: [objective_id, initiative_id], field: initiative_id}}",
    );
    let identity = replaced(
        &identity,
        "      - {name: outcome_id, type: 'Optional<demo.costs.OutcomeId>'}\n      - {name: cents",
        "      - {name: outcome_id, type: 'Optional<demo.costs.InitiativeId>'}\n      - {name: cents",
    );
    let identity = identity.replace(
        "{name: outcome_id, type: 'Optional<demo.costs.OutcomeId>'}",
        "{name: outcome_id, type: 'Optional<demo.costs.InitiativeId>'}",
    );
    let mut failures = Vec::new();
    for (name, text) in [("carried", carried), ("identity", identity)] {
        let result = synthesis(&text);
        let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
        assert!(
            result
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string() == BOOKED),
            "{name}: the booking scenario is synthesized: {refusals:?}"
        );
        for failed in interpreter_failures(&text, &result.suite) {
            failures.push(format!("{name} (refusals {refusals:?}): {failed}"));
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

/// A chain through a self-referencing entity: a note copies the title of its task's parent task.
const SELF_CHAIN: &str = "format: ess/22
system: demo
version: v1
domain: demo.tasks
types:
  - {name: demo.tasks.TaskId, kind: newtype, of: String}
  - {name: demo.tasks.NoteId, kind: newtype, of: String}
entities:
  - name: demo.tasks.Task
    identity: {name: task_id, type: demo.tasks.TaskId}
    fields:
      - {name: title, type: String}
      - {name: parent_id, type: 'Optional<demo.tasks.TaskId>'}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.tasks.Note
    identity: {name: note_id, type: demo.tasks.NoteId}
    fields:
      - {name: parent_title, type: 'Optional<String>'}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept]}
events:
  - name: demo.tasks.TaskAdded
    fields:
      - {name: task_id, type: demo.tasks.TaskId}
  - name: demo.tasks.Noted
    fields:
      - {name: note_id, type: demo.tasks.NoteId}
actors:
  - name: demo.tasks.Planner
    may: [demo.tasks.AddTask, demo.tasks.Annotate]
commands:
  - name: demo.tasks.AddTask
    input:
      - {name: title, type: String}
      - {name: parent_id, type: 'Optional<demo.tasks.TaskId>'}
    outcomes:
      - name: added
        creates: demo.tasks.Task
        instance: task_id
        emits: [demo.tasks.TaskAdded]
        payload:
          demo.tasks.TaskAdded: {task_id: {generated: true}}
        sets:
          title: input.title
          parent_id: input.parent_id
  - name: demo.tasks.Annotate
    input:
      - {name: task_id, type: demo.tasks.TaskId}
    outcomes:
      - name: noted
        creates: demo.tasks.Note
        instance: note_id
        emits: [demo.tasks.Noted]
        payload:
          demo.tasks.Noted: {note_id: {generated: true}}
        sets:
          parent_title: {related: {via: [input.task_id, parent_id], field: title}}
views:
  - name: demo.tasks.Notes
    source: demo.tasks.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.tasks.NoteId}
      - {name: parent_title, type: 'Optional<String>'}
";

/// Cycles: a chain through a self-referencing entity, and one whose second reference names a row
/// of the entity the branch creates. The native interpreter passes the suite of each.
#[test]
fn adversary_285_cyclic_chains_agree_with_the_interpreter() {
    let back = replaced(
        CHAINED,
        "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n    lifecycle: \
         {initial: Active",
        "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n      - {name: \
         last_entry_id, type: 'Optional<demo.costs.EntryId>'}\n    lifecycle: {initial: Active",
    );
    let back = replaced(
        &back,
        "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n    outcomes:\n      - name: set",
        "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n      - {name: \
         last_entry_id, type: 'Optional<demo.costs.EntryId>'}\n    outcomes:\n      - name: set",
    );
    let back = replaced(
        &back,
        "          initiative_id: input.initiative_id\n  - name: demo.costs.Book",
        "          initiative_id: input.initiative_id\n          last_entry_id: \
         input.last_entry_id\n  - name: demo.costs.Book",
    );
    let back = replaced(
        &back,
        "          cents: input.cents\n",
        "          cents: input.cents\n          prior_cents: {related: {via: [objective_id, \
         last_entry_id], field: cents}}\n",
    );
    let back = replaced(
        &back,
        "      - {name: cents, type: Integer}\n    lifecycle: {initial: Booked",
        "      - {name: cents, type: Integer}\n      - {name: prior_cents, type: \
         'Optional<Integer>'}\n    lifecycle: {initial: Booked",
    );
    let mut failures = Vec::new();
    for (name, text) in [("self", SELF_CHAIN.to_owned()), ("back", back)] {
        let result = synthesis(&text);
        let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
        assert!(
            result.suite.scenarios.keys().any(|id| {
                let id = id.to_string();
                id == BOOKED || id == "demo.tasks.Annotate/outcome/noted"
            }),
            "{name}: the copying branch's scenario is synthesized: {refusals:?}"
        );
        for failed in interpreter_failures(&text, &result.suite) {
            failures.push(format!("{name} (refusals {refusals:?}): {failed}"));
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

/// [`optional_objective`] with no aggregate view, so only the booking's own scenario
/// can witness the objective's absent initiative.
#[test]
fn adversary_285_without_an_aggregate_view_a_target_ignoring_the_second_absence_fails() {
    let text = optional_objective();
    let at = text.find("  - name: demo.costs.CostPerOutcome").unwrap();
    let end = text.find("  - name: demo.costs.CostEntries").unwrap();
    let text = format!("{}{}", &text[..at], &text[end..]);
    let suite = synthesis(&text).suite;
    let correct = failing_against(&suite, &Costs::new(Mode::Correct, true));
    let faulty = failing_against(&suite, &Costs::new(Mode::IgnoresHopAbsence, true));
    assert_eq!(correct, Vec::<String>::new(), "the correct target passes");
    assert_ne!(
        faulty,
        Vec::<String>::new(),
        "a target ignoring the objective's absent initiative fails some scenario"
    );
}

/// The faults the brief names that the synthesized suite should catch on the reduction: reading
/// the wrong middle row, and caching the first read.
#[test]
fn adversary_285_wrong_middle_rows_and_cached_reads_fail_the_booking_scenario() {
    let mut survived = Vec::new();
    for mode in [
        Mode::LatestObjective,
        Mode::FirstObjective,
        Mode::CachesFirstRead,
    ] {
        let (correct, faulty) = faulty_against(CHAINED, mode, true);
        assert_eq!(correct, Vec::<String>::new(), "the correct target passes");
        if !faulty.iter().any(|id| id == BOOKED) {
            survived.push(format!("{mode:?}: {faulty:?}"));
        }
    }
    assert_eq!(survived, Vec::<String>::new(), "survived");
}
