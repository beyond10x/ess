//! Generated runners preserve the native read-your-writes boundary when a command returns no
//! consistency token.
//!
//! The tokenless cases deliberately use assertions that would pass if a generated runner queried
//! at `Current`. Their verdict therefore proves that no weaker read was substituted. The snapshot
//! case separately preserves the native suite-error category, and the stale case proves that a
//! successful earlier read is not reused after a later tokenless command.

#[path = "support_one_time/fields.rs"]
#[allow(dead_code)]
mod disclosure_fields;
#[path = "support_one_time/resources.rs"]
#[allow(dead_code)]
mod disclosure_resources;
mod support_go;
#[allow(dead_code)]
mod support_one_time;
#[allow(dead_code)]
mod support_typescript_prerequisite;
mod support_versions;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioInitialState;
use ess_conformance::target::{
    ConformanceTarget, EntitySetupRequest, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use serde_json::{json, Value};
use support_one_time::{Mode, Service, FIRST};

const MISSING_EXPECT: &str = "example.call/authored/missing-token-expect";
const MISSING_SNAPSHOT: &str = "example.call/authored/missing-token-snapshot";
const WITH_TOKEN: &str = "example.call/authored/nonempty-token";
const NO_PRIOR_WRITE: &str = "example.call/authored/no-prior-write";
const EVENTUAL: &str = "example.call/authored/eventual-current";
const STALE: &str = "example.call/authored/stale-read-is-not-reused";
const RESET_BY_SETUP: &str = "example.call/authored/setup-clears-unreadable-view";

fn model(text: &str) -> ess_compiler::EssIr {
    let raw = RawSpecFile::parse(text).expect("the shared fixture parses");
    let spec = Specification::assemble([(Source::new("parity.yaml"), raw)])
        .expect("the shared fixture assembles");
    compile(&spec, &SourceMap::new()).expect("the shared fixture resolves")
}

fn current_document() -> String {
    serde_json::to_string(&json!({
        "provenance": {
            "suite_version": "ess-conformance/34",
            "system": "example",
            "specification_version": "v1",
            "spec_digest": "a".repeat(64),
            "contract_digest": "b".repeat(64),
            "scenario_initial_state": "empty"
        },
        "scenarios": {
            MISSING_EXPECT: {
                "purpose": "A tokenless write cannot be followed by a weaker current read",
                "steps": [
                    {"step": "execute_command", "command": "example.call.OfferCall"},
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "expect_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}}
                ],
                "source": []
            },
            MISSING_SNAPSHOT: {
                "purpose": "A snapshot without a consistent query remains a suite error",
                "steps": [
                    {"step": "execute_command", "command": "example.call.OfferCall"},
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "snapshot_subject", "view": "example.call.Calls", "subject": {
                        "call_id": {"kind": "literal", "value":
                            "00000000-0000-4000-8000-000000000001"}
                    }}
                ],
                "source": []
            },
            WITH_TOKEN: {
                "purpose": "A nonempty write token is demanded by exactly one immediate read",
                "steps": [
                    {"step": "execute_command", "command": "example.call.OpenSession", "input": {
                        "user_id": {"kind": "literal", "value": "user-1"}
                    }},
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "expect_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}}
                ],
                "source": []
            },
            NO_PRIOR_WRITE: {
                "purpose": "A view read before any write is a current read",
                "steps": [
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "expect_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}}
                ],
                "source": []
            },
            EVENTUAL: {
                "purpose": "An eventual read remains current after a tokenless command",
                "steps": [
                    {"step": "execute_command", "command": "example.call.OfferCall"},
                    {"step": "eventually_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}}
                ],
                "source": []
            },
            STALE: {
                "purpose": "A read before a tokenless command cannot satisfy the later assertion",
                "steps": [
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "expect_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}},
                    {"step": "execute_command", "command": "example.call.OfferCall"},
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "expect_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}}
                ],
                "source": []
            },
            RESET_BY_SETUP: {
                "purpose": "Successful setup clears a suppressed read before the next assertion",
                "steps": [
                    {"step": "execute_command", "command": "example.call.OfferCall"},
                    {"step": "query_view", "view": "example.call.Calls"},
                    {"step": "establish_entity", "instance": "established-call",
                     "entity": "example.call.Call",
                     "identity": "00000000-0000-4000-8000-000000000002",
                     "fields": {}, "state": "Open"},
                    {"step": "expect_view", "view": "example.call.Calls",
                     "expectation": {"expect": "counts", "at_least": 0}}
                ],
                "source": []
            }
        }
    }))
    .expect("the fixture serializes")
}

fn admitted_documents() -> Vec<(&'static str, AdmittedSuite)> {
    let current = current_document();
    let historical = support_versions::legacy_json(&current, 32);
    let current = AdmittedSuite::from_json(&current).expect("the current suite is admitted");
    let historical =
        AdmittedSuite::from_json(&historical).expect("the historical suite is admitted");
    assert_eq!(current.suite().provenance.suite_version.major(), 34);
    assert_eq!(
        current.suite().provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    assert_eq!(historical.suite().provenance.suite_version.major(), 32);
    assert_eq!(historical.suite().provenance.scenario_initial_state, None);
    vec![("current-34", current), ("historical-32", historical)]
}

fn expected() -> BTreeMap<String, String> {
    [
        (MISSING_EXPECT, "failed"),
        (MISSING_SNAPSHOT, "error"),
        (WITH_TOKEN, "passed"),
        (NO_PRIOR_WRITE, "passed"),
        (EVENTUAL, "passed"),
        (STALE, "failed"),
        (RESET_BY_SETUP, "error"),
    ]
    .map(|(scenario, status)| (scenario.to_owned(), status.to_owned()))
    .into()
}

#[derive(Default)]
struct Probe {
    current: RefCell<String>,
    queries: RefCell<BTreeMap<String, Vec<QueryConsistency>>>,
}

impl Probe {
    fn assert_queries(&self) {
        let queries = self.queries.borrow();
        assert_eq!(queries[MISSING_EXPECT], []);
        assert_eq!(queries[MISSING_SNAPSHOT], []);
        assert_eq!(
            queries[WITH_TOKEN],
            [QueryConsistency::at_least(
                ConsistencyToken::new("write").expect("a token")
            )]
        );
        assert_eq!(queries[NO_PRIOR_WRITE], [QueryConsistency::Current]);
        assert_eq!(queries[EVENTUAL], [QueryConsistency::Current]);
        assert_eq!(queries[STALE], [QueryConsistency::Current]);
        assert_eq!(queries[RESET_BY_SETUP], []);
    }
}

impl ConformanceTarget for Probe {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("read-your-writes-probe", "1"))
    }

    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        let scenario = context.scenario.to_string();
        self.current.replace(scenario.clone());
        self.queries.borrow_mut().insert(scenario, Vec::new());
        Ok(())
    }

    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let result = SemanticCommandResult::undeclared();
        if request.command.to_string() == "example.call.OpenSession" {
            return Ok(result.with_consistency(ConsistencyToken::new("write").expect("a token")));
        }
        assert_eq!(request.command.to_string(), "example.call.OfferCall");
        Ok(result)
    }

    fn establish_entity(&self, _: EntitySetupRequest) -> Result<(), TargetError> {
        Ok(())
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), "example.call.Calls");
        self.queries
            .borrow_mut()
            .get_mut(&*self.current.borrow())
            .expect("the scenario began")
            .push(request.consistency);
        Ok(SemanticViewResult::of(Vec::<
            BTreeMap<String, ess_primitives::node::Node>,
        >::new()))
    }

    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }

    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("the fixture declares no external outcome")
    }

    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("the fixture declares no binding")
    }
}

