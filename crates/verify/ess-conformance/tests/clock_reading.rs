//! Reading conformance checks actual event values and separately observed scoped clock facts.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_conformance::{
    reading::{ReadingObservationRequest, ReadingOrder, ReadingReference},
    report::Status,
    scenario::{ConformanceScenario, ConformanceSuite, ScenarioStep, SuiteProvenance},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{reading::*, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::cell::RefCell;

fn fixture() -> EssIr {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/clock-reading.yaml");
    let specification = Specification::assemble([(
        Source::new("reading.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("reading.yaml", text);
    compile_locating(&specification, &sources, &["reading.yaml"]).unwrap()
}
fn reference(field: &str) -> ReadingReference {
    ReadingReference::from_event(
        &fixture(),
        "chronology.reading.Sampled".parse().unwrap(),
        0,
        field,
    )
    .unwrap()
}
fn suite() -> ConformanceSuite {
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&fixture()));
    suite.provenance.suite_version = "ess-conformance/6".parse().unwrap();
    suite
        .insert(
            "chronology.reading/authored/clock-reading".parse().unwrap(),
            ConformanceScenario::new(
                "compare actual reading coordinates".parse().unwrap(),
                vec![
                    ScenarioStep::EventuallyEvent {
                        event: "chronology.reading.Sampled".parse().unwrap(),
                        payload: std::collections::BTreeMap::default(),
                        shape: ess_conformance::scenario::PayloadShape::default(),
                    },
                    ScenarioStep::ExpectReadingOrder {
                        left: reference("offset"),
                        right: reference("seconds"),
                        order: ReadingOrder::Equal,
                    },
                    ScenarioStep::ExpectReadingOrder {
                        left: reference("local"),
                        right: reference("seconds"),
                        order: ReadingOrder::Equal,
                    },
                ],
                [],
            ),
        )
        .unwrap();
    suite
}
struct Observing {
    context: RefCell<String>,
    wrong: &'static str,
}
impl Observing {
    fn new(wrong: &'static str) -> Self {
        Self {
            context: RefCell::new(String::new()),
            wrong,
        }
    }
}
impl ConformanceTarget for Observing {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("clock-fixture", "1"))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        *self.context.borrow_mut() = context.correlation.to_string();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.context.borrow_mut().clear();
        Ok(())
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported("command", "fixture observes only"))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "fixture observes only"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "fixture observes only",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "fixture observes only",
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(vec![ObservedEvent::new(request.event)
            .with("offset", Node::Text("2026-09-11T12:00:00+02:00".into()))
            .with("seconds", serde_json::from_str("1789120800").unwrap())
            .with(
                "local",
                Node::Text("2026-09-11T12:00:00.000Z".into()),
            )])
    }
    fn observe_clock_reading(
        &self,
        request: ReadingObservationRequest,
    ) -> Result<ReadingEvidence, TargetError> {
        // The adapter validates the contract from its model before returning actual fixture facts.
        let actual = reference(&request.reading.field);
        if request.reading != actual || request.correlation.to_string() != *self.context.borrow() {
            return Err(TargetError::unsupported(
                "clock reading",
                "declaration or current scope mismatch",
            ));
        }
        let formatter = match request.reading.field.as_str() {
            "offset" => FormatterObservation::EncodedOffset,
            "seconds" => FormatterObservation::UnixSeconds,
            _ => FormatterObservation::FixedOffset { minutes: 120 },
        };
        let mut evidence = ReadingEvidence {
            correlation: self.context.borrow().clone(),
            occurrence: request.reading.occurrence_key(),
            source: Some(SourceEpoch {
                process_instance: "process-1".into(),
                epoch: "epoch-1".into(),
            }),
            origin: ReadingRole::ProducerProcess,
            formatter,
        };
        match self.wrong {
            "unknown-source" => evidence.source = None,
            "unknown-origin" => evidence.origin = ReadingRole::Unknown,
            "stale-correlation" => evidence.correlation = "old".into(),
            "stale-occurrence" => evidence.occurrence = "old".into(),
            "source" if request.reading.field == "seconds" => {
                evidence.source.as_mut().unwrap().process_instance = "process-2".into();
            }
            "epoch" if request.reading.field == "seconds" => {
                evidence.source.as_mut().unwrap().epoch = "epoch-2".into();
            }
            "unknown-formatter" if request.reading.field == "local" => {
                evidence.formatter = FormatterObservation::Unknown;
            }
            "wrong-offset" if request.reading.field == "local" => {
                evidence.formatter = FormatterObservation::FixedOffset { minutes: 0 }
            }
            _ => {}
        }
        Ok(evidence)
    }
}
fn status(wrong: &'static str) -> Status {
    let admitted = AdmittedSuite::from_json(&suite().to_canonical_json().unwrap()).unwrap();
    let result =
        Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Observing::new(wrong));
    result.scenarios[0].status
}
#[test]
fn actual_observed_offset_unix_and_local_coordinates_pass_in_the_runner() {
    assert_eq!(status(""), Status::Passed);
}
#[test]
fn wrong_observed_fixed_offset_fails_the_coordinate_claim() {
    assert_eq!(status("wrong-offset"), Status::Failed);
}
#[test]
fn absent_or_incompatible_clock_authority_is_unsupported_never_passed() {
    for wrong in [
        "unknown-source",
        "unknown-origin",
        "source",
        "epoch",
        "unknown-formatter",
    ] {
        assert_eq!(status(wrong), Status::Unsupported, "{wrong}");
    }
}
#[test]
fn evidence_from_another_scenario_or_occurrence_fails() {
    for wrong in ["stale-correlation", "stale-occurrence"] {
        assert_eq!(status(wrong), Status::Error, "{wrong}");
    }
}
#[test]
fn persisted_reading_contract_is_closed_and_old_suite_versions_refuse_it() {
    let json = suite().to_canonical_json().unwrap();
    AdmittedSuite::from_json(&json).unwrap();
    for old in 1..=5 {
        assert!(AdmittedSuite::from_json(
            &json.replace("ess-conformance/6", &format!("ess-conformance/{old}"))
        )
        .is_err());
    }
    for mutation in ["trusted", "occurrence", "order"] {
        let mut document: serde_json::Value = serde_json::from_str(&json).unwrap();
        let step = &mut document["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap()["steps"][1];
        match mutation {
            "trusted" => step["left"]["contract"]["trusted"] = serde_json::json!(true),
            "occurrence" => step["left"]["occurrence"] = serde_json::json!(65536),
            _ => step["order"] = serde_json::json!("elapsed"),
        }
        assert!(
            AdmittedSuite::from_json(&serde_json::to_string(&document).unwrap()).is_err(),
            "{mutation}"
        );
    }
    if let Some(directory) = std::env::var_os("ESS_CLOCK_CONFORMANCE_OUT") {
        let root = std::path::PathBuf::from(&directory);
        std::fs::create_dir_all(root.join("essconform")).unwrap();
        std::fs::write(root.join("go.mod"), "module clock-conformance\n\ngo 1.26\n").unwrap();
        std::fs::write(
            root.join("essconform/clock_test.go"),
            include_str!("fixtures/clock-runtime.go"),
        )
        .unwrap();
        std::fs::write(
            root.join("essconform/clock_authority_adversary_test.go"),
            include_str!("fixtures/clock-authority-adversary.go"),
        )
        .unwrap();
        for artifact in ess_conformance::go::emit(&suite()).unwrap() {
            let path = std::path::PathBuf::from(&directory).join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
    }
}
#[test]
fn standalone_suite_contract_is_not_an_adapter_certificate() {
    let mut suite = suite();
    let steps = &mut suite.scenarios.values_mut().next().unwrap().steps;
    let ScenarioStep::ExpectReadingOrder { left, .. } = &mut steps[1] else {
        panic!()
    };
    left.contract.origins[0].role = ReadingRole::ConsumerProcess;
    let admitted = AdmittedSuite::from_json(&suite.to_canonical_json().unwrap()).unwrap();
    let result = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Observing::new(""));
    assert_eq!(result.scenarios[0].status, Status::Unsupported);
}

#[test]
fn fresh_reading_suites_select6_and_legacy_suites_keep4() {
    let mut reading = suite();
    reading.provenance.suite_version = "ess-conformance/4".parse().unwrap();
    reading.select_fresh_format();
    assert_eq!(reading.provenance.suite_version.major(), 6);
    for scenario in reading.scenarios.values_mut() {
        scenario
            .steps
            .retain(|step| !matches!(step, ScenarioStep::ExpectReadingOrder { .. }));
    }
    reading.select_fresh_format();
    assert_eq!(reading.provenance.suite_version.major(), 4);
}
