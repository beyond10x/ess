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

#[test]
fn a_survivor_exits_one_and_is_listed_first() {
    let output = mutate(&["--path", SURVIVOR, "--target", "billing"]);
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
        "survived sets-retarget/billing.invoice.CreateInvoice/accepted/contact: `contact: \
         input.customer_email` becomes `contact: input.billing_email`"
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

#[test]
fn a_baseline_that_does_not_pass_exits_three_with_mutate_001() {
    let directory = scratch("baseline");
    let report = directory.join("report.json");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "interpreted",
        "--report-out",
        report.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(stderr.starts_with("refusal[ESS-MUTATE-001]"), "{stderr}");
    assert!(
        stderr.contains("\n  billing.invoice.CreateInvoice/outcome/accepted\n"),
        "every scenario that did not pass is listed: {stderr}"
    );
    assert!(!report.exists(), "a refused audit writes no report");
}

#[test]
fn every_mutant_stillborn_ran_nothing_and_exits_three() {
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--class",
        "transition-to",
    ]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(
        stdout.contains("3 mutant(s), 0 killed, 0 survived, 0 inconclusive, 3 stillborn"),
        "{stdout}"
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
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let written = std::fs::read(&report).expect("--report-out is written");
    assert_eq!(output.stdout, written);
    let value: serde_json::Value = serde_json::from_slice(&written).unwrap();
    assert_eq!(value["format"], "ess-mutation-report/1");
    assert_eq!(value["counts"]["mutants"], 20);
    assert_eq!(value["counts"]["killed"], 12);
    assert_eq!(value["counts"]["stillborn"], 8);

    let yaml = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--format",
        "yaml",
    ]);
    assert_eq!(yaml.status.code(), Some(0));
    assert!(
        text(&yaml.stdout).contains("format: ess-mutation-report/1"),
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
fn the_help_names_the_nine_classes_and_the_exit_statuses() {
    let output = mutate(&["--help"]);
    let help = text(&output.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(output.status.success(), "{help}");
    assert!(
        help.contains(
            "[possible values: from-drop, transition-to, guard-boundary, sets-retarget, \
             guard-negate, guard-connective, error-swap, emit-drop, order-flip]"
        ),
        "{help}"
    );
    assert!(
        help.contains("[possible values: billing, oracle-fixture, interpreted]"),
        "{help}"
    );
    for status in ["Exit 0", "Exit 1", "Exit 3"] {
        assert!(help.contains(status), "{status}: {help}");
    }
}

#[test]
fn every_mutate_code_is_named_in_the_formats_reference() {
    let page = std::fs::read_to_string(root().join("website/docs/reference/formats.md")).unwrap();
    for code in ess_conformance::mutate::MutateCode::ALL {
        let code = code.code().to_string();
        assert!(page.contains(&code), "`{code}` is not named in formats.md");
    }
    assert!(page.contains("ess-mutation-report/1"));
}
