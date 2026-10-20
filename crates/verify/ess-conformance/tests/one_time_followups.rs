//! Follow-up cells of a one-time outcome's disclosure inventory: the commands a caller may send
//! after the origin, and the denied sends of a caller not granted them.
//!
//! The ordinary suite builds the follow-up command's branches with the caller each send is made as
//! and with the family each branch is reached by. A disclosure cell sends the same branch after the
//! origin, so it is held to the same reach: a branch the ordinary suite witnesses is not refused in
//! a disclosure cell for a reason the ordinary suite does not have.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{report::Status, ScenarioId, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const CALLER_GUARD: &str = include_str!("fixtures/one-time-followup/caller-guard.yaml");
const UNKNOWN_FIRST: &str = include_str!("fixtures/one-time-followup/unknown-first.yaml");
const WRONG_STATE_FIRST: &str = include_str!("fixtures/one-time-followup/wrong-state-first.yaml");

const ORIGIN: &str = "devices.api.PairDevice/disclosure/paired/pairing_code";

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every refusal of a follow-up cell, `command/…` or `denied/…`, as it prints.
fn followup_refusals(synthesis: &ess_conformance::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal.scenario.as_ref().is_some_and(|id| {
                matches!(id, ScenarioId::Disclosure { .. })
                    && (id.to_string().contains("/command/") || id.to_string().contains("/denied/"))
            })
        })
        .map(ToString::to_string)
        .collect()
}

/// The suite run against the model's own interpreted target, every scenario's status.
fn statuses(ir: &EssIr, synthesis: &ess_conformance::Synthesis) -> Vec<(String, Status)> {
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir.clone());
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn cell(aspect: &str, actor: &str) -> ScenarioId {
    format!("{ORIGIN}/{aspect}/as/actor/devices.api.{actor}")
        .parse()
        .unwrap()
}

/// The branches a scenario expects, in order, of `command`.
fn expected_branches(
    scenario: &ess_conformance::ConformanceScenario,
    command: &str,
) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectOutcome { outcome } if outcome.command.to_string() == command => {
                Some(outcome.outcome.to_string())
            }
            _ => None,
        })
        .collect()
}

/// The guest's denied cell sends `command` as the guest, carrying the guest's own attributes, and
/// expects it not granted.
fn assert_denied(synthesis: &ess_conformance::Synthesis, command: &str) {
    let id = cell(&format!("denied/{command}"), "Guest");
    let scenario = synthesis
        .suite
        .scenario(&id)
        .unwrap_or_else(|| panic!("no cell {id}"));
    let sent: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: named,
                actor,
                caller,
                ..
            } if named.to_string() == command => Some((actor.as_ref()?.to_string(), caller)),
            _ => None,
        })
        .collect();
    assert!(
        sent.iter()
            .any(|(actor, caller)| actor == "devices.api.Guest" && caller.contains_key("user_id")),
        "{id} does not send `{command}` as the guest: {sent:?}"
    );
    assert!(
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. })),
        "{id} expects no refusal of the grant"
    );
}

fn assert_all_pass(ir: &EssIr, synthesis: &ess_conformance::Synthesis) {
    let statuses = statuses(ir, synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:?}");
}

/// A follow-up refusal guarded on the caller who created the row (`owner != caller.user_id`) is
/// reached in a disclosure cell as the ordinary suite reaches it: the row arranged by one caller,
/// the follow-up sent by another.
#[test]
fn a_caller_guarded_followup_is_built_in_its_disclosure_cells() {
    let ir = model(CALLER_GUARD);
    let synthesis = ess_conformance::synthesize(&ir);
    let ordinary: ScenarioId = "devices.api.RenameDevice/outcome/not-owner"
        .parse()
        .unwrap();
    assert!(synthesis.suite.scenarios.contains_key(&ordinary));
    assert_eq!(followup_refusals(&synthesis), Vec::<String>::new());
    for actor in ["Owner", "Support"] {
        for branch in ["not-owner", "renamed"] {
            let id = cell(&format!("command/devices.api.RenameDevice/{branch}"), actor);
            let scenario = synthesis
                .suite
                .scenario(&id)
                .unwrap_or_else(|| panic!("no cell {id}"));
            assert!(
                expected_branches(scenario, "devices.api.RenameDevice").contains(&branch.into()),
                "{id} does not expect `{branch}`"
            );
        }
    }
    assert_denied(&synthesis, "devices.api.RenameDevice");
    assert_all_pass(&ir, &synthesis);
}

