//! A `fixture_inputs:` binding is refused over every input a guard reads
//! (story:a-fixture-may-be-a-row-the-deployment-already-holds, `docs/design/pre-execution-fixture-values.md`).
//!
//! A fixture value chosen by a target's provider would decide which branch the guard takes, so it
//! cannot stand in for the witness synthesis generates. Below this story the refusal read only an
//! input guard's own paths, and a `when_related:` guard naming its row through the input
//! (`via: input.<field>`) validated: synthesis then sent the fixture to the `exists: false` branch.
//! Each command of [`MODEL`] reads `label` or an identity input through one kind of guard; the
//! binding is refused over the field it reads and admitted over one it does not.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = r#"
format: ess/32
system: linking
version: v1
domain: linking.links
types:
  - {name: linking.links.AppId, kind: newtype, of: Uuid}
  - {name: linking.links.LinkId, kind: newtype, of: Uuid}
entities:
  - name: linking.links.App
    identity: {name: app_id, type: linking.links.AppId}
    fields: [{name: title, type: String}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: linking.links.Link
    identity: {name: link_id, type: linking.links.LinkId}
    fields:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    relations:
      - {name: app, kind: references, target: linking.links.App, cardinality: one, via: app_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - name: linking.links.Refused
events:
  - name: linking.links.AppCreated
    fields: [{name: app_id, type: linking.links.AppId}]
  - name: linking.links.Relabelled
    fields: [{name: link_id, type: linking.links.LinkId}]
  - name: linking.links.Linked
    fields: [{name: link_id, type: linking.links.LinkId}]
commands:
  - name: linking.links.CreateApp
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: title, type: String}
    outcomes:
      - name: created
        creates: linking.links.App
        instance: app_id
        sets: {title: input.title}
        emits: [linking.links.AppCreated]
        payload:
          linking.links.AppCreated: {app_id: input.app_id}
  - name: linking.links.ByVia
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: unknown-app
        when_related: {via: input.app_id, exists: false}
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
  - name: linking.links.ByRowPredicate
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: unknown-app
        when_related: {via: input.app_id, exists: false}
        error: linking.links.Refused
      - name: mismatched
        when_related: {via: input.app_id, predicate: "title != input.label"}
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
  - name: linking.links.ByAll
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: other_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: unknown-app
        when_related: {via: input.app_id, exists: false}
        error: linking.links.Refused
      - name: unknown-other
        when_related: {via: input.other_id, exists: false}
        error: linking.links.Refused
      - name: both-titled
        when_related:
          all:
            - {via: input.app_id, predicate: "title == input.label"}
            - {via: input.other_id, predicate: "title == input.label"}
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
  - name: linking.links.BySelector
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: taken
        when_related: {entity: linking.links.Link, where: "label == input.label", exists: true}
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
  - name: linking.links.ByForall
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: unlabelled
        when_related: {entity: linking.links.Link, where: "state == Open", forall: "label != input.label"}
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
  - name: linking.links.BySubject
    input:
      - {name: link_id, type: linking.links.LinkId}
      - {name: label, type: String}
    outcomes:
      - name: same-label
        when_subject: {predicate: "label == input.label"}
        error: linking.links.Refused
      - name: relabelled
        updates: linking.links.Link
        instance: link_id
        sets: {label: input.label}
        emits: [linking.links.Relabelled]
        payload:
          linking.links.Relabelled: {link_id: input.link_id}
  - name: linking.links.ByInputNamespace
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: blank
        when: input.label == ""
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
  - name: linking.links.ByRoot
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: label, type: String}
    outcomes:
      - name: blank
        when: label == ""
        error: linking.links.Refused
      - name: linked
        creates: linking.links.Link
        instance: link_id
        sets: {app_id: input.app_id, label: input.label}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}}
"#;

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("links.yaml"), raw)])
}

/// `MODEL` with `fixture_inputs: {<field>: provided-value}` on `command`.
fn bound(command: &str, field: &str) -> String {
    let header = format!("  - name: linking.links.{command}\n");
    assert!(MODEL.contains(&header), "{command} is declared");
    MODEL.replace(
        &header,
        &format!("{header}    fixture_inputs: {{{field}: provided-value}}\n"),
    )
}

/// The refusal of a binding over `field` on `command`, at the binding, as `unsupported_construct`.
fn refused_at(command: &str, field: &str) {
    let body = bound(command, field);
    let errors = assemble(&body)
        .err()
        .unwrap_or_else(|| panic!("{command}: a fixture over `{field}` is refused"));
    let location = format!("command.linking.links.{command}.fixture_inputs.{field}");
    assert!(
        errors.as_slice().iter().any(|error| {
            error.location == location
                && error.code == ValidationCode::UnsupportedConstruct
                && error
                    .message
                    .contains("cannot replace a field read by an outcome predicate")
        }),
        "{command}: refused at `{location}`: {errors}"
    );
}

#[test]
fn the_model_without_a_binding_validates() {
    assemble(MODEL).unwrap_or_else(|errors| panic!("the model validates: {errors}"));
}

#[test]
fn a_binding_over_the_identity_a_related_guard_reads_is_refused() {
    refused_at("ByVia", "app_id");
}

#[test]
fn a_binding_over_an_identity_an_all_element_reads_is_refused() {
    refused_at("ByAll", "other_id");
    refused_at("ByAll", "app_id");
}

#[test]
fn a_binding_over_an_input_a_related_row_predicate_reads_is_refused() {
    refused_at("ByRowPredicate", "label");
    refused_at("ByAll", "label");
}

#[test]
fn a_binding_over_an_input_a_row_set_reads_is_refused() {
    refused_at("BySelector", "label");
    refused_at("ByForall", "label");
}

#[test]
fn a_binding_over_an_input_a_subject_predicate_reads_is_refused() {
    refused_at("BySubject", "label");
}

#[test]
fn a_binding_over_an_input_an_input_guard_reads_stays_refused() {
    refused_at("ByRoot", "label");
    refused_at("ByInputNamespace", "label");
}

#[test]
fn a_binding_over_an_input_no_guard_reads_is_admitted() {
    for (command, field) in [
        ("ByVia", "label"),
        ("BySelector", "app_id"),
        ("ByForall", "app_id"),
        ("ByRoot", "app_id"),
    ] {
        let body = bound(command, field);
        assemble(&body).unwrap_or_else(|errors| {
            panic!(
                "{command}: a fixture over `{field}`, which no guard reads, is admitted: {errors}"
            )
        });
    }
}
