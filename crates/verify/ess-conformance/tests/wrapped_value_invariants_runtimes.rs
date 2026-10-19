//! The generated Go and TypeScript runtimes admit a suite/52 that carries a `value_invariants`
//! expectation and report the scenario observing one `unsupported`, naming it; every other
//! scenario keeps the reference runner's verdict
//! (`docs/design/wrapped-value-invariant-observations.md`).
//!
//! Each runtime replays one recorded transcript of the interpreter's answers, so a verdict that
//! differs is a difference in how the suite was read.

mod support_go;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r#"format: ess/24
system: demo
version: v1
domain: demo.tags
types:
  - {name: demo.tags.BoardId, kind: newtype, of: Uuid}
  - name: demo.tags.Tag
    kind: newtype
    of: String
    alphabet: "abcdefghijklmnopqrstuvwxyz0123456789.-"
    invariants: [value.count >= 1]
entities:
  - name: demo.tags.Board
    identity: {name: board_id, type: demo.tags.BoardId}
    fields:
      - {name: tags, type: 'List<demo.tags.Tag>'}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.tags.BoardOpened
    fields:
      - {name: board_id, type: demo.tags.BoardId}
commands:
  - name: demo.tags.OpenBoard
    input:
      - {name: tags, type: 'List<demo.tags.Tag>'}
    outcomes:
      - name: opened
        creates: demo.tags.Board
        instance: board_id
        sets: {tags: input.tags}
        emits: [demo.tags.BoardOpened]
        payload:
          demo.tags.BoardOpened: {board_id: {generated: true}}
views:
  - name: demo.tags.Boards
    source: demo.tags.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.tags.BoardId}
      - {name: tags, type: 'List<demo.tags.Tag>'}
"#;

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(ir);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/52"
    );
    synthesis.suite
}

fn wrapped(verdicts: &BTreeMap<String, String>) -> Vec<(&str, &str)> {
    verdicts
        .iter()
        .filter(|(id, _)| id.contains("/invariant-wrapped/"))
        .map(|(id, status)| (id.as_str(), status.as_str()))
        .collect()
}

/// The Rust verdicts, with every wrapped scenario reported `unsupported` instead: what a generated
/// runtime that admits and does not observe the expectation must answer.
fn expected_generated(rust: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    rust.iter()
        .map(|(id, status)| {
            let status = if id.contains("/invariant-wrapped/") {
                "unsupported".to_owned()
            } else {
                status.clone()
            };
            (id.clone(), status)
        })
        .collect()
}

#[test]
fn the_reference_runner_observes_the_wrapped_position() {
    let ir = ir();
    let suite = suite(&ir);
    let admitted = AdmittedSuite::from_suite(&suite).expect("admitted");
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    for scenario in &report.scenarios {
        assert_eq!(
            scenario.status,
            ess_conformance::report::Status::Passed,
            "{scenario:#?}"
        );
    }
    assert!(report.scenarios.iter().any(|scenario| scenario
        .scenario
        .to_string()
        .contains("/invariant-wrapped/")));
}

#[test]
fn go_admits_the_suite_and_reports_the_observation_unsupported() {
    let ir = ir();
    let suite = suite(&ir);
    let (rust, replayed) =
        support_go::compare("wrapped-invariants", &suite, Interpreted::for_model(ir));
    assert!(
        replayed.divergences.is_empty(),
        "{:#?}\n{}",
        replayed.divergences,
        replayed.go.log
    );
    assert_eq!(wrapped(&rust).len(), 1, "{rust:#?}");
    assert!(wrapped(&rust).iter().all(|(_, status)| *status == "passed"));
    assert_eq!(
        replayed.go.outcomes,
        expected_generated(&rust),
        "{}",
        replayed.go.log
    );
    assert!(
        replayed.go.log.contains("value_invariants"),
        "the skip names the expectation: {}",
        replayed.go.log
    );
}

/// A wrapped value-invariant id arrived in suite/52: below it the Go runtime refuses the id
/// itself, as the TypeScript runtime does, before any target is reached (review 2026-10-19, B9).
#[test]
fn go_refuses_a_wrapped_id_below_suite_52_before_any_target_is_reached() {
    let suite = suite(&ir());
    let directory = support_go::package(
        "wrapped-invariants-old-reader",
        &suite,
        &[support_go::TRANSCRIPT_TARGET],
    );
    support_go::rewrite_suite(&directory, |document| {
        document["provenance"]["suite_version"] = "ess-conformance/50".into();
    });
    let transcript = directory.join("transcript.json");
    std::fs::write(&transcript, "{}").unwrap();
    let go = support_go::go_test(
        &directory,
        "TestTranscriptReplay",
        &[
            ("ESS_TRANSCRIPT", transcript.to_str().unwrap()),
            (
                "ESS_TRANSCRIPT_DIVERGENCE",
                directory.join("divergence.txt").to_str().unwrap(),
            ),
        ],
    );
    assert!(
        go.log
            .contains("a wrapped value-invariant scenario requires suite/52 or /53"),
        "{}",
        go.log
    );
    assert!(go.outcomes.is_empty() && !go.success, "{}", go.log);
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
const skips = [];
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
    configureExternalOutcome: async () => { throw unsupported('none is external'); },
    redeliverEvent: async () => { throw unsupported('not needed'); },
    observeInvocations: async () => { throw unsupported('not needed'); },
  };
};
const scope = {
  diagnostic() {},
  skip(message) { skips.push(String(message)); },
  async test(_name, body) {
    const inner = {...this, skip(message) { skips.push(String(message)); }};
    try { await body(inner); } catch (error) { skips.push(String(error?.message ?? error)); }
  },
};
try { await runWith(scope, target, readFileSync(suiteFile, 'utf8')); }
catch (error) { console.error(String(error)); process.exitCode = 2; }
writeFileSync(divergenceFile, divergences.join('\n'));
process.stdout.write(skips.join('\n'));
";

/// The TypeScript runtime package for `suite`, compiled.
fn typescript_package(suite: &ConformanceSuite) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("wrapped-invariants-ts-{}", std::process::id()));
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

#[test]
fn typescript_admits_the_suite_and_reports_the_observation_unsupported() {
    let ir = ir();
    let suite = suite(&ir);
    let admitted = AdmittedSuite::from_suite(&suite).expect("admitted");
    let recorder = support_go::Recorder::new(Interpreted::for_model(ir));
    let rust = support_go::rust_outcomes_admitted(&admitted, &recorder);
    let package = typescript_package(&suite);
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
    assert_ne!(output.status.code(), Some(2), "{log}");
    assert_eq!(
        std::fs::read_to_string(&divergence).unwrap_or_default(),
        "",
        "{log}"
    );
    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&report).unwrap_or_else(|error| panic!("{error}: {log}")),
    )
    .unwrap();
    let mut typescript = BTreeMap::new();
    for (status, ids) in report["outcomes"].as_object().expect("report/2 outcomes") {
        for id in ids.as_array().expect("ids") {
            typescript.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert_eq!(wrapped(&rust).len(), 1, "{rust:#?}");
    assert_eq!(typescript, expected_generated(&rust), "{log}");
    assert!(
        log.contains("value_invariants"),
        "the skip names the expectation: {log}"
    );
}
