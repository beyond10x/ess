use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{direct_response::Observation, report::Status, scenario::CommandRef, target::*, AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::{cell::Cell, collections::BTreeMap};

fn model(kind: &str) -> ess_compiler::EssIr {
    let text = format!("format: ess/17\nsystem: library\nversion: v1\ndomain: library.api\ncommands:\n  - name: library.api.Read\n    response:\n      - {{name: value, type: '{kind}'}}\n    outcomes:\n      - {{name: returned, returns: true}}\n");
    let spec = Specification::assemble([(Source::new("review.yaml"), RawSpecFile::parse(&text).unwrap())]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn generated(kind: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize(&model(kind));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    assert_eq!(synthesis.suite.scenarios.len(), 1);
    synthesis.suite
}

fn response(suite: &mut ConformanceSuite) -> &mut Observation {
    suite.scenarios.values_mut().next().unwrap().steps.iter_mut().find_map(|step| {
        if let ScenarioStep::ExpectDirectResponse { response } = step { Some(response) } else { None }
    }).unwrap()
}

#[test]
fn reviewer_documented_depth128_literal_survives_original_byte_admission() {
    let mut suite = generated("Json");
    let value = (0..128).fold(Node::Null, |inner, _| Node::Seq(vec![inner]));
    let fields = BTreeMap::from([("value".into(), value)]);
    let observation = response(&mut suite);
    observation.expected = fields.clone();
    observation.validate().expect("the documented depth-128 response literal is valid");
    observation.compare(Some(&fields)).expect("actual depth-128 response is within the profile");
    let admitted = AdmittedSuite::from_suite(&suite)
        .expect("a valid direct response contract must survive the actual original-byte boundary");
    assert_eq!(admitted.suite().provenance.suite_version.major(), 28);
}

#[test]
fn reviewer_authored_depth128_literal_survives_compile_and_admission() {
    let literal = format!("{}null{}", "[".repeat(128), "]".repeat(128));
    let source = format!("type: ess-scenario/4\ndomain: library.api\nscenario: depth-boundary\nsummary: A literal at the declared response depth boundary.\ntimeline:\n  - at: 2026-09-28T00:00:00Z\n    command: library.api.Read\n    outcome: returned\n    response: {{value: {literal}}}\n");
    let ir = model("Json");
    let compiled = ess_conformance::authored::compile(&ir, &[ess_conformance::authored::Source::new("depth-boundary.yaml", source)]);
    assert!(compiled.refusals.is_empty(), "the documented literal depth must compile: {:?}", compiled.refusals);
    assert_eq!(compiled.scenarios.len(), 1);
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.scenarios = compiled.scenarios;
    suite.select_fresh_format();
    AdmittedSuite::from_suite(&suite).expect("the authored depth-128 literal must survive admission");
}

struct Target {
    ordinary: Cell<usize>,
    absent: Cell<usize>,
    absent_response: Option<BTreeMap<String, Node>>,
}
impl Target {
    fn result(command: CommandRef, response: Option<BTreeMap<String, Node>>) -> SemanticCommandResult {
        let mut result = SemanticCommandResult::took(ess_conformance::scenario::OutcomeRef::new(command, "returned".parse().unwrap()));
        result.response = response;
        result
    }
}
impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> { Ok(ImplementationIdentity::new("review", "literal-fixture")) }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> { Ok(()) }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> { Ok(()) }
    fn execute_command(&self, request: SemanticCommandRequest) -> Result<SemanticCommandResult, TargetError> {
        self.ordinary.set(self.ordinary.get() + 1);
        Ok(Self::result(request.command, Some(BTreeMap::from([("value".into(), Node::Text("first".into()))]))))
    }
    fn execute_command_without_input(&self, request: AbsentInputRequest) -> Result<SemanticCommandResult, TargetError> {
        self.absent.set(self.absent.get() + 1);
        Ok(Self::result(request.command, self.absent_response.clone()))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> { panic!("no view declared") }
    fn observe_events(&self, _: EventObservationRequest) -> Result<Vec<ObservedEvent>, TargetError> { panic!("no events declared") }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> { panic!("no external binding declared") }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> { panic!("no events declared") }
}

#[test]
fn reviewer_absent_input_uses_its_own_response_even_for_the_same_command() {
    let mut suite = generated("String");
    let steps = &mut suite.scenarios.values_mut().next().unwrap().steps;
    let mut observation = steps.iter().find_map(|step| match step { ScenarioStep::ExpectDirectResponse { response } => Some(response.clone()), _ => None }).unwrap();
    observation.expected = BTreeMap::from([("value".into(), Node::Text("second".into()))]);
    steps.push(ScenarioStep::ExecuteCommandWithoutInput { command: "library.api.Read".parse().unwrap(), actor: None, caller: BTreeMap::new() });
    steps.push(ScenarioStep::ExpectDirectResponse { response: observation });
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    for (value, expected) in [(Some("second"), Status::Passed), (Some("first"), Status::Failed), (None, Status::Failed)] {
        let target = Target { ordinary: Cell::new(0), absent: Cell::new(0), absent_response: value.map(|value| BTreeMap::from([("value".into(), Node::Text(value.into()))])) };
        let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        assert_eq!(target.ordinary.get(), 1);
        assert_eq!(target.absent.get(), 1);
        assert_eq!(run.scenarios.len(), 1);
        assert_eq!(run.scenarios[0].status, expected, "{:?}", run.scenarios);
    }
}

#[test]
fn reviewer_response_byte_bound_counts_json_escaping_and_field_names() {
    let mut suite = generated("String");
    let observation = response(&mut suite);
    let boundary = 1_048_576 - "{\"value\":\"\"}".len();
    for (value, allowed) in [("x".repeat(boundary), true), ("x".repeat(boundary + 1), false), ("\n".repeat(boundary / 2 + 1), false)] {
        let actual = BTreeMap::from([("value".into(), Node::Text(value))]);
        assert_eq!(observation.compare(Some(&actual)).is_ok(), allowed);
    }
}

#[test]
fn reviewer_typed_map_collection_boundary_is_independent_of_byte_bound() {
    let mut suite = generated("Map<String, String>");
    let observation = response(&mut suite);
    for (members, allowed) in [(65_536, true), (65_537, false)] {
        let actual = BTreeMap::from([("value".into(), Node::Map((0..members).map(|index| (index.to_string(), Node::Text(String::new()))).collect()))]);
        assert_eq!(observation.compare(Some(&actual)).is_ok(), allowed);
    }
}

#[test]
fn reviewer_exact_integer_extremes_survive_suite_bytes() {
    for value in [i64::MIN, -9_007_199_254_740_993, 9_007_199_254_740_993, i64::MAX] {
        let mut suite = generated("Integer");
        let fields = BTreeMap::from([("value".into(), Node::Number(value.into()))]);
        response(&mut suite).expected = fields.clone();
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let observation = admitted.suite().scenarios.values().next().unwrap().steps.iter().find_map(|step| match step { ScenarioStep::ExpectDirectResponse { response } => Some(response), _ => None }).unwrap();
        assert_eq!(observation.expected, fields);
        observation.compare(Some(&fields)).unwrap();
        let neighbour = if value == i64::MAX { value - 1 } else { value + 1 };
        let wrong = BTreeMap::from([("value".into(), Node::Number(neighbour.into()))]);
        assert!(observation.compare(Some(&wrong)).is_err());
    }
}

#[test]
fn reviewer_binary64_in_a_forged_direct_contract_is_refused_before_execution() {
    let mut suite = generated("Integer");
    response(&mut suite).fields[0].type_ref = ess_domain::TypeRef::Primitive(ess_domain::Primitive::Binary64);
    let error = AdmittedSuite::from_suite(&suite).unwrap_err();
    assert!(error.to_string().contains("Binary64"), "{error}");
}
