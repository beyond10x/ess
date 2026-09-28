//! Adversary, story:defined-over-optional-aggregates (beyond10x/ess#176), pass 2: view filters.
//!
//! Pass 1's correction made `shows` mark a *settled* struct, list or map present at its own path.
//! It marks only that path: the settled value is not walked, so an `Optional` aggregate *inside*
//! the settled struct is never marked, and `defined()` — which is never `Unknown` — reads it as
//! absent. The Rust runner's `row_facts` walks the published row and marks it present. So the
//! suite synthesis writes expects no row where a target implementing the specification returns
//! one.
//!
//! Each fixture adds `detail: demo.queue.Detail` to the queue's `Metrics` (required inside the
//! `Optional` metrics, so `metrics.detail` is present exactly when `metrics` is), and one view whose
//! filter reads a path below `metrics`. The target implements the specification: its view returns
//! the rows the filter selects, evaluated over the row it holds.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

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
    AdmittedSuite, ConformanceSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The queue with a nested `detail` struct and a view `Filtered` selecting on `filter`.
fn fixture(filter: &str) -> String {
    let mut text = edit(
        QUEUE,
        "      - {name: waiting, type: Integer}\n",
        "      - {name: waiting, type: Integer}\n      - {name: detail, type: demo.queue.Detail}\n  - name: demo.queue.Detail\n    kind: struct\n    fields:\n      - {name: level, type: Integer}\n",
    );
    write!(
        text,
        "  - name: demo.queue.Filtered\n    source: demo.queue.Queue\n    consistency: read_your_writes\n    filter: '{filter}'\n    fields:\n      - {{name: queue_id, type: demo.queue.QueueId}}\n      - {{name: state, type: demo.queue.Queue.State}}\n      - {{name: metrics, type: Optional<demo.queue.Metrics>}}\n"
    )
    .expect("writing to a String cannot fail");
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

type Row = BTreeMap<String, Node>;

/// Whether `path` (dotted) is present in `row`: a non-null value at the end of it.
fn present(row: &Row, path: &str) -> bool {
    let mut segments = path.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    let mut at = row.get(first);
    for segment in segments {
        at = match at {
            Some(Node::Map(entries)) => entries.get(segment),
            _ => None,
        };
    }
    at.is_some_and(|value| value != &Node::Null)
}

/// A queue implementation that follows the specification; its `Filtered` view holds the rows where
/// `path` is present, and `negated` inverts that for a `missing()` filter.
struct Queues {
    path: &'static str,
    negated: bool,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Queues {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("pass2-queues", "1"))
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
        let find = |rows: &Vec<Row>, from: &str| -> Result<(usize, Node), Option<Node>> {
            let id = request.input.get("queue_id").cloned().unwrap_or(Node::Null);
            let index = rows.iter().position(|row| row["queue_id"] == id);
            match index {
                Some(index) if rows[index]["state"] == Node::Text(from.into()) => Ok((index, id)),
                Some(index) => Err(Some(rows[index]["state"].clone())),
                None => Err(None),
            }
        };
        let conflict = |state: Option<Node>| {
            let error = DeclaredErrorValue::new("demo.queue.QueueStateConflict".parse().unwrap());
            let error = match state {
                Some(state) => error.with("state", state),
                None => error,
            };
            SemanticCommandResult::took(outcome(&command, "wrong-state")).with_error(error)
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
                let (index, id) = match find(&rows, "Running") {
                    Ok(found) => found,
                    Err(state) => return Ok(conflict(state).with_consistency(token)),
                };
                rows[index].insert("state".into(), Node::Text("Paused".into()));
                rows[index].insert(
                    "metrics".into(),
                    request.input.get("metrics").cloned().unwrap_or(Node::Null),
                );
                SemanticCommandResult::took(outcome(&command, "paused")).emitting(
                    ObservedEvent::new("demo.queue.QueuePaused".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            "demo.queue.ResumeQueue" => {
                let (index, id) = match find(&rows, "Paused") {
                    Ok(found) => found,
                    Err(state) => return Ok(conflict(state).with_consistency(token)),
                };
                rows[index].insert("state".into(), Node::Text("Running".into()));
                rows[index].insert("metrics".into(), Node::Null);
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
        let rows = self.rows.borrow();
        match request.view.to_string().as_str() {
            "demo.queue.QueueById" => Ok(SemanticViewResult::of(rows.clone())),
            "demo.queue.Filtered" => Ok(SemanticViewResult::of(
                rows.iter()
                    .filter(|row| present(row, self.path) != self.negated)
                    .cloned()
                    .collect::<Vec<_>>(),
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

/// The scenarios a specification-following target fails, for the view filter `filter`.
fn failing(filter: &str, path: &'static str, negated: bool) -> BTreeSet<String> {
    let suite = suite(&fixture(filter));
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &Queues {
                path,
                negated,
                rows: RefCell::default(),
                minted: Cell::new(0),
            },
        )
        .into_report();
    let failed: BTreeSet<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .inspect(|scenario| eprintln!("{filter}: {scenario:#?}"))
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

/// Control: pass 1's correction, the top-level `Optional` struct itself.
#[test]
fn adversary_defined_pass2_a_view_filter_over_the_optional_struct_itself_passes() {
    assert_eq!(
        failing("defined(metrics)", "metrics", false),
        BTreeSet::new()
    );
}

/// The attack on the correction: an aggregate one level below the settled `Optional` struct.
#[test]
fn adversary_defined_pass2_a_view_filter_over_a_struct_inside_the_optional_struct_passes() {
    assert_eq!(
        failing("defined(metrics.detail)", "metrics.detail", false),
        BTreeSet::new()
    );
}

/// The same path through `missing()`: a paused queue holds `metrics.detail`, so the filter drops it.
#[test]
fn adversary_defined_pass2_a_missing_filter_over_a_struct_inside_the_optional_struct_passes() {
    assert_eq!(
        failing("missing(metrics.detail)", "metrics.detail", true),
        BTreeSet::new()
    );
}

/// The scalar leaf below the settled struct, which the same missing walk leaves unbound.
#[test]
fn adversary_defined_pass2_a_view_filter_over_a_scalar_inside_the_optional_struct_passes() {
    assert_eq!(
        failing("defined(metrics.waiting)", "metrics.waiting", false),
        BTreeSet::new()
    );
}
