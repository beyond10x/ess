//! A list-of-identity membership guard (`exists p in permissions: (p == "telephony")`) beside an
//! `updates:` subject, and in a variant in the `creates:` command too, reported by an adopter
//! (beyond10x/ess#761, `story:a-list-identity-guard-beside-an-updates-subject-is-witnessed`).
//!
//! - `tests/fixtures/list-identity-guard-beside-updates.yaml`: `UpdateRolePermissions` refuses a
//!   list naming a permission no row carries, answers `updated-with-telephony` for a list holding
//!   `telephony`, `updated` otherwise, and `not-found` for a role no row carries. Issue 761 D1: the
//!   refusal's scenario arranged no role, so the target answered `not-found`. Issue 761 D2: the
//!   guarded branch's scenario never sent `telephony`, so the target answered `updated`.
//! - `tests/fixtures/list-identity-guard-beside-creates.yaml`: the same, with the guarded branch in
//!   `DefineRole` as well. Issue 761 D3: every step needing a role sent an empty list and was
//!   refused ESS-SYNTH-019.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

const UPDATES: &str = include_str!("fixtures/list-identity-guard-beside-updates.yaml");
const CREATES: &str = include_str!("fixtures/list-identity-guard-beside-creates.yaml");

const UPDATE: &str = "demo.role.UpdateRolePermissions";
const DEFINE: &str = "demo.role.DefineRole";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("list-identity-guard.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{:?} {}: {}",
                refusal.scenario.as_ref().map(ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect()
}

fn statuses(result: &Synthesis, model: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn document(result: &Synthesis) -> Value {
    serde_json::from_str(&result.suite.to_canonical_json().expect("serialises")).expect("JSON")
}

fn steps(document: &Value, id: &str) -> Vec<Value> {
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized"))
        .clone()
}

/// The `execute_command` steps of `steps` sending `command`, in order.
fn sends<'a>(steps: &'a [Value], command: &str) -> Vec<&'a Value> {
    steps
        .iter()
        .filter(|step| step["step"] == "execute_command" && step["command"] == command)
        .collect()
}

/// The index of the first `execute_command` step sending `command`.
fn first(steps: &[Value], command: &str) -> Option<usize> {
    steps
        .iter()
        .position(|step| step["step"] == "execute_command" && step["command"] == command)
}

/// The elements of the `permissions` list `step` sends.
fn permissions(step: &Value) -> Vec<Value> {
    let list = &step["input"]["permissions"];
    list["items"]
        .as_array()
        .or_else(|| list["value"].as_array())
        .or_else(|| list.as_array())
        .unwrap_or_else(|| panic!("a list is sent: {step:#}"))
        .clone()
}

/// Whether `item` is the literal `telephony`, however a scenario value spells a literal.
fn is_telephony(item: &Value) -> bool {
    item == "telephony" || item["value"] == "telephony"
}

/// Whether `item` sends `telephony`: the literal, or a row of `steps` captured from the event of a
/// `SupplyPermission` step sent `telephony`.
fn names_telephony(steps: &[Value], item: &Value) -> bool {
    if is_telephony(item) {
        return true;
    }
    let Some(instance) = item["instance"].as_str() else {
        return false;
    };
    let Some(at) = steps
        .iter()
        .position(|step| step["step"] == "capture_instance" && step["instance"] == instance)
    else {
        return false;
    };
    steps[..at]
        .iter()
        .rev()
        .find(|step| step["step"] == "execute_command")
        .is_some_and(|step| {
            step["command"] == "demo.role.SupplyPermission"
                && is_telephony(&step["input"]["permission_id"])
        })
}

fn check(label: &str, text: &str, guarded: &[&str]) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refusals(&result);
    assert!(
        !refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-019")),
        "{label}: issue 761 D3, no ESS-SYNTH-019: {refused:#?}"
    );
    let document = document(&result);
    let passed = statuses(&result, &model);
    let failed: Vec<String> = passed
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| {
            format!(
                "{id} {status:?}\n{:#}",
                document["scenarios"][id.as_str()]["steps"]
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "{label}: every scenario passes the interpreted target:\n{}",
        failed.join("\n")
    );
    for wanted in [
        format!("{UPDATE}/outcome/unknown-permission"),
        format!("{UPDATE}/outcome/updated-with-telephony"),
        format!("{UPDATE}/outcome/updated"),
        format!("{UPDATE}/outcome/not-found"),
        format!("{DEFINE}/outcome/unknown-permission"),
        format!("{DEFINE}/outcome/defined"),
    ] {
        assert_eq!(
            passed.get(&wanted),
            Some(&Status::Passed),
            "{label}: {wanted} is witnessed\nrefusals: {refused:#?}"
        );
    }

    // Issue 761 D1: the role is arranged before the update, which is sent one known and one
    // unknown permission.
    let refusal = steps(&document, &format!("{UPDATE}/outcome/unknown-permission"));
    let update = first(&refusal, UPDATE).expect("the update is sent");
    let defined = first(&refusal, DEFINE).expect("a role is arranged");
    assert!(
        defined < update,
        "{label}: D1 arranges the role first: {refusal:#?}"
    );
    let sent = permissions(sends(&refusal, UPDATE)[0]);
    assert_eq!(
        sent.len(),
        2,
        "{label}: D1 sends two permissions: {sent:#?}"
    );
    assert!(
        sent[0]["kind"] == "instance" && sent[1]["kind"] == "literal",
        "{label}: D1 sends one arranged permission and one no row carries: {sent:#?}"
    );

    // Issue 761 D2, and D3 for the creating command: the guarded branch is sent a list holding the
    // guard's literal, arranged as a permission row.
    for (command, outcome) in guarded.iter().map(|name| {
        let (command, outcome) = name.split_once('#').expect("command#outcome");
        (command, outcome)
    }) {
        let id = format!("{command}/outcome/{outcome}");
        assert_eq!(
            passed.get(&id),
            Some(&Status::Passed),
            "{label}: {id} is witnessed\nrefusals: {refused:#?}"
        );
        let scenario = steps(&document, &id);
        // Every run of the command under test, a swapped-caller run included, sends a list
        // naming the row created under `telephony`.
        for sent in sends(&scenario, command) {
            let list = permissions(sent);
            assert!(
                list.iter().any(|item| names_telephony(&scenario, item)),
                "{label}: {id} sends `telephony`, arranged as a row: {scenario:#?}"
            );
        }
    }
}

#[test]
fn a_list_identity_guard_beside_an_updates_subject_is_witnessed() {
    check(
        "updates",
        UPDATES,
        &["demo.role.UpdateRolePermissions#updated-with-telephony"],
    );
}

#[test]
fn a_list_identity_guard_in_the_creating_command_is_witnessed() {
    check(
        "creates",
        CREATES,
        &[
            "demo.role.UpdateRolePermissions#updated-with-telephony",
            "demo.role.DefineRole#defined-with-telephony",
        ],
    );
}
