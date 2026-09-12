//! The lane's own detector, transcribed byte for byte, and its constants read from its source.
//!
//! Two adversarial targets share this module: `host_paths_adversary.rs` and
//! `host_paths_adversary_2.rs`. Both attack `crates/edge/ess-xtask/tests/host_paths.rs`, and
//! neither may edit it, so the code they drive has to be a copy. A copy is worth nothing unless
//! something proves it is still the original, and the first adversary pass proved it with a
//! byte-comparison that then went stale the moment the lane was corrected — four of its five cases
//! reported the behaviour of a frozen detector rather than the behaviour of the lane.
//!
//! So the copy is made as mechanical as a copy can be:
//!
//! * every function below is the lane's own text, from its signature to its closing brace, with
//!   **no substitution at all**. [`transcription_drift`] extracts both and compares them, and each
//!   adversarial target asserts the result is empty. A lane edit turns that case red and names the
//!   function to re-copy; it cannot go quiet.
//! * every constant the lane's detector reads is parsed out of the lane's source at run time
//!   rather than written down here, so a constant *cannot* drift. That is also why this file
//!   carries no home-directory marker as a literal: a text scanner cannot tell a control from a
//!   leak, which is the reason the lane stopped spelling its own controls, and these files are
//!   tracked under `crates/` like any other.
//!
//! The statics below stand in for the lane's `const` arrays. They are zero-sized handles that
//! resolve to the parsed values, which is what lets the transcribed bodies name `HOME_MARKERS`,
//! `SEPARATOR_SPELLINGS`, `SCANNED_PREFIXES` and `RUNNER_HOME_ROOTS` in the lane's own spelling.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::{fs, io, str};

/// The lane under attack, root-relative.
pub const LANE: &str = "crates/edge/ess-xtask/tests/host_paths.rs";

/// Every function copied from the lane, by the signature that identifies it in both sources.
///
/// [`transcription_drift`] walks this list. A function added here that the lane does not define
/// fails loudly rather than being skipped.
pub const TRANSCRIBED: &[&str] = &[
    "fn workspace_root() -> PathBuf {",
    "fn scanned_files(root: &Path) -> Vec<String> {",
    "fn continues_a_path(byte: u8) -> bool {",
    "fn normalise_separators(text: &str) -> String {",
    "fn home_paths(text: &str) -> BTreeSet<String> {",
    "fn home_paths_in(path: &Path) -> Vec<(usize, String)> {",
    "fn ci_runner_labels(root: &Path) -> BTreeSet<String> {",
];

/// The name of each `&[&str]` constant this module resolves out of the lane's source.
const STRING_CONSTANTS: &[&str] = &["HOME_MARKERS", "SEPARATOR_SPELLINGS", "SCANNED_PREFIXES"];

/// The name of each `&[(&str, &str)]` constant this module resolves out of the lane's source.
const PAIR_CONSTANTS: &[&str] = &["RUNNER_HOME_ROOTS"];

/// The lane's source text, read once.
pub fn lane_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        fs::read_to_string(workspace_root().join(LANE)).expect("the lane under attack is readable")
    })
}

/// This module's own source text, read once.
fn transcription_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        fs::read_to_string(workspace_root().join(file!())).expect("this module is readable")
    })
}

/// The text of one free function in `source`, from its signature to its closing brace.
///
/// The signature is looked for at the start of a line, so a signature quoted inside a string
/// literal — [`TRANSCRIBED`] is full of them — cannot be mistaken for the definition.
fn function_text<'a>(source: &'a str, signature: &str) -> &'a str {
    // The copies below are `pub` and the lane's are not; visibility is not part of the copy.
    let start = source
        .find(&format!("\npub {signature}"))
        .map(|offset| offset + "\npub ".len())
        .or_else(|| {
            source
                .find(&format!("\n{signature}"))
                .map(|offset| offset + 1)
        })
        .unwrap_or_else(|| panic!("`{signature}` is defined at column zero in this source"));
    let rest = &source[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("`{signature}` is closed at column zero"));
    &rest[..end + 2]
}

