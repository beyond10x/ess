//! An `external:` branch narrowed by a subject guard that relates the command's input to a stored
//! field (`ess/29`, beyond10x/ess#727): it is witnessed forced on a row and input the guard holds
//! for, with the guard's row/input comparisons grounded on the row as a `when_subject:` branch's
//! are, and refused `ESS-SYNTH-003` where no row and input serve.
//!
//! The model is composed inline: one rollout entity, a creator, a command whose narrowed `refused`
//! branch is the case under test, and `StopRollout` behind its target state.
use std::collections::BTreeMap;

use ess_compiler::ir::{ResolvedCondition, ResolvedOutcome};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, RefusalCause};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const DECIDE: &str = "example.rollout.DecideAdmission";
const REFUSED: &str = "example.rollout.DecideAdmission/outcome/refused";

/// The reported shape: an own `when:` and a sibling that reads the stored row.
const WITH_SIBLING: &str = "  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when:
      all:
        - missing.count >= 1
        - forall: {in: missing, as: m, that: {not: {exists: {in: bindings, as: b, that: b == m}}}}
    when_subject:
      predicate:
        forall: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}
    moves: example.rollout.Rollout.refuse
    emits: [example.rollout.Refused]
    payload:
      example.rollout.Refused: {id: input.id}
    instance: id
  - name: admitted
    when: missing.count == 0
    when_subject:
      predicate:
        forall: {in: requires, as: c, that: {exists: {in: input.bindings, as: b, that: b == c}}}
    moves: example.rollout.Rollout.admit
    emits: [example.rollout.Admitted]
    payload:
      example.rollout.Admitted: {id: input.id}
    instance: id
  - name: incomplete
    error: example.rollout.AdmissionIncomplete
";

/// The same with no own `when:`, the guard needing a non-empty list.
const NO_OWN_WHEN: &str = "  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when_subject:
      predicate:
        exists: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}
    moves: example.rollout.Rollout.refuse
    emits: [example.rollout.Refused]
    payload:
      example.rollout.Refused: {id: input.id}
    instance: id
  - name: admitted
    when: missing.count == 0
    when_subject:
      predicate:
        forall: {in: requires, as: c, that: {exists: {in: input.bindings, as: b, that: b == c}}}
    moves: example.rollout.Rollout.admit
    emits: [example.rollout.Admitted]
    payload:
      example.rollout.Admitted: {id: input.id}
    instance: id
  - name: incomplete
    error: example.rollout.AdmissionIncomplete
";

/// The narrowed branch is the only one of its command that reads the stored row.
const ONLY_READER: &str = "  outcomes:
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
    emits: [example.rollout.Admitted]
    payload:
      example.rollout.Admitted: {id: input.id}
    instance: id
";

/// The scalar form, over `String` fields.
const SCALAR: &str = "  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when_subject:
      predicate: requires == input.missing
    moves: example.rollout.Rollout.refuse
    emits: [example.rollout.Refused]
    payload:
      example.rollout.Refused: {id: input.id}
    instance: id
  - name: admitted
    moves: example.rollout.Rollout.admit
    emits: [example.rollout.Admitted]
    payload:
      example.rollout.Admitted: {id: input.id}
    instance: id
";

/// No row and input satisfy the guard.
const UNSATISFIABLE: &str = "  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when_subject:
      predicate:
        all: [requires == input.missing, requires != input.missing]
    moves: example.rollout.Rollout.refuse
    emits: [example.rollout.Refused]
    payload:
      example.rollout.Refused: {id: input.id}
    instance: id
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
  outcomes:
  - name: created
    creates: example.rollout.Rollout
    instance: id
    sets: {{requires: input.requires}}
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

fn failing(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The inputs of the creator and of the decision in the `refused` scenario.
fn inputs(
    suite: &ConformanceSuite,
) -> (
    BTreeMap<String, ScenarioValue>,
    BTreeMap<String, ScenarioValue>,
) {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == REFUSED)
        .unwrap_or_else(|| panic!("no scenario {REFUSED}"));
    let mut created = None;
    let mut decided = None;
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            if command.to_string() == DECIDE {
                decided = Some(input.clone());
            } else if command.to_string() == "example.rollout.CreateRollout" {
                created = Some(input.clone());
            }
        }
    }
    (
        created.unwrap_or_else(|| panic!("no creator in {scenario:#?}")),
        decided.unwrap_or_else(|| panic!("no decision in {scenario:#?}")),
    )
}

