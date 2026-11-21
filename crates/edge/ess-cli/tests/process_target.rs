//! `ess verify conform run --target process -- <program> [args…]` and `ess verify conform serve`
//! (`docs/design/conformance-target-process.md`, "Tests the acceptance requires";
//! story:a-conformance-run-drives-a-target-process-in-any-language, acceptance statements 2 to 8).
//!
//! The well-behaved peer is `ess verify conform serve --target billing`, spawned through this
//! build's own `ess`. The misbehaving one is the test-only `ess-conformance-peer`
//! (`tests/support/conformance_peer.rs`). Every fault case checks that the peer it started, and the
//! grandchild that peer started in its process group, are gone once `ess` has exited; a case that
//! fails kills whatever its peers wrote down before it unwinds.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const ESS: &str = env!("CARGO_BIN_EXE_ess");
const PEER: &str = env!("CARGO_BIN_EXE_ess-conformance-peer");
const SUITE: &str = "suites/generated/billing/suite.json";
/// A scenario in the middle of the billing suite, with scenarios on both sides of it.
const MIDDLE: &str = "billing.invoice.CreateInvoice/outcome/accepted";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the CLI is three levels below the workspace root")
        .to_path_buf()
}

fn os(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

/// One `ess` invocation, run from the workspace root, and the report it wrote, if any.
struct Ran {
    output: Output,
    report: Option<Vec<u8>>,
}

impl Ran {
    fn code(&self) -> i32 {
        self.output.status.code().expect("ess exits with a status")
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }

    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).into_owned()
    }

    fn json(&self) -> Value {
        let bytes = self.report.as_ref().unwrap_or_else(|| {
            panic!(
                "no report written; exit {}, stderr: {}",
                self.code(),
                self.stderr()
            )
        });
        serde_json::from_slice(bytes).expect("the report is JSON")
    }

    /// Every scenario of the report with the category it is listed under.
    fn statuses(&self) -> BTreeMap<String, String> {
        let report = self.json();
        let mut statuses = BTreeMap::new();
        for (category, ids) in report["outcomes"]
            .as_object()
            .expect("report/2 has outcomes")
        {
            for id in ids.as_array().expect("an outcome category is a list") {
                statuses.insert(
                    id.as_str().expect("a scenario id").to_owned(),
                    category.clone(),
                );
            }
        }
        statuses
    }

    fn listed(&self, category: &str) -> BTreeSet<String> {
        self.json()["outcomes"][category]
            .as_array()
            .expect("an outcome category is a list")
            .iter()
            .map(|id| id.as_str().expect("a scenario id").to_owned())
            .collect()
    }
}

/// `ess verify conform run --report-format 2 --report-out <dir>/report.json` and `arguments`.
fn conform_run(dir: &Path, arguments: &[OsString]) -> Ran {
    let report = dir.join("report.json");
    let output = Command::new(ESS)
        .current_dir(root())
        .args([
            "verify",
            "conform",
            "run",
            "--report-format",
            "2",
            "--report-out",
        ])
        .arg(&report)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .expect("ess runs");
    let report = fs::read(&report).ok();
    Ran { output, report }
}

/// The billing suite against the built-in billing reference.
fn built_in(dir: &Path) -> Ran {
    let ran = conform_run(dir, &os(&["--suite", SUITE, "--target", "billing"]));
    assert!(ran.report.is_some(), "{}", ran.stderr());
    ran
}

/// The billing suite against `command`, and anything else `before` adds to the run.
fn against(dir: &Path, before: &[&str], command: &[OsString]) -> Ran {
    let mut arguments = os(&["--suite", SUITE]);
    arguments.extend(os(before));
    arguments.extend(os(&["--target", "process", "--"]));
    arguments.extend_from_slice(command);
    conform_run(dir, &arguments)
}

fn served_billing() -> Vec<OsString> {
    os(&[ESS, "verify", "conform", "serve", "--target", "billing"])
}

fn peer(flags: &[&str], pids: &Path) -> Vec<OsString> {
    let mut command = os(&[PEER]);
    command.extend(os(flags));
    command.push("--pid-file".into());
    command.push(pids.into());
    command
}

// ---- processes -----------------------------------------------------------------------------------

/// Every process id a peer wrote to its pid file: itself and its grandchild, per start.
fn pids_in(file: &Path) -> Vec<i32> {
    fs::read_to_string(file)
        .unwrap_or_default()
        .split_whitespace()
        .map(|pid| pid.parse().expect("the pid file holds process ids"))
        .collect()
}

