//! Adversary, wave 2026-10-08e unit e1 (beyond10x/ess#499): the String-newtype check on
//! `expect_response_payload` in the generated Go and TypeScript runners.
//!
//! The unit's Go and TypeScript parity cases run the mapped suite with its
//! `expect_direct_response` step in place, and that step fails a broken value first. Deleting the
//! payload observation's check from either generated runner therefore leaves every one of them
//! green. These cases take the direct step out, as the unit's native case
//! `the_payload_observation_checks_the_rule_on_its_own` does, so the payload check is the only
//! thing that can fail the value, and they show the mutant surviving the unit's shape.

mod adversary_e1_support;
mod support_go;

use adversary_e1_support::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use serde_json::json;

const TYPES: &str = "  - name: catalog.items.Code
    kind: newtype
    of: String
    alphabet: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-'
";
const RESPONSE: &str = "      - {name: code, type: catalog.items.Code}\n";
const EVENTS: &str = "events:\n  - name: catalog.items.Registered\n    fields:\n      - {name: code, type: catalog.items.Code}\n";
const EMITS: &str = "        emits: [catalog.items.Registered]\n        payload:\n          catalog.items.Registered:\n            code: {response: code}\n";

const GO_CALL: &str =
    "\tif err := checkResponseConstraints(r.Fields, r.Declarations, r.constraints, response); err != nil {";
const GO_MUTANT: &str = "\tif err := error(nil); err != nil {";
const TS_CALL: &str = "  checkResponseConstraints(\n    observation.fields,\n    observation.declarations,\n    observation.constraints,\n    response,\n  );\n";

fn mapped() -> ConformanceSuite {
    synthesized(&model(TYPES, RESPONSE, EVENTS, EMITS))
}

fn payload_only() -> ConformanceSuite {
    let mut suite = mapped();
    for scenario in suite.scenarios.values_mut() {
        scenario
            .steps
            .retain(|step| !matches!(step, ScenarioStep::ExpectDirectResponse { .. }));
    }
    suite
}

fn broken() -> Fixed {
    Fixed::new(json!({"code": "ab-12"}))
        .emitting("catalog.items.Registered", json!({"code": "ab-12"}))
}

/// The Go verdict on the outcome, with the payload check deleted from the generated runtime when
/// `mutate`.
fn go(label: &str, suite: &ConformanceSuite, mutate: bool) -> String {
    let directory = support_go::package(label, suite, &[support_go::TRANSCRIPT_TARGET]);
    if mutate {
        let mut found = 0;
        for entry in std::fs::read_dir(directory.join("essconform")).unwrap() {
            let path = entry.unwrap().path();
            let source = std::fs::read_to_string(&path).unwrap();
            found += source.matches(GO_CALL).count();
            std::fs::write(&path, source.replace(GO_CALL, GO_MUTANT)).unwrap();
        }
        assert_eq!(found, 1, "the Go payload check moved");
    }
    let recorder = support_go::Recorder::new(broken());
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let _ = Runner::for_suite(suite).run_admitted(&admitted, &recorder);
    let replayed = support_go::replay(&directory, &recorder, &[]);
    replayed
        .go
        .outcomes
        .get(OUTCOME)
        .cloned()
        .unwrap_or_else(|| panic!("{label}: no Go verdict\n{}", replayed.go.log))
}

/// The TypeScript verdict on the outcome, with the payload check deleted when `mutate`.
fn typescript(label: &str, suite: &ConformanceSuite, mutate: bool) -> String {
    let found = std::cell::Cell::new(0);
    let verdicts = typescript_verdicts(label, suite, broken(), |_, source| {
        if mutate {
            found.set(found.get() + source.matches(TS_CALL).count());
            source.replace(TS_CALL, "")
        } else {
            source
        }
    })
    .unwrap_or_else(|log| panic!("{label}: no TypeScript report\n{log}"));
    assert!(
        !mutate || found.get() == 1,
        "the TypeScript payload check moved: {}",
        found.get()
    );
    verdicts[OUTCOME].clone()
}

#[test]
fn native_fails_a_broken_payload_value_with_the_direct_step_taken_out() {
    let suite = payload_only();
    assert_eq!(native_verdicts(&suite, &broken())[OUTCOME], "failed");
}

#[test]
fn go_fails_a_broken_payload_value_with_the_direct_step_taken_out() {
    assert_eq!(go("payload-only", &payload_only(), false), "failed");
}

#[test]
fn typescript_fails_a_broken_payload_value_with_the_direct_step_taken_out() {
    assert_eq!(typescript("payload-only", &payload_only(), false), "failed");
}

/// The mutant: with the Go payload check deleted, the unit's mapped suite still fails the broken
/// value (through its direct step), and only the payload-only suite tells the mutant apart.
#[test]
fn the_go_payload_mutant_survives_the_units_suite_shape() {
    assert_eq!(go("mutant-full", &mapped(), true), "failed");
    assert_eq!(go("mutant-payload-only", &payload_only(), true), "passed");
}

#[test]
fn the_typescript_payload_mutant_survives_the_units_suite_shape() {
    assert_eq!(typescript("mutant-full", &mapped(), true), "failed");
    assert_eq!(
        typescript("mutant-payload-only", &payload_only(), true),
        "passed"
    );
}
