//! `defined(x)` over an `Optional` struct, list or map, read the same way by every evaluator lane
//! (beyond10x/ess#176, source format `ess/16`).
//!
//! Presence is a property of the `Optional`, not of what it holds: a present struct, list or map is
//! present even when it is empty, and `null` or a left-out member is absent. Before `ess/16` every
//! lane flattened a value into its scalar leaves only, so a present struct had no fact at its own
//! path and `defined(metrics)` read `false` for a queue that held metrics.
//!
//! The #176 invariant — metrics only while paused — is synthesized into a check after every
//! state-changing branch, and a queue that resumes without clearing its metrics fails the check
//! after `resumed` and nothing else.
mod support_scratch;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    synthesize::synthesize,
    target::*,
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{
    facts::{FactSource, FactValue},
    node::Node,
    predicate::{Predicate, Truth},
};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");
const AFTER_RESUMED: &str = "demo.queue.Queue/invariant/after/demo.queue.ResumeQueue/resumed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    suite_of(QUEUE)
}

/// The fixture with `resumed` no longer saying it clears the metrics: the outcome #176 is about, which
/// leaves `Paused` and forgets the field, so that only the invariant says it must be gone.
fn forgetful() -> String {
    let text = QUEUE.replace("        sets: {metrics: {cleared: true}}\n", "");
    assert_ne!(text, QUEUE);
    text
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "every invariant check is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn predicate(text: &str) -> Predicate {
    Predicate::parse_expression(text).unwrap_or_else(|error| panic!("`{text}`: {error}"))
}

/// One way a queue implementation could treat its metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// `resume` moves the queue and leaves the metrics it was paused with.
    KeepsMetricsOnResume,
}

type Row = BTreeMap<String, Node>;

