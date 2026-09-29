//! Adversary pass 1 on beyond10x/ess#202: every `sets-retarget` mutant of a branch writing several
//! same-typed inputs gets a suite whose required row the unmutated specification contradicts.
//!
//! The oracle is the one `connective_and_source_mutants.rs` uses, generalised: after a scenario
//! sends `command` with literal inputs, every row it then requires in the view is compared with what
//! the *original* specification writes, `target := input[original source]`. A required row that
//! differs is one every implementation of the original fails, so the mutant is killed.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::scenario::{ScenarioStep, ViewExpectation};
use ess_conformance::synthesize::synthesize;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled((files, texts): &(Vec<Document>, SourceMap)) -> EssIr {
    mutate::compile(files.clone(), texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn retargets(model: &(Vec<Document>, SourceMap)) -> Vec<(String, EssIr)> {
    let (files, texts) = model;
    mutate::mutants(files, &[MutantClass::SetsRetarget])
        .into_iter()
        .map(|mutant| {
            let mutated = mutate::apply(files, &mutant.mutation).expect("the site exists");
            let ir = mutate::compile(mutated, texts).unwrap_or_else(|stillborn| {
                panic!(
                    "{} compiles: {} {}",
                    mutant.id, stillborn.code, stillborn.cause
                )
            });
            (mutant.id, ir)
        })
        .collect()
}

type Row = BTreeMap<String, Node>;

/// Every (literal input sent to `command`, row then required in `view`) pair in the suite.
fn sent_and_required(ir: &EssIr, command: &str, view: &str) -> Vec<(Row, Row)> {
    let synthesis = synthesize(ir);
    let mut out = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        let mut pending: Option<Row> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand {
                    command: sent,
                    input,
                    ..
                } => {
                    pending = (sent.to_string() == command).then(|| {
                        input
                            .iter()
                            .filter_map(|(field, value)| {
                                value
                                    .as_literal()
                                    .cloned()
                                    .map(|node| (field.clone(), node))
                            })
                            .collect()
                    });
                }
                ScenarioStep::ExpectView {
                    view: seen,
                    expectation: ViewExpectation::Contains { fields },
                }
                | ScenarioStep::EventuallyView {
                    view: seen,
                    expectation: ViewExpectation::Contains { fields },
                    ..
                } if seen.to_string() == view => {
                    if let Some(input) = &pending {
                        let row: Row = fields
                            .iter()
                            .filter_map(|(field, value)| {
                                value
                                    .as_literal()
                                    .cloned()
                                    .map(|node| (field.clone(), node))
                            })
                            .collect();
                        out.push((input.clone(), row));
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// The mutants whose suite holds no row the original contradicts, each with what it sent.
fn survivors(text: &str, command: &str, view: &str, original: &[(&str, &str)]) -> Vec<String> {
    let model = documents(text);
    compiled(&model);
    let mutants = retargets(&model);
    assert!(!mutants.is_empty(), "the audit finds retarget sites");
    let mut out = Vec::new();
    for (id, mutated) in mutants {
        let pairs = sent_and_required(&mutated, command, view);
        assert!(
            !pairs.is_empty(),
            "`{id}`: the suite sends `{command}` and then requires a `{view}` row"
        );
        let killed = pairs.iter().any(|(input, row)| {
            original.iter().any(
                |(target, source)| match (row.get(*target), input.get(*source)) {
                    (Some(held), Some(sent)) => held != sent,
                    _ => false,
                },
            )
        });
        if !killed {
            let flags: Vec<String> = pairs
                .iter()
                .map(|(input, row)| {
                    let shown = |map: &Row| {
                        original
                            .iter()
                            .map(|(target, source)| {
                                format!(
                                    "{target}<-{source}:{:?}/{:?}",
                                    input.get(*source),
                                    map.get(*target)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    };
                    shown(row)
                })
                .collect();
            out.push(format!(
                "{id}: {} rows [{}]",
                pairs.len(),
                flags.join(" | ")
            ));
        }
    }
    out
}

// ---- 1. names that do not line up ----------------------------------------------------------------

/// Three Booleans written to three fields whose names are not the inputs' names: option (a) finds
/// no target naming an input, so nothing is pinned and #202 is back.
const UNALIGNED: &str = r"
format: ess/13
system: shop
version: v1
domain: shop.plan

entities:
  - name: shop.plan.Plan
    identity: {name: plan_id, type: String}
    fields:
      - {name: active, type: Boolean}
      - {name: can_cancel, type: Boolean}
      - {name: can_extend, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: shop.plan.PlanOpened
    fields:
      - {name: plan_id, type: String}

views:
  - name: shop.plan.Plans
    source: shop.plan.Plan
    consistency: read_your_writes
    fields:
      - {name: plan_id, type: String}
      - {name: active, type: Boolean}
      - {name: can_cancel, type: Boolean}
      - {name: can_extend, type: Boolean}

commands:
  - name: shop.plan.OpenPlan
    input:
      - {name: plan_id, type: String}
      - {name: is_active, type: Boolean}
      - {name: cancellable, type: Boolean}
      - {name: extendable, type: Boolean}
    outcomes:
      - name: opened
        creates: shop.plan.Plan
        instance: plan_id
        sets: {active: input.is_active, can_cancel: input.cancellable, can_extend: input.extendable}
        emits: [shop.plan.PlanOpened]
        payload:
          shop.plan.PlanOpened: {plan_id: input.plan_id}
";

#[test]
fn retargets_are_killed_when_field_names_differ_from_input_names() {
    let left = survivors(
        UNALIGNED,
        "shop.plan.OpenPlan",
        "shop.plan.Plans",
        &[
            ("active", "is_active"),
            ("can_cancel", "cancellable"),
            ("can_extend", "extendable"),
        ],
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 2. a declared rotation: pins from the unmutated part crowd out the mutated pair ------------

const ROTATED: &str = r"
format: ess/13
system: shop
version: v1
domain: shop.dial

entities:
  - name: shop.dial.Dial
    identity: {name: dial_id, type: String}
    fields:
      - {name: a, type: Boolean}
      - {name: b, type: Boolean}
      - {name: c, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: shop.dial.Rotated
    fields:
      - {name: dial_id, type: String}

views:
  - name: shop.dial.Dials
    source: shop.dial.Dial
    consistency: read_your_writes
    fields:
      - {name: dial_id, type: String}
      - {name: a, type: Boolean}
      - {name: b, type: Boolean}
      - {name: c, type: Boolean}

commands:
  - name: shop.dial.Rotate
    input:
      - {name: dial_id, type: String}
      - {name: a, type: Boolean}
      - {name: b, type: Boolean}
      - {name: c, type: Boolean}
    outcomes:
      - name: rotated
        creates: shop.dial.Dial
        instance: dial_id
        sets: {a: input.b, b: input.c, c: input.a}
        emits: [shop.dial.Rotated]
        payload:
          shop.dial.Rotated: {dial_id: input.dial_id}
";

#[test]
fn retargets_of_a_declared_rotation_are_killed() {
    let left = survivors(
        ROTATED,
        "shop.dial.Rotate",
        "shop.dial.Dials",
        &[("a", "b"), ("b", "c"), ("c", "a")],
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 3. a branch the stored-row search arranges ---------------------------------------------------

const GUARDED_UPDATE: &str = r"
format: ess/13
system: shop
version: v1
domain: shop.order

entities:
  - name: shop.order.Order
    identity: {name: order_id, type: String}
    fields:
      - {name: locked, type: Boolean}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

errors:
  - name: shop.order.Locked
    fields: []

events:
  - name: shop.order.OrderChanged
    fields:
      - {name: order_id, type: String}

views:
  - name: shop.order.Orders
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: String}
      - {name: state, type: shop.order.Order.State}
      - {name: locked, type: Boolean}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}

commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: order_id, type: String}
      - {name: locked, type: Boolean}
    outcomes:
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {locked: input.locked, paid: 'false', gift: 'false', rush: 'false'}
        emits: [shop.order.OrderChanged]
        payload:
          shop.order.OrderChanged: {order_id: input.order_id}
  - name: shop.order.SetFlags
    input:
      - {name: order_id, type: String}
      - {name: paid, type: Boolean}
      - {name: gift, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - name: locked
        when_subject:
          predicate: locked == true
        error: shop.order.Locked
      - name: flagged
        updates: shop.order.Order
        instance: order_id
        sets: {paid: input.paid, gift: input.gift, rush: input.rush}
        emits: [shop.order.OrderChanged]
        payload:
          shop.order.OrderChanged: {order_id: input.order_id}
";

#[test]
fn retargets_are_killed_on_a_branch_the_stored_row_search_arranges() {
    let left = survivors(
        GUARDED_UPDATE,
        "shop.order.SetFlags",
        "shop.order.Orders",
        &[("paid", "paid"), ("gift", "gift"), ("rush", "rush")],
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 4. the command an event invokes -------------------------------------------------------------

const EVENT_FED: &str = r"
format: ess/13
system: chat
version: v1
domain: chat.room

entities:
  - name: chat.room.Member
    identity: {name: member_id, type: String}
    fields:
      - {name: is_me, type: Boolean}
      - {name: moderator, type: Boolean}
      - {name: on_hold, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: chat.room.Joined
    fields:
      - {name: member_id, type: String}
      - {name: is_me, type: Boolean}
      - {name: moderator, type: Boolean}
      - {name: on_hold, type: Boolean}
  - name: chat.room.Recorded
    fields:
      - {name: member_id, type: String}

views:
  - name: chat.room.Members
    source: chat.room.Member
    consistency: read_your_writes
    fields:
      - {name: member_id, type: String}
      - {name: is_me, type: Boolean}
      - {name: moderator, type: Boolean}
      - {name: on_hold, type: Boolean}

commands:
  - name: chat.room.Join
    input:
      - {name: member_id, type: String}
      - {name: is_me, type: Boolean}
      - {name: moderator, type: Boolean}
      - {name: on_hold, type: Boolean}
    outcomes:
      - name: joined
        emits: [chat.room.Joined]
        payload:
          chat.room.Joined:
            member_id: input.member_id
            is_me: input.is_me
            moderator: input.moderator
            on_hold: input.on_hold
  - name: chat.room.Record
    input:
      - {name: member_id, type: String}
      - {name: is_me, type: Boolean}
      - {name: moderator, type: Boolean}
      - {name: on_hold, type: Boolean}
    outcomes:
      - name: recorded
        creates: chat.room.Member
        instance: member_id
        sets: {is_me: input.is_me, moderator: input.moderator, on_hold: input.on_hold}
        emits: [chat.room.Recorded]
        payload:
          chat.room.Recorded: {member_id: input.member_id}

bindings:
  - id: record-on-joined
    when:
      event: chat.room.Joined
    invoke:
      command: chat.room.Record
    mapping:
      member_id: event.member_id
      is_me: event.is_me
      moderator: event.moderator
      on_hold: event.on_hold
    delivery: at_least_once
    on_failure: retry
";

#[test]
fn retargets_are_killed_on_a_command_an_event_invokes() {
    let left = survivors(
        EVENT_FED,
        "chat.room.Record",
        "chat.room.Members",
        &[
            ("is_me", "is_me"),
            ("moderator", "moderator"),
            ("on_hold", "on_hold"),
        ],
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 5. a two-variant enum, three inputs ---------------------------------------------------------

const TWO_VARIANTS: &str = r"
format: ess/13
system: shop
version: v1
domain: shop.order
types:
  - name: shop.order.Switch
    kind: enum
    variants: [On, Off]

entities:
  - name: shop.order.Order
    identity: {name: order_id, type: String}
    fields:
      - {name: paid, type: shop.order.Switch}
      - {name: gift, type: shop.order.Switch}
      - {name: rush, type: shop.order.Switch}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: shop.order.OrderPlaced
    fields:
      - {name: order_id, type: String}

views:
  - name: shop.order.Orders
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: String}
      - {name: paid, type: shop.order.Switch}
      - {name: gift, type: shop.order.Switch}
      - {name: rush, type: shop.order.Switch}

commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: order_id, type: String}
      - {name: paid, type: shop.order.Switch}
      - {name: gift, type: shop.order.Switch}
      - {name: rush, type: shop.order.Switch}
    outcomes:
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {paid: input.paid, gift: input.gift, rush: input.rush}
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced: {order_id: input.order_id}
";

#[test]
fn retargets_are_killed_among_three_inputs_of_a_two_variant_enum() {
    let left = survivors(
        TWO_VARIANTS,
        "shop.order.PlaceOrder",
        "shop.order.Orders",
        &[("paid", "paid"), ("gift", "gift"), ("rush", "rush")],
    );
    assert!(left.is_empty(), "retargets survive: {left:#?}");
}

// ---- 6. determinism ------------------------------------------------------------------------------

#[test]
fn a_retarget_mutant_suite_is_the_same_on_every_synthesis() {
    let model = documents(TWO_VARIANTS);
    for (id, mutated) in retargets(&model) {
        let first = synthesize(&mutated).suite;
        let second = synthesize(&mutated).suite;
        assert_eq!(first, second, "`{id}` synthesizes one suite");
    }
}
