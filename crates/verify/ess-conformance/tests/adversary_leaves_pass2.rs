//! Adversary pass 2 for per-leaf struct comparison (beyond10x/ess#179, E5).
//!
//! Attacks the pass-1 correction (`flatten_leaf` splitting a whole struct value by its declared
//! type) with the declared shapes it has to agree with — an `Optional` struct, a list of structs, a
//! map, an enum and a union inside the partly determined struct, a `Json` leaf — and runs each
//! generated suite against an implementation that copies every input leaf, so a key the runner
//! cannot satisfy shows up as a failing scenario rather than only as a key in a map. Mutant killers
//! drop or change one determined leaf and require the suite to notice.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const SET_LEAD: &str = "demo.dialer.SetLead/outcome/lead-set";

/// A model whose `SetLead` writes the nested mapping `mapping` to `lead`, both in its `LeadSet`
/// payload and in `sets:`. `mapping` is written without indentation, one line per leaf.
fn model(format: &str, types: &str, lead: &str, inputs: &str, mapping: &str) -> String {
    let indent = |by: usize| {
        mapping.lines().fold(String::new(), |mut out, line| {
            out.push_str(&" ".repeat(by));
            out.push_str(line);
            out.push('\n');
            out
        })
    };
    let payload = indent(14);
    let sets = indent(12);
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.dialer
types:
  - {{name: demo.dialer.AgentId, kind: newtype, of: String}}
  - name: demo.dialer.Place
    kind: struct
    fields:
      - {{name: city, type: String}}
      - {{name: code, type: String}}
{types}  - name: demo.dialer.Lead
    kind: struct
    fields:
{lead}entities:
  - name: demo.dialer.Membership
    identity: {{name: agent_id, type: demo.dialer.AgentId}}
    fields:
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
    lifecycle: {{initial: Idle, states: [Idle], terminal: [Idle]}}
events:
  - name: demo.dialer.Joined
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
  - name: demo.dialer.LeadSet
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
      - {{name: lead, type: demo.dialer.Lead}}
actors:
  - {{name: demo.dialer.Agent, may: [demo.dialer.Join, demo.dialer.SetLead]}}
commands:
  - name: demo.dialer.Join
    outcomes:
      - name: joined
        creates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.Joined]
        payload:
          demo.dialer.Joined: {{agent_id: {{generated: true}}}}
  - name: demo.dialer.SetLead
    input:
      - {{name: agent_id, type: demo.dialer.AgentId}}
{inputs}    outcomes:
      - name: lead-set
        updates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.LeadSet]
        payload:
          demo.dialer.LeadSet:
            agent_id: input.agent_id
            lead:
{payload}        sets:
          lead:
{sets}views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("dialer.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
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

fn payload(scenario: &ConformanceScenario) -> BTreeMap<String, Node> {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.dialer.LeadSet" =>
            {
                Some(payload.clone())
            }
            // The literal half of an expectation that also compares a captured identity
            // (beyond10x/ess#273).
            ScenarioStep::ExpectEventValues { event, payload, .. }
                if event.to_string() == "demo.dialer.LeadSet" =>
            {
                Some(
                    payload
                        .iter()
                        .filter_map(|(key, value)| match value {
                            ScenarioValue::Literal { value } => Some((key.clone(), value.clone())),
                            _ => None,
                        })
                        .collect(),
                )
            }
            _ => None,
        })
        .expect("the event is expected")
}

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

