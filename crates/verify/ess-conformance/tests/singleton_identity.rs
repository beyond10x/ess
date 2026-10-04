//! A singleton entity, declared by an identity whose type has one value (beyond10x/ess#287).
//!
//! A system with exactly one row of an entity — one pause switch every job reads — says so with a
//! one-variant enum identity; there is no `singleton:` key. Synthesis used to ask such an identity
//! for a second value and refused five branches with ESS-SYNTH-001 ("too few values to name an
//! identity"). Each scenario now needs only per-scenario isolation (§8): the one value is unknown
//! where the scenario arranged no row, the creation sends it into an empty scenario, and the second
//! creation of the same value is the `existing_instance:` refusal. A run with the callers swapped
//! would need a second row, so it is kept to its first run with an `UnswappedCallers` note.
//!
//! The interpreter does not run `existing_instance:` branches, so the reference target is a
//! hand-written fixture, as in `tests/adversary_275_pass2.rs`.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Note, RefusalCause, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{
    report::Status, AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep,
    ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// One switch, named `Main`, installed once, paused and resumed; a job starts only while it is
/// installed and running.
const ONE_SWITCH: &str = "format: ess/18
system: demo
version: v1
summary: One pause switch, which every job reads.
domain: demo.plant
types:
  - {name: demo.plant.SwitchId, kind: enum, variants: [Main]}
  - {name: demo.plant.JobId, kind: newtype, of: Uuid}
  - {name: demo.plant.PrincipalId, kind: newtype, of: Uuid}
entities:
  - name: demo.plant.Switch
    identity: {name: switch_id, type: demo.plant.SwitchId}
    fields:
      - {name: paused, type: Boolean}
    lifecycle:
      initial: Running
      states: [Running, Paused]
      transitions:
        - {name: pause, from: [Running], to: Paused}
        - {name: resume, from: [Paused], to: Running}
  - name: demo.plant.Job
    identity: {name: job_id, type: demo.plant.JobId}
    fields:
      - {name: switch_id, type: demo.plant.SwitchId}
    lifecycle: {initial: Queued, states: [Queued], terminal: [Queued], transitions: []}
actors:
  - name: demo.plant.Operator
    attributes:
      - {name: principal_id, type: demo.plant.PrincipalId}
    may:
      - demo.plant.InstallSwitch
      - demo.plant.PauseSwitch
      - demo.plant.ResumeSwitch
      - demo.plant.StartJob
      - demo.plant.QueueJob
commands:
  - name: demo.plant.InstallSwitch
    input:
      - {name: switch_id, type: demo.plant.SwitchId}
    outcomes:
      - name: installed
        creates: demo.plant.Switch
        instance: switch_id
        sets: {paused: false}
        emits: [demo.plant.SwitchInstalled]
        payload:
          demo.plant.SwitchInstalled: {switch_id: input.switch_id, acted_by: {caller: principal_id}}
      - {name: already-installed, existing_instance: true, error: demo.plant.AlreadyInstalled}
  - name: demo.plant.PauseSwitch
    input:
      - {name: switch_id, type: demo.plant.SwitchId}
    outcomes:
      - name: paused
        moves: demo.plant.Switch.pause
        instance: switch_id
        sets: {paused: true}
        emits: [demo.plant.SwitchPaused]
        payload:
          demo.plant.SwitchPaused: {switch_id: input.switch_id, acted_by: {caller: principal_id}}
      - {name: wrong-state, wrong_state: true, error: demo.plant.SwitchStateConflict}
  - name: demo.plant.ResumeSwitch
    input:
      - {name: switch_id, type: demo.plant.SwitchId}
    outcomes:
      - name: resumed
        moves: demo.plant.Switch.resume
        instance: switch_id
        sets: {paused: false}
        emits: [demo.plant.SwitchResumed]
        payload:
          demo.plant.SwitchResumed: {switch_id: input.switch_id, acted_by: {caller: principal_id}}
      - {name: wrong-state, wrong_state: true, error: demo.plant.SwitchStateConflict}
  - name: demo.plant.StartJob
    input:
      - {name: job_id, type: demo.plant.JobId}
      - {name: switch_id, type: demo.plant.SwitchId}
    outcomes:
      - name: no-such-switch
        when_related: {via: input.switch_id, exists: false}
        error: demo.plant.SwitchNotFound
      - name: switch-paused
        when_related: {via: input.switch_id, predicate: paused == true}
        error: demo.plant.SwitchIsPaused
      - name: started
        creates: demo.plant.Job
        instance: job_id
        sets: {switch_id: input.switch_id}
        emits: [demo.plant.JobStarted]
        payload:
          demo.plant.JobStarted: {job_id: input.job_id, switch_id: input.switch_id}
  - name: demo.plant.QueueJob
    input:
      - {name: job_id, type: demo.plant.JobId}
      - {name: switch_id, type: demo.plant.SwitchId}
    outcomes:
      - name: queued
        creates: demo.plant.Job
        instance: job_id
        sets: {switch_id: input.switch_id}
        emits: [demo.plant.JobQueued]
        payload:
          demo.plant.JobQueued: {job_id: input.job_id, paused: {related: {via: input.switch_id, field: paused}}}
