//! Adversary, pass 2: the change of an ungrouped aggregate view (story
//! `ungrouped-aggregate-views-are-witnessed`, beyond10x/ess#148).
//!
//! A `Decimal` ledger with one ungrouped, unparameterised view holding a `count`, a required
//! `Decimal` `sum`, a skipping `Decimal` `sum` and an `avg`, driven against a hand-written target
//! (no `ess_conformance::aggregate`) on an empty target and on one that already holds another
//! user's rows, negative and fractional, with one way of getting the view wrong at a time.
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

const TOTALS: &str = "demo.ledger.Totals";

const LEDGER: &str = "format: ess/15
system: demo
version: v1
domain: demo.ledger
events:
  - name: demo.ledger.Booked
    fields: [{name: entry_id, type: Uuid}]
entities:
  - name: demo.ledger.Entry
    identity: {name: entry_id, type: Uuid}
    fields:
      - {name: account, type: String}
      - {name: amount, type: Decimal}
      - {name: tip, type: Optional<Decimal>}
    lifecycle:
      initial: Booked
      states: [Booked]
      terminal: [Booked]
commands:
  - name: demo.ledger.Book
    input:
      - {name: account, type: String}
      - {name: amount, type: Decimal}
      - {name: tip, type: Optional<Decimal>}
    outcomes:
      - name: booked
        creates: demo.ledger.Entry
        instance: entry_id
        sets:
          account: input.account
          amount: input.amount
          tip: input.tip
        emits: [demo.ledger.Booked]
        payload:
          demo.ledger.Booked:
            entry_id: {generated: true}
views:
  - name: demo.ledger.Totals
    source: demo.ledger.Entry
    consistency: read_your_writes
    fields:
      - {name: entries, type: Integer, aggregate: {count: {}}}
      - {name: amount, type: Decimal, aggregate: {sum: amount}}
      - {name: tips, type: Optional<Decimal>, aggregate: {sum: tip, skip_absent: true}}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: amount}}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis() -> Synthesis {
    synthesize(&ir(LEDGER))
}

fn n(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

/// An exact decimal as `(units, scale)`.
fn exact(node: &Node) -> (i128, u32) {
    let Node::Number(number) = node else {
        panic!("not a number: {node:?}");
    };
    let text = number.exact_text();
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    let digits = format!("{whole}{fraction}");
    (
        digits.parse().unwrap_or_else(|_| panic!("{text}")),
        u32::try_from(fraction.len()).unwrap(),
    )
}

fn sum(values: &[Node]) -> Node {
    let scale = values.iter().map(|v| exact(v).1).max().unwrap_or(0);
    let total: i128 = values
        .iter()
        .map(|v| {
            let (units, own) = exact(v);
            units * 10_i128.pow(scale - own)
        })
        .sum();
    let negative = total < 0;
    let digits = format!("{:0>width$}", total.abs(), width = scale as usize + 1);
    let (whole, fraction) = digits.split_at(digits.len() - scale as usize);
    let text = if scale == 0 {
        whole.to_owned()
    } else {
        format!("{whole}.{fraction}")
    };
    n(&format!("{}{text}", if negative { "-" } else { "" }))
}

/// Truncates an exact decimal toward zero.
fn truncated(node: &Node) -> Node {
    let (units, scale) = exact(node);
    n(&(units / 10_i128.pow(scale)).to_string())
}

/// The ledger suite's aggregate scenario only.
fn aggregate_suite() -> ConformanceSuite {
    let mut suite = synthesis().suite;
    suite
        .scenarios
        .retain(|id, _| id.to_string() == format!("{TOTALS}/aggregate"));
    suite
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Over no row, the ungrouped view returns no row at all instead of its one row of zeroes
    /// (the page's zero-row table: one row, `count` 0).
    NoRowWhenEmpty,
    /// Each `amount` is truncated to an integer before it is summed (a `Decimal` stored as an
    /// integer column).
    TruncatesAmounts,
    /// The skipping `sum` of `tip` is absent whenever any admitted row lacks a `tip` (an
    /// absence-propagating fold instead of one that skips).
    TipAbsenceSpreads,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seed {
    Empty,
    /// Another user's entries: `-40.125` with no tip, and `7.5` with a tip of `0.25`.
    Held,
}

type Row = BTreeMap<String, Node>;

struct Ledger {
    mutant: Mutant,
    seed: Seed,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Ledger {
    fn new(mutant: Mutant, seed: Seed) -> Self {
        Self {
            mutant,
            seed,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn totals(&self) -> Vec<Row> {
        let rows = self.rows.borrow();
        if rows.is_empty() && self.mutant == Mutant::NoRowWhenEmpty {
            return Vec::new();
        }
        let amounts: Vec<Node> = rows
            .iter()
            .map(|row| {
                let amount = row.get("amount").expect("amount").clone();
                if self.mutant == Mutant::TruncatesAmounts {
                    truncated(&amount)
                } else {
                    amount
                }
            })
            .collect();
        let tips: Vec<Node> = rows
            .iter()
            .filter_map(|row| row.get("tip").filter(|tip| **tip != Node::Null).cloned())
            .collect();
        let mut out = Row::new();
        out.insert("entries".into(), n(&rows.len().to_string()));
        out.insert("amount".into(), sum(&amounts));
        let spread = self.mutant == Mutant::TipAbsenceSpreads && tips.len() < rows.len();
        out.insert(
            "tips".into(),
            if tips.is_empty() || spread {
                Node::Null
            } else {
                sum(&tips)
            },
        );
        // Not asserted by a change; any number will do.
        out.insert(
            "mean".into(),
            if rows.is_empty() { Node::Null } else { n("1") },
        );
        vec![out]
    }
}

fn entry(id: &str, amount: &str, tip: Option<&str>) -> Row {
    let mut row = Row::new();
    row.insert("entry_id".into(), Node::Text(id.to_owned()));
    row.insert("account".into(), Node::Text("someone else".into()));
    row.insert("amount".into(), n(amount));
    if let Some(tip) = tip {
        row.insert("tip".into(), n(tip));
    }
    row
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("ledger-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(match self.seed {
            Seed::Empty => Vec::new(),
            Seed::Held => vec![
                entry("00000000-0000-4000-8000-900000000001", "-40.125", None),
                entry("00000000-0000-4000-8000-900000000002", "7.5", Some("0.25")),
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
        assert_eq!(command.to_string(), "demo.ledger.Book");
        let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
        let mut row: Row = request.input.clone();
        row.insert("entry_id".into(), Node::Text(id.clone()));
        self.rows.borrow_mut().push(row);
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new("booked").unwrap(),
        ))
        .emitting(
            ObservedEvent::new("demo.ledger.Booked".parse().unwrap())
                .with("entry_id", Node::Text(id)),
        )
        .with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), TOTALS);
        Ok(SemanticViewResult::of(self.totals()))
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

/// Whether the aggregate scenario passes against the target, with the report for a message.
fn passes(mutant: Mutant, seed: Seed) -> (bool, String) {
    let suite = aggregate_suite();
    assert_eq!(suite.scenarios.len(), 1, "the view has its scenario");
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Ledger::new(mutant, seed))
        .into_report();
    let detail = format!("{:#?}", report.scenarios);
    (
        report
            .scenarios
            .iter()
            .all(|scenario| scenario.status == Status::Passed),
        detail,
    )
}

/// The one `changed_by` the scenario asserts.
fn change() -> (BTreeMap<String, Node>, BTreeSet<String>) {
    aggregate_suite()
        .scenarios
        .values()
        .flat_map(|scenario| scenario.steps.iter())
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation:
                    ViewExpectation::ChangedBy {
                        fields,
                        absent_is_zero,
                    },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation:
                    ViewExpectation::ChangedBy {
                        fields,
                        absent_is_zero,
                    },
                ..
            } => Some((fields.clone(), absent_is_zero.clone())),
            _ => None,
        })
        .expect("the scenario asserts a change")
}

