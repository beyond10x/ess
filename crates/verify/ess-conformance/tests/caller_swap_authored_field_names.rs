//! beyond10x/ess#766: the caller-swapped run renames only the instance and instant names it binds.
//! An authored field named `instance` or `instant`, in any authored position, keeps the value the
//! run sends, so the interpreter, which is correct, passes every synthesized scenario.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r"format: ess/19
system: example
version: v1
summary: Fields named instance on a model whose commands read the caller.
domain: example.order
types:
  - {name: example.order.OrderId, kind: newtype, of: Uuid}
  - {name: example.order.AccountId, kind: newtype, of: Uuid}
  - name: example.order.Pair
    kind: struct
    fields: [{name: kind, type: String}, {name: instance, type: String}]
  - name: example.order.Wrap
    kind: struct
    fields: [{name: instance, type: String}]
entities:
  - name: example.order.Order
    identity: {name: order_id, type: example.order.OrderId}
    fields:
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: holder, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
actors:
  - name: example.order.AccountUser
    attributes:
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
    may: [example.order.PlaceOrder]
errors:
  - name: example.order.Refused
    summary: The value is not the caller's.
    fields:
      - {name: instance, type: String}
commands:
  - name: example.order.PlaceOrder
    input:
      - {name: instance, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
    outcomes:
      - name: refused
        when: instance != caller.instance
        error: example.order.Refused
        payload:
          example.order.Refused: {instance: {caller: instance}}
      - name: placed
        creates: example.order.Order
        instance: order_id
        sets:
          account_id: {caller: account_id}
          instance: input.instance
          holder: {caller: instance}
          pair: input.pair
          wrap: input.wrap
        emits: [example.order.OrderPlaced]
        payload:
          example.order.OrderPlaced:
            order_id: {generated: true}
            account_id: {caller: account_id}
            instance: input.instance
            pair: input.pair
            wrap: input.wrap
events:
  - name: example.order.OrderPlaced
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
views:
  - name: example.order.OrderDetails
    source: example.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: holder, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
      - {name: state, type: example.order.Order.State}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(suite: &ConformanceSuite, ir: &EssIr) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn assert_all_pass(text: &str) {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    let results = statuses(&synthesis.suite, &ir);
    assert!(!results.is_empty(), "scenarios are synthesized");
    let failed: Vec<_> = results
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "every scenario passes: {failed:#?}");
    let suite = serde_json::to_string(&synthesis.suite).unwrap();
    assert!(suite.contains("-swapped"), "a swapped run is synthesized");
}

#[test]
fn an_authored_field_named_instance_keeps_its_value_in_the_swapped_run() {
    assert_all_pass(MODEL);
}

#[test]
fn an_authored_field_named_instant_keeps_its_value_in_the_swapped_run() {
    let text = MODEL
        .replace("instance", "instant")
        .replace("instant: order_id", "instance: order_id");
    assert_all_pass(&text);
}
