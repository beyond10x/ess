//! Adversary, pass 2, for a `{related: …}` value beside a `when_related:` guard over the same row
//! (beyond10x/ess#270), attacking correction 1: a second row the guard accepts (the companion),
//! arranged before the named row and holding other values in every copied field, or a
//! `Note::UnaccompaniedRelatedCopy` where none is arranged.
//!
//! The cases run the synthesized suite against a shelf implemented here with one defect switched in
//! at a time — copying from the *last* accepted row, or from the accepted row first or last by the
//! copied value — and check that the companion and the note agree with what the model allows.
#![allow(clippy::too_many_lines, clippy::struct_excessive_bools)]
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::{ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Note, Synthesis},
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
const OWNER_LINK: &str = include_str!("fixtures/related-guard-owner-link.yaml");

const STARTED: &str = "mini.m.Start/outcome/started";

fn edited(base: &str, edits: &[&[(&str, &str)]]) -> String {
    let mut out = base.to_owned();
    for (from, to) in edits.iter().flat_map(|list| list.iter()) {
        let next = out.replacen(from, to, 1);
        assert_ne!(next, out, "`{from}` is in the model");
        out = next;
    }
    out
}

fn text(edits: &[&[(&str, &str)]]) -> String {
    edited(MODEL, edits)
}

/// ess/20: the guard reads the item's lifecycle state (as pass 1's `STATE_GUARD`).
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

/// Every `note` is an enum `Tone` (Calm, Loud, Quiet), not a String.
const ENUM_NOTE: &[(&str, &str)] = &[
    (
        "  - {name: mini.m.RunId, kind: newtype, of: Uuid}\n",
        "  - {name: mini.m.RunId, kind: newtype, of: Uuid}\n  - {name: mini.m.Tone, kind: enum, variants: [Calm, Loud, Quiet]}\n",
    ),
    (
        "      - {name: note, type: String}\n      - {name: archived, type: Boolean}\n    lifecycle",
        "      - {name: note, type: mini.m.Tone}\n      - {name: archived, type: Boolean}\n    lifecycle",
    ),
    (
        "      - {name: note, type: String}\n      - {name: archived, type: Boolean}\n    outcomes",
        "      - {name: note, type: mini.m.Tone}\n      - {name: archived, type: Boolean}\n    outcomes",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n    lifecycle: {initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: mini.m.Tone}\n    lifecycle: {initial: Running",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: mini.m.Tone}\nactors",
    ),
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: mini.m.Tone}\n",
    ),
];

