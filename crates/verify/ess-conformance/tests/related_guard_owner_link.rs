//! A `when_related:` guard comparing the related row's link to its owner with an input naming an
//! owner is witnessed on both sides (beyond10x/ess#271).
//!
//! `Owner` owns `Item` via `owner_id`, and `Start` refuses `other-owner` when the item its
//! `item_id` names carries `owner_id != input.owner_id`. Neither side is a value the specification
//! spells: the item's owner is an instance the arrangement created, and the input names one. So the
//! refusal is sent for an item owned by a second arranged owner, and the success for one owned by
//! the owner the input names — the case `when_subject` already witnesses (beyond10x/ess#193).
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    ConformanceScenario,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/related-guard-owner-link.yaml");

const OTHER_OWNER: &str = "mini.m.Start/outcome/other-owner";
const STARTED: &str = "mini.m.Start/outcome/started";

fn synthesis() -> Synthesis {
    synthesis_of(MODEL)
}

fn synthesis_of(model: &str) -> Synthesis {
    let raw = RawSpecFile::parse(model).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("mini.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(synthesis)))
}

fn instance(value: Option<&ScenarioValue>) -> Option<String> {
    match value? {
        ScenarioValue::Instance { instance } => Some(instance.to_string()),
        _ => None,
    }
}

/// Every item the scenario adds, by the instance it captures, with the owner instance it was added
/// under.
fn item_owners(scenario: &ConformanceScenario) -> BTreeMap<String, Option<String>> {
    let mut out = BTreeMap::new();
    let mut owner = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.AddItem" =>
            {
                owner = Some(instance(input.get("owner_id")));
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Item" => {
                out.insert(
                    instance.to_string(),
                    owner.take().expect("a capture follows its AddItem"),
                );
            }
            _ => {}
        }
    }
    out
}

/// The `(owner named by the input, owner of the item named by the input)` of the last `Start` the
/// scenario sends.
fn start(scenario: &ConformanceScenario) -> (String, String) {
    let input = scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.Start" =>
            {
                Some(input)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("the scenario starts: {scenario:#?}"));
    let named = instance(input.get("owner_id"))
        .unwrap_or_else(|| panic!("owner_id names an arranged owner: {input:#?}"));
    let item = instance(input.get("item_id"))
        .unwrap_or_else(|| panic!("item_id names an arranged item: {input:#?}"));
    let held = item_owners(scenario)
        .get(&item)
        .cloned()
        .flatten()
        .unwrap_or_else(|| panic!("item {item} was added under an arranged owner: {scenario:#?}"));
    (named, held)
}

/// Nothing is refused: neither branch of the guard (ESS-SYNTH-003), nor anything downstream of a
/// run only `started` creates (ESS-SYNTH-004).
#[test]
fn both_sides_of_the_link_guard_are_synthesized() {
    let synthesis = synthesis();
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    scenario(&synthesis, OTHER_OWNER);
    scenario(&synthesis, STARTED);
}

/// A scenario of another family that needs a run sends `Start` as an arranging act, and names the
/// owner of the item it sends: the owner is bound there as in `started`'s own scenario.
#[test]
fn a_driven_start_names_the_items_owner() {
    let synthesis = synthesis();
    let driven: Vec<&ConformanceScenario> = synthesis
        .suite
        .scenarios
        .iter()
        .filter(|(id, _)| !id.to_string().starts_with("mini.m.Start/"))
        .map(|(_, scenario)| scenario)
        .filter(|scenario| {
            scenario.steps.iter().any(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "mini.m.Start")
            })
        })
        .collect();
    assert!(!driven.is_empty(), "some scenario drives Start");
    for scenario in driven {
        let (named, held) = start(scenario);
        assert_eq!(named, held, "{scenario:#?}");
    }
}

/// The refusal is sent for an item another arranged owner holds, and the success for one the
/// named owner holds: both owners are instances the scenario created, never a literal.
#[test]
fn each_side_names_an_arranged_owner() {
    let synthesis = synthesis();
    let (named, held) = start(scenario(&synthesis, OTHER_OWNER));
    assert_ne!(
        named, held,
        "the refusal names another owner than the item's"
    );
    let (named, held) = start(scenario(&synthesis, STARTED));
    assert_eq!(named, held, "the success names the item's own owner");
}

/// The fixture with `Owner` owning `Run` too: `Start` files the run under the owner its
/// `owner_id` names, and `Stop` is sent naming the run's owner.
fn run_owned() -> String {
    let edits = [
        (
            "      - {name: items, kind: owns, target: mini.m.Item, cardinality: many, via: owner_id}\n",
            "      - {name: items, kind: owns, target: mini.m.Item, cardinality: many, via: owner_id}\n      - {name: runs, kind: owns, target: mini.m.Run, cardinality: many, via: owner_id}\n",
        ),
        (
            "      - {name: item_id, type: mini.m.ItemId}\n    lifecycle:\n      initial: Running",
            "      - {name: item_id, type: mini.m.ItemId}\n      - {name: owner_id, type: mini.m.OwnerId}\n    lifecycle:\n      initial: Running",
        ),
        (
            "        sets:\n          item_id: input.item_id\n",
            "        sets:\n          item_id: input.item_id\n          owner_id: input.owner_id\n",
        ),
    ];
    let mut model = MODEL.to_owned();
    for (from, to) in edits {
        let next = model.replacen(from, to, 1);
        assert_ne!(next, model, "`{from}` is in the model");
        model = next;
    }
    model
}

/// Every `Owner` the scenario captures, in order.
fn owners(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Owner" => Some(instance.to_string()),
            _ => None,
        })
        .collect()
}

