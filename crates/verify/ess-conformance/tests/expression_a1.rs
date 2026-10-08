//! Family F part A1 executed (beyond10x/ess#225): a command refuses an edge from a task to itself
//! with `when: task_id == depends_on`, two inputs of one identity type, under `ess/22`.
//!
//! The suite is synthesized from the model and run against the native interpreter, which must pass
//! it, and against three targets wrong in one way each, which must each fail the scenario that
//! decides the guard: one reading the right side as the text `"depends_on"`, one comparing the
//! left side with itself, and one reading the word in the wrong place (a decoy input of the same
//! type). `docs/design/expression-family-source22.md` is the design.

mod support_scratch;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/22
system: graph
version: v1
domain: graph.tasks
types:
  - {name: graph.tasks.TaskId, kind: newtype, of: Uuid}
entities:
  - name: graph.tasks.Edge
    identity: {name: edge_id, type: Uuid}
    fields:
      - {name: task_id, type: graph.tasks.TaskId}
      - {name: depends_on, type: graph.tasks.TaskId}
    lifecycle: {initial: Linked, states: [Linked], terminal: [Linked]}
errors:
  - name: graph.tasks.SelfEdge
    summary: A task cannot depend on itself.
  - name: graph.tasks.Blocked
    summary: The task is blocked by itself.
events:
  - name: graph.tasks.Linked
    fields:
      - {name: edge_id, type: Uuid}
commands:
  - name: graph.tasks.Link
    input:
      - {name: task_id, type: graph.tasks.TaskId}
      - {name: depends_on, type: graph.tasks.TaskId}
      - {name: blocked_by, type: Optional<graph.tasks.TaskId>}
    outcomes:
      - name: self-edge
        when: task_id == depends_on
        error: graph.tasks.SelfEdge
      - name: blocked
        when: task_id == blocked_by
        error: graph.tasks.Blocked
      - name: linked
        creates: graph.tasks.Edge
        instance: edge_id
        emits: [graph.tasks.Linked]
        payload:
          graph.tasks.Linked: {edge_id: {generated: true}}
        sets: {task_id: input.task_id, depends_on: input.depends_on}
";

const SELF_EDGE: &str = "graph.tasks.Link/outcome/self-edge";
const LINKED: &str = "graph.tasks.Link/outcome/linked";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// How a target decides the self-edge guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// `task_id == "depends_on"`: the right side kept as the text it is spelled like.
    RhsAsText,
    /// `task_id == task_id`: the left side compared with itself.
    LeftVsSelf,
    /// `task_id == blocked_by`: the word read in the wrong place.
    WrongNamespace,
}

/// The interpreter, with the self-edge guard decided by `mode` instead of by the model.
///
/// Every other effect is the interpreter's: the input is moved so that the interpreter takes the
/// branch `mode` decided, and the answer is the interpreter's answer for that input.
struct Edges {
    inner: Interpreted,
    mode: Mode,
}

const OTHER: &str = "00000000-0000-4000-8000-0000000000ff";

impl Edges {
    fn new(mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir()),
            mode,
        }
    }

    fn steer(&self, mut request: SemanticCommandRequest) -> SemanticCommandRequest {
        if self.mode == Mode::Correct || request.command.to_string() != "graph.tasks.Link" {
            return request;
        }
        let task = request.input.get("task_id").cloned();
        let refuse = match self.mode {
            Mode::Correct => unreachable!(),
            Mode::RhsAsText => task == Some(Node::Text("depends_on".to_owned())),
            Mode::LeftVsSelf => task.is_some(),
            Mode::WrongNamespace => {
                task.is_some() && request.input.get("blocked_by") == task.as_ref()
            }
        };
        match (refuse, task) {
            (true, Some(task)) => {
                request.input.insert("depends_on".to_owned(), task);
            }
            (false, Some(task)) if request.input.get("depends_on") == Some(&task) => {
                request
                    .input
                    .insert("depends_on".to_owned(), Node::Text(OTHER.to_owned()));
            }
            _ => {}
        }
        request
    }
}

impl ConformanceTarget for Edges {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(self.steer(request))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(r)
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

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

fn sent(suite: &ConformanceSuite, id: &str) -> Vec<BTreeMap<String, String>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(
                input
                    .iter()
                    .map(|(name, value)| (name.clone(), format!("{value:?}")))
                    .collect(),
            ),
            _ => None,
        })
        .collect()
}

