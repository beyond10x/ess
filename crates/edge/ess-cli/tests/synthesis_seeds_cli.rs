//! `ess verify conform synthesize --synthesis-seed FILE INSTANCE` on the issue fixture of
//! beyond10x/ess#413 (`docs/design/synthesis-seeds.md`): explicit seeds discharge both refused
//! obligations, every refusal happens before output, seed and `--scenarios` roles stay apart, and
//! the seed-free run keeps the base bytes.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_conformance::{coverage::AdmittedInput, AdmittedSuite};
use sha2::{Digest as _, Sha256};

/// SHA-256 of the seed-free suite the base build writes for this model. Re-pinned for
/// beyond10x/ess#454: `Authorize/outcome/invalid` is also sent for an identity no row carries.
const BASE_SUITE_SHA256: &str = "a1c2842ce87a08841bbfea6ed29225246f9d3476948fd0d7a7f97c61e1ac444e";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/synthesis-seeds")
}

fn seed(name: &str) -> String {
    fixtures().join("seeds").join(name).display().to_string()
}

fn synthesize(out: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "synthesize", "--path"])
        .arg(fixtures().join("model"))
        .arg("--out")
        .arg(out)
        .args(extra)
        .output()
        .expect("ess runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn sha256(text: &str) -> String {
    use std::fmt::Write as _;
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut out, byte| {
            write!(out, "{byte:02x}").expect("writing to a String");
            out
        })
}

#[test]
fn seeds_discharge_both_refused_obligations_and_the_seed_free_run_keeps_its_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let plain = dir.path().join("plain.json");
    let output = synthesize(&plain, &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(stdout(&output).contains("7 scenario(s) (0 authored), 2 refusal(s)"));
    assert!(!stdout(&output).contains("synthesis seeds"));
    let bytes = std::fs::read_to_string(&plain).unwrap();
    assert_eq!(sha256(&bytes), BASE_SUITE_SHA256);

    let seeded = dir.path().join("seeded.json");
    let max = seed("max.yaml");
    let below = seed("below.yaml");
    let output = synthesize(
        &seeded,
        &[
            "--synthesis-seed",
            &max,
            "at-max",
            "--synthesis-seed",
            &below,
            "below-max",
        ],
    );
    assert!(output.status.success(), "{output:?}");
    let printed = stdout(&output);
    assert!(
        printed.contains("synthesis seeds: 2 selected, 2 applied"),
        "{printed}"
    );
    assert!(
        printed.contains("8 scenario(s) (0 authored), 0 refusal(s)"),
        "{printed}"
    );
    let original = std::fs::read_to_string(&seeded).unwrap();
    let admitted = AdmittedSuite::from_json(&original).unwrap();
    assert_eq!(
        admitted.suite().provenance.suite_version.to_string(),
        "ess-conformance/42"
    );
    let seeds = admitted
        .suite()
        .provenance
        .synthesis_seeds
        .as_ref()
        .unwrap();
    let sources: Vec<&str> = seeds
        .sources
        .keys()
        .map(ess_conformance::coverage::SourceIdentity::as_str)
        .collect();
    assert_eq!(
        sources,
        ["below.yaml", "max.yaml"],
        "no host path is recorded"
    );
    assert!(original.contains("9223372036854775807") && original.contains("9223372036854775806"));

    // Argument order does not move a byte.
    let reordered = dir.path().join("reordered.json");
    let output = synthesize(
        &reordered,
        &[
            "--synthesis-seed",
            &below,
            "below-max",
            "--synthesis-seed",
            &max,
            "at-max",
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(std::fs::read_to_string(&reordered).unwrap(), original);
}

#[test]
fn declared_coverage_takes_the_same_seeds_as_suite_43() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("coverage.json");
    let max = seed("max.yaml");
    let below = seed("below.yaml");
    let output = synthesize(
        &out,
        &[
            "--suite-format",
            "5",
            "--synthesis-seed",
            &max,
            "at-max",
            "--synthesis-seed",
            &below,
            "below-max",
        ],
    );
    assert!(output.status.success(), "{output:?}");
    let original = std::fs::read_to_string(&out).unwrap();
    let suite = AdmittedSuite::from_json(&original).unwrap();
    assert_eq!(
        suite.suite().provenance.suite_version.to_string(),
        "ess-conformance/43"
    );
    let input = AdmittedInput::from_suite(suite).unwrap();
    let inventory = input.selected().coverage().unwrap();
    assert_eq!(inventory.counts.refused, 0, "{:#?}", inventory.refused);
    assert_eq!(inventory.authored_sources.len(), 0);
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .synthesis_seeds
            .as_ref()
            .unwrap()
            .applications
            .len(),
        2
    );
}

