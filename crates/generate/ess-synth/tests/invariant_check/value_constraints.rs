//! A newtype's declared constraints, held by the generated model decoder (beyond10x/ess#579).
//!
//! The JSON Schema projection publishes a newtype's `alphabet:` (`x-ess-alphabet`), its `prefix:`
//! and its `invariants:`. The model crate's wire decoder sits behind the generated `POST` routes,
//! so a value it admits reaches the application: every one of those constraints is checked there,
//! and a value that breaks one is refused with a decode error naming the path and the constraint.

use super::{cargo, ir, scratch, write};
use ess_synth::{synthesize_for, Target};
use std::fmt::Write as _;

/// Every printable ASCII character but the space, as a YAML double-quoted scalar.
fn printable() -> String {
    let alphabet: String = (0x21u8..=0x7e).map(char::from).collect();
    format!("{alphabet:?}")
}

/// The report's reproduction: a unit drawn from printable ASCII without the space, at least one
/// character long, recorded by a served command. A prefixed code and a bounded integer beside it.
fn model() -> String {
    format!(
        "format: ess/16
system: demo
version: v1
domain: demo.units
types:
  - name: demo.units.Unit
    kind: newtype
    of: String
    alphabet: {alphabet}
    invariants: [value.count >= 1]
  - name: demo.units.Code
    kind: newtype
    of: String
    prefix: \"u-\"
  - name: demo.units.Scale
    kind: newtype
    of: Integer
    invariants: [value >= 0]
events:
  - name: demo.units.Recorded
    fields:
      - {{name: unit, type: demo.units.Unit}}
      - {{name: code, type: demo.units.Code}}
      - {{name: scale, type: demo.units.Scale}}
commands:
  - name: demo.units.Record
    input:
      - {{name: unit, type: demo.units.Unit}}
      - {{name: code, type: demo.units.Code}}
      - {{name: scale, type: demo.units.Scale}}
    outcomes:
      - name: recorded
        emits: [demo.units.Recorded]
        payload:
          demo.units.Recorded: {{unit: input.unit, code: input.code, scale: input.scale}}
components:
  - component: units-service
    owns: {{domains: [demo.units]}}
    accepts: {{commands: [demo.units.Record]}}
    publishes: {{events: [demo.units.Recorded]}}
    reached_by: network
",
        alphabet = printable()
    )
}

/// Decodes one command input per argument and prints `ok` or the refusal's path and expectation.
const HARNESS: &str = r#"use demo_server::{json, wire};

fn main() {
    for input in std::env::args().skip(1) {
        let document = json::parse(&input).expect("the input parses");
        match wire::decode_command_demo_units_record(&document, "$") {
            Ok(_) => println!("ok"),
            Err(refusal) => println!("refused {} | {}", refusal.at, refusal.expected),
        }
    }
}
"#;

fn input(unit: &str, code: &str, scale: i64) -> String {
    format!(
        "{{\"unit\":{},\"code\":{},\"scale\":{scale}}}",
        json_text(unit),
        json_text(code)
    )
}

/// A JSON string literal, with every control character escaped.
fn json_text(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[test]
fn model_decoder_refuses_a_value_outside_the_alphabet() {
    let synthesis = synthesize_for(&ir(&model()), Target::Rust).expect("the model synthesizes");
    let root = scratch("value-constraints");
    let generated = root.join("generated");
    write(&synthesis, &generated);
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"constraint-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [dependencies]\ndemo-server = { path = \"../generated/crates/demo-server\" }\n\n\
         [workspace]\n",
    )
    .unwrap();
    std::fs::write(harness.join("src/main.rs"), HARNESS).unwrap();
    let cases = [
        input("m/s", "u-1", 0),
        input("m s", "u-1", 0),
        input("m\u{7f}", "u-1", 0),
        input("µm", "u-1", 0),
        input("", "u-1", 0),
        input("m\ts", "u-1", 0),
        input("m", "x-1", 0),
        input("m", "u-1", -1),
    ];
    let mut arguments = vec![
        "run",
        "--offline",
        "--quiet",
        "--target-dir",
        root.join("target").to_str().unwrap(),
        "--",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    arguments.extend(cases.iter().cloned());
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let run = cargo(&harness, &arguments);
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(run.status.success(), "the harness builds and runs");

    let alphabet = (0x21u8..=0x7e).map(char::from).collect::<String>();
    let outside = format!("refused $.unit | a value whose every character is one of `{alphabet}`");
    let expected = [
        "ok".to_owned(),
        outside.clone(),
        outside.clone(),
        outside.clone(),
        "refused $.unit | a value satisfying `value.count >= 1`".to_owned(),
        outside,
        "refused $.code | a value starting with `u-`".to_owned(),
        "refused $.scale | a value satisfying `value >= 0`".to_owned(),
    ];
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, expected, "{stdout}");
}
