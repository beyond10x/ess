//! One command answer can publish one declared event several times, and a suite that claims it
//! several times requires each occurrence (beyond10x/ess#427).
//!
//! From `ess-conformance/44` (ordinary) and `/45` (coverage), every `expect_event` after one command
//! claims a distinct, unclaimed occurrence of its event: the first unclaimed one that carries its
//! values, else the first unclaimed one, reported as `ESS-CF-PAYLOAD`. None left is `ESS-CF-EVENT`,
//! naming how many were published and how many the act claims. A suite is written at `/44` exactly
//! when one act claims one event more than once; every other suite keeps its bytes and its format,
//! and a suite at `/43` or below keeps first-match semantics.

mod support_go;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Source as AuthoredSource};
use ess_conformance::report::{CheckCode, Status};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::synthesize, AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner,
    RunnerConfig, ScenarioResult,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/event-multiplicity.yaml");
const REWRAPPED: &str = "demo.batch.Rewrap/outcome/rewrapped";
const EVENT: &str = "demo.batch.Rewrapped";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("batch.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesized() -> ConformanceSuite {
    let synthesis = synthesize(&ir(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    synthesis.suite
}

/// What a batch target publishes for one `Rewrap`.
#[derive(Debug, Clone)]
enum Publishes {
    /// `n` occurrences, each carrying the sent `batch_id`: three is the honest implementation.
    Copies(usize),
    /// One occurrence per listed `batch_id`, whatever was sent.
    Carrying(Vec<&'static str>),
}

/// A batch store answering `Rewrap` as [`Publishes`] says.
struct Batches {
    publishes: Publishes,
    serial: Cell<u64>,
}

impl Batches {
    fn new(publishes: Publishes) -> Self {
        Self {
            publishes,
            serial: Cell::new(0),
        }
    }
}

impl ConformanceTarget for Batches {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("batch-427", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command: &CommandRef = &request.command;
        if command.to_string() != "demo.batch.Rewrap" {
            return Err(TargetError::unsupported("command", command.to_string()));
        }
        self.serial.set(self.serial.get() + 1);
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new("rewrapped").unwrap(),
        ));
        let carried: Vec<Node> = match &self.publishes {
            Publishes::Copies(n) => vec![request.input["batch_id"].clone(); *n],
            Publishes::Carrying(values) => values.iter().map(|value| Node::from(*value)).collect(),
        };
        for batch_id in carried {
            result = result
                .emitting(ObservedEvent::new(EVENT.parse().unwrap()).with("batch_id", batch_id));
        }
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            "view",
            "the batch model declares none",
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "none"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("binding", "none"))
    }
}

fn run(suite: &ConformanceSuite, target: &Batches) -> BTreeMap<String, ScenarioResult> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

fn failed_codes(result: &ScenarioResult) -> Vec<CheckCode> {
    result
        .checks
        .iter()
        .filter(|check| check.status == Status::Failed)
        .map(|check| check.code)
        .collect()
}

/// How many steps of a scenario claim `EVENT`, by either event step.
fn claims(suite: &ConformanceSuite, id: &str) -> usize {
    let json = serde_json::to_value(&suite.scenarios).unwrap();
    json[id]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|step| {
            (step["step"] == "expect_event" || step["step"] == "expect_event_values")
                && step["event"] == EVENT
        })
        .count()
}

#[test]
fn repeated_emit_requires_each_occurrence() {
    let suite = synthesized();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/44",
        "a suite whose act claims one event three times is written at /44"
    );
    assert_eq!(claims(&suite, REWRAPPED), 3);
    let honest = run(&suite, &Batches::new(Publishes::Copies(3)));
    assert_eq!(
        honest[REWRAPPED].status,
        Status::Passed,
        "{:#?}",
        honest[REWRAPPED]
    );
    let once = run(&suite, &Batches::new(Publishes::Copies(1)));
    assert_eq!(
        once[REWRAPPED].status,
        Status::Failed,
        "{:#?}",
        once[REWRAPPED]
    );
    assert!(
        failed_codes(&once[REWRAPPED]).contains(&CheckCode::Event),
        "{:#?}",
        once[REWRAPPED]
    );
}

