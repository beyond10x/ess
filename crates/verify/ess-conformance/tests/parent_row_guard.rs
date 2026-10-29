//! A guard on the parent row the input addresses holds on the creator, reader and deleter of its
//! child rows.
//!
//! `demo.d.Parent` holds an `Optional` tenant and owns `demo.d.Child` through `parent_id`.
//! `CreateChild` refuses a missing parent, then a parent of another tenant
//! (`when_related: {via: input.parent_id, predicate: …}`, absent matching absent). `GetChild`
//! addresses the parent and reads its child through the row set `{parent_id == input.parent_id,
//! child_id == input.child_id}`. `DeleteChild` addresses the parent, refuses another tenant with
//! `when_subject:`, and removes the selected child through `affects:`.
//!
//! Two synthesis defects met here:
//!
//! * The rows the reader's selector reads under the addressed parent were created at their own
//!   distinction's tenant, which the creator's guard refused, so `GetChild/found` was refused as
//!   ESS-SYNTH-001 naming the guarded creator. They are now created steered on the parent as it
//!   was created.
//! * The `affects:` segment of `DeleteChild/deleted` arranged its subject without a tenant and
//!   sent the deleter the tenant its plain witness gives, so the deleter answered `other-tenant`
//!   and the interpreted target failed the scenario. The input is now chosen again where the
//!   `when_subject:` refusal takes it on the arranged subject, and the segment is refused by name
//!   where no input passes it.
//!
//! Every model is inline, so it is no repository model the suite-bytes tables walk.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// The creator's refusal of a parent of another tenant, absent matching absent, as [`MODEL`]
/// writes it.
const CREATOR_GUARD: &str = r#"      - name: other-tenant
        when_related:
          via: input.parent_id
          predicate:
            any:
              - all: ["defined(tenant)", "defined(input.tenant)", tenant != input.tenant]
              - all: ["defined(tenant)", "not defined(input.tenant)"]
              - all: ["not defined(tenant)", "defined(input.tenant)"]
        error: demo.d.NotFound
"#;

/// The deleter's refusal of a subject of another tenant, as [`MODEL`] writes it.
const SUBJECT_GUARD: &str = r#"      - name: other-tenant
        when_subject:
          predicate:
            any:
              - all: ["defined(tenant)", "defined(input.tenant)", tenant != input.tenant]
              - all: ["defined(tenant)", "not defined(input.tenant)"]
              - all: ["not defined(tenant)", "defined(input.tenant)"]
        error: demo.d.NotFound
"#;

const MODEL: &str = r#"format: ess/32
system: demo
version: v1
domain: demo.d
summary: A parent row holds a tenant; its child rows are created, read and deleted under it.
types:
  - {name: demo.d.ParentId, kind: newtype, of: String}
  - {name: demo.d.ChildId, kind: newtype, of: String}
  - {name: demo.d.ChildRow, kind: newtype, of: String}
entities:
  - name: demo.d.Parent
    identity: {name: id, type: demo.d.ParentId}
    fields:
      - {name: tenant, type: Optional<String>}
    relations:
      - {name: children, kind: owns, target: demo.d.Child, cardinality: many, via: parent_id}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.d.Child
    identity: {name: row, type: demo.d.ChildRow}
    fields:
      - {name: child_id, type: demo.d.ChildId}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: tenant, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
