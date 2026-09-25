//! No file this repository tracks under `crates/`, `docs/`, `website/` or `models/` names a
//! path inside somebody's home directory.
//!
//! This is the check `story:fixtures-carry-workstation-paths` is accepted against: no tracked file
//! under `crates/`, `docs/`, `website/` or `models/` contains a path under a user's home
//! directory, and a gate lane refuses one. Two coverage fixtures carried the workstation's managed
//! worktree root from 874962d until this lane existed, and the wave-20 records carried the same
//! class before a22d4d14 removed it by hand — which is the reason this is a scan rather than a
//! review habit. A hand pass closes the instances it is shown; it closed two files in a22d4d14 and
//! left four lines of the same class in one of the two files it edited.
//!
//! What is refused is an *absolute* home-directory path, because that is what leaks: it names a
//! machine, a user account and a directory layout that no other reader has. The portable spelling
//! `~/…` names none of those and is what this repository's records were rewritten *to*, so it is
//! deliberately not refused — refusing it would delete the remedy along with the defect.
//!
//! What it does not cover, each measured rather than supposed:
//!
//! * a home directory outside [`HOME_MARKERS`] — an account at `/var/lib/…`, a container's
//!   `/github/home`. The markers are the two platforms CI runs on plus the superuser's, and a host
//!   that puts a home somewhere else is a fact about that host;
//!   [`the_detector_finds_the_home_directory_this_process_runs_under`] skips rather than fails for
//!   exactly that reason.
//! * an account name written with a decomposed combining mark. [`continues_a_path`] accepts a
//!   character a name is made of, and a combining mark is not alphanumeric, so such a name is
//!   collected truncated at the mark — a finding, but one naming a shorter path than the leak.
//! * a Windows home written with the native separator — a drive letter, a colon, and then
//!   backslashes — which contains no marker at all. Written with forward slashes, which is what
//!   `git` and `cargo` print, the part after the drive letter *is* exactly the macOS marker and an
//!   account name, so it is collected — but with the drive chopped off, because the drive letter's
//!   `:` is where the path is taken to begin. The finding still names the file and line,
//!   so the leak is refused and a reader can look; the path in the message is two characters
//!   short. Collecting it whole means reading *backwards* from a marker into a drive prefix, and
//!   the honest form of that is a Windows home root in [`HOME_MARKERS`] and a native-separator
//!   detector. No workflow here runs Windows, [`RUNNER_HOME_ROOTS`] deliberately has no row for
//!   one, and a table that reports a platform as covered when it is not is worse than one that
//!   does not mention it — so this waits for the workflow that needs it.
//! * a `runs-on:` value [`runner_labels`] cannot resolve — an expression that is not a matrix
//!   reference, a matrix key the job defines nowhere, or a runner group named without `labels:`.
//!   Such a line contributes no label, because answering with the unresolved text turns the gate
//!   red on a clean workflow by naming a platform that does not exist. It is silent only for a
//!   workflow this repository does not hold:
//!   [`the_workflow_parse_reads_every_runner_the_workflow_names`] measures
//!   `.github/workflows/ci.yml` **per `runs-on:` line** and names any line that contributes
//!   nothing, so a job here whose platform this gate cannot see is reported rather than skipped.
//!   A count of labels is not that measurement in either direction — two jobs sharing a runner is
//!   two lines and one label, and a line resolving to nothing hides behind a line naming three.
//!
//! Those four trees are the whole of that sentence, and they are named in it rather than
//! qualified below it: the first paragraph is all rustdoc renders beside this module in an index,
//! in a search result or in a re-export listing, and a scope stated seventy lines further down
//! does not travel with it.
//!
//! What it does not read at all, tree by tree:
//!
//! * `.engineering/` — unread, 0 files, 0 lines: the planning store. Those counts are measured
//!   on every run by
//!   [`every_unread_tree_that_carries_the_class_is_named_in_the_module_documentation`], which is
//!   the only reason to believe them. They were 61 files and 32806 lines until 2026-09-22, then 3
//!   files and 4 lines until 2026-09-24. The tree stays unread because it quotes this lane's own
//!   markers, in the review results that measured the detector, in shapes the narrowing refuses.
//!
//! The first count fell when the store was migrated to Eventlog authority: the import read a
//! *source*, and the source was rewritten before it was read — 101830 absolute paths across 81
//! files, to the portable `~/…` spelling this lane admits. Scrubbing the store in place could not
//! have done it, because the journal was append-only and a body corrected after the import appends
//! a new blob and leaves the old one reachable. `story:scrub-the-planning-store-or-say-why-not` is
//! closed by that migration, not by a scan widening.
//!
//! The second fell when the store moved to `aep.project/3`: `aep plan store export` rewrote every
//! home path, the portable spelling included, to `workspace:<path>` or `home-path:sha256:<digest>`,
//! and with them the bare markers and the one account name carrying a combining mark that
//! `story:host-path-lane-detector-bounds` and the wave-22 adversary results quoted. A planning
//! write that names a home directory is refused by `aep plan artifact validate`, and it would
//! turn the counts above red here.
//!
//! `layout.rs` also excludes this tree. That is not authority to borrow: [`SCANNED_PREFIXES`] below
//! says in its own words that the two scans refuse different things — a stale repository-relative
//! citation is not a defect and a workstation path is.
//!
//! That is the whole of that list.
//!
//! A scan has two ways of being worthless, and `layout.rs` beside this file learned both: it can
//! read the wrong files, and it can be looking for a shape nothing has. Each has a case of its own
//! below rather than being assumed, because either failure exits zero.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io, str};

/// The trees whose tracked files carry this rule, as root-relative prefixes.
///
/// These four are the acceptance statement's own list. `crates/` is source and fixtures,
/// `website/` is published adopter-facing source, `models/` is authored specification, and `docs/`
/// is the engineering record — including `docs/design/` and `docs/reviews/`, which `layout.rs`
/// excludes from *its* scan and which are in scope for this one. The distinction is that the two
/// scans refuse different things. A `crates/…` path in a dated record is a citation of the tree as
/// it stood and is not a defect when a later move invalidates it; a workstation path in a dated
/// record is a leak on the day it is written and stays one. a22d4d14 scrubbed `docs/reviews/`, so
/// this is the repository's own answer rather than this file's.
const SCANNED_PREFIXES: &[&str] = &["crates/", "docs/", "website/", "models/"];

/// The absolute prefixes that place what follows inside some user's home directory.
///
/// `/home/` is Linux and `/Users/` is macOS, each followed by an account name; `/root/` is the
/// superuser's home and has no account segment. Only `/home/` has ever appeared here, and the
/// other two are refused anyway: the defect is the class, and a contributor on macOS would
/// introduce the one spelling a `/home/`-only check cannot see.
const HOME_MARKERS: &[&str] = &["/home/", "/Users/", "/root/"];

/// This file, root-relative, taken from the compiler rather than written down.
///
/// Used to hold this file to the rule it enforces, not to exempt it from one. The controls below
/// construct the paths this lane refuses rather than spelling them, so nothing in these bytes is a
/// finding and the file can be scanned like every other tracked file under `crates/` — which is
/// what [`the_scan_reads_each_named_tree_and_holds_this_file_to_the_same_rule`] asserts, in both
/// directions.
///
/// Asking [`file!`] rather than writing the path down keeps that true if this file is ever moved.
fn this_file() -> &'static str {
    file!()
}

/// The workspace root, found by walking up rather than by counting `..`.
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

