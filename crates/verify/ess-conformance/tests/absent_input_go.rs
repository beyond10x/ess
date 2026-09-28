//! The generated Go runtime sends a command with no input at all (`execute_command_without_input`,
//! suite/26, beyond10x/ess#170) and gives the reference verdicts (beyond10x/ess#188).
//!
//! The target is `tests/absent_input.rs`'s, recorded once and replayed to the Go runtime: every
//! mode the Rust suite tells apart, the Go runtime tells apart the same way. A Go target that does
//! not implement `AbsentInputTarget` skips exactly the one scenario that needs it.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/absent-input.yaml");

const ID: &str = "demo.notes.SubmitNote/outcome/body-missing";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("absent-input.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
    synthesis.suite
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    ReadsAbsentAsEmpty,
    AcceptsAbsent,
    CannotOmit,
}

/// `tests/absent_input.rs`'s target.
struct Notes {
    mode: Mode,
    rows: RefCell<BTreeMap<String, String>>,
    sequence: RefCell<u64>,
}

impl Notes {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::new(BTreeMap::new()),
            sequence: RefCell::new(0),
        }
    }

    fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
        let mut result =
            SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        result
    }

    fn submit(&self, command: &CommandRef, text: &Node) -> SemanticCommandResult {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        let id = format!("00000000-0000-4000-8000-{:012}", *sequence);
        self.rows
            .borrow_mut()
            .insert(id.clone(), text.as_text().unwrap_or_default().to_owned());
        let mut result = Self::took(command, "submitted");
        let mut event = ObservedEvent::new("demo.notes.NoteSubmitted".parse().unwrap());
        event.payload.insert("note_id".to_owned(), Node::Text(id));
        result.direct_events.push(event);
        result
    }
}

impl ConformanceTarget for Notes {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("absent-input", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
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
        match command.to_string().as_str() {
            "demo.notes.SubmitNote" => match request.input.get("text") {
                Some(text) => Ok(self.submit(&command, text)),
                None => Ok(SemanticCommandResult::undeclared()),
            },
            "demo.notes.RewordNote" => {
                let id = request
                    .input
                    .get("note_id")
                    .and_then(Node::as_text)
                    .unwrap_or_default()
                    .to_owned();
                let text = request
                    .input
                    .get("text")
                    .and_then(Node::as_text)
                    .unwrap_or_default()
                    .to_owned();
                if !self.rows.borrow().contains_key(&id) {
                    return Ok(SemanticCommandResult::undeclared());
                }
                self.rows.borrow_mut().insert(id.clone(), text);
                let mut result = Self::took(&command, "reworded");
                let mut event = ObservedEvent::new("demo.notes.NoteReworded".parse().unwrap());
                event.payload.insert("note_id".to_owned(), Node::Text(id));
                result.direct_events.push(event);
                Ok(result)
            }
            other => panic!("unexpected command {other}"),
        }
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        match self.mode {
            Mode::Correct => {
                let mut result = Self::took(&command, "body-missing");
                result.error = Some(DeclaredErrorValue::new(
                    "demo.notes.BodyMissing".parse().unwrap(),
                ));
                Ok(result)
            }
            Mode::ReadsAbsentAsEmpty => self.execute_command(SemanticCommandRequest {
                command,
                actor: request.actor,
                caller: request.caller,
                input: BTreeMap::new(),
                correlation: request.correlation,
            }),
            Mode::AcceptsAbsent => Ok(self.submit(&command, &Node::Text(String::new()))),
            Mode::CannotOmit => Err(TargetError::unsupported(
                format!("invoking `{command}` without input"),
                "this adapter always sends a body",
            )),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .keys()
                .map(|id| BTreeMap::from([("note_id".to_owned(), Node::Text(id.clone()))]))
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

#[test]
fn go_gives_the_reference_verdict_for_every_absent_input_mode() {
    let suite = suite();
    for (mode, expected) in [
        (Mode::Correct, vec![]),
        (Mode::ReadsAbsentAsEmpty, vec![(ID, "failed")]),
        (Mode::AcceptsAbsent, vec![(ID, "failed")]),
        (Mode::CannotOmit, vec![(ID, "skipped")]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("absent-{mode:?}").to_lowercase(),
            &suite,
            Notes::new(mode),
        );
        let wrong: Vec<(&str, &str)> = verdicts
            .iter()
            .filter(|(_, status)| *status != "passed")
            .map(|(id, status)| (id.as_str(), status.as_str()))
            .collect();
        assert_eq!(wrong, expected, "{mode:?}");
    }
}

/// A Go target without `AbsentInputTarget` skips the one scenario that sends no input, as a Rust
/// target keeping the method's default reports it unsupported — never passed, never the suite.
#[test]
fn a_go_target_without_the_absent_input_method_skips_only_that_scenario() {
    let (rust, replayed) = support_go::compare_with(
        "absent-bare",
        &suite(),
        Notes::new(Mode::Correct),
        &support_go::Options {
            bare: true,
            ..support_go::Options::default()
        },
    );
    assert_eq!(support_go::not_passed(&rust), Vec::<&str>::new());
    assert_eq!(
        support_go::not_passed(&replayed.go.outcomes),
        vec![ID],
        "{}",
        replayed.go.log
    );
    assert_eq!(replayed.go.outcomes[ID], "skipped");
    assert!(
        replayed
            .go
            .log
            .contains("does not implement AbsentInputTarget"),
        "{}",
        replayed.go.log
    );
}
