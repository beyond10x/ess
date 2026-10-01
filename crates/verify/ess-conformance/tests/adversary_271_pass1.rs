//! Adversary, pass 1, for synthesis of a `when_related` guard comparing the related row's link to
//! its owner with an input naming an owner (beyond10x/ess#271).
//!
//! The story's outcome: the guard "is witnessed on both sides: a row owned by another owner for the
//! refusal, one owned by the input owner for success". Each case below asks that of a shape the
//! unit's own test does not cover, and runs the synthesized suite against a shelf implemented here
//! with one defect switched in at a time: a suite witnessing both sides passes the shelf without a
//! defect and fails every defect that answers one side wrongly.
#![allow(clippy::struct_excessive_bools, clippy::too_many_lines)]
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
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

const GUARD: &str = "predicate: owner_id != input.owner_id";

/// How the shelf decides the related guard of `Start` (and `Claim`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Guard {
    /// `owner_id != input.owner_id`
    Ne,
    /// `owner_id == input.owner_id`
    Eq,
    /// `{all: [owner_id != input.owner_id, note == "locked"]}`
    All,
    /// `{any: [owner_id != input.owner_id, note == "locked"]}`
    Any,
}

struct Shape {
    guard: Guard,
    /// Replacements applied to the fixture text, each of which must match.
    edits: &'static [(&'static str, &'static str)],
}

const PLAIN: Shape = Shape {
    guard: Guard::Ne,
    edits: &[],
};

