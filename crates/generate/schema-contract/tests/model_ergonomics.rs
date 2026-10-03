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
        manifest.contains("time = { version = \"=0.3.55\""),
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
