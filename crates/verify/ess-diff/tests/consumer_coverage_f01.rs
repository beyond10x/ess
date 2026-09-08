//! Isolated F01 comparison witnesses for relation aspects and view parameter contracts.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::change::{
    EntityChange, ParameterContract, RelationContract, SemanticChange, ViewChange,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const SYSTEM: &str = "format: ess/1\nsystem: coverage\nversion: v1\ndomains: [coverage.model]\n";
const DOMAIN: &str = r"
domain: coverage.model
types:
  - name: coverage.model.Id
    kind: newtype
    of: Uuid
entities:
  - name: coverage.model.Parent
    identity: {name: id, type: coverage.model.Id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
    fields:
      - {name: link_id, type: coverage.model.Id}
      - {name: alternate_id, type: coverage.model.Id}
    relations:
      - name: child
        kind: owns
        target: coverage.model.First
        cardinality: one
        via: link_id
  - name: coverage.model.First
    identity: {name: id, type: coverage.model.Id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
    fields:
      - {name: link_id, type: coverage.model.Id}
      - {name: alternate_id, type: coverage.model.Id}
  - name: coverage.model.Second
    identity: {name: id, type: coverage.model.Id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
    fields:
      - {name: link_id, type: coverage.model.Id}
      - {name: alternate_id, type: coverage.model.Id}
views:
  - name: coverage.model.Parents
    source: coverage.model.Parent
    filter:
      all: [param.first == param.first, param.second == param.second]
    params:
      - {name: first, type: String}
      - {name: second, type: String}
    fields:
      - {name: id, type: coverage.model.Id}
";

fn compiled(domain: &str) -> EssIr {
    let files: Vec<_> = [("system.yaml", SYSTEM), ("model.yaml", domain)]
        .into_iter()
        .map(|(name, text)| {
            (
                Source::new(name),
                RawSpecFile::parse(text).expect("valid authored input"),
            )
        })
        .collect();
    let spec = Specification::assemble(files).expect("the isolated fixture validates");
    compile(&spec, &SourceMap::new()).expect("the isolated fixture resolves")
}

fn one_change(needle: &str, replacement: &str) -> SemanticChange {
    assert_eq!(DOMAIN.matches(needle).count(), 1, "one authored edit");
    assert_ne!(needle, replacement);
    let before = compiled(DOMAIN);
    let control = compiled(DOMAIN);
    assert!(ess_diff::diff(&before, &control)
        .unwrap()
        .changes()
        .is_empty());
    let after = compiled(&DOMAIN.replacen(needle, replacement, 1));
    assert_ne!(before.source_digest(), after.source_digest());
    let delta = ess_diff::diff(&before, &after).expect("same system");
    assert_eq!(
        delta.changes().len(),
        1,
        "one isolated semantic change: {delta:?}"
    );
    let decoded = ess_diff::EssDelta::try_from(
        serde_json::from_str::<ess_diff::RawEssDelta>(&delta.to_canonical_json()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        decoded, delta,
        "the complete typed payload survives admission"
    );
    delta.changes()[0].clone()
}

fn relation(kind: &str, target: &str, via: &str) -> RelationContract {
    RelationContract {
        name: "child".into(),
        kind: kind.into(),
        target: target.parse().unwrap(),
        cardinality: "one".into(),
        via: via.into(),
    }
}

fn assert_relation(change: &SemanticChange, after: RelationContract) {
    assert_eq!(
        change,
        &SemanticChange::Entity {
            subject: "coverage.model.Parent".parse().unwrap(),
            changed: EntityChange::RelationsChanged {
                before: vec![relation("owns", "coverage.model.First", "link_id")],
                after: vec![after],
            },
        }
    );
}

#[test]
fn relation_kind_is_compared_with_unchanged_carriers_and_cardinality() {
    assert_relation(
        &one_change("kind: owns", "kind: references"),
        relation("references", "coverage.model.First", "link_id"),
    );
}

#[test]
fn relation_target_is_compared_with_both_targets_already_declared() {
    assert_relation(
        &one_change(
            "target: coverage.model.First",
            "target: coverage.model.Second",
        ),
        relation("owns", "coverage.model.Second", "link_id"),
    );
}

#[test]
fn relation_carrier_is_compared_with_both_fields_already_declared() {
    assert_relation(
        &one_change("via: link_id", "via: alternate_id"),
        relation("owns", "coverage.model.First", "alternate_id"),
    );
}

fn parameter(name: &str, ty: &str) -> ParameterContract {
    ParameterContract {
        name: name.into(),
        type_ref: ty.into(),
        wire: name.into(),
        display: name.into(),
        summary: None,
    }
}

fn assert_parameters(change: &SemanticChange, after: Vec<ParameterContract>) {
    assert_eq!(
        change,
        &SemanticChange::View {
            subject: "coverage.model.Parents".parse().unwrap(),
            changed: ViewChange::ParamsChanged {
                before: vec![parameter("first", "String"), parameter("second", "String")],
                after,
            },
        }
    );
}

#[test]
fn parameter_order_is_compared_without_changing_filter_or_field_types() {
    assert_parameters(
        &one_change(
            "      - {name: first, type: String}\n      - {name: second, type: String}",
            "      - {name: second, type: String}\n      - {name: first, type: String}",
        ),
        vec![parameter("second", "String"), parameter("first", "String")],
    );
}

#[test]
fn parameter_type_is_compared_without_changing_filter_names_or_order() {
    assert_parameters(
        &one_change(
            "{name: first, type: String}",
            "{name: first, type: Integer}",
        ),
        vec![parameter("first", "Integer"), parameter("second", "String")],
    );
}
