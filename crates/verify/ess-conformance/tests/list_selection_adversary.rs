//! Whole-record admission must reject an invariant-invalid unselected tail.
use ess_compiler::{
    ir::{ResolvedBody, ResolvedMappingValue},
    resolve::compile_locating,
    source::SourceMap,
};
use ess_conformance::selection::Observation;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    facts::{FactPath, FactStore, FactValue},
    predicate::Truth,
};

#[test]
fn unselected_tail_must_satisfy_its_declared_record_invariant() {
    let model = include_str!("fixtures/binding-selection.yaml").replace(
        "events:\n",
        "    invariants: ['from != \"blocked\"']\nevents:\n",
    );
    let spec = Specification::assemble([(
        Source::new("selection-invariant.yaml"),
        RawSpecFile::parse(&model).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection-invariant.yaml", model.as_str());
    let ir = compile_locating(&spec, &sources, &["selection-invariant.yaml"]).unwrap();
    let leg_type = ir
        .types()
        .values()
        .find(|declared| declared.name.to_string() == "selection.core.Leg")
        .unwrap();
    let ResolvedBody::Struct { invariants, .. } = &leg_type.body else {
        panic!("fixture requires a declared record")
    };
    assert_eq!(
        invariants.len(),
        1,
        "the compiled source must retain its invariant"
    );
    let mut facts = FactStore::new();
    facts.set(
        FactPath::new("from").unwrap(),
        FactValue::Text("blocked".into()),
    );
    assert_eq!(invariants[0].predicate.evaluate(&facts), Truth::False);

    let binding = ir.bindings().values().next().unwrap();
    let mapped = &binding.mapping[0];
    let ResolvedMappingValue::Selection {
        selector,
        projection,
        ..
    } = &mapped.value
    else {
        panic!("fixture requires a selection mapping")
    };
    let refusal = Observation::of(&ir, binding, *selector, projection, &mapped.target_type)
        .expect_err("a constrained record cannot be certified after its invariant is erased");
    assert!(refusal.contains("selection-constraint"), "{refusal}");
}
