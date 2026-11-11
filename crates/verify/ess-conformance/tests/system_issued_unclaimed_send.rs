//! A send of a system-issued command that claims no branch is not awaited (workbench report item
//! 38, `docs/design/system-issued-commands.md`, *Which branches a scenario can claim*).
//!
//! `TakeDefault` of the committed example 28 is issued by the system actor `gap.demo.Clock`.
//! Its `no-default` scenario (`ObserveSubjectFact`) sends it before any row carries the identity
//! and checks, after it, that the created item is untouched. The send claims no branch, and the
//! rewrite made it an accepting `await_outcome`, which the interpreted target answers `NotIssued`
//! for an identity no record carries: the scenario ended at the runner's deadline. There is no
//! branch for an await to witness, so the send is dropped and the check after it stays.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SYSTEM: &str = include_str!(
    "../../../../.engineering/repro/workbench-2026-11-09/examples/28/spec/system.yaml"
);
const COMPONENTS: &str = include_str!(
    "../../../../.engineering/repro/workbench-2026-11-09/examples/28/spec/components.yaml"
);
const DEMO: &str =
    include_str!("../../../../.engineering/repro/workbench-2026-11-09/examples/28/spec/demo.yaml");

const NO_DEFAULT: &str = "gap.demo.TakeDefault/outcome/no-default";
const TAKE_DEFAULT: &str = "gap.demo.TakeDefault";

fn ir() -> EssIr {
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("demo.yaml", DEMO),
    ] {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        texts.insert(label.to_owned(), text.to_owned());
        parsed.push((Source::new(label), raw));
    }
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

#[test]
fn an_unclaimed_send_of_the_system_issued_step_is_not_awaited() {
    let result = synthesize(&ir());
    let scenario = scenario(&result, NO_DEFAULT);
    let awaits = scenario
        .steps
        .iter()
        .filter(|step| matches!(step, ScenarioStep::AwaitOutcome { .. }))
        .count();
    assert_eq!(awaits, 0, "{scenario:#?}");
    let not_issued: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectNotIssued {
                command, instance, ..
            } if command.to_string() == TAKE_DEFAULT => Some(instance),
            _ => None,
        })
        .collect();
    assert_eq!(not_issued.len(), 1, "{scenario:#?}");
    match not_issued[0] {
        ScenarioValue::Instance { instance } => {
            assert_eq!(instance.as_str(), "item", "{scenario:#?}");
        }
        other => panic!("expected the instance `item`, got {other:?}\n{scenario:#?}"),
    }
}

#[test]
fn the_unclaimed_send_scenario_passes_on_the_interpreted_target() {
    let result = synthesize(&ir());
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report();
    let ran = report
        .scenarios
        .iter()
        .find(|ran| ran.scenario.to_string() == NO_DEFAULT)
        .unwrap_or_else(|| panic!("{NO_DEFAULT} did not run"));
    assert_eq!(ran.status, Status::Passed, "{ran:#?}");
}
