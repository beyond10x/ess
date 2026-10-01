//! Adversary pass 2 for beyond10x/ess#265: actor grants checked on a served surface.
//!
//! Each case drives the implementation from a claim the unit made about itself:
//!
//! - `synthesize/grant.rs`: a denied scenario "is asked of a request that would otherwise have been
//!   accepted", and "nothing the send could have published may then appear anywhere in the
//!   target's log", so a surface that runs the command and refuses afterwards fails it.
//! - `ScenarioStep::ExpectNotGranted::unpublished` and ESS-AUTHOR-039's repair: what is listed
//!   "must not appear anywhere in the target's log after the refused send".
//! - `synthesize/grant.rs`: "Every granted actor sends each command it is granted at least once".
//! - The Go and TypeScript runtimes give the Rust runner's verdict for `expect_not_granted`.

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Source as AuthoredSource};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

// ---- models --------------------------------------------------------------------------------------

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml".to_owned(), text.to_owned());
    compile(&spec, &sources).unwrap_or_else(|error| panic!("{error:?}"))
}

fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{name}: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("{name}: {errors}"))
}

/// `Ping` is `Operator`'s; `Tally` is both actors'. Served over the network.
const DESK: &str = "format: ess/15
system: desk
version: v1
domain: desk.ops
events:
  - name: desk.ops.Pinged
    fields:
      - {name: id, type: String}
  - name: desk.ops.Tallied
    fields:
      - {name: id, type: String}
commands:
  - name: desk.ops.Ping
    input:
      - {name: id, type: String}
    outcomes:
      - name: pinged
        emits: [desk.ops.Pinged]
        payload:
          desk.ops.Pinged: {id: input.id}
  - name: desk.ops.Tally
    input:
      - {name: id, type: String}
    outcomes:
      - name: tallied
        emits: [desk.ops.Tallied]
        payload:
          desk.ops.Tallied: {id: input.id}
actors:
  - name: desk.ops.Operator
    may: [desk.ops.Ping, desk.ops.Tally]
  - name: desk.ops.Watcher
    may: [desk.ops.Tally]
components:
  - component: desk-service
    owns: {domains: [desk.ops]}
    accepts: {commands: [desk.ops.Ping, desk.ops.Tally]}
    publishes: {events: [desk.ops.Pinged, desk.ops.Tallied]}
    reached_by: network
";

/// [`DESK`] in `ess/16`, with `Tally` granted to `Operator` and `Watcher`, and one more actor,
/// `Clerk`, that carries an attribute and is granted `Ping` only. Neither command reads a caller.
const ATTRIBUTED_DESK: &str = "format: ess/16
system: desk
version: v1
domain: desk.ops
events:
  - name: desk.ops.Pinged
    fields:
      - {name: id, type: String}
  - name: desk.ops.Tallied
    fields:
      - {name: id, type: String}
commands:
  - name: desk.ops.Ping
    input:
      - {name: id, type: String}
    outcomes:
      - name: pinged
        emits: [desk.ops.Pinged]
        payload:
          desk.ops.Pinged: {id: input.id}
  - name: desk.ops.Tally
    input:
      - {name: id, type: String}
    outcomes:
      - name: tallied
        emits: [desk.ops.Tallied]
        payload:
          desk.ops.Tallied: {id: input.id}
actors:
  - name: desk.ops.Clerk
    attributes:
      - {name: desk_id, type: String}
    may: [desk.ops.Ping]
  - name: desk.ops.Operator
    may: [desk.ops.Ping, desk.ops.Tally]
  - name: desk.ops.Watcher
    may: [desk.ops.Tally]
components:
  - component: desk-service
    owns: {domains: [desk.ops]}
    accepts: {commands: [desk.ops.Ping, desk.ops.Tally]}
    publishes: {events: [desk.ops.Pinged, desk.ops.Tallied]}
    reached_by: network
";

// ---- a Rust target wrapping the reference interpreter --------------------------------------------

