//! Adversary pass 1 on beyond10x/ess#438 and #439 (list and window parameters on aggregate views).
//!
//! A defect a target can have is expressed as a model: the suite synthesized from the honest model
//! is run by the native interpreter against a mutated model that behaves like the faulty target. A
//! target that ignores `param.from` behaves like a model whose `from` conjunct always holds; a
//! target whose `to` is inclusive behaves like a model with `<=`. The honest model must pass its
//! own suite and every such mutant must fail it, or the suite does not witness the bound.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, scenario::ScenarioId, synthesize::synthesize, AdmittedSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const WINDOW_YAML: &str = include_str!("fixtures/aggregate-timestamp-range.yaml");
const LIST_YAML: &str = include_str!("fixtures/aggregate-list-parameter.yaml");

const WINDOW_FILTER: &str = "filter: [started_at >= param.from, started_at < param.to]";

fn validated(text: &str) -> Result<EssIr, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    let spec = Specification::assemble([(Source::new("work.yaml"), raw)])
        .map_err(|errors| errors.to_string())?;
    compile(&spec, &SourceMap::new()).map_err(|diagnostics| diagnostics.to_string())
}

fn ir(text: &str) -> EssIr {
    validated(text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

/// `text` with every `from` replaced by `to`; fails where `from` is not there.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the source holds `{from}`");
    text.replace(from, to)
}

/// The aggregate scenarios synthesized from `suite_from`, and the aggregate refusals, rendered.
fn aggregate_suite(suite_from: &str) -> (Option<AdmittedSuite>, Vec<String>) {
    let result = synthesize(&ir(suite_from));
    let refusals = result
        .refusals
        .iter()
        .filter(|refusal| {
            matches!(
                refusal.code().to_string().as_str(),
                "ESS-SYNTH-016" | "ESS-SYNTH-017"
            )
        })
        .map(ToString::to_string)
        .collect();
    let mut suite = result.suite;
    suite
        .scenarios
        .retain(|id, _| matches!(id, ScenarioId::Aggregate { .. }));
    if suite.scenarios.is_empty() {
        return (None, refusals);
    }
    (
        Some(AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"))),
        refusals,
    )
}

/// Every aggregate scenario's verdict when the native interpreter of `model` runs `suite`.
fn verdicts(suite: &AdmittedSuite, model: &str) -> BTreeMap<String, String> {
    let target = Interpreted::for_model(ir(model));
    Runner::for_suite(suite.suite())
        .run_admitted(suite, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let status = match result.status {
                ess_conformance::report::Status::Passed => "passed",
                ess_conformance::report::Status::Failed => "failed",
                ess_conformance::report::Status::Error => "error",
                ess_conformance::report::Status::Unsupported => "unsupported",
            };
            (result.scenario.to_string(), status.to_owned())
        })
        .collect()
}

fn not_passed(verdicts: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    verdicts
        .iter()
        .filter(|(_, status)| status.as_str() != "passed")
        .map(|(id, status)| (id.clone(), status.clone()))
        .collect()
}

/// The two window mutants of a filter written `[<rest>started_at <lower> param.from, started_at
/// <upper> param.to]`: `from` ignored (a disjunction that always holds and still reads it), and `to`
/// made inclusive.
fn window_mutants(honest: &str, lower: &str, upper: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "ignores-from",
            replaced(
                honest,
                &format!("started_at {lower} param.from"),
                "{any: [started_at >= param.from, started_at < param.from]}",
            ),
        ),
        (
            "inclusive-to",
            replaced(
                honest,
                &format!("started_at {upper} param.to"),
                "started_at <= param.to",
            ),
        ),
    ]
}

/// The honest model passes its own suite, and every mutant fails both window views.
fn assert_window_witnessed(honest: &str, lower: &str, upper: &str) {
    let (suite, refusals) = aggregate_suite(honest);
    assert_eq!(refusals, Vec::<String>::new(), "the views synthesize");
    let suite = suite.expect("aggregate scenarios");
    assert_eq!(
        not_passed(&verdicts(&suite, honest)),
        BTreeMap::new(),
        "the honest model passes its own suite"
    );
    let mut survived = Vec::new();
    for (fault, mutant) in window_mutants(honest, lower, upper) {
        let observed = verdicts(&suite, &mutant);
        for view in [
            "demo.window.CallsInRange/aggregate",
            "demo.window.QueueCallsInRange/aggregate",
        ] {
            if observed.get(view).map(String::as_str) != Some("failed") {
                survived.push(format!("{fault} {view}: {:?}", observed.get(view)));
            }
        }
    }
    assert_eq!(
        survived,
        Vec::<String>::new(),
        "a target with one of these defects passes the window view"
    );
}

