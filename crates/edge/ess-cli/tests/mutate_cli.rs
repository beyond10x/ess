//! `ess verify conform mutate`: the mutation audit, as an operator runs it.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, Part 1, "Exit status" and the CLI checks. The
//! library's verdicts are decided in `crates/verify/ess-conformance/tests/mutation_audit.rs`; this
//! file holds the verb to its exit codes, its stderr refusals and its two byte-identical outputs.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

const SURVIVOR: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-survivor.yaml";

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

fn mutate(args: &[&str]) -> Output {
    let mut all = vec!["verify", "conform", "mutate"];
    all.extend_from_slice(args);
    ess(&all)
}

fn scratch(name: &str) -> PathBuf {
    let directory = root()
        .join("target/mutate-cli")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

#[test]
fn every_mutant_that_ran_killed_exits_zero() {
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "error-swap",
    ]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(
        stdout.starts_with(
            "mutation audit of billing v3 against billing-reference: 5 mutant(s), 5 killed, 0 \
             survived"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "killed error-swap/billing.invoice.CreateInvoice/rejected: `error: \
             billing.invoice.InvalidAmount` becomes `error: billing.invoice.InvoiceStateConflict` \
             — by billing.invoice.CreateInvoice/outcome/rejected (1 in total)"
        ),
        "{stdout}"
    );
}

/// A survivor's suite differs from the baseline's and still passes: `score >= 50` to `score > 50`
/// writes a witness at 50 that the target answers as the baseline does.
const EXPLORE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";

#[test]
fn a_survivor_exits_one_and_is_listed_first() {
    let output = mutate(&[
        "--path",
        EXPLORE,
        "--target",
        "interpreted",
        "--class",
        "guard-boundary",
    ]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{stdout}{}",
        text(&output.stderr)
    );
    let second = stdout.lines().nth(1).expect("a survivor line");
    assert_eq!(
        second,
        "survived guard-boundary/explore.desk.Grade/high/0: `score >= 50` becomes `score > 50`"
    );
}

/// The P1-10 fixture's `sets-retarget` of a field nothing reads has a suite that asks what the
/// baseline's asks, so it is unwitnessed, not a survivor, and the audit exits 3
/// (<https://github.com/beyond10x/ess/issues/586>).
#[test]
fn a_mutant_whose_suite_does_not_change_exits_three_unwitnessed() {
    let output = mutate(&["--path", SURVIVOR, "--target", "billing"]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(stdout.contains(" 0 survived, "), "{stdout}");
    let second = stdout.lines().nth(1).expect("an unwitnessed line");
    assert_eq!(
        second,
        "unwitnessed sets-retarget/billing.invoice.CreateInvoice/accepted/contact: `contact: \
         input.customer_email` becomes `contact: input.billing_email` — ESS-MUTATE-004: the suite \
         does not change"
    );
}

#[test]
fn a_class_with_no_site_exits_three_and_writes_no_report() {
    let directory = scratch("no-site");
    let report = directory.join("report.json");
    let output = mutate(&[
        "--path",
        "examples/oracle-fixture",
        "--target",
        "oracle-fixture",
        "--class",
        "order-flip",
        "--report-out",
        report.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(
        stderr.starts_with("refusal[ESS-MUTATE-003]: `order-flip`"),
        "{stderr}"
    );
    assert!(!report.exists(), "a refused audit writes no report");
}

/// The oracle reference answers none of billing's commands, so its baseline is red. The
/// interpreter used to be the example; since issue #210 its `unsupported` scenarios are not scored
/// rather than red, and it scores (`ess-conformance`'s `mutation_audit.rs`).
#[test]
fn a_baseline_that_does_not_pass_exits_three_with_mutate_001() {
    let directory = scratch("baseline");
    let report = directory.join("report.json");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "oracle-fixture",
        "--report-out",
        report.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(stderr.starts_with("refusal[ESS-MUTATE-001]"), "{stderr}");
    assert!(
        stderr.contains("\n  billing.invoice.CreateInvoice/outcome/accepted\n"),
        "every scenario that failed or ended error is listed: {stderr}"
    );
    assert!(!report.exists(), "a refused audit writes no report");
}

/// Each billing outcome emits one event, so every `emit-drop` mutant is stillborn
/// (`ESS-COMMAND-007`). Billing's transitions have no admissible other target, so `transition-to`
/// has no mutant there at all and refuses, as a class with no site does (beyond10x/ess#711).
#[test]
fn every_mutant_stillborn_ran_nothing_and_exits_three() {
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "emit-drop",
    ]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(
        stdout.contains("5 mutant(s), 0 killed, 0 survived, 0 inconclusive, 5 stillborn"),
        "{stdout}"
    );
    let transitions = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "transition-to",
    ]);
    let stderr = text(&transitions.stderr);
    assert_eq!(transitions.status.code(), Some(3), "{stderr}");
    assert!(
        stderr.starts_with("refusal[ESS-MUTATE-003]: `transition-to`"),
        "{stderr}"
    );
}