/// Every tracked file under the scanned trees, root-relative.
///
/// Asked of `git` rather than walked, because the acceptance statement is about what the
/// repository *tracks* and `git` is what knows. `ls-files --cached` is exactly that set: committed
/// files plus staged ones, and no build output, no ignored file and nothing a reader of the
/// repository would never receive.
///
/// `-z`, and split on NUL, because `core.quotePath` is unset here and unset is `true`: `git`
/// wraps any path holding a non-ASCII byte in `"` and octal-escapes it. Such a line starts with a
/// quote, matches none of [`SCANNED_PREFIXES`], and the file leaves the scan before it is ever
/// read — invisible to every count, because it was never selected to be counted. `-z` turns the
/// quoting off at the source rather than undoing it afterwards.
///
/// Nothing is excluded, this file included. An unconditional exemption would hold exactly one
/// tracked file under `crates/` outside the rule the lane exists to enforce, and it would be the
/// file a leak is least likely to be noticed in.
fn scanned_files(root: &Path) -> Vec<String> {
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

/// Whether a character can continue a path literal.
///
/// A character rather than a byte, and alphanumeric rather than ASCII-alphanumeric, because an
/// account name is not ASCII on either platform CI runs on and the acceptance statement refuses a
/// path under *a user's* home directory without qualifying whose. Asked of bytes, a name beginning
/// outside ASCII ended the candidate immediately and the path left the scan entirely; a name
/// merely containing one was collected truncated, which is a finding naming a file that exists on
/// no host.
///
/// `is_alphanumeric` rather than "not ASCII", so the replacement character
/// [`String::from_utf8_lossy`] leaves behind still ends a path — the property [`home_paths_in`]
/// relies on to read a tracked binary without joining two of its paths into one finding that is
/// neither. A combining mark is not alphanumeric either, so an account name written decomposed is
/// still collected truncated; the module documentation states that bound.
fn continues_a_path(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | '.' | '/' | '+' | '@')
}

/// Whether the marker at `start` in `text` begins an absolute path rather than continuing one.
///
/// The detector looks for its markers anywhere in a line, and what precedes one decides whether it
/// is a path at all: `pages/` then a marker's own name is a component of a *relative* reference,
/// and so is a URL's path and a JSON Pointer. Collecting one reports an absolute path with its
/// prefix cut away — a file on no host — which is the failure mode that gets a gate switched off.
/// So the character before the marker must be one no path is made of.
///
/// Three absolute paths are preceded by exactly such a character and must survive this. A run of
/// separators before a marker leaves what follows absolute, which is how a `file:` URL spells a
/// local path, so the run is stepped over rather than read. And a serialized line break ends in a
/// letter, which is a byte a directory name is made of, so `\n` and its siblings are a boundary.
///
/// A unified diff's `-` or `+` column is the third. It is not part of the path that follows it, it
/// is the mark that says whether the line was removed or added, and both characters are ones a path
/// is made of — so without this the one record that spells out a leak *being taken out of a file*
/// is the one record this lane cannot read. `docs/reviews/` carries tracked `.patch` files inside
/// the scanned trees. It is a column only when it is the whole of the line before the marker;
/// `my-dir/` and a marker's own name is still a relative reference.
///
/// `~` is refused as a predecessor although no path is made of it: the portable spelling this
/// repository's records were rewritten *to* starts with one, and a marker's own name is an
/// ordinary directory to have inside a home directory.
fn begins_an_absolute_path(text: &str, start: usize) -> bool {
    let before = text[..start].trim_end_matches('/');
    let Some(previous) = before.chars().next_back() else {
        return true;
    };
    let escaped = matches!(previous, 'n' | 'r' | 't' | '0')
        && before[..before.len() - previous.len_utf8()].ends_with('\\');
    let diff_column = matches!(before.trim_start(), "-" | "+");
    escaped || diff_column || (!continues_a_path(previous) && previous != '~')
}

/// The spellings of `/` a serializer in this repository emits, other than `/` itself.
///
/// The acceptance statement refuses a tracked file that *contains* a path under a user's home
/// directory and says nothing about how the separator is written. Each of these is a spelling this
/// repository's own toolchain produces: eight tracked files under `crates/` already carry `\/`,
/// which is what a JSON writer with escaped solidus emits; `%2F` is what a URL path carries; and
/// `/` is the same solidus written as a JSON unicode escape. Normalising before matching
/// costs one allocation per line and, measured over all 1016 selected files, introduces no finding
/// that the plain spelling did not already produce.
const SEPARATOR_SPELLINGS: &[&str] = &["\\u002F", "\\u002f", "%2F", "%2f", "\\/"];

/// `text` with every spelling of the path separator written as `/`.
fn normalise_separators(text: &str) -> String {
    let mut normalised = text.to_owned();
    for spelling in SEPARATOR_SPELLINGS {
        if normalised.contains(spelling) {
            normalised = normalised.replace(spelling, "/");
        }
    }
    normalised
}

/// Every absolute home-directory path in `text`.
///
/// A marker alone is not a path — `/home/` naming the containing directory in prose leaks nothing
/// — so something must follow it that is part of a name. "Something" cannot be "any byte
/// `continues_a_path` accepts", because `.` is one: a marker ending a sentence would then satisfy
/// it, and the trailing trim would take the stop *and* the marker's own slash away, leaving a
/// finding whose whole text is `/home` — a directory on every Linux host, reported as a leak. A
/// gate that refuses a clean file is the failure mode that gets a gate switched off, so what is
/// required is that something survives the trim: the collected path must be longer than the
/// directory the marker names.
///
/// A trailing `.` or `/` is sentence punctuation rather than part of the path and is dropped,
/// which keeps the same path spelled two ways in two files from being reported as two findings.
///
/// What precedes the marker decides whether there is a path here at all, and
/// [`begins_an_absolute_path`] is where that is asked. A marker in the middle of a relative
/// reference is a directory component and names nobody's home.
fn home_paths(text: &str) -> BTreeSet<String> {
    let normalised = normalise_separators(text);
    let mut found = BTreeSet::new();
    for marker in HOME_MARKERS {
        // The marker without its trailing separator is the directory itself. A candidate no longer
        // than this names that directory and nothing inside anybody's home.
        let directory = marker.trim_end_matches('/');
        let mut from = 0;
        while let Some(offset) = normalised[from..].find(marker) {
            let start = from + offset;
            from = start + marker.len();
            if !begins_an_absolute_path(&normalised, start) {
                continue;
            }
            let end = normalised[from..]
                .char_indices()
                .find(|(_, character)| !continues_a_path(*character))
                .map_or(normalised.len(), |(offset, _)| from + offset);
            let candidate = normalised[start..end].trim_end_matches(['.', '/']);
            if candidate.len() > directory.len() {
                found.insert(candidate.to_owned());
            }
        }
    }
    found
}

