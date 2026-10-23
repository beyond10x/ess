//! A `when_subject` guard tests whether a stored field is an element of a `List` input, and its
//! negation, with the existing quantifier `exists: {in: input.<list>, as: x, that: <field> == x}`
//! (story:guard-tests-list-membership).
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::synthesize, ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MEMBER: &str = "{exists: {in: input.frontier_actions, as: candidate, that: {candidate: {eq: {fact: action}}}}}";

/// The walk domain, its `off-frontier` refusal guarded by `guard` and its list input typed `list`.
fn walk(guard: &str, list: &str) -> String {
    format!(
        r"format: ess/30
system: demo
version: v1
domain: demo.walk
summary: A walker steps only to an action the frontier lists.
types:
  - {{name: demo.walk.WalkerId, kind: newtype, of: Uuid}}
entities:
  - name: demo.walk.Walker
    identity: {{name: id, type: demo.walk.WalkerId}}
    fields:
      - {{name: action, type: String}}
    lifecycle:
      initial: Walking
      states: [Walking, Done]
      terminal: [Done]
      transitions:
        - {{name: finish, from: [Walking], to: Done}}
actors:
  - name: demo.walk.Operator
    may: [demo.walk.Start, demo.walk.Step]
errors:
  - {{name: demo.walk.UnknownWalker, summary: No such walker., fields: []}}
  - {{name: demo.walk.NotWalking, summary: The walker is done., fields: []}}
  - {{name: demo.walk.OffFrontier, summary: The frontier does not list the action., fields: []}}
events:
  - name: demo.walk.Started
    fields: [{{name: id, type: demo.walk.WalkerId}}]
  - name: demo.walk.Stepped
    fields: [{{name: id, type: demo.walk.WalkerId}}]
commands:
  - name: demo.walk.Start
    input:
      - {{name: action, type: String}}
    outcomes:
      - name: started
        creates: demo.walk.Walker
        instance: id
        sets: {{action: input.action}}
        emits: [demo.walk.Started]
        payload: {{demo.walk.Started: {{id: {{generated: true}}}}}}
  - name: demo.walk.Step
    input:
      - {{name: id, type: demo.walk.WalkerId}}
      - {{name: frontier_actions, type: {list}}}
    outcomes:
      - name: unknown-walker
        unknown_instance: true
        error: demo.walk.UnknownWalker
      - name: not-walking
        wrong_state: true
        error: demo.walk.NotWalking
      - name: off-frontier
        when_subject: {{predicate: {guard}}}
        error: demo.walk.OffFrontier
      - name: stepped
        moves: demo.walk.Walker.finish
        instance: id
        emits: [demo.walk.Stepped]
        payload: {{demo.walk.Stepped: {{id: input.id}}}}
views:
  - name: demo.walk.Walkers
    source: demo.walk.Walker
    consistency: read_your_writes
    fields:
      - {{name: id, type: demo.walk.WalkerId}}
      - {{name: action, type: String}}
      - {{name: state, type: demo.walk.Walker.State}}
"
    )
}

fn not_member() -> String {
    format!("{{not: {MEMBER}}}")
}

