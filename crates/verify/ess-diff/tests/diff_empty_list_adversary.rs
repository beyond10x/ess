//! Adversary pass on `sets: {field: []}` (ess/31) in `ess-diff`: a creator that starts storing the
//! empty list where it left the field unset, or where it read the list from its input, changes
//! what every created row holds. The delta must say so, classified, and name the field.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, render};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// A job with a required `tags` list; `AddJob` writes `sets`, and reads `tags` from its input when
/// `input` is set.
fn model(format: &str, sets: &str, input: bool) -> String {
    let tags = if input {
        ", {name: tags, type: List<String>}"
    } else {
        ""
    };
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: label, type: String}}
      - {{name: tags, type: List<String>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.AddJob
    input: [{{name: label, type: String}}{tags}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {{{sets}}}
        emits: [demo.jobs.JobAdded]
        payload: {{demo.jobs.JobAdded: {{job_id: {{generated: true}}}}}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn assert_named(before: &str, after: &str, label: &str) {
    let delta = diff(&ir(before), &ir(after)).unwrap();
    let json = delta.to_canonical_json();
    assert!(
        !delta.changes().is_empty(),
        "{label}: the creator now stores `[]` in `tags`, and the delta is empty:\n{json}"
    );
    assert!(!json.contains("unclassified"), "{label}: {json}");
    assert!(
        json.contains("tags") && json.contains("[]"),
        "{label}: the delta's before/after names `tags` and `[]`:\n{json}"
    );
    let text = render::text(&delta);
    assert!(
        text.contains("outcome-sets-changed/added"),
        "{label}: the rendered delta names the outcome:\n{text}"
    );
}

#[test]
fn adversary_absent_to_empty_list_is_a_named_change() {
    assert_named(
        &model("ess/31", "label: input.label", false),
        &model("ess/31", "label: input.label, tags: []", false),
        "absent -> []",
    );
}

#[test]
fn adversary_absent_at_ess_30_to_empty_list_at_ess_31_is_a_named_change() {
    assert_named(
        &model("ess/30", "label: input.label", false),
        &model("ess/31", "label: input.label, tags: []", false),
        "ess/30 absent -> ess/31 []",
    );
}

#[test]
fn adversary_input_to_empty_list_is_a_named_change() {
    assert_named(
        &model("ess/31", "label: input.label, tags: input.tags", true),
        &model("ess/31", "label: input.label, tags: []", true),
        "input.tags -> []",
    );
}

#[test]
fn adversary_empty_list_to_absent_is_a_named_change() {
    assert_named(
        &model("ess/31", "label: input.label, tags: []", false),
        &model("ess/31", "label: input.label", false),
        "[] -> absent",
    );
}
