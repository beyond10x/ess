//! An aggregate grouped by a key the creating command sets is witnessed where the move that brings
//! its rows to a state is guarded by the row's stored fields and never writes the key
//! (beyond10x/ess#279).
//!
//! `Close` refuses a ticket whose `priority` is below 10 or which is not `urgent`, and otherwise
//! closes it; it writes no `region`. A row the plan creates with its chosen `region` and the plain
//! witness for the other fields is refused by `Close`, so the row is created with the other fields
//! chosen toward `Close`'s guards and its `region` kept, rather than searched for afresh with a
//! `region` of the search's own, which read as a move rewriting the key (ESS-SYNTH-017).
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{
    scenario::ScenarioId,
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/aggregate-stored-guarded-move.yaml");
const AGGREGATE: &str = "demo.desk.PerRegion/aggregate";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// The aggregate is synthesized, and nothing in the suite is refused with ESS-SYNTH-017.
#[test]
fn issue_279_a_stored_guarded_move_does_not_refuse_the_aggregate() {
    let synthesis = synthesize(&ir());
    let refused = refusals(&synthesis);
    assert!(
        !refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-017")),
        "{refused:#?}"
    );
    assert!(
        synthesis
            .suite
            .scenarios
            .contains_key(&ScenarioId::parse(AGGREGATE).unwrap()),
        "{refused:#?}"
    );
}

/// No scenario of the suite fails against the interpreted target.
#[test]
fn issue_279_the_interpreted_target_fails_no_scenario() {
    let model = ir();
    let suite = synthesize(&model).suite;
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let statuses: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .map(|run| (run.scenario.to_string(), format!("{:?}", run.status)))
        .collect();
    for run in &report.scenarios {
        assert!(
            matches!(run.status, Status::Passed | Status::Unsupported),
            "{}: {:?}\n{statuses:#?}",
            run.scenario,
            run.checks
        );
    }
}
