//! Adversary and security pass 1, D368 unit U4
//! (story:row-set-beside-named-rows-suite-tells-every-order-apart): pass 2's method on shapes the
//! unit's own cases do not hold. A `when:` beside a named-row refusal that only answers where the
//! ports are the same, or on the source refusal; a `forall` by the sink, over two linked fields; an
//! `exists` on the accepting branch beside named rows; and a creating branch guarded by a row set
//! alone, which the unit lets the per-input pass send without `selects_branch`.
//!
//! The security cases hold a synthesized suite to three invariants:
//!
//! - every scenario it emits passes the interpreter of its own model, so no arrangement holds a
//!   row, or sends a request, that the specification refuses;
//! - synthesis on a crafted model ends within a bound;
//! - a target deciding the accepting `forall` another way fails the suite.

mod support_row_sets;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

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
const SOURCE_TAKEN: &str = "      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          exists: true
        error: wiring.board.SourceTaken
";
const SON_LINE: &str =
    "        when_related: {via: input.from_port, predicate: direction != output}\n";
const SNI_LINE: &str =
    "        when_related: {via: input.to_port, predicate: direction != input}\n";
const MAY: &str = "    may: [wiring.board.AddPort, wiring.board.Connect]\n";
const VIEWS: &str = "views:\n";

const NS: &str = "no-source";
const NK: &str = "no-sink";
const SON: &str = "source-not-output";
const SNI: &str = "sink-not-input";
const ST: &str = "source-taken";

fn edited(text: &str, before: &str, after: &str) -> String {
    let changed = text.replacen(before, after, 1);
    assert_ne!(changed, text, "`{before}` is in the model");
    changed
}

fn moved(text: &str, branch: &str, anchor: &str) -> String {
    let cut = edited(text, branch, "");
    edited(&cut, anchor, &format!("{branch}{anchor}"))
}

/// `line` with a `when:` over the two port inputs below it.
fn with_when(text: &str, line: &str, op: &str) -> String {
    edited(
        text,
        line,
        &format!("{line}        when: input.from_port {op} input.to_port\n"),
    )
}

/// The reproduction with an unguarded second creator of wires.
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

/// Which pair of inputs a `when:` admits.
#[derive(Clone, Copy, PartialEq, Eq)]
enum When {
    Always,
    Apart,
    Same,
}

impl When {
    fn admits(self, from: Option<&Node>, to: Option<&Node>) -> bool {
        match self {
            Self::Always => true,
            Self::Apart => from != to,
            Self::Same => from == to,
        }
    }
}

/// The `when:` each named refusal carries.
#[derive(Clone, Copy)]
struct Shape {
    nk: When,
    son: When,
    sni: When,
}

const PLAIN: Shape = Shape {
    nk: When::Always,
    son: When::Always,
    sni: When::Always,
};

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

fn holds(shape: Shape, world: &World, input: &BTreeMap<String, Node>) -> Vec<&'static str> {
    let from = input.get("from_port");
    let to = input.get("to_port");
    let mut held = Vec::new();
    match world.port(from) {
        None => held.push(NS),
        Some(direction) if direction != "output" && shape.son.admits(from, to) => held.push(SON),
        Some(_) => {}
    }
    match world.port(to) {
        None if shape.nk.admits(from, to) => held.push(NK),
        Some(direction) if direction != "input" && shape.sni.admits(from, to) => held.push(SNI),
        _ => {}
    }
    if world
        .wires
        .iter()
        .any(|(wire_from, _)| Some(wire_from) == from)
    {
        held.push(ST);
    }
    held
}

fn error_of(branch: &str) -> &'static str {
    match branch {
        NS => "wiring.board.SourceNotFound",
        NK => "wiring.board.SinkNotFound",
        SON => "wiring.board.SourceNotOutput",
        SNI => "wiring.board.SinkNotInput",
        _ => "wiring.board.SourceTaken",
    }
}

/// The interpreter, its answer to `Connect` rewritten to the first refusal of `order` that holds.
/// It records where the interpreter disagrees with [`holds`] under the rule's order, so a red
/// result is about the suite, not this model of the store.
struct Ordered<'v> {
    inner: Interpreted,
    shape: Shape,
    rule: &'v [&'static str],
    order: Vec<&'static str>,
    lays: &'v [(&'static str, &'static str)],
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

struct Variant {
    label: &'static str,
    text: String,
    rule: Vec<&'static str>,
    pairs: Vec<(&'static str, &'static str)>,
    shape: Shape,
}

