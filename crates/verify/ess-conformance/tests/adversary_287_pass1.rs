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
    /// Accepts only the first concrete principal this target encountered.
    OnlyFirstCaller,
    /// Cannot establish the suite's declared scenario namespace.
    CannotIsolate,
}

type CallTrace = BTreeMap<String, Vec<(String, Node, Node)>>;

/// The model implemented by hand: one switch row at most, keyed by its identity.
#[derive(Default)]
struct Plant {
    fault: Fault,
    /// Rows outlive a scenario: the target is shared between scenarios, which §8 permits as long as
    /// one scenario's observations cannot satisfy another's.
    shared: bool,
    namespaced: bool,
    attribute_free: bool,
    switches: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    jobs: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
    scenario: RefCell<String>,
    calls: RefCell<CallTrace>,
    first_caller: RefCell<Option<Node>>,
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
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        if self.fault == Fault::CannotIsolate {
            return Err(TargetError::unsupported(
                "begin_scenario",
                "empty logical namespace unavailable",
            ));
        }
        *self.scenario.borrow_mut() = context.scenario.to_string();
        if !self.shared {
            self.switches.borrow_mut().clear();
            self.jobs.borrow_mut().clear();
        }
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    #[allow(clippy::too_many_lines)] // Keep the reference target's command effects together.
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let supplied = if self.attribute_free {
            assert!(
                request.caller.is_none(),
                "no invented credential attributes"
            );
            request
                .actor
                .as_ref()
                .map(|actor| BTreeMap::from([("principal_id".into(), text(&actor.to_string()))]))
        } else {
            request.caller.clone()
        };
        let Some(caller) = supplied else {
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
            let result = match jobs.get(&self.storage_key(&key(&job_id))) {
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
        self.calls
            .borrow_mut()
            .entry(self.scenario.borrow().clone())
            .or_default()
            .push((command.to_string(), switch_id.clone(), acted_by.clone()));
        let mut first = self.first_caller.borrow_mut();
        let first = first.get_or_insert_with(|| acted_by.clone());
        if self.fault == Fault::OnlyFirstCaller && first != &acted_by {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        }
        let row_key = if self.fault == Fault::KeysByCaller {
            format!("{}/{}", key(&acted_by), key(&switch_id))
        } else {
            key(&switch_id)
        };
        let row_key = self.storage_key(&row_key);
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
            "demo.plant.Switches" => self.rows(&self.switches.borrow()),
            "demo.plant.Jobs" => self.rows(&self.jobs.borrow()),
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
    fn storage_key(&self, key: &str) -> String {
        if self.namespaced {
            format!("{}/{key}", self.scenario.borrow())
        } else {
            key.into()
        }
    }

    fn rows(&self, rows: &BTreeMap<String, BTreeMap<String, Node>>) -> Vec<BTreeMap<String, Node>> {
        let prefix = format!("{}/", self.scenario.borrow());
        rows.iter()
            .filter(|(key, _)| !self.namespaced || key.starts_with(&prefix))
            .map(|(_, row)| row.clone())
            .collect()
    }
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
            self.storage_key(&key(&job_id)),
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
fn adv287_control_a_uuid_switch_suite_fails_a_target_keeping_one_switch_per_caller() {
    let model = with_id("{name: demo.plant.SwitchId, kind: newtype, of: Uuid}");
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let healthy = plant(Fault::None);
    assert_eq!(failed(&synthesis.suite, &healthy), Vec::new());
    let id = "demo.plant.StartJob/outcome/switch-paused";
    assert!(failed(&synthesis.suite, &plant(Fault::KeysByCaller))
        .iter()
        .any(|(scenario, status)| scenario == id && *status == Status::Failed));
    let trace = healthy.calls.borrow();
    let calls = &trace[id];
    let mut pairs = Vec::new();
    for (position, (command, identity, principal)) in calls.iter().enumerate() {
        if command != "demo.plant.StartJob" {
            continue;
        }
        let arrangements: Vec<_> = calls[..position]
            .iter()
            .filter(|(command, key, _)| command == "demo.plant.InstallSwitch" && key == identity)
            .collect();
        assert_eq!(
            arrangements.len(),
            1,
            "the acted row was installed exactly once: {calls:?}"
        );
        let owner = &arrangements[0].2;
        assert_ne!(
            owner, principal,
            "the caller actually changes on the same row"
        );
        assert!(calls[..position]
            .iter()
            .any(|(command, key, actor)| command == "demo.plant.PauseSwitch"
                && key == identity
                && actor == owner));
        pairs.push((owner.clone(), principal.clone()));
    }
    assert_eq!(pairs.len(), 2, "both caller orders actually execute");
    assert_eq!(pairs[0], (pairs[1].1.clone(), pairs[1].0.clone()));
}

#[test]
fn equal_grant_attribute_free_actors_act_on_the_same_shared_row() {
    let source = with_id("{name: demo.plant.SwitchId, kind: newtype, of: Uuid}")
        .replace(
            "    attributes:\n      - {name: principal_id, type: demo.plant.PrincipalId}\n",
            "",
        )
        .replace(", acted_by: {caller: principal_id}", "")
        .replace(
            "      - {name: acted_by, type: demo.plant.PrincipalId}\n",
            "",
        );
    let actor = source
        .split("actors:\n")
        .nth(1)
        .unwrap()
        .split("commands:\n")
        .next()
        .unwrap();
    let source = source.replace(
        actor,
        &format!(
            "{actor}{}",
            actor.replace("demo.plant.Operator", "demo.plant.SecondOperator")
        ),
    );
    let synthesis = synthesize(&ir(&source));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    let healthy = Plant {
        attribute_free: true,
        ..Plant::default()
    };
    assert_eq!(failed(&synthesis.suite, &healthy), Vec::new());
    let partitioned = Plant {
        attribute_free: true,
        fault: Fault::KeysByCaller,
        ..Plant::default()
    };
    assert!(failed(&synthesis.suite, &partitioned)
        .iter()
        .any(|(id, status)| id == "demo.plant.PauseSwitch/outcome/paused"
            && *status == Status::Failed));
    let trace = healthy.calls.borrow();
    let steps = &trace["demo.plant.PauseSwitch/outcome/paused"];
    let install = steps
        .iter()
        .find(|(command, _, _)| command == "demo.plant.InstallSwitch")
        .unwrap();
    let pause = steps
        .iter()
        .find(|(command, _, _)| command == "demo.plant.PauseSwitch")
        .unwrap();
    assert_eq!(install.1, pause.1);
    assert_ne!(install.2, pause.2);
}

/// Control for the shared-target case: the `Uuid` switch suite passes a correct target that keeps
/// its rows across scenarios, as §8 has every other suite do.
#[test]
fn adv287_control_a_uuid_switch_suite_passes_a_shared_correct_target() {
    let model = with_id("{name: demo.plant.SwitchId, kind: newtype, of: Uuid}");
    let synthesis = synthesize(&ir(&model));
    let shared = Plant {
        shared: true,
        namespaced: true,
        ..Plant::default()
    };
    let sentinel = BTreeMap::from([("unrelated".into(), text("retained"))]);
    shared
        .switches
        .borrow_mut()
        .insert("other-tenant/sentinel".into(), sentinel.clone());
    assert_eq!(failed(&synthesis.suite, &shared), Vec::new());
    assert_eq!(shared.switches.borrow()["other-tenant/sentinel"], sentinel);
    assert!(
        shared.switches.borrow().len() > 1,
        "scenario rows also remain physically stored"
    );
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
fn adv287_a_shared_target_is_told_the_singleton_scenarios_need_an_empty_one() {
    let synthesis = synthesize(&ir(ONE_SWITCH));
    let shared = Plant {
        shared: true,
        ..Plant::default()
    };
    assert!(
        !failed(&synthesis.suite, &shared).is_empty(),
        "persistent modeled rows violate the declared precondition"
    );
    let wire: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().unwrap()).unwrap();
    assert_eq!(wire["provenance"]["scenario_initial_state"], "empty");
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &plant(Fault::None))
        .into_report();
    assert!(report
        .to_string()
        .contains("Requires an empty logical modeled-instance/event/invocation namespace"));
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

mod support_go;
mod support_initial_state;

#[test]
#[allow(clippy::too_many_lines)] // Compare the two actual foreign runtimes against one native run.
fn live_go_and_typescript_match_native_shared_row_and_isolation_verdicts() {
    use ess_conformance::counts::CountReport;
    use std::process::Command;
    let model = ir(&with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}",
    ));
    let suite = synthesize(&model).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let directory = support_go::package(
        "initial-state",
        &suite,
        &[("live_test.go", support_initial_state::GO)],
    );
    let typescript = directory.join("typescript");
    std::fs::create_dir_all(&typescript).unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = typescript.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = typescript.join("essconform");
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(std::path::Path::new(&modules).join("@types"));
    }
    let output = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(package.join("live.mjs"), support_initial_state::TS).unwrap();
    for fault in [
        Fault::None,
        Fault::KeysByCaller,
        Fault::OnlyFirstCaller,
        Fault::CannotIsolate,
    ] {
        let target = plant(fault);
        let native = Runner::for_suite(&suite).run_admitted(&admitted, &target);
        let expected: serde_json::Value = serde_json::from_str(
            &CountReport::from_run(&native, &admitted)
                .unwrap()
                .to_canonical_json()
                .unwrap(),
        )
        .unwrap();
        let host = support_initial_state::Host::start(plant(fault));
        let go = support_go::go_test(&directory, "TestLive", &[("PARITY_ADDRESS", &host.address)]);
        let observed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(directory.join("report.json")).unwrap())
                .unwrap();
        assert_eq!(
            observed["counts"], expected["counts"],
            "Go {fault:?}: {}",
            go.log
        );
        assert_eq!(
            observed["outcomes"], expected["outcomes"],
            "Go {fault:?}: {}",
            go.log
        );
        assert!(go
            .log
            .contains("Requires an empty logical modeled-instance/event/invocation namespace"));
        let go_target = host.stop();
        let host = support_initial_state::Host::start(plant(fault));
        let report = directory.join("typescript-report.json");
        let output = Command::new("node")
            .arg("live.mjs")
            .arg("suite.json")
            .arg(&host.address)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&package)
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let observed: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&report).unwrap_or_else(|error| panic!("{error}: {log}")),
        )
        .unwrap();
        assert_eq!(
            observed["counts"], expected["counts"],
            "TypeScript {fault:?}: {log}"
        );
        assert_eq!(
            observed["outcomes"], expected["outcomes"],
            "TypeScript {fault:?}: {log}"
        );
        assert!(
            log.contains("Requires an empty logical modeled-instance/event/invocation namespace")
        );
        let ts_target = host.stop();
        let native_codes: std::collections::BTreeSet<_> = native
            .scenarios
            .iter()
            .flat_map(ess_conformance::ScenarioResult::diagnostics)
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        let codes = |log: &str| {
            ess_conformance::report::CheckCode::ALL
                .into_iter()
                .map(ess_conformance::CheckCode::as_str)
                .filter(|code| log.contains(code))
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(
            codes(&go.log),
            native_codes,
            "Go diagnostic set for {fault:?}: {}",
            go.log
        );
        assert_eq!(
            codes(&log),
            native_codes,
            "TypeScript diagnostic set for {fault:?}: {log}"
        );
        if fault == Fault::CannotIsolate {
            assert!(target.calls.borrow().is_empty());
            assert!(go_target.calls.borrow().is_empty());
            assert!(ts_target.calls.borrow().is_empty());
        } else {
            assert!(!go_target.calls.borrow().is_empty());
            assert!(!ts_target.calls.borrow().is_empty());
        }
    }
}