#[test]
fn a1_self_edge_refuses() {
    let suite = suite();
    let inputs = sent(&suite, SELF_EDGE);
    let last = inputs
        .last()
        .expect("the self-edge scenario sends the command");
    assert_eq!(
        last.get("task_id"),
        last.get("depends_on"),
        "the refusal is witnessed with one identity sent twice: {last:#?}"
    );
    let statuses = run(&suite, &Edges::new(Mode::Correct));
    assert_eq!(
        statuses.get(SELF_EDGE),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
    let interpreted = run(&suite, &Interpreted::for_model(ir()));
    assert!(
        not_passed(&interpreted).is_empty(),
        "the native interpreter passes every scenario: {:#?}",
        not_passed(&interpreted)
    );
}

#[test]
fn a1_distinct_edge_accepts() {
    let suite = suite();
    let inputs = sent(&suite, LINKED);
    let last = inputs
        .last()
        .expect("the linked scenario sends the command");
    assert_ne!(
        last.get("task_id"),
        last.get("depends_on"),
        "acceptance is witnessed with two different identities: {last:#?}"
    );
    let statuses = run(&suite, &Interpreted::for_model(ir()));
    assert_eq!(statuses.get(LINKED), Some(&Status::Passed), "{statuses:#?}");
}

/// The outcome the native interpreter takes for one request with `input`, or why it took none.
fn interpreted_outcome(input: &[(&str, &str)]) -> String {
    let target = Interpreted::for_model(ir());
    let correlation = ess_primitives::ids::CorrelationId::new("a1-direct").expect("an id");
    let scenario = ess_conformance::ScenarioId::parse(LINKED).expect("a scenario id");
    target
        .begin_scenario(&ScenarioContext::new(scenario, correlation.clone()))
        .expect("begins");
    let result = match target.execute_command(SemanticCommandRequest {
        command: ess_compiler::refs::CommandRef::new("graph.tasks.Link".parse().unwrap()),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), Node::Text((*value).to_owned())))
            .collect(),
        correlation,
    }) {
        Ok(result) => result,
        Err(error) => return format!("{error:?}"),
    };
    result
        .outcome
        .map(|outcome| outcome.to_string())
        .unwrap_or_default()
}

const A: &str = "00000000-0000-4000-8000-000000000001";
const B: &str = "00000000-0000-4000-8000-000000000002";

