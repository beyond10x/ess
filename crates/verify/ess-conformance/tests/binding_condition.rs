//! A binding invokes only when its event-payload condition holds (ess/22, beyond10x/ess#268), and
//! an Optional path the condition proves present can fill a required input (beyond10x/ess#194).
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Payload condition", names the controls
//! this file holds the model-owned target to: `condition_true_invokes`,
//! `condition_false_never_invokes`, `optional_parent_absent_skips_before_mapping`,
//! `other_binding_still_invokes`, and Unknown as an unmet obligation that invokes nothing.
use std::collections::BTreeMap;

use ess_compiler::refs::{BindingRef, CommandRef};
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};

const MODEL: &str = include_str!("fixtures/binding-condition.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("messages.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.messages/authored/condition".parse().unwrap(),
        CorrelationId::new("condition").unwrap(),
    )
}

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

fn order(id: &str) -> Node {
    Node::Map(BTreeMap::from([("id".into(), text(id))]))
}

/// Publishes one `MessageReceived` through its command on a fresh interpreted target, and answers
/// the command's result with every invocation each binding then made.
fn receive(
    text_model: &str,
    kind: &str,
    order: Option<Node>,
) -> (
    Result<SemanticCommandResult, TargetError>,
    Vec<ObservedInvocation>,
    Vec<ObservedInvocation>,
) {
    let target = Interpreted::for_model(ir_of(text_model));
    target.begin_scenario(&context()).unwrap();
    let mut input = BTreeMap::from([
        ("message_id".to_owned(), text("m-1")),
        ("kind".to_owned(), text(kind)),
    ]);
    if let Some(order) = order {
        input.insert("order".into(), order);
    }
    let result = target.execute_command(SemanticCommandRequest {
        command: CommandRef::new("demo.messages.ReceiveMessage".parse().unwrap()),
        actor: None,
        caller: None,
        input,
        correlation: context().correlation,
    });
    let observed = |binding: &str, command: &str| {
        target
            .observe_invocations(InvocationObservationRequest {
                binding: BindingRef::new(ess_domain::binding::BindingName::new(binding).unwrap()),
                command: CommandRef::new(command.parse().unwrap()),
                correlation: context().correlation,
                deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
            })
            .unwrap()
    };
    let received = observed("received", "demo.messages.MessageEvent");
    let logged = observed("logged", "demo.messages.LogMessage");
    (result, received, logged)
}

#[test]
fn condition_true_invokes_with_the_proved_input() {
    let (result, received, logged) = receive(MODEL, "ship", Some(order("o-1")));
    result.expect("the trigger runs");
    assert_eq!(received.len(), 1, "{received:?}");
    assert_eq!(received[0].input.get("order_id"), Some(&text("o-1")));
    assert_eq!(logged.len(), 1, "{logged:?}");
}

#[test]
fn condition_false_never_invokes_and_other_binding_still_invokes() {
    let (result, received, logged) = receive(MODEL, "note", Some(order("o-1")));
    result.expect("a false condition is a successful skip");
    assert_eq!(received.len(), 0, "{received:?}");
    assert_eq!(logged.len(), 1, "{logged:?}");
}

#[test]
fn optional_parent_absent_skips_before_mapping() {
    for absent in [None, Some(Node::Null)] {
        let (result, received, logged) = receive(MODEL, "ship", absent.clone());
        result.unwrap_or_else(|error| panic!("{absent:?}: an absent order skips: {error}"));
        assert_eq!(received.len(), 0, "{absent:?}: {received:?}");
        assert_eq!(logged.len(), 1, "{absent:?}: {logged:?}");
    }
}

#[test]
fn an_unknown_condition_is_an_unmet_obligation_that_invokes_nothing() {
    // `event.order.note == x` reads an absent member when the order is absent: Unknown.
    let unknown = MODEL
        .replace(
            WHERE,
            "      where: [event.kind == ship, event.order.note == x]\n",
        )
        .replace(
            "      order_id: event.order.id\n",
            "      order_id: event.message_id\n",
        );
    let (result, received, _) = receive(&unknown, "ship", None);
    let error = result.expect_err("Unknown cannot become a successful skip");
    assert!(error.is_unsupported(), "{error}");
    assert!(error.to_string().contains("condition"), "{error}");
    assert_eq!(received.len(), 0, "{received:?}");
}

