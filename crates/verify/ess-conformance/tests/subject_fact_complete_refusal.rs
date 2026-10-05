//! A refusal selected by `when_subject:` changes nothing on the record (beyond10x/ess#461).
//!
//! From `ess/23` every such refusal compiles with `complete_refusal`: its scenario snapshots the
//! complete record before the command and requires it unchanged afterwards, as a wrong-state
//! refusal's does. A refusal whose predicate reads `state` is witnessed on a row in each state it
//! claims, not only in the first one the search reaches. `when_subject:` beside
//! `when_subject_state:` stays `ESS-COMMAND-004`, whose hint names the `state` predicate idiom.
//!
//! The fixture is the command the issue describes: `seed-change-refused` refuses a changed stored
//! field, `not-active` refuses every state but `Active`, and `updated` writes the description.
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::resolve::{compile, diagnose_locating};
use ess_compiler::{ir::EssIr, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{OutcomeRef, ViewExpectation};
use ess_conformance::synthesize::{Note, Synthesis};
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::command::OutcomeName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/subject-fact-complete-refusal.yaml");

const UPDATE: &str = "demo.inst.UpdateInstance";
const SEED_REFUSED: &str = "demo.inst.UpdateInstance/outcome/seed-change-refused";
const NOT_ACTIVE: &str = "demo.inst.UpdateInstance/outcome/not-active";

fn at(format: &str) -> String {
    let out = MODEL.replace("format: ess/23\n", &format!("format: {format}\n"));
    assert!(out.starts_with(&format!("format: {format}\n")), "{format}");
    out
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the model");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("instance.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result.refusals
                )
            },
            |(_, scenario)| scenario,
        )
}

/// One `UpdateInstance` sent to an arranged record and required to take `outcome`: the state the
/// record was observed in before it, and whether a complete snapshot of that record precedes it and
/// a complete comparison follows it, each before any other command is sent.
#[derive(Debug, PartialEq, Eq)]
struct Send {
    state: Option<String>,
    snapshot: bool,
    compared: bool,
}

fn sends(scenario: &ConformanceScenario, outcome: &str) -> Vec<Send> {
    let steps = &scenario.steps;
    let mut out = Vec::new();
    for (at, step) in steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != UPDATE {
            continue;
        }
        let Some(name @ ScenarioValue::Instance { .. }) = input.get("name") else {
            continue;
        };
        let required = steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        if required.as_deref() != Some(outcome) {
            continue;
        }
        let previous = steps[..at]
            .iter()
            .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .map_or(0, |found| found + 1);
        let next = steps[at + 1..]
            .iter()
            .position(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .map_or(steps.len(), |found| at + 1 + found);
        let snapshot = steps[previous..at].iter().any(|step| {
            matches!(step, ScenarioStep::SnapshotCompleteSubject { subject, .. }
                if subject.get("name") == Some(name))
        });
        let compared = steps[at + 1..next]
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. }));
        let state = steps[..at].iter().rev().find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } if fields.get("name") == Some(name) => {
                match fields.get("state").and_then(ScenarioValue::as_literal) {
                    Some(Node::Text(state)) => Some(state.clone()),
                    _ => None,
                }
            }
            _ => None,
        });
        out.push(Send {
            state,
            snapshot,
            compared,
        });
    }
    out
}

fn count(scenario: &ConformanceScenario, wanted: fn(&ScenarioStep) -> bool) -> usize {
    scenario.steps.iter().filter(|step| wanted(step)).count()
}

fn snapshots(step: &ScenarioStep) -> bool {
    matches!(step, ScenarioStep::SnapshotCompleteSubject { .. })
}

fn comparisons(step: &ScenarioStep) -> bool {
    matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. })
}

