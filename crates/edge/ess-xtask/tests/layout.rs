//! The crate tree's shape, and the literal paths that name it.
//!
//! This is the check `story:crates-under-area-directories` is accepted against. Two rules, both
//! mechanical. Every workspace crate sits under exactly one area directory, and every literal
//! `crates/…` path naming a workspace crate resolves on disk, in every file of the repository
//! except `target/`, `CHANGELOG.md`, `.engineering/` and the root `docs/` engineering record.
//!
//! The second rule is the one a move breaks silently. A fixture path in `Taskfile.yml`, a `../..`
//! in a test helper and a source path quoted in a doc comment all keep compiling while pointing at
//! nothing, so the check has to be a scan rather than a compilation. A scan has two ways of being
//! worthless — it can look at the wrong files, and it can look for the wrong shape — and both have
//! already happened here, so each has a case of its own below rather than being assumed.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, str};

/// The areas a crate can sit in.
///
/// `specify` carries an authored system to a validated IR, `generate` turns that IR into
/// artifacts, `verify` compares an implementation against it, `infra` is the separate bounded
/// context over an observed cluster, and `edge` is the binary and the repository's own tooling.
const AREAS: &[&str] = &["specify", "generate", "verify", "infra", "edge"];

/// Root-relative directory prefixes the path scan does not read.
///
/// **Root-relative, and per directory rather than per name.** What is excluded is the *dated*
/// record: `docs/design/` and `docs/reviews/` are argued at a moment and are not rewritten
/// afterwards, so a path in one of them is a citation of the tree as it stood. `docs/plan/` is not
/// dated — it is live, it is edited as the work moves, and a stale path in it is a defect like any
/// other — so it is scanned. `website/docs/` is published adopter-facing source (`b10x.docs.yaml`
/// declares `root: website`) and is scanned too; excluding by bare directory name would have taken
/// both of those out along with the record, which is how this scan first shipped.
///
/// `.engineering/` is the plan store, whose journal is append-only: a path named in a journal line
/// cannot be corrected by a later move even in principle. `target/` is build output.
const UNSCANNED_PREFIXES: &[&str] = &["target/", ".engineering/", "docs/design/", "docs/reviews/"];

/// Root-relative files the path scan does not read, for the same reason the dated record is.
const UNSCANNED_FILES: &[&str] = &["CHANGELOG.md", "docs/extraction.md"];

/// This file, root-relative, taken from the compiler rather than written down.
///
/// A scan must not read its own negative fixtures. The cases below have to name paths that are
/// deliberately absent — that is what they assert — and a scan that reported them would be
/// reporting itself. Asking [`file!`] rather than spelling the path keeps that true if this file
/// is ever moved, which is the very event the rest of the module exists to survive.
fn this_file() -> &'static str {
    file!()
}

/// The workspace root, found by walking up rather than by counting `..`.
///
/// Counting is what this test exists to catch, so it cannot be what the test relies on.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

/// The `crates/…` entries of `[workspace] members`, in manifest order.
///
/// Read a line at a time and a quoted value at a time, because the list carries comments and a
/// comment carries commas.
fn crate_members(manifest: &str) -> Vec<String> {
    manifest
        .split_once("members = [")
        .expect("the workspace names its members")
        .1
        .split_once(']')
        .expect("the member list closes")
        .0
        .lines()
        .filter_map(|line| {
            let quoted = line.split_once('#').map_or(line, |(before, _)| before);
            let (_, rest) = quoted.split_once('"')?;
            let (member, _) = rest.split_once('"')?;
            member.starts_with("crates/").then(|| member.to_owned())
        })
        .collect()
}

/// The name of every workspace crate, read from the manifest rather than listed here.
fn crate_names(manifest: &str) -> BTreeSet<String> {
    crate_members(manifest)
        .iter()
        .filter_map(|member| member.rsplit('/').next().map(str::to_owned))
        .collect()
}

/// Every file of the repository the path scan reads, root-relative.
///
/// Asked of `git` rather than walked, because `git` is what already knows which paths are ignored
/// and the scan must not read build output. `--cached --others --exclude-standard` is the set a
/// `rg` run from the root would read: tracked files plus untracked ones that are not ignored.
fn scanned_files(root: &Path) -> Vec<String> {
    let listed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .output()
        .expect("git lists the repository's files");
    assert!(listed.status.success(), "`git ls-files` failed");
    let mut files: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter(|file| {
            !UNSCANNED_FILES.contains(file)
                && *file != this_file()
                && !UNSCANNED_PREFIXES
                    .iter()
                    .any(|prefix| file.starts_with(prefix))
        })
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    files
}

/// Whether a byte can continue a path literal.
fn continues_a_path(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/')
}

/// Every `crates/…` path in `text` that names somewhere in this repository's own crate tree.
///
/// Two spellings resolve, and both must, because both appear: `crates/<area>/<crate>/…` is what
/// every path here looks like now, and `crates/<crate>/…` is what one looked like before the areas
/// existed — a stale pre-move path is exactly what this scan is for.
///
/// A path whose first segment is a known area belongs to this tree whether or not its second
/// segment still names a crate, so `crates/specify/ess-gone/…` is collected and reported rather
/// than passed over as somebody else's path.
///
/// A path naming something else under some `crates/` directory — a package in a generated tree, a
/// crate of another repository quoted in prose — is not this workspace's to keep true, and is not
/// collected.
fn quoted_crate_paths(text: &str, names: &BTreeSet<String>) -> BTreeSet<String> {
    const MARKER: &str = "crates/";
    let bytes = text.as_bytes();
    let mut found = BTreeSet::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find(MARKER) {
        let start = from + offset;
        from = start + MARKER.len();
        let mut end = from;
        while end < bytes.len() && continues_a_path(bytes[end]) {
            end += 1;
        }
        let path = text[start..end].trim_end_matches(['.', '/']);
        let Some(first) = path.split('/').nth(1) else {
            continue;
        };
        if AREAS.contains(&first) || names.contains(first) {
            found.insert(path.to_owned());
        }
    }
    found
}

