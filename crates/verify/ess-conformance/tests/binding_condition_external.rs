//! A conditioned binding on an event an external channel delivers (ess/22 and ess/18,
//! beyond10x/ess#268 slice 2), and the sibling a negative witness requires still to invoke.
//!
//! Nothing in the model publishes an externally delivered event, so the suite chooses each
//! delivered payload itself. Synthesis varies only the members the condition reads: the four
//! positive aspects deliver a payload the condition holds for, `condition-false` one it fails for
//! with every member present, and `condition-absent` one per proved Optional level with that level
//! left out. The interpreter passes all six, the generated Go runner gives its verdicts, and a
//! target ignoring, inverting or dropping a conjunct of the condition fails the witness about it.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const INBOX: &str = include_str!("fixtures/delivery-context.yaml");
const CONDITION: &str = "      where: [defined(event.order), event.kind == ship]\n";

const FLOW: &str = "received/binding/flow";
const MAPPING: &str = "received/binding/mapping";
const DELIVERY: &str = "received/binding/delivery";
const ON_FAILURE: &str = "received/binding/on-failure";
const FALSE: &str = "received/binding/condition-false";
const ABSENT: &str = "received/binding/condition-absent";

fn rewrite(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
    text.replacen(from, to, 1)
}

/// The inbox at ess/22, its event carrying a kind and an Optional order, and the binding invoking
/// only for a shipping message that names one: `condition` as its `where:`.
fn conditioned(condition: &str) -> String {
    let text = rewrite(INBOX, "format: ess/18\n", "format: ess/22\n");
    let text = rewrite(
        &text,
        "types:\n",
        "types:\n  - name: demo.inbox.Kind\n    kind: enum\n    variants: [note, ship]\n  - name: demo.inbox.Ref\n    kind: struct\n    fields:\n      - {name: id, type: String}\n",
    );
    let text = rewrite(
        &text,
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: kind, type: demo.inbox.Kind}\n      - {name: order, type: Optional<demo.inbox.Ref>}\n",
    );
    rewrite(
        &text,
        "      context_authority: account-messages\n",
        &format!("      context_authority: account-messages\n{condition}"),
    )
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("inbox.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite_of(text: &str) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of(text)).suite
}

fn document(suite: &ConformanceSuite) -> serde_json::Value {
    serde_json::from_str(&suite.to_canonical_json().expect("the suite serialises")).expect("JSON")
}

fn delivered(document: &serde_json::Value, id: &str) -> Vec<serde_json::Value> {
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no scenario {id} in {document:#}"))
        .iter()
        .filter(|step| step["step"] == "deliver_event")
        .map(|step| step["payload"].clone())
        .collect()
}

fn statuses(suite: &ConformanceSuite, text: &str) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir_of(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

#[test]
fn every_aspect_of_an_external_conditioned_binding_is_synthesized() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&conditioned(CONDITION)));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("received"))
        .collect();
    assert_eq!(
        refused,
        Vec::<String>::new(),
        "nothing about the binding is refused"
    );
    let document = document(&synthesis.suite);
    for id in [FLOW, MAPPING, DELIVERY, ON_FAILURE] {
        for payload in delivered(&document, id) {
            assert_eq!(payload["kind"], "ship", "{id}: {payload:#}");
            assert!(payload["order"].is_object(), "{id}: {payload:#}");
        }
    }
    let fails = delivered(&document, FALSE);
    assert_eq!(fails.len(), 1, "{fails:#?}");
    assert_eq!(fails[0]["kind"], "note", "{fails:#?}");
    assert!(
        fails[0]["order"].is_object(),
        "every member present: {fails:#?}"
    );
    let absent = delivered(&document, ABSENT);
    assert_eq!(absent.len(), 1, "{absent:#?}");
    assert_eq!(absent[0]["kind"], "ship", "{absent:#?}");
    assert!(absent[0].get("order").is_none(), "{absent:#?}");
    for id in [FALSE, ABSENT] {
        let steps = document["scenarios"][id]["steps"].as_array().unwrap();
        assert_eq!(
            steps.last().unwrap()["step"],
            "expect_no_invocation",
            "{id}"
        );
    }
}

#[test]
fn the_interpreter_passes_every_external_conditioned_scenario() {
    let text = conditioned(CONDITION);
    let statuses = statuses(&suite_of(&text), &text);
    for id in [FLOW, MAPPING, DELIVERY, ON_FAILURE, FALSE, ABSENT] {
        assert_eq!(
            statuses.get(id),
            Some(&Status::Passed),
            "{id}: {statuses:#?}"
        );
    }
}

#[test]
fn a_target_wrong_about_the_external_condition_fails_the_witness_about_it() {
    let suite = suite_of(&conditioned(CONDITION));
    let ignoring = statuses(&suite, &conditioned("      where: true\n"));
    assert_eq!(ignoring[FALSE], Status::Failed, "{ignoring:#?}");
    assert_eq!(ignoring[ABSENT], Status::Failed, "{ignoring:#?}");
    let inverted = statuses(
        &suite,
        &conditioned("      where: {not: [defined(event.order), event.kind == ship]}\n"),
    );
    assert_eq!(inverted[MAPPING], Status::Failed, "{inverted:#?}");
    assert_eq!(inverted[FALSE], Status::Failed, "{inverted:#?}");
    let on_absence = statuses(&suite, &conditioned("      where: event.kind == ship\n"));
    assert_eq!(on_absence[ABSENT], Status::Failed, "{on_absence:#?}");
    assert_eq!(on_absence[FALSE], Status::Passed, "{on_absence:#?}");
}

