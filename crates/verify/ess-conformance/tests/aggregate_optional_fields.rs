//! Aggregates over, and group keys of, an `Optional` field (beyond10x/ess#148).
//!
//! `docs/design/aggregate-views.md`, "Absent values (ess/15)". A skipping aggregate is witnessed
//! with rows that lack the value, and asserted as SQL computes it: `SUM`, `AVG`, `MIN`, `MAX` and
//! `COUNT(DISTINCT …)` skip an absent value, and over no present value `SUM`, `AVG`, `MIN` and
//! `MAX` are absent while `COUNT(DISTINCT …)` is 0. An absent group key is one group, asserted with
//! `null` in the key. Every number asserted here is worked by hand from the page's pattern, not read
//! off the implementation; the target at the end computes the views without
//! `ess_conformance::aggregate`, and each mutant switches in one way of getting absence wrong.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    aggregate::{evaluate_skipping_absent, ValueKind},
    report::Status,
    scenario::ViewExpectation,
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
    view::AggregateFunction as F,
};
use ess_primitives::{facts::FactValue, node::Node};

const ORDERS: &str = include_str!("fixtures/aggregate-optional-fields.yaml");
const TOTAL: &str = "demo.orders.DurationTotal";
const PER_GROUP: &str = "demo.orders.PerGroup";
const BY_CUSTOMER: &str = "demo.orders.DurationByCustomer";
const FOR_CUSTOMER: &str = "demo.orders.DurationForCustomer";
const PER_CHANNEL: &str = "demo.orders.PerCustomerChannel";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; have {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

fn n(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn number(value: &str) -> ScenarioValue {
    ScenarioValue::literal(n(value))
}

fn text(value: &str) -> ScenarioValue {
    ScenarioValue::literal(Node::Text(value.to_owned()))
}

fn null() -> ScenarioValue {
    ScenarioValue::literal(Node::Null)
}

fn scoped(view: &str, group: &str) -> ScenarioValue {
    text(&format!("{view}/{group}"))
}

fn row(entries: &[(&str, ScenarioValue)]) -> BTreeMap<String, ScenarioValue> {
    entries
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

fn reads(
    scenario: &ConformanceScenario,
    view: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, ViewExpectation)> {
    let mut params = BTreeMap::new();
    let mut out = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::QueryView {
                view: read,
                params: bound,
            } if read.to_string() == view => params.clone_from(bound),
            ScenarioStep::ExpectView {
                view: read,
                expectation,
            } if read.to_string() == view => out.push((params.clone(), expectation.clone())),
            ScenarioStep::EventuallyView {
                view: read,
                params: bound,
                expectation,
            } if read.to_string() == view => out.push((bound.clone(), expectation.clone())),
            _ => {}
        }
    }
    out
}

fn contains(scenario: &ConformanceScenario, view: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    reads(scenario, view)
        .into_iter()
        .filter_map(|(_, expectation)| match expectation {
            ViewExpectation::Contains { fields } => Some(fields),
            _ => None,
        })
        .collect()
}

/// The input of every `Place` the scenario sent, in step order.
fn places(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.orders.Place" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .collect()
}

/// `duration` as each `Place` sent it: `None` where the input left it out.
fn durations(scenario: &ConformanceScenario) -> Vec<Option<ScenarioValue>> {
    places(scenario)
        .into_iter()
        .map(|input| input.get("duration").cloned())
        .collect()
}

// ---- semantics -------------------------------------------------------------------------------

fn skip(function: F, values: &[Node], kind: ValueKind) -> Node {
    evaluate_skipping_absent(function, values, kind).expect("an exact value")
}

#[test]
fn a_skipping_aggregate_reads_only_the_values_present() {
    let values = [Node::Null, n("1"), n("3"), Node::Null, n("3")];
    assert_eq!(skip(F::Sum, &values, ValueKind::Numeric), n("7"));
    assert_eq!(skip(F::Avg, &values, ValueKind::Numeric), n("2.333333"));
    assert_eq!(skip(F::Min, &values, ValueKind::Numeric), n("1"));
    assert_eq!(skip(F::Max, &values, ValueKind::Numeric), n("3"));
    assert_eq!(skip(F::CountDistinct, &values, ValueKind::Numeric), n("2"));
}

#[test]
fn over_no_present_value_sum_avg_and_the_extremes_are_absent_and_the_distinct_count_is_zero() {
    for values in [vec![], vec![Node::Null, Node::Null]] {
        assert_eq!(skip(F::Sum, &values, ValueKind::Numeric), Node::Null);
        assert_eq!(skip(F::Avg, &values, ValueKind::Numeric), Node::Null);
        assert_eq!(skip(F::Min, &values, ValueKind::Numeric), Node::Null);
        assert_eq!(skip(F::Max, &values, ValueKind::Text), Node::Null);
        assert_eq!(skip(F::CountDistinct, &values, ValueKind::Text), n("0"));
    }
}

// ---- synthesis -------------------------------------------------------------------------------

#[test]
fn the_issues_ungrouped_total_is_unscoped_and_every_other_view_has_its_scenario() {
    let result = synthesis(ORDERS);
    let ids: Vec<String> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.ends_with("/aggregate"))
        .collect();
    assert_eq!(
        ids,
        [
            format!("{BY_CUSTOMER}/aggregate"),
            format!("{FOR_CUSTOMER}/aggregate"),
            format!("{PER_CHANNEL}/aggregate"),
            format!("{PER_GROUP}/aggregate"),
        ]
    );
    // The issue's `DurationTotal` has no group key and no parameter, so nothing keeps its one row
    // to rows this scenario made (`ESS-SYNTH-016`) — the page's rule for every aggregate view, not
    // one about absent values. `DurationForCustomer` is the same total, scoped by a parameter.
    let refused: Vec<(String, String)> = result
        .refusals
        .iter()
        .filter(|refusal| refusal.to_string().contains("demo.orders."))
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect();
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert_eq!(refused[0].0, "ESS-SYNTH-016", "{refused:?}");
    assert!(refused[0].1.contains(TOTAL), "{refused:?}");
}

