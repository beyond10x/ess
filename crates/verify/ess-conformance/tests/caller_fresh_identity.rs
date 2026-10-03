//! The caller-swapped run of a synthesized scenario draws fresh values for caller-supplied identity
//! inputs (beyond10x/ess#275).
//!
//! A command takes its identity from input and declares an `existing_instance:` refusal; its actor
//! declares attributes, so every scenario sending it runs a second time with the two callers'
//! roles swapped (`synthesize/caller.rs`). That second run renamed instances but reused literal
//! inputs, so it sent the first run's identity again and still expected the creation — which an
//! implementation honouring `existing_instance:` answers with the refusal.
//!
//! The interpreter does not interpret an `existing_instance:` branch yet (it answers every
//! scenario sending the command `unsupported`), so the reference target here is a fixture that
//! implements the model as specified.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::{
    report::Status, synthesize::synthesize, AdmittedSuite, ConformanceScenario, ConformanceSuite,
    Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// A record whose identity the caller supplies, refused when it is already stored, and stamped
/// with the account of the caller that records it.
const RECORDS: &str = "format: ess/16
system: demo
version: v1
summary: A caller-supplied identity beside existing_instance.
domain: demo.records
types:
  - {name: demo.records.RecordId, kind: newtype, of: Uuid}
  - {name: demo.records.AccountId, kind: newtype, of: Uuid}
entities:
  - name: demo.records.Record
    identity: {name: record_id, type: demo.records.RecordId}
    fields:
      - {name: account_id, type: demo.records.AccountId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.records.Clerk
    attributes:
      - {name: account_id, type: demo.records.AccountId}
    may: [demo.records.RecordEntry]
commands:
  - name: demo.records.RecordEntry
    input:
      - {name: record_id, type: demo.records.RecordId}
      - {name: text, type: String}
    outcomes:
      - name: recorded
        creates: demo.records.Record
        instance: record_id
        sets: {account_id: {caller: account_id}, text: input.text}
        emits: [demo.records.Recorded]
        payload:
          demo.records.Recorded: {record_id: input.record_id, account_id: {caller: account_id}, text: input.text}
      - {name: already-recorded, existing_instance: true, error: demo.records.AlreadyRecorded}
errors:
  - name: demo.records.AlreadyRecorded
    summary: A record with this id already exists.
events:
  - name: demo.records.Recorded
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: account_id, type: demo.records.AccountId}
      - {name: text, type: String}
views:
  - name: demo.records.RecordDetails
    source: demo.records.Record
    consistency: read_your_writes
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: account_id, type: demo.records.AccountId}
      - {name: text, type: String}
";

const RECORDED_SCENARIO: &str = "demo.records.RecordEntry/outcome/recorded";
const ALREADY_SCENARIO: &str = "demo.records.RecordEntry/outcome/already-recorded";
const RECORDED: &str = "demo.records.RecordEntry/recorded";
const ALREADY: &str = "demo.records.RecordEntry/already-recorded";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("records.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&ir(RECORDS));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
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

/// One `RecordEntry` a scenario sends: the literal `record_id`, the caller's account, and the
/// outcome the steps after it require.
type Sent = (Option<Node>, Node, Option<String>);

/// Every `RecordEntry` the scenario sends, in order.
fn sends(scenario: &ConformanceScenario) -> Vec<Sent> {
    let mut sent = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { input, caller, .. } = step else {
            continue;
        };
        let identity = match input.get("record_id") {
            Some(ScenarioValue::Literal { value }) => Some(value.clone()),
            _ => None,
        };
        let expected = scenario.steps[at + 1..]
            .iter()
            .take_while(|step| !matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .find_map(|step| match step {
                ScenarioStep::ExpectOutcome { outcome } => Some(outcome.to_string()),
                _ => None,
            });
        sent.push((
            identity,
            caller.get("account_id").cloned().unwrap_or(Node::Null),
            expected,
        ));
    }
    sent
}

#[test]
fn issue_275_the_recording_scenario_runs_under_both_callers() {
    let suite = suite();
    let sent = sends(scenario(&suite, RECORDED_SCENARIO));
    let callers: BTreeSet<String> = sent
        .iter()
        .filter(|(_, _, expected)| expected.as_deref() == Some(RECORDED))
        .map(|(_, caller, _)| format!("{caller:?}"))
        .collect();
    assert_eq!(callers.len(), 2, "recorded once by each caller: {sent:#?}");
}

#[test]
fn issue_275_no_scenario_sends_one_identity_twice_expecting_the_creation_again() {
    let suite = suite();
    let mut checked = 0;
    for (id, scenario) in &suite.scenarios {
        let mut seen = BTreeSet::new();
        for (identity, _, expected) in sends(scenario) {
            let Some(identity) = identity else { continue };
            checked += 1;
            let again = !seen.insert(format!("{identity:?}"));
            assert!(
                !(again && expected.as_deref() == Some(RECORDED)),
                "{id} sends record_id {identity:?} again and expects {RECORDED}: {:#?}",
                sends(scenario)
            );
        }
    }
    assert!(checked > 0, "some scenario sends a literal record_id");
}

#[test]
fn issue_275_each_run_of_the_refusal_resends_the_identity_it_recorded() {
    let suite = suite();
    let sent = sends(scenario(&suite, ALREADY_SCENARIO));
    let refused: Vec<&Sent> = sent
        .iter()
        .filter(|(_, _, expected)| expected.as_deref() == Some(ALREADY))
        .collect();
    assert_eq!(refused.len(), 2, "refused once per run: {sent:#?}");
    assert_ne!(
        refused[0].0, refused[1].0,
        "each run refuses an identity of its own: {sent:#?}"
    );
    for (identity, account, _) in refused {
        let identity = identity
            .as_ref()
            .expect("the refusal sends a literal identity");
        assert!(
            sent.iter()
                .any(|(stored, by, expected)| stored.as_ref() == Some(identity)
                    && by != account
                    && expected.as_deref() == Some(RECORDED)),
            "the refused identity is one its own run recorded under the other caller: {sent:#?}"
        );
    }
}

// ---- running the suite ------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// As specified: records the caller's account, and refuses an identity already stored.
    Correct,
    /// Never answers `already-recorded`: a second record with one identity overwrites the first.
    Overwrites,
}

