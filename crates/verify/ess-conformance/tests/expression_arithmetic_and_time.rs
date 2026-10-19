//! Expressions compute arithmetic and time in `sets:`, `payload:` and a computed view field
//! (`ess/27`, `story:expression-arithmetic-and-time`, beyond10x/ess#560,
//! `docs/design/generic-language-features.md` § G1).
//!
//! `RecordPastIncident` stores and publishes `ended_at - started_at` as a `Duration` and
//! `coalesce(round(affected / total, 2, half_even), 0)` as a `Decimal`, and the `Incidents` view
//! computes `elapsed` from the row. Synthesis computes every expected value forwards with the shared
//! evaluator, and runs the branch again on inputs that make the figure worth checking: the instants
//! apart, a zero denominator, and a rounding tie either way. The interpreted target passes every
//! scenario; a target computing any figure otherwise fails one.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    ConformanceSuite, Runner, ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{node::Node, time::Rfc3339Instant};

const MODEL: &str = include_str!("fixtures/expression-arithmetic-and-time.yaml");
const RECORDED: &str = "demo.ledger.RecordPastIncident/outcome/recorded";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            let passed = matches!(run.status, Status::Passed);
            (
                run.scenario.to_string(),
                if passed {
                    "passed".to_owned()
                } else {
                    format!("{:?} {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failing(statuses: &BTreeMap<String, String>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.as_str())
        .collect()
}

/// One invocation of `RecordPastIncident` in the `recorded` scenario: what it sent, and what the
/// event it is expected to publish carries.
#[derive(Debug)]
struct Invocation {
    input: BTreeMap<String, Node>,
    published: BTreeMap<String, Node>,
}

fn invocations(suite: &ConformanceSuite) -> Vec<Invocation> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == RECORDED)
        .map_or_else(
            || panic!("no `{RECORDED}` scenario"),
            |(_, scenario)| scenario,
        );
    let mut found: Vec<Invocation> = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.ledger.RecordPastIncident" =>
            {
                found.push(Invocation {
                    input: input
                        .iter()
                        .filter_map(|(name, value)| match value {
                            ScenarioValue::Literal { value } => Some((name.clone(), value.clone())),
                            _ => None,
                        })
                        .collect(),
                    published: BTreeMap::new(),
                });
            }
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.ledger.IncidentRecorded" =>
            {
                if let Some(last) = found.last_mut() {
                    last.published = payload.clone();
                }
            }
            _ => {}
        }
    }
    found
}

fn instant(node: &Node) -> Rfc3339Instant {
    let Node::Text(text) = node else {
        panic!("an instant is text: {node:?}")
    };
    Rfc3339Instant::parse_rfc3339(text).unwrap_or_else(|| panic!("{text} is an instant"))
}

fn integer(node: &Node) -> i64 {
    let Node::Number(number) = node else {
        panic!("an integer is a number: {node:?}")
    };
    number.as_i64().expect("a whole number")
}

/// `affected / total` at two places, half-even, or zero where `total` is zero: the figure the
/// specification states, computed here without the shared evaluator.
fn share(affected: i64, total: i64, half_up: bool) -> String {
    if total == 0 {
        return "0".to_owned();
    }
    let (numerator, denominator) = (i128::from(affected) * 100, i128::from(total));
    let (quotient, remainder) = (numerator / denominator, numerator % denominator);
    let twice = (remainder * 2).abs();
    let up = match twice.cmp(&denominator.abs()) {
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Equal => half_up || quotient % 2 != 0,
    };
    let rounded = if up { quotient + 1 } else { quotient };
    let text = format!("{}.{:02}", rounded / 100, rounded % 100);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn decimal_text(node: &Node) -> String {
    let Node::Number(number) = node else {
        panic!("a decimal is a number: {node:?}")
    };
    number.exact_text()
}

/// The issue's shapes validate; every scenario asserts the computed `Duration` in the event, in the
/// stored row (through the view) and in the computed view field; one invocation sends two instants
/// apart, so the figure checked is not zero; and the interpreted target passes.
#[test]
fn timestamp_difference_is_a_duration_in_sets_payload_and_view_field() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let sent = invocations(&synthesis.suite);
    assert!(!sent.is_empty(), "the branch is invoked");
    let mut apart = false;
    for invocation in &sent {
        let elapsed = instant(&invocation.input["ended_at"])
            .whole_seconds_since(instant(&invocation.input["started_at"]))
            .expect("whole seconds apart");
        apart |= elapsed > 0;
        if elapsed >= 0 {
            assert_eq!(
                invocation.published.get("duration"),
                Some(&Node::Text(format!("PT{elapsed}S"))),
                "{invocation:#?}"
            );
        }
    }
    assert!(apart, "one invocation sends the instants apart: {sent:#?}");

    let suite_text = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| panic!("{error}"));
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == RECORDED)
        .map(|(_, scenario)| scenario)
        .expect("the recorded scenario");
    let computed_rows = scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step,
                ScenarioStep::ExpectView { expectation: ViewExpectation::Contains { fields }, .. }
                | ScenarioStep::EventuallyView { expectation: ViewExpectation::Contains { fields }, .. }
                    if fields.contains_key("elapsed") && fields.contains_key("duration"))
        })
        .count();
    assert!(
        computed_rows > 0,
        "the view's computed `elapsed` and stored `duration` are asserted:\n{suite_text}"
    );

    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(statuses.contains_key(RECORDED), "{statuses:#?}");
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");

    let faulty = run(
        &synthesis.suite,
        &Rewrites(Interpreted::for_model(model), Rewrite::DurationZero),
    );
    assert!(
        failing(&faulty).contains(&RECORDED),
        "a target publishing a zero duration fails {RECORDED}: {faulty:#?}"
    );
}