fn run(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// What a scripted target gets wrong.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    /// Nothing: the interpreted model.
    None,
    /// Answers `seed-change-refused`, and stores the sent description anyway.
    WritesWhileRefusing,
    /// Updates a record held in this state instead of refusing it `not-active`.
    UpdatesIn(&'static str),
}

/// The interpreted model, with one fault laid over what it answers and what its view shows.
///
/// The record's held state is followed from the commands the model answered, and a description
/// the fault wrote is shown in place of the stored one.
struct Target {
    inner: Interpreted,
    fault: Fault,
    states: RefCell<BTreeMap<Node, &'static str>>,
    written: RefCell<BTreeMap<Node, Node>>,
}

impl Target {
    fn new(model: &str, fault: Fault) -> Self {
        Self {
            inner: Interpreted::for_model(ir(model)),
            fault,
            states: RefCell::new(BTreeMap::new()),
            written: RefCell::new(BTreeMap::new()),
        }
    }
}

impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.states.borrow_mut().clear();
        self.written.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let name = request.input.get("name").cloned();
        let description = request.input.get("description").cloned();
        let mut result = self.inner.execute_command(request)?;
        let (Some(name), Some(taken)) = (name, result.outcome.clone()) else {
            return Ok(result);
        };
        let taken = taken.outcome.to_string();
        let moved = match taken.as_str() {
            "created" => Some("Active"),
            "suspended" => Some("Suspended"),
            "removed" => Some("Removed"),
            _ => None,
        };
        if let Some(state) = moved {
            self.states.borrow_mut().insert(name.clone(), state);
        }
        if command.to_string() != UPDATE {
            return Ok(result);
        }
        let held = self.states.borrow().get(&name).copied();
        let wrong = match (self.fault, taken.as_str()) {
            (Fault::WritesWhileRefusing, "seed-change-refused") => true,
            (Fault::UpdatesIn(state), "not-active") if held == Some(state) => {
                result.outcome = Some(OutcomeRef::new(
                    command,
                    OutcomeName::new("updated").expect("a name"),
                ));
                result.error = None;
                true
            }
            _ => false,
        };
        if let (true, Some(description)) = (wrong, description) {
            self.written.borrow_mut().insert(name, description);
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        let written = self.written.borrow();
        for row in &mut result.rows {
            if let Some(description) = row.get("name").and_then(|name| written.get(name)) {
                if row.contains_key("description") {
                    row.insert("description".to_owned(), description.clone());
                }
            }
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

fn outcome_json<'v>(ir: &'v serde_json::Value, command: &str, name: &str) -> &'v serde_json::Value {
    fn find<'v>(value: &'v serde_json::Value, command: &str) -> Option<&'v serde_json::Value> {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("name").is_some_and(|name| name == command)
                    && map.contains_key("outcomes")
                {
                    return Some(value);
                }
                map.values().find_map(|inner| find(inner, command))
            }
            serde_json::Value::Array(items) => items.iter().find_map(|inner| find(inner, command)),
            _ => None,
        }
    }
    find(ir, command)
        .and_then(|found| found["outcomes"].as_array())
        .and_then(|outcomes| outcomes.iter().find(|outcome| outcome["name"] == name))
        .unwrap_or_else(|| panic!("{command}/{name} in {ir:#}"))
}

#[test]
fn when_subject_refusal_observes_complete_subject() {
    let result = synthesis(MODEL);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    for (id, outcome) in [
        (SEED_REFUSED, "seed-change-refused"),
        (NOT_ACTIVE, "not-active"),
    ] {
        let found = sends(scenario(&result, id), outcome);
        assert!(!found.is_empty(), "{id} sends to an arranged record");
        for send in &found {
            assert!(
                send.snapshot && send.compared,
                "{id}: each refused record is snapshot completely before the command and compared \
                 completely after it: {found:#?}"
            );
        }
    }
}

#[test]
fn refusal_that_writes_an_unguarded_field_fails() {
    let result = synthesis(MODEL);
    let honest = run(&result.suite, &Target::new(MODEL, Fault::None));
    assert_eq!(
        not_passed(&honest),
        Vec::<&str>::new(),
        "the interpreted model passes every scenario: {honest:#?}"
    );
    let writes = run(
        &result.suite,
        &Target::new(MODEL, Fault::WritesWhileRefusing),
    );
    assert!(
        not_passed(&writes).contains(&SEED_REFUSED),
        "a target that refuses the seed change but stores the sent description fails \
         `seed-change-refused`: {writes:#?}"
    );
}

