//! Adversary pass 2 against beyond10x/ess#287 (correction 1): the repeat-merge of a second
//! creation, the one-value detection, and the run that arranges the one row as one caller and acts
//! on it as the other.
//!
//! The same model and the same hand-written reference target shape as
//! `tests/adversary_287_pass1.rs`, with the faults and model shapes that file does not try.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, RefusalCause, Synthesis};
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

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("plant.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `ONE_SWITCH` with one exact piece of text replaced, which must be there.
fn edited(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model holds {from:?}");
    model.replacen(from, to, 1)
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
    /// One switch for everybody, but the "already installed" check asks whether *this caller*
    /// installed it: another caller's install is accepted and puts the switch back to `Running`,
    /// unpaused. Caller B can lift caller A's pause by installing again.
    ReinstallsForAnotherCaller,
    /// The installer-only guard is not enforced: anybody pauses.
    IgnoresInstaller,
}

/// The model implemented by hand: one switch row at most, keyed by its identity.
#[derive(Default)]
struct Plant {
    fault: Fault,
    /// Enforce `may:` grants: a command sent as an actor not granted it is a target error.
    grants: BTreeMap<String, Vec<String>>,
    /// Only the installer pauses (the `not-installer` model variant).
    installer_only: Cell<bool>,
    switches: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    /// Who installed each switch, by row key: kept apart from the row the view returns.
    installer: RefCell<BTreeMap<String, Node>>,
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
        Ok(ImplementationIdentity::new("adversary-287-p2-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.switches.borrow_mut().clear();
        self.installer.borrow_mut().clear();
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
        let command = request.command.clone();
        self.granted(&request)?;
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let Some(switch_id) = request.input.get("switch_id").cloned() else {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        };
        let acted_by = caller["principal_id"].clone();
        let row_key = key(&switch_id);
        let mut switches = self.switches.borrow_mut();
        let row = switches.get(&row_key).cloned();
        let state = row.as_ref().map(|row| row["state"].clone());
        let installer = self.installer.borrow().get(&row_key).cloned();
        let event = |name: &str| {
            ObservedEvent::new(name.parse().unwrap())
                .with("switch_id", switch_id.clone())
                .with("acted_by", acted_by.clone())
        };
        let projects_installer = self.installer_only.get();
        let mut put = |state: &str, paused: bool, by: Option<&Node>| {
            let mut row = BTreeMap::from([
                ("switch_id".to_owned(), switch_id.clone()),
                ("paused".to_owned(), Node::Bool(paused)),
                ("state".to_owned(), text(state)),
            ]);
            if let (true, Some(by)) = (projects_installer, by) {
                row.insert("installed_by".to_owned(), by.clone());
            }
            switches.insert(row_key.clone(), row);
        };
        let result = match command.to_string().as_str() {
            "demo.plant.InstallSwitch" => {
                let refused = match self.fault {
                    Fault::ReinstallsForAnotherCaller => {
                        row.is_some() && installer.as_ref() == Some(&acted_by)
                    }
                    _ => row.is_some(),
                };
                if refused {
                    error(&command, "already-installed", "demo.plant.AlreadyInstalled")
                } else {
                    put("Running", false, Some(&acted_by));
                    self.installer
                        .borrow_mut()
                        .insert(row_key.clone(), acted_by.clone());
                    SemanticCommandResult::took(outcome(&command, "installed"))
                        .emitting(event("demo.plant.SwitchInstalled"))
                }
            }
            "demo.plant.PauseSwitch" => {
                let installer_only =
                    self.installer_only.get() && self.fault != Fault::IgnoresInstaller;
                if row.is_some() && installer_only && installer.as_ref() != Some(&acted_by) {
                    error(&command, "not-installer", "demo.plant.NotInstaller")
                } else if state == Some(text("Running")) {
                    put("Paused", true, installer.as_ref());
                    SemanticCommandResult::took(outcome(&command, "paused"))
                        .emitting(event("demo.plant.SwitchPaused"))
                } else {
                    error(&command, "wrong-state", "demo.plant.SwitchStateConflict")
                }
            }
            "demo.plant.ResumeSwitch" => {
                if state == Some(text("Paused")) {
                    put("Running", false, installer.as_ref());
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
    /// A target error where grants are enforced and the request's actor is not granted its command.
    fn granted(&self, request: &SemanticCommandRequest) -> Result<(), TargetError> {
        if self.grants.is_empty() {
            return Ok(());
        }
        let command = request.command.to_string();
        let actor = request.actor.as_ref().map(ToString::to_string);
        let granted = actor.as_ref().is_some_and(|actor| {
            self.grants
                .get(actor)
                .is_some_and(|may| may.contains(&command))
        });
        if granted {
            Ok(())
        } else {
            Err(TargetError::unavailable(
                "authorizing",
                format!("{actor:?} is not granted {command}"),
            ))
        }
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
            if paused == Node::Bool(true) {
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
    assert_ne!(report.scenarios.len(), 0);
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

/// Acceptance 2, across callers. The story's need is one switch "so a guard on that row holds for
/// every caller". A target whose already-installed check asks whether *this caller* installed the
/// switch lets caller B install again and lift caller A's pause. `already-installed` is the
/// scenario that should fail it, and `caller.rs` `one_row_acted_on` keeps it single-caller:
/// `creates_it` is decided per command, not per outcome, so the existing-instance branch — whose
/// row exists — is never sent as the other caller.
#[test]
#[ignore = "known limitation, noted in the #287 PR (coordinator)"]
fn adv287p2_another_callers_second_install_is_refused_in_some_scenario() {
    let synthesis = synthesize(&ir(ONE_SWITCH));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
    let failed = failed(&synthesis.suite, &plant(Fault::ReinstallsForAnotherCaller));
    assert!(
        !failed.is_empty(),
        "a target that lets a second caller install the one switch again passes every scenario"
    );
}

/// Control for the case above: the same fault against the `Uuid` switch suite (byte-identical to
/// 0.49.0's). Red here too says the gap is synthesis at large (beyond10x/ess#312's family), not
/// this unit's alone.
#[test]
#[ignore = "known limitation, noted in the #287 PR (coordinator)"]
fn adv287p2_control_a_uuid_suite_and_another_callers_second_install() {
    let model = edited(
        ONE_SWITCH,
        "{name: demo.plant.SwitchId, kind: enum, variants: [Main]}",
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}",
    );
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
    let failed = failed(&synthesis.suite, &plant(Fault::ReinstallsForAnotherCaller));
    assert!(
        !failed.is_empty(),
        "a target that lets a second caller install a switch again passes every Uuid scenario"
    );
}

/// Fix 1 against a ranked view. A view with `order_by:` is arranged with two rows so its order is a
/// claim; for the singleton those are two installs of `Main`, sent alike. The repeat-merge drops
/// the second, and nothing after it is an outcome on the switch, so the merge goes through — but a
/// view expectation further on was read from a model holding both. Whatever synthesis does with
/// the order, a correct target holding one switch must pass every scenario it keeps.
#[test]
fn adv287p2_a_ranked_switch_view_passes_a_correct_target() {
    let model = edited(
        ONE_SWITCH,
        "  - name: demo.plant.Switches\n    source: demo.plant.Switch\n    consistency: read_your_writes\n",
        "  - name: demo.plant.Switches\n    source: demo.plant.Switch\n    consistency: read_your_writes\n    order_by:\n      - switch_id asc\n",
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
}

/// Acceptance 1 against the same ranked view. Ranking wants two switch rows, which a singleton
/// cannot have, so only the order claim has nothing to compare; but the withdrawal takes the
/// scenarios that carry the order assertion with it — `InstallSwitch/outcome/installed`,
/// `paused`, `resumed` and both transitions — each refused with ESS-SYNTH-001 "creates two in one
/// run". The same model with a `Uuid` switch keeps all 14 scenarios (byte-identical to 0.49.0).
#[test]
#[ignore = "known limitation, noted in the #287 PR (coordinator)"]
fn adv287p2_an_ordered_switch_view_withdraws_no_switch_scenario() {
    let model = edited(
        ONE_SWITCH,
        "  - name: demo.plant.Switches\n    source: demo.plant.Switch\n    consistency: read_your_writes\n",
        "  - name: demo.plant.Switches\n    source: demo.plant.Switch\n    consistency: read_your_writes\n    order_by:\n      - switch_id asc\n",
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(
        refused_for_the_switch(&synthesis),
        Vec::new(),
        "an order on the singleton's view refuses the scenarios that carry it"
    );
}

/// Fix 1 against a ranked view of the rows that reference the one row: two jobs arranged for the
/// order, each started against "its" switch — the merge's own case. A correct target passes.
#[test]
fn adv287p2_a_ranked_job_view_passes_a_correct_target() {
    let model = edited(
        ONE_SWITCH,
        "  - name: demo.plant.Jobs\n    source: demo.plant.Job\n    consistency: read_your_writes\n",
        "  - name: demo.plant.Jobs\n    source: demo.plant.Job\n    consistency: read_your_writes\n    order_by:\n      - job_id asc\n",
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(refused_for_the_switch(&synthesis), Vec::new());
    assert_eq!(failed(&synthesis.suite, &plant(Fault::None)), Vec::new());
}

/// Fix 3 with a caller-attribute guard: only the installer pauses (`when_subject` on the stored
/// `installed_by` against `caller.principal_id`). The run that sends `PauseSwitch` as the other
/// caller reaches `not-installer`, not `paused`, so `paused` must keep its single-caller run; a
/// correct target passes everything, and a target that lets anybody pause fails some scenario.
#[test]
fn adv287p2_an_installer_only_pause_passes_a_correct_target_and_fails_one_ignoring_it() {
    installer_only_holds(&installer_only(ONE_SWITCH));
}

/// Control for the case above: the same installer-only model with a `Uuid` switch identity.
#[test]
fn adv287p2_control_an_installer_only_pause_beside_a_uuid_switch() {
    installer_only_holds(&installer_only(&edited(
        ONE_SWITCH,
        "{name: demo.plant.SwitchId, kind: enum, variants: [Main]}",
        "{name: demo.plant.SwitchId, kind: newtype, of: Uuid}",
    )));
}

/// `model` where only the installer pauses: the switch stores and projects `installed_by`, set
/// from the caller, and `PauseSwitch` refuses any other caller.
fn installer_only(model: &str) -> String {
    let model = edited(
        model,
        "    fields:\n      - {name: paused, type: Boolean}\n",
        "    fields:\n      - {name: paused, type: Boolean}\n      - {name: installed_by, type: demo.plant.PrincipalId}\n",
    );
    let model = edited(
        &model,
        "        sets: {paused: false}\n        emits: [demo.plant.SwitchInstalled]\n",
        "        sets: {paused: false, installed_by: {caller: principal_id}}\n        emits: [demo.plant.SwitchInstalled]\n",
    );
    let model = edited(
        &model,
        "    outcomes:\n      - name: paused\n",
        "    outcomes:\n      - name: not-installer\n        when_subject: {predicate: installed_by != caller.principal_id}\n        error: demo.plant.NotInstaller\n      - name: paused\n",
    );
    let model = edited(
        &model,
        "errors:\n",
        "errors:\n  - name: demo.plant.NotInstaller\n    summary: Only the installer pauses.\n",
    );
    edited(
        &model,
        "      - {name: paused, type: Boolean}\n      - {name: state, type: demo.plant.Switch.State}\n",
        "      - {name: paused, type: Boolean}\n      - {name: installed_by, type: demo.plant.PrincipalId}\n      - {name: state, type: demo.plant.Switch.State}\n",
    )
}

/// The installer-only `model` synthesizes with nothing refused, a correct target passes it, and a
/// target that lets anybody pause fails `not-installer`.
fn installer_only_holds(model: &str) {
    let synthesis = synthesize(&ir(model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let correct = Plant {
        installer_only: Cell::new(true),
        ..Plant::default()
    };
    assert_eq!(failed(&synthesis.suite, &correct), Vec::new());
    let ignoring = Plant {
        installer_only: Cell::new(true),
        fault: Fault::IgnoresInstaller,
        ..Plant::default()
    };
    assert!(
        failed(&synthesis.suite, &ignoring)
            .iter()
            .any(|(id, _)| id == "demo.plant.PauseSwitch/outcome/not-installer"),
        "a target that lets anybody pause passes not-installer"
    );
}

/// Fix 3 with grants: the switch is an administrator's and jobs are an operator's, each actor with
/// a `principal_id`. The run that sends `StartJob` as the other caller must still send it as an
/// actor granted it, so a target enforcing `may:` passes every scenario.
#[test]
fn adv287p2_split_grants_pass_a_target_enforcing_them() {
    let model = edited(
        ONE_SWITCH,
        "    may:\n      - demo.plant.InstallSwitch\n      - demo.plant.PauseSwitch\n      - demo.plant.ResumeSwitch\n      - demo.plant.StartJob\n      - demo.plant.QueueJob\n",
        "    may:\n      - demo.plant.StartJob\n      - demo.plant.QueueJob\n  - name: demo.plant.Admin\n    attributes:\n      - {name: principal_id, type: demo.plant.PrincipalId}\n    may:\n      - demo.plant.InstallSwitch\n      - demo.plant.PauseSwitch\n      - demo.plant.ResumeSwitch\n",
    );
    let synthesis = synthesize(&ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let enforcing = Plant {
        grants: BTreeMap::from([
            (
                "demo.plant.Operator".to_owned(),
                vec![
                    "demo.plant.StartJob".to_owned(),
                    "demo.plant.QueueJob".to_owned(),
                ],
            ),
            (
                "demo.plant.Admin".to_owned(),
                vec![
                    "demo.plant.InstallSwitch".to_owned(),
                    "demo.plant.PauseSwitch".to_owned(),
                    "demo.plant.ResumeSwitch".to_owned(),
                ],
            ),
        ]),
        ..Plant::default()
    };
    assert_eq!(failed(&synthesis.suite, &enforcing), Vec::new());
}