fn statuses(admitted: &AdmittedSuite, target: &Probe) -> BTreeMap<String, String> {
    Runner::for_suite(admitted.suite())
        .run_admitted(admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let status = match result.status {
                Status::Passed => "passed",
                Status::Failed => "failed",
                Status::Error => "error",
                Status::Unsupported => "unsupported",
            };
            (result.scenario.to_string(), status.to_owned())
        })
        .collect()
}

#[test]
fn native_runner_preserves_tokenless_and_current_read_boundaries() {
    for (label, admitted) in admitted_documents() {
        let target = Probe::default();
        assert_eq!(statuses(&admitted, &target), expected(), "{label}");
        target.assert_queries();
    }
}

#[test]
fn generated_go_runner_matches_native_verdicts_and_callback_suppression() {
    for (label, admitted) in admitted_documents() {
        let compared = support_go::compare(label, admitted.suite(), Probe::default());
        assert_eq!(support_go::assert_compared(label, compared), expected());
    }
}

#[test]
fn generated_go_request_comparison_detects_a_dropped_or_changed_token() {
    let (_, admitted) = admitted_documents().remove(0);
    for (label, replacement) in [("dropped", "\"\""), ("changed", "\"other\"")] {
        let recorder = support_go::Recorder::new(Probe::default());
        assert_eq!(
            support_go::rust_outcomes_admitted(&admitted, &recorder),
            expected()
        );
        let directory = support_go::package(
            &format!("read-your-writes-{label}"),
            admitted.suite(),
            &[support_go::TRANSCRIPT_TARGET],
        );
        let runtime = directory.join("essconform/runtime.go");
        let source = std::fs::read_to_string(&runtime).expect("the emitted Go runtime is readable");
        let needle = "AtLeast:     r.consistency,";
        assert_eq!(
            source.matches(needle).count(),
            1,
            "the mutation is decisive"
        );
        std::fs::write(
            &runtime,
            source.replace(needle, &format!("AtLeast:     {replacement},")),
        )
        .expect("the mutant writes");
        let replayed = support_go::replay(&directory, &recorder, &[]);
        assert!(
            replayed
                .divergences
                .iter()
                .any(|line| line.contains(WITH_TOKEN) && line.contains("at_least")),
            "{label}: the exact consistency mutation must be observed:\n{}\n{}",
            replayed.divergences.join("\n"),
            replayed.go.log
        );
        std::fs::remove_dir_all(directory).expect("the Go scratch package is removed");
    }
}

