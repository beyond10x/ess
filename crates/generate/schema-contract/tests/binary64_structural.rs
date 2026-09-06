//! Standalone finite-value codecs retain original numeric tokens through native containers.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;
#[path = "support/model.rs"]
mod old_model;

use schema_contract::{
    bundle::{import, Dialect},
    realize::{Plan, Realization},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const SOURCE: &str = r"format: ess/2
system: sample
version: v1
domains: [sample.float]
domain: sample.float
types:
  - {name: sample.float.Scalar, kind: newtype, of: Binary64}
  - {name: sample.float.Alias, kind: newtype, of: sample.float.Scalar}
  - {name: sample.float.Items, kind: newtype, of: 'List<Optional<Binary64>>'}
  - {name: sample.float.Dictionary, kind: newtype, of: 'Map<String, Binary64>'}
  - {name: sample.float.MapAlias, kind: newtype, of: sample.float.Dictionary}
  - {name: sample.float.Maps, kind: newtype, of: 'Map<String, Map<String, Binary64>>'}
  - {name: sample.float.ListMap, kind: newtype, of: 'Map<String, List<Binary64>>'}
  - {name: sample.float.NullableMap, kind: newtype, of: 'Map<String, Optional<Binary64>>'}
  - name: sample.float.Choice
    kind: union
    tag: value
    variants:
      number: sample.float.Scalar
      text: String
  - {name: sample.float.ChoiceMap, kind: newtype, of: 'Map<String, sample.float.Choice>'}
  - name: sample.float.Record
    kind: struct
    fields:
      - {name: required, type: Binary64}
      - {name: optional, type: 'Optional<Binary64>'}
      - {name: dictionary, type: sample.float.Dictionary}
      - {name: children, type: 'List<sample.float.Record>'}
  - name: sample.float.Nested
    kind: union
    tag: value
    variants:
      record: sample.float.Record
      dictionary: sample.float.Dictionary
  - {name: sample.float.Plain, kind: newtype, of: Decimal}
";

fn plan() -> Plan {
    let roots = [
        "Scalar",
        "Alias",
        "Items",
        "Dictionary",
        "MapAlias",
        "Maps",
        "ListMap",
        "NullableMap",
        "Choice",
        "ChoiceMap",
        "Record",
        "Nested",
        "Plain",
    ]
    .map(|name| format!("sample.float.{name}"));
    Plan::from_model(&model::selection(
        SOURCE,
        &roots.iter().map(String::as_str).collect::<Vec<_>>(),
    ))
    .unwrap()
}

#[test]
fn all_selected_finite_model_types_emit_both_native_libraries() {
    let plan = plan();
    let rust = plan
        .rust("binary64_types")
        .expect("finite model Rust codec");
    let go = plan
        .go("binary64_types", "example.invalid/binary64types")
        .expect("finite model Go codec");
    for result in [rust, go] {
        assert!(result.declarations.contains("EssBinary64"));
        assert!(!json!(result.report)["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "model_binary64"));
    }
}

fn map(result: Realization, extension: &str) -> BTreeMap<String, String> {
    let mut files = result.supporting;
    files.insert(format!("types.{extension}"), result.declarations);
    files.insert(
        "types-report.json".into(),
        serde_json::to_string_pretty(&result.report).unwrap() + "\n",
    );
    files
}

#[test]
fn complete_non_binary64_output_maps_remain_identical() {
    let roots = BTreeSet::from(["Plain".to_owned(), "EssBinary64".to_owned()]);
    let bundle = import(&json!({"components":{"schemas":{
        "Plain":{"type":"object","properties":{"number":{"type":"number"},"choice":{"anyOf":[{"type":"number"},{"type":"string"}]}},"additionalProperties":{"type":"number"}},
        "EssBinary64":{"type":"string"}
    }}}).to_string(), &roots, Dialect::Draft202012).unwrap();
    let imported = Plan::from_bundle(&bundle, &roots).unwrap();
    let mut maps = BTreeMap::new();
    for (name, plan) in [("model", old_model::plan()), ("bundle", imported)] {
        maps.insert(
            format!("{name}/rust"),
            map(plan.rust("old_types").unwrap(), "rs"),
        );
        maps.insert(
            format!("{name}/go"),
            map(
                plan.go("old_types", "example.invalid/oldtypes").unwrap(),
                "go",
            ),
        );
    }
    let digest = Sha256::digest(serde_json::to_vec(&maps).unwrap())
        .iter()
        .fold(String::new(), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        });
    assert_eq!(
        digest, "02312f45fadac5e16b54aa1fce13d8f68cd8aaf14336a2180b85cd923811c79f",
        "complete old output map SHA-256 frozen at c4ba992"
    );
}

#[test]
fn rust_original_token_wire_corpus() {
    let output = plan().rust("binary64_types").unwrap();
    let root = native_root("rust");
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(root.join("Cargo.toml"), &output.supporting["Cargo.toml"]).unwrap();
    std::fs::write(root.join("types.rs"), &output.declarations).unwrap();
    std::fs::write(
        root.join("tests/wire.rs"),
        include_str!("fixtures/binary64_wire_tests.rs.txt"),
    )
    .unwrap();
    for args in [
        vec!["generate-lockfile", "--offline"],
        vec!["test", "--offline", "--locked"],
    ] {
        let result = std::process::Command::new(env!("CARGO"))
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        println!(
            "native Rust stdout:\n{}\nnative Rust stderr:\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            result.status.success(),
            "native Rust exit: {}",
            result.status
        );
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn go_original_token_wire_corpus() {
    let compiler = std::path::PathBuf::from(
        std::env::var_os("ESS_GO_COMPILER").expect("go-typecheck requires ESS_GO_COMPILER"),
    );
    assert!(compiler.is_file());
    let version = std::process::Command::new(&compiler)
        .arg("version")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout)
            .split_whitespace()
            .nth(2)
            .unwrap()
            .split('-')
            .next(),
        Some("go1.26.5")
    );
    let output = plan()
        .go("binary64_types", "example.invalid/binary64types")
        .unwrap();
    let root = native_root("go");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("go.mod"), &output.supporting["go.mod"]).unwrap();
    std::fs::write(root.join("types.go"), &output.declarations).unwrap();
    std::fs::write(
        root.join("wire_test.go"),
        include_str!("fixtures/binary64_wire_tests.go.txt"),
    )
    .unwrap();
    let result = std::process::Command::new(compiler)
        .args(["test", "-count=1", "-v", "./..."])
        .current_dir(&root)
        .env("GOTOOLCHAIN", "local")
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOWORK", "off")
        .env("GOFLAGS", "")
        .output()
        .unwrap();
    println!(
        "native Go stdout:\n{}\nnative Go stderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.status.success(), "native Go exit: {}", result.status);
}

fn native_root(language: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("binary64-{language}-{}", std::process::id()))
}

#[test]
fn excluded_float_selection_preserves_plain_representation_and_reservations() {
    let selected = model::selection(SOURCE, &["sample.float.Plain"]);
    let plan = Plan::from_model(&selected).unwrap();
    let rust = plan.rust("plain_types").unwrap();
    let go = plan
        .go("plain_types", "example.invalid/plaintypes")
        .unwrap();
    assert!(!rust.declarations.contains("EssBinary64"));
    assert!(!rust.supporting["Cargo.toml"].contains("raw_value"));
    assert!(!go.declarations.contains("EssBinary64"));
    let report = json!(self::plan().typescript().report);
    assert!(report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["rule"] == "model_binary64"));
}