/// Whether `pid` names a process that has not exited: a zombie has.
fn alive(raw: i32) -> bool {
    let Some(pid) = rustix::process::Pid::from_raw(raw) else {
        return false;
    };
    if rustix::process::test_kill_process(pid).is_err() {
        return false;
    }
    match fs::read_to_string(format!("/proc/{raw}/stat")) {
        Ok(stat) => !stat
            .rsplit_once(')')
            .is_some_and(|(_, rest)| rest.trim_start().starts_with('Z')),
        Err(_) => !cfg!(target_os = "linux"),
    }
}

/// Asserts every process the pid file names is gone. An orphaned grandchild is reaped by whoever
/// adopts it, so this waits a bounded while for that.
fn assert_gone(file: &Path) {
    let pids = pids_in(file);
    assert!(
        !pids.is_empty(),
        "the peer wrote no pid to {}",
        file.display()
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut left: Vec<i32> = pids.clone();
    while !left.is_empty() && Instant::now() < deadline {
        left.retain(|pid| alive(*pid));
        if !left.is_empty() {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    assert!(
        left.is_empty(),
        "processes left behind after ess exited: {left:?} of {pids:?}"
    );
}

/// Kills every process a pid file names when the case unwinds, so a failing case leaves no process
/// behind either.
struct Reaper(PathBuf);

impl Drop for Reaper {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            return;
        }
        for raw in pids_in(&self.0) {
            if let Some(pid) = rustix::process::Pid::from_raw(raw) {
                let _ = rustix::process::kill_process(pid, rustix::process::Signal::KILL);
            }
        }
    }
}

/// A fault in `MIDDLE`: it ends `error`, every other scenario keeps the built-in run's status, a
/// fresh peer served the rest, and no process is left behind.
fn assert_fault(flag: &str, timeout: Option<&str>) {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let baseline = built_in(dir.path()).statuses();
    let before: Vec<&str> = timeout.map_or_else(Vec::new, |s| vec!["--target-timeout", s]);
    let ran = against(dir.path(), &before, &peer(&[flag, MIDDLE], &pids));
    let statuses = ran.statuses();
    assert_eq!(
        statuses.get(MIDDLE).map(String::as_str),
        Some("error"),
        "{flag}: {}",
        ran.stderr()
    );
    for (scenario, status) in &baseline {
        if scenario != MIDDLE {
            assert_eq!(
                statuses.get(scenario),
                Some(status),
                "{flag}: {scenario} keeps its built-in status"
            );
        }
    }
    assert_eq!(statuses.len(), baseline.len());
    assert_ne!(ran.code(), 0, "an errored scenario does not pass the run");
    let starts = fs::read_to_string(&pids).unwrap().lines().count();
    assert_eq!(starts, 2, "{flag}: one start, one restart after the fault");
    assert_gone(&pids);
}

// ---- statement 2 ---------------------------------------------------------------------------------

#[test]
fn a_served_billing_reference_reports_what_the_built_in_one_reports() {
    let dir = tempfile::tempdir().unwrap();
    let built_in = built_in(dir.path()).json();
    let served = against(dir.path(), &[], &served_billing());
    assert_eq!(served.code(), 0, "{}", served.stderr());
    let served = served.json();
    assert_eq!(served["outcomes"], built_in["outcomes"]);
    assert_eq!(served["counts"], built_in["counts"]);
    assert!(
        served["counts"]["total"].as_u64().unwrap() > 0,
        "the suite ran scenarios: {served}"
    );
}

// ---- statement 3 ---------------------------------------------------------------------------------

/// The fresh coverage suite of `examples/billing`: `--strict` cannot pass a legacy suite.
fn coverage_run(dir: &Path, command: &[OsString]) -> Ran {
    let mut arguments = os(&[
        "--path",
        "examples/billing",
        "--suite-format",
        "5",
        "--strict",
        "--target",
        "process",
        "--",
    ]);
    arguments.extend_from_slice(command);
    conform_run(dir, &arguments)
}

const DELIVERY: &str = "notify-on-invoice-created/binding/delivery";
const SETTLED: &str = "billing.invoice.PayInvoice/outcome/settled";

#[test]
fn strict_exits_nonzero_unless_every_scenario_passed() {
    let dir = tempfile::tempdir().unwrap();
    let served = coverage_run(dir.path(), &served_billing());
    assert_eq!(served.code(), 0, "{}", served.stderr());
    assert_eq!(
        served.json()["counts"]["passed"],
        served.json()["counts"]["total"]
    );

    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let faulty = coverage_run(dir.path(), &peer(&["--wrong-outcome", MIDDLE], &pids));
    assert_eq!(faulty.code(), 1, "{}", faulty.stderr());
    assert_gone(&pids);
}

#[test]
fn each_non_passing_scenario_is_listed_under_its_own_category() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let ran = coverage_run(
        dir.path(),
        &peer(
            &[
                "--wrong-outcome",
                MIDDLE,
                "--exit-in",
                SETTLED,
                "--unsupported",
                "redeliver_event",
            ],
            &pids,
        ),
    );
    assert_eq!(ran.code(), 1, "{}", ran.stderr());
    assert_eq!(ran.listed("failed"), BTreeSet::from([MIDDLE.to_owned()]));
    assert_eq!(ran.listed("error"), BTreeSet::from([SETTLED.to_owned()]));
    assert_eq!(
        ran.listed("unsupported"),
        BTreeSet::from([DELIVERY.to_owned()])
    );
    assert_gone(&pids);
}

