//! Adversary, pass 1, against the Map input witness (beyond10x/ess#196).
//!
//! The unit's own tests read the synthesized suite. These run it: a Rust target, the same target's
//! recorded answers replayed through the generated Go package (`support_go`), and a JavaScript twin
//! through the generated TypeScript package, each with a correct mode and the wrong modes the issue
//! is about — a target that empties, omits, re-keys or blanks a copied map. A wrong mode that every
//! scenario still passes is a map the suite does not check.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::consistency::ConsistencyToken;
use ess_primitives::node::Node;
use serde_json::Value;

type Fields = BTreeMap<String, Node>;
type Verdicts = BTreeMap<String, String>;

// ---- models ----------------------------------------------------------------------------------------

/// The issue's shape, plus the map copied into an event payload.
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
      - {name: tags, type: 'Map<String, String>'}
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
          demo.orders.Changed: {order_id: input.order_id, tags: input.tags}
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
          demo.orders.Changed: {order_id: input.order_id, tags: input.tags}
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

/// Every admitted key primitive, nested maps, a map in a struct, in a struct in a map, in a list,
/// and an optional map, each copied by `sets:` and read back through a view.
const KEYS: &str = "format: ess/17
system: demo
version: v1
domain: demo.k
types:
  - name: demo.k.Pair
    kind: struct
    fields:
      - {name: left, type: String}
      - {name: flags, type: 'Map<Boolean, Integer>'}
  - name: demo.k.Holder
    kind: struct
    fields:
      - {name: ids, type: 'Map<Uuid, String>'}
entities:
  - name: demo.k.Box
    identity: {name: box_id, type: String}
    fields:
      - {name: by_number, type: 'Map<Integer, String>'}
      - {name: by_id, type: 'Map<Uuid, String>'}
      - {name: by_time, type: 'Map<Timestamp, String>'}
      - {name: by_flag, type: 'Map<Boolean, String>'}
      - {name: nested, type: 'Map<String, Map<Integer, String>>'}
      - {name: pairs, type: 'Map<Uuid, demo.k.Pair>'}
      - {name: maybe, type: 'Optional<Map<Timestamp, Integer>>'}
      - {name: holder, type: demo.k.Holder}
      - {name: holders, type: 'List<demo.k.Holder>'}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.k.Packed
    fields:
      - {name: box_id, type: String}
actors:
  - {name: demo.k.Packer, may: [demo.k.Pack]}
commands:
  - name: demo.k.Pack
    input:
      - {name: box_id, type: String}
      - {name: by_number, type: 'Map<Integer, String>'}
      - {name: by_id, type: 'Map<Uuid, String>'}
      - {name: by_time, type: 'Map<Timestamp, String>'}
      - {name: by_flag, type: 'Map<Boolean, String>'}
      - {name: nested, type: 'Map<String, Map<Integer, String>>'}
      - {name: pairs, type: 'Map<Uuid, demo.k.Pair>'}
      - {name: maybe, type: 'Optional<Map<Timestamp, Integer>>'}
      - {name: holder, type: demo.k.Holder}
      - {name: holders, type: 'List<demo.k.Holder>'}
    outcomes:
      - name: packed
        creates: demo.k.Box
        instance: box_id
        emits: [demo.k.Packed]
        payload:
          demo.k.Packed: {box_id: input.box_id}
        sets:
          by_number: input.by_number
          by_id: input.by_id
          by_time: input.by_time
          by_flag: input.by_flag
          nested: input.nested
          pairs: input.pairs
          maybe: input.maybe
          holder: input.holder
          holders: input.holders
views:
  - name: demo.k.Boxes
    source: demo.k.Box
    consistency: read_your_writes
    fields:
      - {name: box_id, type: String}
      - {name: by_number, type: 'Map<Integer, String>'}
      - {name: by_id, type: 'Map<Uuid, String>'}
      - {name: by_time, type: 'Map<Timestamp, String>'}
      - {name: by_flag, type: 'Map<Boolean, String>'}
      - {name: nested, type: 'Map<String, Map<Integer, String>>'}
      - {name: pairs, type: 'Map<Uuid, demo.k.Pair>'}
      - {name: maybe, type: 'Optional<Map<Timestamp, Integer>>'}
      - {name: holder, type: demo.k.Holder}
      - {name: holders, type: 'List<demo.k.Holder>'}
";

/// Count guards over maps: `== 0`, `> 1`, `>= 3`, over an Integer-keyed map, inside a list element,
/// and a list twin of the `>= 3` guard.
const COUNTS: &str = "format: ess/17
system: demo
version: v1
domain: demo.c
types:
  - name: demo.c.Line
    kind: struct
    fields:
      - {name: sku, type: String}
      - {name: labels, type: 'Map<Uuid, String>'}
events:
  - name: demo.c.Counted
    fields:
      - {name: labels, type: 'Map<String, String>'}
  - name: demo.c.Tallied
    fields:
      - {name: counts, type: 'Map<Integer, Integer>'}
  - name: demo.c.Seen
    fields:
      - {name: tag, type: String}
actors:
  - {name: demo.c.Clerk, may: [demo.c.Zero, demo.c.Many, demo.c.Three, demo.c.IntMany, demo.c.LineMany, demo.c.ListThree]}
