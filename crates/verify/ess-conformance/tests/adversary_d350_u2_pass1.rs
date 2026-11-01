//! Adversary pass 1 over wave D350 unit U2 (story
//! `an-outcome-arrangement-follows-the-bindings-its-steps-set-off` and story
//! `related-guard-wrong-state-beside-external`).
//!
//! Story 6 cases vary the `restoring-binding` fixture: the binding set off by the arrangement
//! rewrites the guarded row's lifecycle state, removes it, or reaches it two hops down. Each
//! asserts the acceptance directly: no synthesized scenario fails against the interpreter of the
//! same model, and the branch is refused by name where no other row selects it.
//!
//! Story 7 cases vary the DESK pick model (`claim-search/related.yaml`) with `wrong_state:` beside
//! the external `unlisted`: with an accepting `when:` branch declared before the
//! external, and under exchanged phase orders, among them ones that move the related-row phase.
//! Without a declared `unknown_instance:` (rejected beside `when_related:` before ess/27), the
//! unknown-identity scenario is `CheckPick/outcome/wrong-state` itself.
use std::panic::{catch_unwind, AssertUnwindSafe};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const COMPONENTS: &str = include_str!("fixtures/adversary-0730/restoring-binding/components.yaml");
const DOMAIN: &str = include_str!("fixtures/adversary-0730/restoring-binding/domains/d.yaml");
const DESK20: &str = include_str!("fixtures/claim-search/related.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn system(domain: &str) -> String {
    format!("format: ess/25\nsystem: demo\nversion: v1\ndomains:\n  - {domain}\n")
}

fn model(files: &[(&str, &str)]) -> EssIr {
    let mut sources = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.push((Source::new(*label), raw));
    }
    let spec = Specification::assemble(sources)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn restoring(components: &str, domain: &str) -> EssIr {
    model(&[
        ("system.yaml", &system("demo.d")),
        ("components.yaml", components),
        ("domains/d.yaml", domain),
    ])
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    assert_eq!(report.scenarios.len(), suite.scenarios.len());
    report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<String> = result.diagnostics().map(|d| format!("{d:?}")).collect();
            format!("{}: {:?} {diagnostics:?}", result.scenario, result.status)
        })
        .collect()
}

/// Story 6 acceptance on `ir`: nothing synthesized fails on the interpreter, and `Give`'s
/// `not-flagged` branch is either synthesized and passing, or refused as ESS-SYNTH-003 naming
/// `Lower` and `binding`.
fn story6(ir: &EssIr, binding: &str) {
    let synthesis = synthesize(ir);
    // The binding's own delivery scenario (a second `Lowered` whose `Raise` now refuses under the
    // unbounded `retry` policy) is out of this story's scope and reported apart.
    let failed: Vec<String> = not_passed(ir.clone(), &synthesis.suite)
        .into_iter()
        .filter(|failure| !failure.starts_with(&format!("{binding}/binding/delivery: Unsupported")))
        .collect();
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off: no scenario whose \
         arranged row a binding rewrites is sent; refusals: {:#?}",
        refusals(&synthesis)
    );
    let synthesized = synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == "demo.d.Give/outcome/not-flagged");
    if !synthesized {
        let refused: Vec<String> = refusals(&synthesis)
            .into_iter()
            .filter(|refusal| refusal.contains("`demo.d.Give/outcome/not-flagged`"))
            .collect();
        assert_eq!(refused.len(), 1, "{:#?}", refusals(&synthesis));
        assert!(
            refused[0].contains("ESS-SYNTH-003")
                && refused[0].contains("`demo.d.Lower`")
                && refused[0].contains(&format!("`{binding}`")),
            "story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off: {}",
            refused[0]
        );
    }
}

const LOWER_BODY: &str = "        updates: demo.d.Grant\n        instance: grant_id\n        sets: {flag: false}\n        emits: [demo.d.Lowered]\n";
const RAISE_BODY: &str = "        updates: demo.d.Grant\n        instance: grant_id\n        sets: {flag: true}\n        emits: [demo.d.Raised]\n";

