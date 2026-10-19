//! The generated Go runtime gives the reference verdicts for a branch requiring two related rows at
//! once (`when_related: {all: […]}`, `ess/28`, downstream report 2026-10-19, finding 11). The
//! scenarios use no step a runner lacks; the target is written once, here, and replayed to Go.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, ErrorRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const WIRING: &str = include_str!("fixtures/two-rows-one-branch.yaml");

const CONNECTED: &str = "wiring.board.Connect/outcome/connected";
const BOTH_INBOUND: &str = "wiring.board.Connect/outcome/both-inbound";
const BOTH_OUTBOUND: &str = "wiring.board.Connect/outcome/both-outbound";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("wiring.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn go_gives_the_reference_verdict_for_two_rows_in_one_branch() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(WIRING));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    for (mode, expected) in [
        (Mode::Correct, vec![]),
        (Mode::SourceOnly, vec![CONNECTED]),
        (Mode::SinkOnly, vec![CONNECTED]),
        (Mode::Ignores, vec![BOTH_INBOUND, BOTH_OUTBOUND]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("related-rows-all-{mode:?}").to_lowercase(),
            &suite,
            Wiring::new(mode),
        );
        assert_eq!(support_go::not_passed(&verdicts), expected, "{mode:?}");
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Refuses where both ports face one way.
    Correct,
    /// Refuses where the source port faces the way, whichever way the sink faces.
    SourceOnly,
    /// Refuses where the sink port faces the way, whichever way the source faces.
    SinkOnly,
    /// Connects whatever way the ports face.
    Ignores,
}

struct Wiring {
    mode: Mode,
    ports: RefCell<BTreeMap<String, Node>>,
    wires: RefCell<Vec<BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Wiring {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            ports: RefCell::default(),
            wires: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    /// Whether this implementation refuses a pair facing `from` and `to` as facing one way.
    fn same(&self, from: &Node, to: &Node) -> bool {
        match self.mode {
            Mode::Correct => from == to,
            // The other row is read as facing the way the one it reads does.
            Mode::SourceOnly | Mode::SinkOnly => true,
            Mode::Ignores => false,
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

/// The `n`th identity this target mints: both entities' identities are `Uuid` newtypes.
fn uuid(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}

fn refused(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(outcome(command, name)).with_error(DeclaredErrorValue::new(
        ErrorRef::new(error.parse().unwrap()),
    ))
}

impl ConformanceTarget for Wiring {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("wiring-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.ports.replace(BTreeMap::new());
        self.wires.replace(Vec::new());
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
        let result = match command.to_string().as_str() {
            "wiring.board.AddPort" => {
                let id = uuid(n);
                self.ports
                    .borrow_mut()
                    .insert(id.clone(), request.input["direction"].clone());
                SemanticCommandResult::took(outcome(&command, "added")).emitting(
                    ObservedEvent::new("wiring.board.PortAdded".parse().unwrap())
                        .with("port_id", Node::Text(id)),
                )
            }
            "wiring.board.Connect" => {
                let ports = self.ports.borrow();
                let port = |field: &str| match request.input.get(field) {
                    Some(Node::Text(id)) => ports.get(id).cloned(),
                    _ => None,
                };
                match (port("from_port"), port("to_port")) {
                    (None, _) => refused(&command, "no-source", "wiring.board.SourceNotFound"),
                    (_, None) => refused(&command, "no-sink", "wiring.board.SinkNotFound"),
                    (Some(from), Some(to)) => {
                        let read = match self.mode {
                            Mode::SinkOnly => &to,
                            _ => &from,
                        };
                        let facing = |way: &str| *read == Node::Text(way.to_owned());
                        if self.same(&from, &to) && facing("inbound") {
                            refused(&command, "both-inbound", "wiring.board.SameDirection")
                        } else if self.same(&from, &to) && facing("outbound") {
                            refused(&command, "both-outbound", "wiring.board.SameDirection")
                        } else {
                            let id = uuid(n);
                            self.wires.borrow_mut().push(BTreeMap::from([
                                ("wire_id".to_owned(), Node::Text(id.clone())),
                                ("from_port".to_owned(), request.input["from_port"].clone()),
                                ("to_port".to_owned(), request.input["to_port"].clone()),
                            ]));
                            SemanticCommandResult::took(outcome(&command, "connected")).emitting(
                                ObservedEvent::new("wiring.board.Connected".parse().unwrap())
                                    .with("wire_id", Node::Text(id)),
                            )
                        }
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "wiring.board.Wires" => self.wires.borrow().clone(),
            "wiring.board.Ports" => self
                .ports
                .borrow()
                .iter()
                .map(|(id, direction)| {
                    BTreeMap::from([
                        ("port_id".to_owned(), Node::Text(id.clone())),
                        ("direction".to_owned(), direction.clone()),
                    ])
                })
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