commands:
  - name: demo.c.Zero
    input:
      - {name: labels, type: 'Map<String, String>'}
    outcomes:
      - name: empty
        when: labels.count == 0
        emits: [demo.c.Counted]
        payload:
          demo.c.Counted: {labels: input.labels}
      - name: full
        emits: [demo.c.Counted]
        payload:
          demo.c.Counted: {labels: input.labels}
  - name: demo.c.Many
    input:
      - {name: labels, type: 'Map<String, String>'}
    outcomes:
      - name: many
        when: labels.count > 1
        emits: [demo.c.Counted]
        payload:
          demo.c.Counted: {labels: input.labels}
      - name: few
        emits: [demo.c.Counted]
        payload:
          demo.c.Counted: {labels: input.labels}
  - name: demo.c.Three
    input:
      - {name: labels, type: 'Map<String, String>'}
    outcomes:
      - name: lots
        when: labels.count >= 3
        emits: [demo.c.Counted]
        payload:
          demo.c.Counted: {labels: input.labels}
      - name: few
        emits: [demo.c.Counted]
        payload:
          demo.c.Counted: {labels: input.labels}
  - name: demo.c.IntMany
    input:
      - {name: counts, type: 'Map<Integer, Integer>'}
    outcomes:
      - name: many
        when: counts.count > 1
        emits: [demo.c.Tallied]
        payload:
          demo.c.Tallied: {counts: input.counts}
      - name: few
        emits: [demo.c.Tallied]
        payload:
          demo.c.Tallied: {counts: input.counts}
  - name: demo.c.LineMany
    input:
      - {name: tag, type: String}
      - {name: lines, type: 'List<demo.c.Line>'}
    outcomes:
      - name: many
        when: lines.0.labels.count > 1
        emits: [demo.c.Seen]
        payload:
          demo.c.Seen: {tag: input.tag}
      - name: few
        emits: [demo.c.Seen]
        payload:
          demo.c.Seen: {tag: input.tag}
  - name: demo.c.ListThree
    input:
      - {name: tag, type: String}
      - {name: items, type: 'List<String>'}
    outcomes:
      - name: lots
        when: items.count >= 3
        emits: [demo.c.Seen]
        payload:
          demo.c.Seen: {tag: input.tag}
      - name: few
        emits: [demo.c.Seen]
        payload:
          demo.c.Seen: {tag: input.tag}
";

/// A Boolean-keyed map holds two keys at most: `> 1` is satisfiable, `>= 3` is not.
const FLAGS: &str = "format: ess/17
system: demo
version: v1
domain: demo.f
events:
  - name: demo.f.Seen
    fields:
      - {name: flags, type: 'Map<Boolean, String>'}
actors:
  - {name: demo.f.Clerk, may: [demo.f.Two, demo.f.Three]}
commands:
  - name: demo.f.Two
    input:
      - {name: flags, type: 'Map<Boolean, String>'}
    outcomes:
      - name: both
        when: flags.count > 1
        emits: [demo.f.Seen]
        payload:
          demo.f.Seen: {flags: input.flags}
      - name: one
        emits: [demo.f.Seen]
        payload:
          demo.f.Seen: {flags: input.flags}
  - name: demo.f.Three
    input:
      - {name: flags, type: 'Map<Boolean, String>'}
    outcomes:
      - name: lots
        when: flags.count >= 3
        emits: [demo.f.Seen]
        payload:
          demo.f.Seen: {flags: input.flags}
      - name: few
        emits: [demo.f.Seen]
        payload:
          demo.f.Seen: {flags: input.flags}
";

/// A type that refers to itself only through a map, which was `{}` before and so had a witness.
const TREE: &str = "format: ess/17
system: demo
version: v1
domain: demo.r
types:
  - name: demo.r.Tree
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: children, type: 'Map<String, demo.r.Tree>'}
events:
  - name: demo.r.Planted
    fields:
      - {name: tree, type: demo.r.Tree}
actors:
  - {name: demo.r.Gardener, may: [demo.r.Plant]}
commands:
  - name: demo.r.Plant
    input:
      - {name: tree, type: demo.r.Tree}
    outcomes:
      - name: planted
        emits: [demo.r.Planted]
        payload:
          demo.r.Planted: {tree: input.tree}
";

