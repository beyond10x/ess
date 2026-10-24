//! A `when_related:` guard read through a stored reference, on a state the subject's creator
//! refuses, reported by an adopter: `StartMachine/frozen` reads the machine's site through its
//! stored `site_id`, and `AddMachine`, which writes that field, refuses a frozen site. The state is
//! reachable: add the machine, then freeze the site. The scenario froze the site first and then
//! added the machine to it, so `AddMachine` answered `frozen` and the setup failed.
//!
//! `tests/fixtures/fleet-sites.yaml`. Where the site is created frozen and nothing freezes it, no
//! order of the arrangement reaches the branch, and it is refused by name at synthesis time.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/fleet-sites.yaml");
const BRANCH: &str = "demo.fleet.StartMachine/outcome/frozen";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("fleet-sites.yaml"), raw)])
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

/// The commands the scenario sends, in order.
fn sent(result: &Synthesis, id: &str) -> Vec<String> {
    let (_, scenario) = result
        .suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| panic!("{id} is synthesized"));
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.name().to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_stored_reference_is_pointed_before_the_related_row_moves() {
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
    let order = sent(&result, BRANCH);
    let added = order
        .iter()
        .position(|command| command == "demo.fleet.AddMachine");
    let frozen = order
        .iter()
        .position(|command| command == "demo.fleet.FreezeSite");
    assert!(
        matches!((added, frozen), (Some(added), Some(frozen)) if added < frozen),
        "the machine is added before its site is frozen: {order:?}"
    );
    let passed = statuses(&result, &model);
    assert_eq!(passed.get(BRANCH), Some(&Status::Passed), "{passed:#?}");
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{passed:#?}"
    );
}

#[test]
fn a_stored_reference_no_order_reaches_is_refused_by_name() {
    let text = MODEL.replace(
        "      initial: Open\n      states: [Open, Frozen]\n      terminal: [Frozen]\n      transitions:\n        - {name: freeze, from: [Open], to: Frozen}",
        "      initial: Frozen\n      states: [Frozen, Open]\n      terminal: [Open]\n      transitions:\n        - {name: thaw, from: [Frozen], to: Open}",
    )
    .replace("moves: demo.fleet.Site.freeze", "moves: demo.fleet.Site.thaw");
    assert_ne!(text, MODEL);
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == BRANCH),
        "{BRANCH} is not written as a failing scenario"
    );
    let named: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| refusal.cause.to_string())
        .filter(|cause| cause.contains("no order of the arrangement reaches the branch"))
        .collect();
    assert!(
        named
            .iter()
            .any(|cause| cause.contains("demo.fleet.AddMachine")),
        "{:#?}",
        result
            .refusals
            .iter()
            .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
            .collect::<Vec<_>>()
    );
}
