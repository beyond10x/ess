//! Runtime witnesses distinguish equal input in different held states.
use ess_compiler::refs::OutcomeRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::ConformanceStatus, target::*, AdmittedSuite, ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

const MODEL: &str = include_str!("fixtures/subject-state.yaml");
const ID: &str = "00000000-0000-4000-8000-000000000001";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("subject-state.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let result = ess_conformance::synthesize::synthesize(ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    result.suite
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Behavior {
    Good,
    IgnoreGuard,
    Empty,
    WrongIdentity,
}

struct Backend {
    behavior: Behavior,
    row: RefCell<Option<ViewRow>>,
    sequence: Cell<u64>,
}

impl Backend {
    fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            row: RefCell::default(),
            sequence: Cell::new(0),
        }
    }
}

impl ConformanceTarget for Backend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("subject-state-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.row.replace(None);
        self.sequence.set(0);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.row.replace(None);
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut row = self.row.borrow_mut();
        let command = request.command.to_string();
        let (branch, event) = match command.as_str() {
            "calls.core.Open" => {
                *row = Some(
                    [
                        ("call_id".into(), Node::Text(ID.into())),
                        ("state".into(), Node::Text("Init".into())),
                        ("note".into(), request.input["note"].clone()),
                    ]
                    .into_iter()
                    .collect(),
                );
                (
                    "opened",
                    Some(
                        ObservedEvent::new("calls.core.Opened".parse().unwrap())
                            .with("call_id", Node::Text(ID.into())),
                    ),
                )
            }
            "calls.core.Bridge" | "calls.core.Report" => {
                let Some(row) = row.as_mut() else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                if request.input.get("call_id") != row.get("call_id") {
                    return Ok(SemanticCommandResult::undeclared());
                }
                let held = row["state"].clone();
                if command == "calls.core.Bridge" {
                    if held == Node::Text("Bridged".into()) {
                        ("already-bridged", None)
                    } else {
                        row.insert("state".into(), Node::Text("Bridged".into()));
                        (
                            "bridged",
                            Some(ObservedEvent::new("calls.core.Observed".parse().unwrap())),
                        )
                    }
                } else {
                    row.insert("note".into(), request.input["note"].clone());
                    let branch = if request.input["incoming"] == Node::Text("Ringing".into()) {
                        if held == Node::Text("Bridged".into()) {
                            if self.behavior == Behavior::IgnoreGuard {
                                row.insert("state".into(), Node::Text("Ringing".into()));
                            }
                            "preserved"
                        } else if held == Node::Text("Init".into()) {
                            row.insert("state".into(), Node::Text("Ringing".into()));
                            "ringing"
                        } else {
                            "refreshed"
                        }
                    } else {
                        "enriched"
                    };
                    (
                        branch,
                        Some(ObservedEvent::new("calls.core.Observed".parse().unwrap())),
                    )
                }
            }
            _ => return Err(TargetError::unsupported("command", command)),
        };
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            OutcomeName::new(branch).unwrap(),
        ));
        if let Some(event) = event {
            result = result.emitting(event);
        }
        self.sequence.set(self.sequence.get() + 1);
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!(
                "seq:{}",
                self.sequence.get()
            ))
            .unwrap(),
        ))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut rows: Vec<_> = self.row.borrow().iter().cloned().collect();
        if self.behavior == Behavior::Empty {
            rows.clear();
        }
        if self.behavior == Behavior::WrongIdentity {
            for row in &mut rows {
                row.insert(
                    "call_id".into(),
                    Node::Text("00000000-0000-4000-8000-000000000002".into()),
                );
            }
        }
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("held state cannot be injected as an outcome")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

#[test]
fn runtime_observations_defeat_guard_ignoring_and_wrong_identity_targets() {
    let ir = ir(MODEL);
    let suite = suite(&ir);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    for behavior in [
        Behavior::Good,
        Behavior::IgnoreGuard,
        Behavior::Empty,
        Behavior::WrongIdentity,
    ] {
        let report = Runner::for_suite(&suite)
            .run_admitted(&admitted, &Backend::new(behavior))
            .into_report();
        assert_eq!(
            report.status == ConformanceStatus::Passed,
            behavior == Behavior::Good,
            "{behavior:?}: {report:?}"
        );
        if behavior == Behavior::IgnoreGuard {
            let failed: Vec<_> = report
                .scenarios
                .iter()
                .filter(|scenario| scenario.status != ess_conformance::report::Status::Passed)
                .map(|scenario| scenario.scenario.to_string())
                .collect();
            assert_eq!(failed, ["calls.core.Report/outcome/preserved"]);
        }
    }
}