/// Every home-directory path in one file, as `line: path` pairs.
///
/// Bytes rather than [`fs::read_to_string`]. The acceptance statement is about tracked files, not
/// about tracked files that happen to decode as UTF-8, and this repository tracks two that do not
/// — `website/static/img/favicon.ico` and `website/static/img/social-card.png`. A `read_to_string`
/// that fails leaves a caller with nothing to do but skip the file, and a skipped file is
/// indistinguishable from a cleared one in an exit status. A marker is ASCII, so
/// [`String::from_utf8_lossy`] cannot hide one: the replacement character is not a character
/// [`continues_a_path`] accepts — it is not alphanumeric — so an undecodable run ends a path
/// rather than joining two.
/// Nothing, when the working tree does not hold the file at all.
///
/// `git ls-files --cached` lists what the *index* holds, and an unstaged deletion leaves a path
/// listed that the tree no longer has. That is an ordinary state of somebody's checkout and not a
/// defect in the repository, so it must not abort the scan: aborting reports nothing about the
/// other thousand files and puts the blame in the exit status of the scan rather than on the
/// deletion. Every other read error is still fatal, because it means the scan cannot do its job
/// and silence about that is the failure this whole lane exists to prevent.
///
/// The caller, not this function, is what distinguishes "absent" from "clean" — it asks the tree
/// before reading and counts the two apart. Returning nothing for a file that is not there keeps
/// that decision in one place and keeps this function total.
fn home_paths_in(path: &Path) -> Vec<(usize, String)> {
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

#[test]
fn no_tracked_file_under_the_published_trees_names_a_home_directory_path() {
    let root = workspace_root();
    let selected = scanned_files(&root);
    let mut examined = 0usize;
    let mut absent = Vec::new();
    let mut leaked = Vec::new();
    for file in &selected {
        let full = root.join(file);
        if !full.exists() {
            absent.push(file.clone());
            continue;
        }
        examined += 1;
        for (line, path) in home_paths_in(&full) {
            leaked.push(format!("{file}:{line}: {path}"));
        }
    }
    // The half without which the assertion below is a claim about nothing: a scan that reads no
    // file reports no finding and exits zero, so a clean repository and a broken scan are the same
    // observation. `layout.rs` states the same hazard in the same spelling.
    assert!(
        examined > 0,
        "the scan examined no file at all, so it has stopped looking at the right ones"
    );
    // A skipped file must be one the tree genuinely lacks. This can fail — unlike counting the
    // iterations of a loop nothing leaves early, which is what stood here and restated the loop's
    // own postcondition — and it is what stops the skip above from swallowing a real read failure.
    for file in &absent {
        assert!(
            !root.join(file).exists(),
            "the scan skipped `{file}` as absent from the working tree, and it is on disk"
        );
    }
    assert!(
        leaked.is_empty(),
        "a tracked file names an absolute path inside a user's home directory, which is this \
         workstation's layout and not the repository's ({} findings over {examined} of {} \
         selected files examined, {} listed in the index but absent from the working tree{}); \
         record a repository-relative path, a managed tree id, or the portable `~/` spelling:\n{}",
        leaked.len(),
        selected.len(),
        absent.len(),
        if absent.is_empty() {
            String::new()
        } else {
            format!(": {}", absent.join(", "))
        },
        leaked.join("\n")
    );
}

/// The scan reads every tree the acceptance statement names, and holds itself to its own rule.
///
/// Reading fewer files reports fewer findings and still exits zero, which is how the sibling scan
/// failed. The second half used to assert that the scan excluded *this* file, which restated the
/// filter's own postcondition in the filter's own words and so could not fail for any state of the
/// repository — while exempting one tracked file under `crates/` from the rule for good. This file
/// is a tracked file under `crates/` like any other and is scanned like any other; its controls
/// are joined at run time precisely so that it can be.
#[test]
fn the_scan_reads_each_named_tree_and_holds_this_file_to_the_same_rule() {
    let root = workspace_root();
    let files = scanned_files(&root);
    for prefix in SCANNED_PREFIXES {
        assert!(
            files.iter().any(|file| file.starts_with(prefix)),
            "`{prefix}` is named by the acceptance statement and the scan read none of it"
        );
    }
    assert!(
        files.iter().any(|file| file == this_file()),
        "the lane is a tracked file under `crates/` and must be scanned like the rest; \
         `file!()` names `{}`, which the selection did not produce",
        this_file()
    );
    let own = home_paths_in(&root.join(this_file()));
    assert!(
        own.is_empty(),
        "the lane's own bytes must satisfy the rule the lane enforces; they name: {own:?}"
    );
}

/// The tail each marker in [`HOME_MARKERS`] is joined to for a control, in that order.
///
/// Each is what follows the marker and carries none of its own: an account name and a path for the
/// two markers that take one, and for `/root/` — which is the superuser's home and has no account
/// segment — just the path. Held apart from the marker on purpose; see [`synthetic_home_path`].
const CONTROL_TAILS: &[&str] = &[
    "someone/.local/state/trees/wt-0/examples/billing/system.yaml",
    "someone/src/ess/target/report.json",
    ".cache/ess/report.json",
];

/// A control path, joined from the constant under test at run time rather than written out.
///
/// Spelled as a literal, each of these *is* an absolute home-directory path in a tracked file —
/// the exact thing this lane refuses — and the organization's `personal-paths` scanner refused the
/// commit that first introduced them. It was right to: a scanner cannot tell a control from a leak
/// by looking, and neither can a reader. Joining at run time keeps the repository's bytes honest
/// and lets this file be scanned by its own lane rather than exempted from it. `concat!` would not
/// do — it is still a literal in the source.
///
/// It buys nothing else, and the round that introduced it cost something: with the last
/// independent literal gone, every remaining expectation was derived from [`HOME_MARKERS`], and a
/// constant no case can disagree with is a constant no case tests. The two cases that do not
/// derive from it are named in [`the_detector_finds_each_home_directory_spelling_and_no_portable_one`].
fn synthetic_home_path(marker: &str, tail: &str) -> String {
    format!("{marker}{tail}")
}

/// The home root each runner family gives a job, keyed by the prefix its label starts with.
///
/// The point of this table is that it is *not* [`HOME_MARKERS`]. A case whose expectation is
/// derived from the constant under test cannot disagree with it, and every case in this file once
/// was: replacing `HOME_MARKERS` with `["/nope-a/", "/nope-b/", "/nope-c/"]` left the whole lane
/// green, repository scan included. What a marker set has to be right *about* is the hosts this
/// repository actually builds on, so that is what is written down here and read from
/// `.github/workflows/ci.yml` there.
/// No `windows` row. A Windows runner's home is `C:\Users\<account>`, and the row that used to
/// claim `/Users/` for it made the case below pass — the string is in [`HOME_MARKERS`] — while
/// `C:\Users\…` matches nothing the detector looks for. A table that reports a platform as covered
/// when it is not is worse than a table that does not mention it, because the case exists to fail
/// in exactly that situation. No workflow here runs Windows; when one does, the detector needs a
/// native spelling before this table gets a row.
const RUNNER_HOME_ROOTS: &[(&str, &str)] = &[("ubuntu", "/home/"), ("macos", "/Users/")];

/// A YAML value with any trailing comment removed.
///
/// ` #` after a value opens a comment; a line that is nothing but a comment leaves the value empty.
/// The parse below had no notion of one, so a `runs-on:` line carrying an ordinary trailing comment
/// — a shape `.github/workflows/ci.yml` already uses on other value lines — read as a runner whose
/// name included the comment, and the case that consumes these labels panicked naming a platform
/// that does not exist. A gate that refuses a clean workflow is the failure mode this whole file is
/// most careful about.
fn without_comment(value: &str) -> &str {
    match value.find(" #") {
        Some(offset) => value[..offset].trim(),
        None => value.trim(),
    }
}

/// The items one YAML scalar or inline sequence names, unquoted.
///
/// `[a, b]` is a sequence and anything else is one item. Nothing here is a YAML parser: what a
/// workflow writes for a runner is a label, a short inline list of them, a block sequence, or a
/// matrix expression, and each of those has a caller below that knows it by shape.
fn list_items(value: &str) -> Vec<String> {
    let unquote = |item: &str| item.trim().trim_matches(['"', '\'']).to_owned();
    match value
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        Some(sequence) => sequence
            .split(',')
            .map(unquote)
            .filter(|item| !item.is_empty())
            .collect(),
        None if value.is_empty() => Vec::new(),
        None => vec![unquote(value)],
    }
}

/// Whether `label` names a runner rather than a piece of the document it was read out of.
///
/// Load-bearing, and *inside* the parse rather than beside it. Whatever leaves [`runner_labels`]
/// is handed to [`RUNNER_HOME_ROOTS`] as the name of a platform CI runs on, and a value no row
/// matches is not checked against anything — it is panicked on by name, which is a red gate on a
/// workflow that is not wrong. The same rule written as an assertion in a test case would be
/// applied to that case's own table and to no label read out of a file, so it could not fail for
/// any workflow this repository holds; that is what it was, and it is why it is here now.
///
/// What it refuses is YAML that leaked into a value: an unresolved `${{ … }}` wherever it is
/// written — a `runs-on:` value, a matrix entry, a sequence item — a comment, a bracket, a comma,
/// a quote, a colon out of a mapping, and anything carrying whitespace. A runner label carries
/// none of those.
fn names_a_runner(label: &str) -> bool {
    !label.is_empty()
        && !label.contains(char::is_whitespace)
        && !label.contains(['$', '{', '}', '#', '[', ']', ',', '"', '\'', ':'])
}

/// The indentation of `line`, in bytes, which is what YAML nests with.
fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// The items of the block sequence written under `lines[from]`, if there is one.
///
/// The second of the two spellings GitHub Actions gives a list, and the ordinary one for a runner
/// named by several labels — `- self-hosted` and then the platform, which is exactly how a
/// platform with no row in [`RUNNER_HOME_ROOTS`] arrives. Read it or the job contributes no label
/// at all, which is this gate narrowing itself in silence on the one platform it has never heard
/// of.
///
/// All three ways YAML allows one to be written, because reading one of them is the same silence
/// wearing a smaller coat: an entry may sit at its key's **own** indentation as well as deeper,
/// and a comment or a blank line between two entries interrupts neither. This repository's own
/// workflow carries a comment above a value line in five places.
fn block_sequence(lines: &[&str], from: usize) -> Vec<String> {
    let key = indentation(lines[from]);
    let mut items = Vec::new();
    for line in &lines[from + 1..] {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if indentation(line) < key {
            break;
        }
        let Some(element) = trimmed.strip_prefix("- ") else {
            break;
        };
        items.extend(list_items(without_comment(element)));
    }
    items
}

