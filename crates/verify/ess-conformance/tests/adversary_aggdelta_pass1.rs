//! Adversary, pass 1: the change of an ungrouped aggregate view (story
//! `ungrouped-aggregate-views-are-witnessed`, beyond10x/ess#148).
//!
//! Drives the synthesized `changed_by` scenarios of two ungrouped, unparameterised views over the
//! `aggregate-views.yaml` fixture against a hand-written target (no `ess_conformance::aggregate`),
//! on an empty target and on one that already holds another user's rows, and switches in one way
//! of getting each view wrong at a time.
//!
//! * `Everything` (`read_your_writes`, no filter): `count`, a required `sum`, an `avg` and a
//!   `count_distinct` — the change names only the first two.
//! * `CompletedTotals` (`eventual`, `state == Completed`): `count` and a required `sum`, with a
//!   refuted row the filter must leave out.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::Status,
    scenario::ViewExpectation,
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep,
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

/// The aggregate scenarios that do not pass against `target`, with every failed check's text, and
/// how many ran.
fn failing(mutant: Mutant, seed: Seed) -> (BTreeSet<String>, usize, String) {
    failing_on(&Metrics::new(mutant, seed))
}

fn failing_on<T: ConformanceTarget>(target: &T) -> (BTreeSet<String>, usize, String) {
    let suite = aggregate_suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, target)
        .into_report();
    let failed = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    let detail = format!("{:#?}", report.scenarios);
    (failed, report.scenarios.len(), detail)
}

fn set(views: &[&str]) -> BTreeSet<String> {
    views
        .iter()
        .map(|view| format!("{view}/aggregate"))
        .collect()
}

/// The `changed_by` amounts each scenario asserts, by view.
fn changes() -> BTreeMap<String, BTreeMap<String, Node>> {
    let suite = aggregate_suite();
    let mut out = BTreeMap::new();
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::ExpectView {
                view,
                expectation: ViewExpectation::ChangedBy { fields, .. },
            }
            | ScenarioStep::EventuallyView {
                view,
                expectation: ViewExpectation::ChangedBy { fields, .. },
                ..
            } = step
            {
                out.insert(view.to_string(), fields.clone());
            }
        }
    }
    out
}

#[test]
fn both_ungrouped_views_are_witnessed_by_count_and_sum_only() {
    let result = synthesis();
    let refused: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string().ends_with("/aggregate"))
        })
        .map(ToString::to_string)
        .collect();
    assert_eq!(refused, Vec::<String>::new());
    let changes = changes();
    assert_eq!(
        changes
            .get(EVERYTHING)
            .map(|fields| fields.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["sessions".to_owned(), "talk_seconds".to_owned()]),
        "{changes:#?}"
    );
    assert_eq!(
        changes
            .get(COMPLETED)
            .map(|fields| fields.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["sessions".to_owned(), "talk_seconds".to_owned()]),
        "{changes:#?}"
    );
}

#[test]
fn a_correct_target_passes_on_an_empty_and_on_a_held_target() {
    for seed in [Seed::Empty, Seed::Held] {
        let (failed, ran, detail) = failing(Mutant::None, seed);
        assert_eq!(ran, 2, "{seed:?}");
        assert_eq!(failed, BTreeSet::new(), "{seed:?}: {detail}");
    }
}

#[test]
fn a_filter_the_target_ignores_fails_the_filtered_view() {
    for seed in [Seed::Empty, Seed::Held] {
        let (failed, _, detail) = failing(Mutant::IgnoresFilter, seed);
        assert_eq!(failed, set(&[COMPLETED]), "{seed:?}: {detail}");
    }
}

#[test]
fn a_doubled_sum_fails_both_views_on_both_targets() {
    for seed in [Seed::Empty, Seed::Held] {
        let (failed, _, detail) = failing(Mutant::DoublesSum, seed);
        assert_eq!(failed, set(&[EVERYTHING, COMPLETED]), "{seed:?}: {detail}");
    }
}

