//! Conditional aggregate measures, executed (beyond10x/ess#363).
//!
//! `docs/design/conditional-aggregate-measures.md`. One aggregate row reports measures that each
//! read their own subset of the group: `{count: {}, where: state == Completed}` beside an
//! unconditioned total. The fixture `fixtures/conditional-aggregate-measures.yaml` declares the
//! issue's scorecard, all six functions under one condition, a composite condition and a
//! quantified one.
//!
//! Everything above the first test is the target: a store of its own that computes every view of
//! the fixture itself and never reads an expected value from the suite. It runs natively; the
//! generated Go and TypeScript runtimes run ports of it
//! (`fixtures/conditional-aggregate-measures-runtime.go`, `.mjs`). A `Fault` switches in one
//! defect at a time.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_conformance::target::*;
use ess_domain::command::OutcomeName;
use ess_primitives::{facts::FactValue, node::Node};

/// One defect an implementation could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Fault {
    None,
    /// `Escalations`' measure of this function reads every row of its group.
    Drop(Function),
    /// `Escalations`' measure of this function reads the rows its condition refuses.
    Invert(Function),
    /// `Scorecard`'s two conditions are exchanged.
    Swap,
    /// A view's first condition filters the whole group, every measure of it included.
    WholeView,
    /// A group whose conditioned measures all select nothing is left out.
    DropsZeroSelected,
    /// A transition answers and leaves the row in its state.
    StaleState,
    /// A composite condition is read as its first branch.
    FirstBranch,
    /// `exists` is evaluated as `forall`.
    ForallForExists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Function {
    Count,
    Distinct,
    Sum,
    Min,
    Max,
    Avg,
}

impl Function {
    const ALL: [Self; 6] = [
        Self::Count,
        Self::Distinct,
        Self::Sum,
        Self::Min,
        Self::Max,
        Self::Avg,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Distinct => "count-distinct",
            Self::Sum => "sum",
            Self::Min => "min",
            Self::Max => "max",
            Self::Avg => "avg",
        }
    }
}

impl Fault {
    fn all() -> Vec<Self> {
        let mut out = vec![Self::None];
        out.extend(Function::ALL.map(Self::Drop));
        out.extend(Function::ALL.map(Self::Invert));
        out.extend([
            Self::Swap,
            Self::WholeView,
            Self::DropsZeroSelected,
            Self::StaleState,
            Self::FirstBranch,
            Self::ForallForExists,
        ]);
        out
    }

    fn name(self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::Drop(function) => format!("drop-{}", function.name()),
            Self::Invert(function) => format!("invert-{}", function.name()),
            Self::Swap => "swap".to_owned(),
            Self::WholeView => "whole-view".to_owned(),
            Self::DropsZeroSelected => "drops-zero-selected".to_owned(),
            Self::StaleState => "stale-state".to_owned(),
            Self::FirstBranch => "first-branch".to_owned(),
            Self::ForallForExists => "forall-for-exists".to_owned(),
        }
    }

    fn of(name: &str) -> Self {
        Self::all()
            .into_iter()
            .find(|fault| fault.name() == name)
            .unwrap_or_else(|| panic!("no fault {name}"))
    }
}

