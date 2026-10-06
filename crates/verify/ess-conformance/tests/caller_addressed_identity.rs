//! The caller-swapped run of a synthesized scenario draws a fresh value for every literal identity
//! it sends, whether the command creates the instance or addresses one it does not create
//! (beyond10x/ess#465, after #275 and #430).
//!
//! `CommitTransaction` names its batch by a caller-supplied `transaction` and does not create it.
//! Its `external:` refusals are forced by a test adapter control whose means the target chooses: a
//! target may arrange a prepared batch under the sent identity before the command runs. The swapped
//! run used to send the first run's literal again, so such a target found the batch already
//! arranged and could not arrange the forced condition a second time.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Note};
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/caller-addressed-identity.yaml");

const COMMIT: &str = "demo.ledger.CommitTransaction";
const PREPARE: &str = "demo.ledger.PrepareBatch";
const REVIEW: &str = "demo.ledger.ReviewTransaction";

/// The three `external:` refusals of `CommitTransaction`.
const EXTERNAL: [&str; 3] = [
    "demo.ledger.CommitTransaction/outcome/invalid-reference",
    "demo.ledger.CommitTransaction/outcome/malformed-reference",
    "demo.ledger.CommitTransaction/outcome/delete-target-missing",
];
const WRONG_STATE: &str = "demo.ledger.CommitTransaction/outcome/wrong-state";
const COMMITTED: &str = "demo.ledger.CommitTransaction/outcome/committed";
const FLAGGED: &str = "demo.ledger.ReviewTransaction/outcome/flagged";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    synthesize(&ir(MODEL)).suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// One send of `command`: the value of `field` it carries, literal or not, and its caller.
#[derive(Debug, Clone, PartialEq)]
struct Send {
    value: ScenarioValue,
    caller: BTreeMap<String, Node>,
}

impl Send {
    fn literal(&self) -> Option<&Node> {
        match &self.value {
            ScenarioValue::Literal { value } => Some(value),
            _ => None,
        }
    }
}

/// Every send of `command` in `steps`, in order, with the value of `field`.
fn sends(steps: &[ScenarioStep], command: &str, field: &str) -> Vec<Send> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                caller,
                ..
            } if sent.to_string() == command => Some(Send {
                value: input
                    .get(field)
                    .cloned()
                    .unwrap_or_else(|| panic!("{command} sends `{field}`: {input:?}")),
                caller: caller.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// The first run of `scenario` and the caller-swapped one appended after it: the swapped run starts
/// at the next step of the kind (and, for a send, the command) the scenario starts with.
fn halves(scenario: &ConformanceScenario) -> (&[ScenarioStep], &[ScenarioStep]) {
    let head = |step: &ScenarioStep| match step {
        ScenarioStep::ExecuteCommand { command, .. } => format!("execute {command}"),
        ScenarioStep::ConfigureExternalOutcome { force, .. } => format!("configure {force}"),
        other => panic!("a run starts with a send or an external control: {other:?}"),
    };
    let first = head(&scenario.steps[0]);
    let at = scenario
        .steps
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, step)| {
            matches!(
                step,
                ScenarioStep::ExecuteCommand { .. } | ScenarioStep::ConfigureExternalOutcome { .. }
            ) && head(step) == first
        })
        .map_or_else(
            || panic!("the scenario runs twice: {:#?}", scenario.steps),
            |(at, _)| at,
        );
    scenario.steps.split_at(at)
}

fn verdicts(
    suite: &ConformanceSuite,
    target: &impl ConformanceTarget,
) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let detail = format!("{result:#?}");
            (result.scenario.to_string(), (result.status, detail))
        })
        .collect()
}

fn not_passed(verdicts: &BTreeMap<String, (Status, String)>) -> Vec<(&str, Status)> {
    verdicts
        .iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .inspect(|(_, (_, detail))| eprintln!("{detail}"))
        .map(|(id, (status, _))| (id.as_str(), *status))
        .collect()
}

/// Probe `a`'s refusals: each run of `CommitTransaction` sends its own literal `transaction`, as
/// its own caller, and the interpreter passes every scenario.
#[test]
fn swapped_run_redraws_addressed_identity() {
    let suite = suite();
    for id in EXTERNAL.iter().chain([&WRONG_STATE]) {
        let sent = sends(&scenario(&suite, id).steps, COMMIT, "transaction");
        assert_eq!(sent.len(), 2, "{id} runs once per caller: {sent:#?}");
        assert_ne!(sent[0].caller, sent[1].caller, "{id}: {sent:#?}");
        let (Some(first), Some(again)) = (sent[0].literal(), sent[1].literal()) else {
            panic!("{id} sends a literal transaction: {sent:#?}");
        };
        assert_ne!(
            first, again,
            "{id}'s swapped run sends a transaction of its own: {sent:#?}"
        );
    }
    let verdicts = verdicts(&suite, &Interpreted::for_model(ir(MODEL)));
    assert!(verdicts.len() >= 11, "{verdicts:#?}");
    assert_eq!(not_passed(&verdicts), Vec::new());
}

