//! A declared type that keeps its name while its wire form moves is breaking for the callers of
//! every command that takes it as input, exactly as an `input-type-changed` between two names is.
//!
//! The change the diff sees is a type change — `kind-changed`, `representation-changed`,
//! `field-type-changed` or `variant-type-changed` — and before this it was rated through the
//! type's uses alone: `unknown` for callers, whatever the shapes. It records the shapes beside its
//! uses (`ess-diff/16`) only where the type has an input use and the two wire forms differ; readers
//! and history keep the answer the uses give, and every other type change keeps its answer, its
//! format and its bytes.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, Compatibility, EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

use Compatibility::{Breaking as B, Compatible as C, Unknown as U};

const RECEIPT_KIND: &str = "type/catalog.items.Receipt/kind-changed";
const RECEIPT_REPRESENTATION: &str = "type/catalog.items.Receipt/representation-changed";
const NOTE_TEXT: &str = "type/catalog.items.Note/field-type-changed/text";
const TAGGED_PLAIN: &str = "type/catalog.items.Tagged/variant-type-changed/plain";
const KEPT_KIND: &str = "type/catalog.items.Kept/kind-changed";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

/// A catalog whose `Add` command takes `receipt: Receipt`, `note: Note` and `tagged: Tagged`, and
/// whose `Item` stores an optional `receipt: Receipt` and `kept: Kept`, which no command takes.
fn catalog(receipt: &str, note_text: &str, plain: &str, kept: &str) -> EssIr {
    ir(&format!(
        r#"format: ess/23
system: catalog
version: v1
domain: catalog.items

types:
  - {{name: catalog.items.ItemId, kind: newtype, of: String}}
{receipt}
  - name: catalog.items.Note
    kind: struct
    fields:
      - {{name: text, type: "{note_text}"}}
  - name: catalog.items.Tagged
    kind: union
    tag: kind
    variants:
      plain: {plain}
{kept}
entities:
  - name: catalog.items.Item
    identity: {{name: item_id, type: catalog.items.ItemId}}
    fields:
      - {{name: revision, type: Integer}}
      - {{name: receipt, type: "Optional<catalog.items.Receipt>"}}
      - {{name: kept, type: "Optional<catalog.items.Kept>"}}
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
      - {{name: receipt, type: catalog.items.Receipt}}
      - {{name: note, type: catalog.items.Note}}
      - {{name: tagged, type: catalog.items.Tagged}}
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

const TEXT_RECEIPT: &str = "  - {name: catalog.items.Receipt, kind: newtype, of: String}";
const RECORD_RECEIPT: &str = "  - name: catalog.items.Receipt\n    kind: struct\n    fields:\n      - {name: text, type: String}";
const ENUM_RECEIPT: &str =
    "  - name: catalog.items.Receipt\n    kind: enum\n    variants: [paper, mail]";
const LIST_RECEIPT: &str = "  - {name: catalog.items.Receipt, kind: newtype, of: \"List<String>\"}";
const TEXT_KEPT: &str = "  - {name: catalog.items.Kept, kind: newtype, of: String}";
const RECORD_KEPT: &str = "  - name: catalog.items.Kept\n    kind: struct\n    fields:\n      - {name: text, type: String}";

/// The unchanged revision every case moves one thing away from.
fn base() -> EssIr {
    catalog(TEXT_RECEIPT, "String", "catalog.items.Note", TEXT_KEPT)
}

fn answers(delta: &EssDelta, id: &str) -> [Compatibility; 3] {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    [
        compatibility.callers(),
        compatibility.readers(),
        compatibility.history(),
    ]
}

fn document(delta: &EssDelta) -> Value {
    serde_json::from_str(&delta.to_canonical_json()).unwrap()
}

fn written(delta: &EssDelta, id: &str) -> Value {
    document(delta)["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()))
        .clone()
}

fn format(delta: &EssDelta) -> Value {
    document(delta)["format"].clone()
}

#[test]
fn a_type_redefined_from_text_to_record_is_breaking_for_callers_and_keeps_its_other_answers() {
    let delta = classified(
        &base(),
        &catalog(RECORD_RECEIPT, "String", "catalog.items.Note", TEXT_KEPT),
    )
    .unwrap();
    // `Receipt` is an input and stored: history keeps the `unknown` its stored use gives.
    assert_eq!(answers(&delta, RECEIPT_KIND), [B, C, U]);
    let change = written(&delta, RECEIPT_KIND);
    assert_eq!(
        change["compatibility"]["shapes"],
        serde_json::json!({"before": "scalar", "after": "record"})
    );
    assert_eq!(
        change["compatibility"]["uses"],
        serde_json::json!(["input", "stored"])
    );
    assert_eq!(format(&delta), "ess-diff/16");
}

#[test]
fn a_type_redefined_within_one_wire_form_keeps_its_answer_and_format() {
    let delta = classified(
        &base(),
        &catalog(ENUM_RECEIPT, "String", "catalog.items.Note", TEXT_KEPT),
    )
    .unwrap();
    assert_eq!(answers(&delta, RECEIPT_KIND), [U, C, U]);
    assert!(written(&delta, RECEIPT_KIND)["compatibility"]
        .get("shapes")
        .is_none());
    assert_eq!(format(&delta), "ess-diff/14");
}

#[test]
fn a_type_no_command_takes_keeps_its_answer_when_its_wire_form_moves() {
    let delta = classified(
        &base(),
        &catalog(TEXT_RECEIPT, "String", "catalog.items.Note", RECORD_KEPT),
    )
    .unwrap();
    assert_eq!(answers(&delta, KEPT_KIND), [C, C, U]);
    assert!(written(&delta, KEPT_KIND)["compatibility"]
        .get("shapes")
        .is_none());
    assert_eq!(format(&delta), "ess-diff/14");
}

#[test]
fn a_newtype_rewrapped_from_text_to_list_is_breaking_for_callers() {
    let delta = classified(
        &base(),
        &catalog(LIST_RECEIPT, "String", "catalog.items.Note", TEXT_KEPT),
    )
    .unwrap();
    assert_eq!(answers(&delta, RECEIPT_REPRESENTATION)[0], B);
    assert_eq!(
        written(&delta, RECEIPT_REPRESENTATION)["compatibility"]["shapes"],
        serde_json::json!({"before": "scalar", "after": "list"})
    );
}

#[test]
fn a_member_turned_from_text_to_list_is_breaking_for_callers_and_names_its_shapes() {
    let delta = classified(
        &base(),
        &catalog(
            TEXT_RECEIPT,
            "List<String>",
            "catalog.items.Note",
            TEXT_KEPT,
        ),
    )
    .unwrap();
    assert_eq!(answers(&delta, NOTE_TEXT), [B, C, C]);
    assert_eq!(
        written(&delta, NOTE_TEXT)["compatibility"]["shapes"],
        serde_json::json!({"before": "scalar", "after": "list"})
    );
    assert_eq!(format(&delta), "ess-diff/16");
}

#[test]
fn a_member_changed_within_one_wire_form_keeps_its_answer_and_format() {
    let delta = classified(
        &base(),
        &catalog(TEXT_RECEIPT, "Integer", "catalog.items.Note", TEXT_KEPT),
    )
    .unwrap();
    assert_eq!(answers(&delta, NOTE_TEXT), [U, C, C]);
    assert!(written(&delta, NOTE_TEXT)["compatibility"]
        .get("shapes")
        .is_none());
    assert_eq!(format(&delta), "ess-diff/14");
}

#[test]
fn a_variant_payload_turned_from_record_to_list_is_breaking_for_callers() {
    let delta = classified(
        &base(),
        &catalog(TEXT_RECEIPT, "String", "\"List<String>\"", TEXT_KEPT),
    )
    .unwrap();
    assert_eq!(answers(&delta, TAGGED_PLAIN)[0], B);
    assert_eq!(
        written(&delta, TAGGED_PLAIN)["compatibility"]["shapes"],
        serde_json::json!({"before": "record", "after": "list"})
    );
}

fn reread(after: &EssIr, edit: impl FnOnce(&mut Value)) -> Result<EssDelta, String> {
    let delta = classified(&base(), after).unwrap();
    let mut document = document(&delta);
    edit(&mut document);
    let raw: RawEssDelta = serde_json::from_value(document).map_err(|error| error.to_string())?;
    EssDelta::try_from(raw).map_err(|errors| errors.to_string())
}

fn change_mut<'a>(document: &'a mut Value, id: &str) -> &'a mut Value {
    document["changes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|change| change["id"] == id)
        .unwrap()
}

#[test]
fn a_type_changes_shapes_are_read_back_as_written() {
    let after = catalog(
        RECORD_RECEIPT,
        "List<String>",
        "catalog.items.Note",
        TEXT_KEPT,
    );
    let written = classified(&base(), &after).unwrap();
    let read = reread(&after, |_| {}).unwrap();
    assert_eq!(read.to_canonical_json(), written.to_canonical_json());
}

#[test]
fn a_reader_refuses_shapes_on_a_type_change_without_an_input_use() {
    let refused = reread(
        &catalog(TEXT_RECEIPT, "String", "catalog.items.Note", RECORD_KEPT),
        |document| {
            let format = document["format"].clone();
            assert_eq!(format, "ess-diff/14");
            document["format"] = Value::from("ess-diff/16");
            change_mut(document, KEPT_KIND)["compatibility"]["shapes"] =
                serde_json::json!({"before": "scalar", "after": "record"});
        },
    )
    .expect_err("a type no command takes records no shapes");
    assert!(refused.contains("conflicting_declaration"), "{refused}");
    assert!(refused.contains("shapes"), "{refused}");
}

#[test]
fn a_reader_refuses_type_change_shapes_that_share_a_wire_form() {
    let refused = reread(
        &catalog(RECORD_RECEIPT, "String", "catalog.items.Note", TEXT_KEPT),
        |document| {
            change_mut(document, RECEIPT_KIND)["compatibility"]["shapes"] =
                serde_json::json!({"before": "record", "after": "map"});
        },
    )
    .expect_err("shapes that share a wire form are not recorded");
    assert!(refused.contains("conflicting_declaration"), "{refused}");
    assert!(refused.contains("share a wire form"), "{refused}");
}
