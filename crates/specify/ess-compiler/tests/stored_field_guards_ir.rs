//! `when_subject: {predicate}` lands in the IR as its own variant, beside the unchanged one.
//!
//! `docs/design/cross-record-and-stored-field-guards.md`, "Persisted formats and compatibility":
//! the domain gains `SubjectPredicate` and the IR `ResolvedCondition::SubjectPredicate { predicate,
//! input }` beside `SubjectField`, which is unchanged so every ess/6 model keeps its IR bytes.

use ess_compiler::ir::{EssIr, ResolvedCondition};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");
const HISTORY: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/subject-history.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

#[test]
fn the_predicate_form_compiles_to_its_own_variant_and_reads_its_siblings_subject() {
    let ir = ir(PARCELS);
    let dispatch = &ir.commands()[&"shipping.parcel.Dispatch".parse().unwrap()];
    let refusal = &dispatch.outcomes[0];
    let ResolvedCondition::SubjectPredicate { predicate, input } = &refusal.condition else {
        panic!("expected a subject predicate, got {:?}", refusal.condition)
    };
    assert_eq!(
        predicate.to_string(),
        "(service == Express and weight_kg > 20)"
    );
    assert!(input.is_none());
    let subject = dispatch
        .selection_subject(refusal)
        .expect("the refusal reads the subject its sibling names");
    assert_eq!(subject.entity.to_string(), "shipping.parcel.Parcel");

    let json = ir.to_canonical_json();
    assert!(json.contains(r#""kind": "subject_predicate""#), "{json}");
    assert!(
        !json.contains(r#""input": null"#),
        "an absent input guard is omitted, not written as null"
    );
}

#[test]
fn a_field_equals_model_keeps_its_ir_shape() {
    let json = ir(HISTORY).to_canonical_json();
    assert!(json.contains(r#""kind": "subject_field""#), "{json}");
    assert!(!json.contains("subject_predicate"), "{json}");
}
