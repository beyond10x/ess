//! The aggregate scenarios, run against an implementation and every mutant the page names.
//!
//! `docs/design/aggregate-views.md`, "The mutants it must kill" and "Deciding checks" 5. The target
//! below computes its aggregates itself — it never calls `ess_conformance::aggregate`, whose
//! defects it would otherwise share — and a `Mutant` switches in one defect at a time. Each mutant
//! must fail exactly the scenarios the page says catch it, and the implementation as specified must
//! pass all of them. The Go lane runs the same suite against a Go port of the target (correct, and
//! one mutant); the TypeScript lane must refuse the suite before any callback.
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
use ess_primitives::{facts::FactValue, node::Node};

const METRICS: &str = include_str!("fixtures/aggregate-views.yaml");
const BY_AGENT: &str = "metrics.session.TalkTimeByAgent/aggregate";
const TOTALS: &str = "metrics.session.QueueTotals/aggregate";
const BY_CHANNEL: &str = "metrics.session.SessionsByAgentChannel/aggregate";

fn suite() -> ConformanceSuite {
    let raw = RawSpecFile::parse(METRICS).unwrap();
    let spec = Specification::assemble([(Source::new("metrics.yaml"), raw)]).unwrap();
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
    synthesize(&ir).suite
}

/// One defect an implementation of the three views could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    IgnoresFilter,
    IgnoresKey,
    IgnoresSecondKey,
    DropsFirstRow,
    DropsLastRow,
    EmitsEmptyGroups,
    CountDistinctCountsRows,
    CountDistinctOverTalk,
    CountDistinctOverWait,
    CountDistinctOverKey,
    MinMaxSwapped,
    SumOverWrongInput,
    AvgTruncates,
    UngroupedEmptyHasNoRow,
    UngroupedEmptyMinIsZero,
}

type Row = BTreeMap<String, Node>;

