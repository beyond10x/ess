//! What `ess/14` value expressions refuse (`docs/design/value-expressions.md`).

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("orders.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// One entity and two commands; `{open}` and `{touch}` are the extra lines of the creating and the
/// updating outcome, indented as entries of the outcome.
fn model(format: &str, open: &str, touch: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.orders
types:
  - {{name: demo.orders.OrderId, kind: newtype, of: String}}
  - name: demo.orders.Ref
    kind: struct
    fields:
      - {{name: id, type: demo.orders.OrderId}}
      - {{name: label, type: String}}
entities:
  - name: demo.orders.Order
    identity: {{name: order_id, type: demo.orders.OrderId}}
    fields:
      - {{name: note, type: String}}
      - {{name: retries, type: Integer}}
      - {{name: maybe, type: Optional<Integer>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.orders.Opened
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
  - name: demo.orders.Touched
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: note, type: String}}
      - {{name: ref, type: demo.orders.Ref}}
actors:
  - {{name: demo.orders.Clerk, may: [demo.orders.Open, demo.orders.Touch]}}
commands:
  - name: demo.orders.Open
    input:
      - {{name: note, type: String}}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened: {{order_id: {{generated: true}}}}
        sets:
          note: input.note
          retries: 0
{open}
  - name: demo.orders.Touch
    input:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: label, type: String}}
      - {{name: extra, type: Optional<String>}}
    outcomes:
      - name: touched
        updates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Touched]
{touch}
"
    )
}

/// The updating outcome's payload and sets, with `note`, `ref` and one `sets:` entry replaced.
fn touch(note: &str, reference: &str, sets: &str) -> String {
    format!(
        "        payload:
          demo.orders.Touched:
            order_id: input.order_id
            note: {note}
            ref: {reference}
        sets:
          {sets}"
    )
}

const REF: &str = "{id: input.order_id, label: input.label}";

fn refused(body: &str, expected: &[&str]) {
    let error = spec(body)
        .err()
        .unwrap_or_else(|| panic!("must not compile:\n{body}"));
    for needle in expected {
        assert!(error.contains(needle), "expected {needle:?} in:\n{error}");
    }
}

#[test]
fn every_value_expression_compiles_under_ess_14() {
    let body = model(
        "ess/14",
        "",
        &touch("{subject: note}", REF, "retries: {increment: 1}"),
    );
    spec(&body).unwrap_or_else(|error| panic!("{error}"));
    let body = model(
        "ess/14",
        "",
        &touch(
            "{input: extra, else: {generated: true}}",
            REF,
            "note: {generated: true}",
        ),
    );
    spec(&body).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn a_value_expression_needs_ess_14() {
    let body = model(
        "ess/13",
        "",
        &touch("{subject: note}", REF, "note: input.label"),
    );
    refused(&body, &["unsupported_format_version", "ess/14"]);
    let body = model(
        "ess/13",
        "",
        &touch("input.label", REF, "note: input.label"),
    );
    refused(&body, &["unsupported_format_version", "nested mapping"]);
}

#[test]
fn a_subject_path_written_as_text_is_refused_rather_than_compiled_as_text() {
    // beyond10x/ess#133: `subject.note` compiled to the literal text "subject.note".
    for format in ["ess/13", "ess/14"] {
        let body = model(
            format,
            "",
            &touch("subject.note", "{generated: true}", "note: input.label"),
        );
        refused(&body, &["misspelled_reference", "{subject: note}"]);
    }
}

#[test]
fn a_subject_source_needs_an_existing_subject_and_a_field_it_holds() {
    let body = model(
        "ess/14",
        "          maybe: {subject: retries}",
        &touch("input.label", REF, "note: input.label"),
    );
    refused(&body, &["conflicting_declaration", "no row before it"]);
    let body = model(
        "ess/14",
        "",
        &touch("{subject: nope}", REF, "note: input.label"),
    );
    refused(&body, &["undeclared_reference", "`nope` is not a field"]);
    let body = model(
        "ess/14",
        "",
        &touch("{subject: retries}", REF, "note: input.label"),
    );
    refused(&body, &["type_mismatch"]);
}

#[test]
fn an_increment_is_a_sets_source_over_a_required_number() {
    let body = model(
        "ess/14",
        "",
        &touch("input.label", REF, "note: {increment: 1}"),
    );
    refused(
        &body,
        &["type_mismatch", "only an `Integer` or a `Decimal`"],
    );
    let body = model(
        "ess/14",
        "",
        &touch("input.label", REF, "maybe: {increment: 1}"),
    );
    refused(&body, &["type_mismatch", "nothing to add to"]);
    let body = model(
        "ess/14",
        "",
        &touch("input.label", REF, "retries: {increment: 0}"),
    );
    refused(&body, &["type_mismatch", "non-zero"]);
    let body = model(
        "ess/14",
        "",
        &touch("input.label", REF, "retries: {increment: '0.5'}"),
    );
    refused(&body, &["type_mismatch", "non-zero"]);
    let body = model(
        "ess/14",
        "          maybe: {increment: 1}",
        &touch("input.label", REF, "note: input.label"),
    );
    refused(&body, &["no row before it"]);
}

#[test]
fn a_fallback_reads_an_optional_input() {
    let body = model(
        "ess/14",
        "",
        &touch(
            "{input: label, else: {generated: true}}",
            REF,
            "note: input.label",
        ),
    );
    refused(&body, &["type_mismatch", "always sent"]);
    let body = model(
        "ess/14",
        "",
        &touch("{input: extra}", REF, "note: input.label"),
    );
    refused(&body, &["needs `else: {generated: true}`"]);
    let body = model(
        "ess/14",
        "",
        &touch(
            "{input: extra, else: input.label}",
            REF,
            "note: input.label",
        ),
    );
    refused(&body, &["admits `{generated: true}` only"]);
}

#[test]
fn a_nested_mapping_fills_every_field_of_a_struct_and_nothing_else() {
    let body = model(
        "ess/14",
        "",
        &touch("input.label", "{id: input.order_id}", "note: input.label"),
    );
    refused(&body, &["empty_declaration", "`label`"]);
    let body = model(
        "ess/14",
        "",
        &touch(
            "input.label",
            "{id: input.order_id, label: input.label, x: '1'}",
            "note: input.label",
        ),
    );
    refused(&body, &["undeclared_reference", "`x` is not a field"]);
    let body = model(
        "ess/14",
        "",
        &touch("{id: input.order_id}", REF, "note: input.label"),
    );
    refused(&body, &["type_mismatch", "a nested mapping fills a struct"]);
    let body = model(
        "ess/14",
        "",
        &touch(
            "input.label",
            "{id: input.order_id, label: input.nope}",
            "note: input.label",
        ),
    );
    refused(&body, &["undeclared_reference", "no declared source field"]);
}
