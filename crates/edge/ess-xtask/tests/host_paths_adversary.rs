//! Regression cases against the gate lane in `crates/edge/ess-xtask/tests/host_paths.rs`.
//!
//! The lane is accepted against one sentence: *no tracked file under `crates/`, `docs/`,
//! `website/` or `models/` contains a path under a user's home directory, and a gate lane refuses
//! one*. Each case here holds the lane to one property of that sentence, or to a contract the
//! lane's own documentation states, and each is green against the lane as it now stands.
//!
//! This file replaces the first adversary pass's five cases. Four of those could not go green
//! against any correct lane, because they were a demonstration rather than a suite: two called a
//! frozen copy of the detector as it was *before* it was corrected and so reported the old
//! behaviour whatever the lane did, one required `fs::read_to_string` to succeed on every selected
//! file when two tracked files are binaries, and one asserted on an obsolescence guard that every
//! possible fix trips by construction. What each of them was *about* is kept; what is dropped is
//! the frozen copy and the guards. `host_paths_lane/mod.rs` explains the discipline that replaces
//! them: the detector is copied without substitution, [`the_transcribed_detector_is_byte_identical_to_the_lane_s`]
//! proves it still is, and every constant is read out of the lane's source at run time rather than
//! written down, so no constant can drift and no marker appears in these bytes as a literal.
//!
//! The attacking half of this pass is in `host_paths_adversary_2.rs`, which is red.

mod host_paths_lane;

