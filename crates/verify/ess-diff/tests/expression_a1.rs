//! A literal that becomes a fact is a behaviour change, and the diff says so in what it renders
//! (`docs/design/expression-family-source22.md`, A1, final review decision 7).
//!
//! Under `ess/22` a bare word on the right of a comparison names a root of the place it is written
//! in. So declaring a field moves `note == draft` from the text `"draft"` to the field `draft`
//! without a byte of the guard's source changing. A rendering of the source spelling would report
//! that change with identical before and after; the condition is rendered canonically instead.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(extra_input: &str, extra_field: &str, extra_sets: &str) -> String {
    format!(
        r"format: ess/22
system: graph
version: v1
domain: graph.tasks
entities:
  - name: graph.tasks.Edge
    identity: {{name: edge_id, type: Uuid}}
    fields:
      - {{name: status, type: String}}
{extra_field}    invariants:
      - status != pending
    lifecycle: {{initial: Linked, states: [Linked], terminal: [Linked]}}
errors:
  - name: graph.tasks.Refused
    summary: The link is refused.
events:
  - name: graph.tasks.Linked
    fields:
      - {{name: edge_id, type: Uuid}}
commands:
  - name: graph.tasks.Link
    input:
      - {{name: note, type: String}}
      - {{name: status, type: String}}
{extra_input}    outcomes:
      - name: refused
        when: note == draft
        error: graph.tasks.Refused
      - name: linked
        creates: graph.tasks.Edge
        instance: edge_id
        emits: [graph.tasks.Linked]
        payload:
          graph.tasks.Linked: {{edge_id: {{generated: true}}}}
        sets: {{status: input.status{extra_sets}}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("graph.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn diff_literal_to_fact_is_behaviour() {
    let before = ir(&model("", "", ""));
    let after = ir(&model(
        "      - {name: draft, type: String}\n",
        "      - {name: pending, type: String}\n",
        ", pending: input.note",
    ));
    let delta = diff(&before, &after).unwrap();
    let json = delta.to_canonical_json();
    assert!(json.contains("outcome-condition-changed"), "{json}");
    assert!(
        json.contains(r#""before": "when note == draft""#)
            && json.contains(r#""after": "when note == {fact: draft}""#),
        "the guard that now reads a field renders differently from the text it read: {json}"
    );
    assert!(json.contains("invariants-changed"), "{json}");
    assert!(
        json.contains("status != pending") && json.contains("status != {fact: pending}"),
        "the invariant that now reads a field renders differently from the text it read: {json}"
    );
}

#[test]
fn an_unchanged_literal_is_no_change() {
    let before = ir(&model("", "", ""));
    let delta = diff(&before, &ir(&model("", "", ""))).unwrap();
    assert!(
        !delta.to_canonical_json().contains("condition-changed"),
        "{}",
        delta.to_canonical_json()
    );
}
