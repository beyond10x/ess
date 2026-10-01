//! Adversary, pass 2, for synthesis of a `when_related` guard comparing the related row's link to
//! its owner with an input naming an owner (beyond10x/ess#271), after correction 1: a link input
//! the arrangement already bound ("pinned") is sent naming that owner, and the related row is
//! arranged against it.
//!
//! Each case runs the synthesized suite against a shelf implemented here, with one defect switched
//! in at a time: a suite witnessing both sides of the guard passes the shelf without a defect and
//! fails every defect that answers one side wrongly. The shelf also refuses (as undeclared) any
//! scenario that builds a world the model says no owner reaches: a second item under an owner whose
//! relation holds one.
#![allow(clippy::struct_excessive_bools, clippy::too_many_lines)]
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

const MODEL: &str = include_str!("fixtures/related-guard-owner-link.yaml");

/// A run resting in `Stopped` is refused whatever `Start` guards: `Stop`'s own link guard on a
/// driven `Stop` (pass 1 measured it), not this unit's.
const STOPPED: &str = "mini.m.Run/state/Stopped/refuses/mini.m.Stop";

const ITEMS_MANY: &str =
    "      - {name: items, kind: owns, target: mini.m.Item, cardinality: many, via: owner_id}\n";
const ITEMS_ONE: &str =
    "      - {name: items, kind: owns, target: mini.m.Item, cardinality: one, via: owner_id}\n";

/// `Owner` owns `Run` as well, `Start` files the run under the owner it names, and `Stop` refuses a
/// caller naming another owner — pass 1's `RUN_OWNED`. `Start`'s own arrangement, and every driven
/// `Start`, pins `owner_id` to the owner arranged for the run.
const RUN_OWNED: &[(&str, &str)] = &[
    (
        ITEMS_MANY,
        "      - {name: items, kind: owns, target: mini.m.Item, cardinality: many, via: owner_id}\n      - {name: runs, kind: owns, target: mini.m.Run, cardinality: many, via: owner_id}\n",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n    lifecycle:\n      initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: owner_id, type: mini.m.OwnerId}\n    lifecycle:\n      initial: Running",
    ),
    (
        "  - {name: mini.m.NotRunning, summary: The run is not running., fields: []}\n",
        "  - {name: mini.m.NotRunning, summary: The run is not running., fields: []}\n  - {name: mini.m.NotYours, summary: The run is another owner's., fields: []}\n",
    ),
    (
        "        sets:\n          item_id: input.item_id\n",
        "        sets:\n          item_id: input.item_id\n          owner_id: input.owner_id\n",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n    outcomes:\n      - name: stopped\n",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: owner_id, type: mini.m.OwnerId}\n    outcomes:\n      - name: not-yours\n        when_subject:\n          predicate: owner_id != input.owner_id\n        error: mini.m.NotYours\n      - name: stopped\n",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: item_id, type: mini.m.ItemId}\n",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: state, type: mini.m.Run.State}\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: owner_id, type: mini.m.OwnerId}\n",
    ),
];

/// `Claim` updates the owner its `owner_id` names, and refuses when the item its `item_id` names is
/// another owner's — pass 1's `CLAIM`. Its arrangement pins `owner_id` to the subject.
const CLAIM: &[(&str, &str)] = &[
    (
        "  - name: mini.m.Stop\n",
        "  - name: mini.m.Claim\n    input:\n      - {name: owner_id, type: mini.m.OwnerId}\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: label, type: String}\n    outcomes:\n      - name: no-item-claim\n        when_related: {via: input.item_id, exists: false}\n        error: mini.m.NoItem\n      - name: other-owner-claim\n        when_related: {via: input.item_id, predicate: owner_id != input.owner_id}\n        error: mini.m.OtherOwner\n      - name: claimed\n        updates: mini.m.Owner\n        instance: owner_id\n        emits: [mini.m.Claimed]\n        payload:\n          mini.m.Claimed: {owner_id: input.owner_id}\n        sets:\n          label: input.label\n  - name: mini.m.Stop\n",
    ),
    (
        "events:\n",
        "events:\n  - name: mini.m.Claimed\n    fields:\n      - {name: owner_id, type: mini.m.OwnerId}\n",
    ),
    ("    may: [", "    may: [mini.m.Claim, "),
    (
        "views:\n",
        "views:\n  - name: mini.m.Owners\n    source: mini.m.Owner\n    consistency: read_your_writes\n    fields:\n      - {name: owner_id, type: mini.m.OwnerId}\n      - {name: label, type: String}\n",
    ),
];

