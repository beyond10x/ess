//! The generated Go runtime gives the reference verdicts for a view read through a reference and a
//! refusal publishing the referenced row's state (`ess/28`, beyond10x/ess#564, beyond10x/ess#557):
//! the `<view>/reference/<field>` scenarios are suite/54 vocabulary, and their steps are ones the
//! runtime already executes.
//!
//! The target is written once, here, recorded against the Rust runner and replayed to the Go
//! runtime. Its wrong modes are the defects the scenarios exist to catch.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, ErrorRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const DESK: &str = include_str!("fixtures/navigate-a-reference.yaml");

const PROJECTED: &str = "relstate.desk.Tickets/reference/desk_id";
const FILTERED: &str = "relstate.desk.TicketsAtOpenDesks/reference/desk_id";
const REFUSED: &str = "relstate.desk.OpenTicket/outcome/desk-not-open";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn go_gives_the_reference_verdict_for_every_read_through_a_reference() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(DESK));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/54"
    );
    for (mode, expected) in [
        (Mode::Correct, vec![]),
        (Mode::CopiesAtWrite, vec![PROJECTED, FILTERED]),
        (Mode::LatestDesk, vec![PROJECTED, FILTERED]),
        (Mode::WrongCurrent, vec![REFUSED]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("view-reference-{mode:?}").to_lowercase(),
            &suite,
            Desks::new(mode),
        );
        assert_eq!(support_go::not_passed(&verdicts), expected, "{mode:?}");
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Reads the desk a ticket names, as it is when read.
    Correct,
    /// Reads the desk's state and label as they were when the ticket was opened.
    CopiesAtWrite,
    /// Reads the desk opened last, whichever desk the ticket names.
    LatestDesk,
    /// Refuses a ticket on a closed desk naming `Open` as its state.
    WrongCurrent,
}

#[derive(Clone)]
struct Desk {
    id: String,
    label: Node,
    state: &'static str,
}

#[derive(Clone)]
struct Ticket {
    id: String,
    desk: String,
    /// The desk's state and label when the ticket was opened.
    copied: (&'static str, Node),
}

#[derive(Default)]
struct World {
    desks: Vec<Desk>,
    tickets: Vec<Ticket>,
}

struct Desks {
    mode: Mode,
    world: RefCell<World>,
    minted: Cell<u32>,
}

impl Desks {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            world: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    /// The state and label this implementation reads for `ticket`'s desk.
    fn read(&self, world: &World, ticket: &Ticket) -> Option<(&'static str, Node)> {
        match self.mode {
            Mode::CopiesAtWrite => Some(ticket.copied.clone()),
            Mode::LatestDesk => world
                .desks
                .last()
                .map(|desk| (desk.state, desk.label.clone())),
            Mode::Correct | Mode::WrongCurrent => world
                .desks
                .iter()
                .find(|desk| desk.id == ticket.desk)
                .map(|desk| (desk.state, desk.label.clone())),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

/// The `n`th identity this target mints: both entities' identities are `Uuid` newtypes.
fn uuid(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}

impl ConformanceTarget for Desks {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("desk-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.world.replace(World::default());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let mut world = self.world.borrow_mut();
        let named = |field: &str| match request.input.get(field) {
            Some(Node::Text(id)) => Some(id.clone()),
            _ => None,
        };
        let result = match command.to_string().as_str() {
            "relstate.desk.OpenDesk" => {
                let id = uuid(n);
                world.desks.push(Desk {
                    id: id.clone(),
                    label: request.input["label"].clone(),
                    state: "Open",
                });
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("relstate.desk.DeskOpened".parse().unwrap())
                        .with("desk_id", text(&id)),
                )
            }
            "relstate.desk.CloseDesk" => {
                let desk = named("desk_id").and_then(|id| {
                    world
                        .desks
                        .iter_mut()
                        .find(|desk| desk.id == id && desk.state == "Open")
                });
                match desk {
                    Some(desk) => {
                        desk.state = "Closed";
                        SemanticCommandResult::took(outcome(&command, "closed")).emitting(
                            ObservedEvent::new("relstate.desk.DeskClosed".parse().unwrap())
                                .with("desk_id", text(&desk.id)),
                        )
                    }
                    None => SemanticCommandResult::took(outcome(&command, "wrong-state"))
                        .with_error(DeclaredErrorValue::new(ErrorRef::new(
                            "relstate.desk.WrongState".parse().unwrap(),
                        ))),
                }
            }
            "relstate.desk.OpenTicket" => {
                let desk = named("desk_id")
                    .and_then(|id| world.desks.iter().find(|desk| desk.id == id).cloned());
                match desk {
                    None => SemanticCommandResult::took(outcome(&command, "unknown-desk"))
                        .with_error(DeclaredErrorValue::new(ErrorRef::new(
                            "relstate.desk.DeskNotFound".parse().unwrap(),
                        ))),
                    Some(desk) if desk.state != "Open" => {
                        let current = if self.mode == Mode::WrongCurrent {
                            "Open"
                        } else {
                            desk.state
                        };
                        SemanticCommandResult::took(outcome(&command, "desk-not-open")).with_error(
                            DeclaredErrorValue::new(ErrorRef::new(
                                "relstate.desk.DeskNotOpen".parse().unwrap(),
                            ))
                            .with("current", text(current)),
                        )
                    }
                    Some(desk) => {
                        let id = uuid(n);
                        world.tickets.push(Ticket {
                            id: id.clone(),
                            desk: desk.id.clone(),
                            copied: (desk.state, desk.label.clone()),
                        });
                        SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                            ObservedEvent::new("relstate.desk.TicketOpened".parse().unwrap())
                                .with("ticket_id", text(&id)),
                        )
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let world = self.world.borrow();
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "relstate.desk.Desks" => world
                .desks
                .iter()
                .map(|desk| {
                    BTreeMap::from([
                        ("desk_id".to_owned(), text(&desk.id)),
                        ("state".to_owned(), text(desk.state)),
                    ])
                })
                .collect(),
            "relstate.desk.Tickets" => world
                .tickets
                .iter()
                .map(|ticket| {
                    let (state, label) = self
                        .read(&world, ticket)
                        .map_or((Node::Null, Node::Null), |(state, label)| {
                            (text(state), label)
                        });
                    BTreeMap::from([
                        ("ticket_id".to_owned(), text(&ticket.id)),
                        ("desk_id".to_owned(), text(&ticket.desk)),
                        ("desk_state".to_owned(), state),
                        ("desk_label".to_owned(), label),
                    ])
                })
                .collect(),
            "relstate.desk.TicketsAtOpenDesks" => world
                .tickets
                .iter()
                .filter(|ticket| {
                    self.read(&world, ticket)
                        .is_some_and(|(state, _)| state == "Open")
                })
                .map(|ticket| BTreeMap::from([("ticket_id".to_owned(), text(&ticket.id))]))
                .collect(),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
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
