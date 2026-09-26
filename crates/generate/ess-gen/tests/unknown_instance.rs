//! Which commands a seam must give a second `wrong_state` spelling, and what the contract says.
//!
//! `docs/design/unknown-instance-seams.md`. [`unknown_instance_answer`] is the one question every
//! projection of a command surface asks, so its three conditions are pinned here one by one — each
//! case below differs from the one that answers `Some` in exactly one of them — and the `OpenAPI`
//! document is held to the answer: `payload` is optional on the `wrong-state` response exactly where
//! the unknown-instance spelling exists, and required everywhere else, as before.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::unknown_instance::unknown_instance_answer;

/// A door service: `Close` moves an input-named door and declares `wrong-state` with an error
/// carrying the state; `Install` only creates; `Paint` updates and declares no `wrong_state`.
const DOORS: &str = r"
format: ess/4
system: doors
version: v1
domain: doors.core
entities:
  - name: doors.core.Door
    identity: {name: door_id, type: Uuid}
    fields:
      - {name: colour, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: doors.core.Installed
    fields: [{name: door_id, type: Uuid}]
  - name: doors.core.Closed
    fields: [{name: door_id, type: Uuid}]
  - name: doors.core.Painted
    fields: [{name: door_id, type: Uuid}]
errors:
  - name: doors.core.DoorStateConflict
    fields:
      - {name: state, type: doors.core.Door.State}
commands:
  - name: doors.core.Install
    input:
      - {name: door_id, type: Uuid}
    outcomes:
      - name: installed
        creates: doors.core.Door
        instance: door_id
        sets: {colour: white}
        emits: [doors.core.Installed]
        payload:
          doors.core.Installed: {door_id: input.door_id}
  - name: doors.core.Close
    input:
      - {name: door_id, type: Uuid}
    outcomes:
      - name: closed
        moves: doors.core.Door.close
        instance: door_id
        emits: [doors.core.Closed]
        payload:
          doors.core.Closed: {door_id: input.door_id}
      - name: wrong-state
        wrong_state: true
        error: doors.core.DoorStateConflict
  - name: doors.core.Paint
    input:
      - {name: door_id, type: Uuid}
      - {name: colour, type: String}
    outcomes:
      - name: painted
        updates: doors.core.Door
        instance: door_id
        sets: {colour: input.colour}
        emits: [doors.core.Painted]
        payload:
          doors.core.Painted: {door_id: input.door_id}
components:
  - component: door-service
    owns: {domains: [doors.core]}
    accepts: {commands: [doors.core.Install, doors.core.Close, doors.core.Paint]}
    publishes: {events: [doors.core.Installed, doors.core.Closed, doors.core.Painted]}
    reached_by: network
";

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("doors.yaml"),
        RawSpecFile::parse(source).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("doors.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("{errors}"))
}

/// The answer for one command, by its last name segment.
fn answer(ir: &EssIr, command: &str) -> Option<String> {
    let command = ir
        .commands()
        .values()
        .find(|declared| declared.name.to_string() == format!("doors.core.{command}"))
        .unwrap_or_else(|| panic!("the fixture declares `{command}`"));
    unknown_instance_answer(ir, command).map(|outcome| outcome.name.to_string())
}

#[test]
fn a_move_on_a_named_instance_with_a_describing_wrong_state_error_gets_the_second_spelling() {
    assert_eq!(answer(&ir(DOORS), "Close").as_deref(), Some("wrong-state"));
}

#[test]
fn a_command_that_only_creates_cannot_meet_an_unknown_instance() {
    assert_eq!(answer(&ir(DOORS), "Install"), None);
}

#[test]
fn a_command_declaring_no_wrong_state_has_no_answer_to_spell() {
    assert_eq!(answer(&ir(DOORS), "Paint"), None);
}

#[test]
fn a_wrong_state_error_without_fields_already_says_it_in_the_declared_spelling() {
    let source = DOORS.replace(
        "    fields:\n      - {name: state, type: doors.core.Door.State}\n",
        "",
    );
    assert_eq!(answer(&ir(&source), "Close"), None);
}

/// The `wrong-state` response schema of `Close` in the served contract.
fn close_wrong_state(ir: &EssIr) -> serde_json::Value {
    let component = ir
        .components()
        .values()
        .next()
        .expect("the fixture declares one component");
    let document: serde_json::Value =
        serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("the document is JSON");
    document["components"]["schemas"]["doors.core.Close.wrong-state.Response"].clone()
}

#[test]
fn the_contract_makes_payload_optional_exactly_where_the_second_spelling_exists() {
    let with = close_wrong_state(&ir(DOORS));
    assert_eq!(
        with["required"],
        serde_json::json!(["outcome", "error"]),
        "an unknown door is answered without a payload, so the contract may not require one"
    );
    assert!(
        with["properties"]["payload"].is_object(),
        "a door that exists still carries its payload"
    );
    assert!(with["description"].as_str().is_some_and(
        |text| text.contains("For an instance no record carries, `payload` is absent")
    ));
}

#[test]
fn a_describing_error_on_a_command_that_cannot_meet_an_unknown_instance_keeps_payload_required() {
    // The same describing error, reported by a command that names no instance: the declared
    // spelling is the only one, and its payload stays required.
    let source = DOORS
        .replace(
            "components:\n",
            "  - name: doors.core.Inspect\n    input:\n      - {name: door_id, type: Uuid}\n    \
             outcomes:\n      - name: refused\n        error: doors.core.DoorStateConflict\ncomponents:\n",
        )
        .replace(
            "doors.core.Paint]}",
            "doors.core.Paint, doors.core.Inspect]}",
        );
    let ir = ir(&source);
    assert_eq!(answer(&ir, "Inspect"), None);
    let component = ir.components().values().next().expect("one component");
    let document: serde_json::Value =
        serde_json::from_str(&ess_gen::openapi::json(&ir, component)).expect("JSON");
    assert_eq!(
        document["components"]["schemas"]["doors.core.Inspect.refused.Response"]["required"],
        serde_json::json!(["outcome", "error", "payload"])
    );
}
