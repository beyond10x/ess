//! Adversary pass 1 for beyond10x/ess#461: a refusal selected by `when_subject:` observes the
//! complete record from `ess/23`, and a refusal reading `state` is witnessed in each state it
//! claims.
//!
//! These cases drive the unit's own fixture and three variants of it: an earlier `state` refusal
//! that takes precedence in one of the states a later one claims, a `{field, equals}` refusal
//! beside a `wrong_state:` refusal, and the generated Go and TypeScript runtimes on the new
//! scenarios.
mod support_go;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::resolve::compile;
use ess_compiler::{ir::EssIr, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::OutcomeRef;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceScenario, ConformanceSuite, Ids, Runner,
    RunnerConfig, ScenarioStep, ScenarioValue,
};
use ess_domain::command::OutcomeName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/subject-fact-complete-refusal.yaml");

const UPDATE: &str = "demo.inst.UpdateInstance";
const SUSPEND: &str = "demo.inst.SuspendInstance";
const SEED_REFUSED: &str = "demo.inst.UpdateInstance/outcome/seed-change-refused";
const NOT_ACTIVE: &str = "demo.inst.UpdateInstance/outcome/not-active";
const ALREADY_GONE: &str = "demo.inst.SuspendInstance/outcome/already-gone";

fn at(text: &str, format: &str) -> String {
    let out = text.replace("format: ess/23\n", &format!("format: {format}\n"));
    assert!(out.starts_with(&format!("format: {format}\n")), "{format}");
    out
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the model");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("instance.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result.refusals
                )
            },
            |(_, scenario)| scenario,
        )
}

fn refusal_texts(result: &Synthesis) -> BTreeSet<String> {
    result
        .refusals
        .iter()
        .map(|refusal| format!("{refusal}"))
        .collect()
}

/// For each send of `command` required to take `outcome`: whether a complete snapshot of the
/// addressed record precedes it and a complete comparison follows it, before any other command.
fn sends(scenario: &ConformanceScenario, command_name: &str, outcome: &str) -> Vec<(bool, bool)> {
    let steps = &scenario.steps;
    let mut out = Vec::new();
    for (at, step) in steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != command_name {
            continue;
        }
        let Some(name @ ScenarioValue::Instance { .. }) = input.get("name") else {
            continue;
        };
        let required = steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        if required.as_deref() != Some(outcome) {
            continue;
        }
        let previous = steps[..at]
            .iter()
            .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .map_or(0, |found| found + 1);
        let next = steps[at + 1..]
            .iter()
            .position(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .map_or(steps.len(), |found| at + 1 + found);
        let snapshot = steps[previous..at].iter().any(|step| {
            matches!(step, ScenarioStep::SnapshotCompleteSubject { subject, .. }
                if subject.get("name") == Some(name))
        });
        let compared = steps[at + 1..next]
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. }));
        out.push((snapshot, compared));
    }
    out
}

fn snapshots(scenario: &ConformanceScenario) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. }))
        .count()
}

fn rust_verdicts(
    suite: &ConformanceSuite,
    target: &impl ConformanceTarget,
) -> BTreeMap<String, String> {
    support_go::rust_outcomes(suite, target)
}

/// What a scripted target gets wrong.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fault {
    /// Nothing: the interpreted model.
    None,
    /// Answers the named refusal and stores the sent description anyway, on a record held in the
    /// named state, or in any state.
    WritesOn(&'static str, Option<&'static str>),
    /// Answers the named refusal, and its views then show the record in the named state.
    MovesOn(&'static str, &'static str),
    /// Updates a record held in this state instead of refusing it `not-active`.
    UpdatesIn(&'static str),
}

/// The interpreted model, with one fault laid over what it answers and what its views show.
struct Target {
    inner: Interpreted,
    fault: Fault,
    states: RefCell<BTreeMap<Node, &'static str>>,
    written: RefCell<BTreeMap<Node, Node>>,
    shown: RefCell<BTreeMap<Node, &'static str>>,
}

impl Target {
    fn new(model: &str, fault: Fault) -> Self {
        Self {
            inner: Interpreted::for_model(ir(model)),
            fault,
            states: RefCell::new(BTreeMap::new()),
            written: RefCell::new(BTreeMap::new()),
            shown: RefCell::new(BTreeMap::new()),
        }
    }
}

impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.states.borrow_mut().clear();
        self.written.borrow_mut().clear();
        self.shown.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let name = request.input.get("name").cloned();
        let description = request.input.get("description").cloned();
        let mut result = self.inner.execute_command(request)?;
        let (Some(name), Some(taken)) = (name, result.outcome.clone()) else {
            return Ok(result);
        };
        let taken = taken.outcome.to_string();
        let moved = match taken.as_str() {
            "created" => Some("Active"),
            "suspended" => Some("Suspended"),
            "removed" => Some("Removed"),
            _ => None,
        };
        if let Some(state) = moved {
            self.states.borrow_mut().insert(name.clone(), state);
        }
        let held = self.states.borrow().get(&name).copied();
        match self.fault {
            Fault::WritesOn(outcome, state)
                if taken == outcome && state.is_none_or(|state| held == Some(state)) =>
            {
                if let Some(description) = description {
                    self.written.borrow_mut().insert(name, description);
                }
            }
            Fault::MovesOn(outcome, to) if taken == outcome => {
                self.shown.borrow_mut().insert(name, to);
            }
            Fault::UpdatesIn(state)
                if command.to_string() == UPDATE
                    && taken == "not-active"
                    && held == Some(state) =>
            {
                result.outcome = Some(OutcomeRef::new(
                    command,
                    OutcomeName::new("updated").expect("a name"),
                ));
                result.error = None;
                if let Some(description) = description {
                    self.written.borrow_mut().insert(name, description);
                }
            }
            _ => {}
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        let written = self.written.borrow();
        let shown = self.shown.borrow();
        for row in &mut result.rows {
            let Some(name) = row.get("name").cloned() else {
                continue;
            };
            if let Some(description) = written.get(&name) {
                if row.contains_key("description") {
                    row.insert("description".to_owned(), description.clone());
                }
            }
            if let Some(state) = shown.get(&name) {
                if row.contains_key("state") {
                    row.insert("state".to_owned(), Node::Text((*state).to_owned()));
                }
            }
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

/// The fixture with `removed-refused` (`state == Removed`) declared before `not-active`
/// (`state != Active`): declaration order gives `removed-refused` every removed record, so
/// `not-active` is taken only on a suspended one.
fn precedence() -> String {
    let model = replaced(
        MODEL,
        "  - {name: demo.inst.InstanceNotFound, fields: []}\n",
        "  - {name: demo.inst.InstanceNotFound, fields: []}\n  - {name: demo.inst.InstanceGone, fields: []}\n",
    );
    replaced(
        &model,
        "      - name: not-active\n",
        "      - name: removed-refused\n        when_subject:\n          predicate: state == Removed\n        error: demo.inst.InstanceGone\n      - name: not-active\n",
    )
}

#[test]
fn adv461_precedence_earlier_state_refusal_adds_no_synthesis_refusal() {
    let below = synthesis(&at(&precedence(), "ess/22"));
    let after = synthesis(&precedence());
    assert_eq!(
        refusal_texts(&after),
        refusal_texts(&below),
        "a valid model whose earlier `state == Removed` refusal answers every removed record gains \
         a synthesis refusal at ess/23: `not-active` is claimed in `Removed`, where declaration \
         order never takes it"
    );
}

#[test]
fn adv461_precedence_model_honest_target_passes_every_scenario() {
    let model = precedence();
    let result = synthesis(&model);
    let verdicts = rust_verdicts(&result.suite, &Target::new(&model, Fault::None));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "the interpreted model passes: {verdicts:#?}"
    );
    let found = sends(scenario(&result, NOT_ACTIVE), UPDATE, "not-active");
    assert!(
        !found.is_empty()
            && found
                .iter()
                .all(|(snapshot, compared)| *snapshot && *compared),
        "`not-active` is still observed completely: {found:#?}"
    );
}

#[test]
fn adv461_refusal_that_moves_the_record_fails() {
    let result = synthesis(MODEL);
    let verdicts = rust_verdicts(
        &result.suite,
        &Target::new(MODEL, Fault::MovesOn("seed-change-refused", "Suspended")),
    );
    assert!(
        support_go::not_passed(&verdicts).contains(&SEED_REFUSED),
        "a target that refuses the seed change and moves the record fails: {verdicts:#?}"
    );
}

#[test]
fn adv461_claimed_row_that_writes_while_refusing_fails_in_each_state() {
    let result = synthesis(MODEL);
    for state in ["Suspended", "Removed"] {
        let verdicts = rust_verdicts(
            &result.suite,
            &Target::new(MODEL, Fault::WritesOn("not-active", Some(state))),
        );
        assert!(
            support_go::not_passed(&verdicts).contains(&NOT_ACTIVE),
            "a target that refuses `not-active` on a {state} record and writes its description \
             fails: {verdicts:#?}"
        );
    }
}

/// `UpdateInstance` with `seed-change-refused` in the `{field, equals}` shape. The compiler, the IR
/// doc and the design note say a refusal selected by `when_subject: {predicate: …}` carries
/// `complete_refusal` from ess/23, and that validation refuses a `{field, equals}` refusal naming
/// no subject of its own, so that shape never yields one.
#[test]
fn adv461_field_equals_refusal_compiles_complete() {
    let model = replaced(
        MODEL,
        "        when_subject:\n          predicate: seed_digest != input.seed_digest\n",
        "        when_subject: {field: seed_digest, equals: locked}\n",
    );
    let raw = RawSpecFile::parse(&model).unwrap_or_else(|error| panic!("{error}"));
    let errors = Specification::assemble([(Source::new("instance.yaml"), raw)])
        .err()
        .unwrap_or_else(|| {
            panic!(
                "a `{{field, equals}}` refusal naming no subject of its own validates, so the docs \
                 that say only the predicate shape yields a complete refusal are wrong"
            )
        });
    let refused = errors.as_slice().iter().any(|error| {
        error.location
            == "command.demo.inst.UpdateInstance.outcomes.seed-change-refused.when_subject"
            && error.message
                == "a subject-state guard requires an existing moves or updates subject and input \
                    identity"
    });
    assert!(
        refused,
        "the `{{field, equals}}` refusal is refused at its `when_subject`: {errors}"
    );
}

/// `SuspendInstance` with `already-gone` (`state != Active`) declared before its move, beside its
/// `wrong_state:` refusal: both states it claims are states the move does not start from.
fn state_refusal_beside_wrong_state() -> String {
    replaced(
        MODEL,
        "      - name: suspended\n        moves: demo.inst.Instance.suspend\n",
        "      - name: already-gone\n        when_subject:\n          predicate: state != Active\n        error: demo.inst.InstanceNotActive\n      - name: suspended\n        moves: demo.inst.Instance.suspend\n",
    )
}

#[test]
fn adv461_state_refusal_beside_wrong_state() {
    let model = state_refusal_beside_wrong_state();
    for (format, expected) in [("ess/23", true), ("ess/22", false)] {
        let compiled = ir(&at(&model, format));
        let outcome = compiled
            .commands()
            .values()
            .find(|command| command.name.to_string() == SUSPEND)
            .and_then(|command| {
                command
                    .outcomes
                    .iter()
                    .find(|outcome| outcome.name.to_string() == "already-gone")
            })
            .expect("already-gone");
        assert_eq!(outcome.complete_refusal, expected, "{format}");
    }
    let below = synthesis(&at(&model, "ess/22"));
    let result = synthesis(&model);
    assert_eq!(
        refusal_texts(&result),
        refusal_texts(&below),
        "no new refusal"
    );
    let found = sends(scenario(&result, ALREADY_GONE), SUSPEND, "already-gone");
    assert!(
        found.len() == 2
            && found
                .iter()
                .all(|(snapshot, compared)| *snapshot && *compared),
        "`already-gone` is witnessed completely on a suspended and on a removed record: {found:#?}"
    );
    for (id, scenario) in &below.suite.scenarios {
        let id = id.to_string();
        if id.contains("refuses/demo.inst.SuspendInstance") {
            assert_eq!(
                snapshots(self::scenario(&result, &id)),
                snapshots(scenario),
                "{id}: the wrong-state family keeps its complete observation"
            );
        }
    }
    let honest = rust_verdicts(&result.suite, &Target::new(&model, Fault::None));
    assert_eq!(
        support_go::not_passed(&honest),
        Vec::<&str>::new(),
        "{honest:#?}"
    );
    let moves = rust_verdicts(
        &result.suite,
        &Target::new(&model, Fault::MovesOn("already-gone", "Active")),
    );
    assert!(
        support_go::not_passed(&moves).contains(&ALREADY_GONE),
        "a target that refuses `already-gone` and shows the record active fails: {moves:#?}"
    );
}

const FAULTS: [Fault; 4] = [
    Fault::None,
    Fault::WritesOn("seed-change-refused", None),
    Fault::WritesOn("not-active", Some("Suspended")),
    Fault::UpdatesIn("Suspended"),
];

fn killed(fault: Fault, verdicts: &BTreeMap<String, String>) {
    let failed = support_go::not_passed(verdicts);
    match fault {
        Fault::None => assert_eq!(failed, Vec::<&str>::new(), "{verdicts:#?}"),
        Fault::WritesOn("seed-change-refused", _) => {
            assert!(failed.contains(&SEED_REFUSED), "{fault:?}: {verdicts:#?}");
        }
        _ => assert!(failed.contains(&NOT_ACTIVE), "{fault:?}: {verdicts:#?}"),
    }
}

#[test]
fn adv461_go_runner_gives_the_reference_verdicts() {
    let suite = synthesis(MODEL).suite;
    for (index, fault) in FAULTS.into_iter().enumerate() {
        let label = format!("adv461-{index}");
        let verdicts = support_go::assert_parity(&label, &suite, Target::new(MODEL, fault));
        killed(fault, &verdicts);
    }
}

// ---- TypeScript, the transcript driver of `row_sets_runtimes.rs` ---------------------------------

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

const WALL_MS: u64 = 1_791_115_199_900;

fn typescript_package(label: &str, suite: &ConformanceSuite) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adv461-ts-{label}-{}", std::process::id()));
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

fn typescript_parity<T: ConformanceTarget>(
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
fn adv461_typescript_runner_gives_the_reference_verdicts() {
    let suite = synthesis(MODEL).suite;
    for (index, fault) in FAULTS.into_iter().enumerate() {
        let label = format!("adv461-{index}");
        let verdicts = typescript_parity(&label, &suite, Target::new(MODEL, fault));
        killed(fault, &verdicts);
    }
}

/// Each record `not-active` is sent to is arranged by `CreateInstance` and captured; the
/// description it was created with must differ from the description the refusal is sent, or a
/// target that refuses and stores the sent description leaves the record as it was, and the
/// complete comparison after the refusal cannot see the write.
#[test]
fn adv461_not_active_sends_a_description_that_differs_from_the_stored_one() {
    let result = synthesis(MODEL);
    let steps = &scenario(&result, NOT_ACTIVE).steps;
    let mut created: Option<ScenarioValue> = None;
    let mut stored: BTreeMap<String, ScenarioValue> = BTreeMap::new();
    let mut checked = Vec::new();
    for (at, step) in steps.iter().enumerate() {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.inst.CreateInstance" =>
            {
                created = input.get("description").cloned();
            }
            ScenarioStep::CaptureInstance { instance, .. } => {
                if let Some(description) = created.take() {
                    stored.insert(instance.to_string(), description);
                }
            }
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == UPDATE
                    && matches!(steps.get(at + 1), Some(ScenarioStep::ExpectOutcome { outcome })
                        if outcome.outcome.to_string() == "not-active") =>
            {
                let Some(ScenarioValue::Instance { instance }) = input.get("name") else {
                    continue;
                };
                checked.push((
                    instance.to_string(),
                    stored.get(&instance.to_string()).cloned(),
                    input.get("description").cloned(),
                ));
            }
            _ => {}
        }
    }
    assert_eq!(checked.len(), 2, "two arranged records: {checked:#?}");
    for (instance, held, sent) in &checked {
        assert!(
            held.is_some(),
            "{instance} was created in the scenario: {checked:#?}"
        );
        assert_ne!(
            held, sent,
            "{instance}: `not-active` is sent the description the record already holds, so a \
             target that refuses and stores it changes nothing the complete comparison can see: \
             {checked:#?}"
        );
    }
}
