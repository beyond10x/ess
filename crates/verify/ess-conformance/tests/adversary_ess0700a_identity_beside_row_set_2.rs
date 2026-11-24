//! Adversary pass 2, ESS 0.70.0 unit A (beyond10x/ess#554): the synthesized suite of a command
//! guarding on rows its input names beside a row set, run against a target answering the step-4
//! refusals in every order. A suite that passes against an order the rule tells apart on some
//! reachable request has no scenario holding that request.

mod support_row_sets;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_primitives::node::Node;
use support_row_sets::model;

const REPRO: &str =
    include_str!("../../../specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml");

const NO_SOURCE: &str = "      - name: no-source
        when_related: {via: input.from_port, exists: false}
        error: wiring.board.SourceNotFound
";
const NO_SINK: &str = "      - name: no-sink
        when_related: {via: input.to_port, exists: false}
        error: wiring.board.SinkNotFound
";
const SOURCE_NOT_OUTPUT: &str = "      - name: source-not-output
        when_related: {via: input.from_port, predicate: direction != output}
        error: wiring.board.SourceNotOutput
";
const SINK_NOT_INPUT: &str = "      - name: sink-not-input
        when_related: {via: input.to_port, predicate: direction != input}
        error: wiring.board.SinkNotInput
";
const SOURCE_TAKEN: &str = "      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          exists: true
        error: wiring.board.SourceTaken
";
const CONNECTED: &str = "      - name: connected
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
        sets: {from_port: input.from_port, to_port: input.to_port}
";
/// The first place a row-set refusal may be declared: below every `exists: false` branch over a
/// named row, which answers before it in either order (`ess/27`, `conflicting_declaration`).
const ABOVE_PREDICATES: &str = "      - name: source-not-output\n";
const MAY: &str = "    may: [wiring.board.AddPort, wiring.board.Connect]\n";
const VIEWS: &str = "views:\n";

const NS: &str = "no-source";
const NK: &str = "no-sink";
const SON: &str = "source-not-output";
const SNI: &str = "sink-not-input";
const ST: &str = "source-taken";
const REV: &str = "reversed";
const DUP: &str = "duplicate";

fn edited(text: &str, before: &str, after: &str) -> String {
    let changed = text.replacen(before, after, 1);
    assert_ne!(changed, text, "`{before}` is in the model");
    changed
}

/// `branch` cut out of `text` and declared again right above `anchor`.
fn moved(text: &str, branch: &str, anchor: &str) -> String {
    let cut = edited(text, branch, "");
    edited(&cut, anchor, &format!("{branch}{anchor}"))
}

/// The reproduction with an unguarded second creator of wires: an import may lay a wire from a
/// port no row carries, so a wire can leave a missing or input-facing source.
fn with_import(text: &str) -> String {
    let import = "  - name: wiring.board.ImportWire
    input:
      - {name: from_port, type: wiring.board.PortId}
      - {name: to_port, type: wiring.board.PortId}
    outcomes:
      - name: imported
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
        sets: {from_port: input.from_port, to_port: input.to_port}
";
    let text = edited(text, VIEWS, &format!("{import}{VIEWS}"));
    edited(
        &text,
        MAY,
        "    may: [wiring.board.AddPort, wiring.board.Connect, wiring.board.ImportWire]\n",
    )
}

