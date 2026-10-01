//! Adversary pass 1 against beyond10x/ess#265: an ungranted actor gets one declared refusal,
//! witnessed per command.
//!
//! Each case drives the synthesized suite against an in-process target built from the reference
//! interpreter, changed in exactly one way, and states what the served contract says that target
//! must do.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn compiled(files: &[(&str, &str)]) -> EssIr {
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        parsed.push((Source::new(*label), raw));
    }
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
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

/// One served component. `Ping` is `Operator`'s, `Tally` is both actors', and `Sweep` is no
/// actor's: a command on the served surface that the specification grants nobody.
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
  - name: desk.ops.Swept
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
  - name: desk.ops.Sweep
    input:
      - {name: id, type: String}
    outcomes:
      - name: swept
        emits: [desk.ops.Swept]
        payload:
          desk.ops.Swept: {id: input.id}
actors:
  - name: desk.ops.Operator
    may: [desk.ops.Ping, desk.ops.Tally]
  - name: desk.ops.Watcher
    may: [desk.ops.Tally]
components:
  - component: desk-service
    owns: {domains: [desk.ops]}
    accepts: {commands: [desk.ops.Ping, desk.ops.Tally, desk.ops.Sweep]}
    publishes: {events: [desk.ops.Pinged, desk.ops.Tallied, desk.ops.Swept]}
    reached_by: network
";

/// How the wrapped interpreter departs from the reference.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    /// None: the reference interpreter, as a control.
    Reference,
    /// The served contract, read literally: a request authenticated as no actor is the standard
    /// refusal, `{"refused": "not granted", "actor": null}`.
    RefusesNoActor,
    /// Checks the grant only after the command ran: the command takes effect, publishes, and the
    /// answer is the standard refusal.
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

    fn granted(&self, request: &SemanticCommandRequest) -> bool {
        let Some(actor) = &request.actor else {
            return true;
        };
        self.ir.actors().get(actor.name()).is_some_and(|declared| {
            declared
                .may
                .iter()
                .any(|command| command.name() == request.command.name())
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
            Change::RefusesNoActor => {
                if request.actor.is_none() {
                    return Err(TargetError::not_granted(None::<String>));
                }
                self.inner.execute_command(request)
            }
            Change::ChecksAfterRunning => {
                if self.granted(&request) {
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

fn verdicts(suite: &ConformanceSuite, target: &Wrapped) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).expect("admits");
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn not_passed(verdicts: &[(String, Status)]) -> Vec<&str> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The served contract says a request authenticated as no actor gets the standard refusal, and the
/// generated Rust and Go servers answer it (`admit(None, …)` is `Err(None)`). The synthesized suite
/// sends every command no declared actor holds as no actor and requires its branch, so a target
/// that obeys the contract fails those scenarios — and no target that obeys it can pass them.
#[test]
fn a_target_that_refuses_no_actor_as_the_contract_says_passes_the_synthesized_suite() {
    let ir = compiled(&[("spec.yaml", DESK)]);
    let suite = ess_conformance::synthesize(&ir).suite;

    let reference = verdicts(&suite, &Wrapped::new(ir.clone(), Change::Reference));
    assert!(
        not_passed(&reference).is_empty(),
        "control: the reference interpreter passes the suite: {reference:#?}"
    );

    let sent_as_no_actor: BTreeSet<String> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            scenario.steps.iter().any(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { actor: None, .. })
                    || matches!(
                        step,
                        ScenarioStep::ExecuteCommandWithoutInput { actor: None, .. }
                    )
            })
        })
        .map(|(id, _)| id.to_string())
        .collect();

    let contract = verdicts(&suite, &Wrapped::new(ir, Change::RefusesNoActor));
    assert!(
        not_passed(&contract).is_empty(),
        "a target that refuses a request authenticated as no actor, as the served contract \
         declares, fails {:?}; the suite sent these scenarios as no actor: {sent_as_no_actor:?}",
        not_passed(&contract)
    );
}

/// Every `…/grant/denied` scenario says the command was "refused before it runs" and carries an
/// `ExpectNoEvent` per declared event. Those steps read the refused command's direct events, which
/// a refusal never has, so they cannot fail: a target that runs the command — creating the visit
/// and publishing `VisitRegistered` — and only then answers the standard refusal passes.
#[test]
fn a_target_that_runs_the_command_before_refusing_it_fails_a_denied_scenario() {
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

    let late = Wrapped::new(ir, Change::ChecksAfterRunning);
    let verdicts = verdicts(&suite, &late);
    let published = late.published_then_refused.borrow().clone();
    assert!(
        published.contains(&"gatepass.visit.VisitRegistered".to_owned()),
        "precondition: the late-checking target ran `RegisterVisit` and published: {published:?}"
    );
    assert!(
        verdicts.iter().any(
            |(id, status)| id == "gatepass.visit.RegisterVisit/grant/denied"
                && *status != Status::Passed
        ),
        "a target that ran `RegisterVisit` (published {published:?}) before refusing it passes \
         every denied scenario: {verdicts:#?}"
    );
}

/// The suite sends each granted command as the lowest-named actor holding it and never as any
/// other, so a server whose grant table drops `Watcher`'s `Tally` — refusing an actor the
/// specification grants — passes every scenario.
#[test]
fn a_target_that_refuses_a_granted_actor_fails_the_synthesized_suite() {
    let ir = compiled(&[("spec.yaml", DESK)]);
    let suite = ess_conformance::synthesize(&ir).suite;
    let mutant = verdicts(&suite, &Wrapped::new(ir, Change::RefusesWatcherTally));
    assert!(
        !not_passed(&mutant).is_empty(),
        "a target refusing `Watcher` the `Tally` it is granted passes all {} scenarios",
        mutant.len()
    );
}

/// The decision: denied scenarios are synthesized for served components only. `desk.back.Tally` is
/// accepted by an in-process component (no `reached_by: network`), so no served surface carries it
/// and no surface answers its standard refusal.
#[test]
fn a_command_on_an_in_process_component_gets_no_denied_scenario() {
    let system = "format: ess/1
system: desk
version: v1
domains:
  - desk.ops
  - desk.back
";
    let ops = "domain: desk.ops
events:
  - name: desk.ops.Pinged
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
actors:
  - name: desk.ops.Operator
    may: [desk.ops.Ping, desk.back.Tally]
  - name: desk.ops.Watcher
    may: [desk.ops.Ping]
";
    let back = "domain: desk.back
events:
  - name: desk.back.Tallied
    fields:
      - {name: id, type: String}
commands:
  - name: desk.back.Tally
    input:
      - {name: id, type: String}
    outcomes:
      - name: tallied
        emits: [desk.back.Tallied]
        payload:
          desk.back.Tallied: {id: input.id}
";
    let components = "components:
  - component: desk-service
    owns: {domains: [desk.ops]}
    accepts: {commands: [desk.ops.Ping]}
    publishes: {events: [desk.ops.Pinged]}
    reached_by: network
  - component: back-office
    owns: {domains: [desk.back]}
    accepts: {commands: [desk.back.Tally]}
    publishes: {events: [desk.back.Tallied]}
";
    let ir = compiled(&[
        ("system.yaml", system),
        ("domains/ops.yaml", ops),
        ("domains/back.yaml", back),
        ("components.yaml", components),
    ]);
    let ids: BTreeSet<String> = ess_conformance::synthesize(&ir)
        .suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Grant { .. }))
        .map(ToString::to_string)
        .collect();
    assert!(
        !ids.contains("desk.back.Tally/grant/denied"),
        "a command no served surface carries is witnessed as refused by one: {ids:?}"
    );
}
