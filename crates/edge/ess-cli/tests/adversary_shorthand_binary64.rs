//! Adversary cases for story:a-shorthand-struct-binary64-location-resolves.
//!
//! The story's own cases hold one flat shorthand struct per selection. These drive the compiler's
//! Binary64 locations through a shorthand over a newtype, Optional/List/Map Binary64 fields in the
//! object branch, a shorthand struct nested and listed inside another, a struct without a
//! shorthand holding shorthand structs, a command input and two view rows; they decode both forms
//! in generated Go and type-check both forms against generated TypeScript.

use ess_cli::TemporaryDirectory;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    process::{Command, Output},
};

const SPEC: &str = include_str!("fixtures/adversary-shorthand-binary64/spec.yaml");

struct Fixture(TemporaryDirectory);
impl Fixture {
    fn new(spec: &str) -> Self {
        let path = TemporaryDirectory::create("ess-adversary-shorthand-binary64").unwrap();
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), spec).unwrap();
        Self(path)
    }
    fn generate(&self, roots: &[&str], target: &str, out: &str) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .current_dir(&self.0)
            .args(["generate", "types", "--path", "model"]);
        for root in roots {
            command.args(["--root", root]);
        }
        command.args(["--target", target, "--out", out]);
        match target {
            "rust" => command.args(["--package", "demo"]),
            "go" => command.args(["--package", "demo", "--module", "example.com/demo"]),
            _ => &mut command,
        };
        command.output().unwrap()
    }
    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}

/// Each selection, and whether it holds a struct written as a number alone (which the Rust target
/// refuses as `rust_shorthand_number`, a known refusal that is not this story's).
const SELECTIONS: [(&str, bool); 6] = [
    ("tracker.ops.Meter", true),
    ("tracker.ops.Panel", false),
    ("tracker.ops.Holder", true),
    ("tracker.ops.Record.input", false),
    ("tracker.ops.Samples.row", false),
    ("tracker.ops.Rows.row", false),
];