#[derive(Clone, Copy)]
enum TypeScriptMutation {
    None,
    DropToken,
    ChangeToken,
}

struct TypeScriptPackage {
    directory: PathBuf,
    input: PathBuf,
}

impl TypeScriptPackage {
    fn new(label: &str, admitted: &AdmittedSuite, mutation: TypeScriptMutation) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("read-your-writes-runtime-parity")
            .join(format!("{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut mutated = false;
        for artifact in ess_conformance::ts::emit(admitted.suite()).expect("the package emits") {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().expect("an artifact parent"))
                .expect("the artifact directory is created");
            let mut contents = artifact.contents;
            if !matches!(mutation, TypeScriptMutation::None)
                && path.file_name().is_some_and(|name| name == "runtime.ts")
            {
                let needle = "atLeast: this.consistency,";
                assert_eq!(
                    contents.matches(needle).count(),
                    1,
                    "the mutation is decisive"
                );
                let replacement = match mutation {
                    TypeScriptMutation::DropToken => "atLeast: '',",
                    TypeScriptMutation::ChangeToken => "atLeast: 'other',",
                    TypeScriptMutation::None => unreachable!(),
                };
                contents = contents.replace(needle, replacement);
                mutated = true;
            }
            std::fs::write(path, contents).expect("the artifact writes");
        }
        assert!(matches!(mutation, TypeScriptMutation::None) || mutated);
        let directory = root.join(ess_conformance::ts::PACKAGE);
        let driver = live_driver();
        for (name, contents) in [
            ("live.mjs", driver.as_str()),
            (
                "runtime-test.tsconfig.json",
                r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
            ),
        ] {
            std::fs::write(directory.join(name), contents).expect("the harness file writes");
        }
        let compiled = Command::new("tsc")
            .args(["--project", "runtime-test.tsconfig.json"])
            .current_dir(&directory)
            .output()
            .expect("the required TypeScript compiler runs");
        assert!(
            compiled.status.success(),
            "tsc refused the emitted package:\n{}{}",
            String::from_utf8_lossy(&compiled.stdout),
            String::from_utf8_lossy(&compiled.stderr)
        );
        let input = directory.join("suite.json");
        std::fs::write(&input, admitted.original_json()).expect("the admitted suite writes");
        Self { directory, input }
    }

