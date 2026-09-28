//! Outcomes selected by whether the addressed record exists are witnessed by two calls with one
//! identity (ess/16; beyond10x/ess#164, `docs/design/outcome-shapes.md`).
//!
//! Create-or-update (`PutItem`): the first call with a fresh identity must take the creating
//! branch; a second call with that identity must take the update, leave exactly one row for it and
//! show the new field values. Create-or-refuse (`BookSlot`): the second call must answer the
//! declared error, publish no event and leave the row unchanged. One in-memory target implements
//! the behaviour the issue describes; each wrong mode breaks it one way, and exactly the scenario
//! about the second call fails.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
    ViewExpectation,
};
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
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesis() -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(MODEL))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The commands a scenario sends, in order, with the value each sends for `field`.
fn sends(scenario: &ConformanceScenario, command: &str, field: &str) -> Vec<Option<ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.get(field).cloned()),
            _ => None,
        })
        .collect()
}

/// The outcomes a scenario expects, in order.
fn expected_outcomes(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.to_string()),
            _ => None,
        })
        .collect()
}

// ---- the behaviour the issue describes, and ways to get it wrong ------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Upsert and create-or-refuse as specified.
    Correct,
    /// A second put with one identity stores a second row and answers `created` again.
    PutDuplicates,
    /// A second put with one identity is refused with an undeclared answer.
    PutRefuses,
    /// A second put answers `updated` and leaves the stored label as it was.
    PutIgnoresUpdate,
    /// A second put answers `updated` and stores a second row beside the first.
    PutUpdatesByInsert,
    /// A second booking with one identity overwrites the row and answers `booked`.
    BookOverwrites,
    /// A second booking answers the declared error and still overwrites the row.
    BookRefusesAndOverwrites,
}

struct Store {
    mode: Mode,
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

    fn stored(
        command: &CommandRef,
        outcome: &str,
        event: &str,
        key: &str,
        id: &str,
        label: &str,
    ) -> SemanticCommandResult {
        let mut result = Self::took(command, outcome);
        let mut observed = ObservedEvent::new(event.parse().unwrap());
        observed
            .payload
            .insert(key.to_owned(), Node::Text(id.to_owned()));
        observed
            .payload
            .insert("label".to_owned(), Node::Text(label.to_owned()));
        result.direct_events.push(observed);
        result
    }

    fn put(&self, command: &CommandRef, id: &str, label: &str) -> SemanticCommandResult {
        let mut items = self.items.borrow_mut();
        let existing = items.iter().position(|(held, _)| held == id);
        match (existing, self.mode) {
            (None, _) | (Some(_), Mode::PutDuplicates) => {
                items.push((id.to_owned(), label.to_owned()));
                Self::stored(
                    command,
                    "created",
                    "demo.items.ItemStored",
                    "item_id",
                    id,
                    label,
                )
            }
            (Some(_), Mode::PutRefuses) => SemanticCommandResult::undeclared(),
            (Some(_), Mode::PutUpdatesByInsert) => {
                items.push((id.to_owned(), label.to_owned()));
                Self::stored(
                    command,
                    "updated",
                    "demo.items.ItemStored",
                    "item_id",
                    id,
                    label,
                )
            }
            (Some(at), mode) => {
                if mode != Mode::PutIgnoresUpdate {
                    label.clone_into(&mut items[at].1);
                }
                Self::stored(
                    command,
                    "updated",
                    "demo.items.ItemStored",
                    "item_id",
                    id,
                    label,
                )
            }
        }
    }

