//! Regression for the caller-swap redraw of beyond10x/ess#672: a `when_related:` test that ties a
//! command's input to a stored field (`version == input.expected`) keeps that tie in the
//! caller-swapped half when the stored field is itself written from another command's input that a
//! row-set guard reads. That input is drawn afresh in the swapped half, so the tied input must be
//! drawn with it rather than kept, or the swapped run answers another branch. The nested-member
//! and determinism cases hold the rest of #672 alongside: only guarded struct members are redrawn.

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

fn replaced(text: &str, edits: &[(&str, &str)]) -> String {
    let mut text = text.to_owned();
    for (from, to) in edits {
        assert_eq!(
            text.matches(from).count(),
            1,
            "the edit's anchor occurs once: {from:?}"
        );
        text = text.replacen(from, to, 1);
    }
    text
}

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

fn failed(synthesis: &Synthesis, ir: &EssIr) -> Vec<(String, Status)> {
    statuses(&synthesis.suite, ir)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect()
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

fn at<'n>(input: &'n BTreeMap<String, Node>, path: &[&str]) -> Option<&'n Node> {
    let (root, rest) = path.split_first()?;
    rest.iter()
        .try_fold(input.get(*root)?, |node, name| match node {
            Node::Map(members) => members.get(*name),
            _ => None,
        })
}

fn unswapped(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnswappedCallers { scenario, .. }
            | Note::CrossCallerUnswapped { scenario, .. } => Some(scenario.to_string()),
            _ => None,
        })
        .collect()
}

/// `caller-swap-related-tie.yaml` with the book's stored `version` written from `CreateBook`'s
/// input, a row-set guard (`PinVersion`) reading that stored field, and `StartReview`'s tied input
/// named `expected` rather than `version`. The `same-version` tie is the N25 tie, the row the guard
/// names is still the run's own, and only the input's name and the stored value's source differ.
fn tie_to_input_written_field() -> String {
    replaced(
        RELATED_TIE,
        &[
            (
                "      - {name: book_id, type: demo.notes.BookId}\n      - {name: settings, type: demo.notes.BookSettings}\n",
                "      - {name: book_id, type: demo.notes.BookId}\n      - {name: settings, type: demo.notes.BookSettings}\n      - {name: version, type: Integer}\n",
            ),
            ("          version: '1'\n", "          version: input.version\n"),
            (
                "      - {name: version, type: Integer}\n    outcomes:\n      - name: no-book",
                "      - {name: expected, type: Integer}\n    outcomes:\n      - name: no-book",
            ),
            (
                "predicate: version == input.version",
                "predicate: version == input.expected",
            ),
            (
                "          book_id: input.book_id\n          version: input.version\n",
                "          book_id: input.book_id\n          version: input.expected\n",
            ),
            (
                "      - demo.notes.StartReview\n",
                "      - demo.notes.StartReview\n      - demo.notes.PinVersion\n",
            ),
            (
                "errors:\n",
                "errors:\n  - name: demo.notes.VersionTaken\n    fields: []\n",
            ),
            (
                "commands:\n",
                "commands:
  - name: demo.notes.PinVersion
    input:
      - {name: tag_id, type: demo.notes.TagId}
      - {name: version, type: Integer}
    outcomes:
      - name: taken
        when_related: {entity: demo.notes.Book, where: version == input.version, exists: true}
        error: demo.notes.VersionTaken
      - name: pinned
        creates: demo.notes.Tag
        instance: tag_id
        emits: [demo.notes.TagCreated]
        payload:
          demo.notes.TagCreated: {tag_id: input.tag_id}
        sets:
          owner_id: {caller: owner_id}
",
            ),
        ],
    )
}

/// Every `StartReview` of `steps` beside the stored version of the book it names, as the
/// `CreateBook` that created that book sent it.
fn ties(steps: &[ScenarioStep], tied: &str) -> Vec<(Option<Node>, Option<Node>)> {
    let mut by_literal: BTreeMap<Node, Option<Node>> = BTreeMap::new();
    let mut by_instance: BTreeMap<String, Option<Node>> = BTreeMap::new();
    let mut last_book: Option<Option<Node>> = None;
    let mut found = Vec::new();
    for step in steps {
        match step {
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.notes.Book" => {
                if let Some(version) = last_book.clone() {
                    by_instance.insert(instance.to_string(), version);
                }
            }
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                let name = command.to_string();
                let literals = sends(std::slice::from_ref(step), &name)
                    .pop()
                    .unwrap_or_default();
                if name == "demo.notes.CreateBook" {
                    let version = literals.get("version").cloned();
                    if let Some(book) = literals.get("book_id") {
                        by_literal.insert(book.clone(), version.clone());
                    }
                    last_book = Some(version);
                } else if name == "demo.notes.StartReview" {
                    let version = match input.get("book_id") {
                        Some(ScenarioValue::Literal { value }) => by_literal.get(value).cloned(),
                        Some(ScenarioValue::Instance { instance }) => {
                            by_instance.get(&instance.to_string()).cloned()
                        }
                        _ => None,
                    }
                    .unwrap_or_else(|| {
                        panic!("the run creates the book StartReview names: {input:?}")
                    });
                    found.push((version, literals.get(tied).cloned()));
                }
            }
            _ => {}
        }
    }
    found
}