/// The value `key` takes in the mapping written under `lines[from]`, if it takes one.
///
/// `runs-on:` accepts a mapping of `group:` and `labels:` as well as a scalar and a sequence; it
/// is how a workflow addresses a self-hosted runner group, and the platform is the `labels:` half.
/// A group name is not a platform and is deliberately not read as one: inventing a label out of it
/// hands [`RUNNER_HOME_ROOTS`] something to panic about, which is the failure this parse is most
/// careful to avoid. A mapping naming only a group is stated in the module documentation as a
/// `runs-on:` whose platform this gate cannot check.
fn mapping_value(lines: &[&str], from: usize, key: &str) -> Vec<String> {
    let outer = indentation(lines[from]);
    for (number, line) in lines.iter().enumerate().skip(from + 1) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if indentation(line) <= outer {
            break;
        }
        let Some(rest) = trimmed
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
        else {
            continue;
        };
        let value = without_comment(rest);
        return if value.is_empty() {
            block_sequence(lines, number)
        } else {
            list_items(value)
        };
    }
    Vec::new()
}

/// The lines of the block `lines[number]` belongs to: its parent key's own indented body.
///
/// A matrix belongs to the job that defines it. Looking a key up across the whole file reads
/// *another* job's matrix, and `os` is both GitHub's own name for a runner matrix and an ordinary
/// name for a matrix of container images or target platforms — so every value of every `os:` in
/// the file became a runner label for this job. That over-collection was documented here as the
/// safe direction, on the grounds that "an extra label is a platform the lane checks a home root
/// for". It is not checked: it is looked up in [`RUNNER_HOME_ROOTS`] and panicked on when no row
/// matches, so over-collecting is the direction that turns the gate red on a clean workflow.
fn enclosing_block(lines: &[&str], number: usize) -> (usize, usize) {
    let indent = indentation(lines[number]);
    let mut start = 0;
    let mut parent = 0;
    for (earlier, line) in lines[..number].iter().enumerate() {
        if !line.trim().is_empty() && indentation(line) < indent {
            parent = indentation(line);
            start = earlier + 1;
        }
    }
    let end = lines
        .iter()
        .enumerate()
        .skip(number + 1)
        .find(|(_, line)| !line.trim().is_empty() && indentation(line) <= parent)
        .map_or(lines.len(), |(later, _)| later);
    (start, end)
}

/// Every value `key` takes in a matrix within `lines`.
///
/// Three spellings, because a workflow may use any of them: `key: [a, b]`, a block sequence under
/// `key:`, and an `include:` entry, which writes the key on the same line as its own sequence
/// dash. `lines` is the job's own block — see [`enclosing_block`] — and not the whole file.
fn matrix_values(lines: &[&str], key: &str) -> Vec<String> {
    let mut values = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        // An `include:` entry spells its keys on the same line as the sequence dash.
        let named = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some(rest) = named
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
        else {
            continue;
        };
        let value = without_comment(rest);
        if value.is_empty() {
            values.extend(block_sequence(lines, number));
        } else {
            values.extend(list_items(value));
        }
    }
    values
}

/// Every `runs-on:` line in `workflow`, as its line number and the labels that line names.
///
/// Per line, because that is the only thing a reader can act on. The sentence this gate holds a
/// workflow to is "every `runs-on:` line contributes at least one label", and a *count* of
/// distinct labels is neither half of it: two jobs sharing a runner is two lines and one label, so
/// a count refuses a clean workflow, and a line resolving to nothing is hidden by a line naming
/// three, so a count accepts a job whose platform is invisible. Both directions are the failure
/// this function exists to remove.
///
/// Every `runs-on:` value, rather than the families [`RUNNER_HOME_ROOTS`] already names: searching
/// for the known families is what made [`ci_runner_labels`]'s own doc false, because a platform
/// with no row produced no label at all, the panic on an unrecorded label was unreachable, and the
/// case stayed green on the runners that were already known.
///
/// A runner named through a matrix expression is resolved to the values that matrix holds, within
/// the job that names it. **A value this parse cannot resolve contributes no label**, wherever it
/// is written — the `runs-on:` value, a matrix entry, a sequence item — because the alternative is
/// to hand the case that consumes these labels a string that is not a runner, which panics naming
/// a platform nobody runs. [`names_a_runner`] is that rule, and every value passes through it.
fn runners_per_line(workflow: &str) -> Vec<(usize, BTreeSet<String>)> {
    let lines: Vec<&str> = workflow.lines().collect();
    let mut runners = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        let Some(value) = line.trim_start().strip_prefix("runs-on:") else {
            continue;
        };
        let value = without_comment(value);
        let named = if let Some(reference) = value
            .strip_prefix("${{")
            .and_then(|rest| rest.strip_suffix("}}"))
        {
            let (from, until) = enclosing_block(&lines, number);
            reference
                .trim()
                .strip_prefix("matrix.")
                .map(|key| matrix_values(&lines[from..until], key.trim()))
                .unwrap_or_default()
        } else if value.is_empty() {
            // A block sequence, or the `group:`/`labels:` mapping, whose platform is `labels:`.
            let sequence = block_sequence(&lines, number);
            if sequence.is_empty() {
                mapping_value(&lines, number, "labels")
            } else {
                sequence
            }
        } else {
            list_items(value)
        };
        runners.push((
            number + 1,
            named
                .into_iter()
                .filter(|label| names_a_runner(label))
                .collect(),
        ));
    }
    runners
}

/// Every runner label a workflow's text names.
fn runner_labels(workflow: &str) -> BTreeSet<String> {
    runners_per_line(workflow)
        .into_iter()
        .flat_map(|(_, labels)| labels)
        .collect()
}

/// Every runner label named in `.github/workflows/ci.yml`.
///
/// Read from the workflow rather than listed here, so that adding a platform to CI without adding
/// its home root to [`HOME_MARKERS`] fails this lane instead of quietly narrowing it. The reading
/// is [`runner_labels`], which takes the text: a parse that needs no directory is a parse the cases
/// below can drive without the gate writing a fixture outside the repository.
fn ci_runner_labels(root: &Path) -> BTreeSet<String> {
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("the CI workflow is readable");
    runner_labels(&workflow)
}

/// The markers cover the home root of every platform this repository actually builds on.
///
/// Independent of [`HOME_MARKERS`] in both directions: the platforms come from the CI workflow and
/// the roots from [`RUNNER_HOME_ROOTS`], so a marker set that does not name a real host's home root
/// fails here however self-consistent the rest of the file is.
#[test]
fn the_markers_cover_the_home_root_of_every_platform_ci_runs_on() {
    let root = workspace_root();
    let labels = ci_runner_labels(&root);
    assert!(
        !labels.is_empty(),
        "`.github/workflows/ci.yml` named no runner, so this case measures nothing"
    );
    for label in &labels {
        let (_, home_root) = RUNNER_HOME_ROOTS
            .iter()
            .find(|(family, _)| label.starts_with(family))
            .unwrap_or_else(|| panic!("`{label}` runs CI and no home root is recorded for it"));
        assert!(
            HOME_MARKERS.contains(home_root),
            "CI runs on `{label}`, whose home directories live under `{home_root}`, and the \
             markers this lane refuses are {HOME_MARKERS:?} — a home path on that host is not \
             refused by this gate"
        );
    }
}

/// The home directory `/etc/passwd` records for an account, if it records one.
fn recorded_home(account: &str) -> Option<String> {
    let passwd = fs::read_to_string("/etc/passwd").ok()?;
    passwd.lines().find_map(|line| {
        // `name:password:uid:gid:gecos:directory:shell`; the directory is the fifth field after
        // the name this consumes.
        let mut fields = line.split(':');
        if fields.next()? != account {
            return None;
        }
        fields.nth(4).map(str::to_owned)
    })
}

