//! `{related: …}` through an Optional reference and across two references (ess/22,
//! beyond10x/ess#285): the reduction synthesizes, `CostPerOutcome` is witnessed with no
//! ESS-SYNTH-017, an absent reference is witnessed with an absent copied value, and the suite
//! passes the native interpreter and a hand-written target while failing every faulty one — one
//! that treats an absent reference as a missing row, one that reads another hop, one that ignores
//! absence, and one that reads the latest row.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const CHAINED: &str = include_str!("fixtures/related-values-chained.yaml");
const TWO_HOPS: &str = "{related: {via: [objective_id, initiative_id], field: outcome_id}}";
const ONE_OPTIONAL_HOP: &str = "{related: {via: initiative_id, field: outcome_id}}";

const BOOKED: &str = "demo.costs.Book/outcome/booked";
const AGGREGATE: &str = "demo.costs.CostPerOutcome/aggregate";

/// Which of the two forms the model reads `outcome_id` by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// `via: [objective_id, initiative_id]`: the objective's Optional initiative.
    TwoHops,
    /// `via: initiative_id`, the issue's own form: the entry's Optional initiative.
    OneHop,
}

impl Shape {
    fn text(self) -> String {
        match self {
            Self::TwoHops => CHAINED.to_owned(),
            Self::OneHop => CHAINED.replace(TWO_HOPS, ONE_OPTIONAL_HOP),
        }
    }
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("costs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(shape: Shape) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(&shape.text()))
}

fn suite(shape: Shape) -> ConformanceSuite {
    let synthesis = synthesis(shape);
    assert_eq!(
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        Vec::<String>::new(),
        "{shape:?}: nothing is refused"
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

#[test]
fn issue_285_cost_per_outcome_synthesizes_with_no_synth_017() {
    for shape in [Shape::TwoHops, Shape::OneHop] {
        let result = synthesis(shape);
        let unwitnessed: Vec<String> = result
            .refusals
            .iter()
            .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
            .map(ToString::to_string)
            .collect();
        assert_eq!(unwitnessed, Vec::<String>::new(), "{shape:?}");
        assert!(
            result
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string() == AGGREGATE),
            "{shape:?}: the aggregate is synthesized: {:#?}",
            result.refusals
        );
    }
}

#[test]
fn issue_285_the_native_interpreter_passes_the_suite() {
    for shape in [Shape::TwoHops, Shape::OneHop] {
        let model = ir(&shape.text());
        let suite = suite(shape);
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let run = Runner::for_suite(admitted.suite()).run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        );
        let failed: Vec<String> = run
            .scenarios
            .iter()
            .filter(|result| result.status != Status::Passed)
            .map(|result| format!("{result:#?}"))
            .collect();
        assert_eq!(failed, Vec::<String>::new(), "{shape:?}");
    }
}

/// The `CostBooked` expectations of a scenario, in order: the `outcome_id` each asserts.
fn asserted_outcomes(scenario: &ConformanceScenario) -> Vec<Option<Node>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.costs.CostBooked" =>
            {
                Some(payload.get("outcome_id").cloned())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn issue_285_an_absent_reference_is_witnessed_with_an_absent_copied_value() {
    for shape in [Shape::TwoHops, Shape::OneHop] {
        let suite = suite(shape);
        let booked = scenario(&suite, BOOKED);
        let asserted = asserted_outcomes(booked);
        assert!(
            asserted
                .iter()
                .any(|value| matches!(value, Some(Node::Text(_)))),
            "{shape:?}: a present outcome is asserted: {asserted:?}"
        );
        // An absent top-level event field may be left out or published as `null`, so the event of
        // the absent run asserts no value there; the row it leaves asserts the absent value.
        assert_eq!(asserted.len(), 2, "{shape:?}: two bookings: {asserted:?}");
        assert_eq!(asserted[1], None, "{shape:?}: {asserted:?}");
        let rows: Vec<Option<ScenarioValue>> = booked
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExpectView {
                    view,
                    expectation: ViewExpectation::Contains { fields },
                } if view.to_string() == "demo.costs.CostEntries" => {
                    Some(fields.get("outcome_id").cloned())
                }
                _ => None,
            })
            .collect();
        assert!(
            rows.contains(&Some(ScenarioValue::literal(Node::Null))),
            "{shape:?}: an absent outcome is asserted on the row: {rows:?}"
        );
        // The reference is left out where it is written: the objective's initiative for two hops,
        // the entry's own for one.
        let (command, input) = match shape {
            Shape::TwoHops => ("demo.costs.SetObjective", "initiative_id"),
            Shape::OneHop => ("demo.costs.Book", "initiative_id"),
        };
        assert!(
            booked.steps.iter().any(|step| matches!(
                step,
                ScenarioStep::ExecuteCommand { command: run, input: sent, .. }
                    if run.to_string() == command && !sent.contains_key(input)
            )),
            "{shape:?}: `{command}` is sent without `{input}`"
        );
    }
}

