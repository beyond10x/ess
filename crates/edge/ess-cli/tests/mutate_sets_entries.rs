//! `ess verify conform mutate` accounts for every `sets:` entry
//! (<https://github.com/beyond10x/ess/issues/515>): an entry with a `sets-drop` site is mutated and
//! scored, and every other entry is named on stderr with the reason it has none, after the audit's
//! own output and in every format. The report and the manifest carry no such field. The library's
//! decision is held in `crates/verify/ess-conformance/tests/mutation_sets_entries.rs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const KEYS: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-sets-entries.yaml";

const LISTED: &[&str] = &[
    "note: sets-drop/demo.keys.Issue/issued/label has no mutant (creates_row): ",
    "note: sets-drop/demo.keys.Issue/issued/owner has no mutant (creates_row): ",
    "note: sets-drop/demo.keys.Issue/issued/tier has no mutant (creates_row): ",
    "note: sets-drop/demo.keys.Rename/renamed/note has no mutant (optional_field): ",
    "note: sets-drop/demo.keys.Rename/renamed/tier has no mutant (source_not_input): ",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn mutate(path: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "mutate", "--path", path])
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

/// The `note:` lines of `stderr`, each cut after its reason.
fn notes(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter(|line| line.starts_with("note: sets-drop/"))
        .map(|line| {
            let end = line.find("): ").expect("a reason") + 3;
            line[..end].to_owned()
        })
        .collect()
}

fn scratch(name: &str) -> PathBuf {
    let directory = root()
        .join("target/mutate-sets-entries")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.parent().expect("a parent")).expect("a scratch directory");
    directory
}

#[test]
fn the_site_is_killed_and_every_other_entry_is_listed_with_its_reason() {
    let output = mutate(
        KEYS,
        &[
            "--target",
            "interpreted",
            "--class",
            "sets-drop",
            "--format",
            "json",
        ],
    );
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/3", "{report:#}");
    let mutants = report["mutants"].as_array().expect("mutants");
    assert_eq!(mutants.len(), 1, "{report:#}");
    assert_eq!(
        mutants[0]["id"], "sets-drop/demo.keys.Rename/renamed/label",
        "{report:#}"
    );
    assert_eq!(mutants[0]["verdict"], "killed", "{report:#}");
    assert_eq!(notes(&stderr), LISTED, "{stderr}");
    assert!(
        !text(&output.stdout).contains("creates_row"),
        "the report carries no listed entry"
    );
}

/// Every class is the default, and it selects `sets-drop`; a selection without it lists nothing.
#[test]
fn the_entries_are_listed_only_where_sets_drop_is_selected() {
    let every = mutate(KEYS, &["--target", "interpreted"]);
    assert_eq!(
        notes(&text(&every.stderr)),
        LISTED,
        "{}",
        text(&every.stderr)
    );
    let without = mutate(
        KEYS,
        &["--target", "interpreted", "--class", "guard-negate"],
    );
    let stderr = text(&without.stderr);
    assert!(notes(&stderr).is_empty(), "{stderr}");
}

/// The issue's shape: no entry has a site, so the audit refuses with `ESS-MUTATE-003` first and
/// then says why of every entry.
#[test]
fn a_model_with_no_site_refuses_and_then_names_every_entry() {
    let output = mutate(
        "examples/billing",
        &["--target", "billing", "--class", "sets-drop"],
    );
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(
        stderr.starts_with("refusal[ESS-MUTATE-003]: `sets-drop`"),
        "{stderr}"
    );
    assert_eq!(
        notes(&stderr),
        [
            "note: sets-drop/billing.invoice.CreateInvoice/accepted/account_id has no mutant \
             (creates_row): ",
            "note: sets-drop/billing.invoice.CreateInvoice/accepted/reminder_count has no mutant \
             (creates_row): ",
            "note: sets-drop/billing.invoice.CreateInvoice/accepted/total has no mutant \
             (creates_row): ",
            "note: sets-drop/billing.invoice.IssueInvoice/issued/issued_at has no mutant \
             (optional_field): ",
        ],
        "{stderr}"
    );
}

/// `--emit` lists the same entries, and its manifest is the format the site alone asks for.
#[test]
fn the_emission_lists_the_entries_and_keeps_its_manifest_format() {
    let directory = scratch("emit");
    let output = mutate(
        KEYS,
        &[
            "--class",
            "sets-drop",
            "--emit",
            directory.to_str().unwrap(),
        ],
    );
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert_eq!(notes(&stderr), LISTED, "{stderr}");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["format"], "ess-mutation-manifest/4",
        "{manifest:#}"
    );
    let ids: Vec<&str> = manifest["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .filter_map(|mutant| mutant["id"].as_str())
        .collect();
    assert_eq!(ids, ["sets-drop/demo.keys.Rename/renamed/label"]);
}
