//! story:a-held-state-guard-is-witnessed-without-a-view-publishing-state: a command with a held-state
//! guard (`when_subject_state:`) has its state-selected branches witnessed where no view publishes
//! the entity's `state`, as the equivalent stored-field predicate is (beyond10x/ess#732).
//!
//! * Without a `state` view every branch is synthesized with no refusal, each scenario whose arranged
//!   state no view publishes carries a `PartialObservation` note naming `state`, and the interpreted
//!   run passes the suite.
//! * With a `state` view the suite carries no such note and observes the state through the view.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const SCENARIOS: [&str; 5] = [
    "example.catalogue.SetLevel/outcome/not-prose",
    "example.catalogue.SetLevel/outcome/set",
    "example.catalogue.SetLevel/outcome/wrong-state",
    "example.catalogue.Item/state/Rejected/refuses/example.catalogue.SetLevel",
    "example.catalogue.Item/state/Stale/refuses/example.catalogue.SetLevel",
];

const MODEL: &str = "format: ess/35
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
views:
  - name: example.catalogue.Items
    source: example.catalogue.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: source, type: example.catalogue.Origin}
      - {name: level, type: example.catalogue.Level}
STATE_FIELD
";

const STATE_VIEW: &str = "      - {name: state, type: example.catalogue.Item.State}\n";

fn synthesized(with_state_view: bool) -> (EssIr, Synthesis) {
    let text = MODEL.replace("STATE_FIELD", if with_state_view { STATE_VIEW } else { "" });
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalogue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let model = compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}\n{text}"));
    let synthesis = synthesize(&model);
    (model, synthesis)
}

/// The refusals about `SetLevel`'s branches and the per-state family it belongs to; the other
/// commands of the model declare no `wrong_state:` and are refused for that, whatever the views.
fn about_set_level(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("SetLevel"))
        .collect()
}

fn noted_state(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::PartialObservation {
                scenario,
                unobserved,
            } if unobserved.iter().any(|field| field == "state") => Some(scenario.to_string()),
            _ => None,
        })
        .collect()
}

fn run(model: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

#[test]
fn every_held_state_branch_is_witnessed_without_a_state_view() {
    let (_, synthesis) = synthesized(false);
    let refusals = about_set_level(&synthesis);
    assert!(refusals.is_empty(), "{refusals:#?}");
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for scenario in SCENARIOS {
        assert!(
            ids.iter().any(|id| id == scenario),
            "no {scenario}: {ids:#?}"
        );
    }
}

#[test]
fn a_scenario_whose_state_no_view_publishes_names_it_as_partially_observed() {
    let (_, synthesis) = synthesized(false);
    let noted = noted_state(&synthesis);
    for scenario in SCENARIOS {
        assert!(
            noted.iter().any(|id| id == scenario),
            "no PartialObservation naming `state` for {scenario}: {noted:#?}"
        );
    }
}

#[test]
fn the_interpreted_run_passes_every_scenario_without_a_state_view() {
    let (model, synthesis) = synthesized(false);
    let statuses = run(model, &synthesis);
    assert!(!statuses.is_empty());
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}

/// The suite and refusals of the specification whose view publishes `state` (`b`), as the base
/// revision `c19171522d` synthesizes them: sha256 of the canonical suite JSON and of the refusals'
/// text, with their lengths. Written by this test run with `HELD_STATE_B_WRITE=<path>` in an export
/// of that revision.
const BASE_B_SUITE: &str =
    "0a1c04947c1b19b43ee6dd4837a5a7d6c44bf3831c05154a7ceef41e5c51e4e9:102624";
const BASE_B_REFUSALS: &str =
    "a5432dc085ca729a9dc8937e45bc6c66512f0babaea139977d556676b9e16952:4853";

fn digest(text: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let hex = Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    format!("{hex}:{}", text.len())
}

#[test]
fn a_state_view_keeps_its_suite_bytes_against_the_base() {
    let (_, synthesis) = synthesized(true);
    let suite = synthesis
        .suite
        .to_canonical_json()
        .expect("a canonical suite");
    let refusals: String = synthesis
        .refusals
        .iter()
        .fold(String::new(), |mut out, r| {
            writeln!(out, "{r}").expect("writing to a String");
            out
        });
    let (suite, refusals) = (digest(&suite), digest(&refusals));
    if let Ok(path) = std::env::var("HELD_STATE_B_WRITE") {
        std::fs::write(path, format!("{suite}\n{refusals}\n")).unwrap();
        return;
    }
    assert_eq!(
        suite, BASE_B_SUITE,
        "the suite of a model with a state view moved"
    );
    assert_eq!(refusals, BASE_B_REFUSALS, "its refusals moved");
}

#[test]
fn a_state_view_keeps_the_observation_through_it() {
    let (model, synthesis) = synthesized(true);
    assert!(
        about_set_level(&synthesis).is_empty(),
        "{:#?}",
        synthesis.refusals
    );
    assert!(noted_state(&synthesis).is_empty(), "{:#?}", synthesis.notes);
    let statuses = run(model, &synthesis);
    assert!(statuses.values().all(|status| *status == Status::Passed));
}
