//! Calendar-window guards in the generated Go and TypeScript runtimes (beyond10x/ess#244 part b,
//! `docs/design/calendar-window-guards.md`).
//!
//! Two lanes. The Go reader answers the shared vectors
//! `crates/specify/ess-primitives/tests/vectors/calendar-window.json`, and holds every paired fault to
//! disagree with one of them (`predicate.test.ts` does the same in TypeScript, run by
//! `tests/typescript_runtime.rs`). And the Go and TypeScript suite runtimes give the reference
//! runner's verdict, scenario for scenario, on the suite synthesized from
//! `tests/fixtures/calendar-windows.yaml`, against the healthy interpreter and each faulty target:
//! each runtime replays one recorded transcript of the interpreter's answers, so a verdict that
//! differs is a difference in how the suite was read.

mod support_go;
mod support_occurrence_clock;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::interpret::Interpreted;
use ess_conformance::{
    now_offset, synthesize::synthesize, AdmittedSuite, AdvancingClock, ConformanceSuite, Ids,
    Runner, RunnerConfig,
};
use ess_primitives::time::Timestamp;
use support_occurrence_clock::model;

const RELEASES: &str = include_str!("fixtures/calendar-windows.yaml");

/// A wall clock both runners resolve any `now_offset` against: 2026-10-04T11:59:59.900Z.
const WALL_MS: u64 = 1_791_115_199_900;

/// One fault written into the fixture's windows (as `tests/calendar_windows.rs` writes them), and
/// the scenarios it fails.
const FAULTS: [(&str, &[&str]); 4] = [
    (
        "host time",
        &[
            "demo.releases.Schedule/outcome/scheduled",
            "demo.releases.Maintain/outcome/allowed",
            "demo.releases.Promote/outcome/promoted",
        ],
    ),
    (
        "offset ignored",
        &[
            "demo.releases.Schedule/outcome/scheduled",
            "demo.releases.Maintain/outcome/allowed",
            "demo.releases.Promote/outcome/promoted",
        ],
    ),
    (
        "to inclusive",
        &[
            "demo.releases.Schedule/outcome/outside-hours",
            "demo.releases.Maintain/outcome/outside-maintenance",
            "demo.releases.Promote/outcome/not-ready",
        ],
    ),
    (
        "crossing on its own day",
        &["demo.releases.Maintain/outcome/allowed"],
    ),
];