commands:
  - name: demo.d.CreateParent
    input:
      - {name: tenant, type: Optional<String>}
    outcomes:
      - name: created
        creates: demo.d.Parent
        instance: parent_id
        sets: {tenant: input.tenant}
        emits: [demo.d.ParentCreated]
        payload: {demo.d.ParentCreated: {parent_id: {generated: true}}}
  - name: demo.d.CreateChild
    input:
      - {name: tenant, type: Optional<String>}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: child_id, type: demo.d.ChildId}
    outcomes:
      - name: parent-not-found
        when_related: {via: input.parent_id, exists: false}
        error: demo.d.NotFound
      - name: other-tenant
        when_related:
          via: input.parent_id
          predicate:
            any:
              - all: ["defined(tenant)", "defined(input.tenant)", tenant != input.tenant]
              - all: ["defined(tenant)", "not defined(input.tenant)"]
              - all: ["not defined(tenant)", "defined(input.tenant)"]
        error: demo.d.NotFound
      - name: created
        creates: demo.d.Child
        instance: row
        sets: {child_id: input.child_id, parent_id: input.parent_id, tenant: input.tenant}
        emits: [demo.d.ChildCreated]
        payload:
          demo.d.ChildCreated: {row: {generated: true}, child_id: input.child_id, parent_id: input.parent_id}
  - name: demo.d.GetChild
    input:
      - {name: tenant, type: Optional<String>}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: child_id, type: demo.d.ChildId}
    outcomes:
      - name: not-found
        unknown_instance: true
        error: demo.d.NotFound
      - name: child-not-found
        when_related:
          entity: demo.d.Child
          where: {all: [parent_id == input.parent_id, child_id == input.child_id]}
          exists: false
        error: demo.d.NotFound
      - name: found
        preserves: demo.d.Parent
        instance: parent_id
  - name: demo.d.DeleteChild
    input:
      - {name: tenant, type: Optional<String>}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: child_id, type: demo.d.ChildId}
    outcomes:
      - name: not-found
        unknown_instance: true
        error: demo.d.NotFound
      - name: other-tenant
        when_subject:
          predicate:
            any:
              - all: ["defined(tenant)", "defined(input.tenant)", tenant != input.tenant]
              - all: ["defined(tenant)", "not defined(input.tenant)"]
              - all: ["not defined(tenant)", "defined(input.tenant)"]
        error: demo.d.NotFound
      - name: deleted
        updates: demo.d.Parent
        instance: parent_id
        affects:
          - entity: demo.d.Child
            where: {all: [parent_id == subject.id, child_id == input.child_id]}
            deletes: demo.d.Child
        emits: [demo.d.ChildDeleted]
        payload:
          demo.d.ChildDeleted: {parent_id: input.parent_id, child_id: input.child_id}
errors:
  - name: demo.d.NotFound
events:
  - name: demo.d.ParentCreated
    fields: [{name: parent_id, type: demo.d.ParentId}]
  - name: demo.d.ChildCreated
    fields:
      - {name: row, type: demo.d.ChildRow}
      - {name: child_id, type: demo.d.ChildId}
      - {name: parent_id, type: demo.d.ParentId}
  - name: demo.d.ChildDeleted
    fields:
      - {name: child_id, type: demo.d.ChildId}
      - {name: parent_id, type: demo.d.ParentId}
views:
  - name: demo.d.Parents
    source: demo.d.Parent
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.d.ParentId}
      - {name: state, type: demo.d.Parent.State}
      - {name: tenant, type: Optional<String>}
  - name: demo.d.AllChildren
    source: demo.d.Child
    consistency: read_your_writes
    fields:
      - {name: row, type: demo.d.ChildRow}
      - {name: child_id, type: demo.d.ChildId}
      - {name: state, type: demo.d.Child.State}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: tenant, type: Optional<String>}
"#;

/// A view of one parent's children scoped to a tenant, absent matching absent.
const CHILDREN_VIEW: &str = r#"  - name: demo.d.Children
    source: demo.d.Child
    consistency: read_your_writes
    params:
      - {name: parent_id, type: demo.d.ParentId}
      - {name: tenant, type: Optional<String>}
    filter:
      all:
        - parent_id == param.parent_id
        - any:
            - all: ["defined(param.tenant)", "defined(tenant)", tenant == param.tenant]
            - all: ["not defined(param.tenant)", "not defined(tenant)"]
    fields:
      - {name: child_id, type: demo.d.ChildId}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: tenant, type: Optional<String>}
"#;

