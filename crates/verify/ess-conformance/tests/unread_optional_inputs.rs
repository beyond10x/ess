//! Synthesis leaves an `Optional` input absent unless the scenario needs it present
//! (<https://github.com/beyond10x/ess/issues/546>).
//!
//! The model is a queue: `Submit` takes the instant an item was queued and an optional deadline,
//! and stores both; `Withdraw` moves a queued item. No guard reads the deadline. Up to 0.59.0 every
//! synthesized `Submit` step, the arranging ones included, carried the deadline, and carried it at
//! the same literal as the queue instant: every synthesized item claimed to expire the moment it
//! was queued, and an implementation that expires items by their deadline failed scenarios that
//! never mention expiry.
//!
//! Where a scenario does need the deadline — the branch it observes copies it, a guard reads it, or
//! a later step reads the stored copy — it is sent, and not at the queue instant's literal. Every
//! other step, and every step arranging an item for another branch, leaves it out.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/24
system: demo
version: v1
domain: demo.queue
types:
  - {name: demo.queue.ItemId, kind: newtype, of: Uuid}
errors:
  - {name: demo.queue.NotQueued, summary: The item is not queued., fields: []}
  - {name: demo.queue.NoDeadline, summary: The item has no deadline., fields: []}
entities:
  - name: demo.queue.Item
    identity: {name: item_id, type: demo.queue.ItemId}
    fields:
      - {name: queued_at, type: Timestamp}
      - {name: expires_at, type: Optional<Timestamp>}
    lifecycle:
      initial: Queued
      states: [Queued, Withdrawn]
      terminal: [Withdrawn]
      transitions:
        - {name: withdraw, from: [Queued], to: Withdrawn}
events:
  - name: demo.queue.Submitted
    fields: [{name: item_id, type: demo.queue.ItemId}]
  - name: demo.queue.Withdrawn
    fields: [{name: item_id, type: demo.queue.ItemId}]
actors:
  - {name: demo.queue.Client, may: [demo.queue.Submit, demo.queue.Withdraw]}
commands:
  - name: demo.queue.Submit
    input:
      - {name: queued_at, type: Timestamp}
      - {name: expires_at, type: Optional<Timestamp>}
    outcomes:
      - name: submitted
        creates: demo.queue.Item
        instance: item_id
        emits: [demo.queue.Submitted]
        payload:
          demo.queue.Submitted: {item_id: {generated: true}}
        sets:
          queued_at: input.queued_at
          expires_at: input.expires_at
  - name: demo.queue.Withdraw
    input:
      - {name: item_id, type: demo.queue.ItemId}
    outcomes:
      - name: withdrawn
        moves: demo.queue.Item.withdraw
        instance: item_id
        emits: [demo.queue.Withdrawn]
        payload:
          demo.queue.Withdrawn: {item_id: input.item_id}
      - name: not-queued
        wrong_state: true
        error: demo.queue.NotQueued
views:
  - name: demo.queue.Items
    source: demo.queue.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.queue.ItemId}
      - {name: queued_at, type: Timestamp}
      - {name: expires_at, type: Optional<Timestamp>}
";

/// A later step reads the stored deadline: `Expire` expires an item only where it has one.
const EXPIRE: &str = r"  - name: demo.queue.Expire
    input:
      - {name: item_id, type: demo.queue.ItemId}
    outcomes:
      - name: no-deadline
        when_subject: {predicate: not defined(expires_at)}
        error: demo.queue.NoDeadline
      - name: expired
        moves: demo.queue.Item.expire
        instance: item_id
        emits: [demo.queue.Expired]
        payload:
          demo.queue.Expired: {item_id: input.item_id}
      - name: not-queued
        wrong_state: true
        error: demo.queue.NotQueued
views:
";

