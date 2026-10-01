//! The served contract declares one standard refusal for an actor no grant admits
//! (beyond10x/ess#265): `403` `{refused: "not granted", actor}` on every command, beside whatever
//! else the command answers. How a request proves its actor is the realization's, so the contract
//! declares no parameter for it.
//!
//! One status for the refusal a caller decides and the refusal no grant admits: both say *this
//! sender may not do this*. Where a command already answers `403` with a declared branch, the
//! response is `oneOf` that branch and the standard refusal.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const NOTES: &str = "format: ess/16
system: demo
version: v1
summary: Two actors, one granted less than the other.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.Author
    attributes:
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
  - name: demo.notes.Reader
    attributes:
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {note_id: input.note_id}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
  - name: demo.notes.NoteEdited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
components:
  - component: notes-service
    owns:
      domains: [demo.notes]
    accepts:
      commands: [demo.notes.CreateNote, demo.notes.EditNote]
    publishes:
      events: [demo.notes.NoteCreated, demo.notes.NoteEdited]
    reached_by: network
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn contract(ir: &EssIr) -> Value {
    let component = ir.components().values().next().expect("one component");
    serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("the contract is JSON")
}

fn operation<'a>(contract: &'a Value, path: &str) -> &'a Value {
    &contract["paths"][path]["post"]
}

fn not_granted() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["refused", "actor"],
        "properties": {
            "refused": {
                "type": "string",
                "enum": ["not granted"],
                "description": "The standard refusal for an actor no grant admits.",
            },
            "actor": {
                "type": ["string", "null"],
                "description": "The qualified name of the actor the request was authenticated \
                                as, or null where it was authenticated as none.",
            },
        },
    })
}

/// The contract states which actors may invoke a command and never how a request proves it is
/// one: no parameter names the actor, and the description says the realization authenticates.
#[test]
fn no_request_parameter_names_the_actor_and_the_description_tells_the_two_403s_apart() {
    let contract = contract(&compiled(NOTES));
    let paths = contract["paths"].as_object().expect("paths");
    assert_eq!(paths.len(), 2, "{paths:?}");
    for (path, item) in paths {
        assert!(
            item["post"].get("parameters").is_none(),
            "{path} declares no parameter: {item}"
        );
    }
    let description = contract["info"]["description"].as_str().expect("described");
    assert!(
        description.contains("`{\"refused\": \"not granted\", \"actor\": <name or null>}`"),
        "{description}"
    );
    assert!(
        description.contains("carries `outcome` and the declared `error` instead"),
        "{description}"
    );
    assert!(!description.contains("header"), "{description}");
}

#[test]
fn a_command_no_branch_of_which_the_caller_decides_answers_403_with_the_standard_refusal_alone() {
    let contract = contract(&compiled(NOTES));
    let create = operation(&contract, "/notes/commands/CreateNote");
    assert_eq!(
        create["responses"]["403"]["content"]["application/json"]["schema"],
        not_granted()
    );
}

#[test]
fn a_command_whose_caller_decides_a_branch_answers_403_with_that_branch_or_the_standard_refusal() {
    let contract = contract(&compiled(NOTES));
    let edit = operation(&contract, "/notes/commands/EditNote");
    let schema = &edit["responses"]["403"]["content"]["application/json"]["schema"];
    let one_of = schema["oneOf"].as_array().expect("a oneOf");
    assert_eq!(one_of.len(), 2, "{schema}");
    assert_eq!(
        one_of[0]["$ref"]
            .as_str()
            .expect("the declared branch first"),
        "#/components/schemas/demo.notes.EditNote.forbidden.Response"
    );
    assert_eq!(one_of[1], not_granted());
    let description = edit["responses"]["403"]["description"].as_str().unwrap();
    assert!(description.contains("`forbidden`"), "{description}");
    assert!(description.contains("standard refusal"), "{description}");
}

#[test]
fn every_command_declares_the_same_refusal() {
    let contract = contract(&compiled(NOTES));
    let refusals: Vec<Value> = contract["paths"]
        .as_object()
        .unwrap()
        .values()
        .map(|item| {
            let schema = &item["post"]["responses"]["403"]["content"]["application/json"]["schema"];
            match schema.get("oneOf") {
                Some(one_of) => one_of.as_array().unwrap().last().unwrap().clone(),
                None => schema.clone(),
            }
        })
        .collect();
    assert_eq!(refusals, vec![not_granted(), not_granted()]);
}

#[test]
fn a_model_without_actors_declares_no_standard_refusal() {
    let lone = NOTES
        .replace(
            "actors:
  - name: demo.notes.Author
    attributes:
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
  - name: demo.notes.Reader
    attributes:
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote]
",
            "",
        )
        .replace(
            "sets: {agent_id: {caller: agent_id}, text: input.text}",
            "sets: {text: input.text}",
        )
        .replace(
            "      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
",
            "",
        )
        .replace(
            "errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
",
            "",
        );
    let ir = compiled(&lone);
    assert!(ir.actors().is_empty());
    let contract = contract(&ir);
    for (path, item) in contract["paths"].as_object().unwrap() {
        assert!(item["post"]["responses"].get("403").is_none(), "{path}");
    }
    let description = contract["info"]["description"].as_str().expect("described");
    assert!(!description.contains("not granted"), "{description}");
}

/// A component that is not served has no surface to check a grant on: enforcement is the caller's,
/// so its contract declares no standard refusal, though the model declares actors.
#[test]
fn a_component_that_is_not_served_declares_no_standard_refusal() {
    let ir = compiled(&NOTES.replace("    reached_by: network\n", ""));
    assert!(!ir.actors().is_empty());
    let contract = contract(&ir);
    let edit = operation(&contract, "/notes/commands/EditNote");
    let forbidden = &edit["responses"]["403"]["content"]["application/json"]["schema"];
    assert!(
        forbidden.get("oneOf").is_none() && forbidden.get("$ref").is_some(),
        "only the declared caller-decided branch answers 403: {forbidden}"
    );
    let create = operation(&contract, "/notes/commands/CreateNote");
    assert!(create["responses"].get("403").is_none(), "{create}");
    let description = contract["info"]["description"].as_str().expect("described");
    assert!(!description.contains("not granted"), "{description}");
}

/// The contract says views are not grant-checked.
#[test]
fn the_description_says_views_are_not_grant_checked() {
    let contract = contract(&compiled(NOTES));
    let description = contract["info"]["description"].as_str().expect("described");
    assert!(
        description.contains("Views are not grant-checked"),
        "{description}"
    );
}
