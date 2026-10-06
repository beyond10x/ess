//! `ess specify formats`: which `format: ess/N` headers this build admits, and what each added
//! (beyond10x/ess#460).
//!
//! An author, often an agent with no network, asks the `ess` it is about to run. Before this the
//! only answer was the refusal of a header that was too high.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_domain::system::SUPPORTED_FORMATS;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs")
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "exit {:?}\n{}{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("UTF-8")
}

fn is_release(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn newest() -> u32 {
    *SUPPORTED_FORMATS.iter().max().expect("a supported format")
}

fn json(arguments: &[&str]) -> Vec<serde_json::Value> {
    let text = stdout(&ess(arguments));
    match serde_json::from_str(&text).expect("JSON") {
        serde_json::Value::Array(rows) => rows,
        other => panic!("not an array: {other}"),
    }
}

fn formats(rows: &[serde_json::Value]) -> Vec<String> {
    rows.iter()
        .map(|row| row["format"].as_str().expect("format is text").to_owned())
        .collect()
}

#[test]
fn specify_formats_lists_every_supported_format_newest_marked() {
    let text = stdout(&ess(&["specify", "formats"]));
    let headers: Vec<&str> = text
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with(char::is_whitespace))
        .collect();
    let expected: Vec<String> = SUPPORTED_FORMATS
        .iter()
        .map(|n| format!("ess/{n}"))
        .collect();
    let listed: Vec<&str> = headers
        .iter()
        .map(|line| line.split_whitespace().next().unwrap_or_default())
        .collect();
    assert_eq!(
        listed, expected,
        "one unindented line per major, ascending\n{text}"
    );
    let releases: Vec<serde_json::Value> = json(&["specify", "formats", "--format", "json"]);
    for (line, row) in headers.iter().zip(&releases) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let release = words.get(1).copied().unwrap_or_default();
        match row["release"].as_str() {
            Some(version) => assert_eq!(release, version, "{line}"),
            None => assert_eq!(release, "unreleased", "{line}"),
        }
        assert!(
            release == "unreleased" || is_release(release),
            "`{release}` is neither a release nor `unreleased`: {line}"
        );
        let marked = words.get(2) == Some(&"newest");
        assert_eq!(
            marked,
            words[0] == format!("ess/{}", newest()),
            "only the highest is marked newest: {line}"
        );
        assert!(words.len() <= 3, "{line}");
    }
    assert_eq!(
        headers
            .iter()
            .filter(|line| line.contains("newest"))
            .count(),
        1,
        "{text}"
    );
}

#[test]
fn specify_formats_json_carries_release_and_additions() {
    let rows = json(&["specify", "formats", "--format", "json"]);
    let expected: Vec<String> = SUPPORTED_FORMATS
        .iter()
        .map(|n| format!("ess/{n}"))
        .collect();
    assert_eq!(formats(&rows), expected);
    for row in &rows {
        let object = row.as_object().expect("each row is an object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["added", "format", "newest", "release", "stricter"],
            "{row}"
        );
        match &row["release"] {
            serde_json::Value::Null => {}
            serde_json::Value::String(release) => assert!(is_release(release), "{row}"),
            other => panic!("release is {other}"),
        }
        assert_eq!(
            row["newest"].as_bool(),
            Some(row["format"] == format!("ess/{}", newest())),
            "{row}"
        );
        let added = row["added"].as_array().expect("added is a list");
        assert!(!added.is_empty(), "{row}");
        for item in added
            .iter()
            .chain(row["stricter"].as_array().expect("stricter is a list"))
        {
            assert!(
                item.as_str().is_some_and(|text| !text.trim().is_empty()),
                "{row}"
            );
        }
    }
    let yaml: serde_json::Value =
        serde_yaml::from_str(&stdout(&ess(&["specify", "formats", "--format", "yaml"])))
            .expect("YAML");
    assert_eq!(
        yaml,
        serde_json::Value::Array(rows),
        "yaml and json carry the same rows"
    );
}

#[test]
fn specify_formats_since_lists_only_later_formats() {
    let rows = json(&[
        "specify", "formats", "--since", "ess/20", "--format", "json",
    ]);
    let expected: Vec<String> = (21..=newest()).map(|n| format!("ess/{n}")).collect();
    assert_eq!(formats(&rows), expected);
    let text = stdout(&ess(&["specify", "formats", "--since", "ess/20"]));
    assert!(
        !text.lines().any(|line| line.starts_with("ess/20 ")),
        "{text}"
    );
    assert!(
        text.lines().any(|line| line.starts_with("ess/21 ")),
        "{text}"
    );
    for refused in ["ess/0", "x"] {
        let output = ess(&["specify", "formats", "--since", refused]);
        // clap's usage error: exit 2, naming the value and the flag, and nothing on stdout.
        assert_eq!(output.status.code(), Some(2), "--since {refused}");
        assert!(output.stdout.is_empty(), "--since {refused}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.starts_with(&format!(
                "error: invalid value '{refused}' for '--since <ess/N>'"
            )),
            "--since {refused}: {stderr}"
        );
    }
}

#[test]
fn unsupported_format_hint_names_specify_formats() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("probe.yaml");
    std::fs::write(
        &path,
        "format: ess/99\nsystem: probe\nversion: v1\ndomain: probe.core\n",
    )
    .expect("write the probe");
    let output = ess(&[
        "specify",
        "validate",
        "--path",
        path.to_str().expect("UTF-8 path"),
    ]);
    assert!(!output.status.success(), "ess/99 is refused");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let refusal = printed
        .lines()
        .find(|line| line.contains("[unsupported_format_version]"))
        .unwrap_or_else(|| panic!("no unsupported_format_version refusal:\n{printed}"));
    assert!(
        refusal.contains("`ess specify formats`"),
        "the hint names the command: {refusal}"
    );
}

#[test]
fn cli_reference_lists_specify_formats() {
    let page = std::fs::read_to_string(workspace_root().join("website/docs/reference/cli.md"))
        .expect("the CLI reference page");
    assert!(
        page.lines().any(|line| line == "#### `ess specify formats`"),
        "website/docs/reference/cli.md has no section for `ess specify formats`; run `cargo xtask cli-reference`"
    );
}
