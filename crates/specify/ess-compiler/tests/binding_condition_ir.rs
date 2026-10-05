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

// ---- the presence proof agrees between domain admission and compilation (beyond10x/ess#194) ----

/// `Ok(())` where the domain admits and the compiler compiles `text`; `Err` naming the stage and
/// the errors where either refuses. The two stages must agree: a model the domain admits compiles.
fn stages(text: &str) -> Result<(), String> {
    let spec = admitted(text).map_err(|errors| format!("domain: {errors}"))?;
    compile(&spec, &SourceMap::new())
        .map(|_| ())
        .map_err(|errors| format!("compiler: {errors:?}"))
}

fn rewrite(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
    text.replacen(from, to, 1)
}

/// `MessageEvent` takes the order itself, required, copied from the Optional `event.order`.
fn copied(condition: &str) -> String {
    let text = rewrite(
        MODEL,
        "      - {name: order_id, type: String}\n    outcomes:\n      - name: messaged",
        "      - {name: order_id, type: String}\n      - {name: order, type: demo.messages.Ref}\n    outcomes:\n      - name: messaged",
    );
    let text = rewrite(
        &text,
        "      order_id: event.order.id\n",
        "      order_id: event.order.id\n      order: event.order\n",
    );
    rewrite(&text, WHERE, condition)
}

#[test]
fn a_proved_optional_field_copied_into_a_required_input_compiles_where_the_domain_admits_it() {
    stages(&copied(WHERE)).unwrap_or_else(|why| panic!("{why}"));
    let ir = ir_of(&copied(WHERE));
    let binding = ir
        .bindings()
        .values()
        .find(|binding| binding.name.as_str() == "received")
        .unwrap();
    let json = serde_json::to_value(binding).unwrap();
    let order = json["mapping"]
        .as_array()
        .unwrap()
        .iter()
        .find(|mapping| mapping["target"] == "order")
        .unwrap_or_else(|| panic!("{json:#}"));
    assert!(
        order.get("conversion").is_none() || order["conversion"].is_null(),
        "{order:#}"
    );
    // Unproved, both stages refuse it.
    let unproved = stages(&copied("      where: event.kind == ship\n"));
    let why = unproved.expect_err("nothing proves the order present");
    assert!(why.contains("order"), "{why}");
}

/// The selection fixture of `ess-synth`, at ess/22, its event carrying an Optional `tag` that a
/// required input reads, proved present by the binding's condition.
fn selected(condition: &str) -> String {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml");
    let text = rewrite(text, "format: ess/3\n", "format: ess/22\n");
    let text = rewrite(
        &text,
        "        type: List<Optional<selection.core.Leg>>\n  - name: selection.core.Seen\n",
        "        type: List<Optional<selection.core.Leg>>\n      - name: tag\n        type: Optional<String>\n  - name: selection.core.Seen\n",
    );
    let text = rewrite(
        &text,
        "  - name: selection.core.Receive\n    input:\n",
        "  - name: selection.core.Receive\n    input:\n      - name: tag\n        type: String\n",
    );
    let text = rewrite(
        &text,
        "      event: selection.core.Arrived\n    invoke:",
        &format!("      event: selection.core.Arrived\n{condition}    invoke:"),
    );
    rewrite(
        &text,
        "      external_id: {selection: external, path: [id]}\n",
        "      external_id: {selection: external, path: [id]}\n      tag: event.tag\n",
    )
}

#[test]
fn a_selection_binding_reading_a_proved_optional_field_compiles_where_the_domain_admits_it() {
    stages(&selected("      where: defined(event.tag)\n")).unwrap_or_else(|why| panic!("{why}"));
    let why = stages(&selected("      where: true\n")).expect_err("nothing proves the tag present");
    assert!(why.contains("tag"), "{why}");
}

/// The condition reads the event payload only: a delivery-context field is never proved present
/// by it, even where the event has a member of the same name the condition does prove. The domain
/// refuses the Optional context field into a required input before the compiler runs, so this holds
/// the two stages to agreeing; `resolve::tests::a_condition_proves_no_delivery_context_field_present_in_the_resolver`
/// holds the compiler's own `mapped_context` to the same refusal.
#[test]
fn a_condition_never_proves_a_delivery_context_field_present() {
    let inbox =
        include_str!("../../../verify/ess-conformance/tests/fixtures/delivery-context.yaml");
    let text = rewrite(inbox, "format: ess/18\n", "format: ess/22\n");
    let text = rewrite(
        &text,
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: tag, type: Optional<String>}\n",
    );
    let text = rewrite(
        &text,
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: tag, type: Optional<String>}\n",
    );
    let text = rewrite(
        &text,
        "      - {name: peer, type: String}\n",
        "      - {name: peer, type: String}\n      - {name: tag, type: String}\n",
    );
    let text = rewrite(
        &text,
        "      context_authority: account-messages\n",
        "      context_authority: account-messages\n      where: defined(event.tag)\n",
    );
    let text = rewrite(
        &text,
        "      peer: event.from\n",
        "      peer: event.from\n      tag: context.tag\n",
    );
    let why = stages(&text).expect_err("a context field is never proved present");
    assert!(why.contains("tag"), "{why}");
    // The same input read from the event's own proved member is admitted by both.
    stages(&text.replace("      tag: context.tag\n", "      tag: event.tag\n"))
        .unwrap_or_else(|why| panic!("{why}"));
}
