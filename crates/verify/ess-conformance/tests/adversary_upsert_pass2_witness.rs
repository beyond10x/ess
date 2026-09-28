//! Adversary, pass 2, for the two-call witness of selection by existence (ess/16,
//! beyond10x/ess#164, `docs/design/outcome-shapes.md`, "Conformance" and "Known limits").
//!
//! One in-memory target over a model of its own: an upsert (`PutItem`), a create-or-refuse
//! (`BookSlot`) and an ess/15 refusal for an unknown identity on the same entity (`RetireItem`).
//! Each case either runs a correct target the design note says passes, or a wrong one the note
//! says the suite catches.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/16
system: demo
version: v1
domain: demo.items
types:
  - {name: demo.items.ItemId, kind: newtype, of: String}
  - {name: demo.items.Label, kind: newtype, of: String}
entities:
  - name: demo.items.Item
    identity: {name: item_id, type: demo.items.ItemId}
    fields:
      - {name: label, type: demo.items.Label}
    lifecycle:
      initial: Active
      states: [Active, Retired]
      terminal: [Retired]
      transitions:
        - {name: retire, from: [Active], to: Retired}
  - name: demo.items.Slot
    identity: {name: slot_id, type: demo.items.ItemId}
    fields:
      - {name: label, type: demo.items.Label}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