#[test]
fn the_json_output_is_the_report_bytes() {
    let directory = scratch("json");
    let report = directory.join("report.json");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--format",
        "json",
        "--report-out",
        report.to_str().unwrap(),
    ]);
    // Every class includes `emit-swap` (beyond10x/ess#295): billing's two swaps are killed and its
    // three sites without an alternative are unavailable, so the audit exits 3. Its eleven
    // unmutated sites (beyond10x/ess#711) make it `/8`: the ten of
    // story:the-mutation-audit-reports-the-sites-it-could-not-mutate and `Customer`'s grant of a
    // command nothing serves (story:the-mutation-audit-covers-actor-grants).
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    let written = std::fs::read(&report).expect("--report-out is written");
    assert_eq!(output.stdout, written);
    let value: serde_json::Value = serde_json::from_slice(&written).unwrap();
    assert_eq!(value["format"], "ess-mutation-report/8");
    assert_eq!(value["counts"]["mutants"], 19);
    assert_eq!(value["counts"]["killed"], 14);
    assert_eq!(value["counts"]["stillborn"], 5);
    assert_eq!(value["unavailable_sites"].as_array().map(Vec::len), Some(3));
    assert_eq!(value["unmutated_sites"].as_array().map(Vec::len), Some(11));

    let yaml = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--format",
        "yaml",
    ]);
    assert_eq!(yaml.status.code(), Some(3));
    assert!(
        text(&yaml.stdout).contains("format: ess-mutation-report/8"),
        "{}",
        text(&yaml.stdout)
    );
}

#[test]
fn a_misspelled_class_is_a_usage_error() {
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "drop-sets",
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", text(&output.stderr));
}

#[test]
fn the_help_names_the_twelve_classes_and_the_exit_statuses() {
    let output = mutate(&["--help"]);
    let help = text(&output.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(output.status.success(), "{help}");
    assert!(
        help.contains(
            "[possible values: from-drop, transition-to, guard-boundary, sets-retarget, \
             guard-negate, guard-connective, error-swap, emit-drop, order-flip, sets-drop, \
             precedence-swap, emit-swap, grant-drop, grant-add]"
        ),
        "story:the-mutation-audit-covers-actor-grants: {help}"
    );
    assert!(
        help.contains("[possible values: billing, oracle-fixture, interpreted]"),
        "{help}"
    );
    for status in ["Exit 0", "Exit 1", "Exit 3"] {
        assert!(help.contains(status), "{status}: {help}");
    }
}

/// The toy linear lifecycle of beyond10x/ess#711, brand-free: `lend` is the only transition into
/// `Lent`, so no other target state keeps the lifecycle admissible.
const LINEAR: &str = r"format: ess/32
system: library
version: v1
domain: library.shelf
types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}
entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: position, type: Integer}
    lifecycle:
      initial: Shelved
      states: [Shelved, Lent]
      terminal: [Lent]
      transitions:
        - {name: lend, from: [Shelved], to: Lent}
actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook, library.shelf.LendBook]
errors:
  - name: library.shelf.BookNotFound
  - name: library.shelf.BookStateConflict
commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: position, type: Integer}
    outcomes:
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {book_id: {generated: true}}
        sets: {position: input.position}
  - name: library.shelf.LendBook
    input:
      - {name: book_id, type: library.shelf.BookId}
    outcomes:
      - name: lent
        moves: library.shelf.Book.lend
        instance: book_id
        emits: [library.shelf.BookLent]
        payload:
          library.shelf.BookLent: {book_id: input.book_id}
      - name: wrong-state
        wrong_state: true
        error: library.shelf.BookStateConflict
events:
  - name: library.shelf.BookCatalogued
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookLent
    fields:
      - {name: book_id, type: library.shelf.BookId}
views:
  - name: library.shelf.BookById
    source: library.shelf.Book
    consistency: read_your_writes
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
";

/// A site without a live mutant is listed and leaves the default exit status as it was (D135);
/// `--fail-on-unmutated` makes it block exit 0, as an unavailable emit-swap site does
/// (story:the-mutation-audit-reports-the-sites-it-could-not-mutate).
#[test]
fn a_site_without_a_live_mutant_blocks_exit_zero_only_when_asked() {
    let directory = scratch("linear");
    let spec = directory.join("linear.yaml");
    std::fs::write(&spec, LINEAR).unwrap();
    let args = [
        "--path",
        spec.to_str().unwrap(),
        "--target",
        "interpreted",
        "--class",
        "transition-to",
        "--class",
        "error-swap",
    ];
    let default = mutate(&args);
    let stdout = text(&default.stdout);
    assert_eq!(
        default.status.code(),
        Some(0),
        "{stdout}{}",
        text(&default.stderr)
    );
    assert!(
        stdout.lines().any(|line| line.starts_with(
            "unmutated transition-to/library.shelf.Book.lend (no_admissible_state): "
        )),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .next()
            .is_some_and(|summary| summary.ends_with("; 1 of 2 site(s) scored")),
        "{stdout}"
    );
    let mut strict = args.to_vec();
    strict.push("--fail-on-unmutated");
    let strict = mutate(&strict);
    assert_eq!(
        strict.status.code(),
        Some(3),
        "story:the-mutation-audit-reports-the-sites-it-could-not-mutate: {}{}",
        text(&strict.stdout),
        text(&strict.stderr)
    );
    assert_eq!(
        strict.stdout, default.stdout,
        "the flag changes the exit only"
    );
}

#[test]
fn every_mutate_code_is_named_in_the_formats_reference() {
    let page = std::fs::read_to_string(root().join("website/docs/reference/formats.md")).unwrap();
    for code in ess_conformance::mutate::MutateCode::ALL {
        let code = code.code().to_string();
        assert!(page.contains(&code), "`{code}` is not named in formats.md");
    }
    assert!(page.contains("ess-mutation-report/3"));
}
