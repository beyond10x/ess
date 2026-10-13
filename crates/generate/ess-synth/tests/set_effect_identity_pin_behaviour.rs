//! Generated behaviour executes an `affects:` entry whose filter pins the row's identity (ess/25,
//! beyond10x/ess#547, `docs/design/set-effect-identity-filters.md`, D5).
//!
//! `ExpireEntry` expires a queue entry and ends the call the entry names by its stored `call_id`:
//! `affects: [{entity: Call, where: call_id == subject.call_id, moves: Call.end}]`. The generated
//! behaviour reads that one row by identity through the call's storage port, before any write, and
//! moves it where it rests in `end`'s `from` states. The model is emitted, built as its generated
//! network entry point with no hand-written code, and driven over a real socket through the suite
//! the same model synthesizes: the call the entry names ends, a decoy call is left, a parked call
//! is left, and a reference naming no call creates none. A faulty control that reads no row fails
//! the suite. A filter that does not pin the identity keeps the command owed, named.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as related;

use std::path::PathBuf;

use ess_compiler::ir::EssIr;
use ess_conformance::report::Status;
use ess_conformance::{ConformanceScenario, ScenarioStep};
use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const FIXTURE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/set-effect-identity-pin.yaml");

const EXPIRE: &str = "demo.q.ExpireEntry";
const EXPIRED: &str = "demo.q.ExpireEntry/outcome/expired";

/// The conformance fixture with UUID identities, which the generated context mints, served by one
/// network component accepting every command.
fn text() -> String {
    let from = "of: String}\n";
    assert_eq!(FIXTURE.matches(from).count(), 2, "two identity types");
    related::served(&FIXTURE.replace(from, "of: Uuid}\n"), "queue")
}

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from:?}");
    text.replacen(from, to, 1)
}

/// The emitted tree and built entry point of `text` for `target`, with `faulty` edited into the
/// emitted behaviour before the build, where given.
fn built(
    text: &str,
    target: Target,
    case: &str,
    faulty: Option<(&str, &str)>,
) -> (EssIr, PathBuf, PathBuf) {
    let ir = related::model(text);
    let (root, synthesis) = related::emit(&ir, target, case);
    for command in ir.commands().keys() {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, &command.to_string()),
            Some(&SynthesisDisposition::Generated),
            "{case}: `{command}` is generated:\n{}",
            synthesis.plan.to_markdown()
        );
    }
    let behaviour = related::behaviour_path(&root, target, "demo");
    if let Some((from, to)) = faulty {
        related::mutate(&behaviour, from, to);
    }
    let binary = related::build(&root, target, "demo", "queue", false);
    (ir, root, binary)
}

/// The synthesized scenario of the pinned branch.
fn expired(ir: &EssIr) -> ConformanceScenario {
    ess_conformance::synthesize(ir)
        .suite
        .scenarios
        .into_iter()
        .find(|(id, _)| id.to_string() == EXPIRED)
        .map(|(_, scenario)| scenario)
        .expect("the pinned branch is witnessed")
}

fn sends(scenario: &ConformanceScenario, command: &str) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. }
                if sent.to_string() == command)
        })
        .count()
}