#[test]
fn a_grouped_skipping_aggregate_is_witnessed_by_a_row_that_lacks_the_value() {
    let suite = synthesis(ORDERS).suite;
    let by_customer = scenario(&suite, &format!("{BY_CUSTOMER}/aggregate"));
    // A: three pattern rows (1, 1, 3) and one that lacks `duration`; b lacks it too.
    assert_eq!(
        durations(by_customer),
        vec![
            Some(number("1")),
            Some(number("1")),
            Some(number("3")),
            None,
            None
        ]
    );
    assert_eq!(
        contains(by_customer, BY_CUSTOMER),
        vec![
            row(&[
                ("customer", scoped(BY_CUSTOMER, "A")),
                ("orders", number("4")),
                ("total", number("5")),
                ("mean", number("1.666667")),
                ("shortest", number("1")),
                ("longest", number("3")),
                ("durations", number("2")),
            ]),
            row(&[
                ("customer", scoped(BY_CUSTOMER, "B")),
                ("orders", number("1")),
                ("total", null()),
                ("mean", null()),
                ("shortest", null()),
                ("longest", null()),
                ("durations", number("0")),
            ]),
        ]
    );
}

#[test]
fn an_ungrouped_skipping_aggregate_is_witnessed_and_its_empty_read_is_absent() {
    let suite = synthesis(ORDERS).suite;
    let for_customer = scenario(&suite, &format!("{FOR_CUSTOMER}/aggregate"));
    // A's pattern rows, the row that lacks `duration`, and x, moved out of the scope.
    assert_eq!(
        durations(for_customer),
        vec![
            Some(number("1")),
            Some(number("1")),
            Some(number("3")),
            None,
            Some(number("97")),
        ]
    );
    let one = ViewExpectation::Counts {
        at_least: Some(1),
        at_most: Some(1),
    };
    assert_eq!(
        reads(for_customer, FOR_CUSTOMER),
        vec![
            (
                row(&[("customer", scoped(FOR_CUSTOMER, "in"))]),
                ViewExpectation::Contains {
                    fields: row(&[
                        ("orders", number("4")),
                        ("total", number("5")),
                        ("mean", number("1.666667")),
                    ]),
                }
            ),
            (
                row(&[("customer", scoped(FOR_CUSTOMER, "in"))]),
                one.clone()
            ),
            (
                row(&[("customer", scoped(FOR_CUSTOMER, "empty"))]),
                ViewExpectation::Contains {
                    fields: row(&[("orders", number("0")), ("total", null()), ("mean", null())]),
                }
            ),
            (row(&[("customer", scoped(FOR_CUSTOMER, "empty"))]), one),
        ]
    );
}

