//! The Go and TypeScript runtimes give the reference verdicts on reads of a read-granted view
//! (beyond10x/ess#286).
//!
//! The synthesized suite of `docs/design/view-grants.example.yaml` reads `desk.tickets.Board` as
//! an actor after `read_as`, and files `…/grant/read/denied` and `…/grant/read/admitted/…`. Each
//! runtime runs it against the answers one Rust target gave the Rust reference runner, recorded
//! once: the interpreter, which refuses a read the grant does not admit, and the interpreter
//! serving every read. Every scenario's verdict is the reference's in Go and in TypeScript, the
//! actor each read is sent as is the one the reference runner sent, and only the denied read fails
//! the target that serves it.

mod support_go;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");
const DENIED: &str = "desk.tickets.Board/grant/read/denied";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("view-grants.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let suite = ess_conformance::synthesize(&ir()).suite;
    assert!(
        suite.scenarios.values().any(|scenario| scenario
            .steps
            .iter()
            .any(|step| matches!(step, ess_conformance::ScenarioStep::ReadAs { .. }))),
        "the suite reads as an actor"
    );
    suite
}

/// The interpreter, serving every read whoever it is read as: a surface that checks no read grant.
struct ServesEveryRead(Interpreted);

impl ConformanceTarget for ServesEveryRead {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("serves-every-read", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
}

/// The interpreter, refusing every read of the Board sent as an actor, granted or not: a surface whose
/// read grants dropped the Clerk (security review F1).
struct RefusesGrantedReader(Interpreted);

impl ConformanceTarget for RefusesGrantedReader {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("refuses-granted-reader", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() == "desk.tickets.Board" {
            return Err(TargetError::not_granted(Some(reader.to_string())));
        }
        self.0.query_view_as(request, reader)
    }
    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view_anonymous(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
}

/// The verdicts a target refusing the granted reader gets: every scenario reading the Board as the
/// Clerk fails, as `failed` and not `error`, and the denied reads pass.
fn assert_refused_reader_verdicts(verdicts: &BTreeMap<String, String>) {
    for (id, status) in verdicts {
        let expected = if id == DENIED { "passed" } else { "failed" };
        assert_eq!(status, expected, "`{id}`: {verdicts:?}");
    }
}

fn not_passed(verdicts: &BTreeMap<String, String>) -> Vec<&str> {
    support_go::not_passed(verdicts)
}

#[test]
fn go_gives_the_reference_verdicts_on_read_grants_for_an_honest_and_a_faulty_target() {
    let suite = suite();
    let honest =
        support_go::assert_parity("view-grant-honest", &suite, Interpreted::for_model(ir()));
    assert_eq!(not_passed(&honest), Vec::<&str>::new(), "{honest:?}");
    let faulty = support_go::assert_parity(
        "view-grant-faulty",
        &suite,
        ServesEveryRead(Interpreted::for_model(ir())),
    );
    assert_eq!(not_passed(&faulty), [DENIED], "{faulty:?}");
    let refusing = support_go::assert_parity(
        "view-grant-refuses-reader",
        &suite,
        RefusesGrantedReader(Interpreted::for_model(ir())),
    );
    assert_refused_reader_verdicts(&refusing);
}

// ---- TypeScript ------------------------------------------------------------------------------

/// Replays a recorded transcript (`support_go::Recorder`) as a TypeScript target, matching each
/// question by method and key in order, and refusing a request whose actor is not the one the
/// reference runner sent.
const DRIVER: &str = r"
import {readFileSync, writeFileSync} from 'node:fs';
import {runWith, unsupported} from './dist/runtime.js';
const [suiteFile, transcriptFile, divergenceFile] = process.argv.slice(2);
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key, actor, anonymous = false) => {
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
    const want = entry.request?.actor ?? null;
    if ((actor ?? null) !== want) {
      divergences.push(`${scenario}: ${method} \`${key}\` sent as ${actor ?? null}, the reference runner sent ${want}`);
      throw new Error('transcript divergence');
    }
    if (anonymous !== (entry.request?.anonymous === true)) {
      divergences.push(`${scenario}: ${method} \`${key}\` sent anonymous=${anonymous}, the reference runner did not`);
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
      const r = next('execute_command', request.command, request.actor === '' ? null : request.actor);
      return {
        outcome: r.outcome ?? undefined, error: r.error ?? undefined,
        consistency: r.consistency ?? undefined, directEvents: r.direct_events ?? [],
        response: r.response ?? undefined, notGranted: r.not_granted === true,
        notGrantedActor: r.not_granted_actor ?? undefined,
      };
    },
    queryView: async request => {
      const r = next('query_view', request.view, request.actor, request.anonymous === true);
      if (r.not_granted === true) return {notGranted: true, notGrantedActor: r.not_granted_actor ?? undefined};
      return {rows: r.rows ?? [], total: r.total ?? undefined};
    },
    observeEvents: async request => next('observe_events', request.event, undefined)
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

/// The TypeScript runtime package for `suite`, compiled.
fn typescript_package(label: &str, suite: &ConformanceSuite) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("view-grant-ts-{label}-{}", std::process::id()));
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

/// The TypeScript verdicts against `target`'s recorded answers, beside the Rust reference's.
fn typescript_parity<T: ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let rust = support_go::rust_outcomes_admitted(&admitted, &recorder);
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

#[test]
fn typescript_gives_the_reference_verdicts_on_read_grants_for_an_honest_and_a_faulty_target() {
    let suite = suite();
    let honest = typescript_parity("honest", &suite, Interpreted::for_model(ir()));
    assert_eq!(not_passed(&honest), Vec::<&str>::new(), "{honest:?}");
    let faulty = typescript_parity(
        "faulty",
        &suite,
        ServesEveryRead(Interpreted::for_model(ir())),
    );
    assert_eq!(not_passed(&faulty), [DENIED], "{faulty:?}");
    let refusing = typescript_parity(
        "refuses-reader",
        &suite,
        RefusesGrantedReader(Interpreted::for_model(ir())),
    );
    assert_refused_reader_verdicts(&refusing);
}

// ---- the browser -----------------------------------------------------------------------------

/// The browser coverage admission does not execute reads as an actor: a coverage suite carrying
/// `read_as` and the read-grant scenarios is refused before replay, as one carrying
/// `expect_not_granted` is, never replayed as though the read were anonymous.
#[test]
fn the_browser_admission_refuses_a_suite_reading_as_an_actor() {
    let input = ess_conformance::coverage_build::build(
        &ir(),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let envelope: serde_json::Value =
        serde_json::from_str(&input.document().to_canonical_json().unwrap()).unwrap();
    let document = envelope["suite_json"]
        .as_str()
        .expect("the coverage suite")
        .to_owned();
    assert!(document.contains("\"read_as\""), "{document}");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("view-grant-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("admission.js"),
        include_str!("../assets/coverage-admission.js"),
    )
    .unwrap();
    std::fs::write(root.join("input.json"), &document).unwrap();
    std::fs::write(
        root.join("harness.mjs"),
        r"import {readFileSync} from 'node:fs'
import {admitSuite} from './admission.js'
try { await admitSuite(readFileSync('input.json', 'utf8')); console.log('admitted') } catch (error) { console.log(String(error)) }
",
    )
    .unwrap();
    let output = Command::new("node")
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("node runs");
    let answer = String::from_utf8_lossy(&output.stdout).into_owned();
    // Refused at its first unknown construct: the read-grant ids, before the `read_as` steps.
    assert!(
        answer.contains("Invalid coverage replay") && !answer.contains("admitted"),
        "{answer}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&root);
}
