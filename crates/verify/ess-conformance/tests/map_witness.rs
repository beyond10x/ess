//! A `Map` input is witnessed with one entry, not `{}` (beyond10x/ess#196).
//!
//! Every map input used to be sent as `{}` and every copy of it asserted as `{}`, so a target that
//! dropped or emptied the map passed. The witness now holds one entry: the key is the key
//! primitive's own witness at the map's path, spelled the way a setup key is spelled, and the value
//! is built at `<map>.0` like a list element. A further instance moves both. `<map>.count` is a fact
//! on command input, so a count guard over a map is decided.

use std::collections::BTreeMap;

use ess_compiler::ir::ResolvedCommand;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::witness::{candidates, Distinction};
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::name::QualifiedName;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::FactSource;

/// The issue's shape: a nested struct leaf, a top-level `Map<String, String>` and a
/// `Map<String, Json>`, all copied by `sets:`; and an update that writes the map again.
const ORDERS: &str = "format: ess/17
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Item
    kind: struct
    fields:
      - {name: sku, type: String}
      - {name: attrs, type: 'Map<String, String>'}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: String}
    fields:
      - {name: item, type: Optional<demo.orders.Item>}
      - {name: tags, type: 'Map<String, String>'}
      - {name: meta, type: 'Map<String, Json>'}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.Changed
    fields:
      - {name: order_id, type: String}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.ItemPushed, demo.orders.Retagged]}
commands:
  - name: demo.orders.ItemPushed
    input:
      - {name: order_id, type: String}
      - {name: sku, type: String}
      - {name: attrs, type: 'Map<String, String>'}
      - {name: tags, type: 'Map<String, String>'}
      - {name: meta, type: 'Map<String, Json>'}
    outcomes:
      - name: created
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Changed]
        payload:
          demo.orders.Changed: {order_id: input.order_id}
        sets:
          item: {sku: input.sku, attrs: input.attrs}
          tags: input.tags
          meta: input.meta
  - name: demo.orders.Retagged
    input:
      - {name: order_id, type: String}
      - {name: tags, type: 'Map<String, String>'}
    outcomes:
      - name: retagged
        updates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Changed]
        payload:
          demo.orders.Changed: {order_id: input.order_id}
        sets:
          tags: input.tags
views:
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: String}
      - {name: item, type: Optional<demo.orders.Item>}
      - {name: tags, type: 'Map<String, String>'}
      - {name: meta, type: 'Map<String, Json>'}
";

/// A map of every admitted key primitive, and a guard over a map's `.count`.
const KEYS: &str = "format: ess/17
system: demo
version: v1
domain: demo.keys
types:
  - name: demo.keys.Pair
    kind: struct
    fields:
      - {name: left, type: String}
      - {name: right, type: Integer}
  - name: demo.keys.Code
    kind: newtype
    of: String
    invariants: ['value.count <= 3']
entities:
  - name: demo.keys.Box
    identity: {name: box_id, type: String}
    fields:
      - {name: labels, type: 'Map<String, String>'}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.keys.Packed
    fields:
      - {name: box_id, type: String}
actors:
  - {name: demo.keys.Packer, may: [demo.keys.Pack, demo.keys.Label]}
commands:
  - name: demo.keys.Pack
    input:
      - {name: box_id, type: String}
      - {name: by_text, type: 'Map<String, String>'}
      - {name: by_number, type: 'Map<Integer, String>'}
      - {name: by_id, type: 'Map<Uuid, String>'}
      - {name: by_time, type: 'Map<Timestamp, String>'}
      - {name: by_flag, type: 'Map<Boolean, String>'}
      - {name: by_span, type: 'Map<Duration, String>'}
      - {name: by_bytes, type: 'Map<Bytes, String>'}
      - {name: by_amount, type: 'Map<Decimal, String>'}
      - {name: pairs, type: 'Map<String, demo.keys.Pair>'}
      - {name: codes, type: 'Map<String, demo.keys.Code>'}
      - {name: nested, type: 'Map<String, Map<String, Integer>>'}
    outcomes:
      - name: packed
        creates: demo.keys.Box
        instance: box_id
        emits: [demo.keys.Packed]
        payload:
          demo.keys.Packed: {box_id: input.box_id}
        sets:
          labels: input.by_text
  - name: demo.keys.Label
    input:
      - {name: box_id, type: String}
      - {name: labels, type: 'Map<String, String>'}
    outcomes:
      - name: unlabelled
        creates: demo.keys.Box
        instance: box_id
        when: labels.count == 0
        emits: [demo.keys.Packed]
        payload:
          demo.keys.Packed: {box_id: input.box_id}
        sets:
          labels: input.labels
      - name: labelled
        creates: demo.keys.Box
        instance: box_id
        emits: [demo.keys.Packed]
        payload:
          demo.keys.Packed: {box_id: input.box_id}
        sets:
          labels: input.labels