events:
  - name: demo.items.ItemStored
    fields:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
  - name: demo.items.ItemRetired
    fields:
      - {name: item_id, type: demo.items.ItemId}
  - name: demo.items.SlotBooked
    fields:
      - {name: slot_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
errors:
  - name: demo.items.SlotTaken
    summary: A slot with this id is already booked.
    fields: []
  - name: demo.items.BadLabel
    summary: The label is not accepted.
    fields: []
  - name: demo.items.NoSuchItem
    summary: No item carries this id.
    fields: []
actors:
  - {name: demo.items.Admin, may: [demo.items.PutItem, demo.items.BookSlot, demo.items.RetireItem]}
commands:
  - name: demo.items.PutItem
    input:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
    outcomes:
PUT_EXTRA      - name: updated
        updates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
      - name: created
        unknown_instance: true
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
  - name: demo.items.BookSlot
    input:
      - {name: slot_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
    outcomes:
BOOK_EXTRA      - name: booked
        creates: demo.items.Slot
        instance: slot_id
        emits: [demo.items.SlotBooked]
        payload:
          demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}
        sets:
          label: input.label
      - {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}
RETIRE
views:
  - name: demo.items.ItemDetails
    source: demo.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
  - name: demo.items.SlotDetails
    source: demo.items.Slot
    consistency: read_your_writes
    fields:
      - {name: slot_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
";

const RETIRE: &str = "  - name: demo.items.RetireItem
    input:
      - {name: item_id, type: demo.items.ItemId}
    outcomes:
      - name: retired
        moves: demo.items.Item.retire
        instance: item_id
        emits: [demo.items.ItemRetired]
        payload:
          demo.items.ItemRetired: {item_id: input.item_id}
      - {name: missing, unknown_instance: true, error: demo.items.NoSuchItem}
";

const REJECTED: &str =
    "      - {name: rejected, when: label == \"bad\", error: demo.items.BadLabel}\n";

struct Model {
    put: &'static str,
    book: &'static str,
}

impl Model {
    const PLAIN: Self = Self { put: "", book: "" };

    fn text(&self) -> String {
        MODEL
            .replace("PUT_EXTRA", self.put)
            .replace("BOOK_EXTRA", self.book)
            .replace("RETIRE\n", RETIRE)
    }
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("adversary-upsert-pass2.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

// ---- the target ------------------------------------------------------------------------------------

/// How the target answers a label `bad` beside selection by existence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Precedence {
    /// The input-guarded refusal first, whatever is stored: the coordinator's decision (correction
    /// round 1), stated in `docs/design/outcome-shapes.md` "Precedence".
    InputFirst,
    /// Existence first: a stored identity is answered by the update / `already-booked` whatever
    /// the label; the refusal is checked only for an identity nothing stored.
    ExistenceFirst,
}

struct Store {
    precedence: Precedence,
    /// Rows survive `begin_scenario`: the target the scenarios share.
    shared: bool,
    /// The model declares `booked-express` for `label == "express"` beside the default `booked`.
    express: bool,
    /// Wrong: the default creating path of `BookSlot` never looks for a stored slot, and books it
    /// again.
    default_skips_existence: bool,
    /// (id, label, state)
    items: RefCell<Vec<(String, String, String)>>,
    slots: RefCell<Vec<(String, String)>>,
}

fn key(input: &BTreeMap<String, Node>, field: &str) -> String {
    input
        .get(field)
        .map(|node| {
            node.as_text()
                .map_or_else(|| format!("{node:?}"), ToOwned::to_owned)
        })
        .unwrap_or_default()
}

impl Store {
    fn new(precedence: Precedence) -> Self {
        Self {
            precedence,
            shared: false,
            express: false,
            default_skips_existence: false,
            items: RefCell::new(Vec::new()),
            slots: RefCell::new(Vec::new()),
        }
    }

    fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
        let mut result =
            SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        result
    }

    fn error(command: &CommandRef, outcome: &str, error: &str) -> SemanticCommandResult {
        let mut result = Self::took(command, outcome);
        result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
        result
    }

    fn event(event: &str, fields: &[(&str, &str)]) -> ObservedEvent {
        let mut observed = ObservedEvent::new(event.parse().unwrap());
        for (name, value) in fields {
            observed
                .payload
                .insert((*name).to_owned(), Node::Text((*value).to_owned()));
        }
        observed
    }

    fn put(&self, command: &CommandRef, id: &str, label: &str) -> SemanticCommandResult {
        let rejects = label == "bad" && command_declares_rejection(self, "put");
        let mut items = self.items.borrow_mut();
        let existing = items.iter().position(|(held, _, _)| held == id);
        if rejects && (self.precedence == Precedence::InputFirst || existing.is_none()) {
            return Self::error(command, "rejected", "demo.items.BadLabel");
        }
        let outcome = if let Some(at) = existing {
            label.clone_into(&mut items[at].1);
            "updated"
        } else {
            items.push((id.to_owned(), label.to_owned(), "Active".to_owned()));
            "created"
        };
        let mut result = Self::took(command, outcome);
        result.direct_events.push(Self::event(
            "demo.items.ItemStored",
            &[("item_id", id), ("label", label)],
        ));
        result
    }

    fn book(&self, command: &CommandRef, id: &str, label: &str) -> SemanticCommandResult {
        let rejects = label == "bad" && command_declares_rejection(self, "book");
        let mut slots = self.slots.borrow_mut();
        let exists = slots.iter().any(|(held, _)| held == id);
        if rejects && (self.precedence == Precedence::InputFirst || !exists) {
            return Self::error(command, "rejected", "demo.items.BadLabel");
        }
        let express = self.express && label == "express";
        if exists && (express || !self.default_skips_existence) {
            return Self::error(command, "already-booked", "demo.items.SlotTaken");
        }
        slots.push((id.to_owned(), label.to_owned()));
        let mut result = Self::took(command, if express { "booked-express" } else { "booked" });
        result.direct_events.push(Self::event(
            "demo.items.SlotBooked",
            &[("slot_id", id), ("label", label)],
        ));
        result
    }

    fn retire(&self, command: &CommandRef, id: &str) -> SemanticCommandResult {
        let mut items = self.items.borrow_mut();
        let Some(at) = items.iter().position(|(held, _, _)| held == id) else {
            return Self::error(command, "missing", "demo.items.NoSuchItem");
        };
        if items[at].2 != "Active" {
            return SemanticCommandResult::undeclared();
        }
        "Retired".clone_into(&mut items[at].2);
        let mut result = Self::took(command, "retired");
        result
            .direct_events
            .push(Self::event("demo.items.ItemRetired", &[("item_id", id)]));
        result
    }
}

thread_local! {
    static REJECTING: RefCell<(bool, bool)> = const { RefCell::new((false, false)) };
}

fn command_declares_rejection(_: &Store, which: &str) -> bool {
    REJECTING.with(|flags| {
        let (put, book) = *flags.borrow();
        if which == "put" {
            put
        } else {
            book
        }
    })
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-upsert-pass2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        if !self.shared {
            self.items.borrow_mut().clear();
            self.slots.borrow_mut().clear();
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
        let label = key(&request.input, "label");
        Ok(match command.to_string().as_str() {
            "demo.items.PutItem" => self.put(&command, &key(&request.input, "item_id"), &label),
            "demo.items.BookSlot" => self.book(&command, &key(&request.input, "slot_id"), &label),
            "demo.items.RetireItem" => self.retire(&command, &key(&request.input, "item_id")),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.items.ItemDetails" => self
                .items
                .borrow()
                .iter()
                .map(|(id, label, state)| {
                    BTreeMap::from([
                        ("item_id".to_owned(), Node::Text(id.clone())),
                        ("label".to_owned(), Node::Text(label.clone())),
                        ("state".to_owned(), Node::Text(state.clone())),
                    ])
                })
                .collect(),
            "demo.items.SlotDetails" => self
                .slots
                .borrow()
                .iter()
                .map(|(id, label)| {
                    BTreeMap::from([
                        ("slot_id".to_owned(), Node::Text(id.clone())),
                        ("label".to_owned(), Node::Text(label.clone())),
                        ("state".to_owned(), Node::Text("Open".to_owned())),
                    ])
                })
                .collect(),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult { rows, total: None })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "none declared"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
}

fn suite(model: &Model) -> (ConformanceSuite, Vec<String>) {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model.text()));
    let refusals = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .collect();
    (synthesis.suite, refusals)
}

fn run(model: &Model, target: &Store) -> BTreeMap<String, Status> {
    REJECTING.with(|flags| {
        *flags.borrow_mut() = (!model.put.is_empty(), !model.book.is_empty());
    });
    let (suite, _) = suite(model);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

// ---- the precedence decision, both halves ----------------------------------------------------------

/// `docs/design/outcome-shapes.md`, "Precedence": an input-guarded refusal is answered before
/// selection by existence "in both forms ... A request a declared refusal claims by its input is
/// refused whether or not a record carries the identity". Known limits: "an existence-first target
/// fails it". For create-or-refuse no scenario sends the refused input for an identity a record
/// carries, and the refusal's own scenario sends an identity nothing stored, so a target that
/// answers `already-booked` first (and the refusal only for a fresh identity) passes everything.
#[test]
fn adversary_pass2_create_or_refuse_existence_first_target_is_caught() {
    let model = Model {
        book: REJECTED,
        ..Model::PLAIN
    };
    let correct = run(&model, &Store::new(Precedence::InputFirst));
    assert!(
        correct.contains_key("demo.items.BookSlot/outcome/rejected"),
        "{correct:#?}"
    );
    assert!(
        not_passed(&correct).is_empty(),
        "the input-first target the decision describes fails: {correct:#?}"
    );
    let wrong = run(&model, &Store::new(Precedence::ExistenceFirst));
    assert!(
        !not_passed(&wrong).is_empty(),
        "decision (round 1): input-guarded refusal before existence, in both forms; a \
         create-or-refuse target answering `already-booked` for a stored slot sent with the \
         refused label passes every scenario: {wrong:#?}"
    );
}

/// The same decision on create-or-update, the other half: a target that checks the refusal only
/// on the creating path (identity unknown) and updates a stored row whatever the label passes,
/// because the refusal scenario sends an identity nothing stored and the update scenario sends a
/// label the refusal does not claim.
#[test]
fn adversary_pass2_create_or_update_that_skips_the_refusal_for_a_stored_row_is_caught() {
    let model = Model {
        put: REJECTED,
        ..Model::PLAIN
    };
    let correct = run(&model, &Store::new(Precedence::InputFirst));
    assert!(
        not_passed(&correct).is_empty(),
        "the input-first target the decision describes fails: {correct:#?}"
    );
    let wrong = run(&model, &Store::new(Precedence::ExistenceFirst));
    assert!(
        !not_passed(&wrong).is_empty(),
        "decision (round 1): a request the refusal claims by its input is refused whether or \
         not a record carries the identity; a target that updates a stored item sent with the \
         refused label passes every scenario: {wrong:#?}"
    );
}

// ---- a target the scenarios share, beside an ess/15 unknown-identity refusal ---------------------

/// The design note: the creating branch is sent "an identity no other scenario sends (the ess/15
/// fresh identity), so a target the scenarios share cannot already hold it". The ess/15 fresh
/// identity is exactly what another command's `unknown_instance:` refusal scenario sends for the
/// same entity: after `PutItem/created` stored it, `RetireItem/missing` finds the row on a shared
/// target and a correct implementation fails it.
#[test]
fn adversary_pass2_a_shared_correct_target_passes_beside_an_unknown_identity_refusal() {
    let model = Model::PLAIN;
    let (synthesized, _) = suite(&model);
    let sent = |id: &str, command: &str| -> Vec<String> {
        let (_, scenario) = synthesized
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == id)
            .unwrap_or_else(|| panic!("{id} is filed"));
        scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ess_conformance::ScenarioStep::ExecuteCommand {
                    command: to, input, ..
                } if to.to_string() == command => input.get("item_id").map(|v| format!("{v:?}")),
                _ => None,
            })
            .collect()
    };
    let created = sent("demo.items.PutItem/outcome/created", "demo.items.PutItem");
    let missing = sent(
        "demo.items.RetireItem/outcome/missing",
        "demo.items.RetireItem",
    );
    assert!(
        created.iter().all(|value| !missing.contains(value)),
        "the creation's \"identity no other scenario sends\" is the one the unknown-identity \
         refusal on the same entity sends: created {created:?}, missing {missing:?}"
    );
    let isolated = run(&model, &Store::new(Precedence::InputFirst));
    assert!(
        isolated.contains_key("demo.items.RetireItem/outcome/missing"),
        "{isolated:#?}"
    );
    assert!(
        not_passed(&isolated).is_empty(),
        "an isolated correct target fails: {isolated:#?}"
    );
    let mut shared = Store::new(Precedence::InputFirst);
    shared.shared = true;
    let statuses = run(&model, &shared);
    // Only the two scenarios the fresh identity is about: arrangements through the plain witness
    // collide on a shared target for any caller-supplied identity, before this unit as after it.
    for id in [
        "demo.items.PutItem/outcome/created",
        "demo.items.RetireItem/outcome/missing",
    ] {
        assert_eq!(
            statuses.get(id),
            Some(&Status::Passed),
            "{id}: a correct target the scenarios share fails, because the creation stored the \
             identity the unknown-identity refusal sends: {statuses:#?}"
        );
    }
}

