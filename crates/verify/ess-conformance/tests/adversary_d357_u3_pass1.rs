//! Adversary pass 1 against story:a-no-view-arranged-half-probes-the-row-through-the-command.
//!
//! * The staged model, with `configure` running from two states, files both refusal scenarios
//!   with their arranged half, passes the correct target and catches a target moving the record
//!   to `Configured` on refusal. A target that swaps the record on refusal between two states the
//!   probing branch runs from is caught too, through `Stage`, which tells them apart
//!   (story:a-no-view-probe-tells-from-states-apart); with `stage` running from both, nothing
//!   tells them apart and both scenarios are withdrawn naming the states.
//! * Where the first branch addressing the record has no valid send and a later one does, the
//!   arranged half probes through the later one and is kept.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const INPUT_REFUSAL: &str = include_str!("fixtures/arrangement-input-refusal.yaml");

/// The arrangement fixture with no view and a `Stage` command: `configure` runs from `New` and
/// from `Staged`, and `Stage` runs only from `New`, so the held state is decided by the model
/// through `Stage` even where `Configure` answers the same in both.
const STAGED: &str = r#"format: ess/14
system: vault
version: v1
domain: vault.acct
summary: An account is onboarded, optionally staged, then configured once from either state.
entities:
  - name: vault.acct.Account
    identity: {name: account_id, type: String}
    fields:
      - {name: issuer, type: String}
    lifecycle:
      initial: New
      states: [New, Staged, Configured]
      terminal: [Configured]
      transitions:
        - {name: stage, from: [New], to: Staged}
        - {name: configure, from: [New, Staged], to: Configured}
actors:
  - name: vault.acct.Operator
    may: [vault.acct.Onboard, vault.acct.Stage, vault.acct.Configure]
errors:
  - name: vault.acct.SecretTooShort
    fields: []
  - name: vault.acct.MissingConfiguration
    fields: []
  - name: vault.acct.IdRequired
    fields: []
  - name: vault.acct.AlreadyConfigured
    fields: []
  - name: vault.acct.NotNew
    fields: []
commands:
  - name: vault.acct.Onboard
    input:
      - {name: note, type: String}
    outcomes:
      - name: onboarded
        creates: vault.acct.Account
        instance: account_id
        emits: [vault.acct.Onboarded]
        payload:
          vault.acct.Onboarded: {account_id: {generated: true}}
  - name: vault.acct.Stage
    input:
      - {name: id, type: String}
    outcomes:
      - {name: not-new, wrong_state: true, error: vault.acct.NotNew}
      - name: staged
        moves: vault.acct.Account.stage
        instance: id
        emits: [vault.acct.Staged]
        payload:
          vault.acct.Staged: {account_id: input.id}
  - name: vault.acct.Configure
    input:
      - {name: id, type: String}
      - {name: secret, type: String}
      - {name: issuer, type: String}
    outcomes:
      - {name: id-required, when: id == "", error: vault.acct.IdRequired}
      - {name: secret-too-short, when: secret.count < 12, error: vault.acct.SecretTooShort}
      - {name: missing-configuration, when: issuer == "", error: vault.acct.MissingConfiguration}
      - {name: already-configured, wrong_state: true, error: vault.acct.AlreadyConfigured}
      - name: configured
        moves: vault.acct.Account.configure
        instance: id
        sets: {issuer: input.issuer}
        emits: [vault.acct.Configured]
        payload:
          vault.acct.Configured: {account_id: input.id}
events:
  - name: vault.acct.Onboarded
    fields:
      - {name: account_id, type: String}
  - name: vault.acct.Staged
    fields:
      - {name: account_id, type: String}
  - name: vault.acct.Configured
    fields:
      - {name: account_id, type: String}
"#;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&compiled(text))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn filed(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Input refusals first, then existence and the held state, then the move.
    Correct,
    /// [`Mode::Correct`], except a refusal of `Configure` for a stored account moves it to
    /// `Configured`, with no event.
    MovesOnRefusal,
    /// [`Mode::Correct`], except a refusal of `Configure` for a stored account swaps it between
    /// `New` and `Staged`, with no event. `Configure` runs from both, so its own valid send answers
    /// `configured` either way; `Stage` of a staged account answers `not-new`.
    SwapsFromStateOnRefusal,
}