use host_paths_lane::{
    assert_current, home_paths, home_paths_in, scanned_files, transcription_drift, workspace_root,
    HOME_MARKERS, SCANNED_PREFIXES, SEPARATOR_SPELLINGS, TRANSCRIBED,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A path under a user's home directory, joined at run time from the lane's own marker.
///
/// Never written as a literal, for the reason the lane stopped writing its controls as literals:
/// a text scanner cannot tell a control from a leak, and these files are tracked under `crates/`
/// and scanned by the lane like any other.
fn control_path(marker: &str, tail: &str) -> String {
    format!("{marker}{tail}")
}

/// A throwaway directory outside the repository, named so two runs cannot collide.
fn throwaway(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-host-paths-adversary-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the host clock is after the epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("a throwaway directory can be created");
    // Named so a case that fails before its own cleanup leaves something findable rather than
    // something anonymous; `support.rs` in this package retains its fixtures the same way.
    println!("retained fixture: {}", path.display());
    path
}

/// The home directory `/etc/passwd` records for `account`, if it records one.
fn recorded_home(account: &str) -> Option<String> {
    let passwd = fs::read_to_string("/etc/passwd").ok()?;
    passwd.lines().find_map(|line| {
        let mut fields = line.split(':');
        let name = fields.next()?;
        let home = fields.nth(4)?;
        (name == account && !home.is_empty()).then(|| home.to_owned())
    })
}

/// The copied detector is the lane's own text, so every case below attacks the lane.
///
/// The first pass proved this with a byte comparison too, and it went stale the moment the lane
/// was corrected — its copy kept reporting the pre-correction behaviour while reading as though it
/// reported the lane's. That is the failure this case has to make impossible rather than merely
/// unlikely, so the copy carries no substitution at all and the comparison is over every function
/// that is copied, by name. When it goes red it names the function and prints both texts: the
/// answer is to re-copy that function and re-read the cases that drive it, never to edit the copy.
///
/// The second half is the anti-vacuity half. A comparison of two empty extractions passes, and a
/// constant that parsed to nothing would leave every case below asserting over an empty loop.
#[test]
fn the_transcribed_detector_is_byte_identical_to_the_lane_s() {
    let drift = transcription_drift();
    assert!(
        drift.is_empty(),
        "the copied detector is no longer the lane's own text:\n\n{}",
        drift.join("\n\n")
    );
    assert!(
        TRANSCRIBED.len() >= 7,
        "the detector, its two helpers, the file selection, the per-file reader and the CI \
         workflow parse are what these cases drive; {} function(s) are copied",
        TRANSCRIBED.len()
    );
    for (name, constant) in [
        ("HOME_MARKERS", HOME_MARKERS),
        ("SEPARATOR_SPELLINGS", SEPARATOR_SPELLINGS),
        ("SCANNED_PREFIXES", SCANNED_PREFIXES),
    ] {
        assert!(
            !constant.resolve().is_empty(),
            "`{name}` was read out of the lane's source and resolved to nothing, so every case \
             that loops over it measures nothing"
        );
    }
}

/// Every file the lane's selection hands it is examined, binaries included.
///
/// The lane used to read each selected file with `fs::read_to_string` and take a bare `continue`
/// when that failed, before the counter, so a file it dropped was indistinguishable from a file it
/// cleared. Two tracked files under `website/` are binaries and were dropped on every run.
///
/// The first pass asserted this by requiring `fs::read_to_string` to succeed on every selected
/// file, which is red against a *correct* lane as much as a broken one — the repository tracks two
/// binaries and is entitled to. The property is not "every selected file is text"; it is "every
/// selected file is examined". So this drives the lane's own reader over the lane's own selection
/// and counts what came back, and the case that a home path inside an undecodable file is still
/// found is made against a file built here rather than against whatever the repository happens to
/// track today.
#[test]
fn every_tracked_file_the_lane_selects_is_examined_rather_than_silently_dropped() {
    assert_current();
    let root = workspace_root();
    let selected = scanned_files(&root);
    assert!(
        !selected.is_empty(),
        "the lane's own selection produced nothing, so this case measures nothing"
    );

    let mut examined = 0usize;
    for file in &selected {
        // The lane's reader panics rather than returning when it cannot read a selected file, so
        // reaching the increment is the evidence that the file was examined.
        let _ = home_paths_in(&root.join(file));
        examined += 1;
    }
    assert_eq!(
        examined,
        selected.len(),
        "every file the selection hands the scan must be examined"
    );

    let directory = throwaway("undecodable");
    let leaked = control_path(&HOME_MARKERS.resolve()[0], "someone/.cache/ess/report.json");
    let mut bytes = vec![0xff, 0xfe, 0x00];
    bytes.extend_from_slice(format!("the run wrote {leaked} before exiting\n").as_bytes());
    let file = directory.join("undecodable.bin");
    fs::write(&file, &bytes).expect("the throwaway file can be written");
    assert!(
        String::from_utf8(bytes).is_err(),
        "this case is about a file that does not decode as UTF-8, and this one does"
    );
    let found = home_paths_in(&file);
    assert!(
        found.iter().any(|(_, path)| path == &leaked),
        "a home-directory path inside a file that does not decode as UTF-8 is still a home \
         directory path in a tracked file, and the acceptance statement refuses it; the lane's \
         reader returned {found:?}"
    );
    fs::remove_dir_all(&directory).expect("the throwaway directory can be removed");
}

/// A marker that ends a sentence names a directory, not a path inside anybody's home.
///
/// The lane's module documentation states this contract and the detector now honours it: `.` is a
/// byte `continues_a_path` accepts, so a full stop used to satisfy "something follows the marker",
/// and the trailing trim then took the stop *and* the marker's own slash, leaving a finding whose
/// whole text was the marker's parent directory — a name on every host, reported as a leak. A gate
/// that refuses a clean file is the failure mode that gets a gate switched off.
///
/// Asked of every marker the lane carries rather than of one, and paired with a positive control,
/// because a detector that collects nothing at all would satisfy the negative half alone.
#[test]
fn a_marker_that_ends_a_sentence_is_not_an_absolute_home_directory_path() {
    assert_current();
    for marker in HOME_MARKERS {
        for text in [
            format!("on this host every account lives under {marker}."),
            format!("on this host every account lives under {marker}"),
            format!("the directory {marker} is where such accounts live"),
            format!("the path {marker}/ is the directory itself"),
            format!("it is {marker}.."),
        ] {
            let found = home_paths(&text);
            assert!(
                found.is_empty(),
                "`{text}` names a directory and no user's home path; the detector collected \
                 {found:?}"
            );
        }
        let leaked = control_path(marker, "someone/notes.md");
        assert!(
            home_paths(&format!("it was written to {leaked}.")).contains(&leaked),
            "`{leaked}` ends a sentence too, and it is a path under a user's home directory; a \
             detector that collects nothing satisfies the half above without doing anything"
        );
    }
}

/// A home-directory path is still one when a serializer has escaped or encoded the separator.
///
/// The acceptance statement refuses a tracked file that *contains* a path under a user's home
/// directory and says nothing about how the separator is spelled. Each spelling is taken from the
/// lane's own `SEPARATOR_SPELLINGS` rather than listed here, so a spelling added to the lane is
/// exercised the day it is added and a spelling removed cannot leave a stale expectation behind.
#[test]
fn an_escaped_or_encoded_home_directory_path_is_still_refused() {
    assert_current();
    let plain = control_path(
        &HOME_MARKERS.resolve()[0],
        "someone/.local/state/trees/wt-0/examples/billing/system.yaml",
    );
    let missed: Vec<String> = SEPARATOR_SPELLINGS
        .iter()
        .filter(|spelling| !home_paths(&plain.replace('/', spelling)).contains(&plain))
        .map(String::clone)
        .collect();
    assert!(
        missed.is_empty(),
        "`{plain}` is a path under a user's home directory and the acceptance statement refuses a \
         tracked file that contains one however it is spelled; the detector does not see it \
         written with {missed:?} for its separator"
    );
}

/// Normalising the separator does not invent a finding in the tree as it stands.
///
/// The other side of the case above, and the one the lane cannot check for itself: rewriting three
/// spellings of `/` before matching is a new way to *manufacture* a home path out of bytes that
/// never spelled one, and eight tracked files under `crates/` already carry `\/`. A finding the
/// lane reports must therefore appear in the file the plain way as well. A gate whose first
/// finding on a clean tree is one it invented is a gate that gets switched off.
#[test]
fn normalising_the_separator_invents_no_finding_in_the_tracked_tree() {
    assert_current();
    let root = workspace_root();
    let selected = scanned_files(&root);
    assert!(
        !selected.is_empty(),
        "the lane's own selection produced nothing, so this case measures nothing"
    );
    let mut invented = Vec::new();
    for file in &selected {
        let path = root.join(file);
        let text = String::from_utf8_lossy(&fs::read(&path).expect("a selected file is readable"))
            .into_owned();
        for (line, found) in home_paths_in(&path) {
            if !text.contains(&found) {
                invented.push(format!("{file}:{line}: {found}"));
            }
        }
    }
    assert!(
        invented.is_empty(),
        "the lane reports {} finding(s) that the file does not spell the plain way, so \
         normalising the separator manufactured them:\n{}",
        invented.len(),
        invented.join("\n")
    );
}

/// The lane's selection keeps a tracked path whose name `git` would quote.
///
/// With `core.quotePath` at its default, `git ls-files` wraps any path holding a non-ASCII byte in
/// `"` and octal-escapes it; such a line starts with a quote, matches none of the scanned
/// prefixes, and the file leaves the scan before it is read — and before it is counted, so no
/// guard sees it go. The lane passes `-z` and splits on NUL, which turns the quoting off at the
/// source.
///
/// The first pass asserted this against a hand-written `git` transcript and guarded the assertion
/// with "this case is obsolete if the lane mentions `-z`", which every possible fix trips. This
/// one runs the lane's own selection against a throwaway repository that really does track such a
/// path, so it stays true of whatever the lane does next and goes red if `-z` is ever dropped.
#[test]
fn the_selection_keeps_a_tracked_path_that_git_quotes() {
    assert_current();
    let repository = throwaway("quoted");
    let tree = SCANNED_PREFIXES.resolve()[0]
        .trim_end_matches('/')
        .to_owned();
    fs::create_dir_all(repository.join(&tree)).expect("the scanned tree can be created");
    let quoted = format!("{tree}/straße.md");
    let plain = format!("{tree}/plain.md");
    for file in [&quoted, &plain] {
        fs::write(repository.join(file), "no home path here\n").expect("the file can be written");
    }
    for arguments in [vec!["init", "-q", "-b", "main"], vec!["add", "-A"]] {
        let run = Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(&arguments)
            .output()
            .expect("git runs");
        assert!(
            run.status.success(),
            "`git {}` failed in the throwaway repository: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let selected = scanned_files(&repository);
    assert!(
        selected.contains(&plain),
        "the throwaway repository tracks `{plain}` under a scanned tree and the lane's selection \
         produced {selected:?}, so this case measures nothing"
    );
    assert!(
        selected.contains(&quoted),
        "`git` quotes a tracked path whose name is not ASCII, and a quoted name matches none of \
         the scanned prefixes, so the file leaves the scan before it is read and before anything \
         counts it; the lane's selection produced {selected:?}"
    );
    fs::remove_dir_all(&repository).expect("the throwaway repository can be removed");
}

/// The markers cover the superuser's home directory, as this host itself records it.
///
/// A control the marker set cannot reach, and the one the lane is missing. Its own independent
/// controls are the CI workflow's runner labels and this process's `HOME`, and neither names the
/// superuser: dropping `/root/` from the marker set leaves every case in the lane green, measured
/// against a copy of the lane with that one marker and its control removed. The lane's own
/// documentation says `/root/` is refused on purpose — *the defect is the class* — and nothing
/// held it to that.
///
/// `/etc/passwd` is the host's own record rather than the invoking environment's: it says `/root`
/// on Linux and `/var/root` on macOS, and the second is collected through the same marker because
/// the marker is a substring. It is a fact about the platform, not about who happens to be running
/// the gate, which is the distinction that makes it usable as a control at all.
#[test]
fn the_markers_cover_the_superuser_home_this_host_records() {
    assert_current();
    let home = recorded_home("root")
        .expect("a POSIX host records the superuser's home directory in /etc/passwd");
    assert!(
        Path::new(&home).is_absolute(),
        "`/etc/passwd` records the superuser's home as `{home}`, which is not an absolute path, \
         so this case has nothing to measure"
    );
    let tail = "/.cache/ess/report.json";
    let text = format!("the container wrote {home}{tail} before exiting");
    let found = home_paths(&text);
    assert!(
        found.iter().any(|path| path.ends_with(tail)),
        "this host records the superuser's home directory as `{home}`, so `{home}{tail}` is a \
         path inside a user's home directory on it; the markers the lane refuses do not describe \
         it and the detector collected {found:?}"
    );
}