/// How the wrapped interpreter departs from the reference.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    /// None: the reference interpreter, as a control.
    Reference,
    /// Checks the grant only after the command ran: the command takes effect and publishes, and the
    /// answer is the standard refusal naming the actor it was sent as.
    ChecksAfterRunning,
    /// Refuses `Watcher` the `Tally` the specification grants it.
    RefusesWatcherTally,
}

struct Wrapped {
    inner: Interpreted,
    ir: EssIr,
    change: Change,
    /// What the interpreter published for commands the wrapper then answered as not granted.
    published_then_refused: RefCell<Vec<String>>,
}

impl Wrapped {
    fn new(ir: EssIr, change: Change) -> Self {
        Self {
            inner: Interpreted::for_model(ir.clone()),
            ir,
            change,
            published_then_refused: RefCell::default(),
        }
    }

    fn granted(&self, actor: Option<&ess_compiler::refs::ActorRef>, command: &str) -> bool {
        let Some(actor) = actor else {
            return true;
        };
        self.ir.actors().get(actor.name()).is_some_and(|declared| {
            declared
                .may
                .iter()
                .any(|granted| granted.name().to_string() == command)
        })
    }
}

impl ConformanceTarget for Wrapped {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        match self.change {
            Change::Reference => self.inner.execute_command(request),
            Change::ChecksAfterRunning => {
                if self.granted(request.actor.as_ref(), &request.command.to_string()) {
                    return self.inner.execute_command(request);
                }
                let actor = request
                    .actor
                    .clone()
                    .expect("only a named actor is ungranted");
                let mut ran = request;
                // Run as the interpreter's own authority, which it admits, and refuse afterwards.
                ran.actor = None;
                if let Ok(result) = self.inner.execute_command(ran) {
                    self.published_then_refused.borrow_mut().extend(
                        result
                            .direct_events
                            .iter()
                            .map(|event| event.event.to_string()),
                    );
                }
                Err(TargetError::not_granted(Some(actor.to_string())))
            }
            Change::RefusesWatcherTally => {
                if request.actor.as_ref().map(ToString::to_string).as_deref()
                    == Some("desk.ops.Watcher")
                    && request.command.to_string() == "desk.ops.Tally"
                {
                    return Err(TargetError::not_granted(Some("desk.ops.Watcher")));
                }
                self.inner.execute_command(request)
            }
        }
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
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

fn verdicts(suite: &ConformanceSuite, target: &Wrapped) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).expect("admits");
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn not_passed(verdicts: &BTreeMap<String, Status>) -> Vec<&str> {
    verdicts
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

// ---- 1. the denied scenario's template -------------------------------------------------------------

/// `synthesize/grant.rs` borrows a denied scenario's arrangement "so the refusal is asked of a
/// request that would otherwise have been accepted", and requires that nothing the send could have
/// published appears in the log. The template is chosen by "snapshots first, then fewest steps",
/// with no regard for which branch the borrowed send takes: for `AdmitVisitor` and `SignOutVisitor`
/// it is the one-step `…/outcome/wrong-state` scenario, a visit id nobody registered. A surface
/// that runs the command and only then refuses publishes nothing there, so it passes.
#[test]
fn a_surface_that_checks_the_grant_after_running_fails_every_gatepass_denied_scenario() {
    let ir = example("gatepass");
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite
        .scenarios
        .retain(|id, _| matches!(id, ScenarioId::Grant { .. }));
    assert_eq!(
        suite.scenarios.len(),
        3,
        "the three gatepass denied scenarios"
    );

    let control = verdicts(&suite, &Wrapped::new(ir.clone(), Change::Reference));
    assert!(
        not_passed(&control).is_empty(),
        "control: the reference passes: {control:#?}"
    );

    let late = Wrapped::new(ir, Change::ChecksAfterRunning);
    let verdicts = verdicts(&suite, &late);
    let passed: Vec<&str> = verdicts
        .iter()
        .filter(|(_, status)| **status == Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect();
    assert!(
        passed.is_empty(),
        "a surface that runs the command and refuses afterwards passes {passed:?}: their sends \
         borrow an arrangement in which the command would not have been accepted anyway \
         (published by the late refusals: {:?})",
        late.published_then_refused.borrow()
    );
}

// ---- 2. an occurrence already seen ----------------------------------------------------------------

/// An authored scenario: `Operator` pings `same`, then `Watcher` sends `Ping` with an input of
/// `second` and expects the refusal, listing `Pinged` under `no_events:`.
fn repeated_ping(second: &str) -> String {
    format!(
        "type: ess-scenario/4
domain: desk.ops
scenario: refused-repeat
summary: A refused ping publishes nothing.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: desk.ops.Ping
    actor: desk.ops.Operator
    input: {{id: same}}
    outcome: pinged
  - at: 2026-01-05T09:01:00Z
    command: desk.ops.Ping
    actor: desk.ops.Watcher
    input: {{id: {second}}}
    refused: not_granted
    no_events: [desk.ops.Pinged]
"
    )
}

fn authored_suite(ir: &EssIr, text: &str) -> ConformanceSuite {
    let authoring = compile_authored(ir, &[AuthoredSource::new("scenario.yaml", text)]);
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let mut suite = ess_conformance::synthesize(ir).suite;
    suite.scenarios = authoring.scenarios.into_iter().collect();
    suite
}

/// ESS-AUTHOR-039 tells the author to "list under `no_events:` what must not appear anywhere in
/// the target's log after the refused send", and the step's documentation says the same. All three
/// runners excuse an occurrence equal in event and payload to one seen before the send, so a
/// surface that runs a refused command whose event repeats an earlier one passes. The control — the
/// same scenario with a different input — fails, so the check itself runs.
#[test]
fn a_refused_send_that_repeats_an_earlier_occurrence_still_fails() {
    let ir = compiled(DESK);

    let control = authored_suite(&ir, &repeated_ping("other"));
    let late = Wrapped::new(ir.clone(), Change::ChecksAfterRunning);
    let differing = verdicts(&control, &late);
    assert_eq!(
        not_passed(&differing),
        ["desk.ops/authored/refused-repeat"],
        "control: a new occurrence fails the step: {differing:#?}"
    );
    let reference = verdicts(&control, &Wrapped::new(ir.clone(), Change::Reference));
    assert!(not_passed(&reference).is_empty(), "{reference:#?}");

    let repeated = authored_suite(&ir, &repeated_ping("same"));
    let late = Wrapped::new(ir.clone(), Change::ChecksAfterRunning);
    let verdicts = verdicts(&repeated, &late);
    assert_eq!(
        late.published_then_refused.borrow().as_slice(),
        ["desk.ops.Pinged"],
        "precondition: the refused send ran and published a second `Pinged`"
    );
    assert_eq!(
        not_passed(&verdicts),
        ["desk.ops/authored/refused-repeat"],
        "a surface that ran the refused `Ping` and published a second `Pinged {{id: same}}` passes \
         a step whose claim is that no `Pinged` appears in the log after the refused send"
    );
}

// ---- 3. a granted actor never exercised -------------------------------------------------------------

/// "Every granted actor sends each command it is granted at least once" is skipped for the whole
/// model as soon as any one actor carries an attribute, even for commands no attributed actor is
/// granted. `Watcher` is granted `Tally`, never sends it, and nothing says so: a surface whose grant
/// table drops it passes.
#[test]
fn a_granted_actor_is_exercised_when_an_unrelated_actor_carries_attributes() {
    let ir = compiled(ATTRIBUTED_DESK);
    let synthesis = ess_conformance::synthesize(&ir);
    let suite = synthesis.suite;
    let reference = verdicts(&suite, &Wrapped::new(ir.clone(), Change::Reference));
    assert!(
        not_passed(&reference).is_empty(),
        "control: the reference passes: {reference:#?}"
    );
    let mutant = verdicts(&suite, &Wrapped::new(ir, Change::RefusesWatcherTally));
    let notes: Vec<String> = synthesis.notes.iter().map(ToString::to_string).collect();
    assert!(
        !not_passed(&mutant).is_empty(),
        "a surface refusing `Watcher` the `Tally` it is granted passes all {} scenarios; \
         notes: {notes:#?}",
        mutant.len()
    );
}

// ---- 4. the Go and TypeScript runtimes ---------------------------------------------------------------

/// A Go target for [`DESK`] that runs every command, publishes into its log, and answers an
/// ungranted actor with the standard refusal afterwards. In `forwards-direct-events` mode it also
/// hands back, beside the refusal, the occurrence the run published.
const GO_TARGET: &str = r#"package essconform

import (
	"fmt"
	"os"
	"testing"
)

type grantDesk struct {
	log []ObservedEvent
	seq int
}

func (d *grantDesk) Identity() (Identity, error) { return Identity{Name: "grant-desk", Version: "1"}, nil }
func (d *grantDesk) BeginScenario(ScenarioContext) error { return nil }
func (d *grantDesk) EndScenario(ScenarioContext) error   { return nil }

func (d *grantDesk) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	var outcome, event string
	switch r.Command {
	case "desk.ops.Ping":
		outcome, event = "pinged", "desk.ops.Pinged"
	case "desk.ops.Tally":
		outcome, event = "tallied", "desk.ops.Tallied"
	default:
		return CommandResult{}, ErrUnsupported
	}
	d.seq++
	published := ObservedEvent{Event: event, Payload: map[string]Node{"id": r.Input["id"]}}
	d.log = append(d.log, published)
	granted := r.Actor == "" || r.Actor == "desk.ops.Operator" || (r.Actor == "desk.ops.Watcher" && r.Command == "desk.ops.Tally")
	if !granted {
		refused := CommandResult{NotGranted: true, NotGrantedActor: r.Actor}
		if os.Getenv("ESS_TARGET_MODE") == "forwards-direct-events" {
			refused.DirectEvents = []ObservedEvent{published}
		}
		return refused, nil
	}
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("seq:%d", d.seq), DirectEvents: []ObservedEvent{published}}, nil
}

