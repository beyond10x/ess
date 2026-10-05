//! Browser composition of a conditioned binding (ess/22, beyond10x/ess#268 slice 2).
//!
//! Both browser products compose the model and its synthesized suite/36: the browser conformance
//! product, which binds the original source and the admitted suite by digest and presents the
//! complete compiled model — the condition included — beside every scenario declaration, and the
//! scenario player, whose model projection carries the condition beside the binding. Neither
//! evaluates anything: execution is the product's Rust runner inside the consumer's own WASM build.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::web_execution::bundle::{Execution, SourceDocument};
use ess_conformance::AdmittedSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("fixtures/binding-condition.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("sources/0000.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn the_browser_product_composes_a_conditioned_binding_and_its_suite() {
    let ir = ir_of(MODEL);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/36"
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let artifacts = ess_conformance::web::emit_product(
        &[SourceDocument {
            path: "sources/0000.yaml".into(),
            text: MODEL.into(),
        }],
        &Execution::Ordinary(admitted),
    )
    .unwrap_or_else(|error| panic!("the product composes the conditioned model: {error:?}"));
    let declarations = artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .find(|contents| contents.contains("received/binding/condition-false"))
        .expect("a presented document holds the negative witness");
    assert!(
        declarations.contains("expect_no_invocation"),
        "the zero-invocation step is presented"
    );
    assert!(
        declarations.contains("event.kind"),
        "the compiled model presented carries the condition"
    );
}

#[test]
fn the_scenario_player_carries_the_condition_beside_the_binding() {
    let ir = ir_of(MODEL);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let artifacts = ess_conformance::web::emit(&ir, &suite)
        .unwrap_or_else(|error| panic!("the player composes the conditioned model: {error}"));
    let model: serde_json::Value =
        serde_json::from_str(&artifacts["model.json"].contents).expect("model.json is JSON");
    let bindings = model["bindings"].as_array().expect("bindings");
    let received = bindings
        .iter()
        .find(|binding| binding["name"] == "received")
        .expect("the conditioned binding");
    let predicate = ir
        .bindings()
        .values()
        .find(|binding| binding.name.as_str() == "received")
        .and_then(|binding| binding.condition.as_ref())
        .expect("a condition")
        .plan
        .predicate
        .to_string();
    assert_eq!(received["where"], predicate.as_str(), "{received:#}");
    let logged = bindings
        .iter()
        .find(|binding| binding["name"] == "logged")
        .expect("the plain binding");
    assert!(logged.get("where").is_none(), "{logged:#}");
}
