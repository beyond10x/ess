//! No file this repository tracks names a path inside somebody's home directory.
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
//! A scan has two ways of being worthless, and `layout.rs` beside this file learned both: it can
//! read the wrong files, and it can be looking for a shape nothing has. Each has a case of its own
//! below rather than being assumed, because either failure exits zero.

use std::collections::BTreeSet;
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

/// Whether a byte can continue a path literal.
fn continues_a_path(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b'+' | b'@')
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
fn home_paths(text: &str) -> BTreeSet<String> {
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

/// Every home-directory path in one file, as `line: path` pairs.
///
/// Bytes rather than [`fs::read_to_string`]. The acceptance statement is about tracked files, not
/// about tracked files that happen to decode as UTF-8, and this repository tracks two that do not
/// — `website/static/img/favicon.ico` and `website/static/img/social-card.png`. A `read_to_string`
/// that fails leaves a caller with nothing to do but skip the file, and a skipped file is
/// indistinguishable from a cleared one in an exit status. A marker is ASCII, so
/// [`String::from_utf8_lossy`] cannot hide one: the replacement character is not a byte
/// [`continues_a_path`] accepts, so an undecodable run ends a path rather than joining two.
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

/// Every runner label named in `.github/workflows/ci.yml`.
///
/// Read from the workflow rather than listed here, so that adding a platform to CI without adding
/// its home root to [`HOME_MARKERS`] fails this lane instead of quietly narrowing it.
fn ci_runner_labels(root: &Path) -> BTreeSet<String> {
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
