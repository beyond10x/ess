//! Adversary pass 1 on beyond10x/ess#413 part B (explicit synthesis seeds, suite/42–43).
//!
//! Each case drives the writer or the reader from `docs/design/synthesis-seeds.md`: a seeded suite
//! the writer emits must be one its own readers admit and a healthy target passes, and a seeded row
//! never shares its identity with another step of the scenario it is established in.
mod support_seeds;

use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::{
    authored,
    coverage::{Origins, Scope},
    report::Status,
    scenario::ScenarioStep,
    synthesize::{synthesize, synthesize_with_seeds},
    AdmittedSuite, ConformanceSuite, Runner,
};
use support_seeds::*;

/// The issue model with two attribute-free actors, each granted both commands: an ordinary actor
/// declaration, which seeded synthesis admits (only caller *attributes* are refused).
fn with_two_actors() -> String {
    format!(
        "{COUNTER}actors:
  - name: counter.model.Operator
    may: [counter.model.Create, counter.model.Authorize]
  - name: counter.model.Auditor
    may: [counter.model.Create, counter.model.Authorize]
"
    )
}

/// The issue model with a counter created under an identity its input supplies, so synthesis
/// itself chooses the literal identities its arranged rows carry.
fn supplied_identity() -> String {
    let changed = COUNTER
        .replacen(
            "  - name: counter.model.Create\n    input: []",
            "  - name: counter.model.Create\n    input:\n      - {name: id, type: Uuid}",
            1,
        )
        .replacen(
            "counter.model.Created: {id: {generated: true}}",
            "counter.model.Created: {id: input.id}",
            1,
        );
    assert_ne!(changed, COUNTER, "precondition: the model changed");
    changed
}

fn statuses(suite: &ConformanceSuite, target: &Counters<'_>) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// Every (entity, identity) a scenario establishes more than once.
fn established_twice(suite: &ConformanceSuite) -> Vec<String> {
    let mut twice = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let mut seen = BTreeSet::new();
        for step in &scenario.steps {
            if let ScenarioStep::EstablishEntity {
                entity, identity, ..
            } = step
            {
                let key = format!("{entity} {}", serde_json::to_string(identity).unwrap());
                if !seen.insert(key.clone()) {
                    twice.push(format!("{id}: {key}"));
                }
            }
        }
    }
    twice
}

/// The design: "A seeded row's identity never collides with another row the same scenario
/// establishes". The cross-caller pass appends a second, renamed run of a scenario for a model
/// with two granted actors; the seeded row must not be established twice.
#[test]
fn a_cross_caller_copy_never_establishes_a_seeded_row_twice() {
    let ir = ir_of(&with_two_actors());
    let synthesis = synthesize_with_seeds(&ir, &admitted(&ir, &issue_selections())).unwrap();
    let twice = established_twice(&synthesis.suite);
    assert_eq!(twice.len(), 0, "{twice:#?}");
}

/// The design's writer and reader checks: what the writer emits, the Rust reader admits. A
/// generated scenario establishing a row no application binds is refused by the reader, so the
/// writer must never emit one.
#[test]
fn a_seeded_suite_with_two_granted_actors_is_admitted_by_its_own_reader() {
    let ir = ir_of(&with_two_actors());
    let synthesis = synthesize_with_seeds(&ir, &admitted(&ir, &issue_selections())).unwrap();
    let json = synthesis.suite.to_canonical_json().unwrap();
    if let Err(error) = AdmittedSuite::from_json(&json) {
        panic!("the writer's own seeded suite is refused by the reader: {error}");
    }
}

/// Acceptance: "Healthy target passes the seeded suite" — for the same model with two granted
/// actors, which the healthy counter does not distinguish.
#[test]
fn a_healthy_counter_passes_the_seeded_suite_of_a_model_with_two_granted_actors() {
    let ir = ir_of(&with_two_actors());
    let synthesis = synthesize_with_seeds(&ir, &admitted(&ir, &issue_selections())).unwrap();
    let failed: Vec<(String, Status)> =
        statuses(&synthesis.suite, &Counters::new(&ir, Fault::None))
            .into_iter()
            .filter(|(_, status)| *status != Status::Passed)
            .collect();
    assert_eq!(failed.len(), 0, "{failed:#?}");
}

