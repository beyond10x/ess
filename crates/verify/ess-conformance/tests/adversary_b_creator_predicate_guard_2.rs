//! Adversary pass 2 on `story:a-row-set-branch-arranges-rows-past-the-creators-predicate-guard`:
//! the correction to pass 1 (`compared_optional`/`written`, `StoredRefusal::passes`, the wording)
//! attacked with variants of `tests/fixtures/row-set-creator-predicate-guard.yaml` — several
//! Optional fields, an Optional the creator's refusal wants absent, nested and struct Optionals, an
//! Optional input, a field the creator cannot write, refusals passable alone and not together, the
//! printed predicates read back as documents, and determinism.

mod support_row_sets;

use std::collections::BTreeMap;

use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::synthesize::{Refusal, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;

const BASE: &str = include_str!("fixtures/row-set-creator-predicate-guard.yaml");
const FOUND: &str = "demo.items.GetChild/outcome/found";
const TENANT_GUARD: &str = "predicate: tenant != input.tenant";

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

/// `text` with `branch` declared after `other-tenant`.
fn after_guard(text: &str, branch: &str) -> String {
    once(text, OTHER_TENANT, &format!("{OTHER_TENANT}{branch}"))
}

/// `text` with `branch` declared before `other-tenant`.
fn before_guard(text: &str, branch: &str) -> String {
    once(text, OTHER_TENANT, &format!("{branch}{OTHER_TENANT}"))
}

/// `text` with the tenant guard's predicate replaced by `predicate` (the text after `predicate: `).
fn guard(text: &str, predicate: &str) -> String {
    once(text, TENANT_GUARD, &format!("predicate: {predicate}"))
}

/// A refusing branch of `CreateChild` over the parent `input.parent_id` names.
fn branch(name: &str, predicate: &str) -> String {
    format!(
        "      - name: {name}\n        when_related:\n          via: input.parent_id\n          predicate: {predicate}\n        error: demo.items.NotFound\n"
    )
}

/// `text` with a type declared.
fn with_type(text: &str, declaration: &str) -> String {
    once(
        text,
        "  - name: demo.items.ChildRow\n    kind: newtype\n    of: String\n",
        &format!("  - name: demo.items.ChildRow\n    kind: newtype\n    of: String\n{declaration}"),
    )
}

/// `text` with a stored field on the parent, an input of `CreateParent` and, where given, the
/// `sets:` line writing the field from it.
fn parent_field(text: &str, field: &str, input: &str, set: Option<&str>) -> String {
    let text = once(
        text,
        "    relations:\n      - name: children\n",
        &format!("      - {field}\n    relations:\n      - name: children\n"),
    );
    let text = once(
        &text,
        "  - name: demo.items.CreateParent\n    input:\n",
        &format!("  - name: demo.items.CreateParent\n    input:\n      - {input}\n"),
    );
    match set {
        Some(set) => once(
            &text,
            "          tenant: input.tenant\n        emits: [demo.items.ParentCreated]",
            &format!(
                "          tenant: input.tenant\n          {set}\n        emits: [demo.items.ParentCreated]"
            ),
        ),
        None => text,
    }
}

/// `text` with an input of `CreateChild`.
fn child_input(text: &str, input: &str) -> String {
    once(
        text,
        "  - name: demo.items.CreateChild\n    input:\n",
        &format!("  - name: demo.items.CreateChild\n    input:\n      - {input}\n"),
    )
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

/// `optional_tenant()` with an Optional `region` on the parent, written by its creator, and a
/// `region` input of `CreateChild`.
fn optional_tenant_and_region() -> String {
    let text = parent_field(
        &optional_tenant(),
        "{name: region, type: Optional<String>}",
        "{name: region, type: Optional<String>, presence: omitted_when_absent}",
        Some("region: input.region"),
    );
    child_input(&text, "{name: region, type: String}")
}

struct Synthesized {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn synthesized(text: &str) -> Synthesized {
    let ir = support_row_sets::model(text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Synthesized { synthesis, results }
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

fn found_refused(text: &str) -> String {
    let synthesized = synthesized(text);
    let refusals = refusals_of(&synthesized, FOUND);
    let [refusal] = refusals.as_slice() else {
        panic!("{FOUND} is refused once: {refusals:#?}");
    };
    let message = refusal.cause.to_string();
    assert_eq!(refusal.code().to_string(), "ESS-SYNTH-001", "{message}");
    message
}

/// Every `when_related:` mapping the refusal prints, each between "(`" and "`)".
fn printed_when_related(message: &str) -> Vec<&str> {
    message
        .split("(`")
        .skip(1)
        .filter_map(|rest| rest.split_once("`)").map(|(inside, _)| inside))
        .filter(|inside| inside.starts_with("when_related: "))
        .collect()
}

/// The predicate a printed `when_related: {…}` holds, read back as a document.
fn read_back(printed: &str) -> Predicate {
    let node: Node = ess_yaml::from_str(printed)
        .unwrap_or_else(|error| panic!("{printed:?} reads back as YAML: {error}"));
    let predicate = node
        .as_map()
        .and_then(|map| map.get("when_related"))
        .and_then(Node::as_map)
        .and_then(|map| map.get("predicate"))
        .unwrap_or_else(|| panic!("{printed:?} holds a predicate: {node:?}"));
    Predicate::from_node(predicate)
        .unwrap_or_else(|error| panic!("{predicate:?} reads back as a predicate: {error}"))
}

/// `authored` (YAML for one predicate) read as a document.
fn authored(authored: &str) -> Predicate {
    let node: Node = ess_yaml::from_str(authored).unwrap();
    Predicate::from_node(&node).unwrap()
}

// F1 over two Optional fields, each compared with the input by its own refusal.
#[test]
fn adv2_two_optional_fields_each_compared_by_its_own_refusal() {
    let text = after_guard(
        &optional_tenant_and_region(),
        &branch("other-region", "region != input.region"),
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// F1 over two Optional fields compared with the input in one refusal.
#[test]
fn adv2_two_optional_fields_compared_in_one_refusal() {
    let text = guard(
        &optional_tenant_and_region(),
        "{any: [tenant != input.tenant, region != input.region]}",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// The creator refuses a parent holding a tenant, and one whose tenant is not the input's: every
// parent it accepts leaves `tenant != input.tenant` unknown, so `found` has no witness. The
// refusal names both refusals, the contradiction, and not only the comparison.
#[test]
fn adv2_optional_wanted_absent_beside_a_comparison_names_both_refusals() {
    let text = before_guard(&optional_tenant(), &branch("has-tenant", "defined(tenant)"));
    let message = found_refused(&text);
    for named in ["`has-tenant`", "`other-tenant`"] {
        assert!(message.contains(named), "names {named}: {message}");
    }
}

// The refusal reads an Optional `blocked_at` in the refusing polarity (`defined(blocked_at)`
// refuses) beside the comparison with the input. A parent without `blocked_at`, of the input's
// tenant, passes; arranging it toward `defined(blocked_at)` makes it fail for every input.
#[test]
fn adv2_optional_read_in_the_refusing_polarity_is_left_unwritten() {
    let text = parent_field(
        BASE,
        "{name: blocked_at, type: Optional<String>}",
        "{name: blocked_at, type: Optional<String>, presence: omitted_when_absent}",
        Some("blocked_at: input.blocked_at"),
    );
    let text = guard(
        &text,
        "{any: [\"defined(blocked_at)\", tenant != input.tenant]}",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

// Regression guard for the case above: the interpreter witnesses `found` on a parent without
// `blocked_at`.
#[test]
fn adv2_optional_in_the_refusing_polarity_found_is_witnessable() {
    use ess_conformance::target::ConformanceTarget as _;
    use support_row_sets::{outcome, request, target, text};
    let model = parent_field(
        BASE,
        "{name: blocked_at, type: Optional<String>}",
        "{name: blocked_at, type: Optional<String>, presence: omitted_when_absent}",
        Some("blocked_at: input.blocked_at"),
    );
    let model = guard(
        &model,
        "{any: [\"defined(blocked_at)\", tenant != input.tenant]}",
    );
    let target = target(&model);
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

/// `BASE` with a parent's `profile`, a struct whose `tenant` is Optional, compared with the input.
fn struct_member_model() -> String {
    let text = with_type(
        BASE,
        "  - name: demo.items.Profile\n    kind: struct\n    fields:\n      - {name: tenant, type: Optional<String>}\n",
    );
    let text = parent_field(
        &text,
        "{name: profile, type: demo.items.Profile}",
        "{name: profile, type: demo.items.Profile}",
        Some("profile: input.profile"),
    );
    guard(&text, "profile.tenant != input.tenant")
}

// An Optional struct field whose member is compared with the input.
#[test]
fn adv2_optional_struct_field_compared_with_the_input() {
    let text = with_type(
        BASE,
        "  - name: demo.items.Profile\n    kind: struct\n    fields:\n      - {name: tenant, type: String}\n",
    );
    let text = parent_field(
        &text,
        "{name: profile, type: Optional<demo.items.Profile>}",
        "{name: profile, type: Optional<demo.items.Profile>, presence: omitted_when_absent}",
        Some("profile: input.profile"),
    );
    let text = guard(&text, "profile.tenant != input.tenant");
    assert_witnessed(&synthesized(&text), FOUND);
}

// The refusal compares the stored tenant with an Optional input of the creator.
#[test]
fn adv2_refusal_compares_with_an_optional_input() {
    let text = once(
        BASE,
        "  - name: demo.items.CreateChild\n    input:\n      - name: tenant\n        type: String\n",
        "  - name: demo.items.CreateChild\n    input:\n      - name: tenant\n        type: Optional<String>\n        presence: omitted_when_absent\n",
    );
    assert_witnessed(&synthesized(&text), FOUND);
}

/// `optional_tenant()` with the tenant written by `SetTenant` after the parent is created.
fn later_writer_model() -> String {
    let text = once(
        &optional_tenant(),
        "        sets:\n          tenant: input.tenant\n        emits: [demo.items.ParentCreated]",
        "        emits: [demo.items.ParentCreated]",
    );
    let text = once(
        &text,
        "  - name: demo.items.CreateChild\n",
        "  - name: demo.items.SetTenant\n    input:\n      - {name: parent_id, type: demo.items.ParentId}\n      - {name: tenant, type: String}\n    outcomes:\n      - name: set\n        updates: demo.items.Parent\n        instance: parent_id\n        sets: {tenant: input.tenant}\n        emits: [demo.items.ParentCreated]\n        payload: {demo.items.ParentCreated: {parent_id: input.parent_id}}\n\n  - name: demo.items.CreateChild\n",
    );
    once(
        &text,
        "      - demo.items.CreateChild\n",
        "      - demo.items.SetTenant\n      - demo.items.CreateChild\n",
    )
}

// No command writes the parent's Optional tenant: `defined(tenant)` cannot be arranged, every
// parent leaves the comparison unknown, and the refusal names the comparison, not `exists: false`.
#[test]
fn adv2_optional_no_command_writes_names_the_comparison() {
    let text = once(
        &optional_tenant(),
        "        sets:\n          tenant: input.tenant\n        emits: [demo.items.ParentCreated]",
        "        emits: [demo.items.ParentCreated]",
    );
    let message = found_refused(&text);
    assert!(message.contains("`other-tenant`"), "{message}");
    assert!(!message.contains("exists: false"), "{message}");
}

// F2: two refusals each passable alone (the input of the parent's tenant passes the first, any
// other the second) and not together. Both are named.
#[test]
fn adv2_refusals_passable_alone_not_together_are_both_named() {
    let text = after_guard(BASE, &branch("same-tenant", "tenant == input.tenant"));
    let message = found_refused(&text);
    for named in ["`other-tenant`", "`same-tenant`"] {
        assert!(message.contains(named), "names {named}: {message}");
    }
    assert!(message.contains(" refuse the "), "{message}");
}

// F2 over the row alone: each refusal is passed by some parent, none by every parent. Both are
// named, as the contradiction they are.
#[test]
fn adv2_stored_refusals_passable_alone_not_together_are_both_named() {
    let text = guard(BASE, "tenant == \"home\"");
    let text = once(
        &text,
        "          predicate: tenant == \"home\"\n        error: demo.items.NotFound\n",
        &format!(
            "          predicate: tenant == \"home\"\n        error: demo.items.NotFound\n{}",
            branch("away", "tenant != \"home\"")
        ),
    );
    let message = found_refused(&text);
    for named in ["`other-tenant`", "`away`"] {
        assert!(message.contains(named), "names {named}: {message}");
    }
}

// F3: an unpassable refusal holding a text literal with a comma, a colon and quotes, nested in
// any/all/not, prints a `when_related:` that reads back as the predicate written.
#[test]
fn adv2_printed_nested_text_literal_reads_back() {
    let predicate =
        "{any: [state == Open, {not: {all: ['tenant == \"a, b: c\"', tenant != input.tenant]}}]}";
    let message = found_refused(&guard(BASE, predicate));
    let printed = printed_when_related(&message);
    let [only] = printed.as_slice() else {
        panic!("one guard printed: {message}");
    };
    assert_eq!(read_back(only), authored(predicate), "{message}");
}

// F3: a plain text literal reads back too.
#[test]
fn adv2_printed_text_literal_reads_back() {
    let predicate = "{any: [state == Open, tenant == \"home\"]}";
    let message = found_refused(&guard(BASE, predicate));
    let printed = printed_when_related(&message);
    let [only] = printed.as_slice() else {
        panic!("one guard printed: {message}");
    };
    assert_eq!(read_back(only), authored(predicate), "{message}");
}

// F3: an ordering of a stored Timestamp against `now` reads back.
#[test]
fn adv2_printed_now_comparison_reads_back() {
    let text = parent_field(
        BASE,
        "{name: opened_at, type: Timestamp}",
        "{name: opened_at, type: Timestamp}",
        Some("opened_at: input.opened_at"),
    );
    let predicate = "{any: [state == Open, opened_at > now - 60s]}";
    let message = found_refused(&guard(&text, predicate));
    let printed = printed_when_related(&message);
    let [only] = printed.as_slice() else {
        panic!("one guard printed: {message}");
    };
    let back = read_back(only);
    eprintln!("PRINTED: {only}\nREAD BACK: {back:?}");
}

// Determinism: two unpassable refusals, declared `zeta` then `alpha`, print in declaration order,
// and the suite and refusals are the same bytes on a second synthesis.
#[test]
fn adv2_named_refusals_print_in_declaration_order_and_deterministically() {
    let text = guard(BASE, "state == Open");
    let text = once(
        &text,
        "      - name: other-tenant\n",
        "      - name: zeta\n",
    );
    let text = once(
        &text,
        "          predicate: state == Open\n        error: demo.items.NotFound\n",
        &format!(
            "          predicate: state == Open\n        error: demo.items.NotFound\n{}",
            branch("alpha", "{any: [state == Open, tenant == \"x\"]}")
        ),
    );
    let first = found_refused(&text);
    let zeta = first.find("`zeta`").unwrap_or_else(|| panic!("{first}"));
    let alpha = first.find("`alpha`").unwrap_or_else(|| panic!("{first}"));
    assert!(zeta < alpha, "declaration order: {first}");
    let one = ess_conformance::synthesize::synthesize(&support_row_sets::model(&text));
    let two = ess_conformance::synthesize::synthesize(&support_row_sets::model(&text));
    assert_eq!(format!("{:?}", one.suite), format!("{:?}", two.suite));
    let causes = |synthesis: &Synthesis| -> Vec<String> {
        synthesis
            .refusals
            .iter()
            .map(|refusal| refusal.cause.to_string())
            .collect()
    };
    assert_eq!(causes(&one), causes(&two));
}

/// On the interpreter: a parent created with `parent` (and then sent `then`, if given, with its
/// `parent_id`), a child of tenant `t` under it, and `GetChild` answering `found`.
fn interpreter_finds(model: &str, parent: &[(&str, Node)], then: Option<(&str, &[(&str, Node)])>) {
    use ess_conformance::target::ConformanceTarget as _;
    use support_row_sets::{outcome, request, target, text};
    let target = target(model);
    let created = target
        .execute_command(request("demo.items.CreateParent", parent))
        .unwrap();
    assert_eq!(outcome(&created), "created");
    let parent_id = created.direct_events[0].payload["parent_id"].clone();
    if let Some((command, input)) = then {
        let mut input = input.to_vec();
        input.push(("parent_id", parent_id.clone()));
        let done = target.execute_command(request(command, &input)).unwrap();
        assert_ne!(outcome(&done), "<none>");
    }
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

// Regression guard: `found` of the struct-member model is witnessed on the interpreter.
#[test]
fn adv2_optional_member_of_a_struct_found_is_witnessable() {
    use support_row_sets::text;
    let profile = Node::Map([("tenant".to_owned(), text("t"))].into_iter().collect());
    interpreter_finds(
        &struct_member_model(),
        &[("tenant", text("t")), ("profile", profile)],
        None,
    );
}

// Regression guard: `found` of the later-writer model is witnessed on the interpreter.
#[test]
fn adv2_optional_written_only_by_a_later_command_found_is_witnessable() {
    use support_row_sets::text;
    interpreter_finds(
        &later_writer_model(),
        &[],
        Some(("demo.items.SetTenant", &[("tenant", text("t"))])),
    );
}

// F2 on a refusal comparing with the input by `==` alone: any input of another tenant passes.
#[test]
fn adv2_refusal_of_the_same_tenant_alone() {
    assert_witnessed(&synthesized(&guard(BASE, "tenant == input.tenant")), FOUND);
}

// Control for `adv2_optional_read_in_the_refusing_polarity_is_left_unwritten`: the same Optional
// refused present by a refusal reading the row alone is left unwritten, and `found` is witnessed.
#[test]
fn adv2_optional_refused_present_by_a_stored_refusal_alone() {
    let text = parent_field(
        BASE,
        "{name: blocked_at, type: Optional<String>}",
        "{name: blocked_at, type: Optional<String>, presence: omitted_when_absent}",
        Some("blocked_at: input.blocked_at"),
    );
    let text = after_guard(&text, &branch("blocked", "defined(blocked_at)"));
    assert_witnessed(&synthesized(&text), FOUND);
}
