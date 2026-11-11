//! A refusal guarded by a list of held states, `when_subject_state: [Rejected, Stale]` on an
//! `error:` branch, is witnessed in the states it lists and no others (beyond10x/ess#732).
//!
//! The lifecycle's states sort `Accepted` before `Rejected`, so a witness that tried every state
//! kept the refusal of the first, a state the list does not name, and hid the real cause. The
//! default (`otherwise:`) state refusal is a different family and still tries every state.
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
entities:
  - name: example.catalogue.Item
    identity: {name: id, type: String}
    fields:
      - {name: level, type: String}
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
  - name: example.catalogue.NotFound
commands:
- name: example.catalogue.Extract
  input:
  - {name: id, type: String}
  - {name: level, type: String}
  outcomes:
  - name: extracted
    creates: example.catalogue.Item
    instance: id
    sets: {level: input.level}
    emits: [example.catalogue.Extracted]
    payload:
      example.catalogue.Extracted: {id: input.id}
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
  - {name: level, type: String}
  outcomes:
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
      - {name: level, type: String}
";

fn model(view_state: &str) -> String {
    format!("{MODEL}{view_state}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalogue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn synthesized(view_state: &str) -> (EssIr, Synthesis) {
    let model = ir(&model(view_state));
    let synthesis = synthesize(&model);
    (model, synthesis)
}

fn about_wrong_state(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.to_string().contains(WRONG_STATE))
        .map(ToString::to_string)
        .collect()
}

#[test]
fn a_listed_state_refusal_is_never_reported_against_a_state_it_does_not_list() {
    let (_, synthesis) = synthesized("");
    let refusals = about_wrong_state(&synthesis);
    for refusal in &refusals {
        for unlisted in ["Extracted", "Accepted"] {
            assert!(
                !refusal.contains(&format!("held state {unlisted}")),
                "{refusal}"
            );
        }
    }
    // Where no view publishes the state nothing is refused, so the loop above has nothing to
    // check: the scenarios the refusals used to stand for must be there instead.
    assert!(refusals.is_empty(), "{refusals:#?}");
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .iter()
        .map(|(id, _)| id.to_string())
        .collect();
    for scenario in [
        WRONG_STATE,
        "example.catalogue.Item/state/Rejected/refuses/example.catalogue.SetLevel",
        "example.catalogue.Item/state/Stale/refuses/example.catalogue.SetLevel",
    ] {
        assert!(
            ids.iter().any(|id| id == scenario),
            "no {scenario}: {ids:#?}"
        );
    }
}

#[test]
fn a_listed_state_refusal_is_witnessed_where_no_view_publishes_the_state() {
    // No view publishes `state`: the held state is arranged through the commands and noted as
    // unobserved, not refused (story:a-held-state-guard-is-witnessed-without-a-view-publishing-state).
    let (_, synthesis) = synthesized("");
    assert!(
        about_wrong_state(&synthesis).is_empty(),
        "{:#?}",
        synthesis.refusals
    );
    assert!(
        synthesis
            .suite
            .scenarios
            .iter()
            .any(|(id, _)| id.to_string() == WRONG_STATE),
        "no scenario {WRONG_STATE}"
    );
}

#[test]
fn a_view_publishing_the_state_arranges_the_refusal_in_a_listed_state() {
    let (model, synthesis) =
        synthesized("      - {name: state, type: example.catalogue.Item.State}\n");
    assert!(
        about_wrong_state(&synthesis).is_empty(),
        "{:#?}",
        synthesis.refusals
    );
    let (_, scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == WRONG_STATE)
        .unwrap_or_else(|| panic!("no scenario {WRONG_STATE}"));
    let text = format!("{:?}", scenario.steps);
    assert!(
        text.contains("Rejected") || text.contains("Stale") || text.contains("Reject"),
        "{text}"
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(
        statuses.get(WRONG_STATE),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
}
