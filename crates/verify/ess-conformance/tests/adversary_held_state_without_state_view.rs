//! Adversary pass 1 on story:a-held-state-guard-is-witnessed-without-a-view-publishing-state
//! (beyond10x/ess#732): what the new held-state scenarios still distinguish once no view publishes
//! `state`, and what the preservation fallback names as unobserved.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const BODY: &str = "format: ess/35
system: example
version: v1
domain: example.catalogue
types:
  - {name: example.catalogue.Origin, kind: enum, variants: [Prose, Scraped]}
  - {name: example.catalogue.Level, kind: enum, variants: [Low, High]}
entities:
  - name: example.catalogue.Item
    identity: {name: id, type: String}
    fields:
      - {name: source, type: example.catalogue.Origin}
      - {name: level, type: example.catalogue.Level}
    lifecycle:
      initial: Extracted
      states: [Extracted, Accepted, Rejected, Stale]
      terminal: [Rejected, Stale]
      transitions:
        - {name: accept, from: [Extracted], to: Accepted}
        - {name: reject, from: [Extracted], to: Rejected}
        - {name: age, from: [Accepted], to: Stale}
events:
  - name: example.catalogue.Extracted
    fields: [{name: id, type: String}]
  - name: example.catalogue.Imported
    fields: [{name: id, type: String}]
  - name: example.catalogue.Accepted
    fields: [{name: id, type: String}]
  - name: example.catalogue.Rejected
    fields: [{name: id, type: String}]
  - name: example.catalogue.Aged
    fields: [{name: id, type: String}]
  - name: example.catalogue.LevelSet
    fields: [{name: id, type: String}]
errors:
  - name: example.catalogue.WrongState
    fields: [{name: state, type: example.catalogue.Item.State}]
  - name: example.catalogue.NotSettable
  - name: example.catalogue.NotFound
commands:
- name: example.catalogue.Extract
  input:
  - {name: id, type: String}
  - {name: source, type: example.catalogue.Origin}
  - {name: level, type: example.catalogue.Level}
  outcomes:
  - name: extracted
    creates: example.catalogue.Item
    instance: id
    sets: {source: input.source, level: input.level}
    emits: [example.catalogue.Extracted]
    payload:
      example.catalogue.Extracted: {id: input.id}
- name: example.catalogue.Import
  input:
  - {name: id, type: String}
  - {name: source, type: example.catalogue.Origin}
  - {name: level, type: example.catalogue.Level}
  outcomes:
  - name: imported
    creates: example.catalogue.Item
    into: Accepted
    instance: id
    sets: {source: input.source, level: input.level}
    emits: [example.catalogue.Imported]
    payload:
      example.catalogue.Imported: {id: input.id}
- name: example.catalogue.Accept
  input:
  - {name: id, type: String}
  outcomes:
  - name: accepted
    moves: example.catalogue.Item.accept
    instance: id
    emits: [example.catalogue.Accepted]
    payload:
      example.catalogue.Accepted: {id: input.id}
- name: example.catalogue.Reject
  input:
  - {name: id, type: String}
  outcomes:
  - name: rejected
    moves: example.catalogue.Item.reject
    instance: id
    emits: [example.catalogue.Rejected]
    payload:
      example.catalogue.Rejected: {id: input.id}
- name: example.catalogue.Age
  input:
  - {name: id, type: String}
  outcomes:
  - name: aged
    moves: example.catalogue.Item.age
    instance: id
    emits: [example.catalogue.Aged]
    payload:
      example.catalogue.Aged: {id: input.id}
- name: example.catalogue.SetLevel
  input:
  - {name: id, type: String}
  - {name: level, type: example.catalogue.Level}
  outcomes:
  - name: not-prose
    when_subject: {predicate: source != Prose}
    error: example.catalogue.NotSettable
  - name: wrong-state
    when_subject_state: [Rejected, Stale]
    error: example.catalogue.WrongState
    payload:
      example.catalogue.WrongState: {state: {subject: state}}
  - name: set
    updates: example.catalogue.Item
    instance: id
    sets: {level: input.level}
    emits: [example.catalogue.LevelSet]
    payload:
      example.catalogue.LevelSet: {id: input.id}
  - name: unknown
    unknown_instance: true
    error: example.catalogue.NotFound
";

/// Specification `a` of the story: the only view publishes `id`, `source` and `level`.
const VIEWS_A: &str = "views:
  - name: example.catalogue.Items
    source: example.catalogue.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: source, type: example.catalogue.Origin}
      - {name: level, type: example.catalogue.Level}
";

/// Specification `b`: the same view also publishes `state`.
const VIEWS_B: &str = "views:
  - name: example.catalogue.Items
    source: example.catalogue.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: source, type: example.catalogue.Origin}
      - {name: level, type: example.catalogue.Level}
      - {name: state, type: example.catalogue.Item.State}
";

