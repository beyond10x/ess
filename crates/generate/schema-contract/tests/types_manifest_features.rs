//! The generated Rust manifest asks for `serde_json`'s `arbitrary_precision` only where the realized
//! types hold a JSON number in `serde_json`, and then through a default-on crate feature that a
//! consumer turns off with `default-features = false` (beyond10x/ess#483).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::{Plan, Realization};
use serde_json::{json, Value};

/// The crate feature a generated manifest declares for exact numbers.
const FEATURE: &str = "exact-numbers";

/// The report rule naming each value whose numbers the feature changes.
const RULE: &str = "rust_exact_numbers";

/// Two events with text fields only: the model beyond10x/ess#483 reported.
const STRINGS: &str = r"format: ess/22
system: catalog
version: v1
domain: catalog.items
events:
  - name: catalog.items.ItemAdded
    fields:
      - {name: id, type: String}
      - {name: label, type: String}
  - name: catalog.items.ItemRemoved
    fields:
      - {name: id, type: String}
";

/// An unbounded `Integer`, which the Rust target realizes as `serde_json::Number`.
const COUNTED: &str = r"format: ess/22
system: catalog
version: v1
domain: catalog.items
events:
  - name: catalog.items.ItemCounted
    fields:
      - {name: id, type: String}
      - {name: count, type: Integer}
";

/// A `Binary64`, which the Rust target realizes through a raw token, not `serde_json::Number`.
const MEASURED: &str = r"format: ess/22
system: catalog
version: v1
domain: catalog.items
events:
  - name: catalog.items.ItemMeasured
    fields:
      - {name: id, type: String}
      - {name: ratio, type: Binary64}
";

fn model_plan(source: &str, roots: &[&str]) -> Plan {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, source.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let roots = roots
        .iter()
        .map(|root| (*root).to_owned())
        .collect::<BTreeSet<_>>();
    Plan::from_model(&ModelTypes::select(&ir, &roots).unwrap()).unwrap()
}

fn bundle_plan(schemas: &Value, roots: &[&str]) -> Plan {
    let roots = roots
        .iter()
        .map(|root| (*root).to_owned())
        .collect::<BTreeSet<_>>();
    let bundle = import(
        &json!({"components": {"schemas": schemas}}).to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    Plan::from_bundle(&bundle, &roots).unwrap()
}

/// The non-empty body lines of one `[name]` section of a generated manifest, in order.
fn section<'a>(manifest: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    manifest
        .lines()
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty())
        .collect()
}

fn serde_json_dependency(manifest: &str) -> &str {
    section(manifest, "dependencies")
        .into_iter()
        .find(|line| line.starts_with("serde_json "))
        .unwrap_or_else(|| panic!("no serde_json dependency:\n{manifest}"))
}

