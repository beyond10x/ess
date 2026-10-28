//! A caller attribute keeps a binding's refusal-policy scenario
//! (`story:a-caller-attribute-keeps-the-binding-refusal-scenario`).
//!
//! The binding `give-on-touched` invokes `Give`, whose `missing` branch refuses on
//! `when_related: {via: input.grant_id, exists: false}`. The model's only actor declares a caller
//! attribute, and the related row's creator sets `account` from it. Synthesis writes the binding's
//! refusal-policy scenario for `missing`, the suite is admitted and passes the interpreted target,
//! and its scenario and refusal counts are those of the same model without the attribute, whose
//! suite keeps its bytes.

use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, Runner, Synthesis};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

const ATTR: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/caller-binding-refusal/attr/system.yaml"),
    ),
    (
        "components.yaml",
        include_str!("fixtures/caller-binding-refusal/attr/components.yaml"),
    ),
    (
        "d.yaml",
        include_str!("fixtures/caller-binding-refusal/attr/d.yaml"),
    ),
];
const PLAIN: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/caller-binding-refusal/plain/system.yaml"),
    ),
    (
        "components.yaml",
        include_str!("fixtures/caller-binding-refusal/plain/components.yaml"),
    ),
    (
        "d.yaml",
        include_str!("fixtures/caller-binding-refusal/plain/d.yaml"),
    ),
];

const REFUSAL: &str = "give-on-touched/binding/refusal/missing";

/// The canonical JSON of the twin's suite, taken at the base.
const PLAIN_SHA256: &str = "84f68d8284c7136b3dcb4171441faec4d6747f4dac08af640463ed5b6c6ea80b";

fn model(files: [(&str, &str); 3]) -> EssIr {
    let parsed = files.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

/// The scenarios that did not pass the interpreted target, after the suite was admitted.
fn not_passed(ir: &EssIr, synthesis: &Synthesis) -> Vec<String> {
    let json = synthesis.suite.to_canonical_json().expect("serialises");
    let admitted = AdmittedSuite::from_json(&json).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    assert_eq!(report.scenarios.len(), synthesis.suite.scenarios.len());
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect()
}

#[test]
fn a_caller_attribute_keeps_the_binding_refusal_scenario() {
    let ir = model(ATTR);
    let synthesis = synthesize(&ir);
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == REFUSAL),
        "`{REFUSAL}` is written: {:#?} {:#?}",
        synthesis.suite.scenarios.keys(),
        synthesis.refusals
    );
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());

    let twin = model(PLAIN);
    let plain = synthesize(&twin);
    assert_eq!(
        (synthesis.suite.scenarios.len(), synthesis.refusals.len()),
        (plain.suite.scenarios.len(), plain.refusals.len()),
        "the counts equal the model without the attribute"
    );
    assert_eq!(not_passed(&twin, &plain), Vec::<String>::new());
    let json = plain.suite.to_canonical_json().expect("serialises");
    let digest = Sha256::digest(json.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String");
            hex
        });
    assert_eq!(digest, PLAIN_SHA256, "the twin keeps its suite bytes");
}
