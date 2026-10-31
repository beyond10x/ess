//! Adversary pass 1 over wave D325 (`ess/33`): structured input examples, rows a deployment
//! provides, and lifecycle summaries.
//!
//! Each case states the contract it holds the implementation to; a red case is a finding.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(Source::new("links.yaml"), raw)])
}

/// A model at `format` whose command `Relabel` reads a row by `via: input.app_id` (or, with
/// `subject`, its own subject) through a predicate over the row's stored `label`, and takes an input
/// that is also called `label`. The guard reads the row's `label`, never the input's.
fn row_label_model(format: u32, guard: &str, fixture: &str) -> String {
    format!(
        r"format: ess/{format}
system: linking
version: v1
domain: linking.links
types:
  - {{name: linking.links.AppId, kind: newtype, of: Uuid}}
  - {{name: linking.links.LinkId, kind: newtype, of: Uuid}}
entities:
  - name: linking.links.App
    identity: {{name: app_id, type: linking.links.AppId}}
    fields: [{{name: label, type: String}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: linking.links.Link
    identity: {{name: link_id, type: linking.links.LinkId}}
    fields:
      - {{name: app_id, type: linking.links.AppId}}
      - {{name: label, type: String}}
    relations:
      - {{name: app, kind: references, target: linking.links.App, cardinality: one, via: app_id}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - name: linking.links.Refused
events:
  - name: linking.links.AppCreated
    fields: [{{name: app_id, type: linking.links.AppId}}]
  - name: linking.links.Linked
    fields: [{{name: link_id, type: linking.links.LinkId}}]
  - name: linking.links.Relabelled
    fields: [{{name: link_id, type: linking.links.LinkId}}]
commands:
  - name: linking.links.CreateApp
    input:
      - {{name: app_id, type: linking.links.AppId}}
      - {{name: label, type: String}}
    outcomes:
      - name: created
        creates: linking.links.App
        instance: app_id
        sets: {{label: input.label}}
        emits: [linking.links.AppCreated]
        payload:
          linking.links.AppCreated: {{app_id: input.app_id}}
  - name: linking.links.CreateLink
{fixture}    input:
      - {{name: app_id, type: linking.links.AppId}}
      - {{name: label, type: String}}
    outcomes:
{guard}      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {{app_id: input.app_id, label: input.label}}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {{link_id: {{generated: true}}}}
"
    )
}

const ROW_PREDICATE: &str = r#"      - name: unknown-app
        when_related: {via: input.app_id, exists: false}
        error: linking.links.Refused
      - name: archived-app
        when_related: {via: input.app_id, predicate: 'label == "archived"'}
        error: linking.links.Refused
"#;

const ROW_SET: &str = r#"      - name: archived-links
        when_related: {entity: linking.links.Link, where: 'label == "archived"', exists: true}
        error: linking.links.Refused
"#;

/// A `fixture_inputs:` binding over an input no guard reads stays admitted. The row predicate
/// `label == "archived"` reads the related row's stored `label` as its root, not the command's
/// `label` input; the binding over that input chose no branch before the wave and chooses none now.
/// The wave widened the refusal to row predicates and applied it at every format, so the
/// root-namespace match it kept for an input guard now also matches a row's own field.
#[test]
fn a_binding_over_an_input_that_shares_a_name_with_a_row_field_a_guard_reads_is_admitted() {
    let mut wrongly_refused = Vec::new();
    for format in [32, 33] {
        for (what, guard) in [("row predicate", ROW_PREDICATE), ("row set", ROW_SET)] {
            let body = row_label_model(
                format,
                guard,
                "    fixture_inputs: {label: provided-label}\n",
            );
            assemble(&row_label_model(format, guard, ""))
                .unwrap_or_else(|errors| panic!("ess/{format} {what}: the bare model: {errors}"));
            if let Err(errors) = assemble(&body) {
                let refused: Vec<_> = errors
                    .as_slice()
                    .iter()
                    .filter(|error| error.code == ValidationCode::UnsupportedConstruct)
                    .map(|error| format!("{} {}", error.location, error.message))
                    .collect();
                wrongly_refused.push(format!("ess/{format} {what}: {refused:?}"));
            }
        }
    }
    assert!(
        wrongly_refused.is_empty(),
        "no guard reads the input `label` (each guard reads a row's stored `label`), so the \
         binding is admitted; refused:\n{}",
        wrongly_refused.join("\n")
    );
}

/// A struct example that breaks one of the struct's own invariants is refused at `ess/33`, as a
/// precondition literal is.
#[test]
fn a_struct_example_that_breaks_the_structs_invariant_is_refused() {
    let body = r"format: ess/33
system: demo
version: v1
domain: demo.w
types:
  - name: demo.w.Window
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
    invariants: [low < high]
entities:
  - name: demo.w.Slot
    identity: {name: slot_id, type: Uuid}
    fields: [{name: low, type: Integer}]
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.w.Opened
    fields: [{name: slot_id, type: Uuid}]
commands:
  - name: demo.w.Open
    input:
      - {name: window, type: demo.w.Window, example: {low: 9, high: 1}}
    outcomes:
      - name: opened
        creates: demo.w.Slot
        instance: slot_id
        sets: {low: input.window.low}
        emits: [demo.w.Opened]
        payload:
          demo.w.Opened: {slot_id: {generated: true}}
";
    let errors = assemble(body).expect_err("an example breaking `low < high` is refused");
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.location.contains("window")),
        "{errors}"
    );
}

/// A closed struct nested in an open struct's example keeps refusing its own undeclared members.
#[test]
fn a_closed_struct_inside_an_open_example_still_refuses_undeclared_members() {
    let model = |example: &str| {
        format!(
            r"format: ess/33
system: demo
version: v1
domain: demo.d
types:
  - name: demo.d.Inner
    kind: struct
    fields: [{{name: id, type: String}}]
  - name: demo.d.Body
    kind: struct
    undeclared_fields: ignored
    fields: [{{name: inner, type: demo.d.Inner}}]
entities:
  - name: demo.d.Doc
    identity: {{name: doc_id, type: Uuid}}
    fields: [{{name: name, type: String}}]
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.d.Made
    fields: [{{name: doc_id, type: Uuid}}]
commands:
  - name: demo.d.Make
    input:
      - {{name: body, type: demo.d.Body, example: {example}}}
    outcomes:
      - name: made
        creates: demo.d.Doc
        instance: doc_id
        sets: {{name: input.body.inner.id}}
        emits: [demo.d.Made]
        payload:
          demo.d.Made: {{doc_id: {{generated: true}}}}
"
        )
    };
    assemble(&model("{inner: {id: a}, extra: 1}"))
        .unwrap_or_else(|errors| panic!("the open outer struct keeps `extra`: {errors}"));
    let errors = assemble(&model("{inner: {id: a, stray: 1}}"))
        .expect_err("the closed inner struct refuses `stray`");
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::UndeclaredReference),
        "{errors}"
    );
}
