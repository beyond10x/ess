//! Explicit synthesis seeds (beyond10x/ess#413, part B; `docs/design/synthesis-seeds.md`).
//!
//! The issue's minimal compare-and-swap counter cannot reach `revision == i64::MAX` or the row one
//! below it by bounded command arrangement. An explicitly selected authored setup row supplies that
//! initial state; synthesis still chooses the outcome, grounds the expected revision from the row,
//! invokes the real command and asserts what the branch promises. Ordinary arrangement is always
//! tried first, and seed-free synthesis keeps its bytes.
mod support_seeds;

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_conformance::{
    authored,
    coverage::{Origins, Scope},
    coverage_build::{self, CoverageSource},
    report::{ConformanceStatus, Status},
    scenario::{ScenarioStep, ScenarioValue},
    synthesize::{
        synthesize, synthesize_for_with_seeds, synthesize_with_seeds, AdmittedSeeds,
        SeedAdmissionError, SeedSelection,
    },
    AdmittedSuite, ConformanceSuite, Runner,
};
use ess_primitives::node::Node;
use support_seeds::*;

// ---- the issue fixture -------------------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)] // One scenario walk per obligation, kept together.
fn seeds_discharge_both_issue_obligations_with_real_setup_command_and_assertion_steps() {
    let ir = ir();
    let synthesis = seeded(&ir);
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", rendered(&synthesis));
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/42"
    );

    // revision-exhausted: established at MAX, sent the row's own revision, refused, unchanged.
    let exhausted = &synthesis.suite.scenarios[&id(EXHAUSTED)].steps;
    let (setup, instance, command) = segment(exhausted, AT_MAX_ID);
    let ScenarioStep::EstablishEntity { fields, state, .. } = &exhausted[setup] else {
        unreachable!()
    };
    assert_eq!(fields.get("revision"), Some(&integer(MAX)));
    assert_eq!(state.to_string(), "Active");
    let ScenarioStep::ExecuteCommand {
        command: sent,
        input,
        ..
    } = &exhausted[command]
    else {
        unreachable!()
    };
    assert_eq!(sent.to_string(), "counter.model.Authorize");
    assert_eq!(
        input.get("expected_revision"),
        Some(&ScenarioValue::literal(integer(MAX))),
        "the expected revision is grounded from the row, so stale is not selected"
    );
    assert!(matches!(
        &exhausted[command + 1],
        ScenarioStep::ExpectOutcome { outcome } if outcome.to_string() == "counter.model.Authorize/revision-exhausted"
    ));
    assert!(exhausted[command + 1..]
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExpectError { error, .. } if error.to_string() == "counter.model.Exhausted")));
    // The row is observed before the command and again, unchanged, after it.
    let observed = |range: &[ScenarioStep]| {
        range.iter().any(|step| {
            matches!(step, ScenarioStep::ExpectView { expectation, .. }
                if format!("{expectation:?}").contains(&instance.to_string())
                    && format!("{expectation:?}").contains("9223372036854775807"))
        })
    };
    assert!(observed(&exhausted[setup + 1..command]), "{exhausted:#?}");
    assert!(observed(&exhausted[command + 1..]), "{exhausted:#?}");

    // authorized: the ordinary witness stands; one MAX−1 segment is added where the scenario's
    // further rows go, and reaches exactly MAX. Every ordinary step keeps its place and bytes.
    let plain = synthesize(&ir);
    let ordinary = &plain.suite.scenarios[&id(AUTHORIZED)].steps;
    let authorized = &synthesis.suite.scenarios[&id(AUTHORIZED)].steps;
    let (setup, instance, command) = segment(authorized, BELOW_MAX_ID);
    let added = authorized.len() - ordinary.len();
    assert_eq!(
        &authorized[..setup],
        &ordinary[..setup],
        "the successful ordinary witness keeps its exact steps"
    );
    assert_eq!(
        &authorized[setup + added..],
        &ordinary[setup..],
        "the ordinary steps after the seeded segment keep their exact bytes"
    );
    assert!(command < setup + added);
    let ScenarioStep::ExecuteCommand { input, .. } = &authorized[command] else {
        unreachable!()
    };
    assert_eq!(
        input.get("expected_revision"),
        Some(&ScenarioValue::literal(integer(MAX - 1)))
    );
    assert!(matches!(
        &authorized[command + 1],
        ScenarioStep::ExpectOutcome { outcome } if outcome.to_string() == "counter.model.Authorize/authorized"
    ));
    assert!(authorized[command + 1..].iter().any(|step| {
        matches!(step, ScenarioStep::ExpectView { expectation, .. }
            if format!("{expectation:?}").contains(&instance.to_string())
                && format!("{expectation:?}").contains("9223372036854775807"))
    }));

    // The provenance names both sources, both selections and exactly these two uses.
    let seeds = synthesis.suite.provenance.synthesis_seeds.as_ref().unwrap();
    assert_eq!(seeds.sources.len(), 2);
    assert_eq!(seeds.selections.len(), 2);
    let uses: Vec<(String, String, usize, usize)> = seeds
        .applications
        .iter()
        .map(|application| {
            (
                application.instance.to_string(),
                application.scenario.to_string(),
                application.establish_step,
                application.command_step,
            )
        })
        .collect();
    let (exhausted_setup, _, exhausted_command) = segment(exhausted, AT_MAX_ID);
    let (authorized_setup, _, authorized_command) = segment(authorized, BELOW_MAX_ID);
    assert_eq!(
        uses,
        vec![
            (
                "below-max".to_owned(),
                AUTHORIZED.to_owned(),
                authorized_setup,
                authorized_command
            ),
            (
                "at-max".to_owned(),
                EXHAUSTED.to_owned(),
                exhausted_setup,
                exhausted_command
            ),
        ]
    );
    // Exact i64 survives the persisted suite.
    let json = synthesis.suite.to_canonical_json().unwrap();
    assert!(json.contains("9223372036854775807"));
    assert!(json.contains("9223372036854775806"));
    let back = AdmittedSuite::from_json(&json).unwrap();
    assert_eq!(back.suite(), &synthesis.suite);
}

