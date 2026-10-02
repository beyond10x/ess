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

#[test]
fn native_rust_integer_widths_preserve_wire_boundaries() {
    let output = plan().rust("integer_widths").unwrap();
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("integer-widths-rust-{}", std::process::id()));
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(root.join("Cargo.toml"), &output.supporting["Cargo.toml"]).unwrap();
    std::fs::write(root.join("types.rs"), &output.declarations).unwrap();
    std::fs::write(root.join("tests/wire.rs"), RUST_WIRE).unwrap();
    for args in [
        vec!["generate-lockfile", "--offline"],
        vec!["test", "--offline", "--locked"],
    ] {
        let result = std::process::Command::new(env!("CARGO"))
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

const RUST_WIRE: &str = r##"
use integer_widths::ProbeMeterReading as Reading;

#[test]
fn exact_boundaries_and_overflow() {
    for amount in [i32::MIN, i32::MAX] {
        for total in [9007199254740992_i64, 9007199254740993] {
            let wire = format!(r#"{{"version":2,"amount":{amount},"total":{total},"count":0,"free":0}}"#);
            let decoded: Reading = serde_json::from_str(&wire).unwrap();
            assert_eq!(decoded.amount, amount);
            assert_eq!(decoded.total, total);
            let encoded = serde_json::to_string(&decoded).unwrap();
            assert!(encoded.contains(&format!(r#""total":{total}"#)));
        }
    }
    for (amount, total) in [
        ("2147483648", "0"), ("-2147483649", "0"),
        ("0", "9223372036854775808"), ("0", "-9223372036854775809"),
        ("null", "0"), ("1.5", "0"),
    ] {
        let wire = format!(r#"{{"version":2,"amount":{amount},"total":{total},"count":0,"free":0}}"#);
        assert!(serde_json::from_str::<Reading>(&wire).is_err(), "{wire}");
    }
}
"##;
