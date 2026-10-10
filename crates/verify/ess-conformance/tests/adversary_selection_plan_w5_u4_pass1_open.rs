//! Adversary pass 1 for wave 5 unit 4 (`story:synthesis-reads-selection-plan`), the cases left
//! open: each fails at the unit's base `related_guard.rs` as well as after it, so each is ignored
//! with the record it waits on. Run them with `--ignored`; every one fails.
//!
//! * On a command reading several rows (ess/22, beyond10x/ess#283) the plan reads a predicate
//!   refusal at step 5, before an accepting `when:` declared earlier, but its witness is not the
//!   plain one (A2, A2c).
//! * A stored-reference model synthesized and interpreted under exchanged phase orders (A3).
//! * `CheckPick` in ess/22 with `wrong_state:` beside the external `unlisted` (A4, A4b, A4c).
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

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

fn last_sent(scenario: &ConformanceScenario, command: &str, field: &str) -> ScenarioValue {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.name().to_string() == command => Some(input[field].clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no send of {command}: {scenario:#?}"))
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

/// A2: several rows (ess/22, #283). The plan reads `switch-paused` at step 5, before `rushed`;
/// a paused switch sent `Fast` is answered `switch-paused` by the interpreter. Its witness should
/// be the plain one, `Fast`, as the unit's ess/22 single-row test asserts for `not-accepted`.
#[test]
#[ignore = "pre-existing; story:related-guard-wrong-state-beside-external"]
fn a2_several_rows_the_step5_refusal_witness_is_the_plain_one() {
    let model = ir(&multiple_rushed());
    let plan = ess_compiler::ir::PrecedencePlan::new(
        model
            .commands()
            .values()
            .find(|command| command.name.to_string() == "demo.run.StartRun")
            .expect("StartRun"),
        model.format(),
    );
    let at = |name: &str| {
        plan.iter()
            .find(|(_, branch)| branch.name.to_string() == name)
            .map(|(phase, _)| phase)
    };
    assert_eq!(at("switch-paused"), Some(Phase::PresentRelated));
    assert_eq!(at("rushed"), Some(Phase::Accepting));
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
    assert_eq!(
        last_sent(
            scenario(&result, "demo.run.StartRun/outcome/switch-paused"),
            "demo.run.StartRun",
            "speed"
        ),
        ScenarioValue::literal(Node::Text("Fast".into())),
        "the plan reads `switch-paused` before `rushed`; its witness is the plain first variant"
    );
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
#[ignore = "test seam only; review-result:selection-plan-w5-u4-adversary-pass-1"]
fn a3_exchanged_orders_stored_reference() {
    assert_eq!(
        exchanged_failures(include_str!("fixtures/related-guard-stored-reference.yaml")),
        Vec::<String>::new()
    );
}

/// A2c: the mechanism behind A2. `related_guard.rs`'s `projection` keeps the branches over one row;
/// the plan of that projection reads `switch-paused` among the accepting branches, after `rushed`,
/// where the command's own plan reads it at step 5. The searches in `drive_several` and the
/// overlap search ask `selects` with the projection, so they take the projection's order.
#[test]
#[ignore = "test seam only; review-result:selection-plan-w5-u4-adversary-pass-1"]
fn a2c_the_projection_over_one_row_moves_the_refusal_out_of_step5() {
    use ess_compiler::ir::{PrecedencePlan, ResolvedCondition};
    let model = ir(&multiple_rushed());
    let command = model
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.run.StartRun")
        .expect("StartRun");
    let mut projected = command.clone();
    projected
        .outcomes
        .retain(|outcome| match &outcome.condition {
            ResolvedCondition::Related { via, .. } => via.field() == "switch",
            _ => true,
        });
    let phase = |plan: &PrecedencePlan<'_>, name: &str| {
        plan.iter()
            .find(|(_, branch)| branch.name.to_string() == name)
            .map(|(phase, _)| phase)
    };
    let whole = PrecedencePlan::new(command, model.format());
    let one_row = PrecedencePlan::new(&projected, model.format());
    assert_eq!(
        (
            phase(&whole, "switch-paused"),
            phase(&one_row, "switch-paused")
        ),
        (Some(Phase::PresentRelated), Some(Phase::PresentRelated)),
        "a search over the projection reads the refusal where the command reads it"
    );
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

const STALE: &str = "      - name: stale
        when_related:
          via: input.list_id
          predicate: revision != input.revision
        error: demo.desk.Stale
";

/// [`DESK`] in ess/22 with `wrong_state:` and `stale` declared after the external `unlisted`: the
/// plan reads `stale` at step 5, before `unlisted`, so the forced `unlisted` witness must refute it.
fn desk22_stale_after_with_wrong_state() -> String {
    let text = DESK.replace("format: ess/20", "format: ess/22");
    let text = replaced(&text, STALE, "");
    let text = replaced(
        &text,
        "      - name: accepted\n",
        &format!("{STALE}      - {{name: wrong-state, wrong_state: true, error: demo.desk.NotPicked}}\n      - name: accepted\n"),
    );
    assert!(text.find("name: unlisted") < text.find("name: stale"));
    text
}

/// A4: every scenario of that model passes on the interpreter, in the real order and under each
/// exchanged order.
#[test]
#[ignore = "pre-existing; story:related-guard-wrong-state-beside-external"]
fn a4_desk_ess22_stale_after_external_beside_wrong_state_passes() {
    let model = ir(&desk22_stale_after_with_wrong_state());
    let result = ess_conformance::synthesize::synthesize(&model);
    scenario(&result, "demo.desk.CheckPick/outcome/unlisted");
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
    assert_eq!(
        exchanged_failures(&desk22_stale_after_with_wrong_state()),
        Vec::<String>::new()
    );
}

/// [`DESK`] in ess/22 with `wrong_state:`, `stale` left declared before `unlisted`.
fn desk22_with_wrong_state() -> String {
    let text = DESK.replace("format: ess/20", "format: ess/22");
    replaced(
        &text,
        "      - name: accepted\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.desk.NotPicked}\n      - name: accepted\n",
    )
}

/// A4b: the same with `stale` declared before the external: every scenario passes.
#[test]
#[ignore = "pre-existing; story:related-guard-wrong-state-beside-external"]
fn a4b_desk_ess22_stale_before_external_beside_wrong_state_passes() {
    let model = ir(&desk22_with_wrong_state());
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

/// A4c: the `wrong-state` scenario of [`desk22_with_wrong_state`] names, in its last
/// `CheckPick` send, a list an earlier step bound: a `list_id` no row carries is answered
/// `no-list` at step 1, before the held state.
#[test]
#[ignore = "pre-existing; story:related-guard-wrong-state-beside-external"]
fn a4c_desk_ess22_wrong_state_send_names_an_arranged_list() {
    let model = ir(&desk22_with_wrong_state());
    let result = ess_conformance::synthesize::synthesize(&model);
    let wrong = scenario(&result, "demo.desk.CheckPick/outcome/wrong-state");
    let sent = last_sent(wrong, "demo.desk.CheckPick", "list_id");
    assert!(
        matches!(sent, ScenarioValue::Instance { .. }),
        "list_id sent as {sent:?}; steps: {:#?}",
        wrong.steps
    );
}