const CONNECT_LAYS: &[(&str, &str)] = &[("wiring.board.Connect", "connected")];

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

fn equivalent(rule: &[&str], order: &[&str], pairs: &[(&str, &str)]) -> bool {
    let at = |list: &[&str], name: &str| list.iter().position(|item| *item == name);
    pairs.iter().all(
        |(a, b)| match (at(rule, a), at(rule, b), at(order, a), at(order, b)) {
            (Some(ra), Some(rb), Some(oa), Some(ob)) => (ra < rb) == (oa < ob),
            _ => true,
        },
    )
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
        .collect()
}

fn every_order_told_apart(variants: &[Variant]) {
    let mut report = Vec::new();
    for variant in variants {
        let synthesis = synthesize(&model(&variant.text));
        let refused = refusals(&synthesis);
        if !refused.is_empty() {
            report.push(format!("{}: synthesis refused {refused:?}", variant.label));
        }
        let healthy = run(&synthesis, &Interpreted::for_model(model(&variant.text)));
        let failing: Vec<&String> = healthy
            .iter()
            .filter(|(_, status)| **status != Status::Passed)
            .map(|(id, _)| id)
            .collect();
        if !failing.is_empty() {
            report.push(format!(
                "{}: the interpreter fails {failing:?}",
                variant.label
            ));
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
        for order in permutations(&variant.rule) {
            let target = Ordered {
                inner: Interpreted::for_model(model(&variant.text)),
                shape: variant.shape,
                rule: &variant.rule,
                order: order.clone(),
                lays: CONNECT_LAYS,
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
    assert!(
        report.is_empty(),
        "story:row-set-beside-named-rows-suite-tells-every-order-apart\n{}",
        report.join("\n")
    );
}

/// `sink-not-input` answers only where the two ports are one: `Connect(p, p)` on an output port
/// `p` a wire already leaves holds `sink-not-input` and `source-taken` together, and the rule
/// answers `sink-not-input`. A target reading the row set first answers that request otherwise.
#[test]
fn adversary_u4_when_same_port_beside_a_named_refusal_orders_are_told_apart() {
    let same = with_when(REPRO, SNI_LINE, "==");
    let pairs = vec![(NS, NK), (SON, NK), (ST, NK), (ST, SNI)];
    let shape = Shape {
        sni: When::Same,
        ..PLAIN
    };
    every_order_told_apart(&[
        Variant {
            label: "when same beside sink-not-input",
            text: same.clone(),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: pairs.clone(),
            shape,
        },
        Variant {
            label: "when same beside sink-not-input, row set declared first",
            text: moved(&same, SOURCE_TAKEN, NO_SOURCE),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs,
            shape,
        },
    ]);
}

/// `source-not-output` answers only where the ports differ: `Connect(p, p)` on an input port lays
/// a wire from an input port, so `source-not-output` and `source-taken` hold together on
/// `Connect(p, q)` and the rule answers `source-not-output`.
#[test]
fn adversary_u4_when_apart_beside_the_source_refusal_orders_are_told_apart() {
    let apart = with_when(REPRO, SON_LINE, "!=");
    let mut pairs = source_sink_pairs();
    pairs.push((SON, ST));
    let shape = Shape {
        son: When::Apart,
        ..PLAIN
    };
    every_order_told_apart(&[
        Variant {
            label: "when apart beside source-not-output",
            text: apart.clone(),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: pairs.clone(),
            shape,
        },
        Variant {
            label: "when apart beside source-not-output, row set declared first",
            text: moved(&apart, SOURCE_TAKEN, NO_SOURCE),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs,
            shape,
        },
    ]);
}

/// The scenario of `sink-not-input` carrying `when: input.from_port == input.to_port` passes the
/// interpreter: the send it holds names one port for both inputs, or the scenario is not emitted.
#[test]
fn adversary_u4_when_same_port_sink_scenario_passes_the_interpreter() {
    let text = with_when(REPRO, SNI_LINE, "==");
    let synthesis = synthesize(&model(&text));
    let id = "wiring.board.Connect/outcome/sink-not-input";
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("{error}"));
    let report = ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model(&text)))
        .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<String> = result
                .diagnostics()
                .take(2)
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            format!("{}: {diagnostics:#?}", result.scenario)
        })
        .collect();
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| {
            let text = format!("{scenario:?}");
            text.chars().take(4000).collect::<String>()
        });
    assert!(
        failing.is_empty(),
        "story:row-set-beside-named-rows-suite-tells-every-order-apart: the interpreter fails \
         {failing:#?}\nthe scenario: {scenario:?}"
    );
}

