//! An input guard that combines an enum value with an optional input's presence is honoured by
//! synthesis (beyond10x/ess#280).
//!
//! `provider-not-allowed: {scope: Budget, provider: {exists: true}}` beside the mirror
//! `provider-missing: {scope: Provider, provider: {exists: false}}` and a default `set`: before the
//! fix, `SetQuota/outcome/set` sent `{scope: Budget, provider: "provider"}` and required `set`, an
//! input `provider-not-allowed` refuses, and a target honouring the guards failed the suite.
//!
//! Every synthesized step's input is checked against the guards under
//! [the precedence order](../../../../docs/design/cross-record-and-stored-field-guards.md#the-precedence-order)
//! by the model interpreter, the suite is run against a hand-written target that honours the guards,
//! and a target that ignores the presence half fails a scenario.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{precedence_contradictions, synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The issue's minimal specification, as one document.
const QUOTA: &str = r"
format: ess/19
system: mini
version: v1
domain: mini.q
types:
  - name: mini.q.QuotaId
    kind: newtype
    of: Uuid
  - name: mini.q.Scope
    kind: enum
    variants: [Budget, Provider]
entities:
  - name: mini.q.Quota
    identity: {name: quota_id, type: mini.q.QuotaId}
    fields:
      - {name: scope, type: mini.q.Scope}
      - {name: provider, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - name: mini.q.InvalidScope
    summary: The scope and provider disagree.
    fields:
      - {name: scope, type: mini.q.Scope}
events:
  - name: mini.q.QuotaSet
    fields:
      - {name: quota_id, type: mini.q.QuotaId}
commands:
  - name: mini.q.SetQuota
    input:
      - {name: scope, type: mini.q.Scope}
      - {name: provider, type: Optional<String>}
    outcomes:
      - name: provider-not-allowed
        when: {scope: Budget, provider: {exists: true}}
        error: mini.q.InvalidScope
        payload:
          mini.q.InvalidScope: {scope: input.scope}
      - name: provider-missing
        when: {scope: Provider, provider: {exists: false}}
        error: mini.q.InvalidScope
        payload:
          mini.q.InvalidScope: {scope: input.scope}
      - name: set
        creates: mini.q.Quota
        instance: quota_id
        sets: {scope: input.scope, provider: input.provider}
        emits: [mini.q.QuotaSet]
        payload:
          mini.q.QuotaSet: {quota_id: {generated: true}}
";

/// The requester's second reproduction: a planned task may not name the task it repairs, and a
/// repair must.
const TASKS: &str = r"
format: ess/19
system: mini
version: v1
domain: mini.t
types:
  - name: mini.t.TaskId
    kind: newtype
    of: Uuid
  - name: mini.t.Kind
    kind: enum
    variants: [Planned, Repair]
entities:
  - name: mini.t.Task
    identity: {name: task_id, type: mini.t.TaskId}
    fields:
      - {name: kind, type: mini.t.Kind}
      - {name: repair_of, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - name: mini.t.InvalidKind
    summary: The kind and the repaired task disagree.
    fields:
      - {name: kind, type: mini.t.Kind}
events:
  - name: mini.t.TaskAdded
    fields:
      - {name: task_id, type: mini.t.TaskId}
commands:
  - name: mini.t.AddTask
    input:
      - {name: kind, type: mini.t.Kind}
      - {name: repair_of, type: Optional<String>}
    outcomes:
      - name: repair-not-allowed
        when: {kind: Planned, repair_of: {exists: true}}
        error: mini.t.InvalidKind
        payload:
          mini.t.InvalidKind: {kind: input.kind}
      - name: repair-missing
        when: {kind: Repair, repair_of: {exists: false}}
        error: mini.t.InvalidKind
        payload:
          mini.t.InvalidKind: {kind: input.kind}
      - name: added
        creates: mini.t.Task
        instance: task_id
        sets: {kind: input.kind, repair_of: input.repair_of}
        emits: [mini.t.TaskAdded]
        payload:
          mini.t.TaskAdded: {task_id: {generated: true}}
";

/// [`TASKS`] with both refusals written as one `any:` guard.
fn tasks_any() -> String {
    let text = TASKS.replace(
        "      - name: repair-not-allowed
        when: {kind: Planned, repair_of: {exists: true}}
        error: mini.t.InvalidKind
        payload:
          mini.t.InvalidKind: {kind: input.kind}
      - name: repair-missing
        when: {kind: Repair, repair_of: {exists: false}}
",
        "      - name: invalid-kind
        when:
          any:
            - {kind: Planned, repair_of: {exists: true}}
            - {kind: Repair, repair_of: {exists: false}}
",
    );
    assert_ne!(text, TASKS);
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("mini.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis_of(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesis_of(text);
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// Every invocation of `command` in the suite, by scenario, with its literal input and the outcome
/// the next step requires.
fn invocations(
    suite: &ConformanceSuite,
    command: &str,
) -> Vec<(String, BTreeMap<String, Node>, String)> {
    let mut out = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let mut steps = scenario.steps.iter().peekable();
        while let Some(step) = steps.next() {
            let ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } = step
            else {
                continue;
            };
            if sent.to_string() != command {
                continue;
            }
            let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
                continue;
            };
            let literal = input
                .iter()
                .map(|(field, value)| match value {
                    ScenarioValue::Literal { value } => (field.clone(), value.clone()),
                    other => panic!("`{field}` is sent as a literal in {id}: {other:?}"),
                })
                .collect();
            out.push((id.to_string(), literal, outcome.outcome.to_string()));
        }
    }
    out
}

/// What the model answers `input` with, under the declared precedence.
fn answered(model: &EssIr, command: &str, input: &BTreeMap<String, Node>) -> BTreeSet<String> {
    let name = command.parse().expect("a command name");
    execute(model, &Store::default(), &name, input, &Externals::Withheld)
        .unwrap_or_else(|error| panic!("the model determines {command} at {input:?}: {error:?}"))
        .iter()
        .map(|step| {
            step.outcome.as_ref().map_or("none".to_owned(), |outcome| {
                let outcome = outcome.to_string();
                outcome.rsplit('/').next().unwrap_or_default().to_owned()
            })
        })
        .collect()
}

/// Every synthesized invocation of `command` requires the branch the guards select for its input.
fn assert_every_input_agrees(text: &str, command: &str) {
    let model = ir(text);
    let suite = suite_of(text);
    let sent = invocations(&suite, command);
    assert!(!sent.is_empty(), "{command} is never sent");
    let disagreeing: Vec<_> = sent
        .iter()
        .filter(|(_, input, outcome)| {
            answered(&model, command, input) != BTreeSet::from([outcome.clone()])
        })
        .map(|(id, input, outcome)| {
            format!(
                "{id} sends {input:?} and requires `{outcome}`; the guards select {:?}",
                answered(&model, command, input)
            )
        })
        .collect();
    assert!(
        disagreeing.is_empty(),
        "synthesized inputs the guards answer otherwise:\n{}",
        disagreeing.join("\n")
    );
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

fn not_passed(statuses: &BTreeMap<String, Status>) -> BTreeSet<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.clone())
        .collect()
}

