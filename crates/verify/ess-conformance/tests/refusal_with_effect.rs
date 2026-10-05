//! A refusal that declares its compensating change (`compensates: true`, ess/22,
//! beyond10x/ess#197, `docs/design/refusal-with-effect.md`), in the interpreter, in synthesis and in
//! the Rust, Go and TypeScript runners.
//!
//! The issue's `JoinOrder` model: `failed` is an `external:` refusal answering `Refused` that resets
//! the addressed order to `Offline` and writes its `failure`. Synthesis asserts the error, then the
//! row in `Offline`; a target that (a) refuses without resetting, (b) resets another row, (c) resets
//! and answers success, or (d) answers the error after an extra change fails that scenario in every
//! runner, and the interpreter passes it.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::{ScenarioStep, ViewExpectation};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/refusal-with-effect.yaml");

const FAILED: &str = "shop.order.JoinOrder/outcome/failed";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", synthesis.refusals);
    synthesis.suite
}

fn steps_of(suite: &ConformanceSuite, id: &str) -> Vec<ScenarioStep> {
    suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1
        .steps
        .clone()
}

// ---- the interpreter -----------------------------------------------------------------------------

use ess_conformance::interpret::execute::{execute, Externals, Step, Store};

fn name(text: &str) -> ess_domain::name::QualifiedName {
    text.parse().unwrap()
}

/// Places an order on `store`, returning the store after it and the new order's identity.
fn place(ir: &EssIr, store: &Store) -> (Store, Node) {
    let mut steps = execute(
        ir,
        store,
        &name("shop.order.PlaceOrder"),
        &BTreeMap::new(),
        &Externals::Withheld,
    )
    .unwrap();
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    let id = step.events[0].payload["order_id"].clone();
    (step.next, id)
}

fn join(ir: &EssIr, store: &Store, order: &Node, externals: &Externals) -> Step {
    let mut steps = execute(
        ir,
        store,
        &name("shop.order.JoinOrder"),
        &BTreeMap::from([
            ("order_id".to_owned(), order.clone()),
            ("reason".to_owned(), Node::Text("upstream down".into())),
        ]),
        externals,
    )
    .unwrap();
    assert_eq!(steps.len(), 1, "{steps:#?}");
    steps.remove(0)
}

#[test]
fn the_interpreter_resets_the_addressed_order_and_answers_refused() {
    let ir = ir();
    let (store, order) = place(&ir, &Store::default());
    let (store, other) = place(&ir, &store);
    let step = join(
        &ir,
        &store,
        &order,
        &Externals::Forced("failed".parse().unwrap()),
    );
    assert_eq!(
        step.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("shop.order.JoinOrder/failed"),
        "{step:#?}"
    );
    assert_eq!(
        step.error
            .as_ref()
            .map(|error| error.error.to_string())
            .as_deref(),
        Some("shop.order.Refused"),
        "{step:#?}"
    );
    assert_eq!(step.events.len(), 0, "{step:#?}");
    let entity = name("shop.order.Order");
    let reset = step.next.instance_typed(&entity, &order).unwrap();
    assert_eq!(reset.state.as_str(), "Offline");
    assert_eq!(reset.fields["failure"], Node::Text("upstream down".into()));
    let untouched = step.next.instance_typed(&entity, &other).unwrap();
    assert_eq!(untouched.state.as_str(), "Idle");
    assert!(!untouched.fields.contains_key("failure"));
}

#[test]
fn the_interpreter_answers_a_row_outside_the_move_with_the_wrong_state_branch() {
    let ir = ir();
    let (store, order) = place(&ir, &Store::default());
    let forced = Externals::Forced("failed".parse().unwrap());
    let reset = join(&ir, &store, &order, &forced);
    // `Offline` is where no move starts: the compensating branch is answered as the moving branch
    // of the same command would be, by `wrong_state:`, and changes nothing.
    let again = join(&ir, &reset.next, &order, &forced);
    assert_eq!(
        again.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("shop.order.JoinOrder/closed"),
        "{again:#?}"
    );
    assert_eq!(again.next, reset.next);
}

#[test]
fn the_interpreter_takes_the_accepting_branch_unforced() {
    let ir = ir();
    let (store, order) = place(&ir, &Store::default());
    let step = join(&ir, &store, &order, &Externals::Withheld);
    assert_eq!(
        step.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("shop.order.JoinOrder/joined")
    );
    let entity = name("shop.order.Order");
    assert_eq!(
        step.next
            .instance_typed(&entity, &order)
            .unwrap()
            .state
            .as_str(),
        "Joined"
    );
}