#[test]
fn an_optional_key_that_scopes_nothing_else_asserts_its_null_group_as_a_floor() {
    let suite = synthesis(ORDERS).suite;
    let per_group = scenario(&suite, &format!("{PER_GROUP}/aggregate"));
    let groups: Vec<Option<ScenarioValue>> = places(per_group)
        .into_iter()
        .map(|input| input.get("group").cloned())
        .collect();
    assert_eq!(
        groups,
        vec![
            Some(scoped(PER_GROUP, "A")),
            Some(scoped(PER_GROUP, "A")),
            Some(scoped(PER_GROUP, "A")),
            Some(scoped(PER_GROUP, "B")),
            None,
        ]
    );
    // Rows other scenarios make without a group land in the null group too, so its count is no
    // claim this scenario can make: the group is asserted to exist, and nothing more.
    assert_eq!(
        contains(per_group, PER_GROUP),
        vec![
            row(&[("group", scoped(PER_GROUP, "A")), ("orders", number("3"))]),
            row(&[("group", scoped(PER_GROUP, "B")), ("orders", number("1"))]),
            row(&[("group", null())]),
        ]
    );
}

#[test]
fn an_optional_key_beside_a_scoped_key_asserts_its_null_group_exactly() {
    let suite = synthesis(ORDERS).suite;
    let per_channel = scenario(&suite, &format!("{PER_CHANNEL}/aggregate"));
    assert_eq!(
        contains(per_channel, PER_CHANNEL),
        vec![
            row(&[
                ("customer", scoped(PER_CHANNEL, "A")),
                ("channel", text("Web")),
                ("orders", number("4")),
                ("total", number("5")),
            ]),
            row(&[
                ("customer", scoped(PER_CHANNEL, "B")),
                ("channel", text("Phone")),
                ("orders", number("1")),
                ("total", null()),
            ]),
            row(&[
                ("customer", scoped(PER_CHANNEL, "B")),
                ("channel", text("Web")),
                ("orders", number("1")),
                ("total", number("87")),
            ]),
            row(&[
                ("customer", scoped(PER_CHANNEL, "B")),
                ("channel", null()),
                ("orders", number("1")),
                ("total", number("89")),
            ]),
        ]
    );
}

/// Two skipping inputs are left out one at a time: SQL skips column by column, so a target that
/// drops a row lacking either one (`WHERE duration IS NOT NULL AND wait IS NOT NULL`) must report
/// other sums. A: pattern rows 1, 1, 3, 7, 13, 21 and 101, 101, 101, 103, 107, 113 (`m` = 6), then
/// a₇ lacking `duration` (wait 180, above its pattern) and a₈ lacking `wait` (duration 80).
#[test]
fn two_skipping_inputs_each_get_a_row_that_lacks_only_that_input() {
    let text = ORDERS
        .replace(
            "      - {name: duration, type: Optional<Integer>}\n",
            "      - {name: duration, type: Optional<Integer>}\n      - {name: wait, type: Optional<Integer>}\n",
        )
        .replace(
            "          duration: input.duration\n",
            "          duration: input.duration\n          wait: input.wait\n",
        );
    let head = text.split_once("views:\n").unwrap().0;
    let both = "demo.orders.Both";
    let model = format!(
        "{head}views:\n  - name: {both}\n    source: demo.orders.Order\n    group_by: [customer]\n    fields:\n      - {{name: customer, type: String}}\n      - {{name: total, type: Optional<Integer>, aggregate: {{sum: duration, skip_absent: true}}}}\n      - {{name: waited, type: Optional<Integer>, aggregate: {{sum: wait, skip_absent: true}}}}\n"
    );
    let suite = synthesis(&model).suite;
    let scenario = scenario(&suite, &format!("{both}/aggregate"));
    let sent: Vec<(Option<ScenarioValue>, Option<ScenarioValue>)> = places(scenario)
        .into_iter()
        .map(|input| (input.get("duration").cloned(), input.get("wait").cloned()))
        .collect();
    assert_eq!(
        sent[6..],
        [
            (None, Some(number("180"))),
            (Some(number("80")), None),
            (None, None),
        ]
    );
    assert_eq!(
        contains(scenario, both),
        vec![
            row(&[
                ("customer", scoped(both, "A")),
                ("total", number("126")),
                ("waited", number("806")),
            ]),
            row(&[
                ("customer", scoped(both, "B")),
                ("total", null()),
                ("waited", null()),
            ]),
        ]
    );
}

