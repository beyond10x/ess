//! Two opt-in generator options (beyond10x/ess#661): `with_declared_field_order` emits a model
//! record's fields in the order the specification declares them, for Rust, Go and TypeScript, and
//! the Rust option `box_only_recursion` holds a named type by value, boxing a reference only where
//! it closes a cycle of by-value fields. Without them every output keeps its bytes.
//!
//! The generated crate built with both options is in `rust_realization.rs`, the test file
//! allowed to run cargo.

#[allow(dead_code)]
#[path = "../../ess-gen/tests/fixtures/declaration_order.rs"]
mod fixture;

use std::collections::BTreeSet;

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::{Plan, RustOptions};
use serde_json::json;

/// The plan of every root of the fixture model.
fn plan() -> Plan {
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("demo.yaml"),
        ess_domain::spec::RawSpecFile::parse(fixture::MODEL).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let roots = fixture::ROOTS
        .iter()
        .map(|root| (*root).to_owned())
        .collect::<BTreeSet<_>>();
    Plan::from_model(&ess_gen::schema::ModelTypes::select(&ir, &roots).unwrap()).unwrap()
}

/// The declaration name of a model definition.
fn declared(name: &str) -> String {
    let mut declared = String::new();
    for part in name.split('.') {
        let mut chars = part.chars();
        declared.push(chars.next().unwrap().to_ascii_uppercase());
        declared.push_str(chars.as_str());
    }
    declared
}

/// The lines of one declaration's body, from the line after `head` to its closing brace.
fn body<'a>(text: &'a str, head: &str) -> Vec<&'a str> {
    let start = text
        .find(head)
        .unwrap_or_else(|| panic!("no {head:?} in\n{text}"));
    text[start..]
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with('}'))
        .collect()
}

/// Each declared field of a Rust struct and its type, in emitted order.
fn rust_fields(text: &str, name: &str) -> Vec<(String, String)> {
    body(text, &format!("pub struct {name} {{\n"))
        .into_iter()
        .filter_map(|line| line.strip_prefix("    pub "))
        .map(|line| {
            let (field, ty) = line.split_once(": ").unwrap();
            (field.to_owned(), ty.trim_end_matches(',').to_owned())
        })
        .filter(|(field, _)| field != "ess_extra")
        .collect()
}

/// Each declared field of a Go struct, in emitted order.
fn go_fields(text: &str, name: &str) -> Vec<String> {
    body(text, &format!("type {name} struct {{\n"))
        .into_iter()
        .filter_map(|line| line.strip_prefix('\t')?.split_once(' '))
        .map(|(field, _)| field.to_owned())
        .filter(|field| field != "EssExtra")
        .collect()
}

/// Each property of a TypeScript object type, as written, in emitted order.
fn ts_fields(text: &str, name: &str) -> Vec<String> {
    body(text, &format!("export type {name} = {{\n"))
        .into_iter()
        .filter_map(|line| line.trim().split_once(':'))
        .map(|(field, _)| field.to_owned())
        .collect()
}

/// A declaration's name and its fields as Rust, Go and TypeScript list them.
type Orders = Vec<(String, Vec<String>, Vec<String>, Vec<String>)>;

