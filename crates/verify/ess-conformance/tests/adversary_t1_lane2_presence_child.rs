//! Adversary, turn 1 lane 2, against 2a03419cd7 (beyond10x/ess#729): three-valued isolation of a
//! presence child beside a read of the same `Optional`.
//!
//! Acceptance 5 is checked against the interpreter, which the unit's own test file never runs; the
//! reverse orientation (the read child isolated while its presence sibling must leave the path
//! absent) and a sibling that mixes the read with another field are probed beside it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, RefusalCause, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn ir(body: &str) -> EssIr {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{body}"))
}

fn unwitnessed(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| matches!(refusal.cause, RefusalCause::ChildUnwitnessed { .. }))
        .map(|refusal| format!("{refusal:?}"))
        .collect()
}

/// Each scenario of `synthesis`, run against `target` interpreted as the implementation.
fn statuses(target: &EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn assert_interpreter_passes(body: &str) {
    let model = ir(body);
    let synthesis = synthesize(&model);
    let failing: Vec<_> = statuses(&model, &synthesis)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}\n{body}");
}

fn review(guard: &str) -> String {
    format!(
        "format: ess/23
system: demo
version: v1
domain: demo.review
types:
  - {{name: demo.review.Kind, kind: enum, variants: [Note, Correction]}}
  - {{name: demo.review.CheckStatus, kind: enum, variants: [Reported, Verified]}}
errors:
  - name: demo.review.CorrectionNotVerified
commands:
  - name: demo.review.RecordDecision
    input:
      - {{name: kind, type: demo.review.Kind}}
      - {{name: check_status, type: 'Optional<demo.review.CheckStatus>'}}
    outcomes:
      - name: correction-not-verified
        when:
          {guard}
        error: demo.review.CorrectionNotVerified
      - name: recorded
        accepts: nothing
"
    )
}

fn range(guard: &str) -> String {
    format!(
        "format: ess/23
system: demo
version: v1
domain: demo.range
errors:
  - name: demo.range.Refused
commands:
  - name: demo.range.Check
    input:
      - {{name: minimum, type: 'Optional<Integer>'}}
      - {{name: maximum, type: 'Optional<Integer>'}}
      - {{name: level, type: Integer}}
    outcomes:
      - name: refused
        when:
          {guard}
        error: demo.range.Refused
      - name: accepted
        accepts: nothing
"
    )
}

const SPEC_1: &str = "all: [kind == Correction, defined(check_status), check_status != Verified]";

/// Acceptance 5, first half: a target that reads an absent `check_status` as unequal to `Verified`
/// answers `correction-not-verified` for `{kind: Correction}` alone, and fails `recorded`.
#[test]
fn acceptance_5_a_target_reading_absent_as_unequal_fails_recorded() {
    let synthesis = synthesize(&ir(&review(SPEC_1)));
    let wrong = ir(&review(
        "any: [{all: [kind == Correction, not defined(check_status)]}, \
         {all: [kind == Correction, check_status != Verified]}]",
    ));
    let runs = statuses(&wrong, &synthesis);
    let recorded = runs
        .get("demo.review.RecordDecision/outcome/recorded")
        .unwrap_or_else(|| panic!("no recorded scenario: {runs:#?}"));
    assert_ne!(*recorded, Status::Passed, "{runs:#?}");
}

/// Acceptance 5, second half: the interpreter passes every scenario of specifications 1 to 4.
#[test]
fn acceptance_5_the_interpreter_passes_every_scenario_of_1_to_4() {
    assert_interpreter_passes(&review(SPEC_1));
    assert_interpreter_passes(&review(SPEC_1).replace(
        "    outcomes:\n",
        "    outcomes:\n      - name: incomplete\n        when:\n          \
         all: [kind == Correction, not defined(check_status)]\n        \
         error: demo.review.CorrectionNotVerified\n",
    ));
    assert_interpreter_passes(&range(
        "all: [defined(minimum), defined(maximum), minimum > maximum]",
    ));
    assert_interpreter_passes(&range("any: [not defined(minimum), minimum > 10]"));
}

/// A sibling that reads `minimum` and also `level`: at `minimum` absent it is unknown for
/// `level != 1` and true for `level == 1`. Before 2a03419cd7 the presence child's row needed the
/// sibling satisfied (`level == 1`); the lenient rule also takes `level != 1`, where a target that
/// drops `defined(minimum)` answers the same. Such a target must still fail some scenario.
#[test]
fn a_target_dropping_the_presence_child_beside_a_mixed_sibling_fails() {
    let guard = "all: [defined(minimum), {any: [minimum > 10, level == 1]}]";
    let synthesis = synthesize(&ir(&range(guard)));
    assert!(
        unwitnessed(&synthesis).is_empty(),
        "{:#?}",
        synthesis.refusals
    );
    assert_interpreter_passes(&range(guard));
    let dropped = ir(&range("any: [minimum > 10, level == 1]"));
    let runs = statuses(&dropped, &synthesis);
    assert!(
        runs.values().any(|status| *status != Status::Passed),
        "a target without `defined(minimum)` passes every scenario: {runs:#?}"
    );
}
