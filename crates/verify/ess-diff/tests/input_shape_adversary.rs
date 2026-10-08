//! Adversarial cases for `ess-diff/16` input shapes: a wire-form move the diff does not see as an
//! `input-type-changed`, the reader's refusals, and the written shape names.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, Compatibility, EssDelta, FailOn, Gate, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const BEFORE: &str = include_str!("fixtures/upcast/before.yaml");
const AFTER: &str = include_str!("fixtures/upcast/after.yaml");
const RECEIPT: &str = "command/catalog.items.Accept/input-type-changed/receipt";
const NOTE: &str = "command/catalog.items.Add/input-type-changed/note";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

/// A catalog whose `Add` command takes `note` typed `note_type`, with `extra_types` declared.
fn catalog(note_type: &str, extra_types: &str) -> EssIr {
    ir(&format!(
        r#"format: ess/23
system: catalog
version: v1
domain: catalog.items

types:
  - {{name: catalog.items.ItemId, kind: newtype, of: String}}
{extra_types}
entities:
  - name: catalog.items.Item
    identity: {{name: item_id, type: catalog.items.ItemId}}
    fields:
      - {{name: revision, type: Integer}}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

actors:
  - name: catalog.items.Clerk
    may: [catalog.items.Add]

events:
  - name: catalog.items.Added
    fields:
      - {{name: item_id, type: catalog.items.ItemId}}

commands:
  - name: catalog.items.Add
    input:
      - {{name: revision, type: Integer}}
      - {{name: note, type: "{note_type}"}}
    outcomes:
      - name: added
        creates: catalog.items.Item
        instance: item_id
        sets: {{revision: input.revision}}
        emits: [catalog.items.Added]
        payload:
          catalog.items.Added: {{item_id: {{generated: true}}}}
"#
    ))
}

fn failing(delta: &EssDelta, threshold: FailOn) -> Vec<String> {
    Gate::new(threshold)
        .judge(delta, None)
        .unwrap()
        .failing()
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn callers(delta: &EssDelta) -> Vec<(String, Compatibility)> {
    delta
        .changes()
        .iter()
        .map(|change| {
            (
                change.id().to_string(),
                delta.compatibility_of(&change.id()).unwrap().callers(),
            )
        })
        .collect()
}

/// `note: catalog.items.Receipt` keeps its type name while `Receipt` turns from a newtype of
/// `String` into a struct: a caller still sending text is refused exactly as in the reproducer.
#[test]
fn an_input_type_redefined_from_text_to_record_under_one_name_is_breaking_for_callers() {
    let before = catalog(
        "catalog.items.Receipt",
        "  - {name: catalog.items.Receipt, kind: newtype, of: String}\n",
    );
    let after = catalog(
        "catalog.items.Receipt",
        "  - name: catalog.items.Receipt\n    kind: struct\n    fields:\n      - {name: text, type: String}\n",
    );
    let delta = classified(&before, &after).unwrap();
    assert!(
        !failing(&delta, FailOn::Breaking).is_empty(),
        "the input's wire form moved from a value to an object; callers: {:?}\n{}",
        callers(&delta),
        delta.to_canonical_json()
    );
}

/// A member of a record input moves from text to an array: every caller sending the old member
/// is refused by the same decoder.
#[test]
fn a_member_of_a_record_input_turned_from_text_to_list_is_breaking_for_callers() {
    let note = |member: &str| {
        format!(
            "  - name: catalog.items.Note\n    kind: struct\n    fields:\n      - {{name: text, type: \"{member}\"}}\n"
        )
    };
    let delta = classified(
        &catalog("catalog.items.Note", &note("String")),
        &catalog("catalog.items.Note", &note("List<String>")),
    )
    .unwrap();
    assert!(
        !failing(&delta, FailOn::Breaking).is_empty(),
        "a nested input member's wire form moved; callers: {:?}\n{}",
        callers(&delta),
        delta.to_canonical_json()
    );
}

/// The written shape names each container, so `record` and `map` (one wire form) stay apart.
#[test]
fn the_written_shapes_name_each_side_literally() {
    let types = "  - name: catalog.items.Note\n    kind: struct\n    fields:\n      - {name: text, type: String}\n";
    for (from, to, before, after) in [
        ("Integer", "Map<String, Integer>", "scalar", "map"),
        ("List<String>", "Map<String, String>", "list", "map"),
        ("List<String>", "catalog.items.Note", "list", "record"),
        ("Optional<String>", "List<String>", "scalar", "list"),
    ] {
        let delta = classified(&catalog(from, types), &catalog(to, types)).unwrap();
        let document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
        let change = document["changes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|change| change["id"] == NOTE)
            .unwrap();
        assert_eq!(
            change["compatibility"]["shapes"],
            serde_json::json!({"before": before, "after": after}),
            "{from} → {to}"
        );
    }
}

fn reread(edit: impl FnOnce(&mut Value)) -> Result<EssDelta, String> {
    let delta = classified(&ir(BEFORE), &ir(AFTER)).unwrap();
    let mut document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    edit(&mut document);
    let raw: RawEssDelta = serde_json::from_value(document).map_err(|error| error.to_string())?;
    EssDelta::try_from(raw).map_err(|errors| errors.to_string())
}

#[test]
fn a_reader_refuses_shapes_on_a_change_that_is_not_an_input_type_change() {
    let refused = reread(|document| {
        let other = document["changes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|change| change["id"] != RECEIPT)
            .expect("another change");
        other["compatibility"]["shapes"] =
            serde_json::json!({"before": "scalar", "after": "record"});
    })
    .expect_err("only an input type change records shapes");
    assert!(refused.contains("conflicting_declaration"), "{refused}");
}

#[test]
fn a_reader_refuses_an_unknown_member_of_shapes() {
    let refused = reread(|document| {
        let receipt = document["changes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|change| change["id"] == RECEIPT)
            .unwrap();
        receipt["compatibility"]["shapes"]["via"] = Value::from("newtype");
    })
    .expect_err("shapes refuses unknown members");
    assert!(refused.contains("via"), "{refused}");
}
