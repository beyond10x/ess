//! Adversary pass 1 over beyond10x/ess#805 (`a swapped run keeps an input bound to the caller that
//! a selector reads`).
//!
//! The fix keeps a caller-bound input as sent in the run appended with the callers swapped, and
//! refuses that run where a selector reading the kept input pins no stored field to a value the
//! first run never sends (it would find the first run's rows). The refusal is decided over the
//! inputs recorded as kept, by name. Where two inputs carry the same caller attribute, only the
//! first in input-name order is recorded: the second is kept silently because its value is already
//! kept. A selector reading only the second is then never checked, the swapped run keeps the
//! caller's value, and its selector finds the row the first run left.
//!
//! Oracle: every scenario synthesis writes passes against the interpreted target of the same
//! model.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{report::Status, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// The issue's model with a second input, `approver`, bound to the same caller attribute by its
/// own guard and written into a field the selector reads; the selector pins only `writer`.
/// `approver` sorts before `writer`.
const APPROVER: &str = r#"format: ess/37
system: depot
version: v1
domain: depot.slips
types:
  - name: depot.slips.SlipId
    kind: newtype
    of: String
entities:
  - name: depot.slips.Slip
    identity:
      name: slip_id
      type: depot.slips.SlipId
    fields:
      - {name: writer, type: String}
      - {name: tag, type: String}
      - {name: approver, type: String}
    lifecycle:
      initial: Posted
      states: [Posted]
      terminal: [Posted]
      transitions: []
actors:
  - name: depot.slips.Clerk
    attributes:
      - {name: principal, type: String}
    may:
      - depot.slips.PostSlip
errors:
  - name: depot.slips.NotYours
  - name: depot.slips.Duplicate
  - name: depot.slips.TagTaken
commands:
  - name: depot.slips.PostSlip
    input:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: writer, type: String}
      - {name: tag, type: String}
      - {name: approver, type: String}
    outcomes:
      - name: not-yours
        when: "writer != caller.principal"
        error: depot.slips.NotYours
      - name: not-approver
        when: "approver != caller.principal"
        error: depot.slips.NotYours
      - {name: duplicate, existing_instance: true, error: depot.slips.Duplicate}
      - name: tag-taken
        when_related:
          entity: depot.slips.Slip
          where: {all: ["writer == input.writer", "approver != \"nobody\""]}
          exists: true
        error: depot.slips.TagTaken
      - name: posted
        creates: depot.slips.Slip
        instance: slip_id
        sets:
          writer: input.writer
          tag: input.tag
          approver: input.approver
        emits: [depot.slips.SlipPosted]
        payload:
          depot.slips.SlipPosted: {slip_id: input.slip_id}
events:
  - name: depot.slips.SlipPosted
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
views:
  - name: depot.slips.Slips
    source: depot.slips.Slip
    consistency: read_your_writes
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: writer, type: String}
      - {name: tag, type: String}
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("slips.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every scenario of `synthesis` run against the interpreted target of `ir`, by id.
fn statuses(synthesis: &Synthesis, ir: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The suite of `text` is written and the interpreted target of the same model passes every
/// scenario of it.
fn every_scenario_passes(label: &str, text: &str) {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    assert!(
        !synthesis.suite.scenarios.is_empty(),
        "{label}: synthesis writes scenarios"
    );
    let failed: Vec<_> = statuses(&synthesis, &ir)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(
        failed.is_empty(),
        "{label}: the interpreted target passes every scenario of its own suite; failed {failed:?}"
    );
}

/// Red on ae36116e9e: `approver` is recorded as kept, `writer` is kept silently under the same
/// value, and the selector `writer == input.writer` is never checked. The mixed-caller witness of
/// `posted` appends its reversed run with `writer` as the caller's own principal, which the first
/// run also wrote, and the target answers `tag-taken` where the suite expects `posted`.
#[test]
fn a_second_input_bound_to_the_same_caller_attribute_is_checked_against_the_selector() {
    every_scenario_passes("approver before writer", APPROVER);
}

/// Green control: the same model with `approver` renamed so it sorts after `writer`. `writer` is
/// then the input recorded as kept, the selector is checked, and the run is refused with its note.
#[test]
fn control_the_same_model_with_the_second_input_sorting_last_passes() {
    every_scenario_passes(
        "zapprover after writer",
        &APPROVER.replace("approver", "zapprover"),
    );
}