#[test]
fn without_seeds_the_issue_suite_keeps_the_base_bytes_and_refusals() {
    let ir = ir();
    let plain = synthesize(&ir);
    let json = plain.suite.to_canonical_json().unwrap();
    assert_eq!(sha256(&json), BASE_SUITE_SHA256);
    assert_eq!(
        plain.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert!(!json.contains("synthesis_seeds"));
    let refused: Vec<String> = plain
        .refusals
        .iter()
        .map(|refusal| refusal.scenario.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(refused, vec![EXHAUSTED.to_owned(), AUTHORIZED.to_owned()]);
    // The empty admitted set is the seed-free path itself.
    let empty = synthesize_with_seeds(&ir, &AdmittedSeeds::empty()).unwrap();
    assert_eq!(empty, plain);
}

#[test]
fn selector_order_is_irrelevant_and_row_bytes_move_the_suite() {
    let ir = ir();
    let forward = seeded(&ir).suite.to_canonical_json().unwrap();
    let mut reversed = issue_selections();
    reversed.reverse();
    let backward = synthesize_with_seeds(&ir, &admitted(&ir, &reversed))
        .unwrap()
        .suite
        .to_canonical_json()
        .unwrap();
    assert_eq!(forward, backward);
    let other = vec![
        select(
            authored::Source::new(
                "max.yaml",
                seed_document(
                    "counter-at-max",
                    "at-max",
                    "00000000-0000-4000-8000-00000000b001",
                    &MAX.to_string(),
                ),
            ),
            "at-max",
        ),
        select(below_source(), "below-max"),
    ];
    let moved = synthesize_with_seeds(&ir, &admitted(&ir, &other))
        .unwrap()
        .suite
        .to_canonical_json()
        .unwrap();
    assert_ne!(forward, moved);
}

// ---- ordinary first ----------------------------------------------------------------------------

#[test]
fn an_unused_seed_is_recorded_adds_no_scenario_and_replaces_no_ordinary_witness() {
    let ir = ir();
    let with_two = seeded(&ir);
    // A row at revision 5 is closer to every ordinary obligation and satisfies none still unmet.
    let mut selections = issue_selections();
    selections.push(select(
        authored::Source::new(
            "five.yaml",
            seed_document(
                "counter-at-five",
                "at-five",
                "00000000-0000-4000-8000-00000000a005",
                "5",
            ),
        ),
        "at-five",
    ));
    let with_three = synthesize_with_seeds(&ir, &admitted(&ir, &selections)).unwrap();
    assert_eq!(with_three.suite.scenarios, with_two.suite.scenarios);
    assert_eq!(with_three.refusals, with_two.refusals);
    let seeds = with_three
        .suite
        .provenance
        .synthesis_seeds
        .as_ref()
        .unwrap();
    assert_eq!(seeds.selections.len(), 3);
    assert_eq!(seeds.sources.len(), 3);
    assert!(seeds
        .applications
        .iter()
        .all(|application| application.instance.to_string() != "at-five"));
}

#[test]
fn a_fully_ordinary_model_keeps_every_scenario_when_a_seed_is_supplied() {
    // A finite limit every bounded arrangement reaches: nothing is unmet, so nothing is seeded.
    let finite = COUNTER.replace("9223372036854775807", "2");
    let ir = ir_of(&finite);
    let plain = synthesize(&ir);
    assert_eq!(plain.refusals.len(), 0, "{:#?}", rendered(&plain));
    let seeds = admitted(
        &ir,
        &[select(
            authored::Source::new(
                "one.yaml",
                seed_document(
                    "counter-at-one",
                    "at-one",
                    "00000000-0000-4000-8000-00000000a011",
                    "1",
                ),
            ),
            "at-one",
        )],
    );
    let seeded = synthesize_with_seeds(&ir, &seeds).unwrap();
    assert_eq!(seeded.suite.scenarios, plain.suite.scenarios);
    assert_eq!(seeded.refusals, plain.refusals);
    let record = seeded.suite.provenance.synthesis_seeds.as_ref().unwrap();
    assert_eq!(record.selections.len(), 1);
    assert_eq!(record.applications.len(), 0);
    assert_eq!(
        seeded.suite.provenance.suite_version.to_string(),
        "ess-conformance/42",
        "an explicit unused selection still records its provenance"
    );
    let json = seeded.suite.to_canonical_json().unwrap();
    AdmittedSuite::from_json(&json).unwrap();
}

#[test]
fn a_seed_answering_a_different_goal_removes_no_refusal() {
    let ir = ir();
    // Only the MAX−1 row: the exhaustion obligation stays refused, the authorized side is met.
    let only_below =
        synthesize_with_seeds(&ir, &admitted(&ir, &[select(below_source(), "below-max")])).unwrap();
    let refused: Vec<String> = only_below
        .refusals
        .iter()
        .map(|refusal| refusal.scenario.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(
        refused,
        vec![EXHAUSTED.to_owned()],
        "{:#?}",
        rendered(&only_below)
    );
    // Only the MAX row: the authorized side stays refused, exhaustion is met.
    let only_max =
        synthesize_with_seeds(&ir, &admitted(&ir, &[select(max_source(), "at-max")])).unwrap();
    let refused: Vec<String> = only_max
        .refusals
        .iter()
        .map(|refusal| refusal.scenario.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(
        refused,
        vec![AUTHORIZED.to_owned()],
        "{:#?}",
        rendered(&only_max)
    );
}

#[test]
fn a_source_scenario_named_after_the_obligation_suppresses_nothing() {
    // The seed document is named and summarised as the exhaustion witness and asserts MAX, but it
    // selects only a MAX−1 row: the generated exhaustion obligation stays refused.
    let ir = ir();
    let misleading = authored::Source::new(
        "named.yaml",
        seed_document(
            "revision-exhausted",
            "below-max",
            BELOW_MAX_ID,
            &(MAX - 1).to_string(),
        ),
    );
    let synthesis =
        synthesize_with_seeds(&ir, &admitted(&ir, &[select(misleading, "below-max")])).unwrap();
    assert!(synthesis.refusals.iter().any(|refusal| refusal
        .scenario
        .as_ref()
        .map(ToString::to_string)
        .as_deref()
        == Some(EXHAUSTED)));
}

#[test]
fn a_seed_identity_never_collides_with_a_literal_identity_the_scenario_already_uses() {
    let ir = ir();
    // The exhaustion scenario first sends the command for an identity no row carries.
    let synthesis = seeded(&ir);
    let exhausted = &synthesis.suite.scenarios[&id(EXHAUSTED)].steps;
    let absent = exhausted
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => match input.get("id") {
                Some(ScenarioValue::Literal {
                    value: Node::Text(text),
                }) => Some(text.clone()),
                _ => None,
            },
            _ => None,
        })
        .expect("an absent-row witness with a literal identity");
    let colliding = authored::Source::new(
        "collide.yaml",
        seed_document("counter-collide", "collide", &absent, &MAX.to_string()),
    );
    // Alone, it refuses the obligation naming the collision; it is never renamed.
    let alone = synthesize_with_seeds(&ir, &admitted(&ir, &[select(colliding.clone(), "collide")]))
        .unwrap();
    let refusal = alone
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(EXHAUSTED)
        })
        .expect("exhaustion stays refused");
    assert!(refusal.to_string().contains("collides"), "{refusal}");
    // Beside another eligible row, that one is used.
    let both = synthesize_with_seeds(
        &ir,
        &admitted(
            &ir,
            &[select(colliding, "collide"), select(max_source(), "at-max")],
        ),
    )
    .unwrap();
    let steps = &both.suite.scenarios[&id(EXHAUSTED)].steps;
    segment(steps, AT_MAX_ID);
    assert!(!steps.iter().any(|step| matches!(step,
        ScenarioStep::EstablishEntity { identity: Node::Text(held), .. } if *held == absent)));
}

