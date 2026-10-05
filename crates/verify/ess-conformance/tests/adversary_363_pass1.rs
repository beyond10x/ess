//! Adversary, beyond10x/ess#363 pass 1: conditional aggregate measures against faults the unit's
//! own fixture happens not to reach.
//!
//! `docs/design/conditional-aggregate-measures.md`, "Synthesis and target obligations" and "Named
//! acceptance and decisive faults": a target that drops a zero-selected group, swaps two measures'
//! conditions, or truncates an `avg` must fail the named observation, and synthesis must arrange
//! "all-false membership with a surviving group". The unit's fixture writes conditions that are
//! false for the default row (`escalated == true`, `state == Completed`); these models write
//! conditions an ordinary scorecard writes just as often, and run the synthesized suite against a
//! store with exactly one defect switched in.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::ScenarioId;
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::*;
use ess_conformance::AdmittedSuite;
use ess_domain::command::OutcomeName;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

const CASES: &str = include_str!("fixtures/conditional-aggregate-measures.yaml");

/// The unit's entity and commands, with `views` replaced.
fn model(views: &str) -> String {
    let head = CASES
        .split("\nviews:\n")
        .next()
        .expect("the fixture has views");
    format!("{head}\nviews:\n{views}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("cases.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

type Row = BTreeMap<String, Node>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cond {
    All,
    /// `escalated == true`.
    Escalated,
    /// `escalated == false`.
    Calm,
    /// `priority == High`.
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agg {
    Count,
    Avg(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    /// A group whose conditioned measures all select nothing is left out.
    DropsZeroSelected,
    /// The view's two conditioned measures read each other's condition.
    Swap,
    /// `avg` truncates to six places instead of rounding half-even.
    TruncatingAvg,
    /// The view's first condition filters the whole group, every measure of it included.
    WholeView,
}

struct ViewDef {
    name: &'static str,
    fields: Vec<(&'static str, Agg, Cond)>,
}

fn number(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn integer(node: &Node) -> i128 {
    match node {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("not a number: {other:?}"),
    }
}

/// The mean to six places, half-even (or truncated), spelled without trailing zeroes.
fn mean(values: &[i128], truncate: bool) -> Node {
    if values.is_empty() {
        return Node::Null;
    }
    let sum: i128 = values.iter().sum();
    let n = i128::try_from(values.len()).expect("a count");
    let (mut q, r) = (
        (sum * 1_000_000).div_euclid(n),
        (sum * 1_000_000).rem_euclid(n),
    );
    if !truncate && (2 * r > n || (2 * r == n && q % 2 == 1)) {
        q += 1;
    }
    let sign = if q < 0 { "-" } else { "" };
    let text = format!("{sign}{}.{:06}", q.abs() / 1_000_000, q.abs() % 1_000_000);
    number(text.trim_end_matches('0').trim_end_matches('.'))
}

struct Store {
    fault: Fault,
    views: Vec<ViewDef>,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Store {
    fn new(fault: Fault, views: Vec<ViewDef>) -> Self {
        Self {
            fault,
            views,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn holds(condition: Cond, row: &Row) -> bool {
        let escalated = row.get("escalated") == Some(&Node::Bool(true));
        match condition {
            Cond::All => true,
            Cond::Escalated => escalated,
            Cond::Calm => !escalated,
            Cond::High => row.get("priority") == Some(&Node::Text("High".into())),
        }
    }

    fn query(&self, view: &ViewDef) -> Vec<Row> {
        let rows = self.rows.borrow().clone();
        let mut groups: Vec<(Node, Vec<Row>)> = Vec::new();
        for row in rows {
            let key = row["team"].clone();
            match groups.iter_mut().find(|(held, _)| *held == key) {
                Some((_, members)) => members.push(row),
                None => groups.push((key, vec![row])),
            }
        }
        let conditioned: Vec<Cond> = view
            .fields
            .iter()
            .map(|(_, _, condition)| *condition)
            .filter(|condition| *condition != Cond::All)
            .collect();
        let mut out = Vec::new();
        for (key, mut members) in groups {
            if let (Fault::WholeView, Some(first)) = (self.fault, conditioned.first()) {
                members.retain(|row| Self::holds(*first, row));
                if members.is_empty() {
                    continue;
                }
            }
            let mut reported = Row::from([("team".to_owned(), key)]);
            let mut selected_any = false;
            for (field, aggregate, declared) in &view.fields {
                let condition = match (self.fault, conditioned.as_slice()) {
                    (Fault::Swap, [first, second]) if declared == first => *second,
                    (Fault::Swap, [first, second]) if declared == second => *first,
                    _ => *declared,
                };
                let selected: Vec<&Row> = members
                    .iter()
                    .filter(|row| Self::holds(condition, row))
                    .collect();
                if *declared != Cond::All && !selected.is_empty() {
                    selected_any = true;
                }
                reported.insert(
                    (*field).to_owned(),
                    Self::aggregate(*aggregate, &selected, self.fault == Fault::TruncatingAvg),
                );
            }
            if self.fault == Fault::DropsZeroSelected && !selected_any {
                continue;
            }
            out.push(reported);
        }
        out
    }

    fn aggregate(aggregate: Agg, members: &[&Row], truncate: bool) -> Node {
        let column = |field: &str| {
            members
                .iter()
                .map(|row| integer(&row[field]))
                .collect::<Vec<_>>()
        };
        match aggregate {
            Agg::Count => number(&members.len().to_string()),
            Agg::Avg(field) => mean(&column(field), truncate),
        }
    }

    fn mint(&self) -> (u64, Node) {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        (n, Node::Text(format!("00000000-0000-4000-8000-{n:012}")))
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-363-store", "1"))
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
        let (n, id) = self.mint();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let result = match command.to_string().as_str() {
            "demo.cases.Open" => {
                let mut row = Row::new();
                for field in ["team", "cents", "label", "escalated", "priority", "tags"] {
                    row.insert(
                        field.to_owned(),
                        request.input.get(field).cloned().unwrap_or(Node::Null),
                    );
                }
                row.insert("case_id".into(), id.clone());
                row.insert("state".into(), Node::Text("Open".into()));
                self.rows.borrow_mut().push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.cases.Opened".parse().unwrap()).with("case_id", id),
                )
            }
            "demo.cases.Complete" => {
                let named = request.input.get("case_id").cloned().unwrap_or(Node::Null);
                let mut rows = self.rows.borrow_mut();
                let Some(row) = rows.iter_mut().find(|row| row["case_id"] == named) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                if row["state"] == Node::Text("Open".into()) {
                    row.insert("state".into(), Node::Text("Completed".into()));
                    SemanticCommandResult::took(outcome(&command, "completed")).emitting(
                        ObservedEvent::new("demo.cases.Completed".parse().unwrap())
                            .with("case_id", named),
                    )
                } else {
                    SemanticCommandResult::took(outcome(&command, "unavailable"))
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let name = request.view.to_string();
        let Some(view) = self.views.iter().find(|view| view.name == name) else {
            return Err(TargetError::unsupported("view", &name));
        };
        Ok(SemanticViewResult::of(self.query(view)))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcome", "none"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "none"))
    }
}

/// The model's synthesized aggregate scenario for `view`, admitted; panics naming any refusal.
fn aggregate_suite(text: &str, view: &str) -> AdmittedSuite {
    let result = synthesize(&ir(text));
    let refusals: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| refusal.to_string().contains(view))
        .map(ToString::to_string)
        .collect();
    assert_eq!(refusals, Vec::<String>::new(), "`{view}` is witnessed");
    let mut suite = result.suite;
    suite
        .scenarios
        .retain(|id, _| matches!(id, ScenarioId::Aggregate { view: v } if v.to_string() == view));
    assert_eq!(
        suite.scenarios.len(),
        1,
        "one aggregate scenario for `{view}`"
    );
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"))
}

/// Whether the store, with `fault` switched in, passes every scenario of `suite`.
fn passes(suite: &AdmittedSuite, fault: Fault, views: Vec<ViewDef>) -> bool {
    let report = ess_conformance::Runner::for_suite(suite.suite())
        .run_admitted(suite, &Store::new(fault, views))
        .into_report();
    report
        .scenarios
        .iter()
        .all(|result| result.status == ess_conformance::report::Status::Passed)
}

/// Every row the suite's view reads expect to contain, one line each: the evidence of what the
/// synthesized scenario actually asserts.
fn asserted(suite: &AdmittedSuite) -> String {
    let mut out = Vec::new();
    for scenario in suite.suite().scenarios.values() {
        for step in &scenario.steps {
            if let ess_conformance::ScenarioStep::ExpectView {
                expectation: ess_conformance::scenario::ViewExpectation::Contains { fields },
                ..
            } = step
            {
                out.push(serde_json::to_string(fields).expect("expected fields serialize"));
            }
        }
    }
    out.join("\n")
}

// ---- 1. a zero-selected group --------------------------------------------------------------------

const CALM: &str = r"  - name: demo.cases.Calm
    source: demo.cases.Case
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: total, type: Integer, aggregate: {count: {}}}
      - {name: calm, type: Integer, aggregate: {count: {}, where: escalated == false}}
";

fn calm() -> Vec<ViewDef> {
    vec![ViewDef {
        name: "demo.cases.Calm",
        fields: vec![
            ("total", Agg::Count, Cond::All),
            ("calm", Agg::Count, Cond::Calm),
        ],
    }]
}

/// "drop a zero-selected group … must fail the named observation", and synthesis must arrange
/// "all-false membership with a surviving group". Here the condition holds for the row every
/// group but A is made of, so no asserted group selects nothing.
#[test]
fn adversary_a_dropped_zero_selected_group_fails_when_the_default_row_satisfies_the_condition() {
    let suite = aggregate_suite(&model(CALM), "demo.cases.Calm");
    assert!(
        passes(&suite, Fault::None, calm()),
        "the healthy store passes"
    );
    assert!(
        !passes(&suite, Fault::DropsZeroSelected, calm()),
        "a store that drops a group whose conditioned measure selects nothing passes the \
         synthesized `demo.cases.Calm/aggregate` scenario, which asserts:\n{}",
        asserted(&suite)
    );
}

const ONLY_CALM: &str = r"  - name: demo.cases.OnlyCalm
    source: demo.cases.Case
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: calm, type: Integer, aggregate: {count: {}, where: escalated == false}}
";

fn only_calm() -> Vec<ViewDef> {
    vec![ViewDef {
        name: "demo.cases.OnlyCalm",
        fields: vec![("calm", Agg::Count, Cond::Calm)],
    }]
}

/// "faults that … apply it to the whole view … must fail the named observation". Without an
/// unconditioned measure beside it, only a group the condition selects nothing of tells the two
/// apart, and none is asserted.
#[test]
fn adversary_a_condition_applied_to_the_whole_view_fails_the_observation() {
    let suite = aggregate_suite(&model(ONLY_CALM), "demo.cases.OnlyCalm");
    assert!(
        passes(&suite, Fault::None, only_calm()),
        "the healthy store passes"
    );
    assert!(
        !passes(&suite, Fault::WholeView, only_calm()),
        "a store that applies `calm`'s condition to the whole view passes the synthesized \
         `demo.cases.OnlyCalm/aggregate` scenario, which asserts:\n{}",
        asserted(&suite)
    );
}

// ---- 2. two conditioned counts swapped -------------------------------------------------------------

const PAIR: &str = r"  - name: demo.cases.Pair
    source: demo.cases.Case
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: total, type: Integer, aggregate: {count: {}}}
      - {name: hot, type: Integer, aggregate: {count: {}, where: escalated == true}}
      - {name: high, type: Integer, aggregate: {count: {}, where: priority == High}}
";

fn pair() -> Vec<ViewDef> {
    vec![ViewDef {
        name: "demo.cases.Pair",
        fields: vec![
            ("total", Agg::Count, Cond::All),
            ("hot", Agg::Count, Cond::Escalated),
            ("high", Agg::Count, Cond::High),
        ],
    }]
}

/// "faults that drop, invert or swap the predicate … must fail the named observation".
#[test]
fn adversary_swapped_conditions_of_two_counts_fail_the_observation() {
    let suite = aggregate_suite(&model(PAIR), "demo.cases.Pair");
    assert!(
        passes(&suite, Fault::None, pair()),
        "the healthy store passes"
    );
    assert!(
        !passes(&suite, Fault::Swap, pair()),
        "a store that swaps `hot`'s and `high`'s conditions passes the synthesized \
         `demo.cases.Pair/aggregate` scenario, which asserts:\n{}",
        asserted(&suite)
    );
}

// ---- 3. a conditioned mean that truncates ----------------------------------------------------------

const MEANS: &str = r"  - name: demo.cases.Means
    source: demo.cases.Case
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: cases, type: Integer, aggregate: {count: {}}}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: cents, where: escalated == true}}
";

fn means() -> Vec<ViewDef> {
    vec![ViewDef {
        name: "demo.cases.Means",
        fields: vec![
            ("cases", Agg::Count, Cond::All),
            ("mean", Agg::Avg("cents"), Cond::Escalated),
        ],
    }]
}

/// The aggregate page's mutant "`avg` truncates instead of rounding" is refused for an
/// unconditioned mean that cannot show it (`rounding_is_observable`); a conditioned one skips the
/// check, so nothing in the suite tells the two apart.
#[test]
fn adversary_a_truncating_conditioned_avg_fails_the_observation() {
    let suite = aggregate_suite(&model(MEANS), "demo.cases.Means");
    assert!(
        passes(&suite, Fault::None, means()),
        "the healthy store passes"
    );
    assert!(
        !passes(&suite, Fault::TruncatingAvg, means()),
        "a store whose `avg` truncates passes the synthesized `demo.cases.Means/aggregate` \
         scenario, which asserts:\n{}",
        asserted(&suite)
    );
}

// ---- 4. a condition over the measured field ----------------------------------------------------------

const LARGE: &str = r"  - name: demo.cases.Large
    source: demo.cases.Case
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: cases, type: Integer, aggregate: {count: {}}}
      - {name: large_cost, type: Integer, aggregate: {sum: cents, where: cents > 5}}
";

/// `sum: cents, where: cents > 5` — the measured value is the condition's only input. The contrast
/// search never varies an aggregate input (`contrast::dimensions` skips `chosen`), so it depends on
/// whatever the input ladder happens to hold.
#[test]
fn adversary_a_condition_over_the_measured_field_is_witnessed() {
    let suite = aggregate_suite(&model(LARGE), "demo.cases.Large");
    assert_eq!(suite.suite().scenarios.len(), 1);
}
