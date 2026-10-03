//! A command's declared answer for an absent input is witnessed by a scenario that sends no input
//! at all (`input_absent: true`, ess/16; beyond10x/ess#170, `docs/design/outcome-shapes.md`).
//!
//! The #170 repro, written with the marker, synthesizes a scenario whose step is
//! `execute_command_without_input` — distinct from `execute_command` with `input: {}`, which the
//! implementation the issue describes answers differently. The suite takes `ess-conformance/26`.
//! One in-memory target implements the behaviour the issue describes; each wrong mode breaks it in
//! one way, and exactly the scenario about the absent input fails.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
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

// ---- the behaviour the issue describes, and ways to get it wrong ------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// No body answers the declared error before any field is validated.
    Correct,
    /// No body is read as `{}`, which fails field validation with an undeclared answer.
    ReadsAbsentAsEmpty,
    /// No body is answered with the success branch.
    AcceptsAbsent,
    /// The target cannot send a request without a body at all.
    CannotOmit,
}

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
                // `{}`: present, lacking a field — not the declared absent-body answer.
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

/// A target that implements everything but the absent-input method, which keeps its default.
struct Defaulted(Notes);

impl ConformanceTarget for Defaulted {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.0.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(r)
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
fn the_issue_repro_synthesizes_a_scenario_that_sends_no_input_and_requires_the_error() {
    let synthesis = synthesis();
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    let absent = scenario(&synthesis.suite, ID);
    let ScenarioStep::ExecuteCommandWithoutInput { command, actor, .. } = &absent.steps[0] else {
        panic!(
            "the command is sent with no input, arranging nothing: {:#?}",
            absent.steps
        );
    };
    assert_eq!(command.to_string(), "demo.notes.SubmitNote");
    assert_eq!(
        actor.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.notes.Service")
    );
    assert!(absent.steps.iter().any(|step| matches!(step,
        ScenarioStep::ExpectOutcome { outcome } if outcome.to_string() == ID.replace("/outcome/", "/"))));
    assert!(absent.steps.iter().any(|step| matches!(step,
        ScenarioStep::ExpectError { error, .. } if error.to_string() == "demo.notes.BodyMissing")));
    assert!(absent.steps.iter().any(|step| matches!(step,
        ScenarioStep::ExpectNoEvent { event } if event.to_string() == "demo.notes.NoteSubmitted")));
    assert!(
        !absent
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExecuteCommand { .. })),
        "no step sends an input, not even `{{}}`"
    );
}

#[test]
fn the_step_is_distinct_from_an_empty_input_on_the_wire() {
    let synthesis = synthesis();
    let absent = scenario(&synthesis.suite, ID);
    let json = serde_json::to_value(&absent.steps[0]).unwrap();
    assert_eq!(json["step"], "execute_command_without_input");
    assert!(json.get("input").is_none(), "{json}");
    let back: ScenarioStep = serde_json::from_value(json).unwrap();
    assert_eq!(back, absent.steps[0]);

    let empty: ScenarioStep = serde_json::from_str(
        r#"{"step":"execute_command","command":"demo.notes.SubmitNote","input":{}}"#,
    )
    .unwrap();
    assert!(matches!(empty, ScenarioStep::ExecuteCommand { .. }));
}

#[test]
fn every_scenario_passes_against_the_behaviour_the_issue_describes() {
    let synthesis = synthesis();
    let statuses = run(&synthesis.suite, &Notes::new(Mode::Correct));
    assert!(statuses.contains_key(ID), "{statuses:#?}");
    assert!(
        not_passed(&statuses).is_empty(),
        "every scenario passes: {:#?}",
        not_passed(&statuses)
    );
}