/// Every function whose copy below is no longer the lane's own text, as a complaint each.
///
/// Empty is the only acceptable answer, and each adversarial target asserts it. This is the whole
/// of the transcription's warranty: the cases attack the lane's bytes exactly as long as this is
/// empty, and the moment the lane changes they say so instead of quietly testing a fossil.
pub fn transcription_drift() -> Vec<String> {
    TRANSCRIBED
        .iter()
        .filter_map(|signature| {
            let theirs = function_text(lane_source(), signature);
            let mine = function_text(transcription_source(), signature);
            (theirs != mine).then(|| {
                format!(
                    "`{signature}` has drifted; the lane now reads:\n{theirs}\nand this copy \
                     reads:\n{mine}\nre-copy it from `{LANE}` and re-read the cases that drive it"
                )
            })
        })
        .collect()
}

/// Every string literal in `text`, with `\\` and `\"` unescaped.
fn string_literals(text: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }
        let mut literal = String::new();
        loop {
            match characters.next() {
                Some('"') | None => break,
                Some('\\') => literal.push(characters.next().unwrap_or('\\')),
                Some(other) => literal.push(other),
            }
        }
        literals.push(literal);
    }
    literals
}

/// The literals of one `const NAME: … = &[…];` array in the lane's source.
fn constant_literals(name: &str) -> Vec<String> {
    let source = lane_source();
    let start = source
        .find(&format!("\nconst {name}:"))
        .unwrap_or_else(|| panic!("the lane defines `{name}`"));
    let rest = &source[start..];
    let end = rest
        .find("];")
        .unwrap_or_else(|| panic!("`{name}` is an array literal"));
    let literals = string_literals(&rest[..end]);
    assert!(
        !literals.is_empty(),
        "`{name}` resolved to nothing, so every case that reads it would measure nothing"
    );
    literals
}

/// A handle on one of the lane's `&[&str]` constants.
#[derive(Clone, Copy)]
pub struct LaneStrings(&'static str);

impl LaneStrings {
    /// The constant's values, parsed from the lane's source once.
    pub fn resolve(self) -> &'static [String] {
        static CACHE: OnceLock<BTreeMap<&'static str, Vec<String>>> = OnceLock::new();
        CACHE.get_or_init(|| {
            STRING_CONSTANTS
                .iter()
                .map(|name| (*name, constant_literals(name)))
                .collect()
        })[self.0]
            .as_slice()
    }

    /// The constant's values, borrowed one at a time.
    pub fn iter(self) -> std::slice::Iter<'static, String> {
        self.resolve().iter()
    }

    /// Whether the constant holds `value`.
    pub fn contains(self, value: &str) -> bool {
        self.resolve().iter().any(|held| held == value)
    }
}

impl IntoIterator for LaneStrings {
    type Item = &'static String;
    type IntoIter = std::slice::Iter<'static, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A handle on one of the lane's `&[(&str, &str)]` constants.
#[derive(Clone, Copy)]
pub struct LanePairs(&'static str);

impl LanePairs {
    /// The constant's pairs, parsed from the lane's source once.
    pub fn resolve(self) -> &'static [(String, String)] {
        static CACHE: OnceLock<BTreeMap<&'static str, Vec<(String, String)>>> = OnceLock::new();
        CACHE.get_or_init(|| {
            PAIR_CONSTANTS
                .iter()
                .map(|name| {
                    let literals = constant_literals(name);
                    assert!(
                        literals.len() % 2 == 0,
                        "`{name}` is a list of pairs and parsed to {} literals",
                        literals.len()
                    );
                    let pairs = literals
                        .chunks_exact(2)
                        .map(|pair| (pair[0].clone(), pair[1].clone()))
                        .collect();
                    (*name, pairs)
                })
                .collect()
        })[self.0]
            .as_slice()
    }

    /// The constant's pairs, borrowed one at a time.
    pub fn iter(self) -> std::slice::Iter<'static, (String, String)> {
        self.resolve().iter()
    }
}