/// The markers cover the superuser's home directory, as this host records it.
///
/// The control that reaches `/root/`, which nothing else here does. `.github/workflows/ci.yml`
/// names no runner whose home root is the superuser's, so the CI table cannot see that marker, and
/// `HOME` is not the superuser's when a developer runs the gate — with neither, dropping `/root/`
/// from [`HOME_MARKERS`] left every case in this file green and byte-identical to the unmutated
/// copy. `/etc/passwd` is outside this repository and outside the constant, which is what makes it
/// worth reading.
///
/// Skipped rather than failed where `/etc/passwd` does not exist or records no absolute home for
/// the superuser: that is a fact about the host, and a gate's verdict belongs to the repository.
#[test]
fn the_markers_cover_the_superuser_home_this_host_records() {
    let Some(home) = recorded_home("root").filter(|home| home.starts_with('/')) else {
        eprintln!(
            "skipped: this host records no absolute superuser home in /etc/passwd, so the \
             `/root/` marker has no independent control here"
        );
        return;
    };
    let tail = "/.cache/ess/report.json";
    let found = home_paths(&format!("the container wrote {home}{tail} before exiting"));
    assert!(
        found.iter().any(|path| path.ends_with(tail)),
        "this host records the superuser's home directory as `{home}`, so `{home}{tail}` is a \
         path inside a user's home directory on it; the markers {HOME_MARKERS:?} do not describe \
         it and the detector collected {found:?}"
    );
}

/// The detector finds the home directory this process is running under, when it has a usual one.
///
/// **Skipped, never failed, when `HOME` is absent or lies outside the marker roots.** A gate's
/// verdict has to be a function of the repository, and this case as first written was not: under
/// `HOME=/homeless-shelter`, `/`, `/github/home` or `/var/root`, or with `HOME` unset, it refused a
/// repository with nothing wrong in it and blamed the marker set. Two of the four login accounts on
/// the machine this was found on — `git` at `/` and `postgres` at `/var/lib/postgres` — would have
/// turned the gate red on a clean tree. An account whose home is somewhere unusual is a fact about
/// the host, and the markers are not wrong for not naming it.
///
/// What remains is a corroboration, not a load-bearing control: it fires on the common host and
/// stays quiet elsewhere. The controls that cannot be skipped are
/// [`the_markers_cover_the_home_root_of_every_platform_ci_runs_on`] and
/// [`the_markers_cover_the_superuser_home_this_host_records`].
#[test]
fn the_detector_finds_the_home_directory_this_process_runs_under() {
    let Some(home) = std::env::var_os("HOME")
        .and_then(|home| home.into_string().ok())
        .map(|home| home.trim_end_matches('/').to_owned())
        .filter(|home| {
            HOME_MARKERS
                .iter()
                .any(|marker| home.starts_with(marker) && home.len() > marker.len())
        })
    else {
        eprintln!(
            "skipped: HOME is unset or outside {HOME_MARKERS:?}, which is a fact about this host \
             and not about this repository"
        );
        return;
    };
    let expected = format!("{home}/.cache/ess/report.json");
    let found = home_paths(&format!("the run wrote {expected} before exiting"));
    assert!(
        found.contains(&expected),
        "this process's own home directory is `{home}`, which lies under one of {HOME_MARKERS:?}, \
         so `{expected}` is a path under a user's home directory on this very host; the detector \
         collected {found:?}"
    );
}

/// A marker that ends a sentence names a directory, not a path inside anybody's home.
///
/// The module documentation has always claimed this; until the adversary asked, the detector did
/// not do it. `.` is a byte [`continues_a_path`] accepts, so the stop satisfied "something follows
/// the marker", and the trailing trim then removed the stop and the marker's own slash — a finding
/// whose entire text was `/home`. A gate that refuses a clean file is not a smaller defect than
/// one that passes a dirty file; it is the one that gets the gate switched off.
#[test]
fn a_marker_that_ends_a_sentence_is_not_a_home_directory_path() {
    for marker in HOME_MARKERS {
        for text in [
            format!("on this host every account lives under {marker}."),
            format!("on this host every account lives under {marker}"),
            format!("the path {marker}/ is the directory itself"),
        ] {
            let found = home_paths(&text);
            assert!(
                found.is_empty(),
                "`{text}` names a directory and no user's home path; the detector collected \
                 {found:?}"
            );
        }
    }
}

/// A home-directory path is still one when a serializer has escaped or encoded the separator.
///
/// The acceptance statement refuses a tracked file that *contains* a path under a user's home
/// directory and says nothing about the spelling. Each spelling below is one this repository's own
/// toolchain emits — eight tracked files under `crates/` already carry `\/`.
#[test]
fn an_escaped_or_encoded_home_directory_path_is_still_collected() {
    let plain = synthetic_home_path(HOME_MARKERS[0], CONTROL_TAILS[0]);
    for spelling in SEPARATOR_SPELLINGS {
        let encoded = plain.replace('/', spelling);
        let found = home_paths(&encoded);
        assert!(
            found.contains(&plain),
            "`{plain}` written with `{spelling}` for its separator is the same path under a \
             user's home directory; the detector collected {found:?}"
        );
    }
}

/// The detector fires on every spelling it refuses, and on no portable one.
///
/// This is the other way a scan is worthless: reading every file while looking for a shape nothing
/// has. Each marker gets a control here rather than relying on the repository to contain one,
/// because the repository is supposed to contain none — the day the fix lands, a detector that
/// matches nothing and a repository that carries nothing are indistinguishable.
///
/// On its own this case proves less than it looks: its expectation is built from the constant
/// under test, so it agrees with any marker set whatsoever. It is kept for the shape it pins —
/// one path in prose is one finding — and the independent halves live in
/// [`the_markers_cover_the_home_root_of_every_platform_ci_runs_on`] and
/// [`the_detector_finds_the_home_directory_this_process_runs_under`].
#[test]
fn the_detector_finds_each_home_directory_spelling_and_no_portable_one() {
    assert_eq!(
        HOME_MARKERS.len(),
        CONTROL_TAILS.len(),
        "every marker this lane refuses needs a control; a marker added without one is refused \
         by a detector no case exercises"
    );
    for (marker, tail) in HOME_MARKERS.iter().zip(CONTROL_TAILS) {
        let expected = synthetic_home_path(marker, tail);
        let text = format!("the run recorded {expected} before exiting");
        let found = home_paths(&text);
        assert!(
            found.contains(&expected),
            "`{expected}` is an absolute home-directory path and must be collected; \
             the detector collected {found:?}"
        );
        assert_eq!(
            found.len(),
            1,
            "one path in the prose is one finding; the detector collected {found:?}"
        );
    }
    // A trailing sentence stop is punctuation, not path, and must not split one finding into two
    // spellings of the same path.
    let stopped = synthetic_home_path(HOME_MARKERS[0], "someone/.cache/ess-review/wave");
    assert!(
        home_paths(&format!("it was written under {stopped}.")).contains(&stopped),
        "a trailing sentence stop must be dropped from the collected path"
    );
    // A marker naming its own directory in prose places nothing inside anybody's home and leaks
    // nothing. Asked of every marker rather than one, so a marker added later cannot arrive
    // matching bare mentions of itself.
    let bare: Vec<String> = HOME_MARKERS
        .iter()
        .map(|marker| format!("the directory {marker} is where such accounts live on a host"))
        .collect();
    for portable in bare.iter().map(String::as_str).chain([
        "the tree is at ~/.local/state/worktree/trees/b10x/ess/wt-0 on the workstation",
        "the fixture records examples/billing/system.yaml, repository-relative",
        "under crates/edge/ess-cli/tests/fixtures/coverage-producers, and nowhere else",
    ]) {
        let found = home_paths(portable);
        assert!(
            found.is_empty(),
            "`{portable}` names no user's home directory and must not be collected; \
             the detector collected {found:?}"
        );
    }
}

