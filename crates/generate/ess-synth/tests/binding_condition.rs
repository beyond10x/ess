//! A binding's event-payload condition (ess/22, beyond10x/ess#268) in generated runtimes: every
//! code target that delivers bindings evaluates it in the dispatch — before the transformation,
//! the invocation record and the port — instead of refusing the binding, and a required input the
//! condition proves present is checked rather than unwrapped (beyond10x/ess#194).
//! `tests/binding_condition_runtime.rs` runs the generated applications against the suite.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/binding-condition.yaml");
const COMPONENT: &str = "components:
  - component: messages-service
    summary: Receives messages, and records them against their order.
    owns:
      domains: [demo.messages]
    accepts:
      commands: [demo.messages.ReceiveMessage, demo.messages.MessageEvent, demo.messages.LogMessage]
    publishes:
      events: [demo.messages.MessageReceived, demo.messages.OrderMessaged, demo.messages.MessageLogged]
    reached_by: network
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("binding-condition.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn served(text: &str) -> EssIr {
    ir(&format!("{text}{COMPONENT}"))
}

fn code(ir: &EssIr, target: Target) -> String {
    synthesize_for(ir, target)
        .unwrap_or_else(|failure| panic!("{target:?} generates a conditioned binding: {failure:?}"))
        .artifacts
        .values()
        .map(|artifact| artifact.contents.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn plain() -> String {
    MODEL
        .replace(
            "      where: [defined(event.order), event.kind == ship]\n",
            "",
        )
        .replace(
            "      order_id: event.order.id\n",
            "      order_id: event.message_id\n",
        )
}

#[test]
fn every_code_target_evaluates_the_condition_instead_of_refusing_it() {
    let ir = served(MODEL);
    let rust = code(&ir, Target::Rust);
    assert!(rust.contains("pub mod conditions {"), "{rust}");
    assert!(
        rust.contains("match conditions::received(event) {"),
        "{rust}"
    );
    assert!(
        rust.contains("Some(false) => return Ok(()),"),
        "false skips this binding alone: {rust}"
    );
    assert!(rust.contains("capability: \"binding condition\""), "{rust}");
    assert!(
        rust.contains("-> Option<demo_types::messages::MessageEvent>"),
        "the proved member is checked: {rust}"
    );
    let go = code(&ir, Target::Go);
    assert!(
        go.contains("func ReceivedCondition(event messages.MessageReceived) int8 {"),
        "{go}"
    );
    assert!(go.contains("switch ReceivedCondition(event) {"), "{go}");
    assert!(go.contains("Capability: \"binding condition\""), "{go}");
    assert!(go.contains("(messages.MessageEvent, bool)"), "{go}");
}

#[test]
fn every_direct_workspace_entry_that_delivers_bindings_accepts_the_condition() {
    let ir = served(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    for (target, emitted) in [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
    ] {
        assert!(
            emitted.is_none(),
            "{target} generates a conditioned binding: {:?}",
            emitted.map(|failure| failure.to_canonical_json())
        );
    }
    let web = code(&ir, Target::Web);
    assert!(
        web.contains("\"where\""),
        "the web catalogue names the condition: {web}"
    );
}

#[test]
fn the_same_bindings_without_a_condition_generate_no_condition() {
    let ir = served(&plain());
    for target in [Target::Rust, Target::Go, Target::Web] {
        let code = code(&ir, target);
        for absent in [
            "conditions::",
            "pub mod conditions",
            "Condition(event",
            "binding condition",
            "\"where\"",
        ] {
            assert!(!code.contains(absent), "{target:?} writes `{absent}`");
        }
    }
}

#[test]
fn a_condition_with_no_component_to_deliver_to_is_not_refused() {
    for target in [Target::Rust, Target::Go] {
        if let Err(failure) = synthesize_for(&ir(MODEL), target) {
            let text = format!("{failure:?}");
            assert!(!text.contains("when.where"), "{target:?}: {text}");
        }
    }
}

/// A selection binding that fills a required input from an Optional member its condition proves
/// present (beyond10x/ess#194) is generated like any other conditioned binding: no target refuses
/// it, and the Rust transformation answers absent rather than unwrapping.
#[test]
fn a_selection_binding_reading_a_proved_member_is_generated_not_refused() {
    let rewrite = |text: &str, from: &str, to: &str| {
        assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
        text.replacen(from, to, 1)
    };
    let text = include_str!("fixtures/binding-selection.yaml");
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
        "      event: selection.core.Arrived\n      where: defined(event.tag)\n    invoke:",
    );
    let text = rewrite(
        &text,
        "      external_id: {selection: external, path: [id]}\n",
        "      external_id: {selection: external, path: [id]}\n      tag: event.tag\n",
    );
    let ir = ir(&text);
    for target in [Target::Rust, Target::Go, Target::Web] {
        if let Err(failure) = synthesize_for(&ir, target) {
            let text = format!("{failure:?}");
            assert!(!text.contains("when.where"), "{target:?}: {text}");
        }
    }
    let rust = code(&ir, Target::Rust);
    assert!(rust.contains("Result<Option<"), "{rust}");
    assert!(rust.contains("None => return Ok(None)"), "{rust}");
}

/// A binding on an event an external channel delivers (ess/18), conditioned or not, generates with
/// its delivery a named obligation: nothing in the system publishes the event.
#[test]
fn an_external_binding_owes_its_delivery_by_name() {
    let inbox =
        include_str!("../../../verify/ess-conformance/tests/fixtures/delivery-context.yaml");
    let conditioned = inbox
        .replacen("format: ess/18\n", "format: ess/22\n", 1)
        .replacen(
            "      - {name: from, type: String}\n",
            "      - {name: from, type: String}\n      - {name: urgent, type: Optional<String>}\n",
            1,
        )
        .replacen(
            "      context_authority: account-messages\n",
            "      context_authority: account-messages\n      where: defined(event.urgent)\n",
            1,
        );
    for text in [inbox.to_owned(), conditioned] {
        let ir = ir(&text);
        let rust = code(&ir, Target::Rust);
        assert!(rust.contains("pub trait ReceivedDelivery {"), "{rust}");
        assert!(
            rust.contains("capability: \"binding delivery\", source: \"received\""),
            "{rust}"
        );
        let go = code(&ir, Target::Go);
        assert!(go.contains("type ReceivedDelivery interface {"), "{go}");
    }
}
