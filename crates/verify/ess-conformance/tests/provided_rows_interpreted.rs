//! The interpreted target provides a row the deployment holds
//! (story:a-fixture-may-be-a-row-the-deployment-already-holds).
//!
//! The interpreted target is the deployment: it answers an entity fixture by creating a row of the
//! entity in the state the fixture names before the scenario begins, and answers its identity. A
//! type-only fixture stays unsupported there.
use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ConformanceReport, Status};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const LINKS: &str = include_str!("fixtures/provided-related-row.yaml");

const LINKED: &str = "linking.links.CreateLink/outcome/linked";
const UNKNOWN_APP: &str = "linking.links.CreateLink/outcome/unknown-app";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(LINKS).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("links.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run(admitted: &AdmittedSuite) -> ConformanceReport {
    Runner::for_suite(admitted.suite())
        .run_admitted(admitted, &Interpreted::for_model(ir()))
        .into_report()
}

/// Both branches of the related guard pass: the row is created in the named state before the
/// linked scenario begins, and the absent branch is sent an identity no row holds.
#[test]
fn the_interpreted_target_provides_the_row_and_both_branches_pass() {
    let result = ess_conformance::synthesize::synthesize(&ir());
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = run(&admitted);
    let statuses: Vec<_> = report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(
        statuses,
        [
            (LINKED.to_owned(), Status::Passed),
            (UNKNOWN_APP.to_owned(), Status::Passed)
        ],
        "{:#?}",
        report.scenarios
    );
}

/// A type-only fixture stays unsupported: the target holds no value of a type that no provided row
/// answers.
#[test]
fn the_interpreted_target_leaves_a_type_only_fixture_unsupported() {
    let result = ess_conformance::synthesize::synthesize(&ir());
    let mut document: serde_json::Value =
        serde_json::from_str(&result.suite.to_canonical_json().unwrap()).unwrap();
    let mut edited = false;
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "resolve_fixtures" {
                for field in step["fixtures"]["fields"].as_array_mut().unwrap() {
                    let field = field.as_object_mut().unwrap();
                    field.remove("entity");
                    field.remove("state");
                    edited = true;
                }
            }
        }
    }
    assert!(edited);
    let admitted = AdmittedSuite::from_json(&serde_json::to_string(&document).unwrap()).unwrap();
    let report = run(&admitted);
    let linked = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == LINKED)
        .expect("the linked scenario runs");
    assert_eq!(linked.status, Status::Unsupported, "{linked:#?}");
    assert!(format!("{linked:?}").contains("type-only"), "{linked:#?}");
}
