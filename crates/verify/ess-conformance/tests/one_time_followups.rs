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

/// An adopter's model, made neutral: a creator with a one-time response, an updater whose first
/// branch is an external refusal of the existing row, an actor that may not send the updater, and
/// a `read_your_writes` view of the row. At `ess/26` and later the refusal reads the row back and
/// requires it unchanged (beyond10x/ess#587), which split the read-back across the denied cell.
const EXTERNAL_BEFORE_UPDATE: &str =
    include_str!("fixtures/one-time-followup/external-before-update.yaml");

/// The updater's external refusal, as the fixture declares it.
const REFUSAL_BRANCH: &str = "      - name: store-unavailable\n        external: the store cannot \
                              be written to\n        error: demo.shop.StoreUnavailable\n        \
                              summary: Nothing changed.\n";

/// The last line of the updater's updating branch.
const UPDATE_END: &str = "        summary: The label changes.\n";

/// The denied cell the adopter's synthesis stopped at.
const DENIED: &str =
    "demo.shop.CreateItem/disclosure/created/code/denied/demo.shop.Relabel/as/actor/demo.shop.Clerk";

/// The formats either side of the read-back: the last without it, the first with it, the newest.
const FORMATS: [&str; 3] = ["ess/25", "ess/26", "ess/32"];

/// `text` with `from` replaced by `to`, which must occur in it.
fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture no longer holds {from:?}");
    text.replace(from, to)
}

/// The fixture at `format`, with the updater's two branches in declared order or reversed.
fn external_before_update(format: &str, update_first: bool) -> String {
    let text = edited(
        EXTERNAL_BEFORE_UPDATE,
        "format: ess/26",
        &format!("format: {format}"),
    );
    if update_first {
        let text = edited(&text, REFUSAL_BRANCH, "");
        edited(&text, UPDATE_END, &format!("{UPDATE_END}{REFUSAL_BRANCH}"))
    } else {
        text
    }
}

/// The fixture without one of the four ingredients the adopter found each to be needed, by name.
fn without_ingredient(text: &str, ingredient: &str) -> String {
    match ingredient {
        "one-time response" => edited(text, "        one_time_response: [code]\n", ""),
        "an actor denied the updater" => edited(
            text,
            "may: [demo.shop.CreateItem]",
            "may: [demo.shop.CreateItem, demo.shop.Relabel]",
        ),
        "the external refusal" => edited(text, REFUSAL_BRANCH, ""),
        "a read_your_writes view" => edited(
            text,
            "consistency: read_your_writes",
            "consistency: eventual",
        ),
        other => panic!("no ingredient {other}"),
    }
}

/// Every refusal the synthesis made for a scenario its own admission refused.
fn self_refused(synthesis: &ess_conformance::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|text| text.contains("own admission refuses"))
        .collect()
}

/// The adopter's model, with its updater's branches in either order, synthesizes at `ess/26` and
/// later with the denied cell it stopped at; every scenario is admitted, and the model's own
/// interpreted target passes each. At `ess/25`, before the read-back, it already did.
#[test]
fn a_replay_beside_an_ess26_read_back_is_admitted() {
    let denied: ScenarioId = DENIED.parse().unwrap();
    for format in FORMATS {
        for update_first in [false, true] {
            let ir = model(&external_before_update(format, update_first));
            let synthesis = ess_conformance::synthesize(&ir);
            let case = format!("{format}, update first: {update_first}");
            let scenario = synthesis
                .suite
                .scenario(&denied)
                .unwrap_or_else(|| panic!("{case}: no cell {denied}: {:#?}", synthesis.refusals));
            let tags: Vec<_> = scenario
                .steps
                .iter()
                .map(|step| serde_json::to_value(step).unwrap()["step"].clone())
                .collect();
            let snapshots = tags
                .iter()
                .filter(|tag| *tag == "snapshot_complete_subject")
                .count();
            let comparisons = tags
                .iter()
                .filter(|tag| *tag == "expect_complete_subject_unchanged")
                .count();
            assert_eq!(snapshots, comparisons, "{case}: {tags:?}");
            if let Err(error) = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite) {
                panic!("{case}: {error}");
            }
            assert_eq!(self_refused(&synthesis), Vec::<String>::new(), "{case}");
            assert_all_pass(&ir, &synthesis);
        }
    }
}

/// Synthesis never writes a suite its own admission refuses, whichever ingredient of the adopter's
/// model is left out: a scenario it would refuse is withdrawn and refused, naming the scenario and
/// the step, and the rest of the suite stands.
#[test]
fn a_synthesized_suite_never_fails_its_own_admission() {
    for format in FORMATS {
        for update_first in [false, true] {
            for ingredient in [
                "one-time response",
                "an actor denied the updater",
                "the external refusal",
                "a read_your_writes view",
            ] {
                let text = external_before_update(format, update_first);
                let ir = model(&without_ingredient(&text, ingredient));
                let synthesis = ess_conformance::synthesize(&ir);
                let case = format!("{format}, update first: {update_first}, no {ingredient}");
                if let Err(error) = synthesis.suite.to_canonical_json() {
                    panic!("{case}: {error}");
                }
                assert_eq!(self_refused(&synthesis), Vec::<String>::new(), "{case}");
            }
        }
    }
    // A scenario whose read-back lost its comparison, as the denied cell's did: withdrawn and
    // refused with the step that has no comparison, and the suite then admitted.
    let ir = model(&external_before_update("ess/26", false));
    let mut synthesis = ess_conformance::synthesize(&ir);
    let id: ScenarioId = "demo.shop.Relabel/outcome/store-unavailable"
        .parse()
        .unwrap();
    let scenario = synthesis.suite.scenarios.get_mut(&id).unwrap();
    let snapshot = scenario
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. }))
        .unwrap_or_else(|| panic!("{id} reads no row back: {:#?}", scenario.steps));
    scenario
        .steps
        .retain(|step| !matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. }));
    assert!(synthesis.suite.to_canonical_json().is_err());
    ess_conformance::synthesize::withdraw_inadmissible(&mut synthesis);
    assert!(synthesis.suite.scenario(&id).is_none());
    let refused = self_refused(&synthesis);
    assert_eq!(refused.len(), 1, "{refused:#?}");
    for named in [
        format!("`{id}`"),
        format!(
            "step `/scenarios/{}/steps/{snapshot}` (`snapshot_complete_subject`) with InvalidReplay",
            id.to_string().replace('~', "~0").replace('/', "~1")
        ),
        "InvalidReplay".into(),
        "complete subject snapshot has no comparison".into(),
    ] {
        assert!(refused[0].contains(&named), "{named} in {}", refused[0]);
    }
    assert!(synthesis
        .refusals
        .iter()
        .any(|refusal| refusal.scenario.as_ref() == Some(&id)));
    if let Err(error) = synthesis.suite.to_canonical_json() {
        panic!("{error}");
    }
}
