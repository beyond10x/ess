//! A binding's event-payload condition, admitted and resolved (ess/22, beyond10x/ess#268 and
//! beyond10x/ess#194, `docs/design/conditional-binding-failure-policies.md`).
use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/binding-condition.yaml");
const PERIODIC: &str = include_str!("../../ess-domain/tests/fixtures/periodic.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";

fn admitted(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("messages.yaml"), raw)]).map_err(|e| e.to_string())
}

fn ir_of(text: &str) -> EssIr {
    let spec = admitted(text).unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

fn refused(text: &str) -> String {
    match admitted(text) {
        Ok(spec) => match compile(&spec, &SourceMap::new()) {
            Ok(_) => panic!("admitted, and should have been refused"),
            Err(errors) => format!("{errors:?}"),
        },
        Err(errors) => errors,
    }
}

#[test]
fn a_condition_is_admitted_from_ess_22_and_resolves_beside_the_cause() {
    let ir = ir_of(MODEL);
    let binding = ir
        .bindings()
        .values()
        .find(|binding| binding.name.as_str() == "received")
        .expect("compiled");
    let json = serde_json::to_value(binding).expect("serialises");
    assert_eq!(json["event"], "demo.messages.MessageReceived", "{json:#}");
    let condition = &json["where"];
    assert!(
        condition.is_object(),
        "the condition is in the IR: {json:#}"
    );
    assert_eq!(
        condition["reads"]["event.kind"]["leaf"]["variants"],
        serde_json::json!(["note", "ship"]),
        "{json:#}"
    );
    assert_eq!(
        condition["present"],
        serde_json::json!(["event.kind", "event.order"]),
        "{json:#}"
    );
}

#[test]
fn a_binding_without_a_condition_keeps_its_bytes() {
    let ir = ir_of(MODEL);
    let logged = ir
        .bindings()
        .values()
        .find(|binding| binding.name.as_str() == "logged")
        .expect("compiled");
    let json = serde_json::to_value(logged).expect("serialises");
    assert!(json.get("where").is_none(), "{json:#}");
}

#[test]
fn a_condition_below_ess_22_is_refused_naming_ess_22() {
    let errors = refused(&MODEL.replace("format: ess/22", "format: ess/21"));
    assert!(errors.contains("ess/22"), "{errors}");
    assert!(errors.contains("where"), "{errors}");
}

#[test]
fn optional_presence_proved_by_the_condition_admits_a_required_input() {
    // #194: without the condition, `event.order.id` may be unavailable for a required String.
    let errors = refused(&MODEL.replace(WHERE, ""));
    assert!(errors.contains("event.order.id"), "{errors}");
    // With only the discriminator, nothing proves `order` present: still refused.
    let errors = refused(&MODEL.replace(WHERE, "      where: event.kind == ship\n"));
    assert!(errors.contains("event.order.id"), "{errors}");
    // `defined(event.order)` under `any` proves nothing: either side may hold.
    let errors = refused(&MODEL.replace(
        WHERE,
        "      where: {any: [defined(event.order), event.kind == ship]}\n",
    ));
    assert!(errors.contains("event.order.id"), "{errors}");
    // `not missing(...)` is `defined(...)`.
    ir_of(&MODEL.replace(WHERE, "      where: {not: missing(event.order)}\n"));
}

#[test]
fn a_parent_proved_present_does_not_prove_its_optional_child() {
    let child = MODEL
        .replace(
            "      - {name: order_id, type: String}\n    outcomes:\n      - name: messaged",
            "      - {name: order_id, type: String}\n      - {name: note, type: String}\n    outcomes:\n      - name: messaged",
        )
        .replace(
            "      order_id: event.order.id\n",
            "      order_id: event.order.id\n      note: event.order.note\n",
        );
    let errors = refused(&child);
    assert!(errors.contains("event.order.note"), "{errors}");
    ir_of(&child.replace(
        WHERE,
        "      where: [defined(event.order.note), event.kind == ship]\n",
    ));
}

#[test]
fn a_condition_outside_the_bounded_vocabulary_or_the_event_is_refused() {
    for (written, needle) in [
        ("      where: input.kind == ship\n", "event"),
        ("      where: event.nothing == ship\n", "nothing"),
        ("      where: event.kind == shipped\n", "shipped"),
        (
            "      where: {event.order.id: {starts_with: A}}\n",
            "bounded",
        ),
        ("      where: event.order == ship\n", "String or enum"),
    ] {
        let errors = refused(&MODEL.replace(WHERE, written));
        assert!(errors.contains(needle), "{written}: {errors}");
    }
}

#[test]
fn a_periodic_cause_refuses_a_condition() {
    let text = PERIODIC
        .replace("format: ess/3", "format: ess/22")
        .replacen(
            "      periodic:\n",
            "      where: defined(event.anything)\n      periodic:\n",
            1,
        );
    let errors = refused(&text);
    assert!(
        errors.contains("periodic cause publishes no event"),
        "{errors}"
    );
}
