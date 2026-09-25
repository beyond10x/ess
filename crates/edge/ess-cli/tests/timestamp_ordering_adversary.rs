//! Adversarial cases: `specify validate` and `verify conform synthesize` must agree on every guard
//! that orders `Timestamp` values (beyond10x/ess#74). When validate accepts, synthesize must
//! witness every outcome with no refusal; when validate refuses, synthesize must refuse too.

use std::path::{Path, PathBuf};
use std::process::Command;

const WINDOW: &str = include_str!("fixtures/room-booking/window.yaml");
const GUARD: &str = "when: window.ends_at > window.starts_at";

fn ess(arguments: &[&str], model: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
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
    let dir = std::env::temp_dir().join(format!(
        "ess-timestamp-adversary-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("system.yaml");
    std::fs::write(&path, text).unwrap();
    path
}

fn agree(name: &str, text: &str) {
    let path = model(name, text);
    let (validate_code, validated) = ess(&["specify", "validate"], &path);
    let (synthesize_code, synthesized) = ess(&["verify", "conform", "synthesize"], &path);
    if validate_code == Some(0) {
        assert!(
            synthesize_code == Some(0)
                && synthesized.contains(" 0 refusal(s)")
                && !synthesized.contains("refused:"),
            "validate accepted {name} and synthesize refused it:\n{synthesized}"
        );
    } else {
        assert_ne!(
            synthesize_code,
            Some(0),
            "validate refused {name} ({validated}) and synthesize accepted it:\n{synthesized}"
        );
    }
}

/// A Timestamp ordered against a String member: validate sees two texts; synthesize can only
/// order text that is a declared Timestamp on both sides as an instant.
#[test]
fn adversary_timestamp_ordered_against_a_string_member() {
    let text = WINDOW
        .replace(
            "      - {name: ends_at, type: Timestamp}\n",
            "      - {name: ends_at, type: Timestamp}\n      - {name: label, type: String}\n",
        )
        .replace(GUARD, "when: window.ends_at > window.label");
    assert_ne!(text, WINDOW);
    agree("string-member", &text);
}

/// A newtype over plain Timestamp (no clock contract) is still ordered by instant.
#[test]
fn adversary_newtype_of_timestamp_is_ordered_by_both_verbs() {
    let text = WINDOW
        .replace(
            "types:\n",
            "types:\n  - name: rooms.booking.Moment\n    kind: newtype\n    of: Timestamp\n",
        )
        .replace(
            "{name: starts_at, type: Timestamp}",
            "{name: starts_at, type: rooms.booking.Moment}",
        )
        .replace(
            "{name: ends_at, type: Timestamp}",
            "{name: ends_at, type: rooms.booking.Moment}",
        );
    agree("newtype", &text);
}

/// An Optional Timestamp member ordered against a required one.
#[test]
fn adversary_optional_timestamp_member_is_ordered_by_both_verbs() {
    let text = WINDOW.replace(
        "{name: starts_at, type: Timestamp}",
        "{name: starts_at, type: 'Optional<Timestamp>'}",
    );
    agree("optional", &text);
}

/// Equality of two Timestamp members beside the ordering: an outcome that needs them equal.
#[test]
fn adversary_equal_and_ordered_timestamps_are_witnessed() {
    let text = WINDOW.replace(
        "      - name: refused\n",
        "      - name: same\n        when: window.ends_at == window.starts_at\n        error: rooms.booking.EmptyRange\n      - name: refused\n",
    );
    assert_ne!(text, WINDOW);
    agree("equal", &text);
}

/// `>=` against an instant literal written with a non-UTC offset.
#[test]
fn adversary_offset_literal_under_ge_is_witnessed() {
    agree(
        "offset-ge",
        &WINDOW.replace(
            GUARD,
            "when: window.ends_at >= \"2020-01-01T00:00:00.5-00:30\"",
        ),
    );
}

/// A literal on the left of the ordering.
#[test]
fn adversary_literal_on_the_left_is_witnessed() {
    agree(
        "literal-left",
        &WINDOW.replace(GUARD, "when: \"2020-01-01T00:00:00Z\" < window.ends_at"),
    );
}

/// The last representable RFC 3339 second, written as an upper sentinel.
#[test]
fn adversary_end_of_time_sentinel_is_witnessed() {
    agree(
        "end-of-time",
        &WINDOW.replace(GUARD, "when: window.ends_at < \"9999-12-31T23:59:59Z\""),
    );
}

/// A bare word naming a member of a struct, not a root field: validate lets it pass as a text
/// literal only if it is not ordered against a Timestamp.
#[test]
fn adversary_bare_word_naming_a_nested_member() {
    agree(
        "nested-bare",
        &WINDOW.replace(GUARD, "when: window.ends_at > starts_at"),
    );
}