    fn book(&self, command: &CommandRef, id: &str, label: &str) -> SemanticCommandResult {
        let mut slots = self.slots.borrow_mut();
        let Some(at) = slots.iter().position(|(held, _)| held == id) else {
            slots.push((id.to_owned(), label.to_owned()));
            return Self::stored(
                command,
                "booked",
                "demo.items.SlotBooked",
                "slot_id",
                id,
                label,
            );
        };
        match self.mode {
            Mode::BookOverwrites => {
                label.clone_into(&mut slots[at].1);
                Self::stored(
                    command,
                    "booked",
                    "demo.items.SlotBooked",
                    "slot_id",
                    id,
                    label,
                )
            }
            mode => {
                if mode == Mode::BookRefusesAndOverwrites {
                    label.clone_into(&mut slots[at].1);
                }
                let mut result = Self::took(command, "already-booked");
                result.error = Some(DeclaredErrorValue::new(
                    "demo.items.SlotTaken".parse().unwrap(),
                ));
                result
            }
        }
    }
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("upsert-by-existence", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.items.borrow_mut().clear();
        self.slots.borrow_mut().clear();
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
        Err(TargetError::unsupported(
            "external",
            "the model declares none",
        ))
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

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
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

// ---- the suite ----------------------------------------------------------------------------------

#[test]
fn both_forms_synthesize_without_a_refusal() {
    let synthesis = synthesis();
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    for id in [CREATED, UPDATED, BOOKED, TAKEN] {
        scenario(&synthesis.suite, id);
    }
}

#[test]
fn the_first_call_with_a_fresh_identity_requires_the_create() {
    let synthesis = synthesis();
    let created = scenario(&synthesis.suite, CREATED);
    assert_eq!(
        expected_outcomes(created),
        ["demo.items.PutItem/created"],
        "{:#?}",
        created.steps
    );
    assert!(created.steps.iter().any(|step| matches!(step,
        ScenarioStep::ExpectEvent { event, .. } if event.to_string() == "demo.items.ItemStored")));
    assert!(
        !created
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectError { .. })),
        "the create reports no error: {:#?}",
        created.steps
    );
    // Fresh: not the identity the update scenario's arrangement creates, which a target the
    // scenarios share may already hold.
    let sent = sends(created, "demo.items.PutItem", "item_id");
    let arranged = sends(
        scenario(&synthesis.suite, UPDATED),
        "demo.items.PutItem",
        "item_id",
    );
    assert_eq!(sent.len(), 1, "{:#?}", created.steps);
    assert_ne!(
        sent[0], arranged[0],
        "the create sends an identity of its own"
    );
}

#[test]
fn a_second_call_with_the_same_identity_requires_the_update_on_one_row() {
    let synthesis = synthesis();
    let updated = scenario(&synthesis.suite, UPDATED);
    assert_eq!(
        expected_outcomes(updated),
        ["demo.items.PutItem/created", "demo.items.PutItem/updated"],
        "{:#?}",
        updated.steps
    );
    let ids = sends(updated, "demo.items.PutItem", "item_id");
    assert_eq!(ids.len(), 2, "two calls: {:#?}", updated.steps);
    // The second call names the row the first one created: the identity the first call's event
    // published, bound right after it.
    let captured = updated.steps.iter().find_map(|step| match step {
        ScenarioStep::CaptureInstance {
            instance, entity, ..
        } if entity.to_string() == "demo.items.Item" => {
            Some(ScenarioValue::instance(instance.clone()))
        }
        _ => None,
    });
    assert!(
        ids[0].as_ref().is_some_and(|id| id.as_literal().is_some()),
        "the first call sends a literal identity"
    );
    assert_eq!(
        ids[1], captured,
        "the second call sends the identity the first created"
    );
    let labels = sends(updated, "demo.items.PutItem", "label");
    assert_ne!(labels[0], labels[1], "the second call changes the label");
    let second = labels[1].clone().expect("the second call sends a label");
    assert!(
        updated.steps.iter().any(|step| matches!(step,
        ScenarioStep::ExpectView { expectation: ViewExpectation::Contains { fields }, .. }
            if fields.get("label") == Some(&second))),
        "the row shows the new label: {:#?}",
        updated.steps
    );
    let last_call = updated
        .steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .unwrap();
    assert!(
        updated.steps[last_call..].iter().any(|step| matches!(step,
            ScenarioStep::SnapshotSubject { subject, .. }
                if subject.get("item_id") == captured.as_ref())),
        "exactly one row carries the identity after the second call: {:#?}",
        updated.steps
    );
}

