//! Compiled command inputs supply the quantifier facts exercised here.

use std::collections::BTreeMap;

use ess_compiler::{compile, ir::EssIr, source::SourceMap};
use ess_conformance::{bind, flatten, when, Completeness, Decision, Reason, ShapeError};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{facts::FactPath, node::Node, predicate::Truth, FactSource, FactValue};

const MODEL: &str = r"
format: ess/1
system: collections
version: v1
domain: collections.data
types:
  - name: collections.data.Limit
    kind: struct
    fields:
      - {name: value, type: Integer}
  - name: collections.data.Kind
    kind: enum
    variants: [First, Second]
  - name: collections.data.Entry
    kind: struct
    fields:
      - {name: amount, type: Integer}
      - {name: kind, type: collections.data.Kind}
      - {name: score, type: 'Optional<Integer>'}
      - {name: children, type: 'List<Integer>'}
  - name: collections.data.Wrapped
    kind: newtype
    of: collections.data.Entry
  - name: collections.data.Tree
    kind: struct
    fields:
      - {name: children, type: 'List<collections.data.Tree>'}
events:
  - name: collections.data.Accepted
    fields: []
errors:
  - name: collections.data.Refused
    summary: A declared refusal.
commands:
  - name: collections.data.Check
    input:
      - {name: entries, type: 'List<collections.data.Wrapped>'}
      - {name: groups, type: 'List<List<Optional<Integer>>>'}
      - {name: labels, type: 'Map<String, Optional<Integer>>'}
      - {name: optional, type: 'Optional<List<Integer>>'}
      - {name: tree, type: 'Optional<collections.data.Tree>'}
      - {name: limit, type: collections.data.Limit}
    outcomes:
      - name: accepted
        when: GUARD
        emits: [collections.data.Accepted]
      - name: refused
        error: collections.data.Refused
";

