//! `specify validate` and `verify conform synthesize` agree on a guard that orders two
//! `Timestamp` values (beyond10x/ess#74).
//!
//! The trial wrote `ends_at > starts_at` over two top-level inputs. A right-hand side without a
//! dot is a text literal, so the guard compared `ends_at` with the text `"starts_at"`: validate
//! passed it and synthesize could not order it. Validate now refuses that spelling and names the
//! one that works, and synthesize orders two RFC 3339 instants.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLAT: &str = include_str!("fixtures/room-booking/flat.yaml");
const WINDOW: &str = include_str!("fixtures/room-booking/window.yaml");

fn dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-timestamp-ordering-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn ess(arguments: &[&str], model: &Path) -> (Option<i32>, String) {
    let output: Output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .arg("--path")
        .arg(model)
        .output()
        .unwrap();
    (
        output.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn model(name: &str, text: &str) -> PathBuf {
    let path = dir(name).join("system.yaml");
    std::fs::write(&path, text).unwrap();
    path
}

/// Both verbs accept the model, and synthesize witnesses both outcomes with no refusal.
fn both_accept(name: &str, text: &str) {
    let path = model(name, text);
    let (code, output) = ess(&["specify", "validate"], &path);
    assert_eq!(code, Some(0), "validate refused {name}: {output}");
    let (code, output) = ess(&["verify", "conform", "synthesize"], &path);
    assert_eq!(code, Some(0), "synthesize failed {name}: {output}");
    assert!(
        output.contains("2 scenario(s) (0 authored), 0 refusal(s)") && !output.contains("refused:"),
        "synthesize refused what validate accepted ({name}): {output}"
    );
}

/// Both verbs refuse the model, with the same explanation.
fn both_refuse(name: &str, text: &str, needle: &str) {
    let path = model(name, text);
    let (code, validated) = ess(&["specify", "validate"], &path);
    assert_ne!(code, Some(0), "validate accepted {name}: {validated}");
    assert!(validated.contains(needle), "{name}: {validated}");
    let (code, synthesized) = ess(&["verify", "conform", "synthesize"], &path);
    assert_ne!(code, Some(0), "synthesize accepted {name}: {synthesized}");
    assert!(synthesized.contains(needle), "{name}: {synthesized}");
}

#[test]
fn two_timestamp_fields_in_one_struct_are_ordered_by_both_verbs() {
    both_accept("window", WINDOW);
}

#[test]
fn a_timestamp_ordered_against_an_instant_literal_is_accepted_by_both_verbs() {
    both_accept(
        "literal",
        &FLAT.replace(
            "when: ends_at > starts_at",
            "when: ends_at > \"2020-01-01T12:00:00+01:00\"",
        ),
    );
}

#[test]
fn a_bare_word_naming_an_input_field_is_refused_by_both_verbs() {
    assert!(FLAT.contains("when: ends_at > starts_at"));
    both_refuse("flat", FLAT, "reads `starts_at` as the text literal");
    both_refuse("flat", FLAT, "window.ends_at > window.starts_at");
}

#[test]
fn a_timestamp_ordered_against_text_that_is_no_instant_is_refused_by_both_verbs() {
    both_refuse(
        "not-an-instant",
        &FLAT.replace("when: ends_at > starts_at", "when: ends_at > tomorrow"),
        "not an RFC 3339 instant",
    );
}
