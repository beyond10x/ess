//! A command reading two related rows through identity-addressed `when_related` guards, the second
//! through an Optional reference to the entity the command itself creates (beyond10x/ess#474).
//!
//! Every row a run is not arranged around is arranged beside it, and that arrangement started
//! again from an empty chain of entities being arranged. A row beside of the entity the run
//! creates was then created by the same run, which arranged the same row beside it again, without
//! end: synthesis aborted the process on a stack overflow. The same holds for a required
//! self-reference and for two entities referencing each other through rows beside.
//!
//! These tests live in a binary of their own because an overflow aborts every test beside it.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ScenarioId, ScenarioStep};
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The issue's reproducer as one file.
const MODEL: &str = r"format: ess/22
system: shop
version: v1
domain: shop.tasks
types:
  - {name: shop.tasks.ProjectId, kind: newtype, of: String}
  - {name: shop.tasks.TaskId, kind: newtype, of: String}
entities:
  - name: shop.tasks.Project
    identity: {name: project_id, type: shop.tasks.ProjectId}
    fields: [{name: title, type: String}]
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.tasks.Task
    identity: {name: task_id, type: shop.tasks.TaskId}
    fields:
      - {name: project_id, type: shop.tasks.ProjectId}
      - {name: waits_on, type: Optional<shop.tasks.TaskId>}
    relations:
      - {name: project, kind: references, target: shop.tasks.Project, cardinality: one, via: project_id}
      - {name: awaited, kind: references, target: shop.tasks.Task, cardinality: one, via: waits_on}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: shop.tasks.Clerk, may: [shop.tasks.CreateTask, shop.tasks.OpenProject]}
errors:
  - name: shop.tasks.TaskNotFound
    fields: [{name: task_id, type: shop.tasks.TaskId}]
commands:
  - name: shop.tasks.OpenProject
    input: [{name: project_id, type: shop.tasks.ProjectId}, {name: title, type: String}]
    outcomes:
      - name: opened
        creates: shop.tasks.Project
        instance: project_id
        sets: {title: input.title}
        emits: [shop.tasks.ProjectOpened]
        payload: {shop.tasks.ProjectOpened: {project_id: input.project_id}}
  - name: shop.tasks.CreateTask
    input:
      - {name: task_id, type: shop.tasks.TaskId}
      - {name: project_id, type: shop.tasks.ProjectId}
      - {name: waits_on, type: Optional<shop.tasks.TaskId>}
    outcomes:
      - name: no-such-project
        when_related: {via: input.project_id, exists: false}
        error: shop.tasks.TaskNotFound
        payload: {shop.tasks.TaskNotFound: {task_id: input.task_id}}
      - name: no-such-awaited
        when_related: {via: input.waits_on, exists: false}
        error: shop.tasks.TaskNotFound
        payload: {shop.tasks.TaskNotFound: {task_id: input.task_id}}
      - name: created
        creates: shop.tasks.Task
        instance: task_id
        sets: {waits_on: input.waits_on, project_id: input.project_id}
        emits: [shop.tasks.TaskCreated]
        payload: {shop.tasks.TaskCreated: {task_id: input.task_id}}
events:
  - name: shop.tasks.ProjectOpened
    fields: [{name: project_id, type: shop.tasks.ProjectId}]
  - name: shop.tasks.TaskCreated
    fields: [{name: task_id, type: shop.tasks.TaskId}]
views:
  - name: shop.tasks.Tasks
    source: shop.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: shop.tasks.TaskId}
      - {name: state, type: shop.tasks.Task.State}
      - {name: waits_on, type: Optional<shop.tasks.TaskId>}
";

/// Two entities referencing each other through Optional references, each command also reading a
/// required project: creating either arranges a row of the other beside the project.
const CYCLE: &str = r"format: ess/22
system: shop
version: v1
domain: shop.tasks
types:
  - {name: shop.tasks.ProjectId, kind: newtype, of: String}
  - {name: shop.tasks.TaskId, kind: newtype, of: String}
  - {name: shop.tasks.TicketId, kind: newtype, of: String}
