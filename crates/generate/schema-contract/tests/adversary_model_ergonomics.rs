//! Adversarial caller checks for generated model ergonomics.

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
  - name: probe.meter.Revision
    kind: newtype
    of: Integer
    invariants:
      - value == -2147483649
  - name: probe.meter.Reading
    kind: struct
    fields:
      - {name: created_at, type: Timestamp, wire: createdAt}
      - {name: revision, type: probe.meter.Revision}
";

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &["probe.meter.Reading"])).expect("plan")
}

const POSITION_COLLISION_SOURCE: &str = r"format: ess/20
system: probe
version: v1
domains: [probe.meter]
domain: probe.meter
types:
  - name: probe.meter.ReadingSamples
    kind: struct
    fields:
      - {name: label, type: String}
  - name: probe.meter.Reading
    kind: struct
    fields:
      - {name: samples, type: 'List<String>'}
      - {name: detail, type: probe.meter.ReadingSamples}
";

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
fn generated_rust_canonicalizes_a_valid_fractional_timestamp() {
    let realization = plan().rust("probe-meter").expect("rust");
    let root = scratch("adversary-timestamp-rust");
    fs::create_dir_all(root.join("tests")).expect("tests dir");
    fs::write(
        root.join("Cargo.toml"),
        realization.supporting["Cargo.toml"].as_bytes(),
    )
    .expect("manifest");
    fs::write(root.join("types.rs"), realization.declarations.as_bytes()).expect("types");
    fs::write(
        root.join("tests/round_trip.rs"),
        r##"
use probe_meter::*;

#[test]
fn timestamp_wire_spelling_is_canonical_after_a_generated_type_round_trip() {
    assert_eq!(ProbeMeterRevision::default().0, -2_147_483_649_i64);
    let text = r#"{"createdAt":"2026-10-03T00:00:00.500+02:00","revision":-2147483649}"#;
    let reading: ProbeMeterReading = serde_json::from_str(text).unwrap();
    assert_eq!(
        serde_json::to_string(&reading).unwrap(),
        r#"{"createdAt":"2026-10-03T00:00:00.5+02:00","revision":-2147483649}"#
    );
}
"##,
    )
    .expect("test");
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary-ergonomics-target"),
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
fn generated_go_canonicalizes_a_valid_fractional_timestamp() {
    let realization = plan()
        .go("probemeter", "example.invalid/probemeter")
        .expect("go");
    if Command::new("go").arg("version").output().is_err() {
        println!("skipped compiling: no `go` on PATH");
        return;
    }
    let root = scratch("adversary-timestamp-go");
    fs::write(
        root.join("go.mod"),
        realization.supporting["go.mod"].as_bytes(),
    )
    .expect("go.mod");
    fs::write(root.join("types.go"), realization.declarations.as_bytes()).expect("types");
    fs::write(
        root.join("types_test.go"),
        r#"package probemeter

import (
	"bytes"
	"encoding/json"
	"testing"
)

func TestTimestampWireSpellingIsCanonicalAfterAGeneratedTypeRoundTrip(t *testing.T) {
	if NewProbeMeterRevision().Value != -2147483649 {
		t.Fatal("i64 constant constructor did not carry its value")
	}
	text := []byte(`{"createdAt":"2026-10-03T00:00:00.500+02:00","revision":-2147483649}`)
	var reading ProbeMeterReading
	if err := json.Unmarshal(text, &reading); err != nil {
		t.Fatal(err)
	}
	back, err := json.Marshal(reading)
	if err != nil {
		t.Fatal(err)
	}
	want := []byte(`{"createdAt":"2026-10-03T00:00:00.5+02:00","revision":-2147483649}`)
	if !bytes.Equal(back, want) {
		t.Fatalf("non-canonical timestamp: got %s, want %s", back, want)
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
fn a_reachable_position_name_collision_uses_the_documented_deterministic_hash_fallback() {
    let plan = Plan::from_model(&model::selection(
        POSITION_COLLISION_SOURCE,
        &["probe.meter.Reading"],
    ))
    .expect("plan");
    let rust = plan.clone().rust("probe-meter").expect("rust").declarations;
    let go = plan
        .clone()
        .go("probemeter", "example.invalid/probemeter")
        .expect("go")
        .declarations;
    assert!(
        !rust.contains("EssShape"),
        "Rust's native Vec field introduced an anonymous API name:\n{rust}"
    );
    let go_fallback = go
        .lines()
        .find_map(|line| line.strip_prefix("type EssShape"))
        .and_then(|rest| rest.split_whitespace().next())
        .map_or_else(
            || panic!("Go hash fallback in:\n{go}"),
            |suffix| format!("EssShape{suffix}"),
        );
    assert!(
        go.contains(&format!("Samples {go_fallback}")),
        "Go does not use its collision fallback for Reading.samples:\n{go}"
    );
    assert_eq!(
        go,
        plan.go("probemeter", "example.invalid/probemeter")
            .expect("Go again")
            .declarations,
        "the collision fallback changed across identical realization calls"
    );
}
