//! Adversary cases for beyond10x/ess#727 (a narrowed external branch relating input to stored
//! values): acceptance 5 for the list-form "only reader" suite, a target that drops only the subject
//! guard (keeping the branch's own `when:`), a scenario-instance input read as its bare instance
//! name, a generated `Uuid` identity, and a quantifier over the stored list.
use std::collections::BTreeMap;

use ess_compiler::ir::ResolvedCondition;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, RefusalCause};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REFUSED: &str = "example.rollout.DecideAdmission/outcome/refused";

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

/// The guard compares a stored copy of the row's own identity with the identity the caller sends:
/// true on every row a run can hold.
const SELF_COPY: &str = "  outcomes:
  - name: refused
    external: the row was filed under the identity it is addressed by
    when_subject:
      predicate: origin == input.id
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

/// Every required capability is reported missing: a quantifier over the stored list.
const OVER_STORED: &str = "  outcomes:
  - name: refused
    external: every required capability has no implementation
    when: missing.count >= 1
    when_subject:
      predicate:
        forall: {in: requires, as: c, that: {exists: {in: input.missing, as: m, that: m == c}}}
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

/// `id_type` is the identity's type; `generated` publishes it as a generated value instead of the
/// creator's input; `kind` is the type of `requires` and `missing`.
fn model(kind: &str, outcomes: &str, id_type: &str, generated: bool) -> String {
    let (creator_id, created_id) = if generated {
        (String::new(), "{generated: true}")
    } else {
        (format!("  - {{name: id, type: {id_type}}}\n"), "input.id")
    };
    format!(
        "format: ess/35
system: example
version: v1
domain: example.rollout
entities:
  - name: example.rollout.Rollout
    identity: {{name: id, type: {id_type}}}
    fields:
      - {{name: requires, type: {kind}}}
      - {{name: origin, type: String}}
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
    fields: [{{name: id, type: {id_type}}}]
  - name: example.rollout.Refused
    fields: [{{name: id, type: {id_type}}}]
  - name: example.rollout.Admitted
    fields: [{{name: id, type: {id_type}}}]
  - name: example.rollout.Stopped
    fields: [{{name: id, type: {id_type}}}]
errors:
  - name: example.rollout.AdmissionIncomplete
commands:
- name: example.rollout.CreateRollout
  input:
{creator_id}  - {{name: requires, type: {kind}}}
{origin_input}  outcomes:
  - name: created
    creates: example.rollout.Rollout
    instance: id
    sets: {{requires: input.requires, origin: {origin}}}
    emits: [example.rollout.Created]
    payload:
      example.rollout.Created: {{id: {created_id}}}
- name: example.rollout.DecideAdmission
  input:
  - {{name: id, type: {id_type}}}
  - {{name: missing, type: {kind}}}
  - {{name: bindings, type: List<String>}}
{outcomes}- name: example.rollout.StopRollout
  input:
  - {{name: id, type: {id_type}}}
  outcomes:
  - name: stopped
    moves: example.rollout.Rollout.stop
    instance: id
    emits: [example.rollout.Stopped]
    payload:
      example.rollout.Stopped: {{id: input.id}}
",
        origin = if generated {
            "input.origin"
        } else {
            "input.id"
        },
        origin_input = if generated {
            "  - {name: origin, type: String}\n"
        } else {
            ""
        },
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

/// A target that skips only the `when_subject:` narrowing and still checks the branch's own
/// `when:`: the defect acceptance 5 names, no broader.
fn ignoring_only_the_subject_guard(model: &EssIr) -> EssIr {
    model.with_commands_rewritten(|command| {
        let mut command = command.clone();
        for outcome in &mut command.outcomes {
            let replaced = match &outcome.condition {
                ResolvedCondition::ExternalSubject { cause, input, .. } => Some(match input {
                    Some(predicate) => ResolvedCondition::ExternalWhen {
                        cause: cause.clone(),
                        predicate: predicate.clone(),
                    },
                    None => ResolvedCondition::External {
                        cause: cause.clone(),
                    },
                }),
                _ => None,
            };
            if let Some(condition) = replaced {
                outcome.condition = condition;
            }
        }
        command
    })
}

fn suite_of(text: &str) -> (EssIr, ess_conformance::synthesize::Synthesis) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    (model, synthesis)
}

fn ignoring_target_fails(text: &str) {
    let (model, synthesis) = suite_of(text);
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == REFUSED),
        "no {REFUSED}: {:#?}",
        synthesis.refusals
    );
    let correct = run(&synthesis.suite, model.clone());
    assert!(failing(&correct).is_empty(), "{correct:#?}");
    let statuses = run(&synthesis.suite, ignoring_only_the_subject_guard(&model));
    assert!(
        !failing(&statuses).is_empty(),
        "a target ignoring the subject guard passes every scenario: {statuses:#?}\n{:#?}",
        synthesis.suite.scenarios
    );
}

/// Acceptance 5, suite of 3 (list form): the story's own acceptance names this suite.
#[test]
fn adv_acceptance5_only_reader_list_form_catches_a_target_ignoring_the_subject_guard() {
    ignoring_target_fails(&model("List<String>", ONLY_READER, "String", false));
}

/// Acceptance 5, suite of 1, with a mutant that keeps the own `when:` (the unit's mutant drops it).
#[test]
fn adv_acceptance5_with_sibling_catches_a_target_keeping_its_own_when() {
    ignoring_target_fails(&model("List<String>", WITH_SIBLING, "String", false));
}

/// The scalar form with the most common identity shape: a generated `Uuid`.
#[test]
fn adv_scalar_form_with_a_generated_uuid_identity_catches_the_ignoring_target() {
    ignoring_target_fails(&model("String", SCALAR, "Uuid", true));
}

/// A scenario-instance input is read as its bare instance name, which is not the identity the
/// target holds for it: a guard true on every row is judged false, and the sibling's scenario
/// forces the narrowed branch where a correct target takes it.
#[test]
fn adv_an_identity_input_is_not_read_as_its_instance_name() {
    let (model, synthesis) = suite_of(&model("String", SELF_COPY, "String", false));
    let statuses = run(&synthesis.suite, model);
    assert!(
        failing(&statuses).is_empty(),
        "the interpreted target fails the synthesized suite: {statuses:#?}\n{:#?}",
        synthesis.suite.scenarios
    );
}

/// A quantifier over the stored list relating it to the input (the story's fit review claims it).
#[test]
fn adv_a_quantifier_over_the_stored_list_is_witnessed() {
    let (model, synthesis) = suite_of(&model("List<String>", OVER_STORED, "String", false));
    let refusals: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| !matches!(refusal.cause, RefusalCause::RefusalUndeclared { .. }))
        .collect();
    assert!(refusals.is_empty(), "{refusals:#?}");
    let statuses = run(&synthesis.suite, model);
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
}
