//! Adversary pass 1 for wave 5 unit 4 (`story:synthesis-reads-selection-plan`): the
//! `when_related:` searches in `synthesize/related_guard.rs` take their order from the precedence
//! plan.
//!
//! * On the ess/22 `PublishRelease` model the unit's own test builds, every synthesized scenario,
//!   the `exists: false` one included, passes on the interpreter, in the real order and with the
//!   present-related and accepting phases exchanged.
//! * Every related-guard model of the unit's tests, synthesized and interpreted under each of
//!   several exchanged phase orders, passes on the interpreter reading the same order.
//!
//! The cases this pass left open are in `adversary_selection_plan_w5_u4_pass1_open.rs`, ignored.
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The unit test's `release("ess/22")`, rebuilt from the same fixture by the same edits.
fn release22() -> String {
    let text = include_str!("fixtures/related-guard-release.yaml")
        .replace("format: ess/20", "format: ess/22");
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}\n  - {name: demo.release.Speed, kind: enum, variants: [Fast, Slow]}\n",
    );
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
    );
    let text = replaced(
        &text,
        "      - {name: candidate, type: demo.release.CandidateId}\n    outcomes:\n",
        "      - {name: candidate, type: demo.release.CandidateId}\n      - {name: speed, type: demo.release.Speed}\n    outcomes:\n",
    );
    let text = replaced(
        &text,
        "      - name: published\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: published-slowly\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n",
    );
    replaced(
        &text,
        "      - name: not-accepted\n",
        "      - name: published\n        when: speed == Fast\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n      - name: not-accepted\n",
    )
}

/// The plain release fixture in ess/22 with only `wrong_state:` added: no `speed`, no extra
/// accepting branch.
fn release22_plain() -> String {
    let text = include_str!("fixtures/related-guard-release.yaml")
        .replace("format: ess/20", "format: ess/22");
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
    );
    replaced(
        &text,
        "      - name: published\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: published\n",
    )
}

const SWITCH_PAUSED_BRANCH: &str = "      - name: switch-paused\n        when_related: {via: input.switch, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n";

/// `related-guard-multiple.yaml` (ess/22, two rows) with a `speed` input and an accepting `rushed`
/// (`speed == Fast`) declared before the predicate refusal `switch-paused`.
fn multiple_rushed() -> String {
    let text = include_str!("fixtures/related-guard-multiple.yaml");
    let text = replaced(
        text,
        "  - {name: demo.run.RunId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.run.RunId, kind: newtype, of: Uuid}\n  - {name: demo.run.Speed, kind: enum, variants: [Fast, Slow]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: capability, type: demo.run.CapabilityId}\n    outcomes:\n",
        "      - {name: capability, type: demo.run.CapabilityId}\n      - {name: speed, type: demo.run.Speed}\n    outcomes:\n",
    );
    replaced(
        &text,
        SWITCH_PAUSED_BRANCH,
        &format!(
            "      - name: rushed\n        when: speed == Fast\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {{demo.run.RunStarted: {{run_id: {{generated: true}}}}}}\n{SWITCH_PAUSED_BRANCH}"
        ),
    )
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                let refusals: Vec<String> =
                    result.refusals.iter().map(ToString::to_string).collect();
                panic!("no scenario {id}; refusals: {refusals:#?}")
            },
            |(_, scenario)| scenario,
        )
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, (Status, Vec<String>)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            (result.scenario.to_string(), (result.status, diagnostics))
        })
        .collect();
    results
        .into_iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, diagnostics))| format!("{id}: {status:?} {diagnostics:?}"))
        .collect()
}

/// A1: the ess/22 model of the unit's own test. Every synthesized scenario, the `exists: false`
/// one included, passes on the interpreter. The unit's test drops `no-candidate`'s failure by its
/// message; this asserts it.
#[test]
fn a1_release_ess22_every_synthesized_scenario_passes_including_no_candidate() {
    let model = ir(&release22());
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

/// A1b: the same defect without the unit's `speed` edits: the plain release fixture at ess/22
/// beside `wrong_state:`.
#[test]
fn a1b_release_ess22_plain_every_synthesized_scenario_passes() {
    let model = ir(&release22_plain());
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

fn swap(a: Phase, b: Phase) -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| {
        if phase == a {
            b
        } else if phase == b {
            a
        } else {
            phase
        }
    })
}