/// A measure's condition, as the fixture writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cond {
    /// Every row of the group.
    All,
    /// `state == Completed`.
    Completed,
    /// `escalated == true`.
    Escalated,
    /// `{any: [priority == High, {all: [escalated == true, state == Completed]}]}`.
    Urgent,
    /// `{not: priority == Low}`.
    NotLow,
    /// `priority == High`.
    High,
    /// `{exists: {in: tags, as: t, that: t == "rush"}}`.
    Rush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agg {
    Count,
    Distinct(&'static str),
    Sum(&'static str),
    Min(&'static str),
    Max(&'static str),
    Avg(&'static str),
}

impl Agg {
    fn function(self) -> Function {
        match self {
            Self::Count => Function::Count,
            Self::Distinct(_) => Function::Distinct,
            Self::Sum(_) => Function::Sum,
            Self::Min(_) => Function::Min,
            Self::Max(_) => Function::Max,
            Self::Avg(_) => Function::Avg,
        }
    }
}

/// One view of the fixture: every one is grouped by `team`.
struct ViewDef {
    name: &'static str,
    fields: &'static [(&'static str, Agg, Cond)],
}

const VIEWS: &[ViewDef] = &[
    ViewDef {
        name: "demo.cases.Scorecard",
        fields: &[
            ("total", Agg::Count, Cond::All),
            ("completed", Agg::Count, Cond::Completed),
            ("escalated_cost", Agg::Sum("cents"), Cond::Escalated),
        ],
    },
    ViewDef {
        name: "demo.cases.Escalations",
        fields: &[
            ("cases", Agg::Count, Cond::All),
            ("escalated", Agg::Count, Cond::Escalated),
            ("labels", Agg::Distinct("label"), Cond::Escalated),
            ("cost", Agg::Sum("cents"), Cond::Escalated),
            ("cheapest", Agg::Min("cents"), Cond::Escalated),
            ("dearest", Agg::Max("cents"), Cond::Escalated),
            ("mean", Agg::Avg("cents"), Cond::High),
        ],
    },
    ViewDef {
        name: "demo.cases.Urgent",
        fields: &[
            ("cases", Agg::Count, Cond::All),
            ("urgent", Agg::Count, Cond::Urgent),
            ("urgent_cost", Agg::Sum("cents"), Cond::NotLow),
        ],
    },
    ViewDef {
        name: "demo.cases.Tagged",
        fields: &[
            ("cases", Agg::Count, Cond::All),
            ("rushed", Agg::Count, Cond::Rush),
        ],
    },
];

type Row = BTreeMap<String, Node>;

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

/// The mean of integers to six places, half-even, spelled without trailing zeroes.
fn mean(values: &[i128]) -> Node {
    if values.is_empty() {
        return Node::Null;
    }
    let sum: i128 = values.iter().sum();
    let n = i128::try_from(values.len()).expect("a count");
    let (mut q, r) = ((sum * 1_000_000) / n, (sum * 1_000_000) % n);
    if 2 * r > n || (2 * r == n && q % 2 == 1) {
        q += 1;
    }
    let text = format!("{}.{:06}", q / 1_000_000, q % 1_000_000);
    number(text.trim_end_matches('0').trim_end_matches('.'))
}

struct Store {
    fault: Fault,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Store {
    fn new(fault: Fault) -> Self {
        Self {
            fault,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn text(row: &Row, field: &str) -> String {
        match row.get(field) {
            Some(Node::Text(text)) => text.clone(),
            other => panic!("`{field}` is not text: {other:?}"),
        }
    }

    fn holds(&self, condition: Cond, row: &Row) -> bool {
        let completed = Self::text(row, "state") == "Completed";
        let escalated = row.get("escalated") == Some(&Node::Bool(true));
        let high = Self::text(row, "priority") == "High";
        match condition {
            Cond::All => true,
            Cond::Completed => completed,
            Cond::Escalated => escalated,
            Cond::Urgent if self.fault == Fault::FirstBranch => high,
            Cond::Urgent => high || (escalated && completed),
            Cond::NotLow => Self::text(row, "priority") != "Low",
            Cond::High => high,
            Cond::Rush => {
                let Some(Node::Seq(tags)) = row.get("tags") else {
                    panic!("tags are a list: {row:?}");
                };
                let rush = |tag: &Node| *tag == Node::Text("rush".into());
                if self.fault == Fault::ForallForExists {
                    tags.iter().all(rush)
                } else {
                    tags.iter().any(rush)
                }
            }
        }
    }

    /// The condition this store applies to one measure of `view`.
    fn applied(&self, view: &ViewDef, field: &str, aggregate: Agg, declared: Cond) -> (Cond, bool) {
        match self.fault {
            Fault::Drop(function)
                if view.name == "demo.cases.Escalations"
                    && aggregate.function() == function
                    && declared != Cond::All =>
            {
                (Cond::All, false)
            }
            Fault::Invert(function)
                if view.name == "demo.cases.Escalations"
                    && aggregate.function() == function
                    && declared != Cond::All =>
            {
                (declared, true)
            }
            Fault::Swap if view.name == "demo.cases.Scorecard" => match field {
                "completed" => (Cond::Escalated, false),
                "escalated_cost" => (Cond::Completed, false),
                _ => (declared, false),
            },
            _ => (declared, false),
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
        let first_condition = view
            .fields
            .iter()
            .map(|(_, _, condition)| *condition)
            .find(|condition| *condition != Cond::All);
        let mut out = Vec::new();
        for (key, mut members) in groups {
            if self.fault == Fault::WholeView {
                if let Some(condition) = first_condition {
                    members.retain(|row| self.holds(condition, row));
                    if members.is_empty() {
                        continue;
                    }
                }
            }
            let mut reported = Row::from([("team".to_owned(), key)]);
            let mut selected_any = false;
            for (field, aggregate, declared) in view.fields {
                let (condition, inverted) = self.applied(view, field, *aggregate, *declared);
                let selected: Vec<&Row> = members
                    .iter()
                    .filter(|row| self.holds(condition, row) != inverted)
                    .collect();
                if *declared != Cond::All && !selected.is_empty() {
                    selected_any = true;
                }
                reported.insert((*field).to_owned(), Self::aggregate(*aggregate, &selected));
            }
            if self.fault == Fault::DropsZeroSelected && !selected_any {
                continue;
            }
            out.push(reported);
        }
        out
    }

    fn aggregate(aggregate: Agg, members: &[&Row]) -> Node {
        let column = |field: &str| {
            members
                .iter()
                .map(|row| integer(&row[field]))
                .collect::<Vec<_>>()
        };
        let spelled = |value: i128| number(&value.to_string());
        match aggregate {
            Agg::Count => spelled(i128::try_from(members.len()).expect("a count")),
            Agg::Sum(field) => spelled(column(field).iter().sum()),
            Agg::Min(field) => column(field).into_iter().min().map_or(Node::Null, spelled),
            Agg::Max(field) => column(field).into_iter().max().map_or(Node::Null, spelled),
            Agg::Avg(field) => mean(&column(field)),
            Agg::Distinct(field) => {
                let mut seen: Vec<&Node> = Vec::new();
                for row in members {
                    if !seen.contains(&&row[field]) {
                        seen.push(&row[field]);
                    }
                }
                spelled(i128::try_from(seen.len()).expect("a count"))
            }
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
        Ok(ImplementationIdentity::new(
            "conditional-measures-fixture",
            "1",
        ))
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
                    if self.fault != Fault::StaleState {
                        row.insert("state".into(), Node::Text("Completed".into()));
                    }
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
        let Some(view) = VIEWS.iter().find(|view| view.name == name) else {
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

/// Every scenario's verdict against a store: `passed`, `failed`, `error` or `unsupported`.
fn run_target<T: ConformanceTarget>(
    suite: &ess_conformance::AdmittedSuite,
    target: &T,
) -> Verdicts {
    ess_conformance::Runner::for_suite(suite.suite())
        .run_admitted(suite, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let status = match result.status {
                ess_conformance::report::Status::Passed => "passed",
                ess_conformance::report::Status::Failed => "failed",
                ess_conformance::report::Status::Error => "error",
                ess_conformance::report::Status::Unsupported => "unsupported",
            };
            (result.scenario.to_string(), status.to_owned())
        })
        .collect()
}

type Verdicts = BTreeMap<String, String>;

// ---- the tests -------------------------------------------------------------------------------

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted,
    scenario::{ScenarioId, ScenarioInitialState, SuiteFormat, ViewExpectation},
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const CASES: &str = include_str!("fixtures/conditional-aggregate-measures.yaml");

/// The four views, by scenario id.
const SCENARIOS: [&str; 4] = [
    "demo.cases.Escalations/aggregate",
    "demo.cases.Scorecard/aggregate",
    "demo.cases.Tagged/aggregate",
    "demo.cases.Urgent/aggregate",
];

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("cases.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn aggregate_refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            matches!(
                refusal.code().to_string().as_str(),
                "ESS-SYNTH-016" | "ESS-SYNTH-017"
            )
        })
        .map(ToString::to_string)
        .collect()
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(held, _)| held.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// The fixture's aggregate scenarios only, admitted.
fn aggregate_suite() -> AdmittedSuite {
    let result = synthesis(CASES);
    assert_eq!(
        aggregate_refusals(&result),
        Vec::<String>::new(),
        "every view of the fixture is witnessed"
    );
    let mut suite = result.suite;
    assert_eq!(
        suite.provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    suite
        .scenarios
        .retain(|id, _| matches!(id, ScenarioId::Aggregate { .. }));
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"))
}

/// Every `Contains` a scenario asserts, with its fields.
fn contains(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .collect()
}

fn literal(value: &ScenarioValue) -> &Node {
    match value {
        ScenarioValue::Literal { value } => value,
        other => panic!("not a literal: {other:?}"),
    }
}

fn counted(value: &ScenarioValue) -> i128 {
    integer(literal(value))
}

// ---- synthesis -------------------------------------------------------------------------------

/// The red: on the code before this change the fixture does not validate (`unknown field where`).
#[test]
fn conditional_measures_keep_group_membership_independent() {
    let suite = aggregate_suite();
    let ids: Vec<String> = suite
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(ids, SCENARIOS, "one scenario per view");

    // The scorecard: two differently filtered measures and an unconditioned total in one row.
    let scorecard = contains(scenario(suite.suite(), "demo.cases.Scorecard/aggregate"));
    let mut proper = 0;
    let mut zero_selected = 0;
    for row in &scorecard {
        let total = counted(&row["total"]);
        let completed = counted(&row["completed"]);
        assert!(completed <= total, "{row:?}");
        if completed > 0 && completed < total {
            proper += 1;
        }
        if completed == 0 && counted(&row["escalated_cost"]) == 0 {
            zero_selected += 1;
        }
    }
    assert!(
        proper >= 1,
        "a group holds matching and nonmatching rows: {scorecard:#?}"
    );
    assert!(
        zero_selected >= 1,
        "a group whose conditioned measures select nothing survives: {scorecard:#?}"
    );
    assert!(
        scorecard.len() >= 2,
        "a second group is the decoy: {scorecard:#?}"
    );

    // All six functions under one condition, with an absent extreme and mean where it selects
    // nothing.
    let escalations = contains(scenario(suite.suite(), "demo.cases.Escalations/aggregate"));
    assert!(
        escalations
            .iter()
            .any(|row| *literal(&row["cheapest"]) == Node::Null
                && *literal(&row["mean"]) == Node::Null
                && counted(&row["escalated"]) == 0
                && counted(&row["labels"]) == 0
                && counted(&row["cost"]) == 0
                && counted(&row["cases"]) > 0),
        "{escalations:#?}"
    );
}

#[test]
fn conditional_measures_select_suite_38_and_coverage_39() {
    let ir = ir(CASES);
    let suite = synthesize(&ir).suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/38"
    );
    let built = ess_conformance::coverage_build::build(
        &ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let document: serde_json::Value =
        serde_json::from_str(built.selected().original_json()).expect("a coverage suite is JSON");
    assert_eq!(
        document["provenance"]["suite_version"], "ess-conformance/39",
        "the coverage counterpart"
    );
    // A model whose aggregate views write no condition keeps its format.
    let plain = CASES
        .replace(", where: state == Completed", "")
        .replace(", where: escalated == true", "")
        .replace(
            ", where: {any: [priority == High, {all: [escalated == true, state == Completed]}]}",
            "",
        )
        .replace(", where: {not: priority == Low}", "")
        .replace(", where: priority == High", "")
        .replace(
            ", where: {exists: {in: tags, as: t, that: t == \"rush\"}}",
            "",
        );
    assert!(!plain.contains("where:"), "{plain}");
    assert_eq!(
        synthesize(&self::ir(&plain))
            .suite
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/34"
    );
}

#[test]
fn conditional_measure_readers_reject_changed_authority() {
    let ir = ir(CASES);
    let suite = synthesize(&ir).suite;
    ess_conformance::conditional_measures::admit_for(&ir, &suite)
        .unwrap_or_else(|error| panic!("{error}"));
    // Relabelled below the pair, the suite is refused before any callback.
    for older in ["ess-conformance/34", "ess-conformance/36"] {
        let mut relabelled = suite.clone();
        relabelled.provenance.suite_version = SuiteFormat::parse(older).unwrap();
        let error =
            ess_conformance::conditional_measures::admit_for(&ir, &relabelled).expect_err(older);
        assert!(error.to_string().contains("suite/38"), "{error}");
    }
    // A changed, removed or narrowed condition changes the contract the suite names.
    let digest = |text: &str| {
        ess_conformance::SuiteProvenance::of(&self::ir(text))
            .contract_digest
            .to_string()
    };
    let base = digest(CASES);
    for changed in [
        CASES.replace("where: state == Completed}", "where: state == Open}"),
        CASES.replace(", where: state == Completed}", "}"),
        CASES.replace(
            "where: state == Completed}",
            "where: [state == Completed, cents > 1]}",
        ),
    ] {
        assert_ne!(digest(&changed), base);
    }
    // The generated runtimes admit the pair.
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    ess_conformance::go::emit(admitted.suite()).unwrap_or_else(|error| panic!("{error}"));
    ess_conformance::ts::emit(admitted.suite()).unwrap_or_else(|error| panic!("{error}"));
}

/// An authored scenario that reads a conditioned view needs the pair as much as a synthesized
/// one: its expected rows mean the conditioned measure too.
#[test]
fn an_authored_observation_of_a_conditioned_view_requires_the_pair() {
    let ir = ir(CASES);
    let mut suite = synthesize(&ir).suite;
    let id = suite
        .scenarios
        .keys()
        .find(|id| id.to_string() == "demo.cases.Scorecard/aggregate")
        .cloned()
        .expect("the scorecard's scenario");
    let steps = suite.scenarios.remove(&id).expect("held");
    suite
        .scenarios
        .retain(|id, _| !matches!(id, ScenarioId::Aggregate { .. }));
    assert!(!ess_conformance::conditional_measures::used_by(&ir, &suite));
    suite.scenarios.insert(
        "demo.cases/authored/scorecard"
            .parse()
            .expect("an authored id"),
        steps,
    );
    assert!(ess_conformance::conditional_measures::used_by(&ir, &suite));
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/34").unwrap();
    let error = ess_conformance::conditional_measures::admit_for(&ir, &suite)
        .expect_err("an authored read of a conditioned view under /34");
    assert!(error.to_string().contains("suite/38"), "{error}");
}

#[test]
fn a_view_a_binding_changes_keeps_a_named_refusal() {
    let bound = format!(
        "{CASES}bindings:\n  - id: complete-on-open\n    when: {{event: demo.cases.Opened}}\n    invoke: \
         {{command: demo.cases.Complete}}\n    mapping: {{case_id: event.case_id}}\n    delivery: \
         at_least_once\n    on_failure: drop\n"
    );
    let result = synthesis(&bound);
    let refused = aggregate_refusals(&result);
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("complete-on-open") && refusal.contains("binding cut")),
        "{refused:#?}"
    );
}

#[test]
fn synthesis_is_deterministic() {
    assert_eq!(
        synthesis(CASES).suite.to_canonical_json().unwrap(),
        synthesis(CASES).suite.to_canonical_json().unwrap()
    );
}

// ---- execution ------------------------------------------------------------------------------

/// What each fault must fail: exactly these scenarios.
fn expected_failures(fault: Fault) -> BTreeSet<&'static str> {
    match fault {
        Fault::None => BTreeSet::new(),
        Fault::Drop(_) | Fault::Invert(_) => BTreeSet::from(["demo.cases.Escalations/aggregate"]),
        Fault::Swap => BTreeSet::from(["demo.cases.Scorecard/aggregate"]),
        Fault::WholeView | Fault::DropsZeroSelected => SCENARIOS.into_iter().collect(),
        Fault::StaleState => BTreeSet::from([
            "demo.cases.Scorecard/aggregate",
            "demo.cases.Urgent/aggregate",
        ]),
        Fault::FirstBranch => BTreeSet::from(["demo.cases.Urgent/aggregate"]),
        Fault::ForallForExists => BTreeSet::from(["demo.cases.Tagged/aggregate"]),
    }
}

fn failed(verdicts: &Verdicts) -> BTreeSet<String> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.clone())
        .collect()
}

fn native_verdicts() -> BTreeMap<Fault, Verdicts> {
    let suite = aggregate_suite();
    Fault::all()
        .into_iter()
        .map(|fault| {
            assert_eq!(Fault::of(&fault.name()), fault);
            (fault, run_target(&suite, &Store::new(fault)))
        })
        .collect()
}

#[test]
fn conditional_measure_all_six_functions_execute() {
    let mut wrong = Vec::new();
    for (fault, verdicts) in native_verdicts() {
        assert_eq!(verdicts.len(), SCENARIOS.len(), "{fault:?}: whole report");
        let wanted: BTreeSet<String> = expected_failures(fault)
            .into_iter()
            .map(str::to_owned)
            .collect();
        if failed(&verdicts) != wanted {
            wrong.push(format!(
                "{fault:?}: failed {:?}, expected {wanted:?}",
                failed(&verdicts)
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The native interpreter is the specification run as an implementation: it passes its own suite.
#[test]
fn conditional_measure_targets_detect_faults() {
    let ir = ir(CASES);
    let suite = aggregate_suite();
    let verdicts = run_target(&suite, &Interpreted::for_model(ir));
    assert_eq!(failed(&verdicts), BTreeSet::new(), "{verdicts:#?}");
    assert_eq!(verdicts.len(), SCENARIOS.len());
    // And every fault the store can be switched to fails exactly what it must.
    conditional_measure_all_six_functions_execute();
}

// ---- the native interpreter over arranged rows -------------------------------------------------

const ROWS: &str = r"format: ess/22
system: demo
version: v1
domain: demo.rows
entities:
  - name: demo.rows.Row
    identity: {name: id, type: Uuid}
    fields:
      - {name: team, type: String}
      - {name: cents, type: Integer}
      - {name: flag, type: Boolean}
      - {name: bonus, type: Optional<Integer>}
    lifecycle: {initial: Open, states: [Open, Done], terminal: [Done], transitions: [{name: finish, from: [Open], to: Done}]}
events:
  - {name: demo.rows.Made, fields: [{name: id, type: Uuid}]}
  - {name: demo.rows.Finished, fields: [{name: id, type: Uuid}]}
  - {name: demo.rows.Unflagged, fields: [{name: id, type: Uuid}]}
commands:
  - name: demo.rows.Make
    input: [{name: team, type: String}, {name: cents, type: Integer}, {name: flag, type: Boolean}]
    outcomes:
      - name: made
        creates: demo.rows.Row
        instance: id
        sets: {team: input.team, cents: input.cents, flag: input.flag}
        emits: [demo.rows.Made]
        payload: {demo.rows.Made: {id: {generated: true}}}
  - name: demo.rows.Finish
    input: [{name: id, type: Uuid}]
    outcomes:
      - name: finished
        moves: demo.rows.Row.finish
        instance: id
        emits: [demo.rows.Finished]
        payload: {demo.rows.Finished: {id: input.id}}
  - name: demo.rows.Unflag
    input: [{name: id, type: Uuid}]
    outcomes:
      - name: unflagged
        updates: demo.rows.Row
        instance: id
        sets: {flag: false}
        emits: [demo.rows.Unflagged]
        payload: {demo.rows.Unflagged: {id: input.id}}
views:
  - name: demo.rows.Membership
    source: demo.rows.Row
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: done, type: Integer, aggregate: {count: {}, where: state == Done}}
      - {name: flagged, type: Integer, aggregate: {count: {}, where: flag == true}}
  - name: demo.rows.Grouped
    source: demo.rows.Row
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: rows, type: Integer, aggregate: {count: {}}}
      - {name: flagged, type: Integer, aggregate: {count: {}, where: flag == true}}
      - {name: kinds, type: Integer, aggregate: {count_distinct: cents, where: flag == true}}
      - {name: cost, type: Integer, aggregate: {sum: cents, where: flag == true}}
      - {name: low, type: Optional<Integer>, aggregate: {min: cents, where: flag == true}}
      - {name: high, type: Optional<Integer>, aggregate: {max: cents, where: flag == true}}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: cents, where: flag == true}}
      - {name: bonuses, type: Optional<Integer>, aggregate: {sum: bonus, skip_absent: true, where: flag == true}}
  - name: demo.rows.Ungrouped
    source: demo.rows.Row
    consistency: read_your_writes
    fields:
      - {name: rows, type: Integer, aggregate: {count: {}}}
      - {name: flagged, type: Integer, aggregate: {count: {}, where: flag == true}}
      - {name: cost, type: Integer, aggregate: {sum: cents, where: flag == true}}
      - {name: low, type: Optional<Integer>, aggregate: {min: cents, where: flag == true}}
      - {name: mean, type: Optional<Decimal>, aggregate: {avg: cents, where: flag == true}}
  - name: demo.rows.Big
    source: demo.rows.Row
    consistency: read_your_writes
    params: [{name: floor, type: Integer}]
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: rows, type: Integer, aggregate: {count: {}}}
      - {name: over, type: Integer, aggregate: {count: {}, where: cents > param.floor}}
      - {name: over_cost, type: Integer, aggregate: {sum: cents, where: cents > param.floor}}
  - name: demo.rows.Bonused
    source: demo.rows.Row
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: rich, type: Integer, aggregate: {count: {}, where: bonus > 5}}
  - name: demo.rows.Known
    source: demo.rows.Row
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: with_bonus, type: Integer, aggregate: {count: {}, where: defined(bonus)}}
";

fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.rows/authored/rows".parse().unwrap(),
        ess_primitives::ids::CorrelationId::new("rows").unwrap(),
    )
}

