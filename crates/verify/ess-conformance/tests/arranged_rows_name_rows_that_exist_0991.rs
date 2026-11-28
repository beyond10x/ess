//! Hotfix 0.99.11: a step that arranges a row through a command reading a related row
//! (`when_related: {via: input.<field>, exists: false}`) is sent naming a row that exists, so that
//! step's own existence check does not refuse it. Every scenario synthesized must pass on the
//! interpreter, the reference target.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run_on(suite: &ConformanceSuite, target: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(target))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

/// Synthesizes `text`, runs the suite on the interpreter, and fails naming every scenario that did
/// not pass, with its steps; every id in `expected` must be synthesized.
fn every_scenario_passes(text: &str, expected: &[&str]) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let suite = synthesis.suite;
    let statuses = run_on(&suite, model);
    eprintln!("statuses: {statuses:#?}");
    eprintln!("refusals: {:#?}", synthesis.refusals);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    let steps: Vec<_> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| failing.iter().any(|(name, _)| **name == id.to_string()))
        .map(|(id, scenario)| format!("{id}: {:#?}", scenario.steps))
        .collect();
    assert!(
        failing.is_empty(),
        "failing: {failing:#?}\n{}",
        steps.join("\n")
    );
    for id in expected {
        assert!(
            statuses.contains_key(*id),
            "{id} synthesized: {statuses:#?}"
        );
    }
}

const HELD: &str = "format: ess/37
system: demo
version: v1
domain: demo.line
types:
  - {name: demo.line.PersonId, kind: newtype, of: Uuid}
  - {name: demo.line.ItemId, kind: newtype, of: Uuid}
errors:
  - {name: demo.line.ItemNotFound}
  - {name: demo.line.NotWaiting}
  - {name: demo.line.UnknownPerson}
  - {name: demo.line.ItemStateConflict}
events:
  - name: demo.line.PersonAdded
    fields:
      - {name: person_id, type: demo.line.PersonId}
  - name: demo.line.ItemAdded
    fields:
      - {name: item_id, type: demo.line.ItemId}
  - name: demo.line.ItemClaimed
    fields:
      - {name: item_id, type: demo.line.ItemId}
  - name: demo.line.ItemWithdrawn
    fields:
      - {name: item_id, type: demo.line.ItemId}
views:
  - name: demo.line.Items
    source: demo.line.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.line.ItemId}
      - {name: label, type: String}
      - {name: claimed_by, type: Optional<demo.line.PersonId>}
entities:
  - name: demo.line.Person
    identity: {name: person_id, type: demo.line.PersonId}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
  - name: demo.line.Item
    identity: {name: item_id, type: demo.line.ItemId}
    fields:
      - {name: label, type: String}
      - {name: claimed_by, type: Optional<demo.line.PersonId>}
    relations:
      - {name: claimer, kind: references, target: demo.line.Person, cardinality: one, via: claimed_by}
    lifecycle:
      initial: Waiting
      states: [Waiting, Claimed, Withdrawn]
      terminal: [Claimed, Withdrawn]
      transitions:
        - {name: claim, from: [Waiting], to: Claimed}
        - {name: withdraw, from: [Waiting], to: Withdrawn}
commands:
  - name: demo.line.AddPerson
    input:
      - {name: name, type: String}
    outcomes:
      - name: added
        creates: demo.line.Person
        instance: person_id
        sets:
          name: input.name
        emits: [demo.line.PersonAdded]
        payload:
          demo.line.PersonAdded:
            person_id: {generated: true}
  - name: demo.line.AddItem
    input:
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.line.Item
        instance: item_id
        sets:
          label: input.label
        emits: [demo.line.ItemAdded]
        payload:
          demo.line.ItemAdded:
            item_id: {generated: true}
  - name: demo.line.Claim
    input:
      - {name: item_id, type: demo.line.ItemId}
      - {name: by, type: demo.line.PersonId}
    outcomes:
      - name: no-such-item
        unknown_instance: true
        error: demo.line.ItemNotFound
      - name: not-waiting
        when_subject:
          predicate: state != Waiting
        error: demo.line.NotWaiting
      - name: unknown-claimer
        when_related: {via: input.by, exists: false}
        error: demo.line.UnknownPerson
      - name: claimed
        moves: demo.line.Item.claim
        instance: item_id
        sets:
          claimed_by: input.by
        emits: [demo.line.ItemClaimed]
        payload:
          demo.line.ItemClaimed:
            item_id: input.item_id
  - name: demo.line.Withdraw
    input:
      - {name: item_id, type: demo.line.ItemId}
    outcomes:
      - name: no-such-item
        unknown_instance: true
        error: demo.line.ItemNotFound
      - name: withdrawn
        moves: demo.line.Item.withdraw
        instance: item_id
        emits: [demo.line.ItemWithdrawn]
        payload:
          demo.line.ItemWithdrawn:
            item_id: input.item_id
      - name: wrong-state
        wrong_state: true
        error: demo.line.ItemStateConflict