/// Two maps whose key and value are both Integer, copied to two fields; and the scalar twin.
const TWINS: &str = "format: ess/17
system: demo
version: v1
domain: demo.t
entities:
  - name: demo.t.Pair
    identity: {name: pair_id, type: String}
    fields:
      - {name: left, type: 'Map<Integer, Integer>'}
      - {name: right, type: 'Map<Integer, Integer>'}
      - {name: x, type: Integer}
      - {name: y, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.t.Made
    fields:
      - {name: pair_id, type: String}
actors:
  - {name: demo.t.Clerk, may: [demo.t.Make]}
commands:
  - name: demo.t.Make
    input:
      - {name: pair_id, type: String}
      - {name: left, type: 'Map<Integer, Integer>'}
      - {name: right, type: 'Map<Integer, Integer>'}
      - {name: x, type: Integer}
      - {name: y, type: Integer}
    outcomes:
      - name: made
        creates: demo.t.Pair
        instance: pair_id
        emits: [demo.t.Made]
        payload:
          demo.t.Made: {pair_id: input.pair_id}
        sets:
          left: input.left
          right: input.right
          x: input.x
          y: input.y
views:
  - name: demo.t.Pairs
    source: demo.t.Pair
    consistency: read_your_writes
    fields:
      - {name: pair_id, type: String}
      - {name: left, type: 'Map<Integer, Integer>'}
      - {name: right, type: 'Map<Integer, Integer>'}
      - {name: x, type: Integer}
      - {name: y, type: Integer}
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
        "nothing is refused: {}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    synthesis.suite
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

/// The last literal input the scenario sends `command`.
fn sent(scenario: &ConformanceScenario, command: &str) -> Fields {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: c, input, ..
            } if c.to_string() == command => Some(
                input
                    .iter()
                    .filter_map(|(field, value)| match value {
                        ScenarioValue::Literal { value } => Some((field.clone(), value.clone())),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .next_back()
        .unwrap_or_else(|| panic!("{command} is sent"))
}

fn entries(value: &Node) -> usize {
    value.as_map().map_or(0, BTreeMap::len)
}

// ---- one Rust target, driven by a per-model handler ----------------------------------------------

type Handler = fn(&str, &str, &Fields, &mut Vec<Fields>) -> SemanticCommandResult;

struct Store {
    mode: &'static str,
    handler: Handler,
    rows: RefCell<Vec<Fields>>,
}

impl Store {
    fn new(mode: &'static str, handler: Handler) -> Self {
        Self {
            mode,
            handler,
            rows: RefCell::new(Vec::new()),
        }
    }
}

fn took(command: &str, outcome: &str, event: &str, payload: Fields) -> SemanticCommandResult {
    let command: CommandRef = command.parse().expect("a command");
    let mut result = SemanticCommandResult::took(OutcomeRef::new(
        command,
        outcome.parse().expect("an outcome"),
    ));
    result.consistency = Some(ConsistencyToken::new("write").expect("a token"));
    let mut observed = ObservedEvent::new(event.parse().expect("an event"));
    observed.payload = payload;
    result.direct_events.push(observed);
    result
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("map-witness-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Ok((self.handler)(
            self.mode,
            &request.command.to_string(),
            &request.input,
            &mut self.rows.borrow_mut(),
        ))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self.rows.borrow().clone(),
            total: None,
        })
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "the model declares none",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
}

fn get(input: &Fields, field: &str) -> Node {
    input.get(field).cloned().unwrap_or(Node::Null)
}

fn empty() -> Node {
    Node::Map(BTreeMap::new())
}

/// Every value of a map, rewritten.
fn each_value(value: &Node, edit: impl Fn(&Node) -> Node) -> Node {
    match value {
        Node::Map(entries) => {
            Node::Map(entries.iter().map(|(k, v)| (k.clone(), edit(v))).collect())
        }
        other => other.clone(),
    }
}

/// Every key of a map, rewritten.
fn each_key(value: &Node, edit: impl Fn(&str) -> String) -> Node {
    match value {
        Node::Map(entries) => {
            Node::Map(entries.iter().map(|(k, v)| (edit(k), v.clone())).collect())
        }
        other => other.clone(),
    }
}

fn with_field(value: &Node, field: &str, to: Node) -> Node {
    match value {
        Node::Map(entries) => {
            let mut entries = entries.clone();
            entries.insert(field.to_owned(), to);
            Node::Map(entries)
        }
        other => other.clone(),
    }
}

fn orders(
    mode: &str,
    command: &str,
    input: &Fields,
    rows: &mut Vec<Fields>,
) -> SemanticCommandResult {
    let tags = get(input, "tags");
    let payload_tags = if mode == "drop-payload" {
        empty()
    } else {
        tags.clone()
    };
    let payload = Fields::from([
        ("order_id".to_owned(), get(input, "order_id")),
        ("tags".to_owned(), payload_tags),
    ]);
    match command {
        "demo.orders.ItemPushed" => {
            let mut row = Fields::from([
                ("order_id".to_owned(), get(input, "order_id")),
                (
                    "item".to_owned(),
                    Node::Map(BTreeMap::from([
                        ("sku".to_owned(), get(input, "sku")),
                        ("attrs".to_owned(), get(input, "attrs")),
                    ])),
                ),
                ("tags".to_owned(), tags.clone()),
                ("meta".to_owned(), get(input, "meta")),
            ]);
            match mode {
                "drop-tags" => {
                    row.insert("tags".to_owned(), empty());
                }
                "omit-tags" => {
                    row.remove("tags");
                }
                "drop-attrs" => {
                    let item = with_field(&row["item"], "attrs", empty());
                    row.insert("item".to_owned(), item);
                }
                "drop-meta" => {
                    row.insert("meta".to_owned(), empty());
                }
                "blank-value" => {
                    row.insert(
                        "tags".to_owned(),
                        each_value(&tags, |_| Node::Text(String::new())),
                    );
                }
                "rekey" => {
                    row.insert("tags".to_owned(), each_key(&tags, |_| "k".to_owned()));
                }
                _ => {}
            }
            rows.push(row);
            took(command, "created", "demo.orders.Changed", payload)
        }
        "demo.orders.Retagged" => {
            let id = get(input, "order_id");
            let Some(row) = rows.iter_mut().find(|row| row.get("order_id") == Some(&id)) else {
                return SemanticCommandResult::undeclared();
            };
            if mode != "retag-ignored" {
                row.insert("tags".to_owned(), tags);
            }
            took(command, "retagged", "demo.orders.Changed", payload)
        }
        other => panic!("unexpected command {other}"),
    }
}

