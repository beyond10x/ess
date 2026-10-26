//! Adversary pass on `story:a-row-set-branch-arranges-rows-past-the-creators-predicate-guard`:
//! variants of `tests/fixtures/row-set-creator-predicate-guard.yaml` the unit's own cases do not
//! reach — an Optional related field, an enum guard, several guards, a related row created behind
//! a guard of its own, and a target that ignores the creator's predicate guard.

mod support_row_sets;

use std::collections::BTreeMap;

use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::synthesize::{Refusal, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};

const BASE: &str = include_str!("fixtures/row-set-creator-predicate-guard.yaml");
const FOUND: &str = "demo.items.GetChild/outcome/found";

const OTHER_TENANT: &str = "      - name: other-tenant
        when_related:
          via: input.parent_id
          predicate: tenant != input.tenant
        error: demo.items.NotFound
";

fn once(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "{from:?} occurs once");
    text.replacen(from, to, 1)
}

/// `BASE` with `branch` declared after `other-tenant`.
fn beside(text: &str, branch: &str) -> String {
    once(text, OTHER_TENANT, &format!("{OTHER_TENANT}{branch}"))
}

/// `BASE` with the parent's `tenant` Optional on the entity, its creator and the view.
fn optional_tenant() -> String {
    let text = once(
        BASE,
        "      - name: tenant\n        type: String\n    relations:",
        "      - name: tenant\n        type: Optional<String>\n    relations:",
    );
    let text = once(
        &text,
        "  - name: demo.items.CreateParent\n    input:\n      - name: tenant\n        type: String\n",
        "  - name: demo.items.CreateParent\n    input:\n      - name: tenant\n        type: Optional<String>\n        presence: omitted_when_absent\n",
    );
    once(
        &text,
        "        type: demo.items.Parent.State\n      - name: tenant\n        type: String",
        "        type: demo.items.Parent.State\n      - name: tenant\n        type: Optional<String>",
    )
}

/// `BASE` with an enum `kind` on the parent, written by its creator.
fn with_kind(text: &str) -> String {
    let text = once(
        text,
        "  - name: demo.items.ChildRow\n    kind: newtype\n    of: String\n",
        "  - name: demo.items.ChildRow\n    kind: newtype\n    of: String\n  - {name: demo.items.Kind, kind: enum, variants: [plain, special]}\n",
    );
    let text = once(
        &text,
        "      - name: tenant\n        type: String\n    relations:",
        "      - name: tenant\n        type: String\n      - name: kind\n        type: demo.items.Kind\n    relations:",
    );
    let text = once(
        &text,
        "  - name: demo.items.CreateParent\n    input:\n      - name: tenant\n        type: String\n",
        "  - name: demo.items.CreateParent\n    input:\n      - name: tenant\n        type: String\n      - name: kind\n        type: demo.items.Kind\n",
    );
    once(
        &text,
        "        sets:\n          tenant: input.tenant\n",
        "        sets:\n          tenant: input.tenant\n          kind: input.kind\n",
    )
}

/// `text` with every parent filed under an organisation its creator refuses missing.
fn parent_behind_org(text: &str) -> String {
    let text = once(
        text,
        "  - name: demo.items.ChildRow\n    kind: newtype\n    of: String\n",
        "  - name: demo.items.ChildRow\n    kind: newtype\n    of: String\n  - name: demo.items.OrgId\n    kind: newtype\n    of: String\n",
    );
    let text = once(
        &text,
        "entities:\n",
        "entities:\n  - name: demo.items.Org\n    identity:\n      name: id\n      type: demo.items.OrgId\n    fields:\n      - name: label\n        type: String\n    lifecycle:\n      initial: Live\n      states: [Live]\n      terminal: [Live]\n\n",
    );
    let text = once(
        &text,
        "      - name: tenant\n        type: String\n    relations:",
        "      - name: tenant\n        type: String\n      - name: org_id\n        type: demo.items.OrgId\n    relations:",
    );
    let text = once(
        &text,
        "  - name: demo.items.CreateParent\n    input:\n      - name: tenant\n        type: String\n    outcomes:\n",
        "  - name: demo.items.CreateOrg\n    input:\n      - name: label\n        type: String\n    outcomes:\n      - name: created\n        creates: demo.items.Org\n        instance: org_id\n        sets:\n          label: input.label\n        emits: [demo.items.OrgCreated]\n        payload:\n          demo.items.OrgCreated:\n            org_id: {generated: true}\n\n  - name: demo.items.CreateParent\n    input:\n      - name: tenant\n        type: String\n      - name: org_id\n        type: demo.items.OrgId\n    outcomes:\n      - name: org-not-found\n        when_related:\n          via: input.org_id\n          exists: false\n        error: demo.items.NotFound\n",
    );
    let text = once(
        &text,
        "        sets:\n          tenant: input.tenant\n",
        "        sets:\n          tenant: input.tenant\n          org_id: input.org_id\n",
    );
    let text = once(
        &text,
        "      - demo.items.CreateParent\n",
        "      - demo.items.CreateOrg\n      - demo.items.CreateParent\n",
    );
    once(
        &text,
        "events:\n",
        "events:\n  - name: demo.items.OrgCreated\n    fields:\n      - name: org_id\n        type: demo.items.OrgId\n",
    )
}