fn exact_number_findings(realization: &Realization) -> Vec<Value> {
    serde_json::to_value(&realization.report).unwrap()["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == RULE)
        .cloned()
        .collect()
}

fn pointers(findings: &[Value]) -> BTreeSet<&str> {
    findings
        .iter()
        .map(|finding| finding["pointer"].as_str().unwrap())
        .collect()
}

#[test]
fn a_model_holding_no_json_number_gets_no_arbitrary_precision() {
    let rust = model_plan(
        STRINGS,
        &["catalog.items.ItemAdded", "catalog.items.ItemRemoved"],
    )
    .rust("catalog_types")
    .unwrap();
    assert!(
        !rust.declarations.contains("serde_json"),
        "the case is a model whose types never name serde_json:\n{}",
        rust.declarations
    );
    let manifest = &rust.supporting["Cargo.toml"];
    assert!(!manifest.contains("arbitrary_precision"), "{manifest}");
    assert!(!manifest.contains("[features]"), "{manifest}");
    assert_eq!(exact_number_findings(&rust), Vec::<Value>::new());
}

#[test]
fn a_model_holding_a_json_number_gets_a_default_on_exact_numbers_feature() {
    let rust = model_plan(COUNTED, &["catalog.items.ItemCounted"])
        .rust("catalog_types")
        .unwrap();
    assert!(rust.declarations.contains("::serde_json::Number"));
    let manifest = &rust.supporting["Cargo.toml"];
    assert_eq!(
        section(manifest, "features"),
        [
            format!("default = [\"{FEATURE}\"]"),
            format!("{FEATURE} = [\"serde_json/arbitrary_precision\"]"),
        ],
        "{manifest}"
    );
    assert!(
        !serde_json_dependency(manifest).contains("arbitrary_precision"),
        "{manifest}"
    );
    assert_eq!(
        manifest.matches("arbitrary_precision").count(),
        1,
        "{manifest}"
    );
}

#[test]
fn the_types_report_names_each_value_the_feature_changes() {
    let rust = model_plan(COUNTED, &["catalog.items.ItemCounted"])
        .rust("catalog_types")
        .unwrap();
    let findings = exact_number_findings(&rust);
    assert_eq!(
        pointers(&findings),
        BTreeSet::from(["/$defs/catalog.items.ItemCounted/properties/count"]),
        "{findings:?}"
    );
    for finding in &findings {
        let detail = finding["detail"].as_str().unwrap();
        assert!(
            detail.contains(FEATURE) && detail.contains("binary64"),
            "{detail}"
        );
    }
}

#[test]
fn a_binary64_model_keeps_raw_value_and_gets_no_exact_numbers_feature() {
    let rust = model_plan(MEASURED, &["catalog.items.ItemMeasured"])
        .rust("catalog_types")
        .unwrap();
    assert!(rust.declarations.contains("EssBinary64"));
    assert!(!rust.declarations.contains("::serde_json::Number"));
    let manifest = &rust.supporting["Cargo.toml"];
    assert!(
        serde_json_dependency(manifest).contains("features = [\"raw_value\"]"),
        "{manifest}"
    );
    assert!(!manifest.contains("arbitrary_precision"), "{manifest}");
    assert!(!manifest.contains("[features]"), "{manifest}");
    assert_eq!(exact_number_findings(&rust), Vec::<Value>::new());
}

/// A `serde_json::Value` holds numbers too: an unrestricted value, an open record's extra values,
/// and a union's alternatives, which the Rust target selects by decoding through one.
#[test]
fn json_values_and_value_decoded_unions_also_get_the_feature_and_are_named() {
    let rust = bundle_plan(
        &json!({
            "Holder": {"type": "object", "additionalProperties": false, "required": ["payload"],
                "properties": {"payload": true}},
            "Open": {"type": "object", "required": ["id"], "properties": {"id": {"type": "string"}}},
            "Either": {"anyOf": [{"type": "string"}, {"type": "boolean"}]}
        }),
        &["Holder", "Open", "Either"],
    )
    .rust("value_types")
    .unwrap();
    let manifest = &rust.supporting["Cargo.toml"];
    assert_eq!(
        section(manifest, "features"),
        [
            format!("default = [\"{FEATURE}\"]"),
            format!("{FEATURE} = [\"serde_json/arbitrary_precision\"]"),
        ],
        "{manifest}"
    );
    let findings = exact_number_findings(&rust);
    assert_eq!(
        pointers(&findings),
        BTreeSet::from([
            "/components/schemas/Either",
            "/components/schemas/Holder/properties/payload",
            "/components/schemas/Open",
        ]),
        "{findings:?}"
    );
}

/// Builds the generated crate with default features and without them. The exact bytes come only
/// from the default feature; turning it off realizes the numbers as binary64.
#[test]
fn turning_default_features_off_realizes_numbers_as_binary64() {
    let rust = bundle_plan(
        &json!({"Pair": {"type": "object", "additionalProperties": false, "required": ["a", "b"],
            "properties": {"a": {"type": "number"}, "b": {"type": "number"}}}}),
        &["Pair"],
    )
    .rust("pair_types")
    .unwrap();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("exact-numbers-{}", std::process::id()));
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("Cargo.toml"), &rust.supporting["Cargo.toml"]).unwrap();
    fs::write(root.join("types.rs"), &rust.declarations).unwrap();
    fs::write(
        root.join("tests/bytes.rs"),
        r##"
#[test]
fn a_decoded_pair_serializes_to_the_expected_bytes() {
    let pair: pair_types::Pair = serde_json::from_str(r#"{"a":1.50,"b":1e2}"#).unwrap();
    let bytes = String::from_utf8(serde_json::to_vec(&pair).unwrap()).unwrap();
    assert_eq!(bytes, std::env::var("ESS_EXPECTED_BYTES").unwrap());
}
"##,
    )
    .unwrap();
    for (arguments, expected) in [
        (&[][..], r#"{"a":1.50,"b":1e+2}"#),
        (&["--no-default-features"][..], r#"{"a":1.5,"b":100.0}"#),
    ] {
        let output = Command::new(env!("CARGO"))
            .args(["test", "--offline", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .args(arguments)
            .env("ESS_EXPECTED_BYTES", expected)
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("exact-numbers-target"),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{arguments:?} expected {expected}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