// ---- a hand-written target ---------------------------------------------------------------------

/// The shape both specifications share: one command over an enum and an optional text, refused
/// where the first variant comes with the text or the second without it, and otherwise creating a
/// row and emitting one event carrying its generated identity.
struct Shape {
    command: &'static str,
    variant: &'static str,
    optional: &'static str,
    /// The refusal for the first variant with the optional present.
    present: &'static str,
    /// The refusal for the second variant with the optional absent.
    absent: &'static str,
    created: &'static str,
    error: &'static str,
    event: &'static str,
    identity: &'static str,
    first: &'static str,
    second: &'static str,
}

const QUOTA_SHAPE: Shape = Shape {
    command: "mini.q.SetQuota",
    variant: "scope",
    optional: "provider",
    present: "provider-not-allowed",
    absent: "provider-missing",
    created: "set",
    error: "mini.q.InvalidScope",
    event: "mini.q.QuotaSet",
    identity: "quota_id",
    first: "Budget",
    second: "Provider",
};

const TASKS_SHAPE: Shape = Shape {
    command: "mini.t.AddTask",
    variant: "kind",
    optional: "repair_of",
    present: "repair-not-allowed",
    absent: "repair-missing",
    created: "added",
    error: "mini.t.InvalidKind",
    event: "mini.t.TaskAdded",
    identity: "task_id",
    first: "Planned",
    second: "Repair",
};

