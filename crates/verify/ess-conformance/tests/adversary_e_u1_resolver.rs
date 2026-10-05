//! Adversary pass 1 on unit E-U1, decision 9 (#225): "equal identity witnesses name one arranged
//! instance; unequal keep two rows".
//!
//! `supply` now maps every bound identity input whose *witness value* equals another bound input's
//! (or the subject's) onto that one instance, at every source format. Witness values are equal by
//! construction, not only by a guard: every `Integer` field's plain witness is the same number.
//! Two references of two different entities, both identified by an `Integer` newtype, then collapse
//! onto the subject's row although nothing holds them equal.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(format: u32) -> String {
    format!(
        r"format: ess/{format}
system: shop
version: v1
domain: shop.core
types:
  - {{name: shop.core.OrderNo, kind: newtype, of: Integer}}
  - {{name: shop.core.CustomerNo, kind: newtype, of: Integer}}
entities:
  - name: shop.core.Order
    identity: {{name: order_no, type: shop.core.OrderNo}}
    fields:
      - {{name: customer_no, type: Optional<shop.core.CustomerNo>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
  - name: shop.core.Customer
    identity: {{name: customer_no, type: shop.core.CustomerNo}}
    fields:
      - {{name: name, type: String}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
errors:
  - {{name: shop.core.NoCustomer, summary: No customer carries that number., fields: []}}
events:
  - name: shop.core.CustomerAdded
    fields: [{{name: customer_no, type: shop.core.CustomerNo}}]
  - name: shop.core.OrderAdded
    fields: [{{name: order_no, type: shop.core.OrderNo}}]
  - name: shop.core.Assigned
    fields: [{{name: order_no, type: shop.core.OrderNo}}]
commands:
  - name: shop.core.AddCustomer
    input:
      - {{name: customer_no, type: shop.core.CustomerNo}}
      - {{name: name, type: String}}
    outcomes:
      - name: added
        creates: shop.core.Customer
        instance: customer_no
        sets: {{name: input.name}}
        emits: [shop.core.CustomerAdded]
        payload: {{shop.core.CustomerAdded: {{customer_no: input.customer_no}}}}
  - name: shop.core.AddOrder
    input:
      - {{name: order_no, type: shop.core.OrderNo}}
    outcomes:
      - name: added
        creates: shop.core.Order
        instance: order_no
        emits: [shop.core.OrderAdded]
        payload: {{shop.core.OrderAdded: {{order_no: input.order_no}}}}
  - name: shop.core.Assign
    input:
      - {{name: order_no, type: shop.core.OrderNo}}
      - {{name: customer_no, type: shop.core.CustomerNo}}
    outcomes:
      - name: no-customer
        when_related: {{via: input.customer_no, exists: false}}
        error: shop.core.NoCustomer
      - name: assigned
        updates: shop.core.Order
        instance: order_no
        sets: {{customer_no: input.customer_no}}
        emits: [shop.core.Assigned]
        payload: {{shop.core.Assigned: {{order_no: input.order_no}}}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("shop.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn assign_inputs(suite: &ConformanceSuite, id: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; have {:?}",
                    suite.scenarios.keys().collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        );
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "shop.core.Assign" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .collect()
}

fn check(format: u32) {
    let model = ir(&model(format));
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    for refusal in &synthesis.refusals {
        eprintln!("refused: {refusal:?}");
    }
    let suite = synthesis.suite;
    let assigned = assign_inputs(&suite, "shop.core.Assign/outcome/assigned");
    let sent = assigned.last().expect("the assigned scenario sends Assign");
    assert_ne!(
        sent.get("order_no"),
        sent.get("customer_no"),
        "ess/{format}: the customer reference is sent as the order's own row: {sent:#?}"
    );
    let admitted = AdmittedSuite::from_suite(&suite).expect("admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(failing, Vec::<String>::new(), "ess/{format}");
}

#[test]
fn adversary_two_integer_identity_references_keep_their_own_rows_ess21() {
    check(21);
}

#[test]
fn adversary_two_integer_identity_references_keep_their_own_rows_ess22() {
    check(22);
}
