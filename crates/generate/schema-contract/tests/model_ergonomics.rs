//! Generated Go and Rust libraries a producer can write against (beyond10x/ess#406–#409): native
//! timestamps, a constant newtype that carries its value, anonymous shapes named by position, and
//! opt-in short names.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::realize::Plan;

const SOURCE: &str = r"format: ess/20
system: probe
version: v1
domains: [probe.meter]
domain: probe.meter
types:
  - name: probe.meter.Version
    kind: newtype
    of: Integer
    invariants:
      - value == 2
  - name: probe.meter.Sample
    kind: struct
    fields:
      - {name: amount, type: Integer}
  - name: probe.meter.Reading
    kind: struct
    fields:
      - {name: created_at, type: Timestamp, wire: createdAt}
      - {name: seen_at, type: 'Optional<Timestamp>', presence: omitted_when_absent}
      - {name: version, type: probe.meter.Version}
      - {name: samples, type: 'List<probe.meter.Sample>'}
";

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &["probe.meter.Reading"])).expect("plan")
}

fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).expect("scratch");
    root
}

#[test]
fn rust_holds_a_timestamp_natively_and_a_constant_newtype_carries_its_value() {
    let realization = plan().rust("probe-meter").expect("rust");
    let types = &realization.declarations;
    assert!(
        types.contains("pub struct EssTimestamp(pub ::time::OffsetDateTime);"),
        "{types}"
    );
    assert!(types.contains("pub created_at: EssTimestamp,"), "{types}");
    assert!(
        types.contains("pub struct ProbeMeterVersion(pub i32);"),
        "{types}"
    );
    assert!(types.contains("pub const VALUE: i32 = 2;"), "{types}");
    assert!(
        types.contains("impl ::std::default::Default for ProbeMeterVersion"),
        "{types}"
    );
    let manifest = &realization.supporting["Cargo.toml"];
    assert!(
        manifest.contains("time = { version = \"^0.3.45\""),
        "{manifest}"
    );

    let root = scratch("ergonomics-rust");
    fs::create_dir_all(root.join("tests")).expect("tests dir");
    fs::write(root.join("Cargo.toml"), manifest).expect("manifest");
    fs::write(root.join("types.rs"), types).expect("types");
    fs::write(
        root.join("tests/round_trip.rs"),
        r##"
use probe_meter::*;

#[test]
fn a_reading_round_trips_and_the_constant_is_there() {
    let text = r#"{"createdAt":"2026-10-03T00:00:00.5+02:00","samples":[{"amount":1}],"version":2}"#;
    let reading: ProbeMeterReading = serde_json::from_str(text).unwrap();
    assert_eq!(serde_json::to_value(&reading).unwrap(), serde_json::from_str::<serde_json::Value>(text).unwrap());
    assert_eq!(ProbeMeterVersion::default().0, 2);
    assert!(serde_json::from_str::<ProbeMeterReading>(&text.replace("2026-10-03T00:00:00.5+02:00", "yesterday")).is_err());
    // RFC 2822 and its nested comments are outside this RFC 3339-only wire contract.
    let mail_date = format!("Fri, 21 Nov 1997 09:55:06 -0600 {}comment{}", "(".repeat(1024), ")".repeat(1024));
    assert!(serde_json::from_str::<ProbeMeterReading>(&text.replace("2026-10-03T00:00:00.5+02:00", &mail_date)).is_err());
}
"##,
    )
    .expect("test");
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("ergonomics-target"),
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

#[test]
fn go_holds_a_timestamp_natively_names_shapes_by_position_and_constructs_the_constant() {
    let realization = plan()
        .go("probemeter", "example.invalid/probemeter")
        .expect("go");
    let types = &realization.declarations;
    assert!(types.contains("import \"time\""), "{types}");
    assert!(types.contains("CreatedAt time.Time"), "{types}");
    assert!(
        !types.contains("EssShape"),
        "anonymous shapes are named by position: {types}"
    );
    assert!(types.contains("ProbeMeterReadingSamples"), "{types}");
    assert!(
        types.contains("const ProbeMeterVersionValue int32 = 2"),
        "{types}"
    );
    assert!(
        types.contains("func NewProbeMeterVersion() *ProbeMeterVersion"),
        "{types}"
    );
    if Command::new("go").arg("version").output().is_err() {
        println!("skipped compiling: no `go` on PATH");
        return;
    }
    let root = scratch("ergonomics-go");
    fs::write(root.join("go.mod"), &realization.supporting["go.mod"]).expect("go.mod");
    fs::write(root.join("types.go"), types).expect("types");
    fs::write(
        root.join("types_test.go"),
        r#"package probemeter

import (
	"bytes"
	"encoding/json"
	"testing"
)

func TestReadingRoundTripsAndTheConstantIsThere(t *testing.T) {
	text := []byte(`{"createdAt":"2026-10-03T00:00:00.5+02:00","samples":[{"amount":1}],"version":2}`)
	var reading ProbeMeterReading
	if err := json.Unmarshal(text, &reading); err != nil {
		t.Fatal(err)
	}
	back, err := json.Marshal(reading)
	if err != nil {
		t.Fatal(err)
	}
	var want, got any
	_ = json.Unmarshal(text, &want)
	_ = json.Unmarshal(back, &got)
	wantText, _ := json.Marshal(want)
	gotText, _ := json.Marshal(got)
	if !bytes.Equal(wantText, gotText) {
		t.Fatalf("round trip: %s", back)
	}
	if NewProbeMeterVersion().Value != 2 {
		t.Fatal("constant")
	}
	if err := json.Unmarshal(bytes.Replace(text, []byte("2026-10-03T00:00:00.5+02:00"), []byte("yesterday"), 1), &reading); err == nil {
		t.Fatal("a non-RFC 3339 timestamp was accepted")
	}
}
"#,
    )
    .expect("test");
    let output = Command::new("go")
        .args(["test", "."])
        .current_dir(&root)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .expect("go runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn short_names_drop_the_domain_and_refuse_a_collision() {
    let short = plan().with_short_names().expect("short names");
    let names: Vec<&str> = short.declarations().values().map(String::as_str).collect();
    assert_eq!(names, ["Reading", "Sample", "Version"]);
    let go = short
        .go("probemeter", "example.invalid/probemeter")
        .expect("go")
        .declarations;
    assert!(go.contains("type Reading struct"), "{go}");
    assert!(go.contains("ReadingSamples"), "{go}");

    let meter = SOURCE.replace(
        "domains: [probe.meter]",
        "domains: [probe.meter, probe.other]",
    );
    let other = "domain: probe.other\ntypes:\n  - name: probe.other.Sample\n    kind: struct\n    fields:\n      - {name: amount, type: Integer}\n  - name: probe.other.Holder\n    kind: struct\n    fields:\n      - {name: a, type: probe.meter.Sample}\n      - {name: b, type: probe.other.Sample}\n";
    let selection = two_files(&meter, other, "probe.other.Holder");
    let refused = Plan::from_model(&selection)
        .expect("plan")
        .with_short_names()
        .expect_err("two Samples");
    assert!(
        refused
            .0
            .iter()
            .any(|finding| format!("{finding:?}").contains("short_name_collision")),
        "{refused}"
    );
}

fn two_files(first: &str, second: &str, root: &str) -> ess_gen::schema::ModelTypes {
    use ess_compiler::{resolve::compile, source::SourceMap};
    use ess_domain::{spec::RawSpecFile, spec::Specification, system::Source};
    let mut sources = SourceMap::new();
    sources.insert("meter.yaml", first);
    sources.insert("other.yaml", second);
    let specification = Specification::assemble([
        (
            Source::new("meter.yaml"),
            RawSpecFile::parse(first).expect("meter parses"),
        ),
        (
            Source::new("other.yaml"),
            RawSpecFile::parse(second).expect("other parses"),
        ),
    ])
    .unwrap_or_else(|errors| panic!("assembles: {errors}"));
    let ir =
        compile(&specification, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    ess_gen::schema::ModelTypes::select(&ir, &std::collections::BTreeSet::from([root.to_owned()]))
        .expect("selects")
}

#[test]
fn bundle_input_keeps_its_hash_names_and_refuses_short_names() {
    let bundle = schema_contract::bundle::import(
        &serde_json::json!({"components":{"schemas":{
            "Holder":{"type":"object","properties":{"items":{"type":"array","items":{"type":"string"}}}}
        }}})
        .to_string(),
        &std::collections::BTreeSet::from(["Holder".to_owned()]),
        schema_contract::bundle::Dialect::Draft202012,
    )
    .expect("bundle");
    let plan = Plan::from_bundle(
        &bundle,
        &std::collections::BTreeSet::from(["Holder".to_owned()]),
    )
    .expect("plan");
    let go = plan
        .clone()
        .go("holder", "example.invalid/holder")
        .expect("go")
        .declarations;
    assert!(
        go.contains("EssShape"),
        "bundle anonymous shapes keep their hash names: {go}"
    );
    assert!(plan.with_short_names().is_err());
}

/// A newtype's declared alphabet and text length, held by the realized Rust type's decoder
/// (beyond10x/ess#579): the report's unit, drawn from printable ASCII without the space and at
/// most eight characters long. A newtype that declares neither stays a plain transparent string.
const UNITS: &str = r##"format: ess/20
system: probe
version: v1
domains: [probe.units]
domain: probe.units
types:
  - name: probe.units.Unit
    kind: newtype
    of: String
    alphabet: "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
    invariants: [value.count >= 1, value.count <= 8]
  - name: probe.units.Label
    kind: newtype
    of: String
  - name: probe.units.Reading
    kind: struct
    fields:
      - {name: unit, type: probe.units.Unit}
      - {name: label, type: probe.units.Label}
"##;

/// The rules a realization's report leaves on one definition's alphabet: its `model_alphabet`
/// annotation and any `pattern` beside it.
fn alphabet_obligations(report: &schema_contract::realize::Report, name: &str) -> Vec<String> {
    let report = serde_json::to_value(report).expect("report");
    report["obligations"]
        .as_array()
        .expect("obligations")
        .iter()
        .filter(|item| {
            let pointer = item["pointer"].as_str().unwrap_or_default();
            pointer.ends_with(&format!("/{name}/x-ess-alphabet"))
                || pointer.ends_with(&format!("/{name}/pattern"))
        })
        .map(|item| item["rule"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// The projection now carries the alphabet twice, as `x-ess-alphabet` and as the `pattern` derived
/// from it (beyond10x/ess#592); the types-only realizer reports it once. A prefix still adds a
/// pattern the alphabet does not state, so that newtype keeps its `pattern` obligation.
#[test]
fn issue_592_the_types_only_realizer_reports_the_alphabet_once() {
    let source = UNITS.replacen(
        "  - name: probe.units.Label\n    kind: newtype\n    of: String\n",
        "  - name: probe.units.Label\n    kind: newtype\n    of: String\n    alphabet: \"abcm\"\n    \
         prefix: \"m\"\n",
        1,
    );
    let plan =
        Plan::from_model(&model::selection(&source, &["probe.units.Reading"])).expect("plan");
    let typescript = plan.typescript();
    assert_eq!(
        alphabet_obligations(&typescript.report, "probe.units.Unit"),
        ["model_alphabet"],
        "{}",
        serde_json::to_string(&typescript.report).unwrap()
    );
    assert_eq!(
        alphabet_obligations(&typescript.report, "probe.units.Label"),
        ["pattern", "model_alphabet"]
    );
    let rust = plan.rust("probe-units").expect("rust");
    assert_eq!(
        alphabet_obligations(&rust.report, "probe.units.Unit"),
        Vec::<String>::new(),
        "the Rust decoder checks the alphabet, and the pattern states nothing more"
    );
}

#[test]
fn rust_refuses_a_value_outside_a_declared_alphabet_or_length() {
    let plan = Plan::from_model(&model::selection(UNITS, &["probe.units.Reading"])).expect("plan");
    let realization = plan.rust("probe-units").expect("rust");
    let types = &realization.declarations;
    assert!(
        types.contains(&format!(
            "{}#[serde(transparent)]\npub struct ProbeUnitsLabel(pub ::std::string::String);\n",
            "#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]\n"
        )),
        "an unconstrained newtype keeps its bytes: {types}"
    );
    assert!(
        types.contains("pub fn broken_constraint(&self) -> Option<&'static str>"),
        "{types}"
    );
    assert!(
        !serde_json::to_string(&realization.report)
            .expect("report")
            .contains("model_alphabet"),
        "the Rust decoder checks the alphabet, so it is no obligation left to the consumer"
    );

    let root = scratch("units-rust");
    fs::create_dir_all(root.join("tests")).expect("tests dir");
    fs::write(
        root.join("Cargo.toml"),
        &realization.supporting["Cargo.toml"],
    )
    .expect("manifest");
    fs::write(root.join("types.rs"), types).expect("types");
    fs::write(
        root.join("tests/units.rs"),
        r#"
use probe_units::*;

fn reading(unit: &str) -> String {
    serde_json::json!({"unit": unit, "label": "anything at all"}).to_string()
}

#[test]
fn a_unit_outside_its_alphabet_or_length_is_refused_naming_the_constraint() {
    for unit in ["m", "kg/m2", "12345678", "{x}~"] {
        let decoded: ProbeUnitsReading = serde_json::from_str(&reading(unit)).expect(unit);
        assert_eq!(decoded.unit.0, unit);
        assert_eq!(serde_json::to_string(&decoded.unit).unwrap(), serde_json::to_string(unit).unwrap());
        assert_eq!(decoded.unit.broken_constraint(), None);
    }
    for (unit, constraint) in [
        ("m s", "one of"),
        ("m\t s", "one of"),
        ("m\u{0}", "one of"),
        ("m\u{7f}", "one of"),
        ("\u{b5}m", "one of"),
        (" ", "one of"),
        ("", "at least 1"),
        ("123456789", "at most 8"),
    ] {
        let error = serde_json::from_str::<ProbeUnitsReading>(&reading(unit))
            .expect_err(&format!("{unit:?} decodes"))
            .to_string();
        assert!(error.contains("probe.units.Unit") && error.contains(constraint), "{unit:?}: {error}");
        let direct = serde_json::from_value::<ProbeUnitsUnit>(serde_json::json!(unit))
            .expect_err(&format!("{unit:?} decodes directly"))
            .to_string();
        assert!(direct.contains(constraint), "{unit:?}: {direct}");
        assert!(ProbeUnitsUnit(unit.to_owned()).broken_constraint().unwrap().contains(constraint));
    }
    let label: ProbeUnitsLabel = serde_json::from_str("\" \"").unwrap();
    assert_eq!(label.0, " ");
}
"#,
    )
    .expect("test");
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("ergonomics-target"),
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

/// A struct written as one field alone (`ess/27`, beyond10x/ess#594).
#[path = "model_ergonomics/shorthand.rs"]
mod shorthand;

/// The generated manifest's version ranges and number feature (beyond10x/ess#660).
#[path = "model_ergonomics/manifest.rs"]
mod manifest;

/// The Rust generator options (beyond10x/ess#657, #667, #668) held in a built crate. The
/// option scenarios that need no build are in `rust_generator_options.rs`; these two build
/// here, the file allowed to run cargo, into this file's shared target.
#[allow(dead_code)]
#[path = "fixtures/rust_generator_options.rs"]
mod generator_option_fixtures;

mod generator_options {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use schema_contract::realize::{Realization, RustDerive, RustOptions};

    use super::generator_option_fixtures::{bundle_plan, declarations, open_schemas, shop_plan};

    /// Builds the generated crate and runs `test` against it.
    fn build_and_test(realization: &Realization, name: &str, test: &str) {
        let root = super::scratch(name);
        fs::create_dir_all(root.join("tests")).expect("tests");
        fs::write(
            root.join("Cargo.toml"),
            &realization.supporting["Cargo.toml"],
        )
        .expect("manifest");
        fs::write(root.join("types.rs"), &realization.declarations).expect("types");
        fs::write(root.join("tests/options.rs"), test).expect("test");
        let output = Command::new(env!("CARGO"))
            .args(["test", "--offline", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("ergonomics-target"),
            )
            .output()
            .expect("cargo runs");
        assert!(
            output.status.success(),
            "{name}:\n{}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
            realization.declarations
        );
    }

    /// The same build holds the added derives: `Hash` on every type and `Eq` on the SKU.
    #[test]
    fn fold_null_decodes_null_as_absent() {
        let options = RustOptions {
            derives: BTreeSet::from([RustDerive::Hash]),
            type_derives: BTreeMap::from([(
                "probe.shop.Sku".to_owned(),
                BTreeSet::from([RustDerive::Eq]),
            )]),
            fold_null: true,
            ..RustOptions::default()
        };
        let realization = shop_plan().rust_with("shop_types", &options).unwrap();
        assert!(
            declarations(&realization)["ProbeShopLine"]
                .contains("deserialize_with = \"ess_fold_null\""),
            "{}",
            realization.declarations
        );
        build_and_test(
            &realization,
            "fold-null",
            r##"
use shop_types::*;

#[test]
fn null_and_missing_decode_alike_and_encode_omitted() {
    let missing = r#"{"amount":{"kind":"text","value":"t"},"count":1,"sku":"a"}"#;
    let null = r#"{"amount":{"kind":"text","value":"t"},"colour":null,"count":1,"note":null,"sku":"a"}"#;
    let from_missing: ProbeShopLine = serde_json::from_str(missing).unwrap();
    let from_null: ProbeShopLine = serde_json::from_str(null).unwrap();
    assert_eq!(from_missing, from_null);
    assert!(from_null.note.is_absent() && from_null.colour.is_absent());
    assert_eq!(serde_json::to_string(&from_null).unwrap(), missing);
    let present: ProbeShopLine = serde_json::from_str(&missing.replace(r#""count":1,"#, r#""count":1,"note":"n","#)).unwrap();
    assert_eq!(present.note, EssPresence::Present("n".to_owned()));
    assert!(serde_json::to_string(&present).unwrap().contains(r#""note":"n""#));
}

fn hash_of<T: std::hash::Hash>(value: &T) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn equal_orders_hash_alike() {
    let text = r#"{"lines":[{"sku":"a","count":1,"note":"n","colour":"red","amount":{"kind":"text","value":"t"}}],"payload":{"k":[1,2]}}"#;
    let one: ProbeShopOrder = serde_json::from_str(text).unwrap();
    let two: ProbeShopOrder = serde_json::from_str(text).unwrap();
    assert_eq!(hash_of(&one), hash_of(&two));
    let other: ProbeShopOrder = serde_json::from_str(&text.replace(r#""note":"n","#, "")).unwrap();
    assert_ne!(hash_of(&one), hash_of(&other));
}

#[test]
fn a_sku_goes_in_a_hash_set() {
    let set: std::collections::HashSet<ProbeShopSku> =
        ["a", "b", "a"].iter().map(|sku| ProbeShopSku((*sku).to_owned())).collect();
    assert_eq!(set.len(), 2);
}
"##,
        );
    }

    #[test]
    fn drop_unknown_members_has_no_extra_field() {
        let options = RustOptions {
            drop_unknown_members: true,
            ..RustOptions::default()
        };
        let realization = bundle_plan(&open_schemas())
            .rust_with("open_types", &options)
            .unwrap();
        let declarations = declarations(&realization);
        for name in ["Open", "Counted"] {
            assert!(
                !declarations[name].contains("ess_extra"),
                "{}",
                declarations[name]
            );
        }
        // A map's members are its data, not unknown members of a record.
        assert!(
            declarations["Map"].contains("pub ess_extra"),
            "{}",
            declarations["Map"]
        );
        build_and_test(
            &realization,
            "drop-unknown",
            r##"
use open_types::*;

#[test]
fn unknown_members_are_decoded_and_not_encoded() {
    let open: Open = serde_json::from_str(r#"{"y":1}"#).unwrap();
    assert_eq!(serde_json::to_string(&open).unwrap(), "{}");
    let counted: Counted = serde_json::from_str(r#"{"m":2,"n":1}"#).unwrap();
    assert_eq!(serde_json::to_string(&counted).unwrap(), r#"{"n":1}"#);
    let map: Map = serde_json::from_str(r#"{"a":"b"}"#).unwrap();
    assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"a":"b"}"#);
    assert!(serde_json::from_str::<Closed>(r#"{"z":1}"#).is_err());
}
"##,
        );
    }
}

/// A `Binary64` newtype bounded by its invariants (beyond10x/ess#687): the projection publishes
/// `exclusiveMinimum` and `maximum` on its definition.
const RATES: &str = r"format: ess/20
system: probe
version: v1
domains: [probe.sensor]
domain: probe.sensor
types:
  - name: probe.sensor.Rate
    kind: newtype
    of: Binary64
    invariants: [value > 0, value <= 1]
  - name: probe.sensor.Reading
    kind: struct
    fields:
      - {name: rate, type: probe.sensor.Rate}
";

/// The rules a realization's report leaves on one definition: every obligation whose pointer is
/// the definition's own keyword.
fn definition_obligations(report: &schema_contract::realize::Report, name: &str) -> Vec<String> {
    let report = serde_json::to_value(report).expect("report");
    report["obligations"]
        .as_array()
        .expect("obligations")
        .iter()
        .filter(|item| {
            let pointer = item["pointer"].as_str().unwrap_or_default();
            pointer.contains(&format!("/{name}"))
        })
        .map(|item| item["rule"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// `ess generate types` realizes a bounded `Binary64` newtype in Rust, Go and TypeScript without a
/// refusal; the new keywords are runtime obligations on every target.
#[test]
fn issue_687_a_bounded_binary64_realizes_with_its_bounds_as_obligations() {
    let selection = model::selection(RATES, &["probe.sensor.Reading"]);
    let definitions: serde_json::Value = serde_json::from_str(&selection.to_json()).expect("JSON");
    let rate = &definitions["$defs"]["probe.sensor.Rate"];
    assert_eq!(rate["exclusiveMinimum"], serde_json::json!(0), "{rate:#}");
    assert_eq!(rate["maximum"], serde_json::json!(1), "{rate:#}");

    let plan = Plan::from_model(&selection).expect("plan");
    let reports = [
        ("typescript", plan.typescript().report),
        ("rust", plan.rust("probe-sensor").expect("rust").report),
        (
            "go",
            plan.go("sensor", "example.invalid/sensor")
                .expect("go")
                .report,
        ),
    ];
    for (target, report) in &reports {
        let rules = definition_obligations(report, "probe.sensor.Rate");
        for rule in ["exclusiveMinimum", "maximum"] {
            assert!(
                rules.iter().any(|found| found == rule),
                "{target}: no `{rule}` obligation on the Rate definition: {rules:?}"
            );
        }
    }
    // The bounds do not discharge the finite-binary64 contract: TypeScript, which has no codec of
    // its own for it, still reports `model_binary64`.
    let typescript = definition_obligations(&reports[0].1, "probe.sensor.Rate");
    assert!(
        typescript.iter().any(|found| found == "model_binary64"),
        "{typescript:?}"
    );
}

const LENGTHS: &str = r"format: ess/20
system: probe
version: v1
domains: [probe.units]
domain: probe.units
types:
  - name: probe.units.Unit
    kind: newtype
    of: String
    invariants: [value.count >= 1, value.count <= 8]
  - name: probe.units.Measure
    kind: struct
    fields:
      - {name: unit, type: probe.units.Unit}
      - {name: note, type: String}
    invariants: [note.count <= 40]
";

/// A `String` newtype whose `value.count` bounds are published as `minLength` and `maxLength`
/// realizes on every target with the length reported once: the realizer enforces the text
/// constraint it reads from `x-ess-invariants`, so the keywords add no obligation of their own.
/// A struct field's length has no such constraint, so its `maxLength` stays an obligation.
#[test]
fn issue_685_the_types_only_realizer_reports_the_length_once() {
    let selection = model::selection(LENGTHS, &["probe.units.Measure"]);
    let definitions: serde_json::Value = serde_json::from_str(&selection.to_json()).expect("JSON");
    let unit = &definitions["$defs"]["probe.units.Unit"];
    assert_eq!(unit["minLength"], serde_json::json!(1), "{unit:#}");
    assert_eq!(unit["maxLength"], serde_json::json!(8), "{unit:#}");

    let plan = Plan::from_model(&selection).expect("plan");
    let reports = [
        ("typescript", plan.typescript().report),
        ("rust", plan.rust("probe-units").expect("rust").report),
        (
            "go",
            plan.go("units", "example.invalid/units")
                .expect("go")
                .report,
        ),
    ];
    for (target, report) in &reports {
        let rules = definition_obligations(report, "probe.units.Unit");
        for rule in ["minLength", "maxLength"] {
            assert!(
                !rules.iter().any(|found| found == rule),
                "{target}: a `{rule}` obligation repeats the enforced length: {rules:?}"
            );
        }
        let measure = definition_obligations(report, "probe.units.Measure");
        assert!(
            measure.iter().any(|found| found == "maxLength"),
            "{target}: the field length is not enforced and must stay an obligation: {measure:?}"
        );
    }
}