#[test]
fn emit_count_mutant_fails() {
    let suite = synthesized();
    let dropped = run(&suite, &Batches::new(Publishes::Copies(2)));
    assert_eq!(
        dropped[REWRAPPED].status,
        Status::Failed,
        "{:#?}",
        dropped[REWRAPPED]
    );
    assert_eq!(
        failed_codes(&dropped[REWRAPPED]),
        vec![CheckCode::Event],
        "exactly the third claim is unmet"
    );
    // One more than declared is not this rule's business: the model does not close the count.
    let extra = run(&suite, &Batches::new(Publishes::Copies(4)));
    assert_eq!(extra[REWRAPPED].status, Status::Passed);
}

const AUTHORED: &str = r"type: ess-scenario/1
domain: demo.batch
scenario: three-rewraps
summary: One rewrap publishes one occurrence per record.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.batch.Rewrap
    actor: demo.batch.Operator
    input: {batch_id: b-1}
    outcome: rewrapped
    events:
      - event: demo.batch.Rewrapped
        payload: {batch_id: b-1}
      - event: demo.batch.Rewrapped
        payload: {batch_id: b-1}
      - event: demo.batch.Rewrapped
        payload: {batch_id: b-1}
";
const AUTHORED_ID: &str = "demo.batch/authored/three-rewraps";

/// The synthesized suite with the authored scenarios beside it, as `ess verify conform` writes it.
fn with_authored(model: &str, documents: &[&str]) -> ConformanceSuite {
    let ir = ir(model);
    let mut suite = synthesize(&ir).suite;
    let sources: Vec<_> = documents
        .iter()
        .enumerate()
        .map(|(n, text)| AuthoredSource::new(format!("scenario-{n}.yaml"), *text))
        .collect();
    let authoring = compile_authored(&ir, &sources);
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).unwrap();
    }
    suite.select_fresh_format_for(&ir);
    suite
}

#[test]
fn authored_repeated_event_claims_count() {
    let suite = with_authored(MODEL, &[AUTHORED]);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/44"
    );
    assert_eq!(claims(&suite, AUTHORED_ID), 3);
    let once = run(&suite, &Batches::new(Publishes::Copies(1)));
    let result = &once[AUTHORED_ID];
    assert_eq!(result.status, Status::Failed, "{result:#?}");
    let unmet: Vec<_> = result
        .checks
        .iter()
        .filter(|check| check.status == Status::Failed)
        .collect();
    assert_eq!(unmet.len(), 2, "the second and third claims: {result:#?}");
    for check in unmet {
        assert_eq!(check.code, CheckCode::Event, "{check:#?}");
        let observed = check.diagnostic.as_ref().unwrap().observed.join("\n");
        assert!(
            observed.contains("1 occurrence(s) of demo.batch.Rewrapped published, 3 claimed"),
            "{observed}"
        );
    }
    let honest = run(&suite, &Batches::new(Publishes::Copies(3)));
    assert_eq!(
        honest[AUTHORED_ID].status,
        Status::Passed,
        "{:#?}",
        honest[AUTHORED_ID]
    );
}

#[test]
fn payload_mismatch_still_reports_payload() {
    let suite = with_authored(
        MODEL,
        &[&AUTHORED.replace(
            "      - event: demo.batch.Rewrapped\n        payload: {batch_id: b-1}\n      - event: demo.batch.Rewrapped\n        payload: {batch_id: b-1}\n",
            "",
        )],
    );
    assert_eq!(claims(&suite, AUTHORED_ID), 1);
    let wrong = run(&suite, &Batches::new(Publishes::Carrying(vec!["other"])));
    let result = &wrong[AUTHORED_ID];
    assert_eq!(result.status, Status::Failed, "{result:#?}");
    assert_eq!(
        failed_codes(result),
        vec![CheckCode::Payload],
        "{result:#?}"
    );
    // Still so in the /44 suite the synthesized scenario sits beside: one claim takes the one
    // occurrence, and its wrong value is a payload failure, not a missing event.
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/44"
    );
}

