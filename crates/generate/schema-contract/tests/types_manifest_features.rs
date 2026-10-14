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

/// A tagged union whose numeric alternative is a bounded `Integer`, realized as `i32`.
const SIZED: &str = r"format: ess/22
system: catalog
version: v1
domain: catalog.items
types:
  - name: catalog.items.Small
    kind: newtype
    of: Integer
    invariants:
      - value >= 0
      - value <= 10
  - name: catalog.items.Amount
    kind: union
    tag: kind
    variants:
      count: catalog.items.Small
      text: String
events:
  - name: catalog.items.AmountRecorded
    fields:
      - {name: amount, type: catalog.items.Amount}
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
/// and a union the Rust target decodes through one when an alternative, at any depth, can hold a
/// number. A union of text and booleans holds none and is not named.
#[test]
fn json_values_and_value_decoded_unions_also_get_the_feature_and_are_named() {
    let rust = bundle_plan(
        &json!({
            "Holder": {"type": "object", "additionalProperties": false, "required": ["payload"],
                "properties": {"payload": true}},
            "Open": {"type": "object", "required": ["id"], "properties": {"id": {"type": "string"}}},
            "Either": {"anyOf": [{"type": "string"}, {"type": "boolean"}]},
            "Counted": {"anyOf": [{"type": "string"}, {"type": "object",
                "additionalProperties": false, "required": ["n"],
                "properties": {"n": {"type": "integer"}}}]}
        }),
        &["Holder", "Open", "Either", "Counted"],
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
            "/components/schemas/Counted",
            "/components/schemas/Counted/anyOf/1/properties/n",
            "/components/schemas/Holder/properties/payload",
            "/components/schemas/Open",
        ]),
        "{findings:?}"
    );
}

