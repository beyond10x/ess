//! A guard that says what an absent `Optional` compares as is decided for every record, and
//! synthesis sends the absence (`ess/29`, beyond10x/ess#620).
//!
//! `Classify` reads three optional inputs only through `coalesce(…)`: `stop_reason` and
//! `is_error` in its first two branches and `attempts` in its third. Before, an absent value made
//! the comparison `Unknown`: the interpreted model could not decide such a record, while a running
//! implementation decides it. Now the shared evaluator reads the absence as the call's other
//! argument, synthesis sends each of the three absent at least once, the interpreted target passes
//! every scenario, and a target that reads an absent value as some present one fails. The suite is
//! the same document for the Rust, Go and TypeScript runners, and each admits it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/coalesce-guard.yaml");
const COMMAND: &str = "demo.closing.Classify";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("closing.yaml"), raw)])
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

/// Every `Classify` the suite sends, as the input field names it carries with a value, and the
/// outcome the step after it requires.
fn sent(suite: &ConformanceSuite) -> Vec<(Vec<String>, String)> {
    let mut found: Vec<(Vec<String>, String)> = Vec::new();
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == COMMAND =>
                {
                    let present = input
                        .iter()
                        .filter(|(_, value)| {
                            !matches!(
                                value,
                                ess_conformance::ScenarioValue::Literal { value: Node::Null }
                            )
                        })
                        .map(|(name, _)| name.clone())
                        .collect();
                    found.push((present, String::new()));
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    if let Some(last) = found.last_mut() {
                        if last.1.is_empty() {
                            last.1 = outcome.to_string();
                        }
                    }
                }
                _ => {}
            }
        }
    }
    found
}

#[test]
fn coalesce_guard_absent_inputs_are_witnessed_and_decided() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let sent = sent(&synthesis.suite);
    for (field, outcome) in [
        ("stop_reason", None),
        ("is_error", None),
        ("attempts", Some("demo.closing.Classify/retry")),
    ] {
        let absent: Vec<&(Vec<String>, String)> = sent
            .iter()
            .filter(|(present, _)| !present.iter().any(|name| name == field))
            .collect();
        assert!(
            !absent.is_empty(),
            "some scenario sends `{field}` absent: {sent:#?}"
        );
        if let Some(outcome) = outcome {
            assert!(
                absent.iter().any(|(_, expected)| expected == outcome),
                "absent `{field}` selects `{outcome}`: {sent:#?}"
            );
        }
    }
    // The present sides are witnessed too: each branch is reached.
    for outcome in ["budget-stop", "completed", "retry", "failed"] {
        assert!(
            sent.iter()
                .any(|(_, expected)| expected == &format!("{COMMAND}/{outcome}")),
            "`{outcome}` is witnessed: {sent:#?}"
        );
    }
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    for (fault, fills) in [
        (
            "absent attempts read as three",
            (
                "attempts",
                Node::Number(ess_primitives::facts::Number::from(3_i64)),
            ),
        ),
        (
            "an absent verdict read as success",
            ("is_error", Node::Bool(false)),
        ),
    ] {
        let statuses = run(
            &synthesis.suite,
            &FillsAbsent {
                inner: Interpreted::for_model(model.clone()),
                field: fills.0,
                value: fills.1,
            },
        );
        assert!(
            !failing(&statuses).is_empty(),
            "{fault}: the faulty target passed {statuses:#?}"
        );
    }
    for (runner, emitted) in [
        (
            "go",
            ess_conformance::go::emit(&synthesis.suite).map(|files| files.len()),
        ),
        (
            "typescript",
            ess_conformance::ts::emit(&synthesis.suite).map(|files| files.len()),
        ),
    ] {
        assert!(
            emitted.is_ok(),
            "the {runner} runner admits the suite: {emitted:?}"
        );
    }
}

/// Below `ess/29` the structured form decides the same records the same way: the evaluator's
/// reading of `coalesce` is one rule in every format.
#[test]
fn coalesce_guard_structured_form_below_ess_29_is_decided_the_same_way() {
    let structured = MODEL
        .replacen("format: ess/29", "format: ess/28", 1)
        .replace(
            "when: coalesce(stop_reason, \"none\") == \"budget\"",
            "when: {compare: {left: {expr: 'coalesce(stop_reason, \"none\")'}, op: eq, right: budget}}",
        )
        .replace(
            "            - coalesce(is_error, true) == false",
            "            - {compare: {left: {expr: \"coalesce(is_error, true)\"}, op: eq, right: false}}",
        )
        .replace(
            "when: coalesce(attempts, 0) < 3",
            "when: {compare: {left: {expr: \"coalesce(attempts, 0)\"}, op: lt, right: 3}}",
        );
    let model = ir(&structured);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    assert_eq!(
        synthesis.suite.scenarios.len(),
        synthesize(&ir(MODEL)).suite.scenarios.len()
    );
}

const STORED: &str = "format: ess/29
system: demo
version: v1
domain: demo.job
types:
  - {name: demo.job.JobId, kind: newtype, of: Uuid}
errors:
  - {name: demo.job.Exhausted, summary: The job has no retries left., fields: []}
events:
  - name: demo.job.Opened
    fields: [{name: job, type: demo.job.JobId}]
  - name: demo.job.Retried
    fields: [{name: job, type: demo.job.JobId}]
entities:
  - name: demo.job.Job
    identity: {name: job, type: demo.job.JobId}
    fields:
      - {name: retries, type: Optional<Integer>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
commands:
  - name: demo.job.Open
    input:
      - {name: job, type: demo.job.JobId}
      - {name: retries, type: Optional<Integer>}
    outcomes:
      - name: opened
        creates: demo.job.Job
        instance: job
        sets: {retries: input.retries}
        emits: [demo.job.Opened]
        payload: {demo.job.Opened: {job: input.job}}
  - name: demo.job.Retry
    input:
      - {name: job, type: demo.job.JobId}
    outcomes:
      - name: exhausted
        when_subject: {predicate: \"coalesce(retries, 0) >= 3\"}
        error: demo.job.Exhausted
      - name: retried
        updates: demo.job.Job
        instance: job
        emits: [demo.job.Retried]
        payload: {demo.job.Retried: {job: input.job}}
";

/// Over a stored `Optional` (`when_subject:`), the row is arranged without the field for the
/// branch its absence selects, and the interpreted target decides it.
#[test]
fn coalesce_guard_over_a_stored_optional_is_decided_on_an_absent_row() {
    let model = ir(STORED);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let retried = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.job.Retry/outcome/retried")
        .map(|(_, scenario)| scenario)
        .expect("a `retried` scenario");
    let arranged_absent = retried.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
        if command.to_string() == "demo.job.Open"
            && input.get("retries").is_none_or(|value| matches!(
                value,
                ess_conformance::ScenarioValue::Literal { value: Node::Null }
            )))
    });
    assert!(
        arranged_absent,
        "`retried` is witnessed on a row without `retries`: {retried:#?}"
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model));
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
}

/// A target that reads one absent input as a present value: the default left out.
struct FillsAbsent {
    inner: Interpreted,
    field: &'static str,
    value: Node,
}

impl ConformanceTarget for FillsAbsent {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let absent = request
            .input
            .get(self.field)
            .is_none_or(|value| *value == Node::Null);
        if absent {
            request
                .input
                .insert(self.field.to_owned(), self.value.clone());
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(r)
    }
}
