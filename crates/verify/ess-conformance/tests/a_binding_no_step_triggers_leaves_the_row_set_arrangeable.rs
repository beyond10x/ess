//! A row-set guard over an entity a binding writes, where no step of the scenario publishes the
//! binding's trigger (a downstream adopter's report of 2026-10-26).
//!
//! `Reconcile` keeps a holder's tickets through `when_related: {entity: demo.y.Ticket, where:
//! holder == input.holder, exists: true}`. The binding `issue-on-enrolled` writes `Ticket` when
//! `Enrolled` arrives, and neither branch of `Reconcile` is arranged through `Enrol`: the tickets
//! are issued directly, so the binding never runs and the rows stay the steps' to decide. Both
//! branches are synthesized and pass the interpreted target.
//!
//! The variant `binding-no-step-triggers-published` sets the binding off from `Issued`, the event
//! of the very command that arranges a ticket, and the binding hands that ticket to another holder.
//! Every arrangement of either branch issues a ticket, a selected row or a decoy, so it publishes
//! the trigger; the rows are no longer the steps' to decide and both branches stay refused, while
//! every scenario synthesized passes the interpreted target.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const UNTRIGGERED: [(&str, &str); 2] = [
    (
        "system.yaml",
        include_str!("fixtures/binding-no-step-triggers/system.yaml"),
    ),
    (
        "domains/y.yaml",
        include_str!("fixtures/binding-no-step-triggers/domains/y.yaml"),
    ),
];

const PUBLISHED: [(&str, &str); 2] = [
    (
        "system.yaml",
        include_str!("fixtures/binding-no-step-triggers-published/system.yaml"),
    ),
    (
        "domains/y.yaml",
        include_str!("fixtures/binding-no-step-triggers-published/domains/y.yaml"),
    ),
];

const KEPT: &str = "demo.y.Reconcile/outcome/kept";
const DROPPED: &str = "demo.y.Reconcile/outcome/dropped";

fn model(files: [(&str, &str); 2]) -> EssIr {
    let parsed = files.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn commands_of(synthesis: &Synthesis, id: &str) -> Vec<String> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .iter()
        .filter(|step| step["step"] == "execute_command")
        .filter_map(|step| step["command"].as_str().map(str::to_owned))
        .collect()
}

/// Every scenario of the suite run against the interpreted target, and those that did not pass.
fn interpreted(ir: EssIr, synthesis: &Synthesis) -> (Vec<String>, Vec<String>) {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let ran = report
        .scenarios
        .iter()
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    let not_passed = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect();
    (ran, not_passed)
}

#[test]
fn a_binding_no_step_triggers_leaves_the_row_set_arrangeable() {
    let ir = model(UNTRIGGERED);
    let synthesis = synthesize(&ir);

    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    for id in [KEPT, DROPPED] {
        let commands = commands_of(&synthesis, id);
        assert!(
            !commands.iter().any(|command| command == "demo.y.Enrol"),
            "{id} never sends the trigger: {commands:?}"
        );
        assert_eq!(
            commands.last().map(String::as_str),
            Some("demo.y.Reconcile")
        );
    }
    assert!(
        commands_of(&synthesis, KEPT)
            .iter()
            .any(|command| command == "demo.y.Issue"),
        "kept arranges a ticket"
    );

    let (ran, not_passed) = interpreted(ir, &synthesis);
    assert_eq!(not_passed, Vec::<String>::new());
    for id in [KEPT, DROPPED] {
        assert!(ran.iter().any(|scenario| scenario == id), "`{id}` ran");
    }
}

#[test]
fn a_step_that_publishes_the_trigger_leaves_the_selector_changed() {
    let ir = model(PUBLISHED);
    let synthesis = synthesize(&ir);

    // Every arrangement of either branch issues a ticket (a selected row or a decoy), and that step
    // sets off the handover, so the rows are not the steps' to decide and both stay refused, naming
    // the unfollowed step.
    let refused = refusals(&synthesis);
    for id in [KEPT, DROPPED] {
        let branch: Vec<&String> = refused
            .iter()
            .filter(|refusal| refusal.contains(&format!("`{id}`")))
            .collect();
        assert_eq!(branch.len(), 1, "{id}: {refused:#?}");
        assert!(
            branch[0].contains("ESS-SYNTH-001")
                && branch[0]
                    .contains("a step of the scenario changes rows of the selector's entity"),
            "{id}: {refused:#?}"
        );
    }

    let (ran, not_passed) = interpreted(ir, &synthesis);
    assert_eq!(not_passed, Vec::<String>::new());
    assert!(
        ran.iter()
            .any(|scenario| scenario == "hand-over-on-issued/binding/flow"),
        "{ran:?}"
    );
    for id in [KEPT, DROPPED] {
        assert!(!ran.iter().any(|scenario| scenario == id), "{id}: {ran:?}");
    }
}