#[test]
fn issue_285_two_hops_point_the_entry_at_an_objective_naming_the_initiative() {
    let suite = suite(Shape::TwoHops);
    let booked = scenario(&suite, BOOKED);
    let instance = |value: Option<&ScenarioValue>| match value {
        Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
        _ => None,
    };
    let mut objectives = BTreeMap::new();
    let mut pending = None;
    let mut booked_with = Vec::new();
    for step in &booked.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                match command.to_string().as_str() {
                    "demo.costs.SetObjective" => {
                        pending = Some(instance(input.get("initiative_id")));
                    }
                    "demo.costs.Book" => booked_with.push(instance(input.get("objective_id"))),
                    _ => {}
                }
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.costs.Objective" => {
                objectives.insert(instance.to_string(), pending.take().flatten());
            }
            _ => {}
        }
    }
    let first = booked_with[0]
        .clone()
        .expect("Book names an arranged objective");
    assert!(
        objectives.get(&first).cloned().flatten().is_some(),
        "the objective names an arranged initiative: {objectives:?}"
    );
}

// ---- running the suite against hand-written targets ---------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Reads the outcome of the initiative the references name; absent where one is absent.
    Correct,
    /// Treats an absent reference as a row that does not exist: refuses the booking.
    AbsentAsMissing,
    /// Copies the outcome of the latest initiative where the reference is absent.
    IgnoresAbsence,
    /// Follows the other hop: the entry's own initiative for two hops, the objective's for one.
    WrongHop,
    /// Copies the outcome of the latest initiative, whichever the references name.
    LatestInitiative,
}

type Row = BTreeMap<String, Node>;

#[derive(Default)]
struct World {
    initiatives: Vec<(Node, Node)>,
    objectives: Vec<(Node, Option<Node>)>,
    entries: Vec<Row>,
}

struct Costs {
    shape: Shape,
    mode: Mode,
    world: RefCell<World>,
    minted: Cell<u32>,
}