/// Two named-row refusals carrying a `when:` beside the row set.
#[test]
fn adversary_u4_two_when_refusals_beside_the_row_set_orders_are_told_apart() {
    let both = with_when(&with_when(REPRO, SON_LINE, "!="), SNI_LINE, "!=");
    let mut pairs = source_sink_pairs();
    pairs.push((SON, ST));
    let shape = Shape {
        son: When::Apart,
        sni: When::Apart,
        ..PLAIN
    };
    every_order_told_apart(&[
        Variant {
            label: "two when refusals",
            text: both.clone(),
            rule: vec![NS, NK, SON, SNI, ST],
            pairs: pairs.clone(),
            shape,
        },
        Variant {
            label: "two when refusals, row set declared first",
            text: moved(&both, SOURCE_TAKEN, NO_SOURCE),
            rule: vec![NS, NK, ST, SON, SNI],
            pairs,
            shape,
        },
    ]);
}

/// The reproduction's row-set refusal replaced by `guard` on the accepting branch beside the four
/// named-row refusals, and a default refusal `duplicate` below it.
fn guarded_beside_named_rows(guard: &str) -> String {
    let text = edited(REPRO, SOURCE_TAKEN, "");
    let text = edited(
        &text,
        "      - name: connected\n        creates: wiring.board.Wire\n",
        &format!("      - name: connected\n{guard}        creates: wiring.board.Wire\n"),
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

fn row_guard(filter: &str, test: &str) -> String {
    format!(
        "        when_related:
          entity: wiring.board.Wire
          where: {filter}
          {test}
"
    )
}

/// The suite of `text` refuses nothing, witnesses `connected` and `duplicate`, passes its
/// interpreter, and fails the interpreter of each mutant (`text` with `row_test` replaced). Each
/// mutant answers some reachable request differently.
fn mutants_killed(
    label: &str,
    text: &str,
    row_test: &str,
    mutants: &[(&str, &str)],
) -> Vec<String> {
    let synthesis = synthesize(&model(text));
    let refused = refusals(&synthesis);
    let mut report = Vec::new();
    if !refused.is_empty() {
        report.push(format!("{label}: synthesis refused {refused:?}"));
    }
    for branch in ["connected", "duplicate"] {
        let id = format!("wiring.board.Connect/outcome/{branch}");
        if !synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id)
        {
            report.push(format!("{label}: no scenario `{id}`"));
        }
    }
    let healthy = run(&synthesis, &Interpreted::for_model(model(text)));
    let failed: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    if !failed.is_empty() {
        report.push(format!("{label}: the interpreter fails {failed:?}"));
    }
    for (mutant, line) in mutants {
        let changed = edited(
            text,
            &format!("          {row_test}\n"),
            &format!("          {line}\n"),
        );
        let statuses = run(&synthesis, &Interpreted::for_model(model(&changed)));
        if statuses.values().all(|status| *status == Status::Passed) {
            report.push(format!(
                "{label}: a target {mutant} passes the whole suite ({} scenarios)",
                statuses.len()
            ));
        }
    }
    report
}

fn assert_clean(report: &[String]) {
    assert!(
        report.is_empty(),
        "story:row-set-beside-named-rows-suite-tells-every-order-apart\n{}",
        report.join("\n")
    );
}

/// `forall` by the sink: the wires into the sink, none from the source.
#[test]
fn adversary_u4_forall_by_the_sink_beside_named_rows_kills_mutants() {
    let row_test = "forall: from_port != input.from_port";
    let text = guarded_beside_named_rows(&row_guard("to_port == input.to_port", row_test));
    assert_clean(&mutants_killed(
        "forall by the sink",
        &text,
        row_test,
        &[
            ("refusing on any wire into the sink", "count: {eq: 0}"),
            (
                "reading the predicate negated",
                "forall: from_port == input.from_port",
            ),
        ],
    ));
}

/// `forall` comparing both linked fields with the input: no wire from the source is the pair.
#[test]
fn adversary_u4_forall_over_two_linked_fields_beside_named_rows_kills_mutants() {
    let row_test = "forall: {any: [from_port != input.from_port, to_port != input.to_port]}";
    let text = guarded_beside_named_rows(&row_guard("from_port == input.from_port", row_test));
    assert_clean(&mutants_killed(
        "forall over two linked fields",
        &text,
        row_test,
        &[
            ("refusing on any wire from the source", "count: {eq: 0}"),
            (
                "reading the predicate negated",
                "forall: {all: [from_port == input.from_port, to_port == input.to_port]}",
            ),
        ],
    ));
}

/// `exists: false` on the accepting branch beside named rows: a wire from the source refuses.
#[test]
fn adversary_u4_exists_on_the_accepting_branch_beside_named_rows_kills_mutants() {
    let row_test = "exists: false";
    let text = guarded_beside_named_rows(&row_guard("from_port == input.from_port", row_test));
    assert_clean(&mutants_killed(
        "exists false on connected",
        &text,
        row_test,
        &[("reading the test negated", "exists: true")],
    ));
}

/// The `forall` model with a refusal over the input alone declared above `connected`: the
/// per-input pass now sends `connected` as the creator of the wires its own scenarios hold
/// without `selects_branch`, which is what reads the input refusal. Every emitted scenario passes
/// the interpreter, in both polarities of the refusal.
#[test]
fn adversary_u4_creator_beside_an_input_refusal_passes_the_interpreter() {
    let base = guarded_beside_named_rows(&row_guard(
        "from_port == input.from_port",
        "forall: to_port != input.to_port",
    ));
    let mut report = Vec::new();
    for value in ["true", "false"] {
        let text = edited(
            &base,
            "      - {name: to_port, type: wiring.board.PortId}\n    outcomes:\n      - name: no-source\n",
            "      - {name: to_port, type: wiring.board.PortId}\n      - {name: hold, type: Boolean}\n    outcomes:\n      - name: no-source\n",
        );
        // An input refusal answers at step 1, so it is declared above every named-row refusal.
        let text = edited(
            &text,
            "    outcomes:\n      - name: no-source\n",
            &format!(
                "    outcomes:\n      - name: held\n        when: input.hold == {value}\n        error: wiring.board.SourceTaken\n      - name: no-source\n"
            ),
        );
        report.extend(mutants_killed(
            if value == "true" {
                "held on true"
            } else {
                "held on false"
            },
            &text,
            "forall: to_port != input.to_port",
            &[
                ("refusing on any wire from the source", "count: {eq: 0}"),
                (
                    "reading the predicate negated",
                    "forall: to_port == input.to_port",
                ),
            ],
        ));
    }
    assert_clean(&report);
}

/// A creating branch guarded by a row set alone that only accepts where a wire already leaves the
/// source, and no other creator of wires: `connected` is unreachable. Synthesis may refuse it,
/// but every scenario it emits passes the interpreter, and none claims `connected`.
#[test]
fn adversary_u4_creator_whose_row_set_never_selects_emits_no_failing_scenario() {
    let text =
        guarded_beside_named_rows(&row_guard("from_port == input.from_port", "exists: true"));
    let synthesis = synthesize(&model(&text));
    let healthy = run(&synthesis, &Interpreted::for_model(model(&text)));
    let failed: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    let claims = synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == "wiring.board.Connect/outcome/connected");
    assert!(
        failed.is_empty() && !claims,
        "story:row-set-beside-named-rows-suite-tells-every-order-apart: the interpreter fails \
         {failed:?}; a `connected` scenario emitted: {claims}; refused {:#?}",
        refusals(&synthesis)
    );
}

