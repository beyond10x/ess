//! Adversarial cases against the two opt-in type generator options of open-issue batch 2, unit U7
//! (beyond10x/ess#661): `with_declared_field_order` and the Rust option `box_only_recursion`.
//!
//! The unit's own fixture holds one self cycle, one two-type cycle and one list. These cases hold
//! the cycle shapes it does not: a three-type cycle through a required field, recursion through a
//! union payload, recursion through a map and an optional list, and a type reached both from inside
//! its cycle and from outside it. They also hold the records whose order the fixture does not
//! reach: an open struct, a shorthand struct and a field whose wire name differs from its name.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::realize::{Plan, RustDerive, RustOptions};

const SOURCE: &str = r"format: ess/32
system: demo
version: v1
domains: [demo.r]
domain: demo.r
types:
  - name: demo.r.Leaf
    kind: struct
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
  - name: demo.r.C1
    kind: struct
    fields:
      - {name: c2, type: demo.r.C2}
      - {name: label, type: String}
  - name: demo.r.C2
    kind: struct
    fields:
      - {name: c3, type: demo.r.C3}
  - name: demo.r.C3
    kind: struct
    fields:
      - {name: back, type: 'Optional<demo.r.C1>'}
  - name: demo.r.Neg
    kind: struct
    fields:
      - {name: inner, type: demo.r.Expr}
  - name: demo.r.Expr
    kind: union
    tag: kind
    variants:
      lit: String
      neg: demo.r.Neg
  - name: demo.r.Dir
    kind: struct
    fields:
      - {name: entries, type: 'Map<String, demo.r.Dir>'}
      - {name: kids, type: 'List<demo.r.Dir>'}
      - {name: parent, type: 'Optional<List<demo.r.Dir>>'}
  - name: demo.r.Entry
    kind: struct
    fields:
      - {name: next, type: 'Optional<demo.r.Entry>'}
      - {name: leaf, type: demo.r.Leaf}
  - name: demo.r.Outside
    kind: struct
    fields:
      - {name: entry, type: demo.r.Entry}
      - {name: also, type: 'Optional<demo.r.Entry>'}
      - {name: expr, type: demo.r.Expr}
  - name: demo.r.Open
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: 'Optional<String>'}
      - {name: renamed, type: String, wire: aaa}
  - name: demo.r.Short
    kind: struct
    shorthand: zeta
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: 'Optional<String>'}
";

const ROOTS: [&str; 11] = [
    "demo.r.Leaf",
    "demo.r.C1",
    "demo.r.C2",
    "demo.r.C3",
    "demo.r.Neg",
    "demo.r.Expr",
    "demo.r.Dir",
    "demo.r.Entry",
    "demo.r.Outside",
    "demo.r.Open",
    "demo.r.Short",
];

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

const WIRE_ALL_OPTIONS: &str = r##"
use demo_r::*;

fn round_trip<T>(text: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize + PartialEq + std::fmt::Debug,
{
    let value: T = serde_json::from_str(text).unwrap();
    assert_eq!(serde_json::to_string(&value).unwrap(), text);
    let again: T = serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
    assert_eq!(again, value);
}

#[test]
fn every_cycle_shape_round_trips() {
    round_trip::<DemoRC1>(r#"{"c2":{"c3":{"back":{"c2":{"c3":{}},"label":"in"}}},"label":"out"}"#);
    round_trip::<DemoRExpr>(r#"{"kind":"neg","value":{"inner":{"kind":"neg","value":{"inner":{"kind":"lit","value":"x"}}}}}"#);
    round_trip::<DemoRDir>(r#"{"entries":{"a":{"entries":{},"kids":[]}},"kids":[{"entries":{},"kids":[],"parent":[]}],"parent":[{"entries":{},"kids":[]}]}"#);
    round_trip::<DemoROutside>(r#"{"entry":{"next":{"leaf":{"zeta":"z","alpha":"a"}},"leaf":{"zeta":"y","alpha":"b"}},"also":{"leaf":{"zeta":"x","alpha":"c"}},"expr":{"kind":"lit","value":"v"}}"#);
    // The open record drops its unknown member and writes the declared order.
    let open: DemoROpen = serde_json::from_str(r#"{"aaa":"r","extra":1,"alpha":null,"zeta":"z"}"#).unwrap();
    assert_eq!(serde_json::to_string(&open).unwrap(), r#"{"zeta":"z","aaa":"r"}"#);
    let short: DemoRShort = serde_json::from_str(r#""z""#).unwrap();
    assert_eq!(serde_json::to_string(&short).unwrap(), r#"{"zeta":"z"}"#);
    round_trip::<DemoRShort>(r#"{"zeta":"z","alpha":"a"}"#);
}

#[test]
fn a_derived_ord_compares_in_declaration_order() {
    let first = DemoRLeaf { zeta: "a".to_owned(), alpha: "z".to_owned() };
    let second = DemoRLeaf { zeta: "b".to_owned(), alpha: "a".to_owned() };
    assert!(first < second);
    let mut set = std::collections::BTreeSet::new();
    set.insert(serde_json::from_str::<DemoRC1>(r#"{"c2":{"c3":{}},"label":"x"}"#).unwrap());
    set.insert(serde_json::from_str::<DemoRC1>(r#"{"c2":{"c3":{}},"label":"x"}"#).unwrap());
    assert_eq!(set.len(), 1);
}
"##;

/// Both options with every other Rust option on: the crate builds (no E0072 on any cycle shape),
/// each shape round-trips, and a derived `Ord` compares fields in declaration order, as the
/// option's documentation says.
#[test]
fn every_cycle_shape_builds_and_round_trips_with_every_option() {
    let options = RustOptions {
        derives: BTreeSet::from([
            RustDerive::Eq,
            RustDerive::Hash,
            RustDerive::PartialOrd,
            RustDerive::Ord,
        ]),
        fold_null: true,
        drop_unknown_members: true,
        box_only_recursion: true,
        ..RustOptions::default()
    };
    let realization = plan()
        .with_declared_field_order()
        .expect("ordered")
        .rust_with("demo_r", &options)
        .expect("rust");
    compile(
        "adversary-u7-all",
        &realization.supporting["Cargo.toml"],
        &realization.declarations,
        WIRE_ALL_OPTIONS,
    );
}

const WIRE_OPEN: &str = r##"
use demo_r::*;

#[test]
fn the_open_record_writes_declared_members_first_and_keeps_the_rest() {
    let open: DemoROpen = serde_json::from_str(r#"{"extra":1,"aaa":"r","alpha":"a","zeta":"z"}"#).unwrap();
    assert_eq!(serde_json::to_string(&open).unwrap(), r#"{"zeta":"z","alpha":"a","aaa":"r","extra":1}"#);
}
"##;

/// An open record under declared order alone: declared members in declaration order, then the
/// members it keeps.
#[test]
fn an_open_record_builds_and_encodes_in_declaration_order() {
    let realization = plan()
        .with_declared_field_order()
        .expect("ordered")
        .rust_with("demo_r", &boxed_only())
        .expect("rust");
    compile(
        "adversary-u7-open",
        &realization.supporting["Cargo.toml"],
        &realization.declarations,
        WIRE_OPEN,
    );
}

fn compile(name: &str, manifest: &str, source: &str, tests: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("tests")).expect("scratch");
    fs::write(root.join("Cargo.toml"), manifest).expect("manifest");
    fs::write(root.join("types.rs"), source).expect("types");
    fs::write(root.join("tests/wire.rs"), tests).expect("tests");
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary-u7-target"),
        )
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
