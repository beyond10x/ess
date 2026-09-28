//! A nested struct target is compared leaf by leaf (beyond10x/ess#179,
//! `docs/design/value-expressions.md` E5).
//!
//! A nested `sets:` or payload mapping with one `{generated: true}` leaf used to assert nothing
//! about the struct at all, so an implementation that dropped a copied leaf passed. Each
//! determined leaf is now asserted under its dotted path — `lead.number` — and the generated one
//! stays covered by the shape, for presence and type. A suite carrying such a leaf is written as
//! `ess-conformance/26` (ordinary) or `/27` (coverage); one without keeps its earlier format.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{SuiteFormat, ViewExpectation};
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The #179 shape: four leaves read from the input, one generated.
const DIALER: &str = "format: ess/14
system: demo
version: v1
domain: demo.dialer
types:
  - {name: demo.dialer.AgentId, kind: newtype, of: String}
  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: uid, type: String}
      - {name: number, type: String}
      - {name: rank, type: Integer}
      - {name: data, type: String}
entities:
  - name: demo.dialer.Membership
    identity: {name: agent_id, type: demo.dialer.AgentId}
    fields:
      - {name: lead, type: Optional<demo.dialer.Lead>}
    lifecycle: {initial: Idle, states: [Idle], terminal: [Idle]}
events:
  - name: demo.dialer.Joined
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
  - name: demo.dialer.LeadSet
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: demo.dialer.Lead}
actors:
  - {name: demo.dialer.Agent, may: [demo.dialer.Join, demo.dialer.SetLead]}
commands:
  - name: demo.dialer.Join
    outcomes:
      - name: joined
        creates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.Joined]
        payload:
          demo.dialer.Joined: {agent_id: {generated: true}}
  - name: demo.dialer.SetLead
    input:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead_id, type: String}
      - {name: lead_uid, type: String}
      - {name: lead_number, type: String}
      - {name: lead_data, type: String}
    outcomes:
      - name: lead-set
        updates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.LeadSet]
        payload:
          demo.dialer.LeadSet:
            agent_id: input.agent_id
            lead:
              id: input.lead_id
              uid: input.lead_uid
              number: input.lead_number
              rank: {generated: true}
              data: input.lead_data
        sets:
          lead:
            id: input.lead_id
            uid: input.lead_uid
            number: input.lead_number
            rank: {generated: true}
            data: input.lead_data
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: Optional<demo.dialer.Lead>}
";

const SET_LEAD: &str = "demo.dialer.SetLead/outcome/lead-set";

