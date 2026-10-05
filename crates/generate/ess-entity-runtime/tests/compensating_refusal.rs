//! A refusal that declares its compensating change (`compensates: true`, ess/22,
//! beyond10x/ess#197, `docs/design/refusal-with-effect.md`) at Entity Runtime lowering.
//!
//! An entity-core refusal changes nothing, so a branch answering an error after changing its row
//! is refused by name (`CompensatingRefusalUnsupported`) rather than lowered as a refusal that
//! skips the change. The same command with a plain refusal earns no such refusal.
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringCode, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

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

fn lowered(ir: &EssIr) -> Vec<(LoweringCode, String, String)> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("order-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("shop.order.Order").unwrap(),
            NonZeroU32::new(1).unwrap(),
        )]
        .into_iter()
        .collect(),
        scales: BTreeMap::new(),
    };
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .into_vec()
            .into_iter()
            .map(|d| (d.code, d.path.clone(), d.message.clone()))
            .collect(),
    }
}

#[test]
fn a_compensating_refusal_is_refused_by_name() {
    let refused = lowered(&ir_of(&MODEL.replace("events:\n", CLOSE)));
    assert!(
        refused.iter().any(|(code, path, message)| {
            *code == LoweringCode::CompensatingRefusalUnsupported
                && path.contains("shop.order.JoinOrder.failed")
                && message.contains("compensates")
        }),
        "{refused:#?}"
    );
}

#[test]
fn the_same_refusal_without_the_change_earns_no_such_refusal() {
    let plain = MODEL.replace("events:\n", CLOSE).replace(EFFECT, "");
    let refused = lowered(&ir_of(&plain));
    assert!(
        refused
            .iter()
            .all(|(code, _, _)| *code != LoweringCode::CompensatingRefusalUnsupported),
        "{refused:#?}"
    );
}