/// The repository's committed suites and every single-claim model keep their bytes and format.
#[test]
fn single_claim_suites_keep_their_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../suites/generated");
    for name in ["billing", "gatepass", "oracle-fixture"] {
        let text = std::fs::read_to_string(root.join(name).join("suite.json")).unwrap();
        let admitted = AdmittedSuite::from_json(&text).unwrap();
        let suite = admitted.suite();
        assert!(
            suite.provenance.suite_version.major() < 44,
            "{name}: {}",
            suite.provenance.suite_version
        );
        for id in suite.scenarios.keys() {
            let id = id.to_string();
            let json = serde_json::to_value(&suite.scenarios).unwrap();
            let mut since_command: BTreeMap<String, usize> = BTreeMap::new();
            for step in json[&id]["steps"].as_array().unwrap() {
                let kind = step["step"].as_str().unwrap();
                if kind.starts_with("execute_command") {
                    since_command.clear();
                } else if kind == "expect_event" || kind == "expect_event_values" {
                    let count = since_command
                        .entry(step["event"].as_str().unwrap().to_owned())
                        .or_default();
                    *count += 1;
                    assert_eq!(*count, 1, "{name}: {id} claims one event twice in one act");
                }
            }
        }
        // Re-serialised unchanged: the format pair did not move a byte of a single-claim suite.
        assert_eq!(suite.to_canonical_json().unwrap(), text, "{name}");
    }
    // The fixture model with one emission is written exactly as before the pair existed.
    let single = MODEL.replace(
        "emits: [demo.batch.Rewrapped, demo.batch.Rewrapped, demo.batch.Rewrapped]",
        "emits: [demo.batch.Rewrapped]",
    );
    let suite = synthesize(&ir(&single)).suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert_eq!(claims(&suite, REWRAPPED), 1);
}

/// Relabels a fresh suite to a legacy major, as a suite written before the pair carried it.
fn relabelled(suite: &ConformanceSuite, major: u32) -> ConformanceSuite {
    let mut document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    let provenance = document["provenance"].as_object_mut().unwrap();
    provenance.remove("scenario_initial_state");
    provenance.insert(
        "suite_version".into(),
        format!("ess-conformance/{major}").into(),
    );
    AdmittedSuite::from_json(&serde_json::to_string(&document).unwrap())
        .unwrap_or_else(|error| panic!("{error}"))
        .suite()
        .clone()
}

#[test]
fn older_reader_refuses_repeated_claim_suite() {
    let suite = synthesized();
    // A `/4` suite with repeated claims keeps first-match behaviour: one occurrence meets all three.
    let legacy = relabelled(&suite, 4);
    let once = run(&legacy, &Batches::new(Publishes::Copies(1)));
    assert_eq!(
        once[REWRAPPED].status,
        Status::Passed,
        "{:#?}",
        once[REWRAPPED]
    );
    let verdicts =
        support_go::assert_parity("legacy-4", &legacy, Batches::new(Publishes::Copies(1)));
    assert_eq!(verdicts[REWRAPPED], "passed");

    // A reader admitting through `/43` — the emitted Go runtime with the cap it had before the pair
    // — refuses the `/44` suite before any scenario, rather than under-checking it.
    let directory = support_go::package("older-reader", &suite, &[support_go::TRANSCRIPT_TARGET]);
    let runtime = directory.join("essconform/runtime.go");
    let text = std::fs::read_to_string(&runtime).unwrap();
    assert!(
        text.contains("const newestSuiteMajor = 45\n"),
        "the runtime reads /45"
    );
    std::fs::write(
        &runtime,
        text.replace(
            "const newestSuiteMajor = 45\n",
            "const newestSuiteMajor = 43\n",
        ),
    )
    .unwrap();
    let recorder = support_go::Recorder::new(Batches::new(Publishes::Copies(1)));
    let replayed = support_go::replay(&directory, &recorder, &[]);
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(!replayed.go.success, "{}", replayed.go.log);
    assert_eq!(replayed.go.outcomes.len(), 0, "{}", replayed.go.log);
    assert!(
        replayed.go.log.contains("ess-conformance/44"),
        "{}",
        replayed.go.log
    );
}