#[test]
fn wasm_executes_the_same_shared_row_and_isolation_fixtures() {
    let suite = synthesize(&ir(&with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}",
    )))
    .suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let observed = support_initial_state::wasm(&admitted);
    for (index, fault) in [
        Fault::None,
        Fault::KeysByCaller,
        Fault::OnlyFirstCaller,
        Fault::CannotIsolate,
    ]
    .into_iter()
    .enumerate()
    {
        let target = plant(fault);
        let native = Runner::for_suite(&suite).run_admitted(&admitted, &target);
        let count = ess_conformance::counts::CountReport::from_run(&native, &admitted).unwrap();
        let expected: serde_json::Value =
            serde_json::from_str(&count.to_canonical_json().unwrap()).unwrap();
        assert_eq!(
            observed[index]["report"]["counts"], expected["counts"],
            "{fault:?}"
        );
        assert_eq!(
            observed[index]["report"]["outcomes"], expected["outcomes"],
            "{fault:?}"
        );
        let codes: std::collections::BTreeSet<_> = native
            .scenarios
            .iter()
            .flat_map(ess_conformance::ScenarioResult::diagnostics)
            .map(|d| d.code.as_str())
            .collect();
        assert_eq!(
            observed[index]["codes"],
            serde_json::to_value(codes).unwrap(),
            "{fault:?}"
        );
        assert_eq!(
            observed[index]["calls"],
            target.calls.borrow().len(),
            "{fault:?}"
        );
    }
}

