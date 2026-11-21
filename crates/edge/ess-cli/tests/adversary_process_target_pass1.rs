//! Adversary pass 1 against `ess verify conform run --target process`
//! (`docs/design/conformance-target-process.md`,
//! story:a-conformance-run-drives-a-target-process-in-any-language).
//!
//! The target here is well behaved by the design's own rule: a `/bin/sh` adapter that reads its
//! standard input, exits when it closes (design `:140-142`), and answers through this build's
//! `ess verify conform serve --target billing`, a little slowly so that the run is still going when
//! `ess` is signalled. Like the server the design names (`:146-147`), it starts one more process in
//! its own process group. Every case kills whatever it started when it ends, red or green.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use rustix::process::{kill_process, Pid, Signal};

const ESS: &str = env!("CARGO_BIN_EXE_ess");
const SUITE: &str = "suites/generated/billing/suite.json";

/// The adapter: a server in its group, then every request passed to `serve` after 0.2 s.
const ADAPTER: &str = r#"#!/bin/sh
sleep 100000 </dev/null >/dev/null 2>&1 &
echo "$$ $!" >> "$1"
while IFS= read -r line; do sleep 0.2; printf '%s\n' "$line"; done | "$2" verify conform serve --target billing
"#;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the CLI is three levels below the workspace root")
        .to_path_buf()
}

fn pids_in(file: &Path) -> Vec<i32> {
    fs::read_to_string(file)
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|pid| pid.parse().ok())
        .collect()
}

/// Whether `raw` names a process that has not exited (a zombie has).
fn alive(raw: i32) -> bool {
    let Some(pid) = Pid::from_raw(raw) else {
        return false;
    };
    if rustix::process::test_kill_process(pid).is_err() {
        return false;
    }
    fs::read_to_string(format!("/proc/{raw}/stat")).is_ok_and(|stat| {
        !stat
            .rsplit_once(')')
            .is_some_and(|(_, rest)| rest.trim_start().starts_with('Z'))
    })
}

/// Kills `ess` and every process the adapter wrote down when the case ends, red or green.
struct Cleanup {
    ess: Child,
    pids: PathBuf,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = self.ess.kill();
        let _ = self.ess.wait();
        for raw in pids_in(&self.pids) {
            if alive(raw) {
                if let Some(pid) = Pid::from_raw(raw) {
                    let _ = kill_process(pid, Signal::KILL);
                }
            }
        }
    }
}

/// Runs the billing suite against the adapter, sends `signal` to `ess` while the run is going, and
/// returns the processes of the target's group still alive 10 s after `ess` exited.
fn left_behind_after(signal: Signal) -> (Vec<i32>, Vec<i32>) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let adapter = dir.path().join("adapter.sh");
    fs::write(&adapter, ADAPTER).unwrap();
    let pids = dir.path().join("pids");
    let ess = Command::new(ESS)
        .current_dir(root())
        .args(["verify", "conform", "run", "--suite", SUITE])
        .args(["--report-format", "2", "--report-out"])
        .arg(dir.path().join("report.json"))
        .args(["--target", "process", "--", "/bin/sh"])
        .arg(&adapter)
        .arg(&pids)
        .arg(ESS)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("ess starts");
    let ess_pid = Pid::from_child(&ess);
    let mut cleanup = Cleanup {
        ess,
        pids: pids.clone(),
    };
    let started = Instant::now() + Duration::from_secs(30);
    while pids_in(&pids).len() < 2 {
        assert!(Instant::now() < started, "the adapter never started");
        std::thread::sleep(Duration::from_millis(20));
    }
    // A few requests in: the run takes about 0.2 s per request, well over a minute in all.
    std::thread::sleep(Duration::from_millis(1500));
    assert!(
        cleanup.ess.try_wait().unwrap().is_none(),
        "the run is still going when ess is signalled"
    );
    kill_process(ess_pid, signal).expect("ess is signalled");
    cleanup.ess.wait().expect("ess exits");
    let started = pids_in(&pids);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut left = started.clone();
    while !left.is_empty() && Instant::now() < deadline {
        left.retain(|pid| alive(*pid));
        std::thread::sleep(Duration::from_millis(50));
    }
    (left, started)
}

// Claim broken: story acceptance 5 ("`ess` ends the process and leaves no child behind") and the
// brief's claim 3 for an interrupted `ess`; design `:146-147` ("a server the adapter started in its
// own group dies with it"). `ess` installs no handler: a SIGTERM ends it at once, so neither
// `ChildPeer::finish` nor its `Drop` (`crates/edge/ess-cli/src/target_process.rs:236-242`,
// `:311-315`) runs, and the target's process group, which `process_group(0)` put out of reach of
// any signal sent to `ess` (`:77`), is never signalled. The adapter obeys the design's rule and exits
// when its standard input closes (design `:140-142`); the server in its group is left running,
// reparented to init, forever.
#[test]
fn a_sigterm_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind() {
    let (left, started) = left_behind_after(Signal::TERM);
    assert_eq!(
        left,
        Vec::<i32>::new(),
        "processes of the target's group left behind after ess was terminated: {left:?} of {started:?}"
    );
}

// Claim broken: as above, for the interrupt a terminal sends on Ctrl-C. The terminal sends it to
// the foreground process group, which is `ess`'s and, by `process_group(0)`, not the target's, so
// the target's group never receives it; `ess` dies of it without signalling that group.
#[test]
fn a_sigint_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind() {
    let (left, started) = left_behind_after(Signal::INT);
    assert_eq!(
        left,
        Vec::<i32>::new(),
        "processes of the target's group left behind after ess was interrupted: {left:?} of {started:?}"
    );
}