// ---- update branches the input partitions ----------------------------------------------------------

const PARTITIONED: &str = "      - name: relabelled
        when: label == \"x\"
        updates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
";

/// Create-or-update whose update is two branches the input selects between (`relabelled` for
/// `label == \"x\"`, `updated` otherwise). The creating branch is selected by existence, not by
/// input ("its input refutes every sibling guard" is the rule for a default branch): a fresh
/// identity sent with any label must create. The first call is still required.
#[test]
fn adversary_pass2_a_creation_beside_input_selected_updates_is_still_witnessed() {
    let model = Model {
        put: PARTITIONED,
        ..Model::PLAIN
    };
    let (suite, refusals) = suite(&model);
    let ids: Vec<String> = suite.scenarios.keys().map(ToString::to_string).collect();
    for id in [
        "demo.items.PutItem/outcome/created",
        "demo.items.PutItem/outcome/updated",
        "demo.items.PutItem/outcome/relabelled",
    ] {
        assert!(
            ids.iter().any(|filed| filed == id),
            "{id} is not filed; refusals: {refusals:#?}"
        );
    }
}

// ---- an optional input read through a literal fallback, on both halves -------------------------

/// The model with an optional `note` both `PutItem` branches store through `{input: note, else:
/// 'none yet'}` (ess/16, #163): the same payload on both halves, which is what an upsert writes.
fn with_note() -> String {
    let text = Model::PLAIN
        .text()
        .replacen(
            "      - {name: label, type: demo.items.Label}\n    lifecycle:\n      initial: Active",
            "      - {name: label, type: demo.items.Label}\n      - {name: note, type: Optional<demo.items.Label>}\n    lifecycle:\n      initial: Active",
            1,
        )
        .replacen(
            "      - {name: item_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n      - name: updated",
            "      - {name: item_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n      - {name: note, type: Optional<demo.items.Label>}\n    outcomes:\n      - name: updated",
            1,
        )
        .replacen(
            "        sets:\n          label: input.label\n",
            "        sets:\n          label: input.label\n          note: {input: note, else: 'none yet'}\n",
            2,
        );
    assert_eq!(
        text.matches("else: 'none yet'").count(),
        2,
        "fixture rewritten"
    );
    text
}

