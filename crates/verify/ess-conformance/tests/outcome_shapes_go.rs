//! The generated Go runtime runs the `ess/15` outcome shapes (`expect_subject_absent`,
//! `snapshot_view`, `expect_view_unchanged`; suite/22 and coverage /23, beyond10x/ess#144–#152) and
//! gives the reference verdicts (beyond10x/ess#188).
//!
//! The target is `tests/outcome_shapes.rs`'s, with each mode that breaks one shape, recorded once
//! and replayed to the Go runtime — the ordinary suite and the coverage input both.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::CommandRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/outcome-shapes.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("outcome-shapes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

const MODES: [Mode; 5] = [
    Mode::Correct,
    Mode::FoldsUnknownIntoWrongState,
    Mode::KeepsEndedRows,
    Mode::OffersIntoInitial,
    Mode::TouchWrites,
];

#[test]
fn go_gives_the_reference_verdict_for_every_outcome_shape_mode() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    for mode in MODES {
        let verdicts = support_go::assert_parity(
            &format!("shapes-{mode:?}").to_lowercase(),
            &suite,
            Calls::new(mode),
        );
        assert_eq!(
            support_go::not_passed(&verdicts).is_empty(),
            mode == Mode::Correct,
            "{mode:?}: {verdicts:?}"
        );
    }
}

#[test]
fn go_gives_the_reference_verdict_on_the_coverage_input_23() {
    use ess_conformance::coverage::{Origins, Scope};
    let input = ess_conformance::coverage_build::build(
        &ir_of(MODEL),
        &[],
        Scope::System,
        Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/35"
    );
    for mode in MODES {
        let label = format!("shapes-coverage-{mode:?}").to_lowercase();
        let compared = support_go::compare_input(
            &label,
            &input,
            Calls::new(mode),
            &support_go::Options::default(),
        );
        support_go::assert_compared(&label, compared);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// The behaviour every issue describes as correct.
    Correct,
    /// #145 as reported: an id the system never held answers the `wrong_state` branch.
    FoldsUnknownIntoWrongState,
    /// #151 as reported: an ended call stays, in the state it had.
    KeepsEndedRows,
    /// #150 as reported: an offered call starts where the lifecycle starts.
    OffersIntoInitial,
    /// #144 as reported: the accepted no-op writes a row.
    TouchWrites,
}

struct Calls {
    mode: Mode,
    users: RefCell<BTreeMap<String, String>>,
    rows: RefCell<BTreeMap<String, String>>,
}

impl Calls {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            users: RefCell::new(BTreeMap::new()),
            rows: RefCell::new(BTreeMap::new()),
        }
    }
}

fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
    let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
        command.clone(),
        outcome.parse().unwrap(),
    ));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    result
}

fn emitted(result: &mut SemanticCommandResult, event: &str, field: &str, id: &str) {
    let mut observed = ObservedEvent::new(event.parse().unwrap());
    observed
        .payload
        .insert(field.to_owned(), Node::Text(id.to_owned()));
    result.direct_events.push(observed);
}

impl ConformanceTarget for Calls {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("outcome-shapes", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.users.borrow_mut().clear();
        self.rows.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    // One arm per command of the model, as the issues describe it.
    #[allow(clippy::too_many_lines)]
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let name = request.command.to_string();
        let id = |field: &str| {
            request
                .input
                .get(field)
                .and_then(Node::as_text)
                .map(str::to_owned)
                .unwrap_or_default()
        };
        // #152: the implementation cannot take a call command outside an open session.
        if name != "example.call.OpenSession" && self.users.borrow().is_empty() {
            return Ok(SemanticCommandResult::undeclared());
        }
        let command = request.command.clone();
        let mut calls = self.rows.borrow_mut();
        let result = match name.as_str() {
            "example.call.OpenSession" => {
                self.users
                    .borrow_mut()
                    .insert(id("user_id"), "Active".into());
                let mut result = took(&command, "opened");
                emitted(
                    &mut result,
                    "example.call.SessionOpened",
                    "user_id",
                    &id("user_id"),
                );
                result
            }
            "example.call.PlaceCall" => {
                calls.insert(id("call_id"), "Dialing".into());
                let mut result = took(&command, "placed");
                emitted(
                    &mut result,
                    "example.call.CallPlaced",
                    "call_id",
                    &id("call_id"),
                );
                result
            }
            "example.call.OfferCall" => {
                let state = if self.mode == Mode::OffersIntoInitial {
                    "Dialing"
                } else {
                    "Ringing"
                };
                calls.insert(id("call_id"), state.into());
                let mut result = took(&command, "offered");
                emitted(
                    &mut result,
                    "example.call.CallOffered",
                    "call_id",
                    &id("call_id"),
                );
                result
            }
            "example.call.RingCall" => match calls.get(&id("call_id")).map(String::as_str) {
                Some("Dialing") => {
                    calls.insert(id("call_id"), "Ringing".into());
                    let mut result = took(&command, "rang");
                    emitted(
                        &mut result,
                        "example.call.CallRang",
                        "call_id",
                        &id("call_id"),
                    );
                    result
                }
                _ => took(&command, "not-dialing"),
            },
            "example.call.AnswerCall" => match calls.get(&id("call_id")).cloned() {
                None if self.mode == Mode::FoldsUnknownIntoWrongState => {
                    took(&command, "not-ringing")
                }
                None => {
                    let mut result = took(&command, "no-such-call");
                    result.error = Some(DeclaredErrorValue::new(
                        "example.call.CallNotFound".parse().unwrap(),
                    ));
                    result
                }
                Some(state) if state == "Ringing" => {
                    calls.insert(id("call_id"), "Connected".into());
                    let mut result = took(&command, "answered");
                    emitted(
                        &mut result,
                        "example.call.CallAnswered",
                        "call_id",
                        &id("call_id"),
                    );
                    result
                }
                Some(_) => took(&command, "not-ringing"),
            },
            "example.call.EndCall" => {
                if calls.contains_key(&id("call_id")) {
                    if self.mode != Mode::KeepsEndedRows {
                        calls.remove(&id("call_id"));
                    }
                    let mut result = took(&command, "ended");
                    emitted(
                        &mut result,
                        "example.call.CallEnded",
                        "call_id",
                        &id("call_id"),
                    );
                    result
                } else {
                    let mut result = took(&command, "no-such-call");
                    result.error = Some(DeclaredErrorValue::new(
                        "example.call.CallNotFound".parse().unwrap(),
                    ));
                    result
                }
            }
            "example.call.Touch" => {
                if self.mode == Mode::TouchWrites {
                    calls.insert(
                        "00000000-0000-4000-8000-000000000144".into(),
                        "Dialing".into(),
                    );
                }
                took(&command, "accepted")
            }
            _ => panic!("unexpected command {name}"),
        };
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let (rows, identity) = match request.view.to_string().as_str() {
            "example.call.Calls" => (self.rows.borrow().clone(), "call_id"),
            "example.call.Users" => (self.users.borrow().clone(), "user_id"),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult {
            rows: rows
                .into_iter()
                .map(|(id, state)| {
                    BTreeMap::from([
                        (identity.to_owned(), Node::Text(id)),
                        ("state".to_owned(), Node::Text(state)),
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
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "unused"))
    }
}
