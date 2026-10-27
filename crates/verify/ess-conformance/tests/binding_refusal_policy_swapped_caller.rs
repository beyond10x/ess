//! A binding's refusal-policy scenario run twice, once per caller order.
//!
//! `DefineGrant` writes the caller's `account_id` into the grant, so the scenarios that arrange a
//! grant run a second, swapped pass under the other caller. `Refresh`'s `grant-missing` refusal is
//! dropped by the binding, and its scenario ends each pass by counting the invocations the binding
//! made: the swapped pass sends the same literal `grant_id`, and its count must be its own, not
//! the first pass's invocation plus its own.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SYSTEM: &str = include_str!("fixtures/binding-refusal-policy-swapped-caller/system.yaml");
const COMPONENTS: &str =
    include_str!("fixtures/binding-refusal-policy-swapped-caller/components.yaml");
const DOMAIN: &str = include_str!("fixtures/binding-refusal-policy-swapped-caller/b.yaml");

const REFUSAL: &str = "refresh-on-ticket-moved/binding/refusal/grant-missing";

fn model() -> EssIr {
    let parsed = [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("b.yaml", DOMAIN),
    ]
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

#[test]
fn a_swapped_caller_pass_counts_only_its_own_invocations() {
    let ir = model();
    let synthesis = synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new());
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == REFUSAL),
        "`{REFUSAL}` is synthesized: {:#?}",
        synthesis.suite.scenarios.keys()
    );

    // A count is read over the whole scenario's correlation, so the swapped pass expects its own
    // invocation on top of the first pass's.
    let document: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    let counts: Vec<u64> = document["scenarios"][REFUSAL]["steps"]
        .as_array()
        .expect("steps")
        .iter()
        .filter(|step| step["step"] == "expect_invocation")
        .filter_map(|step| step["count"].as_u64())
        .collect();
    assert_eq!(counts, [1, 2]);

    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let not_passed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(not_passed, Vec::<String>::new());
    assert_eq!(report.scenarios.len(), synthesis.suite.scenarios.len());
}