errors:
  - name: demo.plant.AlreadyInstalled
    summary: The switch is installed already; there is one.
  - name: demo.plant.SwitchStateConflict
    summary: The switch is not in the state this command moves it from.
  - name: demo.plant.SwitchNotFound
    summary: No switch is installed.
  - name: demo.plant.SwitchIsPaused
    summary: The switch is paused, so no job starts.
events:
  - name: demo.plant.SwitchInstalled
    fields:
      - {name: switch_id, type: demo.plant.SwitchId}
      - {name: acted_by, type: demo.plant.PrincipalId}
  - name: demo.plant.SwitchPaused
    fields:
      - {name: switch_id, type: demo.plant.SwitchId}
      - {name: acted_by, type: demo.plant.PrincipalId}
  - name: demo.plant.SwitchResumed
    fields:
      - {name: switch_id, type: demo.plant.SwitchId}
      - {name: acted_by, type: demo.plant.PrincipalId}
  - name: demo.plant.JobStarted
    fields:
      - {name: job_id, type: demo.plant.JobId}
      - {name: switch_id, type: demo.plant.SwitchId}
  - name: demo.plant.JobQueued
    fields:
      - {name: job_id, type: demo.plant.JobId}
      - {name: paused, type: Boolean}
views:
  - name: demo.plant.Switches
    source: demo.plant.Switch
    consistency: read_your_writes
    fields:
      - {name: switch_id, type: demo.plant.SwitchId}
      - {name: paused, type: Boolean}
      - {name: state, type: demo.plant.Switch.State}
  - name: demo.plant.Jobs
    source: demo.plant.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.plant.JobId}
      - {name: switch_id, type: demo.plant.SwitchId}
      - {name: state, type: demo.plant.Job.State}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("plant.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The two ways a type has one value: an enum of one variant, and a text newtype its invariant pins
/// to one literal.
#[derive(Clone, Copy, Debug)]
enum Form {
    Enum,
    Pinned,
}

fn model(form: Form) -> String {
    match form {
        Form::Enum => ONE_SWITCH.to_owned(),
        Form::Pinned => ONE_SWITCH.replace(
            "{name: demo.plant.SwitchId, kind: enum, variants: [Main]}",
            "{name: demo.plant.SwitchId, kind: newtype, of: String, invariants: ['value == \"Main\"']}",
        ),
    }
}

fn synthesis(form: Form) -> Synthesis {
    synthesize(&ir(&model(form)))
}

fn suite(form: Form) -> ConformanceSuite {
    let synthesis = synthesis(form);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused for the {form:?} form: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// Every `(command, switch_id)` a scenario sends, in order.
fn sends(scenario: &ConformanceScenario) -> Vec<(String, Option<serde_json::Value>)> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => Some((
                command.to_string(),
                input.get("switch_id").map(|value| match value {
                    ScenarioValue::Literal { value } => serde_json::to_value(value).unwrap(),
                    other => serde_json::to_value(other).unwrap(),
                }),
            )),
            _ => None,
        })
        .collect()
}

// ---- the reference target ----------------------------------------------------------------------

/// What the fixture gets wrong, if anything.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Fault {
    /// None: the model as specified.
    #[default]
    None,
    /// A second install replaces the switch instead of being refused.
    Reinstalls,
    /// A job starts while the switch is paused.
    IgnoresPause,
    /// A job starts with no switch installed.
    IgnoresAbsence,
}

