//! An `updates:` whose `sets:` writes the identity re-keys the record (ess/23, beyond10x/ess#429,
//! `docs/design/identity-changing-updates.md`). Adding the identity write is an
//! `outcome-sets-changed` like any other change to what a branch writes, and the delta names the
//! write as the re-key it is, in the written entry and in the line a person reads. No delta format
//! moves: the change kind is the `ess-diff/2` one it always was.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{EssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../ess-conformance/tests/fixtures/identity-changing-updates.yaml");

const ID: &str = "command/demo.vault.RenameSecret/outcome-sets-changed/renamed";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("vault.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The same command writing another field, literally: no re-key.
fn in_place() -> String {
    let out = MODEL.replace(
        "        sets: {name: input.new_name}\n",
        "        sets: {value: \"renamed\"}\n",
    );
    assert_ne!(out, MODEL);
    out
}

fn entry(document: &serde_json::Value) -> &serde_json::Value {
    let found = document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == ID)
        .unwrap_or_else(|| panic!("`{ID}` in {document:#}"));
    &found["change"]["changed"]
}

#[test]
fn rename_diff_reports_identity_write() {
    let delta: EssDelta = ess_diff::classified(&ir(&in_place()), &ir(MODEL)).unwrap();
    let json = delta.to_canonical_json();
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == ID)
        .unwrap_or_else(|| panic!("`{ID}` in {json}"));
    assert_eq!(change.relation(), SemanticRelation::Changed);
    assert_eq!(change.minimum_format(), 2, "{json}");
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = entry(&document);
    assert_eq!(
        changed["after"],
        serde_json::json!(["name <- input.new_name, re-keying the record"]),
        "{json}"
    );
    assert_eq!(
        changed["before"],
        serde_json::json!(["value <- literal `renamed`"]),
        "{json}"
    );
    let text = ess_diff::render::text(&delta);
    assert!(
        text.lines()
            .any(|line| line.contains("renamed") && line.contains("re-keys the record by `name`")),
        "{text}"
    );
}

#[test]
fn removing_the_identity_write_is_named_the_other_way() {
    let delta = ess_diff::classified(&ir(MODEL), &ir(&in_place())).unwrap();
    let text = ess_diff::render::text(&delta);
    assert!(
        text.lines().any(|line| line.contains("renamed")
            && line.contains("no longer re-keys the record by `name`")),
        "{text}"
    );
}

#[test]
fn a_write_of_another_field_names_no_re_key() {
    let other = in_place().replace("{value: \"renamed\"}", "{value: \"moved\"}");
    let delta = ess_diff::classified(&ir(&in_place()), &ir(&other)).unwrap();
    let json = delta.to_canonical_json();
    assert!(json.contains(ID), "{json}");
    assert!(!json.contains("re-keying"), "{json}");
    assert!(!ess_diff::render::text(&delta).contains("re-keys"));
}