/// The quotient is rounded as declared and is zero where the denominator is: synthesis sends a zero
/// denominator and a tie at the declared digits, the expected figure follows the declared mode,
/// and the interpreted target passes. The same model declaring `half_up` expects the other
/// neighbour on the tie.
#[test]
fn decimal_division_rounds_as_declared() {
    for (written, half_up) in [("half_even", false), ("half_up", true)] {
        let text = MODEL.replace("half_even", written);
        let model = ir(&text);
        let synthesis = synthesize(&model);
        assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
        let sent = invocations(&synthesis.suite);
        let (mut zero, mut tie_even, mut tie_odd) = (false, false, false);
        for invocation in &sent {
            let (affected, total) = (
                integer(&invocation.input["affected"]),
                integer(&invocation.input["total"]),
            );
            let expected = share(affected, total, half_up);
            assert_eq!(
                invocation.published.get("share").map(decimal_text),
                Some(expected.clone()),
                "{written}: {invocation:#?}"
            );
            zero |= total == 0;
            // A tie at two places: the third place is a five with nothing after it.
            if total != 0 && (i128::from(affected) * 1000) % i128::from(total) == 0 {
                let thousandths = i128::from(affected) * 1000 / i128::from(total);
                if thousandths % 10 == 5 {
                    if (thousandths / 10) % 2 == 0 {
                        tie_even = true;
                    } else {
                        tie_odd = true;
                    }
                }
            }
        }
        assert!(zero, "{written}: a zero denominator is sent: {sent:#?}");
        assert!(
            tie_even && tie_odd,
            "{written}: a tie on an even and on an odd neighbour is sent: {sent:#?}"
        );
        let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
        assert!(failing(&statuses).is_empty(), "{written}: {statuses:#?}");
        let other = if half_up {
            Rewrite::RoundHalfEven
        } else {
            Rewrite::RoundHalfUp
        };
        let faulty = run(
            &synthesis.suite,
            &Rewrites(Interpreted::for_model(model), other),
        );
        assert!(
            failing(&faulty).contains(&RECORDED),
            "{written}: a target rounding the other way fails {RECORDED}: {faulty:#?}"
        );
    }
}

/// What [`Rewrites`] changes in what the target publishes.
#[derive(Clone, Copy)]
enum Rewrite {
    /// Every `duration` it publishes is `PT0S`.
    DurationZero,
    /// Its `share` is the quotient rounded half-up.
    RoundHalfUp,
    /// Its `share` is the quotient rounded half-even.
    RoundHalfEven,
}

/// A target computing one figure otherwise than the specification states.
struct Rewrites(Interpreted, Rewrite);

impl Rewrites {
    fn rewrite(&self, input: &BTreeMap<String, Node>, fields: &mut BTreeMap<String, Node>) {
        match self.1 {
            Rewrite::DurationZero => {
                if fields.contains_key("duration") {
                    fields.insert("duration".to_owned(), Node::Text("PT0S".to_owned()));
                }
            }
            Rewrite::RoundHalfUp | Rewrite::RoundHalfEven => {
                let (Some(affected), Some(total)) = (input.get("affected"), input.get("total"))
                else {
                    return;
                };
                if fields.contains_key("share") {
                    let figure = share(
                        integer(affected),
                        integer(total),
                        matches!(self.1, Rewrite::RoundHalfUp),
                    );
                    let number =
                        ess_primitives::facts::Number::decimal_literal(&figure).expect("a decimal");
                    fields.insert("share".to_owned(), Node::Number(number));
                }
            }
        }
    }
}

thread_local! {
    static LAST_INPUT: std::cell::RefCell<BTreeMap<String, Node>> =
        const { std::cell::RefCell::new(BTreeMap::new()) };
}

impl ConformanceTarget for Rewrites {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.0.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        LAST_INPUT.with(|last| *last.borrow_mut() = request.input.clone());
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.0.query_view(request)?;
        let input = LAST_INPUT.with(|last| last.borrow().clone());
        for row in &mut result.rows {
            self.rewrite(&input, row);
        }
        Ok(result)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.0.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let mut events = self.0.observe_events(r)?;
        let input = LAST_INPUT.with(|last| last.borrow().clone());
        for event in &mut events {
            self.rewrite(&input, &mut event.payload);
        }
        Ok(events)
    }
}
