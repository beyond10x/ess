//! Shared harness for the wave 2026-10-08e unit e1 adversary cases (beyond10x/ess#499): a catalog
//! model with a configurable response, a target that returns a fixed response, and the native, Go
//! and TypeScript verdicts on one suite.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioResult,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

use crate::support_go;

pub const OUTCOME: &str = "catalog.items.Register/outcome/registered";

/// A catalog whose `Register` returns `response` (field lines) and declares `types` (type lines).
/// `emits` adds an event and payload mapping lines verbatim.
pub fn model(types: &str, response: &str, events: &str, emits: &str) -> String {
    format!(
        "format: ess/23
system: catalog
version: v1
domain: catalog.items

components:
  - component: catalog-service
    owns:
      domains: [catalog.items]
    accepts:
      commands: [catalog.items.Register]
    reached_by: network

types:
{types}{events}
commands:
  - name: catalog.items.Register
    input:
      - {{name: label, type: String}}
    response:
{response}    outcomes:
      - name: registered
        returns: true
{emits}"
    )
}

pub fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

pub fn refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
}

/// The synthesized suite, asserted to hold the outcome scenario.
pub fn synthesized(text: &str) -> ConformanceSuite {
    let result = synthesize(&ir(text));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == OUTCOME),
        "the outcome scenario is missing: {:#?}",
        refusals(&result)
    );
    result.suite
}

/// A catalog service returning one fixed response, and publishing `event` when set.
#[derive(Clone)]
pub struct Fixed {
    pub response: BTreeMap<String, Node>,
    pub event: Option<(&'static str, BTreeMap<String, Node>)>,
}

impl Fixed {
    pub fn new(response: serde_json::Value) -> Self {
        Self {
            response: serde_json::from_value(response).expect("a response map"),
            event: None,
        }
    }
    pub fn emitting(mut self, event: &'static str, payload: serde_json::Value) -> Self {
        self.event = Some((
            event,
            serde_json::from_value(payload).expect("a payload map"),
        ));
        self
    }
}

impl ConformanceTarget for Fixed {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("catalog-adversary-e1", "1"))
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
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new("registered").unwrap(),
        ));
        if let Some((event, payload)) = &self.event {
            let mut observed = ObservedEvent::new(event.parse().unwrap());
            for (key, value) in payload {
                observed = observed.with(key, value.clone());
            }
            result = result.emitting(observed);
        }
        result.response = Some(self.response.clone());
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            "view",
            "the catalog declares none",
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

pub fn native(suite: &ConformanceSuite, target: &Fixed) -> BTreeMap<String, ScenarioResult> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

pub fn native_status(suite: &ConformanceSuite, target: &Fixed) -> Status {
    native(suite, target)[OUTCOME].status
}

/// The Go verdict on the outcome, asserted equal to the native one for every scenario.
pub fn go_verdict(label: &str, suite: &ConformanceSuite, target: Fixed) -> String {
    let verdicts = support_go::assert_parity(label, suite, target);
    verdicts[OUTCOME].clone()
}

// ---- TypeScript, as the unit's own test drives it -------------------------------------------

const DRIVER: &str = r"
import {readFileSync, writeFileSync} from 'node:fs';
import {runWith, unsupported} from './dist/runtime.js';
const [suiteFile, transcriptFile, divergenceFile] = process.argv.slice(2);
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key) => {
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
      const r = next('execute_command', request.command);
      return {
        outcome: r.outcome ?? undefined, error: r.error ?? undefined,
        consistency: r.consistency ?? undefined, directEvents: r.direct_events ?? [],
        response: r.response ?? undefined,
      };
    },
    queryView: async request => {
      const r = next('query_view', request.view);
      return {rows: r.rows ?? [], total: r.total ?? undefined};
    },
    observeEvents: async request => next('observe_events', request.event)
      .map(event => ({event: event.event, payload: event.payload})),
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

pub fn scratch(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-e1-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// The TypeScript runtime package for `suite`, compiled; `edit` may rewrite a source file first.
pub fn typescript_package(
    label: &str,
    suite: &ConformanceSuite,
    edit: impl Fn(&str, String) -> String,
) -> PathBuf {
    let directory = scratch(&format!("ts-{label}"));
    for artifact in ess_conformance::ts::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = directory.join(&artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let contents = edit(&path.to_string_lossy(), artifact.contents.clone());
        std::fs::write(path, contents).unwrap();
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

/// Runs the compiled package against `target`'s recorded answers: the log and the report, if any.
pub fn typescript_run(
    package: &Path,
    suite: &ConformanceSuite,
    target: Fixed,
) -> (String, Option<serde_json::Value>) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let _ = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &recorder);
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
    let divergences = std::fs::read_to_string(&divergence).unwrap_or_default();
    assert_eq!(divergences, "", "{log}");
    let report = std::fs::read(&report)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap());
    (log, report)
}

/// The TypeScript verdicts for every scenario, or the log when no report was written.
pub fn typescript_verdicts(
    label: &str,
    suite: &ConformanceSuite,
    target: Fixed,
    edit: impl Fn(&str, String) -> String,
) -> Result<BTreeMap<String, String>, String> {
    let package = typescript_package(label, suite, edit);
    let (log, report) = typescript_run(&package, suite, target);
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    let Some(report) = report else {
        return Err(log);
    };
    let mut typescript = BTreeMap::new();
    for (status, ids) in report["outcomes"].as_object().expect("report/2 outcomes") {
        for id in ids.as_array().expect("ids") {
            typescript.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    Ok(typescript)
}

/// The native verdicts in report/2 spelling.
pub fn native_verdicts(suite: &ConformanceSuite, target: &Fixed) -> BTreeMap<String, String> {
    native(suite, target)
        .into_iter()
        .map(|(id, result)| (id, result.status.to_string()))
        .collect()
}
