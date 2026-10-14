//! Adversary pass 1 for an external refusal answered before the row (`before_row: true`, ess/25,
//! beyond10x/ess#534): `ess verify diff` over the key gained and lost.
//!
//! The design reports the key as the branch's `outcome-condition-changed`, its condition written
//! with `; before the row` where the key is set, classified as every condition change is. Gaining
//! the key answers the refusal for requests the held state or existence answered before, so the
//! change must not be classified compatible for callers.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::Compatibility;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SIGN_IN: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/external-before-row.yaml");

const ID: &str = "command/demo.desk.SignIn/outcome-condition-changed/bad-credential";

fn marked() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("desk.yaml"),
        RawSpecFile::parse(SIGN_IN).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The same IR with the key dropped: the external answered at step 6.
fn unmarked() -> EssIr {
    marked().with_commands_rewritten(|command| {
        let mut command = command.clone();
        for outcome in &mut command.outcomes {
            outcome.before_row = false;
        }
        command
    })
}

fn assert_reported(before: &EssIr, after: &EssIr, written_before: &str, written_after: &str) {
    let delta = ess_diff::classified(before, after).unwrap();
    let json = delta.to_canonical_json();
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == ID)
        .unwrap_or_else(|| panic!("`{ID}` in {json}"));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    assert_ne!(
        compatibility.callers(),
        Compatibility::Compatible,
        "the requests the refusal answers moved: {json}"
    );
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == ID)
        .map(|entry| &entry["change"]["changed"])
        .unwrap();
    assert_eq!(changed["before"], written_before, "{json}");
    assert_eq!(changed["after"], written_after, "{json}");
}

#[test]
fn gaining_before_row_is_a_condition_change_not_compatible_for_callers() {
    let delta = ess_diff::classified(&unmarked(), &marked()).unwrap();
    let json = delta.to_canonical_json();
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == ID)
        .map_or_else(
            || panic!("`{ID}` in {json}"),
            |entry| entry["change"]["changed"].clone(),
        );
    let before = changed["before"].as_str().unwrap().to_owned();
    assert!(
        !before.ends_with("; before the row"),
        "unmarked renders without the suffix: {json}"
    );
    assert_reported(
        &unmarked(),
        &marked(),
        &before,
        &format!("{before}; before the row"),
    );
    assert_reported(
        &marked(),
        &unmarked(),
        &format!("{before}; before the row"),
        &before,
    );
}

#[test]
fn a_model_without_the_key_diffs_to_nothing_against_itself() {
    let delta = ess_diff::classified(&unmarked(), &unmarked()).unwrap();
    assert!(delta.changes().is_empty(), "{}", delta.to_canonical_json());
}
