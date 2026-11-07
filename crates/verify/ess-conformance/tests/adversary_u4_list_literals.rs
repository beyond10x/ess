//! Adversary cases for `story:a-guard-over-several-list-literals-is-witnessed` (wave 0.90.0, U4):
//! variants of `fixtures/list-identity-guard-beside-updates.yaml`, built here by text substitution
//! so no new model file joins the fixture tables.
//!
//! The story's acceptance: the guarded branch's scenario sends a list holding every literal, each
//! arranged as a row, and no emitted scenario sends a literal no row backs. Where a guard is
//! unreachable, synthesis refuses its branch rather than emitting a scenario the target fails.
//! Where a guard's children are told apart, a target deciding the guard with a connective mutated
//! fails the suite.

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

const UPDATE: &str = "demo.role.UpdateRolePermissions";

const GUARD: &str =
    "          exists: {in: input.permissions, as: p, that: 'p == \"telephony\"'}\n";

const ALL_OUTCOMES: &[&str] = &[
    "unknown-permission",
    "updated-with-telephony",
    "updated",
    "not-found",
];

fn replaced(text: &str, from: &str, to: &str, times: usize) -> String {
    assert_eq!(
        text.matches(from).count(),
        times,
        "the variant's anchor occurs {times} time(s): {from}"
    );
    text.replace(from, to)
}

/// One `exists` child over `list`, comparing its element with `literal`.
fn exists(indent: usize, list: &str, literal: &str) -> String {
    format!(
        "{:indent$}- exists: {{in: input.{list}, as: e, that: 'e == \"{literal}\"'}}\n",
        ""
    )
}

/// The fixture with its guard replaced by `connective` over `children` (each a whole line).
fn guarded(connective: &str, children: &[String]) -> String {
    let mut guard = format!("          {connective}:\n");
    for child in children {
        guard.push_str(child);
    }
    replaced(UPDATES, GUARD, &guard, 1)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary-u4.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(result: &Synthesis, target: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn document(result: &Synthesis) -> Value {
    serde_json::from_str(&result.suite.to_canonical_json().expect("serialises")).expect("JSON")
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

/// Every invocation of the command under test that sends a list element no row backs, where the
/// branch it expects is not the refusal over a missing row: `<scenario> #<n>: <input>`.
fn unbacked(document: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let Some(scenarios) = document["scenarios"].as_object() else {
        return found;
    };
    for (id, scenario) in scenarios {
        let Some(steps) = scenario["steps"].as_array() else {
            continue;
        };
        for (at, step) in steps.iter().enumerate() {
            if step["step"] != "execute_command" || !step["command"].to_string().contains(UPDATE)
            {
                continue;
            }
            let expects_missing = steps[at + 1..]
                .iter()
                .find(|next| next["step"] == "expect_outcome")
                .is_some_and(|next| next["outcome"].to_string().contains("unknown-permission"));
            if expects_missing {
                continue;
            }
            let list = &step["input"]["permissions"];
            let literal = match list["kind"].as_str() {
                Some("literal") => list["value"].as_array().is_some_and(|v| !v.is_empty()),
                Some("list") => list["items"]
                    .as_array()
                    .is_some_and(|items| items.iter().any(|item| item["kind"] == "literal")),
                _ => false,
            };
            if literal {
                found.push(format!("{id} #{at}: {list}"));
            }
        }
    }
    found
}

/// Synthesizes `text`, runs it against the interpreted target and asserts the story's acceptance:
/// no ESS-SYNTH-019, every emitted scenario passes, every id in `wanted` is witnessed, and no
/// invocation sends a list element no row backs.
fn check(label: &str, text: &str, wanted: &[&str]) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refusals(&result);
    let document = document(&result);
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
    let missing: Vec<String> = wanted
        .iter()
        .map(|outcome| format!("{UPDATE}/outcome/{outcome}"))
        .filter(|id| passed.get(id.as_str()) != Some(&Status::Passed))
        .collect();
    let synth_019: Vec<&String> = refused
        .iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-019"))
        .collect();
    let unbacked = unbacked(&document);
    assert!(
        synth_019.is_empty() && failed.is_empty() && missing.is_empty() && unbacked.is_empty(),
        "{label}:\nESS-SYNTH-019 refusals: {synth_019:#?}\nfailing scenarios:\n{}\nnot witnessed: \
         {missing:#?}\nunbacked literals: {unbacked:#?}\nall refusals: {refused:#?}",
        failed.join("\n"),
    );
}

/// The guarded branch is unreachable: synthesis emits no scenario the target fails and names the
/// branch among its refusals.
fn refused_not_emitted(label: &str, text: &str) {
    let model = ir(text);
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
        failed.is_empty() && named && !passed.contains_key(&guarded),
        "{label}: failing {failed:#?}; refusal names {guarded}: {named}; emitted: {}\nrefusals: \
         {:#?}",
        passed.contains_key(&guarded),
        refusals(&result)
    );
}

/// The suite synthesized from `text` fails at least one scenario against the target `mutant`
/// decides: the suite tells the two guards apart.
fn mutant_caught(label: &str, text: &str, mutant: &str) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let against_model = statuses(&result, &model);
    let against_mutant = statuses(&result, &ir(mutant));
    let caught: Vec<&String> = against_mutant
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(
        !caught.is_empty() && against_model.values().all(|status| *status == Status::Passed),
        "{label}: the mutant passes every scenario ({} run); against the model: \
         {against_model:#?}\nrefusals: {:#?}",
        against_mutant.len(),
        refusals(&result)
    );
}