/// A run created for another scenario is filed under an owner the arrangement brings into being
/// first (`created_owned`); the driven `Start` that creates it names that owner, and the item it
/// sends is that owner's — so the owner arranged for the run is the one used, not replaced by a
/// second owner the item was filed under (beyond10x/ess#271). And every owner a scenario arranges
/// is named by some later send: none is created and left unused.
#[test]
fn a_driven_start_reuses_the_owner_arranged_for_the_run() {
    let synthesis = synthesis_of(&run_owned());
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let mut driven = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        let sends: Vec<&BTreeMap<String, ScenarioValue>> = scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => Some(input),
                _ => None,
            })
            .collect();
        for owner in owners(scenario) {
            assert!(
                sends.iter().any(|input| input
                    .values()
                    .any(|value| instance(Some(value)).as_deref() == Some(owner.as_str()))),
                "{id}: owner {owner} is created and never named: {scenario:#?}"
            );
        }
        if id.to_string().starts_with("mini.m.Start/") {
            continue;
        }
        if sends.iter().any(|input| input.contains_key("item_id")) {
            let (named, held) = start(scenario);
            assert_eq!(named, held, "{id}: {scenario:#?}");
            driven += 1;
        }
    }
    assert!(driven > 0, "some scenario drives Start");
}

/// A node bound only on the board that owns it: `Bind` refuses a missing board and a missing
/// node, each by `exists: false`, then a node whose `board_id` is not the input's
/// (beyond10x/ess#801).
const BOARD: &str = "format: ess/25
system: wiring
version: v1
domain: wiring.board
summary: A node is bound only on the board that owns it.
types:
  - {name: wiring.board.BoardId, kind: newtype, of: Uuid}
  - {name: wiring.board.NodeId, kind: newtype, of: Uuid}
  - {name: wiring.board.BindingId, kind: newtype, of: Uuid}
entities:
  - name: wiring.board.Board
    identity: {name: board_id, type: wiring.board.BoardId}
    fields:
      - {name: label, type: String}
    relations:
      - {name: nodes, kind: owns, target: wiring.board.Node, cardinality: many, via: board_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: wiring.board.Node
    identity: {name: node_id, type: wiring.board.NodeId}
    fields:
      - {name: board_id, type: wiring.board.BoardId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: wiring.board.Binding
    identity: {name: binding_id, type: wiring.board.BindingId}
    fields:
      - {name: board_id, type: wiring.board.BoardId}
      - {name: node_id, type: wiring.board.NodeId}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
errors:
  - {name: wiring.board.BoardNotFound, summary: No board carries this identity., fields: []}
  - {name: wiring.board.NodeNotFound, summary: No node carries this identity., fields: []}
  - {name: wiring.board.NodeOfAnotherBoard, summary: The node belongs to another board., fields: []}
events:
  - name: wiring.board.BoardOpened
    fields: [{name: board_id, type: wiring.board.BoardId}]
  - name: wiring.board.NodeAdded
    fields: [{name: node_id, type: wiring.board.NodeId}]
  - name: wiring.board.Bound
    fields: [{name: binding_id, type: wiring.board.BindingId}]
actors:
  - name: wiring.board.Operator
    may: [wiring.board.OpenBoard, wiring.board.AddNode, wiring.board.Bind]
commands:
  - name: wiring.board.OpenBoard
    input:
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: wiring.board.Board
        instance: board_id
        emits: [wiring.board.BoardOpened]
        payload: {wiring.board.BoardOpened: {board_id: {generated: true}}}
        sets: {label: input.label}
  - name: wiring.board.AddNode
    input:
      - {name: board_id, type: wiring.board.BoardId}
    outcomes:
      - name: no-board
        when_related: {via: input.board_id, exists: false}
        error: wiring.board.BoardNotFound
      - name: added
        creates: wiring.board.Node
        instance: node_id
        emits: [wiring.board.NodeAdded]
        payload: {wiring.board.NodeAdded: {node_id: {generated: true}}}
        sets: {board_id: input.board_id}
  - name: wiring.board.Bind
    input:
      - {name: board_id, type: wiring.board.BoardId}
      - {name: node_id, type: wiring.board.NodeId}
    outcomes:
      - name: no-board
        when_related: {via: input.board_id, exists: false}
        error: wiring.board.BoardNotFound
      - name: no-node
        when_related: {via: input.node_id, exists: false}
        error: wiring.board.NodeNotFound
      - name: other-board
        when_related: {via: input.node_id, predicate: board_id != input.board_id}
        error: wiring.board.NodeOfAnotherBoard
      - name: bound
        creates: wiring.board.Binding
        instance: binding_id
        emits: [wiring.board.Bound]
        payload: {wiring.board.Bound: {binding_id: {generated: true}}}
        sets: {board_id: input.board_id, node_id: input.node_id}
views:
  - name: wiring.board.Bindings
    source: wiring.board.Binding
    consistency: read_your_writes
    fields:
      - {name: binding_id, type: wiring.board.BindingId}
      - {name: board_id, type: wiring.board.BoardId}
      - {name: node_id, type: wiring.board.NodeId}
";

#[test]
fn an_owner_link_guard_beside_an_owner_existence_guard_is_witnessed() {
    use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};
    let raw = RawSpecFile::parse(BOARD).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    for branch in ["no-board", "no-node", "other-board", "bound"] {
        scenario(&synthesis, &format!("wiring.board.Bind/outcome/{branch}"));
    }
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|run| run.status != Status::Passed)
        .map(|run| format!("{}: {:?}", run.scenario, run.checks))
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