/// The same creator reachable through an unguarded import: `connected` accepts only beside a
/// wire from the source; its scenario passes the interpreter and fails the negated test.
#[test]
fn adversary_u4_creator_needing_a_selected_row_beside_an_import_kills_mutants() {
    let row_test = "exists: true";
    let text = with_import(&guarded_beside_named_rows(&row_guard(
        "from_port == input.from_port",
        row_test,
    )));
    assert_clean(&mutants_killed(
        "exists true on connected beside an import",
        &text,
        row_test,
        &[("reading the test negated", "exists: false")],
    ));
}

/// Every new variant synthesizes the same suite twice.
#[test]
fn adversary_u4_synthesis_is_deterministic_on_the_new_shapes() {
    let texts = [
        with_when(REPRO, SNI_LINE, "=="),
        with_when(REPRO, SON_LINE, "!="),
        guarded_beside_named_rows(&row_guard(
            "from_port == input.from_port",
            "forall: {any: [from_port != input.from_port, to_port != input.to_port]}",
        )),
    ];
    for text in &texts {
        let first = synthesize(&model(text)).suite.to_canonical_json().unwrap();
        let second = synthesize(&model(text)).suite.to_canonical_json().unwrap();
        assert_eq!(first, second);
    }
}

// ---- the security pass ----

