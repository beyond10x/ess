//! `ess verify conform mutate` on a `when_subject:` and a `when_related:` predicate
//! (<https://github.com/beyond10x/ess/issues/515>): the guard classes edit the predicate over the
//! stored subject and over the related row as they edit a `when:`, the report and the manifest
//! name each such site by the key it edits, and the target implementing the unchanged model kills
//! every one of them. The library's verdicts are decided in
//! `crates/verify/ess-conformance/tests/mutation_stored_guards.rs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const STORE: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-stored-guards.yaml";

const STORED_GUARD_MUTANTS: &[&str] = &[
    "guard-boundary/demo.store.Restock/full/when_subject/0",
    "guard-boundary/demo.store.Restock/full/when_subject/1",
    "guard-boundary/demo.store.Restock/full/when_subject/1-outward",
    "guard-boundary/demo.store.Take/too-little/when_related/0",
    "guard-boundary/demo.store.Take/too-little/when_related/1",
    "guard-boundary/demo.store.Take/too-little/when_related/1-outward",
    "guard-connective/demo.store.Restock/full/when_subject/0",
    "guard-connective/demo.store.Take/too-little/when_related/0",
    "guard-negate/demo.store.Restock/full/when_subject",
    "guard-negate/demo.store.Take/too-little/when_related",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn mutate(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "mutate", "--path", STORE])
        .args([
            "--class",
            "guard-boundary",
            "--class",
            "guard-negate",
            "--class",
            "guard-connective",
        ])
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

fn scratch(name: &str) -> PathBuf {
    let directory = root()
        .join("target/mutate-stored-guards")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.parent().expect("a parent")).expect("a scratch directory");
    directory
}

#[test]
fn every_stored_guard_mutant_is_killed_by_the_interpreted_target() {
    let output = mutate(&["--target", "interpreted", "--format", "json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/3", "{report:#}");
    let mutants = report["mutants"].as_array().expect("mutants");
    let ids: Vec<&str> = mutants
        .iter()
        .filter_map(|mutant| mutant["id"].as_str())
        .collect();
    assert_eq!(ids, STORED_GUARD_MUTANTS, "{report:#}");
    for mutant in mutants {
        assert_eq!(mutant["verdict"], "killed", "{mutant:#}");
        assert!(
            mutant["killers"]
                .as_array()
                .is_some_and(|killers| !killers.is_empty()),
            "{mutant:#}"
        );
    }
    assert_eq!(report["counts"]["killed"], STORED_GUARD_MUTANTS.len());
}

#[test]
fn the_text_report_names_the_key_each_mutant_edits() {
    let output = mutate(&["--target", "interpreted"]);
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(
        stdout.lines().any(|line| line.starts_with(
            "killed guard-negate/demo.store.Take/too-little/when_related: `when_related: \
             (stock < input.quantity or stock <= 0)` becomes `when_related: not ((stock < \
             input.quantity or stock <= 0))` — by "
        )),
        "{stdout}"
    );
}

/// The emitted manifest lists the same mutants with suites, in the manifest format a `when:`
/// guard mutant is emitted in: the sites are new, no persisted field is.
#[test]
fn the_manifest_lists_the_stored_guard_mutants_in_the_unchanged_format() {
    let directory = scratch("emit");
    let output = mutate(&["--emit", directory.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["format"], "ess-mutation-manifest/3",
        "{manifest:#}"
    );
    let mutants = manifest["mutants"].as_array().expect("mutants");
    let ids: Vec<&str> = mutants
        .iter()
        .filter_map(|mutant| mutant["id"].as_str())
        .collect();
    assert_eq!(ids, STORED_GUARD_MUTANTS, "{manifest:#}");
    for mutant in mutants {
        let dir = mutant["dir"].as_str().expect("a suite directory");
        assert!(
            directory.join(dir).join("suite.json").is_file(),
            "{mutant:#}"
        );
    }
}

/// `expiry < now` to `expiry <= now` on a stored `Timestamp`: the mutant's suite is the baseline's,
/// so it is `unwitnessed` naming that, and the audit exits 3 rather than 1
/// (<https://github.com/beyond10x/ess/issues/586>). The emit and collect route is decided in
/// `crates/verify/ess-conformance/tests/mutation_identical_answer.rs`.
#[test]
fn identical_suite_mutant_is_unwitnessed() {
    const CLOCKED: &str =
        "crates/verify/ess-conformance/tests/fixtures/mutation-clocked-expiry.yaml";
    const BOUNDARY: &str = "guard-boundary/demo.codes.Redeem/expired/when_subject/0";
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(root())
            .args(["verify", "conform", "mutate", "--path", CLOCKED])
            .args(["--class", "guard-boundary", "--target", "interpreted"])
            .args(args)
            .output()
            .expect("the `ess` binary runs")
    };
    let output = run(&["--format", "json"]);
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/7", "{report:#}");
    let entry = &report["mutants"][0];
    assert_eq!(entry["id"], BOUNDARY, "{report:#}");
    assert_eq!(entry["verdict"], "unwitnessed", "{entry:#}");
    assert_eq!(entry["unchanged_suite"], true, "{entry:#}");

    let output = run(&[]);
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(3), "{stdout}");
    assert!(
        stdout.lines().any(
            |line| line.starts_with(&format!("unwitnessed {BOUNDARY}: "))
                && line.contains(" — ESS-MUTATE-004: the suite does not change")
        ),
        "{stdout}"
    );
}
