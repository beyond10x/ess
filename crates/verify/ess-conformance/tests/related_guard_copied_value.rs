//! A `{related: …}` value beside a `when_related:` guard over the same row witnesses the success
//! branch (beyond10x/ess#270).
//!
//! `Start` is guarded on the item its `item_id` names — refused where no item carries it, or where
//! the item is archived — and copies that item's `note` into the event it emits and the run it
//! creates. The guard and the copied value read one row, so the scenario arranges it once: the
//! item that selects `started` is created first, between decoys holding other notes, `Start` is sent
//! naming it, and the expectation carries the note that item holds.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceScenario, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-guard-copied-value.yaml");

const STARTED: &str = "mini.m.Start/outcome/started";

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn ir_of(model: &str) -> EssIr {
    let raw = RawSpecFile::parse(model).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("mini.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis() -> Synthesis {
    synthesize(&ir())
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(synthesis)))
}

fn literal(value: Option<&ScenarioValue>) -> Node {
    match value {
        Some(ScenarioValue::Literal { value }) => value.clone(),
        other => panic!("a literal: {other:?}"),
    }
}

/// Every item the scenario adds, by the instance it captures, with the note and the archived flag
/// it was added with, in the order they were created.
fn items(scenario: &ConformanceScenario) -> Vec<(String, Node, Node)> {
    let mut out = Vec::new();
    let mut sent = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.AddItem" =>
            {
                sent = Some((literal(input.get("note")), literal(input.get("archived"))));
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Item" => {
                let (note, archived) = sent.take().expect("a capture follows its AddItem");
                out.push((instance.to_string(), note, archived));
            }
            _ => {}
        }
    }
    out
}

/// The step index and the item instance of the scenario's last `Start`.
fn start(scenario: &ConformanceScenario) -> (usize, String) {
    scenario
        .steps
        .iter()
        .enumerate()
        .rev()
        .find_map(|(at, step)| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.Start" =>
            {
                match input.get("item_id") {
                    Some(ScenarioValue::Instance { instance }) => Some((at, instance.to_string())),
                    other => panic!("Start names an arranged item: {other:?}"),
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("the scenario starts: {scenario:#?}"))
}

/// The value the scenario's expectation of `Started` asserts for `note`.
fn asserted_note(scenario: &ConformanceScenario) -> Option<Node> {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "mini.m.Started" =>
            {
                Some(payload.get("note").cloned())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("`Started` is expected: {scenario:#?}"))
}

/// The success branch is synthesized, and no branch of `Start` is refused for want of an
/// arrangement (ESS-SYNTH-008, or the related-row strategy named in a refusal).
#[test]
fn issue_270_the_success_branch_is_synthesized() {
    let synthesis = synthesis();
    let refused = refusals(&synthesis);
    assert!(
        !refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-008") || refusal.contains("mini.m.Start")),
        "{refused:#?}"
    );
    scenario(&synthesis, STARTED);
}

/// The item `Start` names is created before it, is one that selects `started` (not archived), sits
/// between decoys holding other notes, and the note it holds is the one the expectation asserts.
#[test]
fn issue_270_the_expectation_carries_the_guarded_rows_note() {
    let synthesis = synthesis();
    let scenario = scenario(&synthesis, STARTED);
    let (sent_at, named) = start(scenario);
    let items = items(scenario);
    let (_, note, archived) = items
        .iter()
        .find(|(instance, _, _)| *instance == named)
        .cloned()
        .unwrap_or_else(|| panic!("item {named} is arranged: {scenario:#?}"));
    let captured_at = scenario
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::CaptureInstance { instance, .. }
                if instance.to_string() == named)
        })
        .expect("the named item is captured");
    assert!(captured_at < sent_at, "the item exists before Start");
    assert_eq!(
        archived,
        Node::Bool(false),
        "the named item selects started"
    );
    assert_eq!(asserted_note(scenario), Some(note.clone()), "{scenario:#?}");
    let decoys: Vec<&Node> = items
        .iter()
        .filter(|(instance, _, _)| *instance != named)
        .map(|(_, note, _)| note)
        .collect();
    assert!(!decoys.is_empty(), "decoys are arranged: {scenario:#?}");
    assert!(
        decoys.iter().all(|decoy| **decoy != note),
        "no decoy holds the named item's note: {items:#?}"
    );
    assert_eq!(
        items
            .iter()
            .filter(|(instance, _, _)| *instance != named)
            .count()
            + 1,
        items.len(),
        "the guarded row is arranged once, not again for the copied value: {items:#?}"
    );
}