/// A marker preceded by a path component is a relative reference, not an absolute path.
///
/// The detector looks for its markers anywhere in a line and, until this case asked, never asked
/// what came *before* one — so any relative path or URL with a directory component of that name
/// satisfied it. `website/` is a Docusaurus tree where a route so named is an ordinary page, and a
/// JSON Pointer into a document's root is an ordinary pointer. What the lane then printed was an
/// absolute path with the relative prefix cut away: a file that exists on no host, reported as this
/// workstation's layout. It is the same defect as the sentence-ending marker above — a match
/// accepted without looking at the other side of it — and it is the one that gets a gate switched
/// off, because a reader who goes to look finds nothing there.
///
/// The second half is what a narrowing like this gets wrong. A leading run of separators leaves
/// what follows absolute, which is how a `file://` URL spells a local path, and a serialized line
/// break ends in a letter, which is a byte a directory name is made of. Both are absolute paths
/// preceded by a byte the new rule would otherwise refuse, and both must still be collected.
#[test]
fn a_marker_preceded_by_a_path_component_is_not_an_absolute_home_path() {
    for marker in HOME_MARKERS {
        let component = marker.trim_start_matches('/');
        for text in [
            format!("the page is website/src/pages/{component}index.md"),
            format!("see https://example.com/documentation/{component}getting-started"),
            format!("the pointer /document{marker}children/0 addresses it"),
            format!("the portable spelling is ~{marker}projects/ess/report.json"),
            format!("relative to the tree, {component}someone/report.json"),
        ] {
            let found = home_paths(&text);
            assert!(
                found.is_empty(),
                "`{text}` names no absolute path under a user's home directory — the marker is a \
                 component of a relative reference — and the detector collected {found:?}, which \
                 is a path that exists on no host"
            );
        }
        let leaked = synthetic_home_path(marker, "someone/.cache/ess/report.json");
        for text in [
            format!("the run wrote {leaked} before exiting"),
            format!("it is reachable as file://{leaked}"),
            format!("the record reads \"the run exited\\n{leaked}\""),
            format!("-{leaked}"),
            format!("+{leaked}"),
            format!("    +{leaked}"),
        ] {
            let found = home_paths(&text);
            assert!(
                found.contains(&leaked),
                "`{text}` carries `{leaked}`, an absolute path under a user's home directory, \
                 however the bytes before it are written; the detector collected {found:?}"
            );
        }
    }
}

/// An account name outside ASCII is an account name, and the path under it is still a leak.
///
/// [`continues_a_path`] accepted ASCII alone, so for an account whose name starts outside it
/// nothing followed the marker as far as the detector could tell: the candidate trimmed back to the
/// directory the marker names and the whole path left the scan. Only the *first* character decided
/// it — a name whose second character is non-ASCII was collected truncated, which is a finding
/// naming a path nobody can go and look at. Neither platform CI runs on requires an ASCII account
/// name, and the acceptance statement refuses a path under *a user's* home directory without
/// qualifying whose.
///
/// The second half is the property [`home_paths_in`] depends on to read a binary file: the
/// replacement character [`String::from_utf8_lossy`] leaves behind is not part of a name, so an
/// undecodable run still *ends* a path rather than joining two into one finding that is neither.
#[test]
fn a_home_directory_whose_account_name_is_not_ascii_is_still_collected() {
    for marker in HOME_MARKERS {
        for account in ["ärni", "jösé", "北京"] {
            let leaked = synthetic_home_path(marker, &format!("{account}/.cache/ess/report.json"));
            let found = home_paths(&format!("the run wrote {leaked} before exiting"));
            assert!(
                found.contains(&leaked),
                "`{leaked}` is an absolute path under a user's home directory whose account name \
                 is not ASCII; the detector collected {found:?}"
            );
        }
        let first = synthetic_home_path(marker, "someone/.cache/ess/report.json");
        let second = synthetic_home_path(marker, "another/.cache/ess/report.json");
        let found = home_paths(&format!("{first}\u{FFFD}{second}"));
        assert_eq!(
            found,
            BTreeSet::from([first.clone(), second.clone()]),
            "an undecodable run between `{first}` and `{second}` ends a path rather than joining \
             two; the detector collected {found:?}"
        );
    }
}

/// Each spelling a `runs-on:` could be written in here tomorrow, and the labels it names.
///
/// A table rather than a case body, because the body that held it outgrew the line limit the
/// moment the shapes the second adversary pass named were added to it — and the answer to that is
/// never to drop shapes. Every one of these was read by a YAML parser before it was written down.
const RUNNER_SPELLINGS: &[(&str, &str, &[&str])] = &[
        (
            "a platform with no row in the home-root table",
            "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  native:\n    runs-on: freebsd-14\n",
            &["ubuntu-latest", "freebsd-14"],
        ),
        (
            "a matrix expression",
            "jobs:\n  matrixed:\n    runs-on: ${{ matrix.runner }}\n    strategy:\n      \
             matrix:\n        runner: [openbsd-7, netbsd-10]\n",
            &["openbsd-7", "netbsd-10"],
        ),
        (
            "a matrix expression carrying a trailing comment",
            "jobs:\n  matrixed:\n    runs-on: ${{ matrix.runner }} # both architectures\n    \
             strategy:\n      matrix:\n        runner: [openbsd-7] # the one that ships\n",
            &["openbsd-7"],
        ),
        (
            "a matrix written as `include:` entries",
            "jobs:\n  matrixed:\n    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n        include:\n          - os: openbsd-7\n            toolchain: stable\n",
            &["openbsd-7"],
        ),
        (
            "a block sequence, which is how a self-hosted runner is named",
            "jobs:\n  native:\n    runs-on:\n      - self-hosted\n      - freebsd-14\n",
            &["self-hosted", "freebsd-14"],
        ),
        (
            "an inline sequence",
            "jobs:\n  native:\n    runs-on: [self-hosted, freebsd-14]\n",
            &["self-hosted", "freebsd-14"],
        ),
        (
            "a quoted scalar",
            "jobs:\n  native:\n    runs-on: \"freebsd-14\"\n",
            &["freebsd-14"],
        ),
        (
            "a block sequence whose items sit at the key's own indentation",
            "jobs:\n  native:\n    runs-on:\n    - self-hosted\n    - freebsd-14\n",
            &["self-hosted", "freebsd-14"],
        ),
        (
            "a block sequence interrupted by a comment and a blank line",
            "jobs:\n  native:\n    runs-on:\n      - self-hosted\n      # the ARM fleet\n\n      - freebsd-14\n",
            &["self-hosted", "freebsd-14"],
        ),
        (
            "a runner group mapping, which names its platform under `labels:`",
            "jobs:\n  fleet:\n    runs-on:\n      group: ubuntu-runners\n      labels: ubuntu-22.04-16core\n",
            &["ubuntu-22.04-16core"],
        ),
        (
            "a matrix entry that is itself an expression",
            "jobs:\n  matrixed:\n    runs-on: ${{ matrix.runner }}\n    strategy:\n      matrix:\n        runner: [openbsd-7, \"${{ vars.FLEET }}\"]\n",
            &["openbsd-7"],
        ),
        (
            "a matrix key another job of the same file also defines",
            "jobs:\n  gate:\n    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n        os: [openbsd-7]\n  images:\n    runs-on: ubuntu-latest\n    strategy:\n      matrix:\n        os: [alpine, debian]\n    container: ${{ matrix.os }}\n",
            &["openbsd-7", "ubuntu-latest"],
        ),
        (
            "an expression naming a matrix key this workflow never defines",
            "jobs:\n  matrixed:\n    runs-on: ${{ matrix.absent }}\n",
            &[],
        ),
        (
            "an expression that is not a matrix reference at all",
            "jobs:\n  matrixed:\n    runs-on: ${{ inputs.runner }}\n",
            &[],
        ),
];

/// A workflow, and the `runs-on:` lines in it this parse can attribute no label to.
///
/// The two shapes a *count* of labels cannot tell apart: two jobs on one runner is two lines and
/// one label, and a line resolving to nothing hides behind a line naming three.
const BLIND_RUNNER_LINES: &[(&str, &str, &[usize])] = &[
    (
        "two jobs on one runner, every line resolved",
        "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  docs:\n    runs-on: ubuntu-latest\n",
        &[],
    ),
    (
        "one line the parse cannot resolve, beside one naming three labels",
        "jobs:\n  called:\n    runs-on: ${{ inputs.runner }}\n  fleet:\n    runs-on: [self-hosted, linux, x64]\n",
        &[3],
    ),
];

