//! Adversary: `ordered` (`stored.rs`) splits the related row's steps at its capture and refuses by
//! name ("no order of the arrangement reaches the branch") where nothing follows the capture. A
//! site the row search creates frozen directly — `OpenSite` with a second branch `into: Frozen` —
//! has no move after its capture, yet the frozen read is reachable through the other branch:
//! open, add the machine, freeze.
//!
//! The second case moves the site through two states after creation, and the third gives the
//! machine a second refusing act (`StartMachine`) inside the subject's own arrangement, both of
//! which the story's own fixture does not exercise.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/fleet-sites.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("fleet-order.yaml"), raw)])
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

fn refusals(result: &Synthesis) -> Vec<String> {
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

fn passes(text: &str, branch: &str) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refusals(&result);
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == branch),
        "{branch} is synthesized; refusals: {refused:#?}"
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
fn a_site_one_creator_branch_opens_frozen_still_reaches_the_frozen_read() {
    let text = MODEL
        .replace(
            "  - name: demo.fleet.OpenSite\n    input: []\n    outcomes:\n",
            "  - name: demo.fleet.OpenSite\n    input:\n      - {name: frozen, type: Boolean}\n    outcomes:\n      - name: opened-frozen\n        when: frozen == true\n        creates: demo.fleet.Site\n        instance: site_id\n        into: Frozen\n        emits: [demo.fleet.SiteOpened]\n        payload: {demo.fleet.SiteOpened: {site_id: {generated: true}}}\n",
        );
    assert_ne!(text, MODEL);
    passes(&text, "demo.fleet.StartMachine/outcome/frozen");
}

#[test]
fn a_site_moved_twice_after_the_machine_is_added_reaches_the_read() {
    let text = MODEL
        .replace(
            "      states: [Open, Frozen]\n      terminal: [Frozen]\n      transitions:\n        - {name: freeze, from: [Open], to: Frozen}",
            "      states: [Open, Frozen, Sealed]\n      terminal: [Sealed]\n      transitions:\n        - {name: freeze, from: [Open], to: Frozen}\n        - {name: seal, from: [Frozen], to: Sealed}",
        )
        .replace(
            "        when_related: {via: site_id, predicate: state == Frozen}",
            "        when_related: {via: site_id, predicate: state == Sealed}",
        )
        .replace(
            "        when_related: {via: input.site_id, predicate: state == Frozen}",
            "        when_related: {via: input.site_id, predicate: state != Open}",
        )
        .replace(
            "views:\n",
            "  - name: demo.fleet.SealSite\n    input:\n      - {name: site_id, type: demo.fleet.SiteId}\n    outcomes:\n      - {name: not-frozen, wrong_state: true, error: demo.fleet.SiteFrozen}\n      - name: sealed\n        moves: demo.fleet.Site.seal\n        instance: site_id\n        emits: [demo.fleet.SiteFrozenEvent]\n        payload: {demo.fleet.SiteFrozenEvent: {site_id: input.site_id}}\nviews:\n",
        )
        .replace(
            "      - demo.fleet.StartMachine\n",
            "      - demo.fleet.StartMachine\n      - demo.fleet.SealSite\n",
        );
    assert!(text.contains("SealSite\n    input"));
    assert!(text.contains("state != Open"));
    passes(&text, "demo.fleet.StartMachine/outcome/frozen");
}

#[test]
fn a_second_refusing_act_in_the_subject_arrangement_is_ordered_too() {
    let text = MODEL
        .replace(
            "      states: [Idle, Running]\n      terminal: [Running]\n      transitions:\n        - {name: start, from: [Idle], to: Running}",
            "      states: [Idle, Running, Stopped]\n      terminal: [Stopped]\n      transitions:\n        - {name: start, from: [Idle], to: Running}\n        - {name: stop, from: [Running], to: Stopped}",
        )
        .replace(
            "views:\n",
            "  - name: demo.fleet.StopMachine\n    input:\n      - {name: machine_id, type: demo.fleet.MachineId}\n    outcomes:\n      - {name: wrong-state, wrong_state: true, error: demo.fleet.MachineStateConflict}\n      - name: frozen\n        when_related: {via: site_id, predicate: state == Frozen}\n        error: demo.fleet.SiteFrozen\n      - name: stopped\n        moves: demo.fleet.Machine.stop\n        instance: machine_id\n        emits: [demo.fleet.MachineStarted]\n        payload: {demo.fleet.MachineStarted: {machine_id: input.machine_id}}\nviews:\n",
        )
        .replace(
            "      - demo.fleet.StartMachine\n",
            "      - demo.fleet.StartMachine\n      - demo.fleet.StopMachine\n",
        );
    assert!(text.contains("StopMachine\n    input"));
    passes(&text, "demo.fleet.StopMachine/outcome/frozen");
}