// ---- synthesis -----------------------------------------------------------------------------------

#[test]
fn synthesis_asserts_the_error_then_the_row_in_offline() {
    let suite = suite();
    let steps = steps_of(&suite, FAILED);
    let position = |find: &dyn Fn(&ScenarioStep) -> bool| {
        steps
            .iter()
            .position(find)
            .unwrap_or_else(|| panic!("{steps:#?}"))
    };
    let forced = position(
        &|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { force, .. } if force.outcome.as_str() == "failed"),
    );
    let refused = position(
        &|step| matches!(step, ScenarioStep::ExpectError { error, .. } if error.to_string() == "shop.order.Refused"),
    );
    let offline = position(&|step| {
        let (ScenarioStep::ExpectView { expectation, .. }
        | ScenarioStep::EventuallyView { expectation, .. }) = step
        else {
            return false;
        };
        matches!(expectation, ViewExpectation::Contains { fields }
            if fields.get("state").is_some_and(|state| format!("{state:?}").contains("Offline"))
                && fields.contains_key("failure"))
    });
    assert!(forced < refused && refused < offline, "{steps:#?}");
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectNoEvent { event } if event.to_string() == "shop.order.OrderJoined")),
        "{steps:#?}"
    );
}

/// The fixture with no view of the order: nothing can read the change back.
fn unviewed() -> String {
    let start = MODEL.find("views:\n").unwrap();
    let end = MODEL.find("components:\n").unwrap();
    format!("{}{}", &MODEL[..start], &MODEL[end..])
}

#[test]
fn a_compensating_branch_no_view_reads_back_is_refused_by_name_not_synthesized_unchecked() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&unviewed()));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == FAILED)
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        refused.iter().any(|text| text.contains("compensat")),
        "{:#?}",
        synthesis.refusals
    );
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == FAILED),
        "the scenario cannot fail a target that skips the change, so it is not written"
    );
}

#[test]
fn the_interpreter_passes_the_synthesized_suite() {
    let suite = suite();
    let verdicts = support_go::rust_outcomes(&suite, &Interpreted::for_model(ir()));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
    assert_eq!(verdicts[FAILED], "passed");
}

// ---- a hand-written target, healthy and wrong in one way each ------------------------------------

/// How the target answers the forced `failed` branch; every other request it answers correctly.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// (a) answers `Refused` and leaves the order as it was.
    RefusesWithoutReset,
    /// (b) answers `Refused` and resets some other order, never the addressed one.
    ResetsAnotherRow,
    /// (c) resets the order and answers the branch with no error.
    ResetsAndSucceeds,
    /// (d) resets the order, publishes `OrderJoined` as well, and answers `Refused`.
    ErrorAfterAnExtraChange,
    /// (c), from `Joined` only: correct from `Idle`, so only a scenario forcing the branch from
    /// every state its move starts from sees it.
    SucceedsFromJoined,
}

const MODES: [Mode; 6] = [
    Mode::Correct,
    Mode::RefusesWithoutReset,
    Mode::ResetsAnotherRow,
    Mode::ResetsAndSucceeds,
    Mode::ErrorAfterAnExtraChange,
    Mode::SucceedsFromJoined,
];

#[derive(Clone)]
struct Row {
    state: &'static str,
    failure: Option<String>,
}

struct Shop {
    mode: Mode,
    rows: std::cell::RefCell<BTreeMap<String, Row>>,
    forced: std::cell::RefCell<Option<String>>,
    published: std::cell::RefCell<Vec<ObservedEvent>>,
    sequence: std::cell::RefCell<u64>,
}

use ess_conformance::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};