#[test]
fn same_command_arranger_and_actor_cross_the_exact_row() {
    for attributed in [false, true] {
        let mut source = with_id("{name: demo.plant.SwitchId, kind: newtype, of: Uuid}")
            .replace(", acted_by: {caller: principal_id}", "")
            .replace(
                "      - {name: acted_by, type: demo.plant.PrincipalId}\n",
                "",
            );
        if !attributed {
            source = source.replace(
                "    attributes:\n      - {name: principal_id, type: demo.plant.PrincipalId}\n",
                "",
            );
            let actor = source
                .split("actors:\n")
                .nth(1)
                .unwrap()
                .split("commands:\n")
                .next()
                .unwrap()
                .to_owned();
            source = source.replace(
                &actor,
                &format!(
                    "{actor}{}",
                    actor.replace("demo.plant.Operator", "demo.plant.SecondOperator")
                ),
            );
        }
        let synthesis = synthesize(&ir(&source));
        let healthy = Plant {
            attribute_free: !attributed,
            ..Plant::default()
        };
        assert_eq!(failed(&synthesis.suite, &healthy), Vec::new());
        let trace = healthy.calls.borrow();
        let calls = &trace["demo.plant.InstallSwitch/outcome/already-installed"];
        assert!(
            calls
                .windows(2)
                .any(|pair| pair[0].1 == pair[1].1 && pair[0].2 != pair[1].2),
            "attributed={attributed}: {calls:?}"
        );
        let partitioned = Plant {
            attribute_free: !attributed,
            fault: Fault::KeysByCaller,
            ..Plant::default()
        };
        assert!(failed(&synthesis.suite, &partitioned)
            .iter()
            .any(
                |(id, status)| id == "demo.plant.InstallSwitch/outcome/already-installed"
                    && *status == Status::Failed
            ));
    }
}

