//! A string operator compared with a different parameter or input, or with a literal where it
//! read an input, is a behaviour change, and the diff renders both conditions
//! (`docs/design/expression-family-source22.md`, "Target and projection obligations" and final
//! review decision 7: conditions are rendered from the canonical form). A model that changes
//! nothing is reported as changing nothing (beyond10x/ess#200).

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/typed-text-operands.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("directory.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn rendered(before: &str, after: &str) -> String {
    diff(&ir(before), &ir(after)).unwrap().to_canonical_json()
}

fn edited(from: &str, to: &str) -> String {
    let after = MODEL.replacen(from, to, 1);
    assert_ne!(after, MODEL, "the model writes {from}");
    after
}

#[test]
fn t200_a_guard_reading_another_input_is_a_behaviour_change() {
    let after = edited(
        "when: {caller: {starts_with: {input: blocked}}}",
        "when: {caller: {starts_with: {input: digits}}}",
    );
    let json = rendered(MODEL, &after);
    assert!(
        json.contains("{input: blocked}") && json.contains("{input: digits}"),
        "{json}"
    );
}

#[test]
fn t200_a_guard_reading_a_literal_where_it_read_an_input_is_a_behaviour_change() {
    let after = edited(
        "when: {caller: {starts_with: {input: blocked}}}",
        "when: {caller: {starts_with: \"+44\"}}",
    );
    let json = rendered(MODEL, &after);
    assert!(json.contains("{input: blocked}"), "{json}");
    assert!(json.contains("+44"), "{json}");
}

#[test]
fn t200_a_filter_testing_its_parameter_the_other_way_is_a_behaviour_change() {
    let after = edited(
        "filter: {note: {contains: {param: term}}}",
        "filter: {note: {ends_with: {param: term}}}",
    );
    let json = rendered(MODEL, &after);
    assert!(
        json.contains("note contains {param: term}")
            && json.contains("note ends_with {param: term}"),
        "{json}"
    );
}

#[test]
fn t200_an_unchanged_model_changes_nothing() {
    let json = rendered(MODEL, MODEL);
    assert!(!json.contains("{param:"), "{json}");
    assert!(!json.contains("{input:"), "{json}");
}
