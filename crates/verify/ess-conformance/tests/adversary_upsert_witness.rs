//! Adversary cases for the two-call witness of selection by existence (ess/16,
//! beyond10x/ess#164, `docs/design/outcome-shapes.md`, "Conformance").
//!
//! One in-memory target over the design fixture, with wrong modes the unit's own suite does not
//! try, a target that keeps its rows across scenarios, a view the one-row check skips, and an
//! input-guarded refusal beside each form.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

const CREATED: &str = "demo.items.PutItem/outcome/created";
const UPDATED: &str = "demo.items.PutItem/outcome/updated";
const BOOKED: &str = "demo.items.BookSlot/outcome/booked";
const TAKEN: &str = "demo.items.BookSlot/outcome/already-booked";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("upsert-by-existence.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// A put with a fresh identity stores the row and answers `updated`.
    PutUpdatesWhenAbsent,
    /// A put with a fresh identity is refused with an undeclared answer.
    PutRefusesWhenAbsent,
    /// A second put answers `updated` and stores a second row beside the first.
    PutUpdatesByInsert,
    /// A second booking answers the declared error, leaves the row, and publishes `SlotBooked`.
    BookRefusesAndPublishes,
}

// One switch per behaviour the cases toggle; a test target, not a state machine.
#[allow(clippy::struct_excessive_bools)]
struct Store {
    mode: Mode,
    /// Rows survive `begin_scenario`: the target the scenarios share.
    shared: bool,
    /// `label == "bad"` is refused with `demo.items.BadLabel` by the branch `rejected`, on
    /// whichever command declares it, after existence has been answered (existence first).
    guarded_put: bool,
    guarded_book: bool,
    /// `label == "bad"` is refused on `PutItem` before existence is read (input first: the
    /// coordinator's decision, `docs/design/outcome-shapes.md`).
    input_first_put: bool,
    /// The same on `BookSlot` (round 2: the decision holds in both forms).
    input_first_book: bool,
    items: RefCell<Vec<(String, String)>>,
    slots: RefCell<Vec<(String, String)>>,
}

fn text(input: &BTreeMap<String, Node>, field: &str) -> String {
    input
        .get(field)
        .and_then(Node::as_text)
        .unwrap_or_default()
        .to_owned()
}

impl Store {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            shared: false,
            guarded_put: false,
            guarded_book: false,
            input_first_put: false,
            input_first_book: false,
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

    fn event(event: &str, key: &str, id: &str, label: &str) -> ObservedEvent {
        let mut observed = ObservedEvent::new(event.parse().unwrap());
        observed
            .payload
            .insert(key.to_owned(), Node::Text(id.to_owned()));
        observed
            .payload
            .insert("label".to_owned(), Node::Text(label.to_owned()));
        observed
    }

    fn stored(
        command: &CommandRef,
        outcome: &str,
        event: &str,
        key: &str,
        id: &str,
        label: &str,
    ) -> SemanticCommandResult {
        let mut result = Self::took(command, outcome);
        result
            .direct_events
            .push(Self::event(event, key, id, label));
        result
    }

    fn put(&self, command: &CommandRef, id: &str, label: &str) -> SemanticCommandResult {
        if self.input_first_put && label == "bad" {
            return Self::error(command, "rejected", "demo.items.BadLabel");
        }
        let mut items = self.items.borrow_mut();
        let existing = items.iter().position(|(held, _)| held == id);
        let stored = |outcome| {
            Self::stored(
                command,
                outcome,
                "demo.items.ItemStored",
                "item_id",
                id,
                label,
            )
        };
        match (existing, self.mode) {
            (None, Mode::PutRefusesWhenAbsent) => SemanticCommandResult::undeclared(),
            (None, Mode::PutUpdatesWhenAbsent) => {
                items.push((id.to_owned(), label.to_owned()));
                stored("updated")
            }
            (None, _) => {
                items.push((id.to_owned(), label.to_owned()));
                stored("created")
            }
            (Some(_), _) if self.guarded_put && label == "bad" => {
                Self::error(command, "rejected", "demo.items.BadLabel")
            }
            (Some(_), Mode::PutUpdatesByInsert) => {
                items.push((id.to_owned(), label.to_owned()));
                stored("updated")
            }
            (Some(at), _) => {
                label.clone_into(&mut items[at].1);
                stored("updated")
            }
        }
    }

