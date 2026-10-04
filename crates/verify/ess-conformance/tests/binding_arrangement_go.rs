//! The generated Go runtime gives the reference runner's verdict on every binding-arrangement
//! control (beyond10x/ess#266, beyond10x/ess#267, `docs/design/binding-arrangement-and-drop.md`).
//!
//! Each target — the interpreter's own dispatcher, or that dispatcher with one thing wrong — runs
//! the synthesized suite once under the reference runner, recorded, and the recording is replayed to
//! the emitted Go package. The verdicts must agree scenario by scenario, every request the Go
//! runtime sends must be one the reference runner sent, and the faulty targets must fail exactly the
//! scenarios their fault is about.

mod support_binding_arrangement;
mod support_go;

use std::collections::BTreeMap;

use ess_conformance::synthesize::synthesize;
use ess_conformance::ConformanceSuite;
use support_binding_arrangement::*;

fn suite() -> ConformanceSuite {
    synthesize(&model(FIXTURE)).suite
}

fn failed(verdicts: &BTreeMap<String, String>) -> Vec<(&str, &str)> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, status)| (id.as_str(), status.as_str()))
        .collect()
}

#[test]
fn go_passes_the_honest_binding_running_target() {
    let verdicts = support_go::assert_parity("binding-honest", &suite(), interpreted(FIXTURE));
    assert_eq!(failed(&verdicts), Vec::<(&str, &str)>::new());
}

#[test]
fn go_fails_a_dispatcher_without_the_binding_and_a_destination_left_ineligible() {
    let suite = suite();
    let disabled = support_go::assert_parity(
        "binding-disabled",
        &suite,
        interpreted(&without_created_starts()),
    );
    let disabled = failed(&disabled);
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/mapping",
    ] {
        assert!(disabled.contains(&(id, "failed")), "{id}: {disabled:?}");
    }
    let stopped = support_go::assert_parity(
        "binding-stopped",
        &suite,
        interpreted(&lands_stopped("jobs.job.Created")),
    );
    let stopped = failed(&stopped);
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/delivery",
    ] {
        assert!(stopped.contains(&(id, "failed")), "{id}: {stopped:?}");
    }
}

#[test]
fn go_fails_every_drop_control() {
    let suite = suite();
    for (label, verdicts) in [
        (
            "zero",
            support_go::assert_parity("drop-zero", &suite, interpreted(&without_kicked_starts())),
        ),
        (
            "retry",
            support_go::assert_parity("drop-retry", &suite, interpreted(&kicked_retries())),
        ),
        (
            "malformed",
            support_go::assert_parity(
                "drop-malformed",
                &suite,
                Faulty::new(FIXTURE, Fault::MalformedRetry),
            ),
        ),
        (
            "success",
            support_go::assert_parity(
                "drop-success",
                &suite,
                Faulty::new(FIXTURE, Fault::SwallowForce),
            ),
        ),
        (
            "late",
            support_go::assert_parity(
                "drop-late",
                &suite,
                Faulty::new(FIXTURE, Fault::LateRetry(90)),
            ),
        ),
    ] {
        assert_eq!(verdicts[DROP], "failed", "{label}: {:?}", failed(&verdicts));
    }
}