fn telephony_voice() -> Vec<String> {
    vec![
        exists(12, "permissions", "telephony"),
        exists(12, "permissions", "voice"),
    ]
}

/// Three literals in one list: one row each.
#[test]
fn adversary_u4_three_literals_are_witnessed() {
    let text = guarded(
        "all",
        &[
            exists(12, "permissions", "telephony"),
            exists(12, "permissions", "voice"),
            exists(12, "permissions", "fax"),
        ],
    );
    check("three literals", &text, ALL_OUTCOMES);
}

/// A literal written twice beside another: the list holds each once.
#[test]
fn adversary_u4_repeated_literal_is_witnessed() {
    let text = guarded(
        "all",
        &[
            exists(12, "permissions", "telephony"),
            exists(12, "permissions", "telephony"),
            exists(12, "permissions", "voice"),
        ],
    );
    check("repeated literal", &text, ALL_OUTCOMES);
}

/// `any:` of two literals: witnessed, and a target deciding it as `all:` fails the suite, so the
/// suite sends a list holding one literal alone.
#[test]
fn adversary_u4_any_of_two_literals_does_not_force_both() {
    let text = guarded("any", &telephony_voice());
    check("any of two", &text, ALL_OUTCOMES);
    mutant_caught("any read as all", &text, &guarded("all", &telephony_voice()));
}

/// `all:` of two literals: a target deciding it as `any:` fails the suite, so some scenario sends
/// one literal alone and expects the guard refuted.
#[test]
fn adversary_u4_all_of_two_literals_is_told_from_any() {
    let text = guarded("all", &telephony_voice());
    check("all of two", &text, ALL_OUTCOMES);
    mutant_caught("all read as any", &text, &guarded("any", &telephony_voice()));
}