/// S6-1: the binding moves the row back to the state the guard reads (`state == Low`), not a
/// field.
#[test]
fn s6_a_binding_that_moves_the_arranged_row_back_is_followed() {
    let domain = replaced(
        DOMAIN,
        "    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n",
        "    lifecycle:\n      initial: Active\n      states: [Active, Low]\n      terminal: []\n      transitions:\n        - {name: lower, from: [Active], to: Low}\n        - {name: raise, from: [Low], to: Active}\n",
    );
    let domain = replaced(
        &domain,
        LOWER_BODY,
        "        moves: demo.d.Grant.lower\n        instance: grant_id\n        emits: [demo.d.Lowered]\n",
    );
    let domain = replaced(
        &domain,
        RAISE_BODY,
        "        moves: demo.d.Grant.raise\n        instance: grant_id\n        emits: [demo.d.Raised]\n",
    );
    let domain = replaced(
        &domain,
        "      - name: lowered\n",
        "      - {name: not-active, wrong_state: true, error: demo.d.Refused}\n      - name: lowered\n",
    );
    let domain = replaced(
        &domain,
        "      - name: raised\n",
        "      - {name: not-low, wrong_state: true, error: demo.d.Refused}\n      - name: raised\n",
    );
    let domain = replaced(
        &domain,
        "predicate: flag == false",
        "predicate: state == Low",
    );
    story6(&restoring(COMPONENTS, &domain), "raise-on-lowered");
}

/// S6-2: the binding removes the arranged row; `Give` then answers `missing`.
#[test]
fn s6_a_binding_that_removes_the_arranged_row_is_followed() {
    let domain = replaced(
        DOMAIN,
        RAISE_BODY,
        "        deletes: demo.d.Grant\n        instance: grant_id\n        emits: [demo.d.Raised]\n",
    );
    story6(&restoring(COMPONENTS, &domain), "raise-on-lowered");
}

/// S6-3: the rewrite is two bindings down: `Lowered` sets off `Note`, whose `Noted` sets off
/// `Raise`.
#[test]
fn s6_a_binding_two_hops_down_that_rewrites_the_row_is_followed() {
    let components = replaced(COMPONENTS, "demo.d.Raise,", "demo.d.Raise, demo.d.Note,");
    let components = replaced(
        &components,
        "  - id: raise-on-lowered\n    when:\n      event: demo.d.Lowered\n",
        "  - id: note-on-lowered\n    when:\n      event: demo.d.Lowered\n    invoke:\n      command: demo.d.Note\n    mapping:\n      grant_id: event.grant_id\n    delivery: at_least_once\n    on_failure: retry\n  - id: raise-on-noted\n    when:\n      event: demo.d.Noted\n",
    );
    let domain = replaced(
        DOMAIN,
        "  - name: demo.d.Touch\n",
        "  - name: demo.d.Note\n    input:\n      - {name: grant_id, type: demo.d.GrantId}\n    outcomes:\n      - name: noted\n        emits: [demo.d.Noted]\n        payload: {demo.d.Noted: {grant_id: input.grant_id}}\n  - name: demo.d.Touch\n",
    );
    let domain = replaced(
        &domain,
        "  - name: demo.d.Touched\n",
        "  - name: demo.d.Noted\n    fields: [{name: grant_id, type: demo.d.GrantId}]\n  - name: demo.d.Touched\n",
    );
    story6(&restoring(&components, &domain), "raise-on-noted");
}

// ---- story 7 ----

fn desk22() -> String {
    DESK20.replace("format: ess/20", "format: ess/22")
}

const WRONG_STATE: &str =
    "      - {name: wrong-state, wrong_state: true, error: demo.desk.NotPicked}\n";

