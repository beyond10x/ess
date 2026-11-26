//! Adversary pass 2 against the signal handling eb18d0e5fb added to `ess verify conform run
//! --target process` (`crates/edge/ess-cli/src/target_process.rs`, story
//! a-conformance-run-drives-a-target-process-in-any-language).
//!
//! The targets are `/bin/sh` and `/usr/bin/perl` scripts around this build's own `ess verify
//! conform serve --target billing`, so a case can place the target where it needs it: out of its
//! process group, or still running after its standard input closed. Every case kills what it
//! started when it fails.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal};

const ESS: &str = env!("CARGO_BIN_EXE_ess");
const SUITE: &str = "suites/generated/billing/suite.json";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the CLI is three levels below the workspace root")
        .to_path_buf()
}

/// The first number of every line of `file`: the pids the target wrote down.
fn pids_in(file: &Path) -> Vec<i32> {
    fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_whitespace().next()?.parse().ok())
        .collect()
}

/// Whether `raw` names a process that has not exited: a zombie has.
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

/// Whether every pid of `file` is gone within a few seconds (an orphan is reaped by whoever adopts
/// it).
fn gone(file: &Path) -> Vec<i32> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut left = pids_in(file);
    while !left.is_empty() && Instant::now() < deadline {
        left.retain(|pid| alive(*pid));
        std::thread::sleep(Duration::from_millis(20));
    }
    left
}

/// `ess` and every pid its target wrote down, all killed when the case ends, red or green.
struct Run {
    ess: Child,
    pids: PathBuf,
}

impl Drop for Run {
    fn drop(&mut self) {
        let _ = self.ess.kill();
        let _ = self.ess.wait();
        for raw in pids_in(&self.pids) {
            if let Some(pid) = Pid::from_raw(raw) {
                let _ = rustix::process::kill_process(pid, Signal::KILL);
            }
        }
    }
}

impl Run {
    /// `ess verify conform run` over the billing suite against `target`, writing its report to
    /// `report`.
    fn start(report: &Path, pids: &Path, target: &[&str]) -> Self {
        let ess = Command::new(ESS)
            .current_dir(root())
            .args(["verify", "conform", "run", "--suite", SUITE])
            .args(["--report-format", "2", "--report-out"])
            .arg(report)
            .args(["--target", "process", "--"])
            .args(target)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("ess starts");
        Self {
            ess,
            pids: pids.to_path_buf(),
        }
    }

