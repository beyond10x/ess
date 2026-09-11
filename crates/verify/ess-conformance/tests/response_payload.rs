//! Response values are observed from the implementation, never synthesized as expected values.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::Status, target::*, AdmittedSuite, ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;
const MODEL: &str = include_str!("fixtures/response-payload.yaml");
fn ir() -> EssIr {
    ir_text(MODEL)
}
fn ir_text(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("response.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn suite() -> ConformanceSuite {
    let result = ess_conformance::synthesize::synthesize(&ir());
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    result.suite
}
fn body() -> BTreeMap<String, Node> {
    serde_json::from_str(r#"{"item":{"remaining":37,"created":"2026-09-11T10:00:00Z","ended":"2026-09-11T10:01:00Z","state":"Ready","call_type":"incoming","features":{"enabled":true}}}"#).unwrap()
}
struct Backend(&'static str);
impl ConformanceTarget for Backend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("response-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let outcome =
            ess_compiler::refs::OutcomeRef::new(request.command, "cancelled".parse().unwrap());
        let mut result = SemanticCommandResult::took(outcome);
        let mut response = body();
        let mut payload = response.clone();
        payload.insert("receipt".into(), Node::Text("generated-37".into()));
        if self.0 == "wrong-value" {
            if let Some(Node::Map(item)) = response.get_mut("item") {
                item.insert("remaining".into(), serde_json::from_str("38").unwrap());
            }
        }
        if self.0 == "wrong-map" {
            response=serde_json::from_str(r#"{"item":{"remaining":37,"created":"2026-09-11T10:00:00Z","ended":"2026-09-11T10:01:00Z","state":"Ready","call_type":"incoming","features":{"enabled":"true"}}}"#).unwrap();
            payload = response.clone();
            payload.insert("receipt".into(), Node::Text("generated-37".into()));
        }
        if self.0 == "null-item" {
            response.insert("item".into(), Node::Null);
            payload.insert("item".into(), Node::Null);
        }
        if self.0 == "missing-optional" {
            response.clear();
            payload.remove("item");
        }
        if self.0 == "missing-field" {
            response.clear();
        }
        if self.0 == "extra" {
            response.insert("undeclared".into(), Node::Null);
        }
        if self.0 != "missing-response" {
            result.response = Some(response);
        }
        let mut event = ObservedEvent::new("demo.api.Returned".parse().unwrap());
        event.payload = payload;
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "unused"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "unused"))
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
fn status(suite: &ConformanceSuite, mode: &'static str) -> Status {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Backend(mode))
        .scenarios[0]
        .status
}
#[test]
fn actual_typed_response_and_generated_payload_pass() {
    assert_eq!(status(&suite(), "valid"), Status::Passed);
}
#[test]
fn response_mutations_and_missing_data_fail() {
    for mode in [
        "wrong-value",
        "wrong-map",
        "missing-field",
        "missing-response",
        "extra",
    ] {
        assert_eq!(status(&suite(), mode), Status::Failed, "{mode}");
    }
}
#[test]
fn exact_invocation_response_cannot_be_replaced_by_stale_observations() {
    let mut suite = suite();
    let scenario = suite.scenarios.values_mut().next().unwrap();
    let step = scenario
        .steps
        .iter_mut()
        .find(|s| matches!(s, ScenarioStep::ExpectResponsePayload { .. }))
        .unwrap();
    if let ScenarioStep::ExpectResponsePayload { response } = step {
        response.command = "demo.api.Other".parse().unwrap();
        response.outcome.command = response.command.clone();
    }
    assert_eq!(status(&suite, "valid"), Status::Failed);
}
#[test]
fn response_persistence_versions_and_runtime_artifacts() {
    let suite = suite();
    assert_eq!(suite.provenance.suite_version.major(), 8);
    let json = suite.to_canonical_json().unwrap();
    AdmittedSuite::from_json(&json).unwrap();
    for major in 1..8 {
        assert!(AdmittedSuite::from_json(
            &json.replace("ess-conformance/8", &format!("ess-conformance/{major}"))
        )
        .is_err());
    }
    let mut document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let steps = document["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["steps"]
        .as_array_mut()
        .unwrap();
    let response = &mut steps
        .iter_mut()
        .find(|s| s["step"] == "expect_response_payload")
        .unwrap()["response"];
    response["trusted"] = true.into();
    assert!(AdmittedSuite::from_json(&serde_json::to_string(&document).unwrap()).is_err());
    if let Some(root) = std::env::var_os("ESS_RESPONSE_RUNTIME_OUT") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(root.join("essconform")).unwrap();
        std::fs::write(
            root.join("go.mod"),
            "module response-conformance\n\ngo 1.26\n",
        )
        .unwrap();
        std::fs::write(root.join("suite.json"), json).unwrap();
        std::fs::write(
            root.join("essconform/response_test.go"),
            include_str!("fixtures/response-runtime.go"),
        )
        .unwrap();
        for artifact in ess_conformance::go::emit(&suite).unwrap() {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
    }
}

#[test]
fn nullable_response_field_preserves_null_and_absence() {
    let optional = MODEL.replace("type: demo.api.Item", "type: Optional<demo.api.Item>");
    let synthesis = ess_conformance::synthesize::synthesize(&ir_text(&optional));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    for mode in ["null-item", "missing-optional"] {
        assert_eq!(status(&synthesis.suite, mode), Status::Passed, "{mode}");
    }
    assert_eq!(status(&suite(), "null-item"), Status::Failed);
}
#[test]
fn response_schema_and_provenance_include_named_response_fields() {
    let ir = ir();
    let artifacts = ess_gen::artifact::run(&ess_gen::schema::JsonSchema, &ir).unwrap();
    let schema = &artifacts["schema/responses/demo.api.Cancel.schema.json"].contents;
    assert!(schema.contains("command-response"));
    assert!(schema.contains("demo.api.Item"));
    assert!(schema.contains("remaining"));
    ess_gen::artifact::validate_paths(artifacts.keys().map(String::as_str)).unwrap();
    let changed = ir_text(&MODEL.replace(
        "name: remaining, type: Integer",
        "name: remaining, type: String",
    ));
    let before = ess_gen::Provenance::of(&ir);
    let after = ess_gen::Provenance::of(&changed);
    assert_ne!(before.contract_digest, after.contract_digest);
}

#[test]
fn browser_response_vocabulary_is_explicitly_refused() {
    let error = ess_conformance::web::emit(&ir(), &suite()).unwrap_err();
    assert!(error
        .to_string()
        .contains("browser replay does not support observed command response payloads"));
}
