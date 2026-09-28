//! The generated Go runtime reads the change in an ungrouped aggregate view (`snapshot_view` +
//! `changed_by`, suite/26, beyond10x/ess#148) as the reference runner does (beyond10x/ess#188).
//!
//! The target is `tests/adversary_aggdelta_pass1.rs`'s, with every way it gets a view wrong, on an
//! empty target and on one another user already wrote to, and with an `eventual` view that catches
//! up late. Each is recorded once and replayed to the Go runtime.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Synthesis},
    target::*,
    ConformanceSuite,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

const METRICS: &str = include_str!("fixtures/aggregate-views.yaml");
const EVERYTHING: &str = "metrics.session.Everything";
const COMPLETED: &str = "metrics.session.CompletedTotals";

const VIEWS: &str = "  - name: metrics.session.Everything
    source: metrics.session.Session
    consistency: read_your_writes
    fields:
      - {name: sessions, type: Integer, aggregate: {count: {}}}
      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}
      - {name: mean_talk, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}
      - {name: distinct_callers, type: Integer, aggregate: {count_distinct: caller}}
  - name: metrics.session.CompletedTotals
    source: metrics.session.Session
    consistency: eventual
    filter: state == Completed
    fields:
      - {name: sessions, type: Integer, aggregate: {count: {}}}
      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}
";

