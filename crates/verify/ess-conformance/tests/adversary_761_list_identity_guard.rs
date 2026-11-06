//! Adversary cases for beyond10x/ess#761 (`story:a-list-identity-guard-beside-an-updates-subject-is-witnessed`):
//! variants of the unit's own fixtures, built here by text substitution so no new model file joins
//! the fixture tables.
//!
//! Each case asserts the story's acceptance on its variant: no ESS-SYNTH-019 refusal, every emitted
//! scenario passes the interpreted target, and the named branches are witnessed.

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

const GUARD: &str = "          exists: {in: input.permissions, as: p, that: 'p == \"telephony\"'}\n";

const DEFINED: &str = "      - name: defined
        creates: demo.role.Role
        instance: role_id
        emits: [demo.role.RoleDefined]
        payload:
          demo.role.RoleDefined: {role_id: {generated: true}, permissions: input.permissions}
        sets: {name: input.name, permissions: input.permissions, telephony_eligible: false}
";

const GUARDED_EFFECT: &str = "        updates: demo.role.Role
        instance: role_id
        emits: [demo.role.RolePermissionsUpdated]
        payload:
          demo.role.RolePermissionsUpdated: {role_id: input.role_id, permissions: input.permissions}
        sets: {permissions: input.permissions, telephony_eligible: true}
";

fn replaced(text: &str, from: &str, to: &str, times: usize) -> String {
    assert_eq!(
        text.matches(from).count(),
        times,
        "the variant's anchor occurs {times} time(s): {from}"
    );
    text.replace(from, to)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary-761.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
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

/// Synthesizes `text`, runs it against the interpreted target and asserts the story's acceptance:
/// no ESS-SYNTH-019, every emitted scenario passes, and every id in `wanted` is witnessed.
fn check(label: &str, text: &str, wanted: &[String]) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused: Vec<String> = result
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
        .collect();
    let document: Value =
        serde_json::from_str(&result.suite.to_canonical_json().expect("serialises")).expect("JSON");
    let passed = statuses(&result, &model);
    let failed: Vec<String> = passed
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| {
            format!(
                "{id} {status:?}\n{}",
                serde_json::to_string(&document["scenarios"][id.as_str()]["steps"])
                    .unwrap_or_default()
            )
        })
        .collect();
    let missing: Vec<&String> = wanted
        .iter()
        .filter(|id| passed.get(id.as_str()) != Some(&Status::Passed))
        .collect();
    let synth_019: Vec<&String> = refused
        .iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-019"))
        .collect();
    assert!(
        synth_019.is_empty() && failed.is_empty() && missing.is_empty(),
        "{label}:\nESS-SYNTH-019 refusals: {synth_019:#?}\nfailing scenarios:\n{}\nnot witnessed: \
         {missing:#?}\nall refusals: {refused:#?}\nnotes: {:#?}",
        failed.join("\n"),
        result.notes
    );
}

fn ids(command: &str, outcomes: &[&str]) -> Vec<String> {
    outcomes
        .iter()
        .map(|outcome| format!("{command}/outcome/{outcome}"))
        .collect()
}

/// `forall` instead of `exists`: the empty list does not refute the guard, so the guarded branch
/// is still sent two generated rows, which no `forall p: p == "telephony"` holds.
#[test]
fn adversary_761_forall_guard_beside_an_updates_subject_is_witnessed() {
    let text = replaced(
        UPDATES,
        GUARD,
        "          forall: {in: input.permissions, as: p, that: 'p == \"telephony\"'}\n",
        1,
    );
    check(
        "forall",
        &text,
        &ids(UPDATE, &["unknown-permission", "updated-with-telephony", "updated", "not-found"]),
    );
}

/// Two literals in one guard: the list must hold a row for each.
#[test]
fn adversary_761_two_literal_guard_is_witnessed() {
    let text = replaced(
        UPDATES,
        GUARD,
        "          all:
            - exists: {in: input.permissions, as: p, that: 'p == \"telephony\"'}
            - exists: {in: input.permissions, as: q, that: 'q == \"voice\"'}
",
        1,
    );
    check(
        "two literals",
        &text,
        &ids(UPDATE, &["unknown-permission", "updated-with-telephony", "updated", "not-found"]),
    );
}

/// Story D3, second clause: the creating command's only creating branch is the guarded one, so a
/// step that needs a role must be sent a list satisfying the guard.
#[test]
fn adversary_761_only_creating_branch_guarded_by_the_list() {
    let text = replaced(
        CREATES,
        DEFINED,
        "      - name: lacks-telephony
        error: demo.role.TelephonyRequired
",
        1,
    );
    let text = replaced(
        &text,
        "  - {name: demo.role.RoleNotFound, summary: No role has this id.}\n",
        "  - {name: demo.role.RoleNotFound, summary: No role has this id.}
  - {name: demo.role.TelephonyRequired, summary: A role needs telephony.}
",
        1,
    );
    let mut wanted = ids(
        UPDATE,
        &["unknown-permission", "updated-with-telephony", "updated", "not-found"],
    );
    wanted.extend(ids(
        DEFINE,
        &["unknown-permission", "defined-with-telephony", "lacks-telephony"],
    ));
    check("only guarded creator", &text, &wanted);
}

