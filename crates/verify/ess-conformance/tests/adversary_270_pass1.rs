//! Adversary, pass 1, for a `{related: …}` value beside a `when_related:` guard over the same row
//! (beyond10x/ess#270).
//!
//! Each case runs the synthesized suite against a shelf implemented here, with one defect switched
//! in at a time: a suite witnessing the copied value passes the shelf without a defect and fails
//! every defect that copies from another row, or from none. The variants move the copy (payload
//! only, `sets` only), its type (Optional, lifted into Optional), add a second `{related:}` read
//! through another input, a second guard on the same input, and a lifecycle-state guard (ess/20,
//! beyond10x/ess#229).
#![allow(clippy::too_many_lines, clippy::struct_excessive_bools)]
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

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

const MODEL: &str = include_str!("fixtures/related-guard-copied-value.yaml");

const STARTED: &str = "mini.m.Start/outcome/started";

/// The fixture with each edit applied once, in order; every edit must match.
fn text(edits: &[(&str, &str)]) -> String {
    let mut out = MODEL.to_owned();
    for (from, to) in edits {
        let next = out.replacen(from, to, 1);
        assert_ne!(next, out, "`{from}` is in the model");
        out = next;
    }
    out
}

/// The copy reaches the event only: the run stores no note.
const PAYLOAD_ONLY: &[(&str, &str)] = &[
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n    lifecycle: {initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n    lifecycle: {initial: Running",
    ),
    (
        "          item_id: input.item_id\n          note: {related: {via: input.item_id, field: note}}\n",
        "          item_id: input.item_id\n",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n",
        "      - {name: item_id, type: mini.m.ItemId}\n",
    ),
];

/// The copy reaches the run only: `Started` carries no note.
const SETS_ONLY: &[(&str, &str)] = &[
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\nactors",
    ),
    (
        "            run_id: {generated: true}\n            note: {related: {via: input.item_id, field: note}}\n",
        "            run_id: {generated: true}\n",
    ),
];

/// The note is optional everywhere.
const OPTIONAL: &[(&str, &str)] = &[
    (
        "      - {name: note, type: String}\n      - {name: archived, type: Boolean}\n    lifecycle",
        "      - {name: note, type: Optional<String>}\n      - {name: archived, type: Boolean}\n    lifecycle",
    ),
    (
        "      - {name: note, type: String}\n      - {name: archived, type: Boolean}\n    outcomes",
        "      - {name: note, type: Optional<String>}\n      - {name: archived, type: Boolean}\n    outcomes",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n    lifecycle: {initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: Optional<String>}\n    lifecycle: {initial: Running",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: Optional<String>}\nactors",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: Optional<String>}\n",
    ),
];

/// The item's note is required; the run, the event and the view hold it as Optional: the copy is
/// lifted.
const LIFTED: &[(&str, &str)] = &[
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n    lifecycle: {initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: Optional<String>}\n    lifecycle: {initial: Running",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: Optional<String>}\nactors",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: Optional<String>}\n",
    ),
];

/// `Start` also names a second item, `other_id`, unguarded, and `Started` carries its note as
/// `other_note`: a `{related:}` read through another input, beside the guarded one.
const TWO_READS: &[(&str, &str)] = &[
    (
        "  - name: mini.m.Start\n    input:\n      - {name: item_id, type: mini.m.ItemId}\n",
        "  - name: mini.m.Start\n    input:\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: other_id, type: mini.m.ItemId}\n",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\n      - {name: other_note, type: String}\nactors",
    ),
    (
        "            note: {related: {via: input.item_id, field: note}}\n        sets:",
        "            note: {related: {via: input.item_id, field: note}}\n            other_note: {related: {via: input.other_id, field: note}}\n        sets:",
    ),
];

/// A second guard on the same input: `paused` items refuse too.
const TWO_GUARDS: &[(&str, &str)] = &[
    (
        "      - {name: archived, type: Boolean}\n    lifecycle",
        "      - {name: archived, type: Boolean}\n      - {name: paused, type: Boolean}\n    lifecycle",
    ),
    (
        "      - {name: archived, type: Boolean}\n    outcomes",
        "      - {name: archived, type: Boolean}\n      - {name: paused, type: Boolean}\n    outcomes",
    ),
    (
        "          archived: input.archived\n",
        "          archived: input.archived\n          paused: input.paused\n",
    ),
    (
        "  - {name: mini.m.Archived, summary: The item is archived., fields: []}\n",
        "  - {name: mini.m.Archived, summary: The item is archived., fields: []}\n  - {name: mini.m.Paused, summary: The item is paused., fields: []}\n",
    ),
    (
        "      - name: started\n",
        "      - name: paused\n        when_related: {via: input.item_id, predicate: paused == true}\n        error: mini.m.Paused\n      - name: started\n",
    ),
];