// ---- statement 4 ---------------------------------------------------------------------------------

#[test]
fn one_wrong_outcome_fails_that_scenario_and_no_other() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let baseline = built_in(dir.path()).statuses();
    let ran = against(dir.path(), &[], &peer(&["--wrong-outcome", MIDDLE], &pids));
    assert_eq!(ran.listed("failed"), BTreeSet::from([MIDDLE.to_owned()]));
    let statuses = ran.statuses();
    for (scenario, status) in &baseline {
        if scenario != MIDDLE {
            assert_eq!(statuses.get(scenario), Some(status), "{scenario}");
        }
    }
    assert_eq!(ran.code(), 1, "{}", ran.stderr());
    assert_eq!(
        fs::read_to_string(&pids).unwrap().lines().count(),
        1,
        "a failed scenario is not a fault: no restart"
    );
    assert_gone(&pids);
}

// ---- statement 5 ---------------------------------------------------------------------------------

#[test]
fn an_exit_mid_scenario_is_an_error() {
    assert_fault("--exit-in", None);
}

#[test]
fn a_line_that_is_not_an_answer_is_an_error() {
    assert_fault("--garbage-in", None);
}

#[test]
fn an_answer_to_another_request_is_an_error() {
    assert_fault("--wrong-id-in", None);
}

#[test]
fn an_answer_past_the_time_limit_is_an_error() {
    let started = Instant::now();
    assert_fault("--stall-in", Some("1"));
    assert!(
        started.elapsed() < Duration::from_secs(25),
        "a one-second limit bounds the wait: {:?}",
        started.elapsed()
    );
}