impl IntoIterator for LanePairs {
    type Item = &'static (String, String);
    type IntoIter = std::slice::Iter<'static, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The lane's `HOME_MARKERS`, read from its source.
pub static HOME_MARKERS: LaneStrings = LaneStrings("HOME_MARKERS");

/// The lane's `SEPARATOR_SPELLINGS`, read from its source.
pub static SEPARATOR_SPELLINGS: LaneStrings = LaneStrings("SEPARATOR_SPELLINGS");

/// The lane's `SCANNED_PREFIXES`, read from its source.
pub static SCANNED_PREFIXES: LaneStrings = LaneStrings("SCANNED_PREFIXES");

/// The lane's `RUNNER_HOME_ROOTS`, read from its source.
pub static RUNNER_HOME_ROOTS: LanePairs = LanePairs("RUNNER_HOME_ROOTS");

// -------------------------------------------------------------------------------------------
// Below this line every function is the lane's own text, copied without substitution.
// `transcription_drift` proves it. Do not edit one of them: re-copy it.
// -------------------------------------------------------------------------------------------

/// The lane's `workspace_root`, copied.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

/// The lane's `scanned_files`, copied. Reads `SCANNED_PREFIXES` above.
pub fn scanned_files(root: &Path) -> Vec<String> {
    let listed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--cached", "-z"])
        .output()
        .expect("git lists the repository's files");
    assert!(listed.status.success(), "`git ls-files` failed");
    let mut files: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .split('\0')
        .filter(|file| {
            !file.is_empty()
                && SCANNED_PREFIXES
                    .iter()
                    .any(|prefix| file.starts_with(prefix))
        })
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    files
}

/// The lane's `continues_a_path`, copied.
pub fn continues_a_path(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b'+' | b'@')
}

/// The lane's `normalise_separators`, copied. Reads `SEPARATOR_SPELLINGS` above.
pub fn normalise_separators(text: &str) -> String {
    let mut normalised = text.to_owned();
    for spelling in SEPARATOR_SPELLINGS {
        if normalised.contains(spelling) {
            normalised = normalised.replace(spelling, "/");
        }
    }
    normalised
}

/// The lane's `home_paths`, copied. Reads `HOME_MARKERS` above.
pub fn home_paths(text: &str) -> BTreeSet<String> {
    let normalised = normalise_separators(text);
    let bytes = normalised.as_bytes();
    let mut found = BTreeSet::new();
    for marker in HOME_MARKERS {
        // The marker without its trailing separator is the directory itself. A candidate no longer
        // than this names that directory and nothing inside anybody's home.
        let directory = marker.trim_end_matches('/');
        let mut from = 0;
        while let Some(offset) = normalised[from..].find(marker) {
            let start = from + offset;
            from = start + marker.len();
            let mut end = from;
            while end < bytes.len() && continues_a_path(bytes[end]) {
                end += 1;
            }
            let candidate = normalised[start..end].trim_end_matches(['.', '/']);
            if candidate.len() > directory.len() {
                found.insert(candidate.to_owned());
            }
        }
    }
    found
}

/// The lane's `home_paths_in`, copied.
pub fn home_paths_in(path: &Path) -> Vec<(usize, String)> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => panic!(
            "a tracked file the scan selected could not be read: {} ({error})",
            path.display()
        ),
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .enumerate()
        .flat_map(|(index, line)| {
            home_paths(line)
                .into_iter()
                .map(move |found| (index + 1, found))
        })
        .collect()
}

/// The lane's `ci_runner_labels`, copied. Reads `RUNNER_HOME_ROOTS` above.
pub fn ci_runner_labels(root: &Path) -> BTreeSet<String> {
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("the CI workflow is readable");
    let mut labels = BTreeSet::new();
    for (family, _) in RUNNER_HOME_ROOTS {
        let mut from = 0;
        while let Some(offset) = workflow[from..].find(family) {
            let start = from + offset;
            from = start + family.len();
            let end = workflow[start..]
                .find(|byte: char| !byte.is_ascii_alphanumeric() && byte != '-')
                .map_or(workflow.len(), |length| start + length);
            // A label is `<family>-<something>`; the bare family name in prose is not one.
            if end > start + family.len() {
                labels.insert(workflow[start..end].to_owned());
            }
        }
    }
    labels
}
/// The copies above are still the lane's own text, or this fails and says which is not.
///
/// Called at the head of every case in both adversarial targets, so that no case can report the
/// behaviour of a fossil. It is the whole reason those cases are evidence about the lane.
pub fn assert_current() {
    let drift = transcription_drift();
    assert!(
        drift.is_empty(),
        "the copies in this module are no longer the lane's own text, so every case that drives \
         them measures a fossil rather than `{LANE}`:\n\n{}",
        drift.join("\n\n")
    );
}