/// The same model with every leaf determined: the whole struct is asserted, as before.
fn fully_determined() -> String {
    DIALER.replace("rank: {generated: true}", "rank: 3")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("dialer.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
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

/// What the scenario sent for one input field of `SetLead`.
fn sent(scenario: &ConformanceScenario, field: &str) -> Node {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.dialer.SetLead" =>
            {
                match input.get(field) {
                    Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{field}` is sent as a literal"))
}

/// The payload values and the shape the scenario's `LeadSet` expectation carries.
fn lead_set(scenario: &ConformanceScenario) -> (BTreeMap<String, Node>, Vec<String>) {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event,
                payload,
                shape,
            } if event.to_string() == "demo.dialer.LeadSet" => {
                Some((payload.clone(), shape.leaves().keys().cloned().collect()))
            }
            _ => None,
        })
        .expect("the event is expected")
}

/// The row the scenario requires the view to contain.
fn row(scenario: &ConformanceScenario) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .expect("a row is required")
}

#[test]
fn issue_179_every_determined_leaf_is_asserted_and_the_generated_one_by_its_shape() {
    let suite = suite(DIALER);
    let scenario = scenario(&suite, SET_LEAD);
    let (payload, leaves) = lead_set(scenario);
    for (leaf, input) in [
        ("lead.id", "lead_id"),
        ("lead.uid", "lead_uid"),
        ("lead.number", "lead_number"),
        ("lead.data", "lead_data"),
    ] {
        assert_eq!(
            payload.get(leaf),
            Some(&sent(scenario, input)),
            "{leaf} in {payload:?}"
        );
    }
    assert!(!payload.contains_key("lead"), "{payload:?}");
    assert!(!payload.contains_key("lead.rank"), "{payload:?}");
    assert!(
        leaves.iter().any(|leaf| leaf == "lead.rank"),
        "the generated leaf is covered by the shape: {leaves:?}"
    );

    let row = row(scenario);
    for (leaf, input) in [
        ("lead.id", "lead_id"),
        ("lead.uid", "lead_uid"),
        ("lead.number", "lead_number"),
        ("lead.data", "lead_data"),
    ] {
        assert_eq!(
            row.get(leaf),
            Some(&ScenarioValue::literal(sent(scenario, input))),
            "{leaf} in {row:?}"
        );
    }
    assert!(!row.contains_key("lead.rank"), "{row:?}");
}

#[test]
fn a_suite_with_a_partly_determined_struct_is_written_as_suite_26_and_its_coverage_as_27() {
    let suite = suite(DIALER);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
    assert!(ess_conformance::leaf_payloads::used_by(&suite));
    let input = ess_conformance::coverage_build::build(
        &ir(DIALER),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let original = input.selected().original_json().to_owned();
    assert!(original.contains("\"ess-conformance/27\""), "{original}");
    AdmittedSuite::from_json(&original).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn a_suite_whose_structs_are_fully_determined_keeps_its_format_and_the_whole_value() {
    let text = fully_determined();
    let suite = suite(&text);
    assert!(!ess_conformance::leaf_payloads::used_by(&suite));
    assert!(
        suite.provenance.suite_version.major() < 26,
        "{}",
        suite.provenance.suite_version
    );
    let scenario = scenario(&suite, SET_LEAD);
    let (payload, _) = lead_set(scenario);
    let Some(Node::Map(lead)) = payload.get("lead") else {
        panic!("the whole struct is asserted: {payload:?}")
    };
    assert_eq!(lead.get("number"), Some(&sent(scenario, "lead_number")));
    assert!(!payload.keys().any(|key| key.contains('.')), "{payload:?}");
}

/// The #179 model with `data` a struct read whole from a struct-typed input.
fn struct_leaf_from_input() -> String {
    DIALER
        .replace(
            "types:\n",
            "types:\n  - name: demo.dialer.Place\n    kind: struct\n    fields:\n      - {name: city, type: String}\n      - {name: code, type: String}\n",
        )
        .replace(
            "{name: data, type: String}",
            "{name: data, type: demo.dialer.Place}",
        )
        .replace(
            "{name: lead_data, type: String}",
            "{name: lead_data, type: demo.dialer.Place}",
        )
}

#[test]
fn a_struct_leaf_read_whole_is_asserted_by_its_scalar_leaves() {
    let suite = suite(&struct_leaf_from_input());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let Node::Map(place) = sent(scenario, "lead_data") else {
        panic!("a struct input is sent as a map")
    };
    let (payload, _) = lead_set(scenario);
    let row = row(scenario);
    assert!(!payload.contains_key("lead.data"), "{payload:?}");
    assert!(!row.contains_key("lead.data"), "{row:?}");
    for leaf in ["city", "code"] {
        let path = format!("lead.data.{leaf}");
        assert_eq!(payload.get(&path), place.get(leaf), "{path} in {payload:?}");
        assert_eq!(
            row.get(&path),
            place
                .get(leaf)
                .cloned()
                .map(ScenarioValue::literal)
                .as_ref(),
            "{path} in {row:?}"
        );
    }
}

#[test]
fn released_suite_formats_26_and_27_remain_supported_and_future_versions_refuse() {
    for version in ["ess-conformance/26", "ess-conformance/27"] {
        assert!(
            SuiteFormat::parse(version).unwrap().is_supported(),
            "{version}"
        );
    }
    assert!(!SuiteFormat::parse("ess-conformance/30")
        .unwrap()
        .is_supported());
}

#[test]
fn a_leaf_payload_pinned_below_suite_26_is_refused() {
    let mut suite = suite(DIALER);
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/24").unwrap();
    let error = AdmittedSuite::from_suite(&suite).expect_err("suite/24 cannot carry leaf payloads");
    assert!(
        error.to_string().contains("UnsupportedVocabulary"),
        "{error}"
    );
}

#[test]
fn a_leaf_payload_key_naming_no_leaf_of_the_shape_is_refused() {
    let mut suite = suite(DIALER);
    let mut renamed = false;
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectEvent { payload, .. } = step {
                if let Some(value) = payload.remove("lead.number") {
                    payload.insert("lead.numbr".to_owned(), value);
                    renamed = true;
                }
            }
        }
    }
    assert!(renamed);
    let error = AdmittedSuite::from_suite(&suite).expect_err("a misspelt leaf path");
    assert!(error.to_string().contains("lead.numbr"), "{error}");
}

// ---- running the suite ------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Copies every input leaf and keeps `rank` at 0.
    Correct,
    /// #179 as reported: `lead.number` is not written to the row.
    DropsNumberFromTheRow,
    /// The event carries a `lead.number`, but not the one it was sent: presence and type hold.
    WrongNumberOnTheEvent,
}

struct Dialer {
    mode: Mode,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Dialer {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Dialer {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("dialer-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
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
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.dialer.Join" => {
                let id = format!("agent-{}", self.minted.get());
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("agent_id".to_owned(), Node::Text(id.clone())),
                        ("lead".to_owned(), Node::Null),
                    ]),
                );
                SemanticCommandResult::took(outcome(&command, "joined")).emitting(
                    ObservedEvent::new("demo.dialer.Joined".parse().unwrap())
                        .with("agent_id", Node::Text(id)),
                )
            }
            "demo.dialer.SetLead" => {
                let Some(Node::Text(id)) = request.input.get("agent_id") else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(row) = rows.get_mut(id) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let lead = BTreeMap::from([
                    ("id".to_owned(), request.input["lead_id"].clone()),
                    ("uid".to_owned(), request.input["lead_uid"].clone()),
                    ("number".to_owned(), request.input["lead_number"].clone()),
                    ("rank".to_owned(), Node::Number(0_i64.into())),
                    ("data".to_owned(), request.input["lead_data"].clone()),
                ]);
                let mut stored = lead.clone();
                if self.mode == Mode::DropsNumberFromTheRow {
                    stored.remove("number");
                }
                let mut published = lead;
                if self.mode == Mode::WrongNumberOnTheEvent {
                    published.insert("number".to_owned(), request.input["lead_data"].clone());
                }
                row.insert("lead".to_owned(), Node::Map(stored));
                SemanticCommandResult::took(outcome(&command, "lead-set")).emitting(
                    ObservedEvent::new("demo.dialer.LeadSet".parse().unwrap())
                        .with("agent_id", Node::Text(id.clone()))
                        .with("lead", Node::Map(published)),
                )
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
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

/// Every scenario that did not pass against `mode`, by id.
fn failing(suite: &ConformanceSuite, mode: Mode) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Dialer::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect()
}

#[test]
fn issue_179_the_suite_passes_a_correct_implementation() {
    assert_eq!(failing(&suite(DIALER), Mode::Correct), Vec::<String>::new());
}

#[test]
fn issue_179_an_implementation_dropping_lead_number_from_the_row_fails() {
    assert_eq!(
        failing(&suite(DIALER), Mode::DropsNumberFromTheRow),
        vec![SET_LEAD.to_owned()]
    );
}

#[test]
fn issue_179_an_implementation_publishing_another_lead_number_fails() {
    assert_eq!(
        failing(&suite(DIALER), Mode::WrongNumberOnTheEvent),
        vec![SET_LEAD.to_owned()]
    );
}