/// Every output's field order of each definition that declares `zeta, alpha, mid`.
fn orders(plan: &Plan) -> Orders {
    let rust = plan.rust("demo_g").unwrap().declarations;
    let go = plan.go("demog", "example.com/demo/g").unwrap().declarations;
    let ts = plan.typescript().declarations;
    fixture::DECLARED
        .iter()
        .map(|name| {
            let name = declared(name);
            let rust = rust_fields(&rust, &name)
                .into_iter()
                .map(|(field, _)| field)
                .collect();
            (
                name.clone(),
                rust,
                go_fields(&go, &name),
                ts_fields(&ts, &name),
            )
        })
        .collect()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn default_output_keeps_wire_name_order() {
    let plan = plan();
    for (name, rust, go, ts) in orders(&plan) {
        assert_eq!(rust, strings(&["alpha", "mid", "zeta"]), "{name}");
        assert_eq!(go, strings(&["Alpha", "Mid", "Zeta"]), "{name}");
        assert_eq!(
            ts,
            strings(&["\"alpha\"", "\"mid\"?", "\"zeta\""]),
            "{name}"
        );
    }
    let rust = plan.rust("demo_g").unwrap().declarations;
    assert_eq!(
        rust_fields(&rust, "DemoGOuter"),
        [
            ("alpha", "::std::string::String"),
            ("mid", "EssPresence<::std::boxed::Box<DemoGInner>>"),
            ("zeta", "::std::boxed::Box<DemoGInner>"),
        ]
        .map(|(field, ty)| (field.to_owned(), ty.to_owned()))
    );
    assert_eq!(
        rust_fields(&rust, "DemoGNode"),
        [
            ("kids", "::std::vec::Vec<::std::boxed::Box<DemoGNode>>"),
            ("next", "EssPresence<::std::boxed::Box<DemoGNode>>"),
        ]
        .map(|(field, ty)| (field.to_owned(), ty.to_owned()))
    );
    assert_eq!(
        rust_fields(&rust, "DemoGHolder"),
        [("tree".to_owned(), "::std::boxed::Box<DemoGNode>".to_owned())]
    );
    assert_eq!(
        plan.rust_with(
            "demo_g",
            &RustOptions {
                box_only_recursion: false,
                ..RustOptions::default()
            }
        )
        .unwrap()
        .declarations,
        rust
    );
}

#[test]
fn declared_field_order_follows_the_specification() {
    let plan = plan().with_declared_field_order().unwrap();
    for (name, rust, go, ts) in orders(&plan) {
        assert_eq!(rust, strings(&["zeta", "alpha", "mid"]), "{name}");
        assert_eq!(go, strings(&["Zeta", "Alpha", "Mid"]), "{name}");
        assert_eq!(
            ts,
            strings(&["\"zeta\"", "\"alpha\"", "\"mid\"?"]),
            "{name}"
        );
    }
    let rust = plan.rust("demo_g").unwrap().declarations;
    assert_eq!(
        rust_fields(&rust, "DemoGNode")
            .into_iter()
            .map(|(field, _)| field)
            .collect::<Vec<_>>(),
        ["next", "kids"]
    );
}

#[test]
fn every_model_object_has_a_declared_order() {
    let coverage = plan()
        .with_declared_field_order()
        .unwrap()
        .declared_order_coverage();
    let expected = fixture::ROOTS
        .iter()
        .map(|root| format!("/$defs/{root}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        coverage.keys().cloned().collect::<BTreeSet<_>>(),
        expected,
        "{coverage:?}"
    );
    let uncovered = coverage
        .iter()
        .filter(|(_, covered)| !**covered)
        .collect::<Vec<_>>();
    assert!(uncovered.is_empty(), "{uncovered:?}");
}

#[test]
fn a_bundle_refuses_declared_field_order() {
    let roots = BTreeSet::from(["Open".to_owned()]);
    let bundle = import(
        &json!({"components": {"schemas": {
            "Open": {"type": "object", "properties": {"b": {"type": "string"}, "a": {"type": "string"}}}
        }}})
        .to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    let plan = Plan::from_bundle(&bundle, &roots).unwrap();
    let refused = plan.clone().with_declared_field_order().unwrap_err();
    assert_eq!(
        refused
            .0
            .iter()
            .map(|finding| (finding.pointer.as_str(), finding.rule.as_str()))
            .collect::<Vec<_>>(),
        [("/", "declared_field_order_bundle")],
        "{refused}"
    );
    assert!(
        refused.0[0].detail.contains("declaration order"),
        "{refused}"
    );
    // A bundle's coverage names its object, uncovered: its fields keep wire-name order.
    assert_eq!(
        plan.declared_order_coverage()
            .into_iter()
            .collect::<Vec<_>>(),
        [("/components/schemas/Open".to_owned(), false)]
    );
}

#[test]
fn only_recursive_references_are_boxed() {
    let options = RustOptions {
        box_only_recursion: true,
        ..RustOptions::default()
    };
    let rust = plan().rust_with("demo_g", &options).unwrap().declarations;
    let typed = |name: &str| {
        rust_fields(&rust, name)
            .into_iter()
            .map(|(field, ty)| format!("{field}: {ty}"))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        typed("DemoGOuter"),
        [
            "alpha: ::std::string::String",
            "mid: EssPresence<DemoGInner>",
            "zeta: DemoGInner",
        ]
    );
    assert_eq!(
        typed("DemoGNode"),
        [
            "kids: ::std::vec::Vec<DemoGNode>",
            "next: EssPresence<::std::boxed::Box<DemoGNode>>",
        ]
    );
    assert_eq!(
        typed("DemoGA"),
        ["b: EssPresence<::std::boxed::Box<DemoGB>>"]
    );
    assert_eq!(
        typed("DemoGB"),
        ["a: EssPresence<::std::boxed::Box<DemoGA>>"]
    );
    assert_eq!(typed("DemoGHolder"), ["tree: DemoGNode"]);
}

#[test]
fn the_report_records_declared_field_order() {
    let plan = plan();
    let configuration = |plan: &Plan| {
        (
            serde_json::to_value(plan.typescript().report).unwrap()["configuration"].clone(),
            serde_json::to_value(plan.go("demog", "example.com/demo/g").unwrap().report).unwrap()
                ["configuration"]
                .clone(),
        )
    };
    assert_eq!(
        configuration(&plan),
        (
            json!({"language": "typescript"}),
            json!({"language": "go", "package": "demog", "module": "example.com/demo/g"})
        )
    );
    assert_eq!(
        configuration(&plan.with_declared_field_order().unwrap()),
        (
            json!({"language": "typescript", "declared_field_order": true}),
            json!({"language": "go", "package": "demog", "module": "example.com/demo/g",
                "declared_field_order": true})
        )
    );
}