impl Shop {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: std::cell::RefCell::new(BTreeMap::new()),
            forced: std::cell::RefCell::new(None),
            published: std::cell::RefCell::new(Vec::new()),
            sequence: std::cell::RefCell::new(0),
        }
    }

    fn took(command: &str, outcome: &str) -> SemanticCommandResult {
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            ess_compiler::refs::CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        result
    }

    fn refused(command: &str, outcome: &str, error: &str) -> SemanticCommandResult {
        let mut result = Self::took(command, outcome);
        result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
        result
    }

    fn event(&self, name: &str, order: &str) -> ObservedEvent {
        let mut event = ObservedEvent::new(name.parse().unwrap());
        event
            .payload
            .insert("order_id".to_owned(), Node::Text(order.to_owned()));
        self.published.borrow_mut().push(event.clone());
        event
    }

    fn reset(&self, order: &str, reason: &str) {
        let mut rows = self.rows.borrow_mut();
        let row = rows.get_mut(order).expect("a held order");
        row.state = "Offline";
        row.failure = Some(reason.to_owned());
    }

    fn join(&self, order: &str, reason: &str) -> SemanticCommandResult {
        const JOIN: &str = "shop.order.JoinOrder";
        let forced = self.forced.borrow_mut().take();
        let Some(held) = self.rows.borrow().get(order).cloned() else {
            return Self::refused(JOIN, "closed", "shop.order.Closed");
        };
        if forced.as_deref() == Some("failed") && held.state != "Offline" {
            return match self.mode {
                Mode::Correct => {
                    self.reset(order, reason);
                    Self::refused(JOIN, "failed", "shop.order.Refused")
                }
                Mode::RefusesWithoutReset => Self::refused(JOIN, "failed", "shop.order.Refused"),
                Mode::ResetsAnotherRow => {
                    let other = self
                        .rows
                        .borrow()
                        .iter()
                        .find(|(id, row)| id.as_str() != order && row.state != "Offline")
                        .map(|(id, _)| id.clone());
                    if let Some(other) = other {
                        self.reset(&other, reason);
                    }
                    Self::refused(JOIN, "failed", "shop.order.Refused")
                }
                Mode::ResetsAndSucceeds => {
                    self.reset(order, reason);
                    Self::took(JOIN, "failed")
                }
                Mode::ErrorAfterAnExtraChange => {
                    self.reset(order, reason);
                    let mut result = Self::refused(JOIN, "failed", "shop.order.Refused");
                    result
                        .direct_events
                        .push(self.event("shop.order.OrderJoined", order));
                    result
                }
                Mode::SucceedsFromJoined => {
                    self.reset(order, reason);
                    if held.state == "Joined" {
                        Self::took(JOIN, "failed")
                    } else {
                        Self::refused(JOIN, "failed", "shop.order.Refused")
                    }
                }
            };
        }
        if held.state != "Idle" {
            return Self::refused(JOIN, "closed", "shop.order.Closed");
        }
        self.rows.borrow_mut().get_mut(order).unwrap().state = "Joined";
        let mut result = Self::took(JOIN, "joined");
        result
            .direct_events
            .push(self.event("shop.order.OrderJoined", order));
        result
    }
}

impl ConformanceTarget for Shop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("refusal-with-effect", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.published.borrow_mut().clear();
        *self.forced.borrow_mut() = None;
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let text = |field: &str| {
            request
                .input
                .get(field)
                .and_then(Node::as_text)
                .unwrap_or_default()
                .to_owned()
        };
        match request.command.to_string().as_str() {
            "shop.order.PlaceOrder" => {
                let mut sequence = self.sequence.borrow_mut();
                *sequence += 1;
                let id = format!("00000000-0000-4000-8000-{:012}", *sequence);
                self.rows.borrow_mut().insert(
                    id.clone(),
                    Row {
                        state: "Idle",
                        failure: None,
                    },
                );
                let mut result = Self::took("shop.order.PlaceOrder", "placed");
                result
                    .direct_events
                    .push(self.event("shop.order.OrderPlaced", &id));
                Ok(result)
            }
            "shop.order.JoinOrder" => Ok(self.join(&text("order_id"), &text("reason"))),
            other => panic!("unexpected command {other}"),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .iter()
                .map(|(id, row)| {
                    let mut fields = BTreeMap::from([
                        ("order_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text(row.state.to_owned())),
                    ]);
                    if let Some(failure) = &row.failure {
                        fields.insert("failure".to_owned(), Node::Text(failure.clone()));
                    }
                    fields
                })
                .collect(),
            total: None,
        })
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        *self.forced.borrow_mut() = Some(control.force.outcome.to_string());
        Ok(())
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
}

/// The scenario that forces `failed` on an order resting in `Joined`, the second state `reset`
/// starts from.
const RESET_FROM_EVERY_SOURCE: &str =
    "shop.order.Order/transition/reset/by/shop.order.JoinOrder/failed";