const FOUND: &str = "demo.d.GetChild/outcome/found";
const CHILD_NOT_FOUND: &str = "demo.d.GetChild/outcome/child-not-found";
const DELETED: &str = "demo.d.DeleteChild/outcome/deleted";

/// `text` with `part`, which it holds once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

/// [`MODEL`] with a required `String` tenant everywhere, each tenant guard `tenant !=
/// input.tenant`.
fn required_tenant() -> String {
    let text = replaced(
        MODEL,
        CREATOR_GUARD,
        "      - name: other-tenant
        when_related: {via: input.parent_id, predicate: tenant != input.tenant}
        error: demo.d.NotFound
",
    );
    let text = replaced(
        &text,
        SUBJECT_GUARD,
        "      - name: other-tenant
        when_subject: {predicate: tenant != input.tenant}
        error: demo.d.NotFound
",
    );
    text.replace("Optional<String>", "String")
}

/// [`MODEL`] with the tenant-scoped view [`CHILDREN_VIEW`] and a deleter without `when_subject:`.
fn view_parameter() -> String {
    let text = replaced(MODEL, SUBJECT_GUARD, "");
    format!("{text}{CHILDREN_VIEW}")
}

/// [`MODEL`] whose creator refuses a parent holding a tenant, and whose deleter refuses one
/// holding none: no subject holding a child passes the deleter.
fn contradicting() -> String {
    let text = replaced(
        MODEL,
        CREATOR_GUARD,
        r#"      - name: tenanted-parent
        when_related: {via: input.parent_id, predicate: "defined(tenant)"}
        error: demo.d.NotFound
"#,
    );
    replaced(
        &text,
        SUBJECT_GUARD,
        r#"      - name: other-tenant
        when_subject: {predicate: "not defined(tenant)"}
        error: demo.d.NotFound
"#,
    )
}