// ---- synthesis: witnesses on both sides, held against honest and faulty targets ---------------

use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};

const MAPPING: &str = "received/binding/mapping";
const FLOW: &str = "received/binding/flow";
const FALSE: &str = "received/binding/condition-false";
const ABSENT: &str = "received/binding/condition-absent";

fn suite_of(text: &str) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of(text)).suite
}

fn document(suite: &ConformanceSuite) -> serde_json::Value {
    serde_json::from_str(&suite.to_canonical_json().expect("the suite serialises")).expect("JSON")
}

fn trigger_input(document: &serde_json::Value, id: &str) -> serde_json::Value {
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no scenario {id} in {document:#}"))
        .iter()
        .find(|step| {
            step["step"] == "execute_command" && step["command"] == "demo.messages.ReceiveMessage"
        })
        .unwrap_or_else(|| panic!("{id} sends no trigger"))["input"]
        .clone()
}

fn statuses(suite: &ConformanceSuite, text: &str) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Interpreted::for_model(ir_of(text));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let checks = format!("{:?}", result.checks);
            (result.scenario.to_string(), (result.status, checks))
        })
        .collect()
}

fn status(statuses: &BTreeMap<String, (Status, String)>, id: &str) -> Status {
    statuses
        .get(id)
        .unwrap_or_else(|| panic!("no scenario {id}: {statuses:#?}"))
        .0
}

/// The same binding as the model, mapped from a field that is always present so that a faulty
/// condition still compiles.
fn faulty(condition: &str) -> String {
    MODEL.replace(WHERE, condition).replace(
        "      order_id: event.order.id\n",
        "      order_id: event.message_id\n",
    )
}

#[test]
fn the_suite_witnesses_the_condition_on_both_sides_in_suite_36() {
    let suite = suite_of(MODEL);
    let document = document(&suite);
    assert_eq!(
        document["provenance"]["suite_version"], "ess-conformance/36",
        "{:#}",
        document["provenance"]
    );
    // condition_true_invokes: the positive aspects publish a payload the condition holds for.
    let holds = trigger_input(&document, MAPPING);
    assert_eq!(holds["kind"]["value"], "ship", "{holds:#}");
    assert!(holds["order"]["value"].is_object(), "{holds:#}");
    // condition_false_never_invokes: the discriminator fails with the order present.
    let fails = trigger_input(&document, FALSE);
    assert_eq!(fails["kind"]["value"], "note", "{fails:#}");
    assert!(fails["order"]["value"].is_object(), "{fails:#}");
    // optional_parent_absent_skips_before_mapping: the order is left out.
    let absent = trigger_input(&document, ABSENT);
    assert_eq!(absent["kind"]["value"], "ship", "{absent:#}");
    assert!(absent.get("order").is_none(), "{absent:#}");
    for id in [FALSE, ABSENT] {
        let steps = document["scenarios"][id]["steps"].as_array().unwrap();
        let last = steps.last().unwrap();
        assert_eq!(last["step"], "expect_no_invocation", "{id}: {steps:#?}");
        assert_eq!(last["binding"], "received", "{id}");
        assert_eq!(last["command"], "demo.messages.MessageEvent", "{id}");
    }
}

#[test]
fn an_honest_target_passes_every_conditioned_binding_scenario() {
    let suite = suite_of(MODEL);
    let statuses = statuses(&suite, MODEL);
    for id in [MAPPING, FLOW, FALSE, ABSENT] {
        assert_eq!(status(&statuses, id), Status::Passed, "{id}: {statuses:#?}");
    }
    for (id, (status, checks)) in &statuses {
        assert_ne!(*status, Status::Failed, "{id}: {checks}");
    }
}

#[test]
fn a_target_that_ignores_the_condition_fails_both_negative_witnesses() {
    let statuses = statuses(&suite_of(MODEL), &faulty("      where: true\n"));
    assert_eq!(status(&statuses, FALSE), Status::Failed, "{statuses:#?}");
    assert_eq!(status(&statuses, ABSENT), Status::Failed, "{statuses:#?}");
}

