//! Adversarial cases against the controls the lane gained in its second correction.
//!
//! The lane's first independent control was measured to be no control at all: every expectation in
//! it was derived from `HOME_MARKERS`, so replacing that constant with three strings no host uses
//! left the whole lane green. The answer was two controls the constant cannot reach — the home
//! root of each platform `.github/workflows/ci.yml` runs on, read through `RUNNER_HOME_ROOTS`, and
//! a path built from this process's own `HOME`. These cases drive those two, and the file
//! selection's new failure mode, against what they claim about themselves.
//!
//! The green half of this pass — the regressions the lane should keep — is in
//! `host_paths_adversary.rs`. Both files share `host_paths_lane/mod.rs`, which copies the lane's
//! own functions without substitution and reads its constants out of its source at run time, so
//! nothing here is a paraphrase and nothing here carries a home-directory marker as a literal.

mod host_paths_lane;

use host_paths_lane::{
    assert_current, ci_runner_labels, home_paths, home_paths_in, workspace_root, HOME_MARKERS,
    LANE, RUNNER_HOME_ROOTS,
};
use std::fs;
use std::path::PathBuf;

/// A throwaway directory outside the repository, named so two runs cannot collide.
fn throwaway(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-host-paths-adversary-2-{label}-{}-{}",
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

/// The way a process running on `family` spells an absolute path inside its own home directory.
///
/// Written here rather than derived from the lane, because a control derived from the thing under
/// test agrees with it by construction — which is the defect the lane's second correction exists
/// to fix. Each is joined from its parts at run time for the same reason the lane stopped spelling
/// its controls: these bytes are tracked under `crates/` and the lane scans them.
fn native_home_path(family: &str) -> Option<String> {
    let separator = '/';
    match family {
        "ubuntu" => Some(format!(
            "{separator}home{separator}runner{separator}work{separator}ess{separator}report.json"
        )),
        "macos" => Some(format!(
            "{separator}Users{separator}runner{separator}work{separator}ess{separator}report.json"
        )),
        // A Windows runner's home is `C:` then the native separator, `Users`, then the account —
        // `runneradmin` on a GitHub-hosted one — and `Path::display` on that host writes it this
        // way into whatever file records it.
        "windows" => {
            let native = '\\';
            Some(format!(
                "C:{native}Users{native}runneradmin{native}work{native}ess{native}report.json"
            ))
        }
        _ => None,
    }
}

/// Every home root the lane's runner table names is one its detector can actually see.
///
/// `RUNNER_HOME_ROOTS` exists to fail when CI runs on a platform whose home directories the
/// markers do not refuse. It cannot do that for one of the three families it lists. The row maps
/// `windows` to the macOS home root, and the assertion it feeds only asks whether that string is
/// in `HOME_MARKERS` — which it is, because macOS put it there. Add a Windows runner to the
/// workflow and the lane reports the platform as covered, while the spelling a process on that
/// host actually writes, `C:\Users\…`, contains no marker at all and is collected by nothing.
///
/// The table is the lane's own claim about the world, and this is the world's answer.
#[test]
fn every_home_root_the_runner_table_names_is_one_the_detector_can_see() {
    assert_current();
    let mut uncovered = Vec::new();
    let mut checked = 0usize;
    for (family, home_root) in RUNNER_HOME_ROOTS {
        let Some(path) = native_home_path(family) else {
            continue;
        };
        checked += 1;
        let found = home_paths(&format!("the run wrote {path} before exiting"));
        if found.is_empty() {
            uncovered.push(format!(
                "  `{family}` is recorded as having its home directories under `{home_root}`, and \
                 a path inside one of them is spelled `{path}` there; the detector collected \
                 nothing"
            ));
        }
    }
    assert_eq!(
        checked,
        RUNNER_HOME_ROOTS.iter().count(),
        "every family the lane's table names needs a native spelling here, or this case passes \
         the ones it does not know"
    );
    assert!(
        uncovered.is_empty(),
        "`RUNNER_HOME_ROOTS` in `{LANE}` claims a home root for each family it lists, and the \
         case it feeds passes as long as that string is in `HOME_MARKERS`; for these families the \
         claim is true of the string and false of the platform:\n{}",
        uncovered.join("\n")
    );
}

/// A runner family the lane's table has never heard of is still read out of the workflow.
///
/// `ci_runner_labels` is documented as reading the workflow "so that adding a platform to CI
/// without adding its home root to `HOME_MARKERS` fails this lane instead of quietly narrowing
/// it". It cannot do that either: the parse looks for the families `RUNNER_HOME_ROOTS` already
/// names and nothing else, so a platform with no entry produces no label, the `unwrap_or_else`
/// that would panic on an unknown label is unreachable, and the case stays green on the strength
/// of the runners that *are* known. The one shape that does fail is a workflow naming no known
/// family at all, which is the one shape adding a platform never produces.
#[test]
fn a_runner_family_the_table_does_not_know_is_still_read_from_the_workflow() {
    assert_current();
    let root = throwaway("workflow");
    fs::create_dir_all(root.join(".github/workflows")).expect("the workflow directory is created");
    let unknown = "freebsd-14";
    fs::write(
        root.join(".github/workflows/ci.yml"),
        format!("jobs:\n  gate:\n    runs-on: ubuntu-latest\n  native:\n    runs-on: {unknown}\n"),
    )
    .expect("the workflow is written");

    let labels = ci_runner_labels(&root);
    assert!(
        labels.iter().any(|label| label == "ubuntu-latest"),
        "the parse did not find the runner it does know, so this case measures nothing: {labels:?}"
    );
    assert!(
        labels.iter().any(|label| label == unknown),
        "this workflow runs a job on `{unknown}`, a platform `RUNNER_HOME_ROOTS` has no entry \
         for; the parse in `{LANE}` returned {labels:?}, so the case it feeds never reaches the \
         panic that would report the platform as uncovered and passes on the strength of \
         `ubuntu-latest` alone"
    );
    fs::remove_dir_all(&root).expect("the throwaway directory can be removed");
}

/// The gate's verdict is a fact about the repository, not about the account that runs it.
///
/// The lane's other independent control asserts that the detector finds a path built from
/// `std::env::var("HOME")`. That makes the gate's answer depend on an environment variable which
/// neither the repository nor the workflow sets. Run it under an account whose home directory is
/// not under one of the three markers and the lane refuses a clean repository, blaming the marker
/// set; run it with `HOME` unset and it panics on `.expect("a POSIX host gives every process a
/// HOME")`, which POSIX does not promise.
///
/// The homes below are this host's own, read from `/etc/passwd` and restricted to accounts with a
/// login shell — accounts something could plausibly run a build as. They are not invented.
#[test]
fn the_home_control_holds_for_the_home_directories_this_host_records() {
    assert_current();
    let passwd = fs::read_to_string("/etc/passwd").expect("a POSIX host has /etc/passwd");
    let mut refused = Vec::new();
    let mut checked = 0usize;
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        let [account, _, _, _, _, home, shell] = fields[..] else {
            continue;
        };
        if shell.ends_with("nologin") || shell.ends_with("false") || !home.starts_with('/') {
            continue;
        }
        checked += 1;
        // The lane's own control, with the home directory it would have taken from `HOME`.
        let expected = format!("{}/.cache/ess/report.json", home.trim_end_matches('/'));
        if !home_paths(&format!("the run wrote {expected} before exiting")).contains(&expected) {
            refused.push(format!(
                "  `{account}` has a login shell and its home directory is `{home}`, so the lane \
                 refuses this repository when it is run as that account"
            ));
        }
    }
    assert!(
        checked > 0,
        "this host records no account with a login shell, so this case measures nothing"
    );
    assert!(
        refused.is_empty(),
        "the control in `{LANE}` asserts that the markers describe the home directory of \
         whichever account runs the gate, and turns the gate red when they do not; {} of the {} \
         login accounts this host records would turn it red on a repository with nothing wrong \
         in it:\n{}",
        refused.len(),
        checked,
        refused.join("\n")
    );
}

