//! The `ui` area and `ess generate ui`: the ess-ui crates, reached through the `ess` binary.
//!
//! Each case runs the shipped binary against `examples/partner-portal/ui.yaml` and holds what it
//! prints and writes to what the crate it wraps produces for the same input, so the command is
//! shown to be that crate's entry point and not a second implementation of it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The workspace root, found by walking up rather than by counting `..`.
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

/// One run of the built binary, from the workspace root so the example path resolves.
fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs")
}

const EXAMPLE: &str = "examples/partner-portal/ui.yaml";

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("output is UTF-8")
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

#[test]
fn ui_load_prints_the_summary_of_a_document_that_loads() {
    let output = ess(&["ui", "load", "--path", EXAMPLE]);
    let summary = ess_ui::check(&workspace_root().join(EXAMPLE)).expect("the example loads");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), format!("{summary}\n"));
    assert!(output.stderr.is_empty(), "{}", text(&output.stderr));
}

#[test]
fn ui_load_refuses_a_document_that_does_not_load_naming_the_node() {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let document = scratch.path().join("broken.yaml");
    std::fs::write(&document, "format: ess-ui/1\napp: portal\npages: 3\n").expect("writable");
    let refusal = ess_ui::check(&document).expect_err("the document is refused");

    let output = ess(&["ui", "load", "--path", utf8(&document)]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stderr),
        format!("{}: {}\n", refusal.path(), refusal.message())
    );
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
}

#[test]
fn ui_load_without_a_path_is_a_usage_error() {
    let output = ess(&["ui", "load"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("--path"));
}

#[test]
fn ui_docs_writes_the_reference_and_checks_it() {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    for (name, render) in [
        (
            "reference.html",
            ess_ui_docs::render_html_str as fn(&str) -> _,
        ),
        ("reference.md", ess_ui_docs::render_markdown_str),
    ] {
        let out = scratch.path().join(name);
        let expected = render(ess_ui::SCHEMA).expect("the embedded schema renders");

        let written = ess(&["ui", "docs", "--out", utf8(&out)]);
        assert_eq!(written.status.code(), Some(0), "{}", text(&written.stderr));
        assert_eq!(
            text(&written.stdout),
            format!("wrote {} ({} bytes)\n", out.display(), expected.len())
        );
        assert_eq!(std::fs::read_to_string(&out).expect("written"), expected);

        let checked = ess(&["ui", "docs", "--out", utf8(&out), "--check"]);
        assert_eq!(checked.status.code(), Some(0), "{}", text(&checked.stderr));
        assert_eq!(
            text(&checked.stdout),
            format!("{} is current\n", out.display())
        );
    }
}

#[test]
fn ui_docs_check_refuses_a_stale_reference() {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let out = scratch.path().join("reference.md");
    std::fs::write(&out, "stale\n").expect("writable");
    let output = ess(&["ui", "docs", "--out", utf8(&out), "--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(
        text(&output.stderr).contains("is not a fresh render of the schema"),
        "{}",
        text(&output.stderr)
    );
    assert_eq!(std::fs::read_to_string(&out).expect("readable"), "stale\n");
}

#[test]
fn generate_ui_react_writes_the_project_the_crate_renders() {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let out = scratch.path().join("portal");
    let document = workspace_root().join(EXAMPLE);
    let loaded = ess_ui::load_path(&document).expect("the example loads");
    let expected = ess_ui_react::render(&loaded, document.parent().expect("a parent"))
        .expect("the example renders");

    let output = ess(&[
        "generate",
        "ui",
        "--target",
        "react",
        "--path",
        EXAMPLE,
        "--out",
        utf8(&out),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        format!(
            "{} files written to {}\n",
            expected.files.len(),
            out.display()
        )
    );
    for (relative, contents) in &expected.files {
        let written = std::fs::read_to_string(out.join(relative))
            .unwrap_or_else(|error| panic!("{relative} was not written: {error}"));
        assert_eq!(&written, contents, "{relative} differs");
    }
}

#[test]
fn generate_ui_offers_react_as_its_only_target() {
    let unknown = ess(&[
        "generate", "ui", "--target", "vue", "--path", EXAMPLE, "--out", "unused",
    ]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        text(&unknown.stderr).contains("[possible values: react]"),
        "{}",
        text(&unknown.stderr)
    );
    let missing = ess(&["generate", "ui", "--path", EXAMPLE, "--out", "unused"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(text(&missing.stderr).contains("--target"));
}

#[test]
fn ui_run_tui_offers_its_help() {
    let output = ess(&["ui", "run", "--help"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let help = text(&output.stdout);
    for option in ["--tui", "--path", "--fixtures"] {
        assert!(help.contains(option), "{option} is not offered:\n{help}");
    }
}

#[test]
fn ui_run_without_a_path_or_without_tui_is_a_usage_error() {
    let no_path = ess(&["ui", "run", "--tui"]);
    assert_eq!(no_path.status.code(), Some(2));
    assert!(text(&no_path.stderr).contains("--path"));

    let no_renderer = ess(&["ui", "run", "--path", EXAMPLE]);
    assert_eq!(no_renderer.status.code(), Some(2));
    assert!(text(&no_renderer.stderr).contains("--tui"));
}

#[test]
fn the_ui_area_offers_load_docs_and_run() {
    let output = ess(&["ui", "--help"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let help = text(&output.stdout);
    let offered: Vec<&str> = help
        .split_once("Commands:\n")
        .expect("the help lists commands")
        .1
        .lines()
        .take_while(|line| !line.trim().is_empty())
        .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(offered, ["load", "check", "docs", "run", "test"], "{help}");
}

#[test]
fn ui_check_passes_the_example_with_only_its_placeholder_warning() {
    let output = ess(&["ui", "check", "--path", EXAMPLE, "--format", "json"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the report is JSON");
    assert_eq!(report["format"], "ess-ui-check/1");
    let findings = report["findings"].as_array().expect("a findings list");
    assert!(
        findings
            .iter()
            .all(|finding| finding["severity"] == "warning"),
        "{findings:?}"
    );
}

#[test]
fn ui_check_exits_1_naming_the_node_of_an_error() {
    let output = ess(&[
        "ui",
        "check",
        "--path",
        "crates/ui/ess-ui-check/tests/fixtures/broken-nav.yaml",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the report is JSON");
    let errors: Vec<&serde_json::Value> = report["findings"]
        .as_array()
        .expect("a findings list")
        .iter()
        .filter(|finding| finding["severity"] == "error")
        .collect();
    assert!(
        errors.iter().any(|finding| finding["path"]
            .as_str()
            .is_some_and(|path| path.starts_with("navigation"))),
        "{errors:?}"
    );
}

#[test]
fn ui_test_runs_the_example_tests() {
    let output = ess(&[
        "ui",
        "test",
        "--path",
        EXAMPLE,
        "examples/partner-portal/tests/partners-list.yaml",
        "examples/partner-portal/tests/live.yaml",
        "examples/partner-portal/tests/stale.yaml",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the report is JSON");
    assert_eq!(report["format"], "ess-ui-test-report/1");
    let tests = report["tests"].as_array().expect("a tests list");
    assert!(!tests.is_empty());
    assert!(
        tests.iter().all(|test| test["status"] == "passed"),
        "{tests:#?}"
    );
}