entities:
  - name: shop.tasks.Project
    identity: {name: project_id, type: shop.tasks.ProjectId}
    fields: [{name: title, type: String}]
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.tasks.Task
    identity: {name: task_id, type: shop.tasks.TaskId}
    fields:
      - {name: project_id, type: shop.tasks.ProjectId}
      - {name: blocker, type: Optional<shop.tasks.TicketId>}
    relations:
      - {name: project, kind: references, target: shop.tasks.Project, cardinality: one, via: project_id}
      - {name: blocking, kind: references, target: shop.tasks.Ticket, cardinality: one, via: blocker}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.tasks.Ticket
    identity: {name: ticket_id, type: shop.tasks.TicketId}
    fields:
      - {name: project_id, type: shop.tasks.ProjectId}
      - {name: origin, type: Optional<shop.tasks.TaskId>}
    relations:
      - {name: project, kind: references, target: shop.tasks.Project, cardinality: one, via: project_id}
      - {name: origin_task, kind: references, target: shop.tasks.Task, cardinality: one, via: origin}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: shop.tasks.Clerk, may: [shop.tasks.CreateTask, shop.tasks.OpenProject, shop.tasks.OpenTicket]}
errors:
  - name: shop.tasks.NotFound
    fields: [{name: project_id, type: shop.tasks.ProjectId}]
commands:
  - name: shop.tasks.OpenProject
    input: [{name: project_id, type: shop.tasks.ProjectId}, {name: title, type: String}]
    outcomes:
      - name: opened
        creates: shop.tasks.Project
        instance: project_id
        sets: {title: input.title}
        emits: [shop.tasks.ProjectOpened]
        payload: {shop.tasks.ProjectOpened: {project_id: input.project_id}}
  - name: shop.tasks.CreateTask
    input:
      - {name: task_id, type: shop.tasks.TaskId}
      - {name: project_id, type: shop.tasks.ProjectId}
      - {name: blocker, type: Optional<shop.tasks.TicketId>}
    outcomes:
      - name: no-such-project
        when_related: {via: input.project_id, exists: false}
        error: shop.tasks.NotFound
        payload: {shop.tasks.NotFound: {project_id: input.project_id}}
      - name: no-such-blocker
        when_related: {via: input.blocker, exists: false}
        error: shop.tasks.NotFound
        payload: {shop.tasks.NotFound: {project_id: input.project_id}}
      - name: created
        creates: shop.tasks.Task
        instance: task_id
        sets: {blocker: input.blocker, project_id: input.project_id}
        emits: [shop.tasks.TaskCreated]
        payload: {shop.tasks.TaskCreated: {task_id: input.task_id}}
  - name: shop.tasks.OpenTicket
    input:
      - {name: ticket_id, type: shop.tasks.TicketId}
      - {name: project_id, type: shop.tasks.ProjectId}
      - {name: origin, type: Optional<shop.tasks.TaskId>}
    outcomes:
      - name: no-such-project
        when_related: {via: input.project_id, exists: false}
        error: shop.tasks.NotFound
        payload: {shop.tasks.NotFound: {project_id: input.project_id}}
      - name: no-such-origin
        when_related: {via: input.origin, exists: false}
        error: shop.tasks.NotFound
        payload: {shop.tasks.NotFound: {project_id: input.project_id}}
      - name: opened
        creates: shop.tasks.Ticket
        instance: ticket_id
        sets: {origin: input.origin, project_id: input.project_id}
        emits: [shop.tasks.TicketOpened]
        payload: {shop.tasks.TicketOpened: {ticket_id: input.ticket_id}}
events:
  - name: shop.tasks.ProjectOpened
    fields: [{name: project_id, type: shop.tasks.ProjectId}]
  - name: shop.tasks.TaskCreated
    fields: [{name: task_id, type: shop.tasks.TaskId}]
  - name: shop.tasks.TicketOpened
    fields: [{name: ticket_id, type: shop.tasks.TicketId}]
";

const CREATE: &str = "shop.tasks.CreateTask";
const OPEN_TICKET: &str = "shop.tasks.OpenTicket";
const NO_SUCH_PROJECT: &str = "shop.tasks.CreateTask/outcome/no-such-project";
const NO_SUCH_AWAITED: &str = "shop.tasks.CreateTask/outcome/no-such-awaited";
const CREATED: &str = "shop.tasks.CreateTask/outcome/created";
const TASK: &str = "shop.tasks.Task";

