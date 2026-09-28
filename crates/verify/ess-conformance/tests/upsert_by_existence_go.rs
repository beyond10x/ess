//! The generated Go runtime gives the reference verdicts for outcomes selected by whether the
//! addressed record exists (`unknown_instance:` beside an update, `existing_instance:` beside a
//! creation; ess/16, beyond10x/ess#164), which need no suite vocabulary of their own
//! (beyond10x/ess#188).
//!
//! The target is `tests/upsert_by_existence.rs`'s, with every wrong answer, recorded once and
//! replayed to the Go runtime.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

const UPDATED: &str = "demo.items.PutItem/outcome/updated";
const TAKEN: &str = "demo.items.BookSlot/outcome/already-booked";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("upsert-by-existence.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn go_gives_the_reference_verdict_for_every_existence_answer() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    for (mode, expected) in [
        (Mode::Correct, vec![]),
        (Mode::PutDuplicates, vec![UPDATED]),
        (Mode::PutRefuses, vec![UPDATED]),
        (Mode::PutIgnoresUpdate, vec![UPDATED]),
        (Mode::PutUpdatesByInsert, vec![UPDATED]),
        (Mode::BookOverwrites, vec![TAKEN]),
        (Mode::BookRefusesAndOverwrites, vec![TAKEN]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("upsert-{mode:?}").to_lowercase(),
            &suite,
            Store::new(mode),
        );
        assert_eq!(support_go::not_passed(&verdicts), expected, "{mode:?}");
    }
}

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
