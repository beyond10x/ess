//! Adversary pass 1 for wave 5 unit 4 (`story:synthesis-reads-selection-plan`), the cases left
//! open: each fails at the unit's base `related_guard.rs` as well as after it, so each is ignored
//! with the record it waits on. Run them with `--ignored`; every one fails.
//!
//! * On a command reading several rows (ess/22, beyond10x/ess#283) the plan reads a predicate
//!   refusal at step 5, before an accepting `when:` declared earlier, but its witness is not the
//!   plain one (A2, A2c).
//! * A stored-reference model synthesized and interpreted under exchanged phase orders (A3).
//!
//! A4, A4b and A4c (`CheckPick` in ess/22 with `wrong_state:` beside the external `unlisted`)
//! passed once story:related-guard-wrong-state-beside-external landed, and run un-ignored in
//! `adversary_selection_plan_w5_u4_pass1.rs`.
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
