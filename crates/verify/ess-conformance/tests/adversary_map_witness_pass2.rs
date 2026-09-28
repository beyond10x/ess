//! Adversary, pass 2, against the Map input witness (beyond10x/ess#196) and correction 1.
//!
//! Correction 1 unfolds a type that refers to itself through a map or a copied list once, gives a
//! copied list one element, and lets a count boundary shrink a map. These cases drive each claim
//! through synthesis and the Rust runner: mutual recursion, recursion through a copied list and
//! through a newtype over a map, a copied map or list whose element type carries an invariant the
//! plain witness does not meet, and the accepting boundary of an upper count bound.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::consistency::ConsistencyToken;
use ess_primitives::node::Node;

type Fields = BTreeMap<String, Node>;
type Verdicts = BTreeMap<String, String>;

// ---- models ----------------------------------------------------------------------------------------

/// Four shapes of recursion, each copied whole into an event payload by its own command.
const RECURSION: &str = "format: ess/17
system: demo
version: v1
domain: demo.m
types:
  - name: demo.m.Left
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: rights, type: 'Map<String, demo.m.Right>'}
  - name: demo.m.Right
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: lefts, type: 'Map<String, demo.m.Left>'}
  - name: demo.m.Branch
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: twigs, type: 'List<demo.m.Branch>'}
  - name: demo.m.Kids
    kind: newtype
    of: 'Map<String, demo.m.Node>'
  - name: demo.m.Node
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: kids, type: demo.m.Kids}
  - name: demo.m.Mixed
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: next, type: 'Map<String, List<demo.m.Mixed>>'}
events:
  - name: demo.m.LeftSent
    fields:
      - {name: left, type: demo.m.Left}
  - name: demo.m.BranchSent
    fields:
      - {name: branch, type: demo.m.Branch}
  - name: demo.m.NodeSent
    fields:
      - {name: node, type: demo.m.Node}
  - name: demo.m.MixedSent
    fields:
      - {name: mixed, type: demo.m.Mixed}
actors:
  - {name: demo.m.Clerk, may: [demo.m.SendLeft, demo.m.SendBranch, demo.m.SendNode, demo.m.SendMixed]}
commands:
  - name: demo.m.SendLeft
    input:
      - {name: left, type: demo.m.Left}
    outcomes:
      - name: sent
        emits: [demo.m.LeftSent]
        payload:
          demo.m.LeftSent: {left: input.left}
  - name: demo.m.SendBranch
    input:
      - {name: branch, type: demo.m.Branch}
    outcomes:
      - name: sent
        emits: [demo.m.BranchSent]
        payload:
          demo.m.BranchSent: {branch: input.branch}
  - name: demo.m.SendNode
    input:
      - {name: node, type: demo.m.Node}
    outcomes:
      - name: sent
        emits: [demo.m.NodeSent]
        payload:
          demo.m.NodeSent: {node: input.node}
  - name: demo.m.SendMixed
    input:
      - {name: mixed, type: demo.m.Mixed}
    outcomes:
      - name: sent
        emits: [demo.m.MixedSent]
        payload:
          demo.m.MixedSent: {mixed: input.mixed}
";

/// A recursive list stored on an entity and read back through a view, so the flattener builds the
/// row expectation from the unfolded value.
const STORED: &str = "format: ess/17
system: demo
version: v1
domain: demo.s
types:
  - name: demo.s.Branch
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: twigs, type: 'List<demo.s.Branch>'}
entities:
  - name: demo.s.Garden
    identity: {name: garden_id, type: String}
    fields:
      - {name: branch, type: demo.s.Branch}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.s.Sown
    fields:
      - {name: garden_id, type: String}
actors:
  - {name: demo.s.Clerk, may: [demo.s.Sow]}
commands:
  - name: demo.s.Sow
    input:
      - {name: garden_id, type: String}
      - {name: branch, type: demo.s.Branch}
    outcomes:
      - name: sown
        creates: demo.s.Garden
        instance: garden_id
        emits: [demo.s.Sown]
        payload:
          demo.s.Sown: {garden_id: input.garden_id}
        sets:
          branch: input.branch
views:
  - name: demo.s.Gardens
    source: demo.s.Garden
    consistency: read_your_writes
    fields:
      - {name: garden_id, type: String}
      - {name: branch, type: demo.s.Branch}