/// The fixture with the fault `name` written in.
fn faulty(name: &str) -> String {
    let crossing =
        r#"window: {at: requested_at, days: [fri], from: "22:00", to: "02:00", offset: "-05:00"}"#;
    assert!(RELEASES.contains(crossing));
    match name {
        "host time" => RELEASES
            .replace(r#"offset: "+01:00""#, r#"offset: "-07:00""#)
            .replace(r#"offset: "-05:00""#, r#"offset: "-07:00""#),
        "offset ignored" => RELEASES
            .replace(r#"offset: "+01:00""#, "offset: Z")
            .replace(r#"offset: "-05:00""#, "offset: Z"),
        "to inclusive" => RELEASES
            .replace(r#"to: "16:00""#, r#"to: "16:01""#)
            .replace(r#"to: "02:00""#, r#"to: "02:01""#),
        "crossing on its own day" => RELEASES.replace(
            crossing,
            "any:\n            - window: {at: requested_at, days: [fri], from: \"22:00\", to: \"24:00\", offset: \"-05:00\"}\n            - window: {at: requested_at, days: [fri], from: \"00:00\", to: \"02:00\", offset: \"-05:00\"}",
        ),
        other => panic!("no fault {other}"),
    }
}

fn check(label: &str, verdicts: &BTreeMap<String, String>, killed: &[&str]) {
    let failed = support_go::not_passed(verdicts);
    assert!(verdicts.len() >= 8, "{label}: {verdicts:#?}");
    if killed.is_empty() {
        assert_eq!(failed, Vec::<&str>::new(), "{label}: {verdicts:#?}");
    }
    for id in killed {
        assert!(failed.contains(id), "{label}: {id} passed: {verdicts:#?}");
    }
}

// ---- the shared vectors in the generated Go reader --------------------------------------------

#[test]
fn window_the_generated_go_reader_answers_the_shared_vectors() {
    let document = serde_json::json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "demo",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"demo.releases/authored/window": {"purpose": "Read a window",
            "steps": [{"step": "expect_view", "view": "demo.releases.Releases",
                "expectation": {"expect": "satisfies", "predicate": "ready_at == ready_at"}}],
            "source": []}}
    })
    .to_string();
    let suite = AdmittedSuite::from_json(&document).expect("admitted");
    let directory = support_go::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-window-go-{}", std::process::id())),
    );
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/window\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join("essconform/calendar_window_test.go"),
        include_str!("fixtures/calendar-window.go"),
    )
    .expect("fixture");
    std::fs::write(
        directory.join("essconform/calendar-window.json"),
        include_str!("../../../specify/ess-primitives/tests/vectors/calendar-window.json"),
    )
    .expect("vectors");
    let result = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestCalendarWindow",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .expect("go runs");
    let printed = format!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    eprintln!("{printed}");
    assert!(
        result.status.success()
            && printed.contains("--- PASS: TestCalendarWindow ")
            && printed.contains("--- PASS: TestCalendarWindowFaults")
            && printed.contains("--- PASS: TestCalendarWindowSuiteFormat"),
        "Go window reader: {}",
        result.status
    );
}

// ---- Go ---------------------------------------------------------------------------------------

#[test]
fn window_go_gives_the_reference_verdicts_for_the_healthy_and_each_faulty_target() {
    let suite = synthesize(&model(RELEASES)).suite;
    let options = support_go::Options {
        wall_millis: Some(WALL_MS),
        ..support_go::Options::default()
    };
    let verdicts = support_go::assert_parity_with(
        "go-healthy",
        &suite,
        Interpreted::for_model(model(RELEASES)),
        &options,
    );
    check("go-healthy", &verdicts, &[]);
    for (fault, killed) in FAULTS {
        let label = format!("go-{fault}");
        let verdicts = support_go::assert_parity_with(
            &label,
            &suite,
            Interpreted::for_model(model(&faulty(fault))),
            &options,
        );
        check(&label, &verdicts, killed);
    }
}

// ---- TypeScript ------------------------------------------------------------------------------

#[test]
fn window_typescript_gives_the_reference_verdicts_for_the_healthy_and_each_faulty_target() {
    let suite = synthesize(&model(RELEASES)).suite;
    let verdicts = typescript_parity(
        "ts-healthy",
        &suite,
        Interpreted::for_model(model(RELEASES)),
    );
    check("ts-healthy", &verdicts, &[]);
    for (fault, killed) in FAULTS {
        let label = format!("ts-{}", fault.replace(' ', "-"));
        let verdicts = typescript_parity(
            &label,
            &suite,
            Interpreted::for_model(model(&faulty(fault))),
        );
        check(&label, &verdicts, killed);
    }
}

/// Replays a recorded transcript (`support_go::Recorder`) as a TypeScript target, matching each
/// question by method and key in order, with the runtime's wall clock fixed at the reference
/// runner's.
const DRIVER: &str = r"
import {readFileSync, writeFileSync} from 'node:fs';
import {runWith, unsupported, setWallNow} from './dist/runtime.js';
const [suiteFile, transcriptFile, divergenceFile, wall] = process.argv.slice(2);
setWallNow(() => Number(wall));
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key, request) => {
    const matching = (transcript[scenario] ?? []).filter(e => e.method === method && e.key === key);
    const n = used.get(method + '\0' + key) ?? 0;
    used.set(method + '\0' + key, n + 1);
    let entry = matching[n];
    if (entry === undefined && matching.length > 0 && ['query_view', 'observe_events'].includes(method)) {
      entry = matching[matching.length - 1];
    }
    if (entry === undefined) {
      divergences.push(`${scenario}: ${method} \`${key}\` call ${n + 1} was never made by the reference runner`);
      throw new Error('transcript divergence');
    }
    if (method === 'execute_command') {
      const sent = JSON.stringify(request.input ?? {}, Object.keys(request.input ?? {}).sort());
      const want = JSON.stringify(entry.request?.input ?? {}, Object.keys(entry.request?.input ?? {}).sort());
      if (sent !== want) {
        divergences.push(`${scenario}: ${method} \`${key}\` sent ${sent}, the reference runner sent ${want}`);
        throw new Error('transcript divergence');
      }
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
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("calendar-window-ts-{label}-{}", std::process::id()));
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
fn typescript_parity<T: ess_conformance::target::ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let report = Runner::new(
        RunnerConfig::default(),
        now_offset::WithWall::new(AdvancingClock::default(), || {
            Timestamp::from_epoch_millis(WALL_MS)
        }),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &recorder)
    .into_report();
    let rust: BTreeMap<String, String> = report
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
        .arg(WALL_MS.to_string())
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
