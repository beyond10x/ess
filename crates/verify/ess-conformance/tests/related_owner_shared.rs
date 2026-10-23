//! A box owns items and an item owns labels (`tests/fixtures/related-owner/`), reported by an
//! adopter (beyond10x/ess#669, beyond10x/ess#670).
//!
//! `LabelItem` reads the box the input names and the item the input names, and refuses with
//! `elsewhere` while the item's `box_id` is not that box. Synthesis created the item's owner box
//! apart from the box the input names, so `elsewhere` answered in `labelled` and `not-a`.
//!
//! `ShakeItem` reads the item's stored box and refuses with `locked` while it is locked. `AddItem`
//! refuses a locked box, so the box has to be opened, the item added and only then the box
//! locked; synthesis locked it first and `AddItem` answered `locked`.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/related-owner/system.yaml");
const BOX: &str = include_str!("fixtures/related-owner/box.yaml");

fn ir() -> EssIr {
    let parsed = [("system.yaml", SYSTEM), ("box.yaml", BOX)].map(|(name, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name}: {error}"));
        (Source::new(name), raw)
    });
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run() -> Run {
    let model = ir();
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

fn assert_passes(run: &Run, wanted: &str) {
    let result = run
        .results
        .get(wanted)
        .unwrap_or_else(|| panic!("{wanted} is synthesized: {:#?}", run.results.keys()));
    if result.status != Status::Passed {
        let scenario = run
            .synthesis
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == wanted)
            .map(|(_, scenario)| serde_json::to_string_pretty(scenario).unwrap());
        panic!(
            "{wanted} passes on the interpreted target:\n{:#?}\n{}",
            result.diagnostics().collect::<Vec<_>>(),
            scenario.unwrap_or_default()
        );
    }
}

/// The scenario's steps, as the suite document writes them.
fn steps(run: &Run, wanted: &str) -> Vec<Value> {
    let scenario = run
        .synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map_or_else(
            || panic!("{wanted} is synthesized"),
            |(_, scenario)| serde_json::to_value(scenario).unwrap(),
        );
    scenario["steps"].as_array().cloned().unwrap_or_default()
}

/// The instance a command's input field names.
fn named(step: &Value, field: &str) -> Option<String> {
    step["input"][field]["instance"].as_str().map(str::to_owned)
}

/// The index of every `execute_command` of `command`.
fn sends(steps: &[Value], command: &str) -> Vec<usize> {
    steps
        .iter()
        .enumerate()
        .filter(|(_, step)| step["step"] == "execute_command" && step["command"] == command)
        .map(|(index, _)| index)
        .collect()
}

/// `LabelItem` addresses the box its item was added to, and that box name is captured once, so
/// no later row of the arrangement rebinds it to a box the item is not in.
fn assert_item_inside_the_named_box(run: &Run, wanted: &str) {
    let steps = steps(run, wanted);
    let act = *sends(&steps, "repro.box.LabelItem")
        .last()
        .unwrap_or_else(|| panic!("{wanted} sends LabelItem"));
    let the_box = named(&steps[act], "box_id").expect("LabelItem names a box");
    let the_item = named(&steps[act], "item_id").expect("LabelItem names an item");
    let captures = |name: &str| -> Vec<usize> {
        steps[..act]
            .iter()
            .enumerate()
            .filter(|(_, step)| step["step"] == "capture_instance" && step["instance"] == name)
            .map(|(index, _)| index)
            .collect()
    };
    assert_eq!(
        captures(&the_box).len(),
        1,
        "{wanted}: `{the_box}` is created once before the act: {steps:#?}"
    );
    let item_captured = *captures(&the_item)
        .first()
        .unwrap_or_else(|| panic!("{wanted}: `{the_item}` is captured: {steps:#?}"));
    let added = sends(&steps[..item_captured], "repro.box.AddItem");
    let added = added
        .last()
        .expect("the item is added before it is captured");
    assert_eq!(
        named(&steps[*added], "box_id").as_deref(),
        Some(the_box.as_str()),
        "{wanted}: the item is added to the box LabelItem names: {steps:#?}"
    );
}

#[test]
fn label_item_labelled_passes() {
    let run = run();
    assert_item_inside_the_named_box(&run, "repro.box.LabelItem/outcome/labelled");
    assert_passes(&run, "repro.box.LabelItem/outcome/labelled");
}

#[test]
fn label_item_not_a_passes() {
    let run = run();
    assert_item_inside_the_named_box(&run, "repro.box.LabelItem/outcome/not-a");
    assert_passes(&run, "repro.box.LabelItem/outcome/not-a");
}

#[test]
fn label_item_elsewhere_still_answers_elsewhere() {
    assert_passes(&run(), "repro.box.LabelItem/outcome/elsewhere");
}

#[test]
fn shake_item_locked_passes() {
    let run = run();
    let wanted = "repro.box.ShakeItem/outcome/locked";
    let steps = steps(&run, wanted);
    let added = sends(&steps, "repro.box.AddItem");
    let locked = sends(&steps, "repro.box.LockBox");
    let act = sends(&steps, "repro.box.ShakeItem");
    let (Some(&added), Some(&locked), Some(&act)) = (added.last(), locked.last(), act.last())
    else {
        panic!("{wanted} adds the item, locks the box and shakes: {steps:#?}");
    };
    assert!(
        added < locked && locked < act,
        "{wanted}: the box is locked after the item is added to it and before ShakeItem: \
         {steps:#?}"
    );
    assert_eq!(
        named(&steps[added], "box_id"),
        named(&steps[locked], "box_id"),
        "{wanted}: the box locked is the box the item was added to: {steps:#?}"
    );
    assert_passes(&run, wanted);
}

#[test]
fn shake_item_no_box_and_shaken_still_pass() {
    let run = run();
    assert_passes(&run, "repro.box.ShakeItem/outcome/shaken");
    assert_passes(&run, "repro.box.ShakeItem/outcome/wrong-state");
    let no_box: Vec<String> = run
        .synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some("repro.box.ShakeItem/outcome/no-box")
        })
        .map(|refusal| refusal.cause.code().to_string())
        .collect();
    assert_eq!(
        no_box,
        ["ESS-SYNTH-003"],
        "`no-box` stays refused by name: every item is created in a box and no box is removed"
    );
}

#[test]
fn every_scenario_of_the_fixture_passes_in_one_run() {
    let run = run();
    assert_eq!(
        run.results.len(),
        17,
        "the fixture synthesizes 17 scenarios: {:#?}",
        run.results.keys()
    );
    let failing: Vec<(&String, Status)> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| (id, result.status))
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}
