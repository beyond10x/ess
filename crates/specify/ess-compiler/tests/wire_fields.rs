//! Every projection shares the admitted specification's wire-key uniqueness invariant.

use ess_compiler::{compile, resolve::diagnose_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SOURCE: &str = r"
format: ess/1
system: sample
version: v1
domains: [sample.data]
domain: sample.data
types:
  - name: sample.data.Row
    kind: struct
    fields:
      - {name: a, type: String, wire: row_a}
      - {name: b, type: String, wire: row_b}
entities:
  - name: sample.data.Item
    identity: {name: id, type: String, wire: item_id}
    fields:
      - {name: a, type: String, wire: item_a}
      - {name: b, type: String, wire: item_b}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
commands:
  - name: sample.data.Update
    input:
      - {name: a, type: String, wire: input_a}
      - {name: b, type: String, wire: input_b}
    outcomes:
      - name: changed
        emits: [sample.data.Changed]
        payload:
          sample.data.Changed: {a: input.a, b: input.b}
events:
  - name: sample.data.Changed
    fields:
      - {name: a, type: String, wire: event_a}
      - {name: b, type: String, wire: event_b}
errors:
  - name: sample.data.Rejected
    fields:
      - {name: a, type: String, wire: error_a}
      - {name: b, type: String, wire: error_b}
views:
  - name: sample.data.Inline
    source: sample.data.Item
    fields:
      - {name: a, type: String, wire: view_a}
      - {name: b, type: String, wire: view_b}
    params:
      - {name: left, type: String, wire: param_a}
      - {name: right, type: String, wire: param_b}
    filter:
      all: [a == param.left, b == param.right]
  - name: sample.data.Shaped
    source: sample.data.Item
    shape: sample.data.Row
";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    Specification::assemble([(
        Source::new("fields.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
}

#[test]
fn every_object_namespace_refuses_explicit_and_default_key_collisions() {
    assemble(SOURCE).unwrap();
    for (first, second, location, count) in [
        ("row_a", "row_b", "types.sample.data.Row.fields[1]", 2),
        ("item_a", "item_b", "entity sample.data.Item.fields[1]", 1),
        (
            "input_a",
            "input_b",
            "command.sample.data.Update.input[1]",
            1,
        ),
        (
            "event_a",
            "event_b",
            "event.sample.data.Changed.fields[1]",
            1,
        ),
        (
            "error_a",
            "error_b",
            "error.sample.data.Rejected.fields[1]",
            1,
        ),
        ("view_a", "view_b", "view.sample.data.Inline.fields[1]", 1),
        ("param_a", "param_b", "view.sample.data.Inline.params[1]", 1),
    ] {
        for default_first in [false, true] {
            let mut text = SOURCE.replace(second, if default_first { "a" } else { first });
            if default_first {
                text = text.replace(&format!(", wire: {first}"), "");
                // Parameter names differ from row names, so their fallback is `left`.
                if first == "param_a" {
                    text = text.replace("wire: a}", "wire: left}");
                }
            }
            let errors = assemble(&text).unwrap_err();
            assert_eq!(errors.len(), count, "{text}\n{errors}");
            assert!(errors
                .as_slice()
                .iter()
                .all(|error| error.code == ValidationCode::DuplicateDeclaration));
            assert!(errors
                .as_slice()
                .iter()
                .any(|error| error.location == location));
        }
    }
}

#[test]
fn entity_identity_and_synthetic_state_share_the_field_namespace() {
    for (from, to, location) in [
        ("item_a", "item_id", "entity sample.data.Item.fields[0]"),
        ("item_a", "state", "entity sample.data.Item.fields[0]"),
        ("item_id", "state", "entity sample.data.Item.identity"),
    ] {
        let errors = assemble(&SOURCE.replace(from, to)).unwrap_err();
        assert_eq!(errors.len(), 1, "{errors}");
        assert_eq!(errors.as_slice()[0].location, location);
        assert!(errors.as_slice()[0].message.contains(to));
    }
}

#[test]
fn separate_namespaces_special_keys_and_display_names_do_not_collide() {
    let text = SOURCE
        .replace("wire: param_a", "wire: view_a")
        .replace("wire: param_b", "wire: view_b")
        .replace("wire: row_a", "wire: '', display: Same")
        .replace("wire: row_b", "wire: '/~ name', display: Same")
        .replace("wire: item_a", "wire: Key")
        .replace("wire: item_b", "wire: key");
    let spec = assemble(&text).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("fields.yaml", text);
    let first = compile(&spec, &sources).unwrap().to_canonical_json();
    assert_eq!(first, compile(&spec, &sources).unwrap().to_canonical_json());
}

#[test]
fn collisions_accumulate_with_semantic_source_locations_before_ir_exists() {
    let text = SOURCE
        .replace("row_b", "row_a")
        .replace("item_b", "item_a")
        .replace("input_b", "input_a")
        .replace("event_b", "event_a")
        .replace("error_b", "error_a")
        .replace("view_b", "view_a")
        .replace("param_b", "param_a");
    let errors = assemble(&text).unwrap_err();
    assert_eq!(errors.len(), 8, "{errors}");
    let mut sources = SourceMap::new();
    sources.insert("fields.yaml", text);
    let diagnostics = diagnose_locating(&errors, &sources, &["fields.yaml"]);
    assert_eq!(diagnostics.len(), 8);
    for (error, diagnostic) in errors.as_slice().iter().zip(diagnostics.as_slice()) {
        let span = diagnostic.span.as_ref().unwrap();
        assert_eq!(span.source, "fields.yaml", "{diagnostic:?}");
        assert_eq!(span.path, error.location);
        assert!(diagnostic.message.contains("already used at"));
    }
}