views:
  - name: demo.keys.Boxes
    source: demo.keys.Box
    consistency: read_your_writes
    fields:
      - {name: box_id, type: String}
      - {name: labels, type: 'Map<String, String>'}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("maps.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn command<'ir>(ir: &'ir EssIr, name: &str) -> &'ir ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new(name).expect("a valid name"))
        .expect("the fixture declares it")
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every literal input one command was sent in a scenario, in step order.
fn sent_all(scenario: &ConformanceScenario, name: &str) -> Vec<BTreeMap<String, Node>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } if command.to_string() == name => {
                Some(
                    input
                        .iter()
                        .filter_map(|(field, value)| match value {
                            ScenarioValue::Literal { value } => {
                                Some((field.clone(), value.clone()))
                            }
                            _ => None,
                        })
                        .collect(),
                )
            }
            _ => None,
        })
        .collect()
}

/// The last row the scenario requires a view to contain.
fn row(scenario: &ConformanceScenario) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .expect("a row is required")
}

fn entries(value: &Node) -> &BTreeMap<String, Node> {
    value
        .as_map()
        .unwrap_or_else(|| panic!("a map, got {value:?}"))
}

#[test]
fn issue_196_a_copied_map_input_is_sent_with_an_entry_and_asserted_holding_it() {
    let suite = suite(ORDERS);
    let scenario = scenario(&suite, "demo.orders.ItemPushed/outcome/created");
    let sent = sent_all(scenario, "demo.orders.ItemPushed")
        .pop()
        .expect("the command is sent");
    for field in ["attrs", "tags", "meta"] {
        let value = sent.get(field).expect("every input is sent");
        assert_eq!(
            entries(value).len(),
            1,
            "`{field}` holds one entry: {sent:?}"
        );
    }
    // Two maps of one type never carry one value (rule 2).
    assert_ne!(sent.get("attrs"), sent.get("tags"), "{sent:?}");

    let row = row(scenario);
    for (leaf, input) in [("tags", "tags"), ("meta", "meta")] {
        assert_eq!(
            row.get(leaf),
            Some(&ScenarioValue::literal(sent[input].clone())),
            "{leaf} in {row:?}"
        );
    }
    // Every leaf of `item` is determined, so the struct is asserted whole, map included.
    let Some(ScenarioValue::Literal { value: item }) = row.get("item") else {
        panic!("the struct is asserted: {row:?}")
    };
    assert_eq!(entries(item).get("attrs"), Some(&sent["attrs"]), "{row:?}");
}

#[test]
fn issue_196_an_update_that_writes_a_map_is_sent_a_value_different_from_the_prior_one() {
    let suite = suite(ORDERS);
    let scenario = scenario(&suite, "demo.orders.Retagged/outcome/retagged");
    let created = sent_all(scenario, "demo.orders.ItemPushed");
    let updated = sent_all(scenario, "demo.orders.Retagged")
        .pop()
        .expect("the update is sent");
    let prior = created
        .last()
        .expect("the subject is arranged by its creating command")
        .get("tags")
        .expect("the prior value is sent");
    let written = updated.get("tags").expect("the update carries the map");
    assert!(!entries(written).is_empty(), "{written:?}");
    assert_ne!(
        written, prior,
        "an update identical to the prior value cannot tell a write from a no-op"
    );
    assert_eq!(
        row(scenario).get("tags"),
        Some(&ScenarioValue::literal(written.clone()))
    );
}