#[test]
fn a_seed_source_appends_no_scenario_and_scenarios_select_no_seed() {
    let dir = tempfile::tempdir().unwrap();
    let max = seed("max.yaml");
    // The seed alone: its document is not appended.
    let only_seed = dir.path().join("only-seed.json");
    let output = synthesize(&only_seed, &["--synthesis-seed", &max, "at-max"]);
    assert!(output.status.success(), "{output:?}");
    assert!(
        stdout(&output).contains("(0 authored)"),
        "{}",
        stdout(&output)
    );
    // The scenario alone: no seed is selected and the suite stays seed-free.
    let only_scenario = dir.path().join("only-scenario.json");
    let output = synthesize(&only_scenario, &["--scenarios", &max]);
    assert!(output.status.success(), "{output:?}");
    assert!(
        stdout(&output).contains("(1 authored), 2 refusal(s)"),
        "{}",
        stdout(&output)
    );
    let text = std::fs::read_to_string(&only_scenario).unwrap();
    assert!(!text.contains("synthesis_seeds"));
    assert!(text.contains("\"suite_version\": \"ess-conformance/34\""));
    // Both, explicitly: one authored scenario and the recorded seed, apart.
    let both = dir.path().join("both.json");
    let output = synthesize(
        &both,
        &["--scenarios", &max, "--synthesis-seed", &max, "at-max"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(
        stdout(&output).contains("(1 authored)"),
        "{}",
        stdout(&output)
    );
    let admitted = AdmittedSuite::from_json(&std::fs::read_to_string(&both).unwrap()).unwrap();
    assert_eq!(
        admitted.suite().provenance.suite_version.to_string(),
        "ess-conformance/42"
    );
    let seeds = admitted
        .suite()
        .provenance
        .synthesis_seeds
        .as_ref()
        .unwrap();
    assert!(seeds
        .applications
        .iter()
        .all(|application| !application.scenario.to_string().contains("/authored/")));
}

#[test]
fn every_refused_selection_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let max = seed("max.yaml");
    let directory = fixtures().join("seeds").display().to_string();
    let missing = dir.path().join("missing.yaml").display().to_string();
    for (label, extra, expected) in [
        (
            "missing",
            vec!["--synthesis-seed", missing.as_str(), "at-max"],
            "missing",
        ),
        (
            "directory",
            vec!["--synthesis-seed", directory.as_str(), "at-max"],
            "not a regular non-symlink file",
        ),
        (
            "unknown-instance",
            vec!["--synthesis-seed", max.as_str(), "nowhere"],
            "unknown-instance",
        ),
        (
            "duplicate-selector",
            vec![
                "--synthesis-seed",
                max.as_str(),
                "at-max",
                "--synthesis-seed",
                max.as_str(),
                "at-max",
            ],
            "duplicate-selector",
        ),
        (
            "bad-instance-name",
            vec!["--synthesis-seed", max.as_str(), "At_Max"],
            "synthesis seed instance",
        ),
    ] {
        let out = dir.path().join(format!("{label}.json"));
        let output = synthesize(&out, &extra);
        assert!(!output.status.success(), "{label}: {output:?}");
        assert!(
            stderr(&output).contains(expected),
            "{label}: {}",
            stderr(&output)
        );
        assert!(!out.exists(), "{label}: wrote output after a refusal");
    }
    // One value alone is a usage error, before anything runs.
    let out = dir.path().join("one-value.json");
    let output = synthesize(&out, &["--synthesis-seed", &max]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!out.exists());
}
