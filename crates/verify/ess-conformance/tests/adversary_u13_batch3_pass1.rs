//! Adversary pass over a guard on the input-addressed parent row (batch 3, U13).
//!
//! The base model is the one `parent_row_guard` builds: `demo.d.Parent` owns `demo.d.Child`
//! through `parent_id`; `CreateChild` refuses a missing parent and a parent of another tenant;
//! `GetChild` addresses the parent and reads its child through a row set; `DeleteChild` refuses
//! another tenant with `when_subject:` and removes the child through `affects:`. Each case varies
//! one shape the unit's own tests leave out, and asserts the unit's two claims on it: a reader's
//! `found` is witnessed with every child created under its parent's tenant, and an `affects:`
//! segment past a `when_subject:` refusal either sends an input its subject passes or is refused
//! as ESS-SYNTH-001; no synthesized scenario fails on the interpreted target.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

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

const FOUND: &str = "demo.d.GetChild/outcome/found";
const DELETED: &str = "demo.d.DeleteChild/outcome/deleted";
const DELETER_INPUT: &str = "  - name: demo.d.DeleteChild\n    input:\n";
const DELETER_WHERE: &str = "            where: {all: [parent_id == subject.id, child_id == input.child_id]}\n            deletes: demo.d.Child\n";

fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

/// [`MODEL`] with a required `String` tenant everywhere and plain `tenant != input.tenant` guards.
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

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
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

fn scenario(run: &Run, wanted: &str) -> Option<Value> {
    run.synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map(|(_, scenario)| serde_json::to_value(scenario).unwrap())
}