/// The N25 tie, where the stored side is written from another command's input that a row-set
/// guard reads: that input is drawn afresh in the swapped half, and the tied `StartReview` input
/// must follow it, or the swapped `StartReview` answers `started`.
#[test]
fn a_tie_to_a_stored_field_written_from_redrawn_input_survives_the_swap() {
    let ir = ir(&tie_to_input_written_field());
    let synthesis = synthesize(&ir);
    let id = "demo.notes.StartReview/outcome/same-version";
    let same = scenario(&synthesis, id);
    let pairs = ties(&same.steps, "expected");
    assert_eq!(
        pairs.len(),
        2,
        "both halves send StartReview (notes: {:?}): {pairs:?}",
        unswapped(&synthesis)
    );
    for (stored, tied) in &pairs {
        assert_eq!(
            stored, tied,
            "StartReview sends the version of the book it names in each half: {pairs:?}"
        );
    }
    let verdicts = statuses(&synthesis.suite, &ir);
    assert_eq!(verdicts.get(id), Some(&Status::Passed), "{verdicts:#?}");
}

/// The same model, judged only by the interpreter, which implements it as specified: the
/// synthesized `same-version` scenario must pass on it.
#[test]
fn the_same_version_scenario_passes_on_the_interpreter() {
    let ir = ir(&tie_to_input_written_field());
    let synthesis = synthesize(&ir);
    let failed = failed(&synthesis, &ir);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// `caller-swap-unread-members.yaml` with the guarded `tags` one struct deeper:
/// `settings.shelf.tags`, beside an unread `settings.shelf.label` and `settings.cover`.
fn nested_members() -> String {
    replaced(
        UNREAD_MEMBERS,
        &[
            (
                "  - name: demo.notes.BookSettings
    kind: struct
    fields:
      - {name: cover, type: Optional<String>}
      - {name: tags, type: List<demo.notes.TagId>}
",
                "  - name: demo.notes.ShelfSettings
    kind: struct
    fields:
      - {name: label, type: Optional<String>}
      - {name: tags, type: List<demo.notes.TagId>}
  - name: demo.notes.BookSettings
    kind: struct
    fields:
      - {name: cover, type: Optional<String>}
      - {name: shelf, type: demo.notes.ShelfSettings}
",
            ),
            (
                "tags: input.settings.tags",
                "tags: input.settings.shelf.tags",
            ),
        ],
    )
}

#[test]
fn a_nested_guarded_member_is_redrawn_alone() {
    let ir = ir(&nested_members());
    let synthesis = synthesize(&ir);
    let mut pairs = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        let sent = sends(&scenario.steps, "demo.notes.CreateBook");
        if sent.len() < 2 {
            continue;
        }
        let (first, again) = sent.split_at(sent.len() / 2);
        for (first, again) in first.iter().zip(again) {
            pairs += 1;
            for path in [
                &["settings", "cover"][..],
                &["settings", "shelf", "label"][..],
            ] {
                assert_eq!(
                    at(first, path),
                    at(again, path),
                    "{id}: {path:?} is kept: {first:?} then {again:?}"
                );
            }
            let (Some(Node::Seq(before)), Some(Node::Seq(after))) = (
                at(first, &["settings", "shelf", "tags"]),
                at(again, &["settings", "shelf", "tags"]),
            ) else {
                panic!("{id}: both halves send settings.shelf.tags: {first:?} then {again:?}");
            };
            assert!(
                before.iter().all(|tag| !after.contains(tag)),
                "{id}: settings.shelf.tags is fresh: {before:?} then {after:?}"
            );
        }
    }
    assert!(pairs > 0, "some scenario runs a swapped CreateBook");
    let failed = failed(&synthesis, &ir);
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn the_member_redraw_is_deterministic() {
    for text in [nested_members(), tie_to_input_written_field()] {
        let ir = ir(&text);
        assert_eq!(synthesize(&ir), synthesize(&ir));
    }
}