/// `Submit` itself reads the deadline: a submission carrying one is announced apart.
const GUARDED: &str = r"    outcomes:
      - name: with-deadline
        when: defined(expires_at)
        creates: demo.queue.Item
        instance: item_id
        emits: [demo.queue.Submitted]
        payload:
          demo.queue.Submitted: {item_id: {generated: true}}
        sets:
          queued_at: input.queued_at
          expires_at: input.expires_at
      - name: submitted
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model with `Expire` added: a later step that reads the stored deadline.
fn with_expire() -> String {
    let text = MODEL
        .replacen("views:\n", EXPIRE, 1)
        .replacen(
            "      states: [Queued, Withdrawn]\n      terminal: [Withdrawn]\n",
            "      states: [Queued, Withdrawn, Expired]\n      terminal: [Withdrawn, Expired]\n",
            1,
        )
        .replacen(
            "        - {name: withdraw, from: [Queued], to: Withdrawn}\n",
            "        - {name: withdraw, from: [Queued], to: Withdrawn}\n        \
             - {name: expire, from: [Queued], to: Expired}\n",
            1,
        )
        .replacen(
            "actors:\n",
            "  - name: demo.queue.Expired\n    fields: [{name: item_id, type: demo.queue.ItemId}]\n\
             actors:\n",
            1,
        )
        .replacen(
            "demo.queue.Withdraw]}",
            "demo.queue.Withdraw, demo.queue.Expire]}",
            1,
        );
    assert!(text.contains("demo.queue.Expire\n") && text.contains("to: Expired}"));
    text
}

/// The model with a `Submit` branch whose guard reads the deadline.
fn guarded() -> String {
    let text = MODEL.replacen("    outcomes:\n      - name: submitted\n", GUARDED, 1);
    assert!(text.contains("with-deadline"));
    text
}

/// Every `Submit` step of the suite, by scenario, with its literal input.
fn submits(result: &Synthesis) -> Vec<(String, BTreeMap<String, Node>)> {
    result
        .suite
        .scenarios
        .iter()
        .flat_map(|(id, scenario)| {
            scenario.steps.iter().filter_map(move |step| match step {
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.queue.Submit" =>
                {
                    let literal = input
                        .iter()
                        .filter_map(|(name, value)| match value {
                            ScenarioValue::Literal { value } => Some((name.clone(), value.clone())),
                            _ => None,
                        })
                        .collect();
                    Some((id.to_string(), literal))
                }
                _ => None,
            })
        })
        .collect()
}

fn statuses(model: &EssIr, result: &Synthesis) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn assert_all_pass(model: &EssIr, result: &Synthesis) {
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let failing: Vec<_> = statuses(model, result)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}

/// The `Submit` branch under test copies the deadline, so its own scenario sends it, apart from the
/// queue instant; every scenario that only arranges an item through `Submit` leaves it out.
#[test]
fn issue_546_an_optional_input_only_copied_is_absent_from_every_arranging_step() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let own = "demo.queue.Submit/outcome/submitted";
    let arranging: Vec<_> = steps
        .iter()
        .filter(|(scenario, _)| scenario != own)
        .collect();
    assert!(arranging.len() >= 3, "{steps:#?}");
    for (scenario, input) in &steps {
        assert!(
            input.contains_key("queued_at"),
            "{scenario}: the required input is sent: {input:?}"
        );
    }
    for (scenario, input) in arranging {
        assert!(
            !input.contains_key("expires_at"),
            "{scenario}: no guard, invariant or later step reads the deadline, and this scenario \
             does not observe `submitted`'s copy, so it is not sent: {input:?}"
        );
    }
    let observed: Vec<_> = steps
        .iter()
        .filter(|(scenario, _)| scenario == own)
        .collect();
    assert!(!observed.is_empty(), "{steps:#?}");
    for (_, input) in observed {
        let deadline = input
            .get("expires_at")
            .unwrap_or_else(|| panic!("`submitted` copies the deadline it observes: {input:?}"));
        assert_ne!(
            Some(deadline),
            input.get("queued_at"),
            "the deadline is not the queue instant's literal: {input:?}"
        );
    }
    assert_all_pass(&model, &result);
}

