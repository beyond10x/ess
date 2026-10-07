//! Adversary, wave 2026-10-07c unit c3 (beyond10x/ess#474): a one-way cycle through rows beside.
//!
//! A task requires a blocker ticket; a ticket optionally names the task it originated from. Both
//! commands also read a required project, so each reads two rows and every row but the one a run
//! is arranged around is arranged beside it. A witness exists for every outcome of `CreateTask`:
//! open a project, open a ticket leaving its Optional `origin` out, create the task naming both.
//!
//! The fix leaves out an Optional reference only where its entity is *already* in the arranging
//! chain. Arranging the blocker ticket beside a task's project starts the chain at `Ticket`; the
//! ticket's `origin` row (a `Task`) is not in it yet, so the ticket's run tries to arrange a task
//! beside, that task needs a ticket that *is* in the chain through a required reference, and the
//! whole ticket arrangement is refused rather than falling back to leaving `origin` out.
//!
//! In a binary of its own because an overflow aborts every test beside it.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const ONE_WAY: &str = r"format: ess/22
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
      - {name: blocker, type: shop.tasks.TicketId}
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
      - {name: blocker, type: shop.tasks.TicketId}
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
const NO_SUCH_PROJECT: &str = "shop.tasks.CreateTask/outcome/no-such-project";
const NO_SUCH_BLOCKER: &str = "shop.tasks.CreateTask/outcome/no-such-blocker";
const CREATED: &str = "shop.tasks.CreateTask/outcome/created";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tasks.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn has_scenario(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .iter()
        .any(|(key, _)| key.to_string() == id)
}

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

fn statuses(model: &EssIr, result: &Synthesis, prefix: &str) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().starts_with(&format!("{prefix}/")))
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

#[test]
fn adv_c3_a_required_blocker_whose_ticket_optionally_names_a_task_is_witnessed() {
    let model = ir(ONE_WAY);
    let result = ess_conformance::synthesize::synthesize(&model);
    let missing: Vec<&str> = [NO_SUCH_PROJECT, NO_SUCH_BLOCKER, CREATED]
        .into_iter()
        .filter(|id| !has_scenario(&result, id))
        .collect();
    assert!(
        missing.is_empty(),
        "a ticket can be opened with `origin` left out, so every outcome of {CREATE} has a \
         witness; unwitnessed: {missing:#?}\nrefusals for {CREATE}: {:#?}",
        refusals_for(&result, CREATE)
    );
    let statuses = statuses(&model, &result, CREATE);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}