struct Accounts {
    mode: Mode,
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, String>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Accounts {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            minted: Cell::new(0),
            rows: RefCell::default(),
            published: RefCell::default(),
        }
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

impl ConformanceTarget for Accounts {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("accounts-fixture-d357-u3", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        self.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let refuse = |name: &str, error: &str| {
            SemanticCommandResult::took(branch(&command, name))
                .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
        };
        let moved = |event: &str, id: &str| {
            ObservedEvent::new(event.parse::<EventRef>().unwrap())
                .with("account_id", Node::Text(id.to_owned()))
        };
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "vault.acct.Onboard" => {
                let id = format!("acct-{n}");
                rows.insert(id.clone(), "New".to_owned());
                SemanticCommandResult::took(branch(&command, "onboarded"))
                    .emitting(moved("vault.acct.Onboarded", &id))
            }
            "vault.acct.Stage" => {
                let id = text(request.input.get("id"));
                match rows.get_mut(&id) {
                    Some(state) if state == "New" => {
                        "Staged".clone_into(state);
                        SemanticCommandResult::took(branch(&command, "staged"))
                            .emitting(moved("vault.acct.Staged", &id))
                    }
                    _ => refuse("not-new", "vault.acct.NotNew"),
                }
            }
            "vault.acct.Configure" => {
                let id = text(request.input.get("id"));
                let secret = text(request.input.get("secret"));
                let issuer = text(request.input.get("issuer"));
                let refused = if id.is_empty() {
                    Some(("id-required", "vault.acct.IdRequired"))
                } else if secret.chars().count() < 12 {
                    Some(("secret-too-short", "vault.acct.SecretTooShort"))
                } else if issuer.is_empty() {
                    Some(("missing-configuration", "vault.acct.MissingConfiguration"))
                } else {
                    None
                };
                if let Some((name, error)) = refused {
                    if let Some(state) = rows.get_mut(&id) {
                        match (self.mode, state.as_str()) {
                            (Mode::MovesOnRefusal, _) => {
                                "Configured".clone_into(state);
                            }
                            (Mode::SwapsFromStateOnRefusal, "New") => {
                                "Staged".clone_into(state);
                            }
                            (Mode::SwapsFromStateOnRefusal, "Staged") => {
                                "New".clone_into(state);
                            }
                            _ => {}
                        }
                    }
                    refuse(name, error)
                } else {
                    match rows.get_mut(&id) {
                        Some(state) if state == "New" || state == "Staged" => {
                            "Configured".clone_into(state);
                            SemanticCommandResult::took(branch(&command, "configured"))
                                .emitting(moved("vault.acct.Configured", &id))
                        }
                        _ => refuse("already-configured", "vault.acct.AlreadyConfigured"),
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        panic!("no view is declared")
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
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn failures(text: &str, mode: Mode) -> BTreeMap<String, String> {
    let result = synthesis(text);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Accounts::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.status != Status::Passed)
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?}: {:?}", run.status, run.checks),
            )
        })
        .collect()
}

const REFUSED: [&str; 2] = [
    "vault.acct.Configure/outcome/secret-too-short",
    "vault.acct.Configure/outcome/missing-configuration",
];

/// The control: the correct target passes the staged model's suite, and both refusal scenarios
/// are filed with their arranged half, so a failure below is the mutant's.
#[test]
fn staged_model_suite_passes_the_correct_target() {
    let result = synthesis(STAGED);
    for id in REFUSED {
        assert!(filed(&result, id), "{id}: {:#?}", refusals(&result));
    }
    let failed = failures(STAGED, Mode::Correct);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The control's other half: moving the record to `Configured` on refusal is caught on this model
/// too, so the probe is present here.
#[test]
fn staged_model_catches_a_target_moving_the_record_to_configured_on_refusal() {
    let failed = failures(STAGED, Mode::MovesOnRefusal);
    for id in REFUSED {
        assert!(failed.contains_key(id), "{id}: {failed:#?}");
    }
}

/// The no-view fixture with a first addressing branch `quick`, shadowed by `secret-too-short`
/// (every input it admits is refused first), declared ahead of `configured`, which keeps its
/// valid send.
fn shadowed_first_addressing_branch() -> String {
    let cut = INPUT_REFUSAL
        .find("views:")
        .expect("the fixture declares a view");
    let text = INPUT_REFUSAL[..cut].replace(
        "      - name: configured\n        moves:",
        "      - name: quick\n        when: secret.count < 5\n        moves: \
         vault.acct.Account.configure\n        instance: id\n        sets: {issuer: \
         input.issuer}\n        emits: [vault.acct.Configured]\n        payload:\n          \
         vault.acct.Configured: {account_id: input.id}\n      - name: configured\n        moves:",
    );
    assert_eq!(text.matches("name: quick").count(), 1, "{text}");
    text
}

/// The acceptance withdraws the arranged half "when no valid send exists". Here one exists,
/// `configured`'s, so the arranged half must probe through it rather than be withdrawn because
/// the first addressing branch, `quick`, has none.
#[test]
fn no_view_arranged_half_is_kept_where_a_later_addressing_branch_has_a_valid_send() {
    let result = synthesis(&shadowed_first_addressing_branch());
    assert!(
        filed(&result, "vault.acct.Configure/outcome/configured"),
        "the fixture must keep `configured` reachable: {:#?}",
        refusals(&result)
    );
    for id in REFUSED {
        assert!(
            filed(&result, id),
            "{id} withdrawn although `configured` has a valid send to probe the arranged row \
             through (story:a-no-view-arranged-half-probes-the-row-through-the-command): {:#?}",
            refusals(&result)
        );
    }
}

/// The acceptance, "a target that changes the record on refusal fails the arranged half": a
/// target swapping the account between `New` and `Staged` on refusal changes the record, and the
/// model decides that change from the row (`Stage` answers `staged` from `New` and `not-new` from
/// `Staged`). The `Configure` probe alone answers `configured` from both, so the arranged half
/// must also send a branch that tells the two apart.
#[test]
fn no_view_arranged_half_catches_a_target_changing_the_from_state_on_refusal() {
    let failed = failures(STAGED, Mode::SwapsFromStateOnRefusal);
    for id in REFUSED {
        assert!(
            failed.contains_key(id),
            "{id} passed a target that swaps the stored account between two states `Configure` \
             runs from, on refusal, with no view; `Stage` would show it, the `Configure` probe \
             cannot (story:a-no-view-probe-tells-from-states-apart): {failed:#?}"
        );
    }
}

/// The staged model with `stage` running from `Staged` as well: every command now answers a
/// staged account as it answers a new one, so no send tells the two apart.
fn untellable() -> String {
    let text = STAGED.replace(
        "{name: stage, from: [New], to: Staged}",
        "{name: stage, from: [New, Staged], to: Staged}",
    );
    assert_ne!(text, STAGED, "the staged model's `stage` transition moved");
    text
}

/// Where no send tells the arranged state apart from another state the probe answers alike in,
/// a target moving the record between them on refusal would pass, so each arranged refusal
/// scenario is withdrawn and refused as `ESS-SYNTH-001` naming those states, never filed.
#[test]
fn no_view_arranged_half_is_withdrawn_naming_the_states_no_send_tells_apart() {
    let result = synthesis(&untellable());
    let lines = refusals(&result);
    for id in REFUSED {
        assert!(
            !filed(&result, id),
            "{id} filed although no send tells `New` from `Staged` with no view \
             (story:a-no-view-probe-tells-from-states-apart): {lines:#?}"
        );
        assert!(
            lines.iter().any(|line| line.contains("SYNTH-001")
                && line.contains(id)
                && line.contains("`New, Staged`")
                && line.contains("no send synthesis can make tells apart")),
            "{id} withdrawn without naming the states it cannot tell apart \
             (story:a-no-view-probe-tells-from-states-apart): {lines:#?}"
        );
    }
}
