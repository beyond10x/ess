//! `sets: {field: {cleared: true}}` — a branch saying the field it owns holds nothing afterwards.
//!
//! Two rules, and each is about a place rather than a value. The field must be able to be absent,
//! because a required field cannot hold nothing; and the construct is an ENTITY source, because an
//! event field the emitter does not determine is one the outcome does not list.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("cleared.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// One entity with an optional field and a required one, and one command that can clear either.
fn model(target: &str, payload: &str) -> String {
    format!(
        "format: ess/1
system: demo
version: v1
domain: demo.lanes
types:
  - name: demo.lanes.OrderId
    kind: newtype
    of: Uuid
  - name: demo.lanes.LaneId
    kind: newtype
    of: String
entities:
  - name: demo.lanes.Order
    identity:
      name: order_id
      type: demo.lanes.OrderId
    fields:
      - name: lane_id
        type: Optional<demo.lanes.LaneId>
      - name: label
        type: String
    lifecycle:
      initial: Placed
      states: [Placed, Cleared]
      terminal: [Cleared]
      transitions:
        - name: clear
          from: [Placed]
          to: Cleared
events:
  - name: demo.lanes.OrderPlaced
    fields:
      - name: order_id
        type: demo.lanes.OrderId
  - name: demo.lanes.OrderCleared
    fields:
      - name: order_id
        type: demo.lanes.OrderId
      - name: note
        type: String
commands:
  - name: demo.lanes.PlaceOrder
    outcomes:
      - name: accepted
        creates: demo.lanes.Order
        instance: order_id
        emits:
          - demo.lanes.OrderPlaced
  - name: demo.lanes.ClearLane
    input:
      - name: order_id
        type: demo.lanes.OrderId
    outcomes:
      - name: cleared
        moves: demo.lanes.Order.clear
        instance: order_id
        sets:
          {target}: {{cleared: true}}
        emits:
          - demo.lanes.OrderCleared
{payload}"
    )
}

const NOTE: &str = "        payload:
          demo.lanes.OrderCleared:
            note: {cleared: true}
";

#[test]
fn clearing_an_optional_entity_field_is_admitted_and_round_trips() {
    let parsed = spec(&model("lane_id", "")).expect("an Optional field may be cleared");
    let command = parsed
        .commands()
        .values()
        .find(|c| c.name.to_string() == "demo.lanes.ClearLane")
        .expect("the command");
    let encoded = ess_yaml::to_string(command).unwrap();
    assert!(encoded.contains("cleared: true"), "{encoded}");
    let raw: ess_domain::command::RawCommandSpec = ess_yaml::from_str(&encoded).unwrap();
    let roundtrip = ess_domain::command::CommandSpec::try_from(raw).unwrap();
    assert_eq!(&roundtrip, command, "the construct survives a round trip");
}

#[test]
fn clearing_a_field_that_cannot_be_absent_is_refused_by_name() {
    let error = spec(&model("label", "")).expect_err("a required field cannot hold nothing");
    for required in ["cleared: true", "demo.lanes.Order.label", "String"] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
    assert!(
        error.contains("Optional<"),
        "the hint names the repair:\n{error}"
    );
}

#[test]
fn clearing_an_event_payload_field_is_refused_by_name() {
    let error = spec(&model("lane_id", NOTE)).expect_err("an event field is not an entity field");
    for required in ["cleared: true", "note"] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
}

#[test]
fn only_exactly_one_explicit_source_is_admitted() {
    for bad in [
        "{cleared: false}",
        "{cleared: true, generated: true}",
        "{cleared: true, response: item}",
    ] {
        let source = model("lane_id", "").replace("{cleared: true}", bad);
        assert!(
            spec(&source).is_err(),
            "`{bad}` is not a source: it names two things, or none"
        );
    }
}

/// A row-set selector comparing an `Optional` field a row may hold absent: a root node is created
/// with no `parent`, and a selector compares `parent == input.parent` with no `defined(parent)`
/// beside it (story:a-comparison-over-an-optional-field-a-row-holds-absent-is-decided-or-named-by-validate-and-not-left-unknown-for-the-interpreter-to-refuse).
const ABSENT: &str = include_str!("fixtures/absent-comparison.yaml");

#[test]
fn from_ess_38_a_comparison_over_a_field_a_row_may_hold_absent_is_refused_naming_the_idiom() {
    let Err(refused) = spec(ABSENT) else {
        panic!("an unguarded comparison over `parent` is refused");
    };
    let line = refused
        .lines()
        .find(|line| line.contains("[missing_declaration]") && line.contains("`parent`"))
        .unwrap_or_else(|| panic!("names `parent`: {refused}"));
    assert!(line.contains("row-set selector"), "{line}");
    assert!(line.contains("tree.nodes.Node"), "{line}");
    assert!(
        line.contains("defined(parent) and parent == input.parent"),
        "{line}"
    );
}

#[test]
fn a_presence_test_beside_the_comparison_validates() {
    let guarded = ABSENT.replacen(
        "where: {all: [parent == input.parent,",
        "where: {all: [defined(parent), parent == input.parent,",
        1,
    );
    spec(&guarded).unwrap_or_else(|errors| panic!("{errors}"));
    let negated = ABSENT.replacen(
        "where: {all: [parent == input.parent, name == input.name, state == Open]}",
        "where: {all: [name == input.name, state == Open, {any: [missing(parent), parent == input.parent]}]}",
        1,
    );
    spec(&negated).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_field_no_writer_leaves_absent_validates_unchanged() {
    let start = ABSENT.find("  - name: tree.nodes.AddRoot\n").unwrap();
    let end = ABSENT.find("  - name: tree.nodes.AddChild\n").unwrap();
    let children_only = format!("{}{}", &ABSENT[..start], &ABSENT[end..]).replacen(
        "may: [tree.nodes.AddRoot, tree.nodes.AddChild]",
        "may: [tree.nodes.AddChild]",
        1,
    );
    spec(&children_only).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn below_ess_38_the_comparison_validates_as_it_did() {
    spec(&ABSENT.replacen("format: ess/38", "format: ess/37", 1))
        .unwrap_or_else(|errors| panic!("{errors}"));
}