/// Every runner the workflow names is read, including a platform this file has never heard of.
///
/// [`ci_runner_labels`] is documented as reading the workflow "so that adding a platform to CI
/// without adding its home root to [`HOME_MARKERS`] fails this lane instead of quietly narrowing
/// it", and it could not do that: it searched for the families [`RUNNER_HOME_ROOTS`] already names,
/// so a platform with no row produced no label, the `unwrap_or_else` that panics on an unrecorded
/// one was unreachable, and the case stayed green on the strength of the runners that were already
/// known. The one shape that failed — a workflow naming no known family at all — is the one shape
/// adding a platform never produces.
///
/// Driven against [`runner_labels`] rather than [`ci_runner_labels`] so that the gate writes
/// nothing outside the repository. The helper this case used to have created a directory under
/// `TMPDIR` on every run of the gate, printed its path on a green one and left it behind on a red
/// one; the answer to a test fixture the gate has to clean up is a function that needs no fixture.
///
/// Every spelling below is one a workflow in this repository could be written in tomorrow, and the
/// second half is why that matters: three of them arrive at a *clean* workflow, and a parse that
/// answers one of those with the expression it could not read hands
/// [`the_markers_cover_the_home_root_of_every_platform_ci_runs_on`] a platform that does not exist
/// to panic about. A gate that refuses a clean repository is the one that gets switched off.
#[test]
fn the_workflow_parse_reads_every_runner_the_workflow_names() {
    for (spelling, workflow, expected) in RUNNER_SPELLINGS {
        let labels = runner_labels(workflow);
        assert_eq!(
            labels,
            expected.iter().map(|label| (*label).to_owned()).collect(),
            "`{spelling}` is a runner this parse has to read: a platform it cannot see narrows \
             this gate in silence, and a label it invents for a value it could not resolve turns \
             the gate red on a workflow that is not wrong. It read {labels:?}"
        );
        // The class, rather than the instances above: whatever this parse emits is handed to
        // `RUNNER_HOME_ROOTS` as the name of a platform CI runs on, so it has to *be* a runner
        // label and not a fragment of the YAML it was read out of. [`names_a_runner`] is that
        // rule, and it lives inside the parse — a guard written here would be an assertion over
        // this table alone and would never see a label read out of a file, which is precisely the
        // shape of check that cannot fail for any workflow this repository holds. What is left
        // here is the observation that the parse *drops* such a value rather than the claim that
        // this table happens not to contain one.
        for label in &labels {
            assert!(
                names_a_runner(label),
                "reading `{spelling}` produced `{label}`, which is a piece of the workflow's text \
                 rather than a runner label; the case that consumes these panics naming it as a \
                 platform CI runs on"
            );
        }
    }

    // The two shapes the repository control has to tell apart, which a count cannot. Two jobs on
    // one runner is one label and two lines; a line the parse cannot read is hidden by a line that
    // names three. Both are measured per line, which is the sentence the control is written to.
    for (shape, workflow, blind_lines) in BLIND_RUNNER_LINES {
        let blind: Vec<usize> = runners_per_line(workflow)
            .into_iter()
            .filter(|(_, labels)| labels.is_empty())
            .map(|(line, _)| line)
            .collect();
        assert_eq!(
            blind, *blind_lines,
            "`{shape}`: a `runs-on:` line contributing no label is a job whose platform this gate \
             cannot check, and it is the line that has to be named — not a total compared against \
             a total"
        );
    }

    // The repository's own workflow, which is the only one whose verdict this gate carries. Every
    // `runs-on:` line in it has to contribute at least one label: an expression this parse cannot
    // resolve contributes nothing, deliberately, so this is what says so instead of a panic naming
    // a platform nobody runs.
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("the CI workflow is readable");
    let per_line = runners_per_line(&workflow);
    assert!(
        !per_line.is_empty(),
        "`.github/workflows/ci.yml` names no runner, so this half measures nothing"
    );
    let blind: Vec<usize> = per_line
        .iter()
        .filter(|(_, labels)| labels.is_empty())
        .map(|(line, _)| *line)
        .collect();
    assert!(
        blind.is_empty(),
        "these `runs-on:` lines of `.github/workflows/ci.yml` contribute no label, so this gate \
         checks nothing about the platform those jobs run on — either the spelling they use \
         belongs in `runner_labels` or their home root belongs in `RUNNER_HOME_ROOTS`: {blind:?}"
    );
}

/// The two documentation lines that open and close the unread-tree list, in that order.
///
/// A bound this lane states about *how* it looks — a home root it has no marker for, an account
/// name it truncates — is prose a reader checks by reading. A tree it never opens is different: it
/// is a claim about the repository that stops being true when the repository changes, and prose
/// about it goes stale silently. So the list is delimited here and
/// [`every_unread_tree_that_carries_the_class_is_named_in_the_module_documentation`] holds every
/// claim in it to what `git` and the detector actually find.
///
/// **Two anchors rather than one, because the end of the list is where a parse loses a bullet.**
/// A section that runs until "the first line that is not a bullet" has to decide what a wrapped
/// continuation looks like, and nothing holds a continuation to any particular indent: `rustfmt`
/// does not reflow documentation comments and `clippy` has no lint for it, so a tab is an ordinary
/// thing for the next editor to write. Ended at an explicit line instead, every bullet between the
/// anchors is read whatever its continuations are indented with, and a missing closing anchor is a
/// panic rather than a short list.
const UNREAD_TREE_SECTION: &[&str] = &[
    "What it does not read at all, tree by tree:",
    "That is the whole of that list.",
];

/// Every claim the unread-tree list makes, as `(tree, unread, files, lines)` per bullet.
///
/// Each bullet is `` * `tree` — unread, N files, M lines: `` and then its reasons, so what the
/// lane reads is not only *which* tree but **what is being said about it**: whether the tree is
/// claimed unread or read, and the two counts the claim rests on. The first version of this parse
/// took the bullet's first backtick token and stopped, which meant a bullet asserting a tree was
/// read in full and clean parsed to exactly the same answer as one asserting it was unread and
/// carrying sixty files — and it meant every number in the bullet was unread prose. One of those
/// numbers was wrong.
///
/// Taking the source as text rather than reading a path, so the parse can be handed a document
/// that is not on disk. A parse that can only ever be given one input cannot be asked what it
/// accepts.
fn documented_unread_trees(source: &str) -> Vec<(String, bool, usize, usize)> {
    let mut anchors = UNREAD_TREE_SECTION.iter();
    // Held as the iterator's own item and compared through one dereference, rather than converted.
    // This text has to be valid and clippy-clean in two places at once: here, where
    // `UNREAD_TREE_SECTION` is a slice of `&str`, and in `host_paths_lane/mod.rs`, where the copy
    // of this function reads the same constant parsed out of this source into a slice of `String`.
    // The transcription warranty is a byte comparison, so one spelling has to serve both.
    let opening = anchors
        .next()
        .expect("the unread-tree section has an opening anchor");
    let closing = anchors
        .next()
        .expect("the unread-tree section has a closing anchor");
    let doc: Vec<String> = source
        .lines()
        .map_while(|line| line.strip_prefix("//!"))
        .map(str::to_owned)
        .collect();
    let open = doc
        .iter()
        .position(|line| line.trim() == *opening)
        .unwrap_or_else(|| {
            panic!(
                "the module documentation has no line reading `{opening}`, so it names no tree as \
                 unread and nothing can be compared against it"
            )
        });
    let Some(offset) = doc
        .iter()
        .skip(open + 1)
        .position(|line| line.trim() == *closing)
    else {
        panic!(
            "the unread-tree list is opened by `{opening}` and never closed by `{closing}`, so \
             where it ends is a guess and a bullet can fall outside it unnoticed"
        )
    };
    let close = open + 1 + offset;
    let mut claimed = Vec::new();
    for line in &doc[open + 1..close] {
        let Some(bullet) = line.trim().strip_prefix("* ") else {
            continue;
        };
        let mut quoted = bullet.split('`');
        quoted.next();
        let tree = quoted
            .next()
            .unwrap_or_else(|| panic!("the bullet `{bullet}` names no tree in backticks"))
            .to_owned();
        let claim = quoted
            .next()
            .unwrap_or_else(|| panic!("the bullet for `{tree}` states nothing after the tree name"))
            .trim()
            .strip_prefix('—')
            .unwrap_or_else(|| {
                panic!("the bullet for `{tree}` does not state its claim after an em dash")
            })
            .trim();
        // Up to the first colon, so a bullet's reasons can hold anything at all without the
        // counts in front of them becoming unparseable — and so the counts are a fixed, short
        // head a reader and this parse read the same way.
        let head = claim.split_once(':').map_or(claim, |(head, _)| head);
        let mut fields = head.splitn(3, ", ");
        let verdict = fields
            .next()
            .unwrap_or_else(|| panic!("the bullet for `{tree}` states no verdict"));
        let unread = match verdict {
            "unread" => true,
            "read" => false,
            other => panic!(
                "the bullet for `{tree}` opens with `{other}`, and the only two claims this parse \
                 can check are `unread` and `read` — a bullet whose verdict it cannot read is a \
                 decision nothing measures"
            ),
        };
        let count = |field: Option<&str>, unit: &str| -> usize {
            field
                .and_then(|text| text.strip_suffix(unit))
                .unwrap_or_else(|| {
                    panic!("the bullet for `{tree}` states no `<n>{unit}` count for its claim")
                })
                .trim()
                .parse()
                .unwrap_or_else(|_| {
                    panic!("the bullet for `{tree}` states a `{unit}` count that is not a number")
                })
        };
        let files = count(fields.next(), " files");
        let lines = count(fields.next(), " lines");
        claimed.push((tree, unread, files, lines));
    }
    assert!(
        !claimed.is_empty(),
        "the unread-tree list between `{opening}` and `{closing}` holds no bullet at all, so every \
         comparison against it is a comparison with nothing"
    );
    claimed
}

