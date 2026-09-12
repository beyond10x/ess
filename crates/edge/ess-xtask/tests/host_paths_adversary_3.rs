//! Adversarial cases against the three bounds `story:host-path-lane-detector-bounds` closed.
//!
//! The unit narrowed the detector (what may precede a marker), widened it (what a name is made
//! of), and rewrote the workflow parse. This file drives each change against the document the same
//! unit wrote about it — the module doc of `crates/edge/ess-xtask/tests/host_paths.rs`, the doc
//! comments of `begins_an_absolute_path`, `list_items` and `ci_runner_labels` — rather than
//! against the behaviour it shipped, because nothing else compares the two.
//!
//! Like `host_paths_adversary.rs` and `host_paths_adversary_2.rs`, every case drives the copy in
//! `host_paths_lane/mod.rs` and calls `assert_current()` first, so a lane edit turns these into a
//! named complaint rather than a measurement of a fossil. Nothing here spells a home-directory
//! marker as a literal; these bytes are tracked under `crates/` and the lane scans them.

mod host_paths_lane;

use host_paths_lane::{
    assert_current, ci_runner_labels, home_paths, HOME_MARKERS, LANE, RUNNER_HOME_ROOTS,
};
use std::fs;
use std::path::PathBuf;

/// A throwaway directory outside the repository, named so two runs cannot collide.
///
/// A workflow naming a platform CI does not run cannot be written into this repository: the lane's
/// own parse reads `.github/workflows/ci.yml` and would turn the gate red on a clean tree.
fn throwaway(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-host-paths-adversary-3-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the host clock is after the epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("a throwaway directory can be created");
    path
}

/// The labels `workflow` yields, read through the lane's own parse, leaving nothing behind.
///
/// The directory is removed before the caller asserts, so a red case does not retain a fixture
/// outside the repository on every run.
fn labels_of(label: &str, workflow: &str) -> std::collections::BTreeSet<String> {
    let root = throwaway(label);
    fs::create_dir_all(root.join(".github/workflows")).expect("the workflow directory is created");
    fs::write(root.join(".github/workflows/ci.yml"), workflow).expect("the workflow is written");
    let labels = ci_runner_labels(&root);
    fs::remove_dir_all(&root).expect("the throwaway directory can be removed");
    labels
}

/// Whether `label` is a runner the lane can look a home root up for.
fn resolvable(label: &str) -> bool {
    RUNNER_HOME_ROOTS
        .resolve()
        .iter()
        .any(|(family, _)| label.starts_with(family.as_str()))
}

/// A `runs-on:` written as a block sequence names a platform the parse cannot see at all.
///
/// `ci_runner_labels` exists so "adding a platform to CI without adding its home root to
/// `HOME_MARKERS` fails this lane instead of quietly narrowing it", and
/// `story:host-path-lane-detector-bounds` rewrote it to read every `runs-on:` value for exactly
/// that reason. It reads every `runs-on:` value written *on the same line*. GitHub Actions spells
/// `runs-on` two ways — a scalar or an inline sequence after the colon, and a block sequence on
/// the lines below it — and the second is the ordinary spelling for a self-hosted runner, which is
/// the ordinary way a platform with no row in `RUNNER_HOME_ROOTS` arrives.
///
/// For that spelling `value` is empty, `list_items` returns nothing, and the job contributes no
/// label. The lane is then narrowed in silence on a platform whose home root it has never heard
/// of, which is the one outcome that function's own doc says it exists to prevent — and
/// `list_items` states the opposite in its own words: "a shape this does not understand arrives at
/// the case below as itself rather than as silence".
#[test]
fn a_runs_on_written_as_a_block_sequence_is_not_silence() {
    assert_current();
    let workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  native:\n    runs-on:\n      - \
                    self-hosted\n      - freebsd-14\n";
    let labels = labels_of("block-sequence", workflow);
    assert!(
        labels.iter().any(|label| label == "ubuntu-latest"),
        "the parse did not find the runner it does know, so this case measures nothing: {labels:?}"
    );
    assert!(
        labels.iter().any(|label| label == "freebsd-14"),
        "`ci_runner_labels` in `{LANE}` must read every runner the workflow names or a platform \
         added to CI with no home root recorded for it narrows this gate in silence; a block \
         sequence is one of the two spellings GitHub Actions gives `runs-on:` and the ordinary one \
         for a self-hosted runner, and the parse read {labels:?}"
    );
}

/// A `runs-on:` spelling the old parse handled must not turn the gate red on a clean workflow.
///
/// The rewrite traded a substring search for a line parse, and the line parse recognises a matrix
/// expression only when the value is *exactly* `${{ matrix.<key> }}` and the key resolves through
/// a `key: [a, b]` or `- item` list. Two ordinary workflow spellings fall outside that: a trailing
/// YAML comment on the `runs-on:` line — a shape `.github/workflows/ci.yml` already uses on its
/// `uses:` lines — and a matrix written as `include:` entries. Each yields a "label" that is an
/// unresolved expression or a comment, and
/// `the_markers_cover_the_home_root_of_every_platform_ci_runs_on` panics on it by name: "`…` runs
/// CI and no home root is recorded for it", naming a platform that does not exist.
///
/// The substring search this replaced found `macos-15` in both workflows and passed. So this is
/// not a bound the rewrite left open, it is a clean repository the rewrite refuses — the failure
/// mode the lane's own comment calls the one that gets a gate switched off.
#[test]
fn a_runs_on_spelling_the_repository_could_write_tomorrow_does_not_invent_a_platform() {
    assert_current();
    let commented = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  native:\n    runs-on: ${{ \
                     matrix.runner }} # both shipped architectures\n    strategy:\n      \
                     matrix:\n        runner: [macos-15-intel, macos-15]\n";
    let included = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  native:\n    runs-on: ${{ \
                    matrix.os }}\n    strategy:\n      matrix:\n        include:\n          - os: \
                    macos-15\n";
    let mut invented = Vec::new();
    for (name, workflow) in [
        ("a trailing comment on the matrix expression", commented),
        ("a matrix written as `include:` entries", included),
    ] {
        let labels = labels_of("unresolved", workflow);
        assert!(
            labels.iter().any(|label| label == "ubuntu-latest"),
            "the parse did not find the runner it does know, so this case measures nothing: \
             {labels:?}"
        );
        for label in &labels {
            if !resolvable(label) {
                invented.push(format!("  {name}: `{label}`"));
            }
        }
    }
    assert!(
        invented.is_empty(),
        "each of these workflows runs on `ubuntu-latest` and a macOS runner and nothing else, and \
         the parse in `{LANE}` produced a label no row of `RUNNER_HOME_ROOTS` matches. \
         `the_markers_cover_the_home_root_of_every_platform_ci_runs_on` panics on it saying it \
         runs CI, so a clean repository turns this gate red — which the lane's own comment calls \
         the failure mode that gets a gate switched off:\n{}",
        invented.join("\n")
    );
}