func (d *grantDesk) QueryView(ViewRequest) (ViewResult, error) { return ViewResult{}, ErrUnsupported }

func (d *grantDesk) ObserveEvents(r EventObservationRequest) ([]ObservedEvent, error) {
	var found []ObservedEvent
	for _, e := range d.log {
		if e.Event == r.Event {
			found = append(found, e)
		}
	}
	return found, nil
}

func (d *grantDesk) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (d *grantDesk) RedeliverEvent(RedeliveryRequest) error                { return ErrUnsupported }
func (d *grantDesk) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestGrantDesk(t *testing.T) {
	Run(t, func() Target { return &grantDesk{} })
}
"#;

/// The same target in JavaScript, for the TypeScript runtime.
const TS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'refuses-after-running';

class Desk {
  log = [];
  seq = 0;
  identity() {
    return { name: 'grant-desk', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, actor, input }) {
    const kinds = {
      'desk.ops.Ping': ['pinged', 'desk.ops.Pinged'],
      'desk.ops.Tally': ['tallied', 'desk.ops.Tallied'],
    };
    const kind = kinds[command];
    if (kind === undefined) {
      throw new Error(`unexpected ${command}`);
    }
    this.seq += 1;
    const published = { event: kind[1], payload: { id: input.id } };
    this.log.push(published);
    const granted =
      !actor ||
      actor === 'desk.ops.Operator' ||
      (actor === 'desk.ops.Watcher' && command === 'desk.ops.Tally');
    if (!granted) {
      return {
        notGranted: true,
        notGrantedActor: actor,
        directEvents: mode === 'forwards-direct-events' ? [published] : [],
      };
    }
    return { outcome: kind[0], consistency: `seq:${this.seq}`, directEvents: [published] };
  }
  queryView() {
    throw new Error('declares no view');
  }
  observeEvents({ event }) {
    return this.log.filter((seen) => seen.event === event);
  }
  configureExternalOutcome() {
    throw new Error('nothing is external');
  }
  redeliverEvent() {
    throw new Error('no bindings');
  }
}