/// Every tracked file this repository holds, root-relative, whatever tree it is under.
///
/// [`scanned_files`] is the selection the acceptance statement names and is transcribed byte for
/// byte by `host_paths_lane/mod.rs`, so it is not widened to serve this; this asks `git` the same
/// question without the filter. `-z` for the same reason it is there: a path holding a non-ASCII
/// byte is quoted and escaped otherwise, and an unread tree is exactly where such a path would go
/// unnoticed.
fn tracked_files(root: &Path) -> Vec<String> {
    let listed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--cached", "-z"])
        .output()
        .expect("git lists the repository's files");
    assert!(listed.status.success(), "`git ls-files` failed");
    let mut files: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .split('\0')
        .filter(|file| !file.is_empty())
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    files
}

/// The tree `file` belongs to: its first path component with the separator, or the file itself.
///
/// A file at the repository root is its own tree, so a leak in one is reported under a name a
/// reader can find rather than being folded into a prefix that does not exist.
fn tree_of(file: &str) -> String {
    match file.split_once('/') {
        Some((first, _)) => format!("{first}/"),
        None => file.to_owned(),
    }
}

/// A tree the scan never opens, and which carries the class the scan refuses, is documented — and
/// what the documentation says about it is measured, down to its counts.
///
/// This is the half of the acceptance statement a reader cannot check by reading. The four trees
/// in [`SCANNED_PREFIXES`] are clean because this lane refuses to let them be otherwise, and the
/// summary line at the top of this file is true of those four and says so; a tree outside them
/// that carries the class needs a bullet in the module documentation, held here to the repository
/// rather than to its author.
///
/// Measured rather than listed, so the case closes the class instead of the instance: every tree
/// outside [`SCANNED_PREFIXES`] is read with the lane's own detector, and any tree found carrying
/// the class has to be named. A tree added later, or one that starts carrying it later, turns this
/// red by itself, and the bullet that answers it has to state counts that are true on the day it
/// is read.
///
/// Four directions, because the first version had one and a false sentence survived it:
///
/// * a carrying tree the list does not name — the leak the story is about;
/// * a tree the list names that no longer carries, or has been taken into the scan — documentation
///   outliving its reason, which is the same defect pointing the other way;
/// * a bullet claiming a tree is **read** when [`SCANNED_PREFIXES`] does not name it, which is the
///   edit that changes the decision and leaves the tree name where it was;
/// * the counts in each bullet, against the counts the detector finds.
#[test]
fn every_unread_tree_that_carries_the_class_is_named_in_the_module_documentation() {
    let root = workspace_root();
    let read = scanned_files(&root);
    let unread: Vec<String> = tracked_files(&root)
        .into_iter()
        .filter(|file| !read.contains(file))
        .collect();
    assert!(
        !unread.is_empty(),
        "every tracked file is under a scanned tree, so this case measures nothing; if that is \
         genuinely true the module documentation should name no unread tree at all"
    );

    let mut examined = 0usize;
    let mut carried: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut findings: Vec<String> = Vec::new();
    for file in &unread {
        let full = root.join(file);
        if !full.exists() {
            continue;
        }
        examined += 1;
        let found = home_paths_in(&full);
        if found.is_empty() {
            continue;
        }
        let lines: BTreeSet<usize> = found.iter().map(|(line, _)| *line).collect();
        let entry = carried.entry(tree_of(file)).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += lines.len();
        for (line, path) in found {
            findings.push(format!("{file}:{line}: {path}"));
        }
    }
    assert!(
        examined > 0,
        "the unread selection named {} files and none of them could be opened, so a tree that \
         carries the class and a tree that does not are the same observation here",
        unread.len()
    );

    let source = fs::read_to_string(root.join(this_file())).expect("this file is readable");
    let claims = documented_unread_trees(&source);
    // A bullet counting nothing states that its tree is unread and clean, which the count
    // assertion below measures; it is not a claim that the tree carries the class.
    let documented: BTreeSet<String> = claims
        .iter()
        .filter(|(_, unread, files, lines)| *unread && (*files, *lines) != (0, 0))
        .map(|(tree, _, _, _)| tree.clone())
        .collect();
    let carrying: BTreeSet<String> = carried.keys().cloned().collect();

    let undocumented: Vec<&String> = carrying.difference(&documented).collect();
    assert!(
        undocumented.is_empty(),
        "this scan never opens {undocumented:?}, tracked files under them name absolute paths \
         inside a user's home directory, and this file's module documentation does not say so — \
         so its summary line would read as a claim about the repository when it is a claim about \
         {SCANNED_PREFIXES:?} ({} findings over {examined} unread files examined):\n{}",
        findings.len(),
        findings
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    );

    let stale: Vec<&String> = documented.difference(&carrying).collect();
    assert!(
        stale.is_empty(),
        "this file's module documentation names {stale:?} as a tree it does not read and which \
         carries the class, and the repository no longer agrees: either the tree is clean now and \
         belongs in `SCANNED_PREFIXES`, or it is gone — either way the sentence has outlived its \
         reason and must go with it"
    );

    for (tree, unread_claim, files, lines) in &claims {
        let scanned = SCANNED_PREFIXES.contains(&tree.as_str());
        assert_eq!(
            !*unread_claim,
            scanned,
            "the bullet for `{tree}` claims this scan {} it, and `SCANNED_PREFIXES` says {} — the \
             verdict a bullet states is the decision itself, and it is the one word an editor \
             changes when the decision changes",
            if *unread_claim {
                "never opens"
            } else {
                "reads"
            },
            if scanned { "it is read" } else { "it is not" }
        );
        let (measured_files, measured_lines) = carried.get(tree).copied().unwrap_or((0, 0));
        assert_eq!(
            (*files, *lines),
            (measured_files, measured_lines),
            "the bullet for `{tree}` rests on {files} files and {lines} lines carrying the class, \
             and the detector finds {measured_files} files and {measured_lines} lines. A count in \
             that bullet is not decoration: the decision this lane records was argued from one, \
             and it was argued from the wrong one because nothing read it"
        );
    }
}
