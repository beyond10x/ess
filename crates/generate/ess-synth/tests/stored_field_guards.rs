//! The native plan quotes a stored-field guard as the predicate it is (ess#75).
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::SynthesisPlan;

const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");

#[test]
fn the_plan_names_the_stored_fields_a_refusal_reads() {
    let spec = Specification::assemble([(
        Source::new("parcels.yaml"),
        RawSpecFile::parse(PARCELS).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let plan = SynthesisPlan::of(&ir).to_markdown();
    assert!(
        plan.contains(
            "when the existing subject's stored fields satisfy `(service == Express and weight_kg > 20)`"
        ),
        "{plan}"
    );
    assert!(!plan.contains("SubjectPredicate"), "{plan}");
}
