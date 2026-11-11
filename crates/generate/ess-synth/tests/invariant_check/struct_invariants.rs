//! A struct's declared invariants, held by the generated decoders
//! (`story:generated-decoders-check-struct-invariants`).
//!
//! The JSON Schema projection publishes a struct's `invariants:` (`x-ess-invariants`) and lowers an
//! Integer bound to `minimum`. The decoders behind the generated `POST` routes check them: a value
//! that breaks one is refused with a decode error naming the path and the invariant, in the words
//! a newtype's constraint check uses.

use super::{cargo, ir, scratch, write};
use ess_synth::{synthesize_for, Target};
use std::fmt::Write as _;

const MODEL: &str = "format: ess/22
system: probe
version: v1
domain: probe.range
types:
  - name: probe.range.Window
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
    invariants:
      - low >= 0
events:
  - name: probe.range.WindowSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.range.SetWindow
    input:
      - {name: window, type: probe.range.Window}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.range.WindowSet]
        payload:
          probe.range.WindowSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.range]}
    accepts: {commands: [probe.range.SetWindow]}
    publishes: {events: [probe.range.WindowSet]}
    reached_by: network
";

/// The command inputs both decoders read: inside the invariant, then outside it.
fn cases() -> [String; 2] {
    [
        r#"{"window":{"low":0,"high":1},"note":"x"}"#.to_owned(),
        r#"{"window":{"low":-1,"high":1},"note":"x"}"#.to_owned(),
    ]
}

/// The verdict on each of [`cases`], the same from both generated decoders.
fn expected() -> [&'static str; 2] {
    ["ok", "refused $.window | a value satisfying `low >= 0`"]
}

const RUST_HARNESS: &str = r#"use probe_server::{json, wire};

fn main() {
    for input in std::env::args().skip(1) {
        let document = json::parse(&input).expect("the input parses");
        match wire::decode_command_probe_range_set_window(&document, "$") {
            Ok(_) => println!("ok"),
            Err(refusal) => println!("refused {} | {}", refusal.at, refusal.expected),
        }
    }
}
"#;

const GO_HARNESS: &str = r#"package server

import (
	"errors"
	"fmt"
	"testing"
)

func TestStructInvariants(t *testing.T) {
	for _, input := range []string{
{cases}	} {
		document, refused := readJSON([]byte(input))
		if refused != nil {
			t.Fatalf("%s does not parse: %s", input, refused.body)
		}
		_, err := decodeCommandProbeRangeSetWindow(document, "$")
		var refusal DecodeError
		switch {
		case err == nil:
			fmt.Println("case\tok")
		case errors.As(err, &refusal):
			fmt.Printf("case\trefused %s | %s\n", refusal.At, refusal.Expected)
		default:
			t.Fatalf("%s: %v", input, err)
		}
	}
}
"#;

#[test]
fn model_decoder_refuses_a_struct_that_breaks_its_invariant() {
    let synthesis = synthesize_for(&ir(MODEL), Target::Rust).expect("the model synthesizes");
    let root = scratch("struct-invariants");
    let generated = root.join("generated");
    write(&synthesis, &generated);
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"struct-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nprobe-server = { path = \"../generated/crates/probe-server\" }\n\n\
         [workspace]\n",
    )
    .unwrap();
    std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).unwrap();
    let mut arguments = vec![
        "run".to_owned(),
        "--offline".to_owned(),
        "--quiet".to_owned(),
        "--target-dir".to_owned(),
        root.join("target").to_str().unwrap().to_owned(),
        "--".to_owned(),
    ];
    arguments.extend(cases());
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let run = cargo(&harness, &arguments);
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(run.status.success(), "the harness builds and runs");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, expected(), "{stdout}");
}

#[test]
fn go_server_decoder_refuses_a_struct_that_breaks_its_invariant() {
    if !std::process::Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        eprintln!("no Go toolchain on this machine; the Go struct decoder is unchecked here");
        return;
    }
    let synthesis = synthesize_for(&ir(MODEL), Target::Go).expect("the model synthesizes");
    let root = scratch("go-struct-invariants");
    write(&synthesis, &root);
    let literals = cases().iter().fold(String::new(), |mut out, case| {
        let _ = writeln!(out, "\t\t`{case}`,");
        out
    });
    std::fs::write(
        root.join("server/zz_struct_invariants_test.go"),
        GO_HARNESS.replace("{cases}", &literals),
    )
    .unwrap();
    let unformatted = super::go_tool(&root, "gofmt", &["-l", "."]);
    let vet = super::go_tool(&root, "go", &["vet", "./..."]);
    let run = super::go_tool(
        &root,
        "go",
        &[
            "test",
            "-count=1",
            "-run",
            "TestStructInvariants",
            "-v",
            "./server/",
        ],
    );
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        unformatted.status.success() && unformatted.stdout.is_empty(),
        "the generated Go is gofmt-clean: {}",
        String::from_utf8_lossy(&unformatted.stdout)
    );
    assert!(vet.status.success(), "the generated Go is vet clean");
    assert!(
        run.status.success(),
        "the harness builds and runs: {stdout}"
    );
    let lines: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("case\t"))
        .collect();
    assert_eq!(lines, expected(), "{stdout}");
}

/// A struct that declares no invariant carries no check in either target.
#[test]
fn a_struct_without_invariants_carries_no_check() {
    let bare = MODEL.replace("    invariants:\n      - low >= 0\n", "");
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir(&bare), target).expect("the model synthesizes");
        for (path, artifact) in &synthesis.artifacts {
            assert!(
                !artifact.contents.contains("broken_constraint")
                    && !artifact.contents.contains("a value satisfying"),
                "`{path}` carries a constraint check for a struct that declares none"
            );
        }
    }
}
