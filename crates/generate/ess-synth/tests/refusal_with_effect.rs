//! The generated Rust and Go lanes and a refusal that declares its compensating change
//! (`compensates: true`, ess/22, beyond10x/ess#197, `docs/design/refusal-with-effect.md`): neither
//! lane executes a refusal that changes its row, so the command's behaviour stays owed, naming the
//! marker. The same command with a plain refusal is generated.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/refusal-with-effect.yaml");

/// A second command taking `reset`, so the model without the compensating move still gives the
/// transition a cause.
const CLOSE: &str = "  - name: shop.order.CloseOrder\n    input:\n      - {name: order_id, type: shop.order.OrderId}\n    outcomes:\n      - name: reset\n        moves: shop.order.Order.reset\n        instance: order_id\n        emits: [shop.order.OrderJoined]\n        payload:\n          shop.order.OrderJoined: {order_id: input.order_id}\n      - name: closed\n        wrong_state: true\n        error: shop.order.Closed\nevents:\n";

const EFFECT: &str = "        compensates: true\n        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n";

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("order.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn disposition(text: &str, target: Target) -> String {
    let synthesis = synthesize_for(&ir_of(text), target)
        .unwrap_or_else(|error| panic!("{target:?} synthesizes: {error:?}"));
    match synthesis
        .plan
        .disposition_of(CapabilityKind::CommandBehavior, "shop.order.JoinOrder")
    {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            serde_json::to_string(&obligation.reason).unwrap()
        }
        other => format!("{other:?}"),
    }
}

#[test]
fn a_compensating_refusal_keeps_the_behaviour_owed_by_name_in_both_lanes() {
    let compensating = MODEL.replace("events:\n", CLOSE);
    let plain = compensating.replace(EFFECT, "");
    assert_ne!(plain, compensating);
    for target in [Target::Rust, Target::Go] {
        let owed = disposition(&compensating, target);
        assert!(
            owed.contains("compensates: true") && owed.contains("failed"),
            "{target:?}: {owed}"
        );
        // Without the marker the refusal is an ordinary external one, and the reason, if any, is
        // not about compensation.
        let without = disposition(&plain, target);
        assert!(!without.contains("compensates"), "{target:?}: {without}");
    }
}
