//! Test suites leave nothing under `TMPDIR`.
//!
//! The acceptance check of `story:test-scratch-directories-are-removed-on-drop`: a shared
//! workstation's temporary directory once held 4,781 `ess-*` directories (18 GiB) left behind by
//! test runs, each named after the process that made it. This case runs a representative set of
//! this crate's test suites again with `TMPDIR` pointed at a fresh empty directory and fails,
//! naming the leaking prefix, when anything is left in it afterwards.
//!
//! It runs inside `task test-xtask`, which builds every test binary of this package before running
//! any of them. The unit suites are this binary run again with a filter; the integration suites
//! are the sibling binaries cargo built next to it, refused when missing or older than a source
//! file they were built from, so a stale binary is never what passes.

use crate::scratch::Scratch;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Each unit-test source that makes TMPDIR scratch, and the filter that runs those tests.
const UNIT_SUITES: &[(&str, &str)] = &[
    ("src/main.rs", "tests::sync_"),
    ("src/whats_changed.rs", "whats_changed::tests::"),
    (
        "src/infra_acceptance.rs",
        "infra_acceptance::tests::a_scratch_below_",
    ),
    ("src/site_data.rs", "site_data::tests::"),
    ("src/support.rs", "support::tests::adversary_actual_help_"),
];

/// Each integration-test binary of this package that makes TMPDIR scratch.
const INTEGRATION_SUITES: &[&str] = &[
    "adversary_release_preparation",
    "host_paths_adversary",
    "host_paths_adversary_2",
    "host_paths_adversary_3",
];

/// Where this check reads TMPDIR scratch being made, so a new maker cannot stay unlisted.
const MAKERS: [&str; 3] = ["temp_dir()", "crate::scratch::Scratch", "src/scratch.rs\"]"];

/// The prefix a leaked entry was named with: its name without the trailing numeric parts
/// (process id, sequence, clock) that make each run's name unique.
fn prefix(name: &str) -> String {
    let mut parts: Vec<&str> = name.split('-').collect();
    while parts.len() > 1
        && parts
            .last()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
    {
        parts.pop();
    }
    parts.join("-")
}

/// The leaked prefixes left in `tmp`, each with how many entries carry it.
fn leaks(tmp: &Path) -> BTreeMap<String, usize> {
    let mut found = BTreeMap::new();
    for entry in fs::read_dir(tmp)
        .expect("the leak directory is readable")
        .flatten()
    {
        *found
            .entry(prefix(&entry.file_name().to_string_lossy()))
            .or_insert(0) += 1;
    }
    found
}

/// The newest `<name>-<hash>` executable in `deps`, refused when a source it was built from is
/// newer than it.
fn sibling(deps: &Path, name: &str) -> PathBuf {
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(deps)
        .expect("the deps directory is readable")
        .flatten()
    {
        let file = entry.file_name().to_string_lossy().into_owned();
        let Some(hash) = file
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('-'))
        else {
            continue;
        };
        if hash.len() != 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let modified = entry.metadata().and_then(|meta| meta.modified()).unwrap();
        if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
            newest = Some((modified, entry.path()));
        }
    }
    let (built, binary) = newest.unwrap_or_else(|| {
        panic!(
            "the test binary `{name}` is not built next to {}; run `cargo test -p ess-xtask`, \
             which builds every test binary of the package before running any",
            deps.display()
        )
    });
    let depfile = binary.with_extension("d");
    let listed = fs::read_to_string(&depfile)
        .unwrap_or_else(|error| panic!("reading {}: {error}", depfile.display()));
    let sources = listed
        .lines()
        .next()
        .and_then(|line| line.split_once(": "))
        .map(|(_, sources)| {
            sources
                .split_whitespace()
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for source in sources {
        if let Ok(changed) = fs::metadata(&source).and_then(|meta| meta.modified()) {
            assert!(
                changed <= built,
                "the test binary {} is older than {}; run `cargo test -p ess-xtask` so that \
                 the current source is what this check runs",
                binary.display(),
                source.display()
            );
        }
    }
    binary
}

/// Runs `command` with `TMPDIR` set to a fresh empty directory and returns what it left there.
fn run_in_empty_tmpdir(label: &str, command: &mut Command) -> BTreeMap<String, usize> {
    let tmp = Scratch::new("ess-scratch-leak-check");
    let output = command
        .env("TMPDIR", tmp.path())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap_or_else(|error| panic!("running {label}: {error}"));
    assert!(
        output.status.success(),
        "{label} failed under an empty TMPDIR:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    leaks(tmp.path())
}

/// The test code of a source file: the whole of an integration test, and what follows the
/// `#[cfg(test)]` module of a unit.
fn test_code(path: &Path) -> String {
    let source = fs::read_to_string(path).unwrap();
    if path.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")) {
        return source;
    }
    source
        .find("#[cfg(test)]\nmod tests")
        .map_or_else(String::new, |at| source[at..].to_owned())
}

fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leaked_entry_is_named_by_its_prefix_without_the_run_specific_numbers() {
        assert_eq!(
            prefix("ess-support-adversary-refusal-4242-1759912345678901234"),
            "ess-support-adversary-refusal"
        );
        assert_eq!(prefix("ess-xtask-sync-4242-0"), "ess-xtask-sync");
        assert_eq!(prefix("ess-go-parity-x1"), "ess-go-parity-x1");
        assert_eq!(prefix("4242"), "4242");
    }

    #[test]
    fn every_source_that_makes_tmpdir_scratch_is_run_by_the_leak_check() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        rust_files(&root.join("src"), &mut files);
        rust_files(&root.join("tests"), &mut files);
        let own = [
            root.join("src/scratch.rs"),
            root.join("src/scratch_leaks.rs"),
        ];
        let mut unlisted = Vec::new();
        for file in files.iter().filter(|file| !own.contains(file)) {
            let code = test_code(file);
            if !MAKERS.iter().any(|maker| code.contains(maker)) {
                continue;
            }
            let relative = file.strip_prefix(root).unwrap();
            let listed = UNIT_SUITES
                .iter()
                .any(|(source, _)| Path::new(source) == relative)
                || INTEGRATION_SUITES
                    .iter()
                    .any(|name| relative == Path::new("tests").join(format!("{name}.rs")));
            if !listed {
                unlisted.push(relative.display().to_string());
            }
        }
        assert!(
            unlisted.is_empty(),
            "these sources make TMPDIR scratch in test code and the leak check does not run \
             them; add them to UNIT_SUITES or INTEGRATION_SUITES: {unlisted:?}"
        );
    }

    #[test]
    fn the_representative_suites_leave_nothing_under_tmpdir() {
        let this = std::env::current_exe().expect("the running test binary has a path");
        let deps = this.parent().expect("a test binary lives in a directory");
        let mut leaked: Vec<String> = Vec::new();

        let mut unit = Command::new(&this);
        for (_, filter) in UNIT_SUITES {
            unit.arg(filter);
        }
        unit.args(["--skip", "scratch_leaks::"]);
        for (leak, count) in run_in_empty_tmpdir("the ess-xtask unit tests", &mut unit) {
            leaked.push(format!("ess-xtask unit tests left {count} × `{leak}-*`"));
        }

        for name in INTEGRATION_SUITES {
            let binary = sibling(deps, name);
            for (leak, count) in run_in_empty_tmpdir(name, &mut Command::new(&binary)) {
                leaked.push(format!("{name} left {count} × `{leak}-*`"));
            }
        }

        assert!(
            leaked.is_empty(),
            "test suites left scratch under TMPDIR; each must be removed when its guard drops:\n{}",
            leaked.join("\n")
        );
    }
}