#[test]
fn a_target_that_inverts_the_condition_fails_both_sides() {
    let statuses = statuses(
        &suite_of(MODEL),
        &faulty("      where: {not: [defined(event.order), event.kind == ship]}\n"),
    );
    assert_eq!(status(&statuses, FALSE), Status::Failed, "{statuses:#?}");
    assert_eq!(status(&statuses, MAPPING), Status::Failed, "{statuses:#?}");
}

#[test]
fn a_target_that_fires_on_absence_fails_the_absent_witness_only_there() {
    let statuses = statuses(
        &suite_of(MODEL),
        &faulty("      where: event.kind == ship\n"),
    );
    assert_eq!(status(&statuses, ABSENT), Status::Failed, "{statuses:#?}");
    assert_eq!(status(&statuses, FALSE), Status::Passed, "{statuses:#?}");
}

#[test]
fn an_older_suite_envelope_refuses_zero_invocation_observation() {
    let suite = suite_of(MODEL);
    let text = suite
        .to_canonical_json()
        .unwrap()
        .replace("\"ess-conformance/36\"", "\"ess-conformance/35\"");
    let error = AdmittedSuite::from_json(&text).expect_err("suite/35 cannot carry the step");
    assert!(error.to_string().contains("suite"), "{error}");
}

#[test]
fn a_model_without_a_condition_keeps_its_suite_major() {
    let plain = MODEL.replace(WHERE, "").replace(
        "      order_id: event.order.id\n",
        "      order_id: event.message_id\n",
    );
    let document = document(&suite_of(&plain));
    assert_eq!(
        document["provenance"]["suite_version"],
        "ess-conformance/34"
    );
}

// ---- the generated Go runtime gives the native verdicts, honest and faulty ---------------------

mod support_go;

#[test]
fn go_gives_the_native_verdicts_for_the_conditioned_binding_suite() {
    let suite = suite_of(MODEL);
    for (label, text, wanted) in [
        (
            "correct",
            MODEL.to_owned(),
            [("passed", FALSE), ("passed", ABSENT)],
        ),
        (
            "ignore",
            faulty("      where: true\n"),
            [("failed", FALSE), ("failed", ABSENT)],
        ),
        (
            "absence",
            faulty("      where: event.kind == ship\n"),
            [("passed", FALSE), ("failed", ABSENT)],
        ),
        (
            "invert",
            faulty("      where: {not: [defined(event.order), event.kind == ship]}\n"),
            [("failed", FALSE), ("failed", ABSENT)],
        ),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("binding-condition-{label}"),
            &suite,
            Interpreted::for_model(ir_of(&text)),
        );
        for (status, id) in wanted {
            assert_eq!(verdicts[id], status, "{label}: {id}: {verdicts:#?}");
        }
    }
}

// ---- correction round: literal branches, every proved level, Unknown, one occurrence ------------

fn refusals(text: &str) -> Vec<String> {
    ess_conformance::synthesize::synthesize(&ir_of(text))
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
        .collect()
}

/// The publishing branches, each writing `kind` as a literal: `ship` and `note`.
fn literal_branches() -> String {
    MODEL.replace(
        "  - name: demo.messages.ReceiveMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: kind, type: demo.messages.Kind}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: received\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: input.kind, order: input.order}\n",
        "  - name: demo.messages.ShipMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: shipped\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: ship, order: input.order}\n  - name: demo.messages.NoteMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: noted\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: note, order: input.order}\n",
    )
}

fn commands_sent(document: &serde_json::Value, id: &str) -> Vec<serde_json::Value> {
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .iter()
        .filter(|step| step["step"] == "execute_command")
        .cloned()
        .collect()
}

#[test]
fn a_literal_discriminator_is_witnessed_by_the_branch_that_writes_each_value() {
    let text = literal_branches();
    let document = document(&suite_of(&text));
    let holds = commands_sent(&document, MAPPING);
    assert_eq!(
        holds[0]["command"], "demo.messages.ShipMessage",
        "{holds:#?}"
    );
    let fails = commands_sent(&document, FALSE);
    assert_eq!(
        fails[0]["command"], "demo.messages.NoteMessage",
        "{fails:#?}"
    );
    assert!(
        fails[0]["input"]["order"]["value"].is_object(),
        "{fails:#?}"
    );
    let statuses = statuses(&suite_of(&text), &text);
    for id in [MAPPING, FLOW, FALSE, ABSENT] {
        assert_eq!(status(&statuses, id), Status::Passed, "{id}: {statuses:#?}");
    }
}