/// The committed fixture: the oracle above holds where the window is the whole filter.
#[test]
fn adv_window_mutants_fail_the_committed_fixture() {
    assert_window_witnessed(WINDOW_YAML, ">=", "<");
}

/// The window fixture with a second state, reached by one move, and a filter that reads it too:
/// "the reviewed calls in the range". Rows outside the window are created as refuted rows, and a
/// refuted row stays in the initial state, so it is refuted by `state == Reviewed` as well as by the
/// bound: a target that ignores `param.from` or makes `to` inclusive answers the same numbers.
fn reviewed_window() -> String {
    let lifecycle = replaced(
        WINDOW_YAML,
        "      states: [Recorded]\n      terminal: [Recorded]\n      transitions: []\n",
        "      states: [Recorded, Reviewed]\n      terminal: [Reviewed]\n      transitions: [{name: review, from: [Recorded], to: Reviewed}]\n",
    );
    let events = replaced(
        &lifecycle,
        "entities:\n",
        "  - name: demo.window.CallReviewed\n    fields: [{name: call_id, type: Uuid}]\nentities:\n",
    );
    let command = replaced(
        &events,
        "views:\n",
        "  - name: demo.window.Review
    input: [{name: call_id, type: Uuid}]
    outcomes:
      - name: reviewed
        moves: demo.window.Call.review
        instance: call_id
        emits: [demo.window.CallReviewed]
        payload: {demo.window.CallReviewed: {call_id: input.call_id}}
      - {name: unavailable, wrong_state: true, refuses: false}
views:\n",
    );
    replaced(
        &command,
        WINDOW_FILTER,
        "filter: [state == Reviewed, started_at >= param.from, started_at < param.to]",
    )
}

#[test]
fn adv_window_beside_a_state_conjunct_witnesses_its_bounds() {
    assert_window_witnessed(&reviewed_window(), ">=", "<");
}

/// Exclusive `from`, inclusive `to`, each written either way round: the honest model passes and a
/// target that moves either edge fails.
#[test]
fn adv_window_other_orderings_are_witnessed() {
    let honest = replaced(
        WINDOW_YAML,
        WINDOW_FILTER,
        "filter: [param.from < started_at, param.to > started_at]",
    );
    let (suite, refusals) = aggregate_suite(&honest);
    assert_eq!(refusals, Vec::<String>::new(), "the views synthesize");
    let suite = suite.expect("aggregate scenarios");
    assert_eq!(
        not_passed(&verdicts(&suite, &honest)),
        BTreeMap::new(),
        "the honest model passes its own suite"
    );
    for (fault, mutant) in [
        (
            "inclusive-from",
            replaced(
                &honest,
                "param.from < started_at",
                "param.from <= started_at",
            ),
        ),
        (
            "ignores-to",
            replaced(
                &honest,
                "param.to > started_at",
                "{any: [started_at >= param.to, started_at < param.to]}",
            ),
        ),
    ] {
        let observed = verdicts(&suite, &mutant);
        assert_eq!(
            observed
                .get("demo.window.CallsInRange/aggregate")
                .map(String::as_str),
            Some("failed"),
            "{fault}: {observed:#?}"
        );
    }
}

/// A creating command that refuses a call started more than an hour ago: the window's rows are
/// arranged at fixed 2020 instants, so either the view is refused by name or the honest model
/// passes the suite synthesized for it.
#[test]
fn adv_window_rows_respect_a_clock_guard_on_the_creating_command() {
    let errors = replaced(
        WINDOW_YAML,
        "commands:\n",
        "errors:\n  - {name: demo.window.Stale, summary: The call started too long ago., fields: []}\ncommands:\n",
    );
    let honest = replaced(
        &errors,
        "      - name: recorded\n",
        "      - name: stale\n        when: input.started_at < now - 1h\n        error: demo.window.Stale\n      - name: recorded\n",
    );
    let (suite, refusals) = aggregate_suite(&honest);
    let Some(suite) = suite else {
        assert_ne!(refusals, Vec::<String>::new(), "refused by name");
        return;
    };
    let observed = verdicts(&suite, &honest);
    for view in [
        "demo.window.CallsInRange/aggregate",
        "demo.window.QueueCallsInRange/aggregate",
    ] {
        if let Some(status) = observed.get(view) {
            assert_eq!(status, "passed", "{view}: {observed:#?}");
        } else {
            assert!(
                refusals
                    .iter()
                    .any(|refusal| refusal.contains(view.split('/').next().unwrap())),
                "{view}: neither run nor refused: {refusals:#?}"
            );
        }
    }
}