    fn run(&self) -> (BTreeMap<String, String>, Vec<Value>) {
        let report = self.directory.join("report.json");
        let host = support_typescript_prerequisite::Host::start("read-your-writes", "correct");
        let output = Command::new("node")
            .arg("live.mjs")
            .arg(&self.input)
            .arg(&host.address)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&self.directory)
            .output()
            .expect("the required Node runtime runs");
        let trace = host.stop();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let document: Value =
            serde_json::from_slice(&std::fs::read(&report).unwrap_or_else(|error| {
                panic!("the TypeScript run wrote no report: {error}\n{log}")
            }))
            .expect("report/2 is JSON");
        let mut outcomes = BTreeMap::new();
        for (status, scenarios) in document["outcomes"]
            .as_object()
            .expect("report/2 carries outcomes")
        {
            for scenario in scenarios.as_array().expect("an outcome list") {
                outcomes.insert(
                    scenario.as_str().expect("a scenario id").to_owned(),
                    status.clone(),
                );
            }
        }
        assert_eq!(outcomes.len(), 7, "every scenario executed:\n{log}");
        (outcomes, trace)
    }
}

impl Drop for TypeScriptPackage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(
            self.directory
                .parent()
                .expect("the package has a scratch root"),
        );
    }
}

fn live_driver() -> String {
    const SOURCE: &str = include_str!("runtime_parity_typescript_28_35.rs");
    const START: &str = "const LIVE_DRIVER: &str = r\"\n";
    const END: &str = "\n\";\n\nfn live_run";
    let source = SOURCE
        .split_once(START)
        .expect("the shared live driver exists")
        .1;
    let driver = source
        .split_once(END)
        .expect("the shared live driver ends")
        .0;
    let needle = "response:value.Response??undefined,directEvents";
    assert_eq!(driver.matches(needle).count(), 1);
    let driver = driver.replace(
        needle,
        "response:value.Response??undefined,consistency:value.Consistency??'',directEvents",
    );
    let needle = "configureExternalOutcome:async request=>call('configure',upper(request)),";
    assert_eq!(driver.matches(needle).count(), 1);
    driver.replace(
        needle,
        "establishEntity:async request=>call('establish',upper(request)),\n configureExternalOutcome:async request=>call('configure',upper(request)),",
    )
}

fn traced_queries(trace: &[Value]) -> BTreeMap<String, Vec<String>> {
    let mut current = String::new();
    let mut queries = BTreeMap::<String, Vec<String>>::new();
    for entry in trace {
        match entry["method"].as_str() {
            Some("begin") => {
                entry["args"]["scenario"]
                    .as_str()
                    .expect("a scenario id")
                    .clone_into(&mut current);
                queries.insert(current.clone(), Vec::new());
            }
            Some("query") => queries
                .get_mut(&current)
                .expect("a query follows begin")
                .push(
                    entry["args"]["at_least"]
                        .as_str()
                        .expect("the consistency is traced")
                        .to_owned(),
                ),
            _ => {}
        }
    }
    queries
}

fn expected_queries() -> BTreeMap<String, Vec<String>> {
    [
        (MISSING_EXPECT, Vec::new()),
        (MISSING_SNAPSHOT, Vec::new()),
        (WITH_TOKEN, vec!["write".to_owned()]),
        (NO_PRIOR_WRITE, vec![String::new()]),
        (EVENTUAL, vec![String::new()]),
        (STALE, vec![String::new()]),
        (RESET_BY_SETUP, Vec::new()),
    ]
    .map(|(scenario, queries)| (scenario.to_owned(), queries))
    .into()
}

#[test]
fn generated_typescript_runner_matches_native_tokenless_verdicts() {
    for (label, admitted) in admitted_documents() {
        let package = TypeScriptPackage::new(label, &admitted, TypeScriptMutation::None);
        let (outcomes, trace) = package.run();
        assert_eq!(outcomes, expected(), "{label}");
        assert_eq!(traced_queries(&trace), expected_queries(), "{label}");
    }
}

#[test]
fn generated_typescript_trace_detects_a_dropped_or_changed_token() {
    let (_, admitted) = admitted_documents().remove(0);
    for (label, mutation, observed) in [
        ("dropped", TypeScriptMutation::DropToken, ""),
        ("changed", TypeScriptMutation::ChangeToken, "other"),
    ] {
        let package = TypeScriptPackage::new(label, &admitted, mutation);
        let (_, trace) = package.run();
        let queries = traced_queries(&trace);
        assert_eq!(queries[WITH_TOKEN], [observed], "{label}");
        assert_ne!(queries, expected_queries(), "{label} must be caught");
    }
}