/// The fixture with each edit list applied in order; every edit must match.
fn text(edits: &[&[(&str, &str)]]) -> String {
    let mut out = MODEL.to_owned();
    for (from, to) in edits.iter().flat_map(|list| list.iter()) {
        let next = out.replacen(from, to, 1);
        assert_ne!(next, out, "`{from}` is in the model");
        out = next;
    }
    out
}

/// `edits`, then `Owner owns Item` narrowed to `cardinality: one`.
fn one(edits: &[&[(&str, &str)]]) -> String {
    let base = text(edits);
    let next = base.replacen(ITEMS_MANY, ITEMS_ONE, 1);
    assert_ne!(next, base, "the items relation is in the model");
    next
}

fn compiled(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("mini.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.cause.code()))
        .collect()
}

/// One defect an implementation of the related guard could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Takes the branch whatever owner the input names.
    IgnoresGuard,
    /// Refuses whatever owner the input names.
    RefusesEvery,
    /// Reads "the named owner holds some item", not "the named owner holds this item".
    NamedHoldsAny,
    /// Compares the item's owner with the first owner ever created, not the owner the input names:
    /// a target that checks against a default (the first tenant, the first account) in place of
    /// the caller's.
    FirstOwner,
    /// Compares the item's owner with the last owner created, not the owner the input names.
    LastOwner,
    /// Compares the named owner with the owner of the first item added, not the item named.
    FirstItem,
    /// Compares the named owner with the owner of the last item added, not the item named.
    LastItem,
}

type Row = BTreeMap<String, Node>;