/// The reproduction where wires are laid by a command of their own, `Lay`, guarded by `guards`
/// (named-row refusals over its input), and `Connect` records a patch instead: the row set's rows
/// have a creator other than the command under test. `rack` adds a third entity the creator names.
fn with_layer(guards: &str, rack: bool) -> String {
    let mut text = edited(
        REPRO,
        "  - {name: wiring.board.WireId, kind: newtype, of: Uuid}\n",
        "  - {name: wiring.board.WireId, kind: newtype, of: Uuid}
  - {name: wiring.board.PatchId, kind: newtype, of: Uuid}
  - {name: wiring.board.RackId, kind: newtype, of: Uuid}
",
    );
    text = edited(
        &text,
        "errors:\n",
        "  - name: wiring.board.Patch
    identity: {name: patch_id, type: wiring.board.PatchId}
    fields:
      - {name: from_port, type: wiring.board.PortId}
      - {name: to_port, type: wiring.board.PortId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: wiring.board.Rack
    identity: {name: rack_id, type: wiring.board.RackId}
    fields:
      - {name: direction, type: wiring.board.Direction}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: wiring.board.RackNotFound, summary: No rack carries the identity., fields: []}
",
    );
    text = edited(
        &text,
        "  - name: wiring.board.Connected\n",
        "  - name: wiring.board.Patched
    fields: [{name: patch_id, type: wiring.board.PatchId}]
  - name: wiring.board.RackAdded
    fields: [{name: rack_id, type: wiring.board.RackId}]
  - name: wiring.board.Connected\n",
    );
    text = edited(
        &text,
        CONNECTED,
        "      - name: connected
        creates: wiring.board.Patch
        instance: patch_id
        emits: [wiring.board.Patched]
        payload: {wiring.board.Patched: {patch_id: {generated: true}}}
        sets: {from_port: input.from_port, to_port: input.to_port}
",
    );
    let (rack_field, rack_input, rack_set) = if rack {
        (
            "      - {name: rack_id, type: wiring.board.RackId}\n",
            "      - {name: rack_id, type: wiring.board.RackId}\n",
            ", rack_id: input.rack_id",
        )
    } else {
        ("", "", "")
    };
    if rack {
        text = edited(
            &text,
            "      - {name: to_port, type: wiring.board.PortId}\n    relations:",
            &format!(
                "      - {{name: to_port, type: wiring.board.PortId}}\n{rack_field}    relations:"
            ),
        );
    }
    let commands = format!(
        "  - name: wiring.board.AddRack
    input:
      - {{name: direction, type: wiring.board.Direction}}
    outcomes:
      - name: racked
        creates: wiring.board.Rack
        instance: rack_id
        emits: [wiring.board.RackAdded]
        payload: {{wiring.board.RackAdded: {{rack_id: {{generated: true}}}}}}
        sets: {{direction: input.direction}}
  - name: wiring.board.Lay
    input:
      - {{name: from_port, type: wiring.board.PortId}}
      - {{name: to_port, type: wiring.board.PortId}}
{rack_input}    outcomes:
{guards}      - name: laid
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {{wiring.board.Connected: {{wire_id: {{generated: true}}}}}}
        sets: {{from_port: input.from_port, to_port: input.to_port{rack_set}}}
"
    );
    text = edited(&text, VIEWS, &format!("{commands}{VIEWS}"));
    edited(
        &text,
        MAY,
        "    may: [wiring.board.AddPort, wiring.board.Connect, wiring.board.AddRack, wiring.board.Lay]\n",
    )
}

/// What a request's rows hold, as the target under test tracks them.
#[derive(Default)]
struct World {
    ports: Vec<(Node, String)>,
    wires: Vec<(Node, Node)>,
}

impl World {
    fn port(&self, id: Option<&Node>) -> Option<&str> {
        let id = id?;
        self.ports
            .iter()
            .find(|(port, _)| port == id)
            .map(|(_, direction)| direction.as_str())
    }
}

fn word(node: &Node) -> String {
    match node {
        Node::Text(text) => text.clone(),
        other => format!("{other:?}"),
    }
}

/// How one variant's refusals are decided.
#[derive(Clone, Copy)]
struct Shape {
    /// The row set selects wires into the sink rather than out of the source.
    by_sink: bool,
    /// How many selected wires the row set refuses at.
    at_least: usize,
    /// `sink-not-input` carries `when: input.from_port != input.to_port`.
    sink_apart: bool,
    /// The commands whose accepting branch lays a wire from its input.
    lays: &'static [(&'static str, &'static str)],
}

const CONNECT_LAYS: &[(&str, &str)] = &[("wiring.board.Connect", "connected")];
const IMPORT_LAYS: &[(&str, &str)] = &[
    ("wiring.board.Connect", "connected"),
    ("wiring.board.ImportWire", "imported"),
];
const LAYER_LAYS: &[(&str, &str)] = &[("wiring.board.Lay", "laid")];

const PLAIN: Shape = Shape {
    by_sink: false,
    at_least: 1,
    sink_apart: false,
    lays: CONNECT_LAYS,
};

