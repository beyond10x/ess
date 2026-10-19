//! A committed suite can be drift-checked and reviewed as a change (beyond10x/ess#571,
//! beyond10x/ess#572).
//!
//! `ess verify conform synthesize --check` compares what `--out` holds with what the command would
//! write, writing nothing; `ess verify conform diff` says which scenarios two suites add, remove
//! and change, by construct, in text and in JSON.
use ess_cli::TemporaryDirectory;
use serde_json::Value;
use std::fmt::Write as _;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn synthesize(model: &Path, out: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "verify",
        "conform",
        "synthesize",
        "--path",
        model.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    ess(&args)
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn synthesize_check_reports_drift() {
    let dir = TemporaryDirectory::create("ess-suite-check").unwrap();
    let model = root().join("examples/billing");
    let suite = dir.join("suite.json");
    let missing = synthesize(&model, &suite, &["--check"]);
    assert_eq!(missing.status.code(), Some(1), "{}", text(&missing));
    assert!(text(&missing).contains("is missing"), "{}", text(&missing));
    assert!(!suite.exists(), "--check writes nothing");

    let written = synthesize(&model, &suite, &[]);
    assert!(written.status.success(), "{}", text(&written));
    let current = synthesize(&model, &suite, &["--check"]);
    assert!(current.status.success(), "{}", text(&current));
    assert!(text(&current).contains("current in"), "{}", text(&current));

    // The committed spelling is part of what is checked: a pretty suite is not the compact one.
    let compact = synthesize(&model, &suite, &["--check", "--compact"]);
    assert_eq!(compact.status.code(), Some(1), "{}", text(&compact));

    let mut edited = fs::read_to_string(&suite).unwrap();
    edited.push(' ');
    fs::write(&suite, &edited).unwrap();
    let drifted = synthesize(&model, &suite, &["--check"]);
    assert_eq!(drifted.status.code(), Some(1), "{}", text(&drifted));
    assert!(
        text(&drifted).contains("differs from the synthesized suite")
            && text(&drifted).contains("ess verify conform synthesize"),
        "{}",
        text(&drifted)
    );
    assert_eq!(
        fs::read_to_string(&suite).unwrap(),
        edited,
        "--check writes nothing"
    );

    // A coverage suite and a generated package are checked the same way.
    let coverage = dir.join("coverage.json");
    let five = ["--suite-format", "5"];
    assert!(synthesize(&model, &coverage, &five).status.success());
    let current = synthesize(&model, &coverage, &[five[0], five[1], "--check"]);
    assert!(current.status.success(), "{}", text(&current));
    let package = dir.join("go");
    let go = ["--target", "go"];
    assert!(synthesize(&model, &package, &go).status.success());
    let current = synthesize(&model, &package, &[go[0], go[1], "--check"]);
    assert!(current.status.success(), "{}", text(&current));
    let embedded = fs::read_dir(package.join("essconform"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|e| e == "json"))
        .unwrap();
    fs::write(&embedded, "{}\n").unwrap();
    let drifted = synthesize(&model, &package, &[go[0], go[1], "--check"]);
    assert_eq!(drifted.status.code(), Some(1), "{}", text(&drifted));
    assert!(text(&drifted).contains(embedded.file_name().unwrap().to_str().unwrap()));
}

const BEFORE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/one-time-caller/with-caller.yaml");

/// The next revision: an event nothing emits, no `Viewer`, and a second command with its event.
fn after() -> String {
    let mut text = BEFORE
        .replace(
            "  - name: keys.api.Viewer\n    attributes:\n      - {name: user_id, type: String}\n    may: []\n",
            "",
        )
        .replace(
            "commands:\n",
            "commands:\n  - name: keys.api.Touch\n    input:\n      - {name: key_id, type: Uuid}\n    outcomes:\n      - name: touched\n        emits: [keys.api.Touched]\n        payload:\n          keys.api.Touched: {key_id: input.key_id}\n",
        );
    for event in ["Audited", "Touched"] {
        write!(
            text,
            "  - name: keys.api.{event}\n    fields:\n      - {{name: key_id, type: Uuid}}\n"
        )
        .unwrap();
    }
    text
}

#[test]
fn conform_diff_groups_changes_by_construct() {
    let dir = TemporaryDirectory::create("ess-suite-diff").unwrap();
    let mut suites = Vec::new();
    for (name, model) in [("before", BEFORE.to_owned()), ("after", after())] {
        let spec = dir.join(name);
        fs::create_dir_all(&spec).unwrap();
        fs::write(spec.join("system.yaml"), model).unwrap();
        let suite = dir.join(format!("{name}.json"));
        let output = synthesize(&spec.join("system.yaml"), &suite, &[]);
        assert!(output.status.success(), "{}", text(&output));
        suites.push(suite);
    }
    let (from, to) = (suites[0].to_str().unwrap(), suites[1].to_str().unwrap());
    let json = ess(&[
        "verify", "conform", "diff", "--from", from, "--to", to, "--format", "json",
    ]);
    assert!(json.status.success(), "{}", text(&json));
    let diff: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(diff["format"], "ess-suite-diff/1");
    let constructs = diff["constructs"].as_object().unwrap();
    let denied = "keys.api.IssueKey/disclosure/issued/secret/denied";
    assert_eq!(
        constructs[denied]["removed"],
        serde_json::json!([
            "keys.api.IssueKey/disclosure/issued/secret/denied/keys.api.IssueKey/as/actor/keys.api.Viewer"
        ]),
        "{diff:#}"
    );
    assert!(
        constructs["keys.api.Touch/outcome"]["added"]
            .as_array()
            .unwrap()
            .contains(&Value::from("keys.api.Touch/outcome/touched")),
        "{diff:#}"
    );
    let issued = constructs["keys.api.IssueKey/outcome"]["changed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|changed| changed["scenario"] == "keys.api.IssueKey/outcome/issued")
        .unwrap_or_else(|| panic!("{diff:#}"));
    assert!(
        issued["added_steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|step| step["step"]["step"] == "expect_no_event"
                && step["step"]["event"] == "keys.api.Audited"),
        "{issued:#}"
    );
    let counts = &diff["counts"];
    assert!(counts["added"].as_u64().unwrap() >= 1);
    assert!(counts["removed"].as_u64().unwrap() >= 1);
    assert!(counts["changed"].as_u64().unwrap() >= 1);

    let plain = ess(&["verify", "conform", "diff", "--from", from, "--to", to]);
    assert!(plain.status.success(), "{}", text(&plain));
    let plain = String::from_utf8(plain.stdout).unwrap();
    assert!(
        plain.contains(&format!("{denied}: 1 added, 1 removed, 0 changed")),
        "{plain}"
    );
    assert!(
        plain.contains("  + keys.api.Touch/outcome/touched"),
        "{plain}"
    );
    assert!(
        plain.contains("  ~ keys.api.IssueKey/outcome/issued (steps"),
        "{plain}"
    );

    let same = ess(&[
        "verify", "conform", "diff", "--from", from, "--to", from, "--format", "json",
    ]);
    let same: Value = serde_json::from_slice(&same.stdout).unwrap();
    assert_eq!(same["constructs"], serde_json::json!({}));
    assert_eq!(same["counts"]["changed"], 0);

    let refused = ess(&[
        "verify",
        "conform",
        "diff",
        "--from",
        from,
        "--to",
        "/nonexistent.json",
    ]);
    assert_eq!(refused.status.code(), Some(1));
}

/// A suite is read as every other verb admits one (review 2026-10-19, B7): a document carrying a
/// key no suite format defines is refused by name, not diffed with the key dropped.
#[test]
fn conform_diff_refuses_a_document_admission_refuses() {
    let dir = TemporaryDirectory::create("ess-suite-diff-admission").unwrap();
    let spec = dir.join("model");
    fs::create_dir_all(&spec).unwrap();
    fs::write(spec.join("system.yaml"), BEFORE).unwrap();
    let suite = dir.join("suite.json");
    let output = synthesize(&spec.join("system.yaml"), &suite, &[]);
    assert!(output.status.success(), "{}", text(&output));
    let mut document: Value = serde_json::from_str(&fs::read_to_string(&suite).unwrap()).unwrap();
    document["extension"] = Value::Bool(true);
    let extended = dir.join("extended.json");
    fs::write(&extended, document.to_string()).unwrap();
    let refused = ess(&[
        "verify",
        "conform",
        "diff",
        "--from",
        suite.to_str().unwrap(),
        "--to",
        extended.to_str().unwrap(),
    ]);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused));
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("extended.json") && stderr.contains("extension"),
        "{stderr}"
    );
}