/// The declared-coverage counterpart is `/45`, admitted and executed with the reference verdicts.
#[test]
fn coverage_suite_with_repeated_claims_is_45() {
    use ess_conformance::coverage::{Origins, Scope};
    let input =
        ess_conformance::coverage_build::build(&ir(MODEL), &[], Scope::System, Origins::Generated)
            .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/45"
    );
    for (n, verdict) in [(3, "passed"), (1, "failed")] {
        let compared = support_go::compare_input(
            &format!("coverage-{n}"),
            &input,
            Batches::new(Publishes::Copies(n)),
            &support_go::Options::default(),
        );
        let verdicts = support_go::assert_compared(&format!("coverage-{n}"), compared);
        assert_eq!(verdicts[REWRAPPED], verdict, "{n} occurrence(s)");
    }
}

// ---- the three runners agree ------------------------------------------------------------------

/// The synthesized suite with authored acts that each claim two occurrences in an order.
fn distinct_payload_suite() -> ConformanceSuite {
    let act = |name: &str, claims: &[&str]| {
        let mut text = format!(
            "type: ess-scenario/1\ndomain: demo.batch\nscenario: {name}\nsummary: Claims in an order.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    command: demo.batch.Rewrap\n    actor: demo.batch.Operator\n    input: {{batch_id: b-1}}\n    outcome: rewrapped\n    events:\n"
        );
        for value in claims {
            write!(
                text,
                "      - event: demo.batch.Rewrapped\n        payload: {{batch_id: {value}}}\n"
            )
            .unwrap();
        }
        text
    };
    let documents = [
        act("x-then-y", &["x", "y"]),
        act("y-then-x", &["y", "x"]),
        act("x-twice", &["x", "x"]),
        act("x-y-x", &["x", "y", "x"]),
        act("only-y", &["y"]),
    ];
    let refs: Vec<&str> = documents.iter().map(String::as_str).collect();
    with_authored(MODEL, &refs)
}

/// Every scenario's verdict against occurrences carrying `x` then `y`.
const DISTINCT: [(&str, &str); 5] = [
    ("demo.batch/authored/x-then-y", "passed"),
    ("demo.batch/authored/y-then-x", "passed"),
    ("demo.batch/authored/x-twice", "failed"),
    ("demo.batch/authored/x-y-x", "failed"),
    ("demo.batch/authored/only-y", "passed"),
];

#[test]
fn distinct_payload_occurrences_agree_across_runners() {
    let suite = distinct_payload_suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/44"
    );
    let target = || Batches::new(Publishes::Carrying(vec!["x", "y"]));

    let rust = run(&suite, &target());
    for (id, verdict) in DISTINCT {
        let status = match rust[id].status {
            Status::Passed => "passed",
            Status::Failed => "failed",
            _ => "other",
        };
        assert_eq!(status, verdict, "Rust {id}: {:#?}", rust[id]);
    }
    assert_eq!(
        failed_codes(&rust["demo.batch/authored/x-twice"]),
        vec![CheckCode::Payload],
        "two claims of `x` against `x` and `y`: the second takes `y` and reports its value"
    );
    assert_eq!(
        failed_codes(&rust["demo.batch/authored/x-y-x"]),
        vec![CheckCode::Event],
        "three claims against two occurrences"
    );

    let go = support_go::assert_parity("distinct-payload", &suite, target());
    for (id, verdict) in DISTINCT {
        assert_eq!(go[id], verdict, "Go {id}");
    }
    let typescript = typescript_parity("distinct-payload", &suite, target());
    for (id, verdict) in DISTINCT {
        assert_eq!(typescript[id], verdict, "TypeScript {id}");
    }
}

#[test]
fn repeated_emit_verdicts_agree_across_runners() {
    let suite = synthesized();
    for (label, n, verdict) in [
        ("three", 3, "passed"),
        ("two", 2, "failed"),
        ("one", 1, "failed"),
    ] {
        let go = support_go::assert_parity(label, &suite, Batches::new(Publishes::Copies(n)));
        assert_eq!(go[REWRAPPED], verdict, "Go, {label}");
        let typescript = typescript_parity(label, &suite, Batches::new(Publishes::Copies(n)));
        assert_eq!(typescript[REWRAPPED], verdict, "TypeScript, {label}");
    }
}

