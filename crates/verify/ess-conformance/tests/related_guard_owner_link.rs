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
