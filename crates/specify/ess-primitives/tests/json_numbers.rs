//! A JSON number means one thing whichever `serde_json` build parsed it.
//!
//! `entity-core` enables `serde_json`'s `arbitrary_precision`, and Cargo unifies that feature into
//! every build that holds both, so the same ESS source runs against two `serde_json` builds. With the
//! feature on, `serde_json::Value` keeps the literal spelling — `1.50` stays `1.50`, `1E2` stays
//! `1e2`, `-0` becomes the integer `0`, and `1e400` is admitted — where the build without it holds
//! the binary64. `ess_primitives::json` is the reader that answers as the build without the feature
//! does, in both builds; each expectation here is that build's own answer.

use ess_primitives::json;
use serde_json::Value;

fn read(text: &str) -> Value {
    json::from_str(text).unwrap_or_else(|error| panic!("{text} reads: {error}"))
}

#[test]
fn a_number_reads_as_the_value_serde_json_holds_without_arbitrary_precision() {
    for (token, written) in [
        ("0", "0"),
        ("-1", "-1"),
        ("-0", "-0.0"),
        ("1.50", "1.5"),
        ("1E2", "100.0"),
        ("1e-7", "1e-7"),
        ("0.0", "0.0"),
        ("-9223372036854775808", "-9223372036854775808"),
        ("18446744073709551615", "18446744073709551615"),
        ("18446744073709551616", "1.8446744073709552e+19"),
        ("-9223372036854775809", "-9.223372036854776e+18"),
        ("100000000000000000000000000000", "1e+29"),
    ] {
        let value = read(token);
        assert_eq!(value.to_string(), written, "{token}");
        assert_eq!(
            read(&format!("[{token}]")).to_string(),
            format!("[{written}]"),
            "{token} inside an array"
        );
        assert_eq!(
            read(&format!(r#"{{"n":{token}}}"#)).to_string(),
            format!(r#"{{"n":{written}}}"#),
            "{token} inside an object"
        );
        let number = value.as_number().expect("a number");
        let float = written.contains(['.', 'e']);
        assert_eq!(number.is_f64(), float, "{token}: is_f64");
        assert_eq!(
            number.is_i64() || number.is_u64(),
            !float,
            "{token}: integer"
        );
    }
}

#[test]
fn two_spellings_of_one_binary64_are_one_value() {
    assert_eq!(read("1.0"), read("1.00"));
    assert_eq!(read("100.0"), read("1e2"));
    assert_eq!(read("-0"), read("-0.0"));
    assert_ne!(read("1"), read("1.0"));
}

#[test]
fn a_number_past_binary64_is_refused() {
    for text in ["1e400", "[1e400]", r#"{"a":-1e400}"#, "[1, [2, 1e999]]"] {
        assert!(json::from_str(text).is_err(), "{text}");
        assert!(json::from_slice(text.as_bytes()).is_err(), "{text}");
    }
}

/// YAML written from a `serde_json` value carries plain numbers.
///
/// The oracle is the same document as typed Rust, whose numbers never pass through
/// `serde_json::Number` and so are written the same way in both builds.
#[test]
#[allow(clippy::items_after_statements)] // the two wrapper types read best beside the case they serve
fn yaml_written_from_a_json_value_carries_plain_numbers() {
    // Fields in key order, because a `serde_json::Map` is sorted.
    #[derive(serde::Serialize)]
    struct Typed {
        count: u64,
        delta: i64,
        list: Vec<f64>,
        name: &'static str,
        ratio: f64,
    }
    let typed = Typed {
        count: 18_446_744_073_709_551_615,
        delta: -2,
        list: vec![100.0, 1e300, -0.0],
        name: "1.50",
        ratio: 1.5,
    };
    let value = read(
        r#"{"count":18446744073709551615,"delta":-2,"ratio":1.50,"list":[1E2,1e300,-0],"name":"1.50"}"#,
    );
    let expected = serde_yaml::to_string(&typed).expect("typed YAML");
    assert_eq!(json::to_yaml_string(&value).expect("YAML"), expected);
    // Also where the JSON value is one field of a larger serialisable document.
    #[derive(serde::Serialize)]
    struct Outer<'a> {
        inner: &'a Value,
    }
    #[derive(serde::Serialize)]
    struct TypedOuter<'a> {
        inner: &'a Typed,
    }
    assert_eq!(
        json::to_yaml_string(&Outer { inner: &value }).expect("YAML"),
        serde_yaml::to_string(&TypedOuter { inner: &typed }).expect("typed YAML")
    );
    // And a value built in-process rather than read.
    let built = serde_json::json!({"count": 3_u64, "ratio": 0.25});
    assert_eq!(
        json::to_yaml_string(&built).expect("YAML"),
        "count: 3\nratio: 0.25\n"
    );
}

/// `Node`, the domain's untyped value, reads a number as the build without the feature does.
#[test]
fn a_node_reads_a_number_as_the_build_without_arbitrary_precision_does() {
    for (token, written) in [
        ("-0", "-0.0"),
        ("1.50", "1.5"),
        ("1E2", "100.0"),
        ("18446744073709551617", "1.8446744073709552e+19"),
        ("7", "7.0"),
    ] {
        let node: ess_primitives::node::Node =
            serde_json::from_str(token).unwrap_or_else(|error| panic!("{token}: {error}"));
        assert_eq!(
            serde_json::to_string(&node).expect("a node writes"),
            written,
            "{token}"
        );
    }
    assert!(serde_json::from_str::<ess_primitives::node::Node>("[1e400]").is_err());
}

/// A value parsed by plain `serde_json` is brought to the same numbers.
#[test]
fn a_value_parsed_elsewhere_is_brought_to_the_same_numbers() {
    let text = r#"{"a":[1.50,1E2,18446744073709551616,7,-7],"b":{"c":0.10}}"#;
    let parsed: Value = serde_json::from_str(text).expect("json");
    assert_eq!(
        json::canonical(&parsed).expect("finite").to_string(),
        read(text).to_string()
    );
    if let Ok(wide) = serde_json::from_str::<Value>("[1e400]") {
        assert!(json::canonical(&wide).is_err(), "{wide}");
    }
}

#[test]
fn every_other_value_reads_as_serde_json_reads_it() {
    let text = r#"{"a":[true,false,null,"xé",{"b":{}}],"c":"1.50"}"#;
    assert_eq!(
        read(text),
        serde_json::from_str::<Value>(text).expect("json")
    );
    assert!(json::from_str("[1,").is_err());
    assert!(json::from_str("1 2").is_err());
}
