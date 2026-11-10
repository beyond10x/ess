//! Adversary, pass 2, of beyond10x/ess#732 after correction E: the listed-state refusal now takes
//! its row from the wrong-state witness whenever the command reads the stored row. That witness
//! refutes every stored-guard sibling, also those declared after the refusal, which the one rule
//! (step 3, declaration order) answers only after it.
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
  - name: example.catalogue.IsProse
  - name: example.catalogue.IsHigh
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
OUTCOMES  - name: set
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

const LISTED: &str = "  - name: wrong-state
    when_subject_state: [Rejected, Stale]
    error: example.catalogue.WrongState
    payload:
      example.catalogue.WrongState: {state: {subject: state}}
";

const IS_PROSE: &str = "  - name: is-prose
    when_subject: {predicate: source == Prose}
    error: example.catalogue.IsProse
";

const NOT_PROSE: &str = "  - name: not-prose
    when_subject: {predicate: source != Prose}
    error: example.catalogue.NotSettable
";

const IS_HIGH: &str = "  - name: is-high
    when_subject: {predicate: level == High}
    error: example.catalogue.IsHigh
";

const MIXED: &str = "  - name: not-prose
    when_subject: {predicate: source != Prose}
    when: level == High
    error: example.catalogue.NotSettable
";

fn text(variants: &str, outcomes: &[&str]) -> String {
    MODEL
        .replace("VARIANTS", variants)
        .replace("OUTCOMES", &outcomes.concat())
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalogue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn statuses(target: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn refused_about(synthesis: &Synthesis, needle: &str) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(needle))
        .collect()
}

/// Every `SetLevel` scenario passes the interpreted target, `wrong-state` is synthesized, and no
/// refusal names it.
fn wrong_state_witnessed_and_passes(text: &str) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let mut refused = refused_about(&synthesis, WRONG_STATE);
    refused.extend(refused_about(
        &synthesis,
        "refuses/example.catalogue.SetLevel",
    ));
    assert!(refused.is_empty(), "{refused:#?}");
    let statuses = statuses(model, &synthesis);
    let set_level: BTreeMap<&String, &Status> = statuses
        .iter()
        .filter(|(id, _)| id.contains("SetLevel"))
        .collect();
    assert_eq!(
        statuses.get(WRONG_STATE),
        Some(&Status::Passed),
        "{set_level:#?}"
    );
    assert!(
        set_level.values().all(|status| **status == Status::Passed),
        "{set_level:#?}"
    );
}

/// `wrong-state` is declared first: the one rule answers it in `Rejected` and `Stale` whatever the
/// row holds. The two later siblings cover every row, so no row falsifies both; a witness that
/// insists on falsifying them refuses a branch any row in a listed state witnesses.
#[test]
fn a_later_sibling_pair_no_row_misses_leaves_wrong_state_witnessed() {
    wrong_state_witnessed_and_passes(&text("Prose, Table", &[LISTED, IS_PROSE, NOT_PROSE]));
}

/// The suite tells a target answering `when_subject:` before an earlier-declared
/// `when_subject_state:` apart from the declared order, as the guard-pair corpus's
/// `SubjectBesideState` pins for its explorer (`explore_guards_first.rs`, mode `vip-first`): the
/// mutant declares `is-prose` before `wrong-state`.
#[test]
fn a_target_reading_the_later_subject_guard_first_fails_some_scenario() {
    let correct = text("Prose, Table", &[LISTED, IS_PROSE]);
    let synthesis = synthesize(&ir(&correct));
    let refused = refused_about(&synthesis, "SetLevel");
    assert!(refused.is_empty(), "{refused:#?}");
    let mutant = ir(&text("Prose, Table", &[IS_PROSE, LISTED]));
    let statuses = statuses(mutant, &synthesis);
    assert!(
        statuses.values().any(|status| *status != Status::Passed),
        "the reordered target passes every scenario: {statuses:#?}"
    );
}

/// Two stored-guard siblings declared before the list, each falsified by a different field.
#[test]
fn two_earlier_subject_siblings_are_both_missed() {
    wrong_state_witnessed_and_passes(&text("Table, Prose", &[NOT_PROSE, IS_HIGH, LISTED]));
}

/// An earlier sibling reading the row and the input together: the input may miss it.
#[test]
fn an_earlier_sibling_reading_row_and_input_is_missed() {
    wrong_state_witnessed_and_passes(&text("Table, Prose", &[MIXED, LISTED]));
}

/// Two earlier siblings that together hold on every row: `wrong-state` is never answered, so it is
/// refused with a cause met in a listed state, and nothing synthesized fails a correct target.
#[test]
fn earlier_siblings_no_row_misses_refuse_wrong_state_in_a_listed_state() {
    let text = text("Table, Prose", &[IS_PROSE, NOT_PROSE, LISTED]);
    let model = ir(&text);
    let synthesis = synthesize(&model);
    let refused = refused_about(&synthesis, WRONG_STATE);
    assert_eq!(refused.len(), 1, "{:#?}", synthesis.refusals);
    for state in ["Extracted", "Accepted"] {
        assert!(
            !refused[0].contains(&format!("state {state}")),
            "{}",
            refused[0]
        );
    }
    assert!(
        !synthesis
            .suite
            .scenarios
            .iter()
            .any(|(id, _)| id.to_string() == WRONG_STATE),
        "a scenario for a branch no row reaches"
    );
    let statuses = statuses(model, &synthesis);
    let failed: Vec<&String> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The per-state scenarios tell apart a target whose list drops `Stale` or `Rejected`.
#[test]
fn a_target_narrowing_the_list_fails_some_scenario() {
    let correct = text("Table, Prose", &[NOT_PROSE, LISTED]);
    let synthesis = synthesize(&ir(&correct));
    for narrowed in ["[Rejected]", "[Stale]"] {
        let mutant = ir(&correct.replace(
            "when_subject_state: [Rejected, Stale]",
            &format!("when_subject_state: {narrowed}"),
        ));
        let statuses = statuses(mutant, &synthesis);
        assert!(
            statuses.values().any(|status| *status != Status::Passed),
            "{narrowed}: {statuses:#?}"
        );
    }
}