fn interpreted(rows: &[(&str, &str, bool, Option<&str>, &str)]) -> Interpreted {
    let target = Interpreted::for_model(ir(ROWS));
    target.begin_scenario(&context()).unwrap();
    for (index, (team, cents, flag, bonus, state)) in rows.iter().enumerate() {
        let mut fields = BTreeMap::from([
            ("team".to_owned(), Node::Text((*team).to_owned())),
            ("cents".to_owned(), number(cents)),
            ("flag".to_owned(), Node::Bool(*flag)),
        ]);
        if let Some(bonus) = bonus {
            fields.insert("bonus".to_owned(), number(bonus));
        }
        target
            .establish_entity(EntitySetupRequest {
                entity: "demo.rows.Row".parse().unwrap(),
                identity: Node::Text(format!("00000000-0000-4000-8000-{index:012}")),
                fields,
                state: (*state).parse().unwrap(),
                correlation: context().correlation,
            })
            .unwrap_or_else(|error| panic!("{error:?}"));
    }
    target
}

fn read(
    target: &Interpreted,
    view: &str,
    params: &[(&str, Node)],
) -> Result<Vec<ViewRow>, TargetError> {
    target
        .query_view(SemanticViewRequest {
            view: view.parse().unwrap(),
            params: params
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .map(|result| result.rows)
}

fn row(pairs: &[(&str, Node)]) -> ViewRow {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

#[test]
fn conditional_measures_preserve_empty_and_optional_semantics() {
    // A group whose rows the condition refuses: zero counts and sum, absent extremes and mean, and
    // a skipping sum over no present value absent.
    let target = interpreted(&[
        ("a", "3", false, Some("2"), "Open"),
        ("a", "7", true, Some("4"), "Open"),
        ("a", "8", true, None, "Done"),
        ("b", "5", false, Some("1"), "Open"),
    ]);
    let rows = read(&target, "demo.rows.Grouped", &[]).unwrap();
    assert_eq!(
        rows,
        vec![
            row(&[
                ("team", Node::Text("a".into())),
                ("rows", number("3")),
                ("flagged", number("2")),
                ("kinds", number("2")),
                ("cost", number("15")),
                ("low", number("7")),
                ("high", number("8")),
                ("mean", number("7.5")),
                ("bonuses", number("4")),
            ]),
            row(&[
                ("team", Node::Text("b".into())),
                ("rows", number("1")),
                ("flagged", number("0")),
                ("kinds", number("0")),
                ("cost", number("0")),
                ("low", Node::Null),
                ("high", Node::Null),
                ("mean", Node::Null),
                ("bonuses", Node::Null),
            ]),
        ]
    );
    // An ungrouped view over no row: one row, every measure at its empty value.
    let empty = interpreted(&[]);
    assert_eq!(
        read(&empty, "demo.rows.Ungrouped", &[]).unwrap(),
        vec![row(&[
            ("rows", number("0")),
            ("flagged", number("0")),
            ("cost", number("0")),
            ("low", Node::Null),
            ("mean", Node::Null),
        ])]
    );
    // A condition over an absent value is unknown: the whole observation is undetermined, never a
    // partial row and never a false.
    assert!(read(&target, "demo.rows.Bonused", &[]).is_err());
    // Asked whether it is defined, the same absence is a known answer.
    assert_eq!(
        read(&target, "demo.rows.Known", &[]).unwrap(),
        vec![
            row(&[
                ("team", Node::Text("a".into())),
                ("with_bonus", number("2"))
            ]),
            row(&[
                ("team", Node::Text("b".into())),
                ("with_bonus", number("1"))
            ]),
        ]
    );
    // Exact integers past 2^53, and the existing overflow refusal where a selected sum leaves
    // `Integer`; a value the condition excludes overflows nothing.
    let big = interpreted(&[
        ("a", "9007199254740993", true, None, "Open"),
        ("a", "9007199254740993", true, None, "Open"),
        ("a", "9223372036854775807", false, None, "Open"),
    ]);
    let rows = read(&big, "demo.rows.Grouped", &[]).unwrap();
    assert_eq!(rows[0]["cost"], number("18014398509481986"));
    // A selected sum past `Integer` is what the unconditioned arithmetic makes of those values,
    // unchanged; a value the condition excludes adds nothing.
    let overflowing = interpreted(&[
        ("a", "9223372036854775807", true, None, "Open"),
        ("a", "1", true, None, "Open"),
        ("a", "9223372036854775807", false, None, "Open"),
    ]);
    let unconditioned = ess_conformance::aggregate::evaluate(
        ess_domain::view::AggregateFunction::Sum,
        &[number("9223372036854775807"), number("1")],
        ess_conformance::aggregate::ValueKind::Numeric,
    );
    assert_eq!(
        read(&overflowing, "demo.rows.Grouped", &[])
            .ok()
            .map(|rows| rows[0]["cost"].clone()),
        unconditioned
    );
}

#[test]
fn conditional_measure_parameters_are_view_inputs() {
    let target = interpreted(&[
        ("a", "3", false, None, "Open"),
        ("a", "7", true, None, "Open"),
        ("a", "12", true, None, "Open"),
    ]);
    let at = |floor: &str| read(&target, "demo.rows.Big", &[("floor", number(floor))]).unwrap();
    assert_eq!(at("5")[0]["over"], number("2"));
    assert_eq!(at("5")[0]["over_cost"], number("19"));
    assert_eq!(at("10")[0]["over"], number("1"));
    assert_eq!(
        at("10")[0]["rows"],
        number("3"),
        "a measure never filters the view"
    );
    // A value of another type is refused, as any parameter is.
    assert!(read(
        &target,
        "demo.rows.Big",
        &[("floor", Node::Text("x".into()))]
    )
    .is_err());
    assert!(
        read(&target, "demo.rows.Big", &[]).is_err(),
        "a required parameter is required"
    );
}

#[test]
fn conditional_measures_follow_membership_changes() {
    // A real transition and a real update move one row into and out of a measure.
    let target = Interpreted::for_model(ir(ROWS));
    target.begin_scenario(&context()).unwrap();
    let run = |command: &str, input: &[(&str, Node)]| {
        target
            .execute_command(SemanticCommandRequest {
                command: command.parse().unwrap(),
                actor: None,
                caller: None,
                input: input
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), value.clone()))
                    .collect(),
                correlation: context().correlation,
            })
            .unwrap_or_else(|error| panic!("{error:?}"))
    };
    let made = run(
        "demo.rows.Make",
        &[
            ("team", Node::Text("a".into())),
            ("cents", number("1")),
            ("flag", Node::Bool(true)),
        ],
    );
    let id = made.direct_events[0].payload["id"].clone();
    let measures = |target: &Interpreted| {
        let rows = target
            .query_view(SemanticViewRequest {
                view: "demo.rows.Membership".parse().unwrap(),
                params: BTreeMap::new(),
                consistency: ess_primitives::consistency::QueryConsistency::Current,
                correlation: context().correlation,
                deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
            })
            .unwrap()
            .rows;
        (rows[0]["done"].clone(), rows[0]["flagged"].clone())
    };
    assert_eq!(measures(&target), (number("0"), number("1")));
    run("demo.rows.Finish", &[("id", id.clone())]);
    assert_eq!(measures(&target), (number("1"), number("1")));
    run("demo.rows.Unflag", &[("id", id)]);
    assert_eq!(measures(&target), (number("1"), number("0")));
}