fn keys(
    mode: &str,
    command: &str,
    input: &Fields,
    rows: &mut Vec<Fields>,
) -> SemanticCommandResult {
    assert_eq!(command, "demo.k.Pack");
    let mut row = input.clone();
    let edit = |row: &mut Fields, field: &str, edit: &dyn Fn(&Node) -> Node| {
        if let Some(value) = row.get(field).cloned() {
            row.insert(field.to_owned(), edit(&value));
        }
    };
    match mode {
        "drop-number" => edit(&mut row, "by_number", &|_| empty()),
        "rekey-number" => edit(&mut row, "by_number", &|v| {
            each_key(v, |k| format!("{k}.0"))
        }),
        "blank-id" => edit(&mut row, "by_id", &|v| {
            each_value(v, |_| Node::Text(String::new()))
        }),
        "drop-time" => edit(&mut row, "by_time", &|_| empty()),
        "drop-flag" => edit(&mut row, "by_flag", &|_| empty()),
        "drop-inner" => edit(&mut row, "nested", &|v| each_value(v, |_| empty())),
        "drop-pair-flags" => edit(&mut row, "pairs", &|v| {
            each_value(v, |pair| with_field(pair, "flags", empty()))
        }),
        "omit-maybe" => {
            row.remove("maybe");
        }
        "drop-maybe" => edit(&mut row, "maybe", &|_| empty()),
        "drop-holder-ids" => edit(&mut row, "holder", &|v| with_field(v, "ids", empty())),
        "drop-holders-ids" => edit(&mut row, "holders", &|v| match v {
            Node::Seq(items) => Node::Seq(
                items
                    .iter()
                    .map(|h| with_field(h, "ids", empty()))
                    .collect(),
            ),
            other => other.clone(),
        }),
        _ => {}
    }
    rows.push(row);
    took(
        command,
        "packed",
        "demo.k.Packed",
        Fields::from([("box_id".to_owned(), get(input, "box_id"))]),
    )
}

fn counts(mode: &str, command: &str, input: &Fields, _: &mut Vec<Fields>) -> SemanticCommandResult {
    let labels = get(input, "labels");
    let n = entries(&labels);
    let counted = |outcome: &str| {
        let labels = if mode == "drop-payload" {
            empty()
        } else {
            labels.clone()
        };
        took(
            command,
            outcome,
            "demo.c.Counted",
            Fields::from([("labels".to_owned(), labels)]),
        )
    };
    let seen = |outcome: &str| {
        took(
            command,
            outcome,
            "demo.c.Seen",
            Fields::from([("tag".to_owned(), get(input, "tag"))]),
        )
    };
    match command {
        "demo.c.Zero" => {
            let empty = if mode == "zero-at-most-one" {
                n <= 1
            } else {
                n == 0
            };
            counted(if empty { "empty" } else { "full" })
        }
        "demo.c.Many" => {
            let many = if mode == "many-at-least-one" {
                n >= 1
            } else {
                n > 1
            };
            counted(if many { "many" } else { "few" })
        }
        "demo.c.Three" => {
            let lots = match mode {
                "three-above-three" => n > 3,
                "three-at-least-two" => n >= 2,
                _ => n >= 3,
            };
            counted(if lots { "lots" } else { "few" })
        }
        "demo.c.IntMany" => {
            let tally = get(input, "counts");
            let k = entries(&tally);
            let many = if mode == "int-many-at-least-one" {
                k >= 1
            } else {
                k > 1
            };
            took(
                command,
                if many { "many" } else { "few" },
                "demo.c.Tallied",
                Fields::from([("counts".to_owned(), tally)]),
            )
        }
        "demo.c.LineMany" => {
            let k = match get(input, "lines") {
                Node::Seq(lines) => lines
                    .first()
                    .and_then(Node::as_map)
                    .and_then(|line| line.get("labels"))
                    .map_or(0, entries),
                _ => 0,
            };
            let many = if mode == "line-many-at-least-one" {
                k >= 1
            } else {
                k > 1
            };
            seen(if many { "many" } else { "few" })
        }
        "demo.c.ListThree" => {
            let k = match get(input, "items") {
                Node::Seq(items) => items.len(),
                _ => 0,
            };
            let lots = if mode == "list-three-at-least-two" {
                k >= 2
            } else {
                k >= 3
            };
            seen(if lots { "lots" } else { "few" })
        }
        other => panic!("unexpected command {other}"),
    }
}