#[test]
fn the_interpreter_executes_both_absent_and_supplied_input_scenarios() {
    let ir = ir_of(MODEL);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let statuses = run(
        &suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    assert_eq!(statuses.len(), 3, "{statuses:#?}");
    assert_eq!(statuses[ID], Status::Passed);
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn interpreted_absence_preserves_rows_and_differs_from_empty_and_ungranted_requests() {
    use ess_primitives::{consistency::QueryConsistency, ids::CorrelationId};
    let target = ess_conformance::interpret::Interpreted::for_model(ir_of(MODEL));
    let correlation = CorrelationId::new("absent-control").unwrap();
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.notes/authored/absent-control".parse().unwrap(),
            correlation.clone(),
        ))
        .unwrap();
    let supplied = |input| SemanticCommandRequest {
        command: "demo.notes.SubmitNote".parse().unwrap(),
        actor: Some("demo.notes.Service".parse().unwrap()),
        caller: None,
        input,
        correlation: correlation.clone(),
    };
    let created = target
        .execute_command(supplied(BTreeMap::from([(
            "text".into(),
            Node::Text("kept".into()),
        )])))
        .unwrap();
    assert_eq!(created.direct_events.len(), 1);
    let empty = target.execute_command(supplied(BTreeMap::new()));
    assert!(
        matches!(empty, Err(TargetError::Unavailable { .. })),
        "{empty:?}"
    );
    let absent = AbsentInputRequest {
        command: "demo.notes.SubmitNote".parse().unwrap(),
        actor: Some("demo.notes.Service".parse().unwrap()),
        caller: None,
        correlation: correlation.clone(),
    };
    let refused = target
        .execute_command_without_input(absent.clone())
        .unwrap();
    assert_eq!(
        refused.outcome.unwrap().to_string(),
        "demo.notes.SubmitNote/body-missing"
    );
    assert_eq!(
        refused.error.unwrap().error.to_string(),
        "demo.notes.BodyMissing"
    );
    assert!(refused.direct_events.is_empty());
    assert!(refused.response.is_none());
    assert!(refused.consistency.is_some());
    let undeclared = target
        .execute_command_without_input(AbsentInputRequest {
            command: "demo.notes.RewordNote".parse().unwrap(),
            ..absent.clone()
        })
        .unwrap();
    assert!(undeclared.outcome.is_none());
    assert!(undeclared.error.is_none());
    assert!(undeclared.direct_events.is_empty());
    let ungranted = target.execute_command_without_input(AbsentInputRequest {
        actor: Some("demo.notes.Stranger".parse().unwrap()),
        ..absent
    });
    assert!(
        matches!(ungranted, Err(TargetError::NotGranted { .. })),
        "{ungranted:?}"
    );
    let view = target
        .query_view(SemanticViewRequest {
            view: "demo.notes.NoteDetails".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation,
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .unwrap();
    assert_eq!(view.rows.len(), 1);
    assert_eq!(
        view.rows[0]["note_id"],
        created.direct_events[0].payload["note_id"]
    );
}

#[test]
fn exactly_the_absent_input_scenario_catches_each_wrong_answer() {
    let synthesis = synthesis();
    for mode in [Mode::ReadsAbsentAsEmpty, Mode::AcceptsAbsent] {
        let statuses = run(&synthesis.suite, &Notes::new(mode));
        assert_eq!(not_passed(&statuses), vec![ID], "{mode:?}");
        assert_eq!(statuses[ID], Status::Failed, "{mode:?}");
    }
}

#[test]
fn a_target_that_cannot_omit_the_input_is_unsupported_never_passed() {
    let synthesis = synthesis();
    let statuses = run(&synthesis.suite, &Notes::new(Mode::CannotOmit));
    assert_eq!(not_passed(&statuses), vec![ID]);
    assert_eq!(statuses[ID], Status::Unsupported);

    let statuses = run(&synthesis.suite, &Defaulted(Notes::new(Mode::Correct)));
    assert_eq!(not_passed(&statuses), vec![ID]);
    assert_eq!(statuses[ID], Status::Unsupported);
}

#[test]
fn a_suite_pinned_below_26_is_refused() {
    let mut suite = synthesis().suite;
    suite.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/25").unwrap();
    let Err(error) = AdmittedSuite::from_suite(&suite) else {
        panic!("a suite/25 carrying the step is refused");
    };
    assert!(
        error.to_string().contains("suite/26"),
        "the refusal names the format: {error}"
    );
}

#[test]
fn a_model_without_the_marker_keeps_its_suite_format() {
    let model = MODEL.replace(
        "      - name: body-missing\n        input_absent: true\n        error: demo.notes.BodyMissing\n",
        "",
    );
    assert_ne!(model, MODEL);
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    assert!(!synthesis
        .suite
        .provenance
        .suite_version
        .to_string()
        .ends_with("/26"));
    assert!(!ess_conformance::absent_input::used_by(&synthesis.suite));
}