fn unwitnessed(model: &str, view: &str) -> Vec<String> {
    synthesis(model)
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains(view))
        .collect()
}

#[test]
fn a_value_no_created_row_can_lack_is_refused_rather_than_asserted_unwitnessed() {
    let model = ORDERS
        .replace(
            "      - {name: duration, type: Optional<Integer>}\n      - {name: group, type: Optional<demo.orders.Group>}\n      - {name: channel, type: Optional<demo.orders.Channel>}\n    outcomes:",
            "      - {name: duration, type: Integer}\n      - {name: group, type: demo.orders.Group}\n      - {name: channel, type: Optional<demo.orders.Channel>}\n    outcomes:",
        );
    for view in [BY_CUSTOMER, FOR_CUSTOMER, PER_GROUP] {
        let refused = unwitnessed(&model, view);
        assert_eq!(refused.len(), 1, "{view}: {refused:?}");
        assert!(refused[0].contains("absent"), "{refused:?}");
    }
}

// ---- runs ------------------------------------------------------------------------------------

/// One way to get absence wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// An absent `duration` is read as 0.
    AbsentAsZero,
    /// `count_distinct` counts an absent value as one more value.
    AbsentCountedDistinct,
    /// A row whose group key is absent is in no group.
    DropsAbsentKeyRows,
    /// A row that lacks `duration` is in no group (`WHERE duration IS NOT NULL`).
    DropsAbsentValueRows,
}

type Row = BTreeMap<String, Node>;

struct Orders {
    mutant: Mutant,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn int(node: &Node) -> i128 {
    match node {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("{other:?}"),
    }
}

/// The mean of integers to six places, half-even, spelled without trailing zeroes.
fn mean(values: &[i128]) -> Node {
    if values.is_empty() {
        return Node::Null;
    }
    let (sum, count) = (values.iter().sum::<i128>(), values.len() as i128);
    let scaled = sum * 1_000_000;
    let (mut q, r) = (scaled / count, scaled % count);
    if 2 * r > count || (2 * r == count && q % 2 == 1) {
        q += 1;
    }
    let text = format!("{}.{:06}", q / 1_000_000, q % 1_000_000);
    n(text.trim_end_matches('0').trim_end_matches('.'))
}

impl Orders {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn duration(&self, row: &Row) -> Option<i128> {
        match row.get("duration") {
            None | Some(Node::Null) if self.mutant == Mutant::AbsentAsZero => Some(0),
            None | Some(Node::Null) => None,
            Some(value) => Some(int(value)),
        }
    }