    /// `ess`'s exit status, or `None` when it has not exited within `limit`.
    fn exit_within(&mut self, limit: Duration) -> Option<std::process::ExitStatus> {
        let deadline = Instant::now() + limit;
        while Instant::now() < deadline {
            if let Some(status) = self.ess.try_wait().unwrap() {
                return Some(status);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    fn signal(&self, signal: Signal) {
        let pid = Pid::from_raw(i32::try_from(self.ess.id()).unwrap()).unwrap();
        rustix::process::kill_process(pid, signal).expect("ess is signalled");
    }
}

/// Waits up to `limit` for `file` to exist.
fn appears(file: &Path, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if file.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

/// A target that serves the billing reference and, once its standard input has closed and the
/// reference has returned, writes `done` and keeps running: so `ess` waits out its end grace.
const LINGERS: &str =
    r#"echo $$ >> "$1"; "$0" verify conform serve --target billing; : > "$2"; exec sleep 1000"#;

// Claim under test: `target_process.rs:81-87` ("SIGKILL to the group leader leads, and to leader
// itself, which may have left its group; either way the reap that follows cannot wait forever").
// A target whose leader moved into another process group of its session, and which keeps running
// after its standard input closed, is killed at the end of the run and `ess` exits. Kills the mutant
// that keeps the group kill and drops the leader's: there `ess` waits in `Child::wait` forever.
// Bounded at 120 s, which is the billing run through `serve` (about 10 s here, debug build) plus the
// 5 s end grace, with room for a loaded machine.
#[test]
fn a_target_leader_that_left_its_process_group_is_still_ended_with_the_run() {
    let dir = tempfile::tempdir().unwrap();
    let (report, pids, done) = (
        dir.path().join("report.json"),
        dir.path().join("pids"),
        dir.path().join("done"),
    );
    let (pids_s, done_s) = (pids.to_str().unwrap(), done.to_str().unwrap());
    // Joins the process group of its parent, `ess`, records its pid and group, then runs LINGERS.
    let perl = r#"setpgrp(0, getpgrp(getppid())) or die "setpgrp: $!"; exec @ARGV"#;
    let mut run = Run::start(
        &report,
        &pids,
        &[
            "/usr/bin/perl",
            "-e",
            perl,
            "/bin/sh",
            "-c",
            &format!(r#"echo "$$ $(ps -o pgid= -p $$)" >> "$1.groups"; {LINGERS}"#),
            ESS,
            pids_s,
            done_s,
        ],
    );
    let status = run.exit_within(Duration::from_secs(120));
    let groups = fs::read_to_string(format!("{pids_s}.groups")).unwrap_or_default();
    assert!(
        groups
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(2)
            .next()
            .is_some_and(|pair| pair[0] != pair[1]),
        "the target left its own process group: {groups:?}"
    );
    assert!(done.exists(), "the target served the run");
    let status = status.expect("ess exits once the run is over, within 120 s");
    assert_eq!(status.code(), Some(0), "{status:?}");
    assert_eq!(
        gone(&pids),
        Vec::<i32>::new(),
        "the target's leader is gone"
    );
}

// Claim under test: eb18d0e5fb's message ("on SIGINT/SIGTERM/SIGHUP it kills the group and the
// leader, reaps, and exits 128 + the signal number without writing a report"), for a signal during
// the five-second end grace, which no case of the unit sends one in, with a report from an earlier
// run already at `--report-out`. Bounded at 60 s for the run and 10 s for the exit after the signal,
// both far past what they take; the signal lands inside the 5 s grace because the target writes
// `done` only once the run has closed its input.
#[test]
fn a_sigterm_during_the_end_grace_dies_of_sigterm_kills_the_target_and_leaves_an_earlier_report_as_it_was(
) {
    let dir = tempfile::tempdir().unwrap();
    let (report, pids, done) = (
        dir.path().join("report.json"),
        dir.path().join("pids"),
        dir.path().join("done"),
    );
    fs::write(&report, "an earlier report\n").unwrap();
    let mut run = Run::start(
        &report,
        &pids,
        &[
            "/bin/sh",
            "-c",
            LINGERS,
            ESS,
            pids.to_str().unwrap(),
            done.to_str().unwrap(),
        ],
    );
    assert!(
        appears(&done, Duration::from_secs(60)),
        "the target served the run and saw its input close"
    );
    run.signal(Signal::TERM);
    let status = run
        .exit_within(Duration::from_secs(10))
        .expect("ess exits on SIGTERM");
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&status),
        Some(15),
        "{status:?}"
    );
    assert_eq!(
        fs::read_to_string(&report).unwrap(),
        "an earlier report\n",
        "a signalled run leaves the report file as it was"
    );
    assert_eq!(gone(&pids), Vec::<i32>::new(), "the target is gone");
}

// Claim under test: the same, for a signal while `ess` waits for the handshake, and a second signal
// right after the first: the first one decides the status. Bounded at 30 s for the target to start
// and 10 s for the exit.
#[test]
fn a_sigint_and_a_sigterm_during_the_handshake_die_of_the_signal_and_kill_the_target() {
    let dir = tempfile::tempdir().unwrap();
    let (report, pids) = (dir.path().join("report.json"), dir.path().join("pids"));
    let mut run = Run::start(
        &report,
        &pids,
        &[
            "/bin/sh",
            "-c",
            r#"echo $$ >> "$0"; exec sleep 1000"#,
            pids.to_str().unwrap(),
        ],
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    while pids_in(&pids).is_empty() {
        assert!(Instant::now() < deadline, "the target never started");
        std::thread::sleep(Duration::from_millis(20));
    }
    run.signal(Signal::INT);
    run.signal(Signal::TERM);
    let status = run
        .exit_within(Duration::from_secs(10))
        .expect("ess exits on SIGINT");
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&status),
        Some(2),
        "{status:?}"
    );
    assert!(!report.exists(), "a signalled run writes no report");
    assert_eq!(gone(&pids), Vec::<i32>::new(), "the target is gone");
}

/// The names of `ess`'s threads that do not block `SIGHUP`, `SIGINT` and `SIGTERM` (bits 0, 1 and
/// 14 of `SigBlk` in `/proc/<pid>/task/<tid>/status`), sorted.
fn threads_that_take_an_ending_signal(pid: u32) -> Vec<String> {
    const ENDING: u64 = 1 << 0 | 1 << 1 | 1 << 14;
    let mut names = Vec::new();
    for task in fs::read_dir(format!("/proc/{pid}/task")).unwrap().flatten() {
        let Ok(status) = fs::read_to_string(task.path().join("status")) else {
            continue;
        };
        let blocked = status
            .lines()
            .find_map(|line| line.strip_prefix("SigBlk:"))
            .and_then(|mask| u64::from_str_radix(mask.trim(), 16).ok())
            .unwrap_or(0);
        if blocked & ENDING != ENDING {
            let name = fs::read_to_string(task.path().join("comm")).unwrap_or_default();
            names.push(name.trim().to_owned());
        }
    }
    names.sort();
    names
}

// Claim under test: the first of two signals decides, however the kernel spreads them over `ess`'s
// threads. A process-directed signal goes to any thread that does not block it, so with every
// thread open a SIGINT and a SIGTERM sent right after it run their handlers on two threads at once,
// and the one that records first decides: the SIGTERM did, when the SIGINT's thread was preempted
// between taking the signal and recording it (3 CI runs of this file, 1 in 100 local runs under
// load). That window cannot be hit from outside on demand, so this case holds what closes it:
// during the handshake only the signal thread takes an ending signal, and it takes them one at a
// time, in the order the kernel hands them out. Bounded at 30 s for the target to start and 10 s
// for the exit.
#[test]
fn only_the_signal_thread_takes_an_ending_signal_so_the_first_of_two_decides() {
    let dir = tempfile::tempdir().unwrap();
    let (report, pids) = (dir.path().join("report.json"), dir.path().join("pids"));
    let mut run = Run::start(
        &report,
        &pids,
        &[
            "/bin/sh",
            "-c",
            r#"echo $$ >> "$0"; exec sleep 1000"#,
            pids.to_str().unwrap(),
        ],
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    while pids_in(&pids).is_empty() {
        assert!(Instant::now() < deadline, "the target never started");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        threads_that_take_an_ending_signal(run.ess.id()),
        ["ess-target-sign"],
        "the signal thread (its name cut to 15 bytes) is the only one open to an ending signal"
    );
    run.signal(Signal::INT);
    run.signal(Signal::TERM);
    let status = run
        .exit_within(Duration::from_secs(10))
        .expect("ess exits on SIGINT");
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&status),
        Some(2),
        "{status:?}"
    );
    assert_eq!(gone(&pids), Vec::<i32>::new(), "the target is gone");
}
