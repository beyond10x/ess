//! A model integer with a complete range is realized at a native width (beyond10x/ess#394).
//!
//! `minimum` and `maximum` inside `i32` give Rust `i32` and Go `int32`; inside `i64`, `i64` and
//! `int64`. An integer `const` (`version == 2`) is realized at the width of its value and stays a
//! runtime obligation. An integer with one bound or none keeps the exact JSON-number type: the model
//! names no width, and none is invented.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use schema_contract::realize::Plan;

const SOURCE: &str = r"format: ess/20
system: probe
version: v1
domains: [probe.meter]
domain: probe.meter
types:
  - name: probe.meter.Reading
    kind: struct
    fields:
      - {name: version, type: Integer}
      - {name: amount, type: Integer}
      - {name: total, type: Integer}
      - {name: count, type: Integer}
      - {name: free, type: Integer}
    invariants:
      - version == 2
      - amount >= -2147483648
      - amount <= 2147483647
      - total >= 0
      - total <= 9007199254740993
      - count >= 0
";

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &["probe.meter.Reading"])).expect("plan")
}

fn field_line<'a>(declarations: &'a str, field: &str) -> &'a str {
    declarations
        .lines()
        .find(|line| line.trim_start().starts_with(field))
        .unwrap_or_else(|| panic!("no `{field}` in:\n{declarations}"))
}

#[test]
fn rust_fields_take_the_narrowest_native_width() {
    let realization = plan().rust("probe-meter").expect("rust");
    let declarations = &realization.declarations;
    assert!(
        field_line(declarations, "pub version:").ends_with("i32,"),
        "{declarations}"
    );
    assert!(
        field_line(declarations, "pub amount:").ends_with("i32,"),
        "{declarations}"
    );
    assert!(
        field_line(declarations, "pub total:").ends_with("i64,"),
        "{declarations}"
    );
    assert!(
        field_line(declarations, "pub count:").ends_with("::serde_json::Number,"),
        "one bound names no width: {declarations}"
    );
    assert!(
        field_line(declarations, "pub free:").ends_with("::serde_json::Number,"),
        "{declarations}"
    );
}

#[test]
fn go_fields_take_the_narrowest_native_width() {
    let realization = plan()
        .go("probemeter", "example.com/probemeter")
        .expect("go");
    let declarations = &realization.declarations;
    assert!(
        field_line(declarations, "Version ").ends_with("int32"),
        "{declarations}"
    );
    assert!(
        field_line(declarations, "Amount ").ends_with("int32"),
        "{declarations}"
    );
    assert!(
        field_line(declarations, "Total ").ends_with("int64"),
        "{declarations}"
    );
    assert!(
        field_line(declarations, "Count ").ends_with("EssNumber"),
        "{declarations}"
    );
}

#[test]
fn an_integer_constant_stays_a_runtime_obligation() {
    let realization = plan().rust("probe-meter").expect("rust");
    let report = serde_json::to_value(&realization.report).expect("report serializes");
    let obligations = report["obligations"].to_string();
    assert!(
        obligations.contains("/properties/version/const"),
        "the constant is not discharged by the native type: {obligations}"
    );
}
