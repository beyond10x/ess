//! Adversary cases for `story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation`
//! (turn 2): variants of `fixtures/list-identity-guard-beside-updates.yaml` built by text
//! substitution, as `adversary_u4_list_literals` builds them.
//!
//! The candidates the story adds fill every literal list on top of each earlier candidate, then
//! leave each literal out of its list on the base input alone, the whole addition bounded by
//! `MAX_CANDIDATES`. These cases drive a negated literal beside another input, and a negated
//! literal beside enough other leaves that the earlier candidates fill that bound.

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

fn exists(indent: usize, list: &str, literal: &str) -> String {
    format!(
        "{:indent$}- exists: {{in: input.{list}, as: e, that: 'e == \"{literal}\"'}}\n",
        ""
    )
}

fn not_exists(indent: usize, list: &str, literal: &str) -> String {
    format!(
        "{:indent$}- not: {{exists: {{in: input.{list}, as: e, that: 'e == \"{literal}\"'}}}}\n",
        ""
    )
}

fn guarded(connective: &str, children: &[String]) -> String {
    let mut guard = format!("          {connective}:\n");
    for child in children {
        guard.push_str(child);
    }
    replaced(UPDATES, GUARD, &guard, 1)
}

/// The command's input gains `fields` (each a flow mapping), after `permissions`.
fn with_inputs(text: &str, fields: &[String]) -> String {
    let anchor = "      - {name: permissions, type: \"List<demo.role.PermissionId>\"}\n    outcomes:\n      - name: unknown-permission\n        when_related: {via: input.permissions, exists: false}\n        error: demo.role.UnknownPermission\n      - name: updated-with-telephony\n";
    let mut added =
        String::from("      - {name: permissions, type: \"List<demo.role.PermissionId>\"}\n");
    for field in fields {
        added.push_str("      - ");
        added.push_str(field);
        added.push('\n');
    }
    added.push_str("    outcomes:\n      - name: unknown-permission\n        when_related: {via: input.permissions, exists: false}\n        error: demo.role.UnknownPermission\n      - name: updated-with-telephony\n");
    replaced(text, anchor, &added, 1)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary-t2.yaml"), raw)])
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

/// Every emitted scenario passes against the model and every outcome in `wanted` is witnessed.
fn check(label: &str, text: &str, wanted: &[&str]) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
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
    assert!(
        failed.is_empty() && missing.is_empty(),
        "{label}:\nfailing scenarios:\n{}\nnot witnessed: {missing:#?}\nall refusals: {:#?}",
        failed.join("\n"),
        refusals(&result)
    );
}

fn mutant_caught(label: &str, text: &str, mutant: &str) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let against_model = statuses(&result, &model);
    let against_mutant = statuses(&result, &ir(mutant));
    let caught = against_mutant
        .values()
        .any(|status| *status != Status::Passed);
    assert!(
        caught
            && against_model
                .values()
                .all(|status| *status == Status::Passed),
        "{label}: the mutant passes every scenario ({} run); against the model: \
         {against_model:#?}\nrefusals: {:#?}",
        against_mutant.len(),
        refusals(&result)
    );
}

/// Two literals held and a third excluded, from one list.
fn held_held_excluded() -> Vec<String> {
    vec![
        exists(12, "permissions", "telephony"),
        exists(12, "permissions", "voice"),
        not_exists(12, "permissions", "fax"),
    ]
}

/// The story's variant 3 under the story's own `check`: the guarded branch (telephony held,
/// voice excluded) is witnessed too, not only the else branch the unit's case asserts.
#[test]
fn adversary_t2_list_literal_variant_three_guarded_branch_witnessed() {
    let text = guarded(
        "all",
        &[
            exists(12, "permissions", "telephony"),
            not_exists(12, "permissions", "voice"),
        ],
    );
    check("variant 3 guarded branch", &text, ALL_OUTCOMES);
}

/// Negation nested two levels down: `not any [not exists telephony, exists voice]` is telephony
/// held and voice excluded.
#[test]
fn adversary_t2_list_literal_double_negation_witnessed() {
    let inner = format!(
        "            - not:\n                any:\n{}{}",
        not_exists(18, "permissions", "telephony"),
        exists(18, "permissions", "voice"),
    );
    let text = guarded("all", &[exists(12, "permissions", "telephony"), inner]);
    check("double negation", &text, ALL_OUTCOMES);
}

/// Two literals held, one excluded, beside a leaf whose base value satisfies its comparison:
/// the leave-one-out candidate on the base input witnesses it, and dropping the negated child
/// is caught.
#[test]
fn adversary_t2_list_literal_negation_beside_base_satisfied_leaf() {
    let mut children = held_held_excluded();
    children.push("            - 'a0 < 100'\n".to_string());
    let text = with_inputs(
        &guarded("all", &children),
        &["{name: a0, type: Integer}".to_string()],
    );
    check(
        "negation beside one base-satisfied leaf",
        &text,
        ALL_OUTCOMES,
    );
    let mut kept = held_held_excluded();
    kept.pop();
    kept.push("            - 'a0 < 100'\n".to_string());
    let mutant = with_inputs(
        &guarded("all", &kept),
        &["{name: a0, type: Integer}".to_string()],
    );
    mutant_caught("negated child dropped", &text, &mutant);
}

/// Two literals held, one excluded, beside `level > 3`, which the base value refutes (the
/// story's variant 2 with a negated literal added): the leave-one-out candidates sit on the base
/// input only, where `level > 3` fails, and every candidate that raises `level` holds `fax`.
#[test]
#[ignore = "backlog: story:a-negated-list-literal-is-witnessed-beside-any-other-leaf"]
fn adversary_t2_list_literal_negation_beside_base_refuted_leaf() {
    let mut children = held_held_excluded();
    children.push("            - 'level > 3'\n".to_string());
    let text = with_inputs(
        &guarded("all", &children),
        &["{name: level, type: Integer}".to_string()],
    );
    check("negation beside level > 3", &text, ALL_OUTCOMES);
}

/// The same guard beside eight leaves the base satisfies: the walk's candidates fill
/// `MAX_CANDIDATES`, the "every list full on every candidate" fills spend the whole bound, and the
/// leave-one-out candidate that alone witnesses the branch is never appended.
#[test]
#[ignore = "backlog: story:a-negated-list-literal-is-witnessed-beside-any-other-leaf"]
fn adversary_t2_list_literal_negation_beside_many_leaves_survives_the_bound() {
    let mut children = held_held_excluded();
    let mut fields = Vec::new();
    for at in 0..8 {
        children.push(format!("            - 'a{at} < 100'\n"));
        fields.push(format!("{{name: a{at}, type: Integer}}"));
    }
    let text = with_inputs(&guarded("all", &children), &fields);
    check("negation beside eight leaves", &text, ALL_OUTCOMES);
}
