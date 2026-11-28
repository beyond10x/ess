//! Hotfix 0.99.11: a branch that copies a field of a related row (`{related: {via: input.<field>,
//! field: f}}`) while its guards also read that row, beside a row-set guard over the rows created
//! from it. The value expected is the one of the row the command is sent, never of another row of
//! that entity. Every scenario synthesized must pass on the interpreter, the reference target.

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

const MODEL: &str = "format: ess/37
system: demo
version: v1
domain: demo.shelf
types:
  - {name: demo.shelf.BatchId, kind: newtype, of: Uuid}
  - {name: demo.shelf.SampleId, kind: newtype, of: Uuid}
  - {name: demo.shelf.FolderId, kind: newtype, of: Uuid}
  - {name: demo.shelf.EntryId, kind: newtype, of: Uuid}
  - name: demo.shelf.Body
    kind: struct
    fields:
      - {name: words, type: List<String>}
errors:
  - {name: demo.shelf.NoSuchBatch}
  - {name: demo.shelf.BatchRunning}
  - {name: demo.shelf.NoSuchFolder}
  - {name: demo.shelf.NoSuchSample}
  - {name: demo.shelf.FolderClosed}
  - {name: demo.shelf.AdoptedBefore}
events:
  - name: demo.shelf.BatchStarted
    fields:
      - {name: batch_id, type: demo.shelf.BatchId}
  - name: demo.shelf.BatchFinished
    fields:
      - {name: batch_id, type: demo.shelf.BatchId}
  - name: demo.shelf.SampleTaken
    fields:
      - {name: sample_id, type: demo.shelf.SampleId}
  - name: demo.shelf.FolderOpened
    fields:
      - {name: folder_id, type: demo.shelf.FolderId}
  - name: demo.shelf.FolderClosedEvent
    fields:
      - {name: folder_id, type: demo.shelf.FolderId}
  - name: demo.shelf.SampleAdopted
    fields:
      - {name: entry_id, type: demo.shelf.EntryId}
      - {name: folder_id, type: demo.shelf.FolderId}
      - {name: sample_id, type: demo.shelf.SampleId}
views:
  - name: demo.shelf.Entries
    source: demo.shelf.Entry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: demo.shelf.EntryId}
      - {name: folder_id, type: demo.shelf.FolderId}
      - {name: text, type: demo.shelf.Body}
      - {name: sample_id, type: Optional<demo.shelf.SampleId>}
entities:
  - name: demo.shelf.Batch
    identity: {name: batch_id, type: demo.shelf.BatchId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Running
      states: [Running, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Running], to: Done}
  - name: demo.shelf.Sample
    identity: {name: sample_id, type: demo.shelf.SampleId}
    fields:
      - {name: batch_id, type: demo.shelf.BatchId}
      - {name: text, type: demo.shelf.Body}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept], transitions: []}
  - name: demo.shelf.Folder
    identity: {name: folder_id, type: demo.shelf.FolderId}
    fields:
      - {name: title, type: String}
    relations:
      - {name: entries, kind: owns, target: demo.shelf.Entry, cardinality: many, via: folder_id}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.shelf.Entry
    identity: {name: entry_id, type: demo.shelf.EntryId}
    fields:
      - {name: folder_id, type: demo.shelf.FolderId}
      - {name: text, type: demo.shelf.Body}
      - {name: sample_id, type: Optional<demo.shelf.SampleId>}
    relations:
      - {name: sample, kind: references, target: demo.shelf.Sample, cardinality: one, via: sample_id}
    lifecycle: {initial: Taken, states: [Taken], terminal: [Taken], transitions: []}
commands:
  - name: demo.shelf.StartBatch
    input:
      - {name: label, type: String}
    outcomes:
      - name: started
        creates: demo.shelf.Batch
        instance: batch_id
        sets:
          label: input.label
        emits: [demo.shelf.BatchStarted]
        payload:
          demo.shelf.BatchStarted:
            batch_id: {generated: true}
  - name: demo.shelf.FinishBatch
    input:
      - {name: batch_id, type: demo.shelf.BatchId}
    outcomes:
      - name: no-such-batch
        unknown_instance: true
        error: demo.shelf.NoSuchBatch
      - name: finished
        moves: demo.shelf.Batch.finish
        instance: batch_id
        emits: [demo.shelf.BatchFinished]
        payload:
          demo.shelf.BatchFinished:
            batch_id: input.batch_id
      - name: wrong-state
        wrong_state: true
        error: demo.shelf.BatchRunning
  - name: demo.shelf.TakeSample
    input:
      - {name: batch_id, type: demo.shelf.BatchId}
      - {name: text, type: demo.shelf.Body}
    outcomes:
      - name: no-batch
        when_related: {via: input.batch_id, exists: false}
        error: demo.shelf.NoSuchBatch
      - name: running
        when_related: {via: input.batch_id, predicate: {state: {none_of: [Done]}}}
        error: demo.shelf.BatchRunning
      - name: taken
        creates: demo.shelf.Sample
        instance: sample_id
        sets:
          batch_id: input.batch_id
          text: input.text
        emits: [demo.shelf.SampleTaken]
        payload:
          demo.shelf.SampleTaken:
            sample_id: {generated: true}
  - name: demo.shelf.OpenFolder
    input:
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.shelf.Folder
        instance: folder_id
        sets:
          title: input.title
        emits: [demo.shelf.FolderOpened]
        payload:
          demo.shelf.FolderOpened:
            folder_id: {generated: true}
  - name: demo.shelf.CloseFolder
    input:
      - {name: folder_id, type: demo.shelf.FolderId}
    outcomes:
      - name: no-such-folder
        unknown_instance: true
        error: demo.shelf.NoSuchFolder
      - name: closed
        moves: demo.shelf.Folder.close
        instance: folder_id
        emits: [demo.shelf.FolderClosedEvent]
        payload:
          demo.shelf.FolderClosedEvent:
            folder_id: input.folder_id
      - name: wrong-state
        wrong_state: true
        error: demo.shelf.FolderClosed
  - name: demo.shelf.Adopt
    input:
      - {name: folder_id, type: demo.shelf.FolderId}
      - {name: sample_id, type: demo.shelf.SampleId}
    outcomes:
      - name: no-folder
        when_related: {via: input.folder_id, exists: false}
        error: demo.shelf.NoSuchFolder
      - name: no-sample
        when_related: {via: input.sample_id, exists: false}
        error: demo.shelf.NoSuchSample
      - name: closed
        when_related: {via: input.folder_id, predicate: state == Closed}
        error: demo.shelf.FolderClosed
      - name: adopted-before
        when_related:
          entity: demo.shelf.Entry
          where: {all: [defined(sample_id), sample_id == input.sample_id]}
          exists: true
        error: demo.shelf.AdoptedBefore
      - name: adopted
        creates: demo.shelf.Entry
        instance: entry_id
        sets:
          folder_id: input.folder_id
          text: {related: {via: input.sample_id, field: text}}
          sample_id: input.sample_id
        emits: [demo.shelf.SampleAdopted]
        payload:
          demo.shelf.SampleAdopted:
            entry_id: {generated: true}
            folder_id: input.folder_id
            sample_id: input.sample_id
";

/// The entry is expected with the text of the sample the command names, not of another sample.
#[test]
fn a_value_copied_from_a_related_row_is_the_named_rows() {
    every_scenario_passes(MODEL, &["demo.shelf.Adopt/outcome/adopted"]);
}