#[test]
fn a_decimal_ledger_names_count_and_both_sums_and_lists_only_the_skipping_sum() {
    let (fields, absent_is_zero) = change();
    assert_eq!(
        fields.keys().cloned().collect::<Vec<_>>(),
        ["amount", "entries", "tips"],
        "{fields:?}"
    );
    assert_eq!(absent_is_zero, BTreeSet::from(["tips".to_owned()]));
}

#[test]
fn a_correct_decimal_ledger_passes_on_an_empty_and_on_a_held_target() {
    for seed in [Seed::Empty, Seed::Held] {
        let (passed, detail) = passes(Mutant::None, seed);
        assert!(passed, "{seed:?}: {detail}");
    }
}

/// The one-row requirement is the only thing that catches this on the empty target: `tips` reads
/// as zero where absent, so a runner that took a missing row for a row of absent values would
/// pass the change of `tips`.
#[test]
fn an_ungrouped_view_that_returns_no_row_when_empty_fails_on_an_empty_target() {
    let (passed, detail) = passes(Mutant::NoRowWhenEmpty, Seed::Empty);
    assert!(!passed, "{detail}");
}

/// A skipping `sum` that turns absent as soon as one row lacks the value is not a skipping sum.
#[test]
fn a_skipping_sum_that_spreads_absence_fails_on_both_targets() {
    for seed in [Seed::Empty, Seed::Held] {
        let (passed, detail) = passes(Mutant::TipAbsenceSpreads, seed);
        assert!(!passed, "{seed:?}: {detail}");
    }
}

/// `Decimal` is exact (the page's "Semantics"). A target that stores each amount as an integer
/// sums the arranged amounts wrong unless every arranged amount is itself an integer — and today
/// every one is: story:ungrouped-aggregate-views-are-witnessed records the limit (Decimal aggregate
/// inputs use whole amounts, `docs/design/aggregate-views.md`, "The ladder"), because a fractional
/// ladder changes what earlier aggregate scenarios assert. This pins the limit: when the ladder
/// gains fractional amounts, this case must be turned back into the failing claim it was.
#[test]
fn a_target_that_truncates_decimal_amounts_fails_its_change() {
    for seed in [Seed::Empty, Seed::Held] {
        let (passed, detail) = passes(Mutant::TruncatesAmounts, seed);
        assert!(
            passed,
            "{seed:?}: story:ungrouped-aggregate-views-are-witnessed — Decimal aggregate inputs use \
             whole amounts, so a truncating target is not yet told apart; if this now fails, the \
             ladder gained fractional amounts and this case should assert the failure: {detail}"
        );
    }
}