/// The shelf, implemented here and not by the synthesizer.
struct Shelf {
    mutant: Mutant,
    /// `Owner owns Item` holds one item per owner: a second is a world the model says is not
    /// reached, and the shelf answers it as undeclared.
    one: bool,
    views: BTreeMap<String, Vec<String>>,
    owners: RefCell<Vec<Row>>,
    items: RefCell<Vec<Row>>,
    runs: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Shelf {
    fn new(text: &str, one: bool, mutant: Mutant) -> Self {
        let raw = RawSpecFile::parse(text).unwrap();
        let spec = Specification::assemble([(Source::new("mini.yaml"), raw)]).unwrap();
        let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
        let views = ir
            .views()
            .values()
            .map(|view| {
                (
                    view.name.to_string(),
                    view.fields.iter().map(|field| field.name.clone()).collect(),
                )
            })
            .collect();
        Self {
            mutant,
            one,
            views,
            owners: RefCell::default(),
            items: RefCell::default(),
            runs: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn mint(&self) -> Node {
        self.minted.set(self.minted.get() + 1);
        Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
    }

    fn token(&self) -> ConsistencyToken {
        ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap()
    }

    /// Whether the guard `owner_id != input.owner_id` refuses for this item and the named owner.
    fn refuses(&self, item: &Row, named: &Node) -> bool {
        let items = self.items.borrow();
        let owners = self.owners.borrow();
        match self.mutant {
            Mutant::IgnoresGuard => false,
            Mutant::RefusesEvery => true,
            Mutant::NamedHoldsAny => !items.iter().any(|row| &row["owner_id"] == named),
            Mutant::FirstOwner => owners
                .first()
                .is_none_or(|owner| item["owner_id"] != owner["owner_id"]),
            Mutant::LastOwner => owners
                .last()
                .is_none_or(|owner| item["owner_id"] != owner["owner_id"]),
            Mutant::FirstItem => items.first().is_none_or(|row| &row["owner_id"] != named),
            Mutant::LastItem => items.last().is_none_or(|row| &row["owner_id"] != named),
            Mutant::None => &item["owner_id"] != named,
        }
    }

    fn table(&self, entity: &str) -> &RefCell<Vec<Row>> {
        match entity {
            "Owner" => &self.owners,
            "Item" => &self.items,
            "Run" => &self.runs,
            other => panic!("no table {other}"),
        }
    }
}

fn find(table: &RefCell<Vec<Row>>, key: &str, value: &Node) -> Option<Row> {
    table
        .borrow()
        .iter()
        .find(|row| row.get(key) == Some(value))
        .cloned()
}

impl ConformanceTarget for Shelf {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shelf-adversary-2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        for entity in ["Owner", "Item", "Run"] {
            self.table(entity).replace(Vec::new());
        }
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let input = &request.input;
        let took = |name: &str| {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new(name).unwrap(),
            ))
        };
        let refused = |name: &str, error: &str| {
            took(name).with_error(DeclaredErrorValue::new(error.parse().unwrap()))
        };
        let event = |name: &str| ObservedEvent::new(format!("mini.m.{name}").parse().unwrap());
        let result = match command.to_string().as_str() {
            "mini.m.CreateOwner" => {
                let id = self.mint();
                let mut row = input.clone();
                row.insert("owner_id".into(), id.clone());
                self.owners.borrow_mut().push(row);
                took("created").emitting(event("OwnerCreated").with("owner_id", id))
            }
            "mini.m.AddItem" => {
                let owner = &input["owner_id"];
                if find(&self.owners, "owner_id", owner).is_none() {
                    return Ok(SemanticCommandResult::undeclared());
                }
                // A second item under an owner that holds one: no scenario may build this.
                if self.one && find(&self.items, "owner_id", owner).is_some() {
                    return Ok(SemanticCommandResult::undeclared());
                }
                let id = self.mint();
                let mut row = input.clone();
                row.insert("item_id".into(), id.clone());
                self.items.borrow_mut().push(row);
                took("added").emitting(event("ItemAdded").with("item_id", id))
            }
            "mini.m.Start" => {
                let named = input["owner_id"].clone();
                let Some(item) = find(&self.items, "item_id", &input["item_id"]) else {
                    return Ok(refused("no-item", "mini.m.NoItem").with_consistency(self.token()));
                };
                if self.refuses(&item, &named) {
                    refused("other-owner", "mini.m.OtherOwner")
                } else {
                    let id = self.mint();
                    let mut row = input.clone();
                    row.insert("run_id".into(), id.clone());
                    row.insert("state".into(), Node::Text("Running".into()));
                    self.runs.borrow_mut().push(row);
                    took("started").emitting(event("Started").with("run_id", id))
                }
            }
            "mini.m.Stop" => {
                let id = input["run_id"].clone();
                let Some(run) = find(&self.runs, "run_id", &id) else {
                    let refusal = refused("not-running", "mini.m.NotRunning");
                    return Ok(refusal.with_consistency(self.token()));
                };
                if let Some(named) = input.get("owner_id") {
                    if run.get("owner_id") != Some(named) {
                        let refusal = refused("not-yours", "mini.m.NotYours");
                        return Ok(refusal.with_consistency(self.token()));
                    }
                }
                if run["state"] == Node::Text("Running".into()) {
                    for row in self.runs.borrow_mut().iter_mut() {
                        if row["run_id"] == id {
                            row.insert("state".into(), Node::Text("Stopped".into()));
                        }
                    }
                    took("stopped").emitting(event("RunStopped").with("run_id", id))
                } else {
                    refused("not-running", "mini.m.NotRunning")
                }
            }
            "mini.m.Claim" => {
                let named = input["owner_id"].clone();
                let Some(item) = find(&self.items, "item_id", &input["item_id"]) else {
                    let refusal = refused("no-item-claim", "mini.m.NoItem");
                    return Ok(refusal.with_consistency(self.token()));
                };
                if find(&self.owners, "owner_id", &named).is_none() {
                    return Ok(SemanticCommandResult::undeclared());
                }
                if self.refuses(&item, &named) {
                    refused("other-owner-claim", "mini.m.OtherOwner")
                } else {
                    for row in self.owners.borrow_mut().iter_mut() {
                        if row["owner_id"] == named {
                            row.insert("label".into(), input["label"].clone());
                        }
                    }
                    took("claimed").emitting(event("Claimed").with("owner_id", named))
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(self.token()))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let view = request.view.to_string();
        let fields = self
            .views
            .get(&view)
            .unwrap_or_else(|| panic!("no view {view}"));
        let entity = match view.as_str() {
            "mini.m.Items" => "Item",
            "mini.m.Runs" => "Run",
            "mini.m.Owners" => "Owner",
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(
            self.table(entity)
                .borrow()
                .iter()
                .map(|row| {
                    fields
                        .iter()
                        .map(|field| (field.clone(), row[field].clone()))
                        .collect::<Row>()
                })
                .collect::<Vec<Row>>(),
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// The scenarios of the model's suite that do not pass against `mutant`, with what they reported.
/// Every refusal but those naming one of `beside` fails the case first.
fn failing(text: &str, one: bool, mutant: Mutant, beside: &[&str]) -> BTreeMap<String, String> {
    let synthesis = compiled(text);
    let other: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| !beside.iter().any(|named| refusal.contains(named)))
        .collect();
    assert!(other.is_empty(), "refused: {other:#?}");
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Shelf::new(text, one, mutant))
        .into_report();
    let failed: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                format!(
                    "{:?} {:?}",
                    scenario.status,
                    scenario.diagnostics().collect::<Vec<_>>()
                ),
            )
        })
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

/// A shelf without a defect passes every scenario, and each of `mutants` fails at least one.
fn kills(text: &str, one: bool, beside: &[&str], mutants: &[Mutant]) {
    assert_eq!(
        failing(text, one, Mutant::None, beside),
        BTreeMap::new(),
        "a correct shelf fails"
    );
    let survivors: Vec<Mutant> = mutants
        .iter()
        .copied()
        .filter(|mutant| failing(text, one, *mutant, beside).is_empty())
        .collect();
    assert_eq!(
        survivors,
        Vec::<Mutant>::new(),
        "defects that pass every scenario"
    );
}

const OWNER_DEFECTS: &[Mutant] = &[
    Mutant::IgnoresGuard,
    Mutant::RefusesEvery,
    Mutant::NamedHoldsAny,
    Mutant::FirstOwner,
    Mutant::LastOwner,
    Mutant::FirstItem,
    Mutant::LastItem,
];

// ---- control: nothing pinned -----------------------------------------------------------------

/// The story's fixture, where no link input is pinned: every owner defect is killed. The control
/// for the pinned cases below — the same mutants, the same shelf.
#[test]
fn control_the_unpinned_fixture_kills_every_owner_defect() {
    kills(&text(&[]), false, &[], OWNER_DEFECTS);
}

/// The story's fixture with one item per owner, where nothing is pinned either.
#[test]
fn control_one_item_per_owner_unpinned_is_witnessed_on_both_sides() {
    kills(
        &one(&[]),
        true,
        &[],
        &[Mutant::IgnoresGuard, Mutant::RefusesEvery],
    );
}

// ---- a relation holding one row per owner, pinned ---------------------------------------------

/// `Claim` over an owner that holds at most one item. The owner the subject arrangement creates
/// holds none, so an item filed under it is a state the relation admits, and it is the only one on
/// which `claimed` is taken: both sides are reachable and both are owed (the story's outcome
/// names an `owns` relation, not a `cardinality: many` one).
#[test]
fn a_claim_by_an_owner_of_one_item_is_witnessed_on_both_sides() {
    kills(
        &one(&[CLAIM]),
        true,
        &[],
        &[Mutant::IgnoresGuard, Mutant::RefusesEvery],
    );
}

/// `Start` for a run filed under an owner that holds at most one item. `Start`'s own success
/// scenario, and every `Stop` scenario that needs a run, sends `Start` pinned to a fresh owner that
/// holds no item yet: an item under it is admitted, and without one no run is ever started.
#[test]
fn a_run_owned_start_by_an_owner_of_one_item_is_witnessed() {
    kills(
        &one(&[RUN_OWNED]),
        true,
        &[STOPPED],
        &[Mutant::IgnoresGuard, Mutant::RefusesEvery],
    );
}

/// Whatever is refused over one item per owner, nothing emitted builds a world the relation forbids
/// or answers the guard inconsistently: the correct shelf, which answers a second item under one
/// owner as undeclared, passes every scenario that is emitted.
#[test]
fn one_item_per_owner_emits_no_scenario_the_relation_forbids() {
    for model in [one(&[CLAIM]), one(&[RUN_OWNED])] {
        let beside = [
            STOPPED,
            "mini.m.Claim/outcome/claimed",
            "mini.m.Start/outcome/started",
            "which no input reaches",
        ];
        assert_eq!(
            failing(&model, true, Mutant::None, &beside),
            BTreeMap::new()
        );
    }
}

// ---- mutants that compare against the pinned owner --------------------------------------------

/// The pinned `Start` (the run is filed under the owner `Start` names): the same owner defects the
/// unpinned fixture kills are killed here too. Correction 1 moves the related rows after the
/// arrangement whenever a pin is held, so the owner the input names is created before any other.
#[test]
fn the_pinned_start_kills_every_owner_defect() {
    kills(&text(&[RUN_OWNED]), false, &[STOPPED], OWNER_DEFECTS);
}

/// The pinned `Claim` (the input names the subject): the same owner defects are killed.
#[test]
fn the_pinned_claim_kills_every_owner_defect() {
    kills(&text(&[CLAIM]), false, &[], OWNER_DEFECTS);
}