// ---- admission ---------------------------------------------------------------------------------

fn refused(ir: &EssIr, selections: &[SeedSelection]) -> SeedAdmissionError {
    match AdmittedSeeds::compile(ir, selections) {
        Ok(_) => panic!("admitted {selections:?}"),
        Err(error) => error,
    }
}

#[test]
#[allow(clippy::too_many_lines)] // One refusal per admission rule, kept together.
fn admission_refuses_every_malformed_selection_before_synthesis() {
    let ir = ir();
    assert_eq!(refused(&ir, &[]).code(), "empty-selection");
    assert_eq!(
        refused(
            &ir,
            &[
                select(max_source(), "at-max"),
                select(
                    authored::Source::new("max.yaml", below_source().text),
                    "below-max"
                ),
            ]
        )
        .code(),
        "ambiguous-source"
    );
    assert_eq!(
        refused(
            &ir,
            &[
                select(max_source(), "at-max"),
                select(max_source(), "at-max")
            ]
        )
        .code(),
        "duplicate-selector"
    );
    assert_eq!(
        refused(&ir, &[select(max_source(), "nowhere")]).code(),
        "unknown-instance"
    );
    assert_eq!(
        refused(
            &ir,
            &[select(
                authored::Source::new("/abs/max.yaml", max_source().text),
                "at-max"
            )]
        )
        .code(),
        "invalid-source"
    );
    assert_eq!(
        refused(
            &ir,
            &[select(
                authored::Source::new("broken.yaml", "type: ess-scenario/2\ndomain: nope\n"),
                "at-max"
            )]
        )
        .code(),
        "source-refused"
    );
    // An arrangement with no setup is bound by the timeline, not established.
    let without_setup = format!(
        "{}  - instance: fresh
    entity: counter.model.Counter
timeline:
  - at: '2026-01-01T00:00:00Z'
    command: counter.model.Create
    outcome: created
    capture: {{instance: fresh, event: counter.model.Created, field: id}}
",
        seed_document("counter-mixed", "at-max", AT_MAX_ID, &MAX.to_string())
            .split("assert:")
            .next()
            .unwrap()
    ) + "assert:\n  - view: counter.model.Counters\n    contains: {id: {$instance: fresh}, revision: 0}\n";
    let mixed = authored::Source::new("mixed.yaml", without_setup);
    assert!(
        AdmittedSeeds::compile(&ir, &[select(mixed.clone(), "at-max")]).is_ok(),
        "the document compiles and its setup arrangement is selectable"
    );
    assert_eq!(
        refused(&ir, &[select(mixed, "fresh")]).code(),
        "instance-without-setup"
    );
    // Conflicting duplicate qualified identities across selections.
    let twin = authored::Source::new(
        "twin.yaml",
        seed_document("counter-twin", "twin", AT_MAX_ID, "7"),
    );
    assert_eq!(
        refused(&ir, &[select(max_source(), "at-max"), select(twin, "twin")]).code(),
        "duplicate-identity"
    );
    // More than 64 selections.
    let many: Vec<SeedSelection> = (0..65)
        .map(|n| {
            select(
                authored::Source::new(
                    format!("many-{n}.yaml"),
                    seed_document(
                        &format!("counter-{n}"),
                        "row",
                        &format!("00000000-0000-4000-8000-{:012}", 100 + n),
                        &n.to_string(),
                    ),
                ),
                "row",
            )
        })
        .collect();
    assert_eq!(refused(&ir, &many).code(), "too-many-selections");
    assert_eq!(AdmittedSeeds::compile(&ir, &many[..64]).map(|_| ()), Ok(()));
    // A row the model's invariants refuse is refused by the authored compiler.
    let negative = authored::Source::new(
        "negative.yaml",
        seed_document("counter-negative", "below-zero", AT_MAX_ID, "-1"),
    );
    assert_eq!(
        refused(&ir, &[select(negative, "below-zero")]).code(),
        "source-refused"
    );
}