// ---- TypeScript ------------------------------------------------------------------------------

/// Replays a recorded transcript (`support_go::Recorder`) as a TypeScript target, matching each
/// question by method and key in order.
const DRIVER: &str = r"
import {readFileSync, writeFileSync} from 'node:fs';
import {runWith, unsupported} from './dist/runtime.js';
const [suiteFile, transcriptFile, divergenceFile] = process.argv.slice(2);
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key, request) => {
    const matching = (transcript[scenario] ?? []).filter(e => e.method === method && e.key === key);
    const n = used.get(method + '\0' + key) ?? 0;
    used.set(method + '\0' + key, n + 1);
    const entry = matching[n];
    if (entry === undefined) {
      divergences.push(`${scenario}: ${method} \`${key}\` call ${n + 1} was never made by the reference runner`);
      throw new Error('transcript divergence');
    }
    if (entry.error === 'unsupported') throw unsupported('recorded');
    if (entry.error !== null) throw new Error(entry.error);
    return entry.result;
  };
  return {
    identity: async () => ({name: 'transcript', version: '1'}),
    beginScenario: async context => { scenario = context.scenario; used = new Map(); },
    endScenario: async () => {},
    executeCommand: async request => {
      const r = next('execute_command', request.command, request);
      return {
        outcome: r.outcome ?? undefined, error: r.error ?? undefined,
        consistency: r.consistency ?? undefined, directEvents: r.direct_events ?? [],
        response: r.response ?? undefined,
      };
    },
    queryView: async request => {
      const r = next('query_view', request.view, request);
      return {rows: r.rows ?? [], total: r.total ?? undefined};
    },
    observeEvents: async request => next('observe_events', request.event, request)
      .map(event => ({event: event.event, payload: event.payload})),
    establishEntity: async request => next('establish_entity', request.entity, request),
    configureExternalOutcome: async () => { throw unsupported('none is external'); },
    redeliverEvent: async () => { throw unsupported('not needed'); },
    observeInvocations: async () => { throw unsupported('not needed'); },
  };
};
const scope = {diagnostic() {}, skip() {}, async test(_name, body) { try { await body(this); } catch {} }};
try { await runWith(scope, target, readFileSync(suiteFile, 'utf8')); }
catch (error) { console.error(String(error)); process.exitCode = 2; }
writeFileSync(divergenceFile, divergences.join('\n'));
";

/// The TypeScript runtime package for `suite`, compiled.
fn typescript_package(label: &str, suite: &ConformanceSuite) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "event-multiplicity-ts-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for artifact in ess_conformance::ts::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = directory.join("essconform");
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(Path::new(&modules).join("@types"));
    }
    let output = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(package.join("transcript.mjs"), DRIVER).unwrap();
    package
}

/// The TypeScript verdicts against `target`'s recorded answers, asserted equal to the Rust
/// reference's, which are returned.
fn typescript_parity<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &recorder)
    .into_report();
    let rust: BTreeMap<String, String> = report
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
        .collect();
    let package = typescript_package(label, suite);
    let suite_file = package.join("suite.json");
    std::fs::write(&suite_file, admitted.original_json()).unwrap();
    let transcript = package.join("transcript.json");
    std::fs::write(&transcript, recorder.transcript().to_string()).unwrap();
    let divergence = package.join("divergence.txt");
    let report = package.join("report.json");
    let output = Command::new("node")
        .arg(package.join("transcript.mjs"))
        .arg(&suite_file)
        .arg(&transcript)
        .arg(&divergence)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(output.status.code(), Some(2), "{label}: {log}");
    let divergences = std::fs::read_to_string(&divergence).unwrap_or_default();
    assert_eq!(divergences, "", "{label}: {log}");
    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&report).unwrap_or_else(|error| panic!("{label}: {error}: {log}")),
    )
    .unwrap();
    let mut typescript = BTreeMap::new();
    for (status, ids) in report["outcomes"].as_object().expect("report/2 outcomes") {
        for id in ids.as_array().expect("ids") {
            typescript.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert_eq!(
        typescript, rust,
        "{label}: per-scenario verdicts, TypeScript (left) and Rust (right)\n{log}"
    );
    rust
}
