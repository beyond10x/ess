//! Synthesis leaves an `Optional` input absent unless the scenario needs it present
//! (<https://github.com/beyond10x/ess/issues/546>).
//!
//! The model is a queue: `Submit` takes the instant an item was queued and an optional deadline,
//! and stores both; `Withdraw` moves a queued item. No guard reads the deadline. Up to 0.59.0 every
//! synthesized `Submit` step, the arranging ones included, carried the deadline, and carried it at
//! the same literal as the queue instant: every synthesized item claimed to expire the moment it
//! was queued, and an implementation that expires items by their deadline failed scenarios that
//! never mention expiry.
//!
//! Where a scenario does need the deadline — the branch it observes copies it, a guard reads it, or
//! a later step reads the stored copy — it is sent, and not at the queue instant's literal. Every
//! other step, and every step arranging an item for another branch, leaves it out.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/24
system: demo
version: v1
domain: demo.queue
types:
  - {name: demo.queue.ItemId, kind: newtype, of: Uuid}
errors:
  - {name: demo.queue.NotQueued, summary: The item is not queued., fields: []}
  - {name: demo.queue.NoDeadline, summary: The item has no deadline., fields: []}
entities:
  - name: demo.queue.Item
    identity: {name: item_id, type: demo.queue.ItemId}
    fields:
      - {name: queued_at, type: Timestamp}
      - {name: expires_at, type: Optional<Timestamp>}
    lifecycle:
      initial: Queued
      states: [Queued, Withdrawn]
      terminal: [Withdrawn]
      transitions:
        - {name: withdraw, from: [Queued], to: Withdrawn}
events:
  - name: demo.queue.Submitted
    fields: [{name: item_id, type: demo.queue.ItemId}]
  - name: demo.queue.Withdrawn
    fields: [{name: item_id, type: demo.queue.ItemId}]
actors:
  - {name: demo.queue.Client, may: [demo.queue.Submit, demo.queue.Withdraw]}
commands:
  - name: demo.queue.Submit
    input:
      - {name: queued_at, type: Timestamp}
      - {name: expires_at, type: Optional<Timestamp>}
    outcomes:
      - name: submitted
        creates: demo.queue.Item
        instance: item_id
        emits: [demo.queue.Submitted]
        payload:
          demo.queue.Submitted: {item_id: {generated: true}}
        sets:
          queued_at: input.queued_at
          expires_at: input.expires_at
  - name: demo.queue.Withdraw
    input:
      - {name: item_id, type: demo.queue.ItemId}
    outcomes:
      - name: withdrawn
        moves: demo.queue.Item.withdraw
        instance: item_id
        emits: [demo.queue.Withdrawn]
        payload:
          demo.queue.Withdrawn: {item_id: input.item_id}
      - name: not-queued
        wrong_state: true
        error: demo.queue.NotQueued
views:
  - name: demo.queue.Items
    source: demo.queue.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.queue.ItemId}
      - {name: queued_at, type: Timestamp}
      - {name: expires_at, type: Optional<Timestamp>}
";

/// A later step reads the stored deadline: `Expire` expires an item only where it has one.
const EXPIRE: &str = r"  - name: demo.queue.Expire
    input:
      - {name: item_id, type: demo.queue.ItemId}
    outcomes:
      - name: no-deadline
        when_subject: {predicate: not defined(expires_at)}
        error: demo.queue.NoDeadline
      - name: expired
        moves: demo.queue.Item.expire
        instance: item_id
        emits: [demo.queue.Expired]
        payload:
          demo.queue.Expired: {item_id: input.item_id}
      - name: not-queued
        wrong_state: true
        error: demo.queue.NotQueued
views:
";

/// `Submit` itself reads the deadline: a submission carrying one is announced apart.
const GUARDED: &str = r"    outcomes:
      - name: with-deadline
        when: defined(expires_at)
        creates: demo.queue.Item
        instance: item_id
        emits: [demo.queue.Submitted]
        payload:
          demo.queue.Submitted: {item_id: {generated: true}}
        sets:
          queued_at: input.queued_at
          expires_at: input.expires_at
      - name: submitted
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model with `Expire` added: a later step that reads the stored deadline.
fn with_expire() -> String {
    let text = MODEL
        .replacen("views:\n", EXPIRE, 1)
        .replacen(
            "      states: [Queued, Withdrawn]\n      terminal: [Withdrawn]\n",
            "      states: [Queued, Withdrawn, Expired]\n      terminal: [Withdrawn, Expired]\n",
            1,
        )
        .replacen(
            "        - {name: withdraw, from: [Queued], to: Withdrawn}\n",
            "        - {name: withdraw, from: [Queued], to: Withdrawn}\n        \
             - {name: expire, from: [Queued], to: Expired}\n",
            1,
        )
        .replacen(
            "actors:\n",
            "  - name: demo.queue.Expired\n    fields: [{name: item_id, type: demo.queue.ItemId}]\n\
             actors:\n",
            1,
        )
        .replacen(
            "demo.queue.Withdraw]}",
            "demo.queue.Withdraw, demo.queue.Expire]}",
            1,
        );
    assert!(text.contains("demo.queue.Expire\n") && text.contains("to: Expired}"));
    text
}

