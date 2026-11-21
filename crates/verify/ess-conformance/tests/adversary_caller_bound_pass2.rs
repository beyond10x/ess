//! Adversary pass 2 over beyond10x/ess#805 (`a swapped run keeps an input bound to the caller that
//! a selector reads`).
//!
//! The fix writes the step's caller into the command before `keeps_branch`, so a fresh value has to
//! decide every input guard as before under the caller the step is sent as. `keeps_branch` decides
//! nothing where the step's input cannot be flattened, and `Sent::input` holds only the step's
//! literal inputs: a step that also sends an input bound by the run (an instance the scenario
//! captured, `{kind: instance}`) is not flattened, the caller guard is never read, and the swapped
//! run sends a fresh value the guard answers. The issue's failure then stands wherever the command
//! names an arranged row beside the caller-bound input: a `via:` input, or the addressed identity
//! of a move command.
//!
//! Oracle: every scenario synthesis writes passes against the interpreted target of the same
//! model.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{report::Status, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// The issue's model with one more input, `badge_id`, naming a `Badge` an arranging command
/// creates (`when_related: {via: input.badge_id, exists: false}`). The scenario of `posted`
/// captures the badge, so `badge_id` is sent as an instance, not a literal.
const BADGE: &str = r#"format: ess/37
system: depot
version: v1
domain: depot.slips
types:
  - name: depot.slips.BadgeId
    kind: newtype
    of: String
  - name: depot.slips.SlipId
    kind: newtype
    of: String
entities:
  - name: depot.slips.Badge
    identity: {name: badge_id, type: depot.slips.BadgeId}
    fields:
      - {name: holder, type: String}
    lifecycle: {initial: On, states: [On], terminal: [On], transitions: []}
  - name: depot.slips.Slip
    identity:
      name: slip_id
      type: depot.slips.SlipId
    fields:
      - {name: writer, type: String}
      - {name: tag, type: String}
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
      - depot.slips.Register
errors:
  - name: depot.slips.NotYours
  - name: depot.slips.Duplicate
  - name: depot.slips.TagTaken
  - name: depot.slips.Missing
commands:
  - name: depot.slips.Register
    input:
      - {name: badge_id, type: depot.slips.BadgeId}
      - {name: holder, type: String}
    outcomes:
      - {name: registered-twice, existing_instance: true, error: depot.slips.Duplicate}
      - name: registered
        creates: depot.slips.Badge
        instance: badge_id
        sets:
          holder: input.holder
        emits: [depot.slips.Registered]
        payload:
          depot.slips.Registered: {badge_id: input.badge_id}
  - name: depot.slips.PostSlip
    input:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: badge_id, type: depot.slips.BadgeId}
      - {name: writer, type: String}
      - {name: tag, type: String}
    outcomes:
      - name: not-yours
        when: "writer != caller.principal"
        error: depot.slips.NotYours
      - name: no-badge
        when_related: {via: input.badge_id, exists: false}
        error: depot.slips.Missing
      - {name: duplicate, existing_instance: true, error: depot.slips.Duplicate}
      - name: tag-taken
        when_related:
          entity: depot.slips.Slip
          where: {all: ["tag == input.tag", "writer == input.writer"]}
          exists: true
        error: depot.slips.TagTaken
      - name: posted
        creates: depot.slips.Slip
        instance: slip_id
        sets:
          writer: input.writer
          tag: input.tag
        emits: [depot.slips.SlipPosted]
        payload:
          depot.slips.SlipPosted: {slip_id: input.slip_id}
events:
  - name: depot.slips.Registered
    fields:
      - {name: badge_id, type: depot.slips.BadgeId}
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

/// A move command, `Sign`, whose input guard binds `signer` to the caller and whose row-set
/// selector reads `signer`; the slip it moves is captured from `PostSlip`.
const SIGN: &str = r#"format: ess/37
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
      - {name: signed_by, type: String}
    lifecycle:
      initial: Posted
      states: [Posted, Signed]
      terminal: [Signed]
      transitions:
        - {name: sign, from: [Posted], to: Signed}
actors:
  - name: depot.slips.Clerk
    attributes:
      - {name: principal, type: String}
    may:
      - depot.slips.PostSlip
      - depot.slips.Sign
errors:
  - name: depot.slips.NotYours
  - name: depot.slips.Duplicate
  - name: depot.slips.Busy
  - name: depot.slips.Missing
commands:
  - name: depot.slips.PostSlip
    input:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: tag, type: String}
    outcomes:
      - {name: duplicate, existing_instance: true, error: depot.slips.Duplicate}
      - name: posted
        creates: depot.slips.Slip
        instance: slip_id
        sets:
          tag: input.tag
          writer: "nobody"
          signed_by: "nobody"
        emits: [depot.slips.SlipPosted]
        payload:
          depot.slips.SlipPosted: {slip_id: input.slip_id}
  - name: depot.slips.Sign
    input:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: signer, type: String}
    outcomes:
      - name: not-yours
        when: "signer != caller.principal"
        error: depot.slips.NotYours
      - name: unknown
        when_related: {via: input.slip_id, exists: false}
        error: depot.slips.Missing
      - name: signer-busy
        when_related:
          entity: depot.slips.Slip
          where: "signed_by == input.signer"
          exists: true
        error: depot.slips.Busy
      - name: signed
        moves: depot.slips.Slip.sign
        instance: slip_id
        sets:
          signed_by: input.signer
        emits: [depot.slips.SlipSigned]
        payload:
          depot.slips.SlipSigned: {slip_id: input.slip_id}
events:
  - name: depot.slips.SlipPosted
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
  - name: depot.slips.SlipSigned
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
views:
  - name: depot.slips.Slips
    source: depot.slips.Slip
    consistency: read_your_writes
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: signed_by, type: String}
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

/// The scenarios `scenarios` of the suite of `text` are written, and the interpreted target of the
/// same model passes each of them.
fn scenarios_pass(label: &str, text: &str, scenarios: &[&str]) {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    let statuses = statuses(&synthesis, &ir);
    let failed: Vec<_> = scenarios
        .iter()
        .map(|scenario| (*scenario, statuses.get(*scenario)))
        .filter(|(_, status)| *status != Some(&Status::Passed))
        .collect();
    assert!(
        failed.is_empty(),
        "{label}: the interpreted target passes these scenarios of its own suite; failed {failed:?}"
    );
}

/// Red on ae36116e9e plus the uncommitted `caller.rs` fix: the swapped run of `posted` sends
/// `writer` a fresh value (`writer-524480`) under the caller `principal-262144`, and the target
/// answers `not-yours`. `badge_id` is sent as a captured instance, so the step's literal inputs do
/// not flatten and `keeps_branch` takes any value without reading the caller guard.
#[test]
fn a_caller_bound_input_beside_a_captured_instance_keeps_its_guard_in_the_swapped_run() {
    scenarios_pass(
        "issue model with a captured badge",
        BADGE,
        &["depot.slips.PostSlip/outcome/posted"],
    );
}

/// Red the same way on a move command: the swapped run of `signed` sends `signer` a fresh value
/// (`signer-524480`) under the caller `principal-262144`, and the target answers `not-yours`.
#[test]
fn a_move_command_binding_its_input_to_the_caller_keeps_its_guard_in_the_swapped_run() {
    scenarios_pass(
        "move command",
        SIGN,
        &[
            "depot.slips.Sign/outcome/signed",
            "depot.slips.Slip/transition/sign/by/depot.slips.Sign/signed",
        ],
    );
}
