//! `undeclared_fields: ignored | refused` (`ess/24`, beyond10x/ess#500): `ess specify validate`
//! admits the key on a `kind: struct` type and on a command with a `response:`, and refuses it by
//! name, at its own line, everywhere else and under an earlier format.
//!
//! The key is the opt-in openness a protocol needs when its readers must ignore members they do
//! not recognise. A refusal that stopped the whole file as an unknown field would carry no line
//! and hide every other refusal, so each one here is asserted as a located diagnostic.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

/// A catalog whose `PlaceOrder` returns a response reaching the struct `Extension`, and whose
/// `NoteOrder` returns none. `{format}` is the header's major.
const MODEL: &str = "\
format: ess/{format}
system: catalog
version: v1
summary: A catalog whose order command returns a response.
domain: catalog.orders
types:
  - name: catalog.orders.Extension
    kind: struct
    fields:
      - name: vendor
        type: String
  - name: catalog.orders.OrderRef
    kind: newtype
    of: String
actors:
  - name: catalog.orders.Buyer
    may:
      - catalog.orders.PlaceOrder
      - catalog.orders.NoteOrder
commands:
  - name: catalog.orders.PlaceOrder
    input:
      - name: item
        type: String
    response:
      - name: order_ref
        type: catalog.orders.OrderRef
      - name: extension
        type: catalog.orders.Extension
    outcomes:
      - name: placed
        returns: true
        summary: The order is accepted and its reference returned.
  - name: catalog.orders.NoteOrder
    input:
      - name: item
        type: String
    outcomes:
      - name: noted
        emits: [catalog.orders.OrderNoted]
        payload:
          catalog.orders.OrderNoted:
            item: input.item
      - name: refused
        when: input.item == \"\"
        error: catalog.orders.OutOfStock
events:
  - name: catalog.orders.OrderNoted
    fields:
      - name: item
        type: String
errors:
  - name: catalog.orders.OutOfStock
    summary: The item is not in stock.
";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The model under `ess/<format>`, with `undeclared_fields: <value>` written on each declaration
/// `on` names, as the key right after its `name:` line.
fn model(format: u32, on: &[(&str, &str)]) -> String {
    let mut text = MODEL.replace("{format}", &format.to_string());
    for (declaration, value) in on {
        let line = format!("  - name: {declaration}\n");
        assert!(
            text.contains(&line),
            "`{declaration}` is declared in the model"
        );
        text = text.replacen(&line, &format!("{line}    undeclared_fields: {value}\n"), 1);
    }
    text
}

/// What `ess specify validate --format json` printed for `text`.
struct Validated {
    code: Option<i32>,
    problems: Vec<String>,
    diagnostics: Vec<Value>,
    output: String,
    text: String,
}

impl Validated {
    /// The diagnostic located at `path`, which must be the only one there.
    fn diagnostic(&self, path: &str) -> &Value {
        let found: Vec<&Value> = self
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic["span"]["path"] == path)
            .collect();
        assert_eq!(
            found.len(),
            1,
            "one diagnostic at `{path}`:\n{}",
            self.output
        );
        found[0]
    }

    /// Asserts the refusal of the key at `path`: the problem carries `class`, the message names
    /// `needle`, and the diagnostic is located on the key's own line.
    fn refuses(&self, path: &str, class: &str, needle: &str) {
        assert_eq!(self.code, Some(1), "validate refuses:\n{}", self.output);
        let prefix = format!("[{class}] {path}: ");
        let problems: Vec<&String> = self
            .problems
            .iter()
            .filter(|problem| problem.starts_with(&prefix))
            .collect();
        assert_eq!(
            problems.len(),
            1,
            "one `{class}` problem at `{path}`:\n{}",
            self.output
        );
        assert!(
            problems[0].contains(needle),
            "the refusal names `{needle}`: {}",
            problems[0]
        );
        let diagnostic = self.diagnostic(path);
        let line = diagnostic["span"]["located"]["line"]
            .as_u64()
            .unwrap_or_else(|| panic!("`{path}` carries a line:\n{}", self.output));
        assert_eq!(
            line,
            key_line(&self.text),
            "`{path}` is located at the key's own line:\n{}",
            self.output
        );
    }
}

