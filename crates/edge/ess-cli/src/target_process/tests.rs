//! The bounded reap a signalled `ess` waits on (`reap_within`), against a real child.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal};

use super::reap_within;

/// A child that runs until it is killed, and its pid.
fn sleeper() -> (std::process::Child, Pid) {
    let child = Command::new("sleep")
        .arg("1000")
        .stdin(Stdio::null())
        .spawn()
        .expect("sleep starts");
    let pid = Pid::from_child(&child);
    (child, pid)
}

/// A child that does not exit is waited for no longer than the limit, and is not reaped: the
/// signal path ends `ess` anyway rather than wait on a leader it cannot reap.
#[test]
fn a_child_that_does_not_exit_is_waited_for_no_longer_than_the_limit() {
    let (mut child, pid) = sleeper();
    let started = Instant::now();
    let reaped = reap_within(pid, Duration::from_millis(300));
    let waited = started.elapsed();
    let _ = child.kill();
    let _ = child.wait();
    assert!(!reaped, "a running child is not reaped");
    assert!(waited >= Duration::from_millis(300), "{waited:?}");
    assert!(waited < Duration::from_secs(5), "{waited:?}");
}

/// A child that is killed is reaped as soon as it has exited, long before the limit.
#[test]
fn a_killed_child_is_reaped_before_the_limit() {
    let (child, pid) = sleeper();
    rustix::process::kill_process(pid, Signal::KILL).expect("the child is killed");
    let started = Instant::now();
    let reaped = reap_within(pid, Duration::from_secs(30));
    let waited = started.elapsed();
    assert!(reaped, "a killed child is reaped");
    assert!(waited < Duration::from_secs(5), "{waited:?}");
    // Reaped already: dropping the handle waits for nothing.
    drop(child);
}