/// The design note: the creating branch is sent "an identity no other scenario sends ..., so a
/// target the scenarios share cannot already hold it". With an `else:` fallback the scenario gains
/// a second invocation that leaves `note` out (`Witness::LiteralFallbacks`), built by
/// `arranged_without_fallbacks` at `Distinction::further(FRESH_WITNESSES + 1)`, which never
/// passes through `existence::fresh_created`. Every identity the creating scenario sends to
/// `PutItem` must be one no other scenario sends.
#[test]
fn adversary_pass2_every_creating_call_sends_an_identity_no_other_scenario_sends() {
    let text = with_note();
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    let sent = |scenario: &ess_conformance::ConformanceScenario| -> Vec<String> {
        scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ess_conformance::ScenarioStep::ExecuteCommand { input, .. } => {
                    input.get("item_id").map(|v| format!("{v:?}"))
                }
                _ => None,
            })
            .collect()
    };
    let (_, created) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.items.PutItem/outcome/created")
        .unwrap_or_else(|| panic!("created is filed: {:#?}", synthesis.refusals));
    let mine = sent(created);
    assert!(
        mine.len() > 1,
        "the creating scenario is expected to carry the literal-fallback invocation too: {:#?}",
        created.steps
    );
    for (id, scenario) in &synthesis.suite.scenarios {
        if id.to_string() == "demo.items.PutItem/outcome/created" {
            continue;
        }
        let theirs = sent(scenario);
        let shared: Vec<&String> = mine.iter().filter(|value| theirs.contains(value)).collect();
        assert!(
            shared.is_empty(),
            "`PutItem/created` sends {shared:?}, which `{id}` sends too; the creating scenario \
             sends {mine:?}"
        );
    }
}