#[test]
fn state_reading_refusal_witnessed_in_every_claimed_state() {
    let result = synthesis(MODEL);
    let found = sends(scenario(&result, NOT_ACTIVE), "not-active");
    let states: BTreeSet<_> = found.iter().filter_map(|send| send.state.clone()).collect();
    assert_eq!(
        states,
        BTreeSet::from(["Removed".to_owned(), "Suspended".to_owned()]),
        "`not-active` is witnessed on a suspended and on a removed record, and never on an active \
         one: {found:#?}"
    );
    assert!(
        found.iter().all(|send| send.snapshot && send.compared),
        "each with the complete snapshot: {found:#?}"
    );
    for state in ["Suspended", "Removed"] {
        let updates = run(&result.suite, &Target::new(MODEL, Fault::UpdatesIn(state)));
        assert!(
            not_passed(&updates).contains(&NOT_ACTIVE),
            "a target that updates a {state} record fails `not-active`: {updates:#?}"
        );
    }
}

#[test]
fn complete_refusal_minted_from_ess23_only() {
    for (format, expected) in [("ess/23", true), ("ess/22", false)] {
        let compiled = serde_json::to_value(ir(&at(format))).expect("IR serializes");
        for name in ["seed-change-refused", "not-active"] {
            let outcome = outcome_json(&compiled, UPDATE, name);
            assert_eq!(
                outcome.get("complete_refusal") == Some(&serde_json::Value::Bool(true)),
                expected,
                "{format}: `{name}` carries `complete_refusal` exactly from ess/23: {outcome:#}"
            );
        }
        for name in ["no-such-instance", "updated"] {
            let outcome = outcome_json(&compiled, UPDATE, name);
            assert!(
                outcome.get("complete_refusal").is_none(),
                "{format}: `{name}` is no refusal selected by `when_subject`: {outcome:#}"
            );
        }
    }
}

#[test]
fn below_ess23_suites_unchanged() {
    let result = synthesis(&at("ess/22"));
    for (id, outcome) in [
        (SEED_REFUSED, "seed-change-refused"),
        (NOT_ACTIVE, "not-active"),
    ] {
        let found = scenario(&result, id);
        assert_eq!(
            (count(found, snapshots), count(found, comparisons)),
            (0, 0),
            "{id} at ess/22 keeps the stored-field observation: {:#?}",
            found.steps
        );
        assert_eq!(
            sends(found, outcome).len(),
            1,
            "{id} at ess/22 keeps its one arranged record: {:#?}",
            found.steps
        );
    }
}

/// Every single-document model under `root` whose format is below `ess/23`.
fn models_below_23(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !matches!(name.as_ref(), "target" | "node_modules" | ".git") {
                models_below_23(&path, out);
            }
            continue;
        }
        if !name.ends_with(".yaml") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let major = text
            .strip_prefix("format: ess/")
            .and_then(|rest| rest.lines().next())
            .and_then(|major| major.trim().parse::<u32>().ok());
        if major.is_some_and(|major| major < 23) {
            out.push(path);
        }
    }
}

#[test]
fn below_ess23_no_subject_fact_refusal_is_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut models = Vec::new();
    for area in ["examples", "crates"] {
        models_below_23(&root.join(area), &mut models);
    }
    let mut compiled = 0;
    for path in &models {
        let text = std::fs::read_to_string(path).expect("read");
        let Ok(raw) = RawSpecFile::parse(&text) else {
            continue;
        };
        let Ok(spec) = Specification::assemble([(Source::new("model.yaml"), raw)]) else {
            continue;
        };
        let Ok(ir) = compile(&spec, &SourceMap::new()) else {
            continue;
        };
        compiled += 1;
        for command in ir.commands().values() {
            for outcome in &command.outcomes {
                assert!(
                    !outcome.complete_refusal
                        || outcome.condition == ess_compiler::ir::ResolvedCondition::WrongState,
                    "{}: `{}/{}` below ess/23 carries `complete_refusal` beside a condition other \
                     than `wrong_state`",
                    path.display(),
                    command.name,
                    outcome.name
                );
            }
        }
    }
    assert!(
        compiled > 50,
        "the walk compiles the repository's models: {compiled} of {}",
        models.len()
    );
}