fn tree(mode: &str, command: &str, input: &Fields, _: &mut Vec<Fields>) -> SemanticCommandResult {
    let mut tree = get(input, "tree");
    if mode == "drop-children" {
        tree = with_field(&tree, "children", empty());
    }
    took(
        command,
        "planted",
        "demo.r.Planted",
        Fields::from([("tree".to_owned(), tree)]),
    )
}

fn twins(
    mode: &str,
    command: &str,
    input: &Fields,
    rows: &mut Vec<Fields>,
) -> SemanticCommandResult {
    let mut row = input.clone();
    match mode {
        "left-into-right" => {
            row.insert("right".to_owned(), get(input, "left"));
        }
        "x-into-y" => {
            row.insert("y".to_owned(), get(input, "x"));
        }
        _ => {}
    }
    rows.push(row);
    took(
        command,
        "made",
        "demo.t.Made",
        Fields::from([("pair_id".to_owned(), get(input, "pair_id"))]),
    )
}

fn not_passed(verdicts: &Verdicts) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, status)| format!("{id}={status}"))
        .collect()
}

/// Runs every mode through the Rust runner; returns the verdicts per mode.
fn rust_modes(
    suite: &ConformanceSuite,
    handler: Handler,
    modes: &[&'static str],
) -> BTreeMap<&'static str, Verdicts> {
    modes
        .iter()
        .map(|mode| {
            (
                *mode,
                support_go::rust_outcomes(suite, &Store::new(mode, handler)),
            )
        })
        .collect()
}

/// The modes after the first that every scenario still passes: wrong targets the suite accepts.
fn uncaught(results: &BTreeMap<&'static str, Verdicts>, correct: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (mode, verdicts) in results {
        let failing = not_passed(verdicts);
        if *mode == correct {
            if !failing.is_empty() {
                problems.push(format!("the correct target fails: {failing:?}"));
            }
        } else if failing.is_empty() {
            problems.push(format!("mode `{mode}` passes every scenario"));
        }
    }
    problems
}

// ---- the issue's defect, through the Rust runner --------------------------------------------------

const ORDER_MODES: &[&str] = &[
    "correct",
    "drop-tags",
    "omit-tags",
    "drop-attrs",
    "drop-meta",
    "blank-value",
    "rekey",
    "retag-ignored",
    "drop-payload",
];

#[test]
fn adv196_a_target_that_drops_or_mangles_a_copied_map_fails_the_rust_runner() {
    let suite = suite(ORDERS);
    let results = rust_modes(&suite, orders, ORDER_MODES);
    let problems = uncaught(&results, "correct");
    assert!(problems.is_empty(), "{problems:#?}\n{results:#?}");
}

const KEY_MODES: &[&str] = &[
    "correct",
    "drop-number",
    "rekey-number",
    "blank-id",
    "drop-time",
    "drop-flag",
    "drop-inner",
    "drop-pair-flags",
    "omit-maybe",
    "drop-maybe",
    "drop-holder-ids",
];

#[test]
fn adv196_every_key_primitive_and_nesting_is_checked_by_the_rust_runner() {
    let suite = suite(KEYS);
    let results = rust_modes(&suite, keys, KEY_MODES);
    let problems = uncaught(&results, "correct");
    assert!(problems.is_empty(), "{problems:#?}\n{results:#?}");
}

/// A map inside a list element: the claim names maps inside lists.
#[test]
fn adv196_a_map_inside_a_copied_list_is_checked() {
    let suite = suite(KEYS);
    let results = rust_modes(&suite, keys, &["correct", "drop-holders-ids"]);
    let problems = uncaught(&results, "correct");
    assert!(problems.is_empty(), "{problems:#?}\n{results:#?}");
}

// ---- count guards ---------------------------------------------------------------------------------