const CONNECTED: &str = "      - name: connected
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
        sets: {from_port: input.from_port, to_port: input.to_port}
";

/// The bound one synthesis of a model here has to end within (a debug build).
const BOUND: Duration = Duration::from_secs(120);

fn run_text(synthesis: &Synthesis, text: &str) -> BTreeMap<String, Status> {
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("{error}"));
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// What one synthesis of `text` emitted, refused, took, and which of its scenarios its own
/// interpreter fails.
struct Probe {
    synthesis: Synthesis,
    elapsed: Duration,
    failed: Vec<String>,
}

fn probe(text: &str) -> Probe {
    let ir = model(text);
    let start = Instant::now();
    let synthesis = synthesize(&ir);
    let elapsed = start.elapsed();
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("{error}"));
    let failed = ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model(text)))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let steps = synthesis
                .suite
                .scenarios
                .get(&result.scenario)
                .map(|scenario| format!("{:?}", scenario.steps))
                .unwrap_or_default();
            let why: Vec<String> = result
                .diagnostics()
                .map(|d| {
                    format!(
                        "{:?} expected {:?} observed {:?}",
                        d.input, d.expected, d.observed
                    )
                })
                .collect();
            format!(
                "{}: {:?} {why:?}\n  steps {steps}",
                result.scenario, result.status
            )
        })
        .collect();
    Probe {
        synthesis,
        elapsed,
        failed,
    }
}

/// Every model in `variants` synthesizes within [`BOUND`] and its suite passes its interpreter.
fn sound(variants: &[(&str, String)]) {
    let mut report = Vec::new();
    for (label, text) in variants {
        let probe = probe(text);
        eprintln!(
            "{label}: {} scenarios, {} refusals, {:?}",
            probe.synthesis.suite.scenarios.len(),
            probe.synthesis.refusals.len(),
            probe.elapsed
        );
        if probe.elapsed > BOUND {
            report.push(format!("{label}: synthesis took {:?}", probe.elapsed));
        }
        if !probe.failed.is_empty() {
            report.push(format!(
                "{label}: the interpreter fails {:?}; synthesis refused {:?}",
                probe.failed,
                refusals(&probe.synthesis)
            ));
        }
    }
    assert!(
        report.is_empty(),
        "story:row-set-beside-named-rows-suite-tells-every-order-apart\n{}",
        report.join("\n")
    );
}

