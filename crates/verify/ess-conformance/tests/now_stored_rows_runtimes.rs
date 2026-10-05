//! The generated Go and TypeScript suite runtimes give the reference runner's verdict, scenario for
//! scenario, on the suite `tests/fixtures/now-stored-rows.yaml` synthesizes — stored and related
//! instants ordered against `now` (`docs/design/expression-family-source22.md`, A3; beyond10x/ess#244
//! part a, unit U5) — against the healthy interpreter and each of its clock faults.
//!
//! Each runtime replays one recorded transcript of the interpreter's answers, so a verdict that
//! differs is a difference in how the suite was read; every runtime is handed the same wall clock,
//! so each must resolve every `now_offset` to the same instant and send the same request, which the
//! replay compares.

mod support_go;
mod support_now_stored;
mod support_occurrence_clock;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::{
    now_offset, synthesize::synthesize, AdmittedSuite, AdvancingClock, ConformanceSuite, Ids,
    Runner, RunnerConfig,
};
use ess_primitives::time::Timestamp;
use support_now_stored::{Clocking, Staged, Steered, LEASES, WALL_MS};
use support_occurrence_clock::model;

fn suite() -> ConformanceSuite {
    // The refusals synthesis makes for this fixture are its fixed-instant control's too
    // (`tests/now_stored_rows_suite.rs`); none is about a row decided against `now`.
    let synthesis = synthesize(&model(LEASES));
    for refusal in &synthesis.refusals {
        let named = format!("{:?}", refusal.scenario);
        assert!(
            ![
                "renewed",
                "graced",
                "lapsed",
                "joined",
                "banned",
                "embargoed",
                "lent"
            ]
            .iter()
            .any(|outcome| named.contains(&format!("OutcomeName({outcome})"))),
            "{refusal:#?}"
        );
    }
    synthesis.suite
}

/// What each wiring of the clock fails: none for the healthy target, and for each fault at least
/// the scenarios its own control names (`tests/now_stored_rows_suite.rs`).
fn expected(clocking: Clocking) -> &'static [&'static str] {
    match clocking {
        Clocking::Healthy => &[],
        Clocking::SetupReused => &[
            "demo.leases.RenewLease/outcome/graced",
            "demo.leases.Join/outcome/joined",
            "demo.leases.Lend/outcome/embargoed",
            "demo.leases.Lend/outcome/lent",
        ],
        Clocking::Reread => &[
            "demo.leases.RenewLease/outcome/renewed",
            "demo.leases.RenewLease/outcome/graced",
            "demo.leases.Join/outcome/banned-from-joining",
            "demo.leases.Lend/outcome/banned",
            "demo.leases.Lend/outcome/embargoed",
        ],
        Clocking::Absent => &[
            "demo.leases.RenewLease/outcome/renewed",
            "demo.leases.Join/outcome/joined",
            "demo.leases.Lend/outcome/lent",
        ],
    }
}

fn check(label: &str, verdicts: &BTreeMap<String, String>, clocking: Clocking) {
    let failed = support_go::not_passed(verdicts);
    if clocking == Clocking::Healthy {
        assert_eq!(failed, Vec::<&str>::new(), "{label}: {verdicts:#?}");
    }
    for id in expected(clocking) {
        assert!(failed.contains(id), "{label}: {id} passed: {verdicts:#?}");
    }
}

const CLOCKINGS: [Clocking; 4] = [
    Clocking::Healthy,
    Clocking::SetupReused,
    Clocking::Reread,
    Clocking::Absent,
];

#[test]
fn a3_go_gives_the_reference_verdicts_for_the_healthy_and_each_faulty_clock() {
    let suite = suite();
    let options = support_go::Options {
        wall_millis: Some(WALL_MS),
        ..support_go::Options::default()
    };
    for clocking in CLOCKINGS {
        let label = format!("go-{clocking:?}");
        let verdicts = support_go::assert_parity_with(
            &label,
            &suite,
            Staged::new(model(LEASES), &suite, clocking),
            &options,
        );
        check(&label, &verdicts, clocking);
    }
    for (fault, killed) in support_now_stored::FAULT_KILLS {
        let label = format!("go-{fault:?}");
        let verdicts = support_go::assert_parity_with(
            &label,
            &suite,
            Steered::new(model(LEASES), Some(*fault)),
            &options,
        );
        let failed = support_go::not_passed(&verdicts);
        for id in *killed {
            assert!(failed.contains(id), "{label}: {id} passed: {verdicts:#?}");
        }
    }
}

// ---- TypeScript ------------------------------------------------------------------------------

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
        .join(format!("now-stored-ts-{label}-{}", std::process::id()));
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

#[test]
fn a3_typescript_gives_the_reference_verdicts_for_the_healthy_and_each_faulty_clock() {
    let suite = suite();
    for clocking in CLOCKINGS {
        let label = format!("ts-{clocking:?}");
        let verdicts =
            typescript_parity(&label, &suite, Staged::new(model(LEASES), &suite, clocking));
        check(&label, &verdicts, clocking);
    }
    for (fault, killed) in support_now_stored::FAULT_KILLS {
        let label = format!("ts-{fault:?}");
        let verdicts = typescript_parity(&label, &suite, Steered::new(model(LEASES), Some(*fault)));
        let failed = support_go::not_passed(&verdicts);
        for id in *killed {
            assert!(failed.contains(id), "{label}: {id} passed: {verdicts:#?}");
        }
    }
}