#[test]
fn absent_or_different_subject_is_never_created_or_selected_as_initial() {
    for held in [
        None,
        Some(
            [
                ("call_id".into(), Node::Text(ID.into())),
                ("state".into(), Node::Text("Bridged".into())),
                ("note".into(), Node::Text("original".into())),
            ]
            .into_iter()
            .collect(),
        ),
    ] {
        let backend = Backend::new(Behavior::Good);
        backend.row.replace(held.clone());
        let request = SemanticCommandRequest {
            command: "calls.core.Report".parse().unwrap(),
            actor: None,
            correlation: ess_primitives::ids::CorrelationId::new("subject-absence").unwrap(),
            input: [
                (
                    "call_id".into(),
                    Node::Text("00000000-0000-4000-8000-000000000099".into()),
                ),
                ("incoming".into(), Node::Text("Ringing".into())),
                ("note".into(), Node::Text("replacement".into())),
            ]
            .into_iter()
            .collect(),
        };
        let result = backend.execute_command(request).unwrap();
        assert!(result.outcome.is_none());
        assert!(result.direct_events.is_empty());
        assert_eq!(*backend.row.borrow(), held);
    }
}

#[test]
fn same_ringing_input_has_different_established_state_and_no_fault_injection() {
    let suite = suite(&ir(MODEL));
    let mut inputs = BTreeMap::new();
    for (id, scenario) in &suite.scenarios {
        let text = id.to_string();
        if text == "calls.core.Report/outcome/ringing"
            || text == "calls.core.Report/outcome/preserved"
        {
            assert!(!scenario
                .steps
                .iter()
                .any(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. })));
            let input = scenario
                .steps
                .iter()
                .filter_map(|step| match step {
                    ScenarioStep::ExecuteCommand { command, input, .. }
                        if command.to_string() == "calls.core.Report" =>
                    {
                        Some(input.clone())
                    }
                    _ => None,
                })
                .next_back()
                .unwrap();
            inputs.insert(text, input);
        }
    }
    assert_eq!(inputs.len(), 2);
    assert_eq!(
        inputs.values().next().unwrap(),
        inputs.values().nth(1).unwrap()
    );
}

#[test]
fn subject_state_without_an_observable_identity_state_pair_refuses_synthesis() {
    let ir = ir(&MODEL.replace("      - {name: state, type: calls.core.Call.State}\n", ""));
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert!(result.refusals.iter().any(|refusal| refusal
        .to_string()
        .contains("projecting identity and lifecycle state")));
}

#[test]
fn generated_go_runs_the_state_witness_and_rejects_the_guard_ignoring_mutation() {
    let suite = suite(&ir(MODEL));
    let directory =
        std::env::temp_dir().join(format!("ess-subject-state-go-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/subjectstate\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/subject_state_test.go"),
        include_str!("fixtures/subject-state-runtime.go"),
    )
    .unwrap();
    for ignore in [false, true] {
        let output = std::process::Command::new("go")
            .args([
                "test",
                "./essconform",
                "-run",
                "TestSubjectStateRuntime",
                "-count=1",
                "-v",
            ])
            .env("GOMAXPROCS", "2")
            .env(
                "ESS_TEST_IGNORE_SUBJECT_GUARD",
                if ignore { "1" } else { "0" },
            )
            .env("ESS_REPORT_FORMAT", "2")
            .current_dir(&directory)
            .output()
            .expect("required Go toolchain");
        let log = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.success(), !ignore, "ignore={ignore}: {log}");
        if ignore {
            assert!(log.contains("calls.core.Report/outcome/preserved"), "{log}");
            assert!(
                log.contains("`calls.core.Calls` holds no row where call_id")
                    && log.contains("state = \"Bridged\""),
                "mutation must fail the observed state assertion: {log}"
            );
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}
