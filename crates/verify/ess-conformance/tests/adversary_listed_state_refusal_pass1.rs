//! Adversary, pass 1, of beyond10x/ess#732: the listed-state refusal driven from the story's own
//! reproduction rather than the unit's reduced model.
//!
//! The reproduction declares a subject-predicate refusal (`not-prose`, `source != Prose`) before
//! `wrong-state` and reaches `Accepted` by a creation `into: Accepted` as well as by a move. With a
//! view publishing the state (the story's `b`), the acceptance asks for `SetLevel/outcome/wrong-state`
//! arranged in `Rejected` or `Stale`, no refusal, and an interpreted run that passes it. The row the
//! witness arranges must therefore carry `source: Prose`, or the target rightly answers `not-prose`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const WRONG_STATE: &str = "example.catalogue.SetLevel/outcome/wrong-state";

const MODEL: &str = "format: ess/35
system: example
version: v1
domain: example.catalogue
types:
  - {name: example.catalogue.Origin, kind: enum, variants: [VARIANTS]}
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
views:
  - name: example.catalogue.Items
    source: example.catalogue.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: source, type: example.catalogue.Origin}
      - {name: level, type: example.catalogue.Level}
      - {name: state, type: example.catalogue.Item.State}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalogue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn statuses(model: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// Story acceptance, bullet 2, on the story's reproduction with the state published.
fn the_reproduction_synthesizes_and_passes_wrong_state(variants: &str) {
    let text = MODEL.replace("VARIANTS", variants);
    let model = ir(&text);
    let synthesis = synthesize(&model);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("SetLevel"))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    assert!(
        synthesis
            .suite
            .scenarios
            .iter()
            .any(|(id, _)| id.to_string() == WRONG_STATE),
        "no scenario {WRONG_STATE}"
    );
    let statuses = statuses(model, &synthesis);
    let set_level: BTreeMap<&String, &Status> = statuses
        .iter()
        .filter(|(id, _)| id.contains("SetLevel"))
        .collect();
    assert!(
        set_level.values().all(|status| **status == Status::Passed),
        "{set_level:#?}"
    );
    assert_eq!(
        statuses.get(WRONG_STATE),
        Some(&Status::Passed),
        "{set_level:#?}"
    );
}

#[test]
fn reproduction_with_prose_first_passes_wrong_state() {
    the_reproduction_synthesizes_and_passes_wrong_state("Prose, Table");
}

#[test]
fn reproduction_with_prose_not_first_passes_wrong_state() {
    the_reproduction_synthesizes_and_passes_wrong_state("Table, Prose");
}

const NOT_PROSE: &str = "  - name: not-prose
    when_subject: {predicate: source != Prose}
    error: example.catalogue.NotSettable
";

const LISTED: &str = "    when_subject_state: [Rejected, Stale]
";

/// The reproduction without `not-prose`, its state list replaced by `listed`.
fn without_subject_predicate(listed: &str) -> (EssIr, Synthesis) {
    let text = MODEL
        .replace("VARIANTS", "Prose, Table")
        .replace(NOT_PROSE, "")
        .replace(LISTED, listed);
    assert!(
        !text.contains("not-prose") && text.contains(listed),
        "{text}"
    );
    let model = ir(&text);
    let synthesis = synthesize(&model);
    (model, synthesis)
}

fn wrong_state_passes(listed: &str) {
    let (model, synthesis) = without_subject_predicate(listed);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(WRONG_STATE))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    let statuses = statuses(model, &synthesis);
    assert_eq!(
        statuses.get(WRONG_STATE),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
}

#[test]
fn a_one_element_list_reached_by_two_moves_passes() {
    wrong_state_passes("    when_subject_state: [Stale]\n");
}

#[test]
fn a_list_naming_the_initial_state_passes() {
    wrong_state_passes("    when_subject_state: [Extracted, Stale]\n");
}

#[test]
fn a_list_beside_an_input_guard_passes() {
    wrong_state_passes("    when_subject_state: [Rejected, Stale]\n    when: level == High\n");
}
