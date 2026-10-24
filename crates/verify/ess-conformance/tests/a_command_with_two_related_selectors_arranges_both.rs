//! A command whose `when_related:` branches read two selectors, each guarding its own outcome — one
//! row set per group key, one per workspace — reported by an adopter: synthesis arranged the rows
//! of one selector per command and left every outcome the second separates without a witness.
//!
//! `tests/fixtures/two-related-selectors.yaml`: `Assign` refuses with `NoMember` while no member of
//! the input's group exists, then with `NoSeat` while no seat of the input's workspace exists, and
//! otherwise creates the assignment.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/two-related-selectors.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("two-related-selectors.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(result: &Synthesis, model: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

#[test]
fn a_command_with_two_related_selectors_arranges_both() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{:?} {}: {}",
                refusal.scenario.as_ref().map(ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    let passed = statuses(&result, &model);
    for wanted in [
        "demo.seats.Assign/outcome/no-member",
        "demo.seats.Assign/outcome/no-seat",
        "demo.seats.Assign/outcome/assigned",
    ] {
        assert_eq!(
            passed.get(wanted),
            Some(&Status::Passed),
            "{wanted} is witnessed: {passed:#?}"
        );
    }
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{passed:#?}"
    );
}