#[test]
fn a1_optional_absent_unknown() {
    // An absent `blocked_by` is no identity equal to `task_id`: the guard is Unknown. An Unknown
    // guard selects no branch — neither `blocked`, which reading the absent operand as anything
    // would select, nor the default `linked` — so the interpreter answers that it cannot decide.
    let predicate: ess_primitives::predicate::Predicate =
        serde_json::from_str(r#"{"task_id": {"eq": {"fact": "blocked_by"}}}"#).expect("reads");
    let mut facts = ess_primitives::facts::FactStore::new();
    facts.set_path("task_id", ess_primitives::facts::FactValue::text(A));
    assert_eq!(
        predicate.evaluate(&facts),
        ess_primitives::predicate::Truth::Unknown
    );
    let absent = interpreted_outcome(&[("task_id", A), ("depends_on", B)]);
    assert!(
        absent.contains("Unsupported") && absent.contains("blocked") && absent.contains("Unknown"),
        "{absent}"
    );
    assert_eq!(
        interpreted_outcome(&[("task_id", A), ("depends_on", B), ("blocked_by", B)]),
        "graph.tasks.Link/linked"
    );
    assert_eq!(
        interpreted_outcome(&[("task_id", A), ("depends_on", B), ("blocked_by", A)]),
        "graph.tasks.Link/blocked"
    );
    assert_eq!(
        interpreted_outcome(&[("task_id", A), ("depends_on", A), ("blocked_by", B)]),
        "graph.tasks.Link/self-edge"
    );
}

fn caught(mode: Mode) {
    let suite = suite();
    let healthy = run(&suite, &Edges::new(Mode::Correct));
    assert!(
        not_passed(&healthy).is_empty(),
        "{:#?}",
        not_passed(&healthy)
    );
    let faulty = run(&suite, &Edges::new(mode));
    let failing = not_passed(&faulty);
    assert!(
        failing
            .iter()
            .any(|line| line.starts_with(SELF_EDGE) || line.starts_with(LINKED)),
        "{mode:?} fails a scenario that decides the self-edge guard: {failing:#?}"
    );
    assert!(
        failing.iter().all(|line| line.contains("Failed")),
        "{mode:?} fails by outcome, not by setup or admission: {failing:#?}"
    );
}

#[test]
fn rhs_as_text() {
    caught(Mode::RhsAsText);
}

#[test]
fn left_vs_self() {
    caught(Mode::LeftVsSelf);
}

#[test]
fn wrong_namespace() {
    caught(Mode::WrongNamespace);
}

#[test]
fn a1_guard_reads_both_inputs_in_the_compiled_model() {
    let model = ir();
    let json = model.to_canonical_json();
    assert!(
        json.contains(r#""task_id": {"#) && json.contains(r#""fact": "depends_on""#),
        "the IR carries the explicit fact operand"
    );
}

// ---- the persisted format: suite /40 and /41 ------------------------------------------------

const ROOT_FACT: &str = r#"{"valid_until": {"gte": {"fact": "valid_from"}}}"#;
const BINDER: &str = r#"{"forall": {"in": "tags", "as": "b", "that": "tag != b"}}"#;
const DOTTED: &str = r#""window.end >= window.start""#;

#[test]
fn suite39_relabel_refuses_fact() {
    use ess_conformance::expression_format::{admit_predicate, ADMITTED, COVERAGE, ORDINARY};
    for major in [1, 8, 35, 39] {
        let refused =
            admit_predicate(ROOT_FACT, major).expect_err("an older reader cannot read it");
        assert!(
            refused.to_string().contains("suite/40 or /41"),
            "{major}: {refused}"
        );
    }
    for major in [ORDINARY, COVERAGE] {
        admit_predicate(ROOT_FACT, major).expect("the /40 pair reads it");
    }
    // A binder and a dotted path were always facts: no newer reader is needed for them.
    for unchanged in [BINDER, DOTTED] {
        admit_predicate(unchanged, 35).expect("an older reader reads it");
    }
    assert_eq!(ADMITTED, [40, 41]);
    let supported: Vec<u32> = ess_conformance::scenario::SUPPORTED_SUITE_FORMATS
        .iter()
        .copied()
        .filter(|major| (38..=41).contains(major))
        .collect();
    assert_eq!(
        supported,
        vec![
            ess_conformance::conditional_measures::ORDINARY,
            ess_conformance::conditional_measures::COVERAGE,
            ORDINARY,
            COVERAGE
        ],
        "the conditional measure pair registers below the expression pair"
    );
}

#[test]
fn a_suite_without_the_vocabulary_keeps_its_format() {
    let suite = suite();
    assert!(!ess_conformance::expression_format::used_by(&suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34",
        "input guards are decided at synthesis and persist no predicate"
    );
}

// ---- the Go reader ---------------------------------------------------------------------------

#[test]
fn the_generated_go_reader_answers_the_shared_fact_operand_vectors() {
    let document = serde_json::json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "graph",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"graph.tasks/authored/operand": {"purpose": "Compare two facts",
            "steps": [{"step": "expect_view", "view": "graph.tasks.Edges",
                "expectation": {"expect": "satisfies", "predicate": "DOTTED_NODE"}}], "source": []}}
    })
    .to_string()
    .replace("\"DOTTED_NODE\"", DOTTED);
    let suite = AdmittedSuite::from_json(&document).expect("admitted");
    let directory = support_scratch::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-a1-root-fact-{}", std::process::id())),
    );
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/a1\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join("essconform/root_fact_test.go"),
        include_str!("fixtures/root-fact-operand.go"),
    )
    .expect("fixture");
    std::fs::write(
        directory.join("essconform/root-fact-operand.json"),
        include_str!("../../../specify/ess-primitives/tests/vectors/root-fact-operand.json"),
    )
    .expect("vectors");
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestRootFactOperand",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .expect("go runs");
    let printed = format!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    eprintln!("{printed}");
    assert!(
        result.status.success() && printed.contains("--- PASS: TestRootFactOperand"),
        "Go fact operand reader: {}",
        result.status
    );
}

/// Runs one Go vector fixture inside an emitted runtime package, with its vectors beside it.
fn go_vectors(label: &str, test: &str, fixture: &str, vectors: (&str, &str)) {
    let document = serde_json::json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "graph",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"graph.tasks/authored/operand": {"purpose": "Compare two facts",
            "steps": [{"step": "expect_view", "view": "graph.tasks.Edges",
                "expectation": {"expect": "satisfies", "predicate": "window.end >= window.start"}}],
            "source": []}}
    })
    .to_string();
    let suite = AdmittedSuite::from_json(&document).expect("admitted");
    let directory = support_scratch::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-a1-{label}-{}", std::process::id())),
    );
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/a1\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join(format!("essconform/{label}_test.go")),
        fixture,
    )
    .expect("fixture");
    std::fs::write(directory.join("essconform").join(vectors.0), vectors.1).expect("vectors");
    let result = std::process::Command::new("go")
        .args(["test", "./essconform", "-run", test, "-count=1", "-v"])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .expect("go runs");
    let printed = format!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    eprintln!("{printed}");
    assert!(
        result.status.success() && printed.contains(&format!("--- PASS: {test}")),
        "{test}: {}",
        result.status
    );
}

#[test]
fn a1_timestamp_sibling_instant_order_in_the_generated_go_reader() {
    go_vectors(
        "instant",
        "TestInstantComparison",
        include_str!("fixtures/instant-comparison.go"),
        (
            "rfc3339-instants.json",
            include_str!("../../../specify/ess-primitives/tests/vectors/rfc3339-instants.json"),
        ),
    );
}