#[test]
fn seeds_compiled_for_another_model_are_refused() {
    let ir = ir();
    let seeds = admitted(&ir, &issue_selections());
    let other = ir_of(COUNTER);
    // Same model: admitted.
    synthesize_with_seeds(&other, &seeds).unwrap();
    let changed = ir_of(&COUNTER.replace(
        "counter.model.Invalid, fields: []",
        "counter.model.Invalid, fields: [{name: why, type: String}]",
    ));
    assert_eq!(
        synthesize_with_seeds(&changed, &seeds).unwrap_err().code(),
        "wrong-model"
    );
}

#[test]
fn a_caller_attribute_model_is_refused_rather_than_synthesized_without_its_seeds() {
    let with_callers = format!(
        "{COUNTER}actors:
  - name: counter.model.Operator
    attributes:
      - {{name: operator_id, type: Uuid}}
    may: [counter.model.Create, counter.model.Authorize]
"
    );
    let ir = ir_of(&with_callers);
    let seeds = admitted(&ir, &issue_selections());
    assert_eq!(
        synthesize_with_seeds(&ir, &seeds).unwrap_err().code(),
        "caller-model"
    );
}

#[test]
fn a_component_suite_keeps_selections_and_drops_uses_outside_it() {
    let ir = ir();
    let seeds = admitted(&ir, &issue_selections());
    // The issue model declares no component.
    assert_eq!(
        synthesize_for_with_seeds(&ir, "nowhere", &seeds)
            .unwrap_err()
            .code(),
        "unknown-component"
    );
    // Two components: the writer creates counters and owns the view, the authority authorizes.
    let ir = ir_files(&[
        (
            "system.yaml",
            &SYSTEM.replace("[counter.model]", "[counter.model, counter.audit]"),
        ),
        ("counter.yaml", COUNTER),
        ("audit.yaml", AUDIT),
        ("components.yaml", COMPONENTS),
    ]);
    let seeds = admitted(&ir, &issue_selections());
    let whole = synthesize_with_seeds(&ir, &seeds).unwrap();
    assert_eq!(whole.refusals.len(), 0, "{:#?}", rendered(&whole));
    let record = whole.suite.provenance.synthesis_seeds.as_ref().unwrap();
    assert_eq!(record.applications.len(), 2);
    for component in ["counter-writer", "counter-authority"] {
        let scoped = synthesize_for_with_seeds(&ir, component, &seeds).unwrap();
        let kept = scoped.suite.provenance.synthesis_seeds.as_ref().unwrap();
        assert_eq!(kept.selections, record.selections, "{component}");
        assert_eq!(kept.sources, record.sources, "{component}");
        for application in &kept.applications {
            assert!(
                scoped.suite.scenarios.contains_key(&application.scenario),
                "{component}: a use outside the component suite"
            );
        }
        assert_eq!(
            kept.applications.len(),
            record
                .applications
                .iter()
                .filter(|application| scoped.suite.scenarios.contains_key(&application.scenario))
                .count(),
            "{component}"
        );
        let expected = if component == "counter-writer" { 2 } else { 0 };
        assert_eq!(kept.applications.len(), expected, "{component}");
        AdmittedSuite::from_suite(&scoped.suite).unwrap();
    }
}

