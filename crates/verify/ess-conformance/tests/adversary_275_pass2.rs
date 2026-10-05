//! Adversary pass 2 for beyond10x/ess#275, against correction 1 of `synthesize/caller.rs`.
//!
//! Correction 1 replaces a text identity by value wherever the swapped run carries it, and a
//! number or Boolean identity only under names the model declares at the identity input's type.
//! Its fresh value is drawn from the type alone (`Identities::draw` passes no guards). These cases
//! drive each of those three choices against an implementation that does exactly what the model
//! says:
//!
//! - an integer identity copied into a field declared `Integer` (not the identity's newtype) keeps
//!   the first run's value there, so the swapped run expects a stale copy;
//! - an integer identity an outcome guards (`record_id < 100`) is redrawn past the guard, so the
//!   swapped run expects an outcome the model does not take for the value it sends;
//! - a text identity's plain witness is its input's own path (`record_id`), so value replacement
//!   also rewrites every *field name* the run spells as a string value — the fact path of an
//!   entity invariant the run asserts.
//!
//! The interpreter does not run `existing_instance:` branches, so the reference target is a
//! hand-written fixture, as in `tests/caller_fresh_identity.rs`.

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

/// An integer identity, copied into a stored field and an event field both declared `Integer`.
const INTEGER_COPIED: &str = "format: ess/16
system: demo
version: v1
summary: An integer identity copied under a plain Integer type.
conversions:
  - {from: demo.records.RecordId, to: Integer, because: The copy is the number the clerk sees.}
domain: demo.records
types:
  - {name: demo.records.RecordId, kind: newtype, of: Integer}
  - {name: demo.records.AccountId, kind: newtype, of: Uuid}
entities:
  - name: demo.records.Record
    identity: {name: record_id, type: demo.records.RecordId}
    fields:
      - {name: account_id, type: demo.records.AccountId}
      - {name: origin, type: Integer}
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
        sets: {account_id: {caller: account_id}, origin: input.record_id, text: input.text}
        emits: [demo.records.Recorded]
        payload:
          demo.records.Recorded: {record_id: input.record_id, echo: input.record_id, account_id: {caller: account_id}, text: input.text}
      - {name: already-recorded, existing_instance: true, error: demo.records.AlreadyRecorded}
errors:
  - name: demo.records.AlreadyRecorded
    summary: A record with this id already exists.
events:
  - name: demo.records.Recorded
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: echo, type: Integer}
      - {name: account_id, type: demo.records.AccountId}
      - {name: text, type: String}
views:
  - name: demo.records.RecordDetails
    source: demo.records.Record
    consistency: read_your_writes
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: account_id, type: demo.records.AccountId}
      - {name: origin, type: Integer}
      - {name: text, type: String}
";

/// An integer identity whose creating outcome is guarded on it.
const INTEGER_GUARDED: &str = "format: ess/16
system: demo
version: v1
summary: An integer identity the creating outcome guards.
domain: demo.records
types:
  - {name: demo.records.RecordId, kind: newtype, of: Integer}
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
        when: record_id < 100
        creates: demo.records.Record
        instance: record_id
        sets: {account_id: {caller: account_id}, text: input.text}
        emits: [demo.records.Recorded]
        payload:
          demo.records.Recorded: {record_id: input.record_id, account_id: {caller: account_id}, text: input.text}
      - {name: out-of-range, error: demo.records.OutOfRange}
      - {name: already-recorded, existing_instance: true, error: demo.records.AlreadyRecorded}
errors:
  - name: demo.records.AlreadyRecorded
    summary: A record with this id already exists.
  - name: demo.records.OutOfRange
    summary: Record ids stop at 99.
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

/// A text identity, copied into `origin`, with an entity invariant reading the identity field.
const TEXT_INVARIANT: &str = "format: ess/16
system: demo
version: v1
summary: A text identity an entity invariant reads.
domain: demo.records
types:
  - {name: demo.records.RecordId, kind: newtype, of: String}
  - {name: demo.records.AccountId, kind: newtype, of: Uuid}
entities:
  - name: demo.records.Record
    identity: {name: record_id, type: demo.records.RecordId}
    fields:
      - {name: account_id, type: demo.records.AccountId}
      - {name: origin, type: demo.records.RecordId}
      - {name: text, type: String}
    invariants:
      - record_id != \"none\"
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
        sets: {account_id: {caller: account_id}, origin: input.record_id, text: input.text}
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
      - {name: origin, type: demo.records.RecordId}
      - {name: text, type: String}
";