";

/// A copied map and a copied list whose element type has an invariant the plain witness breaks:
/// `1` is not `>= 10`, and the text `qty…`/`codes.0` does not start with `SKU-`.
const INVARIANTS: &str = "format: ess/17
system: demo
version: v1
domain: demo.i
types:
  - name: demo.i.Qty
    kind: newtype
    of: Integer
    invariants: ['value >= 10']
  - name: demo.i.Code
    kind: newtype
    of: String
    invariants:
      - value: {starts_with: \"SKU-\"}
events:
  - name: demo.i.QtysSent
    fields:
      - {name: qtys, type: 'Map<String, demo.i.Qty>'}
  - name: demo.i.CodesSent
    fields:
      - {name: codes, type: 'List<demo.i.Code>'}
actors:
  - {name: demo.i.Clerk, may: [demo.i.SendQtys, demo.i.SendCodes]}
commands:
  - name: demo.i.SendQtys
    input:
      - {name: qtys, type: 'Map<String, demo.i.Qty>'}
    outcomes:
      - name: sent
        emits: [demo.i.QtysSent]
        payload:
          demo.i.QtysSent: {qtys: input.qtys}
  - name: demo.i.SendCodes
    input:
      - {name: codes, type: 'List<demo.i.Code>'}
    outcomes:
      - name: sent
        emits: [demo.i.CodesSent]
        payload:
          demo.i.CodesSent: {codes: input.codes}
";

/// An upper count bound on a map, `<= 2`, and its list twin.
const UPPER: &str = "format: ess/17
system: demo
version: v1
domain: demo.u
events:
  - name: demo.u.Counted
    fields:
      - {name: labels, type: 'Map<String, String>'}
  - name: demo.u.Listed
    fields:
      - {name: items, type: 'List<String>'}
actors:
  - {name: demo.u.Clerk, may: [demo.u.MapFew, demo.u.ListFew]}
commands:
  - name: demo.u.MapFew
    input:
      - {name: labels, type: 'Map<String, String>'}
    outcomes:
      - name: few
        when: labels.count <= 2
        emits: [demo.u.Counted]
        payload:
          demo.u.Counted: {labels: input.labels}
      - name: many
        emits: [demo.u.Counted]
        payload:
          demo.u.Counted: {labels: input.labels}
  - name: demo.u.ListFew
    input:
      - {name: items, type: 'List<String>'}
    outcomes:
      - name: few
        when: items.count <= 2
        emits: [demo.u.Listed]
        payload:
          demo.u.Listed: {items: input.items}
      - name: many
        emits: [demo.u.Listed]
        payload:
          demo.u.Listed: {items: input.items}
";

// ---- helpers ---------------------------------------------------------------------------------------

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("maps.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The suite, and every refusal synthesis reported, one line each.
fn synthesized(text: &str) -> (ConformanceSuite, Vec<String>) {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    let refusals = synthesis.refusals.iter().map(ToString::to_string).collect();
    (synthesis.suite, refusals)
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// Every literal input any scenario of the suite sends `command`.
fn all_sent(suite: &ConformanceSuite, command: &str) -> Vec<(String, Fields)> {
    let mut out = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand {
                command: c, input, ..
            } = step
            {
                if c.to_string() == command {
                    out.push((
                        id.to_string(),
                        input
                            .iter()
                            .filter_map(|(field, value)| match value {
                                ScenarioValue::Literal { value } => {
                                    Some((field.clone(), value.clone()))
                                }
                                _ => None,
                            })
                            .collect(),
                    ));
                }
            }
        }
    }
    out
}

fn sent_in(scenario: &ConformanceScenario, command: &str) -> Option<Fields> {
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
}

fn size(value: Option<&Node>) -> usize {
    match value {
        Some(Node::Map(entries)) => entries.len(),
        Some(Node::Seq(items)) => items.len(),
        _ => 0,
    }
}

fn field<'a>(value: &'a Node, name: &str) -> Option<&'a Node> {
    value.as_map().and_then(|members| members.get(name))
}

fn first(value: Option<&Node>) -> Option<&Node> {
    match value? {
        Node::Map(entries) => entries.values().next(),
        Node::Seq(items) => items.first(),
        _ => None,
    }
}