fn sent(scenario: &ConformanceScenario, field: &str) -> Option<Node> {
    scenario.steps.iter().find_map(|step| match step {
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
}

// ---- a target that builds `lead` from the command input -----------------------------------------

type Build = fn(&BTreeMap<String, Node>) -> BTreeMap<String, Node>;

struct Echo {
    build: Build,
    /// What the row stores instead, where it differs from what the event publishes.
    stored: Option<Build>,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Echo {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("echo-fixture", "1"))
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
                let lead = Node::Map((self.build)(&request.input));
                let kept = self
                    .stored
                    .map_or_else(|| lead.clone(), |stored| Node::Map(stored(&request.input)));
                row.insert("lead".to_owned(), kept);
                SemanticCommandResult::took(outcome(&command, "lead-set")).emitting(
                    ObservedEvent::new("demo.dialer.LeadSet".parse().unwrap())
                        .with("agent_id", Node::Text(id.clone()))
                        .with("lead", lead),
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

/// Every scenario that did not pass against `build`, with its failure.
fn failing(suite: &ConformanceSuite, build: Build) -> Vec<String> {
    failing_with(suite, build, None)
}

/// As [`failing`], with the row storing `stored` rather than what the event published.
fn failing_with(suite: &ConformanceSuite, build: Build, stored: Option<Build>) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite)
        .unwrap_or_else(|error| panic!("the synthesizer wrote a suite admission refuses: {error}"));
    let target = Echo {
        build,
        stored,
        rows: RefCell::default(),
        minted: Cell::new(0),
    };
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| format!("{}: {:?}", result.scenario, result))
        .collect()
}

fn copied(input: &BTreeMap<String, Node>, pairs: &[(&str, &str)]) -> BTreeMap<String, Node> {
    pairs
        .iter()
        .map(|(leaf, from)| {
            (
                (*leaf).to_owned(),
                input.get(*from).cloned().unwrap_or(Node::Null),
            )
        })
        .collect()
}

fn zero() -> Node {
    Node::Number(0_i64.into())
}

// ---- an Optional struct leaf read whole ----------------------------------------------------------

fn optional_place() -> String {
    model(
        "ess/14",
        "",
        "      - {name: id, type: String}\n      - {name: place, type: Optional<demo.dialer.Place>}\n      - {name: rank, type: Integer}\n",
        "      - {name: lead_id, type: String}\n      - {name: place, type: Optional<demo.dialer.Place>}\n",
        "id: input.lead_id\nplace: input.place\nrank: {generated: true}",
    )
}

fn optional_place_correct(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = copied(input, &[("id", "lead_id"), ("place", "place")]);
    lead.insert("rank".to_owned(), zero());
    lead
}

fn optional_place_wrong_city(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = optional_place_correct(input);
    if let Some(Node::Map(place)) = lead.get_mut("place") {
        place.insert("city".to_owned(), Node::Text("elsewhere".to_owned()));
    }
    lead
}

#[test]
fn an_optional_struct_leaf_read_whole_is_split_by_its_declared_fields() {
    let suite = suite(&optional_place());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let payload = payload(scenario);
    assert!(!payload.contains_key("lead.place"), "{payload:?}");
    {
        let Some(Node::Map(place)) = sent(scenario, "place") else {
            panic!("the scenario sends a place")
        };
        for leaf in ["city", "code"] {
            let path = format!("lead.place.{leaf}");
            assert_eq!(payload.get(&path), place.get(leaf), "{path} in {payload:?}");
        }
    }
}

#[test]
fn an_optional_struct_leaf_passes_a_copying_implementation() {
    assert_eq!(
        failing(&suite(&optional_place()), optional_place_correct),
        Vec::<String>::new()
    );
}

#[test]
fn an_optional_struct_leaf_with_a_wrong_city_fails_when_the_city_was_sent() {
    let suite = suite(&optional_place());
    let sent_place = matches!(
        sent(scenario(&suite, SET_LEAD), "place"),
        Some(Node::Map(_))
    );
    let failed = failing(&suite, optional_place_wrong_city);
    assert!(sent_place, "the scenario sends a place");
    {
        assert!(
            failed.iter().any(|f| f.starts_with(SET_LEAD)),
            "a wrong lead.place.city passed: {failed:?}"
        );
    }
}

// ---- a list of structs and a map, read whole ----------------------------------------------------

fn collections() -> String {
    model(
        "ess/14",
        "",
        "      - {name: id, type: String}\n      - {name: places, type: List<demo.dialer.Place>}\n      - {name: tags, type: \"Map<String, String>\"}\n      - {name: rank, type: Integer}\n",
        "      - {name: lead_id, type: String}\n      - {name: places, type: List<demo.dialer.Place>}\n      - {name: tags, type: \"Map<String, String>\"}\n",
        "id: input.lead_id\nplaces: input.places\ntags: input.tags\nrank: {generated: true}",
    )
}

fn collections_correct(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = copied(
        input,
        &[("id", "lead_id"), ("places", "places"), ("tags", "tags")],
    );
    lead.insert("rank".to_owned(), zero());
    lead
}

fn collections_extra_place(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = collections_correct(input);
    let extra = Node::Map(BTreeMap::from([
        ("city".to_owned(), Node::Text("x".to_owned())),
        ("code".to_owned(), Node::Text("y".to_owned())),
    ]));
    match lead.get_mut("places") {
        Some(Node::Seq(items)) => items.push(extra),
        _ => {
            lead.insert("places".to_owned(), Node::Seq(vec![extra]));
        }
    }
    lead
}

#[test]
fn a_list_and_a_map_leaf_are_asserted_whole_under_their_own_path() {
    let suite = suite(&collections());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let payload = payload(scenario);
    for (leaf, input) in [("lead.places", "places"), ("lead.tags", "tags")] {
        assert_eq!(payload.get(leaf), sent(scenario, input).as_ref(), "{leaf}");
    }
    assert!(
        !payload.keys().any(|key| key.starts_with("lead.places.")),
        "{payload:?}"
    );
}

#[test]
fn a_list_and_a_map_leaf_pass_a_copying_implementation() {
    assert_eq!(
        failing(&suite(&collections()), collections_correct),
        Vec::<String>::new()
    );
}

#[test]
fn an_implementation_adding_a_place_to_the_list_of_structs_fails() {
    let suite = suite(&collections());
    assert!(
        matches!(
            sent(scenario(&suite, SET_LEAD), "places"),
            Some(Node::Seq(_))
        ),
        "the scenario sends a list"
    );
    let failed = failing(&suite, collections_extra_place);
    assert!(failed.iter().any(|f| f.starts_with(SET_LEAD)), "{failed:?}");
}

// ---- an enum from input, an enum literal, a union and a Json leaf -------------------------------

fn kinds() -> String {
    model(
        "ess/15",
        "  - {name: demo.dialer.Service, kind: enum, variants: [Standard, Express]}\n  - name: demo.dialer.Choice\n    kind: union\n    tag: kind\n    variants:\n      text: String\n      count: Integer\n",
        "      - {name: id, type: String}\n      - {name: service, type: demo.dialer.Service}\n      - {name: tier, type: demo.dialer.Service}\n      - {name: choice, type: demo.dialer.Choice}\n      - {name: meta, type: Json}\n      - {name: rank, type: Integer}\n",
        "      - {name: lead_id, type: String}\n      - {name: service, type: demo.dialer.Service}\n      - {name: choice, type: demo.dialer.Choice}\n      - {name: meta, type: Json}\n",
        "id: input.lead_id\nservice: input.service\ntier: Express\nchoice: input.choice\nmeta: input.meta\nrank: {generated: true}",
    )
}

fn kinds_correct(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = copied(
        input,
        &[
            ("id", "lead_id"),
            ("service", "service"),
            ("choice", "choice"),
            ("meta", "meta"),
        ],
    );
    lead.insert("tier".to_owned(), Node::Text("Express".to_owned()));
    lead.insert("rank".to_owned(), zero());
    lead
}

fn kinds_wrong_tier(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = kinds_correct(input);
    lead.insert("tier".to_owned(), Node::Text("Standard".to_owned()));
    lead
}

#[test]
fn enum_union_and_json_leaves_are_asserted_and_admitted() {
    let suite = suite(&kinds());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let payload = payload(scenario);
    assert_eq!(
        payload.get("lead.tier"),
        Some(&Node::Text("Express".to_owned())),
        "{payload:?}"
    );
    for (leaf, input) in [
        ("lead.service", "service"),
        ("lead.choice", "choice"),
        ("lead.meta", "meta"),
    ] {
        {
            let value = sent(scenario, input).unwrap_or_else(|| panic!("`{input}` is sent"));
            assert_eq!(payload.get(leaf), Some(&value), "{leaf} in {payload:?}");
        }
    }
}

#[test]
fn enum_union_and_json_leaves_pass_a_copying_implementation() {
    assert_eq!(
        failing(&suite(&kinds()), kinds_correct),
        Vec::<String>::new()
    );
}

#[test]
fn an_implementation_writing_another_enum_literal_fails() {
    let failed = failing(&suite(&kinds()), kinds_wrong_tier);
    assert!(failed.iter().any(|f| f.starts_with(SET_LEAD)), "{failed:?}");
}

// ---- several generated leaves, two levels, one of them an Optional struct ------------------------

fn several() -> String {
    model(
        "ess/14",
        "  - name: demo.dialer.Contact\n    kind: struct\n    fields:\n      - {name: phone, type: String}\n      - {name: place, type: Optional<demo.dialer.Place>}\n      - {name: seq, type: Integer}\n",
        "      - {name: id, type: String}\n      - {name: uid, type: String}\n      - {name: contact, type: Optional<demo.dialer.Contact>}\n      - {name: rank, type: Integer}\n",
        "      - {name: lead_id, type: String}\n      - {name: phone, type: String}\n      - {name: city, type: String}\n",
        "id: input.lead_id\nuid: {generated: true}\ncontact:\n  phone: input.phone\n  place:\n    city: input.city\n    code: {generated: true}\n  seq: {generated: true}\nrank: {generated: true}",
    )
}

fn several_correct(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let place = BTreeMap::from([
        (
            "city".to_owned(),
            input.get("city").cloned().unwrap_or(Node::Null),
        ),
        ("code".to_owned(), Node::Text("c".to_owned())),
    ]);
    let contact = BTreeMap::from([
        (
            "phone".to_owned(),
            input.get("phone").cloned().unwrap_or(Node::Null),
        ),
        ("place".to_owned(), Node::Map(place)),
        ("seq".to_owned(), zero()),
    ]);
    BTreeMap::from([
        (
            "id".to_owned(),
            input.get("lead_id").cloned().unwrap_or(Node::Null),
        ),
        ("uid".to_owned(), Node::Text("u".to_owned())),
        ("contact".to_owned(), Node::Map(contact)),
        ("rank".to_owned(), zero()),
    ])
}

fn several_drop_city(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = several_correct(input);
    if let Some(Node::Map(contact)) = lead.get_mut("contact") {
        if let Some(Node::Map(place)) = contact.get_mut("place") {
            place.remove("city");
        }
    }
    lead
}

#[test]
fn several_generated_leaves_over_two_levels_assert_every_determined_leaf() {
    let suite = suite(&several());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let payload = payload(scenario);
    let keys: Vec<_> = payload.keys().cloned().collect();
    for key in ["lead.id", "lead.contact.phone", "lead.contact.place.city"] {
        assert!(payload.contains_key(key), "{key} in {keys:?}");
    }
    for key in [
        "lead.uid",
        "lead.rank",
        "lead.contact.seq",
        "lead.contact.place.code",
    ] {
        assert!(!payload.contains_key(key), "{key} in {keys:?}");
    }
    let row = row(scenario);
    for key in ["lead.id", "lead.contact.phone", "lead.contact.place.city"] {
        assert!(row.contains_key(key), "{key} in the row {row:?}");
    }
}

#[test]
fn several_generated_leaves_pass_a_copying_implementation() {
    assert_eq!(
        failing(&suite(&several()), several_correct),
        Vec::<String>::new()
    );
}

#[test]
fn an_implementation_dropping_the_innermost_determined_leaf_fails() {
    let failed = failing(&suite(&several()), several_drop_city);
    assert!(failed.iter().any(|f| f.starts_with(SET_LEAD)), "{failed:?}");
}

// ---- a newtype over a struct, as the field and as a leaf read whole ------------------------------

fn newtypes() -> String {
    model(
        "ess/14",
        "  - {name: demo.dialer.PlaceRef, kind: newtype, of: demo.dialer.Place}\n",
        "      - {name: id, type: String}\n      - {name: place, type: demo.dialer.PlaceRef}\n      - {name: rank, type: Integer}\n",
        "      - {name: lead_id, type: String}\n      - {name: place, type: demo.dialer.PlaceRef}\n",
        "id: input.lead_id\nplace: input.place\nrank: {generated: true}",
    )
    .replace(
        "  - name: demo.dialer.Lead\n    kind: struct",
        "  - {name: demo.dialer.LeadRef, kind: newtype, of: demo.dialer.Lead}\n  - name: demo.dialer.Lead\n    kind: struct",
    )
    .replace(
        "      - {name: lead, type: demo.dialer.Lead}",
        "      - {name: lead, type: demo.dialer.LeadRef}",
    )
}

#[test]
fn a_newtype_over_a_struct_is_walked_like_the_struct_it_wraps() {
    let suite = suite(&newtypes());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let payload = payload(scenario);
    let Some(Node::Map(place)) = sent(scenario, "place") else {
        panic!("the scenario sends a place")
    };
    for leaf in ["city", "code"] {
        let path = format!("lead.place.{leaf}");
        assert_eq!(payload.get(&path), place.get(leaf), "{path} in {payload:?}");
    }
    assert!(payload.contains_key("lead.id"), "{payload:?}");
    assert_eq!(
        failing(&suite, optional_place_correct),
        Vec::<String>::new()
    );
}

// ---- the story's scope: presence and type of the generated leaf, in `sets:` as in the payload -----

/// The #179 shape: four leaves read from the input, one generated, in the payload and in `sets:`.
fn issue_179() -> String {
    model(
        "ess/14",
        "",
        "      - {name: id, type: String}\n      - {name: uid, type: String}\n      - {name: number, type: String}\n      - {name: rank, type: Integer}\n      - {name: data, type: String}\n",
        "      - {name: lead_id, type: String}\n      - {name: lead_uid, type: String}\n      - {name: lead_number, type: String}\n      - {name: lead_data, type: String}\n",
        "id: input.lead_id\nuid: input.lead_uid\nnumber: input.lead_number\nrank: {generated: true}\ndata: input.lead_data",
    )
}

fn issue_179_correct(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = copied(
        input,
        &[
            ("id", "lead_id"),
            ("uid", "lead_uid"),
            ("number", "lead_number"),
            ("data", "lead_data"),
        ],
    );
    lead.insert("rank".to_owned(), zero());
    lead
}

/// The row keeps every copied leaf and never writes the generated `rank`.
fn issue_179_row_without_rank(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = issue_179_correct(input);
    lead.remove("rank");
    lead
}

/// The row writes the generated `rank` as text, where `Lead.rank` is an `Integer`.
fn issue_179_row_with_text_rank(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = issue_179_correct(input);
    lead.insert("rank".to_owned(), Node::Text("first".to_owned()));
    lead
}

#[test]
fn issue_179_passes_a_correct_implementation_here_too() {
    assert_eq!(
        failing(&suite(&issue_179()), issue_179_correct),
        Vec::<String>::new()
    );
}

/// Story scope: "a nested `sets:`/payload mapping with one `{generated: true}` leaf still asserts
/// every determined leaf, and presence and type for the generated one". The payload half holds
/// through the shape; nothing holds the `sets:` half, so a row missing `lead.rank` passes.
#[test]
fn issue_179_a_row_that_never_writes_the_generated_leaf_fails() {
    let failed = failing_with(
        &suite(&issue_179()),
        issue_179_correct,
        Some(issue_179_row_without_rank),
    );
    assert!(
        failed.iter().any(|f| f.starts_with(SET_LEAD)),
        "the row carries no `lead.rank` and every scenario passed"
    );
}

/// The same scope, for type: the row holds `lead.rank` as text.
///
/// Rewritten by coordinator decision (correction round 2): no existing view expectation can say
/// "holds an `Integer`", so the row's type of a generated leaf is not asserted; the event payload's
/// shape is, and `value-expressions.md` E5 says so. This pins today's behaviour.
#[test]
fn issue_179_a_row_holding_the_generated_leaf_at_the_wrong_type_is_not_caught_by_the_row() {
    let failed = failing_with(
        &suite(&issue_179()),
        issue_179_correct,
        Some(issue_179_row_with_text_rank),
    );
    assert!(
        !failed.iter().any(|f| f.starts_with(SET_LEAD)),
        "story:nested-struct-per-leaf-comparison: row type of a generated leaf is not asserted; \
         payload shape is — a change here needs E5 updated: {failed:?}"
    );
}

// ---- a partly determined struct only in `sets:`: the row alone moves the format --------------------

/// `LeadSet` publishes only the agent; the struct is written by `sets:` alone, so the only dotted
/// keys are in the view row. A `used_by` that looked at payloads only would leave the suite at an
/// older major whose readers look for a field literally named `lead.number`.
fn sets_only() -> String {
    issue_179()
        .replace(
            "      - {name: agent_id, type: demo.dialer.AgentId}\n      - {name: lead, type: demo.dialer.Lead}\n",
            "      - {name: agent_id, type: demo.dialer.AgentId}\n",
        )
        .replace(
            "            agent_id: input.agent_id\n            lead:\n              id: input.lead_id\n              uid: input.lead_uid\n              number: input.lead_number\n              rank: {generated: true}\n              data: input.lead_data\n",
            "            agent_id: input.agent_id\n",
        )
}

#[test]
fn a_struct_written_only_by_sets_moves_the_suite_to_26_and_its_coverage_to_27() {
    let text = sets_only();
    assert!(
        !text.contains("              number: input.lead_number"),
        "{text}"
    );
    let suite = suite(&text);
    let row = row(scenario(&suite, SET_LEAD));
    assert!(row.contains_key("lead.number"), "{row:?}");
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    let input = ess_conformance::coverage_build::build(
        &ir(&text),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        input
            .selected()
            .original_json()
            .contains("\"ess-conformance/35\""),
        "coverage is written as /27"
    );
    let mut pinned = suite.clone();
    pinned.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/25").unwrap();
    let error = AdmittedSuite::from_suite(&pinned).expect_err("a dotted row key at /25");
    assert!(
        error.to_string().contains("UnsupportedVocabulary"),
        "{error}"
    );
}

fn sets_only_row_drops_number(input: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let mut lead = issue_179_correct(input);
    lead.remove("number");
    lead
}

#[test]
fn a_struct_written_only_by_sets_fails_a_row_dropping_lead_number() {
    let suite = suite(&sets_only());
    assert_eq!(failing(&suite, issue_179_correct), Vec::<String>::new());
    let failed = failing(&suite, sets_only_row_drops_number);
    assert!(failed.iter().any(|f| f.starts_with(SET_LEAD)), "{failed:?}");
}
