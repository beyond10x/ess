//! An ordinary typed return is executed by the product Interpreter itself.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{direct_response::Observation, interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::collections::BTreeMap;
#[test]
fn unmarked_typed_response_is_complete_and_declared() {
    let source = r"format: ess/17
system: library
version: v1
domain: library.api
types:
  - name: library.api.Item
    kind: struct
    fields: [{name: label, type: String}, {name: ordinal, type: Integer}]
commands:
  - name: library.api.Read
    response:
      - {name: value, type: String}
      - {name: sequence, type: 'List<Integer>'}
      - {name: item, type: library.api.Item}
    outcomes: [{name: returned, returns: true}]
";
    let specification = Specification::assemble([(
        Source::new("library.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let ir = compile(&specification, &SourceMap::new()).unwrap();
    let command = ir.commands().values().next().unwrap();
    let observation = Observation::of(&ir, command, None, BTreeMap::new()).unwrap();
    let request = SemanticCommandRequest {
        command: ess_conformance::scenario::CommandRef::new(command.name.clone()),
        actor: None,
        caller: None,
        input: BTreeMap::new(),
        correlation: ess_primitives::ids::CorrelationId::new("response-test").unwrap(),
    };
    let target = Interpreted::for_model(ir);
    let result = target.execute_command(request).unwrap();
    assert!(result.outcome.is_some());
    observation.compare(result.response.as_ref()).unwrap();
}