";

/// A row driven to a held state for another command's refusal there: the item is claimed by a
/// person who exists, then refused by `Withdraw` in that state.
#[test]
fn a_held_state_is_reached_through_a_related_guard_naming_a_row_that_exists() {
    every_scenario_passes(
        HELD,
        &["demo.line.Item/state/Claimed/refuses/demo.line.Withdraw"],
    );
}

const PLACED: &str = "format: ess/37
system: demo
version: v1
domain: demo.desk
types:
  - {name: demo.desk.BoardId, kind: newtype, of: Uuid}
  - {name: demo.desk.CardId, kind: newtype, of: Uuid}
  - {name: demo.desk.NoteId, kind: newtype, of: Uuid}
  - {name: demo.desk.JobId, kind: newtype, of: Uuid}
errors:
  - {name: demo.desk.ItemMissing}
  - {name: demo.desk.NegativePosition}
  - {name: demo.desk.CardExists}
  - {name: demo.desk.BoardNotFound}
  - {name: demo.desk.UnknownNote}
  - {name: demo.desk.UnknownJob}
  - {name: demo.desk.BoardArchived}
  - {name: demo.desk.BoardStateConflict}
  - {name: demo.desk.CardNotFound}
events:
  - name: demo.desk.BoardCreated
    fields:
      - {name: board_id, type: demo.desk.BoardId}
  - name: demo.desk.BoardArchivedEvent
    fields:
      - {name: board_id, type: demo.desk.BoardId}
  - name: demo.desk.NoteOpened
    fields:
      - {name: note_id, type: demo.desk.NoteId}
  - name: demo.desk.JobCreated
    fields:
      - {name: job_id, type: demo.desk.JobId}
  - name: demo.desk.CardMoved
    fields:
      - {name: card_id, type: demo.desk.CardId}
  - name: demo.desk.CardAdded
    fields:
      - {name: card_id, type: demo.desk.CardId}
views:
  - name: demo.desk.Boards
    source: demo.desk.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.desk.BoardId}
      - {name: name, type: String}
  - name: demo.desk.Cards
    source: demo.desk.Card
    consistency: read_your_writes
    fields:
      - {name: card_id, type: demo.desk.CardId}
      - {name: board_id, type: demo.desk.BoardId}
      - {name: position, type: Integer}