/// Kills `ess` when the case ends, red or green.
struct Running(std::process::Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A signal to `ess` while it waits for an answer: the target's group is killed, `ess` exits
/// 128 plus the signal's number, and no report is written.
#[test]
fn a_sighup_to_ess_waiting_for_an_answer_ends_the_targets_group_and_writes_no_report() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let record = dir.path().join("record");
    let report = dir.path().join("report.json");
    let _reaper = Reaper(pids.clone());
    let mut command = peer(&["--stall-in", MIDDLE], &pids);
    command.push("--record".into());
    command.push(record.clone().into());
    let mut ess = Running(
        Command::new(ESS)
            .current_dir(root())
            .args(["verify", "conform", "run", "--suite", SUITE])
            .args(["--report-format", "2", "--report-out"])
            .arg(&report)
            .args(["--target-timeout", "600", "--target", "process", "--"])
            .args(&command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("ess starts"),
    );
    // Waiting: the peer has recorded the request it stalls on, and nothing after it.
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut last = (String::new(), Instant::now());
    loop {
        assert!(Instant::now() < deadline, "the peer never stalled");
        assert!(ess.0.try_wait().unwrap().is_none(), "ess is still running");
        let seen = fs::read_to_string(&record).unwrap_or_default();
        if seen != last.0 {
            last = (seen, Instant::now());
        } else if seen.ends_with("execute_command\n")
            && last.1.elapsed() > Duration::from_millis(1000)
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let pid = rustix::process::Pid::from_child(&ess.0);
    rustix::process::kill_process(pid, rustix::process::Signal::HUP).expect("ess is signalled");
    let status = ess.0.wait().expect("ess exits");
    assert_eq!(status.code(), Some(128 + 1), "{status:?}");
    assert!(!report.exists(), "a signalled run writes no report");
    assert_gone(&pids);
}

#[test]
fn a_failed_restart_errors_every_remaining_scenario() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let baseline = built_in(dir.path()).statuses();
    let ran = against(
        dir.path(),
        &[],
        &peer(&["--exit-in", MIDDLE, "--fail-restart"], &pids),
    );
    let statuses = ran.statuses();
    let mut seen = false;
    for (scenario, status) in &baseline {
        seen |= scenario == MIDDLE;
        if seen {
            assert_eq!(
                statuses.get(scenario).map(String::as_str),
                Some("error"),
                "{scenario} is at or after the fault"
            );
        } else {
            assert_eq!(statuses.get(scenario), Some(status), "{scenario}");
        }
    }
    assert!(seen);
    assert_ne!(ran.code(), 0);
    let starts = fs::read_to_string(&pids).unwrap().lines().count();
    assert_eq!(starts, 2, "one start and one failed restart, never another");
    assert_gone(&pids);
}

// ---- statement 6 ---------------------------------------------------------------------------------

#[test]
fn an_unsupported_answer_is_unsupported_with_its_reason() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let suite: Value =
        serde_json::from_str(&fs::read_to_string(root().join(SUITE)).unwrap()).unwrap();
    let reading: BTreeSet<String> = suite["scenarios"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, scenario)| {
            scenario["steps"].as_array().unwrap().iter().any(|step| {
                matches!(
                    step["step"].as_str(),
                    Some("query_view" | "eventually_view")
                )
            })
        })
        .map(|(id, _)| id.clone())
        .collect();
    assert!(!reading.is_empty());
    let ran = against(
        dir.path(),
        &["--format", "json"],
        &peer(&["--unsupported", "query_view"], &pids),
    );
    assert_eq!(ran.code(), 1, "{}", ran.stderr());
    assert_eq!(ran.listed("unsupported"), reading);
    assert!(
        ran.stdout()
            .contains("the conformance peer declines query_view"),
        "the detailed run carries the target's reason: {}",
        ran.stdout()
    );
    assert_gone(&pids);
}

// ---- statement 7 ---------------------------------------------------------------------------------

#[test]
fn two_runs_write_identical_reports() {
    let dir = tempfile::tempdir().unwrap();
    let first = against(dir.path(), &[], &served_billing());
    let second = against(dir.path(), &[], &served_billing());
    assert_eq!(first.code(), 0, "{}", first.stderr());
    assert!(first.report.is_some());
    assert_eq!(first.report, second.report);
}

// ---- statement 8 ---------------------------------------------------------------------------------

#[test]
fn an_unread_protocol_version_is_refused_before_any_scenario() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let record = dir.path().join("record");
    let _reaper = Reaper(pids.clone());
    let mut command = peer(&["--speaks", "ess-conformance-target/2"], &pids);
    command.push("--record".into());
    command.push(record.clone().into());
    let ran = against(dir.path(), &[], &command);
    assert_eq!(ran.code(), 2, "{}", ran.stderr());
    assert!(
        ran.stderr().contains("ess-conformance-target/1"),
        "the refusal names the versions this ess reads: {}",
        ran.stderr()
    );
    assert!(ran.report.is_none(), "a refused run writes no report");
    let asked = fs::read_to_string(&record).unwrap();
    assert!(asked.lines().any(|op| op == "handshake"), "{asked}");
    assert!(
        !asked.lines().any(|op| op == "begin_scenario"),
        "no scenario began: {asked}"
    );
    assert_gone(&pids);
}

// ---- refusals, the working directory and the serve verb -------------------------------------------