/// The model implemented by hand, with the stored records of one scenario.
struct Ledger {
    mode: Mode,
    records: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Ledger {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            records: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("records-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.records.borrow_mut().clear();
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
        assert_eq!(command.to_string(), "demo.records.RecordEntry");
        let (Some(id), Some(text)) = (
            request.input.get("record_id").cloned(),
            request.input.get("text").cloned(),
        ) else {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        };
        let Node::Text(key) = &id else {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        };
        let mut records = self.records.borrow_mut();
        let result = if self.mode == Mode::Correct && records.contains_key(key) {
            SemanticCommandResult::took(outcome(&command, "already-recorded")).with_error(
                DeclaredErrorValue::new("demo.records.AlreadyRecorded".parse().unwrap()),
            )
        } else {
            let account = caller["account_id"].clone();
            records.insert(
                key.clone(),
                BTreeMap::from([
                    ("record_id".to_owned(), id.clone()),
                    ("account_id".to_owned(), account.clone()),
                    ("text".to_owned(), text.clone()),
                    ("state".to_owned(), Node::Text("Open".to_owned())),
                ]),
            );
            SemanticCommandResult::took(outcome(&command, "recorded")).emitting(
                ObservedEvent::new("demo.records.Recorded".parse().unwrap())
                    .with("record_id", id)
                    .with("account_id", account)
                    .with("text", text),
            )
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.records.borrow().values().cloned(),
        ))
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

/// Every scenario that did not pass against `mode`, by id, out of how many ran.
fn failing(suite: &ConformanceSuite, mode: Mode) -> (Vec<(String, Status)>, usize) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Ledger::new(mode))
        .into_report();
    let ran = report.scenarios.len();
    let failed = report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .inspect(|result| eprintln!("{result:#?}"))
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    (failed, ran)
}

#[test]
fn issue_275_every_scenario_passes_an_implementation_honouring_existing_instance() {
    let (failed, ran) = failing(&suite(), Mode::Correct);
    assert!(ran > 0);
    assert_eq!(failed, Vec::new());
}

#[test]
fn issue_275_an_implementation_that_never_refuses_an_existing_record_fails_the_refusal() {
    let (failed, _) = failing(&suite(), Mode::Overwrites);
    let failed: Vec<&str> = failed.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(failed, [ALREADY_SCENARIO]);
}

// ---- a narrow identity type --------------------------------------------------------------------

const AMENDED_SCENARIO: &str = "demo.records.Amend/outcome/amended";

/// [`RECORDS`] with an integer identity in `1..=max` and a command that amends a record, whose
/// scenario arranges one by `RecordEntry` and so runs under both callers. (`RecordEntry`'s own
/// scenarios are refused for so narrow a type, before any swap.)
fn narrow(max: u32) -> EssIr {
    let text = RECORDS
        .replace(
            "{name: demo.records.RecordId, kind: newtype, of: Uuid}",
            &format!(
                "{{name: demo.records.RecordId, kind: newtype, of: Integer, invariants: [value >= 1, value <= {max}]}}"
            ),
        )
        .replace(
            "may: [demo.records.RecordEntry]",
            "may: [demo.records.RecordEntry, demo.records.Amend]",
        )
        .replace(
            "errors:\n",
            "  - name: demo.records.Amend
    input:
      - {name: target, type: demo.records.RecordId}
      - {name: text, type: String}
    outcomes:
      - name: amended
        updates: demo.records.Record
        instance: target
        sets: {text: input.text}
        emits: [demo.records.Amended]
        payload:
          demo.records.Amended: {record_id: input.target, text: input.text}
errors:\n",
        )
        .replace(
            "views:\n",
            "  - name: demo.records.Amended
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: text, type: String}
views:\n",
        );
    ir(&text)
}