/// `Started` and the run also copy the item's `archived` flag — the very field the guard refuses
/// on, so every accepted item holds `false` there.
const COPY_ARCHIVED_TOO: &[(&str, &str)] = &[
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n    lifecycle: {initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n      - {name: archived, type: Boolean}\n    lifecycle: {initial: Running",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\n      - {name: archived, type: Boolean}\nactors",
    ),
    (
        "            note: {related: {via: input.item_id, field: note}}\n        sets:",
        "            note: {related: {via: input.item_id, field: note}}\n            archived: {related: {via: input.item_id, field: archived}}\n        sets:",
    ),
    (
        "          note: {related: {via: input.item_id, field: note}}\nviews",
        "          note: {related: {via: input.item_id, field: note}}\n          archived: {related: {via: input.item_id, field: archived}}\nviews",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n",
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n      - {name: archived, type: Boolean}\n",
    ),
];

/// The guard accepts only an item whose note is exactly "x": no second accepted item can hold
/// another note.
const ONLY_X_ACCEPTED: &[(&str, &str)] = &[(
    "        when_related: {via: input.item_id, predicate: archived == true}\n",
    "        when_related: {via: input.item_id, predicate: note != \"x\"}\n",
)];

/// No copy anywhere: the event and the run carry no note.
const NO_COPY: &[(&str, &str)] = &[
    (
        "      - {name: item_id, type: mini.m.ItemId}\n      - {name: note, type: String}\n    lifecycle: {initial: Running",
        "      - {name: item_id, type: mini.m.ItemId}\n    lifecycle: {initial: Running",
    ),
    (
        "      - {name: run_id, type: mini.m.RunId}\n      - {name: note, type: String}\nactors",
        "      - {name: run_id, type: mini.m.RunId}\nactors",
    ),
    (
        "            run_id: {generated: true}\n            note: {related: {via: input.item_id, field: note}}\n",
        "            run_id: {generated: true}\n",
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

/// The fields the suite's `UnaccompaniedRelatedCopy` note names for `started`, if any.
fn noted(synthesis: &Synthesis) -> Option<Vec<String>> {
    synthesis.notes.iter().find_map(|note| match note {
        Note::UnaccompaniedRelatedCopy { scenario, fields } if scenario.to_string() == STARTED => {
            Some(fields.clone())
        }
        _ => None,
    })
}

/// Which accepted item an implementation copies from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// The first item the guard accepts (pass 1's defect, the one the companion is built for).
    FirstSelecting,
    /// The last item the guard accepts: the most recently created accepted row.
    LastSelecting,
    /// The accepted item whose note sorts first.
    LeastNoteSelecting,
    /// The accepted item whose note sorts last.
    GreatestNoteSelecting,
}

type Row = BTreeMap<String, Node>;

struct Shelf {
    mutant: Mutant,
    views: BTreeMap<String, Vec<String>>,
    started: Vec<String>,
    guard_on_flag: bool,
    guard_on_state: bool,
    guard_only_x: bool,
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
        let started = ir
            .events()
            .values()
            .find(|event| event.name.to_string() == "mini.m.Started")
            .map(|event| {
                event
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .filter(|name| name != "run_id")
                    .collect()
            })
            .unwrap_or_default();
        Self {
            mutant,
            views,
            started,
            guard_on_flag: text.contains("predicate: archived == true"),
            guard_on_state: text.contains("predicate: state == Archived"),
            guard_only_x: text.contains("predicate: note != \"x\""),
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

    fn accepted(&self, item: &Row) -> bool {
        let refused = if self.guard_only_x {
            item.get("note") != Some(&Node::Text("x".into()))
        } else if self.guard_on_state {
            item.get("state") == Some(&Node::Text("Archived".into()))
        } else {
            self.guard_on_flag && item.get("archived") == Some(&Node::Bool(true))
        };
        !refused
    }
}

fn sort_key(row: &Row) -> String {
    format!("{:?}", row.get("note"))
}

impl ConformanceTarget for Shelf {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shelf-adversary-270-p2", "1"))
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
                let items = self.items.borrow();
                let Some(item) = items.iter().find(|row| row["item_id"] == input["item_id"]) else {
                    return Ok(refused("no-item", "mini.m.NoItem").with_consistency(self.token()));
                };
                if !self.accepted(item) {
                    return Ok(
                        refused("archived", "mini.m.Archived").with_consistency(self.token())
                    );
                }
                let accepted: Vec<&Row> = items.iter().filter(|row| self.accepted(row)).collect();
                let source: &Row = match self.mutant {
                    Mutant::None => item,
                    Mutant::FirstSelecting => accepted[0],
                    Mutant::LastSelecting => accepted[accepted.len() - 1],
                    Mutant::LeastNoteSelecting => {
                        accepted.iter().min_by_key(|row| sort_key(row)).unwrap()
                    }
                    Mutant::GreatestNoteSelecting => {
                        accepted.iter().max_by_key(|row| sort_key(row)).unwrap()
                    }
                };
                let id = self.mint();
                let mut run = Row::new();
                run.insert("run_id".into(), id.clone());
                run.insert("item_id".into(), input["item_id"].clone());
                run.insert("state".into(), Node::Text("Running".into()));
                let mut started = event("Started").with("run_id", id);
                for field in &self.started {
                    let value = source.get(field).cloned().unwrap_or(Node::Null);
                    started = started.with(field, value);
                }
                for field in ["note", "archived"] {
                    if let Some(value) = source.get(field) {
                        run.insert(field.into(), value.clone());
                    }
                }
                drop(items);
                self.runs.borrow_mut().push(run);
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
        let table = match view.as_str() {
            "mini.m.Items" => &self.items,
            "mini.m.Runs" => &self.runs,
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(
            table
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

// ---- 1. companion ordering --------------------------------------------------------------------

/// The companion is created before the named row and the decoy after it selects another branch,
/// so the named row is the *last* accepted row: copying from the last accepted row passes.
#[test]
fn adv270_p2_a_copy_from_the_last_accepted_row_is_killed() {
    kills(MODEL, &[Mutant::LastSelecting]);
}

/// The same under the ess/20 lifecycle-state guard.
#[test]
fn adv270_p2_a_copy_from_the_last_state_accepted_row_is_killed() {
    kills(&text(&[STATE_GUARD]), &[Mutant::LastSelecting]);
}

/// With two accepted rows, the named one is either least or greatest by note: one of the two
/// sort-order mutants copies from it.
#[test]
fn adv270_p2_a_copy_from_the_accepted_row_first_or_last_by_note_is_killed() {
    kills(
        MODEL,
        &[Mutant::LeastNoteSelecting, Mutant::GreatestNoteSelecting],
    );
}

// ---- 2. companion values ----------------------------------------------------------------------

/// An enum-typed copied field: the companion holds another variant.
#[test]
fn adv270_p2_an_enum_copy_kills_a_copy_from_the_first_accepted_row() {
    kills(&text(&[ENUM_NOTE]), &[Mutant::FirstSelecting]);
}

/// The branch copies `note` and `archived`; every accepted row holds `archived: false`, so no row
/// differs in *both*, and correction 1 then arranges no companion at all — although one differing
/// in `note` alone would fail a target copying the whole of another accepted row.
#[test]
fn adv270_p2_a_copy_including_a_guard_fixed_field_still_kills_a_copy_from_the_first_accepted_row() {
    kills(&text(&[COPY_ARCHIVED_TOO]), &[Mutant::FirstSelecting]);
}

// ---- 3. the note, never missing and never spurious --------------------------------------------

/// Where a companion is impossible — the guard accepts only one note — the note is emitted.
#[test]
fn adv270_p2_a_guard_accepting_one_value_only_is_noted() {
    let synthesis = compiled(&text(&[ONLY_X_ACCEPTED]));
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == STARTED),
        "{:#?}",
        refusals(&synthesis)
    );
    assert_eq!(noted(&synthesis), Some(vec!["note".to_owned()]));
}

/// Where the companion is arranged, or nothing is copied, no note.
#[test]
fn adv270_p2_no_note_where_a_companion_exists_or_nothing_is_copied() {
    for model in [
        MODEL.to_owned(),
        text(&[STATE_GUARD]),
        text(&[ENUM_NOTE]),
        text(&[NO_COPY]),
    ] {
        let synthesis = compiled(&model);
        assert_eq!(noted(&synthesis), None, "{model}");
    }
}

/// The note agrees with the suite: present exactly where a target copying from the first accepted
/// row survives.
#[test]
fn adv270_p2_the_note_is_present_exactly_where_the_first_accepted_row_survives() {
    for model in [
        MODEL.to_owned(),
        text(&[STATE_GUARD]),
        text(&[ENUM_NOTE]),
        text(&[COPY_ARCHIVED_TOO]),
    ] {
        let synthesis = compiled(&model);
        let survives = failing(&model, Mutant::FirstSelecting).is_empty();
        assert_eq!(noted(&synthesis).is_some(), survives, "{model}");
    }
}

/// `Started` carries the note of the item `Start` is guarded on (as pass 1's `LINK_NOTE`).
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

/// The owner-link guard accepts any item whose owner is the one `Start` names. A second item added
/// under that same owner with another note is a plain `AddItem` away — `AddItem` takes the owner
/// as input — so a companion is constructible; correction 1 arranges none and notes the gap.
#[test]
fn adv270_p2_the_owner_link_guard_gets_a_companion_under_the_named_owner() {
    let synthesis = compiled(&edited(OWNER_LINK, &[LINK_NOTE]));
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == STARTED)
        .map_or_else(
            || panic!("no {STARTED}: {:#?}", refusals(&synthesis)),
            |(_, scenario)| scenario,
        );
    // (instance, owner instance, note) per item, in creation order.
    let mut sent = None;
    let mut items = Vec::new();
    let mut named = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                match command.to_string().as_str() {
                    "mini.m.AddItem" => {
                        let owner = match input.get("owner_id") {
                            Some(ScenarioValue::Instance { instance }) => instance.to_string(),
                            other => format!("{other:?}"),
                        };
                        let note = match input.get("note") {
                            Some(ScenarioValue::Literal { value }) => value.clone(),
                            other => panic!("a literal note: {other:?}"),
                        };
                        sent = Some((owner, note));
                    }
                    "mini.m.Start" => {
                        if let Some(ScenarioValue::Instance { instance }) = input.get("item_id") {
                            named = Some(instance.to_string());
                        }
                    }
                    _ => {}
                }
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Item" => {
                let (owner, note) = sent.take().expect("an AddItem");
                items.push((instance.to_string(), owner, note));
            }
            _ => {}
        }
    }
    let named = named.expect("Start names an item");
    let (_, owner, note) = items
        .iter()
        .find(|(instance, _, _)| *instance == named)
        .cloned()
        .expect("the named item is arranged");
    assert!(
        items
            .iter()
            .any(|(instance, other_owner, other_note)| *instance != named
                && *other_owner == owner
                && *other_note != note),
        "no second item under the named owner {owner} with another note; note: {:?}; items: {items:#?}",
        noted(&synthesis)
    );
}
