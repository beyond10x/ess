//! The generated Go and TypeScript suite runtimes give the reference runner's verdict, scenario for
//! scenario, on the suites the two row-set fixtures synthesize (`docs/design/filtered-related-reads.md`,
//! "Decisive acceptance"; beyond10x/ess#228, #299), against the healthy interpreter and against each
//! faulty implementation of `support_row_sets`.
//!
//! Each runtime replays one recorded transcript of the target's answers, so a verdict that differs
//! is a difference in how the suite was read, and every faulty target fails the scenarios
//! `support_row_sets::FAULT_KILLS` names in each runtime.

mod support_go;
mod support_row_sets;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::{
    synthesize::synthesize, AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner,
    RunnerConfig,
};
use support_row_sets::{model, Fault, Faulty, FAULT_KILLS, READS, UNIQUE};

fn suite(text: &str) -> ConformanceSuite {
    synthesize(&model(text)).suite
}

fn target(fault: Option<Fault>, reads: bool) -> Faulty {
    if reads {
        Faulty::new(fault)
    } else {
        Faulty::unique(fault)
    }
}

#[test]
fn row_sets_go_gives_the_reference_verdicts_for_the_healthy_and_each_faulty_target() {
    for (text, reads) in [(READS, true), (UNIQUE, false)] {
        let suite = suite(text);
        let label = format!("go-healthy-{reads}");
        let verdicts = support_go::assert_parity(&label, &suite, target(None, reads));
        assert_eq!(
            support_go::not_passed(&verdicts),
            Vec::<&str>::new(),
            "{label}: {verdicts:#?}"
        );
        for (fault, killed) in FAULT_KILLS {
            if Fault::READS.contains(fault) != reads {
                continue;
            }
            let label = format!("go-{fault:?}");
            let verdicts = support_go::assert_parity(&label, &suite, target(Some(*fault), reads));
            let failed = support_go::not_passed(&verdicts);
            for id in *killed {
                assert!(failed.contains(id), "{label}: {id} passed: {verdicts:#?}");
            }
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
        .join(format!("row-sets-ts-{label}-{}", std::process::id()));
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

/// The wall clock both runners are handed; no row-set scenario reads it.
const WALL_MS: u64 = 1_791_115_199_900;

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
        ess_conformance::now_offset::WithWall::new(AdvancingClock::default(), || {
            ess_primitives::time::Timestamp::from_epoch_millis(WALL_MS)
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
fn row_sets_typescript_gives_the_reference_verdicts_for_the_healthy_and_each_faulty_target() {
    for (text, reads) in [(READS, true), (UNIQUE, false)] {
        let suite = suite(text);
        let label = format!("ts-healthy-{reads}");
        let verdicts = typescript_parity(&label, &suite, target(None, reads));
        assert_eq!(
            support_go::not_passed(&verdicts),
            Vec::<&str>::new(),
            "{label}: {verdicts:#?}"
        );
        for (fault, killed) in FAULT_KILLS {
            if Fault::READS.contains(fault) != reads {
                continue;
            }
            let label = format!("ts-{fault:?}");
            let verdicts = typescript_parity(&label, &suite, target(Some(*fault), reads));
            let failed = support_go::not_passed(&verdicts);
            for id in *killed {
                assert!(failed.contains(id), "{label}: {id} passed: {verdicts:#?}");
            }
        }
    }
}