fn text(shape: &Shape) -> String {
    let mut out = MODEL.to_owned();
    for (from, to) in shape.edits {
        let next = out.replacen(from, to, 1);
        assert_ne!(next, out, "`{from}` is in the model");
        out = next;
    }
    out
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

/// FNV-1a as `witness::uuid_of` spreads it, so the Uuid token of an instance is recognisable.
fn uuid_of(seed: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in seed.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("00000000-0000-4000-8000-{:012x}", hash & 0xffff_ffff_ffff)
}

/// Every search token that reached the emitted suite as a literal value.
fn leaked(synthesis: &Synthesis) -> Vec<String> {
    let json = synthesis
        .suite
        .to_canonical_json()
        .expect("the suite renders");
    let mut out = Vec::new();
    if json.contains("instance:") {
        out.push("`instance:` text".to_owned());
    }
    let mut names = vec!["owner".to_owned()];
    names.extend((2..=80).map(|n| format!("owner-{n}")));
    for name in names {
        let token = uuid_of(&format!("instance:{name}"));
        if json.contains(&token) {
            out.push(format!("the Uuid token of {name}: {token}"));
        }
    }
    out
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
    /// Compares the named owner with the owner of the first item added, not the item named.
    FirstItem,
    /// Compares the named owner with the owner of the last item added, not the item named.
    LastItem,
    /// Under `all:`/`any:`, reads the note half only.
    IgnoresLink,
    /// Under `all:`/`any:`, reads the link half only.
    IgnoresNote,
}

type Row = BTreeMap<String, Node>;

/// The shelf, implemented here and not by the synthesizer.
struct Shelf {
    guard: Guard,
    mutant: Mutant,
    views: BTreeMap<String, Vec<String>>,
    orgs: RefCell<Vec<Row>>,
    owners: RefCell<Vec<Row>>,
    items: RefCell<Vec<Row>>,
    runs: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Shelf {
    fn new(shape: &Shape, mutant: Mutant) -> Self {
        let raw = RawSpecFile::parse(&text(shape)).unwrap();
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
            guard: shape.guard,
            mutant,
            views,
            orgs: RefCell::default(),
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

    /// Whether the guard's refusal answers for this item and the named owner.
    fn refuses(&self, item: &Row, named: &Node) -> bool {
        let items = self.items.borrow();
        let differs = match self.mutant {
            Mutant::NamedHoldsAny => !items.iter().any(|row| &row["owner_id"] == named),
            Mutant::FirstItem => items.first().is_none_or(|row| &row["owner_id"] != named),
            Mutant::LastItem => items.last().is_none_or(|row| &row["owner_id"] != named),
            _ => &item["owner_id"] != named,
        };
        let locked = item.get("note") == Some(&Node::Text("locked".into()));
        match (self.mutant, self.guard) {
            (Mutant::IgnoresGuard, _) => false,
            (Mutant::RefusesEvery, _) => true,
            (Mutant::IgnoresLink, Guard::All | Guard::Any) => locked,
            (Mutant::IgnoresNote, Guard::All | Guard::Any) | (_, Guard::Ne) => differs,
            (_, Guard::Eq) => !differs,
            (_, Guard::All) => differs && locked,
            (_, Guard::Any) => differs || locked,
        }
    }

    fn table(&self, entity: &str) -> &RefCell<Vec<Row>> {
        match entity {
            "Org" => &self.orgs,
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
        Ok(ImplementationIdentity::new("shelf-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        for entity in ["Org", "Owner", "Item", "Run"] {
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
            "mini.m.CreateOrg" => {
                let id = self.mint();
                self.orgs
                    .borrow_mut()
                    .push(Row::from([("org_id".into(), id.clone())]));
                took("created").emitting(event("OrgCreated").with("org_id", id))
            }
            "mini.m.CreateOwner" => {
                if let Some(org) = input.get("org_id") {
                    if find(&self.orgs, "org_id", org).is_none() {
                        return Ok(SemanticCommandResult::undeclared());
                    }
                }
                let id = self.mint();
                let mut row = input.clone();
                row.insert("owner_id".into(), id.clone());
                self.owners.borrow_mut().push(row);
                took("created").emitting(event("OwnerCreated").with("owner_id", id))
            }
            "mini.m.CreateShelf" => {
                let id = self.mint();
                took("created").emitting(event("ShelfCreated").with("shelf_id", id))
            }
            "mini.m.AddItem" => {
                if find(&self.owners, "owner_id", &input["owner_id"]).is_none() {
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
                // The model declares no unknown-instance branch: a run nobody started is not
                // running.
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

/// The scenarios of `shape`'s suite that do not pass against `mutant`, with what they reported.
fn failing(shape: &Shape, mutant: Mutant) -> BTreeMap<String, String> {
    failing_beside(shape, mutant, &[])
}

/// [`failing`], where the refusals naming one of `beside` are ones this case is not about.
fn failing_beside(shape: &Shape, mutant: Mutant, beside: &[&str]) -> BTreeMap<String, String> {
    let synthesis = compiled(&text(shape));
    let other: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| !beside.iter().any(|named| refusal.contains(named)))
        .collect();
    assert!(other.is_empty(), "{other:#?}");
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Shelf::new(shape, mutant))
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

fn names(failed: &BTreeMap<String, String>) -> BTreeSet<String> {
    failed.keys().cloned().collect()
}

/// Each defect in `mutants` fails at least one scenario of `shape`, and a shelf without one passes.
fn kills(shape: &Shape, mutants: &[Mutant]) {
    let clean = failing(shape, Mutant::None);
    assert_eq!(clean, BTreeMap::new(), "a correct shelf fails");
    let survivors: Vec<Mutant> = mutants
        .iter()
        .copied()
        .filter(|mutant| failing(shape, *mutant).is_empty())
        .collect();
    assert_eq!(
        survivors,
        Vec::<Mutant>::new(),
        "defects that pass every scenario"
    );
}

// ---- control: the unit's own fixture ---------------------------------------------------------

#[test]
fn control_the_fixture_passes_the_shelf_and_kills_every_owner_defect() {
    let synthesis = compiled(&text(&PLAIN));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    kills(
        &PLAIN,
        &[
            Mutant::IgnoresGuard,
            Mutant::RefusesEvery,
            Mutant::NamedHoldsAny,
            Mutant::FirstItem,
            Mutant::LastItem,
        ],
    );
    let ignored = names(&failing(&PLAIN, Mutant::IgnoresGuard));
    assert!(
        ignored.contains("mini.m.Start/outcome/other-owner"),
        "{ignored:?}"
    );
    let refused = names(&failing(&PLAIN, Mutant::RefusesEvery));
    assert!(
        refused.contains("mini.m.Start/outcome/started"),
        "{refused:?}"
    );
}

// ---- `==` rather than `!=`, and `not` ---------------------------------------------------------

const EQ: Shape = Shape {
    guard: Guard::Eq,
    edits: &[(GUARD, "predicate: owner_id == input.owner_id")],
};

#[test]
fn an_equality_link_guard_is_witnessed_on_both_sides() {
    let synthesis = compiled(&text(&EQ));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    kills(
        &EQ,
        &[
            Mutant::IgnoresGuard,
            Mutant::RefusesEvery,
            Mutant::FirstItem,
            Mutant::LastItem,
        ],
    );
}

const NOT: Shape = Shape {
    guard: Guard::Ne,
    edits: &[(GUARD, "predicate: not owner_id == input.owner_id")],
};

#[test]
fn a_negated_equality_is_witnessed_like_its_inequality() {
    let synthesis = compiled(&text(&NOT));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    kills(&NOT, &[Mutant::IgnoresGuard, Mutant::RefusesEvery]);
}

// ---- connectives -----------------------------------------------------------------------------

const ALL: Shape = Shape {
    guard: Guard::All,
    edits: &[(
        GUARD,
        "predicate: {all: [owner_id != input.owner_id, note == \"locked\"]}",
    )],
};

#[test]
fn a_link_guard_in_all_is_witnessed_half_by_half() {
    let synthesis = compiled(&text(&ALL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    kills(
        &ALL,
        &[
            Mutant::IgnoresGuard,
            Mutant::RefusesEvery,
            Mutant::IgnoresLink,
            Mutant::IgnoresNote,
        ],
    );
}

const ANY: Shape = Shape {
    guard: Guard::Any,
    edits: &[(
        GUARD,
        "predicate: {any: [owner_id != input.owner_id, note == \"locked\"]}",
    )],
};

#[test]
fn a_link_guard_in_any_is_witnessed_half_by_half() {
    let synthesis = compiled(&text(&ANY));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    kills(
        &ANY,
        &[
            Mutant::IgnoresGuard,
            Mutant::RefusesEvery,
            Mutant::IgnoresLink,
            Mutant::IgnoresNote,
        ],
    );
}

// ---- an owner of the owner -------------------------------------------------------------------

const CHAIN: Shape = Shape {
    guard: Guard::Ne,
    edits: &[
        (
            "  - {name: mini.m.OwnerId, kind: newtype, of: Uuid}\n",
            "  - {name: mini.m.OwnerId, kind: newtype, of: Uuid}\n  - {name: mini.m.OrgId, kind: newtype, of: Uuid}\n",
        ),
        (
            "entities:\n",
            "entities:\n  - name: mini.m.Org\n    identity: {name: org_id, type: mini.m.OrgId}\n    fields: []\n    relations:\n      - {name: owners, kind: owns, target: mini.m.Owner, cardinality: many, via: org_id}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\n",
        ),
        (
            "      - {name: label, type: String}\n    relations:",
            "      - {name: label, type: String}\n      - {name: org_id, type: mini.m.OrgId}\n    relations:",
        ),
        (
            "events:\n",
            "events:\n  - name: mini.m.OrgCreated\n    fields:\n      - {name: org_id, type: mini.m.OrgId}\n",
        ),
        ("    may: [", "    may: [mini.m.CreateOrg, "),
        (
            "commands:\n",
            "commands:\n  - name: mini.m.CreateOrg\n    input: []\n    outcomes:\n      - name: created\n        creates: mini.m.Org\n        instance: org_id\n        emits: [mini.m.OrgCreated]\n        payload:\n          mini.m.OrgCreated: {org_id: {generated: true}}\n",
        ),
        (
            "    input:\n      - {name: label, type: String}\n",
            "    input:\n      - {name: label, type: String}\n      - {name: org_id, type: mini.m.OrgId}\n",
        ),
        (
            "          label: input.label\n",
            "          label: input.label\n          org_id: input.org_id\n",
        ),
    ],
};

#[test]
fn a_link_guard_on_an_owner_with_an_owner_of_its_own_is_witnessed() {
    let synthesis = compiled(&text(&CHAIN));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    kills(
        &CHAIN,
        &[
            Mutant::IgnoresGuard,
            Mutant::RefusesEvery,
            Mutant::NamedHoldsAny,
            Mutant::FirstItem,
            Mutant::LastItem,
        ],
    );
}

// ---- the run is owned by the owner too: a link guard downstream of the driven Start ----------

/// `Owner` owns `Run` as well, `Start` files the run under the owner it names, and `Stop` refuses a
/// caller naming another owner (`when_subject`, beyond10x/ess#193). Every `Stop` scenario needs a
/// run, so `Start` is driven with an owner the arrangement binds for the run (`created_owned`) —
/// the case the implementor left open, where `bind_links` overwrites that binding.
const RUN_OWNED: Shape = Shape {
    guard: Guard::Ne,
    edits: &[
        (
            "      - {name: items, kind: owns, target: mini.m.Item, cardinality: many, via: owner_id}\n",
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
    ],
};

#[test]
fn a_run_filed_under_the_driven_owner_is_stopped_by_that_owner() {
    // A run resting in `Stopped` is refused whatever `Start` guards (measured with `Start` guarded
    // by `note == "locked"` alone): `Stop`'s own link guard on a driven `Stop`, not this unit's.
    const STOPPED: &str = "mini.m.Run/state/Stopped/refuses/mini.m.Stop";
    let synthesis = compiled(&text(&RUN_OWNED));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    assert_eq!(
        failing_beside(&RUN_OWNED, Mutant::None, &[STOPPED]),
        BTreeMap::new()
    );
    for mutant in [Mutant::IgnoresGuard, Mutant::RefusesEvery] {
        assert!(
            !failing_beside(&RUN_OWNED, mutant, &[STOPPED]).is_empty(),
            "{mutant:?} passed every scenario"
        );
    }
}

// ---- the command's own subject is the owner the input names ----------------------------------

/// `Claim` updates the owner its `owner_id` names, and refuses when the item its `item_id` names
/// is another owner's. The subject arrangement binds `owner_id` to the subject; `prepare_at` then
/// extends the binding with the owner the related row's link names (the open overwrite case).
const CLAIM: Shape = Shape {
    guard: Guard::Ne,
    edits: &[
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
    ],
};

#[test]
fn a_claim_of_the_subject_owner_is_witnessed_on_both_sides() {
    let synthesis = compiled(&text(&CLAIM));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    kills(
        &CLAIM,
        &[
            Mutant::IgnoresGuard,
            Mutant::RefusesEvery,
            Mutant::NamedHoldsAny,
            Mutant::FirstItem,
            Mutant::LastItem,
        ],
    );
}

/// The instance an input field names, where it names one.
fn named(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<String> {
    match input.get(field)? {
        ScenarioValue::Instance { instance } => Some(instance.to_string()),
        _ => None,
    }
}

/// The success is sent naming, as `owner_id`, the owner the item it names was added under: the
/// scenario, read without running it.
#[test]
fn the_claimed_scenario_names_the_owner_of_the_item_it_sends() {
    let synthesis = compiled(&text(&CLAIM));
    let id = ScenarioId::parse("mini.m.Claim/outcome/claimed").unwrap();
    let scenario = synthesis
        .suite
        .scenarios
        .get(&id)
        .unwrap_or_else(|| panic!("no claimed scenario: {:#?}", refusals(&synthesis)));
    let mut owner_of = BTreeMap::new();
    let mut pending = None;
    let mut claim = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                match command.to_string().as_str() {
                    "mini.m.AddItem" => pending = named(input, "owner_id"),
                    "mini.m.Claim" => claim = Some(input.clone()),
                    _ => {}
                }
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Item" => {
                owner_of.insert(instance.to_string(), pending.take());
            }
            _ => {}
        }
    }
    let claim = claim.expect("the scenario claims");
    let item = named(&claim, "item_id").expect("item_id names an arranged item");
    let held = owner_of.get(&item).cloned().flatten();
    assert_eq!(
        named(&claim, "owner_id"),
        held,
        "the success names another owner than the item's: {claim:?}"
    );
}