#[test]
fn adv196_count_guards_over_maps_are_witnessed_on_both_sides() {
    let suite = suite(COUNTS);
    let mut wrong = Vec::new();
    for (id, command, field, holds) in [
        (
            "demo.c.Zero/outcome/empty",
            "demo.c.Zero",
            "labels",
            (|n| n == 0) as fn(usize) -> bool,
        ),
        ("demo.c.Zero/outcome/full", "demo.c.Zero", "labels", |n| {
            n != 0
        }),
        ("demo.c.Many/outcome/many", "demo.c.Many", "labels", |n| {
            n > 1
        }),
        ("demo.c.Many/outcome/few", "demo.c.Many", "labels", |n| {
            n <= 1
        }),
        ("demo.c.Three/outcome/lots", "demo.c.Three", "labels", |n| {
            n >= 3
        }),
        ("demo.c.Three/outcome/few", "demo.c.Three", "labels", |n| {
            n < 3
        }),
        (
            "demo.c.IntMany/outcome/many",
            "demo.c.IntMany",
            "counts",
            |n| n > 1,
        ),
        (
            "demo.c.IntMany/outcome/few",
            "demo.c.IntMany",
            "counts",
            |n| n <= 1,
        ),
    ] {
        let input = sent(scenario(&suite, id), command);
        let n = entries(&input[field]);
        if !holds(n) {
            wrong.push(format!("{id}: sent {n} entries: {:?}", input[field]));
        }
        // Every entry is a distinct key; a count witness that collapsed keys holds fewer.
        if let Some(map) = input[field].as_map() {
            let values: std::collections::BTreeSet<_> = map.keys().collect();
            assert_eq!(values.len(), map.len());
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

const COUNT_MODES: &[&str] = &[
    "correct",
    "zero-at-most-one",
    "many-at-least-one",
    "three-above-three",
    "int-many-at-least-one",
    "line-many-at-least-one",
    "drop-payload",
];

#[test]
fn adv196_an_off_by_one_count_guard_is_caught_by_the_rust_runner() {
    let suite = suite(COUNTS);
    let results = rust_modes(&suite, counts, COUNT_MODES);
    let problems = uncaught(&results, "correct");
    assert!(problems.is_empty(), "{problems:#?}\n{results:#?}");
}

/// `>= 3` implemented as `>= 2`: only a false-side witness of exactly 2 entries tells them apart.
#[test]
fn adv196_a_count_guard_at_least_three_is_witnessed_at_its_false_boundary() {
    let suite = suite(COUNTS);
    let results = rust_modes(&suite, counts, &["correct", "three-at-least-two"]);
    let problems = uncaught(&results, "correct");
    let sent_few = sent(scenario(&suite, "demo.c.Three/outcome/few"), "demo.c.Three");
    assert!(
        problems.is_empty(),
        "{problems:#?}; the false side sent {:?}",
        sent_few["labels"]
    );
}

/// The same guard over a list, so the boundary finding above can be told apart from a list rule.
#[test]
fn adv196_control_a_list_count_guard_at_least_three_is_witnessed_at_its_false_boundary() {
    let suite = suite(COUNTS);
    let results = rust_modes(&suite, counts, &["correct", "list-three-at-least-two"]);
    let problems = uncaught(&results, "correct");
    let sent_few = sent(
        scenario(&suite, "demo.c.ListThree/outcome/few"),
        "demo.c.ListThree",
    );
    assert!(
        problems.is_empty(),
        "{problems:#?}; the false side sent {:?}",
        sent_few["items"]
    );
}

/// A Boolean-keyed map holds two entries at most; no scenario may claim three.
#[test]
fn adv196_a_boolean_keyed_count_guard_never_sends_an_input_that_misses_its_outcome() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(FLAGS));
    let suite = synthesis.suite;
    let both = sent(scenario(&suite, "demo.f.Two/outcome/both"), "demo.f.Two");
    assert_eq!(entries(&both["flags"]), 2, "{both:?}");
    if let Some((_, lots)) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.f.Three/outcome/lots")
    {
        let input = sent(lots, "demo.f.Three");
        assert!(
            entries(&input["flags"]) >= 3,
            "a lots scenario sent {input:?}"
        );
    }
}

// ---- regressions ------------------------------------------------------------------------------

/// A type that reaches itself only through a map had the witness `{}`; it must still synthesize,
/// and a correct echo must still pass.
#[test]
fn adv196_a_type_recursive_through_a_map_still_synthesizes_and_passes() {
    let model = ir(TREE);
    let plant = model
        .commands()
        .get(&ess_domain::name::QualifiedName::new("demo.r.Plant").unwrap())
        .unwrap();
    let base = ess_conformance::witness::candidates(
        &model,
        plant,
        &[],
        ess_conformance::witness::Distinction::PLAIN,
    );
    let depth = |mut node: &Node| {
        let mut levels = 0;
        while let Some((_, child)) = node
            .as_map()
            .and_then(|tree| tree.get("children"))
            .and_then(Node::as_map)
            .and_then(|children| children.iter().next())
        {
            levels += 1;
            node = child;
        }
        levels
    };
    match &base {
        Ok(options) if !options.is_empty() => println!(
            "{} candidate(s); the base tree nests {} map level(s)",
            options.len(),
            depth(&options[0]["tree"])
        ),
        other => panic!("the base witness of a map-recursive type: {other:?}"),
    }
    if let Ok(options) = &base {
        let input = &options[0];
        let admitted = ess_conformance::flatten(&model, plant, input);
        assert!(
            admitted.is_ok(),
            "the base witness is refused by the flattener: {}",
            admitted.err().map(|e| e.to_string()).unwrap_or_default()
        );
    }
    let suite = suite(TREE);
    let results = rust_modes(&suite, tree, &["correct", "drop-children"]);
    let problems = uncaught(&results, "correct");
    assert!(problems.is_empty(), "{problems:#?}\n{results:#?}");
}

/// Two maps of `Map<Integer, Integer>` copied to two fields: a target that copies one into both.
#[test]
fn adv196_two_integer_maps_carry_different_witnesses() {
    let suite = suite(TWINS);
    let results = rust_modes(&suite, twins, &["correct", "left-into-right"]);
    let problems = uncaught(&results, "correct");
    let input = sent(scenario(&suite, "demo.t.Make/outcome/made"), "demo.t.Make");
    assert!(
        problems.is_empty(),
        "{problems:#?}; sent left={:?} right={:?}",
        input["left"],
        input["right"]
    );
}

/// Control for the finding above: the scalar Integer twin.
#[test]
fn adv196_control_two_integer_scalars_carry_different_witnesses() {
    let suite = suite(TWINS);
    let results = rust_modes(&suite, twins, &["correct", "x-into-y"]);
    let problems = uncaught(&results, "correct");
    let input = sent(scenario(&suite, "demo.t.Make/outcome/made"), "demo.t.Make");
    assert!(
        problems.is_empty(),
        "{problems:#?}; sent x={:?} y={:?}",
        input["x"],
        input["y"]
    );
}

fn load(directory: &Path) -> EssIr {
    let mut files = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in files {
        let label = path.strip_prefix(directory).unwrap().display().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|e| panic!("{label}: {e}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let spec =
        Specification::assemble(parsed).unwrap_or_else(|e| panic!("{}: {e}", directory.display()));
    compile(&spec, &sources).unwrap_or_else(|e| panic!("{}: {e}", directory.display()))
}

/// Every committed generated scenario is what synthesis writes today, and no example gains a
/// refusal that names a map.
#[test]
fn adv196_every_example_still_synthesizes_to_its_committed_suite() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut drift = Vec::new();
    for (example, golden) in [
        (
            "examples/billing",
            Some("suites/generated/billing/suite.json"),
        ),
        (
            "examples/gatepass",
            Some("suites/generated/gatepass/suite.json"),
        ),
        (
            "examples/oracle-fixture",
            Some("suites/generated/oracle-fixture/suite.json"),
        ),
        ("examples/revision-pair/before", None),
        ("examples/revision-pair/after", None),
    ] {
        let synthesis = ess_conformance::synthesize::synthesize(&load(&root.join(example)));
        for refusal in &synthesis.refusals {
            let text = refusal.to_string();
            // The ESS-SYNTH-013 help text lists "a list, a map, a union" whatever the model holds.
            let reason = text.replace("outside a list, a map, a union", "");
            if reason.to_lowercase().contains("map") {
                drift.push(format!("{example}: refusal names a map: {text}"));
            }
        }
        let Some(golden) = golden else { continue };
        let committed: Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(golden)).unwrap()).unwrap();
        let written = serde_json::to_value(&synthesis.suite).unwrap();
        let committed = committed["scenarios"].as_object().unwrap();
        let written = written["scenarios"].as_object().unwrap();
        for (id, scenario) in written {
            match committed.get(id) {
                Some(held) if held == scenario => {}
                Some(_) => drift.push(format!("{golden}: {id} differs from the committed bytes")),
                None => drift.push(format!("{golden}: {id} is not committed")),
            }
        }
        for id in committed.keys() {
            if !written.contains_key(id) && !id.contains("/authored/") {
                drift.push(format!("{golden}: committed {id} is no longer synthesized"));
            }
        }
    }
    assert!(drift.is_empty(), "{drift:#?}");
}

