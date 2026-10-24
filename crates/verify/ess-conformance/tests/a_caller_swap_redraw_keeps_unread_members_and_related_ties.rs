//! A caller-swap redraw keeps unread members and related ties (beyond10x/ess#672, adopter reports
//! N24 and N25; `story:a-caller-swap-redraw-keeps-unread-members-and-related-ties`).
//!
//! The caller-swapped half of a synthesized scenario redraws every input a guard of another command
//! compares with stored rows (beyond10x/ess#580). Three models, one domain:
//!
//! - `caller-swap-unread-members.yaml` (N24): `DeleteTag` reads every book's stored `tags` through
//!   a row-set guard, so `CreateBook`'s `settings.tags`, written there, is redrawn. `settings.cover`
//!   is read by no such guard and `ShelveBook`'s `no-cover` branch reads it from the stored row, so
//!   the swapped half sends the cover the first half sent.
//! - `caller-swap-related-tie.yaml` (N25): `StartReview`'s `same-version` branch ties
//!   `input.version` to the stored `version` of the book `input.book_id` names. That book is the
//!   run's own, so the swapped half sends the same version the first half sent.
//! - `caller-swap-command-under-test.yaml`: `ShelveBook`'s `untagged` branch reads the stored
//!   `tags` the redraw writes. A fresh list would move the swapped `ShelveBook` to another branch,
//!   so that redraw is refused and the scenario keeps its first run, named by a note.
//!
//! The interpreter implements each model as specified, so every scenario passes on it.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{
    report::Status, AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep,
    ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const UNREAD_MEMBERS: &str = include_str!("fixtures/caller-swap-unread-members.yaml");
const RELATED_TIE: &str = include_str!("fixtures/caller-swap-related-tie.yaml");
const COMMAND_UNDER_TEST: &str = include_str!("fixtures/caller-swap-command-under-test.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(suite: &ConformanceSuite, ir: &EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Every scenario of `synthesis` passes on the interpreted target, and there are `count` of them.
fn all_pass(synthesis: &Synthesis, ir: &EssIr, count: usize) {
    let verdicts = statuses(&synthesis.suite, ir);
    let failed: Vec<_> = verdicts
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
    assert_eq!(verdicts.len(), count, "{:#?}", verdicts.keys());
}

fn scenario<'s>(synthesis: &'s Synthesis, id: &str) -> &'s ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .iter()
        .find_map(|(name, scenario)| (name.to_string() == id).then_some(scenario))
        .unwrap_or_else(|| {
            panic!(
                "{id} is synthesized: {:#?}",
                synthesis.suite.scenarios.keys()
            )
        })
}

/// The literal inputs of every send of `command` in `steps`, in order.
fn sends(steps: &[ScenarioStep], command: &str) -> Vec<BTreeMap<String, Node>> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(
                input
                    .iter()
                    .filter_map(|(name, value)| match value {
                        ScenarioValue::Literal { value } => Some((name.clone(), value.clone())),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .collect()
}

/// The member `name` of the struct literal `input.settings`.
fn member<'n>(input: &'n BTreeMap<String, Node>, name: &str) -> Option<&'n Node> {
    match input.get("settings") {
        Some(Node::Map(members)) => members.get(name),
        _ => None,
    }
}

/// Every `CreateBook` send of `scenario`'s first half beside the same send of its swapped half.
fn halves(scenario: &ConformanceScenario) -> Vec<(BTreeMap<String, Node>, BTreeMap<String, Node>)> {
    let sent = sends(&scenario.steps, "demo.notes.CreateBook");
    assert!(
        sent.len() % 2 == 0 && !sent.is_empty(),
        "both halves send CreateBook as often: {sent:#?}"
    );
    let (first, again) = sent.split_at(sent.len() / 2);
    first.iter().cloned().zip(again.iter().cloned()).collect()
}

#[test]
fn a_swapped_redraw_keeps_unread_struct_members() {
    let ir = ir(UNREAD_MEMBERS);
    let synthesis = synthesize(&ir);
    let shelved = scenario(&synthesis, "demo.notes.ShelveBook/outcome/shelved");
    let mut covered = 0;
    for (first, again) in halves(shelved) {
        assert_eq!(
            member(&first, "cover"),
            member(&again, "cover"),
            "the swapped CreateBook keeps settings.cover: {first:?} then {again:?}"
        );
        covered += usize::from(member(&first, "cover").is_some());
    }
    assert!(covered > 0, "the first half arranges a book with a cover");
    all_pass(&synthesis, &ir, 6);
}

#[test]
fn a_swapped_redraw_redraws_only_the_read_member() {
    let ir = ir(UNREAD_MEMBERS);
    let synthesis = synthesize(&ir);
    let mut pairs = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        if sends(&scenario.steps, "demo.notes.CreateBook").len() < 2 {
            continue;
        }
        for (first, again) in halves(scenario) {
            pairs += 1;
            let (Some(Node::Seq(before)), Some(Node::Seq(after))) =
                (member(&first, "tags"), member(&again, "tags"))
            else {
                panic!("{id}: both halves send settings.tags: {first:?} then {again:?}");
            };
            assert!(
                before.iter().all(|tag| !after.contains(tag)),
                "{id}: settings.tags is fresh in the swapped half: {before:?} then {after:?}"
            );
            assert_eq!(
                member(&first, "cover"),
                member(&again, "cover"),
                "{id}: settings.cover equals the first half's"
            );
        }
    }
    assert!(pairs > 0, "some scenario runs a swapped CreateBook");
}

#[test]
fn a_related_predicate_tie_survives_the_swap() {
    let ir = ir(RELATED_TIE);
    let synthesis = synthesize(&ir);
    let same = scenario(&synthesis, "demo.notes.StartReview/outcome/same-version");
    let versions: Vec<Option<Node>> = sends(&same.steps, "demo.notes.StartReview")
        .into_iter()
        .map(|input| input.get("version").cloned())
        .collect();
    assert_eq!(
        versions.len(),
        2,
        "both halves send StartReview: {versions:?}"
    );
    let stored = Node::Number(ess_primitives::facts::Number::new(1.0).expect("a number"));
    for version in &versions {
        assert_eq!(
            version.as_ref(),
            Some(&stored),
            "StartReview sends the arranged book's stored version in both halves: {versions:?}"
        );
    }
    all_pass(&synthesis, &ir, 10);
}

#[test]
fn keeps_branch_checks_the_command_under_test() {
    let ir = ir(COMMAND_UNDER_TEST);
    let synthesis = synthesize(&ir);
    let untagged = "demo.notes.ShelveBook/outcome/untagged";
    let shelves = sends(
        &scenario(&synthesis, untagged).steps,
        "demo.notes.ShelveBook",
    );
    assert_eq!(
        shelves.len(),
        1,
        "a redraw moving ShelveBook off `untagged` is refused: the scenario keeps its first run"
    );
    let noted: Vec<(String, String)> = synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnswappedCallers {
                scenario, input, ..
            } => Some((scenario.to_string(), input.clone())),
            Note::CrossCallerUnswapped { scenario, reason } => {
                Some((scenario.to_string(), (*reason).to_owned()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        noted,
        vec![(
            untagged.to_owned(),
            "the follow-up arrangement exhausted its fresh identity witnesses".to_owned()
        )],
        "the refused swap is named, and only it"
    );
    let kept = scenario(&synthesis, "demo.notes.ShelveBook/outcome/shelved");
    assert_eq!(
        sends(&kept.steps, "demo.notes.ShelveBook").len(),
        2,
        "a redraw keeping the branch still runs the swapped half"
    );
    all_pass(&synthesis, &ir, 7);
}