/// The list fixture: the honest model passes, and a target that ignores the list fails both list
/// views — also beside a state conjunct.
#[test]
fn adv_list_views_pass_honestly_and_fail_when_the_list_is_ignored() {
    let (suite, _) = aggregate_suite(LIST_YAML);
    let suite = suite.expect("aggregate scenarios");
    let honest = verdicts(&suite, LIST_YAML);
    for view in [
        "demo.calls.InQueues/aggregate",
        "demo.calls.InQueuesOrAll/aggregate",
    ] {
        assert_eq!(
            honest.get(view).map(String::as_str),
            Some("passed"),
            "{honest:#?}"
        );
    }
    let ignores = replaced(
        LIST_YAML,
        "filter: {exists: {in: param.queues, as: q, that: queue_id == q}}",
        "filter: {any: [param.queues.count == 0, param.queues.count > 0]}",
    );
    let observed = verdicts(&suite, &ignores);
    assert_eq!(
        observed
            .get("demo.calls.InQueues/aggregate")
            .map(String::as_str),
        Some("failed"),
        "{observed:#?}"
    );
}

/// The membership written the other way round in both places: `q == queue_id`, and the empty-list
/// disjunct second. The honest model passes what is synthesized.
#[test]
fn adv_list_selector_written_the_other_way_round_passes_honestly() {
    let honest = replaced(
        LIST_YAML,
        "        - param.queues.count == 0\n        - {exists: {in: param.queues, as: q, that: queue_id == q}}\n",
        "        - {exists: {in: param.queues, as: q, that: q == queue_id}}\n        - param.queues.count == 0\n",
    );
    let (suite, _) = aggregate_suite(&honest);
    let suite = suite.expect("aggregate scenarios");
    let observed = verdicts(&suite, &honest);
    assert_eq!(
        observed
            .get("demo.calls.InQueuesOrAll/aggregate")
            .map(String::as_str),
        Some("passed"),
        "{observed:#?}"
    );
}

/// `in_ignore_case` is a membership operator over text literals too: `label: {in_ignore_case:
/// [param.queues]}` compares with the text `param.queues` exactly as `label: [param.queues]` does,
/// which the change refuses naming the quantifier. The parameter is read elsewhere too, so the
/// view is not refused as reading it nowhere.
#[test]
fn adv_in_ignore_case_naming_a_parameter_is_refused_like_in() {
    let labelled = replaced(
        LIST_YAML,
        "      - {name: duration_ms, type: Integer}\n    lifecycle",
        "      - {name: duration_ms, type: Integer}\n      - {name: label, type: String}\n    lifecycle",
    );
    let shorthand = labelled.replacen(
        "filter: {exists: {in: param.queues, as: q, that: queue_id == q}}",
        "filter: [{label: [param.queues]}, param.queues.count >= 0]",
        1,
    );
    let control = validated(&shorthand).expect_err("the control is refused");
    assert!(
        control.contains("exists: {in: param.queues, as: x, that: label == x}"),
        "the control is refused by the new refusal: {control}"
    );
    let folded = labelled.replacen(
        "filter: {exists: {in: param.queues, as: q, that: queue_id == q}}",
        "filter: [{label: {in_ignore_case: [param.queues]}}, param.queues.count >= 0]",
        1,
    );
    match validated(&folded) {
        Ok(_) => panic!("`label: {{in_ignore_case: [param.queues]}}` validates, reading the text"),
        Err(error) => assert!(error.contains("exists: {in: param.queues"), "{error}"),
    }
}

/// The refusal's hint is a rewrite the author can apply: for a scalar parameter, `queue_id: {in:
/// param.queue}` means `queue_id == param.queue`, and the form the hint names must validate.
#[test]
fn adv_membership_hint_for_a_scalar_parameter_names_a_form_that_validates() {
    let scalar = LIST_YAML.replacen(
        "    params: [{name: queues, type: List<Integer>}]\n    filter: {exists: {in: param.queues, as: q, that: queue_id == q}}",
        "    params: [{name: queue, type: Integer}]\n    filter: {queue_id: {in: param.queue}}",
        1,
    );
    assert_ne!(scalar, LIST_YAML, "the view is rewritten");
    let error = validated(&scalar).expect_err("refused");
    let hinted = "queue_id == param.queue";
    assert!(error.contains(hinted), "{error}");
    let applied = scalar.replacen(
        "filter: {queue_id: {in: param.queue}}",
        &format!("filter: {hinted}"),
        1,
    );
    if let Err(error) = validated(&applied) {
        panic!("the hinted rewrite is refused:\n{error}");
    }
}