/// An identity of two values, `1..=2`, and two commands that update a record and so arrange one by
/// `RecordEntry`: the first swapped run takes the one value no scenario sends, the second has none
/// left and must be named in a note. (`RecordEntry`'s own scenarios are refused for so narrow a
/// type, before any swap.)
const NARROW: &str = "format: ess/16
system: demo
version: v1
summary: A caller-supplied identity of two values.
domain: demo.records
types:
  - {name: demo.records.RecordId, kind: newtype, of: Integer, invariants: [value >= 1, value <= 2]}
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
    may: [demo.records.RecordEntry, demo.records.Amend, demo.records.Rename]
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
  - name: demo.records.Amend
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
  - name: demo.records.Rename
    input:
      - {name: target, type: demo.records.RecordId}
      - {name: text, type: String}
    outcomes:
      - name: renamed
        updates: demo.records.Record
        instance: target
        sets: {text: input.text}
        emits: [demo.records.Renamed]
        payload:
          demo.records.Renamed: {record_id: input.target, text: input.text}
errors:
  - name: demo.records.AlreadyRecorded
    summary: A record with this id already exists.
events:
  - name: demo.records.Recorded
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: account_id, type: demo.records.AccountId}
      - {name: text, type: String}
  - name: demo.records.Amended
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: text, type: String}
  - name: demo.records.Renamed
    fields:
      - {name: record_id, type: demo.records.RecordId}
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

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("records.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
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

/// The scenario's two runs: the steps before the first `RecordEntry` sent by a caller other than
/// the first step's, and the steps from it on; `None` where it has one run only.
fn halves(scenario: &ConformanceScenario) -> Option<(&[ScenarioStep], &[ScenarioStep])> {
    let caller_of = |step: &ScenarioStep| match step {
        ScenarioStep::ExecuteCommand {
            command, caller, ..
        } if command.to_string() == "demo.records.RecordEntry" => Some(caller.clone()),
        _ => None,
    };
    let first = scenario.steps.iter().find_map(caller_of)?;
    let at = scenario
        .steps
        .iter()
        .position(|step| caller_of(step).is_some_and(|caller| caller != first))?;
    Some(scenario.steps.split_at(at))
}

/// Every literal `record_id` a run sends to `RecordEntry`, serialized.
fn sent_identities(steps: &[ScenarioStep]) -> BTreeSet<String> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.records.RecordEntry" =>
            {
                match input.get("record_id") {
                    Some(ScenarioValue::Literal { value }) => {
                        Some(serde_json::to_string(value).unwrap())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

/// Every `ExpectView` / `EventuallyView` expectation of a run, serialized, in order.
fn view_expectations(steps: &[ScenarioStep]) -> Vec<String> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. } => {
                Some(serde_json::to_string(expectation).unwrap())
            }
            _ => None,
        })
        .collect()
}

// ---- the reference target ----------------------------------------------------------------------

/// What the fixture implements of the three models.
#[derive(Clone, Copy, Default)]
struct Model {
    /// Stores the identity in `origin` too.
    origin: bool,
    /// Emits the identity in `Recorded.echo` too.
    echo: bool,
    /// Refuses an identity at or above this with `out-of-range`.
    limit: Option<f64>,
}