/// The model with a `Submit` branch whose guard reads the deadline.
fn guarded() -> String {
    let text = MODEL.replacen("    outcomes:\n      - name: submitted\n", GUARDED, 1);
    assert!(text.contains("with-deadline"));
    text
}

/// Every `Submit` step of the suite, by scenario, with its literal input.
fn submits(result: &Synthesis) -> Vec<(String, BTreeMap<String, Node>)> {
    result
        .suite
        .scenarios
        .iter()
        .flat_map(|(id, scenario)| {
            scenario.steps.iter().filter_map(move |step| match step {
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.queue.Submit" =>
                {
                    let literal = input
                        .iter()
                        .filter_map(|(name, value)| match value {
                            ScenarioValue::Literal { value } => Some((name.clone(), value.clone())),
                            _ => None,
                        })
                        .collect();
                    Some((id.to_string(), literal))
                }
                _ => None,
            })
        })
        .collect()
}

fn statuses(model: &EssIr, result: &Synthesis) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn assert_all_pass(model: &EssIr, result: &Synthesis) {
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let failing: Vec<_> = statuses(model, result)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}

/// The `Submit` branch under test copies the deadline, so its own scenario sends it, apart from the
/// queue instant; every scenario that only arranges an item through `Submit` leaves it out.
#[test]
fn issue_546_an_optional_input_only_copied_is_absent_from_every_arranging_step() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let own = "demo.queue.Submit/outcome/submitted";
    let arranging: Vec<_> = steps
        .iter()
        .filter(|(scenario, _)| scenario != own)
        .collect();
    assert!(arranging.len() >= 3, "{steps:#?}");
    for (scenario, input) in &steps {
        assert!(
            input.contains_key("queued_at"),
            "{scenario}: the required input is sent: {input:?}"
        );
    }
    for (scenario, input) in arranging {
        assert!(
            !input.contains_key("expires_at"),
            "{scenario}: no guard, invariant or later step reads the deadline, and this scenario \
             does not observe `submitted`'s copy, so it is not sent: {input:?}"
        );
    }
    let observed: Vec<_> = steps
        .iter()
        .filter(|(scenario, _)| scenario == own)
        .collect();
    assert!(!observed.is_empty(), "{steps:#?}");
    for (_, input) in observed {
        let deadline = input
            .get("expires_at")
            .unwrap_or_else(|| panic!("`submitted` copies the deadline it observes: {input:?}"));
        assert_ne!(
            Some(deadline),
            input.get("queued_at"),
            "the deadline is not the queue instant's literal: {input:?}"
        );
    }
    assert_all_pass(&model, &result);
}

#[test]
fn issue_546_a_later_step_reading_the_stored_copy_gets_it_present_and_apart() {
    let model = ir(&with_expire());
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let in_scenario = |id: &str| {
        steps
            .iter()
            .filter(|(scenario, _)| scenario == id)
            .map(|(_, input)| input)
            .collect::<Vec<_>>()
    };
    let expired = in_scenario("demo.queue.Expire/outcome/expired");
    assert_eq!(expired.len(), 1, "{steps:#?}");
    let deadline = expired[0]
        .get("expires_at")
        .unwrap_or_else(|| panic!("`expired` reads the stored deadline: {:?}", expired[0]));
    assert_ne!(
        Some(deadline),
        expired[0].get("queued_at"),
        "the deadline is not the queue instant's literal: {:?}",
        expired[0]
    );
    for id in [
        "demo.queue.Expire/outcome/no-deadline",
        "demo.queue.Withdraw/outcome/withdrawn",
    ] {
        let found = in_scenario(id);
        assert!(!found.is_empty(), "{id}: {steps:#?}");
        for input in found {
            assert!(!input.contains_key("expires_at"), "{id}: {input:?}");
        }
    }
    assert_all_pass(&model, &result);
}

#[test]
fn issue_546_a_guard_reading_the_optional_gets_it_present_and_apart() {
    let model = ir(&guarded());
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let with = steps
        .iter()
        .find(|(scenario, _)| scenario == "demo.queue.Submit/outcome/with-deadline")
        .map_or_else(|| panic!("{steps:#?}"), |(_, input)| input);
    let deadline = with
        .get("expires_at")
        .unwrap_or_else(|| panic!("the guard needs the deadline present: {with:?}"));
    assert_ne!(
        Some(deadline),
        with.get("queued_at"),
        "the deadline is not the queue instant's literal: {with:?}"
    );
    let without = steps
        .iter()
        .find(|(scenario, _)| scenario == "demo.queue.Submit/outcome/submitted")
        .map_or_else(|| panic!("{steps:#?}"), |(_, input)| input);
    assert!(!without.contains_key("expires_at"), "{without:?}");
    assert_all_pass(&model, &result);
}