struct Synthesized {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str, target: &str) -> Synthesized {
    let ir = support_row_sets::model(text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &Interpreted::for_model(support_row_sets::model(target)),
        )
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Synthesized { synthesis, results }
}

fn synthesized(text: &str) -> Synthesized {
    run(text, text)
}

fn refusals_of<'s>(synthesized: &'s Synthesized, wanted: &str) -> Vec<&'s Refusal> {
    synthesized
        .synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(wanted)
        })
        .collect()
}

fn assert_witnessed(synthesized: &Synthesized, wanted: &str) {
    let refusals: Vec<String> = refusals_of(synthesized, wanted)
        .into_iter()
        .map(|refusal| format!("[{}] {}", refusal.code(), refusal.cause))
        .collect();
    assert!(
        refusals.is_empty(),
        "{wanted} is not refused: {refusals:#?}"
    );
    let result = synthesized.results.get(wanted).unwrap_or_else(|| {
        panic!(
            "{wanted} is synthesized: {:#?}",
            synthesized.results.keys().collect::<Vec<_>>()
        )
    });
    assert_eq!(
        result.status,
        Status::Passed,
        "{wanted} passes: {:#?}",
        result.diagnostics().collect::<Vec<_>>()
    );
}

fn only_refusal(synthesized: &Synthesized) -> (String, &'static str) {
    let refusals = refusals_of(synthesized, FOUND);
    let [refusal] = refusals.as_slice() else {
        panic!("{FOUND} is refused once: {refusals:#?}");
    };
    assert_eq!(refusal.code().to_string(), "ESS-SYNTH-001");
    (refusal.cause.to_string(), refusal.cause.hint())
}

