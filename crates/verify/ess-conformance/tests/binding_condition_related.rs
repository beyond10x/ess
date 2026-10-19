//! A binding condition over an event member the trigger fills from a field of the row an input
//! names, `{related: {via: input.role_id, field: feature_eligible}}` (downstream report,
//! 2026-10-18).
//!
//! Synthesis chooses that member's value by creating the row with it, through the row's creating
//! command, before the trigger: the flow creates an eligible role, `condition-false` an ineligible
//! one. An enum leaf works under every format that has binding conditions, a Boolean leaf from
//! `ess/27`. A failure policy is forced only through an `external:` refusal: an `external:` branch
//! that accepts is no failure.

use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const ENUM: &str = r"
format: ess/24
system: roles
version: v1
domain: roles.access
types:
  - {name: roles.access.RoleId, kind: newtype, of: Uuid}
  - {name: roles.access.UserId, kind: newtype, of: Uuid}
  - {name: roles.access.Eligibility, kind: enum, variants: [Eligible, Ineligible]}
entities:
  - name: roles.access.Role
    identity: {name: role_id, type: roles.access.RoleId}
    fields:
      - {name: feature_eligible, type: roles.access.Eligibility}
    lifecycle: {initial: Defined, states: [Defined], terminal: [Defined]}
events:
  - name: roles.access.RoleDefined
    fields: [{name: role_id, type: roles.access.RoleId}]
  - name: roles.access.RoleAssigned
    fields:
      - {name: user_id, type: roles.access.UserId}
      - {name: role_id, type: roles.access.RoleId}
      - {name: feature_eligible, type: roles.access.Eligibility}
  - name: roles.access.FeatureGranted
    fields: [{name: user_id, type: roles.access.UserId}]
  - name: roles.access.FeatureKept
    fields: [{name: user_id, type: roles.access.UserId}]
commands:
  - name: roles.access.DefineRole
    input: [{name: feature_eligible, type: roles.access.Eligibility}]
    outcomes:
      - name: defined
        creates: roles.access.Role
        instance: role_id
        sets: {feature_eligible: input.feature_eligible}
        emits: [roles.access.RoleDefined]
        payload: {roles.access.RoleDefined: {role_id: {generated: true}}}
  - name: roles.access.AssignRole
    input:
      - {name: user_id, type: roles.access.UserId}
      - {name: role_id, type: roles.access.RoleId}
    outcomes:
      - name: assigned
        emits: [roles.access.RoleAssigned]
        payload:
          roles.access.RoleAssigned:
            user_id: input.user_id
            role_id: input.role_id
            feature_eligible: {related: {via: input.role_id, field: feature_eligible}}
  - name: roles.access.GrantFeature
    input: [{name: user_id, type: roles.access.UserId}]
    outcomes:
      - name: kept
        external: the user already holds the feature
        emits: [roles.access.FeatureKept]
        payload: {roles.access.FeatureKept: {user_id: input.user_id}}
      - name: granted
        emits: [roles.access.FeatureGranted]
        payload: {roles.access.FeatureGranted: {user_id: input.user_id}}
bindings:
  - id: grant-on-eligible-assignment
    when:
      event: roles.access.RoleAssigned
      where: [event.feature_eligible == Eligible]
    invoke: {command: roles.access.GrantFeature}
    mapping: {user_id: event.user_id}
    delivery: at_least_once
    on_failure: retry
views:
  - name: roles.access.Roles
    source: roles.access.Role
    consistency: read_your_writes
    fields:
      - {name: role_id, type: roles.access.RoleId}
      - {name: feature_eligible, type: roles.access.Eligibility}
";

const BINDING: &str = "grant-on-eligible-assignment";

/// The same model with a Boolean flag, under `ess/27`.
fn boolean() -> String {
    ENUM.replace("format: ess/24", "format: ess/27")
        .replace("type: roles.access.Eligibility}", "type: Boolean}")
        .replace(
            "event.feature_eligible == Eligible",
            "event.feature_eligible == true",
        )
}

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("roles.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the model resolves:\n{d}"))
}

fn statuses(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The `feature_eligible` value the role `AssignRole` names is created with, in scenario `id`.
fn assigned_role_value(synthesis: &Synthesis, id: &str) -> Value {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    let steps = document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", synthesis.refusals))
        .clone();
    let named = steps
        .iter()
        .find(|step| step["command"] == "roles.access.AssignRole")
        .and_then(|step| step["input"]["role_id"]["instance"].as_str())
        .expect("the trigger names an arranged role")
        .to_owned();
    let captured = steps
        .iter()
        .position(|step| step["step"] == "capture_instance" && step["instance"] == named.as_str())
        .expect("the role is captured");
    steps[..captured]
        .iter()
        .rev()
        .find(|step| step["command"] == "roles.access.DefineRole")
        .map(|step| step["input"]["feature_eligible"]["value"].clone())
        .expect("the role is created")
}

#[test]
fn the_related_row_is_created_with_the_value_each_side_needs() {
    for (text, holds, fails) in [
        (
            ENUM.to_owned(),
            Value::from("Eligible"),
            Value::from("Ineligible"),
        ),
        (boolean(), Value::Bool(true), Value::Bool(false)),
    ] {
        let synthesis = synthesize(&model(&text));
        for aspect in ["flow", "mapping", "delivery"] {
            assert_eq!(
                assigned_role_value(&synthesis, &format!("{BINDING}/binding/{aspect}")),
                holds,
                "{aspect}"
            );
        }
        assert_eq!(
            assigned_role_value(&synthesis, &format!("{BINDING}/binding/condition-false")),
            fails
        );
    }
}

#[test]
fn the_suite_passes_the_interpreter_and_fails_a_target_ignoring_the_condition() {
    for text in [ENUM.to_owned(), boolean()] {
        let suite = synthesize(&model(&text)).suite;
        let honest = statuses(&suite, &Interpreted::for_model(model(&text)));
        let not_passed: Vec<_> = honest
            .iter()
            .filter(|(_, status)| **status != Status::Passed)
            .collect();
        assert_eq!(not_passed, Vec::<(&String, &Status)>::new());
        for aspect in ["flow", "mapping", "delivery", "condition-false"] {
            assert!(honest.contains_key(&format!("{BINDING}/binding/{aspect}")));
        }
        let ignoring = text
            .replace("      where: [event.feature_eligible == Eligible]\n", "")
            .replace("      where: [event.feature_eligible == true]\n", "");
        let faulty = statuses(&suite, &Interpreted::for_model(model(&ignoring)));
        assert_ne!(
            faulty[&format!("{BINDING}/binding/condition-false")],
            Status::Passed
        );
    }
}

#[test]
fn an_accepting_external_branch_is_not_forced_as_a_failure() {
    let synthesis = synthesize(&model(ENUM));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(&format!("`{BINDING}/binding/on-failure`")))
        .collect();
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("can be forced to fail")),
        "{refused:#?}"
    );
}