#[test]
fn every_refusal_exits_2_and_writes_no_report() {
    let dir = tempfile::tempdir().unwrap();
    let pids = dir.path().join("pids");
    let _reaper = Reaper(pids.clone());
    let missing = dir.path().join("no-such-directory");
    let started_once = dir.path().join("started-once");
    fs::write(&started_once, "1\n").unwrap();
    let declaration = dir.path().join("known.yaml");
    let accounting = dir.path().join("accounting.json");
    let suite = os(&["--suite", SUITE]);
    let cases: Vec<(Vec<OsString>, &str)> = vec![
        (os(&["--target", "process"]), "--"),
        (
            [os(&["--target", "billing", "--"]), served_billing()].concat(),
            "--target process",
        ),
        (
            os(&["--target", "billing", "--target-dir", "."]),
            "--target-dir",
        ),
        (
            os(&["--target", "billing", "--target-timeout", "5"]),
            "--target-timeout",
        ),
        (
            [
                os(&["--target", "process", "--target-dir"]),
                vec![missing.clone().into()],
                os(&["--"]),
                served_billing(),
            ]
            .concat(),
            "no-such-directory",
        ),
        (
            [
                os(&["--target", "process", "--known-failing"]),
                vec![
                    declaration.into(),
                    "--accounting-out".into(),
                    accounting.into(),
                ],
                os(&["--"]),
                served_billing(),
            ]
            .concat(),
            "--known-failing",
        ),
        (
            [
                os(&["--target", "process", "--"]),
                vec![missing.join("adapter").into()],
            ]
            .concat(),
            "adapter",
        ),
        (
            [
                os(&[
                    "--target",
                    "process",
                    "--",
                    PEER,
                    "--fail-restart",
                    "--pid-file",
                ]),
                vec![started_once.clone().into()],
            ]
            .concat(),
            "handshake",
        ),
        (
            [
                os(&["--target", "process", "--target-timeout", "0", "--"]),
                served_billing(),
            ]
            .concat(),
            "--target-timeout",
        ),
    ];
    for (arguments, named) in cases {
        let ran = conform_run(dir.path(), &[suite.clone(), arguments.clone()].concat());
        assert_eq!(ran.code(), 2, "{arguments:?}: {}", ran.stderr());
        assert!(ran.report.is_none(), "{arguments:?} wrote a report");
        assert!(
            ran.stderr().contains(named),
            "{arguments:?}: the refusal names {named}: {}",
            ran.stderr()
        );
    }
}

#[test]
fn mutate_refuses_the_process_target_naming_the_split_route() {
    let mutate = Command::new(ESS)
        .current_dir(root())
        .args([
            "verify",
            "conform",
            "mutate",
            "--path",
            "examples/billing",
            "--target",
            "process",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&mutate.stderr);
    assert_eq!(mutate.status.code(), Some(2), "{stderr}");
    for route in ["--emit", "--collect", "run --target process"] {
        assert!(stderr.contains(route), "mutate names {route}: {stderr}");
    }
}

#[test]
fn a_relative_program_is_found_in_the_target_dir() {
    let dir = tempfile::tempdir().unwrap();
    let implementation = dir.path().join("implementation");
    fs::create_dir(&implementation).unwrap();
    std::os::unix::fs::symlink(ESS, implementation.join("adapter")).unwrap();
    let mut arguments = os(&["--suite", SUITE, "--target", "process", "--target-dir"]);
    arguments.push(implementation.into());
    arguments.extend(os(&[
        "--",
        "./adapter",
        "verify",
        "conform",
        "serve",
        "--target",
        "billing",
    ]));
    let ran = conform_run(dir.path(), &arguments);
    assert_eq!(ran.code(), 0, "{}", ran.stderr());
}

#[test]
fn serve_ends_at_end_of_input_and_refuses_a_line_by_kind_only() {
    let serve = |input: &str| {
        let mut child = Command::new(ESS)
            .args(["verify", "conform", "serve", "--target", "billing"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        std::io::Write::write_all(child.stdin.as_mut().unwrap(), input.as_bytes()).unwrap();
        drop(child.stdin.take());
        child.wait_with_output().unwrap()
    };
    let ended = serve("");
    assert_eq!(ended.status.code(), Some(0));
    assert_eq!(ended.stdout, b"");

    let handshake = serve("{\"id\":0,\"op\":\"handshake\",\"limit_ms\":1000,\"request\":{}}\n");
    assert_eq!(handshake.status.code(), Some(0));
    let answer: Value = serde_json::from_slice(&handshake.stdout).unwrap();
    assert_eq!(answer["ok"]["speaks"], "ess-conformance-target/1");

    let refused = serve("not a request: marker-7f3a\n");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("not-json"), "{stderr}");
    assert!(!stderr.contains("marker-7f3a"), "{stderr}");
}
