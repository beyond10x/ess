//! The accepting branch behind a `when_related:` refusal over the related run's stored digest
//! (workbench report item 37, `.engineering/repro/workbench-2026-11-09/examples/37/spec`).
//!
//! `MarkConforming` refuses `digest-mismatch` where the run the subject names stores a digest other
//! than the one the caller sends (`when_related: {via: run_id, predicate: digest != input.digest}`).
//! The scenario of the accepting branch arranges a run whose stored digest is the one it sends, so
//! the refusal does not select it, and the interpreted target answers the accepting branch.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!(
    "../../../../.engineering/repro/workbench-2026-11-09/examples/37/spec/system.yaml"
);

const ACCEPTING: &str = "probe.run.MarkConforming/outcome/conforming";
const RECORD_RUN: &str = "probe.run.RecordRun";
const RECORD_SUBJECT: &str = "probe.run.RecordSubject";
const MARK: &str = "probe.run.MarkConforming";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert("system.yaml".to_owned(), MODEL.to_owned());
    let spec = Specification::assemble([(Source::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
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

/// The `digest` the scenario's one send of `command` carries.
fn sent_digest<'a>(scenario: &'a ConformanceScenario, command: &str) -> &'a ScenarioValue {
    let sent: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => input.get("digest"),
            _ => None,
        })
        .collect();
    assert_eq!(sent.len(), 1, "{scenario:#?}");
    sent[0]
}

/// The digest each run the scenario records stores, by the name the run is captured under.
fn stored_digests(scenario: &ConformanceScenario) -> BTreeMap<String, &ScenarioValue> {
    let mut stored = BTreeMap::new();
    let mut last = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == RECORD_RUN =>
            {
                last = input.get("digest");
            }
            ScenarioStep::CaptureInstance { instance, .. } => {
                if let Some(digest) = last.take() {
                    stored.insert(instance.to_string(), digest);
                }
            }
            _ => {}
        }
    }
    stored
}

/// The name of the run the scenario's subject is recorded against.
fn named_run(scenario: &ConformanceScenario) -> String {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == RECORD_SUBJECT =>
            {
                match input.get("run_id") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("the subject names a run\n{scenario:#?}"))
}

#[test]
fn the_accepting_scenario_arranges_a_run_the_refusal_does_not_select() {
    let result = synthesize(&ir());
    let scenario = scenario(&result, ACCEPTING);
    let sent = sent_digest(scenario, MARK);
    let stored = stored_digests(scenario);
    let run = named_run(scenario);
    assert_eq!(
        stored.get(&run),
        Some(&sent),
        "the run the subject names stores the digest the caller sends\n{scenario:#?}"
    );
}

#[test]
fn the_accepting_scenario_passes_on_the_interpreted_target() {
    let result = synthesize(&ir());
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report();
    let ran = report
        .scenarios
        .iter()
        .find(|ran| ran.scenario.to_string() == ACCEPTING)
        .unwrap_or_else(|| panic!("{ACCEPTING} did not run"));
    assert_eq!(ran.status, Status::Passed, "{ran:#?}");
    let failing: Vec<_> = report
        .scenarios
        .iter()
        .filter(|ran| ran.status != Status::Passed)
        .map(|ran| ran.scenario.to_string())
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}