/// What a mode must fail: nothing when healthy, and for every fault a scenario forcing the
/// compensating branch — the branch's own, or for a fault shown only from `Joined` the scenario
/// arranging that source.
fn check(label: &str, mode: Mode, verdicts: &BTreeMap<String, String>) {
    let failed = support_go::not_passed(verdicts);
    if mode == Mode::Correct {
        assert_eq!(failed, Vec::<&str>::new(), "{label}: {verdicts:#?}");
    } else {
        let id = if mode == Mode::SucceedsFromJoined {
            RESET_FROM_EVERY_SOURCE
        } else {
            FAILED
        };
        assert_eq!(verdicts[id], "failed", "{label}: {verdicts:#?}");
        // Scenarios that never force the branch pass against every mode.
        for id in [
            "shop.order.PlaceOrder/outcome/placed",
            "shop.order.JoinOrder/outcome/joined",
            "shop.order.JoinOrder/outcome/closed",
        ] {
            assert_eq!(verdicts[id], "passed", "{label}: {id}: {verdicts:#?}");
        }
    }
}

#[test]
fn the_rust_runner_fails_every_faulty_target_on_the_compensating_scenario() {
    let suite = suite();
    for mode in MODES {
        let verdicts = support_go::rust_outcomes(&suite, &Shop::new(mode));
        check(&format!("rust-{mode:?}"), mode, &verdicts);
    }
}

#[test]
fn the_go_runner_gives_the_reference_verdict_for_every_mode() {
    let suite = suite();
    for mode in MODES {
        let label = format!("go-{mode:?}").to_lowercase();
        let verdicts = support_go::assert_parity(&label, &suite, Shop::new(mode));
        check(&label, mode, &verdicts);
    }
}

// ---- TypeScript ------------------------------------------------------------------------------

/// Replays a recorded transcript (`support_go::Recorder`) as a TypeScript target, matching each
/// question by method and key in order, the external control included.
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
    configureExternalOutcome: async control => {
      next('configure_external_outcome', control.command + '/' + control.outcome, control);
    },
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
fn typescript_package(label: &str, suite: &ConformanceSuite) -> std::path::PathBuf {
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("refusal-effect-ts-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for artifact in ess_conformance::ts::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = directory.join("essconform");
    let mut compile = std::process::Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(std::path::Path::new(&modules).join("@types"));
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
fn typescript_parity(
    label: &str,
    suite: &ConformanceSuite,
    target: Shop,
) -> BTreeMap<String, String> {
    let admitted =
        ess_conformance::AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let rust = support_go::rust_outcomes_admitted(&admitted, &recorder);
    let package = typescript_package(label, suite);
    let suite_file = package.join("suite.json");
    std::fs::write(&suite_file, admitted.original_json()).unwrap();
    let transcript = package.join("transcript.json");
    std::fs::write(&transcript, recorder.transcript().to_string()).unwrap();
    let divergence = package.join("divergence.txt");
    let report = package.join("report.json");
    let output = std::process::Command::new("node")
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
fn the_typescript_runner_gives_the_reference_verdict_for_every_mode() {
    let suite = suite();
    for mode in MODES {
        let label = format!("ts-{mode:?}").to_lowercase();
        let verdicts = typescript_parity(&label, &suite, Shop::new(mode));
        check(&label, mode, &verdicts);
    }
}

// ---- mutation ------------------------------------------------------------------------------------

#[test]
fn the_mutation_audit_reaches_the_compensating_branch_and_its_suite_kills_it() {
    use ess_conformance::mutate::{self, MutantClass, Verdict};
    let mut texts = SourceMap::new();
    texts.insert("order.yaml".to_owned(), MODEL.to_owned());
    let documents = vec![(
        Source::new("order.yaml"),
        RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}")),
    )];
    let original = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| {
        panic!("{error:?}");
    });
    let mutants = mutate::mutants(&documents, MutantClass::ALL);
    let reaching: Vec<_> = mutants
        .iter()
        .filter(|mutant| {
            // `reset` is the one transition to `Offline`, and `failed` the one branch reporting
            // `Refused`: the compensating move's arrival and the compensating branch's error.
            (mutant.class == MutantClass::TransitionTo
                && mutant.change == "`to: Offline` becomes `to: Idle`")
                || (mutant.class == MutantClass::ErrorSwap
                    && mutant.change.starts_with("`error: shop.order.Refused`"))
        })
        .collect();
    assert_eq!(
        reaching.len(),
        2,
        "{:#?}",
        mutants
            .iter()
            .map(|mutant| &mutant.change)
            .collect::<Vec<_>>()
    );
    for mutant in reaching {
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        // Sending `reset` back to `Idle` leaves `Offline` unreachable, which the model refuses:
        // stillborn, never a survivor. Reporting another error is killed by the compensating
        // scenario's `expect_error`.
        let expected = match mutant.class {
            MutantClass::TransitionTo => Verdict::Stillborn,
            _ => Verdict::Killed,
        };
        assert_eq!(entry.verdict, expected, "{}: {entry:#?}", mutant.id);
    }
}