fn orders() -> Vec<(String, [Phase; 8])> {
    use Phase::*;
    [
        (InputRefusal, Existence),
        (Existence, HeldState),
        (HeldState, PresentRelated),
        (PresentRelated, Accepting),
        (InputRefusal, PresentRelated),
        (RelatedRow, InputRefusal),
        (InputRefusal, Accepting),
    ]
    .into_iter()
    .map(|(a, b)| (format!("{a}<->{b}"), swap(a, b)))
    .collect()
}

/// Every scenario synthesized under each exchanged order, run on the interpreter reading that
/// order; panics are reported as such. The known `no-candidate` defect (A1) is kept out here so
/// that this case speaks only to the exchange.
fn exchanged_failures(text: &str) -> Vec<String> {
    let model = ir(text);
    let mut failures = Vec::new();
    for (name, order) in orders() {
        let run = catch_unwind(AssertUnwindSafe(|| {
            with_phase_order(order, || {
                let result = ess_conformance::synthesize::synthesize(&model);
                not_passed(model.clone(), &result.suite)
            })
        }));
        match run {
            Ok(failed) => failures.extend(
                failed
                    .into_iter()
                    .filter(|failure| {
                        !(failure
                            .starts_with("demo.release.PublishRelease/outcome/no-candidate: Error")
                            && failure.contains("no earlier step bound the instance `release`"))
                    })
                    .map(|failure| format!("[{name}] {failure}")),
            ),
            Err(_) => failures.push(format!("[{name}] panicked")),
        }
    }
    failures
}

#[test]
fn a3_exchanged_orders_release_ess22() {
    assert_eq!(exchanged_failures(&release22()), Vec::<String>::new());
}

#[test]
fn a3_exchanged_orders_several_rows() {
    assert_eq!(
        exchanged_failures(include_str!("fixtures/related-guard-multiple.yaml")),
        Vec::<String>::new()
    );
}

#[test]
fn a3_exchanged_orders_several_rows_rushed() {
    assert_eq!(exchanged_failures(&multiple_rushed()), Vec::<String>::new());
}

#[test]
fn a3_exchanged_orders_sign_in() {
    assert_eq!(
        exchanged_failures(include_str!("fixtures/related-guard-sign-in.yaml")),
        Vec::<String>::new()
    );
}

/// A1c: `no-candidate` is synthesized on the unit's ess/22 model in the real order, and passes:
/// the failure the unit's `ess22_…` test drops by its message does not occur there, so that
/// filter only hides a regression.
#[test]
fn a1c_release_ess22_no_candidate_is_synthesized_and_passes_in_the_real_order() {
    let model = ir(&release22());
    let result = ess_conformance::synthesize::synthesize(&model);
    scenario(&result, "demo.release.PublishRelease/outcome/no-candidate");
    let failed: Vec<String> = not_passed(model, &result.suite)
        .into_iter()
        .filter(|failure| failure.starts_with("demo.release.PublishRelease/outcome/no-candidate"))
        .collect();
    assert_eq!(failed, Vec::<String>::new());
}

/// A1d: with present-related and accepting exchanged (the unit's own exchange), `no-candidate`
/// is synthesized and passes on the interpreter reading the same order.
#[test]
fn a1d_release_ess22_no_candidate_passes_with_present_related_and_accepting_exchanged() {
    let model = ir(&release22());
    let failed = with_phase_order(swap(Phase::PresentRelated, Phase::Accepting), || {
        let result = ess_conformance::synthesize::synthesize(&model);
        scenario(&result, "demo.release.PublishRelease/outcome/no-candidate");
        not_passed(model.clone(), &result.suite)
    });
    let failed: Vec<String> = failed
        .into_iter()
        .filter(|failure| failure.starts_with("demo.release.PublishRelease/outcome/no-candidate"))
        .collect();
    assert_eq!(failed, Vec::<String>::new());
}

