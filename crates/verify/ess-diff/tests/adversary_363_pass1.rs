//! Adversary, beyond10x/ess#363 pass 1: the aggregate comparison in `compare_aggregations` moved
//! from the rendered text to the whole `ResolvedAggregate`, whose `input` is a `ResolvedField`
//! carrying the source field's type and `Naming`. An unconditioned model must diff as it did.
//!
//! The models write no `where:`, so this file reads the same on the unit's base.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r"format: ess/22
system: demo
version: v1
domain: demo.cases
events:
  - name: demo.cases.Opened
    fields: [{name: case_id, type: Uuid}]
  - name: demo.cases.Closed
    fields: [{name: case_id, type: Uuid}]
entities:
  - name: demo.cases.Case
    identity: {name: case_id, type: Uuid}
    fields:
      - {name: team, type: String}
      - {name: cents, type: Integer}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions: [{name: close, from: [Open], to: Closed}]
commands:
  - name: demo.cases.Open
    input:
      - {name: team, type: String}
      - {name: cents, type: Integer}
    outcomes:
      - name: opened
        creates: demo.cases.Case
        instance: case_id
        sets: {team: input.team, cents: input.cents}
        emits: [demo.cases.Opened]
        payload: {demo.cases.Opened: {case_id: {generated: true}}}
  - name: demo.cases.Close
    input: [{name: case_id, type: Uuid}]
    outcomes:
      - name: closed
        moves: demo.cases.Case.close
        instance: case_id
        emits: [demo.cases.Closed]
        payload: {demo.cases.Closed: {case_id: input.case_id}}
views:
  - name: demo.cases.Totals
    source: demo.cases.Case
    consistency: read_your_writes
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: cost, type: Integer, aggregate: {sum: cents}}
";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("cases.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every `kind` anywhere in the delta whose `kind` names a view field's aggregate.
fn aggregate_changes(before: &str, after: &str) -> Vec<String> {
    let delta = diff(&ir(before), &ir(after)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    let mut out = Vec::new();
    collect(&json, &mut out);
    out
}

fn collect(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str)
                == Some("field-aggregate-changed")
            {
                out.push(serde_json::Value::Object(map.clone()).to_string());
            }
            for inner in map.values() {
                collect(inner, out);
            }
        }
        serde_json::Value::Array(items) => {
            for inner in items {
                collect(inner, out);
            }
        }
        _ => {}
    }
}

/// `Naming` says a display change "is free". Renaming how the source field `cents` is shown
/// changes no view's computation: `sum(cents)` before and after.
#[test]
fn adversary_a_display_rename_of_an_aggregated_source_field_is_no_aggregate_change() {
    let renamed = MODEL.replace(
        "      - {name: cents, type: Integer}\n    lifecycle:",
        "      - {name: cents, type: Integer, display: Amount}\n    lifecycle:",
    );
    assert_ne!(renamed, MODEL, "the rename applies");
    assert_eq!(
        aggregate_changes(MODEL, &renamed),
        Vec::<String>::new(),
        "an unconditioned `sum(cents)` reports a behavioural aggregate change for a display rename"
    );
}

/// The same for a summary, which only generated documentation reads.
#[test]
fn adversary_a_summary_of_an_aggregated_source_field_is_no_aggregate_change() {
    let summarized = MODEL.replace(
        "      - {name: cents, type: Integer}\n    lifecycle:",
        "      - {name: cents, type: Integer, summary: What the case cost.}\n    lifecycle:",
    );
    assert_ne!(summarized, MODEL, "the summary applies");
    assert_eq!(
        aggregate_changes(MODEL, &summarized),
        Vec::<String>::new(),
        "an unconditioned `sum(cents)` reports a behavioural aggregate change for a new summary"
    );
}
