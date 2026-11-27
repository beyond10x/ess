//! The generated Rust dispatch of a refusal-selected failure policy (ess/22, beyond10x/ess#269)
//! and of a plain bounded retry (ess/16, beyond10x/ess#165), run.
//!
//! `tests/fixtures/refusal-policy-harness/model.yaml.in` is the fixture of `refusal_policy.rs`
//! with one component and two plainly bounded bindings added, one of them reading its input
//! through an owed transformation. The generated system is not built
//! here: a test that runs cargo compiles a crate graph inside its lane
//! (`crates/edge/ess-xtask/tests/no_test_builds.rs`). `task refusal-policy-harness` synthesizes
//! the model with `ess generate synthesize`, links the generated system with the scripted command
//! ports of `tests/fixtures/refusal-policy-harness/main.rs` and builds the binary once; this test
//! runs the binary [`HARNESS`] names. Without it every case fails, naming the task.
//!
//! The harness reads one occurrence per line, `place <order> <answer>…` (the `notify-ledger`
//! binding, through `Record`), `close <order> <answer>…` (`archive-order`, through `Archive`) or
//! `audit <order> <answer>…` (`note-order`, through `Note`, its input read by an owed
//! transformation that fails for an order whose id starts with `noinput`): the answers the command
//! port gives that order's attempts, in order, `unmet` for an unmet obligation. It runs the
//! triggering command, pumps until a pump makes no attempt and asks no transformation, and prints
//! one line per attempt — command, order, attempt number, answer, the policy applied and the events
//! the pump of that attempt published — for `audit` the line `inputs <order> <asked> reported
//! <failed pumps>`, then `end <order>`. Nothing here is the conformance suite: no
//! generated-Rust runner exists, so these are the verdicts the suite's
//! `notify-ledger/binding/refusal/<outcome>` scenarios state for the fixture, checked directly.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const HARNESS: &str = "ESS_REFUSAL_POLICY_HARNESS";
const RECORD: &str = "demo.ledger.Record";
const ARCHIVE: &str = "demo.ledger.Archive";
const RECORDED: &str = "demo.ledger.Recorded";
const ESCALATED: &str = "demo.ledger.RecordEscalated";
const ARCHIVED: &str = "demo.ledger.Archived";
const NOTE: &str = "demo.ledger.Note";
const NOTED: &str = "demo.ledger.Noted";

/// The harness `task refusal-policy-harness` built against the generated system, from the path
/// [`HARNESS`] names.
///
/// A file that is not executable is made so: an uploaded CI artifact loses its mode bits, and the
/// test shards receive the harness beside the nextest archives.
fn harness() -> PathBuf {
    let Some(path) = std::env::var_os(HARNESS).map(PathBuf::from) else {
        panic!(
            "{HARNESS} is not set: run `task refusal-policy-harness` and set it to the binary \
             that task names, or run the tests through `task test`, which does both"
        );
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "{HARNESS}={} names no harness ({error}): run `task refusal-policy-harness`",
            path.display()
        )
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions).expect("the harness is made executable");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    path
}

