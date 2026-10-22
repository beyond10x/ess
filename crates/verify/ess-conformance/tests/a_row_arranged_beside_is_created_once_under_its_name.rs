//! A command with `when_related` branches on two inputs, a cross-field branch on the second after
//! the branches on the first, reported by an adopter: the site arranged beside the machine rows
//! (beyond10x/ess#283) was opened again under its instance name for every machine row, because the
//! site a machine's stored reference is pointed at was created in the same block of distinctions as
//! the row beside, and so under the same name. The machine then sat in another site than the one the
//! send named, and `elsewhere` answered `not-alpha` and `tagged`.
//!
//! `tests/fixtures/beside-row-once.yaml`: a site owns its machines, `StartMachine` reads the site
//! through the machine's stored `site_id`, and `TagMachine` reads both the machine and the site.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/beside-row-once.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("beside-row-once.yaml"), raw)])
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
fn a_row_arranged_beside_is_created_once_under_its_name() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    for wanted in [
        "demo.yard.TagMachine/outcome/elsewhere",
        "demo.yard.TagMachine/outcome/not-alpha",
        "demo.yard.TagMachine/outcome/tagged",
    ] {
        assert!(
            result
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string() == wanted),
            "{wanted} is synthesized"
        );
    }
    for (id, scenario) in &result.suite.scenarios {
        let mut captured = BTreeSet::new();
        for step in &scenario.steps {
            if let ScenarioStep::CaptureInstance { instance, .. } = step {
                assert!(
                    captured.insert(instance.clone()),
                    "{id} captures `{instance}` twice"
                );
            }
        }
    }
    let passed = statuses(&result, &model);
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{passed:#?}"
    );
}