/// The model implemented by hand: one switch row at most, keyed by its identity.
#[derive(Default)]
struct Plant {
    fault: Fault,
    switches: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    jobs: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Plant {
    fn with(fault: Fault) -> Self {
        Self {
            fault,
            ..Self::default()
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn error(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(outcome(command, name))
        .with_error(DeclaredErrorValue::new(error.parse().unwrap()))
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn key(value: &Node) -> String {
    serde_json::to_string(value).unwrap()
}

impl ConformanceTarget for Plant {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "singleton-identity-fixture",
            "1",
        ))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.switches.borrow_mut().clear();
        self.jobs.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let Some(caller) = request.caller.clone() else {
            return Err(TargetError::unavailable(
                "authenticating",
                "every command of this system is sent as a caller",
            ));
        };
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let Some(switch_id) = request.input.get("switch_id").cloned() else {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        };
        let acted_by = caller["principal_id"].clone();
        let mut switches = self.switches.borrow_mut();
        let row = switches.get(&key(&switch_id)).cloned();
        let state = row.as_ref().map(|row| row["state"].clone());
        let event = |name: &str| {
            ObservedEvent::new(name.parse().unwrap())
                .with("switch_id", switch_id.clone())
                .with("acted_by", acted_by.clone())
        };
        let mut put = |state: &str, paused: bool| {
            switches.insert(
                key(&switch_id),
                BTreeMap::from([
                    ("switch_id".to_owned(), switch_id.clone()),
                    ("paused".to_owned(), Node::Bool(paused)),
                    ("state".to_owned(), text(state)),
                ]),
            );
        };
        let result = match command.to_string().as_str() {
            "demo.plant.InstallSwitch" => {
                if row.is_some() && self.fault != Fault::Reinstalls {
                    error(&command, "already-installed", "demo.plant.AlreadyInstalled")
                } else {
                    put("Running", false);
                    SemanticCommandResult::took(outcome(&command, "installed"))
                        .emitting(event("demo.plant.SwitchInstalled"))
                }
            }
            "demo.plant.PauseSwitch" => {
                if state == Some(text("Running")) {
                    put("Paused", true);
                    SemanticCommandResult::took(outcome(&command, "paused"))
                        .emitting(event("demo.plant.SwitchPaused"))
                } else {
                    error(&command, "wrong-state", "demo.plant.SwitchStateConflict")
                }
            }
            "demo.plant.ResumeSwitch" => {
                if state == Some(text("Paused")) {
                    put("Running", false);
                    SemanticCommandResult::took(outcome(&command, "resumed"))
                        .emitting(event("demo.plant.SwitchResumed"))
                } else {
                    error(&command, "wrong-state", "demo.plant.SwitchStateConflict")
                }
            }
            "demo.plant.StartJob" | "demo.plant.QueueJob" => {
                let Some(result) = self.job(&command, &request, row.as_ref(), &switch_id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                result
            }
            other => panic!("no command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows = match request.view.to_string().as_str() {
            "demo.plant.Switches" => self.switches.borrow().values().cloned().collect::<Vec<_>>(),
            "demo.plant.Jobs" => self.jobs.borrow().values().cloned().collect(),
            "demo.plant.SwitchCount" => {
                let count = u32::try_from(self.switches.borrow().len()).unwrap();
                vec![BTreeMap::from([(
                    "switches".to_owned(),
                    Node::Number(ess_primitives::facts::Number::new(f64::from(count)).unwrap()),
                )])]
            }
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

impl Plant {
    /// `StartJob` or `QueueJob` against the switch row `row`, if any; `None` where no `job_id` was
    /// sent.
    fn job(
        &self,
        command: &CommandRef,
        request: &SemanticCommandRequest,
        row: Option<&BTreeMap<String, Node>>,
        switch_id: &Node,
    ) -> Option<SemanticCommandResult> {
        let job_id = request.input.get("job_id").cloned()?;
        let queued = command.to_string() == "demo.plant.QueueJob";
        let paused = row.map_or(Node::Bool(false), |row| row["paused"].clone());
        if !queued {
            if row.is_none() && self.fault != Fault::IgnoresAbsence {
                return Some(error(
                    command,
                    "no-such-switch",
                    "demo.plant.SwitchNotFound",
                ));
            }
            if paused == Node::Bool(true) && self.fault != Fault::IgnoresPause {
                return Some(error(command, "switch-paused", "demo.plant.SwitchIsPaused"));
            }
        }
        self.jobs.borrow_mut().insert(
            key(&job_id),
            BTreeMap::from([
                ("job_id".to_owned(), job_id.clone()),
                ("switch_id".to_owned(), switch_id.clone()),
                ("state".to_owned(), text("Queued")),
            ]),
        );
        Some(if queued {
            SemanticCommandResult::took(outcome(command, "queued")).emitting(
                ObservedEvent::new("demo.plant.JobQueued".parse().unwrap())
                    .with("job_id", job_id)
                    .with("paused", paused),
            )
        } else {
            SemanticCommandResult::took(outcome(command, "started")).emitting(
                ObservedEvent::new("demo.plant.JobStarted".parse().unwrap())
                    .with("job_id", job_id)
                    .with("switch_id", switch_id.clone()),
            )
        })
    }
}

/// Every scenario of `suite` the fixture does not pass, by id.
fn failed(suite: &ConformanceSuite, fault: Fault) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Plant::with(fault))
        .into_report();
    assert_ne!(report.scenarios.len(), 0);
    report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .inspect(|result| {
            if fault == Fault::None {
                eprintln!("{result:#?}");
            }
        })
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

// ---- acceptance --------------------------------------------------------------------------------

/// Acceptance 1: a one-value identity synthesizes with no ESS-SYNTH-001, and every branch the
/// requester saw refused has its scenario.
#[test]
fn singleton_a_one_value_identity_synthesizes_every_branch() {
    let suite = suite(Form::Enum);
    for id in [
        "demo.plant.InstallSwitch/outcome/installed",
        "demo.plant.InstallSwitch/outcome/already-installed",
        "demo.plant.PauseSwitch/outcome/wrong-state",
        "demo.plant.ResumeSwitch/outcome/wrong-state",
        "demo.plant.StartJob/outcome/no-such-switch",
        "demo.plant.StartJob/outcome/switch-paused",
        "demo.plant.StartJob/outcome/started",
        "demo.plant.QueueJob/outcome/queued",
    ] {
        scenario(&suite, id);
    }
}

/// Acceptance 2: a second install of the one row is refused, and the refusal is witnessed — the
/// scenario installs `Main` and installs `Main` again.
#[test]
fn singleton_a_second_install_is_witnessed_as_existing_instance() {
    let suite = suite(Form::Enum);
    let installs: Vec<_> = sends(scenario(
        &suite,
        "demo.plant.InstallSwitch/outcome/already-installed",
    ))
    .into_iter()
    .filter(|(command, _)| command == "demo.plant.InstallSwitch")
    .map(|(_, switch_id)| switch_id)
    .collect();
    assert_eq!(
        installs,
        vec![
            Some(serde_json::json!("Main")),
            Some(serde_json::json!("Main"))
        ],
        "the one value is installed, then installed again"
    );
}

/// Every synthesized scenario passes the model implemented by hand.
#[test]
fn singleton_every_scenario_passes_a_target_that_keeps_one_switch() {
    assert_eq!(failed(&suite(Form::Enum), Fault::None), Vec::new());
}

/// A text identity pinned to one literal by its type's invariant is the same singleton: nothing is
/// refused for want of a second value, and every scenario synthesized passes the same target.
///
/// What it is still refused is not about the identity's value: a complete-subject snapshot through
/// `Switches`, whose `switch_id` column is a constrained newtype the replay observer does not
/// support (`replay.rs`, "replay response invariant/reading observer is unsupported") — the same
/// for any entity with such an identity type, singleton or not.
#[test]
fn singleton_a_pinned_text_identity_draws_no_second_value() {
    let synthesis = synthesis(Form::Pinned);
    let about_the_identity: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            matches!(&refusal.cause, RefusalCause::NoWitness(gap) if gap.path == "switch_id")
        })
        .collect();
    assert!(
        about_the_identity.is_empty(),
        "no scenario is refused for want of a second switch_id: {about_the_identity:#?}"
    );
    for id in [
        "demo.plant.InstallSwitch/outcome/installed",
        "demo.plant.PauseSwitch/outcome/wrong-state",
        "demo.plant.ResumeSwitch/outcome/wrong-state",
        "demo.plant.StartJob/outcome/no-such-switch",
        "demo.plant.StartJob/outcome/switch-paused",
        "demo.plant.StartJob/outcome/started",
        "demo.plant.QueueJob/outcome/queued",
    ] {
        scenario(&synthesis.suite, id);
    }
    assert_eq!(failed(&synthesis.suite, Fault::None), Vec::new());
}

/// A target that installs a second switch over the first is failed by the existing-instance
/// scenario, one that ignores the pause by the paused-switch scenario, and one that starts a job
/// with no switch by the no-such-switch scenario.
#[test]
fn singleton_each_fault_is_failed_by_its_scenario() {
    let suite = suite(Form::Enum);
    for (fault, id) in [
        (
            Fault::Reinstalls,
            "demo.plant.InstallSwitch/outcome/already-installed",
        ),
        (
            Fault::IgnoresPause,
            "demo.plant.StartJob/outcome/switch-paused",
        ),
        (
            Fault::IgnoresAbsence,
            "demo.plant.StartJob/outcome/no-such-switch",
        ),
    ] {
        let failed = failed(&suite, fault);
        assert!(
            failed.iter().any(|(scenario, _)| scenario == id),
            "{id} fails the fault; failed: {failed:#?}"
        );
    }
}

/// The run with the callers swapped would install the one value a second time, so each scenario
/// that installs it keeps its first run and says so in an `UnswappedCallers` note.
#[test]
fn singleton_a_swapped_run_that_needs_a_second_row_is_noted() {
    let synthesis = synthesis(Form::Enum);
    let noted: Vec<_> = synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnswappedCallers {
                scenario,
                input,
                type_ref,
            } => Some((scenario.to_string(), input.clone(), type_ref.clone())),
            _ => None,
        })
        .collect();
    assert!(
        noted.iter().any(|(scenario, input, type_ref)| scenario
            == "demo.plant.InstallSwitch/outcome/installed"
            && input == "switch_id"
            && type_ref == "demo.plant.SwitchId"),
        "the installing scenario is noted: {noted:#?}"
    );
}

/// A family that arranges several rows of an entity in one run — an aggregate view counting them —
/// cannot hold a singleton: whichever family built it, a scenario expecting the one row created
/// twice in a run is withdrawn and refused by name, never filed to fail a correct target.
#[test]
fn singleton_a_scenario_creating_the_one_row_twice_is_withdrawn_and_refused() {
    let counted = model(Form::Enum).replace(
        "views:\n",
        "views:\n  - name: demo.plant.SwitchCount\n    source: demo.plant.Switch\n    fields:\n      - {name: switches, type: Integer, aggregate: {count: {}}}\n",
    );
    let synthesis = synthesize(&ir(&counted));
    let refused: Vec<_> = synthesis
        .refusals
        .iter()
        .filter_map(|refusal| match &refusal.cause {
            RefusalCause::NoWitness(gap) if gap.path == "switch_id" => Some((
                refusal.scenario.as_ref().map(ToString::to_string),
                gap.reason,
            )),
            _ => None,
        })
        .collect();
    assert!(
        !refused.is_empty()
            && refused.iter().all(|(scenario, reason)| {
                scenario.is_some() && reason.contains("names one row")
            }),
        "the scenarios creating the switch twice are refused by name: {refused:#?}"
    );
    for (scenario, _) in &refused {
        let scenario = scenario.as_deref().unwrap();
        assert!(
            !synthesis
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string() == scenario),
            "{scenario} is withdrawn"
        );
    }
    assert_eq!(failed(&synthesis.suite, Fault::None), Vec::new());
}

/// A job command copying the job's `switch_id`: every job row references the one switch, so its
/// scenarios arrange the switch once and every job against it, and no companion job can hold another
/// `switch_id` — which the scenario says in a note naming the field, rather than being refused.
#[test]
fn singleton_a_copied_reference_to_the_one_row_is_noted_unaccompanied() {
    let model = model(Form::Enum)
        .replace(
            "      - demo.plant.QueueJob\n",
            "      - demo.plant.QueueJob\n      - demo.plant.CancelJob\n",
        )
        .replace(
            "errors:\n",
            "  - name: demo.plant.CancelJob\n    input:\n      - {name: job_id, type: demo.plant.JobId}\n    outcomes:\n      - name: no-such-job\n        when_related: {via: input.job_id, exists: false}\n        error: demo.plant.JobNotFound\n      - name: cancelled\n        emits: [demo.plant.JobCancelled]\n        payload:\n          demo.plant.JobCancelled: {job_id: input.job_id, switch_id: {related: {via: input.job_id, field: switch_id}}}\nerrors:\n  - name: demo.plant.JobNotFound\n    summary: No such job.\n",
        )
        .replace(
            "views:\n",
            "  - name: demo.plant.JobCancelled\n    fields:\n      - {name: job_id, type: demo.plant.JobId}\n      - {name: switch_id, type: demo.plant.SwitchId}\nviews:\n",
        );
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert!(
        synthesis.notes.iter().any(|note| matches!(
            note,
            Note::UnaccompaniedRelatedCopy { scenario, fields }
                if scenario.to_string() == "demo.plant.CancelJob/outcome/cancelled"
                    && fields == &["switch_id".to_owned()]
        )),
        "{:#?}",
        synthesis.notes
    );
}