/// [`MODEL`] whose deleter refuses a subject on its stored field alone, comparing nothing with
/// the input.
fn blocked_subject() -> String {
    replaced(
        MODEL,
        SUBJECT_GUARD,
        r#"      - name: other-tenant
        when_subject: {predicate: tenant == "blocked"}
        error: demo.d.NotFound
"#,
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str) -> Run {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

/// Every refusal naming `wanted`, as it prints.
fn refused(run: &Run, wanted: &str) -> Vec<String> {
    run.synthesis
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
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect()
}

/// The scenario `wanted` as the suite serializes it.
fn scenario(run: &Run, wanted: &str) -> Value {
    let found = run
        .synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map(|(_, scenario)| serde_json::to_value(scenario).unwrap());
    found.unwrap_or_else(|| {
        panic!(
            "{wanted} is synthesized: {:#?}\n{:#?}",
            run.synthesis
                .suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            refused(run, wanted)
        )
    })
}

fn assert_synthesized(run: &Run, wanted: &str) {
    let refusals = refused(run, wanted);
    assert!(
        refusals.is_empty(),
        "{wanted} is not refused: {refusals:#?}"
    );
    scenario(run, wanted);
}

/// Every scenario passes on the interpreted target.
fn assert_all_pass(run: &Run) {
    let failed: Vec<String> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| {
            format!(
                "{id}: {:?} {:#?}\n{}",
                result.status,
                result.diagnostics().collect::<Vec<_>>(),
                serde_json::to_string_pretty(&scenario(run, id)).unwrap()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// A parent a `CreateParent` step of the scenario created, by the tenant that step sent (`None`
/// for absent).
#[derive(Debug, Clone)]
struct Created {
    tenant: Option<String>,
}

/// One send of a command naming a parent: the tenant it sends, and the parent it names where a
/// step of the scenario created it.
#[derive(Debug)]
struct Send {
    tenant: Option<String>,
    parent: Option<Created>,
}

/// The literal text `input.<field>` holds in a send, `None` where the send leaves it out.
fn sent_text(step: &Value, field: &str) -> Option<String> {
    let value = step.get("input")?.get(field)?;
    assert_eq!(
        value.get("kind").and_then(Value::as_str),
        Some("literal"),
        "{field} is sent as a literal: {step}"
    );
    match value.get("value")? {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => panic!("{field} is text: {other}"),
    }
}

/// Every send of `command` in `scenario`, beside the tenant its parent was created with.
fn sends(scenario: &Value, command: &str) -> Vec<Send> {
    let mut parents: BTreeMap<String, Created> = BTreeMap::new();
    let mut pending: Option<Created> = None;
    let mut out = Vec::new();
    for step in scenario["steps"].as_array().expect("steps") {
        match step.get("step").and_then(Value::as_str) {
            Some("execute_command") => {
                let sent = step["command"].as_str().expect("command");
                if sent == "demo.d.CreateParent" {
                    pending = Some(Created {
                        tenant: sent_text(step, "tenant"),
                    });
                } else {
                    pending = None;
                }
                if sent == command {
                    let parent = step
                        .get("input")
                        .and_then(|input| input.get("parent_id"))
                        .and_then(|value| value.get("instance"))
                        .and_then(Value::as_str)
                        .and_then(|instance| parents.get(instance).cloned());
                    out.push(Send {
                        tenant: sent_text(step, "tenant"),
                        parent,
                    });
                }
            }
            Some("capture_instance") if step["entity"] == "demo.d.Parent" => {
                if let Some(created) = pending.take() {
                    parents.insert(step["instance"].as_str().unwrap().to_owned(), created);
                }
            }
            _ => {}
        }
    }
    out
}

/// Every `CreateChild` step of `wanted` names a parent the scenario created and sends the tenant
/// that parent was created with, absent where it was absent.
fn assert_children_under_their_parents_tenant(run: &Run, wanted: &str) {
    let scenario = scenario(run, wanted);
    for send in sends(&scenario, "demo.d.CreateChild") {
        let Some(parent) = &send.parent else {
            panic!("{wanted}: every child names a created parent: {send:?}\n{scenario:#}");
        };
        assert_eq!(
            &send.tenant, &parent.tenant,
            "{wanted}: the child is created under its parent's tenant\n{scenario:#}"
        );
    }
}

/// [`assert_children_under_their_parents_tenant`] for every scenario not of `CreateChild` itself,
/// whose own send is the command under test.
fn assert_every_arranged_child_under_its_parents_tenant(run: &Run) {
    for id in run.synthesis.suite.scenarios.keys() {
        let id = id.to_string();
        if !id.starts_with("demo.d.CreateChild/") {
            assert_children_under_their_parents_tenant(run, &id);
        }
    }
}

#[test]
fn a_creator_guard_on_the_addressed_parent_leaves_the_readers_found_witnessed() {
    let run = run(MODEL);
    assert_synthesized(&run, FOUND);
    assert_synthesized(&run, CHILD_NOT_FOUND);
    assert_every_arranged_child_under_its_parents_tenant(&run);
    assert_all_pass(&run);
}

#[test]
fn a_required_tenant_guard_on_the_addressed_parent_is_witnessed() {
    let run = run(&required_tenant());
    assert_synthesized(&run, FOUND);
    assert_synthesized(&run, CHILD_NOT_FOUND);
    assert_every_arranged_child_under_its_parents_tenant(&run);
    assert_all_pass(&run);
}

#[test]
fn a_tenant_view_parameter_beside_the_creator_guard_keeps_child_rows_under_the_parents_tenant() {
    let run = run(&view_parameter());
    for wanted in [FOUND, DELETED] {
        assert_synthesized(&run, wanted);
        assert_children_under_their_parents_tenant(&run, wanted);
        let scenario = scenario(&run, wanted);
        for send in sends(&scenario, "demo.d.CreateChild") {
            if let Some(tenant) = &send.tenant {
                assert!(
                    !tenant.starts_with("tenant-")
                        || send.parent.as_ref().map(|parent| &parent.tenant) == Some(&send.tenant),
                    "{wanted}: no fresh tenant is sent under a parent of another: {send:?}"
                );
            }
        }
    }
    assert_all_pass(&run);
}

#[test]
fn an_affects_segment_past_a_subject_refusal_sends_an_input_its_subject_accepts() {
    // With the creator's guard, and without it: the reported deleter alone.
    for text in [MODEL.to_owned(), replaced(MODEL, CREATOR_GUARD, "")] {
        let run = run(&text);
        assert_synthesized(&run, DELETED);
        let scenario = scenario(&run, DELETED);
        let deletes = sends(&scenario, "demo.d.DeleteChild");
        assert!(!deletes.is_empty(), "{DELETED} sends the deleter");
        for send in deletes {
            let Some(parent) = &send.parent else {
                panic!("the deleter addresses a created parent: {send:?}\n{scenario:#}");
            };
            assert_eq!(
                &send.tenant, &parent.tenant,
                "the deleter is sent a tenant its subject accepts\n{scenario:#}"
            );
        }
        let result = run.results.get(DELETED).expect("run");
        assert_eq!(
            result.status,
            Status::Passed,
            "{DELETED} passes on the interpreted target: {:#?}\n{scenario:#}",
            result.diagnostics().collect::<Vec<_>>()
        );
        assert_all_pass(&run);
    }
}

#[test]
fn an_affects_segment_no_subject_takes_is_refused_by_name() {
    let run = run(&contradicting());
    let refusals: Vec<_> = run
        .synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(DELETED)
        })
        .collect();
    let [refusal] = refusals.as_slice() else {
        panic!("{DELETED} is refused once: {refusals:#?}");
    };
    assert_eq!(refusal.code().to_string(), "ESS-SYNTH-001", "{refusal:#?}");
    let message = refusal.cause.to_string();
    assert!(
        message.contains("other-tenant"),
        "the refusal names the subject refusal that answers first: {message}"
    );
    let hint = refusal.hint();
    assert!(
        hint.contains("`when_subject:`") && !hint.contains("give the field a type"),
        "the help names the subject refusal, not an input type: {hint}"
    );
    assert!(
        !run.synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == DELETED),
        "no scenario is written for {DELETED}"
    );
    assert_all_pass(&run);
}

#[test]
fn a_subject_refusal_the_first_input_misses_leaves_the_segment_as_it_was() {
    let run = run(&blocked_subject());
    assert_synthesized(&run, DELETED);
    let scenario = scenario(&run, DELETED);
    let deletes = sends(&scenario, "demo.d.DeleteChild");
    assert!(!deletes.is_empty(), "{DELETED} sends the deleter");
    for send in deletes {
        assert_eq!(
            send.tenant.as_deref(),
            Some("tenant"),
            "the deleter is sent the input its plain witness gives\n{scenario:#}"
        );
    }
    // The interpreted target reads `tenant == "blocked"` over a subject holding no tenant as
    // Unknown and reports the send unsupported, where synthesis reads the specification's own
    // unknown as not taken; that holds without this change too. No scenario fails.
    let failed: Vec<&String> = run
        .results
        .iter()
        .filter(|(_, result)| matches!(result.status, Status::Failed | Status::Error))
        .map(|(id, _)| id)
        .collect();
    assert!(failed.is_empty(), "no scenario fails: {failed:#?}");
}

/// The deleter's input, as [`MODEL`] writes it.
const DELETER_INPUT: &str = "  - name: demo.d.DeleteChild\n    input:\n";

/// The deleter's `affects:` filter, as [`MODEL`] writes it.
const DELETER_WHERE: &str = "            where: {all: [parent_id == subject.id, child_id == \
     input.child_id]}\n            deletes: demo.d.Child\n";

/// No scenario the suite holds fails on the interpreted target.
fn assert_none_fails(run: &Run) {
    let failed: Vec<String> = run
        .results
        .iter()
        .filter(|(_, result)| matches!(result.status, Status::Failed | Status::Error))
        .map(|(id, result)| {
            format!(
                "{id}: {:?} {:#?}\n{}",
                result.status,
                result.diagnostics().collect::<Vec<_>>(),
                serde_json::to_string_pretty(&scenario(run, id)).unwrap()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "no scenario fails on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// `wanted` is either synthesized and passes, or refused as ESS-SYNTH-001.
fn assert_passes_or_refused(run: &Run, wanted: &str) {
    let written = run
        .synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == wanted);
    if written {
        let result = run.results.get(wanted).expect("run");
        assert_eq!(
            result.status,
            Status::Passed,
            "{wanted} passes on the interpreted target: {:#?}\n{:#}",
            result.diagnostics().collect::<Vec<_>>(),
            scenario(run, wanted)
        );
    } else {
        let refusals = refused(run, wanted);
        assert!(
            refusals
                .iter()
                .any(|line| line.starts_with("[ESS-SYNTH-001]")),
            "{wanted} is refused as ESS-SYNTH-001: {refusals:#?}"
        );
    }
}

/// `base` whose deleter names the child by its identity (`row == input.row`), a pinned entry.
fn pinned(base: &str) -> String {
    let text = replaced(
        base,
        DELETER_INPUT,
        "  - name: demo.d.DeleteChild\n    input:\n      - {name: row, type: demo.d.ChildRow}\n",
    );
    replaced(
        &text,
        DELETER_WHERE,
        "            where: {all: [parent_id == subject.id, row == input.row]}\n            \
         deletes: demo.d.Child\n",
    )
}

/// `base` whose deleter removes the children a list names (`each:`), an entry naming its rows.
fn per_element(base: &str) -> String {
    let text = replaced(
        base,
        "  - {name: demo.d.ChildRow, kind: newtype, of: String}\n",
        "  - {name: demo.d.ChildRow, kind: newtype, of: String}\n  - name: demo.d.Pick\n    kind: \
         struct\n    fields:\n      - {name: row, type: demo.d.ChildRow}\n",
    );
    let text = replaced(
        &text,
        DELETER_INPUT,
        "  - name: demo.d.DeleteChild\n    input:\n      - {name: picks, type: \
         List<demo.d.Pick>}\n",
    );
    let text = replaced(
        &text,
        "      - name: not-found\n        unknown_instance: true\n        error: demo.d.NotFound\n      \
         - name: other-tenant\n        when_subject:",
        "      - name: repeated\n        when: {not: {distinct: {in: picks, as: p, by: p.row}}}\n        \
         error: demo.d.NotFound\n      - name: not-found\n        unknown_instance: true\n        \
         error: demo.d.NotFound\n      - name: other-tenant\n        when_subject:",
    );
    replaced(
        &text,
        DELETER_WHERE,
        "            each: {in: input.picks, as: p}\n            instance: p.row\n            \
         deletes: demo.d.Child\n",
    )
}

#[test]
fn a_pinned_deleter_past_a_subject_refusal_sends_an_input_its_subject_accepts() {
    let run = run(&pinned(MODEL));
    assert_passes_or_refused(&run, DELETED);
    assert_none_fails(&run);
}

#[test]
fn a_pinned_deleter_past_a_required_tenant_refusal_sends_an_input_its_subject_accepts() {
    let run = run(&pinned(&required_tenant()));
    assert_passes_or_refused(&run, DELETED);
    assert_none_fails(&run);
}

#[test]
fn a_per_element_deleter_past_a_subject_refusal_sends_an_input_its_subject_accepts() {
    let run = run(&per_element(MODEL));
    assert_passes_or_refused(&run, DELETED);
    assert_none_fails(&run);
}

#[test]
fn a_per_element_deleter_past_a_required_tenant_refusal_sends_an_input_its_subject_accepts() {
    let run = run(&per_element(&required_tenant()));
    assert_passes_or_refused(&run, DELETED);
    assert_none_fails(&run);
}