/// One selected file the working tree no longer holds does not hide the findings in the others.
///
/// `git ls-files --cached` reads the index, so a tracked file deleted in the working tree is still
/// selected — an unstaged deletion, which is what a rename in progress looks like. Measured
/// against real `git`: after `rm docs/b.md`, `ls-files --cached -z` still lists `docs/b.md`.
///
/// The lane's reader panics on such a file, so the scan stops at it. The guard the correction
/// asked for — `assert_eq!(read, selected.len())` — is never reached, and cannot be: the counter
/// is incremented unconditionally at the head of a loop no branch leaves early, so it restates the
/// loop's own postcondition. What actually happens is that an unstaged deletion aborts the scan
/// before it reports the home-directory paths in every other file, and the message names the scan
/// rather than the deletion.
#[test]
fn one_selected_file_the_working_tree_lacks_does_not_abort_the_scan() {
    assert_current();
    let directory = throwaway("absent");
    let leaked = format!(
        "{}someone/.cache/ess/report.json",
        HOME_MARKERS.resolve()[0]
    );
    let present = directory.join("present.md");
    fs::write(&present, format!("the run wrote {leaked} before exiting\n"))
        .expect("the throwaway file can be written");
    let absent = directory.join("deleted-between-listing-and-reading.md");

    // The lane's loop over its selection, in the lane's order: the file `git` still lists but the
    // working tree no longer holds comes first, the file with something to report comes second.
    let selection = [absent.clone(), present.clone()];
    let scan = std::panic::catch_unwind(|| {
        let mut leaks = Vec::new();
        for file in &selection {
            for (line, path) in home_paths_in(file) {
                leaks.push(format!("{}:{line}: {path}", file.display()));
            }
        }
        leaks
    });

    let reported = scan.unwrap_or_else(|_| {
        panic!(
            "`{LANE}` stops its whole scan on the first selected file the working tree does not \
             hold — an unstaged deletion, which `git ls-files --cached` still lists — so the home \
             path in `{}` is never reported and the exit status blames the scan rather than the \
             deletion",
            present.display()
        )
    });
    assert!(
        reported.iter().any(|line| line.contains(&leaked)),
        "the scan returned {reported:?} and none of it is the home path in `{}`",
        present.display()
    );
    fs::remove_dir_all(&directory).expect("the throwaway directory can be removed");
}