/// No scenario of the suite fails against the interpreted target, which answers the missing row by
/// its `exists: false` branch and names the related guard it does not evaluate otherwise.
#[test]
fn issue_270_the_interpreted_target_fails_no_scenario() {
    let model = ir();
    let result = synthesize(&model);
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let statuses: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?} {:?}", run.status, run.checks),
            )
        })
        .collect();
    assert!(statuses.contains_key(STARTED), "{statuses:#?}");
    for run in &report.scenarios {
        assert!(
            matches!(run.status, Status::Passed | Status::Unsupported),
            "{}: {statuses:#?}",
            run.scenario
        );
    }
}

/// An item the guard accepts is created before the named one and another after it, the one's note
/// sorting below the named item's and the other's above, so a target copying from "an item the
/// guard accepts" — the first, the last, the least or the greatest — rather than the one named
/// fails; and the suite carries no note saying they could not be arranged (beyond10x/ess#270,
/// corrections 1 and 2).
#[test]
fn issue_270_accepted_items_either_side_hold_notes_either_side() {
    let synthesis = synthesis();
    let scenario = scenario(&synthesis, STARTED);
    let (_, named) = start(scenario);
    let items = items(scenario);
    let at = items
        .iter()
        .position(|(instance, _, _)| *instance == named)
        .expect("the named item is arranged");
    let text = |node: &Node| match node {
        Node::Text(text) => text.clone(),
        other => panic!("a text note: {other:?}"),
    };
    let note = text(&items[at].1);
    let accepted = |range: &[(String, Node, Node)]| -> Vec<String> {
        range
            .iter()
            .filter(|(_, _, archived)| *archived == Node::Bool(false))
            .map(|(_, other, _)| text(other))
            .collect()
    };
    let (before, after) = (accepted(&items[..at]), accepted(&items[at + 1..]));
    assert!(
        before.iter().any(|other| *other < note) && after.iter().any(|other| *other > note),
        "accepted notes below {note:?} before it and above it after it: {items:#?}"
    );
    assert!(
        !synthesis.notes.iter().any(|note| matches!(
            note,
            ess_conformance::synthesize::Note::UnaccompaniedRelatedCopy { .. }
        )),
        "{:#?}",
        synthesis.notes
    );
}

/// Where no second item the guard accepts can hold another note — it accepts only the note `"x"` —
/// the scenario is kept as it was, and a note names the copied field (beyond10x/ess#270).
#[test]
fn issue_270_an_unaccompanied_copy_is_noted() {
    let model = MODEL.replacen(
        "predicate: archived == true}",
        "predicate: note != \"x\"}",
        1,
    );
    assert_ne!(model, MODEL, "the guard is in the model");
    let synthesis = synthesize(&ir_of(&model));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    scenario(&synthesis, STARTED);
    let noted: Vec<String> = synthesis.notes.iter().map(ToString::to_string).collect();
    assert!(
        synthesis.notes.iter().any(|note| matches!(
            note,
            ess_conformance::synthesize::Note::UnaccompaniedRelatedCopy { scenario, fields }
                if scenario.to_string() == STARTED && *fields == ["note".to_owned()]
        )),
        "{noted:#?}"
    );
}
