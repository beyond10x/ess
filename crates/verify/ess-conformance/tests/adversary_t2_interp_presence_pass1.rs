//! Adversary pass 1 on beyond10x/ess#765: the interpreted target publishes an absent event field in
//! its declared presence. Each case runs the suite synthesized from a specification against the
//! interpreted target (the acceptance's own check, in-process), or one interpreter step.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::runner::Runner;
use ess_conformance::AdmittedSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn ir_of(body: &str) -> EssIr {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let spec = Specification::assemble([(Source::new("token.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{body}"))
}

fn run_interpreted(body: &str) -> Vec<ScenarioResult> {
    let ir = ir_of(body);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
}

fn assert_all_passed(body: &str, expected: Option<usize>) {
    let results = run_interpreted(body);
    let bad: Vec<&ScenarioResult> = results
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(
        bad.is_empty(),
        "{} of {} scenarios not passed:\n{bad:#?}",
        bad.len(),
        results.len()
    );
    if let Some(expected) = expected {
        assert_eq!(results.len(), expected, "scenario count");
    }
}

// ---- the CLI reproduction of the story's Need -----------------------------------------------

fn input_model(presence: &str) -> String {
    format!(
        "format: ess/25
system: example
version: v1
domain: example.token
types:
  - {{name: example.token.TokenId, kind: newtype, of: String}}
entities:
  - name: example.token.PoolToken
    identity: {{name: token_id, type: example.token.TokenId}}
    fields:
      - {{name: owner, type: String}}
    lifecycle:
      initial: Contributed
      states: [Contributed]
      terminal: [Contributed]
events:
  - name: example.token.TokenContributed
    fields:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: note, type: 'Optional<String>'{presence}}}
commands:
  - name: example.token.ContributeToken
    input:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: owner, type: String}}
      - {{name: note, type: 'Optional<String>'}}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {{owner: input.owner}}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed: {{token_id: input.token_id, note: input.note}}
"
    )
}

#[test]
fn adv_input_repro_null_when_absent_passes() {
    assert_all_passed(&input_model(", presence: null_when_absent"), Some(1));
}

#[test]
fn adv_input_repro_omitted_and_no_presence_pass() {
    assert_all_passed(&input_model(", presence: omitted_when_absent"), None);
    assert_all_passed(&input_model(""), None);
}

// ---- the row-read reproduction (TokenWithdrawn.agent: {subject: agent}) -----------------------

fn row_model(agent_presence: &str) -> String {
    format!(
        "format: ess/25
system: example
version: v1
domain: example.token
types:
  - {{name: example.token.TokenId, kind: newtype, of: String}}
  - {{name: example.token.AgentName, kind: newtype, of: String}}
entities:
  - name: example.token.PoolToken
    identity: {{name: token_id, type: example.token.TokenId}}
    fields:
      - {{name: owner, type: String}}
      - {{name: agent, type: 'Optional<example.token.AgentName>'}}
    lifecycle:
      initial: Contributed
      states: [Contributed, Withdrawn]
      terminal: [Withdrawn]
      transitions:
        - {{name: withdraw, from: [Contributed], to: Withdrawn}}
events:
  - name: example.token.TokenContributed
    fields:
      - {{name: token_id, type: example.token.TokenId}}
  - name: example.token.TokenWithdrawn
    fields:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: owner, type: String}}
      - {{name: agent, type: 'Optional<example.token.AgentName>'{agent_presence}}}
commands:
  - name: example.token.ContributeToken
    input:
      - {{name: owner, type: String}}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {{owner: input.owner, agent: {{cleared: true}}}}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed:
            token_id: {{generated: true}}
  - name: example.token.WithdrawToken
    input:
      - {{name: token_id, type: example.token.TokenId}}
    outcomes:
      - name: withdrawn
        moves: example.token.PoolToken.withdraw
        instance: token_id
        sets: {{agent: {{cleared: true}}}}
        emits: [example.token.TokenWithdrawn]
        payload:
          example.token.TokenWithdrawn:
            token_id: input.token_id
            owner: {{subject: owner}}
            agent: {{subject: agent}}
"
    )
}

#[test]
fn adv_row_read_null_when_absent_passes() {
    assert_all_passed(&row_model(", presence: null_when_absent"), None);
}

#[test]
fn adv_row_read_omitted_when_absent_passes() {
    assert_all_passed(&row_model(", presence: omitted_when_absent"), None);
}

#[test]
fn adv_row_read_no_presence_passes() {
    assert_all_passed(&row_model(""), None);
}

// ---- a struct payload whose member declares the policy ----------------------------------------

fn struct_model(presence: &str) -> String {
    format!(
        "format: ess/25
system: example
version: v1
domain: example.token
types:
  - {{name: example.token.TokenId, kind: newtype, of: String}}
  - name: example.token.Detail
    kind: struct
    fields:
      - {{name: kind, type: String}}
      - {{name: note, type: 'Optional<String>'{presence}}}
entities:
  - name: example.token.PoolToken
    identity: {{name: token_id, type: example.token.TokenId}}
    fields:
      - {{name: owner, type: String}}
    lifecycle:
      initial: Contributed
      states: [Contributed]
      terminal: [Contributed]
events:
  - name: example.token.TokenContributed
    fields:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: detail, type: example.token.Detail}}
commands:
  - name: example.token.ContributeToken
    input:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: owner, type: String}}
      - {{name: kind, type: String}}
      - {{name: note, type: 'Optional<String>'}}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {{owner: input.owner}}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed:
            token_id: input.token_id
            detail: {{kind: input.kind, note: input.note}}
"
    )
}