fn compiled(guard: &str) -> EssIr {
    let raw = RawSpecFile::parse(&MODEL.replace("GUARD", guard)).unwrap();
    let spec = Specification::assemble([(Source::new("collections.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn input(json: &str) -> BTreeMap<String, Node> {
    serde_json::from_str(json).unwrap()
}

fn candidate() -> BTreeMap<String, Node> {
    input(
        r#"{
        "entries": [{"amount": 3, "kind": "First", "children": [4, 5]}],
        "groups": [[null, 4], []],
        "labels": {"z": 9, "count": 7, "a.b": 5, "0": 3},
        "limit": {"value": 2}
    }"#,
    )
}

fn evaluate(ir: &EssIr, values: &BTreeMap<String, Node>) -> Truth {
    let command = ir.commands().values().next().unwrap();
    let facts = flatten(ir, command, values).unwrap();
    when(&command.outcomes[0]).unwrap().evaluate(&facts)
}

#[test]
fn compiled_count_and_ordinal_guards_read_real_inputs() {
    for guard in [
        "entries.count == 1",
        "entries.0.amount == 3",
        "entries.0.children.1 == 5",
        "groups.0.count == 2",
        "groups.1.count == 0",
        "labels.count == 4",
        "{forall: {in: entries, as: entry, that: entry.kind == First}}",
    ] {
        let ir = compiled(guard);
        assert_eq!(evaluate(&ir, &candidate()), Truth::True, "{guard}");
    }
}

#[test]
fn maps_bind_values_in_key_order_without_key_path_collisions() {
    let ir = compiled("{forall: {in: labels, as: value, that: value > limit.value}}");
    let values = candidate();
    assert_eq!(evaluate(&ir, &values), Truth::True);
    let command = ir.commands().values().next().unwrap();
    let facts = flatten(&ir, command, &values).unwrap();
    for (path, expected) in [
        ("labels.count", 4),
        ("labels.0", 3),
        ("labels.1", 5),
        ("labels.2", 7),
        ("labels.3", 9),
    ] {
        assert_eq!(
            facts.fact(&FactPath::new(path).unwrap()),
            Some(FactValue::count(expected))
        );
    }
    assert_eq!(facts.fact(&FactPath::new("labels.a.b").unwrap()), None);
    assert!(ess_compiler::expression::resolve_path(
        &ir,
        &command.input,
        &FactPath::new("labels.0").unwrap(),
        "map selector"
    )
    .is_err());
}

#[test]
fn empty_and_absent_collections_have_different_quantified_truth() {
    for (operator, empty) in [("forall", Truth::True), ("exists", Truth::False)] {
        for field in ["groups", "labels", "optional"] {
            let ir = compiled(&format!(
                "{{{operator}: {{in: {field}, as: item, that: false}}}}"
            ));
            let mut values = candidate();
            values.insert(
                field.into(),
                if field == "labels" {
                    Node::Map(BTreeMap::new())
                } else {
                    Node::Seq(vec![])
                },
            );
            assert_eq!(evaluate(&ir, &values), empty, "{operator} {field}");
            if field == "optional" {
                values.remove(field);
                assert_eq!(evaluate(&ir, &values), Truth::Unknown);
                values.insert(field.into(), Node::Null);
                assert_eq!(evaluate(&ir, &values), Truth::Unknown);
            }
        }
    }
}

#[test]
fn nested_binders_preserve_outer_and_free_references_and_allow_shadowing() {
    for guard in [
        "{forall: {in: entries, as: entry, that: {exists: {in: entry.children, as: child, that: {all: ['child > entry.amount', 'child > limit.value']}}}}}",
        "{forall: {in: entries, as: entry, that: {forall: {in: entry.children, as: entry, that: entry > limit.value}}}}",
        "{forall: {in: groups, as: group, that: {any: ['group.count == 0', {exists: {in: group, as: item, that: item > limit.value}}]}}}",
    ] {
        let ir = compiled(guard);
        assert_eq!(evaluate(&ir, &candidate()), Truth::True, "{guard}");
    }
}

#[test]
fn optional_elements_preserve_ordinals_and_kleene_dominance() {
    for (operator, values, truth) in [
        ("forall", "[null, 4]", Truth::Unknown),
        ("forall", "[null, 1]", Truth::False),
        ("exists", "[null, 4]", Truth::True),
        ("exists", "[null, 1]", Truth::Unknown),
    ] {
        let ir = compiled(&format!(
            "{{{operator}: {{in: groups.0, as: value, that: value > limit.value}}}}"
        ));
        let mut data = candidate();
        data.insert(
            "groups".into(),
            serde_json::from_str(&format!("[{values}]")).unwrap(),
        );
        assert_eq!(evaluate(&ir, &data), truth);
        let command = ir.commands().values().next().unwrap();
        let facts = flatten(&ir, command, &data).unwrap();
        assert_eq!(
            facts.cardinality(&FactPath::new("groups.0").unwrap()),
            Some(2)
        );
        assert_eq!(facts.fact(&FactPath::new("groups.0.0").unwrap()), None);
        assert!(facts.fact(&FactPath::new("groups.0.1").unwrap()).is_some());
    }
}

#[test]
fn missing_input_counts_and_ordinals_report_absent_values() {
    for guard in [
        "optional.count > 0",
        "entries.5.amount > 0",
        "{forall: {in: optional, as: value, that: value > 0}}",
    ] {
        let ir = compiled(guard);
        let command = ir.commands().values().next().unwrap();
        let data = candidate();
        let facts = flatten(&ir, command, &data).unwrap();
        let Decision::Unevaluable(unknown) = facts.decide(when(&command.outcomes[0]).unwrap())
        else {
            panic!("{guard} must remain Unknown");
        };
        assert!(
            matches!(unknown.causes[0].reason, Reason::ValueAbsent { .. }),
            "{:?}",
            unknown.causes
        );
    }
}

#[test]
fn malformed_collection_elements_accumulate_before_evaluation() {
    let ir = compiled("entries.count > 0");
    let command = ir.commands().values().next().unwrap();
    let values = input(
        r#"{
        "entries": [{"amount": "bad", "kind": "Unknown", "children": [null], "extra": true}, null],
        "groups": [[true]], "labels": {"one": "bad"}, "limit": {"value": 2}
    }"#,
    );
    let errors = flatten(&ir, command, &values).unwrap_err();
    let rendered = errors.to_string();
    for path in [
        "entries.0.amount",
        "entries.0.kind",
        "entries.0.children.0",
        "entries.1",
        "groups.0.0",
        "labels.0",
        "extra",
    ] {
        assert!(rendered.contains(path), "missing {path}: {rendered}");
    }
    assert_eq!(errors.len(), 7);
}

#[test]
fn partial_binding_still_checks_supplied_nested_fields() {
    let ir = compiled("limit.value > 0");
    let command = ir.commands().values().next().unwrap();
    let valid = input(r#"{"entries": [{"amount": 1, "kind": "Second", "children": []}]}"#);
    let facts = bind(&ir, &command.input, &valid, Completeness::Partial).unwrap();
    assert_eq!(
        facts.cardinality(&FactPath::new("entries").unwrap()),
        Some(1)
    );
    let invalid = input(r#"{"entries": [{"kind": "Second", "children": []}]}"#);
    let errors = bind(&ir, &command.input, &invalid, Completeness::Partial).unwrap_err();
    assert!(
        matches!(errors.iter().next().unwrap(), ShapeError::MissingField { at, field } if at == "entries.0" && field == "amount")
    );
}

#[test]
fn collection_descent_obeys_the_existing_recursive_type_bound() {
    let ir = compiled("limit.value > 0");
    let command = ir.commands().values().next().unwrap();
    let mut tree = Node::Map(BTreeMap::from([("children".into(), Node::Seq(vec![]))]));
    for _ in 0..20 {
        tree = Node::Map(BTreeMap::from([("children".into(), Node::Seq(vec![tree]))]));
    }
    let mut values = candidate();
    values.insert("tree".into(), tree);
    let errors = flatten(&ir, command, &values).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| matches!(error, ShapeError::TooDeep { .. })));
}