// The story's own Fit review: "one parent entity with an optional tenant". A parent without a
// tenant is refused, and so is one of another tenant.
#[test]
fn adv_optional_related_tenant_compared_with_the_input() {
    let text = once(
        &optional_tenant(),
        "predicate: tenant != input.tenant",
        "predicate: {any: [\"not defined(tenant)\", tenant != input.tenant]}",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// The story's guard as written (`tenant != input.tenant`) over the Optional tenant.
#[test]
fn adv_optional_related_tenant_bare_guard() {
    assert_witnessed(&synthesized(&optional_tenant()), FOUND);
}

// Regression guard for the two cases above: `found` has a witness on the interpreter (a parent of
// tenant `t`, a child sent with tenant `t`), so the refusal is synthesis's, not the model's.
#[test]
fn adv_optional_related_tenant_found_is_witnessable() {
    use ess_conformance::target::ConformanceTarget as _;
    use support_row_sets::{outcome, request, target, text};
    let target = target(&optional_tenant());
    let parent = target
        .execute_command(request("demo.items.CreateParent", &[("tenant", text("t"))]))
        .unwrap();
    assert_eq!(outcome(&parent), "created");
    let parent_id = parent.direct_events[0].payload["parent_id"].clone();
    let child = target
        .execute_command(request(
            "demo.items.CreateChild",
            &[
                ("tenant", text("t")),
                ("parent_id", parent_id.clone()),
                ("id", text("c")),
            ],
        ))
        .unwrap();
    assert_eq!(outcome(&child), "created");
    let read = target
        .execute_command(request(
            "demo.items.GetChild",
            &[("parent_id", parent_id), ("id", text("c"))],
        ))
        .unwrap();
    assert_eq!(outcome(&read), "found");
}

// A stored-only guard on the Optional field: parents without a tenant are refused.
#[test]
fn adv_optional_related_tenant_required_by_a_stored_guard() {
    let text = beside(
        &optional_tenant(),
        "      - name: no-tenant\n        when_related:\n          via: input.parent_id\n          predicate: not defined(tenant)\n        error: demo.items.NotFound\n",
    );
    let text = once(
        &text,
        "predicate: tenant != input.tenant",
        "predicate: {all: [\"defined(tenant)\", tenant != input.tenant]}",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

#[test]
fn adv_enum_guard_on_the_related_row() {
    let text = once(
        &with_kind(BASE),
        "predicate: tenant != input.tenant",
        "predicate: kind != special",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

#[test]
fn adv_several_satisfiable_guards_stored_and_input() {
    let text = beside(
        &with_kind(BASE),
        "      - name: not-special\n        when_related:\n          via: input.parent_id\n          predicate: kind != special\n        error: demo.items.NotFound\n      - name: not-home\n        when_related:\n          via: input.parent_id\n          predicate: tenant != \"home\"\n        error: demo.items.NotFound\n",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// `other-tenant` is passable (the input is steered to the parent's tenant); only `open` refuses
// every parent. The refusal names the guard that refused, and not the one that did not.
#[test]
fn adv_unsatisfiable_beside_satisfiable_names_only_the_refusing_guard() {
    let text = beside(
        BASE,
        "      - name: open\n        when_related:\n          via: input.parent_id\n          predicate: state == Open\n        error: demo.items.NotFound\n",
    );
    let (message, _) = only_refusal(&synthesized(&text));
    assert!(message.contains("`open`"), "{message}");
    assert!(
        !message.contains("`other-tenant`"),
        "names only the refusing guard: {message}"
    );
}

#[test]
fn adv_unreachable_refusal_wording_and_hint() {
    let text = once(
        BASE,
        "predicate: tenant != input.tenant",
        "predicate: state == Open",
    );
    let (message, hint) = only_refusal(&synthesized(&text));
    eprintln!("MESSAGE: {message}\nHINT: {hint}");
    assert!(message.contains("`demo.items.CreateChild`"), "{message}");
    assert!(message.contains("predicate: state == Open"), "{message}");
    assert!(hint.contains("ess-scenario/1"), "{hint}");
}

#[test]
fn adv_related_row_created_behind_its_own_guard_input_comparison() {
    assert_witnessed(&synthesized(&parent_behind_org(BASE)), FOUND);
}

#[test]
fn adv_related_row_created_behind_its_own_guard_stored_guard() {
    let text = once(
        &parent_behind_org(BASE),
        "predicate: tenant != input.tenant",
        "predicate: tenant != \"home\"",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// The creator accepts only an approved parent: the related row is created and then moved.
#[test]
fn adv_related_row_moved_before_the_creator_accepts_it() {
    let text = once(
        BASE,
        "    lifecycle:\n      initial: Open\n      states: [Open]\n      terminal: [Open]\n",
        "    lifecycle:\n      initial: Open\n      states: [Open, Approved]\n      terminal: [Approved]\n      transitions:\n        - {name: approve, from: [Open], to: Approved}\n",
    );
    let text = once(
        &text,
        "  - name: demo.items.CreateChild\n",
        "  - name: demo.items.ApproveParent\n    input:\n      - name: parent_id\n        type: demo.items.ParentId\n    outcomes:\n      - {name: already, when_subject_state: Approved, error: demo.items.NotFound}\n      - name: approved\n        when_subject_state: Open\n        moves: demo.items.Parent.approve\n        instance: parent_id\n        emits: [demo.items.ParentCreated]\n        payload:\n          demo.items.ParentCreated:\n            parent_id: input.parent_id\n\n  - name: demo.items.CreateChild\n",
    );
    let text = once(
        &text,
        "      - demo.items.CreateChild\n",
        "      - demo.items.ApproveParent\n      - demo.items.CreateChild\n",
    );
    let text = beside(
        &text,
        "      - name: unapproved\n        when_related:\n          via: input.parent_id\n          predicate: state != Approved\n        error: demo.items.NotFound\n",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// A target that ignores the creator's predicate guard fails the synthesized suite.
#[test]
fn adv_target_ignoring_the_creators_predicate_guard_is_caught() {
    let ignoring = once(BASE, OTHER_TENANT, "");
    let synthesized = run(BASE, &ignoring);
    let failed: Vec<&String> = synthesized
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(
        failed
            .iter()
            .any(|id| id.as_str() == "demo.items.CreateChild/outcome/other-tenant"),
        "the guard's own scenario fails: {failed:#?}"
    );
}

// Same, for a stored-only guard filled by the arrangement.
#[test]
fn adv_target_ignoring_a_stored_only_guard_is_caught() {
    let text = once(
        BASE,
        "predicate: tenant != input.tenant",
        "predicate: tenant != \"home\"",
    );
    let ignoring = once(BASE, OTHER_TENANT, "");
    let synthesized = run(&text, &ignoring);
    assert!(
        synthesized.results.iter().any(|(id, result)| id
            == "demo.items.CreateChild/outcome/other-tenant"
            && result.status != Status::Passed),
        "the guard's own scenario fails: {:#?}",
        synthesized.results.keys().collect::<Vec<_>>()
    );
}
