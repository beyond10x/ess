//! A selection binding whose required input copies an Optional member its condition proves present
//! (ess/22, beyond10x/ess#194 on a selection binding, beyond10x/ess#268).
//!
//! `fixtures/binding-condition-selected.yaml`: `received` selects the first leg of the message into
//! the Optional `leg` and copies the `tag` that `defined(event.tag)` proves present into the
//! required `order_id`. Synthesis witnesses it, the interpreter passes it, the generated Go runner
//! gives the interpreter's verdicts, and a target whose condition is wrong fails the witness about
//! it. The generated Rust and Go applications run it in `ess-synth`'s `binding_condition_runtime`.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("fixtures/binding-condition-selected.yaml");
const WHERE: &str = "      where: [defined(event.tag), event.kind == ship]\n";

const FLOW: &str = "received/binding/flow";
const MAPPING: &str = "received/binding/mapping";
const FALSE: &str = "received/binding/condition-false";
const ABSENT: &str = "received/binding/condition-absent";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("selected.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// The same binding with `condition` for its condition, its required input then filled from a
/// field every occurrence carries so that the faulty condition compiles.
fn faulty(condition: &str) -> String {
    assert_eq!(MODEL.matches(WHERE).count(), 1);
    MODEL.replace(WHERE, condition).replace(
        "      order_id: event.tag\n",
        "      order_id: event.message_id\n",
    )
}

fn suite() -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite
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
fn the_mapping_witness_requires_the_proved_tag_and_the_selected_leg() {
    let document: serde_json::Value =
        serde_json::from_str(&suite().to_canonical_json().unwrap()).unwrap();
    let steps = document["scenarios"][MAPPING]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no {MAPPING}: {document:#}"));
    let sent = steps
        .iter()
        .find(|step| step["step"] == "execute_command")
        .expect("a trigger");
    let expected = steps
        .iter()
        .find(|step| step["step"] == "expect_invocation")
        .expect("the invocation is required");
    assert_eq!(
        expected["input"]["order_id"],
        serde_json::json!({"kind": "observed", "event": "demo.messages.MessageReceived", "field": "tag"}),
        "the required input is the proved tag the occurrence carried: {steps:#?}"
    );
    assert_eq!(
        expected["input"]["leg"]["kind"], "observed_selection",
        "the Optional input is the selected leg: {steps:#?}"
    );
    assert!(
        sent["input"]["tag"]["value"].is_string(),
        "the trigger carries the tag the condition proves: {steps:#?}"
    );
}

#[test]
fn the_interpreter_passes_and_a_wrong_condition_fails_its_witness() {
    let suite = suite();
    let honest = statuses(&suite, MODEL);
    for id in [FLOW, MAPPING, FALSE, ABSENT] {
        assert_eq!(honest.get(id), Some(&Status::Passed), "{id}: {honest:#?}");
    }
    assert!(
        honest.values().all(|status| *status != Status::Failed),
        "{honest:#?}"
    );
    let ignoring = statuses(&suite, &faulty("      where: true\n"));
    assert_eq!(ignoring[FALSE], Status::Failed, "{ignoring:#?}");
    assert_eq!(ignoring[ABSENT], Status::Failed, "{ignoring:#?}");
    let on_absence = statuses(&suite, &faulty("      where: event.kind == ship\n"));
    assert_eq!(on_absence[ABSENT], Status::Failed, "{on_absence:#?}");
    assert_eq!(on_absence[FALSE], Status::Passed, "{on_absence:#?}");
}

#[test]
fn the_go_runner_gives_the_native_verdicts_for_the_selected_suite() {
    let suite = suite();
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
    ] {
        let verdicts = support_go::assert_parity(
            &format!("selected-condition-{label}"),
            &suite,
            Interpreted::for_model(ir_of(&text)),
        );
        for (status, id) in wanted {
            assert_eq!(verdicts[id], status, "{label}: {id}: {verdicts:#?}");
        }
    }
}