    fn book(&self, command: &CommandRef, id: &str, label: &str) -> SemanticCommandResult {
        if self.input_first_book && label == "bad" {
            return Self::error(command, "rejected", "demo.items.BadLabel");
        }
        let mut slots = self.slots.borrow_mut();
        if slots.iter().any(|(held, _)| held == id) {
            let mut result = Self::error(command, "already-booked", "demo.items.SlotTaken");
            if self.mode == Mode::BookRefusesAndPublishes {
                result.direct_events.push(Self::event(
                    "demo.items.SlotBooked",
                    "slot_id",
                    id,
                    label,
                ));
            }
            return result;
        }
        if self.guarded_book && label == "bad" {
            return Self::error(command, "rejected", "demo.items.BadLabel");
        }
        slots.push((id.to_owned(), label.to_owned()));
        Self::stored(
            command,
            "booked",
            "demo.items.SlotBooked",
            "slot_id",
            id,
            label,
        )
    }
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-upsert", "1"))
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
        let label = text(&request.input, "label");
        Ok(match command.to_string().as_str() {
            "demo.items.PutItem" => self.put(&command, &text(&request.input, "item_id"), &label),
            "demo.items.BookSlot" => self.book(&command, &text(&request.input, "slot_id"), &label),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let (rows, key, state) = match request.view.to_string().as_str() {
            "demo.items.ItemDetails" => (self.items.borrow().clone(), "item_id", "Active"),
            "demo.items.SlotDetails" => (self.slots.borrow().clone(), "slot_id", "Open"),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult {
            rows: rows
                .into_iter()
                .map(|(id, label)| {
                    BTreeMap::from([
                        (key.to_owned(), Node::Text(id)),
                        ("label".to_owned(), Node::Text(label)),
                        ("state".to_owned(), Node::Text(state.to_owned())),
                    ])
                })
                .collect(),
            total: None,
        })
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

fn run(text: &str, target: &Store) -> BTreeMap<String, Status> {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(text));
    run_suite(&synthesis.suite, target)
}

fn run_suite(suite: &ConformanceSuite, target: &Store) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
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

// ---- wrong answers the unit's suite does not try -----------------------------------------------

/// Acceptance: "the second call ... requires the declared error (no event, row unchanged)".
#[test]
fn adversary_a_refused_second_booking_that_publishes_its_event_is_caught() {
    let statuses = run(MODEL, &Store::new(Mode::BookRefusesAndPublishes));
    assert!(
        not_passed(&statuses).contains(&TAKEN),
        "an `already-booked` answer that still publishes SlotBooked passes: {statuses:#?}"
    );
}

/// Acceptance: "first call with a fresh identity requires the create".
#[test]
fn adversary_a_first_put_answered_as_an_update_or_refused_is_caught() {
    for mode in [Mode::PutUpdatesWhenAbsent, Mode::PutRefusesWhenAbsent] {
        let statuses = run(MODEL, &Store::new(mode));
        assert!(
            not_passed(&statuses).contains(&CREATED),
            "{mode:?} passes the creating scenario: {statuses:#?}"
        );
    }
}

// ---- a target the scenarios share ----------------------------------------------------------------

/// The design note: the creating branch is sent "with an identity no other scenario sends ... so a
/// target the scenarios share cannot already hold it". A correct target that keeps its rows across
/// scenarios must pass every scenario, in both forms.
#[test]
fn adversary_a_correct_target_that_keeps_its_rows_passes_every_scenario() {
    let mut store = Store::new(Mode::Correct);
    store.shared = true;
    let statuses = run(MODEL, &store);
    for id in [CREATED, UPDATED, BOOKED, TAKEN] {
        assert!(statuses.contains_key(id), "{statuses:#?}");
    }
    assert!(
        not_passed(&statuses).is_empty(),
        "a correct shared target fails: {statuses:#?}"
    );
}

// ---- the one-row claim ----------------------------------------------------------------------------

/// Acceptance: "a second call with the same identity requires the update (one row, fields
/// updated)". The one-row snapshot is added only over unfiltered views; an Item view filtered to
/// the only state an Item can be in reads the same rows, and an implementation that answers
/// `updated` by inserting a second row must still be caught.
#[test]
fn adversary_a_duplicate_row_is_caught_when_the_only_view_is_filtered() {
    let text = MODEL.replace(
        "  - name: demo.items.ItemDetails\n    source: demo.items.Item\n    consistency: read_your_writes\n",
        "  - name: demo.items.ItemDetails\n    source: demo.items.Item\n    consistency: read_your_writes\n    filter: state == Active\n",
    );
    assert!(
        text.contains("filter: state == Active"),
        "fixture rewritten"
    );
    let statuses = run(&text, &Store::new(Mode::PutUpdatesByInsert));
    assert!(
        not_passed(&statuses).contains(&UPDATED),
        "a second row for one identity passes the update scenario: {statuses:#?}"
    );
}

// ---- an input-guarded refusal beside each form ----------------------------------------------------

const BAD_LABEL: &str =
    "  - name: demo.items.BadLabel\n    summary: The label is not accepted.\n    fields: []\n";

fn with_rejection(command_marker: &str) -> String {
    let rejected = "      - {name: rejected, when: label == \"bad\", error: demo.items.BadLabel}\n";
    let text = MODEL
        .replace("errors:\n", &format!("errors:\n{BAD_LABEL}"))
        .replacen(command_marker, &format!("{command_marker}{rejected}"), 1);
    assert!(text.contains("name: rejected"), "fixture rewritten");
    text
}

/// Create-or-refuse with an input-guarded refusal. Rewritten in correction round 2: the
/// coordinator's decision (round 1) puts the input-guarded refusal BEFORE existence in both forms,
/// and round 2 requires the suite to catch an existence-first create-or-refuse target
/// (`adversary_upsert_pass2_witness.rs`). So an input-first target passes every scenario, and an
/// existence-first one fails exactly the refusal scenario.
#[test]
fn adversary_create_or_refuse_beside_an_input_refusal_is_witnessed_against_existence_first() {
    let text = with_rejection(
        "  - name: demo.items.BookSlot\n    input:\n      - {name: slot_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n",
    );
    let mut input_first = Store::new(Mode::Correct);
    input_first.input_first_book = true;
    let statuses = run(&text, &input_first);
    assert!(
        statuses.contains_key("demo.items.BookSlot/outcome/rejected"),
        "{statuses:#?}"
    );
    assert!(
        not_passed(&statuses).is_empty(),
        "decision (round 1): input-guarded refusal first; an input-first target fails: \
         {statuses:#?}"
    );
    let mut existence_first = Store::new(Mode::Correct);
    existence_first.guarded_book = true;
    let statuses = run(&text, &existence_first);
    assert_eq!(
        not_passed(&statuses),
        ["demo.items.BookSlot/outcome/rejected"],
        "decision (round 1): input-guarded refusal first; an existence-first target must fail \
         exactly the refusal scenario: {statuses:#?}"
    );
}

/// Create-or-update with an input-guarded refusal. Rewritten for the coordinator's decision
/// (correction round 1): an input-guarded refusal (`when:` + `error:`) is answered BEFORE
/// selection by existence, the precedence #178 fixed for overlapping branches
/// (`docs/design/input-guard-overlap-precedence.md`, stated in `docs/design/outcome-shapes.md`).
/// So a target answering the refusal first passes every scenario, and one answering existence
/// first creates a fresh identity sent with `label: bad` and fails the refusal scenario.
#[test]
fn adversary_create_or_update_beside_an_input_refusal_is_witnessed_against_existence_first() {
    let text = with_rejection(
        "  - name: demo.items.PutItem\n    input:\n      - {name: item_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n",
    );
    let mut input_first = Store::new(Mode::Correct);
    input_first.input_first_put = true;
    let statuses = run(&text, &input_first);
    assert!(
        statuses.contains_key("demo.items.PutItem/outcome/rejected"),
        "{statuses:#?}"
    );
    assert!(
        not_passed(&statuses).is_empty(),
        "decision (round 1): input-guarded refusal first; an input-first target fails: \
         {statuses:#?}"
    );
    let mut existence_first = Store::new(Mode::Correct);
    existence_first.guarded_put = true;
    let statuses = run(&text, &existence_first);
    assert_eq!(
        not_passed(&statuses),
        ["demo.items.PutItem/outcome/rejected"],
        "decision (round 1): input-guarded refusal first; an existence-first target must fail \
         exactly the refusal scenario: {statuses:#?}"
    );
}

/// Under the same decision the refusal is witnessed on both sides of existence (round 2, decision
/// 1): its scenario first sends the refused input for an identity nothing stored, arranging
/// nothing, and then stores a row under a fresh identity and sends the refused input for that
/// identity, requiring the refusal again.
#[test]
fn adversary_the_create_or_update_rejection_scenario_arranges_the_row_it_refuses() {
    let text = with_rejection(
        "  - name: demo.items.PutItem\n    input:\n      - {name: item_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n",
    );
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    let (_, scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.items.PutItem/outcome/rejected")
        .expect("rejected is filed");
    let sends: Vec<(Option<Node>, Option<Node>)> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ess_conformance::ScenarioStep::ExecuteCommand { input, .. } => Some((
                input
                    .get("item_id")
                    .and_then(|value| value.as_literal().cloned()),
                input
                    .get("label")
                    .and_then(|value| value.as_literal().cloned()),
            )),
            _ => None,
        })
        .collect();
    let bad = Some(Node::Text("bad".to_owned()));
    assert_eq!(
        sends.len(),
        3,
        "decision (round 1, witnessed per round 2): the refused input once for an identity \
         nothing stored, then a stored row and the refused input for its identity: {:#?}",
        scenario.steps
    );
    assert_eq!(sends[0].1, bad, "the scenario opens with the refused input");
    assert_ne!(sends[1].1, bad, "the second call stores the row");
    assert_eq!(
        sends[2].1, bad,
        "the third call sends the refused input again"
    );
    assert_eq!(
        sends[1].0, sends[2].0,
        "the refused input is sent for the identity just stored"
    );
    assert_ne!(
        sends[0].0, sends[1].0,
        "the first refused call names no stored identity"
    );
}

/// The same create-or-refuse with its input refusal, on a target the scenarios share. Rewritten in
/// correction round 2 for the decided precedence (input-guarded refusal first): an input-first
/// target that keeps its rows across scenarios passes every scenario.
#[test]
fn adversary_create_or_refuse_beside_an_input_refusal_passes_on_a_shared_target() {
    let text = with_rejection(
        "  - name: demo.items.BookSlot\n    input:\n      - {name: slot_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n",
    );
    let mut store = Store::new(Mode::Correct);
    store.input_first_book = true;
    store.shared = true;
    let statuses = run(&text, &store);
    assert!(
        not_passed(&statuses).is_empty(),
        "decision (round 1): input-guarded refusal first; an input-first shared target fails: \
         {statuses:#?}"
    );
}