/// [`MODEL`] with the self-reference required: no task can be created before another exists.
fn required() -> String {
    let out = MODEL.replace(
        "type: Optional<shop.tasks.TaskId>",
        "type: shop.tasks.TaskId",
    );
    assert_ne!(out, MODEL, "the reference is Optional in the fixture");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tasks.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every outcome the model declares, as the scenario id synthesis files it under.
fn outcomes(model: &EssIr) -> Vec<String> {
    model
        .commands()
        .iter()
        .flat_map(|(name, command)| {
            command
                .outcomes
                .iter()
                .map(move |outcome| format!("{name}/outcome/{}", outcome.name))
        })
        .collect()
}

fn has_scenario(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .iter()
        .any(|(key, _)| key.to_string() == id)
}

/// The refusals naming `id` as the scenario they withhold, each as `code: cause`.
fn refused(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

/// Every refusal whose debug form names `command`, each as `code [scenario]: cause`.
fn refusals_for(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| {
            format!(
                "{} [{}]: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(|| format!("{:?}", refusal.subject), ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn scenario<'s>(result: &'s Synthesis, id: &str) -> &'s ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("{id} is witnessed; refusals: {:#?}", result.refusals),
            |(_, scenario)| scenario,
        )
}

/// Each outcome of `model` has a scenario, or a refusal naming that scenario.
fn assert_witnessed_or_refused(model: &EssIr, result: &Synthesis) {
    let unaccounted: Vec<String> = outcomes(model)
        .into_iter()
        .filter(|id| !has_scenario(result, id) && refused(result, id).is_empty())
        .collect();
    assert!(
        unaccounted.is_empty(),
        "every outcome is witnessed or refused by name; neither: {unaccounted:#?}\nrefusals: {:#?}",
        result.refusals
    );
}

/// The status of every scenario of the commands `prefixes` name, run against the interpreter of
/// the model the suite was synthesized from.
fn statuses(model: &EssIr, result: &Synthesis, prefixes: &[&str]) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| {
            let id = run.scenario.to_string();
            prefixes
                .iter()
                .any(|prefix| id.starts_with(&format!("{prefix}/")))
        })
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn assert_all_pass(statuses: &BTreeMap<String, Status>) {
    let failing: Vec<(&String, &Status)> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failing.is_empty(),
        "every synthesized scenario passes against the model's own interpreter: {failing:#?}"
    );
}

#[test]
fn issue_474_each_outcome_is_witnessed_or_refused_by_name() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_witnessed_or_refused(&model, &result);
}

#[test]
fn issue_474_every_outcome_of_the_command_is_witnessed_and_passes() {
    // The same model with the two guards declared the other way round synthesizes every outcome
    // with no refusal on ess 0.55.0; the declaration order of two `exists: false` branches over
    // different rows changes which is answered first, not whether each can be witnessed.
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refusals = refusals_for(&result, CREATE);
    assert!(refusals.is_empty(), "{CREATE}: {refusals:#?}");
    for id in [NO_SUCH_PROJECT, NO_SUCH_AWAITED, CREATED] {
        assert!(
            has_scenario(&result, id),
            "{id} is witnessed; refusals: {:#?}",
            result.refusals
        );
    }
    let statuses = statuses(&model, &result, &[CREATE]);
    for id in [NO_SUCH_PROJECT, NO_SUCH_AWAITED, CREATED] {
        assert!(statuses.contains_key(id), "{id} was run: {statuses:#?}");
    }
    assert_all_pass(&statuses);
}

#[test]
fn issue_474_the_awaited_task_is_created_before_the_task_that_waits_on_it() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let created = scenario(&result, CREATED);
    let mut captured_tasks = Vec::new();
    let mut pending: Option<Option<String>> = None;
    let mut awaited = Vec::new();
    for step in &created.steps {
        match step {
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == TASK => captured_tasks.push(instance.to_string()),
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == CREATE =>
            {
                pending = Some(match input.get("waits_on") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    _ => None,
                });
            }
            ScenarioStep::ExecuteCommand { .. } => pending = None,
            ScenarioStep::ExpectOutcome { outcome } => {
                let id = ScenarioId::Outcome {
                    outcome: outcome.clone(),
                }
                .to_string();
                if let (Some(Some(named)), true) = (pending.take(), id == CREATED) {
                    let before = captured_tasks.contains(&named);
                    awaited.push((named, before));
                }
            }
            _ => {}
        }
    }
    assert!(
        !awaited.is_empty(),
        "{CREATED} is sent naming an awaited task: {:#?}",
        created.steps
    );
    assert!(
        awaited.iter().all(|(_, before)| *before),
        "every awaited task {CREATED} names was captured earlier in the scenario: {awaited:#?}"
    );
}

#[test]
fn issue_474_a_required_self_reference_is_refused_by_name_rather_than_overflowing() {
    let text = required();
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_witnessed_or_refused(&model, &result);
    // No task can exist before another does, so no scenario that reaches `created` is sound; one
    // that is synthesized at all still has to pass.
    assert_all_pass(&statuses(&model, &result, &[CREATE]));
}

#[test]
fn issue_474_two_entities_referencing_each_other_beside_a_project_synthesize() {
    let model = ir(CYCLE);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_witnessed_or_refused(&model, &result);
    assert_all_pass(&statuses(&model, &result, &[CREATE, OPEN_TICKET]));
}