/// A union whose only number is a native-width integer, reached through a reference. Through a
/// `Value` that integer decodes differently with and without `arbitrary_precision` (`-0` is
/// accepted only with it), so the union is named and the feature declared; the `i32` is not named.
#[test]
fn a_union_holding_only_a_native_width_integer_is_named() {
    let rust = model_plan(SIZED, &["catalog.items.AmountRecorded"])
        .rust("catalog_types")
        .unwrap();
    assert!(rust.declarations.contains("(pub i32);"));
    assert!(!rust.declarations.contains("::serde_json::Number"));
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
        BTreeSet::from(["/$defs/catalog.items.Amount"]),
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

/// `--rust-raw-json` (beyond10x/ess#577): a `Json` value kept as its exact JSON text, so its bytes
/// round-trip whatever `serde_json` features a consumer's build unifies.
mod raw_json {
    use super::*;

    /// The report rule naming each value the Rust target keeps as raw JSON text.
    const RAW_RULE: &str = "rust_raw_json";

    /// The float beyond10x/ess#577 reported: best-effort parsing reads it back one unit lower.
    const INPUT: &str = r#"{"v":1.0715660391465826e-75}"#;

    const SAMPLE: &str = r"format: ess/22
system: catalog
version: v1
domain: catalog.items
types:
  - name: catalog.items.Sample
    kind: struct
    fields:
      - {name: v, type: Json}
";

    const RECORDS: &str = r"format: ess/22
system: catalog
version: v1
domain: catalog.items
types:
  - name: catalog.items.Document
    kind: newtype
    of: Json
  - name: catalog.items.Plain
    kind: struct
    fields:
      - {name: label, type: String}
  - name: catalog.items.Record
    kind: struct
    fields:
      - {name: v, type: Json}
      - {name: extra, type: Optional<Json>}
      - {name: body_text, type: Json, wire: bodyText}
      - {name: note, type: String}
      - {name: count, type: Integer}
      - {name: doc, type: catalog.items.Document}
      - {name: plain, type: catalog.items.Plain}
";

    const ROOTS: &[&str] = &["catalog.items.Record"];

    fn raw_pointers(realization: &Realization) -> BTreeSet<String> {
        serde_json::to_value(&realization.report).unwrap()["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["rule"] == RAW_RULE)
            .map(|finding| finding["pointer"].as_str().unwrap().to_owned())
            .collect()
    }

    /// Builds `rust` and decodes then encodes `INPUT` through its `Sample` with default features
    /// off, returning the bytes it serialized.
    fn round_trip_without_default_features(rust: &Realization, package: &str) -> String {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("raw-json-{package}-{}", std::process::id()));
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("Cargo.toml"), &rust.supporting["Cargo.toml"]).unwrap();
        fs::write(root.join("types.rs"), &rust.declarations).unwrap();
        let bytes = root.join("bytes.json");
        fs::write(
            root.join("tests/bytes.rs"),
            format!(
                r#"
#[test]
fn a_decoded_sample_is_serialized_again() {{
    let input = std::env::var("ESS_INPUT").unwrap();
    let sample: {package}::CatalogItemsSample = serde_json::from_str(&input).unwrap();
    std::fs::write(std::env::var("ESS_OUTPUT").unwrap(), serde_json::to_vec(&sample).unwrap()).unwrap();
}}
"#
            ),
        )
        .unwrap();
        let output = Command::new(env!("CARGO"))
            .args(["test", "--offline", "--quiet", "--no-default-features"])
            .arg("--manifest-path")
            .arg(root.join("Cargo.toml"))
            .env("ESS_INPUT", INPUT)
            .env("ESS_OUTPUT", &bytes)
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("exact-numbers-target"),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::read_to_string(bytes).unwrap()
    }

    #[test]
    fn raw_json_field_round_trips_its_bytes() {
        let raw = model_plan(SAMPLE, &["catalog.items.Sample"])
            .with_raw_json(["catalog.items.Sample.v"])
            .unwrap()
            .rust("raw_types")
            .unwrap();
        let manifest = &raw.supporting["Cargo.toml"];
        assert!(
            serde_json_dependency(manifest).contains("features = [\"raw_value\"]"),
            "{manifest}"
        );
        assert!(!manifest.contains("arbitrary_precision"), "{manifest}");
        assert_eq!(exact_number_findings(&raw), Vec::<Value>::new());
        assert_eq!(
            raw_pointers(&raw),
            BTreeSet::from(["/$defs/catalog.items.Sample/properties/v".to_owned()])
        );
        assert_eq!(
            round_trip_without_default_features(&raw, "raw_types"),
            INPUT
        );

        // The same model without the option holds the value in `serde_json::Value`, and with
        // `exact-numbers` off its bytes move: the case above is not passing by accident.
        let value = model_plan(SAMPLE, &["catalog.items.Sample"])
            .rust("value_types")
            .unwrap();
        assert_ne!(
            round_trip_without_default_features(&value, "value_types"),
            INPUT
        );
    }

    #[test]
    fn raw_json_on_a_type_covers_every_json_field_of_it() {
        let rust = model_plan(RECORDS, ROOTS)
            .with_raw_json(["catalog.items.Record", "catalog.items.Document"])
            .unwrap()
            .rust("catalog_types")
            .unwrap();
        let declarations = &rust.declarations;
        for line in [
            "pub v: EssRawJson,",
            "pub extra: EssPresence<EssRawJson>,",
            "pub body_text: EssRawJson,",
            "pub struct CatalogItemsDocument(pub EssRawJson);",
        ] {
            assert!(declarations.contains(line), "{line}\n{declarations}");
        }
        assert!(
            !declarations.contains("::serde_json::Value,"),
            "{declarations}"
        );
        let at = "/$defs/catalog.items.Record/properties";
        assert_eq!(
            raw_pointers(&rust),
            BTreeSet::from([
                "/$defs/catalog.items.Document".to_owned(),
                format!("{at}/bodyText"),
                format!("{at}/extra"),
                format!("{at}/v"),
            ])
        );
        // The raw values no longer count toward `exact-numbers`; the `Integer` still does.
        assert_eq!(
            pointers(&exact_number_findings(&rust)),
            BTreeSet::from([format!("{at}/count").as_str()])
        );

        // A field, by its declared or its wire name, covers that field alone.
        for named in [
            "catalog.items.Record.body_text",
            "catalog.items.Record.bodyText",
        ] {
            let one = model_plan(RECORDS, ROOTS)
                .with_raw_json([named])
                .unwrap()
                .rust("catalog_types")
                .unwrap();
            assert_eq!(
                raw_pointers(&one),
                BTreeSet::from([format!("{at}/bodyText")]),
                "{named}"
            );
            assert!(
                one.declarations.contains("pub v: ::serde_json::Value,"),
                "{}",
                one.declarations
            );
        }
    }

    #[test]
    fn raw_json_names_an_unknown_or_non_json_location_and_is_refused() {
        for named in [
            "catalog.items.Missing",
            "catalog.items.Record.missing",
            "catalog.items.Record.note",
            "catalog.items.Record.doc",
            "catalog.items.Plain",
            "Record",
            "",
        ] {
            let refused = model_plan(RECORDS, ROOTS)
                .with_raw_json(["catalog.items.Record.v", named])
                .expect_err(named);
            assert_eq!(refused.0.len(), 1, "{named}: {refused}");
            let finding = &refused.0[0];
            assert_eq!(finding.rule, RAW_RULE, "{named}: {refused}");
            assert!(
                finding.detail.contains(&format!("{named:?}")),
                "{named}: {refused}"
            );
        }
        // A bundle names its components the same way.
        let bundle = bundle_plan(
            &json!({"Holder": {"type": "object", "additionalProperties": false,
                "required": ["payload", "id"],
                "properties": {"payload": true, "id": {"type": "string"}}}}),
            &["Holder"],
        );
        assert!(bundle.clone().with_raw_json(["Holder.payload"]).is_ok());
        assert!(bundle.with_raw_json(["Holder.id"]).is_err());
    }

    /// Raw text decodes only from `serde_json`'s own deserializer: a union holding it is decoded
    /// through a raw token rather than a `Value`, and a record that buffers its fields for
    /// `#[serde(flatten)]` cannot hold it and is refused.
    #[test]
    fn raw_json_is_decoded_from_a_raw_token_or_refused_where_it_cannot_be() {
        let schemas = json!({
            "Holder": {"type": "object", "additionalProperties": false, "required": ["payload"],
                "properties": {"payload": true}},
            "Either": {"anyOf": [{"type": "string"},
                {"$ref": "#/components/schemas/Holder"}]},
            "Open": {"type": "object", "required": ["payload"], "properties": {"payload": true}}
        });
        let rust = bundle_plan(&schemas, &["Holder", "Either"])
            .with_raw_json(["Holder.payload"])
            .unwrap()
            .rust("raw_types")
            .unwrap();
        assert!(
            rust.declarations.contains(
                "let value = <::std::boxed::Box<::serde_json::value::RawValue> as ::serde::Deserialize>::deserialize(deserializer)?;"
            ),
            "{}",
            rust.declarations
        );
        assert_eq!(exact_number_findings(&rust), Vec::<Value>::new());

        let refused = bundle_plan(&schemas, &["Open"])
            .with_raw_json(["Open.payload"])
            .unwrap()
            .rust("raw_types")
            .unwrap_err();
        assert_eq!(
            refused
                .0
                .iter()
                .map(|finding| (finding.pointer.as_str(), finding.rule.as_str()))
                .collect::<Vec<_>>(),
            [("/components/schemas/Open", "rust_raw_json_open_record")]
        );
    }
}
