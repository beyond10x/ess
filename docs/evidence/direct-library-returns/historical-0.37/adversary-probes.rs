use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{report::Status, target::*, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

struct Library(BTreeMap<String, Node>);

impl ConformanceTarget for Library {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("nested-presence-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> { Ok(()) }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> { Ok(()) }
    fn execute_command(&self, request: SemanticCommandRequest) -> Result<SemanticCommandResult, TargetError> {
        let mut result = SemanticCommandResult::took(ess_conformance::scenario::OutcomeRef::new(request.command, "returned".parse().unwrap()));
        result.response = Some(self.0.clone());
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> { panic!("no view") }
    fn observe_events(&self, _: EventObservationRequest) -> Result<Vec<ObservedEvent>, TargetError> { panic!("no events") }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> { panic!("no external") }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> { panic!("no events") }
}

fn model(policy: &str) -> ess_compiler::EssIr {
    let model = format!(r"format: ess/16
system: library
version: v1
domain: library.api
types:
  - name: library.api.Item
    kind: struct
    fields:
      - {{name: note, type: 'Optional<String>', presence: {policy}}}
commands:
  - name: library.api.Read
    response:
      - {{name: item, type: library.api.Item}}
    outcomes:
      - name: returned
        returns: true
");
    let specification = Specification::assemble([(Source::new("presence.yaml"), RawSpecFile::parse(&model).unwrap())]).unwrap();
    compile(&specification, &SourceMap::new()).unwrap()
}

fn run(policy: &str, nested: BTreeMap<String, Node>) -> Status {
    let ir = model(policy);
    let generated = ess_conformance::synthesize(&ir);
    assert!(generated.refusals.is_empty(), "{:?}", generated.refusals);
    let admitted = AdmittedSuite::from_suite(&generated.suite).unwrap();
    let target = Library(BTreeMap::from([("item".to_owned(), Node::Map(nested))]));
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(run.scenarios.len(), 1);
    run.scenarios[0].status.clone()
}

#[test]
fn nested_null_when_absent_requires_explicit_null() {
    assert_eq!(run("null_when_absent", BTreeMap::new()), Status::Failed);
}

#[test]
fn nested_omitted_when_absent_rejects_explicit_null() {
    assert_eq!(run("omitted_when_absent", BTreeMap::from([("note".to_owned(), Node::Null)])), Status::Failed);
}

fn authored(policy: &str, response: &str) -> ess_conformance::authored::Authoring {
    let source = format!(r"type: ess-scenario/4
domain: library.api
scenario: nested-presence
summary: Named response object literals honor each declared presence policy.
timeline:
  - at: 2026-09-28T00:00:00Z
    command: library.api.Read
    outcome: returned
    response: {response}
");
    ess_conformance::authored::compile(&model(policy), &[ess_conformance::authored::Source::new("presence.yaml", source)])
}

#[test]
fn authored_named_literal_requires_nested_null_when_absent() {
    let compiled = authored("null_when_absent", "{item: {}}");
    assert_eq!(compiled.scenarios.len(), 0);
    assert_eq!(compiled.refusals.len(), 1);
}

#[test]
fn authored_named_literal_rejects_nested_null_when_omitted() {
    let compiled = authored("omitted_when_absent", "{item: {note: null}}");
    assert_eq!(compiled.scenarios.len(), 0);
    assert_eq!(compiled.refusals.len(), 1);
}

#[test]
fn valid_presence_and_partial_top_level_literals_remain_admitted() {
    assert_eq!(run("null_when_absent", BTreeMap::from([("note".to_owned(), Node::Null)])), Status::Passed);
    assert_eq!(run("omitted_when_absent", BTreeMap::new()), Status::Passed);
    for (policy, response) in [("null_when_absent", "{item: {note: null}}"), ("omitted_when_absent", "{item: {}}"), ("null_when_absent", "{}"), ("omitted_when_absent", "{}")] {
        let compiled = authored(policy, response);
        assert_eq!(compiled.scenarios.len(), 1);
        assert_eq!(compiled.refusals.len(), 0);
    }
}

fn run_json(value: Node) -> Status {
    let specification = Specification::assemble([(Source::new("json.yaml"), RawSpecFile::parse(r"format: ess/16
system: library
version: v1
domain: library.api
commands:
  - name: library.api.Read
    response:
      - {name: item, type: Json}
    outcomes:
      - {name: returned, returns: true}
").unwrap())]).unwrap();
    let ir = compile(&specification, &SourceMap::new()).unwrap();
    let generated = ess_conformance::synthesize(&ir);
    assert!(generated.refusals.is_empty(), "{:?}", generated.refusals);
    let admitted = AdmittedSuite::from_suite(&generated.suite).unwrap();
    let target = Library(BTreeMap::from([("item".to_owned(), value)]));
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(run.scenarios.len(), 1);
    run.scenarios[0].status.clone()
}

#[test]
fn json_return_collection_limit_is_enforced() {
    assert_eq!(run_json(Node::Seq(vec![Node::Null; 65_537])), Status::Failed);
}

#[test]
fn json_return_depth_limit_is_enforced() {
    let mut value = Node::Null;
    for _ in 0..130 { value = Node::Seq(vec![value]); }
    assert_eq!(run_json(value), Status::Failed);
}