const TASKS_ANY_SHAPE: Shape = Shape {
    present: "invalid-kind",
    absent: "invalid-kind",
    ..TASKS_SHAPE
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Honours both halves of each guard.
    Correct,
    /// Reads only the enum half: every first or second variant is refused, whatever the optional.
    IgnoresPresence,
}

struct Hand {
    shape: Shape,
    mode: Mode,
    sequence: RefCell<u64>,
}

impl Hand {
    fn new(shape: Shape, mode: Mode) -> Self {
        Self {
            shape,
            mode,
            sequence: RefCell::new(0),
        }
    }

    fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()))
            .with_consistency(ess_primitives::consistency::ConsistencyToken::new("write").unwrap())
    }
}

impl ConformanceTarget for Hand {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("enum-presence", "1"))
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
        let shape = &self.shape;
        assert_eq!(request.command.to_string(), shape.command);
        let variant = request.input.get(shape.variant).cloned();
        let Some(Node::Text(text)) = &variant else {
            return Ok(SemanticCommandResult::undeclared());
        };
        let present = !matches!(request.input.get(shape.optional), None | Some(Node::Null));
        let present = present || self.mode == Mode::IgnoresPresence;
        let absent = !present || self.mode == Mode::IgnoresPresence;
        let refusal = if text == shape.first && present {
            Some(shape.present)
        } else if text == shape.second && absent {
            Some(shape.absent)
        } else {
            None
        };
        if let Some(refusal) = refusal {
            let error = DeclaredErrorValue::new(shape.error.parse().unwrap())
                .with(shape.variant, variant.clone().unwrap());
            return Ok(Self::took(&request.command, refusal).with_error(error));
        }
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        let id = format!("00000000-0000-4000-8000-{:012}", *sequence);
        let mut result = Self::took(&request.command, shape.created);
        result.direct_events.push(
            ObservedEvent::new(shape.event.parse().unwrap()).with(shape.identity, Node::Text(id)),
        );
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
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

// ---- #280 ---------------------------------------------------------------------------------------

#[test]
fn issue_280_every_synthesized_input_agrees_with_the_guards() {
    assert_every_input_agrees(QUOTA, "mini.q.SetQuota");
}

#[test]
fn issue_280_both_refusals_are_witnessed_and_set_avoids_their_inputs() {
    let suite = suite_of(QUOTA);
    let sent = invocations(&suite, "mini.q.SetQuota");
    for refusal in ["provider-not-allowed", "provider-missing"] {
        let id = format!("mini.q.SetQuota/outcome/{refusal}");
        assert!(
            sent.iter()
                .any(|(scenario, _, outcome)| *scenario == id && outcome == refusal),
            "`{refusal}` is not witnessed: {sent:#?}"
        );
    }
    for (scenario, input, outcome) in &sent {
        if outcome != "set" {
            continue;
        }
        let scope = input.get("scope").and_then(Node::as_text);
        let provider = !matches!(input.get("provider"), None | Some(Node::Null));
        assert!(
            !(scope == Some("Budget") && provider) && !(scope == Some("Provider") && !provider),
            "{scenario} requires `set` at {input:?}, which a refusal claims"
        );
    }
}

#[test]
fn issue_280_a_target_honouring_the_guards_passes_and_one_ignoring_presence_fails() {
    let suite = suite_of(QUOTA);
    assert_eq!(
        not_passed(&run(&suite, &Interpreted::for_model(ir(QUOTA)))),
        BTreeSet::new()
    );
    assert_eq!(
        not_passed(&run(&suite, &Hand::new(QUOTA_SHAPE, Mode::Correct))),
        BTreeSet::new()
    );
    let failed = not_passed(&run(&suite, &Hand::new(QUOTA_SHAPE, Mode::IgnoresPresence)));
    assert!(
        failed.contains("mini.q.SetQuota/outcome/set"),
        "a target ignoring the presence half passes `set`: {failed:?}"
    );
}

