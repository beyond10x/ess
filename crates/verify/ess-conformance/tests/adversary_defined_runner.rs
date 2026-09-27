//! Adversary, story:defined-over-optional-aggregates (beyond10x/ess#176), pass 1: the runner.
//!
//! The unit's own end-to-end fixture reads `defined()` over an `Optional` *struct* in one entity
//! invariant. This one widens it along every axis the acceptance names and the unit's suite does
//! not drive through synthesis and the Rust runner together:
//!
//! * the same invariant over an `Optional<List<String>>` (`tags`);
//! * a stored-field `when_subject: defined(metrics)` guard, arranged through `sets:` from an
//!   `Optional` input (the path the coordinator's `RowAndInput::present` patch exists for);
//! * a view `filter:` over the struct and over the list.
//!
//! A target that implements the specification must pass every synthesized scenario; each mutant
//! must fail the scenario that reads what it broke.
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

/// The widened queue.
fn widened() -> String {
    let mut text = QUEUE.to_owned();
    text = edit(
        &text,
        "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n      - {name: tags, type: Optional<List<String>>}\n    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n      - any: [state == Paused, {not: \"defined(tags)\"}]\n",
    );
    text = edit(
        &text,
        "      - {name: metrics, type: demo.queue.Metrics}\n",
        "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n      - {name: tags, type: Optional<List<String>>}\n",
    );
    text = edit(
        &text,
        "        sets: {metrics: input.metrics}\n",
        "        sets: {metrics: input.metrics, tags: input.tags}\n",
    );
    text = edit(
        &text,
        "        sets: {metrics: {cleared: true}}\n",
        "        sets: {metrics: {cleared: true}, tags: {cleared: true}}\n",
    );
    text = edit(
        &text,
        "errors:\n",
        "errors:\n  - name: demo.queue.QueueHoldsMetrics\n    summary: The queue still holds metrics.\n    fields: []\n",
    );
    text = edit(
        &text,
        "events:\n",
        "  - name: demo.queue.DrainQueue\n    input:\n      - {name: queue_id, type: demo.queue.QueueId}\n    outcomes:\n      - name: refused-holding\n        when_subject: {predicate: 'defined(metrics)'}\n        error: demo.queue.QueueHoldsMetrics\n        summary: A queue holding metrics is not drained.\n      - name: drained\n        moves: demo.queue.Queue.resume\n        instance: queue_id\n        sets: {tags: {cleared: true}}\n        emits: [demo.queue.QueueResumed]\n        payload: {demo.queue.QueueResumed: {queue_id: input.queue_id}}\n        summary: A paused queue without metrics runs again.\n      - {name: wrong-state, wrong_state: true, error: demo.queue.QueueStateConflict, summary: The queue is not paused.}\nevents:\n",
    );
    text = edit(
        &text,
        "may: [demo.queue.OpenQueue,",
        "may: [demo.queue.DrainQueue, demo.queue.OpenQueue,",
    );
    text = edit(
        &text,
        "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n",
        "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n",
    );
    text.push_str("      - {name: tags, type: Optional<List<String>>}\n");
    text.push_str(
        "  - name: demo.queue.HoldingQueues\n    source: demo.queue.Queue\n    consistency: read_your_writes\n    filter: 'defined(metrics)'\n    fields:\n      - {name: queue_id, type: demo.queue.QueueId}\n      - {name: state, type: demo.queue.Queue.State}\n      - {name: metrics, type: Optional<demo.queue.Metrics>}\n",
    );
    text.push_str(
        "  - name: demo.queue.TaggedQueues\n    source: demo.queue.Queue\n    consistency: read_your_writes\n    filter: 'defined(tags)'\n    fields:\n      - {name: queue_id, type: demo.queue.QueueId}\n      - {name: state, type: demo.queue.Queue.State}\n      - {name: tags, type: Optional<List<String>>}\n",
    );
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    let text = widened();
    let synthesis = synthesize(&ir(&text));
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario of the widened queue is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// `drain` ignores the stored metrics and drains every paused queue.
    DrainIgnoresMetrics,
    /// `resume` clears the metrics and keeps the tags.
    KeepsTagsOnResume,
    /// Both filtered views return every queue.
    FiltersIgnored,
}

type Row = BTreeMap<String, Node>;