impl Costs {
    fn new(shape: Shape, mode: Mode) -> Self {
        Self {
            shape,
            mode,
            world: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    /// The initiative the objective `id` names, `None` where it names none; `Err` where there is
    /// no such objective.
    fn objective_initiative(world: &World, id: &Node) -> Result<Option<Node>, ()> {
        world
            .objectives
            .iter()
            .find(|(objective, _)| objective == id)
            .map(|(_, initiative)| initiative.clone())
            .ok_or(())
    }

    /// The outcome this implementation copies for a booking, `Ok(None)` for absent, `Err` for a
    /// refusal.
    fn outcome(&self, world: &World, input: &Row) -> Result<Option<Node>, ()> {
        let own = input
            .get("initiative_id")
            .filter(|value| **value != Node::Null)
            .cloned();
        let through_objective = || {
            input
                .get("objective_id")
                .map_or(Err(()), |id| Self::objective_initiative(world, id))
        };
        let reference = match (self.shape, self.mode) {
            (Shape::TwoHops, Mode::WrongHop) => own,
            (Shape::TwoHops, _) => through_objective()?,
            (Shape::OneHop, Mode::WrongHop) => through_objective().ok().flatten(),
            (Shape::OneHop, _) => own,
        };
        let latest = || world.initiatives.last().map(|(_, outcome)| outcome.clone());
        let Some(reference) = reference else {
            return match self.mode {
                Mode::AbsentAsMissing => Err(()),
                Mode::IgnoresAbsence => Ok(latest()),
                _ => Ok(None),
            };
        };
        if self.mode == Mode::LatestInitiative {
            return Ok(latest());
        }
        world
            .initiatives
            .iter()
            .find(|(initiative, _)| *initiative == reference)
            .map(|(_, outcome)| Some(outcome.clone()))
            .ok_or(())
    }

    fn per_outcome(&self) -> Vec<Row> {
        let world = self.world.borrow();
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
        groups
            .into_iter()
            .map(|(key, total)| {
                Row::from([
                    ("outcome_id".to_owned(), key),
                    ("total".to_owned(), Node::Number(total.into())),
                ])
            })
            .collect()
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Costs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("costs-fixture", "1"))
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
                let initiative = input
                    .get("initiative_id")
                    .filter(|value| **value != Node::Null)
                    .cloned();
                world.objectives.push((id.clone(), initiative));
                SemanticCommandResult::took(outcome(&command, "set")).emitting(
                    ObservedEvent::new("demo.costs.ObjectiveSet".parse().unwrap())
                        .with("objective_id", id),
                )
            }
            "demo.costs.Book" => {
                let Ok(copied) = self.outcome(&world, input) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let copied = copied.unwrap_or(Node::Null);
                let id = Node::Text(format!("entry-{n}"));
                world.entries.push(Row::from([
                    ("entry_id".to_owned(), id.clone()),
                    ("outcome_id".to_owned(), copied.clone()),
                    ("cents".to_owned(), input["cents"].clone()),
                ]));
                SemanticCommandResult::took(outcome(&command, "booked")).emitting(
                    ObservedEvent::new("demo.costs.CostBooked".parse().unwrap())
                        .with("entry_id", id)
                        .with("outcome_id", copied),
                )
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        match request.view.to_string().as_str() {
            "demo.costs.CostPerOutcome" => Ok(SemanticViewResult::of(self.per_outcome())),
            "demo.costs.CostEntries" => Ok(SemanticViewResult::of(
                self.world.borrow().entries.iter().cloned(),
            )),
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

/// Every scenario that did not pass against `mode`, by id.
fn failing(shape: Shape, mode: Mode) -> Vec<String> {
    let suite = suite(shape);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let mut failed: Vec<String> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Costs::new(shape, mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect();
    failed.sort();
    failed
}

#[test]
fn issue_285_a_target_reading_the_named_rows_passes() {
    for shape in [Shape::TwoHops, Shape::OneHop] {
        assert_eq!(
            failing(shape, Mode::Correct),
            Vec::<String>::new(),
            "{shape:?}"
        );
    }
}

#[test]
fn issue_285_every_faulty_target_fails_the_booking_scenario() {
    let mut survived = Vec::new();
    for shape in [Shape::TwoHops, Shape::OneHop] {
        for mode in [
            Mode::AbsentAsMissing,
            Mode::IgnoresAbsence,
            Mode::WrongHop,
            Mode::LatestInitiative,
        ] {
            let failed = failing(shape, mode);
            if !failed.iter().any(|id| id == BOOKED) {
                survived.push(format!("{shape:?}/{mode:?}: {failed:?}"));
            }
        }
    }
    assert_eq!(survived, Vec::<String>::new(), "survived");
}

#[test]
fn issue_285_a_target_copying_the_wrong_outcome_fails_the_aggregate() {
    let mut survived = Vec::new();
    for shape in [Shape::TwoHops, Shape::OneHop] {
        for mode in [Mode::IgnoresAbsence, Mode::WrongHop, Mode::LatestInitiative] {
            let failed = failing(shape, mode);
            if !failed.iter().any(|id| id == AGGREGATE) {
                survived.push(format!("{shape:?}/{mode:?}: {failed:?}"));
            }
        }
    }
    assert_eq!(survived, Vec::<String>::new(), "survived");
}

/// The objective's initiative is `Optional`, but the only branch creating an objective fills it
/// from a required input: no run can leave it absent. The booking's scenario stands, and the
/// absence it cannot witness is refused by name under that scenario's id.
#[test]
fn issue_285_an_absence_no_run_can_arrange_is_refused_by_name() {
    let text = CHAINED.replace(
        "  - name: demo.costs.SetObjective\n    input:\n      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}",
        "  - name: demo.costs.SetObjective\n    input:\n      - {name: initiative_id, type: demo.costs.InitiativeId}",
    );
    assert_ne!(
        text, CHAINED,
        "the fixture holds SetObjective's Optional input"
    );
    let result = ess_conformance::synthesize::synthesize(&ir(&text));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == BOOKED),
        "the booking's scenario stands"
    );
    let named: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-020")
        .map(ToString::to_string)
        .collect();
    assert_eq!(named.len(), 1, "{named:#?}");
    let refusal = &named[0];
    assert!(refusal.contains(&format!("`{BOOKED}`")), "{refusal}");
    assert!(
        refusal.contains(
            "no arrangement leaves `demo.costs.Objective.initiative_id` absent: no branch \
             creating `demo.costs.Objective` fills `initiative_id` from an Optional input"
        ),
        "{refusal}"
    );
    assert!(!refusal.contains("has no scenario"), "{refusal}");
}