fn with_wrong_state(text: &str) -> String {
    replaced(
        text,
        "      - name: accepted\n",
        &format!("{WRONG_STATE}      - name: accepted\n"),
    )
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

/// Exchanged orders, the ones `adversary_selection_plan_w5_u4_pass1` reads and the ones moving the
/// related-row phase that it does not.
fn orders() -> Vec<(String, [Phase; 8])> {
    use Phase::*;
    [
        (RelatedRow, Existence),
        (RelatedRow, HeldState),
        (Existence, HeldState),
        (HeldState, PresentRelated),
        (HeldState, Accepting),
        (PresentRelated, Accepting),
        (InputRefusal, Existence),
        (RelatedRow, InputRefusal),
    ]
    .into_iter()
    .map(|(a, b)| (format!("{a}<->{b}"), swap(a, b)))
    .collect()
}

/// Every scenario of `text`, in the real order and under each exchanged order, that the
/// interpreter reading the same order does not pass.
fn failures_every_order(text: &str) -> Vec<String> {
    let model = ir(text);
    let result = synthesize(&model);
    let mut failures: Vec<String> = not_passed(model.clone(), &result.suite)
        .into_iter()
        .map(|failure| format!("[real] {failure}"))
        .collect();
    for (name, order) in orders() {
        let run = catch_unwind(AssertUnwindSafe(|| {
            with_phase_order(order, || {
                let result = synthesize(&model);
                not_passed(model.clone(), &result.suite)
            })
        }));
        match run {
            Ok(failed) => failures.extend(
                failed
                    .into_iter()
                    .map(|failure| format!("[{name}] {failure}")),
            ),
            Err(_) => failures.push(format!("[{name}] panicked")),
        }
    }
    failures
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("no scenario {id}; refusals: {:#?}", refusals(result)),
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

/// S7-1: A4b's model (ess/22, `wrong_state:`, `stale` before the external) under exchanged
/// orders, which A4b does not run.
#[test]
fn s7_wrong_state_beside_external_passes_under_exchanged_orders() {
    assert_eq!(
        failures_every_order(&with_wrong_state(&desk22())),
        Vec::<String>::new(),
        "story:related-guard-wrong-state-beside-external"
    );
}

/// S7-2: the unknown-identity scenario of that model, `CheckPick/outcome/wrong-state` (the
/// command declares no `unknown_instance:`, so `wrong_state:` answers an identity no row carries,
/// at the held state), synthesized with existence read before the related row and the related
/// row before the held state: the list's `exists: false` still answers before the branch the
/// scenario proves, so the send must name a present list (A4c's assertion, under that order).
#[test]
fn s7_unknown_identity_names_a_present_list_with_existence_before_the_related_row() {
    let model = ir(&with_wrong_state(&desk22()));
    let sent = with_phase_order(swap(Phase::RelatedRow, Phase::Existence), || {
        let result = synthesize(&model);
        let wrong = scenario(&result, "demo.desk.CheckPick/outcome/wrong-state");
        (
            last_sent(wrong, "demo.desk.CheckPick", "list_id"),
            format!("{:#?}", wrong.steps),
        )
    });
    assert!(
        matches!(sent.0, ScenarioValue::Instance { .. }),
        "story:related-guard-wrong-state-beside-external: list_id sent as {:?}; steps: {}",
        sent.0,
        sent.1
    );
}

/// S7-4: ess/22 with `wrong_state:` and an accepting `when:` branch declared before the external,
/// in every order.
#[test]
fn s7_wrong_state_with_an_accepting_when_before_the_external_passes() {
    let text = replaced(
        &with_wrong_state(&desk22()),
        "      - name: unlisted\n",
        "      - name: rushed\n        when: revision > 100\n        moves: demo.desk.Pick.accept\n        instance: pick_id\n        emits: [demo.desk.PickAccepted]\n        payload:\n          demo.desk.PickAccepted: {pick_id: input.pick_id}\n      - name: unlisted\n",
    );
    assert_eq!(
        failures_every_order(&text),
        Vec::<String>::new(),
        "story:related-guard-wrong-state-beside-external"
    );
}

/// S7-5: the ess/20 model whose suite bytes the unit moved (`claim-search/related.yaml`, no
/// `wrong_state:`), in every order.
#[test]
fn s7_the_moved_ess20_model_passes_in_every_order() {
    assert_eq!(
        failures_every_order(DESK20),
        Vec::<String>::new(),
        "story:related-guard-wrong-state-beside-external"
    );
}
