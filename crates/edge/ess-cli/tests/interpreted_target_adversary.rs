//! Actual Billing execution is distinct from the runner's vacuous empty-suite verdict.
//! The support matrix must also list every offered target.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::{fs, io::Write};

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

/// The values clap offers for one option, read out of its help block the way
/// `ess-xtask/src/support.rs:90-113` reads them.
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
    let (values, _) = suffix
        .split_once(']')
        .expect("the possible-values list is terminated");
    values.split(", ").map(str::to_owned).collect()
}

/// The back-quoted capability cell of one row of the generated source-support block.
fn support_row_values(status: &str, name: &str) -> Vec<String> {
    let prefix = format!("| {name} | ");
    let line = status
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("the source-support block holds a `{name}` row"));
    let capability = line
        .split(" | ")
        .nth(1)
        .expect("a row has a capability cell");
    capability
        .split(", ")
        .map(|value| value.trim_matches('`').to_owned())
        .collect()
}

/// The generated support matrix must name every conformance target the CLI offers.
///
/// `support.rs:334` builds this row from exactly the help text asserted in
/// `interpreted_target.rs:36-45`, and `support.rs:31-67` compares the rendered row against the
/// committed page. A target added to the clap enum without the matching row is a `task check`
/// failure that the unit's `-p ess-conformance -p ess-cli` scope never executes.
#[test]
fn the_support_matrix_names_every_conformance_target_the_cli_offers() {
    let help = ess(&["verify", "conform", "run", "--help"]);
    assert!(help.status.success());
    let offered = possible_values(
        &String::from_utf8(help.stdout).expect("help is UTF-8"),
        "--target",
    );

    let status = fs::read_to_string(root().join("website/docs/status/where-this-stands.md"))
        .expect("the public status page is committed");
    let recorded = support_row_values(&status, "Conformance targets");

    assert_eq!(
        recorded, offered,
        "website/docs/status/where-this-stands.md's `Conformance targets` row is generated from \
         `conform run --help` by ess-xtask/src/support.rs:334 and compared by \
         `cargo xtask support --check`; it must list the same values in the same order"
    );
}

/// Runs one suite file against one target and returns the report and the exit code.
fn run_suite(suite: &Path, target: &str) -> (serde_json::Value, Option<i32>) {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "run", "--target", target])
        // Every target is handed the model; `interpreted` requires it and the others ignore it.
        .args(["--path", "examples/billing"])
        .arg("--suite")
        .arg(suite)
        .args(["--report-format", "2", "--format", "json"])
        .output()
        .expect("the `ess` binary runs");
    let stdout = String::from_utf8(output.stdout).expect("the report is UTF-8");
    let report = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("report/2 is rendered as JSON: {error}\n{stdout}"));
    (report, output.status.code())
}

/// A complete real run passes; an empty run remains vacuously green across targets.
#[test]
fn real_interpreted_execution_and_empty_suite_verdicts_are_distinct() {
    let committed = root().join("suites/generated/billing/suite.json");
    let text = fs::read_to_string(&committed).expect("the committed billing suite");
    let mut suite: serde_json::Value =
        serde_json::from_str(&text).expect("the committed suite is JSON");
    assert!(
        !suite["scenarios"]
            .as_object()
            .expect("the committed suite holds scenarios")
            .is_empty(),
        "the committed suite is the non-empty half of this case"
    );

    let (report, code) = run_suite(&committed, "interpreted");
    assert_eq!(
        report["summary"]["execution_status"], "passed",
        "{report:#}"
    );
    assert_eq!(code, Some(0));
    let scenarios = report["scenarios"].as_array().expect("executed scenarios");
    assert_eq!(scenarios.len(), 33);
    assert!(scenarios
        .iter()
        .all(|scenario| scenario["status"] == "passed"));

    // The empty-suite control belongs at the runner, not at the target.
    suite["scenarios"] = serde_json::json!({});
    let path =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join("interpreted-adversary-empty-suite.json");
    let mut file = fs::File::create(&path).expect("a suite file under the test target directory");
    file.write_all(format!("{suite:#}\n").as_bytes())
        .expect("writing the suite");
    drop(file);

    let (interpreted, interpreted_code) = run_suite(&path, "interpreted");
    let (billing, billing_code) = run_suite(&path, "billing");
    assert_eq!(
        (&interpreted["summary"]["execution_status"], interpreted_code),
        (&billing["summary"]["execution_status"], billing_code),
        "a suite holding no scenarios must come out the same for every target, because the verdict \
         of no scenarios is the runner's and not the implementation's: interpreted \
         {interpreted:#}\nbilling {billing:#}"
    );
    assert_eq!(
        (&interpreted["summary"]["execution_status"], interpreted_code),
        (&serde_json::json!("passed"), Some(0)),
        "and today that shared answer is `passed`, exit 0 — the documented `[]` selection reports a \
         target that executed nothing as conformant. Pinned so a change to it is deliberate: \
         {interpreted:#}"
    );
    assert_eq!(
        interpreted["scenarios"].as_array().map(Vec::len),
        Some(0),
        "nothing was executed to earn it"
    );
}
