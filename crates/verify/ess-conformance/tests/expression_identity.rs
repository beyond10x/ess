//! #225 as asked (`docs/design/expression-family-source22.md`, final review decision 9): a command
//! that links one task to another refuses the edge from a task to itself, `task_id == depends_on`,
//! where both inputs are identities of arranged `Task` rows.
//!
//! The equal case is witnessed with one arranged instance named twice; the unequal case needs a
//! second arranged instance, because `depends_on` must name a row that exists. Identity tokens
//! compare only by `==` and `!=`.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r"format: ess/22
system: graph
version: v1
domain: graph.deps
types:
  - {name: graph.deps.TaskId, kind: newtype, of: Uuid}
entities:
  - name: graph.deps.Task
    identity: {name: task_id, type: graph.deps.TaskId}
    fields:
      - {name: title, type: String}
      - {name: depends_on, type: Optional<graph.deps.TaskId>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: graph.deps.SelfEdge, summary: A task cannot depend on itself., fields: []}
  - {name: graph.deps.MissingDependency, summary: No task carries that identity., fields: []}
events:
  - name: graph.deps.TaskAdded
    fields: [{name: task_id, type: graph.deps.TaskId}]
  - name: graph.deps.Linked
    fields: [{name: task_id, type: graph.deps.TaskId}]
commands:
  - name: graph.deps.AddTask
    input:
      - {name: title, type: String}
    outcomes:
      - name: added
        creates: graph.deps.Task
        instance: task_id
        sets: {title: input.title}
        emits: [graph.deps.TaskAdded]
        payload: {graph.deps.TaskAdded: {task_id: {generated: true}}}
  - name: graph.deps.Link
    input:
      - {name: task_id, type: graph.deps.TaskId}
      - {name: depends_on, type: graph.deps.TaskId}
    outcomes:
      - name: self-edge
        when: task_id == depends_on
        error: graph.deps.SelfEdge
      - name: missing
        when_related: {via: input.depends_on, exists: false}
        error: graph.deps.MissingDependency
      - name: linked
        updates: graph.deps.Task
        instance: task_id
        sets: {depends_on: input.depends_on}
        emits: [graph.deps.Linked]
        payload: {graph.deps.Linked: {task_id: input.task_id}}
views:
  - name: graph.deps.Tasks
    source: graph.deps.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: graph.deps.TaskId}
      - {name: title, type: String}
      - {name: depends_on, type: Optional<graph.deps.TaskId>}
";

fn ir(text: &str) -> Result<EssIr, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    let spec = Specification::assemble([(Source::new("graph.yaml"), raw)])
        .map_err(|errors| errors.to_string())?;
    compile(&spec, &SourceMap::new()).map_err(|error| format!("{error:?}"))
}

fn link_inputs(suite: &ConformanceSuite, id: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "graph.deps.Link" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a1_identity_tokens_name_one_arranged_instance_when_equal_and_two_when_not() {
    let model = ir(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let suite = synthesis.suite;
    for refusal in &synthesis.refusals {
        eprintln!("refused: {refusal:?}");
    }
    let equal = link_inputs(&suite, "graph.deps.Link/outcome/self-edge");
    let sent = equal.last().expect("the self-edge scenario sends Link");
    assert_eq!(
        sent.get("task_id"),
        sent.get("depends_on"),
        "one arranged instance named twice: {sent:#?}"
    );
    let unequal = link_inputs(&suite, "graph.deps.Link/outcome/linked");
    let sent = unequal.last().expect("the linked scenario sends Link");
    assert_ne!(
        sent.get("task_id"),
        sent.get("depends_on"),
        "a second arranged instance: {sent:#?}"
    );
    let admitted = AdmittedSuite::from_suite(&suite).expect("admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(failing, Vec::<String>::new());
}

#[test]
fn identity_tokens_compare_only_by_equality() {
    for op in ["<", "<=", ">", ">="] {
        let refused = ir(&MODEL.replace(
            "when: task_id == depends_on",
            &format!("when: task_id {op} depends_on"),
        ))
        .expect_err("an identity token has no order");
        assert!(
            refused.contains("identity") && refused.contains("only by `==` and `!=`"),
            "{op}: {refused}"
        );
    }
    ir(&MODEL.replace("when: task_id == depends_on", "when: task_id != depends_on"))
        .expect("inequality is admitted");
}