#[test]
fn issue_546_a_later_step_reading_the_stored_copy_gets_it_present_and_apart() {
    let model = ir(&with_expire());
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let in_scenario = |id: &str| {
        steps
            .iter()
            .filter(|(scenario, _)| scenario == id)
            .map(|(_, input)| input)
            .collect::<Vec<_>>()
    };
    let expired = in_scenario("demo.queue.Expire/outcome/expired");
    assert_eq!(expired.len(), 1, "{steps:#?}");
    let deadline = expired[0]
        .get("expires_at")
        .unwrap_or_else(|| panic!("`expired` reads the stored deadline: {:?}", expired[0]));
    assert_ne!(
        Some(deadline),
        expired[0].get("queued_at"),
        "the deadline is not the queue instant's literal: {:?}",
        expired[0]
    );
    for id in [
        "demo.queue.Expire/outcome/no-deadline",
        "demo.queue.Withdraw/outcome/withdrawn",
    ] {
        let found = in_scenario(id);
        assert!(!found.is_empty(), "{id}: {steps:#?}");
        for input in found {
            assert!(!input.contains_key("expires_at"), "{id}: {input:?}");
        }
    }
    assert_all_pass(&model, &result);
}

#[test]
fn issue_546_a_guard_reading_the_optional_gets_it_present_and_apart() {
    let model = ir(&guarded());
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let with = steps
        .iter()
        .find(|(scenario, _)| scenario == "demo.queue.Submit/outcome/with-deadline")
        .map_or_else(|| panic!("{steps:#?}"), |(_, input)| input);
    let deadline = with
        .get("expires_at")
        .unwrap_or_else(|| panic!("the guard needs the deadline present: {with:?}"));
    assert_ne!(
        Some(deadline),
        with.get("queued_at"),
        "the deadline is not the queue instant's literal: {with:?}"
    );
    let without = steps
        .iter()
        .find(|(scenario, _)| scenario == "demo.queue.Submit/outcome/submitted")
        .map_or_else(|| panic!("{steps:#?}"), |(_, input)| input);
    assert!(!without.contains_key("expires_at"), "{without:?}");
    assert_all_pass(&model, &result);
}

/// A view filtering on the stored copy: `LongJobs` shows a job by its optional `hours`.
const FILTERED: &str = r"format: ess/18
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
errors:
  - {name: demo.jobs.AlreadyDone, summary: The job is already done., fields: []}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: hours, type: Optional<Integer>}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
events:
  - name: demo.jobs.Created
    fields: [{name: job_id, type: demo.jobs.JobId}]
  - name: demo.jobs.Finished
    fields: [{name: job_id, type: demo.jobs.JobId}]
actors:
  - {name: demo.jobs.Planner, may: [demo.jobs.CreateJob, demo.jobs.FinishJob]}
commands:
  - name: demo.jobs.CreateJob
    input:
      - {name: hours, type: Optional<Integer>}
    outcomes:
      - name: created
        creates: demo.jobs.Job
        instance: job_id
        sets: {hours: input.hours}
        emits: [demo.jobs.Created]
        payload:
          demo.jobs.Created: {job_id: {generated: true}}
  - name: demo.jobs.FinishJob
    input:
      - {name: job_id, type: demo.jobs.JobId}
    outcomes:
      - name: finished
        moves: demo.jobs.Job.finish
        instance: job_id
        emits: [demo.jobs.Finished]
        payload:
          demo.jobs.Finished: {job_id: input.job_id}
      - {name: already-done, wrong_state: true, error: demo.jobs.AlreadyDone}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: state, type: demo.jobs.Job.State}
      - {name: hours, type: Optional<Integer>}
  - name: demo.jobs.LongJobs
    source: demo.jobs.Job
    consistency: read_your_writes
    filter: hours >= 5
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: hours, type: Optional<Integer>}
";

/// A view filter over the stored copy reads it: the scenario querying `LongJobs` after `FinishJob`
/// decides which rows it shows by the hours sent, so the arranging `CreateJob` sends them. Left out,
/// the filter is undecided and the scenario skips its read of the view (`ESS-SYNTH-005`).
#[test]
fn issue_546_a_view_filter_over_the_stored_copy_reads_it() {
    let model = ir(FILTERED);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let finished = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.jobs.FinishJob/outcome/finished")
        .map_or_else(|| panic!("no finished scenario"), |(_, scenario)| scenario);
    let created = finished.steps.iter().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "demo.jobs.CreateJob" =>
        {
            Some(input)
        }
        _ => None,
    });
    let created = created.unwrap_or_else(|| panic!("{:#?}", finished.steps));
    assert!(created.contains_key("hours"), "{created:?}");
    assert_all_pass(&model, &result);
}