#[test]
fn a_second_create_with_the_same_identity_requires_the_declared_error_and_no_change() {
    let synthesis = synthesis();
    let taken = scenario(&synthesis.suite, TAKEN);
    assert_eq!(
        expected_outcomes(taken),
        [
            "demo.items.BookSlot/booked",
            "demo.items.BookSlot/already-booked"
        ],
        "{:#?}",
        taken.steps
    );
    let ids = sends(taken, "demo.items.BookSlot", "slot_id");
    assert_eq!(ids.len(), 2, "two calls: {:#?}", taken.steps);
    assert_eq!(ids[0], ids[1], "one identity, sent twice");
    let labels = sends(taken, "demo.items.BookSlot", "label");
    assert_ne!(labels[0], labels[1], "an overwrite would be visible");
    let second_call = taken
        .steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .unwrap();
    let after = &taken.steps[second_call..];
    assert!(after.iter().any(|step| matches!(step,
        ScenarioStep::ExpectError { error, .. } if error.to_string() == "demo.items.SlotTaken")));
    assert!(after.iter().any(|step| matches!(step,
        ScenarioStep::ExpectNoEvent { event } if event.to_string() == "demo.items.SlotBooked")));
    // The row as the first call made it: snapshotted before the second call, compared after it.
    assert!(
        taken.steps[..second_call].iter().any(|step| matches!(
            step,
            ScenarioStep::SnapshotSubject { .. } | ScenarioStep::SnapshotCompleteSubject { .. }
        )),
        "the row is snapshotted before the second call: {:#?}",
        taken.steps
    );
    assert!(
        after.iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectSubjectUnchanged { .. }
                | ScenarioStep::ExpectCompleteSubjectUnchanged { .. }
        )),
        "the row is unchanged after it: {:#?}",
        taken.steps
    );
}

#[test]
fn a_view_publishing_the_state_as_well_is_snapshotted_and_compared() {
    let text = MODEL.replace(
        "  - name: demo.items.SlotDetails
    source: demo.items.Slot
    consistency: read_your_writes
    fields:
",
        "  - name: demo.items.SlotDetails
    source: demo.items.Slot
    consistency: read_your_writes
    fields:
      - {name: state, type: demo.items.Slot.State}
",
    );
    assert_ne!(text, MODEL, "the fixture was rewritten");
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let taken = scenario(&synthesis.suite, TAKEN);
    let second_call = taken
        .steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .unwrap();
    assert!(
        taken.steps[..second_call].iter().any(|step| matches!(
            step,
            ScenarioStep::SnapshotSubject { .. } | ScenarioStep::SnapshotCompleteSubject { .. }
        )),
        "{:#?}",
        taken.steps
    );
    assert!(
        taken.steps[second_call..].iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectSubjectUnchanged { .. }
                | ScenarioStep::ExpectCompleteSubjectUnchanged { .. }
        )),
        "{:#?}",
        taken.steps
    );
    let statuses = run(&synthesis.suite, &Store::new(Mode::Correct));
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
    let statuses = run(
        &synthesis.suite,
        &Store::new(Mode::BookRefusesAndOverwrites),
    );
    assert_eq!(not_passed(&statuses), [TAKEN], "{statuses:#?}");
}

#[test]
fn every_scenario_passes_against_the_behaviour_the_issue_describes() {
    let synthesis = synthesis();
    let statuses = run(&synthesis.suite, &Store::new(Mode::Correct));
    for id in [CREATED, UPDATED, BOOKED, TAKEN] {
        assert!(statuses.contains_key(id), "{statuses:#?}");
    }
    assert!(
        not_passed(&statuses).is_empty(),
        "every scenario passes: {statuses:#?}"
    );
}

#[test]
fn exactly_the_second_call_scenario_catches_each_wrong_answer() {
    let synthesis = synthesis();
    for (mode, caught) in [
        (Mode::PutDuplicates, UPDATED),
        (Mode::PutRefuses, UPDATED),
        (Mode::PutIgnoresUpdate, UPDATED),
        (Mode::PutUpdatesByInsert, UPDATED),
        (Mode::BookOverwrites, TAKEN),
        (Mode::BookRefusesAndOverwrites, TAKEN),
    ] {
        let statuses = run(&synthesis.suite, &Store::new(mode));
        assert_eq!(
            not_passed(&statuses),
            [caught],
            "{mode:?} fails exactly {caught}: {statuses:#?}"
        );
    }
}

#[test]
fn the_witness_needs_no_new_suite_format() {
    let synthesis = synthesis();
    let version = synthesis.suite.provenance.suite_version.to_string();
    assert!(
        version != "ess-conformance/26" && version != "ess-conformance/27",
        "the two-call witness uses existing steps only: {version}"
    );
}