fn list(value: Option<&ScenarioValue>) -> Vec<Node> {
    match value {
        Some(ScenarioValue::Literal {
            value: Node::Seq(items),
        }) => items.clone(),
        other => panic!("not a list: {other:?}"),
    }
}

/// The suite is synthesized whole and the interpreted target passes every scenario of it.
fn witnessed(kind: &str, outcomes: &str) -> (EssIr, ConformanceSuite) {
    let model = ir(&model(kind, outcomes));
    let synthesis = synthesize(&model);
    // The model declares no refusal for a state a command is not taken in; those are not the case.
    let refusals: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| !matches!(refusal.cause, RefusalCause::RefusalUndeclared { .. }))
        .collect();
    assert!(refusals.is_empty(), "{refusals:#?}");
    let statuses = run(&synthesis.suite, model.clone());
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
    (model, synthesis.suite)
}

fn every_missing_is_required(suite: &ConformanceSuite) {
    let (created, decided) = inputs(suite);
    let requires = list(created.get("requires"));
    let missing = list(decided.get("missing"));
    assert!(
        !requires.is_empty() && !missing.is_empty(),
        "{created:?} {decided:?}"
    );
    assert!(
        missing.iter().all(|element| requires.contains(element)),
        "{requires:?} {missing:?}"
    );
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

#[test]
fn a_narrowed_branch_beside_a_sibling_reading_the_row_is_witnessed_on_its_guard() {
    let (model, suite) = witnessed("List<String>", WITH_SIBLING);
    every_missing_is_required(&suite);
    let statuses = run(&suite, ignoring_the_narrowing(&model));
    assert!(!failing(&statuses).is_empty(), "{statuses:#?}");
}

/// The guard is an `exists` over the input list: the one element the stored list holds is neither
/// first nor last of three, so a target reading only the first or only the last element fails.
#[test]
fn a_narrowed_branch_with_no_own_guard_is_witnessed_on_its_guard() {
    let (_, suite) = witnessed("List<String>", NO_OWN_WHEN);
    let (created, decided) = inputs(&suite);
    let requires = list(created.get("requires"));
    let missing = list(decided.get("missing"));
    let matching: Vec<usize> = missing
        .iter()
        .enumerate()
        .filter(|(_, element)| requires.contains(element))
        .map(|(at, _)| at)
        .collect();
    assert!(
        missing.len() >= 3
            && matching.len() == 1
            && matching[0] != 0
            && matching[0] != missing.len() - 1,
        "{requires:?} {missing:?}"
    );
}

#[test]
fn the_only_branch_reading_the_row_is_checked_against_its_guard() {
    let (model, suite) = witnessed("List<String>", ONLY_READER);
    every_missing_is_required(&suite);
    let statuses = run(&suite, ignoring_the_narrowing(&model));
    assert!(!failing(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn a_scalar_comparison_with_the_input_is_witnessed_on_its_guard() {
    let (model, suite) = witnessed("String", SCALAR);
    let (created, decided) = inputs(&suite);
    assert_eq!(created.get("requires"), decided.get("missing"));
    let statuses = run(&suite, ignoring_the_narrowing(&model));
    assert!(!failing(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn a_guard_no_row_and_input_satisfy_is_refused_and_written_no_scenario() {
    let model = ir(&model("String", UNSATISFIABLE));
    let synthesis = synthesize(&model);
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .all(|id| id.to_string() != REFUSED),
        "a scenario was written"
    );
    assert!(
        synthesis.refusals.iter().any(|refusal| matches!(
            refusal.cause,
            RefusalCause::GuardUnsatisfiable { .. }
        ) && refusal.to_string().contains("refused")),
        "{:#?}",
        synthesis.refusals
    );
}