const COMPONENTS: &str = "components:
  - component: counter-writer
    summary: Creates and authorizes counters and serves their view.
    owns:
      domains: [counter.model]
    accepts:
      commands: [counter.model.Create, counter.model.Authorize]
    publishes:
      events: [counter.model.Created, counter.model.Authorized]
  - component: counter-authority
    summary: Owns an audit domain no counter scenario reads.
    owns:
      domains: [counter.audit]
    publishes:
      events: [counter.audit.Noted]
";

const AUDIT: &str = "domain: counter.audit
events:
  - {name: counter.audit.Noted, fields: []}
";

/// The issue model with each counter step moved by 40: the limit at 100 has its nearest values at
/// 80 and 120, both past the search's reach of 16, so each side is refused before it is searched.
fn by_forty() -> String {
    COUNTER
        .replace("{increment: 1}", "{increment: 40}")
        .replace(
            "when_subject: {predicate: revision >= 9223372036854775807}",
            "when_subject: {predicate: revision >= 100}",
        )
}

#[test]
fn a_side_past_the_search_reach_is_offered_a_seed_before_it_is_refused() {
    let ir = ir_of(&by_forty());
    let plain = synthesize(&ir);
    let past: Vec<String> = rendered(&plain)
        .into_iter()
        .filter(|refusal| refusal.contains("so the nearest value it holds"))
        .collect();
    assert_ne!(past.len(), 0, "{:#?}", rendered(&plain));
    let row = |name: &str, instance: &str, identity: &str, revision: &str| {
        select(
            authored::Source::new(
                format!("{name}.yaml"),
                seed_document(name, instance, identity, revision),
            ),
            instance,
        )
    };
    let seeds = admitted(
        &ir,
        &[
            row(
                "eighty",
                "at-eighty",
                "00000000-0000-4000-8000-00000000c080",
                "80",
            ),
            row(
                "one-twenty",
                "at-one-twenty",
                "00000000-0000-4000-8000-00000000c120",
                "120",
            ),
        ],
    );
    let seeded = synthesize_with_seeds(&ir, &seeds).unwrap();
    assert_eq!(seeded.refusals.len(), 0, "{:#?}", rendered(&seeded));
    let record = seeded.suite.provenance.synthesis_seeds.as_ref().unwrap();
    assert_ne!(record.applications.len(), 0);
    for application in &record.applications {
        let steps = &seeded.suite.scenarios[&application.scenario].steps;
        let ScenarioStep::EstablishEntity { fields, .. } = &steps[application.establish_step]
        else {
            panic!("not a setup step")
        };
        let ScenarioStep::ExecuteCommand { input, .. } = &steps[application.command_step] else {
            panic!("not a command step")
        };
        assert_eq!(
            input.get("expected_revision"),
            Some(&ScenarioValue::literal(fields["revision"].clone())),
            "the expected revision is grounded from the seeded row"
        );
    }
    // A counter exhausted at 100 and moved by 40 passes; one with no exhaustion guard fails.
    let statuses = run_with(&seeded.suite, &Counters::with(&ir, Fault::None, 100, 40));
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
    let statuses = run_with(
        &seeded.suite,
        &Counters::with(&ir, Fault::GuardOmitted, 100, 40),
    );
    assert_ne!(
        statuses.get(EXHAUSTED),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
}

/// The issue model with a counter created under an identity its input supplies.
fn supplied_identity() -> String {
    COUNTER.replace(
        "  - name: counter.model.Create
    input: []
    outcomes:
      - name: created
        creates: counter.model.Counter
        instance: id
        sets: {revision: 0}
        emits: [counter.model.Created]
        payload:
          counter.model.Created: {id: {generated: true}}",
        "  - name: counter.model.Create
    input:
      - {name: id, type: Uuid}
    outcomes:
      - name: created
        creates: counter.model.Counter
        instance: id
        sets: {revision: 0}
        emits: [counter.model.Created]
        payload:
          counter.model.Created: {id: input.id}",
    )
}

#[test]
fn a_seed_never_shares_an_identity_with_a_row_the_scenario_arranges() {
    let text = supplied_identity();
    assert_ne!(text, COUNTER, "precondition: the model changed");
    let ir = ir_of(&text);
    let plain = synthesize(&ir);
    // The identity the ordinary witness creates its counter under.
    let created = plain.suite.scenarios[&id(AUTHORIZED)]
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "counter.model.Create" =>
            {
                match input.get("id") {
                    Some(ScenarioValue::Literal {
                        value: Node::Text(text),
                    }) => Some(text.clone()),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("a created literal identity: {:#?}", plain.suite));
    let colliding = select(
        authored::Source::new(
            "collide.yaml",
            seed_document(
                "counter-collide",
                "collide",
                &created,
                &(MAX - 1).to_string(),
            ),
        ),
        "collide",
    );
    let alone =
        synthesize_with_seeds(&ir, &admitted(&ir, std::slice::from_ref(&colliding))).unwrap();
    let refusal = alone
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(AUTHORIZED)
        })
        .unwrap_or_else(|| panic!("the MAX−1 side stays refused: {:#?}", rendered(&alone)));
    assert!(refusal.to_string().contains("collides"), "{refusal}");
    let both = synthesize_with_seeds(
        &ir,
        &admitted(&ir, &[colliding, select(below_source(), "below-max")]),
    )
    .unwrap();
    let steps = &both.suite.scenarios[&id(AUTHORIZED)].steps;
    segment(steps, BELOW_MAX_ID);
    assert!(!steps.iter().any(|step| matches!(step,
        ScenarioStep::EstablishEntity { identity: Node::Text(held), .. } if *held == created)));
}

