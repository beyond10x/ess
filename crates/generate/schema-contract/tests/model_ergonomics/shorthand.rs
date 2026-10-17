//! A struct with `shorthand: <field>` (`ess/27`, beyond10x/ess#594) in the types-only outputs:
//! Rust and Go decode either the field's value alone or the record into the struct and encode the
//! record; TypeScript declares the wire union of the two, and its report says a consumer
//! normalizes the short form.

use std::fs;
use std::path::Path;
use std::process::Command;

use schema_contract::realize::Plan;

use super::{model, scratch};

const SOURCE: &str = r"format: ess/27
system: probe
version: v1
domains: [probe.ops]
domain: probe.ops
types:
  - name: probe.ops.Mode
    kind: enum
    variants: [fast, careful]
  - name: probe.ops.Operation
    kind: struct
    shorthand: op
    fields:
      - {name: op, type: String, wire: operation}
      - {name: paging, type: Optional<String>}
  - name: probe.ops.Step
    kind: struct
    shorthand: mode
    fields:
      - {name: mode, type: probe.ops.Mode}
      - {name: note, type: Optional<String>}
  - name: probe.ops.Batch
    kind: struct
    fields:
      - {name: operations, type: 'List<probe.ops.Operation>'}
      - {name: step, type: probe.ops.Step}
";

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &["probe.ops.Batch"])).expect("plan")
}