struct Queues {
    mutant: Mutant,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn held(row: &Row, field: &str) -> bool {
    row.get(field).is_some_and(|value| value != &Node::Null)
}

fn project(row: &Row, fields: &[&str]) -> Row {
    fields
        .iter()
        .filter_map(|field| {
            row.get(*field)
                .map(|value| ((*field).to_owned(), value.clone()))
        })
        .collect()
}

impl ConformanceTarget for Queues {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("widened-queues", "1"))
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
        let conflict = |state: Option<Node>| {
            let error = DeclaredErrorValue::new("demo.queue.QueueStateConflict".parse().unwrap());
            let error = match state {
                Some(state) => error.with("state", state),
                None => error,
            };
            SemanticCommandResult::took(outcome(&command, "wrong-state")).with_error(error)
        };
        let find = |rows: &Vec<Row>, from: &str| -> Result<(usize, Node), Option<Node>> {
            let id = request.input.get("queue_id").cloned().unwrap_or(Node::Null);
            let index = rows.iter().position(|row| row["queue_id"] == id);
            match index {
                Some(index) if rows[index]["state"] == Node::Text(from.into()) => Ok((index, id)),
                Some(index) => Err(Some(rows[index]["state"].clone())),
                None => Err(None),
            }
        };
        let input = |field: &str| request.input.get(field).cloned().unwrap_or(Node::Null);
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
                rows[index].insert("metrics".into(), input("metrics"));
                rows[index].insert("tags".into(), input("tags"));
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
                if self.mutant != Mutant::KeepsTagsOnResume {
                    rows[index].insert("tags".into(), Node::Null);
                }
                SemanticCommandResult::took(outcome(&command, "resumed")).emitting(
                    ObservedEvent::new("demo.queue.QueueResumed".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            "demo.queue.DrainQueue" => {
                let (index, id) = match find(&rows, "Paused") {
                    Ok(found) => found,
                    Err(state) => return Ok(conflict(state).with_consistency(token)),
                };
                if held(&rows[index], "metrics") && self.mutant != Mutant::DrainIgnoresMetrics {
                    return Ok(
                        SemanticCommandResult::took(outcome(&command, "refused-holding"))
                            .with_error(DeclaredErrorValue::new(
                                "demo.queue.QueueHoldsMetrics".parse().unwrap(),
                            ))
                            .with_consistency(token),
                    );
                }
                rows[index].insert("state".into(), Node::Text("Running".into()));
                rows[index].insert("tags".into(), Node::Null);
                SemanticCommandResult::took(outcome(&command, "drained")).emitting(
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
        let filtered = |field: &str, fields: &[&str]| -> Vec<Row> {
            rows.iter()
                .filter(|row| self.mutant == Mutant::FiltersIgnored || held(row, field))
                .map(|row| project(row, fields))
                .collect()
        };
        match request.view.to_string().as_str() {
            "demo.queue.QueueById" => Ok(SemanticViewResult::of(
                rows.iter()
                    .map(|row| project(row, &["queue_id", "state", "metrics", "tags"]))
                    .collect::<Vec<_>>(),
            )),
            "demo.queue.HoldingQueues" => Ok(SemanticViewResult::of(filtered(
                "metrics",
                &["queue_id", "state", "metrics"],
            ))),
            "demo.queue.TaggedQueues" => Ok(SemanticViewResult::of(filtered(
                "tags",
                &["queue_id", "state", "tags"],
            ))),
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

fn failing(mutant: Mutant) -> BTreeSet<String> {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &Queues {
                mutant,
                rows: RefCell::default(),
                minted: Cell::new(0),
            },
        )
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
fn adversary_defined_the_widened_queue_synthesizes_a_when_subject_scenario_for_each_branch() {
    let ids: BTreeSet<String> = suite().scenarios.keys().map(ToString::to_string).collect();
    for id in [
        "demo.queue.DrainQueue/outcome/refused-holding",
        "demo.queue.DrainQueue/outcome/drained",
    ] {
        assert!(ids.contains(id), "no scenario {id}: {ids:#?}");
    }
    assert!(
        ids.iter()
            .any(|id| id.contains("/invariant/") && id.contains("ResumeQueue/resumed")),
        "{ids:#?}"
    );
}

#[test]
fn adversary_defined_the_widened_queue_as_specified_passes_its_own_suite() {
    assert_eq!(failing(Mutant::None), BTreeSet::new());
}

#[test]
fn adversary_defined_a_drain_that_ignores_stored_metrics_fails_the_refusal_scenario() {
    let failed = failing(Mutant::DrainIgnoresMetrics);
    assert!(
        failed.contains("demo.queue.DrainQueue/outcome/refused-holding"),
        "{failed:#?}"
    );
}

#[test]
fn adversary_defined_a_resume_that_keeps_its_tags_fails_the_list_invariant() {
    let failed = failing(Mutant::KeepsTagsOnResume);
    assert!(
        failed
            .iter()
            .any(|id| id.contains("/invariant/") && id.contains("ResumeQueue/resumed")),
        "{failed:#?}"
    );
}

#[test]
fn adversary_defined_views_that_ignore_their_presence_filter_fail_a_scenario() {
    assert_ne!(failing(Mutant::FiltersIgnored), BTreeSet::new());
}