/// The second literal is an identity no creation accepts: the branch is unreachable, refused and
/// not emitted.
#[test]
fn adversary_u4_second_literal_no_row_can_carry_is_refused() {
    let text = guarded("all", &telephony_voice());
    let text = replaced(
        &text,
        "      - name: supplied\n",
        "      - name: reserved
        when: 'permission_id == \"voice\"'
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
    refused_not_emitted("unbackable second literal", &text);
}

/// A refusal before the guarded branch bounds the list below the literal count: the branch is
/// unreachable, refused and not emitted.
#[test]
fn adversary_u4_list_bound_below_literal_count_is_refused() {
    let text = guarded(
        "all",
        &[
            exists(12, "permissions", "telephony"),
            exists(12, "permissions", "voice"),
            exists(12, "permissions", "fax"),
        ],
    );
    let text = replaced(
        &text,
        "      - name: unknown-permission
        when_related: {via: input.permissions, exists: false}
        error: demo.role.UnknownPermission
      - name: updated-with-telephony
",
        "      - name: too-many
        when: permissions.count > 2
        error: demo.role.TooMany
      - name: unknown-permission
        when_related: {via: input.permissions, exists: false}
        error: demo.role.UnknownPermission
      - name: updated-with-telephony
",
        1,
    );
    let text = replaced(
        &text,
        "  - {name: demo.role.RoleNotFound, summary: No role has this id.}\n",
        "  - {name: demo.role.RoleNotFound, summary: No role has this id.}
  - {name: demo.role.TooMany, summary: Too many permissions.}
",
        1,
    );
    refused_not_emitted("bounded list", &text);
}

/// Two attribute-free actors each granted every command, so synthesis appends each scenario again
/// with the callers swapped and fresh identities drawn.
fn swappable(text: &str) -> String {
    let text = replaced(
        text,
        "    may: [demo.role.DefineRole, demo.role.UpdateRolePermissions]\n",
        "    may: [demo.role.SupplyPermission, demo.role.DefineRole, demo.role.UpdateRolePermissions]\n",
        1,
    );
    let text = replaced(
        &text,
        "    attributes: [{name: service_id, type: demo.role.ServiceRef}]\n    may: [demo.role.SupplyPermission]\n",
        "    may: [demo.role.SupplyPermission, demo.role.DefineRole, demo.role.UpdateRolePermissions]\n",
        1,
    );
    let text = replaced(
        &text,
        "    input: [{name: permission_id, type: demo.role.PermissionId}]\n",
        "    input:\n      - {name: permission_id, type: demo.role.PermissionId}\n      - {name: supplier, type: demo.role.ServiceRef}\n",
        1,
    );
    replaced(
        &text,
        "        sets: {supplier: {caller: service_id}}\n",
        "        sets: {supplier: input.supplier}\n",
        1,
    )
}

/// Swapped-caller rerun, one literal (the 761 shape): every emitted scenario still passes.
#[test]
fn adversary_u4_swapped_callers_one_literal() {
    check("swapped, one literal", &swappable(UPDATES), ALL_OUTCOMES);
}

/// The swapped-caller variant reaches the guarded scenario: its command under test is sent by two
/// actors, or a note says why the swap was left out, so the two cases above exercise the rerun.
#[test]
fn adversary_u4_swapped_callers_reach_the_guarded_scenario() {
    let text = swappable(&guarded("all", &telephony_voice()));
    let result = ess_conformance::synthesize::synthesize(&ir(&text));
    let document = document(&result);
    let guarded_id = format!("{UPDATE}/outcome/updated-with-telephony");
    let actors: std::collections::BTreeSet<String> = document["scenarios"][guarded_id.as_str()]
        ["steps"]
        .as_array()
        .map(|steps| {
            steps
                .iter()
                .filter(|step| {
                    step["step"] == "execute_command" && step["command"].to_string().contains(UPDATE)
                })
                .map(|step| step["actor"].to_string())
                .collect()
        })
        .unwrap_or_default();
    let noted = format!("{:?}", result.notes).contains("updated-with-telephony");
    assert!(
        actors.len() > 1 || noted,
        "swap not exercised: actors {actors:?}, noted {noted}\nnotes: {:#?}",
        result.notes
    );
}

/// Swapped-caller rerun, two literals: every emitted scenario still passes.
#[test]
fn adversary_u4_swapped_callers_two_literals() {
    let text = swappable(&guarded("all", &telephony_voice()));
    check("swapped, two literals", &text, ALL_OUTCOMES);
}