/// The literal `record_id` of every `RecordEntry` the scenario sends, in order.
fn recorded_identities(scenario: &ConformanceScenario) -> Vec<Node> {
    sends(scenario)
        .into_iter()
        .filter_map(|(identity, _, _)| identity)
        .collect()
}

/// A type of three values: the swapped run takes one the suite does not send (the witness builder
/// answers every far distance of so narrow a range with its lowest value, so the near ones are
/// tried too) rather than dropping the swap.
#[test]
fn issue_275_a_narrow_identity_type_still_gives_the_swapped_run_an_unused_value() {
    let synthesis = synthesize(&narrow(3));
    let sent = recorded_identities(scenario(&synthesis.suite, AMENDED_SCENARIO));
    assert_eq!(sent.len(), 2, "arranged once per run: {sent:#?}");
    assert_ne!(sent[0], sent[1], "the swapped run arranges its own record");
}

/// A type of one value: no identity is left for a swapped run, so the scenario runs once — it does
/// not send the identity again. `Amend` acts on the one record without creating it, so that run
/// arranges the record as one caller and amends it as the other (beyond10x/ess#287) and carries no
/// note; `RecordEntry` creates the identity, so its own scenario keeps the note.
#[test]
fn issue_275_an_exhausted_identity_type_drops_the_swap_with_a_note() {
    use ess_conformance::synthesize::Note;
    let synthesis = synthesize(&narrow(1));
    let amended = sends(scenario(&synthesis.suite, AMENDED_SCENARIO));
    let sent: Vec<&Node> = amended
        .iter()
        .filter_map(|(id, _, _)| id.as_ref())
        .collect();
    assert_eq!(sent.len(), 1, "the record is arranged once: {amended:#?}");
    let callers: BTreeSet<String> = amended
        .iter()
        .map(|(_, caller, _)| serde_json::to_string(caller).unwrap())
        .collect();
    assert_eq!(
        callers.len(),
        2,
        "arranged and amended by two callers: {amended:#?}"
    );
    let noted = |id: &str| {
        synthesis.notes.iter().any(|note| {
            matches!(
                note,
                Note::UnswappedCallers { scenario, input, type_ref }
                    if scenario.to_string() == id
                        && input == "record_id"
                        && type_ref == "demo.records.RecordId"
            )
        })
    };
    assert!(!noted(AMENDED_SCENARIO), "{:#?}", synthesis.notes);
    assert!(noted(RECORDED_SCENARIO), "{:#?}", synthesis.notes);
}

/// `RECORDS` with an integer identity that `recorded` takes only when it is `7`: one value inside
/// the guard, which an arrangement may send, so no scenario has a fresh one to create with.
fn pinned() -> EssIr {
    let text = RECORDS
        .replace(
            "kind: newtype, of: Uuid}\n  - {name: demo.records.AccountId",
            "kind: newtype, of: Integer}\n  - {name: demo.records.AccountId",
        )
        .replace(
            "      - name: recorded\n",
            "      - name: recorded\n        when: record_id == 7\n",
        )
        .replace(
            "      - {name: already-recorded,",
            "      - {name: out-of-range, error: demo.records.AlreadyRecorded}\n      - {name: already-recorded,",
        );
    assert!(text.contains("record_id == 7") && text.contains("of: Integer}"));
    ir(&text)
}

/// Where no value inside the guard is left for a fresh identity, the scenario is refused, naming
/// the input: no step sends a value the guard refuses and still expects `recorded`.
#[test]
fn issue_275_a_guard_leaving_no_fresh_identity_refuses_rather_than_breaking_it() {
    let synthesis = synthesize(&pinned());
    for (id, scenario) in &synthesis.suite.scenarios {
        for (identity, _, expected) in sends(scenario) {
            if expected.as_deref() == Some(RECORDED) {
                assert_eq!(
                    identity.and_then(|value| serde_json::to_value(value).ok()),
                    Some(serde_json::json!(7.0)),
                    "{id} expects `recorded` for a record_id `when: record_id == 7` refuses"
                );
            }
        }
    }
    let refused = synthesis.refusals.iter().any(|refusal| {
        let text = refusal.to_string();
        text.contains("record_id") && text.contains("inside the guards")
    });
    assert!(refused, "{:#?}", synthesis.refusals);
}