/// The 1-based line the one `undeclared_fields:` of `text` is written on.
fn key_line(text: &str) -> u64 {
    let lines: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("undeclared_fields:"))
        .map(|(index, _)| index + 1)
        .collect();
    assert_eq!(lines.len(), 1, "one key in the model");
    u64::try_from(lines[0]).expect("a line number")
}

fn validate(text: &str) -> Validated {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "validate-undeclared-fields-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let file = directory.join("catalog.yaml");
    std::fs::write(&file, text).expect("the specification is written");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["specify", "validate", "--format", "json", "--path"])
        .arg(&file)
        .output()
        .expect("the `ess` binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let report: Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!(
            "a JSON report: {error}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let problems = report["problems"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|problem| problem.as_str().expect("a problem is text").to_owned())
        .collect();
    let diagnostics = report["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    Validated {
        code: output.status.code(),
        problems,
        diagnostics,
        output: stdout,
        text: text.to_owned(),
    }
}

#[test]
fn the_model_without_the_key_is_valid_under_ess_24() {
    let validated = validate(&model(24, &[]));
    assert_eq!(validated.code, Some(0), "{}", validated.output);
    assert!(validated.problems.is_empty(), "{}", validated.output);
}

#[test]
fn the_key_is_admitted_on_a_command_with_a_response_and_on_a_struct_under_ess_24() {
    for value in ["ignored", "refused"] {
        let validated = validate(&model(
            24,
            &[
                ("catalog.orders.PlaceOrder", value),
                ("catalog.orders.Extension", value),
            ],
        ));
        assert_eq!(
            validated.code,
            Some(0),
            "`undeclared_fields: {value}` validates:\n{}",
            validated.output
        );
        assert!(validated.problems.is_empty(), "{}", validated.output);
    }
}

#[test]
fn the_key_on_a_command_below_ess_24_is_refused_naming_ess_24() {
    let validated = validate(&model(23, &[("catalog.orders.PlaceOrder", "ignored")]));
    validated.refuses(
        "command.catalog.orders.PlaceOrder.undeclared_fields",
        "unsupported_format_version",
        "ess/24",
    );
}

#[test]
fn the_key_on_a_struct_below_ess_24_is_refused_naming_ess_24() {
    let validated = validate(&model(23, &[("catalog.orders.Extension", "ignored")]));
    validated.refuses(
        "type.catalog.orders.Extension.undeclared_fields",
        "unsupported_format_version",
        "ess/24",
    );
}

#[test]
fn the_key_on_a_command_without_a_response_is_a_missing_declaration() {
    let validated = validate(&model(24, &[("catalog.orders.NoteOrder", "ignored")]));
    validated.refuses(
        "command.catalog.orders.NoteOrder.undeclared_fields",
        "missing_declaration",
        "response:",
    );
}

#[test]
fn the_key_on_an_event_is_refused_by_name() {
    let validated = validate(&model(24, &[("catalog.orders.OrderNoted", "ignored")]));
    validated.refuses(
        "event.catalog.orders.OrderNoted.undeclared_fields",
        "unsupported_construct",
        "undeclared_fields:",
    );
}

#[test]
fn the_key_on_an_error_is_refused_by_name() {
    let validated = validate(&model(24, &[("catalog.orders.OutOfStock", "ignored")]));
    validated.refuses(
        "error.catalog.orders.OutOfStock.undeclared_fields",
        "unsupported_construct",
        "undeclared_fields:",
    );
}

#[test]
fn the_key_on_a_type_that_is_no_struct_is_refused_by_name() {
    let validated = validate(&model(24, &[("catalog.orders.OrderRef", "ignored")]));
    validated.refuses(
        "type.catalog.orders.OrderRef.undeclared_fields",
        "unsupported_construct",
        "newtype",
    );
}

#[test]
fn the_key_on_an_actor_is_refused_by_name() {
    let validated = validate(&model(24, &[("catalog.orders.Buyer", "ignored")]));
    validated.refuses(
        "actor.catalog.orders.Buyer.undeclared_fields",
        "unsupported_construct",
        "undeclared_fields:",
    );
}

#[test]
fn a_value_other_than_ignored_or_refused_is_refused_naming_the_key() {
    let validated = validate(&model(24, &[("catalog.orders.Extension", "open")]));
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    assert!(
        validated
            .problems
            .iter()
            .any(|problem| problem.contains("undeclared_fields") && problem.contains("open")),
        "the refusal names the key and the value:\n{}",
        validated.output
    );
}
