//! Adversary pass 1 on beyond10x/ess#483: the generated Rust manifest's default-on
//! `exact-numbers` feature, attacked through every place a JSON number can sit.
//!
//! The failure that matters is a model whose types hold a JSON number in `serde_json` while the
//! manifest declares no feature, so the default build silently realizes it as binary64. The
//! cases below drive detection through map values, list items, tuple positions, optional and
//! nullable values, unions decoded through a `Value` or a raw token, recursive references, open
//! records, event roots and type roots, and then build the generated crates with default features
//! and through a consumer that turns them off.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
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

const FEATURE_SECTION: [&str; 2] = [
    "default = [\"exact-numbers\"]",
    "exact-numbers = [\"serde_json/arbitrary_precision\"]",
];

/// Every model position a number can sit in, one type each, plus a raw-token union (a Binary64
/// alternative beside an Integer one) and a union whose alternatives hold no number at all.
const MODEL: &str = r"format: ess/22
system: probe
version: v1
domain: probe.n
types:
  - {name: probe.n.Count, kind: newtype, of: Integer}
  - name: probe.n.Bounded
    kind: newtype
    of: Integer
    invariants:
      - value >= 0
      - value <= 10
  - name: probe.n.Choice
    kind: union
    tag: kind
    variants:
      count: Integer
      text: String
  - name: probe.n.Mixed
    kind: union
    tag: kind
    variants:
      ratio: Binary64
      count: Integer
  - name: probe.n.Words
    kind: union
    tag: kind
    variants:
      left: String
      right: Boolean
  - name: probe.n.Holder
    kind: struct
    fields:
      - {name: counts, type: 'Map<String, Integer>'}
      - {name: list, type: 'List<Optional<Integer>>'}
      - {name: choice, type: probe.n.Choice}
      - {name: mixed, type: probe.n.Mixed}
      - {name: payload, type: Json}
      - {name: bounded, type: probe.n.Bounded}
      - {name: maybe, type: Optional<Integer>}
events:
  - name: probe.n.Counted
    fields:
      - {name: count, type: probe.n.Count}
  - name: probe.n.Named
    fields:
      - {name: label, type: String}
      - {name: words, type: probe.n.Words}
";

const ALL_MODEL_ROOTS: [&str; 8] = [
    "probe.n.Count",
    "probe.n.Bounded",
    "probe.n.Choice",
    "probe.n.Mixed",
    "probe.n.Words",
    "probe.n.Holder",
    "probe.n.Counted",
    "probe.n.Named",
];

fn model_plan(roots: &[&str]) -> Plan {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, MODEL.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(MODEL).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let roots = roots
        .iter()
        .map(|root| (*root).to_owned())
        .collect::<BTreeSet<_>>();
    Plan::from_model(&ModelTypes::select(&ir, &roots).unwrap()).unwrap()
}

