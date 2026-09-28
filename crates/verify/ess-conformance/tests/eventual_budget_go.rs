//! The Go runtime gives an eventually step the budget the Rust runner gives it (beyond10x/ess#188,
//! adversary correction 2).
//!
//! The Rust runner asks until its clock passes `RunnerConfig::DEFAULT_EVENTUAL_TIMEOUT_MS`, and the
//! clock advances `AdvancingClock::DEFAULT_STEP_MS` per read. The Go harness computes its attempts
//! from two constants that restate those values. This test reads the two Go constants from
//! `src/go/runtime.go` and fails when either one drifts from the Rust constant it restates.

use ess_conformance::{AdvancingClock, RunnerConfig};

fn go_constant(name: &str) -> u64 {
    let source = include_str!("../src/go/runtime.go");
    source
        .lines()
        .find_map(|line| {
            let (key, value) = line.trim().split_once('=')?;
            (key.trim() == name).then(|| value.trim().parse().ok())?
        })
        .unwrap_or_else(|| panic!("src/go/runtime.go declares `{name} = <integer>`"))
}

#[test]
fn the_go_eventual_budget_is_the_rust_runners() {
    assert_eq!(
        go_constant("eventualTimeoutMillis"),
        RunnerConfig::DEFAULT_EVENTUAL_TIMEOUT_MS
    );
    assert_eq!(go_constant("stepMillis"), AdvancingClock::DEFAULT_STEP_MS);
    assert!(
        include_str!("../src/go/runtime.go")
            .contains("attempts: eventualTimeoutMillis / stepMillis"),
        "the harness derives its attempts from the two constants"
    );
}