// ---- a target arranging the addressed record ---------------------------------------------------

/// The interpreter, behind an adapter that arranges a prepared batch under the `transaction` the
/// command after an external control sends, as a target may to bring the forced condition about.
/// One identity is arranged once per scenario: a second arrangement of it finds the batch already
/// there and cannot arrange the condition.
struct Arranging {
    inner: Interpreted,
    pending: RefCell<bool>,
    arranged: RefCell<BTreeSet<String>>,
}

impl Arranging {
    fn new() -> Self {
        Self {
            inner: Interpreted::for_model(ir(MODEL)),
            pending: RefCell::new(false),
            arranged: RefCell::default(),
        }
    }
}

impl ConformanceTarget for Arranging {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("arranging-465", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.pending.replace(false);
        self.arranged.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if self.pending.replace(false) {
            let transaction = request
                .input
                .get("transaction")
                .map(|value| serde_json::to_string(value).unwrap())
                .unwrap_or_default();
            if !self.arranged.borrow_mut().insert(transaction.clone()) {
                return Err(TargetError::unavailable(
                    "arranging the forced outcome",
                    format!("a batch is already prepared under transaction {transaction}"),
                ));
            }
        }
        self.inner.execute_command(request)
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
        self.pending.replace(true);
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// `suite` with the swapped run of each external refusal sending the first run's `transaction`
/// again, as the suite did before beyond10x/ess#465.
fn resent(suite: &ConformanceSuite) -> ConformanceSuite {
    let mut resent = suite.clone();
    for (id, scenario) in &mut resent.scenarios {
        if !EXTERNAL.contains(&id.to_string().as_str()) {
            continue;
        }
        let (first, again) = halves(scenario);
        let old = sends(first, COMMIT, "transaction")[0].literal().cloned();
        let new = sends(again, COMMIT, "transaction")[0].literal().cloned();
        let (Some(old), Some(new)) = (old, new) else {
            panic!("{id} sends a literal transaction");
        };
        let text = serde_json::to_string(again).unwrap().replace(
            &serde_json::to_string(&new).unwrap(),
            &serde_json::to_string(&old).unwrap(),
        );
        let again: Vec<ScenarioStep> = serde_json::from_str(&text).unwrap();
        let mut steps = first.to_vec();
        steps.extend(again);
        scenario.steps = steps;
    }
    resent
}

#[test]
fn arranging_target_passes_both_runs() {
    let suite = suite();
    let verdicts = verdicts(&suite, &Arranging::new());
    for id in EXTERNAL {
        assert!(verdicts.contains_key(id), "{id} is synthesized");
    }
    assert_eq!(not_passed(&verdicts), Vec::new());

    // The target is the one the issue describes: given the first run's identity again, it cannot
    // arrange the forced condition, and each external refusal errors.
    let resent = verdicts_of_resent(&suite);
    for id in EXTERNAL {
        assert_eq!(resent[id].0, Status::Error, "{}", resent[id].1);
    }
}

fn verdicts_of_resent(suite: &ConformanceSuite) -> BTreeMap<String, (Status, String)> {
    verdicts(&resent(suite), &Arranging::new())
}

// ---- copies of the addressed identity ----------------------------------------------------------

/// The swapped run of `ReviewTransaction/outcome/flagged` expects `BatchFlagged` carrying the
/// fresh `transaction` it sent; no step of any swapped run carries the first run's addressed
/// literal — an event payload, a view row or a parameter included.
#[test]
fn addressed_identity_follows_its_copies() {
    let suite = suite();
    let (first, again) = halves(scenario(&suite, FLAGGED));
    let mut carried = Vec::new();
    for run in [first, again] {
        let sent = sends(run, REVIEW, "transaction");
        assert_eq!(sent.len(), 1, "{sent:#?}");
        let literal = sent[0].literal().expect("a literal transaction").clone();
        let payload = run
            .iter()
            .find_map(|step| match step {
                ScenarioStep::ExpectEvent { event, payload, .. }
                    if event.to_string() == "demo.ledger.BatchFlagged" =>
                {
                    payload.get("transaction").cloned()
                }
                _ => None,
            })
            .expect("the run expects BatchFlagged carrying its transaction");
        assert_eq!(
            payload, literal,
            "the event copies the run's own transaction"
        );
        carried.push(literal);
    }
    assert_ne!(carried[0], carried[1], "{carried:#?}");

    for id in EXTERNAL.iter().chain([&WRONG_STATE, &FLAGGED]) {
        let (first, again) = halves(scenario(&suite, id));
        let command = if *id == FLAGGED { REVIEW } else { COMMIT };
        let old = sends(first, command, "transaction")[0]
            .literal()
            .cloned()
            .expect("a literal transaction");
        let old = serde_json::to_string(&old).unwrap();
        let text = serde_json::to_string(again).unwrap();
        assert!(
            !text.contains(&old),
            "{id}'s swapped run carries the first run's transaction {old}: {text}"
        );
    }
}

/// `committed` arranges its batch by `PrepareBatch` and commits it: each run names one row in both
/// sends, and the swapped run's row is not the first run's.
#[test]
fn created_then_addressed_stays_consistent() {
    let suite = suite();
    let (first, again) = halves(scenario(&suite, COMMITTED));
    let mut prepared = Vec::new();
    for (run, suffix) in [(first, ""), (again, "-swapped")] {
        let created = sends(run, PREPARE, "transaction");
        assert_eq!(created.len(), 1, "{created:#?}");
        let literal = created[0].literal().expect("a literal transaction").clone();
        let committed = sends(run, COMMIT, "transaction");
        assert_eq!(committed.len(), 1, "{committed:#?}");
        match &committed[0].value {
            ScenarioValue::Literal { value } => assert_eq!(value, &literal),
            ScenarioValue::Instance { instance } => {
                let captured = run.iter().any(|step| {
                    matches!(step, ScenarioStep::CaptureInstance { instance: name, event, .. }
                        if name == instance && event.to_string() == "demo.ledger.BatchPrepared")
                });
                assert!(
                    captured && instance.to_string().ends_with(suffix),
                    "the commit names the row its own run prepared: {run:#?}"
                );
            }
            other => panic!("{other:?}"),
        }
        prepared.push(literal);
    }
    assert_ne!(prepared[0], prepared[1], "{prepared:#?}");
    let verdicts = verdicts(&suite, &Interpreted::for_model(ir(MODEL)));
    assert_eq!(
        verdicts[COMMITTED].0,
        Status::Passed,
        "{}",
        verdicts[COMMITTED].1
    );
}

// ---- a struct identity -------------------------------------------------------------------------

/// Boxes addressed by a struct identity; `SealBox` does not create its box and has an external
/// refusal, so its scenario sends a literal slot.
const BOXES: &str = "format: ess/22
system: demo
version: v1
domain: demo.box
summary: Boxes addressed by a struct identity, sealed by an authenticated caller.
types:
  - name: demo.box.Slot
    kind: struct
    fields:
      - {name: shelf, type: String}
      - {name: label, type: String}
entities:
  - name: demo.box.Box
    identity: {name: slot, type: demo.box.Slot}
    fields:
      - {name: opened_by, type: String}
    lifecycle:
      initial: Open
      states: [Open, Sealed]
      terminal: [Sealed]
      transitions:
        - {name: seal, from: [Open], to: Sealed}
actors:
  - name: demo.box.Clerk
    attributes:
      - {name: subject, type: String}
    may: [demo.box.OpenBox, demo.box.SealBox]
commands:
  - name: demo.box.OpenBox
    input:
      - {name: slot, type: demo.box.Slot}
    outcomes:
      - name: opened
        creates: demo.box.Box
        instance: slot
        sets: {opened_by: {caller: subject}}
        emits: [demo.box.BoxOpened]
        payload:
          demo.box.BoxOpened: {slot: input.slot}
  - name: demo.box.SealBox
    input:
      - {name: slot, type: demo.box.Slot}
    outcomes:
      - name: jammed
        external: the seal jams
        error: demo.box.Jammed
      - name: sealed
        moves: demo.box.Box.seal
        instance: slot
        sets: {opened_by: {caller: subject}}
        emits: [demo.box.BoxSealed]
        payload:
          demo.box.BoxSealed: {slot: input.slot}
      - name: not-open
        wrong_state: true
        error: demo.box.Jammed
events:
  - name: demo.box.BoxOpened
    fields:
      - {name: slot, type: demo.box.Slot}
  - name: demo.box.BoxSealed
    fields:
      - {name: slot, type: demo.box.Slot}
errors:
  - name: demo.box.Jammed
";

#[test]
fn addressed_struct_identity_member_fresh() {
    let suite = synthesize(&ir(BOXES)).suite;
    let sent = sends(
        &scenario(&suite, "demo.box.SealBox/outcome/jammed").steps,
        "demo.box.SealBox",
        "slot",
    );
    assert_eq!(sent.len(), 2, "{sent:#?}");
    let (Some(Node::Map(first)), Some(Node::Map(again))) = (sent[0].literal(), sent[1].literal())
    else {
        panic!("each run sends a literal struct slot: {sent:#?}");
    };
    for name in ["shelf", "label"] {
        assert_ne!(
            first[name], again[name],
            "the swapped run's `slot.{name}` is its own: {sent:#?}"
        );
    }
}

// ---- a one-value identity ----------------------------------------------------------------------

/// The one switch: an identity of one value, which `Flip` addresses and does not create.
const SWITCH: &str = "format: ess/22
system: demo
version: v1
domain: demo.switch
summary: The one switch, flipped by an authenticated caller.
types:
  - {name: demo.switch.SwitchId, kind: newtype, of: Integer, invariants: [value >= 1, value <= 1]}
entities:
  - name: demo.switch.Switch
    identity: {name: switch, type: demo.switch.SwitchId}
    fields:
      - {name: flipped_by, type: String}
    lifecycle:
      initial: Off
      states: [Off, On]
      terminal: [On]
      transitions:
        - {name: flip, from: [Off], to: On}
actors:
  - name: demo.switch.Operator
    attributes:
      - {name: subject, type: String}
    may: [demo.switch.Install, demo.switch.Flip]
commands:
  - name: demo.switch.Install
    input:
      - {name: switch, type: demo.switch.SwitchId}
    outcomes:
      - name: installed
        creates: demo.switch.Switch
        instance: switch
        sets: {flipped_by: {caller: subject}}
        emits: [demo.switch.Installed]
        payload:
          demo.switch.Installed: {switch: input.switch}
  - name: demo.switch.Flip
    input:
      - {name: switch, type: demo.switch.SwitchId}
    outcomes:
      - name: jammed
        external: the relay jams
        error: demo.switch.Jammed
      - name: flipped
        moves: demo.switch.Switch.flip
        instance: switch
        sets: {flipped_by: {caller: subject}}
        emits: [demo.switch.Flipped]
        payload:
          demo.switch.Flipped: {switch: input.switch}
      - name: already-on
        wrong_state: true
        error: demo.switch.Jammed
events:
  - name: demo.switch.Installed
    fields:
      - {name: switch, type: demo.switch.SwitchId}
  - name: demo.switch.Flipped
    fields:
      - {name: switch, type: demo.switch.SwitchId}
errors:
  - name: demo.switch.Jammed
";

/// No fresh value is left for the one switch: the external refusal is never sent the one value
/// twice. It takes the one-row route (sent once, as the other caller) or keeps its first run with
/// the note naming the input.
#[test]
fn singleton_addressed_identity_keeps_one_row_route() {
    const JAMMED: &str = "demo.switch.Flip/outcome/jammed";
    let synthesis = synthesize(&ir(SWITCH));
    let sent = sends(
        &scenario(&synthesis.suite, JAMMED).steps,
        "demo.switch.Flip",
        "switch",
    );
    let literals: Vec<&Node> = sent.iter().filter_map(Send::literal).collect();
    let distinct: BTreeSet<String> = literals
        .iter()
        .map(|value| serde_json::to_string(value).unwrap())
        .collect();
    assert_eq!(
        distinct.len(),
        literals.len(),
        "the one switch is sent once: {sent:#?}"
    );
    let noted = synthesis.notes.iter().any(|note| {
        matches!(note, Note::UnswappedCallers { scenario, input, .. }
            if scenario.to_string() == JAMMED && input == "switch")
    });
    assert_eq!(sent.len(), 1, "the one switch is sent once: {sent:#?}");
    if !noted {
        // The one-row route: the command under test sent as the other caller than the one every
        // other step is sent as — `Install`'s own scenario runs once, as the first caller.
        let first = sends(
            &scenario(&synthesis.suite, "demo.switch.Install/outcome/installed").steps,
            "demo.switch.Install",
            "switch",
        );
        assert_ne!(
            sent[0].caller, first[0].caller,
            "the one-row route sends the command under test as the second caller: {sent:#?} {:#?}",
            synthesis.notes
        );
    }
}
