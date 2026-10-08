//! A command input field whose type moves between a scalar and a record, a list or a map is
//! breaking for callers: every caller still sending the old shape is refused by the generated
//! decoder before anything runs.
//!
//! Before `ess-diff/16` the catch-all command arm answered `unknown` for callers and `compatible`
//! for history, so `--fail-on breaking` let the change through. The shapes each revision declares
//! are recorded beside the answer, as a type change's `uses` are, so a reader re-derives the answer
//! from the document alone. A change whose two types share a wire form keeps its earlier verdict and
//! its earlier format.
//!
//! `fixtures/upcast/` is the reported reproducer: `receipt: String` becomes
//! `receipt: catalog.items.Receipt {revision, evidence}`, and `after-additive.yaml` is the same
//! revision written with the existing idiom, an added `Optional` input.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, diff, Compatibility, EssDelta, FailOn, Gate, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const BEFORE: &str = include_str!("fixtures/upcast/before.yaml");
const AFTER: &str = include_str!("fixtures/upcast/after.yaml");
const AFTER_ADDITIVE: &str = include_str!("fixtures/upcast/after-additive.yaml");

const RECEIPT: &str = "command/catalog.items.Accept/input-type-changed/receipt";
const NOTE: &str = "command/catalog.items.Add/input-type-changed/note";

use Compatibility::{Breaking as B, Compatible as C, Unknown as U};

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