#[test]
fn adversary_every_selection_realizes_or_is_refused_only_as_a_number_short_form() {
    let fixture = Fixture::new(SPEC);
    let mut failures = Vec::new();
    for (index, (root, number_short)) in SELECTIONS.iter().enumerate() {
        for target in ["rust", "go", "typescript"] {
            let out = format!("realize-{index}-{target}");
            let output = fixture.generate(&[root], target, &out);
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let ok = if target == "rust" && *number_short {
                !output.status.success()
                    && stderr.contains("rust_shorthand_number")
                    && !stderr.contains("model_binary64_layout")
            } else {
                output.status.success()
            };
            if !ok {
                failures.push(format!(
                    "{root} --target {target}: {} {stderr}",
                    output.status
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// The pointer of every node of `schema` whose `type` is `number`.
fn numeric_nodes(at: &str, node: &Value, found: &mut BTreeSet<String>) {
    match node {
        Value::Object(map) => {
            if map.get("type") == Some(&Value::from("number")) {
                found.insert(at.to_owned());
            }
            for (key, child) in map {
                let key = key.replace('~', "~0").replace('/', "~1");
                numeric_nodes(&format!("{at}/{key}"), child, found);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                numeric_nodes(&format!("{at}/{index}"), child, found);
            }
        }
        _ => {}
    }
}

fn binary64_obligations(report: &str) -> BTreeSet<String> {
    let report: Value = serde_json::from_str(report).unwrap();
    report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["rule"] == "model_binary64")
        .map(|item| item["pointer"].as_str().unwrap().to_owned())
        .collect()
}

/// Every number in this model is a `Binary64`: the compiler-owned locations the TypeScript
/// types-report names are exactly the numeric nodes of the published schema, no fewer and no more.
#[test]
fn adversary_binary64_obligations_are_exactly_the_numeric_nodes_of_the_published_schema() {
    let fixture = Fixture::new(SPEC);
    let mut failures = Vec::new();
    for (index, (root, _)) in SELECTIONS.iter().enumerate() {
        let out = format!("report-{index}");
        let output = fixture.generate(&[root], "typescript", &out);
        assert!(output.status.success(), "{root}: {output:?}");
        let schema: Value =
            serde_json::from_str(&fixture.read(&format!("{out}/source.schema.json"))).unwrap();
        let mut numeric = BTreeSet::new();
        numeric_nodes("", &schema, &mut numeric);
        let located = binary64_obligations(&fixture.read(&format!("{out}/types-report.json")));
        if numeric.is_empty() || located != numeric {
            failures.push(format!("{root}: located {located:#?} numeric {numeric:#?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// A shorthand over a newtype of `Binary64`: the short form is a reference, so the location is the
/// newtype's own definition and nothing points at `oneOf/0`.
#[test]
fn adversary_a_newtype_shorthand_locates_binary64_on_the_newtype() {
    let fixture = Fixture::new(SPEC);
    let output = fixture.generate(&["tracker.ops.Meter"], "typescript", "ts");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        binary64_obligations(&fixture.read("ts/types-report.json")),
        BTreeSet::from(["/$defs/tracker.ops.Level".to_owned()])
    );
    let schema: Value = serde_json::from_str(&fixture.read("ts/source.schema.json")).unwrap();
    let short = schema
        .pointer("/$defs/tracker.ops.Meter/oneOf/0/$ref")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{schema:#}"));
    assert!(short.ends_with("tracker.ops.Level"), "{short}");
}

const GO_WIRE_TEST: &str = r#"package demo

import (
	"encoding/json"
	"strings"
	"testing"
)

func decode[T any](t *testing.T, input string) string {
	t.Helper()
	var value T
	if err := json.Unmarshal([]byte(input), &value); err != nil {
		t.Fatalf("%s: %v", input, err)
	}
	out, err := json.Marshal(&value)
	if err != nil {
		t.Fatalf("%s: encode: %v", input, err)
	}
	return string(out)
}

func refuse[T any](t *testing.T, input string) {
	t.Helper()
	var value T
	if err := json.Unmarshal([]byte(input), &value); err == nil {
		t.Errorf("%s decoded", input)
	}
}

func want(t *testing.T, got, part string) {
	t.Helper()
	if !strings.Contains(got, part) {
		t.Errorf("%s lacks %s", got, part)
	}
}

func TestGaugeNumberShortForm(t *testing.T) {
	want(t, decode[TrackerOpsGauge](t, `1.5`), `"level":1.5`)
	want(t, decode[TrackerOpsGauge](t, ` -0`), `"level":-0.0`)
	want(t, decode[TrackerOpsGauge](t, `{"level":2.5,"unit":"m"}`), `"unit":"m"`)
	refuse[TrackerOpsGauge](t, `1e400`)
	refuse[TrackerOpsGauge](t, `"1.5"`)
	refuse[TrackerOpsGauge](t, `null`)
	refuse[TrackerOpsGauge](t, `{"level":1e400}`)
}

func TestMeterNewtypeShortForm(t *testing.T) {
	want(t, decode[TrackerOpsMeter](t, `-0`), `"level":-0.0`)
	want(t, decode[TrackerOpsMeter](t, `{"level":3}`), `"level":3.0`)
	refuse[TrackerOpsMeter](t, `1e400`)
	refuse[TrackerOpsMeter](t, `"x"`)
}

func TestNestedAndListedShorthandStructs(t *testing.T) {
	got := decode[TrackerOpsPanel](t, `{"title":"p","main":"x","readings":["a",{"label":"b","value":-0,"either":null,"series":[1.5,-0],"table":{"k":2.5}}]}`)
	want(t, got, `"value":-0.0`)
	want(t, got, `"series":[1.5,-0.0]`)
	want(t, got, `"table":{"k":2.5}`)
	want(t, decode[TrackerOpsPanel](t, `"p"`), `"title":"p"`)
	refuse[TrackerOpsPanel](t, `{"title":"p","readings":[{"label":"b","series":[1e400]}]}`)
	refuse[TrackerOpsPanel](t, `{"title":"p","main":{"label":"b","table":{"k":1e400}}}`)
	refuse[TrackerOpsPanel](t, `{"title":"p","main":{"label":"b","value":"1.5"}}`)
}

func TestStructWithoutShorthandHoldingShorthandStructs(t *testing.T) {
	got := decode[TrackerOpsHolder](t, `{"amount":-0,"reading":"r","gauges":[1.5,{"level":-0}]}`)
	want(t, got, `"amount":-0.0`)
	want(t, got, `{"level":1.5}`)
	want(t, got, `{"level":-0.0}`)
	refuse[TrackerOpsHolder](t, `{"amount":1e400,"reading":"r"}`)
	refuse[TrackerOpsHolder](t, `{"amount":1,"reading":"r","gauges":[1e400]}`)
}
"#;

#[test]
fn adversary_go_decodes_number_short_forms_and_nested_shorthand_structs() {
    let fixture = Fixture::new(SPEC);
    let roots = [
        "tracker.ops.Gauge",
        "tracker.ops.Meter",
        "tracker.ops.Panel",
        "tracker.ops.Holder",
    ];
    let output = fixture.generate(&roots, "go", "go");
    assert!(output.status.success(), "{output:?}");
    fs::write(fixture.0.join("go/wire_test.go"), GO_WIRE_TEST).unwrap();
    let result = Command::new("go")
        .args(["test", "-count=1", "./..."])
        .current_dir(fixture.0.join("go"))
        .env("GOTOOLCHAIN", "local")
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOWORK", "off")
        .env("GOFLAGS", "-buildvcs=false")
        .output()
        .expect("a Go toolchain on PATH");
    assert!(
        result.status.success(),
        "go test: {}\n{}\n{}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

const TS_CHECK: &str = r#"import type { TrackerOpsGauge, TrackerOpsMeter, TrackerOpsPanel, TrackerOpsReading } from "./types.js";

export const g1: TrackerOpsGauge = 1.5;
export const g2: TrackerOpsGauge = { level: 1.5 };
export const m1: TrackerOpsMeter = -0;
export const m2: TrackerOpsMeter = { level: 2.5, unit: "m" };
export const r1: TrackerOpsReading = "x";
export const r2: TrackerOpsReading = { label: "x", value: 1.5, either: null, series: [1.5], table: { k: 2.5 } };
export const p1: TrackerOpsPanel = "p";
export const p2: TrackerOpsPanel = { title: "p", main: "x", readings: ["a", { label: "b", value: -0 }] };
// @ts-expect-error a Gauge is not written as a string
export const bad1: TrackerOpsGauge = "x";
// @ts-expect-error a Reading is not written as a number
export const bad2: TrackerOpsReading = 1.5;
"#;

#[test]
fn adversary_typescript_declares_both_forms() {
    let fixture = Fixture::new(SPEC);
    let roots = [
        "tracker.ops.Gauge",
        "tracker.ops.Meter",
        "tracker.ops.Panel",
    ];
    let output = fixture.generate(&roots, "typescript", "ts");
    assert!(output.status.success(), "{output:?}");
    fs::write(fixture.0.join("ts/check.ts"), TS_CHECK).unwrap();
    let result = Command::new("tsc")
        .args([
            "--strict",
            "--exactOptionalPropertyTypes",
            "--noEmit",
            "--skipLibCheck",
            "--module",
            "nodenext",
            "--moduleResolution",
            "nodenext",
            "--target",
            "es2022",
            "types.ts",
            "check.ts",
        ])
        .current_dir(fixture.0.join("ts"))
        .output()
        .expect("tsc on PATH");
    assert!(
        result.status.success(),
        "tsc: {}\n{}\n{}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

/// A shorthand struct whose other field is `Optional<Binary64>` with `presence: null_when_absent`
/// compiles (`shorthand.rs` asks only that every other field is `Optional`), and its Binary64
/// location sits under `oneOf/1/properties/nulled/anyOf/0`.
const NULLED: &str = "format: ess/31
system: tracker
version: v1
domain: tracker.ops
types:
  - name: tracker.ops.Marked
    kind: struct
    shorthand: label
    fields:
      - {name: label, type: String}
      - {name: nulled, type: 'Optional<Binary64>', presence: null_when_absent}
";

#[test]
#[ignore = "story:a-null-when-absent-field-beside-a-shorthand-is-realized-or-refused (backlog)"]
fn adversary_a_null_when_absent_binary64_field_in_a_shorthand_struct_realizes() {
    let fixture = Fixture::new(NULLED);
    let mut failures = Vec::new();
    for target in ["go", "typescript"] {
        let output = fixture.generate(&["tracker.ops.Marked"], target, target);
        if !output.status.success() {
            failures.push(format!(
                "--target {target}: {} {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