fn assembled(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("walk.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = assembled(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn requires(scenario: &ConformanceScenario, outcome: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

/// For each scenario requiring `outcome`: the stored action and the list the `Step` sent.
fn witnessed(text: &str, outcome: &str) -> Vec<(Node, Vec<Node>)> {
    let suite = synthesize(&ir(text)).suite;
    suite
        .scenarios
        .values()
        .filter(|scenario| requires(scenario, outcome))
        .filter_map(|scenario| {
            let start = inputs(scenario, "demo.walk.Start");
            let step = inputs(scenario, "demo.walk.Step");
            let action = start.last()?.get("action")?.as_literal()?.clone();
            let Some(Node::Seq(list)) = step.last()?.get("frontier_actions")?.as_literal().cloned()
            else {
                return None;
            };
            Some((action, list))
        })
        .collect()
}

#[test]
fn membership_and_its_negation_validate() {
    for guard in [MEMBER.to_owned(), not_member()] {
        let text = walk(&guard, "List<String>");
        assert!(
            assembled(&text).is_ok(),
            "{guard}: {:?}",
            assembled(&text).err()
        );
        let _ = ir(&text);
    }
}

#[test]
fn an_element_type_other_than_the_field_type_is_refused() {
    let text = walk(&not_member(), "List<Integer>");
    let error = assembled(&text).map_or_else(|error| error, |_| panic!("must be refused"));
    assert!(error.contains("[type_mismatch]"), "{error}");
}

/// Whether one of `found` sends a list holding the stored action behind another element, so a
/// target reading only the first element answers it wrongly.
fn holds_behind_another(found: &[(Node, Vec<Node>)]) -> bool {
    found
        .iter()
        .any(|(action, list)| list.contains(action) && list.first() != Some(action))
}

/// Whether one of `found` sends a non-empty list without the stored action, so a target that
/// ignores the list, or tests only that it is empty, answers it wrongly.
fn lacks_non_empty(found: &[(Node, Vec<Node>)]) -> bool {
    found
        .iter()
        .any(|(action, list)| !list.is_empty() && !list.contains(action))
}

#[test]
fn each_side_of_the_negated_guard_is_witnessed_with_a_non_empty_list() {
    let text = walk(&not_member(), "List<String>");
    let refused = witnessed(&text, "demo.walk.Step/off-frontier");
    assert!(lacks_non_empty(&refused), "off-frontier: {refused:#?}");
    let taken = witnessed(&text, "demo.walk.Step/stepped");
    assert!(holds_behind_another(&taken), "stepped: {taken:#?}");
}

#[test]
fn each_side_of_the_guard_is_witnessed_with_a_non_empty_list() {
    let text = walk(MEMBER, "List<String>");
    let refused = witnessed(&text, "demo.walk.Step/off-frontier");
    assert!(holds_behind_another(&refused), "off-frontier: {refused:#?}");
    let taken = witnessed(&text, "demo.walk.Step/stepped");
    assert!(lacks_non_empty(&taken), "stepped: {taken:#?}");
}

/// The compact spellings an adopter tries are refused naming the quantifier, and the form named,
/// applied as written, validates.
#[test]
fn the_compact_membership_spelling_names_a_form_that_validates() {
    for (written, hint) in [
        (
            "\"action in input.frontier_actions\"",
            "exists: {in: input.frontier_actions, as: x, that: action == x}",
        ),
        (
            "\"action not in input.frontier_actions\"",
            "not: {exists: {in: input.frontier_actions, as: x, that: action == x}}",
        ),
    ] {
        let error = assembled(&walk(written, "List<String>"))
            .map_or_else(|error| error, |_| panic!("{written} must be refused"));
        assert!(
            error.contains("a predicate is either a comparison"),
            "{written}: {error}"
        );
        assert!(error.contains(hint), "{written} hints `{hint}`: {error}");
        let applied = walk(&format!("{{{hint}}}"), "List<String>");
        if let Err(error) = assembled(&applied) {
            panic!("{written}: the hinted `{hint}` is refused:\n{error}");
        }
        let _ = ir(&applied);
    }
}

#[test]
fn the_interpreted_target_passes_every_scenario() {
    for guard in [MEMBER.to_owned(), not_member()] {
        let model = ir(&walk(&guard, "List<String>"));
        let suite = synthesize(&model).suite;
        let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
        let report = ess_conformance::Runner::for_suite(&suite)
            .run_admitted(
                &admitted,
                &ess_conformance::interpret::Interpreted::for_model(model),
            )
            .into_report();
        assert_ne!(report.scenarios.len(), 0, "{guard}: no scenario ran");
        for run in &report.scenarios {
            assert_eq!(
                run.status,
                ess_conformance::report::Status::Passed,
                "{guard}: {}: {:?}",
                run.scenario,
                run.checks
            );
        }
    }
}