/// A follow-up's `unknown_instance:` branch is reached in a disclosure cell as the ordinary suite
/// reaches it, by an identity no record carries, and the denied cell sends a branch the follow-up
/// reaches rather than the first one declared.
#[test]
fn an_unknown_instance_followup_is_built_in_its_disclosure_cells() {
    let ir = model(UNKNOWN_FIRST);
    let synthesis = ess_conformance::synthesize(&ir);
    let ordinary: ScenarioId = "devices.api.RenameDevice/outcome/no-device"
        .parse()
        .unwrap();
    assert!(synthesis.suite.scenarios.contains_key(&ordinary));
    let refused = followup_refusals(&synthesis);
    assert!(
        refused.iter().all(|text| !text.contains("drifted apart")),
        "{refused:#?}"
    );
    assert_eq!(refused, Vec::<String>::new());
    for actor in ["Owner", "Support"] {
        let id = cell("command/devices.api.RenameDevice/no-device", actor);
        let scenario = synthesis
            .suite
            .scenario(&id)
            .unwrap_or_else(|| panic!("no cell {id}"));
        assert_eq!(
            expected_branches(scenario, "devices.api.RenameDevice"),
            ["no-device"]
        );
    }
    assert_denied(&synthesis, "devices.api.RenameDevice");
    assert_all_pass(&ir, &synthesis);
}

/// The wrong-state fixture with a view that observes a device's state, so a refused row can be
/// observed where it stands.
fn with_state_view() -> String {
    format!(
        "{WRONG_STATE_FIRST}views:\n  - name: devices.api.Devices\n    source: devices.api.Device\n    \
         consistency: read_your_writes\n    fields:\n      - {{name: device_id, type: Uuid}}\n      \
         - {{name: state, type: devices.api.Device.State}}\n"
    )
}

/// A follow-up's `wrong_state:` branch declared first: with a view observing the row, its cells
/// are built over a row the arrangement drove out of the states the command moves from.
#[test]
fn a_wrong_state_followup_is_built_on_a_row_in_that_state() {
    let ir = model(&with_state_view());
    let synthesis = ess_conformance::synthesize(&ir);
    assert_eq!(followup_refusals(&synthesis), Vec::<String>::new());
    let id = cell("command/devices.api.RetireDevice/not-active", "Support");
    let scenario = synthesis
        .suite
        .scenario(&id)
        .unwrap_or_else(|| panic!("no cell {id}"));
    // The row is retired first, then refused: two sends of the command, the move and the refusal.
    assert_eq!(
        expected_branches(scenario, "devices.api.RetireDevice")
            .iter()
            .filter(|branch| *branch == "not-active")
            .count(),
        1,
        "{:#?}",
        scenario.steps
    );
    assert!(
        expected_branches(scenario, "devices.api.RetireDevice").contains(&"retired".to_owned()),
        "{id} never drives the row to `Retired`: {:#?}",
        scenario.steps
    );
    assert_denied(&synthesis, "devices.api.RetireDevice");
    assert_all_pass(&ir, &synthesis);
}

/// A follow-up's `wrong_state:` branch declared first, with no view to observe a refused row: its
/// cells send it, as the ordinary suite does, for an identity no record carries, and the denied
/// cell sends a reachable branch.
#[test]
fn a_wrong_state_followup_is_built_in_its_disclosure_cells() {
    let ir = model(WRONG_STATE_FIRST);
    let synthesis = ess_conformance::synthesize(&ir);
    let refused = followup_refusals(&synthesis);
    assert_eq!(refused, Vec::<String>::new());
    for actor in ["Owner", "Support"] {
        for branch in ["not-active", "retired"] {
            let id = cell(&format!("command/devices.api.RetireDevice/{branch}"), actor);
            let scenario = synthesis
                .suite
                .scenario(&id)
                .unwrap_or_else(|| panic!("no cell {id}"));
            assert!(
                expected_branches(scenario, "devices.api.RetireDevice").contains(&branch.into()),
                "{id} does not expect `{branch}`"
            );
        }
    }
    assert_denied(&synthesis, "devices.api.RetireDevice");
    assert_all_pass(&ir, &synthesis);
}
