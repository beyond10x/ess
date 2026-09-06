//! Standalone finite-value codecs retain original numeric tokens through native containers.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;
#[path = "support/model.rs"]
mod old_model;

#[path = "support/generator_version.rs"]
mod generator_version;

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
    // Retain actual producer bytes before applying the test-only version projection.
    let capture =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("non-binary64-legacy-current");
    std::fs::create_dir_all(&capture).unwrap();
    std::fs::write(
        capture.join("raw-maps.json"),
        serde_json::to_vec(&maps).unwrap(),
    )
    .unwrap();
    std::fs::write(
        capture.join("generator-version.txt"),
        env!("CARGO_PKG_VERSION"),
    )
    .unwrap();
    maps_at_baseline(&mut maps, env!("CARGO_PKG_VERSION"));
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

fn declaration_at_baseline(source: &str, language: &str, current: &str) -> Result<String, String> {
    let header = |version: &str| match language {
        "rust" => format!("// @generated by ESS {version}; do not edit.\n"),
        "go" => format!("// Code generated by ESS {version}; DO NOT EDIT.\n"),
        _ => panic!("unrecognized frozen target"),
    };
    let rest = source.strip_prefix(&header(current)).ok_or_else(|| {
        "declaration producer header does not match the current generator".to_owned()
    })?;
    Ok(format!(
        "{}{rest}",
        header(generator_version::BASELINE_VERSION)
    ))
}

#[test]
fn declaration_projection_requires_the_current_first_line_and_preserves_the_body() {
    for (language, old_header, current_header) in [
        (
            "rust",
            "// @generated by ESS 0.19.0; do not edit.\n",
            "// @generated by ESS 0.20.0; do not edit.\n",
        ),
        (
            "go",
            "// Code generated by ESS 0.19.0; DO NOT EDIT.\n",
            "// Code generated by ESS 0.20.0; DO NOT EDIT.\n",
        ),
    ] {
        let body = "// body literal 0.20.0 stays unchanged\n";
        let current = format!("{current_header}{body}");
        assert_eq!(
            declaration_at_baseline(&current, language, "0.20.0").unwrap(),
            format!("{old_header}{body}")
        );
        assert!(
            declaration_at_baseline(&format!("{old_header}{body}"), language, "0.20.0").is_err()
        );
        assert!(declaration_at_baseline(&format!(" {current}"), language, "0.20.0").is_err());
        assert_ne!(
            declaration_at_baseline(
                &current.replace("body literal", "changed body"),
                language,
                "0.20.0"
            )
            .unwrap(),
            format!("{old_header}{body}")
        );
    }
}

fn maps_at_baseline(maps: &mut BTreeMap<String, BTreeMap<String, String>>, current_version: &str) {
    for (name, files) in maps {
        let language = name.rsplit('/').next().unwrap();
        let declaration = if language == "rust" {
            "types.rs"
        } else {
            "types.go"
        };
        let current = files.get_mut(declaration).unwrap();
        *current = declaration_at_baseline(current, language, current_version).unwrap();
        let report = files.get_mut("types-report.json").unwrap();
        *report = generator_version::report_at_baseline(report, current_version).unwrap();
    }
}

#[test]
fn complete_map_projection_keeps_supporting_bytes_and_membership() {
    let files = BTreeMap::from([
        (
            "types.go".to_owned(),
            "// Code generated by ESS 0.20.0; DO NOT EDIT.\npackage old_types\n".to_owned(),
        ),
        (
            "types-report.json".to_owned(),
            "{\n  \"generator_version\": \"0.20.0\",\n  \"format\": \"ess-types-report/3\"\n}\n"
                .to_owned(),
        ),
        (
            "go.mod".to_owned(),
            "module example.invalid/oldtypes\n".to_owned(),
        ),
    ]);
    let current = BTreeMap::from([("bundle/go".to_owned(), files)]);
    let mut expected = current.clone();
    maps_at_baseline(&mut expected, "0.20.0");
    for mutation in 0..3 {
        let mut changed = current.clone();
        let files = changed.get_mut("bundle/go").unwrap();
        match mutation {
            0 => {
                files.get_mut("go.mod").unwrap().push('\n');
            }
            1 => {
                files.insert("added.txt".to_owned(), "extra".to_owned());
            }
            _ => {
                files.remove("go.mod");
            }
        }
        maps_at_baseline(&mut changed, "0.20.0");
        assert_ne!(
            serde_json::to_vec(&changed).unwrap(),
            serde_json::to_vec(&expected).unwrap()
        );
    }
}
