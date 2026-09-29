//! Adversary pass 2 against the #218 unit: the transition-to-outcome mapping.
//!
//! A transition is performed by every outcome that moves along it, an `instances:` set outcome
//! (ess/16) as much as a single-subject one. The baseline refuses a set outcome's scenario
//! (`ESS-SYNTH-001` at `<outcome>.instances`), so a transition only set outcomes perform has no
//! scenario in either suite, and a mutant on it is `unwitnessed` naming that refusal, as a
//! transition only baseline-refused single-subject outcomes perform is (decisions #218, pass 1).
#![allow(
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    clippy::needless_raw_string_hashes
)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutantEntry, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn parsed(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

/// `escalate` is performed only by `escalated-all`, an `instances:` set outcome whose scenario
/// the baseline refuses with `ESS-SYNTH-001`; `flag` (a witnessed single-subject outcome) keeps
/// `Escalated` reachable, so a `transition-to` mutant on `escalate` compiles.
const LADDER_SET: &str = r#"format: ess/16
system: desk
version: v1
domain: desk.case
summary: A case opened, parked or closed singly, and escalated in bulk by note.
naming:
  wire: case
  display: Case

entities:
  - name: desk.case.Case
    identity:
      name: case_id
      type: String
    fields:
      - name: note
        type: String
    lifecycle:
      initial: Open
      states: [Open, Parked, Escalated, Closed]
      terminal: [Closed]
      transitions:
        - {name: park, from: [Open], to: Parked}
        - {name: escalate, from: [Open, Parked], to: Escalated}
        - {name: flag, from: [Parked], to: Escalated}
        - {name: close, from: [Open, Parked, Escalated], to: Closed}

errors:
  - name: desk.case.WrongState
    fields: []

commands:
  - name: desk.case.OpenCase
    input:
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: desk.case.Case
        instance: case_id
        sets: {note: input.note}
        emits: [desk.case.CaseOpened]
        payload:
          desk.case.CaseOpened: {case_id: {generated: true}}
  - name: desk.case.Park
    input:
      - {name: id, type: String}
    outcomes:
      - {name: wrong-state, wrong_state: true, error: desk.case.WrongState}
      - name: parked
        moves: desk.case.Case.park
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}
  - name: desk.case.Close
    input:
      - {name: id, type: String}
    outcomes:
      - {name: wrong-state, wrong_state: true, error: desk.case.WrongState}
      - name: closed
        moves: desk.case.Case.close
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}
  - name: desk.case.Flag
    input:
      - {name: id, type: String}
    outcomes:
      - {name: wrong-state, wrong_state: true, error: desk.case.WrongState}
      - name: flagged
        moves: desk.case.Case.flag
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}
  - name: desk.case.EscalateAll
    input:
      - {name: note, type: String}
    outcomes:
      - name: escalated-all
        moves: desk.case.Case.escalate
        instances: {where: note == input.note}
        emits: [desk.case.NotesEscalated]
        payload:
          desk.case.NotesEscalated: {note: input.note, escalated: {count: changed}}

events:
  - name: desk.case.CaseOpened
    fields:
      - {name: case_id, type: String}
  - name: desk.case.CaseReviewed
    fields:
      - {name: case_id, type: String}
  - name: desk.case.NotesEscalated
    fields:
      - {name: note, type: String}
      - {name: escalated, type: Integer}
"#;

#[test]
fn a_mutant_on_a_transition_only_a_set_outcome_performs_is_not_survived() {
    let spec = parsed("ladder-set.yaml", LADDER_SET);
    let (files, texts) = &spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    let report = mutate::audit(
        files,
        texts,
        &[MutantClass::TransitionTo, MutantClass::FromDrop],
        || Interpreted::for_model(ir.clone()),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert!(
        report.baseline.refusals > 0,
        "the baseline refuses the set outcome's scenario: {report:#?}"
    );
    let on_escalate: Vec<&MutantEntry> = report
        .mutants
        .iter()
        .filter(|entry| entry.id.contains("desk.case.Case.escalate"))
        .filter(|entry| entry.verdict != Verdict::Stillborn)
        .collect();
    assert!(
        !on_escalate.is_empty(),
        "a mutant on `escalate` ran: {:#?}",
        report.mutants
    );
    for entry in on_escalate {
        eprintln!(
            "{} {:?} added={:?} baseline={:?}",
            entry.id, entry.verdict, entry.added_refusals, entry.baseline_refusals
        );
        assert_ne!(
            entry.verdict,
            Verdict::Survived,
            "only the set outcome `escalated-all` performs `escalate`, and the baseline refuses its \
             scenario (ESS-SYNTH-001); no scenario of either suite can kill this mutant, so it is \
             not a survivor: {entry:?}"
        );
    }
}
