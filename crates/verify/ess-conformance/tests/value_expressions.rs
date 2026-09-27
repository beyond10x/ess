//! Value expressions in `sets:` and `payload:` (`docs/design/value-expressions.md`): what a
//! synthesized suite asserts for each source, read against the arranged row.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{scenario::ViewExpectation, ConformanceSuite as Suite, ScenarioStep};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

fn suite(body: &str) -> Suite {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir).suite
}

fn scenario<'a>(suite: &'a Suite, id: &str) -> &'a [ScenarioStep] {
    &suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| {
            panic!(
                "no scenario `{id}`; have: {:?}",
                suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1
        .steps
}

/// The last row the scenario requires the view to contain.
fn last_row(
    steps: &[ScenarioStep],
) -> std::collections::BTreeMap<String, ess_conformance::ScenarioValue> {
    steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no row expectation in {steps:#?}"))
}

/// The payload values the last `ExpectEvent` of `event` compares.
fn payload(steps: &[ScenarioStep], event: &str) -> std::collections::BTreeMap<String, Node> {
    steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event: seen,
                payload,
                ..
            } if seen.to_string() == event => Some(payload.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no `{event}` expectation in {steps:#?}"))
}

fn decimal(text: &str) -> Node {
    Node::Number(Number::decimal_literal(text).expect("a decimal literal"))
}

const LITERALS: &str = "format: ess/13
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: discount, type: Decimal}
      - {name: retries, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.Opened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: retries, type: Integer}
      - {name: rate, type: Decimal}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Open]}
commands:
  - name: demo.orders.Open
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened: {order_id: {generated: true}, retries: 0, rate: '0.25'}
        sets:
          discount: 0.0
          retries: '0'
views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: discount, type: Decimal}
      - {name: retries, type: Integer}
";

const EXPRESSIONS: &str = "format: ess/14
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - name: demo.orders.Ref
    kind: struct
    fields:
      - {name: id, type: demo.orders.OrderId}
      - {name: label, type: String}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: note, type: String}
      - {name: retries, type: Integer}
      - {name: score, type: Decimal}
      - {name: stamp, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: demo.orders.Opened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: seq, type: Integer}
      - {name: ref, type: demo.orders.Ref}
  - name: demo.orders.Retried
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: previous, type: Integer}
  - name: demo.orders.Closed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: note, type: String}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Open, demo.orders.Retry, demo.orders.Close]}
commands:
  - name: demo.orders.Open
    input:
      - {name: note, type: String}
      - {name: seq, type: Optional<Integer>}
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {generated: true}
            seq: {input: seq, else: {generated: true}}
            ref:
              id: '7'
              label: input.label
        sets:
          note: input.note
          retries: 2
          score: 0.5
          stamp: 'a'
  - name: demo.orders.Retry
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: retried
        updates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Retried]
        payload:
          demo.orders.Retried:
            order_id: input.order_id
            previous: {subject: retries}
        sets:
          retries: {increment: 1}
          score: {increment: '-0.25'}
          stamp: {generated: true}
  - name: demo.orders.Close
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: order_id
        emits: [demo.orders.Closed]
        payload:
          demo.orders.Closed:
            order_id: input.order_id
            note: {subject: note}
views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: note, type: String}
      - {name: retries, type: Integer}
      - {name: score, type: Decimal}
      - {name: stamp, type: String}
      - {name: state, type: demo.orders.Order.State}
";

fn literal(value: Node) -> ess_conformance::ScenarioValue {
    ess_conformance::ScenarioValue::Literal { value }
}

#[test]
fn a_subject_source_in_a_payload_is_the_arranged_rows_value() {
    // beyond10x/ess#133: `note: {subject: note}` is the note the creating act wrote.
    let suite = suite(EXPRESSIONS);
    let steps = scenario(&suite, "demo.orders.Close/outcome/closed");
    let sent = steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.orders.Open" =>
            {
                input.get("note").cloned()
            }
            _ => None,
        })
        .expect("the arrangement opens the order");
    let ess_conformance::ScenarioValue::Literal { value: note } = sent else {
        panic!("the note is a literal: {sent:?}")
    };
    assert_eq!(
        payload(steps, "demo.orders.Closed").get("note"),
        Some(&note)
    );
}

#[test]
fn an_increment_is_asserted_as_the_arranged_value_plus_the_amount() {
    // beyond10x/ess#134.
    let suite = suite(EXPRESSIONS);
    let steps = scenario(&suite, "demo.orders.Retry/outcome/retried");
    let row = last_row(steps);
    assert_eq!(
        row.get("retries"),
        Some(&literal(Node::Number(3_i64.into())))
    );
    assert_eq!(row.get("score"), Some(&literal(decimal("0.25"))));
    assert_eq!(
        payload(steps, "demo.orders.Retried").get("previous"),
        Some(&Node::Number(2_i64.into()))
    );
}

#[test]
fn a_generated_sets_entry_makes_no_claim_about_the_row() {
    // beyond10x/ess#134: preservation used to assert the old value of a field the branch changes.
    let suite = suite(EXPRESSIONS);
    let row = last_row(scenario(&suite, "demo.orders.Retry/outcome/retried"));
    assert_eq!(row.get("stamp"), None, "{row:?}");
}

#[test]
fn a_fallback_is_asserted_when_the_input_is_sent_and_a_nested_struct_when_every_leaf_is_known() {
    // beyond10x/ess#137 and #136.
    let suite = suite(EXPRESSIONS);
    let steps = scenario(&suite, "demo.orders.Open/outcome/opened");
    let input = steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input.clone()),
            _ => None,
        })
        .expect("the command is executed");
    let values = payload(steps, "demo.orders.Opened");
    match input.get("seq") {
        Some(ess_conformance::ScenarioValue::Literal { value }) if value != &Node::Null => {
            assert_eq!(values.get("seq"), Some(value));
        }
        _ => assert_eq!(values.get("seq"), None, "{values:?}"),
    }
    let ess_conformance::ScenarioValue::Literal { value: label } =
        input.get("label").expect("a label is sent")
    else {
        panic!("the label is a literal")
    };
    assert_eq!(
        values.get("ref"),
        Some(&Node::Map(
            [
                ("id".to_owned(), Node::Text("7".to_owned())),
                ("label".to_owned(), label.clone()),
            ]
            .into_iter()
            .collect()
        ))
    );
}

#[test]
fn a_decimal_sets_literal_is_asserted_on_the_row_as_a_number() {
    // beyond10x/ess#135.
    let suite = suite(LITERALS);
    let row = last_row(scenario(&suite, "demo.orders.Open/outcome/opened"));
    assert_eq!(
        row.get("discount"),
        Some(&ess_conformance::ScenarioValue::Literal {
            value: decimal("0")
        })
    );
}

#[test]
fn a_numeric_payload_literal_is_compared_as_the_number_it_spells() {
    // Before, every payload literal was asserted as its text, so `retries: 0` required the string
    // `"0"` of an implementation publishing the number 0.
    let suite = suite(LITERALS);
    let values = payload(
        scenario(&suite, "demo.orders.Open/outcome/opened"),
        "demo.orders.Opened",
    );
    assert_eq!(values.get("retries"), Some(&Node::Number(0_i64.into())));
    assert_eq!(values.get("rate"), Some(&decimal("0.25")));
}
