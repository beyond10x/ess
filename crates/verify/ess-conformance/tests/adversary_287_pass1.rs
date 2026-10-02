//! Adversary pass 1 against beyond10x/ess#287: a singleton entity, declared by an identity whose
//! type has one value.
//!
//! Each case drives synthesis or a synthesized suite against a hand-written reference target (the
//! interpreter does not run `existing_instance:` branches), the same shape as
//! `tests/singleton_identity.rs`, extended with the faults and model shapes that file does not try.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Note, RefusalCause, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{report::Status, AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// One switch, named `Main`; a job starts only while it is installed and running. The same model as
/// `tests/singleton_identity.rs`.
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

const ENUM_ID: &str = "{name: demo.plant.SwitchId, kind: enum, variants: [Main]}";

/// `ONE_SWITCH` with its switch identity type replaced.
fn with_id(type_decl: &str) -> String {
    ONE_SWITCH.replace(ENUM_ID, type_decl)
}

/// `ONE_SWITCH` with `QueueJob` replaced by `CancelJob`: a command about a job, not the switch, that
/// refuses an unknown job and copies the job's `switch_id` into its event. Its scenarios arrange
/// job rows, and every job row references the one switch.
fn with_cancel_job(model: &str) -> String {
    let queue = "  - name: demo.plant.QueueJob
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
";
    let cancel = "  - name: demo.plant.CancelJob
    input:
      - {name: job_id, type: demo.plant.JobId}
    outcomes:
      - name: no-such-job
        when_related: {via: input.job_id, exists: false}
        error: demo.plant.JobNotFound
      - name: cancelled
        emits: [demo.plant.JobCancelled]
        payload:
          demo.plant.JobCancelled: {job_id: input.job_id, switch_id: {related: {via: input.job_id, field: switch_id}}}
";
    let queued_event = "  - name: demo.plant.JobQueued
    fields:
      - {name: job_id, type: demo.plant.JobId}
      - {name: paused, type: Boolean}
";
    let cancelled_event = "  - name: demo.plant.JobCancelled
    fields:
      - {name: job_id, type: demo.plant.JobId}
      - {name: switch_id, type: demo.plant.SwitchId}
";
    assert!(model.contains(queue) && model.contains(queued_event));
    model
        .replace(queue, cancel)
        .replace(queued_event, cancelled_event)
        .replace(
            "      - demo.plant.QueueJob\n",
            "      - demo.plant.CancelJob\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.plant.JobNotFound\n    summary: No such job.\n",
        )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("plant.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every refusal of `synthesis` that is about the switch identity: `(scenario, reason)`.
fn refused_for_the_switch(synthesis: &Synthesis) -> Vec<(Option<String>, &'static str)> {
    synthesis
        .refusals
        .iter()
        .filter_map(|refusal| match &refusal.cause {
            RefusalCause::NoWitness(gap) if gap.path == "switch_id" => Some((
                refusal.scenario.as_ref().map(ToString::to_string),
                gap.reason,
            )),
            _ => None,
        })
        .collect()
}

// ---- the reference target ----------------------------------------------------------------------

/// What the fixture gets wrong, if anything.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
enum Fault {
    /// None: the model as specified.
    #[default]
    None,
    /// Each caller has a switch of its own: the row is keyed by caller and identity, so a second
    /// caller's install succeeds and one caller's pause does not stop another caller's job.
    KeysByCaller,
    /// A job starts while the switch is paused.
    IgnoresPause,
    /// A second install replaces the switch instead of being refused.
    Reinstalls,
}

/// The model implemented by hand: one switch row at most, keyed by its identity.
#[derive(Default)]
struct Plant {
    fault: Fault,
    /// Rows outlive a scenario: the target is shared between scenarios, which §8 permits as long as
    /// one scenario's observations cannot satisfy another's.
    shared: bool,
    switches: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    jobs: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
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
        Ok(ImplementationIdentity::new("adversary-287-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        if !self.shared {
            self.switches.borrow_mut().clear();
            self.jobs.borrow_mut().clear();
        }
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
        if command.to_string() == "demo.plant.CancelJob" {
            let Some(job_id) = request.input.get("job_id").cloned() else {
                return Ok(SemanticCommandResult::undeclared().with_consistency(token));
            };
            let jobs = self.jobs.borrow();
            let result = match jobs.get(&key(&job_id)) {
                None => error(&command, "no-such-job", "demo.plant.JobNotFound"),
                Some(row) => SemanticCommandResult::took(outcome(&command, "cancelled")).emitting(
                    ObservedEvent::new("demo.plant.JobCancelled".parse().unwrap())
                        .with("job_id", job_id.clone())
                        .with("switch_id", row["switch_id"].clone()),
                ),
            };
            return Ok(result.with_consistency(token));
        }
        let Some(switch_id) = request.input.get("switch_id").cloned() else {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        };
        let acted_by = caller["principal_id"].clone();
        let row_key = if self.fault == Fault::KeysByCaller {
            format!("{}/{}", key(&acted_by), key(&switch_id))
        } else {
            key(&switch_id)
        };
        let mut switches = self.switches.borrow_mut();
        let row = switches.get(&row_key).cloned();
        let state = row.as_ref().map(|row| row["state"].clone());
        let event = |name: &str| {
            ObservedEvent::new(name.parse().unwrap())
                .with("switch_id", switch_id.clone())
                .with("acted_by", acted_by.clone())
        };
        let mut put = |state: &str, paused: bool| {
            switches.insert(
                row_key.clone(),
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
            if row.is_none() {
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

/// Every scenario of `suite` that `plant` does not pass, by id.
fn failed(suite: &ConformanceSuite, plant: &Plant) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, plant)
        .into_report();
    assert!(!report.scenarios.is_empty());
    report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .inspect(|result| {
            if std::env::var_os("ADV287_DEBUG").is_some() {
                eprintln!("{result:#?}");
            }
        })
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn plant(fault: Fault) -> Plant {
    Plant {
        fault,
        ..Plant::default()
    }
}

// ---- attacks -----------------------------------------------------------------------------------

/// Mutant: a target that keeps one switch per caller. The story's need is "one pause switch, so a
/// guard on that row holds for every caller"; a target where caller B's job ignores caller A's
/// pause, and caller B may install a second switch beside A's, is exactly the defect a singleton
/// declaration exists to rule out. Some scenario must fail it.
#[test]
fn adv287_a_target_keeping_one_switch_per_caller_fails_some_scenario() {
    let synthesis = synthesize(&ir(ONE_SWITCH));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
    let failed = failed(&synthesis.suite, &plant(Fault::KeysByCaller));
    assert!(
        !failed.is_empty(),
        "a target with one switch per caller passes every scenario of the singleton suite"
    );
}

/// Control for the per-caller mutant: the same model with a `Uuid` switch identity (its suite
/// byte-identical to 0.49.0's). Red here says the per-caller store is a gap of synthesis at large,
/// not of the singleton alone.
#[test]
#[ignore = "pre-existing, not this unit: beyond10x/ess#312 (coordinator, 2026-10-01)"]
fn adv287_control_a_uuid_switch_suite_fails_a_target_keeping_one_switch_per_caller() {
    let model = with_id("{name: demo.plant.SwitchId, kind: newtype, of: Uuid}");
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
    assert!(
        !failed(&synthesis.suite, &plant(Fault::KeysByCaller)).is_empty(),
        "a target with one switch per caller passes every scenario of the Uuid suite"
    );
}

/// Control for the shared-target case: the `Uuid` switch suite passes a correct target that keeps
/// its rows across scenarios, as §8 has every other suite do.
#[test]
#[ignore = "pre-existing, not this unit: beyond10x/ess#312 (coordinator, 2026-10-01)"]
fn adv287_control_a_uuid_switch_suite_passes_a_shared_correct_target() {
    let model = with_id("{name: demo.plant.SwitchId, kind: newtype, of: Uuid}");
    let synthesis = synthesize(&ir(&model));
    let shared = Plant {
        shared: true,
        ..Plant::default()
    };
    assert_eq!(failed(&synthesis.suite, &shared), Vec::new());
}

/// Withdrawal with a correct form:`CancelJob` is about a job, not the switch, and its two
/// scenarios need job rows, every one of which references the one switch. A correct form exists —
/// install `Main` once and start every arranged job against it; a related copy with no companion
/// is the `UnaccompaniedRelatedCopy` note, not a refusal. Acceptance 1 says a one-value identity
/// synthesizes with no ESS-SYNTH-001; this model is refused twice for the switch identity.
#[test]
fn adv287_a_command_about_another_entity_that_references_the_one_row_is_not_refused() {
    let model = with_cancel_job(ONE_SWITCH);
    let synthesis = synthesize(&ir(&model));
    let refused = refused_for_the_switch(&synthesis);
    assert_eq!(
        refused,
        Vec::new(),
        "scenarios about a job are refused for the one switch they reference"
    );
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
}

/// Control for the case above, run on the tree: the same model with a `Uuid` switch identity
/// synthesizes both `CancelJob` scenarios and a correct target passes them, so the refusal is the
/// singleton's.
#[test]
fn adv287_control_the_same_job_command_beside_a_uuid_switch_synthesizes() {
    let model = with_cancel_job(&with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}",
    ));
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    for id in [
        "demo.plant.CancelJob/outcome/no-such-job",
        "demo.plant.CancelJob/outcome/cancelled",
    ] {
        assert!(
            synthesis
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id),
            "{id}"
        );
    }
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
}

/// Detection, false negative: an integer newtype bounded to one value by `>=` and `<=` has one
/// value, as surely as `value == 1`, and the witness builder already knows it ("too few values").
/// It is refused with ESS-SYNTH-001 as on 0.49.0.
#[test]
fn adv287_an_integer_identity_bounded_to_one_value_draws_no_second_value() {
    let model = with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: Integer, invariants: [\"value >= 1\", \"value <= 1\"]}",
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(refused_for_the_switch(&synthesis), Vec::new());
}

/// Detection, false negative: a text newtype over a one-letter alphabet and `.count == 1` has one
/// value; refused with ESS-SYNTH-001 as on 0.49.0.
#[test]
fn adv287_a_one_letter_alphabet_identity_of_count_one_draws_no_second_value() {
    let model = with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: String, alphabet: \"A\", invariants: [\"value.count == 1\"]}",
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(refused_for_the_switch(&synthesis), Vec::new());
}

/// Detection, false negative: `in: [Main, Main]` lists one value twice; `pins` counts list
/// entries, not distinct values.
#[test]
fn adv287_an_in_list_naming_one_value_twice_draws_no_second_value() {
    let model = with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: String, invariants: [{value: {in: [Main, Main]}}]}",
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(refused_for_the_switch(&synthesis), Vec::new());
}

/// Shared target: §8 permits a target shared between scenarios ("a shared staging system is the
/// case it has in mind", `scenario.rs` `Counts`), and synthesis otherwise keeps every scenario
/// passing on one. The singleton scenarios rely on an empty target per scenario instead. A correct
/// target that keeps its rows across scenarios fails them; each scenario that does must carry a
/// note saying it needs an empty target, or the suite claims more than it says.
#[test]
#[ignore = "pre-existing, not this unit: beyond10x/ess#312 (coordinator, 2026-10-01)"]
fn adv287_a_shared_target_is_told_the_singleton_scenarios_need_an_empty_one() {
    let synthesis = synthesize(&ir(ONE_SWITCH));
    let shared = Plant {
        shared: true,
        ..Plant::default()
    };
    let failed = failed(&synthesis.suite, &shared);
    let unexplained: Vec<_> = failed
        .iter()
        .filter(|(scenario, _)| {
            !synthesis.notes.iter().any(|note| {
                !matches!(note, Note::UnswappedCallers { .. })
                    && format!("{note:?}").contains(scenario.as_str())
            })
        })
        .collect();
    assert_eq!(
        unexplained,
        Vec::<&(String, Status)>::new(),
        "scenarios that fail a correct shared target with no note saying why"
    );
}

/// Interaction with ess/20: the pause guard read as the switch's lifecycle `state`. Synthesizes
/// with nothing refused, passes the correct target, and fails a target that ignores the pause.
#[test]
fn adv287_a_state_guard_on_the_one_switch_fails_a_target_ignoring_the_pause() {
    let model = ONE_SWITCH
        .replace("format: ess/18", "format: ess/20")
        .replace("predicate: paused == true}", "predicate: state == Paused}");
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
    let ignoring = failed(&synthesis.suite, &plant(Fault::IgnoresPause));
    assert!(
        ignoring
            .iter()
            .any(|(id, _)| id == "demo.plant.StartJob/outcome/switch-paused"),
        "{ignoring:#?}"
    );
    let reinstalling = failed(&synthesis.suite, &plant(Fault::Reinstalls));
    assert!(
        reinstalling
            .iter()
            .any(|(id, _)| id == "demo.plant.InstallSwitch/outcome/already-installed"),
        "{reinstalling:#?}"
    );
}

/// A one-variant enum on a field that is no identity changes nothing: the suite is the one the
/// same model synthesizes with that field absent, but for the field itself. Checked here as: the
/// non-singleton switch model synthesizes with no refusal and the correct target passes.
#[test]
fn adv287_a_one_variant_enum_on_a_plain_field_leaves_a_uuid_switch_alone() {
    let model = with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}\n  - {name: demo.plant.Kind, kind: enum, variants: [Only]}",
    )
    .replace(
        "      - {name: switch_id, type: demo.plant.SwitchId}\n    lifecycle: {initial: Queued",
        "      - {name: switch_id, type: demo.plant.SwitchId}\n      - {name: kind, type: demo.plant.Kind}\n    lifecycle: {initial: Queued",
    );
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert!(
        !synthesis
            .notes
            .iter()
            .any(|note| matches!(note, Note::UnswappedCallers { .. })),
        "a Uuid switch has room for the swapped run"
    );
}
