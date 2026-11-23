//! `ess verify conform synthesize --help` describes the suite version `--target ir` writes as it is
//! chosen, not as one fixed number (beyond10x/ess#186).
//!
//! The help once said "The canonical `ess-conformance/1` document" while a plain `ess/1`
//! specification wrote `ess-conformance/4`, and a specification using newer constructs writes a
//! higher number still: the synthesizer picks the version from the constructs the suite holds.

use std::path::{Path, PathBuf};
use std::process::Command;

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

fn help() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "synthesize", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs");
    assert!(output.status.success(), "`synthesize --help` failed");
    String::from_utf8(output.stdout).expect("the help is UTF-8")
}

/// Every `ess-conformance/<digit>` the text spells, as written.
fn fixed_suite_versions(text: &str) -> Vec<String> {
    let prefix = "ess-conformance/";
    text.match_indices(prefix)
        .filter_map(|(at, _)| {
            let digits: String = text[at + prefix.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            (!digits.is_empty()).then(|| format!("{prefix}{digits}"))
        })
        .collect()
}

#[test]
fn generated_docs_ir_target_help_names_no_fixed_suite_version() {
    let help = help();
    assert_eq!(
        fixed_suite_versions(&help),
        Vec::<String>::new(),
        "the help names a fixed suite version the synthesizer does not always write:\n{help}"
    );
}

#[test]
fn generated_docs_ir_target_help_names_the_rule_that_picks_the_version() {
    let help = help();
    let flat = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains(
            "`suite_version` is chosen by `--suite-format` and by the constructs the \
             specification uses"
        ),
        "the `ir` target does not say how its suite version is chosen:\n{help}"
    );
    assert!(
        flat.contains("--suite-format <SUITE_FORMAT>"),
        "the help names `--suite-format` as a rule and offers no such option:\n{help}"
    );
}