fn empty_like(value: &Node) -> Node {
    match value {
        Node::Seq(_) => Node::Seq(Vec::new()),
        _ => Node::Map(BTreeMap::new()),
    }
}

/// `value` with its member `name` emptied.
fn emptied(value: &Node, name: &str) -> Node {
    match value {
        Node::Map(members) => {
            let mut members = members.clone();
            if let Some(held) = members.get(name).cloned() {
                members.insert(name.to_owned(), empty_like(&held));
            }
            Node::Map(members)
        }
        other => other.clone(),
    }
}

/// `value` with the member `name` of its first element or entry emptied: the level the unfolding
/// adds, which a target that copies only one level deep drops.
fn emptied_below(value: &Node, outer: &str, inner: &str) -> Node {
    let Some(held) = field(value, outer) else {
        return value.clone();
    };
    let changed = match held {
        Node::Map(entries) => Node::Map(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), emptied(v, inner)))
                .collect(),
        ),
        Node::Seq(items) => Node::Seq(items.iter().map(|v| emptied(v, inner)).collect()),
        other => other.clone(),
    };
    let mut members = value.as_map().cloned().unwrap_or_default();
    members.insert(outer.to_owned(), changed);
    Node::Map(members)
}

// ---- one Rust target -------------------------------------------------------------------------------

type Handler = fn(&str, &str, &Fields, &mut Vec<Fields>) -> SemanticCommandResult;

struct Store {
    mode: &'static str,
    handler: Handler,
    rows: RefCell<Vec<Fields>>,
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
        Ok(ImplementationIdentity::new("map-witness-adversary-2", "1"))
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
        Err(TargetError::unsupported("external", "none declared"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
}

fn get(input: &Fields, name: &str) -> Node {
    input.get(name).cloned().unwrap_or(Node::Null)
}

fn recursion(
    mode: &str,
    command: &str,
    input: &Fields,
    _: &mut Vec<Fields>,
) -> SemanticCommandResult {
    let (name, event, outer, inner) = match command {
        "demo.m.SendLeft" => ("left", "demo.m.LeftSent", "rights", "lefts"),
        "demo.m.SendBranch" => ("branch", "demo.m.BranchSent", "twigs", "twigs"),
        "demo.m.SendNode" => ("node", "demo.m.NodeSent", "kids", "kids"),
        "demo.m.SendMixed" => ("mixed", "demo.m.MixedSent", "next", "next"),
        other => panic!("unexpected command {other}"),
    };
    let value = get(input, name);
    let value = match mode {
        "drop-top" => emptied(&value, outer),
        "drop-below" => emptied_below(&value, outer, inner),
        _ => value,
    };
    took(
        command,
        "sent",
        event,
        Fields::from([(name.to_owned(), value)]),
    )
}

fn stored(
    mode: &str,
    command: &str,
    input: &Fields,
    rows: &mut Vec<Fields>,
) -> SemanticCommandResult {
    assert_eq!(command, "demo.s.Sow");
    let branch = get(input, "branch");
    let branch = match mode {
        "drop-top" => emptied(&branch, "twigs"),
        _ => branch,
    };
    rows.push(Fields::from([
        ("garden_id".to_owned(), get(input, "garden_id")),
        ("branch".to_owned(), branch),
    ]));
    took(
        command,
        "sown",
        "demo.s.Sown",
        Fields::from([("garden_id".to_owned(), get(input, "garden_id"))]),
    )
}

fn invariants(
    mode: &str,
    command: &str,
    input: &Fields,
    _: &mut Vec<Fields>,
) -> SemanticCommandResult {
    let (name, event) = match command {
        "demo.i.SendQtys" => ("qtys", "demo.i.QtysSent"),
        "demo.i.SendCodes" => ("codes", "demo.i.CodesSent"),
        other => panic!("unexpected command {other}"),
    };
    let value = get(input, name);
    let value = if mode == "drop" {
        empty_like(&value)
    } else {
        value
    };
    took(
        command,
        "sent",
        event,
        Fields::from([(name.to_owned(), value)]),
    )
}

fn upper(mode: &str, command: &str, input: &Fields, _: &mut Vec<Fields>) -> SemanticCommandResult {
    let (name, event) = match command {
        "demo.u.MapFew" => ("labels", "demo.u.Counted"),
        "demo.u.ListFew" => ("items", "demo.u.Listed"),
        other => panic!("unexpected command {other}"),
    };
    let value = get(input, name);
    let n = size(Some(&value));
    // The off-by-one mutant: `< 2` where the model says `<= 2`.
    let few = if mode == "strict" { n < 2 } else { n <= 2 };
    took(
        command,
        if few { "few" } else { "many" },
        event,
        Fields::from([(name.to_owned(), value)]),
    )
}

fn run(suite: &ConformanceSuite, handler: Handler, mode: &'static str) -> Verdicts {
    support_go::rust_outcomes(
        suite,
        &Store {
            mode,
            handler,
            rows: RefCell::new(Vec::new()),
        },
    )
}

fn failing(verdicts: &Verdicts, prefix: &str) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(id, status)| id.starts_with(prefix) && *status != "passed")
        .map(|(id, status)| format!("{id}={status}"))
        .collect()
}