#[test]
fn rust_types_decode_the_short_form() {
    let realization = plan().rust("probe-ops").expect("rust");
    let root = scratch("shorthand-rust");
    fs::create_dir_all(root.join("tests")).expect("tests dir");
    fs::write(
        root.join("Cargo.toml"),
        &realization.supporting["Cargo.toml"],
    )
    .expect("manifest");
    fs::write(root.join("types.rs"), &realization.declarations).expect("types");
    fs::write(
        root.join("tests/short_form.rs"),
        r##"
use probe_ops::*;

#[test]
fn the_short_form_decodes_and_the_long_form_is_written() {
    let short: ProbeOpsOperation = serde_json::from_str("\"getIssue\"").unwrap();
    assert_eq!(short.op, "getIssue");
    assert!(short.paging.is_absent());
    assert_eq!(serde_json::to_string(&short).unwrap(), r#"{"operation":"getIssue"}"#);

    let long: ProbeOpsOperation =
        serde_json::from_str(r#"{"operation":"getIssue","paging":"token"}"#).unwrap();
    assert_eq!(long.op, "getIssue");
    assert_eq!(
        serde_json::to_string(&long).unwrap(),
        r#"{"operation":"getIssue","paging":"token"}"#
    );
    // Through a `Value` as well as from text.
    let through: ProbeOpsOperation = serde_json::from_value(serde_json::json!("getIssue")).unwrap();
    assert_eq!(through, short);

    for refused in ["7", "true", "null", r#"["getIssue"]"#, r#"{"paging":"token"}"#, r#"{"operation":"getIssue","extra":1}"#] {
        assert!(serde_json::from_str::<ProbeOpsOperation>(refused).is_err(), "{refused}");
    }

    // An enum field written alone, and both short forms nested in a record.
    let batch: ProbeOpsBatch =
        serde_json::from_str(r#"{"operations":["getIssue",{"operation":"list"}],"step":"careful"}"#)
            .unwrap();
    assert_eq!(batch.operations[1].op, "list");
    assert_eq!(*batch.step.mode, ProbeOpsMode::V0);
    assert!(serde_json::from_str::<ProbeOpsStep>("\"reckless\"").is_err());
    assert_eq!(
        serde_json::to_string(&batch).unwrap(),
        r#"{"operations":[{"operation":"getIssue"},{"operation":"list"}],"step":{"mode":"careful"}}"#
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
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("ergonomics-target"),
        )
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "{}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        realization.declarations
    );
}

#[test]
fn rust_refuses_a_numeric_short_form_by_name() {
    let source = SOURCE.replace(
        "{name: op, type: String, wire: operation}",
        "{name: op, type: Integer, wire: operation}",
    );
    let plan =
        Plan::from_model(&model::selection(&source, &["probe.ops.Operation"])).expect("plan");
    let refused = plan
        .rust("probe-ops")
        .expect_err("a numeric short form is refused");
    assert!(
        refused
            .0
            .iter()
            .any(|finding| finding.rule == "rust_shorthand_number"
                && finding.pointer == "/$defs/probe.ops.Operation"),
        "{refused:?}"
    );
    // Go reads the number's own token and decodes it.
    plan.go("ops", "example.invalid/ops").expect("go");
}

#[test]
fn go_types_decode_the_short_form() {
    let realization = plan().go("ops", "example.invalid/ops").expect("go");
    if Command::new("go").arg("version").output().is_err() {
        println!("skipped compiling Go: no `go` on PATH");
        return;
    }
    let root = scratch("shorthand-go");
    fs::write(root.join("go.mod"), &realization.supporting["go.mod"]).expect("go.mod");
    fs::write(root.join("types.go"), &realization.declarations).expect("types");
    fs::write(
        root.join("types_test.go"),
        r#"package ops

import (
	"encoding/json"
	"testing"
)

func TestTheShortFormDecodesAndTheLongFormIsWritten(t *testing.T) {
	var short ProbeOpsOperation
	if err := json.Unmarshal([]byte(" \"getIssue\""), &short); err != nil {
		t.Fatal(err)
	}
	if short.Operation != "getIssue" || short.Paging.Present {
		t.Fatalf("%+v", short)
	}
	encoded, err := json.Marshal(short)
	if err != nil || string(encoded) != `{"operation":"getIssue"}` {
		t.Fatalf("%s %v", encoded, err)
	}
	var long ProbeOpsOperation
	if err := json.Unmarshal([]byte(`{"operation":"getIssue","paging":"token"}`), &long); err != nil {
		t.Fatal(err)
	}
	if long.Operation != "getIssue" || !long.Paging.Present || long.Paging.Value != "token" {
		t.Fatalf("%+v", long)
	}
	for _, refused := range []string{`7`, `true`, `null`, `["getIssue"]`, `{"paging":"token"}`, `{"operation":"getIssue","extra":1}`} {
		var value ProbeOpsOperation
		if err := json.Unmarshal([]byte(refused), &value); err == nil {
			t.Fatalf("%s decoded", refused)
		}
	}
	var batch ProbeOpsBatch
	if err := json.Unmarshal([]byte(`{"operations":["getIssue",{"operation":"list"}],"step":"careful"}`), &batch); err != nil {
		t.Fatal(err)
	}
	if batch.Operations[1].Operation != "list" || *batch.Step.Mode != ProbeOpsModeV0 {
		t.Fatalf("%+v", batch)
	}
	var step ProbeOpsStep
	if err := json.Unmarshal([]byte(`"reckless"`), &step); err == nil {
		t.Fatal("an undeclared variant decoded")
	}
	encoded, err = json.Marshal(batch)
	if err != nil || string(encoded) != `{"operations":[{"operation":"getIssue"},{"operation":"list"}],"step":{"mode":"careful"}}` {
		t.Fatalf("%s %v", encoded, err)
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
        "{}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        realization.declarations
    );
}

#[test]
fn typescript_types_declare_the_wire_union() {
    let realization = plan().typescript();
    let types = &realization.declarations;
    assert!(
        types.contains(
            "export type ProbeOpsOperation = string | {\n  \"operation\": string;\n  \"paging\"?: string;\n};"
        ),
        "{types}"
    );
    assert!(
        types.contains("export type ProbeOpsStep = ProbeOpsMode | {"),
        "{types}"
    );
    let report = serde_json::to_value(&realization.report).expect("report");
    let obligations = report["obligations"].as_array().expect("obligations");
    for at in ["/$defs/probe.ops.Operation", "/$defs/probe.ops.Step"] {
        assert!(
            obligations
                .iter()
                .any(|item| item["rule"] == "typescript_shorthand" && item["pointer"] == at),
            "{at}: {report}"
        );
    }
}