/// A relative path whose directory happens to be named like a marker is not an absolute one.
///
/// The lane's module documentation is explicit about what it refuses and why: *"What is refused is
/// an **absolute** home-directory path, because that is what leaks: it names a machine, a user
/// account and a directory layout that no other reader has."* The detector looks for its markers
/// anywhere in a line and never asks what precedes them, so any path or URL with a directory
/// component of that name satisfies it — `website/` is a Docusaurus site, where a route named
/// after the marker is an ordinary page, and a JSON Pointer into a document's root is an ordinary
/// pointer. The finding the lane then prints is an absolute path that exists on no host: the
/// relative prefix is cut away, so the report names a file the reader cannot go and look at.
///
/// This is the failure mode the lane's own comment calls the one that gets a gate switched off,
/// and it is the same bug as the one the sentence-ending marker had — a match is being accepted
/// without asking what is on the other side of it.
#[test]
fn a_relative_path_named_like_a_marker_is_not_an_absolute_home_directory_path() {
    assert_current();
    let mut invented = Vec::new();
    for marker in HOME_MARKERS {
        let tail = marker.trim_start_matches('/');
        for text in [
            format!("the page is website/src/pages/{tail}index.md"),
            format!("see https://example.com/documentation/{tail}getting-started"),
            format!("the pointer /document{marker}children/0 addresses it"),
        ] {
            let found = home_paths(&text);
            if !found.is_empty() {
                invented.push(format!("  `{text}` was collected as {found:?}"));
            }
        }
        // The control: the same marker, absolute, is a leak and must still be collected.
        let leaked = format!("{marker}someone/.cache/ess/report.json");
        assert!(
            home_paths(&format!("the run wrote {leaked}")).contains(&leaked),
            "`{leaked}` is an absolute path under a user's home directory and must be collected, \
             or the assertion below passes for the wrong reason"
        );
    }
    assert!(
        invented.is_empty(),
        "`{LANE}` refuses an *absolute* path under a user's home directory, and says so; it \
         collected these relative ones, cutting the prefix away so that the finding names a file \
         that exists on no host:\n{}",
        invented.join("\n")
    );
}

/// A home directory whose account name does not start with an ASCII byte is still refused.
///
/// `continues_a_path` accepts ASCII bytes only, so nothing follows the marker as far as the
/// detector is concerned, the candidate trims back to the directory the marker names, and the
/// whole path leaves the scan. The lane reasons about this encoding once — *"A marker is ASCII, so
/// `String::from_utf8_lossy` cannot hide one"* — and that is true of the marker and false of what
/// comes after it. An account name is not required to be ASCII on either platform CI runs on, and
/// the acceptance statement refuses a tracked file that contains a path under *a user's* home
/// directory without qualifying whose.
///
/// Only the first byte after the marker decides it: a non-ASCII byte later in the path ends the
/// candidate early and the truncated path is still reported.
#[test]
fn a_home_directory_whose_account_name_is_not_ascii_is_still_refused() {
    assert_current();
    let mut missed = Vec::new();
    for marker in HOME_MARKERS {
        let leaked = format!("{marker}ärni/.cache/ess/report.json");
        let found = home_paths(&format!("the run wrote {leaked} before exiting"));
        if found.is_empty() {
            missed.push(format!("  `{leaked}` was collected as nothing"));
        }
    }
    assert!(
        missed.is_empty(),
        "an account name is not required to start with an ASCII byte, and the acceptance \
         statement refuses a tracked file that contains a path under a user's home directory \
         whoever the user is; `{LANE}` does not see these at all:\n{}",
        missed.join("\n")
    );
}

/// The lane's own bytes are scanned, and the scan is reaching them.
///
/// Not an attack; the anti-vacuity half of the two cases above that read the repository. If the
/// selection ever stops producing the lane itself, the cases that drive `scanned_files` are
/// measuring a tree that is not this one.
#[test]
fn the_lane_is_itself_in_the_selection_these_cases_read() {
    assert_current();
    let root = workspace_root();
    assert!(
        root.join(LANE).is_file(),
        "`{LANE}` is the file under attack and it is not where these cases look for it"
    );
    assert!(
        home_paths_in(&root.join(LANE)).is_empty(),
        "the lane's own bytes name a path under a user's home directory: {:?}",
        home_paths_in(&root.join(LANE))
    );
}