/// Every line the harness prints for `script`, one occurrence per line, all in one process, so a
/// count one occurrence leaves behind would show in the next.
fn run(script: &[&str]) -> Vec<String> {
    let mut child = Command::new(harness())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the harness starts");
    {
        let mut stdin = child.stdin.take().expect("piped");
        for line in script {
            writeln!(stdin, "{line}").expect("the harness reads its script");
        }
    }
    let output = child.wait_with_output().expect("the harness ends");
    assert!(
        output.status.success(),
        "the harness fails: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("the harness prints UTF-8")
        .lines()
        .map(str::to_owned)
        .collect()
}

/// One attempt's line: `attempt <command> <order> <n> <answer> <policy> <published>`.
fn attempt(
    command: &str,
    order: &str,
    n: u32,
    answer: &str,
    policy: &str,
    published: &[&str],
) -> String {
    let published = if published.is_empty() {
        "-".to_owned()
    } else {
        published.join(",")
    };
    format!("attempt {command} {order} {n} {answer} {policy} {published}")
}

fn end(order: &str) -> String {
    format!("end {order}")
}

#[test]
fn rust_dispatch_selects_the_policy_per_refusal() {
    let lines = run(&[
        "place o1 wrong-state",
        "place o2 busy recorded",
        "place o3 unavailable recorded",
        "place o4 rejected recorded",
        "place o5 at-limit recorded",
        "place o6 unmet recorded",
    ]);
    let expected = vec![
        // `drop: [wrong-state]`: given up at once, nothing published.
        attempt(RECORD, "o1", 1, "wrong-state", "drop", &[]),
        end("o1"),
        // `retry` selects both refusals reporting `demo.ledger.Unavailable`.
        attempt(RECORD, "o2", 1, "busy", "retry", &[]),
        attempt(RECORD, "o2", 2, "recorded", "accepted", &[RECORDED]),
        end("o2"),
        attempt(RECORD, "o3", 1, "unavailable", "retry", &[]),
        attempt(RECORD, "o3", 2, "recorded", "accepted", &[RECORDED]),
        end("o3"),
        // `rejected` is retried and final: it ends the retry at its first answer, dropped, never
        // rerouted to the escalation.
        attempt(RECORD, "o4", 1, "rejected", "drop", &[]),
        end("o4"),
        // `at-limit`, which no positive policy names, takes the fallback: one escalation.
        attempt(RECORD, "o5", 1, "at-limit", "escalate", &[ESCALATED]),
        end("o5"),
        // An unmet obligation of the port propagates out of the pump before any policy and
        // publishes nothing; `at_least_once` attempts the occurrence again.
        attempt(RECORD, "o6", 1, "unmet", "unmet", &[]),
        attempt(RECORD, "o6", 2, "recorded", "accepted", &[RECORDED]),
        end("o6"),
    ];
    assert_eq!(lines, expected);
}

#[test]
fn rust_dispatch_counts_attempts_of_one_occurrence() {
    let lines = run(&[
        "place o1 unavailable busy unavailable recorded",
        "place o2 busy busy recorded",
        "place o3 busy rejected recorded",
    ]);
    let expected = vec![
        // The third answer that selects the retry reaches the bound of 3 attempts in all, across
        // the two outcomes that select it: the occurrence is dropped, nothing published, and the
        // scripted `recorded` is never asked for.
        attempt(RECORD, "o1", 1, "unavailable", "retry", &[]),
        attempt(RECORD, "o1", 2, "busy", "retry", &[]),
        attempt(RECORD, "o1", 3, "unavailable", "drop", &[]),
        end("o1"),
        // A second occurrence starts at attempt 1, so it has three attempts of its own.
        attempt(RECORD, "o2", 1, "busy", "retry", &[]),
        attempt(RECORD, "o2", 2, "busy", "retry", &[]),
        attempt(RECORD, "o2", 3, "recorded", "accepted", &[RECORDED]),
        end("o2"),
        // A final refusal after a retried one ends the retry at once, without an escalation.
        attempt(RECORD, "o3", 1, "busy", "retry", &[]),
        attempt(RECORD, "o3", 2, "rejected", "drop", &[]),
        end("o3"),
    ];
    assert_eq!(lines, expected);
}

#[test]
fn rust_dispatch_stops_a_plain_bounded_retry() {
    let lines = run(&[
        "close c1 unavailable unavailable archived",
        "close c2 rejected archived",
        "close c3 unavailable archived",
    ]);
    let expected = vec![
        // `retry: {attempts: 2, …}`: the second refusal is the last attempt, dropped.
        attempt(ARCHIVE, "c1", 1, "unavailable", "retry", &[]),
        attempt(ARCHIVE, "c1", 2, "unavailable", "drop", &[]),
        end("c1"),
        // `final: [demo.ledger.Unknown]` stops at once.
        attempt(ARCHIVE, "c2", 1, "rejected", "drop", &[]),
        end("c2"),
        attempt(ARCHIVE, "c3", 1, "unavailable", "retry", &[]),
        attempt(ARCHIVE, "c3", 2, "archived", "accepted", &[ARCHIVED]),
        end("c3"),
    ];
    assert_eq!(lines, expected);
}

/// A failure the dispatch does not hold again is reported out of `pump`, never dropped: an unmet
/// obligation of the port at the bound, and a failure before the port is invoked, which makes no
/// attempt and is not held either. The next occurrence of the same binding is still delivered.
#[test]
fn rust_dispatch_reports_what_it_does_not_hold() {
    let lines = run(&[
        "close c4 unmet unmet archived",
        "close c5 archived",
        "audit noinput1",
        "audit a1 unavailable noted",
    ]);
    let expected = vec![
        // Each invocation counts against `attempts: 2`: the first unmet obligation is held, the
        // second is the bound, held by nothing, and fails the pump that made it.
        attempt(ARCHIVE, "c4", 1, "unmet", "unmet", &[]),
        attempt(ARCHIVE, "c4", 2, "unmet", "unmet", &[]),
        end("c4"),
        attempt(ARCHIVE, "c5", 1, "archived", "accepted", &[ARCHIVED]),
        end("c5"),
        // The owed transformation fails: the port is never invoked, the transformation is asked
        // once, and the one pump that asked it fails.
        "inputs noinput1 1 reported 1".to_owned(),
        end("noinput1"),
        attempt(NOTE, "a1", 1, "unavailable", "retry", &[]),
        attempt(NOTE, "a1", 2, "noted", "accepted", &[NOTED]),
        "inputs a1 2 reported 0".to_owned(),
        end("a1"),
    ];
    assert_eq!(lines, expected);
}
