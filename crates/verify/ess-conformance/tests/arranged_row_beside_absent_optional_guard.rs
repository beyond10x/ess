//! A row arranged through a creating command whose sibling refusals read an `Optional` input is
//! arranged by a candidate that supplies the input or decides the guard `false`
//! (beyond10x/ess#728): a candidate `Unknown` only for an input it left out is passed over, not the
//! end of the search.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/absent-optional-beside-stored-guard.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("absent-optional.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refused(result: &Synthesis) -> Vec<String> {
    result
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
        .collect()
}

fn statuses(result: &Synthesis, target: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

#[test]
fn a_stored_guard_row_is_arranged_beside_an_absent_optional_refusal() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    assert!(refused.is_empty(), "{refused:#?}");
    let ran = statuses(&result, &model);
    for id in [
        "example.review.Annotate/outcome/not-accepted",
        "example.review.Annotate/outcome/annotated",
    ] {
        assert!(ran.contains_key(id), "{id}: {:#?}", ran.keys());
    }
    let failed: Vec<_> = ran
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