// ---- coverage ----------------------------------------------------------------------------------

#[test]
fn declared_coverage_consumes_the_same_seeds_and_records_the_same_bindings() {
    let ir = ir();
    let seeds = admitted(&ir, &issue_selections());
    let ordinary = synthesize_with_seeds(&ir, &seeds).unwrap();
    let input =
        coverage_build::build_with_seeds(&ir, &[], Scope::System, Origins::Generated, &seeds)
            .unwrap();
    let suite = input.selected().suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/43"
    );
    assert_eq!(
        suite.provenance.synthesis_seeds,
        ordinary.suite.provenance.synthesis_seeds
    );
    let inventory = input.selected().coverage().unwrap();
    assert_eq!(inventory.counts.refused, 0, "{:#?}", inventory.refused);
    assert_eq!(
        usize::try_from(inventory.counts.generated).unwrap(),
        ordinary.suite.len()
    );
    assert_eq!(
        inventory.authored_sources.len(),
        0,
        "a seed source is not an authored scenario"
    );
    // Narrowing keeps the parent's provenance, applications included.
    let selected = input.select(&[id(STALE)]).unwrap();
    assert_eq!(selected.selected().suite().provenance, suite.provenance);
    // Authored-only acquisition with seeds is a contradiction.
    assert!(
        coverage_build::build_with_seeds(&ir, &[], Scope::System, Origins::Authored, &seeds)
            .is_err()
    );
    // A seed source also passed as authored input has both roles, apart.
    let both = coverage_build::build_with_seeds(
        &ir,
        &[CoverageSource::new("max.yaml", max_source().text).unwrap()],
        Scope::System,
        Origins::GeneratedAndAuthored,
        &seeds,
    )
    .unwrap();
    let inventory = both.selected().coverage().unwrap();
    assert_eq!(inventory.authored_sources.len(), 1);
    assert_eq!(inventory.counts.authored, 1);
    assert_eq!(
        both.selected().suite().provenance.synthesis_seeds,
        ordinary.suite.provenance.synthesis_seeds
    );
    // Seed-free coverage keeps format 35 and no seed member.
    let plain = coverage_build::build(&ir, &[], Scope::System, Origins::Generated).unwrap();
    assert_eq!(
        plain
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/35"
    );
    assert!(!plain.selected().original_json().contains("synthesis_seeds"));
}