/// An absolute home path on a unified-diff line is still an absolute home path.
///
/// `begins_an_absolute_path` refuses a marker whose preceding character is one `continues_a_path`
/// accepts, and `-` and `+` are two of them. In a unified diff they are not part of the path, they
/// are the column that says whether the line was removed or added — so the one record that spells
/// out a leak being taken out of a file is the one record this lane no longer reads.
///
/// `docs/reviews/` holds four tracked `.patch` files and a fenced diff block, all of them
/// inside the scanned trees, and the module doc opens by saying the wave-20 records carried this
/// very class before `a22d4d14` removed it by hand. The doc lists three bounds the lane does not
/// cover and this is not among them.
#[test]
fn an_absolute_home_path_on_a_diff_line_is_still_collected() {
    assert_current();
    let mut dropped = Vec::new();
    for marker in HOME_MARKERS {
        let leaked = format!("{marker}someone/.cache/ess/report.json");
        for column in ["-", "+"] {
            let text = format!("{column}{leaked}");
            let found = home_paths(&text);
            if !found.contains(&leaked) {
                dropped.push(format!(
                    "  a `{column}` diff column before `{leaked}` collected {found:?}"
                ));
            }
        }
    }
    assert!(
        dropped.is_empty(),
        "a unified-diff column is not part of the path that follows it, and `{LANE}` refuses an \
         absolute path under a user's home directory wherever it appears in a tracked file — \
         `docs/reviews/` carries tracked `.patch` files inside the scanned trees:\n{}",
        dropped.join("\n")
    );
}

/// The scanned trees hold no marker the narrowing drops, measured rather than supposed.
///
/// The green half. The unit's claim is that requiring a non-path character before a marker loses
/// no true positive here, and this is that claim as a case: every marker occurrence in every
/// tracked file under the scanned trees is preceded by a character the detector still accepts.
/// Without it the three cases above are a list of shapes with no statement about the repository.
#[test]
fn every_marker_in_a_scanned_file_survives_the_narrowing() {
    assert_current();
    let root = host_paths_lane::workspace_root();
    let mut lost = Vec::new();
    let mut occurrences = 0usize;
    for file in host_paths_lane::scanned_files(&root) {
        let full = root.join(&file);
        if !full.exists() {
            continue;
        }
        let bytes = fs::read(&full).expect("a tracked file the scan selected is readable");
        let text = String::from_utf8_lossy(&bytes).into_owned();
        for (number, line) in text.lines().enumerate() {
            let normalised = host_paths_lane::normalise_separators(line);
            for marker in HOME_MARKERS {
                let mut from = 0;
                while let Some(offset) = normalised[from..].find(marker.as_str()) {
                    let start = from + offset;
                    from = start + marker.len();
                    occurrences += 1;
                    if !host_paths_lane::begins_an_absolute_path(&normalised, start) {
                        lost.push(format!("  {file}:{}", number + 1));
                    }
                }
            }
        }
    }
    assert!(
        occurrences > 0,
        "no tracked file under the scanned trees holds a marker at all, so this case measures \
         nothing and the count the unit reported cannot be checked against it"
    );
    assert!(
        lost.is_empty(),
        "{} of {occurrences} marker occurrences in the scanned trees are refused by \
         `begins_an_absolute_path`, so the narrowing in `{LANE}` drops a match that used to be \
         examined:\n{}",
        lost.len(),
        lost.join("\n")
    );
}

/// The lane's own parse of the repository's workflow, held to the file rather than to itself.
///
/// `ci_runner_labels` reads `.github/workflows/ci.yml` only. The second half of the assertion is
/// the one the lane does not make: every `runs-on:` line in that file must contribute at least one
/// label, so a job whose platform the parse cannot see is named here rather than being absent from
/// a set nothing counts.
#[test]
fn every_runs_on_line_in_the_repository_workflow_contributes_a_label() {
    assert_current();
    let root = host_paths_lane::workspace_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("the CI workflow is readable");
    let written = workflow
        .lines()
        .filter(|line| line.trim_start().starts_with("runs-on:"))
        .count();
    let labels = ci_runner_labels(&root);
    assert!(
        written > 0,
        "`.github/workflows/ci.yml` names no runner, so this case measures nothing"
    );
    assert!(
        labels.len() >= written,
        "`.github/workflows/ci.yml` has {written} `runs-on:` lines and the parse in `{LANE}` \
         produced {} labels, so at least one job's platform is invisible to it: {labels:?}",
        labels.len()
    );
}