/// The guarded branch deletes the subject instead of updating it.
#[test]
fn adversary_761_guard_beside_a_deletes_subject_is_witnessed() {
    let text = replaced(
        UPDATES,
        GUARDED_EFFECT,
        "        deletes: demo.role.Role
        instance: role_id
        emits: [demo.role.RolePermissionsUpdated]
        payload:
          demo.role.RolePermissionsUpdated: {role_id: input.role_id, permissions: input.permissions}
",
        1,
    );
    check(
        "deletes",
        &text,
        &ids(UPDATE, &["unknown-permission", "updated-with-telephony", "updated", "not-found"]),
    );
}

/// The guarded branch moves the subject through a transition instead of updating it.
#[test]
fn adversary_761_guard_beside_a_moves_subject_is_witnessed() {
    let anchor = "  - name: demo.role.Role\n";
    let at = UPDATES.find(anchor).expect("the role entity") + anchor.len();
    let (head, tail) = UPDATES.split_at(at);
    let tail = tail.replacen(
        "    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n",
        "    lifecycle:
      initial: Active
      states: [Active, Certified]
      terminal: [Certified]
      transitions:
        - {name: certify, from: [Active], to: Certified}
",
        1,
    );
    let text = format!("{head}{tail}");
    let text = replaced(
        &text,
        GUARDED_EFFECT,
        "        moves: demo.role.Role.certify
        instance: role_id
        emits: [demo.role.RolePermissionsUpdated]
        payload:
          demo.role.RolePermissionsUpdated: {role_id: input.role_id, permissions: input.permissions}
        sets: {permissions: input.permissions, telephony_eligible: true}
",
        1,
    );
    check(
        "moves",
        &text,
        &ids(UPDATE, &["unknown-permission", "updated-with-telephony", "updated", "not-found"]),
    );
}

/// Two commands guard on the same list, each on its own literal.
#[test]
fn adversary_761_two_commands_share_the_list() {
    let start = UPDATES
        .find("  - name: demo.role.UpdateRolePermissions\n")
        .expect("the update command");
    let end = UPDATES.find("events:\n").expect("events");
    let extend = UPDATES[start..end]
        .replace("UpdateRolePermissions", "ExtendRolePermissions")
        .replace("update-role-permissions", "extend-role-permissions")
        .replace("Update role permissions", "Extend role permissions")
        .replace("p == \"telephony\"", "p == \"voice\"")
        .replace("updated-with-telephony", "updated-with-voice");
    let text = format!("{}{extend}{}", &UPDATES[..end], &UPDATES[end..]);
    let text = replaced(
        &text,
        "may: [demo.role.DefineRole, demo.role.UpdateRolePermissions]",
        "may: [demo.role.DefineRole, demo.role.UpdateRolePermissions, demo.role.ExtendRolePermissions]",
        1,
    );
    let text = replaced(
        &text,
        "        - demo.role.UpdateRolePermissions\n",
        "        - demo.role.UpdateRolePermissions\n        - demo.role.ExtendRolePermissions\n",
        1,
    );
    let mut wanted = ids(
        UPDATE,
        &["unknown-permission", "updated-with-telephony", "updated", "not-found"],
    );
    wanted.extend(ids(
        "demo.role.ExtendRolePermissions",
        &["unknown-permission", "updated-with-voice", "updated", "not-found"],
    ));
    check("two commands", &text, &wanted);
}

/// The guard's literal is an identity no creation accepts: the branch is unreachable, so synthesis
/// emits no scenario for it that the target fails, and names it among its refusals.
#[test]
fn adversary_761_literal_no_row_can_carry_is_refused_not_emitted() {
    let text = replaced(
        UPDATES,
        "      - name: supplied\n",
        "      - name: reserved
        when: 'permission_id == \"telephony\"'
        error: demo.role.Reserved
      - name: supplied
",
        1,
    );
    let text = replaced(
        &text,
        "  - {name: demo.role.RoleNotFound, summary: No role has this id.}\n",
        "  - {name: demo.role.RoleNotFound, summary: No role has this id.}
  - {name: demo.role.Reserved, summary: The identity is reserved.}
",
        1,
    );
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let passed = statuses(&result, &model);
    let failed: Vec<&String> = passed
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    let guarded = format!("{UPDATE}/outcome/updated-with-telephony");
    let named = result.refusals.iter().any(|refusal| {
        refusal
            .scenario
            .as_ref()
            .is_some_and(|id| id.to_string() == guarded)
            || refusal.to_string().contains("updated-with-telephony")
    });
    assert!(
        failed.is_empty() && named,
        "unreachable literal: failing {failed:#?}; refusal names {guarded}: {named}\nrefusals: {:#?}",
        result
            .refusals
            .iter()
            .map(|refusal| format!("{} {}", refusal.code(), refusal.cause))
            .collect::<Vec<_>>()
    );
}