/// The suite against the generated entry point of `target`: every scenario passes, the pinned
/// branch's among them, and a control reading no row fails it.
fn moves_the_row_the_subject_names(target: Target, label: &str, faulty: (&str, &str)) {
    let text = text();
    let (ir, root, binary) = built(&text, target, label, None);
    let statuses = related::run(&ir, &binary);
    related::assert_passes(label, &statuses, &[EXPIRED]);
    let _ = std::fs::remove_dir_all(&root);

    let (ir, root, binary) = built(&text, target, &format!("{label}-faulty"), Some(faulty));
    let statuses = related::run(&ir, &binary);
    assert_eq!(
        statuses.get(EXPIRED),
        Some(&Status::Failed),
        "{label}: a behaviour reading no row fails the pinned branch: {statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The pinned branch's scenario carries what `witnessed` checks, and the generated entry point of
/// `target` passes it.
fn leaves_the_rows_the_effect_skips(
    target: Target,
    label: &str,
    witnessed: &dyn Fn(&ConformanceScenario),
) {
    let text = text();
    let (ir, root, binary) = built(&text, target, label, None);
    witnessed(&expired(&ir));
    let statuses = related::run(&ir, &binary);
    assert_eq!(
        statuses.get(EXPIRED),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A call resting outside `end`'s `from` states, named by a send of its own.
fn names_a_parked_call(scenario: &ConformanceScenario) {
    assert!(sends(scenario, EXPIRE) >= 3, "{:#?}", scenario.steps);
    assert!(
        sends(scenario, "demo.q.ParkCall") >= 1,
        "{:#?}",
        scenario.steps
    );
}

/// An entry whose reference names an identity no call carries, sent, and that call read absent.
fn names_no_call(scenario: &ConformanceScenario) {
    let dangling = scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "demo.q.QueueEntry"
                && input.get("call_id").is_some_and(|value| value.as_literal().is_some()))
    });
    assert!(dangling, "{:#?}", scenario.steps);
    assert!(
        scenario.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectSubjectAbsent { view, .. } if view.to_string() == "demo.q.Calls"
        )),
        "the call a dangling reference names is read absent: {:#?}",
        scenario.steps
    );
}

/// The Rust edit that reads no row: the pinned identity is dropped before the lookup.
const RUST_READS_NO_ROW: (&str, &str) = (
    ".cloned().and_then(",
    ".cloned().filter(|_| false).and_then(",
);

#[test]
fn generated_rust_moves_the_row_the_subject_names() {
    moves_the_row_the_subject_names(Target::Rust, "pin-rust", RUST_READS_NO_ROW);
}

#[test]
fn generated_rust_skips_a_row_outside_from() {
    leaves_the_rows_the_effect_skips(Target::Rust, "pin-rust-from", &names_a_parked_call);
}

#[test]
fn generated_rust_changes_nothing_for_a_dangling_reference() {
    leaves_the_rows_the_effect_skips(Target::Rust, "pin-rust-dangling", &names_no_call);
}

#[test]
fn an_unpinned_entry_stays_owed_naming_it() {
    for (label, text) in [
        (
            "identity read other than by a pin",
            edited(
                &text(),
                "where: call_id == subject.call_id",
                "where: call_id != subject.call_id",
            ),
        ),
        (
            "a filter over a stored field",
            edited(
                &text(),
                "where: call_id == subject.call_id",
                "where: line == \"l\"",
            ),
        ),
    ] {
        let ir = related::model(&text);
        for target in [Target::Rust, Target::Go] {
            let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
            match synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, EXPIRE)
            {
                Some(SynthesisDisposition::Obligation(owed)) => assert!(
                    owed.reason.describes().contains("`affects:`"),
                    "{label}, {target:?}: {}",
                    owed.reason.describes()
                ),
                other => panic!("{label}, {target:?}: `{EXPIRE}` stays owed, not {other:?}"),
            }
        }
    }
}

/// The Go edit that reads no row: the lookup's found flag is ignored.
const GO_READS_NO_ROW: (&str, &str) = ("if affectedFound0 {", "if affectedFound0 && false {");

#[test]
fn generated_go_moves_the_row_the_subject_names() {
    if !related::go_available() {
        eprintln!("skipped: no Go toolchain");
        return;
    }
    moves_the_row_the_subject_names(Target::Go, "pin-go", GO_READS_NO_ROW);
}

#[test]
fn generated_go_skips_a_row_outside_from() {
    if !related::go_available() {
        eprintln!("skipped: no Go toolchain");
        return;
    }
    leaves_the_rows_the_effect_skips(Target::Go, "pin-go-from", &names_a_parked_call);
}

#[test]
fn generated_go_changes_nothing_for_a_dangling_reference() {
    if !related::go_available() {
        eprintln!("skipped: no Go toolchain");
        return;
    }
    leaves_the_rows_the_effect_skips(Target::Go, "pin-go-dangling", &names_no_call);
}