/// The guard reads the copied field itself: an item with an empty note is refused.
const GUARD_ON_COPIED: &[(&str, &str)] = &[(
    "        when_related: {via: input.item_id, predicate: archived == true}\n",
    "        when_related: {via: input.item_id, predicate: note == \"\"}\n",
)];

/// ess/20: the item's lifecycle has an `Archived` state reached by `ArchiveItem`, and the guard
/// reads the state, not the flag (beyond10x/ess#229).
const STATE_GUARD: &[(&str, &str)] = &[
    ("format: ess/18\n", "format: ess/20\n"),
    (
        "    lifecycle: {initial: Listed, states: [Listed], terminal: [Listed]}\n",
        "    lifecycle:\n      initial: Listed\n      states: [Listed, Archived]\n      terminal: [Archived]\n      transitions:\n        - {name: archive, from: [Listed], to: Archived}\n",
    ),
    (
        "  - name: mini.m.Started\n",
        "  - name: mini.m.ItemArchived\n    fields:\n      - {name: item_id, type: mini.m.ItemId}\n  - name: mini.m.Started\n",
    ),
    (
        "    may: [mini.m.AddItem, mini.m.Start]\n",
        "    may: [mini.m.AddItem, mini.m.ArchiveItem, mini.m.Start]\n",
    ),
    (
        "  - name: mini.m.Start\n",
        "  - name: mini.m.ArchiveItem\n    input:\n      - {name: item_id, type: mini.m.ItemId}\n    outcomes:\n      - name: archived-item\n        moves: mini.m.Item.archive\n        instance: item_id\n        emits: [mini.m.ItemArchived]\n        payload: {mini.m.ItemArchived: {item_id: input.item_id}}\n      - {name: already-archived, wrong_state: true, error: mini.m.Archived}\n  - name: mini.m.Start\n",
    ),
    (
        "        when_related: {via: input.item_id, predicate: archived == true}\n",
        "        when_related: {via: input.item_id, predicate: state == Archived}\n",
    ),
    (
        "views:\n",
        "views:\n  - name: mini.m.Items\n    source: mini.m.Item\n    consistency: read_your_writes\n    fields:\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: state, type: mini.m.Item.State}\n",
    ),
];

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

/// One defect an implementation copying the guarded row's note could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Copies the note of the first item ever added.
    FirstItem,
    /// Copies the note of the last item added.
    LastItem,
    /// Copies nothing: publishes an empty note.
    NoCopy,
    /// Copies the note of the first item the guard would take `started` on, not the one named.
    FirstSelecting,
    /// Copies the note of the item `other_id` names (two reads only).
    OtherInput,
    /// Copies the right note into the event, and the first item's note into the run.
    RunFromFirst,
}

type Row = BTreeMap<String, Node>;