/// The model implemented as specified: refuses an identity already stored, copies it where the
/// model copies it, and guards it where the model guards it.
#[derive(Default)]
struct Ledger {
    model: Model,
    records: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Ledger {
    fn of(model: Model) -> Self {
        Self {
            model,
            ..Self::default()
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "adversary-275-pass2-fixture",
            "1",
        ))
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
        let key = serde_json::to_string(&id).unwrap();
        let mut records = self.records.borrow_mut();
        let numeric = serde_json::to_value(&id).ok().and_then(|v| v.as_f64());
        let result = if records.contains_key(&key) {
            SemanticCommandResult::took(outcome(&command, "already-recorded")).with_error(
                DeclaredErrorValue::new("demo.records.AlreadyRecorded".parse().unwrap()),
            )
        } else if let (Some(limit), Some(number)) = (self.model.limit, numeric) {
            if number >= limit {
                SemanticCommandResult::took(outcome(&command, "out-of-range")).with_error(
                    DeclaredErrorValue::new("demo.records.OutOfRange".parse().unwrap()),
                )
            } else {
                self.record(&mut records, &command, key, id, text, &caller)
            }
        } else {
            self.record(&mut records, &command, key, id, text, &caller)
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

impl Ledger {
    fn record(
        &self,
        records: &mut BTreeMap<String, BTreeMap<String, Node>>,
        command: &CommandRef,
        key: String,
        id: Node,
        text: Node,
        caller: &BTreeMap<String, Node>,
    ) -> SemanticCommandResult {
        let account = caller["account_id"].clone();
        let mut row = BTreeMap::from([
            ("record_id".to_owned(), id.clone()),
            ("account_id".to_owned(), account.clone()),
            ("text".to_owned(), text.clone()),
            ("state".to_owned(), Node::Text("Open".to_owned())),
        ]);
        if self.model.origin {
            row.insert("origin".to_owned(), id.clone());
        }
        records.insert(key, row);
        let mut event = ObservedEvent::new("demo.records.Recorded".parse().unwrap())
            .with("record_id", id.clone())
            .with("account_id", account)
            .with("text", text);
        if self.model.echo {
            event = event.with("echo", id);
        }
        SemanticCommandResult::took(outcome(command, "recorded")).emitting(event)
    }
}

/// Every scenario of `suite` not passed by the fixture, by id.
fn failed(suite: &ConformanceSuite, model: Model) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Ledger::of(model))
        .into_report();
    assert_ne!(report.scenarios.len(), 0);
    report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .inspect(|result| eprintln!("{result:#?}"))
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

// ---- the number limit --------------------------------------------------------------------------

/// `echo: input.record_id`, declared `Integer`: the swapped run sends a fresh `record_id` and must
/// expect that same value echoed, not the first run's.
#[test]
fn adversary_275_p2_an_integer_identity_copied_into_a_plain_integer_field_is_redrawn_there_too() {
    let suite = suite(INTEGER_COPIED);
    let (first, again) = halves(scenario(&suite, RECORDED_SCENARIO))
        .expect("the recording scenario runs under both callers");
    let old = sent_identities(first);
    let new = sent_identities(again);
    assert!(old.is_disjoint(&new), "the swapped run draws afresh");
    for step in again {
        if let ScenarioStep::ExpectEvent { event, payload, .. } = step {
            if event.to_string() == "demo.records.Recorded" {
                let echo = payload
                    .get("echo")
                    .map(|value| serde_json::to_string(value).unwrap())
                    .expect("Recorded.echo is expected");
                assert!(
                    new.contains(&echo),
                    "the swapped run sends record_id {new:?} and expects Recorded.echo {echo} \
                     (the first run's is {old:?})"
                );
            }
        }
    }
}

/// Acceptance 1 on the same model: an implementation that copies the identity it was sent passes
/// every scenario under both caller orders.
#[test]
fn adversary_275_p2_every_scenario_passes_when_an_integer_identity_is_copied_as_integer() {
    let model = Model {
        origin: true,
        echo: true,
        limit: None,
    };
    assert_eq!(failed(&suite(INTEGER_COPIED), model), Vec::new());
}

// ---- the draw ignores guards -------------------------------------------------------------------

/// `recorded` holds only below 100; the swapped run's fresh identity must still be one `recorded`
/// takes, or a correct implementation answers `out-of-range` where the scenario expects
/// `recorded`.
#[test]
fn adversary_275_p2_a_guarded_identity_is_redrawn_inside_its_guard() {
    let suite = suite(INTEGER_GUARDED);
    let (_, again) = halves(scenario(&suite, RECORDED_SCENARIO))
        .expect("the recording scenario runs under both callers");
    for sent in sent_identities(again) {
        let number: f64 = serde_json::from_str(&sent).unwrap();
        assert!(
            number < 100.0,
            "the swapped run expects `recorded` for record_id {sent}, which `when: record_id < 100` \
             refuses"
        );
    }
}

/// Acceptance 1 on the guarded model.
#[test]
fn adversary_275_p2_every_scenario_passes_when_the_identity_is_guarded() {
    let model = Model {
        origin: false,
        echo: false,
        limit: Some(100.0),
    };
    assert_eq!(failed(&suite(INTEGER_GUARDED), model), Vec::new());
}

// ---- text replaced by value --------------------------------------------------------------------

/// A text identity's plain witness spells its input's path, `record_id`. Replacing that text by
/// value throughout the swapped run must not rewrite what the run *names* `record_id`: the swapped
/// run's view expectations are the first run's, field for field.
#[test]
fn adversary_275_p2_value_replacement_leaves_field_names_alone() {
    let suite = suite(TEXT_INVARIANT);
    let mut compared = 0;
    for (id, scenario) in &suite.scenarios {
        let Some((first, again)) = halves(scenario) else {
            continue;
        };
        let old = sent_identities(first);
        let before: Vec<String> = view_expectations(first)
            .into_iter()
            .filter(|text| text.contains("\"satisfies\"") || text.contains("predicate"))
            .collect();
        let after: Vec<String> = view_expectations(again)
            .into_iter()
            .filter(|text| text.contains("\"satisfies\"") || text.contains("predicate"))
            .collect();
        compared += before.len();
        assert_eq!(
            before, after,
            "{id}: the swapped run (first run's identities {old:?}) asserts another invariant"
        );
    }
    assert!(compared > 0, "some scenario asserts the invariant");
}

/// Acceptance 1 on the text model.
#[test]
fn adversary_275_p2_every_scenario_passes_when_an_invariant_reads_a_text_identity() {
    let model = Model {
        origin: true,
        echo: false,
        limit: None,
    };
    assert_eq!(failed(&suite(TEXT_INVARIANT), model), Vec::new());
}

// ---- a type with too few values ---------------------------------------------------------------

/// Empty namespaces separate scenarios, while both caller orders within one scenario must execute
/// against the same state. Reusing the first arrangement's identity in the second must fail.
#[test]
fn caller_swaps_execute_in_empty_namespaces_and_reject_an_in_scenario_collision() {
    let model = ir(NARROW);
    let synthesis = synthesize(&model);
    assert_eq!(
        synthesis.suite.provenance.scenario_initial_state,
        Some(ess_conformance::scenario::ScenarioInitialState::Empty)
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("admitted suite");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model.clone()),
        )
        .into_report();
    assert_ne!(report.scenarios.len(), 0);
    for result in report.scenarios {
        assert_eq!(result.status, Status::Passed, "{result:#?}");
    }

    let mut faulty = synthesis.suite;
    faulty
        .scenarios
        .retain(|id, _| id.to_string() == "demo.records.Amend/outcome/amended");
    assert_eq!(faulty.scenarios.len(), 1);
    let scenario = faulty.scenarios.values_mut().next().unwrap();
    let mut first: Option<ScenarioValue> = None;
    let mut changed = 0;
    for step in &mut scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            if command.to_string() == "demo.records.RecordEntry" {
                let identity = input.get_mut("record_id").expect("creating identity");
                if let Some(first) = &first {
                    *identity = first.clone();
                    changed += 1;
                } else {
                    first = Some(identity.clone());
                }
            }
        }
    }
    assert_eq!(
        changed, 1,
        "one second arrangement receives the stale identity"
    );
    let admitted = AdmittedSuite::from_suite(&faulty).expect("admitted faulty suite");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    assert_eq!(report.scenarios.len(), 1);
    assert_eq!(report.scenarios[0].status, Status::Failed);
    assert!(report.scenarios[0]
        .diagnostics()
        .any(|diagnostic| diagnostic.code == ess_conformance::report::CheckCode::Outcome));
}

