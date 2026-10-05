//! A value read through a different input path is a behaviour change, and the diff renders both
//! paths (`docs/design/expression-family-source22.md`, A4 and "Target and projection
//! obligations": resolved value paths are behaviour). So is a fallback that moves from one input
//! to another; a model that changes nothing is reported as changing nothing.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/input-value-paths.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("leases.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn rendered(before: &str, after: &str) -> String {
    diff(&ir(before), &ir(after)).unwrap().to_canonical_json()
}

#[test]
fn a4_a_retargeted_path_is_a_behaviour_change() {
    let after = MODEL.replacen(
        "          sealed_label: input.sealed.label",
        "          sealed_label: input.opening.label",
        1,
    );
    assert_ne!(after, MODEL);
    let json = rendered(MODEL, &after);
    assert!(json.contains("input.sealed.label"), "{json}");
    assert!(json.contains("input.opening.label"), "{json}");
    assert!(json.contains("outcome-sets-changed"), "{json}");
}

#[test]
fn a4_a_moved_fallback_is_a_behaviour_change() {
    let after = MODEL.replacen(
        "          label: {input: previous.label, else: input.settings.defaults.label}\n          sealed_label",
        "          label: {input: previous.label, else: input.opening.label}\n          sealed_label",
        1,
    );
    assert_ne!(after, MODEL);
    let json = rendered(MODEL, &after);
    assert!(
        json.contains("input.previous.label, else input.settings.defaults.label")
            && json.contains("input.previous.label, else input.opening.label"),
        "{json}"
    );
}

#[test]
fn a4_an_unchanged_model_changes_nothing() {
    let json = rendered(MODEL, MODEL);
    assert!(!json.contains("outcome-sets-changed"), "{json}");
}
