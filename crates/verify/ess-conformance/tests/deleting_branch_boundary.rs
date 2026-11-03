//! A boundary row of a deleting branch is sent for a row that exists
//! (`story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists`).
//!
//! `CloseNote(note_id, count)` drops a close whose `count` is above 3 and otherwise deletes the
//! note. The `closed` scenario is further witnessed at the guard's boundary (`count = 3`). The
//! scenario's own act has already deleted the note it opened, so the boundary send needs a note of
//! its own: synthesis arranges a fresh one, as it does for a `moves:` branch, and the interpreted
//! target answers `closed` on it.
//!
//! Every model is inline, so it is no repository model the suite-bytes tables walk.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::synthesize, AdmittedSuite, Runner};
use ess_conformance::{ConformanceSuite, ScenarioId, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/33
system: demo
version: v1
domain: demo.notes
summary: Notes are opened and closed; a close counting above three is dropped.
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
commands:
  - name: demo.notes.OpenNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: opened
        creates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteOpened]
        payload:
          demo.notes.NoteOpened: {note_id: input.note_id}
  - name: demo.notes.CloseNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: count, type: Integer}
    outcomes:
      - name: dropped
        when: 'count > 3'
        accepts: nothing
      - name: closed
        deletes: demo.notes.Note
        instance: note_id
        emits: [demo.notes.NoteClosed]
        payload:
          demo.notes.NoteClosed: {note_id: input.note_id}
events:
  - name: demo.notes.NoteOpened
    fields:
      - {name: note_id, type: demo.notes.NoteId}
  - name: demo.notes.NoteClosed
    fields:
      - {name: note_id, type: demo.notes.NoteId}
";

const CLOSED: &str = "demo.notes.CloseNote/outcome/closed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn statuses(text: &str, suite: &ConformanceSuite) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Each `CloseNote` send of `steps` that requires `closed`: the note it names and its `count`.
fn closing_sends(steps: &[ScenarioStep]) -> Vec<(ScenarioValue, Option<Node>)> {
    steps
        .windows(2)
        .filter_map(|pair| match pair {
            [ScenarioStep::ExecuteCommand { command, input, .. }, ScenarioStep::ExpectOutcome { outcome }]
                if command.to_string() == "demo.notes.CloseNote"
                    && outcome.outcome.as_str() == "closed" =>
            {
                Some((
                    input.get("note_id").cloned().expect("a note is named"),
                    input.get("count").and_then(ScenarioValue::as_literal).cloned(),
                ))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_deleting_branch_is_sent_its_boundary_row_for_a_note_that_exists() {
    let synthesis = synthesize(&ir(MODEL));
    let id = ScenarioId::parse(CLOSED).expect("an id");
    let scenario = synthesis
        .suite
        .scenario(&id)
        .unwrap_or_else(|| panic!("`{CLOSED}` is synthesized: {:#?}", synthesis.refusals));
    let sends = closing_sends(&scenario.steps);
    assert!(
        sends
            .iter()
            .any(|(_, count)| count
                == &Some(Node::Number(ess_primitives::facts::Number::from(3_i64)))),
        "`{CLOSED}` is further witnessed at `count = 3`: {:#?}",
        scenario.steps
    );
    let notes: Vec<&ScenarioValue> = sends.iter().map(|(note, _)| note).collect();
    for (at, note) in notes.iter().enumerate() {
        assert!(
            matches!(note, ScenarioValue::Instance { .. }),
            "every closing send names an opened note: {:#?}",
            scenario.steps
        );
        assert!(
            !notes[..at].contains(note),
            "a closing send names a note an earlier send of the scenario deleted: {:#?}",
            scenario.steps
        );
    }
    let ran = statuses(MODEL, &synthesis.suite);
    assert_eq!(
        ran.get(CLOSED),
        Some(&Status::Passed),
        "the interpreted target passes `{CLOSED}`: {ran:#?}"
    );
    let failed: Vec<&String> = ran
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