#[test]
fn adv_struct_member_null_when_absent_passes() {
    assert_all_passed(&struct_model(", presence: null_when_absent"), None);
}

#[test]
#[ignore = "backlog: story:an-interpreted-struct-member-follows-its-declared-presence"]
fn adv_struct_member_null_when_absent_step_spells_null() {
    let ir = ir_of(&struct_model(", presence: null_when_absent"));
    let input = BTreeMap::from([
        ("token_id".to_owned(), Node::Text("token-1".to_owned())),
        ("owner".to_owned(), Node::Text("owner-1".to_owned())),
        ("kind".to_owned(), Node::Text("k".to_owned())),
    ]);
    let detail = contributed(&ir, &input, "detail").expect("detail is published");
    let Node::Map(members) = detail else {
        panic!("detail is a struct: {detail:?}")
    };
    assert_eq!(
        members.get("note"),
        Some(&Node::Null),
        "a struct member declaring null_when_absent is published as null: {members:?}"
    );
}

// ---- an input sent present-but-null ------------------------------------------------------------

fn contributed(ir: &EssIr, input: &BTreeMap<String, Node>, field: &str) -> Option<Node> {
    let command = ess_domain::name::QualifiedName::new("example.token.ContributeToken").unwrap();
    let mut steps = execute(ir, &Store::default(), &command, input, &Externals::Withheld)
        .unwrap_or_else(|error| panic!("interpreted: {error}"));
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    let event = step
        .events
        .iter()
        .find(|published| published.event.to_string() == "example.token.TokenContributed")
        .unwrap_or_else(|| panic!("published: {:?}", step.events));
    event.payload.get(field).cloned()
}

fn with_null_note() -> BTreeMap<String, Node> {
    BTreeMap::from([
        ("token_id".to_owned(), Node::Text("token-1".to_owned())),
        ("owner".to_owned(), Node::Text("owner-1".to_owned())),
        ("note".to_owned(), Node::Null),
    ])
}

#[test]
fn adv_null_input_follows_the_declared_presence() {
    let null = ir_of(&input_model(", presence: null_when_absent"));
    assert_eq!(
        contributed(&null, &with_null_note(), "note"),
        Some(Node::Null)
    );
    let omitted = ir_of(&input_model(", presence: omitted_when_absent"));
    assert_eq!(contributed(&omitted, &with_null_note(), "note"), None);
    let none = ir_of(&input_model(""));
    assert_eq!(contributed(&none, &with_null_note(), "note"), None);
}

// ---- an event a binding-invoked command publishes ---------------------------------------------

fn binding_model(presence: &str) -> String {
    "format: ess/25
system: example
version: v1
domain: example.token
types:
  - {name: example.token.TokenId, kind: newtype, of: String}
entities:
  - name: example.token.PoolToken
    identity: {name: token_id, type: example.token.TokenId}
    fields:
      - {name: owner, type: String}
    lifecycle:
      initial: Contributed
      states: [Contributed, Noted]
      terminal: [Noted]
      transitions:
        - {name: note, from: [Contributed], to: Noted}
events:
  - name: example.token.TokenContributed
    fields:
      - {name: token_id, type: example.token.TokenId}
      - {name: note, type: 'Optional<String>'PRESENCE}
  - name: example.token.NoteRecorded
    fields:
      - {name: token_id, type: example.token.TokenId}
      - {name: note, type: 'Optional<String>', presence: omitted_when_absent}
commands:
  - name: example.token.ContributeToken
    input:
      - {name: token_id, type: example.token.TokenId}
      - {name: owner, type: String}
      - {name: note, type: 'Optional<String>'}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {owner: input.owner}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed: {token_id: input.token_id, note: input.note}
  - name: example.token.RecordNote
    input:
      - {name: token_id, type: example.token.TokenId}
      - {name: note, type: 'Optional<String>'}
    outcomes:
      - name: noted
        moves: example.token.PoolToken.note
        instance: token_id
        emits: [example.token.NoteRecorded]
        payload:
          example.token.NoteRecorded: {token_id: input.token_id, note: input.note}
bindings:
  - id: record-on-contributed
    when: {event: example.token.TokenContributed}
    invoke: {command: example.token.RecordNote}
    mapping: {token_id: event.token_id, note: event.note}
    delivery: at_least_once
    on_failure: retry
"
    .replace("PRESENCE", presence)
}

const MAPPING: &str = "record-on-contributed/binding/mapping";

fn mapping_status(presence: &str) -> ScenarioResult {
    run_interpreted(&binding_model(presence))
        .into_iter()
        .find(|result| result.scenario.to_string() == MAPPING)
        .unwrap_or_else(|| panic!("no scenario {MAPPING}"))
}

#[test]
fn adv_binding_mapping_of_a_null_when_absent_field_passes() {
    let result = mapping_status(", presence: null_when_absent");
    assert_eq!(result.status, Status::Passed, "{result:#?}");
}

/// A binding mapping an `Optional` event field the trigger leaves out (no presence declared, the
/// default) reports an error, not a pass: the runner reads the mapped field from the published event.
#[test]
#[ignore = "backlog: story:a-binding-mapping-of-an-absent-optional-event-field-passes"]
fn adv_binding_mapping_of_an_absent_optional_field_passes_without_presence() {
    let result = mapping_status("");
    assert_eq!(result.status, Status::Passed, "{result:#?}");
}

#[test]
#[ignore = "backlog: story:a-binding-mapping-of-an-absent-optional-event-field-passes"]
fn adv_binding_mapping_of_an_omitted_when_absent_field_passes() {
    let result = mapping_status(", presence: omitted_when_absent");
    assert_eq!(result.status, Status::Passed, "{result:#?}");
}