#[test]
fn partial_observer_keeps_published_fallback() {
    let partial = replaced(
        MODEL,
        "      - {name: state, type: demo.inst.Instance.State}\n      - {name: description, type: String}\n",
        "      - {name: state, type: demo.inst.Instance.State}\n",
    );
    let before = synthesis(&partial.replace("format: ess/23\n", "format: ess/22\n"));
    let after = synthesis(&partial);
    let refused = |result: &Synthesis| -> BTreeSet<String> {
        result
            .refusals
            .iter()
            .map(|refusal| format!("{refusal}"))
            .collect()
    };
    assert_eq!(
        refused(&after),
        refused(&before),
        "no new synthesis refusal where no view publishes `description`"
    );
    for (id, outcome) in [
        (SEED_REFUSED, "seed-change-refused"),
        (NOT_ACTIVE, "not-active"),
    ] {
        let found = sends(scenario(&after, id), outcome);
        assert!(
            !found.is_empty() && found.iter().all(|send| send.snapshot && send.compared),
            "{id} observes what the view publishes, before and after: {found:#?}"
        );
        assert!(
            after.notes.iter().any(|note| matches!(
                note,
                Note::PartialObservation { scenario, unobserved }
                    if scenario.to_string() == id && unobserved == &["description".to_owned()]
            )),
            "{id} names `description` as unobserved: {:#?}",
            after.notes
        );
    }
}

/// The issue's first arrangement: the stored-field refusal beside a state-guarded update.
fn lifecycle_mix() -> String {
    let mixed = replaced(
        MODEL,
        "      - name: not-active\n        when_subject:\n          predicate: state != Active\n        error: demo.inst.InstanceNotActive\n",
        "",
    );
    let mixed = replaced(
        &mixed,
        "      - name: updated\n        updates: demo.inst.Instance\n",
        "      - name: updated\n        when_subject_state: Active\n        updates: demo.inst.Instance\n",
    );
    replaced(
        &mixed,
        "        payload: {demo.inst.InstanceUpdated: {name: input.name}}\n",
        "        payload: {demo.inst.InstanceUpdated: {name: input.name}}\n      - {name: not-active, error: demo.inst.InstanceNotActive}\n",
    )
}

#[test]
fn lifecycle_mix_hint_names_state_predicate() {
    let text = lifecycle_mix();
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let errors = Specification::assemble([(Source::new("instance.yaml"), raw)])
        .expect_err("a stored-field guard beside a state guard is refused");
    let mut sources = SourceMap::new();
    sources.insert("instance.yaml".to_owned(), text.clone());
    let diagnostics = diagnose_locating(&errors, &sources, &["instance.yaml"]);
    let mix: Vec<_> = diagnostics
        .as_slice()
        .iter()
        .filter(|diagnostic| {
            diagnostic.message
                == "subject fact and lifecycle guards cannot be combined in one command"
        })
        .collect();
    assert!(!mix.is_empty(), "{diagnostics:#?}");
    for diagnostic in mix {
        assert_eq!(
            diagnostic.code.to_string(),
            "ESS-COMMAND-004",
            "{diagnostic:#?}"
        );
        let hint = diagnostic.hint.as_deref().unwrap_or_default();
        assert!(
            hint.contains("`when_subject: {predicate: state"),
            "the hint names the `state` predicate idiom: {diagnostic:#?}"
        );
    }
}

/// The `## Guard an outcome by the subject's stored fields` section of the guide.
fn stored_fields_section(page: &str) -> &str {
    let heading = "\n## Guard an outcome by the subject's stored fields\n";
    let start = page
        .find(heading)
        .unwrap_or_else(|| panic!("the guide has `{}`", heading.trim()))
        + 1;
    let rest = &page[start + heading.len() - 1..];
    let end = rest
        .find("\n## ")
        .map_or(page.len(), |at| start + heading.len() - 1 + at + 1);
    &page[start..end]
}

#[test]
fn refused_row_sentence_states_complete_observation() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/guards-and-predicates.md");
    let page = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let section = stored_fields_section(&page);
    for phrase in [
        "From `ess/23` a refused record is asserted unchanged in every field",
        "`when_subject: {predicate: state != Active}`",
        "`ESS-COMMAND-004`",
    ] {
        assert!(
            section.contains(phrase),
            "the stored-fields section says {phrase:?}:\n{section}"
        );
    }
}
