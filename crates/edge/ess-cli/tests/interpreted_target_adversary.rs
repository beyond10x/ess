//! Adversary pass 1 against `story:interpreted-target-selection`.
//!
//! Two claims the unit made about itself, driven against the code the same unit wrote.
//!
//! 1. The public source-capability block in `website/docs/status/where-this-stands.md` is generated
//!    from `ess verify conform run --help` by `crates/edge/ess-xtask/src/support.rs:334,568` and
//!    compared back by `cargo xtask support --check`, which `task check` runs
//!    (`Taskfile.yml:165-170,204`). Adding a `--target` value without updating that row is drift the
//!    unit's package-scoped gate cannot see.
//! 2. `crates/verify/ess-conformance/src/interpret.rs` states, of the target it adds: "selecting
//!    this target cannot make a suite green, and a run against it exits non-zero today and will keep
//!    doing so until interpretation actually decides something." That is a claim about every suite,
//!    and the runner decides a run's verdict from its scenarios
//!    (`crates/verify/ess-conformance/src/report.rs:566-578`), of which an admitted suite may hold
//!    none. `website/docs/guides/verify-conformance.md:238-240` documents `[]` as an explicit
//!    selection of no scenarios, so a zero-scenario admitted suite is a state the documented
//!    workflow reaches.
//!
//! Round 1 answered both. The sentence quoted in (2) no longer reads that way: `interpret.rs` now
//! qualifies the claim to suites holding at least one scenario and names the empty-suite verdict as
//! the runner's, target-independent behaviour. The second case below keeps the adversary's
//! construction and still executes it, and asserts the corrected claim plus the cross-target
//! control that establishes whose behaviour the exception is.

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
        .arg("--suite")
        .arg(suite)
        .args(["--format", "json"])
        .output()
        .expect("the `ess` binary runs");
    let stdout = String::from_utf8(output.stdout).expect("the report is UTF-8");
    let report = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("report/1 is rendered as JSON: {error}\n{stdout}"));
    (report, output.status.code())
}

/// Selecting the interpreted target cannot produce a passing conformance report.
///
/// The claim this pins is `interpret.rs`'s, as corrected in round 1: a run over a suite holding **at
/// least one** scenario comes back `failed` and exits non-zero, because §28 makes every unsupported
/// obligation fail conformance.
///
/// The adversary that wrote this case asserted the sentence's original, unqualified form and found
/// the hole: an admitted suite may hold **no** scenarios (`admission.rs:103-145` imposes no lower
/// bound, and `verify-conformance.md:238-240` documents `[]` as a selection somebody can ask for),
/// and `ConformanceReport::verdict(&[])` is `Passed`. The empty-suite half of this case is kept and
/// still executed, and the assertion on it is now the true one — pinned **across targets**, because
/// that is the fact that decides whose defect it is: `billing` reports `passed` and exits 0 for the
/// same file. A vacuous suite is vacuously conformant for every implementation, which is a property
/// of `report.rs:566-578` and not a capability the interpreter has. Making an empty suite refuse
/// would change `billing` and `oracle-fixture` too and is not this story's to do.
#[test]
fn a_run_against_the_interpreted_target_is_never_green() {
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

    // The claim itself: a suite with scenarios in it is never green against this target.
    let (report, code) = run_suite(&committed, "interpreted");
    assert_eq!(
        report["status"], "failed",
        "interpret.rs claims a suite holding at least one scenario cannot come back green: {report:#}"
    );
    assert_eq!(
        code,
        Some(1),
        "and that such a run exits non-zero — 1 for failed, not 3 for error: {report:#}"
    );

    // The exception the adversary found, pinned where it belongs: at the runner, not at the target.
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
        (&interpreted["status"], interpreted_code),
        (&billing["status"], billing_code),
        "a suite holding no scenarios must come out the same for every target, because the verdict \
         of no scenarios is the runner's and not the implementation's: interpreted \
         {interpreted:#}\nbilling {billing:#}"
    );
    assert_eq!(
        (&interpreted["status"], interpreted_code),
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