entities:
  - name: demo.desk.Board
    identity: {name: board_id, type: demo.desk.BoardId}
    fields:
      - {name: name, type: String}
    relations:
      - {name: cards, kind: owns, target: demo.desk.Card, cardinality: many, via: board_id}
    lifecycle:
      initial: Active
      states: [Active, Archived]
      terminal: [Archived]
      transitions:
        - {name: archive, from: [Active], to: Archived}
  - name: demo.desk.Note
    identity: {name: note_id, type: demo.desk.NoteId}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
  - name: demo.desk.Job
    identity: {name: job_id, type: demo.desk.JobId}
    fields:
      - {name: goal, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
  - name: demo.desk.Card
    identity: {name: card_id, type: demo.desk.CardId}
    fields:
      - {name: board_id, type: demo.desk.BoardId}
      - {name: position, type: Integer}
      - {name: note_id, type: Optional<demo.desk.NoteId>}
      - {name: job_id, type: Optional<demo.desk.JobId>}
    invariants:
      - position >= 0
    relations:
      - {name: note, kind: references, target: demo.desk.Note, cardinality: one, via: note_id}
      - {name: job, kind: references, target: demo.desk.Job, cardinality: one, via: job_id}
    lifecycle: {initial: OnBoard, states: [OnBoard], terminal: [OnBoard], transitions: []}
commands:
  - name: demo.desk.CreateBoard
    input:
      - {name: name, type: String}
    outcomes:
      - name: created
        creates: demo.desk.Board
        instance: board_id
        sets:
          name: input.name
        emits: [demo.desk.BoardCreated]
        payload:
          demo.desk.BoardCreated:
            board_id: {generated: true}
  - name: demo.desk.ArchiveBoard
    input:
      - {name: board_id, type: demo.desk.BoardId}
    outcomes:
      - name: no-such-board
        unknown_instance: true
        error: demo.desk.BoardNotFound
      - name: archived
        moves: demo.desk.Board.archive
        instance: board_id
        emits: [demo.desk.BoardArchivedEvent]
        payload:
          demo.desk.BoardArchivedEvent:
            board_id: input.board_id
      - name: wrong-state
        wrong_state: true
        error: demo.desk.BoardStateConflict
  - name: demo.desk.OpenNote
    input:
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.desk.Note
        instance: note_id
        sets:
          title: input.title
        emits: [demo.desk.NoteOpened]
        payload:
          demo.desk.NoteOpened:
            note_id: {generated: true}
  - name: demo.desk.CreateJob
    input:
      - {name: goal, type: String}
    outcomes:
      - name: created
        creates: demo.desk.Job
        instance: job_id
        sets:
          goal: input.goal
        emits: [demo.desk.JobCreated]
        payload:
          demo.desk.JobCreated:
            job_id: {generated: true}
  - name: demo.desk.AddCard
    input:
      - {name: card_id, type: demo.desk.CardId}
      - {name: board_id, type: demo.desk.BoardId}
      - {name: position, type: Integer}
      - {name: note_id, type: Optional<demo.desk.NoteId>}
      - {name: job_id, type: Optional<demo.desk.JobId>}
    outcomes:
      - name: no-item
        when:
          all:
            - not defined(note_id)
            - not defined(job_id)
        error: demo.desk.ItemMissing
      - name: two-items
        when:
          all:
            - defined(note_id)
            - defined(job_id)
        error: demo.desk.ItemMissing
      - name: negative-position
        when: position < 0
        error: demo.desk.NegativePosition
      - name: already-exists
        existing_instance: true
        error: demo.desk.CardExists
      - name: no-such-board
        when_related: {via: input.board_id, exists: false}
        error: demo.desk.BoardNotFound
      - name: unknown-note
        when_related: {via: input.note_id, exists: false}
        error: demo.desk.UnknownNote
      - name: unknown-job
        when_related: {via: input.job_id, exists: false}
        error: demo.desk.UnknownJob
      - name: archived
        when_related: {via: input.board_id, predicate: state == Archived}
        error: demo.desk.BoardArchived
      - name: added
        creates: demo.desk.Card
        instance: card_id
        sets:
          board_id: input.board_id
          position: input.position
          note_id: input.note_id
          job_id: input.job_id
        emits: [demo.desk.CardAdded]
        payload:
          demo.desk.CardAdded:
            card_id: input.card_id
  - name: demo.desk.MoveCard
    input:
      - {name: card_id, type: demo.desk.CardId}
      - {name: position, type: Integer}
    outcomes:
      - name: no-such-card
        unknown_instance: true
        error: demo.desk.CardNotFound
      - name: archived
        when_related: {via: board_id, predicate: state == Archived}
        error: demo.desk.BoardArchived
      - name: moved
        updates: demo.desk.Card
        instance: card_id
        sets:
          position: input.position
        emits: [demo.desk.CardMoved]
        payload:
          demo.desk.CardMoved:
            card_id: input.card_id
";

/// A card placed for the read of a kept reference after a board's branch (ess/30): the card is
/// added naming a job or a note that exists.
#[test]
fn a_row_placed_for_an_invariant_check_names_related_rows_that_exist() {
    every_scenario_passes(
        PLACED,
        &[
            "demo.desk.Card/invariant/after/demo.desk.CreateBoard/created",
            "demo.desk.Card/invariant/after/demo.desk.ArchiveBoard/archived",
        ],
    );
}