/// The reproduction where wires are laid by `Lay`, whose branches are `lay` (a missing-source
/// refusal first, then whatever the case adds), and `Connect` records a patch instead: the row
/// set's rows have a creator of their own. `Lay` also takes an `armed` flag; `armed` is a refusal
/// over it alone, declared first as the precedence order reads it.
fn with_layer(armed: &str, lay: &str) -> String {
    let mut text = edited(
        REPRO,
        "  - {name: wiring.board.WireId, kind: newtype, of: Uuid}\n",
        "  - {name: wiring.board.WireId, kind: newtype, of: Uuid}
  - {name: wiring.board.PatchId, kind: newtype, of: Uuid}
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
errors:
  - {name: wiring.board.NotArmed, summary: The layer is not armed., fields: []}
  - {name: wiring.board.SinkBusy, summary: A wire already enters the sink port., fields: []}
",
    );
    text = edited(
        &text,
        "  - name: wiring.board.Connected\n",
        "  - name: wiring.board.Patched
    fields: [{name: patch_id, type: wiring.board.PatchId}]
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
    let commands = format!(
        "  - name: wiring.board.Lay
    input:
      - {{name: from_port, type: wiring.board.PortId}}
      - {{name: to_port, type: wiring.board.PortId}}
      - {{name: armed, type: Boolean}}
    outcomes:
{armed}      - name: lay-no-source
        when_related: {{via: input.from_port, exists: false}}
        error: wiring.board.SourceNotFound
{lay}"
    );
    text = edited(&text, VIEWS, &format!("{commands}{VIEWS}"));
    edited(
        &text,
        MAY,
        "    may: [wiring.board.AddPort, wiring.board.Connect, wiring.board.Lay]\n",
    )
}

/// `Lay`'s creating branch, guarded by `guard` (indented as a branch key) and then followed by a
/// default refusal, or by nothing.
fn laid(guard: &str) -> String {
    let default = if guard.is_empty() {
        ""
    } else {
        "      - name: lay-refused
        error: wiring.board.SinkBusy
"
    };
    format!(
        "      - name: laid
{guard}        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {{wiring.board.Connected: {{wire_id: {{generated: true}}}}}}
        sets: {{from_port: input.from_port, to_port: input.to_port}}
{default}"
    )
}

const NOT_ARMED_IF_FALSE: &str = "      - name: not-armed
        when: input.armed == false
        error: wiring.board.NotArmed
";
const NOT_ARMED_IF_TRUE: &str = "      - name: not-armed
        when: input.armed == true
        error: wiring.board.NotArmed
";
const ONE_PER_SINK: &str = "        when_related:
          entity: wiring.board.Wire
          where: to_port == input.to_port
          exists: false
";
const ONE_PER_SOURCE: &str = "        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          count: {eq: 0}
";
const ONLY_AFTER_A_WIRE: &str = "        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          exists: true
";

/// The per-input pass now sends a creator whose creating branch a row set alone guards without
/// deciding its input first (`synthesize.rs` `arrange_toward_with`, `by_rows`). An input-only
/// refusal declared above that branch still has to be refuted by the input the arrangement sends:
/// a `Lay` sent `armed` the wrong way is refused, and a scenario built on it fails its own
/// interpreter. Both polarities, so one of them is the candidate the creator is sent first.
#[test]
fn security_u4_row_set_creator_beside_an_input_refusal_is_sent_an_input_it_admits() {
    sound(&[
        (
            "one wire per sink, refused unless armed",
            with_layer(NOT_ARMED_IF_FALSE, &laid(ONE_PER_SINK)),
        ),
        (
            "one wire per sink, refused when armed",
            with_layer(NOT_ARMED_IF_TRUE, &laid(ONE_PER_SINK)),
        ),
        (
            "one wire per source, refused unless armed",
            with_layer(NOT_ARMED_IF_FALSE, &laid(ONE_PER_SOURCE)),
        ),
        (
            "one wire per source, refused when armed",
            with_layer(NOT_ARMED_IF_TRUE, &laid(ONE_PER_SOURCE)),
        ),
    ]);
}

/// A row-set creator that can lay at most one wire from a source, beside a command that refuses
/// on two: no arrangement holds two, so synthesis ends, and whatever it emits passes. And a creator
/// that lays a wire only after one already leaves the source, which nothing else creates: its
/// branch is never selected from an empty store, so no scenario may hold a wire it laid.
#[test]
fn security_u4_row_set_creator_that_cannot_lay_the_rows_is_never_arranged() {
    let two = edited(
        &with_layer("", &laid(ONE_PER_SOURCE)),
        "          exists: true\n        error: wiring.board.SourceTaken\n",
        "          count: {gte: 2}\n        error: wiring.board.SourceTaken\n",
    );
    sound(&[
        ("one wire per source, two refused", two),
        (
            "a wire only after a wire",
            with_layer("", &laid(ONLY_AFTER_A_WIRE)),
        ),
        (
            "a wire only after a wire, refused unless armed",
            with_layer(NOT_ARMED_IF_FALSE, &laid(ONLY_AFTER_A_WIRE)),
        ),
    ]);
}

/// `present_on` now passes over a creator's named-row branch that carries a `when:`. The creator
/// of the row set's rows refuses an input-facing source only when armed, and an output-facing sink
/// only when the ports differ: every scenario holding a wire it laid passes its interpreter.
#[test]
fn security_u4_creator_named_refusal_with_when_is_passed_over_soundly() {
    let guards = "      - name: lay-no-sink
        when_related: {via: input.to_port, exists: false}
        error: wiring.board.SinkNotFound
      - name: lay-source-not-output
        when_related: {via: input.from_port, predicate: direction != output}
        when: input.armed == true
        error: wiring.board.SourceNotOutput
      - name: lay-sink-not-input
        when_related: {via: input.to_port, predicate: direction != input}
        when: input.from_port != input.to_port
        error: wiring.board.SinkNotInput
";
    let reversed = guards.replace("input.armed == true", "input.armed == false");
    sound(&[
        (
            "creator refusals carrying when:",
            with_layer("", &format!("{guards}{}", laid(""))),
        ),
        (
            "creator refusals carrying when:, flag reversed",
            with_layer("", &format!("{reversed}{}", laid(""))),
        ),
        (
            "creator refusals carrying when:, a row-set creator",
            with_layer("", &format!("{guards}{}", laid(ONE_PER_SINK))),
        ),
    ]);
}

/// `named_refusal` now admits a named-row refusal carrying a `when:` over an input that names no
/// row: `sink-not-input` answers only where the request is strict. The rows arranged for its
/// witness, and for every branch below it, have to send the flag that makes the refusal answer or
/// pass, as the scenario expects.
#[test]
fn security_u4_named_refusal_with_when_over_a_plain_input_is_witnessed_soundly() {
    let strict = edited(
        REPRO,
        "      - {name: to_port, type: wiring.board.PortId}\n    outcomes:\n      - name: no-source\n",
        "      - {name: to_port, type: wiring.board.PortId}
      - {name: strict, type: Boolean}
    outcomes:
      - name: no-source\n",
    );
    let on = edited(
        &strict,
        "        when_related: {via: input.to_port, predicate: direction != input}\n",
        "        when_related: {via: input.to_port, predicate: direction != input}
        when: input.strict == true
",
    );
    let off = on.replace("input.strict == true", "input.strict == false");
    let both = edited(
        &on,
        "        when_related: {via: input.from_port, predicate: direction != output}\n",
        "        when_related: {via: input.from_port, predicate: direction != output}
        when: input.strict == false
",
    );
    let first = edited(&on, SOURCE_TAKEN, "");
    let first = edited(
        &first,
        "      - name: no-source\n",
        &format!("{SOURCE_TAKEN}      - name: no-source\n"),
    );
    sound(&[
        ("strict sink refusal", on),
        ("lenient sink refusal", off),
        ("strict sink, lenient source", both),
        ("strict sink, row set declared first", first),
    ]);
}

/// The model the four-named-rows security case synthesizes: a command naming four rows beside two
/// row sets.
const FOUR_ROWS: &str = "format: ess/32
system: wiring
version: v1
domain: wiring.board
summary: A bridge joins two output ports to two input ports.
types:
  - {name: wiring.board.PortId, kind: newtype, of: Uuid}
  - {name: wiring.board.WireId, kind: newtype, of: Uuid}
  - {name: wiring.board.Direction, kind: enum, variants: [input, output]}
entities:
  - name: wiring.board.Port
    identity: {name: port_id, type: wiring.board.PortId}
    fields:
      - {name: direction, type: wiring.board.Direction}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: wiring.board.Wire
    identity: {name: wire_id, type: wiring.board.WireId}
    fields:
      - {name: a, type: wiring.board.PortId}
      - {name: b, type: wiring.board.PortId}
      - {name: c, type: wiring.board.PortId}
      - {name: d, type: wiring.board.PortId}
    relations:
      - {name: pa, kind: references, target: wiring.board.Port, cardinality: one, via: a}
      - {name: pb, kind: references, target: wiring.board.Port, cardinality: one, via: b}
      - {name: pc, kind: references, target: wiring.board.Port, cardinality: one, via: c}
      - {name: pd, kind: references, target: wiring.board.Port, cardinality: one, via: d}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
errors:
  - {name: wiring.board.Missing, summary: No port carries the identity., fields: []}
  - {name: wiring.board.Facing, summary: A port faces the wrong way., fields: []}
  - {name: wiring.board.Taken, summary: A wire already uses the port., fields: []}
  - {name: wiring.board.Duplicate, summary: The wire already exists., fields: []}
events:
  - name: wiring.board.PortAdded
    fields: [{name: port_id, type: wiring.board.PortId}]
  - name: wiring.board.Connected
    fields: [{name: wire_id, type: wiring.board.WireId}]
actors:
  - name: wiring.board.Operator
    may: [wiring.board.AddPort, wiring.board.Connect]
commands:
  - name: wiring.board.AddPort
    input:
      - {name: direction, type: wiring.board.Direction}
    outcomes:
      - name: added
        creates: wiring.board.Port
        instance: port_id
        emits: [wiring.board.PortAdded]
        payload: {wiring.board.PortAdded: {port_id: {generated: true}}}
        sets: {direction: input.direction}
  - name: wiring.board.Connect
    input:
      - {name: a, type: wiring.board.PortId}
      - {name: b, type: wiring.board.PortId}
      - {name: c, type: wiring.board.PortId}
      - {name: d, type: wiring.board.PortId}
    outcomes:
      - name: no-a
        when_related: {via: input.a, exists: false}
        error: wiring.board.Missing
      - name: no-b
        when_related: {via: input.b, exists: false}
        error: wiring.board.Missing
      - name: no-c
        when_related: {via: input.c, exists: false}
        error: wiring.board.Missing
      - name: no-d
        when_related: {via: input.d, exists: false}
        error: wiring.board.Missing
      - name: a-facing
        when_related: {via: input.a, predicate: direction != output}
        when: input.a != input.c
        error: wiring.board.Facing
      - name: b-facing
        when_related: {via: input.b, predicate: direction != output}
        when: input.b != input.d
        error: wiring.board.Facing
      - name: c-facing
        when_related: {via: input.c, predicate: direction != input}
        when: input.a != input.c
        error: wiring.board.Facing
      - name: d-facing
        when_related: {via: input.d, predicate: direction != input}
        when: input.b != input.d
        error: wiring.board.Facing
      - name: a-taken
        when_related:
          entity: wiring.board.Wire
          where: a == input.a
          exists: true
        error: wiring.board.Taken
      - name: d-taken
        when_related:
          entity: wiring.board.Wire
          where: d == input.d
          count: {gte: 2}
        error: wiring.board.Taken
      - name: connected
        when_related:
          entity: wiring.board.Wire
          where: b == input.b
          forall: {all: [c != input.c, d != input.d]}
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
        sets: {a: input.a, b: input.b, c: input.c, d: input.d}
      - name: duplicate
        error: wiring.board.Duplicate
views:
  - name: wiring.board.Wires
    source: wiring.board.Wire
    consistency: read_your_writes
    fields:
      - {name: wire_id, type: wiring.board.WireId}
      - {name: a, type: wiring.board.PortId}
";

/// The new last attempt (`Arranging::SEND_ROWS`) and the linked retry of an arrangement standing on
/// no selected row, on a command naming four rows, each predicate refusal carrying a `when:`, two
/// row sets and an accepting `forall` over a linked field beside a default: synthesis ends within
/// the bound, and every scenario passes.
#[test]
fn security_u4_four_named_rows_with_when_two_row_sets_and_a_forall_end_within_the_bound() {
    let text = FOUR_ROWS;
    let filter_ne = text.replace("where: b == input.b", "where: b != input.b");
    sound(&[
        ("four named rows", text.to_owned()),
        ("four named rows, forall selector !=", filter_ne),
    ]);
}

const FORALL: &str = "        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          forall: to_port == input.to_port
";
const FORALL_LINE: &str = "          forall: to_port == input.to_port\n";

/// The reproduction with its row-set refusal replaced by an accepting `forall: to_port ==
/// input.to_port` beside the four named refusals: a source may fan out to one sink only, and the
/// default answers a wire from the source to another sink. A target refusing on any wire from the
/// source, or reading the predicate negated, answers a reachable request differently (a wire from
/// the source into this same sink), so the suite has to fail it, and its interpreter passes.
#[test]
fn security_u4_forall_equal_to_the_send_beside_named_rows_fails_every_wrong_target() {
    let text = edited(REPRO, SOURCE_TAKEN, "");
    let text = edited(
        &text,
        "      - name: connected\n        creates: wiring.board.Wire\n",
        &format!("      - name: connected\n{FORALL}        creates: wiring.board.Wire\n"),
    );
    let text = edited(
        &text,
        "        sets: {from_port: input.from_port, to_port: input.to_port}\nviews:\n",
        "        sets: {from_port: input.from_port, to_port: input.to_port}
      - name: fanned-out
        error: wiring.board.SourceTaken
views:
",
    );
    let probe = probe(&text);
    let mut report = Vec::new();
    if !probe.failed.is_empty() {
        report.push(format!("the interpreter fails {:?}", probe.failed));
    }
    if probe.elapsed > BOUND {
        report.push(format!("synthesis took {:?}", probe.elapsed));
    }
    for (mutant, line) in [
        (
            "refusing on any wire from the source",
            "          count: {eq: 0}\n",
        ),
        (
            "reading the predicate negated",
            "          forall: to_port != input.to_port\n",
        ),
    ] {
        let changed = edited(&text, FORALL_LINE, line);
        let statuses = run_text(&probe.synthesis, &changed);
        if statuses.values().all(|status| *status == Status::Passed) {
            report.push(format!(
                "a target {mutant} passes the whole suite ({} scenarios)",
                statuses.len()
            ));
        }
    }
    assert!(
        report.is_empty(),
        "story:row-set-beside-named-rows-suite-tells-every-order-apart\n{}\nsynthesis refused \
         {:#?}",
        report.join("\n"),
        refusals(&probe.synthesis)
    );
}