/// `synthesis_related_guard_plan.rs`'s `DESK`, copied verbatim.
const DESK: &str = r"format: ess/20
system: demo
version: v1
domain: demo.desk
summary: A pick checked against its list's revision before it is accepted.
types:
  - {name: demo.desk.PickId, kind: newtype, of: Uuid}
  - {name: demo.desk.ListId, kind: newtype, of: Uuid}
entities:
  - name: demo.desk.List
    identity: {name: list_id, type: demo.desk.ListId}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.desk.Pick
    identity: {name: pick_id, type: demo.desk.PickId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Picked
      states: [Picked, Accepted, Refused]
      terminal: [Accepted, Refused]
      transitions:
        - {name: accept, from: [Picked], to: Accepted}
        - {name: refuse, from: [Picked], to: Refused}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.OpenList, demo.desk.MakePick, demo.desk.CheckPick]
commands:
  - name: demo.desk.OpenList
    input:
      - {name: revision, type: Integer}
    outcomes:
      - name: opened
        creates: demo.desk.List
        instance: list_id
        sets: {revision: input.revision}
        emits: [demo.desk.ListOpened]
        payload:
          demo.desk.ListOpened: {list_id: {generated: true}}
  - name: demo.desk.MakePick
    input:
      - {name: note, type: String}
    outcomes:
      - name: picked
        creates: demo.desk.Pick
        instance: pick_id
        sets: {note: input.note}
        emits: [demo.desk.Picked]
        payload:
          demo.desk.Picked: {pick_id: {generated: true}}
  - name: demo.desk.CheckPick
    input:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: list_id, type: demo.desk.ListId}
      - {name: revision, type: Integer}
    outcomes:
      - name: no-list
        when_related: {via: input.list_id, exists: false}
        error: demo.desk.NoList
      - name: stale
        when_related:
          via: input.list_id
          predicate: revision != input.revision
        error: demo.desk.Stale
      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
      - name: accepted
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
events:
  - name: demo.desk.ListOpened
    fields: [{name: list_id, type: demo.desk.ListId}]
  - name: demo.desk.Picked
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickUnlisted
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickAccepted
    fields: [{name: pick_id, type: demo.desk.PickId}]
errors:
  - name: demo.desk.NotPicked
  - name: demo.desk.NoList
  - name: demo.desk.Stale
views:
  - name: demo.desk.Picks
    source: demo.desk.Pick
    consistency: read_your_writes
    fields:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: note, type: String}
      - {name: state, type: demo.desk.Pick.State}
  - name: demo.desk.Lists
    source: demo.desk.List
    consistency: read_your_writes
    fields:
      - {name: list_id, type: demo.desk.ListId}
      - {name: revision, type: Integer}
";

/// [`DESK`] in ess/22 with `wrong_state:`, `stale` left declared before `unlisted`.
fn desk22_with_wrong_state() -> String {
    let text = DESK.replace("format: ess/20", "format: ess/22");
    replaced(
        &text,
        "      - name: accepted\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.desk.NotPicked}\n      - name: accepted\n",
    )
}

/// A4d: [`desk22_with_wrong_state`] with `unlisted` an accepting `when:` instead of external: every scenario passes, so the
/// external branch is what A4b and A4c turn on.
#[test]
fn a4d_desk_ess22_wrong_state_without_the_external_passes() {
    let text = replaced(
        &desk22_with_wrong_state(),
        "      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
",
        "      - name: unlisted\n        when: revision == 0\n        moves: demo.desk.Pick.refuse\n        instance: pick_id\n        emits: [demo.desk.PickUnlisted]\n        payload:\n          demo.desk.PickUnlisted: {pick_id: input.pick_id}\n",
    );
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}