struct Queues {
    mutant: Mutant,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Queues {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Queues {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("queues-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
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
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        // The row a command acts on where it is in `from`, and otherwise the declared conflict —
        // naming the state it is in, where there is a row at all.
        let find = |rows: &mut Vec<Row>, from: &str| {
            let id = request.input["queue_id"].clone();
            let index = rows.iter().position(|row| row["queue_id"] == id);
            let state = index.map(|index| rows[index]["state"].clone());
            match (index, state) {
                (Some(index), Some(state)) if state == Node::Text(from.into()) => Ok((index, id)),
                (_, state) => {
                    let error =
                        DeclaredErrorValue::new("demo.queue.QueueStateConflict".parse().unwrap());
                    let error = match state {
                        Some(state) => error.with("state", state),
                        None => error,
                    };
                    Err(Box::new(
                        SemanticCommandResult::took(outcome(&command, "wrong-state"))
                            .with_error(error),
                    ))
                }
            }
        };
        let result = match command.to_string().as_str() {
            "demo.queue.OpenQueue" => {
                let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
                let mut row = Row::new();
                row.insert("queue_id".into(), id.clone());
                row.insert("state".into(), Node::Text("Running".into()));
                rows.push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.queue.QueueOpened".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            "demo.queue.PauseQueue" => {
                let (index, id) = match find(&mut rows, "Running") {
                    Ok(found) => found,
                    Err(refused) => return Ok(refused.with_consistency(token)),
                };
                rows[index].insert("state".into(), Node::Text("Paused".into()));
                rows[index].insert("metrics".into(), request.input["metrics"].clone());
                SemanticCommandResult::took(outcome(&command, "paused")).emitting(
                    ObservedEvent::new("demo.queue.QueuePaused".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            "demo.queue.ResumeQueue" => {
                let (index, id) = match find(&mut rows, "Paused") {
                    Ok(found) => found,
                    Err(refused) => return Ok(refused.with_consistency(token)),
                };
                rows[index].insert("state".into(), Node::Text("Running".into()));
                if self.mutant != Mutant::KeepsMetricsOnResume {
                    rows[index].insert("metrics".into(), Node::Null);
                }
                SemanticCommandResult::took(outcome(&command, "resumed")).emitting(
                    ObservedEvent::new("demo.queue.QueueResumed".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        match request.view.to_string().as_str() {
            "demo.queue.QueueById" => Ok(SemanticViewResult::of(self.rows.borrow().clone())),
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

/// The scenarios that do not pass against this target.
fn failing(mutant: Mutant) -> BTreeSet<String> {
    failing_against(&suite(), mutant)
}

fn failing_against(suite: &ConformanceSuite, mutant: Mutant) -> BTreeSet<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &Queues::new(mutant))
        .into_report();
    let failed: BTreeSet<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .inspect(|scenario| eprintln!("{mutant:?}: {scenario:#?}"))
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

#[test]
fn issue_176_the_invariant_is_checked_after_every_state_changing_branch() {
    let ids: BTreeSet<String> = suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.contains("/invariant/"))
        .collect();
    assert!(ids.contains(AFTER_RESUMED), "{ids:?}");
}

#[test]
fn issue_176_the_queue_as_specified_passes_its_own_suite() {
    assert_eq!(failing(Mutant::None), BTreeSet::new());
}

#[test]
fn issue_176_a_queue_that_resumes_without_clearing_its_metrics_fails_the_check_after_resumed() {
    // The branch says `{cleared: true}` as well, so its own row checks catch the same queue.
    assert_eq!(
        failing(Mutant::KeepsMetricsOnResume),
        BTreeSet::from([
            AFTER_RESUMED.to_owned(),
            "demo.queue.Queue/transition/resume/by/demo.queue.ResumeQueue/resumed".to_owned(),
            "demo.queue.ResumeQueue/outcome/resumed".to_owned(),
        ])
    );
}

/// #176 itself: an outcome that leaves `Paused` and does not say it clears the metrics. Only the
/// invariant says they must be gone, and the check synthesized from it is the one that fails.
#[test]
fn issue_176_an_outcome_that_forgets_to_clear_the_metrics_is_refuted_by_the_invariant_alone() {
    let suite = suite_of(&forgetful());
    assert_eq!(
        failing_against(&suite, Mutant::KeepsMetricsOnResume),
        BTreeSet::from([AFTER_RESUMED.to_owned()])
    );
}

/// A guard over an `Optional` struct in a command's input: synthesis witnesses both branches, the
/// guarded one with the struct sent and the other with it left out (`witness.rs` `omissions`).
#[test]
fn a_guard_over_an_optional_struct_input_is_witnessed_on_both_sides() {
    let text = QUEUE
        .replace(
            "commands:\n",
            "commands:\n  - name: demo.queue.Probe\n    input:\n      - {name: metrics, type: Optional<demo.queue.Metrics>}\n    outcomes:\n      - name: probed\n        when: defined(metrics)\n        emits: [demo.queue.QueueOpened]\n        payload: {demo.queue.QueueOpened: {queue_id: {generated: true}}}\n      - name: declined\n        emits: [demo.queue.QueueResumed]\n        payload: {demo.queue.QueueResumed: {queue_id: {generated: true}}}\n",
        )
        .replace(
            "may: [demo.queue.OpenQueue,",
            "may: [demo.queue.Probe, demo.queue.OpenQueue,",
        );
    let suite = suite_of(&text);
    let sent = |outcome: &str| -> bool {
        let id = format!("demo.queue.Probe/outcome/{outcome}");
        let scenario = suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == id)
            .unwrap_or_else(|| panic!("no scenario {id}"))
            .1;
        scenario
            .steps
            .iter()
            .find_map(|step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => Some(
                    input
                        .get("metrics")
                        .is_some_and(|value| value != &ScenarioValue::literal(Node::Null)),
                ),
                _ => None,
            })
            .expect("the scenario executes the command")
    };
    assert!(sent("probed"), "`probed` needs the metrics sent");
    assert!(!sent("declined"), "`declined` needs the metrics left out");
}

/// The command-input lane: a present struct, list or map is present even when it is empty.
#[test]
fn command_input_reads_a_present_aggregate_as_defined_and_null_or_omitted_as_absent() {
    let text = QUEUE
        .replace(
            "      - {name: waiting, type: Integer}\n",
            "      - {name: waiting, type: Integer}\n  - name: demo.queue.Notes\n    kind: struct\n    fields:\n      - {name: text, type: Optional<String>}\n",
        )
        .replace(
            "  - name: demo.queue.ResumeQueue\n    input:\n",
            "  - name: demo.queue.ResumeQueue\n    input:\n      - {name: notes, type: Optional<demo.queue.Notes>}\n      - {name: tags, type: Optional<List<String>>}\n      - {name: counts, type: 'Optional<Map<String, Integer>>'}\n",
        );
    let ir = ir(&text);
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.queue.ResumeQueue")
        .expect("declared");
    let id = Node::Text("00000000-0000-4000-8000-000000000001".into());
    let cases: [(&str, Node, Truth); 7] = [
        ("notes", Node::Map(BTreeMap::new()), Truth::True),
        (
            "notes",
            Node::Map(BTreeMap::from([("text".into(), Node::Text("x".into()))])),
            Truth::True,
        ),
        ("notes", Node::Null, Truth::False),
        ("tags", Node::Seq(Vec::new()), Truth::True),
        ("tags", Node::Null, Truth::False),
        ("counts", Node::Map(BTreeMap::new()), Truth::True),
        ("counts", Node::Null, Truth::False),
    ];
    for (field, value, expected) in cases {
        let candidate = BTreeMap::from([
            ("queue_id".to_owned(), id.clone()),
            (field.to_owned(), value.clone()),
        ]);
        let facts = ess_conformance::flatten(&ir, command, &candidate)
            .unwrap_or_else(|errors| panic!("{errors:?}"));
        let read = predicate(&format!("defined({field})")).evaluate(&facts);
        assert_eq!(read, expected, "defined({field}) over {value:?}");
    }
    let facts =
        ess_conformance::flatten(&ir, command, &BTreeMap::from([("queue_id".to_owned(), id)]))
            .unwrap();
    for field in ["notes", "tags", "counts"] {
        assert_eq!(
            predicate(&format!("defined({field})")).evaluate(&facts),
            Truth::False,
            "an omitted {field}"
        );
        assert_eq!(facts.fact(&field.parse().unwrap()), None::<FactValue>);
    }
}

/// The Go lane: the emitted runtime's `facts` and `defined` answer the vectors the Rust lanes do.
#[test]
fn the_go_runtime_reads_a_present_aggregate_as_defined() {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let directory = support_scratch::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-defined-aggregates-{}", std::process::id())),
    );
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    for artifact in ess_conformance::go::emit(admitted.suite()).unwrap() {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/definedaggregates\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/defined_aggregates_test.go"),
        include_str!("fixtures/defined-over-optional-aggregates.go"),
    )
    .unwrap();
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestDefinedAggregate",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&result.stdout);
    eprintln!("{stdout}\n{}", String::from_utf8_lossy(&result.stderr));
    std::fs::remove_dir_all(&directory).ok();
    assert!(result.status.success(), "Go lane: {}", result.status);
    assert!(
        stdout.contains("--- PASS: TestDefinedAggregate"),
        "the Go case did not run"
    );
}