/// The #164 repro as filed, with the one line it lacked: `unknown_instance: true` on `created`.
const REPRO: &str = "format: ess/16
system: demo
version: v1
domains: [demo.items]
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
events:
  - name: demo.items.ItemStored
    fields:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
actors:
  - {name: demo.items.Admin, may: [demo.items.PutItem, demo.items.RetireItem]}
commands:
  - name: demo.items.PutItem
    input:
      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
    outcomes:
      - name: updated
        updates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
        summary: An item with this id exists; its label is replaced.
      - name: created
        unknown_instance: true
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
        summary: No item with this id exists; one is created.
  - name: demo.items.RetireItem
    input:
      - {name: item_id, type: demo.items.ItemId}
    outcomes:
      - name: retired
        moves: demo.items.Item.retire
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: {subject: label}}
";

#[test]
fn the_issue_repro_validates_and_its_second_call_is_required_to_update() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(REPRO));
    let updated = scenario(&synthesis.suite, UPDATED);
    assert_eq!(
        expected_outcomes(updated),
        ["demo.items.PutItem/created", "demo.items.PutItem/updated"],
        "{:#?}",
        updated.steps
    );
    let created = scenario(&synthesis.suite, CREATED);
    assert_eq!(expected_outcomes(created), ["demo.items.PutItem/created"]);
    assert!(
        !synthesis.refusals.iter().any(|refusal| {
            let text = format!("{:?}", refusal.subject);
            text.contains("PutItem")
        }),
        "neither PutItem branch is refused: {:#?}",
        synthesis.refusals
    );
}

/// The repro with its update selected by the held state: `updated` from `Active`, and a branch
/// that keeps the label from `Retired`; plus the view held-state selection is observed through.
fn held_state_repro() -> String {
    let text = format!(
        "{REPRO}views:
  - name: demo.items.ItemDetails
    source: demo.items.Item
    consistency: read_your_writes
    fields:
      - {{name: item_id, type: demo.items.ItemId}}
      - {{name: label, type: demo.items.Label}}
      - {{name: state, type: demo.items.Item.State}}
"
    );
    let text = text
        .replace(
            "      - name: updated\n        updates: demo.items.Item\n",
            "      - name: updated\n        when_subject_state: Active\n        updates: demo.items.Item\n",
        )
        .replace(
            "      - name: created\n",
            "      - name: retired-item\n        when_subject_state: Retired\n        updates: demo.items.Item\n        instance: item_id\n        emits: [demo.items.ItemStored]\n        payload:\n          demo.items.ItemStored: {item_id: input.item_id, label: {subject: label}}\n      - name: created\n",
        )
;
    assert!(text.contains("when_subject_state: Active") && text.contains("retired-item"));
    text
}

#[test]
fn a_held_state_update_beside_the_creation_is_admitted_and_both_calls_are_witnessed() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&held_state_repro()));
    let created = scenario(&synthesis.suite, CREATED);
    assert_eq!(expected_outcomes(created), ["demo.items.PutItem/created"]);
    let updated = scenario(&synthesis.suite, UPDATED);
    assert_eq!(
        expected_outcomes(updated),
        ["demo.items.PutItem/created", "demo.items.PutItem/updated"],
        "{:#?}",
        updated.steps
    );
}

/// The #164 follow-up with an optional caller id (`{input: slot_id, else: {generated: true}}`):
/// the two-call witness sends the id both times and requires the refusal the second time.
#[test]
fn an_optional_caller_id_is_witnessed_by_two_calls_that_send_it() {
    let text = MODEL
        .replace(
            "      - {name: slot_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n      - name: booked",
            "      - {name: slot_id, type: Optional<demo.items.ItemId>}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n      - name: booked",
        )
        .replace(
            "demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}",
            "demo.items.SlotBooked: {slot_id: {input: slot_id, else: {generated: true}}, label: input.label}",
        );
    assert!(
        text.contains("else: {generated: true}") && text.contains("Optional<demo.items.ItemId>")
    );
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    let taken = scenario(&synthesis.suite, TAKEN);
    let ids = sends(taken, "demo.items.BookSlot", "slot_id");
    assert_eq!(ids.len(), 2, "{:#?}", taken.steps);
    assert!(ids[0].is_some(), "the first call sends the id");
    assert_eq!(ids[0], ids[1], "one identity, sent twice");
    let statuses = run(&synthesis.suite, &Store::new(Mode::Correct));
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
}