#[test]
fn every_workspace_crate_lives_under_an_area_directory() {
    let root = workspace_root();
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest reads");
    let mut misplaced = Vec::new();
    let mut absent = Vec::new();
    for member in crate_members(&manifest) {
        let segments: Vec<&str> = member.split('/').collect();
        if !matches!(segments.as_slice(), [_, area, _] if AREAS.contains(area)) {
            misplaced.push(member.clone());
        }
        if !root.join(&member).join("Cargo.toml").is_file() {
            absent.push(member);
        }
    }
    assert!(
        absent.is_empty(),
        "a workspace member names a manifest that is not on disk: {absent:?}"
    );
    assert!(
        misplaced.is_empty(),
        "every crate belongs to one of the areas {AREAS:?}, at `crates/<area>/<crate>`; \
         these do not: {misplaced:?}"
    );
}

#[test]
fn every_literal_path_naming_a_workspace_crate_exists() {
    let root = workspace_root();
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest reads");
    let names = crate_names(&manifest);
    let mut collected = 0usize;
    let mut dangling = BTreeSet::new();
    for file in scanned_files(&root) {
        let Ok(text) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        for path in quoted_crate_paths(&text, &names) {
            collected += 1;
            if !root.join(&path).exists() {
                dangling.insert(format!("{file}: {path}"));
            }
        }
    }
    // The half without which the assertion below is a claim about nothing, in the spelling
    // `infra-spec`'s `the_validated_type_is_only_reachable_through_validation` already uses for the
    // same hazard: an empty scan is indistinguishable from a clean one.
    assert!(
        collected > 0,
        "the scan collected no path at all, or it has stopped looking at the right files"
    );
    assert!(
        dangling.is_empty(),
        "these name somewhere in this repository's crate tree at a path that does not exist \
         ({} of {collected} collected):\n{}",
        dangling.len(),
        dangling.into_iter().collect::<Vec<_>>().join("\n")
    );
}

/// The files the scan reads are chosen by root-relative path, not by directory name.
///
/// The distinction is not academic: `website/docs/` is adopter-facing published source and the
/// root `docs/` is the dated engineering record. A `file_name()` match excludes both, and it did —
/// silently, because a scan that reads fewer files reports fewer findings and still exits zero.
#[test]
fn the_path_scan_excludes_by_root_relative_path_and_reads_published_website_source() {
    let root = workspace_root();
    let files = scanned_files(&root);
    assert!(
        files.iter().any(|file| file.starts_with("website/docs/")),
        "`website/docs/` is published source and is in scope; the scan read none of it"
    );
    let excluded: Vec<&String> = files
        .iter()
        .filter(|file| {
            UNSCANNED_FILES.contains(&file.as_str())
                || UNSCANNED_PREFIXES
                    .iter()
                    .any(|prefix| file.starts_with(prefix))
        })
        .collect();
    assert!(
        excluded.is_empty(),
        "the scan read files it excludes: {excluded:?}"
    );
    assert!(
        !files.iter().any(|file| file == this_file()),
        "the scan read the file that defines it, whose fixtures are deliberately absent paths"
    );
    assert!(
        fs::read_to_string(root.join(this_file())).is_ok(),
        "`file!()` must name this file relative to the workspace root, and names `{}`",
        this_file()
    );
}

// --- Added by an adversarial pass, 2026-09-03. Both cases were red, and both are about the two
// --- functions above rather than about anything they scan.

/// The scan sees a path in the shape every literal path in this repository now takes.
///
/// `quoted_crate_paths` keys on the one segment directly after `crates/`. Before the area move
/// that segment was the crate name; after it, it is the area, and no area is a crate name — so
/// the rule the module doc states ("any text in this repository that names a workspace crate by
/// path names a path that exists") is enforced for no path this repository still contains.
#[test]
fn the_path_scan_reads_an_area_qualified_path() {
    let names: BTreeSet<String> = ["ess-domain".to_owned()].into_iter().collect();
    let text = "the argument is in `crates/specify/ess-domain/src/deleted.rs`, beside the type";
    let found = quoted_crate_paths(text, &names);
    assert!(
        found.contains("crates/specify/ess-domain/src/deleted.rs"),
        "a path under `crates/<area>/<crate>/` names a workspace crate and must be collected; \
         the scan collected {found:?}"
    );
}

/// The path scan is looking at something.
///
/// The anti-vacuity half this repository already writes for a source scan — `infra-spec`'s
/// `the_validated_type_is_only_reachable_through_validation` asserts its second read is *not*
/// empty, "or this scan has stopped looking at the right file". `every_literal_path_naming_a_
/// workspace_crate_exists` has no such half, and over the whole repository it collects nothing
/// at all: it now passes on the empty set and would pass on an empty repository.
#[test]
fn the_path_scan_finds_at_least_one_path_in_this_repository() {
    let root = workspace_root();
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest reads");
    let names = crate_names(&manifest);
    let mut collected = BTreeSet::new();
    let mut read = 0usize;
    for file in scanned_files(&root) {
        let Ok(text) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        read += 1;
        collected.extend(quoted_crate_paths(&text, &names));
    }
    assert!(
        !collected.is_empty(),
        "the path scan read {read} files and collected no path at all, so the assertion it \
         makes about them is vacuous"
    );
}