/// Every refusal name that holds for `input` on `world`; a variant reads those it declares.
fn holds(shape: Shape, world: &World, input: &BTreeMap<String, Node>) -> Vec<&'static str> {
    let from = input.get("from_port");
    let to = input.get("to_port");
    let mut held = Vec::new();
    let source = world.port(from);
    let sink = world.port(to);
    match source {
        None => held.push(NS),
        Some(direction) if direction != "output" => held.push(SON),
        Some(_) => {}
    }
    match sink {
        None => held.push(NK),
        Some(direction) if direction != "input" && (!shape.sink_apart || from != to) => {
            held.push(SNI);
        }
        Some(_) => {}
    }
    if source == Some("input") && sink == Some("output") {
        held.push(REV);
    }
    let selected = world
        .wires
        .iter()
        .filter(|(wire_from, wire_to)| {
            if shape.by_sink {
                Some(wire_to) == to
            } else {
                Some(wire_from) == from
            }
        })
        .count();
    if selected >= shape.at_least {
        held.push(ST);
    }
    if world
        .wires
        .iter()
        .any(|(wire_from, wire_to)| Some(wire_from) == from && Some(wire_to) == to)
    {
        held.push(DUP);
    }
    held
}

fn error_of(branch: &str) -> &'static str {
    match branch {
        NS => "wiring.board.SourceNotFound",
        NK => "wiring.board.SinkNotFound",
        SON | REV => "wiring.board.SourceNotOutput",
        SNI => "wiring.board.SinkNotInput",
        _ => "wiring.board.SourceTaken",
    }
}

/// The interpreter of a variant, whose answer to `Connect` is rewritten to the first refusal of
/// `order` that holds: a target deciding the same refusals in another order. It also records
/// wherever the interpreter disagrees with [`holds`] under the rule's own order, so a red result
/// is about the suite, not this model of the store.
struct Ordered<'v> {
    inner: Interpreted,
    shape: Shape,
    rule: &'v [&'static str],
    order: Vec<&'static str>,
    world: RefCell<World>,
    disagreements: RefCell<Vec<String>>,
}

impl ConformanceTarget for Ordered<'_> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.world.borrow_mut() = World::default();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        match request.entity.to_string().as_str() {
            "wiring.board.Port" => {
                let direction = request.fields.get("direction").map(word);
                self.world
                    .borrow_mut()
                    .ports
                    .push((request.identity.clone(), direction.unwrap_or_default()));
            }
            "wiring.board.Wire" => {
                let from = request.fields.get("from_port").cloned().unwrap_or_default();
                let to = request.fields.get("to_port").cloned().unwrap_or_default();
                self.world.borrow_mut().wires.push((from, to));
            }
            _ => {}
        }
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        let input = request.input.clone();
        let mut answer = self.inner.execute_command(request)?;
        let taken = answer
            .outcome
            .as_ref()
            .map(|taken| taken.outcome.to_string());
        if command == "wiring.board.Connect" {
            let held = holds(self.shape, &self.world.borrow(), &input);
            let first = self.rule.iter().find(|branch| held.contains(*branch));
            let agrees = match (first, taken.as_deref()) {
                (Some(first), Some(taken)) => *first == taken,
                (None, Some(taken)) => !self.rule.contains(&taken),
                _ => false,
            };
            if !agrees {
                self.disagreements.borrow_mut().push(format!(
                    "{input:?}: rule {first:?}, interpreter {taken:?}, holding {held:?}"
                ));
            }
            if let Some(chosen) = self.order.iter().find(|branch| held.contains(*branch)) {
                if taken.as_deref() != Some(*chosen) {
                    if let Some(answered) = answer.outcome.as_mut() {
                        answered.outcome = chosen.parse().unwrap();
                    }
                    answer.error = Some(DeclaredErrorValue::new(error_of(chosen).parse().unwrap()));
                }
            }
        }
        if command == "wiring.board.AddPort" && taken.as_deref() == Some("added") {
            let id = answer
                .direct_events
                .iter()
                .find_map(|event| event.payload.get("port_id").cloned());
            let direction = input.get("direction").map(word);
            match (id, direction) {
                (Some(id), Some(direction)) => self.world.borrow_mut().ports.push((id, direction)),
                _ => self
                    .disagreements
                    .borrow_mut()
                    .push(format!("AddPort {input:?}: no port identity published")),
            }
        }
        if let Some(taken) = taken.as_deref() {
            if self
                .shape
                .lays
                .iter()
                .any(|(by, laid)| *by == command && *laid == taken)
            {
                let from = input.get("from_port").cloned().unwrap_or_default();
                let to = input.get("to_port").cloned().unwrap_or_default();
                self.world.borrow_mut().wires.push((from, to));
            }
        }
        Ok(answer)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// One model, its refusals in the rule's order, and the pairs of them some reachable request
/// holds together.
struct Variant {
    label: &'static str,
    text: String,
    rule: Vec<&'static str>,
    pairs: Vec<(&'static str, &'static str)>,
    shape: Shape,
}

