//! Two checks synthesis repeats for nothing, held to a work bound rather than a clock.
//!
//! A ranked view of `n` keys checks, for each key after the first, whether a creating command can
//! set every key before it: `n (n - 1) / 2 + n - 1` checks for a view of 24 keys, each rebuilding
//! the model's driver map. It is held to one driver map and one check per key.
//!
//! A shorthand on one unused struct made every scenario traverse the declared types from the
//! types it observes. The types are traversed once per run, and not at all for a scenario that
//! observes no type reaching a shorthand.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{
    ranked_steering_kept_equals_fresh, ranked_steering_work, shorthand_reach_kept_equals_fresh,
    shorthand_traversals, synthesize,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const KEYS: usize = 24;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ranked(keys: usize) -> String {
    let names: Vec<String> = (0..keys).map(|at| format!("flag_{at:02}")).collect();
    let fields: String = names
        .iter()
        .map(|name| format!("      - {{name: {name}, type: Integer}}\n"))
        .collect();
    let sets: Vec<String> = names
        .iter()
        .map(|name| format!("{name}: input.{name}"))
        .collect();
    let order: Vec<String> = names.iter().map(|name| format!("{name} asc")).collect();
    format!(
        "format: ess/32\nsystem: shop\nversion: v1\ndomain: shop.order\n\
         types:\n  - {{name: shop.order.OrderId, kind: newtype, of: Uuid}}\n\
         entities:\n  - name: shop.order.Order\n    identity: {{name: order_id, type: shop.order.OrderId}}\n    fields:\n{fields}\
         \x20   lifecycle: {{initial: Placed, states: [Placed], terminal: [Placed], transitions: []}}\n\
         events:\n  - name: shop.order.OrderPlaced\n    fields: [{{name: order_id, type: shop.order.OrderId}}]\n\
         commands:\n  - name: shop.order.PlaceOrder\n    input:\n{fields}\
         \x20   outcomes:\n      - name: placed\n        creates: shop.order.Order\n        instance: order_id\n\
         \x20       emits: [shop.order.OrderPlaced]\n        payload:\n          shop.order.OrderPlaced: {{order_id: {{generated: true}}}}\n\
         \x20       sets: {{{}}}\n\
         views:\n  - name: shop.order.OrderQueue\n    source: shop.order.Order\n    consistency: read_your_writes\n\
         \x20   order_by: [{}]\n    fields:\n      - {{name: order_id, type: shop.order.OrderId}}\n{fields}",
        sets.join(", "),
        order.join(", ")
    )
}

/// Driver maps built and keys checked for the whole synthesis: 1 and 24 when each key is checked
/// once; 299 and 299 when each later key checks every earlier one anew.
#[test]
fn a_ranked_view_checks_each_key_once() {
    let ir = ir(&ranked(KEYS));
    let (synthesis, (maps, checks)) = ranked_steering_work(|| synthesize(&ir));
    assert!(
        !synthesis.suite.scenarios.is_empty(),
        "a scenario was synthesized"
    );
    assert_eq!(
        (maps, checks),
        (1, KEYS as u64),
        "driver maps built and keys checked for steerability"
    );
    assert!(
        ranked_steering_kept_equals_fresh(&ir),
        "a kept answer is a fresh one"
    );
}

fn layered(unused_shorthand: bool, commands: usize) -> String {
    let shorthand = if unused_shorthand {
        "    shorthand: tag\n"
    } else {
        ""
    };
    let mut text = format!(
        "format: ess/27\nsystem: demo\nversion: v1\ndomain: demo.work\ntypes:\n  - name: demo.work.Unused\n    kind: struct\n{shorthand}    fields: [{{name: tag, type: String}}]\n  - {{name: demo.work.Layer0, kind: struct, fields: [{{name: member, type: String}}]}}\n"
    );
    for at in 1..4 {
        text += &format!(
            "  - {{name: demo.work.Layer{at}, kind: struct, fields: [{{name: member, type: demo.work.Layer{}}}]}}\n",
            at - 1
        );
    }
    text += "events: [{name: demo.work.Shared, fields: [{name: body, type: demo.work.Layer3}]}]\ncommands:\n";
    for at in 0..commands {
        text += &format!(
            "  - name: demo.work.Do{at}\n    input: [{{name: body, type: demo.work.Layer3}}]\n    outcomes: [{{name: done, emits: [demo.work.Shared], payload: {{demo.work.Shared: {{body: input.body}}}}}}]\n"
        );
    }
    text
}

/// Traversals of the declared types for the whole synthesis: 1 with a shorthand somewhere in the
/// model, however many scenarios observe other types, where each scenario traversed on its own.
#[test]
fn an_unused_shorthand_traverses_the_types_once() {
    let plain = ir(&layered(false, 8));
    let (without, none) = shorthand_traversals(|| synthesize(&plain));
    assert_eq!(none, 0, "no shorthand, no traversal");

    let model = ir(&layered(true, 8));
    let (with, traversals) = shorthand_traversals(|| synthesize(&model));
    assert!(
        with.suite.scenarios.len() >= 8,
        "scenarios: {}",
        with.suite.scenarios.len()
    );
    assert_eq!(traversals, 1, "traversals of the declared types");
    assert_eq!(
        with.suite.scenarios.keys().collect::<Vec<_>>(),
        without.suite.scenarios.keys().collect::<Vec<_>>(),
        "the unused shorthand withdraws no scenario"
    );
    assert!(
        shorthand_reach_kept_equals_fresh(&model),
        "the kept reach is a fresh one"
    );
}