#[test]
fn upsert_updated_witness_crosses_callers_between_create_and_update() {
    use ess_conformance::scenario::ScenarioStep;
    let original =
        include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");
    for attributed in [false, true] {
        let actors = if attributed {
            "  - {name: demo.items.Admin, attributes: [{name: principal, type: String}], may: [demo.items.PutItem, demo.items.BookSlot]}"
        } else {
            "  - {name: demo.items.Admin, may: [demo.items.PutItem, demo.items.BookSlot]}\n  - {name: demo.items.SecondAdmin, may: [demo.items.PutItem, demo.items.BookSlot]}"
        };
        let model = ir(&original
            .replace("format: ess/16", "format: ess/18")
            .replace(
                "  - {name: demo.items.Admin, may: [demo.items.PutItem, demo.items.BookSlot]}",
                actors,
            ));
        let synthesis = synthesize(&model);
        let scenario =
            &synthesis.suite.scenarios[&"demo.items.PutItem/outcome/updated".parse().unwrap()];
        let calls: Vec<_> = scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand {
                    command,
                    actor,
                    caller,
                    input,
                    ..
                } if command.to_string() == "demo.items.PutItem" => Some((actor, caller, input)),
                _ => None,
            })
            .collect();
        assert!(calls.len() >= 2);
        assert_ne!(
            (calls[0].0, calls[0].1),
            (calls[1].0, calls[1].1),
            "attributed={attributed}"
        );
    }
}

#[test]
fn caller_sensitive_same_command_gap_is_explicit_without_false_mixed_claim() {
    let synthesis = synthesize(&ir(&with_id(
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}",
    )));
    let id = "demo.plant.InstallSwitch/outcome/already-installed";
    assert!(synthesis.notes.iter().any(
        |note| matches!(note,Note::CrossCallerUnwitnessed {scenario,..} if scenario.to_string()==id)
    ));
    assert!(!synthesis.notes.iter().any(
        |note| matches!(note,Note::CrossCallerUnswapped {scenario,..} if scenario.to_string()==id)
    ));
}

#[test]
fn caller_valued_upsert_records_the_remaining_per_invocation_witness_gap() {
    let original =
        include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");
    let source=original.replace("format: ess/16","format: ess/18")
        .replace("  - {name: demo.items.Admin, may: [demo.items.PutItem, demo.items.BookSlot]}",
            "  - {name: demo.items.Admin, attributes: [{name: principal, type: demo.items.Label}], may: [demo.items.PutItem, demo.items.BookSlot]}")
        .replace("label: input.label","label: {caller: principal}");
    let synthesis = synthesize(&ir(&source));
    assert!(synthesis
        .suite
        .scenarios
        .contains_key(&"demo.items.PutItem/outcome/updated".parse().unwrap()));
    assert!(synthesis.notes.iter().any(|note|matches!(note,Note::CrossCallerUnwitnessed {scenario,..} if scenario.to_string()=="demo.items.PutItem/outcome/updated")));
}