/// Catches a runner that checks only the first named field (`sessions` sorts before
/// `talk_seconds`).
#[test]
fn a_wrong_sum_beside_a_right_count_fails_both_views() {
    // A constant offset cancels out of a change, so only an empty target shows it.
    let (failed, _, detail) = failing(Mutant::SumOffByOne, Seed::Empty);
    assert_eq!(failed, set(&[EVERYTHING, COMPLETED]), "{detail}");
}

/// Catches a runner that checks only the last named field.
#[test]
fn a_wrong_count_beside_a_right_sum_fails_both_views() {
    let (failed, _, detail) = failing(Mutant::CountOffByOne, Seed::Empty);
    assert_eq!(failed, set(&[EVERYTHING, COMPLETED]), "{detail}");
}

/// The page's zero-row table: `count` over zero rows is `0`, not absent, and the view field is a
/// required `Integer`. Before this story an ungrouped view's empty read asserted `count: 0`
/// ("ungrouped view returns no row when empty", "reports `min` 0 when empty"). The change reads an
/// absent value as zero for every function, so a target that reports `sessions: null` over an
/// empty source passes: nothing any scenario asserts rules it out.
#[test]
fn a_count_reported_absent_over_no_row_fails_on_an_empty_target() {
    let (failed, _, detail) = failing(Mutant::CountAbsentWhenEmpty, Seed::Empty);
    assert_eq!(failed, set(&[EVERYTHING, COMPLETED]), "{detail}");
}

/// The same for a `sum` over a required input: over zero rows it is `0`, never absent.
#[test]
fn a_required_sum_reported_absent_over_no_row_fails_on_an_empty_target() {
    let (failed, _, detail) = failing(Mutant::SumAbsentWhenEmpty, Seed::Empty);
    assert_eq!(failed, set(&[EVERYTHING, COMPLETED]), "{detail}");
}

/// The doc: "In an `eventually` block the read is retried, like any other expectation, until the
/// change holds". A projection that shows the scenario's own rows two reads late passes.
#[test]
fn an_eventual_view_that_catches_up_late_passes_its_change() {
    for seed in [Seed::Empty, Seed::Held] {
        let target = Lagging {
            inner: Metrics::new(Mutant::None, seed),
            pending: Cell::new(0),
            projected: RefCell::default(),
        };
        let (failed, _, detail) = failing_on(&target);
        assert_eq!(failed, BTreeSet::new(), "{seed:?}: {detail}");
    }
}

/// Every ordinary label below the round-3 pair refuses the expectation by vocabulary, not only
/// the /24 the unit's own test relabels to.
#[test]
fn a_change_under_any_older_ordinary_label_is_unsupported_vocabulary() {
    let original = aggregate_suite().to_canonical_json().unwrap();
    for older in [16, 20, 22, 24] {
        let text = original.replace(
            "\"ess-conformance/26\"",
            &format!("\"ess-conformance/{older}\""),
        );
        assert_ne!(text, original);
        let error = AdmittedSuite::from_json(&text).expect_err("a change needs suite/26");
        assert_eq!(
            error.issues[0].reason, "UnsupportedVocabulary",
            "/{older}: {error}"
        );
    }
}

/// "an amount that is not a number, is refused as `InvalidSuite`" — for a boolean and an array
/// too.
#[test]
fn a_boolean_or_list_amount_is_refused_as_invalid_suite() {
    let original = aggregate_suite().to_canonical_json().unwrap();
    for amount in [serde_json::json!(true), serde_json::json!([5])] {
        let mut document: serde_json::Value = serde_json::from_str(&original).unwrap();
        let steps = document["scenarios"]["metrics.session.Everything/aggregate"]["steps"]
            .as_array_mut()
            .unwrap();
        let change = steps
            .iter_mut()
            .find(|step| step["expectation"]["expect"] == "changed_by")
            .expect("the scenario asserts a change");
        change["expectation"]["fields"]["sessions"] = amount.clone();
        let text = serde_json::to_string_pretty(&document).unwrap();
        let error = AdmittedSuite::from_json(&text).expect_err("a change is a number");
        assert_eq!(error.issues[0].reason, "InvalidSuite", "{amount}: {error}");
    }
}
