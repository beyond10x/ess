//! `ess specify validate` on a directory names the file a refusal was read from, including when
//! the refused declaration's name is declared in two files
//! (`story:a-refusal-records-the-document-it-was-read-from`).
//!
//! The file used to be recovered by searching every file for the declaration's name, and a name
//! declared twice is in two files, so the refusal was cited against `<document>`: the whole
//! specification, not a file. Assembly now records which file it was reading when it refused, and
//! which file the declaration it kept came from, and the refused path reads that record.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

/// The header a specification directory must hold in `system.yaml`, which sorts after both copies.
const SYSTEM: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.probe]
";

/// The types, and the entity, whose invariant names a variant `Channel` does not have.
const DUP_A: &str = "\
domain: shop.probe
types:
  - name: shop.probe.OrderId
    kind: newtype
    of: Uuid
  - name: shop.probe.Channel
    kind: enum
    variants: [Email, Post]
entities:
  - name: shop.probe.Order
    identity:
      name: order_id
      type: shop.probe.OrderId
    fields:
      - name: channel
        type: shop.probe.Channel
    invariants:
      - channel == Fax
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

/// The same entity again, without the invariant: the copy-paste `DuplicateDeclaration` is for.
const DUP_B: &str = "\
domain: shop.probe
entities:
  - name: shop.probe.Order
    identity:
      name: order_id
      type: shop.probe.OrderId
    fields:
      - name: channel
        type: shop.probe.Channel
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

const INVARIANT: &str = "entity shop.probe.Order.invariants[0]";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// A fresh directory holding `system.yaml`, `dup_a.yaml` and `dup_b.yaml`.
fn two_files() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "validate-names-the-file-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    std::fs::write(directory.join("system.yaml"), SYSTEM).expect("system.yaml is written");
    std::fs::write(directory.join("dup_a.yaml"), DUP_A).expect("dup_a.yaml is written");
    std::fs::write(directory.join("dup_b.yaml"), DUP_B).expect("dup_b.yaml is written");
    directory
}

fn validate(directory: &Path, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.current_dir(root()).args(["specify", "validate"]);
    if json {
        command.args(["--format", "json"]);
    }
    command
        .arg("--path")
        .arg(directory)
        .output()
        .expect("the `ess` binary runs")
}

/// The 1-based line of `text` whose trimmed text ends with `needle`, which occurs once.
fn line_ending_with(text: &str, needle: &str) -> u64 {
    let lines: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_end().ends_with(needle))
        .map(|(index, _)| index + 1)
        .collect();
    assert_eq!(lines.len(), 1, "`{needle}` ends one line");
    u64::try_from(lines[0]).expect("a small line number")
}

#[test]
fn the_json_report_names_the_file_an_invariant_refusal_was_read_from() {
    let output = validate(&two_files(), true);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_ne!(
        output.status.code(),
        Some(0),
        "the specification is refused:\n{stdout}"
    );
    let report: Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!(
            "a JSON report: {error}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let diagnostics = report["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let found: Vec<&Value> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic["span"]["path"] == INVARIANT)
        .collect();
    assert_eq!(found.len(), 1, "one diagnostic at `{INVARIANT}`:\n{stdout}");
    let span = &found[0]["span"];
    let source = span["source"].as_str().unwrap_or_default();
    assert!(
        source.ends_with("dup_a.yaml"),
        "the invariant is read from dup_a.yaml, whose declaration is the one kept; cited \
         `{source}`:\n{stdout}"
    );
    assert_eq!(
        span["located"]["line"].as_u64(),
        Some(line_ending_with(DUP_A, "name: shop.probe.Order")),
        "the entity's declaration in dup_a.yaml:\n{stdout}"
    );
}

#[test]
fn the_text_report_names_the_file_an_invariant_refusal_was_read_from() {
    let output = validate(&two_files(), false);
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(
        output.status.code(),
        Some(0),
        "the specification is refused:\n{printed}"
    );
    // A located span prints as `<file>:<line>:<column>` and an unlocated one as `<file> (<path>)`,
    // so the invariant's diagnostic is found by its message, the last line that carries it, and
    // its span is the next `-->` line.
    let lines: Vec<&str> = printed.lines().collect();
    let message = lines
        .iter()
        .rposition(|line| line.contains("compares `channel` to `Fax`"))
        .unwrap_or_else(|| panic!("the invariant is refused:\n{printed}"));
    let span = lines[message..]
        .iter()
        .find(|line| line.trim_start().starts_with("-->"))
        .unwrap_or_else(|| panic!("the invariant's diagnostic has a span:\n{printed}"));
    let line = line_ending_with(DUP_A, "name: shop.probe.Order");
    assert!(
        span.contains(&format!("dup_a.yaml:{line}:")) && !span.contains("<document>"),
        "the span names dup_a.yaml at the entity's declaration, not the whole specification: \
         `{span}`\n{printed}"
    );
}