#[test]
fn issue_280_the_planned_and_repair_refusals_agree_with_the_guards() {
    assert_every_input_agrees(TASKS, "mini.t.AddTask");
    let suite = suite_of(TASKS);
    assert_eq!(
        not_passed(&run(&suite, &Hand::new(TASKS_SHAPE, Mode::Correct))),
        BTreeSet::new()
    );
    assert!(!not_passed(&run(&suite, &Hand::new(TASKS_SHAPE, Mode::IgnoresPresence))).is_empty());
}

#[test]
fn issue_280_one_any_guard_over_both_refusals_agrees_with_the_guards() {
    let text = tasks_any();
    assert_every_input_agrees(&text, "mini.t.AddTask");
    let suite = suite_of(&text);
    assert_eq!(
        not_passed(&run(&suite, &Hand::new(TASKS_ANY_SHAPE, Mode::Correct))),
        BTreeSet::new()
    );
    assert!(!not_passed(&run(
        &suite,
        &Hand::new(TASKS_ANY_SHAPE, Mode::IgnoresPresence)
    ))
    .is_empty());
}

/// The interpreter itself, which the agreement check reads, answers the four corners as the guards
/// say.
#[test]
fn issue_280_the_interpreter_answers_each_corner() {
    let model = ir(QUOTA);
    let at = |scope: &str, provider: Option<&str>| {
        let mut input = BTreeMap::from([("scope".to_owned(), Node::Text(scope.to_owned()))]);
        if let Some(provider) = provider {
            input.insert("provider".to_owned(), Node::Text(provider.to_owned()));
        }
        answered(&model, "mini.q.SetQuota", &input)
    };
    let one = |name: &str| BTreeSet::from([name.to_owned()]);
    assert_eq!(at("Budget", Some("p")), one("provider-not-allowed"));
    assert_eq!(at("Budget", None), one("set"));
    assert_eq!(at("Provider", Some("p")), one("set"));
    assert_eq!(at("Provider", None), one("provider-missing"));
}

// ---- the synthesis-wide check -------------------------------------------------------------------

/// `suite` with the first invocation of the scenario `id` sent `input` instead.
fn resent(suite: &ConformanceSuite, id: &str, input: &[(&str, &str)]) -> ConformanceSuite {
    let mut suite = suite.clone();
    let (_, scenario) = suite
        .scenarios
        .iter_mut()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"));
    let sent = scenario
        .steps
        .iter_mut()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input),
            _ => None,
        })
        .expect("the scenario sends a command");
    *sent = input
        .iter()
        .map(|(field, value)| {
            (
                (*field).to_owned(),
                ScenarioValue::literal(Node::Text((*value).to_owned())),
            )
        })
        .collect();
    suite
}

/// The check reads a finished suite and refuses an accepting step whose input an input-guarded
/// refusal claims, naming both outcomes — the #280 scenario as 0.48.0 synthesized it.
#[test]
fn the_check_refuses_an_accepting_step_a_refusal_claims() {
    let model = ir(QUOTA);
    let suite = suite_of(QUOTA);
    assert_eq!(precedence_contradictions(&model, &suite), Vec::new());
    let wrong = resent(
        &suite,
        "mini.q.SetQuota/outcome/set",
        &[("scope", "Budget"), ("provider", "provider")],
    );
    let refusals = precedence_contradictions(&model, &wrong);
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    let rendered = format!("{} {}", refusals[0].code(), refusals[0]);
    assert_eq!(
        refusals[0]
            .scenario
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("mini.q.SetQuota/outcome/set")
    );
    for named in ["ESS-SYNTH-019", "`set`", "`provider-not-allowed`"] {
        assert!(rendered.contains(named), "{named} is not named: {rendered}");
    }
}