/// The deadline an arranging `Submit` left out is absent on the row `Withdraw` moves, and the view
/// declares no `null_when_absent`, so the row may leave the key out or carry `null`: the row
/// `withdrawn` asserts claims neither spelling (a generated Rust or Go server leaves it out).
#[test]
fn issue_546_an_absent_copy_the_branch_does_not_write_is_not_claimed_as_null() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let withdrawn = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.queue.Withdraw/outcome/withdrawn")
        .map_or_else(|| panic!("no withdrawn scenario"), |(_, scenario)| scenario);
    let rows: Vec<_> = withdrawn
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ess_conformance::ViewExpectation::Contains { fields },
                ..
            } => Some(fields),
            _ => None,
        })
        .collect();
    assert!(!rows.is_empty(), "{:#?}", withdrawn.steps);
    for fields in rows {
        assert!(fields.contains_key("item_id"), "{fields:?}");
        assert!(!fields.contains_key("expires_at"), "{fields:?}");
    }
    assert_all_pass(&model, &result);
}

/// `Remind` copies the deadline of the item it names, `{related: {via: input.item_id, field:
/// expires_at}}`, onto a reminder of its own.
const REMIND: &str = r"  - name: demo.queue.Remind
    input:
      - {name: item_id, type: demo.queue.ItemId}
    outcomes:
      - name: no-item
        when_related: {via: input.item_id, exists: false}
        error: demo.queue.NotQueued
      - name: reminded
        creates: demo.queue.Reminder
        instance: reminder_id
        emits: [demo.queue.Reminded]
        payload:
          demo.queue.Reminded: {reminder_id: {generated: true}}
        sets:
          item_id: input.item_id
          due: {related: {via: input.item_id, field: expires_at}}
views:
  - name: demo.queue.Reminders
    source: demo.queue.Reminder
    consistency: read_your_writes
    fields:
      - {name: reminder_id, type: demo.queue.ReminderId}
      - {name: item_id, type: demo.queue.ItemId}
      - {name: due, type: Optional<Timestamp>}
";

/// The model with `Remind` added: a later command that copies the stored deadline as a value.
fn with_remind() -> String {
    let text = MODEL
        .replacen("views:\n", REMIND, 1)
        .replacen(
            "  - {name: demo.queue.ItemId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.queue.ItemId, kind: newtype, of: Uuid}\n  \
             - {name: demo.queue.ReminderId, kind: newtype, of: Uuid}\n",
            1,
        )
        .replacen(
            "events:\n",
            "  - name: demo.queue.Reminder\n    identity: {name: reminder_id, type: \
             demo.queue.ReminderId}\n    fields:\n      - {name: item_id, type: \
             demo.queue.ItemId}\n      - {name: due, type: Optional<Timestamp>}\n    lifecycle: \
             {initial: Set, states: [Set], terminal: [Set]}\nevents:\n",
            1,
        )
        .replacen(
            "actors:\n",
            "  - name: demo.queue.Reminded\n    fields: [{name: reminder_id, type: \
             demo.queue.ReminderId}]\nactors:\n",
            1,
        )
        .replacen(
            "demo.queue.Withdraw]}",
            "demo.queue.Withdraw, demo.queue.Remind]}",
            1,
        );
    assert!(text.contains("demo.queue.Remind\n") && text.contains("name: demo.queue.Reminder\n"));
    text
}

/// A later command copying the stored copy as a value reads it: `reminded` observes the deadline
/// of the item it names, so the `Submit` arranging that item sends one. Left out, every item holds
/// none, and a target copying the deadline of another item, or none, passes.
#[test]
fn issue_546_a_related_value_read_of_the_stored_copy_reads_it() {
    let model = ir(&with_remind());
    let result = ess_conformance::synthesize::synthesize(&model);
    let steps = submits(&result);
    let reminded: Vec<_> = steps
        .iter()
        .filter(|(scenario, _)| scenario == "demo.queue.Remind/outcome/reminded")
        .map(|(_, input)| input)
        .collect();
    assert!(!reminded.is_empty(), "{steps:#?}");
    for input in reminded {
        assert!(
            input.contains_key("expires_at"),
            "`reminded` copies the stored deadline, so the item it names is arranged with one: \
             {input:?}"
        );
    }
    assert_all_pass(&model, &result);
}
