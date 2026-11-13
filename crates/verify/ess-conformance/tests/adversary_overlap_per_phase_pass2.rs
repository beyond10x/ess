//! Adversary pass 2 on story:overlap-witnesses-per-phase (beyond10x/ess#472), against fix1
//! (`1555aeb53`): the `Equivalent` score a disjoint stored pair gets.
use std::fmt::Write as _;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn audit(text: &str) -> mutate::MutationReport {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    let files = vec![(Source::new("fixture.yaml"), raw)];
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(
        &files,
        &texts,
        &[mutate::MutantClass::PrecedenceSwap],
        || Interpreted::for_model(ir.clone()),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdict_of(report: &mutate::MutationReport, id: &str) -> mutate::Verdict {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no site {id}:\n{}", report.render_text()))
        .verdict
}

/// `adversary_overlap_per_phase_pass1.rs`'s model C: `Place` stores `quantity`, `discount` and
/// `locked` from its input, and `Ship` declares `refusals` in order, then `shipped`.
fn order_model(format: &str, refusals: &[(&str, &str, &str)]) -> String {
    let mut ship = String::new();
    for (name, predicate, error) in refusals {
        write!(
            ship,
            "      - name: {name}\n        when_subject: {{predicate: {predicate}}}\n        error: demo.order.{error}\n"
        )
        .unwrap();
    }
    format!(
        r"format: {format}
system: demo
version: v1
domain: demo.order
summary: Orders refused for two conditions their stored fields can hold together.
types:
  - {{name: demo.order.OrderId, kind: newtype, of: Uuid}}
entities:
  - name: demo.order.Order
    identity: {{name: order_id, type: demo.order.OrderId}}
    fields:
      - {{name: quantity, type: Integer}}
      - {{name: discount, type: Integer}}
      - {{name: locked, type: Boolean}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - name: demo.order.Agent
    may: [demo.order.Place, demo.order.Ship]
errors:
  - {{name: demo.order.Locked, fields: []}}
  - {{name: demo.order.Empty, fields: []}}
events:
  - name: demo.order.Placed
    fields: [{{name: order_id, type: demo.order.OrderId}}]
  - name: demo.order.Changed
    fields: [{{name: order_id, type: demo.order.OrderId}}]
commands:
  - name: demo.order.Place
    input:
      - {{name: quantity, type: Integer}}
      - {{name: discount, type: Integer}}
      - {{name: locked, type: Boolean}}
    outcomes:
      - name: placed
        creates: demo.order.Order
        instance: order_id
        sets: {{quantity: input.quantity, discount: input.discount, locked: input.locked}}
        emits: [demo.order.Placed]
        payload: {{demo.order.Placed: {{order_id: {{generated: true}}}}}}
  - name: demo.order.Ship
    input:
      - {{name: order_id, type: demo.order.OrderId}}
    outcomes:
{ship}      - name: shipped
        updates: demo.order.Order
        instance: order_id
        sets: {{locked: true}}
        emits: [demo.order.Changed]
        payload: {{demo.order.Changed: {{order_id: input.order_id}}}}
components:
  - component: orders
    owns: {{domains: [demo.order]}}
    accepts: {{commands: [demo.order.Place, demo.order.Ship]}}
    reached_by: network
"
    )
}

/// `quantity > 9007199254740992` (2^53, an `Integer` literal well inside `i64`) beside
/// `locked == true`: a row `Place` stores with `quantity: 9007199254740993, locked: true` holds
/// both, and there `empty` answers and the swapped model answers `locked`. `number_regions` tries
/// the region above the literal at `high + 1.0` in `f64`, which rounds back to 2^53, so
/// `disjoint_on_rows` never tries a value above it and calls the pair disjoint: the swap is scored
/// `equivalent` and never run, though the changelog says a pair some row satisfies is `killed` or
/// `unwitnessed`. At `86fea5ad1` an ordering comparison was not `plain`, so the pair was not
/// called disjoint.
#[test]
fn pair_above_two_to_the_53_is_not_scored_equivalent() {
    let model = order_model(
        "ess/20",
        &[
            ("empty", "quantity > 9007199254740992", "Empty"),
            ("locked", "locked == true", "Locked"),
        ],
    );
    let report = audit(&model);
    let swap = verdict_of(&report, "precedence-swap/demo.order.Ship/empty/locked");
    assert_ne!(
        swap,
        mutate::Verdict::Equivalent,
        "a row with quantity 9007199254740993 and locked true selects both refusals, so their swap \
         is not equivalent:\n{}",
        report.render_text()
    );
}