export function makeTarget() {
  return new Desk();
}
";

fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-265-pass2")
        .join(format!("{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

fn report_outcomes(report: &Path, printed: &str) -> BTreeMap<String, String> {
    let text = std::fs::read_to_string(report)
        .unwrap_or_else(|_| panic!("the run wrote no report:\n{printed}"));
    let document: serde_json::Value = serde_json::from_str(&text).expect("report/2 is JSON");
    let mut outcomes = BTreeMap::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            outcomes.insert(id.as_str().expect("an id").to_owned(), status.clone());
        }
    }
    outcomes
}

fn go_verdicts(directory: &Path, mode: &str) -> BTreeMap<String, String> {
    let report = directory.join(format!("report-{mode}.json"));
    let output = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestGrantDesk$",
            "-count=1",
        ])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .env("ESS_TARGET_MODE", mode)
        .current_dir(directory)
        .output()
        .expect("the Go toolchain runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    report_outcomes(&report, &printed)
}

/// The denied scenario's verdict, per mode, through the Go runtime.
#[test]
fn go_fails_a_refusal_that_hands_back_what_the_refused_run_published() {
    let suite = ess_conformance::synthesize(&compiled(DESK)).suite;
    assert!(suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "desk.ops.Ping/grant/denied"));
    let directory = scratch("go");
    std::fs::create_dir_all(directory.join("essconform")).expect("a package directory");
    for artifact in ess_conformance::go::emit(&suite).expect("the package emits") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("writes");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/grantdesk\n\ngo 1.21\n",
    )
    .expect("writes");
    std::fs::write(
        directory.join("essconform").join("grant_test.go"),
        GO_TARGET,
    )
    .expect("writes");

    let control = go_verdicts(&directory, "refuses-after-running");
    assert_eq!(
        control
            .get("desk.ops.Ping/grant/denied")
            .map(String::as_str),
        Some("failed"),
        "control: a new occurrence in the log fails the step: {control:#?}"
    );
    let forwards = go_verdicts(&directory, "forwards-direct-events");
    let _ = std::fs::remove_dir_all(&directory);
    assert_eq!(
        forwards
            .get("desk.ops.Ping/grant/denied")
            .map(String::as_str),
        Some("failed"),
        "a Go target that ran `Ping`, published `Pinged`, and returned that occurrence beside \
         `NotGranted` passes: `took` remembers a refused result's DirectEvents, so the log check \
         reads its own run's occurrence as already seen. The Rust runner cannot even be handed \
         this answer. Verdicts: {forwards:#?}"
    );
}