/// The pairs the reproduction's requests hold together: one fact about the source beside one
/// about the sink.
fn source_sink_pairs() -> Vec<(&'static str, &'static str)> {
    vec![
        (NS, NK),
        (NS, SNI),
        (SON, NK),
        (SON, SNI),
        (ST, NK),
        (ST, SNI),
    ]
}

fn run<T: ConformanceTarget>(synthesis: &Synthesis, target: &T) -> BTreeMap<String, Status> {
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("{error}"));
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn permutations(items: &[&'static str]) -> Vec<Vec<&'static str>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut all = Vec::new();
    for (index, first) in items.iter().enumerate() {
        let mut rest = items.to_vec();
        rest.remove(index);
        for mut tail in permutations(&rest) {
            tail.insert(0, *first);
            all.push(tail);
        }
    }
    all
}

/// Which orders of a rule a sweep tries.
#[derive(Clone, Copy)]
enum Sweep {
    /// The rule's own order, each order one adjacent swap away from it (n orders), and the two
    /// transpositions of the non-adjacent pairs of the reproduction's requests that no adjacent
    /// swap reverses: (no-source, sink-not-input) and (source-taken, no-sink).
    Adjacent,
    /// Every order of the rule: n! orders.
    Exhaustive,
}

fn orders(rule: &[&'static str], sweep: Sweep) -> Vec<Vec<&'static str>> {
    match sweep {
        Sweep::Exhaustive => permutations(rule),
        Sweep::Adjacent => {
            let mut all = vec![rule.to_vec()];
            for at in 0..rule.len().saturating_sub(1) {
                let mut swapped = rule.to_vec();
                swapped.swap(at, at + 1);
                all.push(swapped);
            }
            for (a, b) in [(NS, SNI), (ST, NK)] {
                if let (Some(x), Some(y)) = (
                    rule.iter().position(|item| *item == a),
                    rule.iter().position(|item| *item == b),
                ) {
                    let mut swapped = rule.to_vec();
                    swapped.swap(x, y);
                    all.push(swapped);
                }
            }
            all
        }
    }
}

/// Whether `order` answers every reachable request as `rule` does.
fn equivalent(rule: &[&str], order: &[&str], pairs: &[(&str, &str)]) -> bool {
    let at = |list: &[&str], name: &str| list.iter().position(|item| *item == name);
    pairs.iter().all(
        |(a, b)| match (at(rule, a), at(rule, b), at(order, a), at(order, b)) {
            (Some(ra), Some(rb), Some(oa), Some(ob)) => (ra < rb) == (oa < ob),
            _ => true,
        },
    )
}

/// For each variant: synthesis refuses nothing and witnesses every refusal, the suite passes the
/// rule's own order, and fails exactly the orders the rule tells apart on a reachable request.
fn every_order_told_apart(variants: &[Variant], sweep: Sweep) {
    let mut report = Vec::new();
    for variant in variants {
        let synthesis = synthesize(&model(&variant.text));
        let refused: Vec<String> = synthesis
            .refusals
            .iter()
            .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
            .collect();
        if !refused.is_empty() {
            report.push(format!("{}: synthesis refused {refused:?}", variant.label));
        }
        for branch in &variant.rule {
            let id = format!("wiring.board.Connect/outcome/{branch}");
            if !synthesis
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id)
            {
                report.push(format!("{}: no scenario `{id}`", variant.label));
            }
        }
        let mut uncaught = Vec::new();
        let mut overcaught = Vec::new();
        let mut disagreements = Vec::new();
        for order in orders(&variant.rule, sweep) {
            let target = Ordered {
                inner: Interpreted::for_model(model(&variant.text)),
                shape: variant.shape,
                rule: &variant.rule,
                order: order.clone(),
                world: RefCell::new(World::default()),
                disagreements: RefCell::new(Vec::new()),
            };
            let statuses = run(&synthesis, &target);
            let failed: Vec<&String> = statuses
                .iter()
                .filter(|(_, status)| **status != Status::Passed)
                .map(|(id, _)| id)
                .collect();
            disagreements.extend(target.disagreements.into_inner());
            let same = equivalent(&variant.rule, &order, &variant.pairs);
            if same && !failed.is_empty() {
                overcaught.push(format!("{order:?} fails {failed:?}"));
            }
            if !same && failed.is_empty() {
                uncaught.push(format!("{order:?}"));
            }
        }
        disagreements.sort();
        disagreements.dedup();
        if !disagreements.is_empty() {
            report.push(format!(
                "{}: the store model disagrees with the interpreter: {:?}",
                variant.label,
                &disagreements[..disagreements.len().min(3)]
            ));
        }
        if !uncaught.is_empty() {
            report.push(format!(
                "{}: {} orders the rule tells apart pass the whole suite, e.g. {:?}",
                variant.label,
                uncaught.len(),
                &uncaught[..uncaught.len().min(3)]
            ));
        }
        if !overcaught.is_empty() {
            report.push(format!(
                "{}: {} orders the rule answers alike fail, e.g. {:?}",
                variant.label,
                overcaught.len(),
                &overcaught[..overcaught.len().min(3)]
            ));
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// The reproduction in each declaration order of its step-4 refusals.
fn variants_declaration_orders() -> Vec<Variant> {
    let rule_plain = vec![NS, NK, SON, SNI, ST];
    vec![
        Variant {
            label: "reproduction",
            text: REPRO.to_owned(),
            rule: rule_plain.clone(),
            pairs: source_sink_pairs(),
            shape: PLAIN,
        },
        Variant {
            label: "row set above the predicates",
            text: moved(REPRO, SOURCE_TAKEN, ABOVE_PREDICATES),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs: source_sink_pairs(),
            shape: PLAIN,
        },
        Variant {
            label: "row set between the predicates",
            text: moved(REPRO, SOURCE_TAKEN, SINK_NOT_INPUT),
            rule: vec![NS, NK, SON, ST, SNI],
            pairs: source_sink_pairs(),
            shape: PLAIN,
        },
        Variant {
            label: "sink predicate above source predicate",
            text: moved(REPRO, SINK_NOT_INPUT, SOURCE_NOT_OUTPUT),
            rule: vec![NS, NK, SNI, SON, ST],
            pairs: source_sink_pairs(),
            shape: PLAIN,
        },
        Variant {
            label: "missing sink above missing source",
            text: moved(REPRO, NO_SINK, NO_SOURCE),
            rule: vec![NK, NS, SON, SNI, ST],
            pairs: source_sink_pairs(),
            shape: PLAIN,
        },
    ]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_every_step_4_order_is_told_apart_across_declaration_orders() {
    every_order_told_apart(&variants_declaration_orders(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_every_step_4_order_is_told_apart_across_declaration_orders_in_adjacent_swaps() {
    every_order_told_apart(&variants_declaration_orders(), Sweep::Adjacent);
}

/// `count: {gte: 2}`: the scenario of a refusal answering before the row set has to hold two
/// selected wires, not one, or a target reading the row set first passes.
fn variants_count_row_set() -> Vec<Variant> {
    let count = edited(
        REPRO,
        "          exists: true\n",
        "          count: {gte: 2}\n",
    );
    vec![
        Variant {
            label: "count gte 2",
            text: count.clone(),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: source_sink_pairs(),
            shape: Shape {
                at_least: 2,
                ..PLAIN
            },
        },
        Variant {
            label: "count gte 2 above the predicates",
            text: moved(
                &count,
                &SOURCE_TAKEN.replace("          exists: true\n", "          count: {gte: 2}\n"),
                ABOVE_PREDICATES,
            ),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs: source_sink_pairs(),
            shape: Shape {
                at_least: 2,
                ..PLAIN
            },
        },
    ]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_count_row_set_orders_are_told_apart() {
    every_order_told_apart(&variants_count_row_set(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_count_row_set_orders_are_told_apart_in_adjacent_swaps() {
    every_order_told_apart(&variants_count_row_set(), Sweep::Adjacent);
}

/// The row set over the sink: an occupied sink beside a missing or input-facing source.
fn variants_row_set_over_other_row() -> Vec<Variant> {
    let sink = edited(
        REPRO,
        "          where: from_port == input.from_port\n",
        "          where: to_port == input.to_port\n",
    );
    let pairs = vec![
        (NS, NK),
        (NS, SNI),
        (SON, NK),
        (SON, SNI),
        (NS, ST),
        (SON, ST),
    ];
    vec![Variant {
        label: "row set over the sink",
        text: sink,
        rule: vec![NS, NK, SON, SNI, ST],
        pairs,
        shape: Shape {
            by_sink: true,
            ..PLAIN
        },
    }]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_row_set_over_the_other_named_row_orders_are_told_apart() {
    every_order_told_apart(&variants_row_set_over_other_row(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_row_set_over_the_other_named_row_orders_are_told_apart_in_adjacent_swaps() {
    every_order_told_apart(&variants_row_set_over_other_row(), Sweep::Adjacent);
}

/// A row set over a named row that is itself the missing one: an unguarded import lays a wire from
/// a port no row carries, so `no-source` and `source-taken` hold together and the rule answers
/// `no-source`. Also the row set over an input-facing source, and with one named row only.
fn variants_row_set_over_missing_row() -> Vec<Variant> {
    let import = with_import(REPRO);
    let all = vec![
        (NS, NK),
        (NS, SNI),
        (SON, NK),
        (SON, SNI),
        (ST, NK),
        (ST, SNI),
        (NS, ST),
        (SON, ST),
    ];
    let shape = Shape {
        lays: IMPORT_LAYS,
        ..PLAIN
    };
    let one_row = edited(&edited(REPRO, NO_SINK, ""), SINK_NOT_INPUT, "");
    vec![
        Variant {
            label: "imported wires",
            text: import.clone(),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: all.clone(),
            shape,
        },
        Variant {
            label: "imported wires, row set above the predicates",
            text: moved(&import, SOURCE_TAKEN, ABOVE_PREDICATES),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs: all,
            shape,
        },
        Variant {
            label: "imported wires, one named row",
            text: with_import(&one_row),
            rule: vec![NS, SON, ST],
            pairs: vec![(NS, ST), (SON, ST)],
            shape,
        },
    ]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_row_set_over_a_missing_named_row_orders_are_told_apart() {
    every_order_told_apart(&variants_row_set_over_missing_row(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_row_set_over_a_missing_named_row_orders_are_told_apart_in_adjacent_swaps() {
    every_order_told_apart(&variants_row_set_over_missing_row(), Sweep::Adjacent);
}

/// The row set's rows created by a command other than the one under test: a creator with no other
/// row input, one whose other input is the same entity as the named row, and one whose other input
/// is a third entity the scenario's own send never names.
fn variants_separate_creator() -> Vec<Variant> {
    let lay_source = "      - name: lay-no-source
        when_related: {via: input.from_port, exists: false}
        error: wiring.board.SourceNotFound
";
    let lay_sink = "      - name: lay-no-sink
        when_related: {via: input.to_port, exists: false}
        error: wiring.board.SinkNotFound
";
    let lay_rack = "      - name: lay-no-rack
        when_related: {via: input.rack_id, exists: false}
        error: wiring.board.RackNotFound
";
    let mut pairs = source_sink_pairs();
    pairs.push((SON, ST));
    let shape = Shape {
        lays: LAYER_LAYS,
        ..PLAIN
    };
    vec![
        Variant {
            label: "creator reads the source only",
            text: with_layer(lay_source, false),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: pairs.clone(),
            shape,
        },
        Variant {
            label: "creator reads the source and the sink",
            text: with_layer(&format!("{lay_source}{lay_sink}"), false),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: pairs.clone(),
            shape,
        },
        Variant {
            label: "creator reads the source and a rack",
            text: with_layer(&format!("{lay_source}{lay_rack}"), true),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs,
            shape,
        },
    ]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_separate_creator_orders_are_told_apart() {
    every_order_told_apart(&variants_separate_creator(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_separate_creator_orders_are_told_apart_in_adjacent_swaps() {
    every_order_told_apart(&variants_separate_creator(), Sweep::Adjacent);
}

const FORALL: &str = "        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          forall: to_port != input.to_port
";

/// Control for the case above, on the code below the unit's gate: the same `forall` and default on
/// a command with no named-row guard, its wires laid by a separate creator that refuses a missing
/// source, at `ess/31`. Red here means the `duplicate` gap is not the unit's.
#[test]
fn adversary_a2_forall_without_named_rows_witnesses_duplicate_at_ess_31() {
    let text = forall_without_named_rows_at_31();
    let synthesis = synthesize(&model(&text));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
        .collect();
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == "wiring.board.Connect/outcome/duplicate"),
        "no `duplicate` scenario at ess/31; refused {refused:#?}"
    );
}

fn forall_without_named_rows_at_31() -> String {
    let lay_source = "      - name: lay-no-source
        when_related: {via: input.from_port, exists: false}
        error: wiring.board.SourceNotFound
";
    let mut text = with_layer(lay_source, false);
    for branch in [
        NO_SOURCE,
        NO_SINK,
        SOURCE_NOT_OUTPUT,
        SINK_NOT_INPUT,
        SOURCE_TAKEN,
    ] {
        text = edited(&text, branch, "");
    }
    text = edited(
        &text,
        "      - name: connected\n        creates: wiring.board.Patch\n",
        &format!("      - name: connected\n{FORALL}        creates: wiring.board.Patch\n"),
    );
    text = edited(
        &text,
        "        sets: {from_port: input.from_port, to_port: input.to_port}\n  - name: wiring.board.AddRack\n",
        "        sets: {from_port: input.from_port, to_port: input.to_port}
      - name: duplicate
        error: wiring.board.SourceTaken
  - name: wiring.board.AddRack\n",
    );
    edited(&text, "format: ess/32\n", "format: ess/31\n")
}

/// A `when:` over the input on a named-row refusal beside the row set
/// (story:row-set-beside-named-rows-suite-tells-every-order-apart): `sink-not-input` answers only
/// where the ports differ. The row-set branch is still witnessed, and every order the rule tells
/// apart fails, in both declaration orders of the row set.
fn variants_when_beside_refusal() -> Vec<Variant> {
    let apart = edited(
        REPRO,
        "        when_related: {via: input.to_port, predicate: direction != input}\n",
        "        when_related: {via: input.to_port, predicate: direction != input}
        when: input.from_port != input.to_port
",
    );
    let shape = Shape {
        sink_apart: true,
        ..PLAIN
    };
    vec![
        Variant {
            label: "when: beside a named refusal",
            text: apart.clone(),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: source_sink_pairs(),
            shape,
        },
        Variant {
            label: "when: beside a named refusal, row set above the predicates",
            text: moved(&apart, SOURCE_TAKEN, ABOVE_PREDICATES),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs: source_sink_pairs(),
            shape,
        },
    ]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_when_beside_a_named_refusal_orders_are_told_apart() {
    every_order_told_apart(&variants_when_beside_refusal(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_when_beside_a_named_refusal_orders_are_told_apart_in_adjacent_swaps() {
    every_order_told_apart(&variants_when_beside_refusal(), Sweep::Adjacent);
}

const FORALL_LINE: &str = "          forall: to_port != input.to_port\n";

/// The reproduction with its row-set refusal replaced by a `forall` on the accepting branch beside
/// the four named-row refusals, the default answering a wire already joining the two ports.
fn forall_beside_named_rows() -> String {
    let text = edited(REPRO, SOURCE_TAKEN, "");
    let text = edited(
        &text,
        "      - name: connected\n        creates: wiring.board.Wire\n",
        &format!("      - name: connected\n{FORALL}        creates: wiring.board.Wire\n"),
    );
    edited(
        &text,
        "        sets: {from_port: input.from_port, to_port: input.to_port}\nviews:\n",
        "        sets: {from_port: input.from_port, to_port: input.to_port}
      - name: duplicate
        error: wiring.board.SourceTaken
views:
",
    )
}

/// The suite of `text` passes its interpreter, and fails each target deciding the `forall` of
/// `connected` another way: one refusing on any wire from the source, and one reading the
/// predicate negated. Each answers some reachable request differently: a wire from the source to
/// another sink, and a wire joining the two ports.
fn forall_mutants_killed(label: &str, text: &str) {
    let synthesis = synthesize(&model(text));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
        .collect();
    let mut report = Vec::new();
    let healthy = run(&synthesis, &Interpreted::for_model(model(text)));
    let failed: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    if !failed.is_empty() {
        report.push(format!("{label}: the interpreter fails {failed:?}"));
    }
    for (mutant, line) in [
        (
            "refusing on any wire from the source",
            "          count: {eq: 0}\n",
        ),
        (
            "reading the predicate negated",
            "          forall: to_port == input.to_port\n",
        ),
    ] {
        let changed = edited(text, FORALL_LINE, line);
        let statuses = run(&synthesis, &Interpreted::for_model(model(&changed)));
        if statuses.values().all(|status| *status == Status::Passed) {
            report.push(format!(
                "{label}: a target {mutant} passes the whole suite ({} scenarios)",
                statuses.len()
            ));
        }
    }
    assert!(
        report.is_empty(),
        "story:row-set-beside-named-rows-suite-tells-every-order-apart\n{}\nsynthesis refused \
         {refused:#?}",
        report.join("\n")
    );
}

/// A `forall` on the accepting branch beside named rows: a target refusing on any wire from the
/// source, or reading the predicate negated, fails the suite
/// (story:row-set-beside-named-rows-suite-tells-every-order-apart).
#[test]
fn adversary_a2_forall_beside_named_rows_kills_forall_mutants() {
    forall_mutants_killed("forall beside named rows", &forall_beside_named_rows());
}

/// The same model: synthesis refuses nothing, `duplicate` has its scenario, and every order of the
/// named-row refusals and the default the rule tells apart fails
/// (story:row-set-beside-named-rows-suite-tells-every-order-apart).
fn variants_forall_beside_named_rows() -> Vec<Variant> {
    vec![Variant {
        label: "forall beside named rows",
        text: forall_beside_named_rows(),
        rule: vec![NS, NK, SON, SNI, DUP],
        pairs: vec![(NS, NK), (NS, SNI), (SON, NK), (SON, SNI)],
        shape: PLAIN,
    }]
}

/// Every order of the rule, in the slow lane.
#[test]
#[ignore = "slow lane: tries every order of the rule"]
fn adversary_a2_forall_beside_named_rows_witnesses_every_branch() {
    every_order_told_apart(&variants_forall_beside_named_rows(), Sweep::Exhaustive);
}

/// The rule's own order and its adjacent swaps, in the test step.
#[test]
fn adversary_a2_forall_beside_named_rows_witnesses_every_branch_in_adjacent_swaps() {
    every_order_told_apart(&variants_forall_beside_named_rows(), Sweep::Adjacent);
}

/// The same `forall` mutants on the `ess/31` control, which has no named-row guard: the accepting
/// scenario has to hold a wire from the source to another sink too
/// (story:row-set-beside-named-rows-suite-tells-every-order-apart).
#[test]
fn adversary_a2_forall_without_named_rows_kills_forall_mutants_at_ess_31() {
    forall_mutants_killed(
        "forall without named rows at ess/31",
        &forall_without_named_rows_at_31(),
    );
}

/// The step tries the rule, its adjacent swaps (n orders) and the two non-adjacent transpositions
/// of the pairs the adjacent swaps leave in order; the slow lane tries all n!.
#[test]
fn the_adjacent_sweep_holds_n_plus_two_orders_and_the_exhaustive_sweep_n_factorial() {
    let rule = [NS, NK, SON, SNI, ST];
    let adjacent = orders(&rule, Sweep::Adjacent);
    assert_eq!(adjacent.len(), 7);
    assert_eq!(adjacent[0], rule.to_vec());
    for (at, (a, b)) in [(5, (NS, SNI)), (6, (ST, NK))] {
        let other = &adjacent[at];
        let place = |list: &[&str], name: &str| list.iter().position(|i| *i == name).unwrap();
        let before = |list: &[&str]| place(list, a) < place(list, b);
        assert!(
            before(other) != before(&rule),
            "{other:?} reverses {a}, {b}"
        );
        let moved = (0..rule.len()).filter(|i| other[*i] != rule[*i]).count();
        assert_eq!(moved, 2, "{other:?} is one transposition");
    }
    for other in &adjacent[1..5] {
        let moved: Vec<usize> = (0..rule.len())
            .filter(|at| other[*at] != rule[*at])
            .collect();
        assert_eq!(
            moved.len(),
            2,
            "{other:?} differs from the rule by one swap"
        );
        assert_eq!(moved[1], moved[0] + 1, "{other:?} swaps neighbours");
        assert_eq!(other[moved[0]], rule[moved[1]]);
        assert_eq!(other[moved[1]], rule[moved[0]]);
    }
    let exhaustive = orders(&rule, Sweep::Exhaustive);
    assert_eq!(exhaustive.len(), 120);
    let distinct: std::collections::BTreeSet<_> = exhaustive.iter().collect();
    assert_eq!(distinct.len(), 120);
}

/// The same model synthesizes the same suite twice in one process, for every variant here.
#[test]
fn adversary_a2_synthesis_is_deterministic_across_variants() {
    let texts = [
        REPRO.to_owned(),
        moved(REPRO, SOURCE_TAKEN, ABOVE_PREDICATES),
        edited(
            REPRO,
            "          exists: true\n",
            "          count: {gte: 2}\n",
        ),
        with_import(REPRO),
        with_layer(
            "      - name: lay-no-source
        when_related: {via: input.from_port, exists: false}
        error: wiring.board.SourceNotFound
",
            false,
        ),
    ];
    for text in &texts {
        let first = synthesize(&model(text)).suite.to_canonical_json().unwrap();
        let second = synthesize(&model(text)).suite.to_canonical_json().unwrap();
        assert_eq!(first, second);
    }
    let repro = synthesize(&model(REPRO)).suite.to_canonical_json().unwrap();
    assert_eq!(
        repro.len(),
        33908,
        "the pinned suite bytes of the reproduction"
    );
}