#[test]
fn the_go_runner_gives_the_native_verdicts_for_the_external_conditioned_suite() {
    let suite = suite_of(&conditioned(CONDITION));
    for (label, condition, wanted) in [
        (
            "correct",
            CONDITION,
            [("passed", FALSE), ("passed", ABSENT)],
        ),
        (
            "ignore",
            "      where: true\n",
            [("failed", FALSE), ("failed", ABSENT)],
        ),
        (
            "absence",
            "      where: event.kind == ship\n",
            [("passed", FALSE), ("failed", ABSENT)],
        ),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("external-condition-{label}"),
            &suite,
            Interpreted::for_model(ir_of(&conditioned(condition))),
        );
        for (status, id) in wanted {
            assert_eq!(verdicts[id], status, "{label}: {id}: {verdicts:#?}");
        }
    }
}

// ---- other_binding_still_invokes ------------------------------------------------------------------

const MODEL: &str = include_str!("fixtures/binding-condition.yaml");

/// Both negative witnesses require the unconditioned binding beside the conditioned one to invoke
/// for the same occurrence, before the zero-invocation window: a condition that does not hold skips
/// its own binding alone.
#[test]
fn every_negative_witness_requires_the_sibling_binding_to_invoke() {
    let document = document(&suite_of(MODEL));
    for id in [FALSE, ABSENT] {
        let steps = document["scenarios"][id]["steps"].as_array().unwrap();
        let sibling = steps
            .iter()
            .position(|step| step["step"] == "expect_invocation" && step["binding"] == "logged")
            .unwrap_or_else(|| panic!("{id} requires `logged` to invoke: {steps:#?}"));
        assert_eq!(steps[sibling]["command"], "demo.messages.LogMessage");
        assert_eq!(
            sibling + 2,
            steps.len(),
            "{id}: just before the window: {steps:#?}"
        );
    }
    let statuses = statuses(&suite_of(MODEL), MODEL);
    for id in [FALSE, ABSENT] {
        assert_eq!(statuses[id], Status::Passed, "{id}: {statuses:#?}");
    }
}

/// A target whose condition is right but which stops the binding beside it on a skipped occurrence
/// — modelled as that sibling's own binding never invoking — fails both witnesses.
#[test]
fn a_target_that_stops_the_sibling_fails_both_negative_witnesses() {
    let stopped = rewrite(
        MODEL,
        "  - id: logged\n    when: {event: demo.messages.MessageReceived}\n",
        "  - id: logged\n    when:\n      event: demo.messages.MessageReceived\n      where: [defined(event.order), event.kind == ship]\n",
    );
    let statuses = statuses(&suite_of(MODEL), &stopped);
    for id in [FALSE, ABSENT] {
        assert_eq!(statuses[id], Status::Failed, "{id}: {statuses:#?}");
    }
}

// ---- other_binding_still_invokes, for an event an external channel delivers ----------------------

/// The conditioned inbox with an unconditioned `audited` beside `received` on the same channel and
/// event; `audited` takes `audit_condition` where one is given, as a target sharing `received`'s
/// condition would.
fn audited(audit_condition: &str) -> String {
    let text = conditioned(CONDITION);
    let text = rewrite(
        &text,
        "events:\n",
        "events:\n  - name: demo.inbox.MessageAudited\n    fields:\n      - {name: message_id, type: String}\n",
    );
    let text = rewrite(
        &text,
        "commands:\n",
        "commands:\n  - name: demo.inbox.AuditMessage\n    input:\n      - {name: message_id, type: String}\n    outcomes:\n      - name: audited\n        emits: [demo.inbox.MessageAudited]\n        payload:\n          demo.inbox.MessageAudited: {message_id: input.message_id}\n",
    );
    format!(
        "{text}  - id: audited\n    when:\n      event: demo.inbox.MessageReceived\n      \
         context_authority: account-messages\n{audit_condition}      context_fields:\n        - \
         {{name: account_id, type: demo.inbox.AccountId}}\n    invoke: {{command: \
         demo.inbox.AuditMessage}}\n    mapping:\n      message_id: event.message_id\n    \
         delivery: at_least_once\n    on_failure: retry\n"
    )
}

/// Both external negative witnesses require the binding beside the conditioned one on the
/// delivered event to invoke, before the zero-invocation window; a target whose sibling shares the
/// condition, and so stops where it skips, fails both.
#[test]
fn every_external_negative_witness_requires_the_sibling_binding_to_invoke() {
    let text = audited("");
    let suite = suite_of(&text);
    let document = document(&suite);
    for id in [FALSE, ABSENT] {
        let steps = document["scenarios"][id]["steps"].as_array().unwrap();
        let sibling = steps
            .iter()
            .position(|step| step["step"] == "expect_invocation" && step["binding"] == "audited")
            .unwrap_or_else(|| panic!("{id} requires `audited` to invoke: {steps:#?}"));
        assert_eq!(
            sibling + 2,
            steps.len(),
            "{id}: just before the window: {steps:#?}"
        );
    }
    let honest = statuses(&suite, &text);
    for id in [FALSE, ABSENT] {
        assert_eq!(honest[id], Status::Passed, "{id}: {honest:#?}");
    }
    let shared = statuses(
        &suite,
        &audited("      where: [defined(event.order), event.kind == ship]\n"),
    );
    for id in [FALSE, ABSENT] {
        assert_eq!(shared[id], Status::Failed, "{id}: {shared:#?}");
    }
}