// ---- identities that are not text ----------------------------------------------------------------

/// Both forms over a numeric identity: every scenario about `PutItem` and `BookSlot` is filed and
/// none is refused, as over the text identity. (A `Decimal` identity refuses `already-booked` by
/// name: the refused-subject observation reads no `Decimal` exactly, as for wrong-state refusals.)
#[test]
fn adversary_pass2_numeric_identities_witness_both_forms() {
    for of in ["Integer"] {
        let text = Model::PLAIN.text().replace(
            "{name: demo.items.ItemId, kind: newtype, of: String}",
            &format!("{{name: demo.items.ItemId, kind: newtype, of: {of}}}"),
        );
        let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
        let ids: Vec<String> = synthesis
            .suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect();
        for id in [
            "demo.items.PutItem/outcome/created",
            "demo.items.PutItem/outcome/updated",
            "demo.items.BookSlot/outcome/booked",
            "demo.items.BookSlot/outcome/already-booked",
        ] {
            assert!(
                ids.iter().any(|filed| filed == id),
                "{of}: {id} is not filed; refusals: {:#?}",
                synthesis.refusals
            );
        }
    }
}

// ---- create-or-refuse with two creating paths --------------------------------------------------------

const EXPRESS: &str = "      - name: booked-express
        when: label == \"express\"
        creates: demo.items.Slot
        instance: slot_id
        emits: [demo.items.SlotBooked]
        payload:
          demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}
        sets:
          label: input.label
";

/// Create-or-refuse whose creation is two branches the input selects between (`booked-express`
/// for `label == "express"`, `booked` otherwise). `existing_instance:` answers a stored identity on
/// either path, but `synthesize/existence.rs` builds the one two-call scenario from the first
/// creating branch it finds, so a target that skips the existence check on the other path (and
/// books a second row) passes every scenario.
#[test]
fn adversary_pass2_create_or_refuse_is_witnessed_on_every_creating_path() {
    let model = Model {
        book: EXPRESS,
        ..Model::PLAIN
    };
    let mut correct = Store::new(Precedence::InputFirst);
    correct.express = true;
    let statuses = run(&model, &correct);
    assert!(
        statuses.contains_key("demo.items.BookSlot/outcome/booked-express"),
        "{statuses:#?}"
    );
    assert!(
        not_passed(&statuses).is_empty(),
        "a correct target fails: {statuses:#?}"
    );
    let mut wrong = Store::new(Precedence::InputFirst);
    wrong.express = true;
    wrong.default_skips_existence = true;
    let statuses = run(&model, &wrong);
    assert!(
        !not_passed(&statuses).is_empty(),
        "a target that books a stored slot a second time on the default path passes every \
         scenario: {statuses:#?}"
    );
}