/// The `Ping/grant/denied` verdict through the TypeScript runtime against [`TS_TARGET`] in `mode`;
/// `None` where `tsc` or `node` is missing.
fn typescript_denied_verdict(label: &str, mode: &str) -> Option<String> {
    for name in ["tsc", "node"] {
        if !Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            println!("skipped: no `{name}` on PATH");
            return None;
        }
    }
    let suite = ess_conformance::synthesize(&compiled(DESK)).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let root = scratch(label);
    for artifact in ess_conformance::ts::emit(admitted.suite()).expect("the package emits") {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path, artifact.contents).expect("writes");
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    for (name, contents) in [
        ("target.mjs", TS_TARGET),
        (
            "driver.mjs",
            include_str!("fixtures/typescript-parity-driver.mjs"),
        ),
        (
            "runtime-test.tsconfig.json",
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        ),
    ] {
        std::fs::write(dir.join(name), contents).expect("writes");
    }
    let compiled_ts = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&dir)
        .output()
        .expect("tsc runs");
    assert!(
        compiled_ts.status.success(),
        "{}{}",
        String::from_utf8_lossy(&compiled_ts.stdout),
        String::from_utf8_lossy(&compiled_ts.stderr)
    );
    let report = dir.join("report.json");
    let output = Command::new("node")
        .args(["--test", "driver.mjs"])
        .env("ESS_TARGET_MODE", mode)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .current_dir(&dir)
        .output()
        .expect("node runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let outcomes = report_outcomes(&report, &printed);
    let _ = std::fs::remove_dir_all(&root);
    Some(
        outcomes
            .get("desk.ops.Ping/grant/denied")
            .cloned()
            .unwrap_or_else(|| panic!("no denied verdict: {outcomes:#?}")),
    )
}

