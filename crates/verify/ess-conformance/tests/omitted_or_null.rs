//! `presence: omitted_or_null` (beyond10x/ess#678, the wire half): a member that may be left out or
//! sent as `null`, both read as one absent value. Projected as the nullable, not required property;
//! never carried into a suite, so a runner admits both spellings as it does for a field without a
//! policy.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::direct_response::Observation;
use ess_conformance::{bind, Completeness, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use ess_primitives::node::Node;
use serde_json::json;
use std::collections::BTreeMap;

const MODEL: &str = "format: ess/31
system: demo
version: v1
domain: demo.notes
types:
  - name: demo.notes.Edit
    kind: struct
    fields:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
commands:
  - name: demo.notes.Revise
    input:
      - {name: edit, type: demo.notes.Edit}
    response:
      - {name: edit, type: demo.notes.Edit}
    outcomes:
      - name: revised
        returns: true
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn schema(model: &EssIr, path: &str) -> serde_json::Value {
    let artifacts = run(&JsonSchema, model).expect("generates");
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

fn edit(reason: Option<Node>) -> BTreeMap<String, Node> {
    let mut edit = BTreeMap::new();
    if let Some(reason) = reason {
        edit.insert("reason".to_owned(), reason);
    }
    BTreeMap::from([("edit".to_owned(), Node::Map(edit))])
}

#[test]
fn a_struct_field_admits_missing_and_null() {
    let model = ir(MODEL);
    for path in [
        "schema/commands/demo.notes.Revise.schema.json",
        "schema/responses/demo.notes.Revise.schema.json",
    ] {
        let schema = schema(&model, path);
        let declared = &schema["$defs"]["demo.notes.Edit"];
        assert_eq!(
            declared["properties"]["reason"],
            json!({"anyOf": [{"type": "string"}, {"type": "null"}]}),
            "{path}: {schema:#}"
        );
        assert!(
            declared["required"]
                .as_array()
                .is_none_or(|required| !required.contains(&json!("reason"))),
            "{path}: {schema:#}"
        );
        let validator = jsonschema::validator_for(&schema).expect("a valid schema");
        for member in [json!({}), json!({"reason": null}), json!({"reason": "x"})] {
            let instance = json!({ "edit": member });
            assert!(
                validator.is_valid(&instance),
                "{path}: {instance} against {schema:#}"
            );
        }
    }

    let command = model
        .commands()
        .get(&"demo.notes.Revise".parse().unwrap())
        .expect("the command");
    let observation =
        Observation::of(&model, command, None, BTreeMap::new()).expect("an observation");
    let observed = serde_json::to_string(&observation).expect("serialises");
    assert!(!observed.contains("presence"), "{observed}");
    for (which, payload) in [
        ("missing", edit(None)),
        ("null", edit(Some(Node::Null))),
        ("a value", edit(Some(Node::Text("x".to_owned())))),
    ] {
        bind(&model, &command.input, &payload, Completeness::Total)
            .unwrap_or_else(|errors| panic!("the input with the member {which}: {errors:?}"));
        observation
            .compare(Some(&payload))
            .unwrap_or_else(|error| panic!("the response with the member {which}: {error}"));
    }
}

/// A synthesized suite never carries the value: its leaves read as a field without a policy, and
/// the suite stays off the presence formats.
#[test]
fn a_synthesized_suite_carries_no_omitted_or_null() {
    let text = "format: ess/31
system: demo
version: v1
domain: demo.notes
events:
  - name: demo.notes.Edited
    fields:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
actors:
  - {name: demo.notes.Clerk, may: [demo.notes.Edit]}
commands:
  - name: demo.notes.Edit
    input:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
    outcomes:
      - name: edited
        emits: [demo.notes.Edited]
        payload:
          demo.notes.Edited: {reason: input.reason}
";
    let result = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let leaves: Vec<_> = result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExpectEvent { shape, .. } => Some(shape),
            _ => None,
        })
        .filter_map(|shape| shape.leaves().get("reason").cloned())
        .collect();
    assert!(!leaves.is_empty(), "the suite expects the event");
    assert!(leaves.iter().all(|leaf| leaf.presence.is_none()));
    assert!(!ess_conformance::presence::used_by(&result.suite));
    let bytes = result.suite.to_canonical_json().unwrap();
    assert!(!bytes.contains("omitted_or_null"), "{bytes}");
}

/// A view carrying its source fields' policies (beyond10x/ess#677): the synthesized suite passes
/// against the interpreted target, which leaves an absent `omitted_when_absent` member out.
#[test]
fn a_view_carrying_its_source_presence_passes_its_own_suite() {
    let text = "format: ess/31
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: embed_token, type: Optional<String>, presence: null_when_absent}
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
      - {name: hint, type: Optional<String>, presence: omitted_when_absent}
      - {name: note, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.notes.Opened
    fields:
      - {name: note_id, type: demo.notes.NoteId}
actors:
  - {name: demo.notes.Clerk, may: [demo.notes.Open]}
commands:
  - name: demo.notes.Open
    input:
      - {name: embed_token, type: Optional<String>}
      - {name: reason, type: Optional<String>}
      - {name: hint, type: Optional<String>}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: opened
        creates: demo.notes.Note
        instance: note_id
        emits: [demo.notes.Opened]
        payload:
          demo.notes.Opened: {note_id: {generated: true}}
        sets:
          embed_token: input.embed_token
          reason: input.reason
          hint: input.hint
          note: input.note
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: embed_token, type: Optional<String>}
      - {name: reason, type: Optional<String>}
      - {name: hint, type: Optional<String>}
      - {name: note, type: Optional<String>}
";
    let model = ir(text);
    let view = model
        .views()
        .get(&"demo.notes.Notes".parse().unwrap())
        .expect("the view");
    assert_eq!(
        view.field("hint").expect("a view field").naming.presence,
        Some(ess_domain::types::Presence::OmittedWhenAbsent)
    );
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = ess_conformance::runner::Runner::for_suite(&synthesis.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model.clone()),
        )
        .into_report();
    assert_eq!(
        report.status,
        ess_conformance::ConformanceStatus::Passed,
        "{report:#?}"
    );
}
