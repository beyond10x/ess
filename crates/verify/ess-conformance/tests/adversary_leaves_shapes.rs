//! Adversary cases for per-leaf struct comparison (beyond10x/ess#179, E5): shapes the unit's own
//! suite does not exercise.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::collections::BTreeMap;

use ess_primitives::node::Node;

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

fn event_payload(scenario: &ConformanceScenario, event: &str) -> BTreeMap<String, Node> {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event: name,
                payload,
                ..
            } if name.to_string() == event => Some(payload.clone()),
            _ => None,
        })
        .expect("the event is expected")
}

fn row_keys(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.keys().cloned().collect()),
            _ => None,
        })
        .expect("a row is required")
}

const HEAD: &str = "format: ess/14
system: demo
version: v1
domain: demo.dialer
types:
  - {name: demo.dialer.AgentId, kind: newtype, of: String}
  - name: demo.dialer.Place
    kind: struct
    fields:
      - {name: city, type: String}
      - {name: code, type: String}
";

/// `lead.place` is read whole from a struct-typed input, beside a generated `rank`.
///
/// The struct is partly determined, so its determined leaves are written under dotted paths — and
/// `lead.place` is one of them, holding a map. The payload shape has no leaf named `lead.place`
/// (a struct has no leaf of its own; `describe` walks to `lead.place.city`), so admission's
/// "names no leaf of the step's own shape" check refuses the suite the synthesizer just wrote.
fn struct_leaf_from_input() -> String {
    format!(
        "{HEAD}  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {{name: id, type: String}}
      - {{name: place, type: demo.dialer.Place}}
      - {{name: rank, type: Integer}}
entities:
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
      - {{name: lead_id, type: String}}
      - {{name: place, type: demo.dialer.Place}}
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
              place: input.place
              rank: {{generated: true}}
        sets:
          lead:
            id: input.lead_id
            place: input.place
            rank: {{generated: true}}
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
"
    )
}

/// Two nested levels: the outer struct is partly determined only because its inner one is.
fn two_levels() -> String {
    format!(
        "{HEAD}  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {{name: id, type: String}}
      - {{name: place, type: demo.dialer.Place}}
entities:
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
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
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
      - {{name: lead_id, type: String}}
      - {{name: city, type: String}}
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
              place:
                city: input.city
                code: {{generated: true}}
        sets:
          lead:
            id: input.lead_id
            place:
              city: input.city
              code: {{generated: true}}
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
"
    )
}

const SET_LEAD: &str = "demo.dialer.SetLead/outcome/lead-set";

#[test]
fn a_struct_leaf_read_whole_from_input_leaves_the_synthesized_suite_admissible() {
    let suite = suite(&struct_leaf_from_input());
    AdmittedSuite::from_suite(&suite)
        .unwrap_or_else(|error| panic!("the synthesizer wrote a suite admission refuses: {error}"));
}

#[test]
fn two_nested_levels_assert_the_inner_determined_leaf_in_payload_and_row() {
    let suite = suite(&two_levels());
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let scenario = scenario(&suite, SET_LEAD);
    let payload = event_payload(scenario, "demo.dialer.LeadSet");
    for key in ["lead.id", "lead.place.city"] {
        assert!(payload.contains_key(key), "{key} in {payload:?}");
    }
    assert!(!payload.contains_key("lead.place.code"), "{payload:?}");
    assert!(!payload.contains_key("lead.place"), "{payload:?}");
    let row = row_keys(scenario);
    for key in ["lead.id", "lead.place.city"] {
        assert!(row.iter().any(|k| k == key), "{key} in {row:?}");
    }
}

/// The #179 shape on a transition with two sources, so the second source is run by
/// `other_sources`/`from_source` — the one path the unit's own suite (a single-state lifecycle)
/// never reaches. Every row asserted after a `SetLead` must still carry `lead.number`.
fn two_sources() -> String {
    format!(
        "{HEAD}  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {{name: id, type: String}}
      - {{name: number, type: String}}
      - {{name: rank, type: Integer}}
entities:
  - name: demo.dialer.Membership
    identity: {{name: agent_id, type: demo.dialer.AgentId}}
    fields:
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
    lifecycle:
      initial: Idle
      states: [Idle, Paused, Assigned]
      terminal: [Assigned]
      transitions:
        - {{name: pause, from: [Idle], to: Paused}}
        - {{name: assign, from: [Idle, Paused], to: Assigned}}
events:
  - name: demo.dialer.Joined
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
  - name: demo.dialer.Paused
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
  - name: demo.dialer.LeadSet
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
actors:
  - {{name: demo.dialer.Agent, may: [demo.dialer.Join, demo.dialer.Pause, demo.dialer.SetLead]}}
commands:
  - name: demo.dialer.Join
    outcomes:
      - name: joined
        creates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.Joined]
        payload:
          demo.dialer.Joined: {{agent_id: {{generated: true}}}}
  - name: demo.dialer.Pause
    input:
      - {{name: agent_id, type: demo.dialer.AgentId}}
    outcomes:
      - name: paused
        moves: demo.dialer.Membership.pause
        instance: agent_id
        emits: [demo.dialer.Paused]
        payload:
          demo.dialer.Paused: {{agent_id: input.agent_id}}
  - name: demo.dialer.SetLead
    input:
      - {{name: agent_id, type: demo.dialer.AgentId}}
      - {{name: lead_id, type: String}}
      - {{name: lead_number, type: String}}
    outcomes:
      - name: lead-set
        moves: demo.dialer.Membership.assign
        instance: agent_id
        emits: [demo.dialer.LeadSet]
        payload:
          demo.dialer.LeadSet: {{agent_id: input.agent_id}}
        sets:
          lead:
            id: input.lead_id
            number: input.lead_number
            rank: {{generated: true}}
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {{name: agent_id, type: demo.dialer.AgentId}}
      - {{name: lead, type: Optional<demo.dialer.Lead>}}
"
    )
}

const ASSIGN: &str = "demo.dialer.Membership/transition/assign/by/demo.dialer.SetLead/lead-set";

/// The rows each `SetLead` execution of the transition scenario is followed by, in order.
fn rows_per_set_lead(
    scenario: &ConformanceScenario,
) -> Vec<Vec<BTreeMap<String, ess_conformance::ScenarioValue>>> {
    let mut out: Vec<Vec<_>> = Vec::new();
    let mut after_set_lead = false;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, .. } => {
                after_set_lead = command.to_string() == "demo.dialer.SetLead";
                if after_set_lead {
                    out.push(Vec::new());
                }
            }
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } if after_set_lead => out.last_mut().expect("pushed").push(fields.clone()),
            _ => {}
        }
    }
    out
}

#[test]
fn the_second_source_of_a_transition_asserts_lead_number_on_its_row() {
    // Refusals for undeclared (state, command) pairs are unrelated to #179 and tolerated here.
    let suite = ess_conformance::synthesize::synthesize(&ir(&two_sources())).suite;
    let rows = rows_per_set_lead(scenario(&suite, ASSIGN));
    assert_eq!(
        rows.len(),
        2,
        "SetLead runs from Idle and from Paused: {rows:#?}"
    );
    for (source, rows) in ["Idle", "Paused"].iter().zip(&rows) {
        assert!(
            !rows.is_empty(),
            "a row is asserted after SetLead from {source}"
        );
        for row in rows {
            assert!(
                row.contains_key("lead.number") && row.contains_key("lead.id"),
                "from {source}: the row omits a determined leaf: {row:?}"
            );
        }
    }
}