struct Sessions {
    mutant: Mutant,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Sessions {
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

fn int(row: &Row, field: &str) -> i128 {
    match &row[field] {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("{field} is {other:?}"),
    }
}

fn number(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn count(n: usize) -> Node {
    number(&n.to_string())
}

/// The mean of integers to six places, half-even, spelled without trailing zeroes — or truncated.
fn mean(values: &[i128], truncate: bool) -> Node {
    if values.is_empty() {
        return Node::Null;
    }
    let (sum, n) = (values.iter().sum::<i128>(), values.len() as i128);
    let scaled = sum * 1_000_000;
    let (mut q, r) = (scaled / n, scaled % n);
    if !truncate && (2 * r > n || (2 * r == n && q % 2 == 1)) {
        q += 1;
    }
    let text = format!("{}.{:06}", q / 1_000_000, q % 1_000_000);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    number(text)
}

impl Sessions {
    fn admitted(&self, row: &Row, view: &str, params: &BTreeMap<String, Node>) -> bool {
        if self.mutant == Mutant::IgnoresFilter {
            return true;
        }
        let completed = row["state"] == Node::Text("Completed".into());
        match view {
            "metrics.session.QueueTotals" => {
                completed && Some(&row["queue_id"]) == params.get("queue_id")
            }
            _ => completed,
        }
    }

    /// The rows the view holds, before the mutant touches them, and after it drops one.
    fn source(&self, view: &str, params: &BTreeMap<String, Node>) -> Vec<Row> {
        let mut rows: Vec<Row> = self.rows.borrow().clone();
        let admitted = |row: &Row| self.admitted(row, view, params);
        let dropped = match self.mutant {
            Mutant::DropsFirstRow => rows.iter().position(admitted),
            Mutant::DropsLastRow => rows.iter().rposition(admitted),
            _ => None,
        };
        if let Some(index) = dropped {
            rows.remove(index);
        }
        rows
    }

    /// Groups in first-seen order: the first row of each (for its reported keys) and its admitted
    /// members.
    fn groups(&self, view: &str, params: &BTreeMap<String, Node>) -> Vec<(Row, Vec<Row>)> {
        let keys: Vec<&str> = match view {
            "metrics.session.TalkTimeByAgent" if self.mutant == Mutant::IgnoresKey => vec![],
            "metrics.session.TalkTimeByAgent" => vec!["agent_id"],
            "metrics.session.SessionsByAgentChannel" if self.mutant == Mutant::IgnoresSecondKey => {
                vec!["agent_id"]
            }
            "metrics.session.SessionsByAgentChannel" => vec!["agent_id", "channel"],
            _ => vec![],
        };
        let mut groups: Vec<(Vec<Node>, Row, Vec<Row>)> = Vec::new();
        for row in self.source(view, params) {
            let key: Vec<Node> = keys.iter().map(|k| row[*k].clone()).collect();
            let index = if let Some(index) = groups.iter().position(|(held, ..)| *held == key) {
                index
            } else {
                groups.push((key, row.clone(), Vec::new()));
                groups.len() - 1
            };
            if self.admitted(&row, view, params) {
                groups[index].2.push(row);
            }
        }
        groups
            .into_iter()
            .map(|(_, first, members)| (first, members))
            .collect()
    }

    fn view(&self, view: &str, params: &BTreeMap<String, Node>) -> Vec<Row> {
        let reported: Vec<&str> = match view {
            "metrics.session.TalkTimeByAgent" => vec!["agent_id"],
            "metrics.session.SessionsByAgentChannel" => vec!["agent_id", "channel"],
            _ => vec![],
        };
        let ungrouped = reported.is_empty();
        let mut groups = self.groups(view, params);
        if ungrouped && groups.is_empty() {
            groups.push((Row::new(), Vec::new()));
        }
        let mut out = Vec::new();
        for (first, members) in groups {
            if members.is_empty() && !ungrouped && self.mutant != Mutant::EmitsEmptyGroups {
                continue;
            }
            if members.is_empty() && ungrouped && self.mutant == Mutant::UngroupedEmptyHasNoRow {
                continue;
            }
            let mut row = self.computed(view, &members);
            for key in &reported {
                row.insert((*key).to_owned(), first[*key].clone());
            }
            out.push(row);
        }
        out
    }

    /// One group's aggregate fields.
    fn computed(&self, view: &str, members: &[Row]) -> Row {
        let column = |field: &str| members.iter().map(|m| int(m, field)).collect::<Vec<_>>();
        let mut row = Row::new();
        row.insert("sessions".into(), count(members.len()));
        let waits = column("wait_seconds");
        let extreme = if self.mutant == Mutant::MinMaxSwapped {
            waits.iter().min()
        } else {
            waits.iter().max()
        };
        let longest = match extreme {
            Some(value) => number(&value.to_string()),
            None if self.mutant == Mutant::UngroupedEmptyMinIsZero => number("0"),
            None => Node::Null,
        };
        let summed = if self.mutant == Mutant::SumOverWrongInput {
            "wait_seconds"
        } else {
            "talk_seconds"
        };
        let total = number(&column(summed).iter().sum::<i128>().to_string());
        match view {
            "metrics.session.TalkTimeByAgent" => {
                row.insert("talk_seconds".into(), total);
                row.insert("longest_wait".into(), longest);
                row.insert("distinct_callers".into(), count(self.distinct(members)));
                row.insert(
                    "mean_talk".into(),
                    mean(&column("talk_seconds"), self.mutant == Mutant::AvgTruncates),
                );
            }
            "metrics.session.SessionsByAgentChannel" => {
                row.insert("talk_seconds".into(), total);
            }
            _ => {
                row.insert("longest_wait".into(), longest);
            }
        }
        row
    }

    /// `count_distinct(caller)`, or what a mutant counts instead.
    fn distinct(&self, members: &[Row]) -> usize {
        let over = match self.mutant {
            Mutant::CountDistinctCountsRows => return members.len(),
            Mutant::CountDistinctOverTalk => "talk_seconds",
            Mutant::CountDistinctOverWait => "wait_seconds",
            Mutant::CountDistinctOverKey => "agent_id",
            _ => "caller",
        };
        let mut seen: Vec<&Node> = Vec::new();
        for member in members {
            if !seen.contains(&&member[over]) {
                seen.push(&member[over]);
            }
        }
        seen.len()
    }
}

impl ConformanceTarget for Sessions {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("sessions-fixture", "1"))
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
        let result = match command.to_string().as_str() {
            "metrics.session.Record" => {
                let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
                let mut row: Row = request.input.clone();
                row.insert("session_id".into(), Node::Text(id.clone()));
                row.insert("state".into(), Node::Text("Open".into()));
                rows.push(row);
                SemanticCommandResult::took(outcome(&command, "recorded")).emitting(
                    ObservedEvent::new("metrics.session.Recorded".parse().unwrap())
                        .with("session_id", Node::Text(id)),
                )
            }
            "metrics.session.Complete" => {
                let id = request.input["session_id"].clone();
                let Some(row) = rows.iter_mut().find(|row| row["session_id"] == id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                if row["state"] != Node::Text("Open".into()) {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                }
                row.insert("state".into(), Node::Text("Completed".into()));
                SemanticCommandResult::took(outcome(&command, "completed")).emitting(
                    ObservedEvent::new("metrics.session.Completed".parse().unwrap())
                        .with("session_id", id),
                )
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
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

/// The aggregate scenarios that do not pass against this target.
fn failing(mutant: Mutant) -> BTreeSet<String> {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Sessions::new(mutant))
        .into_report();
    let failed: BTreeSet<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

fn set(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

#[test]
fn the_views_as_specified_pass_their_own_suite() {
    assert_eq!(failing(Mutant::None), BTreeSet::new());
}

#[test]
fn every_mutant_the_page_names_fails_the_scenarios_the_page_says_catch_it() {
    let table = [
        (Mutant::IgnoresFilter, set(&[BY_AGENT, TOTALS, BY_CHANNEL])),
        (Mutant::IgnoresKey, set(&[BY_AGENT])),
        (Mutant::IgnoresSecondKey, set(&[BY_CHANNEL])),
        (Mutant::DropsFirstRow, set(&[BY_AGENT, TOTALS, BY_CHANNEL])),
        (Mutant::DropsLastRow, set(&[BY_AGENT, TOTALS, BY_CHANNEL])),
        (Mutant::EmitsEmptyGroups, set(&[BY_AGENT, BY_CHANNEL])),
        (Mutant::CountDistinctCountsRows, set(&[BY_AGENT])),
        (Mutant::CountDistinctOverTalk, set(&[BY_AGENT])),
        (Mutant::CountDistinctOverWait, set(&[BY_AGENT])),
        (Mutant::CountDistinctOverKey, set(&[BY_AGENT])),
        (Mutant::MinMaxSwapped, set(&[BY_AGENT, TOTALS])),
        (Mutant::SumOverWrongInput, set(&[BY_AGENT, BY_CHANNEL])),
        (Mutant::AvgTruncates, set(&[BY_AGENT])),
        (Mutant::UngroupedEmptyHasNoRow, set(&[TOTALS])),
        (Mutant::UngroupedEmptyMinIsZero, set(&[TOTALS])),
    ];
    let mut wrong = Vec::new();
    for (mutant, expected) in table {
        let failed = failing(mutant);
        if failed != expected {
            wrong.push(format!(
                "{mutant:?}: failed {failed:?}, expected {expected:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The generated Go runtime runs the suite against a Go port of the target: the implementation
/// passes, the mutant that ignores the filter fails, and the TypeScript runtime refuses the suite's
/// version before constructing a target.
#[test]
fn the_go_lane_runs_the_suite_and_the_typescript_lane_refuses_it_before_any_callback() {
    let suite = suite();
    let root = std::env::temp_dir().join(format!("ess-aggregate-views-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let docs = admission_documents(&suite, &root);
    run_go(&suite, &root, &docs);
    run_typescript(&suite, &root, &docs);
    std::fs::remove_dir_all(&root).ok();
}

/// The documents the admission cases read: the suite under the ordinary major before the construct,
/// and a refusal-only coverage suite under older coverage majors.
fn admission_documents(suite: &ConformanceSuite, root: &std::path::Path) -> std::path::PathBuf {
    let docs = root.join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let ordinary = AdmittedSuite::from_suite(suite).unwrap();
    std::fs::write(
        docs.join("ordinary-14.json"),
        ordinary
            .original_json()
            .replace("\"ess-conformance/16\"", "\"ess-conformance/14\""),
    )
    .unwrap();
    let unscoped = unscoped_coverage();
    for major in [11, 15, 17] {
        std::fs::write(
            docs.join(format!("coverage-{major}.json")),
            unscoped.replace(
                "\"ess-conformance/17\"",
                &format!("\"ess-conformance/{major}\""),
            ),
        )
        .unwrap();
    }
    docs
}

fn run_go(suite: &ConformanceSuite, root: &std::path::Path, docs: &std::path::Path) {
    for artifact in ess_conformance::go::emit(suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/sessions\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/sessions_test.go"),
        include_str!("fixtures/aggregate-views-runtime.go"),
    )
    .unwrap();
    for (mutant, passes) in [("", true), ("filter", false)] {
        let output = std::process::Command::new("go")
            .args(["test", "./essconform", "-count=1", "-v"])
            .env("ESS_AGGREGATE_DOCS", docs)
            .env("ESS_SESSIONS_MUTANT", mutant)
            .env("ESS_REPORT_FORMAT", "2")
            .env("GOWORK", "off")
            .current_dir(root)
            .output()
            .unwrap();
        let log = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.status.success(),
            passes,
            "go mutant={mutant:?}: {log}"
        );
        assert!(log.contains("--- PASS: TestAggregateAdmission"), "{log}");
        if !passes {
            assert!(log.contains("TalkTimeByAgent/aggregate"), "{log}");
        }
    }
}

fn run_typescript(suite: &ConformanceSuite, root: &std::path::Path, docs: &std::path::Path) {
    for artifact in ess_conformance::ts::emit(suite).unwrap() {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("sessions.mjs"),
        include_str!("fixtures/aggregate-views-runtime.mjs"),
    )
    .unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = std::process::Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    let output = std::process::Command::new("node")
        .args(["--test", "sessions.mjs"])
        .env("ESS_AGGREGATE_DOCS", docs)
        .env("ESS_REPORT_FORMAT", "2")
        .current_dir(&ts)
        .output()
        .unwrap();
    let log = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.status.success(), "{log}");
    assert!(
        log.contains("unsupported suite version \"ess-conformance/16\""),
        "{log}"
    );
    assert!(log.contains("constructed: 0"), "{log}");
    assert!(
        log.contains("aggregate-refusal-admission: refused"),
        "{log}"
    );
}

/// A refusal-only coverage suite carrying `ESS-SYNTH-016`: the fixture with one enum-keyed view.
fn unscoped_coverage() -> String {
    use ess_conformance::coverage::{Origins, Scope};
    let head = METRICS.split_once("views:\n").unwrap().0;
    let text = format!(
        "{head}views:\n  - name: metrics.session.ByChannel\n    source: metrics.session.Session\n    group_by: [channel]\n    fields:\n      - {{name: channel, type: metrics.session.Channel}}\n      - {{name: sessions, type: Integer, aggregate: {{count: {{}}}}}}\n"
    );
    let raw = RawSpecFile::parse(&text).unwrap();
    let spec = Specification::assemble([(Source::new("metrics.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let input = ess_conformance::coverage_build::build(&ir, &[], Scope::System, Origins::Generated)
        .unwrap();
    let original = input.selected().original_json().to_owned();
    assert!(original.contains("\"ess-conformance/17\""), "{original}");
    assert!(original.contains("ESS-SYNTH-016"), "{original}");
    original
}