/// The Rust and Go runners fail a target that runs `Ping` for `Watcher`, publishes `Pinged` into
/// its log, and only then answers the standard refusal (the Go control above; pass 1 for Rust).
/// The TypeScript runtime's `decodeStep` copies no `unpublished` member, so `expectNotGranted`
/// observes nothing and passes it — and an authored `no_events:` beside `refused: not_granted` is
/// dropped the same way.
#[test]
fn typescript_fails_a_target_that_runs_the_command_before_refusing_it() {
    let Some(verdict) = typescript_denied_verdict("ts-late", "refuses-after-running") else {
        return;
    };
    assert_eq!(
        verdict, "failed",
        "the TypeScript runtime passes `desk.ops.Ping/grant/denied` against a target whose log \
         holds the `Pinged` the refused send published"
    );
}

/// The same answer [`go_fails_a_refusal_that_hands_back_what_the_refused_run_published`] gives,
/// through the TypeScript runtime: its `executeCommand` remembers a refused result's
/// `directEvents` too.
#[test]
fn typescript_fails_a_refusal_that_hands_back_what_the_refused_run_published() {
    let Some(verdict) = typescript_denied_verdict("ts-forwards", "forwards-direct-events") else {
        return;
    };
    assert_eq!(
        verdict, "failed",
        "a TypeScript target that ran `Ping`, published `Pinged`, and returned that occurrence \
         beside `notGranted` passes"
    );
}

// ---- 5. an authored send no conforming surface runs ------------------------------------------------

/// [`DESK`] with `Sweep`, which the served component accepts and no actor is granted.
fn desk_with_sweep() -> String {
    DESK.replace(
        "actors:\n",
        "  - name: desk.ops.Sweep
    input:
      - {name: id, type: String}
    outcomes:
      - name: swept
        emits: [desk.ops.Tallied]
        payload:
          desk.ops.Tallied: {id: input.id}
actors:\n",
    )
    .replace(
        "accepts: {commands: [desk.ops.Ping, desk.ops.Tally]}",
        "accepts: {commands: [desk.ops.Ping, desk.ops.Tally, desk.ops.Sweep]}",
    )
}

/// Synthesis withholds every scenario that sends `Sweep` expecting it to run, with
/// `Note::GrantedToNoActor`, because the served surface refuses it to every caller. An authored act
/// sending it with no `actor:` and requiring `swept` asks the same impossible thing, and authoring
/// accepts it without a word.
#[test]
fn an_authored_act_that_runs_a_command_granted_to_no_actor_on_a_served_surface_is_refused() {
    let ir = compiled(&desk_with_sweep());
    let notes: Vec<String> = ess_conformance::synthesize(&ir)
        .notes
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        notes
            .iter()
            .any(|note| note.contains("no declared actor may invoke `desk.ops.Sweep`")),
        "precondition: synthesis withholds `Sweep`'s scenarios: {notes:#?}"
    );
    let text = "type: ess-scenario/4
domain: desk.ops
scenario: sweeps
summary: A sweep runs.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: desk.ops.Sweep
    input: {id: one}
    outcome: swept
";
    let authoring = compile_authored(&ir, &[AuthoredSource::new("scenario.yaml", text)]);
    assert!(
        !authoring.is_complete(),
        "an act requiring `Sweep` to run on a surface that refuses it to every caller is accepted: \
         {:?}",
        authoring.scenarios.keys().collect::<Vec<_>>()
    );
}