// ---- the generated Go and TypeScript runtimes -------------------------------------------------

fn scratch(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("conditional-aggregate-measures")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn run(command: &mut Command) -> std::process::Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{command:?}: {error}"));
    assert!(
        output.status.success(),
        "{command:?}\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn report_verdicts(path: &Path, log: &str) -> Verdicts {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("no report at {}: {error}\n{log}", path.display()));
    let document: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut out = Verdicts::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            out.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    out
}

fn assert_agrees(lane: &str, observed: &BTreeMap<Fault, Verdicts>) {
    let native = native_verdicts();
    let mut wrong = Vec::new();
    for (fault, verdicts) in &native {
        if observed.get(fault) != Some(verdicts) {
            wrong.push(format!(
                "{fault:?}: {lane} {:?}, native {:?}",
                observed.get(fault).map(failed),
                failed(verdicts)
            ));
        }
    }
    assert!(wrong.is_empty(), "{lane} disagrees:\n{}", wrong.join("\n"));
    println!(
        "{lane}: {} cases, every verdict the native runner's",
        native.len()
    );
}

#[test]
fn the_generated_go_runtime_agrees_with_the_native_runner() {
    let suite = aggregate_suite();
    let root = scratch("go");
    for artifact in ess_conformance::go::emit(suite.suite()).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/cases\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/cases_test.go"),
        include_str!("fixtures/conditional-aggregate-measures-runtime.go"),
    )
    .unwrap();
    let mut observed = BTreeMap::new();
    for fault in Fault::all() {
        let report = root.join(format!("report-{}.json", fault.name()));
        let output = Command::new("go")
            .args(["test", "./essconform", "-count=1", "-run", "TestCases"])
            .env("ESS_CASES_FAULT", fault.name())
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .env("GOWORK", "off")
            .env("GOFLAGS", "-mod=mod")
            .current_dir(&root)
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        observed.insert(fault, report_verdicts(&report, &log));
    }
    std::fs::remove_dir_all(&root).ok();
    assert_agrees("go", &observed);
}

#[test]
fn the_generated_typescript_runtime_agrees_with_the_native_runner() {
    let suite = aggregate_suite();
    let root = scratch("ts");
    for artifact in ess_conformance::ts::emit(suite.suite()).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    for (name, contents) in [
        (
            "target.mjs",
            include_str!("fixtures/conditional-aggregate-measures-runtime.mjs"),
        ),
        (
            "driver.mjs",
            include_str!("fixtures/typescript-parity-driver.mjs"),
        ),
        (
            "runtime-test.tsconfig.json",
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        ),
    ] {
        std::fs::write(dir.join(name), contents).unwrap();
    }
    run(Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&dir));
    let mut observed = BTreeMap::new();
    for fault in Fault::all() {
        let report = dir.join(format!("report-{}.json", fault.name()));
        let output = Command::new("node")
            .args(["--test", "driver.mjs"])
            .env("ESS_CASES_FAULT", fault.name())
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&dir)
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        observed.insert(fault, report_verdicts(&report, &log));
    }
    std::fs::remove_dir_all(&root).ok();
    assert_agrees("typescript", &observed);
}
