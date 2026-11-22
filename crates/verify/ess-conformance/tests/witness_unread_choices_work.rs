//! The witness walk builds no candidate whose input it already holds because one of its choices
//! is read by no build.
//!
//! A guard that reads into a list varies the leaves of the list's element. While a candidate
//! leaves the list empty, the element is built and dropped, so a choice at one of its leaves
//! changes nothing. Before this change the walk built every such candidate in full, and then
//! dropped it as a repeat. A downstream specification spent about a fifth of its first synthesis
//! reading in this walk. The walk over one guard reading two leaves of a list's element is held to a bound on the
//! inputs it builds. It is a work bound, not a clock.

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate;
use ess_conformance::when;
use ess_conformance::witness::{builders_cloned, candidates, inputs_built, Distinction};
use ess_domain::name::QualifiedName;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const MODEL: &str = r"
format: ess/32
system: demo
version: v1
domain: demo.depot
types:
  - {name: demo.depot.OrderId, kind: newtype, of: Uuid}
  - {name: demo.depot.Kind, kind: enum, variants: [single, bulk, sample]}
  - name: demo.depot.Line
    kind: struct
    fields:
      - {name: kind, type: demo.depot.Kind}
      - {name: qty, type: Integer}
      - {name: note, type: String}
errors:
  - {name: demo.depot.TooLarge, summary: A bulk line is too large., fields: []}
entities:
  - name: demo.depot.Order
    identity: {name: order_id, type: demo.depot.OrderId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}
events:
  - name: demo.depot.OrderPlaced
    fields: [{name: order_id, type: demo.depot.OrderId}]
actors:
  - {name: demo.depot.Clerk, may: [demo.depot.PlaceOrder]}
commands:
  - name: demo.depot.PlaceOrder
    input:
      - {name: label, type: String}
      - {name: lines, type: 'List<demo.depot.Line>'}
    outcomes:
      - name: too-large
        when:
          exists:
            in: input.lines
            as: l
            that: {all: ['l.kind == bulk', 'l.qty > 5']}
        error: demo.depot.TooLarge
      - name: placed
        creates: demo.depot.Order
        instance: order_id
        emits: [demo.depot.OrderPlaced]
        payload:
          demo.depot.OrderPlaced: {order_id: {generated: true}}
        sets: {label: input.label}
";

/// The inputs the walk over the guard may build. Before the change it built 37, 17 of them a
/// repeat whose only difference was an unread choice; with it, it builds 20.
const BOUND: usize = 24;

fn compiled(model: &str) -> EssIr {
    let raw = RawSpecFile::parse(model).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("depot.yaml".to_owned(), model.to_owned());
    mutate::compile(vec![(Source::new("depot.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn command(ir: &EssIr) -> &ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new("demo.depot.PlaceOrder").expect("a valid name"))
        .expect("the command is declared")
}

#[test]
fn a_walk_over_a_list_element_builds_no_input_it_already_holds() {
    let ir = compiled(MODEL);
    let command = command(&ir);
    let guard = command
        .outcomes
        .iter()
        .find_map(when)
        .expect("the refusal is guarded");
    let (found, built) = inputs_built(|| candidates(&ir, command, &[guard], Distinction::PLAIN));
    let found = found.expect("the walk finds candidates");
    assert!(
        built <= BOUND,
        "the walk built {built} inputs for {} candidates, over the bound of {BOUND}",
        found.len()
    );
}

/// A command whose branch copies a list of structs whole, each element holding a copied list and a
/// map of structs of its own: every candidate builds one element of each, at every level.
const COPIED: &str = r"
format: ess/32
system: demo
version: v1
domain: demo.depot
types:
  - {name: demo.depot.OrderId, kind: newtype, of: Uuid}
  - {name: demo.depot.Kind, kind: enum, variants: [single, bulk, sample]}
  - name: demo.depot.Tag
    kind: struct
    fields:
      - {name: key, type: String}
      - {name: weight, type: Optional<Integer>}
  - name: demo.depot.Part
    kind: struct
    fields:
      - {name: sku, type: String}
      - {name: tags, type: 'List<demo.depot.Tag>'}
      - {name: by_key, type: 'Map<String, demo.depot.Tag>'}
  - name: demo.depot.Line
    kind: struct
    fields:
      - {name: kind, type: demo.depot.Kind}
      - {name: qty, type: Integer}
      - {name: parts, type: 'List<demo.depot.Part>'}
errors:
  - {name: demo.depot.TooLarge, summary: An order is too large., fields: []}
entities:
  - name: demo.depot.Order
    identity: {name: order_id, type: demo.depot.OrderId}
    fields:
      - {name: lines, type: 'List<demo.depot.Line>'}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}
events:
  - name: demo.depot.OrderPlaced
    fields: [{name: order_id, type: demo.depot.OrderId}]
actors:
  - {name: demo.depot.Clerk, may: [demo.depot.PlaceOrder]}
commands:
  - name: demo.depot.PlaceOrder
    input:
      - {name: lines, type: 'List<demo.depot.Line>'}
      - {name: flag_1, type: Integer}
      - {name: flag_2, type: Integer}
    outcomes:
      - name: too-large
        when: {all: ['input.flag_1 > 5', 'input.flag_2 < 3']}
        error: demo.depot.TooLarge
      - name: placed
        creates: demo.depot.Order
        instance: order_id
        emits: [demo.depot.OrderPlaced]
        payload:
          demo.depot.OrderPlaced: {order_id: {generated: true}}
        sets: {lines: input.lines}
";

/// The copied list's element, and every copied list and map inside it, is built on the builder of
/// the input itself: no candidate copies the whole builder for one. Before the change each such
/// element was built on a copy, kept on success, so the walk over this guard copied the whole
/// builder for every element of every input it built.
#[test]
fn a_copied_list_element_is_built_without_copying_the_builder() {
    let ir = compiled(COPIED);
    let command = command(&ir);
    let guard = command
        .outcomes
        .iter()
        .find_map(when)
        .expect("the refusal is guarded");
    let ((found, built), cloned) =
        builders_cloned(|| inputs_built(|| candidates(&ir, command, &[guard], Distinction::PLAIN)));
    let found = found.expect("the walk finds candidates");
    assert!(built > 1, "the walk builds more than the base: {built}");
    assert_eq!(
        cloned,
        0,
        "the walk copied the whole builder {cloned} times while it built {built} inputs for {} \
         candidates",
        found.len()
    );
}
