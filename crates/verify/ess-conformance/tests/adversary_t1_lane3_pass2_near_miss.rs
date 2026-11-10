//! Adversary cases, second pass, for beyond10x/ess#727 (a narrowed external branch relating input
//! to stored values), against the correction that left identity inputs unbound and added
//! `near_miss`: the near miss changing an input a sibling's `sets:` stores, the near miss sending an
//! input another branch's subject guard selects, and the skip of a guard reading the identity input
//! hiding a force its other conjunct decides.
use std::collections::BTreeMap;

use ess_compiler::ir::{ResolvedCondition, ResolvedOutcome};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REFUSED: &str = "example.rollout.DecideAdmission/outcome/refused";
const ADMITTED: &str = "example.rollout.DecideAdmission/outcome/admitted";

/// The story's "only reader" specification, with the default sibling storing the list it was sent.
const SIBLING_STORES_THE_LIST: &str = "  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when: missing.count >= 1
    when_subject:
      predicate:
        forall: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}
    moves: example.rollout.Rollout.refuse
    emits: [example.rollout.Refused]
    payload:
      example.rollout.Refused: {id: input.id}
    instance: id
  - name: admitted
    moves: example.rollout.Rollout.admit
    sets: {granted: input.missing}
    emits: [example.rollout.Admitted]
    payload:
      example.rollout.Admitted: {id: input.id}
    instance: id
";

/// The story's "only reader" specification, with a refusal taken where some element of `missing`
/// is not required: exactly where the narrowed guard is false over a non-empty list.
const ANOTHER_BRANCH_READS_THE_INPUT: &str = "  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when: missing.count >= 1
    when_subject:
      predicate:
        forall: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}
    moves: example.rollout.Rollout.refuse
    emits: [example.rollout.Refused]
    payload:
      example.rollout.Refused: {id: input.id}
    instance: id
  - name: unknown-capability
    when_subject:
      predicate:
        exists: {in: input.missing, as: m, that: {not: {exists: {in: requires, as: c, that: m == c}}}}
    error: example.rollout.AdmissionIncomplete
  - name: admitted
    moves: example.rollout.Rollout.admit
    emits: [example.rollout.Admitted]
    payload:
      example.rollout.Admitted: {id: input.id}
    instance: id
";

fn model(kind: &str, outcomes: &str) -> String {
    format!(
        "format: ess/35
system: example
version: v1
domain: example.rollout
entities:
  - name: example.rollout.Rollout
    identity: {{name: id, type: String}}
    fields:
      - {{name: requires, type: {kind}}}
      - {{name: origin, type: String}}
      - {{name: granted, type: {kind}}}
    lifecycle:
      initial: Pending
      states: [Pending, Refused, Admitted, Stopped]
      terminal: [Admitted, Stopped]
      transitions:
        - {{name: refuse, from: [Pending], to: Refused}}
        - {{name: admit, from: [Pending], to: Admitted}}
        - {{name: stop, from: [Refused], to: Stopped}}
events:
  - name: example.rollout.Created
    fields: [{{name: id, type: String}}]
  - name: example.rollout.Refused
    fields: [{{name: id, type: String}}]
  - name: example.rollout.Admitted
    fields: [{{name: id, type: String}}]
  - name: example.rollout.Stopped
    fields: [{{name: id, type: String}}]
errors:
  - name: example.rollout.AdmissionIncomplete
commands:
- name: example.rollout.CreateRollout
  input:
  - {{name: id, type: String}}
  - {{name: requires, type: {kind}}}
  - {{name: granted, type: {kind}}}
  outcomes:
  - name: created
    creates: example.rollout.Rollout
    instance: id
    sets: {{requires: input.requires, origin: input.id, granted: input.granted}}
    emits: [example.rollout.Created]
    payload:
      example.rollout.Created: {{id: input.id}}
- name: example.rollout.DecideAdmission
  input:
  - {{name: id, type: String}}
  - {{name: missing, type: {kind}}}
  - {{name: bindings, type: List<String>}}
{outcomes}- name: example.rollout.StopRollout
  input:
  - {{name: id, type: String}}
  outcomes:
  - name: stopped
    moves: example.rollout.Rollout.stop
    instance: id
    emits: [example.rollout.Stopped]
    payload:
      example.rollout.Stopped: {{id: input.id}}
views:
  - name: example.rollout.Rollouts
    source: example.rollout.Rollout
    consistency: read_your_writes
    fields:
      - {{name: id, type: String}}
      - {{name: granted, type: {kind}}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("rollout.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn run(suite: &ConformanceSuite, model: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn failing(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.clone())
        .collect()
}

fn ignoring_the_narrowing(model: &EssIr) -> EssIr {
    model.with_commands_rewritten(|command| {
        let mut command = command.clone();
        for outcome in &mut command.outcomes {
            let ResolvedOutcome { condition, .. } = outcome;
            if let ResolvedCondition::ExternalSubject { cause, .. }
            | ResolvedCondition::ExternalWhen { cause, .. } = condition
            {
                *condition = ResolvedCondition::External {
                    cause: cause.clone(),
                };
            }
        }
        command
    })
}

/// The interpreted target passes every scenario synthesized for `text`, which holds both
/// `refused` and `admitted`.
fn correct_target_passes(text: &str) -> (EssIr, ConformanceSuite) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    for wanted in [REFUSED, ADMITTED] {
        assert!(
            synthesis
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string() == wanted),
            "no {wanted}: {:#?}",
            synthesis.refusals
        );
    }
    let statuses = run(&synthesis.suite, model.clone());
    assert!(
        failing(&statuses).is_empty(),
        "the interpreted target fails the synthesized suite: {:#?}\n{:#?}",
        failing(&statuses),
        synthesis
            .suite
            .scenarios
            .iter()
            .filter(|(id, _)| id.to_string() == ADMITTED)
            .collect::<Vec<_>>()
    );
    (model, synthesis.suite)
}

/// `near_miss` changes `missing` in the `admitted` scenario while the sibling's `sets:` stores
/// `missing` and a view reads it back: the row expected there is the one the changed input writes,
/// so a correct target passes, and the near miss still fires (a target ignoring the narrowing
/// fails), as it does without the stored copy.
#[test]
fn adv2_near_miss_keeps_the_siblings_stored_copy_of_the_input_consistent() {
    let (model, suite) = correct_target_passes(&model("List<String>", SIBLING_STORES_THE_LIST));
    let statuses = run(&suite, ignoring_the_narrowing(&model));
    assert!(
        !failing(&statuses).is_empty(),
        "a target ignoring the narrowing passes every scenario: {statuses:#?}"
    );
}

/// `near_miss` checks only the other branches' input guards. The input it sends makes the
/// `unknown-capability` subject guard true, so a correct target refuses with
/// `AdmissionIncomplete` where the scenario expects `admitted`.
#[test]
fn adv2_near_miss_sends_an_input_another_branchs_subject_guard_selects() {
    correct_target_passes(&model("List<String>", ANOTHER_BRANCH_READS_THE_INPUT));
}

/// Control for the case above: the same model with the narrowed branch's own `when:` removed, so
/// `near_miss` (which needs one) never runs and the `admitted` scenario keeps its plain input. The
/// interpreted target passes, so the failure above is the near miss's input, not the arrangement's.
#[test]
fn adv2_control_without_the_near_miss_the_correct_target_passes() {
    correct_target_passes(&model(
        "List<String>",
        &ANOTHER_BRANCH_READS_THE_INPUT.replace("    when: missing.count >= 1\n", ""),
    ));
}