fn spec() -> String {
    let head = METRICS.split_once("views:\n").unwrap().0;
    format!("{head}views:\n{VIEWS}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("metrics.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis() -> Synthesis {
    synthesize(&ir(&spec()))
}

fn n(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn int(node: &Node) -> i128 {
    match node {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("not a number: {other:?}"),
    }
}

/// The aggregate scenarios only, admitted.
fn aggregate_suite() -> ConformanceSuite {
    let mut suite = synthesis().suite;
    suite
        .scenarios
        .retain(|id, _| id.to_string().ends_with("/aggregate"));
    suite
}

/// One way of getting a view wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// `CompletedTotals` counts and sums every session, whatever its state.
    IgnoresFilter,
    /// Over no row, `count` is reported absent instead of `0` (the page's table: `0`).
    CountAbsentWhenEmpty,
    /// Over no row, a required `sum` is reported absent instead of `0` (the page's table: `0`).
    SumAbsentWhenEmpty,
    /// Every `sum` is doubled.
    DoublesSum,
    /// `talk_seconds` is one too many whenever any row is admitted; `sessions` stays right.
    SumOffByOne,
    /// `count` counts one row too many whenever any row is admitted.
    CountOffByOne,
}

/// What the target holds before a scenario starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seed {
    Empty,
    /// Another user's sessions: one `Completed` with 40 seconds, one `Open` with 7.
    Held,
}

type Row = BTreeMap<String, Node>;

struct Metrics {
    mutant: Mutant,
    seed: Seed,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn other(id: &str, state: &str, talk: &str) -> Row {
    let mut row = Row::new();
    row.insert("session_id".into(), Node::Text(id.to_owned()));
    row.insert("queue_id".into(), Node::Text("someone else".into()));
    row.insert("agent_id".into(), Node::Text("someone else".into()));
    row.insert("caller".into(), Node::Text("someone else".into()));
    row.insert("channel".into(), Node::Text("Voice".into()));
    row.insert("talk_seconds".into(), n(talk));
    row.insert("wait_seconds".into(), n("1"));
    row.insert("state".into(), Node::Text(state.to_owned()));
    row
}

impl Metrics {
    fn new(mutant: Mutant, seed: Seed) -> Self {
        Self {
            mutant,
            seed,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn view(&self, view: &str) -> Vec<Row> {
        let rows = self.rows.borrow().clone();
        self.view_over(view, &rows)
    }

    fn view_over(&self, view: &str, rows: &[Row]) -> Vec<Row> {
        let admitted: Vec<Row> = rows
            .iter()
            .filter(|row| {
                view != COMPLETED
                    || self.mutant == Mutant::IgnoresFilter
                    || row.get("state") == Some(&Node::Text("Completed".into()))
            })
            .cloned()
            .collect();
        let talk: Vec<i128> = admitted
            .iter()
            .map(|row| int(row.get("talk_seconds").expect("talk_seconds")))
            .collect();
        let mut out = Row::new();
        let count = admitted.len()
            + usize::from(self.mutant == Mutant::CountOffByOne && !admitted.is_empty());
        out.insert(
            "sessions".into(),
            if admitted.is_empty() && self.mutant == Mutant::CountAbsentWhenEmpty {
                Node::Null
            } else {
                n(&count.to_string())
            },
        );
        let mut sum: i128 = talk.iter().sum();
        if self.mutant == Mutant::DoublesSum {
            sum *= 2;
        }
        if self.mutant == Mutant::SumOffByOne && !admitted.is_empty() {
            sum += 1;
        }
        out.insert(
            "talk_seconds".into(),
            if admitted.is_empty() && self.mutant == Mutant::SumAbsentWhenEmpty {
                Node::Null
            } else {
                n(&sum.to_string())
            },
        );
        if view == EVERYTHING {
            out.insert(
                "mean_talk".into(),
                if talk.is_empty() {
                    Node::Null
                } else {
                    // Not asserted by a change; any number will do.
                    n("1")
                },
            );
            let callers: BTreeSet<String> = admitted
                .iter()
                .map(|row| format!("{:?}", row.get("caller")))
                .collect();
            out.insert("distinct_callers".into(), n(&callers.len().to_string()));
        }
        vec![out]
    }
}

impl ConformanceTarget for Metrics {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("metrics-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(match self.seed {
            Seed::Empty => Vec::new(),
            Seed::Held => vec![
                other("00000000-0000-4000-8000-900000000001", "Completed", "40"),
                other("00000000-0000-4000-8000-900000000002", "Open", "7"),
            ],
        });
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
        match command.to_string().as_str() {
            "metrics.session.Record" => {
                let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
                let mut row: Row = request.input.clone();
                row.insert("session_id".into(), Node::Text(id.clone()));
                row.insert("state".into(), Node::Text("Open".into()));
                self.rows.borrow_mut().push(row);
                Ok(SemanticCommandResult::took(OutcomeRef::new(
                    command.clone(),
                    OutcomeName::new("recorded").unwrap(),
                ))
                .emitting(
                    ObservedEvent::new("metrics.session.Recorded".parse().unwrap())
                        .with("session_id", Node::Text(id)),
                )
                .with_consistency(token))
            }
            "metrics.session.Complete" => {
                let id = request.input.get("session_id").cloned().expect("an id");
                let mut rows = self.rows.borrow_mut();
                let row = rows
                    .iter_mut()
                    .find(|row| row.get("session_id") == Some(&id))
                    .expect("a session this scenario recorded");
                row.insert("state".into(), Node::Text("Completed".into()));
                Ok(SemanticCommandResult::took(OutcomeRef::new(
                    command.clone(),
                    OutcomeName::new("completed").unwrap(),
                ))
                .emitting(
                    ObservedEvent::new("metrics.session.Completed".parse().unwrap())
                        .with("session_id", id),
                )
                .with_consistency(token))
            }
            other => Err(TargetError::unsupported("command", other.to_owned())),
        }
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.view(&request.view.to_string())))
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

/// A correct target whose `eventual` view lags: after every command, the next two reads of
/// `CompletedTotals` still answer over the rows as they stood before it. Rows held before the
/// scenario began are settled, as the design page requires.
struct Lagging {
    inner: Metrics,
    pending: Cell<u32>,
    projected: RefCell<Vec<Row>>,
}

impl ConformanceTarget for Lagging {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)?;
        self.projected.replace(self.inner.rows.borrow().clone());
        self.pending.set(0);
        Ok(())
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.pending.set(2);
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let view = request.view.to_string();
        if view == COMPLETED && self.pending.get() > 0 {
            self.pending.set(self.pending.get() - 1);
            let stale = self.projected.borrow().clone();
            return Ok(SemanticViewResult::of(self.inner.view_over(&view, &stale)));
        }
        self.projected.replace(self.inner.rows.borrow().clone());
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

#[test]
fn go_gives_the_reference_verdict_for_every_change_mutant_on_both_seeds() {
    let suite = aggregate_suite();
    assert!(ess_conformance::aggregate_delta::used_by(&suite));
    for seed in [Seed::Empty, Seed::Held] {
        for mutant in [
            Mutant::None,
            Mutant::IgnoresFilter,
            Mutant::CountAbsentWhenEmpty,
            Mutant::SumAbsentWhenEmpty,
            Mutant::DoublesSum,
            Mutant::SumOffByOne,
            Mutant::CountOffByOne,
        ] {
            let verdicts = support_go::assert_parity(
                &format!("delta-{mutant:?}-{seed:?}").to_lowercase(),
                &suite,
                Metrics::new(mutant, seed),
            );
            if mutant == Mutant::None {
                assert!(
                    support_go::not_passed(&verdicts).is_empty(),
                    "{seed:?}: {verdicts:?}"
                );
            }
        }
    }
}

#[test]
fn go_waits_for_an_eventual_change_as_the_reference_runner_does() {
    let verdicts = support_go::assert_parity(
        "delta-lagging",
        &aggregate_suite(),
        Lagging {
            inner: Metrics::new(Mutant::None, Seed::Held),
            pending: Cell::new(0),
            projected: RefCell::default(),
        },
    );
    assert!(support_go::not_passed(&verdicts).is_empty(), "{verdicts:?}");
}
