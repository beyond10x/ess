//! An `affects:` row is arranged past its creator's `exists: false` branch whether that branch
//! emits an event or names an error.
//!
//! `ResetMember/reset` deletes the `Ticket` rows `where: member_id == input.member_id`. The only
//! creator of a `Ticket` is `OpenTicket/opened`, behind a `not-found` branch guarded by
//! `when_related: {via: input.member_id, exists: false}`. A creator sent naming a captured member
//! never takes that branch: the member exists. Synthesis passed it over only where the branch named
//! an error, so the same model with the branch emitting `demo.w.Missed` was refused as ESS-SYNTH-001
//! ("created only by `demo.w.OpenTicket/opened`, which no arrangement takes"). The two fixtures
//! differ in that one line.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const EMITTING: &str = include_str!("fixtures/affects-past-emitting-not-found.yaml");
const ERRORING: &str = include_str!("fixtures/affects-past-erroring-not-found.yaml");

const RESET: &str = "demo.w.ResetMember/outcome/reset";

fn ir(model: &str) -> EssIr {
    let raw = RawSpecFile::parse(model).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), model.to_owned());
    let spec = Specification::assemble(vec![(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Each scenario's id and the status the model interpreter gives it.
fn interpreted(model: &EssIr) -> BTreeMap<String, Status> {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

#[test]
fn an_affects_row_is_arranged_past_a_creators_emitting_not_found_branch() {
    let model = ir(EMITTING);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(refusals.is_empty(), "synthesis refuses {refusals:#?}");
    let statuses = interpreted(&model);
    assert_eq!(
        statuses.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "demo.w.CreateMember/outcome/created",
            "demo.w.OpenTicket/outcome/not-found",
            "demo.w.OpenTicket/outcome/opened",
            RESET,
        ],
        "every branch keeps its scenario"
    );
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "the interpreter fails {failing:?}");
}

#[test]
fn the_reset_scenario_is_the_same_whether_the_not_found_branch_emits_or_errs() {
    let steps = |model: &str| {
        let synthesis = ess_conformance::synthesize::synthesize(&ir(model));
        synthesis
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == RESET)
            .map_or_else(
                || panic!("{RESET} is synthesized"),
                |(_, scenario)| scenario.steps.clone(),
            )
    };
    assert_eq!(steps(EMITTING), steps(ERRORING));
}

#[test]
fn the_erroring_not_found_branch_suite_passes_the_model_interpreter() {
    let statuses = interpreted(&ir(ERRORING));
    assert_eq!(statuses.len(), 4, "{statuses:?}");
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "the interpreter fails {failing:?}");
}