/// One revision of a small catalog whose `Add` command takes `note` typed `note_type`.
///
/// Every declared type is declared on both sides, so the input field's type is the only thing that
/// moves between two revisions built from this.
fn catalog(note_type: &str) -> EssIr {
    ir(&format!(
        r#"format: ess/23
system: catalog
version: v1
domain: catalog.items

types:
  - {{name: catalog.items.ItemId, kind: newtype, of: String}}
  - name: catalog.items.Note
    kind: struct
    fields:
      - {{name: text, type: String}}
  - {{name: catalog.items.Label, kind: newtype, of: String}}
  - {{name: catalog.items.Wrapped, kind: newtype, of: catalog.items.Note}}
  - {{name: catalog.items.Rewrapped, kind: newtype, of: catalog.items.Wrapped}}
  - name: catalog.items.Tagged
    kind: union
    tag: kind
    variants:
      plain: catalog.items.Note
  - name: catalog.items.Colour
    kind: enum
    variants: [red, green]

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

/// The written change with this id, as the document carries it.
fn written(delta: &EssDelta, id: &str) -> Value {
    let document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` in {document}"))
        .clone()
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

#[test]
fn the_reproducers_text_receipt_turned_record_is_breaking_for_callers_and_fails_the_gate() {
    let delta = classified(&ir(BEFORE), &ir(AFTER)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(answers(&delta, RECEIPT), [B, U, C], "{json}");
    assert!(
        failing(&delta, FailOn::Breaking).contains(&RECEIPT.to_owned()),
        "--fail-on breaking fails on the receipt\n{json}"
    );
    let text = ess_diff::render::text(&delta);
    assert!(text.contains("breaking for callers"), "{text}");
}

#[test]
fn a_record_input_turned_back_into_text_is_breaking_for_callers() {
    let delta = classified(&ir(AFTER), &ir(BEFORE)).unwrap();
    assert_eq!(
        answers(&delta, RECEIPT),
        [B, U, C],
        "{}",
        delta.to_canonical_json()
    );
}

#[test]
fn text_turned_list_or_map_is_breaking_for_callers_in_either_direction() {
    for other in ["List<String>", "Map<String, String>", "catalog.items.Note"] {
        for (from, to) in [("String", other), (other, "String")] {
            let delta = classified(&catalog(from), &catalog(to)).unwrap();
            assert_eq!(
                answers(&delta, NOTE),
                [B, U, C],
                "{from} → {to}\n{}",
                delta.to_canonical_json()
            );
        }
    }
}

#[test]
fn a_shape_is_read_through_optional_newtypes_unions_and_enums() {
    // An `Optional` record still refuses text; a newtype (of a newtype) of a record is a record on
    // the wire; a union is an object; an enum is text, as a newtype of `String` is.
    for (from, to) in [
        ("String", "Optional<catalog.items.Note>"),
        ("Optional<String>", "catalog.items.Note"),
        ("String", "catalog.items.Rewrapped"),
        ("catalog.items.Label", "catalog.items.Tagged"),
        ("catalog.items.Colour", "List<catalog.items.Colour>"),
        ("Integer", "Map<String, Integer>"),
        // An array is never an object either.
        ("catalog.items.Note", "List<catalog.items.Note>"),
        ("List<String>", "Map<String, String>"),
    ] {
        let delta = classified(&catalog(from), &catalog(to)).unwrap();
        assert_eq!(
            answers(&delta, NOTE),
            [B, U, C],
            "{from} → {to}\n{}",
            delta.to_canonical_json()
        );
    }
}

#[test]
fn a_change_between_two_types_of_one_wire_form_keeps_its_verdict_and_format() {
    for (from, to) in [
        ("String", "Integer"),
        ("String", "catalog.items.Label"),
        ("String", "catalog.items.Colour"),
        ("String", "Optional<String>"),
        ("catalog.items.Note", "catalog.items.Tagged"),
        ("catalog.items.Note", "Map<String, String>"),
        ("List<String>", "List<catalog.items.Note>"),
        ("String", "Json"),
        ("Json", "catalog.items.Note"),
    ] {
        let delta = classified(&catalog(from), &catalog(to)).unwrap();
        let json = delta.to_canonical_json();
        assert_eq!(answers(&delta, NOTE), [U, U, C], "{from} → {to}\n{json}");
        assert_eq!(
            delta.format.to_string(),
            "ess-diff/14",
            "{from} → {to}\n{json}"
        );
        assert!(
            written(&delta, NOTE)["compatibility"]
                .get("shapes")
                .is_none(),
            "{from} → {to}\n{json}"
        );
    }
}

#[test]
fn the_additive_idiom_keeps_its_verdict_and_passes_the_breaking_gate() {
    let delta = classified(&ir(BEFORE), &ir(AFTER_ADDITIVE)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(
        answers(
            &delta,
            "command/catalog.items.Accept/input-added/receipt_revision"
        ),
        [U, U, C],
        "{json}"
    );
    assert!(failing(&delta, FailOn::Breaking).is_empty(), "{json}");
    assert_eq!(delta.format.to_string(), "ess-diff/14", "{json}");
}

#[test]
fn the_shapes_are_written_beside_the_answer_as_ess_diff_16_and_read_back() {
    let delta = classified(&ir(BEFORE), &ir(AFTER)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/16", "{json}");
    assert_eq!(
        written(&delta, RECEIPT)["compatibility"]["shapes"],
        serde_json::json!({"before": "scalar", "after": "record"}),
        "{json}"
    );
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");

    assert!(
        delta
            .to_canonical_json_for("ess-diff/15".parse().unwrap())
            .is_err(),
        "ess-diff/15 cannot carry the shapes"
    );
}

/// Edits `RECEIPT`'s written compatibility in the reproducer's delta and reads the result back.
fn reread(edit: impl FnOnce(&mut Value)) -> Result<EssDelta, String> {
    let delta = classified(&ir(BEFORE), &ir(AFTER)).unwrap();
    let mut document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    edit(&mut document);
    let raw: RawEssDelta = serde_json::from_value(document).map_err(|error| error.to_string())?;
    EssDelta::try_from(raw).map_err(|errors| errors.to_string())
}

fn receipt_mut(document: &mut Value) -> &mut Value {
    document["changes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|change| change["id"] == RECEIPT)
        .unwrap()
}

#[test]
fn a_reader_refuses_shapes_below_ess_diff_16() {
    let refused = reread(|document| document["format"] = Value::from("ess-diff/15"))
        .expect_err("an `ess-diff/15` document carries no shapes");
    assert!(refused.contains("unsupported_format_version"), "{refused}");
}

#[test]
fn a_reader_refuses_an_answer_the_written_shapes_do_not_derive() {
    let refused = reread(|document| {
        receipt_mut(document)["compatibility"]["shapes"]["after"] = Value::from("scalar");
    })
    .expect_err("scalar → scalar records no shapes and derives no breaking answer");
    assert!(refused.contains("conflicting_declaration"), "{refused}");

    let refused = reread(|document| {
        let compatibility = receipt_mut(document)["compatibility"]
            .as_object_mut()
            .unwrap();
        compatibility.remove("shapes");
    })
    .expect_err("without its shapes the change derives unknown, not breaking");
    assert!(refused.contains("conflicting_declaration"), "{refused}");
}

#[test]
fn the_unclassified_delta_keeps_its_format_and_records_no_shapes() {
    let delta = diff(&ir(BEFORE), &ir(AFTER)).unwrap();
    let json = delta.to_canonical_json();
    assert!(delta.compatibility().is_none(), "{json}");
    assert!(!json.contains("\"shapes\""), "{json}");
    assert_ne!(delta.format.to_string(), "ess-diff/16", "{json}");
}