fn synthesized(run: &Run, wanted: &str) -> Value {
    let refusals = refused(run, wanted);
    assert!(
        refusals.is_empty(),
        "{wanted} is not refused: {refusals:#?}"
    );
    scenario(run, wanted).unwrap_or_else(|| panic!("{wanted} is synthesized"))
}

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
    if let Some(scenario) = scenario(run, wanted) {
        let result = run.results.get(wanted).expect("run");
        assert_eq!(
            result.status,
            Status::Passed,
            "{wanted} passes on the interpreted target: {:#?}\n{scenario:#}",
            result.diagnostics().collect::<Vec<_>>()
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

#[derive(Debug, Clone)]
struct Created {
    tenant: Option<String>,
}

#[derive(Debug)]
struct Send {
    tenant: Option<String>,
    parent: Option<Created>,
}

fn sent_text(step: &Value, field: &str) -> Option<String> {
    let value = step.get("input")?.get(field)?;
    match value.get("value")? {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => panic!("{field} is text: {other}"),
    }
}

fn sends(scenario: &Value, command: &str) -> Vec<Send> {
    let mut parents: BTreeMap<String, Created> = BTreeMap::new();
    let mut pending: Option<Created> = None;
    let mut out = Vec::new();
    for step in scenario["steps"].as_array().expect("steps") {
        match step.get("step").and_then(Value::as_str) {
            Some("execute_command") => {
                let sent = step["command"].as_str().expect("command");
                pending = (sent == "demo.d.CreateParent").then(|| Created {
                    tenant: sent_text(step, "tenant"),
                });
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

/// Every `CreateChild` of `wanted` names a created parent and sends that parent's tenant.
fn assert_children_under_parents_tenant(run: &Run, wanted: &str) {
    let Some(scenario) = scenario(run, wanted) else {
        return;
    };
    for send in sends(&scenario, "demo.d.CreateChild") {
        let parent = send
            .parent
            .as_ref()
            .unwrap_or_else(|| panic!("{wanted}: the child names a created parent\n{scenario:#}"));
        assert_eq!(
            &send.tenant, &parent.tenant,
            "{wanted}: the child is created under its parent's tenant\n{scenario:#}"
        );
    }
}

// ---- the `affects:` segment's other witness paths --------------------------------------------

/// `base` whose deleter names the child by its identity (`row == input.row`, a pin): a pinned
/// entry returns before the unit's input choice.
fn pinned(base: &str) -> String {
    let text = replaced(
        base,
        DELETER_INPUT,
        "  - name: demo.d.DeleteChild\n    input:\n      - {name: row, type: demo.d.ChildRow}\n",
    );
    replaced(
        &text,
        DELETER_WHERE,
        "            where: {all: [parent_id == subject.id, row == input.row]}\n            deletes: demo.d.Child\n",
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

/// A reader whose row set also compares the child's tenant with its input, beside the creator's
/// guard: required tenants, so the comparison is decided.
#[test]
fn a_reader_comparing_the_childs_tenant_beside_the_creator_guard_is_witnessed() {
    let text = replaced(
        &required_tenant(),
        "          where: {all: [parent_id == input.parent_id, child_id == input.child_id]}\n",
        "          where: {all: [parent_id == input.parent_id, child_id == input.child_id, tenant == input.tenant]}\n",
    );
    let run = run(&text);
    synthesized(&run, FOUND);
    assert_children_under_parents_tenant(&run, FOUND);
    assert_none_fails(&run);
}

// ---- `when_subject:` refusals reading two fields ---------------------------------------------

/// [`required_tenant`] whose parent holds a `region` beside its tenant, set from its creator's
/// input, and whose deleter sends a region too.
fn with_region(subject_guard: &str) -> String {
    let text = replaced(
        &required_tenant(),
        "      - {name: tenant, type: String}\n    relations:",
        "      - {name: tenant, type: String}\n      - {name: region, type: String}\n    relations:",
    );
    let text = replaced(
        &text,
        "    input:\n      - {name: tenant, type: String}\n    outcomes:\n      - name: created\n        creates: demo.d.Parent\n",
        "    input:\n      - {name: tenant, type: String}\n      - {name: region, type: String}\n    outcomes:\n      - name: created\n        creates: demo.d.Parent\n",
    );
    let text = replaced(
        &text,
        "sets: {tenant: input.tenant}\n",
        "sets: {tenant: input.tenant, region: input.region}\n",
    );
    let text = replaced(
        &text,
        DELETER_INPUT,
        "  - name: demo.d.DeleteChild\n    input:\n      - {name: region, type: String}\n",
    );
    replaced(
        &text,
        "      - name: other-tenant\n        when_subject: {predicate: tenant != input.tenant}\n        error: demo.d.NotFound\n",
        subject_guard,
    )
}

#[test]
fn a_subject_refusal_reading_tenant_and_region_is_passed() {
    let run = run(&with_region(
        "      - name: other-tenant\n        when_subject: {predicate: {any: [tenant != input.tenant, region != input.region]}}\n        error: demo.d.NotFound\n",
    ));
    synthesized(&run, DELETED);
    assert_passes_or_refused(&run, DELETED);
    assert_none_fails(&run);
}

#[test]
fn two_subject_refusals_on_tenant_and_region_are_both_passed() {
    let run = run(&with_region(
        "      - name: other-tenant\n        when_subject: {predicate: tenant != input.tenant}\n        error: demo.d.NotFound\n      - name: other-region\n        when_subject: {predicate: region != input.region}\n        error: demo.d.NotFound\n",
    ));
    synthesized(&run, DELETED);
    assert_passes_or_refused(&run, DELETED);
    assert_none_fails(&run);
}

/// The deleter's `affects:` filter reads the input field the subject refusal reads: the rows are
/// arranged after the input is chosen again, so they are selected by the input sent.
#[test]
fn an_affects_filter_reading_the_refused_input_selects_the_rows_arranged() {
    let text = replaced(
        &required_tenant(),
        "      - name: other-tenant\n        when_related: {via: input.parent_id, predicate: tenant != input.tenant}\n        error: demo.d.NotFound\n",
        "",
    );
    let text = replaced(
        &text,
        DELETER_WHERE,
        "            where: {all: [parent_id == subject.id, child_id == input.child_id, tenant == input.tenant]}\n            deletes: demo.d.Child\n",
    );
    let run = run(&text);
    let scenario = synthesized(&run, DELETED);
    assert_passes_or_refused(&run, DELETED);
    for send in sends(&scenario, "demo.d.DeleteChild") {
        let parent = send
            .parent
            .as_ref()
            .expect("the deleter names a created parent");
        assert_eq!(&send.tenant, &parent.tenant, "{scenario:#}");
    }
    assert_none_fails(&run);
}

// ---- tenant types and views ------------------------------------------------------------------

#[test]
fn a_required_tenant_view_parameter_beside_both_guards_keeps_children_under_the_parent() {
    let text = format!(
        "{}{}",
        required_tenant(),
        "  - name: demo.d.Children
    source: demo.d.Child
    consistency: read_your_writes
    params:
      - {name: parent_id, type: demo.d.ParentId}
      - {name: tenant, type: String}
    filter: {all: [parent_id == param.parent_id, tenant == param.tenant]}
    fields:
      - {name: child_id, type: demo.d.ChildId}
      - {name: parent_id, type: demo.d.ParentId}
      - {name: tenant, type: String}
"
    );
    let run = run(&text);
    for wanted in [FOUND, DELETED] {
        synthesized(&run, wanted);
        assert_children_under_parents_tenant(&run, wanted);
    }
    assert_none_fails(&run);
}

/// A required parent tenant beside `Optional` child and input tenants.
#[test]
fn a_required_parent_tenant_beside_optional_inputs_is_witnessed() {
    let text = replaced(
        MODEL,
        "      - {name: tenant, type: Optional<String>}\n    relations:",
        "      - {name: tenant, type: String}\n    relations:",
    );
    let text = replaced(
        &text,
        "    input:\n      - {name: tenant, type: Optional<String>}\n    outcomes:\n      - name: created\n        creates: demo.d.Parent\n",
        "    input:\n      - {name: tenant, type: String}\n    outcomes:\n      - name: created\n        creates: demo.d.Parent\n",
    );
    let text = replaced(
        &text,
        "      - {name: state, type: demo.d.Parent.State}\n      - {name: tenant, type: Optional<String>}\n",
        "      - {name: state, type: demo.d.Parent.State}\n      - {name: tenant, type: String}\n",
    );
    let run = run(&text);
    synthesized(&run, FOUND);
    assert_children_under_parents_tenant(&run, FOUND);
    assert_passes_or_refused(&run, DELETED);
    assert_children_under_parents_tenant(&run, DELETED);
    assert_none_fails(&run);
}

// ---- readers ---------------------------------------------------------------------------------

/// A reader of the parent refusing a crowded parent (`count: {gte: 2}`): its error branch is
/// witnessed with two children, each under the parent's tenant.
#[test]
fn a_count_refusal_over_children_of_a_guarded_parent_is_witnessed() {
    let text = replaced(
        MODEL,
        "errors:\n  - name: demo.d.NotFound\n",
        "  - name: demo.d.CountChildren
    input:
      - {name: tenant, type: Optional<String>}
      - {name: parent_id, type: demo.d.ParentId}
    outcomes:
      - name: not-found
        unknown_instance: true
        error: demo.d.NotFound
      - name: crowded
        when_related:
          entity: demo.d.Child
          where: parent_id == input.parent_id
          count: {gte: 2}
        error: demo.d.Crowded
      - name: roomy
        preserves: demo.d.Parent
        instance: parent_id
errors:
  - name: demo.d.NotFound
  - name: demo.d.Crowded
",
    );
    let run = run(&text);
    for wanted in [
        "demo.d.CountChildren/outcome/crowded",
        "demo.d.CountChildren/outcome/roomy",
    ] {
        synthesized(&run, wanted);
        assert_children_under_parents_tenant(&run, wanted);
    }
    assert_none_fails(&run);
}

// ---- determinism -----------------------------------------------------------------------------

#[test]
fn synthesis_is_the_same_bytes_twice() {
    for text in [MODEL.to_owned(), required_tenant()] {
        let model = ir(&text);
        let first = ess_conformance::synthesize::synthesize(&model);
        let second = ess_conformance::synthesize::synthesize(&ir(&text));
        assert_eq!(
            serde_json::to_string(&first.suite).unwrap(),
            serde_json::to_string(&second.suite).unwrap()
        );
        let refusals = |synthesis: &Synthesis| -> Vec<String> {
            synthesis
                .refusals
                .iter()
                .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
                .collect()
        };
        assert_eq!(refusals(&first), refusals(&second));
    }
}