fn help_of(verb: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", verb, "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs");
    assert!(output.status.success(), "`{verb} --help` failed");
    let text = String::from_utf8(output.stdout).expect("the help is UTF-8");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn names_jobs(flat: &str) {
    assert!(
        flat.contains("--jobs <JOBS>"),
        "no `--jobs <JOBS>`:\n{flat}"
    );
    assert!(
        flat.contains("the same bytes at every count"),
        "no promise of equal bytes:\n{flat}"
    );
    assert!(flat.contains("ESS_SYNTH_JOBS"), "no variable:\n{flat}");
}

#[test]
fn generated_docs_synthesize_help_names_jobs_and_its_variable() {
    names_jobs(&help_of("synthesize"));
}

#[test]
fn generated_docs_run_help_names_jobs() {
    names_jobs(&help_of("run"));
}

const FIXTURE: &str = "crates/verify/ess-conformance/tests/fixtures/caller-swap-related-tie.yaml";

struct Outcome {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn ess(args: &[&str], variable: Option<&str>) -> Outcome {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["verify", "conform"])
        .args(args)
        .current_dir(workspace_root())
        .env_remove("ESS_SYNTH_JOBS");
    if let Some(value) = variable {
        command.env("ESS_SYNTH_JOBS", value);
    }
    let output = command.output().expect("the ess binary runs");
    Outcome {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn out_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("synthesize-jobs")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the directory is made");
    dir
}

/// Runs `synthesize` to `<dir>/<file>`; returns the outcome with the path blanked from stdout,
/// and the bytes written, if any.
fn synthesize_to(
    dir: &Path,
    file: &str,
    extra: &[&str],
    variable: Option<&str>,
) -> (Outcome, Option<Vec<u8>>) {
    let out = dir.join(file);
    let _ = std::fs::remove_file(&out);
    let out_text = out.to_str().expect("a UTF-8 path");
    let mut args = vec![
        "synthesize",
        "--path",
        FIXTURE,
        "--target",
        "ir",
        "--out",
        out_text,
    ];
    args.extend_from_slice(extra);
    let mut outcome = ess(&args, variable);
    outcome.stdout = outcome.stdout.replace(out_text, "<out>");
    (outcome, std::fs::read(&out).ok())
}

fn one_job_suite() -> Vec<u8> {
    let dir = out_dir("one");
    let (one, bytes) = synthesize_to(&dir, "one.json", &["--jobs", "1"], None);
    assert_eq!(one.code, Some(0), "{}", one.stderr);
    bytes.expect("one job writes the suite")
}

#[test]
fn four_jobs_write_the_suite_one_job_writes() {
    let dir = out_dir("four");
    let (one, one_bytes) = synthesize_to(&dir, "one.json", &["--jobs", "1"], None);
    let (four, four_bytes) = synthesize_to(&dir, "four.json", &["--jobs", "4"], None);
    assert_eq!(one.code, Some(0), "{}", one.stderr);
    assert_eq!(four.code, Some(0), "{}", four.stderr);
    assert!(
        one_bytes.is_some() && one_bytes == four_bytes,
        "the suites differ"
    );
    assert_eq!(one.stdout, four.stdout);
}

#[test]
fn the_variable_sets_the_job_count_and_the_flag_wins() {
    let dir = out_dir("variable");
    let expected = one_job_suite();
    let (by_variable, bytes) = synthesize_to(&dir, "variable.json", &[], Some("4"));
    assert_eq!(by_variable.code, Some(0), "{}", by_variable.stderr);
    assert_eq!(bytes.as_deref(), Some(expected.as_slice()));
    let (by_flag, bytes) = synthesize_to(&dir, "flag.json", &["--jobs", "2"], Some("nonsense"));
    assert_eq!(by_flag.code, Some(0), "{}", by_flag.stderr);
    assert_eq!(bytes.as_deref(), Some(expected.as_slice()));
}

#[test]
fn a_job_count_outside_one_to_256_is_refused() {
    let dir = out_dir("refused");
    for count in ["0", "257"] {
        let (refused, bytes) = synthesize_to(&dir, "flag.json", &["--jobs", count], None);
        assert_eq!(refused.code, Some(2), "--jobs {count}: {}", refused.stderr);
        assert!(
            refused.stderr.contains("--jobs")
                && (refused.stderr.contains("value") || refused.stderr.contains("range")),
            "--jobs {count}: {}",
            refused.stderr
        );
        assert!(bytes.is_none(), "--jobs {count} wrote a suite");
    }
    for value in ["0", "257", "two"] {
        let (refused, bytes) = synthesize_to(&dir, "variable.json", &[], Some(value));
        assert_eq!(
            refused.code,
            Some(1),
            "ESS_SYNTH_JOBS={value}: {}",
            refused.stderr
        );
        assert!(bytes.is_none(), "ESS_SYNTH_JOBS={value} wrote a suite");
        assert!(
            refused.stderr.contains(&format!(
                "ESS_SYNTH_JOBS is `{value}`; it takes a whole number from 1 to 256"
            )),
            "{}",
            refused.stderr
        );
    }
}

#[test]
fn run_synthesizes_with_jobs() {
    let run = |jobs: &str| {
        ess(
            &[
                "run",
                "--path",
                FIXTURE,
                "--target",
                "interpreted",
                "--jobs",
                jobs,
                "--format",
                "json",
            ],
            None,
        )
    };
    let (one, four) = (run("1"), run("4"));
    assert_ne!(one.code, Some(2), "{}", one.stderr);
    assert_eq!(one.code, four.code, "{}", four.stderr);
    assert_eq!(one.stdout, four.stdout);
}

fn run_args(suite: Option<&str>) -> Vec<&str> {
    let mut args = vec!["run", "--path", FIXTURE, "--target", "interpreted"];
    if let Some(file) = suite {
        args.extend(["--suite", file]);
    }
    args.extend(["--format", "json"]);
    args
}

#[test]
fn run_with_a_suite_does_not_read_the_job_variable() {
    let dir = out_dir("run-suite");
    let (made, bytes) = synthesize_to(&dir, "suite.json", &[], None);
    assert_eq!(made.code, Some(0), "{}", made.stderr);
    assert!(bytes.is_some(), "no suite written");
    let file = dir.join("suite.json");
    let file = file.to_str().expect("a UTF-8 path");
    let without = ess(&run_args(Some(file)), None);
    assert_ne!(without.code, Some(2), "{}", without.stderr);
    for value in ["abc", "0", "257"] {
        let with = ess(&run_args(Some(file)), Some(value));
        assert_eq!(with.code, without.code, "{value}: {}", with.stderr);
        assert_eq!(with.stdout, without.stdout, "{value}");
        assert!(!with.stderr.contains("ESS_SYNTH_JOBS"), "{}", with.stderr);
    }
}

#[test]
fn run_that_synthesizes_refuses_an_invalid_job_variable() {
    let refused = ess(&run_args(None), Some("abc"));
    assert_eq!(refused.code, Some(1), "{}", refused.stderr);
    assert!(
        refused
            .stderr
            .contains("ESS_SYNTH_JOBS is `abc`; it takes a whole number from 1 to 256"),
        "{}",
        refused.stderr
    );
}