// ---- reader checks -----------------------------------------------------------------------------

#[test]
fn readers_refuse_forged_or_dangling_seed_metadata() {
    let ir = ir();
    let (good, with_authored, authored_id) = documents(&ir);
    AdmittedSuite::from_json(&text(&good)).unwrap();
    // An authored scenario establishing the same row is authored setup, not a seed use.
    AdmittedSuite::from_json(&text(&with_authored)).unwrap();
    let forgeries = forgeries(&good, &with_authored, &authored_id);
    assert!(forgeries.len() >= 25, "{}", forgeries.len());
    let mut admitted = Vec::new();
    for forgery in forgeries {
        match AdmittedSuite::from_json(&text(&forgery.document)) {
            Ok(_) => admitted.push(forgery.label),
            Err(error) => assert!(
                error.to_string().contains(forgery.rust),
                "{}: refused for another reason: {error}",
                forgery.label
            ),
        }
    }
    assert_eq!(admitted.len(), 0, "forged documents admitted: {admitted:?}");
}

#[test]
fn the_browser_player_refuses_a_seeded_suite_by_name() {
    let ir = ir();
    let synthesis = seeded(&ir);
    let error = ess_conformance::web::emit(&ir, &synthesis.suite).unwrap_err();
    assert!(error.to_string().contains("synthesis seed"), "{error}");
    let seeds = admitted(&ir, &issue_selections());
    let input =
        coverage_build::build_with_seeds(&ir, &[], Scope::System, Origins::Generated, &seeds)
            .unwrap();
    let error = ess_conformance::web::emit_input(&ir, &input).unwrap_err();
    assert!(error.to_string().contains("synthesis seed"), "{error}");
}

// ---- execution ---------------------------------------------------------------------------------

fn run(ir: &EssIr, suite: &ConformanceSuite, fault: Fault) -> BTreeMap<String, Status> {
    run_with(suite, &Counters::new(ir, fault))
}

fn run_with(suite: &ConformanceSuite, target: &Counters<'_>) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report();
    let statuses: BTreeMap<String, Status> = report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        statuses.values().all(|status| *status == Status::Passed),
        "{report:#?}"
    );
    statuses
}

#[test]
fn a_healthy_counter_passes_the_seeded_suite_and_each_fault_fails_its_scenario() {
    let ir = ir();
    let suite = seeded(&ir).suite;
    let healthy = run(&ir, &suite, Fault::None);
    let failed: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(failed.len(), 0, "{healthy:#?}");
    for (fault, scenario) in [
        (Fault::CasIgnored, STALE),
        (Fault::GuardOmitted, EXHAUSTED),
        (Fault::ThresholdEarly, AUTHORIZED),
        (Fault::AcknowledgedOnly, EXHAUSTED),
        (Fault::AcknowledgedOnly, AUTHORIZED),
        (Fault::Unsupported, EXHAUSTED),
        (Fault::Unsupported, AUTHORIZED),
    ] {
        let statuses = run(&ir, &suite, fault);
        assert_ne!(
            statuses.get(scenario),
            Some(&Status::Passed),
            "{fault:?} survived {scenario}: {statuses:#?}"
        );
    }
    // Without seeds the guard-omitted and early-threshold faults are invisible to the suite.
    let plain = synthesize(&ir).suite;
    for fault in [Fault::GuardOmitted, Fault::ThresholdEarly] {
        let statuses = run(&ir, &plain, fault);
        assert!(
            statuses.values().all(|status| *status == Status::Passed),
            "{fault:?}: {statuses:#?}"
        );
    }
}