/// Every scenario that sends the creating command either runs under both callers or is named in a
/// `Note::UnswappedCallers`, never neither and never both; and a swapped run's identity is one no
/// first run in the same scenario sends. Suite34/35 requires an empty namespace before the next
/// scenario, so independently arranged scenarios may reuse a finite identity.
#[test]
fn adversary_275_p2_every_dropped_swap_is_noted_and_every_drawn_identity_is_unshared() {
    let synthesis = synthesize(&ir(NARROW));
    assert_eq!(
        synthesis.suite.provenance.scenario_initial_state,
        Some(ess_conformance::scenario::ScenarioInitialState::Empty)
    );
    let noted: BTreeSet<String> = synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            ess_conformance::synthesize::Note::UnswappedCallers { scenario, .. } => {
                Some(scenario.to_string())
            }
            _ => None,
        })
        .collect();
    let mut drawn = Vec::new();
    let mut sending = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        let id = id.to_string();
        if sent_identities(&scenario.steps).is_empty() {
            continue;
        }
        sending += 1;
        if let Some((first, again)) = halves(scenario) {
            assert!(!noted.contains(&id), "{id} runs swapped and is noted");
            let first_sent = sent_identities(first);
            let fresh = sent_identities(again);
            // This fixture's only Integer values are 1 and 2. Compare numeric values so JSON
            // spelling differences cannot disguise an identity collision.
            let old: Vec<f64> = first_sent
                .iter()
                .map(|value| serde_json::from_str(value).unwrap())
                .collect();
            for value in &fresh {
                let numeric: f64 = serde_json::from_str(value).unwrap();
                assert!(
                    !old.contains(&numeric),
                    "{id}: swapped identity {value} is reused"
                );
            }
            drawn.extend(fresh);
        } else {
            assert!(noted.contains(&id), "{id} runs once and no note says so");
        }
    }
    assert!(sending > 0, "some scenario sends record_id");
    assert!(
        !drawn.is_empty() && !noted.is_empty(),
        "one swap is drawn ({drawn:?}) and one is noted ({noted:?})"
    );
}