// ---- the generated Go runtime ---------------------------------------------------------------------

fn go_modes(
    label: &str,
    suite: &ConformanceSuite,
    handler: Handler,
    modes: &[&'static str],
) -> Vec<String> {
    let mut problems = Vec::new();
    for mode in modes {
        let verdicts = support_go::assert_parity(
            &format!("adv196-{label}-{mode}"),
            suite,
            Store::new(mode, handler),
        );
        let failing = not_passed(&verdicts);
        if *mode == "correct" && !failing.is_empty() {
            problems.push(format!("correct fails in Go: {failing:?}"));
        }
        if *mode != "correct" && failing.is_empty() {
            problems.push(format!("mode `{mode}` passes every scenario in Go"));
        }
    }
    problems
}

#[test]
fn adv196_go_runtime_agrees_and_catches_every_dropped_map() {
    let mut problems = go_modes("orders", &suite(ORDERS), orders, ORDER_MODES);
    problems.extend(go_modes("keys", &suite(KEYS), keys, KEY_MODES));
    problems.extend(go_modes("counts", &suite(COUNTS), counts, COUNT_MODES));
    // TREE is left out: it no longer synthesizes (the recursion case above).
    assert!(problems.is_empty(), "{problems:#?}");
}

// ---- the generated TypeScript runtime -------------------------------------------------------------

fn toolchain() -> bool {
    ["tsc", "node"].iter().all(|name| {
        let found = Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if !found {
            println!("skipped: no `{name}` on PATH, so the TypeScript runtime was not run");
        }
        found
    })
}

const JS_PRELUDE: &str = r"import { unsupported } from './dist/runtime.js';
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const took = (outcome, event, payload) => ({ outcome, consistency: 'write', directEvents: [{ event, payload }] });
const size = (value) => (value !== null && typeof value === 'object' ? Object.keys(value).length : 0);
const eachValue = (map, edit) => Object.fromEntries(Object.entries(map ?? {}).map(([k, v]) => [k, edit(v)]));
const eachKey = (map, edit) => Object.fromEntries(Object.entries(map ?? {}).map(([k, v]) => [edit(k), v]));
class Store {
  rows = [];
  identity() { return { name: 'map-witness-adversary', version: '1' }; }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) { return handle(command, input, this.rows); }
  queryView() { return { rows: this.rows.map((row) => ({ ...row })) }; }
  configureExternalOutcome() { throw unsupported('the model declares none'); }
  redeliverEvent() { throw unsupported('unused'); }
  observeEvents() { throw unsupported('unused'); }
}
export function makeTarget() { return new Store(); }
";