/// Every bundle position a number can sit in, in one closed record, plus a recursive reference.
fn positions() -> Value {
    json!({
        "Positions": {
            "type": "object",
            "additionalProperties": false,
            "required": ["either", "json", "list", "map", "maybe", "multi", "nullable", "open",
                "plain", "tree", "tuple"],
            "properties": {
                "either": {"anyOf": [{"type": "string"}, {"type": "number"}]},
                "json": true,
                "list": {"type": "array", "items": {"type": "integer"}},
                "map": {"type": "object", "additionalProperties": {"type": "number"}},
                "maybe": {"type": ["number", "null"]},
                "multi": {"type": ["integer", "string"]},
                "nullable": {"anyOf": [{"type": "number"}, {"type": "null"}]},
                "open": {"type": "object", "required": ["id"],
                    "properties": {"id": {"type": "string"}}},
                "plain": {"type": "string"},
                "tree": {"$ref": "#/components/schemas/Tree"},
                "tuple": {"type": "array", "prefixItems": [{"type": "string"}, {"type": "number"}],
                    "minItems": 2, "maxItems": 2}
            }
        },
        "Tree": {
            "type": "object",
            "additionalProperties": false,
            "required": ["kids", "value"],
            "properties": {
                "kids": {"type": "array", "items": {"$ref": "#/components/schemas/Tree"}},
                "value": {"type": "number"}
            }
        }
    })
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

/// The non-empty body lines of one `[name]` section of a generated manifest.
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

fn reported(realization: &Realization) -> BTreeSet<String> {
    serde_json::to_value(&realization.report).unwrap()["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "rust_exact_numbers")
        .map(|finding| finding["pointer"].as_str().unwrap().to_owned())
        .collect()
}

/// The `Value` a union decodes through is not itself a held number: whether that union holds one
/// depends on its alternatives, and none of the selections here holds a union whose only number is
/// a native-width integer (that case is `types_manifest_features.rs`).
fn holds_serde_json_number(declarations: &str) -> bool {
    declarations.contains("::serde_json::Number")
        || declarations
            .replace(
                "<::serde_json::Value as ::serde::Deserialize>::deserialize",
                "",
            )
            .contains("::serde_json::Value")
}

fn declares_feature(realization: &Realization) -> bool {
    let manifest = &realization.supporting["Cargo.toml"];
    let features = section(manifest, "features");
    assert!(
        features.is_empty() || features == FEATURE_SECTION,
        "unexpected [features]:\n{manifest}"
    );
    let dependency = section(manifest, "dependencies")
        .into_iter()
        .find(|line| line.starts_with("serde_json "))
        .unwrap();
    assert!(!dependency.contains("arbitrary_precision"), "{manifest}");
    !features.is_empty()
}

/// Every bundle position holding a JSON number in `serde_json` is named, by the pointer of the
/// value itself, and nothing else is: the string field, the tuple's unused `items` and the
/// references are not.
#[test]
fn every_bundle_number_position_is_named_by_its_own_pointer() {
    let rust = bundle_plan(&positions(), &["Positions"])
        .rust("positions_types")
        .unwrap();
    assert!(declares_feature(&rust));
    let at = |tail: &str| format!("/components/schemas/{tail}");
    assert_eq!(
        reported(&rust),
        BTreeSet::from([
            at("Positions/properties/either"),
            at("Positions/properties/either/anyOf/1"),
            at("Positions/properties/json"),
            at("Positions/properties/list/items"),
            at("Positions/properties/map/additionalProperties"),
            at("Positions/properties/maybe"),
            at("Positions/properties/multi"),
            at("Positions/properties/nullable/anyOf/0"),
            at("Positions/properties/open"),
            at("Positions/properties/tuple/prefixItems/1"),
            at("Tree/properties/value"),
        ])
    );
}

/// Every model position holding a JSON number in `serde_json` is named: a newtype reached as a
/// type root, a map value, an optional list item, an optional field, a `Json` field, and the
/// Integer alternative of both a `Value`-decoded and a raw-token union. The bounded integer
/// (`i32`) and the Binary64 alternative hold none and are not named.
#[test]
fn every_model_number_position_is_named_and_native_widths_are_not() {
    let rust = model_plan(&ALL_MODEL_ROOTS).rust("probe_types").unwrap();
    assert!(declares_feature(&rust));
    let named = reported(&rust);
    let at = |tail: &str| format!("/$defs/probe.n.{tail}");
    for required in [
        at("Count"),
        at("Holder/properties/counts/additionalProperties"),
        at("Holder/properties/list/items/anyOf/0"),
        at("Holder/properties/maybe"),
        at("Holder/properties/payload"),
        at("Choice/oneOf/0/properties/value"),
        at("Mixed/oneOf/0/properties/value"),
    ] {
        assert!(named.contains(&required), "{required} not in {named:?}");
    }
    for absent in [
        at("Bounded"),
        at("Holder/properties/bounded"),
        at("Mixed"),
        at("Mixed/oneOf/1/properties/value"),
    ] {
        assert!(!named.contains(&absent), "{absent} in {named:?}");
    }
}

/// A number reached only through an event root is detected as one reached through a type root,
/// and a root selecting only a native-width integer gets no feature.
#[test]
fn event_roots_and_type_roots_reach_the_same_detection() {
    let event = model_plan(&["probe.n.Counted"])
        .rust("probe_types")
        .unwrap();
    assert!(declares_feature(&event));
    assert_eq!(
        reported(&event),
        BTreeSet::from(["/$defs/probe.n.Count".to_owned()])
    );
    let bounded = model_plan(&["probe.n.Bounded"])
        .rust("probe_types")
        .unwrap();
    assert!(!declares_feature(&bounded));
    assert_eq!(reported(&bounded), BTreeSet::new());
}

/// Over every selection here, the manifest declares the feature exactly when the emitted
/// declarations hold a JSON number in `serde_json`.
#[test]
fn the_feature_is_declared_exactly_when_the_declarations_name_serde_json_numbers() {
    let mut realizations = vec![(
        "bundle Positions".to_owned(),
        bundle_plan(&positions(), &["Positions"])
            .rust("positions_types")
            .unwrap(),
    )];
    let mut selections = ALL_MODEL_ROOTS
        .iter()
        .map(|root| vec![*root])
        .collect::<Vec<_>>();
    selections.push(ALL_MODEL_ROOTS.to_vec());
    for roots in selections {
        realizations.push((
            format!("model {roots:?}"),
            model_plan(&roots).rust("probe_types").unwrap(),
        ));
    }
    for (label, rust) in &realizations {
        assert_eq!(
            declares_feature(rust),
            holds_serde_json_number(&rust.declarations),
            "{label}:\n{}\n{}",
            rust.supporting["Cargo.toml"],
            rust.declarations
        );
        assert_eq!(
            reported(rust).is_empty(),
            !declares_feature(rust),
            "{label}"
        );
    }
}

/// The report's own words for `rust_exact_numbers` are "this value holds JSON numbers in
/// `serde_json`". An event whose fields are a string and a tagged union of a string and a boolean
/// holds no JSON number anywhere: no alternative can decode one, so `arbitrary_precision` changes
/// nothing it can produce. It is the issue's own case (a text-only event library) with a union in
/// it, and it should get no feature and no `rust_exact_numbers` entry.
#[test]
fn a_union_whose_alternatives_hold_no_number_holds_no_exact_number() {
    let rust = model_plan(&["probe.n.Named"]).rust("probe_types").unwrap();
    assert!(!rust.declarations.contains("::serde_json::Number"));
    assert_eq!(
        reported(&rust),
        BTreeSet::new(),
        "named as holding JSON numbers although no alternative holds one"
    );
    assert!(
        !declares_feature(&rust),
        "{}",
        rust.supporting["Cargo.toml"]
    );
}

fn scratch(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-exact-numbers-{name}-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

/// Writes the generated library under `root/lib` and a consumer under `root/<mode>` that depends
/// on it with or without default features, then runs the consumer's test.
fn run_consumer(root: &Path, rust: &Realization, package: &str, test: &str, exact: bool) {
    let lib = root.join("lib");
    fs::create_dir_all(&lib).unwrap();
    fs::write(lib.join("Cargo.toml"), &rust.supporting["Cargo.toml"]).unwrap();
    fs::write(lib.join("types.rs"), &rust.declarations).unwrap();
    let mode = if exact { "default" } else { "off" };
    let consumer = root.join(mode);
    fs::create_dir_all(consumer.join("tests")).unwrap();
    let default_features = if exact {
        ""
    } else {
        ", default-features = false"
    };
    fs::write(
        consumer.join("Cargo.toml"),
        format!(
            "[package]\nname = \"consumer-{mode}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\n{package} = {{ path = \"../lib\"{default_features} }}\nserde_json = \"=1.0.151\"\n\n[workspace]\n"
        ),
    )
    .unwrap();
    fs::create_dir_all(consumer.join("src")).unwrap();
    fs::write(consumer.join("src/lib.rs"), "").unwrap();
    fs::write(consumer.join("tests/bytes.rs"), test).unwrap();
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(consumer.join("Cargo.toml"))
        .env("ESS_EXPECT_EXACT", if exact { "1" } else { "0" })
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary-exact-numbers-target"),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode} consumer of {package}:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const POSITIONS_TEST: &str = r##"
const INPUT: &str = r#"{"either":2.20,"json":{"x":3.30},"list":[40000000000000000000001],"map":{"k":5.50},"maybe":6.60,"multi":12345678901234567890123,"nullable":7.70,"open":{"id":"i","z":1.10},"plain":"p","tree":{"kids":[{"kids":[],"value":8.80}],"value":9.90},"tuple":["t",1.10]}"#;

#[test]
fn every_position_round_trips() {
    let decoded: positions_types::Positions = serde_json::from_str(INPUT).unwrap();
    let bytes = serde_json::to_string(&decoded).unwrap();
    if std::env::var("ESS_EXPECT_EXACT").unwrap() == "1" {
        assert_eq!(bytes, INPUT, "the default build keeps every number exact");
    } else {
        assert_ne!(bytes, INPUT);
        assert!(bytes.contains("\"either\":2.2,"), "{bytes}");
    }
}
"##;

const HOLDER_TEST: &str = r##"
const INPUT: &str = r#"{"bounded":3,"choice":{"kind":"count","value":1.10},"counts":{"a":2.20},"list":[3.30,null],"maybe":4.40,"mixed":{"kind":"count","value":12345678901234567890123},"payload":{"x":6.60}}"#;

#[test]
fn every_position_round_trips() {
    let decoded: probe_types::ProbeNHolder = serde_json::from_str(INPUT).unwrap();
    let bytes = serde_json::to_string(&decoded).unwrap();
    if std::env::var("ESS_EXPECT_EXACT").unwrap() == "1" {
        assert_eq!(bytes, INPUT, "the default build keeps every number exact");
    } else {
        assert_ne!(bytes, INPUT);
        assert!(bytes.contains("\"a\":2.2}"), "{bytes}");
    }
}
"##;

/// The default build keeps every number exact at every bundle position; a consumer that turns
/// default features off compiles and gets binary64.
#[test]
fn a_bundle_crate_is_exact_by_default_and_compiles_without_default_features() {
    let rust = bundle_plan(&positions(), &["Positions"])
        .rust("positions_types")
        .unwrap();
    let root = scratch("positions");
    for exact in [true, false] {
        run_consumer(&root, &rust, "positions_types", POSITIONS_TEST, exact);
    }
}

/// The same for a model crate holding a raw-token union (Binary64 beside Integer), a
/// `Value`-decoded union, a map, an optional list item, an optional field and `Json`.
#[test]
fn a_model_crate_is_exact_by_default_and_compiles_without_default_features() {
    let rust = model_plan(&ALL_MODEL_ROOTS).rust("probe_types").unwrap();
    let root = scratch("model");
    for exact in [true, false] {
        run_consumer(&root, &rust, "probe_types", HOLDER_TEST, exact);
    }
}
