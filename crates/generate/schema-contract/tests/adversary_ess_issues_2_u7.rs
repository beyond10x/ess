//! Adversarial cases against the two opt-in type generator options of open-issue batch 2, unit U7
//! (beyond10x/ess#661): `with_declared_field_order` and the Rust option `box_only_recursion`.
//!
//! The unit's own fixture holds one self cycle, one two-type cycle and one list. These cases hold
//! the cycle shapes it does not: a three-type cycle through a required field, recursion through a
//! union payload, recursion through a map and an optional list, and a type reached both from inside
//! its cycle and from outside it. They also hold the records whose order the fixture does not
//! reach: an open struct, a shorthand struct and a field whose wire name differs from its name.
//!
//! The cases that build the generated crate are in `rust_realization.rs`.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

#[path = "fixtures/cycle_shapes.rs"]
mod cycle_shapes;

use std::collections::BTreeSet;

use cycle_shapes::{ROOTS, SOURCE};
use schema_contract::realize::{Plan, RustOptions};

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &ROOTS)).expect("plan")
}

fn boxed_only() -> RustOptions {
    RustOptions {
        box_only_recursion: true,
        ..RustOptions::default()
    }
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

/// Each declared field of a Rust struct and its type, in emitted order, as `field: type`.
fn rust_fields(text: &str, name: &str) -> Vec<String> {
    body(text, &format!("pub struct {name} {{\n"))
        .into_iter()
        .filter_map(|line| line.strip_prefix("    pub "))
        .map(|line| line.trim_end_matches(',').to_owned())
        .filter(|line| !line.starts_with("ess_extra:"))
        .collect()
}

fn rust_names(text: &str, name: &str) -> Vec<String> {
    rust_fields(text, name)
        .into_iter()
        .map(|line| line.split_once(": ").unwrap().0.to_owned())
        .collect()
}

fn go_fields(text: &str, name: &str) -> Vec<String> {
    body(text, &format!("type {name} struct {{\n"))
        .into_iter()
        .filter_map(|line| line.strip_prefix('\t')?.split_once(' '))
        .map(|(field, _)| field.to_owned())
        .filter(|field| field != "EssExtra")
        .collect()
}

/// Each property of a TypeScript object type; an open record is written `({ ... } & { ... })`.
fn ts_fields(text: &str, name: &str) -> Vec<String> {
    let head = format!("export type {name} = ");
    let opener = if text.contains(&format!("{head}({{\n")) {
        "({\n"
    } else {
        "{\n"
    };
    body(text, &format!("{head}{opener}"))
        .into_iter()
        .filter_map(|line| line.trim().split_once(':'))
        .map(|(field, _)| field.to_owned())
        .collect()
}

/// Every cycle shape the unit's fixture leaves out: each reference on a cycle of by-value fields
/// is boxed, every other one, and every one inside a list or a map, is held by value.
#[test]
fn box_only_recursion_boxes_exactly_the_cycle_edges_of_every_shape() {
    let rust = plan()
        .rust_with("demo_r", &boxed_only())
        .expect("rust")
        .declarations;
    let boxed = |inner: &str| format!("::std::boxed::Box<{inner}>");
    // A three-type cycle closed by one optional field boxes all three edges.
    assert_eq!(
        rust_fields(&rust, "DemoRC1"),
        [
            format!("c2: {}", boxed("DemoRC2")),
            "label: ::std::string::String".to_owned()
        ]
    );
    assert_eq!(
        rust_fields(&rust, "DemoRC2"),
        [format!("c3: {}", boxed("DemoRC3"))]
    );
    assert_eq!(
        rust_fields(&rust, "DemoRC3"),
        [format!("back: EssPresence<{}>", boxed("DemoRC1"))]
    );
    // Through a union payload: the struct's edge to the union is on the cycle.
    assert_eq!(
        rust_fields(&rust, "DemoRNeg"),
        [format!("inner: {}", boxed("DemoRExpr"))]
    );
    assert!(
        rust.contains(&boxed("DemoRNeg")),
        "the union's payload edge closes the cycle too:\n{rust}"
    );
    // A list, a map and an optional list are heap-held: nothing boxed.
    let dir = rust_fields(&rust, "DemoRDir");
    assert_eq!(dir.len(), 3, "{dir:?}");
    assert!(dir.iter().all(|field| !field.contains("Box<")), "{dir:?}");
    assert!(
        dir.contains(&"kids: ::std::vec::Vec<DemoRDir>".to_owned()),
        "{dir:?}"
    );
    // Reached from inside its cycle and from outside it: only the inside edge is boxed.
    assert_eq!(
        rust_fields(&rust, "DemoREntry"),
        [
            "leaf: DemoRLeaf".to_owned(),
            format!("next: EssPresence<{}>", boxed("DemoREntry")),
        ]
    );
    assert_eq!(
        rust_fields(&rust, "DemoROutside"),
        [
            "also: EssPresence<DemoREntry>",
            "entry: DemoREntry",
            "expr: DemoRExpr"
        ]
    );
}

/// A bundle's exact tuple holds its positions by value, so a cycle through a tuple position is
/// boxed, and a tuple position off any cycle is not. No model reaches a tuple; without this case
/// dropping tuple positions from the by-value walk leaves every test green.
#[test]
fn box_only_recursion_boxes_a_cycle_through_a_tuple_position() {
    let roots = BTreeSet::from(["Pair".to_owned(), "Tree".to_owned(), "Plain".to_owned()]);
    let bundle = schema_contract::bundle::import(
        &serde_json::json!({"components": {"schemas": {
            "Pair": {"type": "array", "prefixItems": [{"$ref": "#/components/schemas/Tree"}, {"$ref": "#/components/schemas/Plain"}],
                "items": false, "minItems": 2, "maxItems": 2},
            "Tree": {"type": "object", "properties": {"pair": {"$ref": "#/components/schemas/Pair"}}},
            "Plain": {"type": "object", "properties": {"label": {"type": "string"}}}
        }}})
        .to_string(),
        &roots,
        schema_contract::bundle::Dialect::Draft202012,
    )
    .expect("bundle");
    let rust = Plan::from_bundle(&bundle, &roots)
        .expect("plan")
        .rust_with("tuples", &boxed_only())
        .expect("rust")
        .declarations;
    assert!(
        rust.contains("pub struct Pair(pub ::std::boxed::Box<Tree>, pub Plain);"),
        "{rust}"
    );
    assert!(rust.contains("<::std::boxed::Box<Pair>>"), "{rust}");
}

/// Declared order reaches the records the unit's fixture has none of: an open struct, a
/// shorthand struct and a field whose wire name sorts first, in every target and under short
/// names.
#[test]
fn declared_order_reaches_open_shorthand_and_renamed_records() {
    let plan = plan().with_declared_field_order().expect("ordered");
    let short = plan
        .clone()
        .with_short_names()
        .expect("short names")
        .rust("demo_r")
        .expect("rust")
        .declarations;
    let rust = plan.rust("demo_r").expect("rust").declarations;
    let go = plan
        .go("demor", "example.invalid/demo/r")
        .expect("go")
        .declarations;
    let ts = plan.typescript().declarations;
    assert_eq!(rust_names(&rust, "DemoROpen"), ["zeta", "alpha", "aaa"]);
    assert_eq!(rust_names(&short, "Open"), ["zeta", "alpha", "aaa"]);
    assert_eq!(go_fields(&go, "DemoROpen"), ["Zeta", "Alpha", "Aaa"]);
    assert_eq!(
        ts_fields(&ts, "DemoROpen"),
        ["\"zeta\"", "\"alpha\"?", "\"aaa\""]
    );
    assert_eq!(rust_names(&rust, "DemoRShort"), ["zeta", "alpha"]);
    assert_eq!(go_fields(&go, "DemoRShort"), ["Zeta", "Alpha"]);
    assert_eq!(rust_names(&rust, "DemoRLeaf"), ["zeta", "alpha"]);
    assert_eq!(rust_names(&rust, "DemoREntry"), ["next", "leaf"]);
    // The open record's collision list follows the same order.
    assert!(
        rust.contains("if [\"zeta\", \"alpha\", \"aaa\"].iter().any("),
        "{rust}"
    );
}

/// Every object with declared fields has a declared order, the open, shorthand and renamed
/// records included; the only ones without are a union's variant objects, which the story leaves
/// unentered on purpose (tag and content).
#[test]
fn only_union_variant_objects_lack_a_declared_order() {
    let coverage = plan()
        .with_declared_field_order()
        .expect("ordered")
        .declared_order_coverage();
    let uncovered = coverage
        .iter()
        .filter(|(_, covered)| !**covered)
        .map(|(pointer, _)| pointer.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        uncovered,
        ["/$defs/demo.r.Expr/oneOf/0", "/$defs/demo.r.Expr/oneOf/1"],
        "{coverage:?}"
    );
    for root in ROOTS.iter().filter(|root| **root != "demo.r.Expr") {
        assert_eq!(
            coverage.get(&format!("/$defs/{root}")),
            Some(&true),
            "{root}: {coverage:?}"
        );
    }
}

/// Neither option moves a byte of any output, and asking for the option at its default is the
/// same as not asking.
#[test]
fn options_off_keep_every_output_byte_for_byte() {
    let plan = plan();
    let rust = plan.rust("demo_r").expect("rust");
    let off = plan
        .rust_with("demo_r", &RustOptions::default())
        .expect("rust");
    assert_eq!(rust.declarations, off.declarations);
    assert_eq!(
        serde_json::to_string(&rust.report).unwrap(),
        serde_json::to_string(&off.report).unwrap()
    );
    let configuration = |report: &schema_contract::realize::Report| {
        serde_json::to_string(&serde_json::to_value(report).unwrap()["configuration"]).unwrap()
    };
    assert_eq!(
        configuration(&rust.report),
        r#"{"language":"rust","package":"demo_r"}"#
    );
    assert_eq!(
        configuration(&plan.typescript().report),
        r#"{"language":"typescript"}"#
    );
    assert_eq!(
        configuration(
            &plan
                .go("demor", "example.invalid/demo/r")
                .expect("go")
                .report
        ),
        r#"{"language":"go","module":"example.invalid/demo/r","package":"demor"}"#
    );
    // The default boxes every reference, the list items and the map values included.
    assert_eq!(
        rust_fields(&rust.declarations, "DemoROutside"),
        [
            "also: EssPresence<::std::boxed::Box<DemoREntry>>",
            "entry: ::std::boxed::Box<DemoREntry>",
            "expr: ::std::boxed::Box<DemoRExpr>"
        ]
    );
}