#[test]
fn issue_196_every_key_primitive_gets_an_entry_that_the_type_admits() {
    let ir = ir(KEYS);
    let pack = command(&ir, "demo.keys.Pack");
    let options = candidates(&ir, pack, &[], Distinction::PLAIN).expect("a witness");
    let base = options.first().expect("the base witness is admitted");
    for field in [
        "by_text",
        "by_number",
        "by_id",
        "by_time",
        "by_flag",
        "by_span",
        "by_bytes",
        "pairs",
        "codes",
        "nested",
    ] {
        let value = base.get(field).expect("every input is sent");
        assert_eq!(entries(value).len(), 1, "`{field}`: {value:?}");
    }
    // A `Decimal` key has no setup spelling, so nothing can be put in that map.
    assert_eq!(base.get("by_amount"), Some(&Node::Map(BTreeMap::new())));
    assert_eq!(
        entries(&base["by_number"]).keys().collect::<Vec<_>>(),
        ["1"],
        "an Integer key is spelled in canonical decimal"
    );
    assert!(
        entries(&base["by_flag"]).contains_key("true")
            || entries(&base["by_flag"]).contains_key("false")
    );
    // A nested map is witnessed all the way down.
    let (_, inner) = entries(&base["nested"]).iter().next().expect("one entry");
    assert_eq!(entries(inner).len(), 1, "{inner:?}");
    // A value held to an invariant is one the invariant admits: the base text is too long.
    let (_, code) = entries(&base["codes"]).iter().next().expect("one entry");
    assert!(
        code.as_text().is_some_and(|text| text.chars().count() <= 3),
        "{code:?}"
    );
}

#[test]
fn issue_196_a_further_instance_moves_the_key_and_the_value() {
    let ir = ir(KEYS);
    let pack = command(&ir, "demo.keys.Pack");
    let plain = candidates(&ir, pack, &[], Distinction::PLAIN).expect("a witness");
    let further = candidates(&ir, pack, &[], Distinction::further(1)).expect("a witness");
    for field in [
        "by_text",
        "by_number",
        "by_id",
        "by_time",
        "by_span",
        "pairs",
    ] {
        let (plain_key, plain_value) = entries(&plain[0][field]).iter().next().expect("an entry");
        let (further_key, further_value) =
            entries(&further[0][field]).iter().next().expect("an entry");
        assert_ne!(plain_key, further_key, "`{field}` key");
        assert_ne!(plain_value, further_value, "`{field}` value");
    }
}

#[test]
fn issue_196_a_map_input_publishes_its_count_so_a_count_guard_is_decided() {
    let ir = ir(KEYS);
    let label = command(&ir, "demo.keys.Label");
    let facts = ess_conformance::flatten(
        &ir,
        label,
        &BTreeMap::from([
            ("box_id".to_owned(), Node::Text("b".into())),
            (
                "labels".to_owned(),
                Node::Map(BTreeMap::from([
                    ("a".to_owned(), Node::Text("x".into())),
                    ("b".to_owned(), Node::Text("y".into())),
                ])),
            ),
        ]),
    )
    .unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(
        facts.fact(&FactPath::new("labels.count").expect("a path")),
        Some(FactValue::count(2))
    );

    // `Code`'s own invariant is refused at the type, which is not this test's business; nothing
    // about `Label` is.
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let about_label: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains("demo.keys.Label"))
        .collect();
    assert!(about_label.is_empty(), "{about_label:#?}");
    let suite = synthesis.suite;
    for (id, held) in [
        ("demo.keys.Label/outcome/unlabelled", 0),
        ("demo.keys.Label/outcome/labelled", 1),
    ] {
        let scenario = scenario(&suite, id);
        let sent = sent_all(scenario, "demo.keys.Label")
            .pop()
            .expect("the command is sent");
        assert_eq!(entries(&sent["labels"]).len(), held, "{id}: {sent:?}");
    }
}