#[test]
fn no_branch_holding_the_condition_refuses_each_positive_aspect_by_scenario() {
    let only_note = literal_branches().replace(
        "  - name: demo.messages.ShipMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: shipped\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: ship, order: input.order}\n",
        "",
    );
    let refused = refusals(&only_note);
    for aspect in ["Flow", "Mapping", "Delivery", "OnFailure", "ConditionFalse"] {
        assert!(
            refused
                .iter()
                .any(|line| line.contains("BindingName(\"received\")")
                    && line.contains(&format!("aspect: {aspect} }}"))
                    && line.contains("no publishing branch")),
            "{aspect}: {refused:#?}"
        );
    }
    assert!(
        !refused
            .iter()
            .any(|line| line.starts_with("None") && line.contains("`received`")),
        "no whole-binding refusal: {refused:#?}"
    );
}

#[test]
fn every_proved_optional_level_is_left_out_in_its_own_occurrence() {
    let child = MODEL
        .replace(
            WHERE,
            "      where: [defined(event.order.note), event.kind == ship]\n",
        )
        .replace(
            "      order_id: event.order.id\n",
            "      order_id: event.order.note\n",
        );
    let document = document(&suite_of(&child));
    let sent = commands_sent(&document, ABSENT);
    assert_eq!(sent.len(), 2, "{sent:#?}");
    assert!(
        sent[0]["input"].get("order").is_none(),
        "parent absent: {sent:#?}"
    );
    let order = &sent[1]["input"]["order"]["value"];
    assert!(
        order.is_object() && order.get("note").is_none(),
        "child absent: {sent:#?}"
    );
    let statuses = statuses(&suite_of(&child), &child);
    assert_eq!(status(&statuses, ABSENT), Status::Passed, "{statuses:#?}");
}

#[test]
fn a_member_proved_only_by_a_comparison_is_witnessed_as_unknown_when_absent() {
    let compared = MODEL.replace(
        WHERE,
        "      where: [event.kind == ship, event.order.id == o-1]\n",
    );
    let suite = suite_of(&compared);
    let document = document(&suite);
    let sent = commands_sent(&document, ABSENT);
    assert!(sent[0]["input"].get("order").is_none(), "{sent:#?}");
    let steps = document["scenarios"][ABSENT]["steps"].as_array().unwrap();
    assert_eq!(steps.last().unwrap()["step"], "expect_no_invocation");
    // The honest interpreter reports the Unknown condition as its unmet obligation, never a pass
    // and never a failure.
    let statuses = statuses(&suite, &compared);
    assert_eq!(
        status(&statuses, ABSENT),
        Status::Unsupported,
        "{statuses:#?}"
    );
}

#[test]
fn a_witness_whose_chain_publishes_the_event_again_is_refused_by_name() {
    let chain = MODEL
        .replace(
            "  - name: demo.messages.MessageLogged\n",
            "  - name: demo.messages.Forwarded\n    fields:\n      - {name: message_id, type: String}\n  - name: demo.messages.MessageLogged\n",
        )
        .replace(
            "  - name: demo.messages.LogMessage\n",
            "  - name: demo.messages.Forward\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: forwarded\n        emits: [demo.messages.Forwarded, demo.messages.MessageReceived]\n        payload:\n          demo.messages.Forwarded: {message_id: input.message_id}\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: ship, order: input.order}\n  - name: demo.messages.LogMessage\n",
        )
        + "  - id: forward\n    when:\n      event: demo.messages.MessageReceived\n      where: event.kind == note\n    invoke: {command: demo.messages.Forward}\n    mapping:\n      message_id: event.message_id\n      order: event.order\n    delivery: at_least_once\n    on_failure: retry\n";
    let refused = refusals(&chain);
    assert!(
        refused
            .iter()
            .any(|line| line.contains("aspect: ConditionFalse")
                && line.contains("`forward` may publish `demo.messages.MessageReceived` again")),
        "{refused:#?}"
    );
    let statuses = statuses(&suite_of(&chain), &chain);
    for (id, (status, checks)) in &statuses {
        assert_ne!(*status, Status::Failed, "{id}: {checks}");
    }
}