/// Every UUID-shaped literal anywhere in `suite`'s scenarios.
fn uuid_literals(suite: &ConformanceSuite) -> BTreeSet<String> {
    fn walk(value: &serde_json::Value, out: &mut BTreeSet<String>) {
        match value {
            serde_json::Value::String(text)
                if text.len() == 36
                    && text.bytes().enumerate().all(|(at, byte)| {
                        if [8, 13, 18, 23].contains(&at) {
                            byte == b'-'
                        } else {
                            byte.is_ascii_hexdigit()
                        }
                    }) =>
            {
                out.insert(text.clone());
            }
            serde_json::Value::Array(items) => items.iter().for_each(|item| walk(item, out)),
            serde_json::Value::Object(members) => {
                members.values().for_each(|member| walk(member, out));
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    walk(&serde_json::to_value(&suite.scenarios).unwrap(), &mut out);
    out
}

/// The design: "A seeded row's identity never collides with another row the same scenario
/// establishes", and the seed is never renamed. For each literal identity the seeded issue suite
/// sends anywhere, a seed carrying that identity is either not applied or, where applied, its
/// identity appears in exactly one step of its scenario: its own `establish_entity`.
#[test]
fn a_seeded_identity_appears_in_no_other_step_of_its_scenario() {
    for model in [COUNTER.to_owned(), supplied_identity()] {
        let ir = ir_of(&model);
        let literals = uuid_literals(
            &synthesize_with_seeds(&ir, &admitted(&ir, &issue_selections()))
                .unwrap()
                .suite,
        );
        assert_ne!(literals.len(), 0);
        let mut shared = Vec::new();
        let runs = literals.iter().flat_map(|literal| {
            let at_max = select(
                authored::Source::new(
                    "max.yaml",
                    seed_document("counter-at-max", "at-max", literal, &MAX.to_string()),
                ),
                "at-max",
            );
            let below_max = select(
                authored::Source::new(
                    "below.yaml",
                    seed_document(
                        "counter-below-max",
                        "below-max",
                        literal,
                        &(MAX - 1).to_string(),
                    ),
                ),
                "below-max",
            );
            [
                (literal != BELOW_MAX_ID)
                    .then(|| vec![at_max, select(below_source(), "below-max")]),
                (literal != AT_MAX_ID).then(|| vec![select(max_source(), "at-max"), below_max]),
            ]
            .into_iter()
            .flatten()
        });
        for selections in runs {
            let synthesis = synthesize_with_seeds(&ir, &admitted(&ir, &selections)).unwrap();
            let record = synthesis.suite.provenance.synthesis_seeds.as_ref().unwrap();
            for application in &record.applications {
                let steps = &synthesis.suite.scenarios[&application.scenario].steps;
                let selection = record
                    .selections
                    .iter()
                    .find(|selection| selection.instance == application.instance)
                    .unwrap();
                let wanted = serde_json::to_string(&selection.identity).unwrap();
                let holding: Vec<usize> = steps
                    .iter()
                    .enumerate()
                    .filter(|(_, step)| serde_json::to_string(step).unwrap().contains(&wanted))
                    .map(|(at, _)| at)
                    .collect();
                if holding != vec![application.establish_step] {
                    shared.push(format!(
                        "{}: identity {wanted} in steps {holding:?}",
                        application.scenario
                    ));
                }
            }
        }
        assert_eq!(shared.len(), 0, "{shared:#?}");
    }
}

/// The design: a coverage selection "keeps the parent provenance unchanged; there, an application
/// may name a scenario the selection filter moved outside". The selected document is a suite/43
/// the Rust reader must admit from its own bytes.
#[test]
fn a_selected_seeded_coverage_suite_is_admitted_from_its_own_bytes() {
    let ir = ir();
    let seeds = admitted(&ir, &issue_selections());
    let input = ess_conformance::coverage_build::build_with_seeds(
        &ir,
        &[],
        Scope::System,
        Origins::Generated,
        &seeds,
    )
    .unwrap();
    let selected = input.select(&[id(STALE)]).unwrap();
    assert!(selected
        .selected()
        .original_json()
        .contains("\"applications\""));
    // A selected child is admitted through its `ess-conformance-input/1` carrier, parents included.
    let carrier = selected.document().to_canonical_json().unwrap();
    if let Err(error) = ess_conformance::coverage::AdmittedInput::from_json(&carrier) {
        panic!("the selected seeded coverage carrier is refused by the reader: {error}");
    }
}

/// Seed-free synthesis of the actors model is the ordinary path, unchanged by the unit; kept here
/// as the control the actor cases above are read against.
#[test]
fn control_the_seed_free_actor_model_is_admitted_and_passes() {
    let ir = ir_of(&with_two_actors());
    let plain = synthesize(&ir);
    let json = plain.suite.to_canonical_json().unwrap();
    AdmittedSuite::from_json(&json).unwrap();
    let failed: Vec<(String, Status)> = statuses(&plain.suite, &Counters::new(&ir, Fault::None))
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert_eq!(failed.len(), 0, "{failed:#?}");
}

/// The scenarios a synthesis refuses, by id.
fn refused_ids(synthesis: &ess_conformance::synthesize::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .filter_map(|refusal| refusal.scenario.as_ref().map(ToString::to_string))
        .collect()
}

/// The design: "A seed is tried only at the exact obligation whose ordinary attempt was unmet", and
/// "Only the obligation actually built loses its refusal occurrence". A row that is on the
/// accepting side of the limit but is not the limit's nearest value does not witness the limit
/// side: both issue refusals stand, and no use is recorded.
#[test]
fn a_seed_short_of_the_limit_edge_discharges_neither_issue_obligation() {
    let ir = ir();
    for (name, revision) in [("five", 5_i64), ("two-below", MAX - 2)] {
        let row = select(
            authored::Source::new(
                format!("{name}.yaml"),
                seed_document(
                    &format!("counter-{name}"),
                    name,
                    "00000000-0000-4000-8000-00000000d001",
                    &revision.to_string(),
                ),
            ),
            name,
        );
        let synthesis = synthesize_with_seeds(&ir, &admitted(&ir, &[row])).unwrap();
        assert_eq!(
            refused_ids(&synthesis),
            vec![EXHAUSTED.to_owned(), AUTHORIZED.to_owned()],
            "{name}: {:#?}",
            rendered(&synthesis)
        );
        let record = synthesis.suite.provenance.synthesis_seeds.as_ref().unwrap();
        assert_eq!(record.applications.len(), 0, "{name}: {record:#?}");
    }
}

/// Every scenario of `suite` the model interpreter does not pass.
fn interpreted_failures(
    ir: &ess_compiler::ir::EssIr,
    suite: &ConformanceSuite,
) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir.clone()),
        )
        .into_report()
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// Acceptance: "Healthy target passes the seeded suite". The model interpreter is the
/// specification itself selected as the implementation; it passes the seed-free suite of the issue
/// model (the control), and must pass the seeded one.
#[test]
fn the_interpreted_issue_model_passes_its_seeded_suite() {
    let ir = ir();
    let control = interpreted_failures(&ir, &synthesize(&ir).suite);
    assert_eq!(control.len(), 0, "control: {control:#?}");
    let failed = interpreted_failures(&ir, &seeded(&ir).suite);
    assert_eq!(failed.len(), 0, "{failed:#?}");
}

/// Acceptance: "a target with the CAS or the overflow guard removed fails" the seeded suite. Here
/// the faulty target is not a hand-written fixture but the model interpreter holding a mutant of
/// the issue model, run against the suite seeded from the original model.
#[test]
fn the_interpreted_mutants_of_the_issue_model_each_fail_the_seeded_suite() {
    let ir = ir();
    let suite = seeded(&ir).suite;
    let exhausted = "      - name: revision-exhausted
        when_subject: {predicate: revision >= 9223372036854775807}
        error: counter.model.Exhausted
";
    let stale = "      - name: stale
        when_subject: {predicate: revision != input.expected_revision}
        error: counter.model.Stale
";
    assert!(COUNTER.contains(exhausted) && COUNTER.contains(stale));
    let mutants = [
        ("overflow-guard-removed", COUNTER.replacen(exhausted, "", 1)),
        (
            "overflow-guard-one-early",
            COUNTER.replacen(
                "revision >= 9223372036854775807}",
                "revision >= 9223372036854775806}",
                1,
            ),
        ),
        ("cas-removed", COUNTER.replacen(stale, "", 1)),
    ];
    let mut survived = Vec::new();
    for (name, text) in mutants {
        assert_ne!(text, COUNTER, "{name}: the mutant changed nothing");
        let mutant = ir_of(&text);
        if interpreted_failures(&mutant, &suite).is_empty() {
            survived.push(name);
        }
    }
    assert_eq!(
        survived.len(),
        0,
        "mutants passing the seeded suite: {survived:?}"
    );
}
