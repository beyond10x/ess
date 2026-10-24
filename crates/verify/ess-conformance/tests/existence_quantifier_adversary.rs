//! Adversary: `held_in_body` (`subject_fact.rs`) grounds an `exists` or `forall` leaf of a
//! `when_related:` guard on the input the existence family fixed. The story's fixture uses
//! `not: {exists: …, that: entry.action == input.action}`. This case states the same membership
//! rule in the other shape an adopter writes, `forall: {…, that: entry.action != input.action}`:
//! the same refusal, De Morgan's form, whose grounded hint keeps `!=`, so it steers the row toward
//! a list *without* the action.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/existence-list-membership.yaml");
const EXISTS: &str = "demo.pick.Select/outcome/selection-exists";
const NOT_LISTED: &str = "          predicate:\n            not:\n              exists:\n                in: entries\n                as: entry\n                that: entry.action == input.action\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("existence-quantifier.yaml"), raw)])
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

fn passes(text: &str) {
    let model = ir(text);
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
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == EXISTS),
        "{EXISTS} is synthesized; refusals: {refused:#?}"
    );
    let passed = statuses(&result, &model);
    let failing: Vec<String> = passed
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect();
    assert!(
        failing.is_empty(),
        "not passing: {failing:#?}\nrefusals: {refused:#?}"
    );
}

#[test]
fn an_existence_outcome_passes_the_forall_form_of_the_membership_guard() {
    assert!(MODEL.contains(NOT_LISTED));
    let text = MODEL.replace(
        NOT_LISTED,
        "          predicate:\n            forall:\n              in: entries\n              as: entry\n              that: entry.action != input.action\n",
    );
    passes(&text);
}