// ---- recursion -------------------------------------------------------------------------------------

/// Every recursion shape synthesizes, with no refusal, and its payload scenario sends the value
/// unfolded one level: the outer map or copied list holds one entry, and the unfolded entry's own
/// recursive member is empty.
#[test]
#[ignore = "story:collections-reach-their-upper-count-boundary; a nested list inside a recursive map entry is left empty"]
fn adv196p2_every_recursion_shape_synthesizes_unfolded_once() {
    let (suite, refusals) = synthesized(RECURSION);
    let mut problems = Vec::new();
    if !refusals.is_empty() {
        problems.push(format!("refusals: {refusals:#?}"));
    }
    for (command, name, outer, inner) in [
        ("demo.m.SendLeft", "left", "rights", "lefts"),
        ("demo.m.SendBranch", "branch", "twigs", "twigs"),
        ("demo.m.SendNode", "node", "kids", "kids"),
        ("demo.m.SendMixed", "mixed", "next", "next"),
    ] {
        let id = format!("{command}/outcome/sent");
        let Some(scenario) = scenario(&suite, &id) else {
            problems.push(format!("{id}: no scenario"));
            continue;
        };
        let Some(input) = sent_in(scenario, command) else {
            problems.push(format!("{id}: nothing sent"));
            continue;
        };
        let value = get(&input, name);
        let outer_value = field(&value, outer);
        if size(outer_value) != 1 {
            problems.push(format!(
                "{id}: `{name}.{outer}` holds {} entries: {value:?}",
                size(outer_value)
            ));
            continue;
        }
        let entry = first(outer_value).expect("one entry");
        // Mixed nests a list inside the map; the unfolded Mixed is one level further in.
        let unfolded = if command == "demo.m.SendMixed" {
            first(Some(entry))
        } else {
            Some(entry)
        };
        let Some(unfolded) = unfolded else {
            problems.push(format!(
                "{id}: the entry of `{name}.{outer}` holds nothing: {value:?}"
            ));
            continue;
        };
        if command == "demo.m.SendLeft" {
            // Left -> Right -> Left: the Right is the unfolded one, so its `lefts` is empty.
            if size(field(unfolded, inner)) != 0 {
                problems.push(format!("{id}: the unfolded Right is not closed: {value:?}"));
            }
        } else if size(field(unfolded, inner)) != 0 {
            problems.push(format!("{id}: the unfolded entry is not closed: {value:?}"));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

/// A target that empties the outer recursive member fails every recursion shape.
#[test]
fn adv196p2_a_target_dropping_the_unfolded_level_fails_every_recursion_shape() {
    let (suite, refusals) = synthesized(RECURSION);
    assert!(refusals.is_empty(), "{refusals:#?}");
    let correct = run(&suite, recursion, "correct");
    let dropped = run(&suite, recursion, "drop-top");
    let mut problems = Vec::new();
    let broken = failing(&correct, "");
    if !broken.is_empty() {
        problems.push(format!("the correct target fails: {broken:?}"));
    }
    for command in [
        "demo.m.SendLeft",
        "demo.m.SendBranch",
        "demo.m.SendNode",
        "demo.m.SendMixed",
    ] {
        if failing(&dropped, command).is_empty() {
            problems.push(format!(
                "{command}: a target that drops the copied level passes"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "{problems:#?}\n{correct:#?}\n{dropped:#?}"
    );
}

/// A recursive list stored and read back through a view: synthesized with no refusal, the view
/// row asserts the one twig, and a target that stores the branch without it fails.
#[test]
fn adv196p2_a_stored_recursive_list_is_asserted_on_the_view_row() {
    let (suite, refusals) = synthesized(STORED);
    assert!(refusals.is_empty(), "{refusals:#?}");
    let sent = all_sent(&suite, "demo.s.Sow");
    assert!(
        sent.iter()
            .any(|(_, input)| size(field(&get(input, "branch"), "twigs")) == 1),
        "no scenario sends a branch with one twig: {sent:#?}"
    );
    let correct = run(&suite, stored, "correct");
    let dropped = run(&suite, stored, "drop-top");
    assert!(failing(&correct, "").is_empty(), "{correct:#?}");
    assert!(
        !failing(&dropped, "").is_empty(),
        "a target that drops the twigs passes: {dropped:#?}"
    );
}

// ---- invariants on the element type ----------------------------------------------------------------

/// A copied map and a copied list whose element type has an invariant: synthesized with no
/// refusal, the payload scenario sends one element the invariant admits, and a target that drops
/// the collection fails.
#[test]
fn adv196p2_a_copied_collection_of_an_invariant_type_is_witnessed_and_checked() {
    let (suite, refusals) = synthesized(INVARIANTS);
    // No view publishes the element types, so their invariants have no scenario of their own
    // (ESS-SYNTH-013); that refusal is about the model, not the collections under attack.
    let refusals: Vec<String> = refusals
        .into_iter()
        .filter(|refusal| !refusal.contains("ESS-SYNTH-013"))
        .collect();
    let mut problems = Vec::new();
    if !refusals.is_empty() {
        problems.push(format!("refusals: {refusals:#?}"));
    }
    for (command, name) in [("demo.i.SendQtys", "qtys"), ("demo.i.SendCodes", "codes")] {
        let id = format!("{command}/outcome/sent");
        let Some(input) = scenario(&suite, &id).and_then(|s| sent_in(s, command)) else {
            problems.push(format!("{id}: no scenario sends it"));
            continue;
        };
        let value = get(&input, name);
        if size(Some(&value)) != 1 {
            problems.push(format!("{id}: `{name}` is {value:?}, not one element"));
            continue;
        }
        let element = first(Some(&value)).cloned().unwrap_or(Node::Null);
        let admitted = match (&element, name) {
            (Node::Number(n), "qtys") => n.get() >= 10.0,
            (Node::Text(t), "codes") => t.starts_with("SKU-"),
            _ => false,
        };
        if !admitted {
            problems.push(format!(
                "{id}: `{name}` element {element:?} breaks the invariant"
            ));
        }
    }
    let dropped = run(&suite, invariants, "drop");
    for command in ["demo.i.SendQtys", "demo.i.SendCodes"] {
        if failing(&dropped, command).is_empty() {
            problems.push(format!(
                "{command}: a target that drops the collection passes"
            ));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}\n{dropped:#?}");
}

// ---- the accepting boundary of an upper bound ------------------------------------------------------

/// `labels.count <= 2` accepts at 2: a target implementing `< 2` fails some scenario. The list twin
/// is the control.
#[test]
#[ignore = "story:collections-reach-their-upper-count-boundary"]
fn adv196p2_an_upper_count_bound_on_a_map_is_witnessed_at_its_accepting_boundary() {
    let (suite, refusals) = synthesized(UPPER);
    assert!(refusals.is_empty(), "{refusals:#?}");
    let strict = run(&suite, upper, "strict");
    let correct = run(&suite, upper, "correct");
    assert!(failing(&correct, "").is_empty(), "{correct:#?}");
    let sizes: Vec<(String, usize)> = all_sent(&suite, "demo.u.MapFew")
        .into_iter()
        .map(|(id, input)| (id, size(input.get("labels"))))
        .collect();
    assert!(
        !failing(&strict, "demo.u.MapFew").is_empty(),
        "a target implementing `labels.count < 2` passes; map sizes sent: {sizes:?}\n{strict:#?}"
    );
}

#[test]
#[ignore = "story:collections-reach-their-upper-count-boundary"]
fn adv196p2_control_an_upper_count_bound_on_a_list_is_witnessed_at_its_accepting_boundary() {
    let (suite, refusals) = synthesized(UPPER);
    assert!(refusals.is_empty(), "{refusals:#?}");
    let strict = run(&suite, upper, "strict");
    let sizes: Vec<(String, usize)> = all_sent(&suite, "demo.u.ListFew")
        .into_iter()
        .map(|(id, input)| (id, size(input.get("items"))))
        .collect();
    assert!(
        !failing(&strict, "demo.u.ListFew").is_empty(),
        "a target implementing `items.count < 2` passes; list sizes sent: {sizes:?}\n{strict:#?}"
    );
}

// ---- an update that writes a copied list -----------------------------------------------------------

/// A list created and then replaced by an update, each copied from the input.
const RELIST: &str = "format: ess/17
system: demo
version: v1
domain: demo.l
entities:
  - name: demo.l.Bag
    identity: {name: bag_id, type: String}
    fields:
      - {name: items, type: 'List<String>'}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.l.Changed
    fields:
      - {name: bag_id, type: String}
actors:
  - {name: demo.l.Clerk, may: [demo.l.Fill, demo.l.Refill]}
commands:
  - name: demo.l.Fill
    input:
      - {name: bag_id, type: String}
      - {name: items, type: 'List<String>'}
    outcomes:
      - name: filled
        creates: demo.l.Bag
        instance: bag_id
        emits: [demo.l.Changed]
        payload:
          demo.l.Changed: {bag_id: input.bag_id}
        sets:
          items: input.items
  - name: demo.l.Refill
    input:
      - {name: bag_id, type: String}
      - {name: items, type: 'List<String>'}
    outcomes:
      - name: refilled
        updates: demo.l.Bag
        instance: bag_id
        emits: [demo.l.Changed]
        payload:
          demo.l.Changed: {bag_id: input.bag_id}
        sets:
          items: input.items
views:
  - name: demo.l.Bags
    source: demo.l.Bag
    consistency: read_your_writes
    fields:
      - {name: bag_id, type: String}
      - {name: items, type: 'List<String>'}
";

fn relist(
    mode: &str,
    command: &str,
    input: &Fields,
    rows: &mut Vec<Fields>,
) -> SemanticCommandResult {
    let id = get(input, "bag_id");
    let payload = Fields::from([("bag_id".to_owned(), id.clone())]);
    match command {
        "demo.l.Fill" => {
            rows.push(Fields::from([
                ("bag_id".to_owned(), id),
                ("items".to_owned(), get(input, "items")),
            ]));
            took(command, "filled", "demo.l.Changed", payload)
        }
        "demo.l.Refill" => {
            let Some(row) = rows.iter_mut().find(|row| row.get("bag_id") == Some(&id)) else {
                return SemanticCommandResult::undeclared();
            };
            if mode != "refill-ignored" {
                row.insert("items".to_owned(), get(input, "items"));
            }
            took(command, "refilled", "demo.l.Changed", payload)
        }
        other => panic!("unexpected command {other}"),
    }
}

/// The update sends a list other than the one the row already holds, so a target that ignores the
/// update fails (the ess#161 rule, for a copied list now that it holds an element).
#[test]
fn adv196p2_an_update_that_writes_a_copied_list_is_moved_off_the_prior_value() {
    let (suite, refusals) = synthesized(RELIST);
    assert!(refusals.is_empty(), "{refusals:#?}");
    let correct = run(&suite, relist, "correct");
    let ignored = run(&suite, relist, "refill-ignored");
    assert!(failing(&correct, "").is_empty(), "{correct:#?}");
    let fills: Vec<_> = all_sent(&suite, "demo.l.Fill");
    let refills: Vec<_> = all_sent(&suite, "demo.l.Refill");
    assert!(
        !failing(&ignored, "demo.l.Refill").is_empty(),
        "a target that ignores the update passes\nfills: {fills:?}\nrefills: {refills:?}\n{ignored:#?}"
    );
}
