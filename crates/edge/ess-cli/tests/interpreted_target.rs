//! An operator can select the interpreter target, and it executes the model it is handed.
//!
//! `--target interpreted` was introduced as a seam that decided nothing
//! (`story:interpreted-target-selection`); `story:interpreted-command-execution` fills its command
//! path. What is checked here: the value is offered beside the existing targets, it requires the
//! specification by `--path` and refuses one the suite was not synthesized from, and a run of
//! `examples/billing`'s committed suite against it passes every scenario that needs only command
//! execution while every other scenario is an unsatisfied obligation naming what is not interpreted
//! yet. Not one is an `error` or a failure, because nothing went wrong — §28's fourth word is the
//! honest answer for what the target cannot yet derive.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

/// Help text with every run of whitespace collapsed, so a wrapped option block still reads as one
/// line and the assertion is about the values rather than about the terminal width.
fn unwrapped(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .expect("help is UTF-8")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn conform_run_help_offers_interpreted_beside_the_existing_targets() {
    let output = ess(&["verify", "conform", "run", "--help"]);
    assert!(output.status.success(), "{}", unwrapped(&output.stderr));
    let help = unwrapped(&output.stdout);
    assert!(
        help.contains("[possible values: billing, oracle-fixture, interpreted]"),
        "`conform run --help` offers the interpreter beside the two reference targets: {help}"
    );
}

/// The values one option accepts, read out of clap's own inline metadata.
fn possible_values(help: &str, flag: &str) -> Vec<String> {
    let block: String = help
        .lines()
        .skip_while(|line| line.split_whitespace().next() != Some(flag))
        .take(8)
        .collect::<Vec<_>>()
        .join("\n");
    let (_, suffix) = block
        .split_once("[possible values: ")
        .unwrap_or_else(|| panic!("`{flag}` has an inline possible-values list: {block}"));
    let (values, _) = suffix.split_once(']').expect("the list is terminated");
    values.split(", ").map(str::to_owned).collect()
}

/// The adopter guide's target list is prose with no generator behind it, so it is checked here.
///
/// `website/docs/status/where-this-stands.md` states the same list and has a generator and a gate
/// step (`ess-xtask/src/support.rs:334`, `cargo xtask support --check`). The guide's sentence had
/// neither, which is why it went stale on the round that added a third target and why nothing in
/// `task check` noticed. This case is that missing generator's cheapest substitute: a fourth target
/// now fails a package-scoped run rather than a published page.
#[test]
fn the_conformance_guide_names_every_target_the_cli_offers() {
    let help = ess(&["verify", "conform", "run", "--help"]);
    assert!(help.status.success(), "{}", unwrapped(&help.stderr));
    let offered = possible_values(
        &String::from_utf8(help.stdout).expect("help is UTF-8"),
        "--target",
    );

    let guide = std::fs::read_to_string(root().join("website/docs/guides/verify-conformance.md"))
        .expect("the conformance guide is committed");
    let claim = "The built-in choices are ";
    let start = guide
        .find(claim)
        .unwrap_or_else(|| panic!("the guide states the built-in choices"));
    let sentence = &guide[start..][..guide[start..]
        .find(';')
        .expect("the built-in-choices sentence is delimited")];
    let stated: Vec<String> = sentence
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect();

    assert_eq!(
        stated, offered,
        "`website/docs/guides/verify-conformance.md` names the built-in `--target` choices in prose \
         and nothing generates it; it must list the same values in the same order as the CLI's own \
         help: {sentence}"
    );
}

/// A report names the implementation that answered, and that name is not the `--target` value.
///
/// `interpret.rs` picks its own name to equal its selector, which is a choice it is entitled to
/// make and not a convention: the two hand-written targets name an implementation rather than a
/// selection. Pinned across all three so the asymmetry is a decision on the record rather than
/// something a reader infers from one of them.
#[test]
fn each_target_reports_the_implementation_name_it_declares() {
    let suite = root().join("suites/generated/billing/suite.json");
    for (target, implementation) in [
        ("billing", "billing-reference"),
        ("oracle-fixture", "oracle-reference"),
        ("interpreted", "interpreted"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(root())
            .args(["verify", "conform", "run", "--target", target])
            // `interpreted` requires the model; the hand-written targets ignore it.
            .args(["--path", "examples/billing"])
            .arg("--suite")
            .arg(&suite)
            .args(["--format", "json"])
            .output()
            .expect("the `ess` binary runs");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{target} renders report/1 as JSON: {error}"));
        assert_eq!(
            report["implementation"]["name"], implementation,
            "`--target {target}` reports `{implementation}`"
        );
    }
}

/// What `ess verify conform run --target interpreted` answers for the committed billing suite.
///
/// Rewritten for `story:interpreted-command-execution`; the name keeps the obligation half of the
/// claim, which still holds for every scenario the interpreter does not cover. The earlier form
/// asserted that **every** scenario and check was `unsupported`, because the seam derived nothing.
/// Now: the scenarios that need only command execution, transitions, `sets:` writes, emitted events,
/// declared refusals and a forced external outcome pass; every other scenario is `unsupported`, and
/// each unsupported check names what is not interpreted yet. No scenario and no check is `failed`
/// or `error`, and the run still fails conformance (exit 1), because §28 counts an unsatisfied
/// obligation as a failure of the run.
#[test]
fn the_committed_billing_suite_runs_against_interpreted_as_unsatisfied_obligations() {
    let suite = root().join("suites/generated/billing/suite.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "run", "--target", "interpreted"])
        .args(["--path", "examples/billing"])
        .arg("--suite")
        .arg(&suite)
        .args(["--format", "json"])
        .output()
        .expect("the `ess` binary runs");
    assert_eq!(
        output.status.code(),
        Some(1),
        "an unsupported obligation fails conformance (1) and is not an execution error (3): {}",
        unwrapped(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("report/1 is rendered as JSON");
    assert_eq!(
        report["implementation"]["name"], "interpreted",
        "the report names which implementation answered"
    );
    assert_eq!(report["status"], "failed");

    let scenarios = report["scenarios"].as_array().expect("scenarios");
    assert_eq!(
        scenarios.len(),
        33,
        "every scenario of the committed billing suite was run"
    );
    let with_status = |wanted: &str| -> BTreeSet<&str> {
        scenarios
            .iter()
            .filter(|scenario| scenario["status"] == wanted)
            .map(|scenario| scenario["scenario"].as_str().expect("a scenario id"))
            .collect()
    };
    let passed: BTreeSet<&str> = BTreeSet::from([
        "billing.email.SendEmail/outcome/failed",
        "billing.email.SendEmail/outcome/sent",
        "billing.invoice.CancelInvoice/outcome/wrong-state",
        "billing.invoice.CreateInvoice/outcome/rejected",
        "billing.invoice.Invoice/state/Cancelled/refuses/billing.invoice.CancelInvoice",
        "billing.invoice.Invoice/state/Cancelled/refuses/billing.invoice.IssueInvoice",
        "billing.invoice.Invoice/state/Cancelled/refuses/billing.invoice.PayInvoice",
        "billing.invoice.Invoice/state/Draft/refuses/billing.invoice.PayInvoice",
        "billing.invoice.Invoice/state/Issued/refuses/billing.invoice.IssueInvoice",
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.CancelInvoice",
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.IssueInvoice",
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.PayInvoice",
        "billing.invoice.IssueInvoice/outcome/wrong-state",
        "billing.invoice.PayInvoice/outcome/rejected",
        "billing.invoice.PayInvoice/outcome/wrong-state",
    ]);
    assert_eq!(
        with_status("passed"),
        passed,
        "command execution is interpreted"
    );
    assert_eq!(
        with_status("unsupported").len(),
        33 - passed.len(),
        "everything else is an unsatisfied obligation"
    );

    for scenario in scenarios {
        let id = &scenario["scenario"];
        for check in scenario["checks"].as_array().expect("checks") {
            let status = check["status"].as_str().expect("a check status");
            assert!(
                status == "passed" || status == "unsupported",
                "`{id}` holds no failed or errored check: {check:#}"
            );
            if status == "unsupported" {
                let observed = check["diagnostic"]["observed"].to_string();
                assert!(
                    observed.contains("not interpreted yet"),
                    "`{id}`'s unsupported check names what is not interpreted: {observed}"
                );
            }
        }
    }
}

/// `--target interpreted` requires the specification, and refuses the wrong one by name.
#[test]
fn the_interpreted_target_refuses_a_missing_or_foreign_model() {
    let suite = root().join("suites/generated/billing/suite.json");
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(root())
            .args(["verify", "conform", "run", "--target", "interpreted"])
            .args(extra)
            .arg("--suite")
            .arg(&suite)
            .args(["--format", "json"])
            .output()
            .expect("the `ess` binary runs")
    };

    let missing = run(&[]);
    let stderr = unwrapped(&missing.stderr);
    assert_eq!(missing.status.code(), Some(1), "{stderr}");
    assert!(missing.stdout.is_empty(), "no report is rendered");
    assert!(
        stderr.contains("`--target interpreted` requires `--path`"),
        "{stderr}"
    );

    let foreign = run(&["--path", "examples/oracle-fixture"]);
    let stderr = unwrapped(&foreign.stderr);
    assert_eq!(foreign.status.code(), Some(1), "{stderr}");
    assert!(foreign.stdout.is_empty(), "no report is rendered");
    assert!(
        stderr.contains("refuses the specification at examples/oracle-fixture")
            && stderr.contains("is not the suite's spec_digest"),
        "{stderr}"
    );
}