    fn view(&self, view: &str, params: &BTreeMap<String, Node>) -> Vec<Row> {
        let keys: &[&str] = match view {
            PER_GROUP => &["group"],
            BY_CUSTOMER => &["customer"],
            PER_CHANNEL => &["customer", "channel"],
            _ => &[],
        };
        let admitted: Vec<Row> = self
            .rows
            .borrow()
            .iter()
            .filter(|row| match view {
                FOR_CUSTOMER => row.get("customer") == params.get("customer"),
                _ => true,
            })
            .filter(|row| {
                !(self.mutant == Mutant::DropsAbsentValueRows
                    && matches!(row.get("duration"), None | Some(Node::Null)))
            })
            .filter(|row| {
                !(self.mutant == Mutant::DropsAbsentKeyRows
                    && keys
                        .iter()
                        .any(|key| matches!(row.get(*key), None | Some(Node::Null))))
            })
            .cloned()
            .collect();
        let mut groups: Vec<(Vec<Node>, Vec<Row>)> = Vec::new();
        for row in admitted {
            let key: Vec<Node> = keys
                .iter()
                .map(|key| row.get(*key).cloned().unwrap_or(Node::Null))
                .collect();
            match groups.iter_mut().find(|(held, _)| *held == key) {
                Some((_, members)) => members.push(row),
                None => groups.push((key, vec![row])),
            }
        }
        if keys.is_empty() && groups.is_empty() {
            groups.push((Vec::new(), Vec::new()));
        }
        groups
            .into_iter()
            .map(|(key, members)| {
                let mut out = Row::new();
                for (name, value) in keys.iter().zip(key) {
                    out.insert((*name).to_owned(), value);
                }
                out.insert("orders".into(), n(&members.len().to_string()));
                let present: Vec<i128> = members
                    .iter()
                    .filter_map(|row| self.duration(row))
                    .collect();
                let total = if present.is_empty() {
                    Node::Null
                } else {
                    n(&present.iter().sum::<i128>().to_string())
                };
                let extreme =
                    |value: Option<&i128>| value.map_or(Node::Null, |v| n(&v.to_string()));
                let distinct: BTreeSet<i128> = present.iter().copied().collect();
                let absent = members.len() > present.len();
                let counted = distinct.len()
                    + usize::from(absent && self.mutant == Mutant::AbsentCountedDistinct);
                match view {
                    BY_CUSTOMER => {
                        out.insert("total".into(), total);
                        out.insert("mean".into(), mean(&present));
                        out.insert("shortest".into(), extreme(present.iter().min()));
                        out.insert("longest".into(), extreme(present.iter().max()));
                        out.insert("durations".into(), n(&counted.to_string()));
                    }
                    FOR_CUSTOMER => {
                        out.insert("total".into(), total);
                        out.insert("mean".into(), mean(&present));
                    }
                    PER_CHANNEL => {
                        out.insert("total".into(), total);
                    }
                    _ => {}
                }
                out
            })
            .collect()
    }
}

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("orders-fixture", "1"))
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
        if command.to_string() != "demo.orders.Place" {
            return Err(TargetError::unsupported("command", command.to_string()));
        }
        let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
        let mut row: Row = request.input.clone();
        row.insert("order_id".into(), Node::Text(id.clone()));
        self.rows.borrow_mut().push(row);
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new("placed").unwrap(),
        ))
        .emitting(
            ObservedEvent::new("demo.orders.Placed".parse().unwrap())
                .with("order_id", Node::Text(id)),
        )
        .with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.view(&request.view.to_string(), &request.params),
        ))
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

/// The aggregate scenarios that do not pass against this target, and how many ran.
fn failing(mutant: Mutant) -> (BTreeSet<String>, usize) {
    let suite = synthesis(ORDERS).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Orders::new(mutant))
        .into_report();
    let aggregate: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.scenario.to_string().ends_with("/aggregate"))
        .collect();
    let failed = aggregate
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    (failed, aggregate.len())
}

fn set(views: &[&str]) -> BTreeSet<String> {
    views
        .iter()
        .map(|view| format!("{view}/aggregate"))
        .collect()
}

#[test]
fn the_views_as_specified_pass_their_own_suite() {
    let (failed, ran) = failing(Mutant::None);
    assert_eq!(ran, 4);
    assert_eq!(failed, BTreeSet::new());
}

#[test]
fn every_way_of_getting_absence_wrong_fails_the_scenarios_that_witness_it() {
    let table = [
        (
            Mutant::AbsentAsZero,
            set(&[BY_CUSTOMER, FOR_CUSTOMER, PER_CHANNEL]),
        ),
        (Mutant::AbsentCountedDistinct, set(&[BY_CUSTOMER])),
        (Mutant::DropsAbsentKeyRows, set(&[PER_GROUP, PER_CHANNEL])),
        (
            Mutant::DropsAbsentValueRows,
            set(&[BY_CUSTOMER, FOR_CUSTOMER, PER_CHANNEL]),
        ),
    ];
    let mut wrong = Vec::new();
    for (mutant, expected) in table {
        let (failed, _) = failing(mutant);
        if failed != expected {
            wrong.push(format!(
                "{mutant:?}: failed {failed:?}, expected {expected:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