const JS_ORDERS: &str = r"function handle(command, input, rows) {
  const payload = { order_id: input.order_id, tags: mode === 'drop-payload' ? {} : input.tags };
  if (command === 'demo.orders.ItemPushed') {
    const row = { order_id: input.order_id, item: { sku: input.sku, attrs: input.attrs }, tags: input.tags, meta: input.meta };
    if (mode === 'drop-tags') row.tags = {};
    if (mode === 'omit-tags') delete row.tags;
    if (mode === 'drop-attrs') row.item = { ...row.item, attrs: {} };
    if (mode === 'drop-meta') row.meta = {};
    if (mode === 'blank-value') row.tags = eachValue(input.tags, () => '');
    if (mode === 'rekey') row.tags = eachKey(input.tags, () => 'k');
    rows.push(row);
    return took('created', 'demo.orders.Changed', payload);
  }
  const row = rows.find((held) => held.order_id === input.order_id);
  if (row === undefined) return {};
  if (mode !== 'retag-ignored') row.tags = input.tags;
  return took('retagged', 'demo.orders.Changed', payload);
}
";

const JS_KEYS: &str = r"function handle(command, input, rows) {
  const row = { ...input };
  const edit = (field, f) => { if (field in row) row[field] = f(row[field]); };
  if (mode === 'drop-number') edit('by_number', () => ({}));
  if (mode === 'rekey-number') edit('by_number', (m) => eachKey(m, (k) => `${k}.0`));
  if (mode === 'blank-id') edit('by_id', (m) => eachValue(m, () => ''));
  if (mode === 'drop-time') edit('by_time', () => ({}));
  if (mode === 'drop-flag') edit('by_flag', () => ({}));
  if (mode === 'drop-inner') edit('nested', (m) => eachValue(m, () => ({})));
  if (mode === 'drop-pair-flags') edit('pairs', (m) => eachValue(m, (p) => ({ ...p, flags: {} })));
  if (mode === 'omit-maybe') delete row.maybe;
  if (mode === 'drop-maybe') edit('maybe', () => ({}));
  if (mode === 'drop-holder-ids') edit('holder', (h) => ({ ...h, ids: {} }));
  rows.push(row);
  return took('packed', 'demo.k.Packed', { box_id: input.box_id });
}
";

/// The TypeScript runtime's verdict per mode, from its report/2.
fn typescript_modes(
    case: &str,
    suite: &ConformanceSuite,
    handler_js: &str,
    modes: &[&str],
) -> BTreeMap<String, Verdicts> {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-map-witness-pass1")
        .join(format!("{case}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(suite).expect("the package emits") {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    std::fs::write(dir.join("target.mjs"), format!("{JS_PRELUDE}{handler_js}")).unwrap();
    std::fs::write(
        dir.join("driver.mjs"),
        include_str!("fixtures/typescript-parity-driver.mjs"),
    )
    .unwrap();
    std::fs::write(
        dir.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&dir)
        .output()
        .expect("tsc runs");
    assert!(
        compiled.status.success(),
        "tsc refused the emitted package:\n{}{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut all = BTreeMap::new();
    for mode in modes {
        let report = dir.join(format!("report-{mode}.json"));
        let run = Command::new("node")
            .args(["--test", "driver.mjs"])
            .env("ESS_TARGET_MODE", mode)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&dir)
            .output()
            .expect("node runs");
        let text = std::fs::read_to_string(&report).unwrap_or_else(|_| {
            panic!(
                "{case} {mode}: no report\n{}{}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            )
        });
        let document: Value = serde_json::from_str(&text).unwrap();
        let mut verdicts = Verdicts::new();
        for (status, ids) in document["outcomes"].as_object().unwrap() {
            for id in ids.as_array().unwrap() {
                verdicts.insert(id.as_str().unwrap().to_owned(), status.clone());
            }
        }
        all.insert((*mode).to_owned(), verdicts);
    }
    let _ = std::fs::remove_dir_all(&root);
    all
}

#[test]
fn adv196_typescript_runtime_agrees_and_catches_every_dropped_map() {
    if !toolchain() {
        return;
    }
    let mut problems = Vec::new();
    for (case, model, handler, js, modes) in [
        ("orders", ORDERS, orders as Handler, JS_ORDERS, ORDER_MODES),
        ("keys", KEYS, keys as Handler, JS_KEYS, KEY_MODES),
    ] {
        let suite = suite(model);
        let rust = rust_modes(&suite, handler, modes);
        let typescript = typescript_modes(case, &suite, js, modes);
        for mode in modes {
            let ts = &typescript[*mode];
            if ts != &rust[mode] {
                problems.push(format!(
                    "{case} {mode}: TypeScript {:?} vs Rust {:?}",
                    not_passed(ts),
                    not_passed(&rust[mode])
                ));
            }
            if *mode == "correct" && !not_passed(ts).is_empty() {
                problems.push(format!(
                    "{case}: correct fails in TypeScript: {:?}",
                    not_passed(ts)
                ));
            }
            if *mode != "correct" && not_passed(ts).is_empty() {
                problems.push(format!(
                    "{case}: mode `{mode}` passes every scenario in TypeScript"
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