/// The shelf, implemented here and not by the synthesizer.
struct Shelf {
    mutant: Mutant,
    views: BTreeMap<String, Vec<String>>,
    events: BTreeMap<String, BTreeSet<String>>,
    sets_note: bool,
    guard_on_note: bool,
    guard_on_state: bool,
    guard_on_flag: bool,
    items: RefCell<Vec<Row>>,
    runs: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Shelf {
    fn new(text: &str, mutant: Mutant) -> Self {
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
        let events = ir
            .events()
            .values()
            .map(|event| {
                (
                    event.name.to_string(),
                    event
                        .fields
                        .iter()
                        .map(|field| field.name.clone())
                        .collect(),
                )
            })
            .collect();
        Self {
            mutant,
            views,
            events,
            guard_on_note: text.contains("predicate: note == \"\""),
            guard_on_state: text.contains("predicate: state == Archived"),
            guard_on_flag: text.contains("predicate: archived == true"),
            sets_note: text
                .contains("          note: {related: {via: input.item_id, field: note}}\n"),
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

    /// The branch `Start` takes on `item`, by the guards the model may declare.
    fn refusal(&self, item: &Row) -> Option<(&'static str, &'static str)> {
        let archived = if self.guard_on_note {
            item.get("note") == Some(&Node::Text(String::new()))
        } else if self.guard_on_state {
            item.get("state") == Some(&Node::Text("Archived".into()))
        } else {
            self.guard_on_flag && item.get("archived") == Some(&Node::Bool(true))
        };
        if archived {
            return Some(("archived", "mini.m.Archived"));
        }
        if item.get("paused") == Some(&Node::Bool(true)) {
            return Some(("paused", "mini.m.Paused"));
        }
        None
    }

    fn table(&self, entity: &str) -> &RefCell<Vec<Row>> {
        match entity {
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
        Ok(ImplementationIdentity::new("shelf-adversary-270", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.items.replace(Vec::new());
        self.runs.replace(Vec::new());
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
            "mini.m.AddItem" => {
                let id = self.mint();
                let mut row = input.clone();
                row.insert("item_id".into(), id.clone());
                row.insert("state".into(), Node::Text("Listed".into()));
                self.items.borrow_mut().push(row);
                took("added").emitting(event("ItemAdded").with("item_id", id))
            }
            "mini.m.ArchiveItem" => {
                let id = input["item_id"].clone();
                let mut items = self.items.borrow_mut();
                let row = items.iter_mut().find(|row| row["item_id"] == id);
                let Some(row) = row.filter(|row| row["state"] == Node::Text("Listed".into()))
                else {
                    return Ok(refused("already-archived", "mini.m.Archived")
                        .with_consistency(self.token()));
                };
                row.insert("state".into(), Node::Text("Archived".into()));
                took("archived-item").emitting(event("ItemArchived").with("item_id", id))
            }
            "mini.m.Start" => {
                let Some(item) = find(&self.items, "item_id", &input["item_id"]) else {
                    return Ok(refused("no-item", "mini.m.NoItem").with_consistency(self.token()));
                };
                if let Some((name, error)) = self.refusal(&item) {
                    return Ok(refused(name, error).with_consistency(self.token()));
                }
                let items = self.items.borrow();
                let note_of = |row: Option<&Row>| row.and_then(|row| row.get("note").cloned());
                let right = item.get("note").cloned();
                let other = input
                    .get("other_id")
                    .and_then(|other| items.iter().find(|row| &row["item_id"] == other));
                let copied = match self.mutant {
                    Mutant::None | Mutant::RunFromFirst => right.clone(),
                    Mutant::FirstItem => note_of(items.first()),
                    Mutant::LastItem => note_of(items.last()),
                    Mutant::NoCopy => Some(Node::Text(String::new())),
                    Mutant::FirstSelecting => {
                        note_of(items.iter().find(|row| self.refusal(row).is_none()))
                    }
                    Mutant::OtherInput => note_of(other),
                };
                let stored = match self.mutant {
                    Mutant::RunFromFirst => note_of(items.first()),
                    _ => copied.clone(),
                };
                let id = self.mint();
                let mut row = Row::new();
                row.insert("run_id".into(), id.clone());
                row.insert("item_id".into(), input["item_id"].clone());
                row.insert("state".into(), Node::Text("Running".into()));
                if self.sets_note {
                    row.insert("note".into(), stored.unwrap_or(Node::Null));
                }
                self.runs.borrow_mut().push(row);
                let declared = &self.events["mini.m.Started"];
                let mut started = event("Started").with("run_id", id);
                if declared.contains("note") {
                    started = started.with("note", copied.unwrap_or(Node::Null));
                }
                if declared.contains("other_note") {
                    started = started.with("other_note", note_of(other).unwrap_or(Node::Null));
                }
                took("started").emitting(started)
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
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(
            self.table(entity)
                .borrow()
                .iter()
                .map(|row| {
                    fields
                        .iter()
                        .map(|field| (field.clone(), row.get(field).cloned().unwrap_or(Node::Null)))
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
/// Any refusal fails the case first, and the success scenario must exist.
fn failing(text: &str, mutant: Mutant) -> BTreeMap<String, String> {
    let synthesis = compiled(text);
    let refused = refusals(&synthesis);
    assert!(refused.is_empty(), "refused: {refused:#?}");
    let suite = synthesis.suite;
    assert!(
        suite.scenarios.keys().any(|id| id.to_string() == STARTED),
        "no {STARTED}"
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Shelf::new(text, mutant))
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
fn kills(text: &str, mutants: &[Mutant]) {
    assert_eq!(
        failing(text, Mutant::None),
        BTreeMap::new(),
        "a correct shelf fails"
    );
    let survivors: Vec<Mutant> = mutants
        .iter()
        .copied()
        .filter(|mutant| failing(text, *mutant).is_empty())
        .collect();
    assert_eq!(
        survivors,
        Vec::<Mutant>::new(),
        "defects that pass every scenario"
    );
}

const COPY_DEFECTS: &[Mutant] = &[Mutant::FirstItem, Mutant::LastItem, Mutant::NoCopy];

#[test]
fn adv270_p1_the_fixture_kills_every_copy_defect() {
    let mut mutants = COPY_DEFECTS.to_vec();
    mutants.push(Mutant::RunFromFirst);
    kills(MODEL, &mutants);
}

#[test]
fn adv270_p1_a_payload_only_copy_kills_every_copy_defect() {
    kills(&text(PAYLOAD_ONLY), COPY_DEFECTS);
}

#[test]
fn adv270_p1_a_sets_only_copy_kills_every_copy_defect() {
    kills(&text(SETS_ONLY), COPY_DEFECTS);
}

#[test]
fn adv270_p1_an_optional_copy_kills_every_copy_defect() {
    kills(&text(OPTIONAL), COPY_DEFECTS);
}

#[test]
fn adv270_p1_a_lifted_copy_kills_every_copy_defect() {
    kills(&text(LIFTED), COPY_DEFECTS);
}

#[test]
fn adv270_p1_a_second_read_through_another_input_kills_every_copy_defect() {
    let mut mutants = COPY_DEFECTS.to_vec();
    mutants.push(Mutant::OtherInput);
    kills(&text(TWO_READS), &mutants);
}

#[test]
fn adv270_p1_two_guards_on_one_input_kill_every_copy_defect() {
    kills(&text(TWO_GUARDS), COPY_DEFECTS);
}

#[test]
fn adv270_p1_a_guard_on_the_copied_field_kills_every_copy_defect() {
    kills(&text(GUARD_ON_COPIED), COPY_DEFECTS);
}

/// Control for the case above: the same guard on `note`, with no copy anywhere.
#[test]
fn adv270_p1_control_a_guard_on_the_field_without_a_copy_passes_a_correct_shelf() {
    let edits: Vec<(&str, &str)> = PAYLOAD_ONLY
        .iter()
        .chain(SETS_ONLY.iter())
        .chain(GUARD_ON_COPIED.iter())
        .copied()
        .collect();
    assert_eq!(failing(&text(&edits), Mutant::None), BTreeMap::new());
}

#[test]
fn adv270_p1_a_state_guard_kills_every_copy_defect() {
    kills(&text(STATE_GUARD), COPY_DEFECTS);
}

/// Only the `exists: false` guard: no row selects another branch, so the decoys are plain rows.
#[test]
fn adv270_p1_an_existence_guard_alone_kills_every_copy_defect() {
    let edits: &[(&str, &str)] = &[(
        "      - name: archived\n        when_related: {via: input.item_id, predicate: archived == true}\n        error: mini.m.Archived\n",
        "",
    )];
    let mut mutants = COPY_DEFECTS.to_vec();
    mutants.push(Mutant::FirstSelecting);
    kills(&text(edits), &mutants);
}

/// Every row beside the named one is a decoy selecting another branch, so the named row is the
/// only one in the scenario `started` would be taken on. A target that copies the note of "an item
/// the guard accepts" — not the item the input names — publishes the same note and passes. A row
/// beside the named one that also selects `started`, holding another note, would fail it.
#[test]
fn adv270_p1_a_copy_from_another_row_the_guard_accepts_is_killed() {
    kills(MODEL, &[Mutant::FirstSelecting]);
}

/// The same, under the lifecycle-state guard (ess/20, beyond10x/ess#229).
#[test]
fn adv270_p1_a_copy_from_another_row_the_state_guard_accepts_is_killed() {
    kills(&text(STATE_GUARD), &[Mutant::FirstSelecting]);
}

// ---- beyond10x/ess#271: a guard comparing the row's owner link with an input owner -----------

const OWNER_LINK: &str = include_str!("fixtures/related-guard-owner-link.yaml");

/// `Started` carries the note of the item `Start` is guarded on.
const LINK_NOTE: &[(&str, &str)] = &[
    (
        "      - {name: run_id, type: mini.m.RunId}\n  - name: mini.m.RunStopped\n",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\n  - name: mini.m.RunStopped\n",
    ),
    (
        "          mini.m.Started: {run_id: {generated: true}}\n",
        "          mini.m.Started:\n            run_id: {generated: true}\n            note: {related: {via: input.item_id, field: note}}\n",
    ),
];

/// `Owner` owns `Run` too and `Start` files the run under the owner it names: the arrangement pins
/// `owner_id` (beyond10x/ess#271 `link_pins`).
const LINK_RUN_OWNED: &[(&str, &str)] = &[
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
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: item_id, type: mini.m.ItemId}\n",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: owner_id, type: mini.m.OwnerId}\n",
    ),
];

fn edited(base: &str, edits: &[&[(&str, &str)]]) -> String {
    let mut out = base.to_owned();
    for (from, to) in edits.iter().flat_map(|list| list.iter()) {
        let next = out.replacen(from, to, 1);
        assert_ne!(next, out, "`{from}` is in the model");
        out = next;
    }
    out
}

/// The note the `started` scenario's `AddItem` gave the item `Start` names, the note its `Started`
/// expectation asserts, and the notes of every other item it adds.
fn witnessed_note(text: &str) -> (Node, Node, Vec<Node>) {
    use ess_conformance::{scenario::ScenarioStep, ScenarioValue};
    let synthesis = compiled(text);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == STARTED)
        })
        .map(ToString::to_string)
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == STARTED)
        .map_or_else(
            || panic!("no {STARTED}: {:#?}", refusals(&synthesis)),
            |(_, scenario)| scenario,
        );
    let mut sent = None;
    let mut items = BTreeMap::new();
    let mut named = None;
    let mut asserted = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                match command.to_string().as_str() {
                    "mini.m.AddItem" => match input.get("note") {
                        Some(ScenarioValue::Literal { value }) => sent = Some(value.clone()),
                        other => panic!("a literal note: {other:?}"),
                    },
                    "mini.m.Start" => match input.get("item_id") {
                        Some(ScenarioValue::Instance { instance }) => {
                            named = Some(instance.to_string());
                        }
                        other => panic!("Start names an item: {other:?}"),
                    },
                    _ => {}
                }
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Item" => {
                items.insert(instance.to_string(), sent.take().expect("an AddItem"));
            }
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "mini.m.Started" =>
            {
                asserted = payload.get("note").cloned();
            }
            _ => {}
        }
    }
    let named = named.expect("Start is sent");
    let note = items
        .get(&named)
        .cloned()
        .unwrap_or_else(|| panic!("{named} is arranged: {items:#?}"));
    let others = items
        .iter()
        .filter(|(instance, _)| **instance != named)
        .map(|(_, note)| note.clone())
        .collect();
    (
        note,
        asserted.unwrap_or_else(|| panic!("`Started` asserts a note: {scenario:#?}")),
        others,
    )
}

/// The owner-link guard with a copied note: the expectation carries the named item's note, and no
/// other item holds it.
#[test]
fn adv270_p1_an_owner_link_guard_carries_the_named_items_note() {
    let (note, asserted, others) = witnessed_note(&edited(OWNER_LINK, &[LINK_NOTE]));
    assert_eq!(asserted, note);
    assert!(!others.is_empty(), "decoys are arranged");
    assert!(
        !others.contains(&note),
        "{note:?} is held by another item: {others:#?}"
    );
}

/// The same with the owner pinned by the run's own relation (beyond10x/ess#271 `link_pins`).
#[test]
fn adv270_p1_a_pinned_owner_link_guard_carries_the_named_items_note() {
    let (note, asserted, others) =
        witnessed_note(&edited(OWNER_LINK, &[LINK_RUN_OWNED, LINK_NOTE]));
    assert_eq!(asserted, note);
    assert!(!others.is_empty(), "decoys are arranged");
    assert!(
        !others.contains(&note),
        "{note:?} is held by another item: {others:#?}"
    );
}