/// Of two input-guarded refusals an input selects, the first declared answers, so a step requiring
/// the later one at such an input is refused as well.
#[test]
fn the_check_refuses_a_refusal_step_an_earlier_refusal_claims() {
    let model = ir(QUOTA);
    let wrong = resent(
        &suite_of(QUOTA),
        "mini.q.SetQuota/outcome/provider-missing",
        &[("scope", "Budget"), ("provider", "provider")],
    );
    let refusals = precedence_contradictions(&model, &wrong);
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    let rendered = refusals[0].to_string();
    assert!(
        rendered.contains("`provider-missing`") && rendered.contains("`provider-not-allowed`"),
        "{rendered}"
    );
}

/// The second half of the requester's report: one `any:` refusal required for a valid Repair task,
/// an input its own guard refutes, is refused as well.
#[test]
fn the_check_refuses_a_step_its_own_guard_refutes() {
    let text = tasks_any();
    let model = ir(&text);
    let suite = suite_of(&text);
    assert_eq!(precedence_contradictions(&model, &suite), Vec::new());
    let wrong = resent(
        &suite,
        "mini.t.AddTask/outcome/invalid-kind",
        &[("kind", "Repair"), ("repair_of", "repair_of")],
    );
    let refusals = precedence_contradictions(&model, &wrong);
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    let rendered = refusals[0].to_string();
    assert!(
        rendered.contains("`invalid-kind`") && rendered.contains("its own guard"),
        "{rendered}"
    );
}

/// Every specification in the repository a synthesis runs over: each `examples/` and `models/`
/// system, and every single-document `.yaml` fixture under a crate's `tests/fixtures/`, that
/// assembles and compiles.
fn corpus() -> Vec<(String, EssIr)> {
    use std::path::{Path, PathBuf};
    fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, found);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    fn compiled(files: &[PathBuf], base: &Path) -> Option<EssIr> {
        let mut sources = SourceMap::new();
        let mut parsed = Vec::new();
        for path in files {
            let label = path.strip_prefix(base).unwrap().display().to_string();
            let text = std::fs::read_to_string(path).unwrap();
            let Ok(raw) = RawSpecFile::parse(&text) else {
                continue;
            };
            sources.insert(label.clone(), text);
            parsed.push((Source::new(label), raw));
        }
        if parsed.is_empty() {
            return None;
        }
        let specification = Specification::assemble(parsed).ok()?;
        compile(&specification, &sources).ok()
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let mut out = Vec::new();
    for family in ["examples", "models"] {
        let mut systems: Vec<PathBuf> = std::fs::read_dir(root.join(family))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        systems.sort();
        for system in systems {
            let mut files = Vec::new();
            walk(&system, &mut files);
            files.sort();
            if let Some(model) = compiled(&files, &system) {
                out.push((
                    system.strip_prefix(&root).unwrap().display().to_string(),
                    model,
                ));
            }
        }
    }
    let mut fixtures = Vec::new();
    for area in std::fs::read_dir(root.join("crates")).unwrap() {
        for krate in std::fs::read_dir(area.unwrap().path())
            .into_iter()
            .flatten()
        {
            walk(&krate.unwrap().path().join("tests/fixtures"), &mut fixtures);
        }
    }
    fixtures.sort();
    for fixture in fixtures {
        let base = fixture.parent().unwrap().to_path_buf();
        if let Some(model) = compiled(std::slice::from_ref(&fixture), &base) {
            out.push((
                fixture.strip_prefix(&root).unwrap().display().to_string(),
                model,
            ));
        }
    }
    out
}

/// No synthesized suite of any example, model or fixture carries a step the check refuses, and no
/// synthesis refuses one.
#[test]
fn no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere() {
    let corpus = corpus();
    eprintln!("corpus: {} specifications", corpus.len());
    assert!(
        corpus.len() >= 60,
        "the corpus walk found {} specifications; it is not reading the repository",
        corpus.len()
    );
    let mut found = Vec::new();
    for (label, model) in &corpus {
        let synthesis = synthesize(model);
        for refusal in synthesis
            .refusals
            .iter()
            .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-019")
        {
            found.push(format!("{label}: {refusal}"));
        }
        for refusal in precedence_contradictions(model, &synthesis.suite) {
            found.push(format!("{label}: {refusal}"));
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}