/// The view publishes `id` and `source`, neither `level` nor `state`.
const VIEWS_NO_LEVEL: &str = "views:
  - name: example.catalogue.Items
    source: example.catalogue.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: source, type: example.catalogue.Origin}
";

const RESTING: [&str; 3] = [
    "example.catalogue.SetLevel/outcome/wrong-state",
    "example.catalogue.Item/state/Rejected/refuses/example.catalogue.SetLevel",
    "example.catalogue.Item/state/Stale/refuses/example.catalogue.SetLevel",
];

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalogue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

/// The target's model with `reject` landing in `Stale` and `age` in `Rejected`: both still
/// states the `wrong-state` branch lists, so only the held state itself tells them apart.
fn swapped(views: &str) -> EssIr {
    let text = format!("{BODY}{views}")
        .replace(
            "{name: reject, from: [Extracted], to: Rejected}",
            "{name: reject, from: [Extracted], to: Stale}",
        )
        .replace(
            "{name: age, from: [Accepted], to: Stale}",
            "{name: age, from: [Accepted], to: Rejected}",
        );
    assert!(text.contains("to: Stale}") && text.contains("[Accepted], to: Rejected}"));
    ir(&text)
}

fn run(target: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> String {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .map_or_else(
            || format!("no scenario {id}"),
            |(_, s)| format!("{:#?}", s.steps),
        )
}

fn noted(synthesis: &Synthesis, id: &str) -> Vec<String> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::PartialObservation {
                scenario,
                unobserved,
            } if scenario.to_string() == id => Some(unobserved.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// The per-state family `Item/state/Rejected/refuses/SetLevel` claims the refusal in `Rejected`.
/// The `WrongState` error carries `{subject: state}`, so the refusal itself is evidence of the
/// held state: a target whose `Reject` lands the row in `Stale` answers `state: Stale`.
#[test]
fn a_target_landing_the_row_in_the_other_listed_state_fails_without_a_state_view() {
    let synthesis = synthesize(&ir(&format!("{BODY}{VIEWS_A}")));
    let statuses = run(swapped(VIEWS_A), &synthesis);
    let rejected = RESTING[1];
    assert!(
        statuses.get(rejected).is_some_and(|s| *s != Status::Passed),
        "{rejected} passes a target that holds the row in Stale: {statuses:#?}\n{}",
        steps_of(&synthesis, rejected)
    );
}

/// The same mutant against `b`, where the view reads the held state back: caught.
#[test]
fn a_target_landing_the_row_in_the_other_listed_state_fails_with_a_state_view() {
    let synthesis = synthesize(&ir(&format!("{BODY}{VIEWS_B}")));
    let statuses = run(swapped(VIEWS_B), &synthesis);
    assert!(
        statuses
            .get(RESTING[1])
            .is_some_and(|s| *s != Status::Passed),
        "{statuses:#?}"
    );
}

/// The preservation fallback names every field it does not observe, not only `state`.
#[test]
fn a_resting_branch_names_each_field_no_view_publishes() {
    let synthesis = synthesize(&ir(&format!("{BODY}{VIEWS_NO_LEVEL}")));
    for id in RESTING {
        let unobserved = noted(&synthesis, id);
        assert!(
            ["level", "state"]
                .iter()
                .all(|field| unobserved.iter().any(|u| u == field)),
            "{id} notes {unobserved:?}: {:#?}\n{}",
            synthesis.notes,
            steps_of(&synthesis, id)
        );
    }
    let statuses = run(ir(&format!("{BODY}{VIEWS_NO_LEVEL}")), &synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, s)| **s != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}

/// No view of the entity at all: each resting branch observes nothing, so each names `source`,
/// `level` and `state`.
#[test]
fn a_resting_branch_with_no_view_names_every_field() {
    let synthesis = synthesize(&ir(BODY));
    for id in RESTING {
        let unobserved = noted(&synthesis, id);
        assert!(
            ["level", "source", "state"]
                .iter()
                .all(|field| unobserved.iter().any(|u| u == field)),
            "{id} notes {unobserved:?}: {:#?}\n{:#?}\n{}",
            synthesis.notes,
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            steps_of(&synthesis, id)
        );
    }
}

/// The two re-pinned rows: the interpreted target passes every scenario they gained.
#[test]
fn the_repinned_clock_fixtures_pass_the_interpreted_target() {
    for (name, text) in [
        (
            "occurrence-clock-claims",
            include_str!("fixtures/occurrence-clock-claims.yaml"),
        ),
        (
            "occurrence-clock-offers",
            include_str!("fixtures/occurrence-clock-offers.yaml"),
        ),
    ] {
        let model = ir(text);
        let synthesis = synthesize(&model);
        let statuses = run(model, &synthesis);
        let failing: Vec<_> = statuses
            .iter()
            .filter(|(_, s)| matches!(**s, Status::Failed | Status::Error))
            .collect();
        assert!(failing.is_empty(), "{name}: {failing:#?}\n{statuses:#?}");
    }
}