// ---- correction round after adversary pass 1 ---------------------------------------------------

/// The issue model with two attribute-free actors, each granted both commands.
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

/// A seeded scenario is not run a second time with the callers reversed: its row's literal identity
/// cannot be established twice and is never renamed, so the reversed run is left out and named in
/// a `CrossCallerUnswapped` note. The seed-free scenarios of the same model keep their reversed run.
#[test]
fn a_seeded_scenario_names_its_skipped_reversed_caller_run() {
    use ess_conformance::synthesize::Note;
    let ir = ir_of(&with_two_actors());
    let seeded = synthesize_with_seeds(&ir, &admitted(&ir, &issue_selections())).unwrap();
    let unswapped: Vec<String> = seeded
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::CrossCallerUnswapped { scenario, reason } if reason.contains("setup") => {
                Some(scenario.to_string())
            }
            _ => None,
        })
        .collect();
    let record = seeded.suite.provenance.synthesis_seeds.as_ref().unwrap();
    for application in &record.applications {
        assert!(
            unswapped.contains(&application.scenario.to_string()),
            "{}: {unswapped:?}",
            application.scenario
        );
    }
    let plain = synthesize(&ir);
    assert!(
        plain.notes.iter().all(|note| !matches!(note,
            Note::CrossCallerUnswapped { reason, .. } if reason.contains("setup"))),
        "a seed-free suite has no setup to skip"
    );
    AdmittedSuite::from_suite(&seeded.suite).unwrap();
}

/// A guard over an integer binary64 does not carry (2^53 + 1) is witnessed exactly without any
/// seed: the input is stepped from the literal exactly. Base refused the branch with `ESS-SYNTH-003`
/// and held one scenario; both branches now have one, and the model itself passes the suite.
#[test]
fn a_guard_beyond_two_to_the_fifty_three_is_witnessed_exactly_without_seeds() {
    let ir = ir_files(&[
        (
            "system.yaml",
            "format: ess/20\nsystem: big\nversion: v1\ndomains: [big.model]\n",
        ),
        (
            "big.yaml",
            "domain: big.model
entities:
  - name: big.model.Ledger
    identity: {name: id, type: Uuid}
    fields:
      - {name: balance, type: Integer}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
commands:
  - name: big.model.Open
    input:
      - {name: amount, type: Integer}
    outcomes:
      - name: too-large
        when: amount >= 9007199254740993
        error: big.model.TooLarge
      - name: opened
        creates: big.model.Ledger
        instance: id
        sets: {balance: input.amount}
        emits: [big.model.Opened]
        payload:
          big.model.Opened: {id: {generated: true}}
errors:
  - {name: big.model.TooLarge, fields: []}
events:
  - name: big.model.Opened
    fields: [{name: id, type: Uuid}]
views:
  - name: big.model.Ledgers
    source: big.model.Ledger
    consistency: read_your_writes
    fields:
      - {name: id, type: Uuid}
      - {name: balance, type: Integer}
      - {name: state, type: big.model.Ledger.State}
",
        ),
    ]);
    let synthesis = synthesize(&ir);
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", rendered(&synthesis));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        ids,
        [
            "big.model.Open/outcome/opened",
            "big.model.Open/outcome/too-large"
        ]
    );
    let sent: Vec<String> = synthesis.suite.scenarios[&id("big.model.Open/outcome/too-large")]
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => {
                Some(serde_json::to_string(&input["amount"]).unwrap())
            }
            _ => None,
        })
        .collect();
    assert_eq!(sent, [r#"{"kind":"literal","value":9007199254740993}"#]);
    assert!(synthesis.suite.provenance.synthesis_seeds.is_none());
    let json = synthesis.suite.to_canonical_json().unwrap();
    assert_eq!(
        AdmittedSuite::from_json(&json).unwrap().suite(),
        &synthesis.suite
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(&synthesis.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir.clone()),
        )
        .into_report();
    assert_eq!(report.status, ConformanceStatus::Passed, "{report:#?}");
}
