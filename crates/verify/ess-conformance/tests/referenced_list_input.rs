//! An accepted scenario arranges the rows a `references` relation carried by an input field names
//! (`docs/design/referenced-input-arrangement.md`): a creating command writes `sources` from its
//! input, the model relates `sources` to `Source` rows, and the scenario the suite calls accepted
//! sends only identities an earlier `AddSource` step created.

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, InstanceName, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const ADD: &str = "demo.reg.AddSource";
const REGISTER: &str = "demo.reg.Register";
const ACCEPTED: &str = "demo.reg.Register/outcome/accepted";

const HEAD: &str = r#"
format: ess/30
system: demo
version: v1
domain: demo.reg
types:
  - {name: demo.reg.ServerId, kind: newtype, of: Uuid}
  - {name: demo.reg.SourceId, kind: newtype, of: Uuid}
entities:
  - name: demo.reg.Source
    identity: {name: id, type: demo.reg.SourceId}
    fields: []
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: demo.reg.Server
    identity: {name: id, type: demo.reg.ServerId}
    fields:
      - {name: sources, type: "FIELD_TYPE"}
    relations:
      - {name: sources, kind: references, target: demo.reg.Source, cardinality: CARDINALITY, via: sources}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
events:
  - name: demo.reg.SourceAdded
    fields: [{name: id, type: demo.reg.SourceId}]
  - name: demo.reg.Registered
    fields: [{name: id, type: demo.reg.ServerId}]
actors:
  - name: demo.reg.Operator
    may: [ACTOR_MAY]
commands:
"#;

const ADD_SOURCE: &str = r"
  - name: demo.reg.AddSource
    input: [{name: id, type: demo.reg.SourceId}]
    outcomes:
      - name: added
        creates: demo.reg.Source
        instance: id
        emits: [demo.reg.SourceAdded]
        payload: {demo.reg.SourceAdded: {id: input.id}}
";

const REGISTER_COMMAND: &str = r#"
  - name: demo.reg.Register
    input:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "FIELD_TYPE"}
    outcomes:
      - name: accepted
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
views:
  - name: demo.reg.Servers
    source: demo.reg.Server
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "FIELD_TYPE"}
"#;

/// The model: `many` is a `List` of identities, `one` a plain one; `route` keeps `AddSource`.
fn model(many: bool, route: bool) -> EssIr {
    let (field, cardinality) = if many {
        ("List<demo.reg.SourceId>", "many")
    } else {
        ("demo.reg.SourceId", "one")
    };
    let granted = if route {
        "demo.reg.AddSource, demo.reg.Register"
    } else {
        "demo.reg.Register"
    };
    let text = format!(
        "{}{}{}",
        HEAD,
        if route { ADD_SOURCE } else { "" },
        REGISTER_COMMAND
    )
    .replace("FIELD_TYPE", field)
    .replace("CARDINALITY", cardinality)
    .replace("ACTOR_MAY", granted);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("reg.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> Option<&'a ConformanceScenario> {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// The `sources` input of the `Register` step of `scenario`, with the instances captured after
/// each earlier `AddSource` step: its `id` is what the capture binds.
fn register_sources(scenario: &ConformanceScenario) -> (ScenarioValue, Vec<InstanceName>) {
    let mut added: Vec<InstanceName> = Vec::new();
    let mut after_add = false;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                after_add = command.to_string() == ADD;
                if command.to_string() == REGISTER {
                    return (input["sources"].clone(), added);
                }
            }
            ScenarioStep::CaptureInstance { instance, .. } if after_add => {
                added.push(instance.clone());
            }
            _ => {}
        }
    }
    panic!("no Register step in {:#?}", scenario.steps);
}

fn instances(value: &ScenarioValue) -> Vec<InstanceName> {
    let items = match value {
        ScenarioValue::List { items } => items.as_slice(),
        single => std::slice::from_ref(single),
    };
    items
        .iter()
        .map(|item| match item {
            ScenarioValue::Instance { instance } => instance.clone(),
            other => panic!("an identity no step arranged: {other:#?}"),
        })
        .collect()
}

#[test]
fn an_accepted_scenario_arranges_every_referenced_list_member() {
    let synthesis = synthesize(&model(true, true));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (sent, added) = register_sources(accepted);
    let named = instances(&sent);
    assert!(!named.is_empty(), "a non-empty list: {sent:#?}");
    for name in &named {
        assert!(
            added.contains(name),
            "{name:?} is the identity of no earlier AddSource step: {added:#?}"
        );
    }
}

#[test]
fn an_unarrangeable_reference_list_sends_an_empty_list() {
    let synthesis = synthesize(&model(true, false));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (sent, added) = register_sources(accepted);
    assert_eq!(added, Vec::<InstanceName>::new(), "no AddSource step");
    assert_eq!(sent, ScenarioValue::literal(Node::Seq(Vec::new())));
}

#[test]
fn an_unarrangeable_one_cardinality_reference_is_refused_by_name() {
    let synthesis = synthesize(&model(false, false));
    assert!(
        scenario(&synthesis, ACCEPTED).is_none(),
        "no accepted scenario sends a reference to no row"
    );
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(ACCEPTED)
        })
        .unwrap_or_else(|| panic!("no refusal of {ACCEPTED}: {:#?}", synthesis.refusals));
    assert!(
        refusal.to_string().contains("demo.reg.Server.sources"),
        "the refusal names the relation: {refusal}"
    );
}

#[test]
fn a_one_cardinality_reference_by_input_is_arranged_too() {
    let synthesis = synthesize(&model(false, true));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (sent, added) = register_sources(accepted);
    let named = instances(&sent);
    assert_eq!(named.len(), 1, "one identity: {sent:#?}");
    assert!(
        added.contains(&named[0]),
        "{:?} is the identity of no earlier AddSource step: {added:#?}",
        named[0]
    );
}

#[test]
fn the_arranged_suites_run_on_the_interpreter() {
    for many in [true, false] {
        let model = model(many, true);
        let synthesis = synthesize(&model);
        assert!(
            synthesis.refusals.is_empty(),
            "nothing is refused: {:#?}",
            synthesis.refusals
        );
        let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(model));
        let failed: Vec<_> = run
            .scenarios
            .iter()
            .filter(|result| result.status != Status::Passed)
            .collect();
        assert!(failed.is_empty(), "{failed:#?}");
    }
}
