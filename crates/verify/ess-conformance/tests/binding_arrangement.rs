//! Bindings that move state, in arrangement and in what a binding scenario observes
//! (beyond10x/ess#266, beyond10x/ess#267, `docs/design/binding-arrangement-and-drop.md`).
//!
//! The fixture is one `Job` whose `Created` event starts it through `created-starts`, a second
//! creation (`Register`) that sets nothing off, and `Kick`, which names a job it does not own so
//! that `kicked-starts` addresses a row arranged before its trigger. The interpreter runs bindings
//! through its own dispatcher, so it is the honest binding-running target; every faulty target below
//! is the same dispatcher with one thing wrong.

mod support_binding_arrangement;

use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::report::{CheckCode, ConformanceReport, Status};
use ess_conformance::scenario::{BindingRef, CommandRef, ConformanceSuite, ScenarioStep};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner, ScenarioId};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;
use serde_json::Value;
use support_binding_arrangement::*;

/// The minimal reproduction attached to beyond10x/ess#266, in one document: `Create` mints the
/// job's identity, so its own job is the only one `created-starts` can address.
const REPRO: &str = r"
format: ess/18
system: repro
version: v1
domain: repro.job
types:
  - {name: repro.job.JobId, kind: newtype, of: Uuid}
entities:
  - name: repro.job.Job
    identity: {name: job_id, type: repro.job.JobId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: New
      states: [New, Started]
      terminal: [Started]
      transitions:
        - {name: start, from: [New], to: Started}
errors:
  - name: repro.job.JobStateConflict
    summary: The job is not New.
    fields:
      - {name: state, type: repro.job.Job.State}
commands:
  - name: repro.job.Create
    input:
      - {name: label, type: String}
    outcomes:
      - name: created
        creates: repro.job.Job
        instance: job_id
        sets: {label: input.label}
        emits: [repro.job.Created]
        payload: {repro.job.Created: {job_id: {generated: true}}}
  - name: repro.job.Start
    input:
      - {name: job_id, type: repro.job.JobId}
    outcomes:
      - name: started
        moves: repro.job.Job.start
        instance: job_id
        emits: [repro.job.Started]
        payload: {repro.job.Started: {job_id: input.job_id}}
      - {name: wrong-state, wrong_state: true, error: repro.job.JobStateConflict}
events:
  - name: repro.job.Created
    fields: [{name: job_id, type: repro.job.JobId}]
  - name: repro.job.Started
    fields: [{name: job_id, type: repro.job.JobId}]
views:
  - name: repro.job.Jobs
    source: repro.job.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: repro.job.JobId}
      - {name: label, type: String}
      - {name: state, type: repro.job.Job.State}
bindings:
  - id: created-starts
    when: {event: repro.job.Created}
    invoke: {command: repro.job.Start}
    mapping: {job_id: event.job_id}
    delivery: at_least_once
    on_failure: drop
";

// ---- the model, the suite and a run ------------------------------------------------------------

fn synth(text: &str) -> Synthesis {
    synthesize(&model(text))
}

fn run_against<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> ConformanceReport {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
}

fn status(report: &ConformanceReport, id: &str) -> Status {
    report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` ran: {:?}", ids_of(report)))
        .status
}

fn ids_of(report: &ConformanceReport) -> Vec<String> {
    report
        .scenarios
        .iter()
        .map(|scenario| scenario.scenario.to_string())
        .collect()
}

/// Every scenario that did not pass, with its checks that did not pass.
fn not_passed(report: &ConformanceReport) -> BTreeMap<String, Vec<String>> {
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                scenario
                    .checks
                    .iter()
                    .filter(|check| check.status != Status::Passed)
                    .map(|check| format!("{:?} {:?} {}", check.status, check.code, check.about))
                    .collect(),
            )
        })
        .collect()
}

fn failed_checks(report: &ConformanceReport, id: &str) -> Vec<(CheckCode, String)> {
    report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` ran"))
        .checks
        .iter()
        .filter(|check| check.status == Status::Failed)
        .map(|check| (check.code, check.about.clone()))
        .collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| {
            panic!(
                "`{id}` is synthesized: {:?}\nrefused: {}",
                synthesis
                    .suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                refusals(synthesis).join("\n")
            )
        })
        .steps
        .iter()
        .map(|step| serde_json::to_value(step).unwrap())
        .collect()
}

fn kinds(steps: &[Value]) -> Vec<String> {
    steps
        .iter()
        .map(|step| step["step"].as_str().unwrap().to_owned())
        .collect()
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} :: {:?}",
                refusal
                    .scenario
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                refusal.cause
            )
        })
        .collect()
}

/// The refusals filed under `id`, as `Debug` text.
fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{:?} / {refusal}", refusal.cause))
        .collect()
}

// ---- the two regression readings ---------------------------------------------------------------

/// Every scenario that sends `bound` to a row a step of the same scenario just made with `trigger`,
/// where the trigger's event sets the binding off on that very row: the race #266 reported. An
/// eventual view read between the two is the observation that the binding has settled the row, and
/// a send after it is no race.
fn races(suite: &ConformanceSuite, trigger_event: &str, bound: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let mut made: BTreeSet<String> = BTreeSet::new();
        let mut last_trigger_input: Option<Value> = None;
        for step in &scenario.steps {
            let step = serde_json::to_value(step).unwrap();
            match step["step"].as_str().unwrap() {
                "capture_instance" if step["event"] == trigger_event => {
                    made.insert(step["instance"].as_str().unwrap().to_owned());
                }
                "execute_command" if step["command"] == bound => {
                    let job = &step["input"]["job_id"];
                    let names_made = job["kind"] == "instance"
                        && made.contains(job["instance"].as_str().unwrap());
                    let names_trigger = job["kind"] == "observed" && job["event"] == trigger_event;
                    let same_literal = last_trigger_input.as_ref() == Some(job);
                    if names_made || names_trigger || same_literal {
                        found.push(id.to_string());
                        break;
                    }
                }
                "execute_command" if step["command"].as_str().unwrap().ends_with(".Create") => {
                    last_trigger_input = Some(step["input"]["job_id"].clone());
                }
                "eventually_view" => {
                    let job = &step["expectation"]["fields"]["job_id"];
                    if job["kind"] == "instance" {
                        made.remove(job["instance"].as_str().unwrap());
                    }
                    if !job.is_null() {
                        last_trigger_input = None;
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// Every immediate view expectation asserting `state` for a row, by scenario.
fn immediate_states(suite: &ConformanceSuite, state: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            let step = serde_json::to_value(step).unwrap();
            if step["step"] == "expect_view"
                && step["expectation"]["expect"] == "contains"
                && step["expectation"]["fields"]["state"]["value"] == state
            {
                found.push(id.to_string());
            }
        }
    }
    found
}

/// Every eventual view expectation asserting `state`, by scenario.
fn eventual_states(suite: &ConformanceSuite, state: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            let step = serde_json::to_value(step).unwrap();
            if step["step"] == "eventually_view"
                && step["expectation"]["expect"] == "contains"
                && step["expectation"]["fields"]["state"]["value"] == state
            {
                found.push(id.to_string());
            }
        }
    }
    found
}

// ---- beyond10x/ess#266 ---------------------------------------------------------------------------

#[test]
fn issue_266_minimal_reproduction_never_races_the_binding() {
    let synthesis = synth(REPRO);
    let suite = &synthesis.suite;

    // No scenario sends `Start` to the job `Create` just made.
    assert_eq!(
        races(suite, "repro.job.Created", "repro.job.Start"),
        Vec::<String>::new()
    );
    // `Start/outcome/started` cannot be arranged without racing the binding: every job begins in
    // `New` through `Create`, and `created-starts` moves it on. Named in coverage, with its witness.
    assert!(!suite
        .scenarios
        .contains_key(&ScenarioId::parse("repro.job.Start/outcome/started").unwrap()));
    let named = refused_as(&synthesis, "repro.job.Start/outcome/started");
    assert!(
        named
            .iter()
            .any(|text| text.contains("created-starts") && text.contains("binding/flow")),
        "{named:?}\n{}",
        refusals(&synthesis).join("\n")
    );
    // No view expectation after `Create` asserts `New`; an eventual one asserts `Started`.
    assert_eq!(immediate_states(suite, "New"), Vec::<String>::new());
    assert_eq!(eventual_states(suite, "New"), Vec::<String>::new());
    assert!(
        eventual_states(suite, "Started").contains(&"repro.job.Create/outcome/created".to_owned()),
        "{:#?}",
        steps_of(&synthesis, "repro.job.Create/outcome/created")
    );

    // A target that runs its bindings passes; one that does not fails the flow.
    let honest = run_against(suite, &interpreted(REPRO));
    assert_eq!(not_passed(&honest), BTreeMap::new());
    let unbound = rewrite(
        REPRO,
        "bindings:\n  - id: created-starts\n    when: {event: repro.job.Created}\n    invoke: {command: repro.job.Start}\n    mapping: {job_id: event.job_id}\n    delivery: at_least_once\n    on_failure: drop\n",
        "",
    );
    let disabled = run_against(suite, &interpreted(&unbound));
    assert_eq!(
        status(&disabled, "created-starts/binding/flow"),
        Status::Failed
    );
}

#[test]
fn binding_arrangement_waits_for_started() {
    let synthesis = synth(FIXTURE);
    let suite = &synthesis.suite;
    assert_eq!(
        races(suite, "jobs.job.Created", "jobs.job.Start"),
        Vec::<String>::new()
    );
    // The row `Create` makes is asserted `Started`, eventually, and never `New` at once.
    assert!(!immediate_states(suite, "New").contains(&"jobs.job.Create/outcome/created".to_owned()));
    assert!(
        eventual_states(suite, "Started").contains(&"jobs.job.Create/outcome/created".to_owned())
    );
    // `Started` is arranged through `Create` and an eventual observation, not a second `Start`.
    let refused = steps_of(
        &synthesis,
        "jobs.job.Job/state/Started/refuses/jobs.job.Start",
    );
    assert_eq!(
        kinds(&refused)[..4],
        [
            "execute_command",
            "expect_outcome",
            "capture_instance",
            "eventually_view"
        ],
        "{refused:#?}"
    );
    assert_eq!(
        refused
            .iter()
            .filter(|step| step["step"] == "execute_command" && step["command"] == "jobs.job.Start")
            .count(),
        1,
        "only the refused `Start` itself: {refused:#?}"
    );

    let honest = run_against(suite, &interpreted(FIXTURE));
    assert_eq!(not_passed(&honest), BTreeMap::new());

    // The two regression readings are not vacuous: the shapes #266 reported are found, and fail.
    let mut raced = suite.clone();
    let id = ScenarioId::parse("jobs.job.Job/state/Started/refuses/jobs.job.Start").unwrap();
    let scenario = raced.scenarios.get_mut(&id).unwrap();
    let start: ScenarioStep = serde_json::from_value(serde_json::json!({
        "step": "execute_command",
        "command": "jobs.job.Start",
        "input": {"job_id": {"kind": "instance", "instance": "job"}}
    }))
    .unwrap();
    let started: ScenarioStep = serde_json::from_value(serde_json::json!({
        "step": "expect_outcome",
        "outcome": {"command": "jobs.job.Start", "outcome": "started"}
    }))
    .unwrap();
    scenario.steps.splice(3..4, [start, started]);
    assert_eq!(
        races(&raced, "jobs.job.Created", "jobs.job.Start"),
        vec![id.to_string()]
    );
    assert_eq!(
        status(&run_against(&raced, &interpreted(FIXTURE)), &id.to_string()),
        Status::Failed
    );

    let mut immediate = suite.clone();
    let id = ScenarioId::parse("jobs.job.Create/outcome/created").unwrap();
    let scenario = immediate.scenarios.get_mut(&id).unwrap();
    let query: ScenarioStep = serde_json::from_value(serde_json::json!({
        "step": "query_view", "view": "jobs.job.Jobs"
    }))
    .unwrap();
    let new: ScenarioStep = serde_json::from_value(serde_json::json!({
        "step": "expect_view", "view": "jobs.job.Jobs",
        "expectation": {"expect": "contains", "fields": {
            "job_id": {"kind": "observed", "event": "jobs.job.Created", "field": "job_id"},
            "state": {"kind": "literal", "value": "New"}
        }}
    }))
    .unwrap();
    scenario.steps.extend([query, new]);
    assert!(immediate_states(&immediate, "New").contains(&id.to_string()));
    assert_eq!(
        status(
            &run_against(&immediate, &interpreted(FIXTURE)),
            &id.to_string()
        ),
        Status::Failed
    );
}

#[test]
fn binding_suites_are_deterministic_and_keep_the_fresh_format() {
    // Two syntheses of one model are one document, and the new scenarios use only instructions
    // the fresh format already carries: no reader is asked to understand anything new.
    for text in [FIXTURE, REPRO] {
        let first = synth(text).suite.to_canonical_json().unwrap();
        let second = synth(text).suite.to_canonical_json().unwrap();
        assert_eq!(first, second);
        let suite = synth(text).suite;
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/34"
        );
        AdmittedSuite::from_json(&first).expect("the canonical bytes are admitted");
    }
}

#[test]
fn binding_disabled_fails_flow() {
    let suite = synth(FIXTURE).suite;
    let disabled = run_against(&suite, &interpreted(&rewrite(FIXTURE, CREATED_STARTS, "")));
    assert_eq!(
        status(&disabled, "created-starts/binding/flow"),
        Status::Failed
    );
    assert_eq!(
        status(&disabled, "created-starts/binding/mapping"),
        Status::Failed
    );
    let flow = failed_checks(&disabled, "created-starts/binding/flow");
    assert!(
        flow.iter()
            .any(|(code, _)| *code == CheckCode::EventualEvent)
            && flow
                .iter()
                .any(|(code, _)| *code == CheckCode::EventualView),
        "the missing consequence and the missing state: {flow:?}"
    );
    // The other binding still runs.
    assert_eq!(
        status(&disabled, "kicked-starts/binding/flow"),
        Status::Passed
    );
}

#[test]
fn binding_direct_route_is_preserved() {
    let synthesis = synth(FIXTURE);
    for id in [
        "jobs.job.Start/outcome/started",
        "jobs.job.Job/transition/start/by/jobs.job.Start/started",
    ] {
        let steps = steps_of(&synthesis, id);
        let commands: Vec<&str> = steps
            .iter()
            .filter(|step| step["step"] == "execute_command")
            .map(|step| step["command"].as_str().unwrap())
            .collect();
        assert_eq!(
            commands,
            ["jobs.job.Register", "jobs.job.Start"],
            "{id}: {steps:#?}"
        );
    }
}

// ---- beyond10x/ess#267 ---------------------------------------------------------------------------

#[test]
fn binding_wrong_state_destination_is_arranged() {
    let synthesis = synth(FIXTURE);
    let ten: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|text| {
            (text.contains("/binding/flow") || text.contains("/binding/delivery"))
                && text.contains("BranchUndecided")
        })
        .collect();
    assert_eq!(
        ten,
        Vec::<String>::new(),
        "no ESS-SYNTH-010 for wrong_state"
    );

    // The kicked job is arranged in `New` before the trigger, and the trigger names exactly it.
    let flow = steps_of(&synthesis, "kicked-starts/binding/flow");
    let sent: Vec<&Value> = flow
        .iter()
        .filter(|step| step["step"] == "execute_command")
        .collect();
    assert_eq!(sent.len(), 2, "{flow:#?}");
    assert_eq!(sent[0]["command"], "jobs.job.Register");
    assert_eq!(sent[1]["command"], "jobs.job.Kick");
    assert_eq!(sent[1]["input"]["job_id"]["kind"], "instance", "{flow:#?}");
    assert!(eventual_states(&synthesis.suite, "Started")
        .contains(&"kicked-starts/binding/flow".to_owned()));
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/delivery",
        "kicked-starts/binding/delivery",
    ] {
        steps_of(&synthesis, id);
    }

    let honest = run_against(&synthesis.suite, &interpreted(FIXTURE));
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/delivery",
        "kicked-starts/binding/flow",
        "kicked-starts/binding/delivery",
    ] {
        assert_eq!(status(&honest, id), Status::Passed, "{id}");
    }

    // A target that leaves the job `Stopped` where the binding is delivered fails both.
    let stopped = rewrite(
        FIXTURE,
        "        creates: jobs.job.Job\n        instance: job_id\n        sets: {label: input.label}\n        emits: [jobs.job.Created]",
        "        creates: jobs.job.Job\n        into: Stopped\n        instance: job_id\n        sets: {label: input.label}\n        emits: [jobs.job.Created]",
    );
    let wrong = run_against(&synthesis.suite, &interpreted(&stopped));
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/delivery",
    ] {
        assert_eq!(status(&wrong, id), Status::Failed, "{id}");
    }
    let stopped = rewrite(
        FIXTURE,
        "        creates: jobs.job.Job\n        instance: job_id\n        sets: {label: input.label}\n        emits: [jobs.job.Registered]",
        "        creates: jobs.job.Job\n        into: Stopped\n        instance: job_id\n        sets: {label: input.label}\n        emits: [jobs.job.Registered]",
    );
    let wrong = run_against(&synthesis.suite, &interpreted(&stopped));
    for id in [
        "kicked-starts/binding/flow",
        "kicked-starts/binding/delivery",
    ] {
        assert_eq!(status(&wrong, id), Status::Failed, "{id}");
    }
}

#[test]
fn binding_drop_observes_one_failed_attempt() {
    let synthesis = synth(FIXTURE);
    let steps = steps_of(&synthesis, DROP);
    assert_eq!(
        kinds(&steps),
        [
            "execute_command",
            "expect_outcome",
            "capture_instance",
            "query_view",
            "snapshot_subject",
            "configure_external_outcome",
            "execute_command",
            "expect_outcome",
            "expect_event",
            "expect_every_invocation",
            "expect_invocation",
            "query_view",
            "expect_subject_unchanged",
        ],
        "{steps:#?}"
    );
    let every = &steps[9];
    assert!(
        every.get("selecting").is_none(),
        "an empty selector selects every attempt: {every}"
    );
    assert_eq!(every["input"]["job_id"]["kind"], "observed", "{every}");
    let count = &steps[10];
    assert_eq!(count["count"], 1, "{count}");
    assert!(
        count.get("input").is_none(),
        "empty input counts every attempt: {count}"
    );
    assert_eq!(steps[5]["force"]["outcome"], "unavailable");
    assert_eq!(steps[4]["subject"]["job_id"]["kind"], "instance");

    // The trigger creates the row `created-starts` addresses, so nothing can be read before it.
    let named = refused_as(&synthesis, "created-starts/binding/on-failure");
    assert!(
        named
            .iter()
            .any(|text| text.contains("UnchangedUnobservable")),
        "{named:?}"
    );

    let honest = run_against(&synthesis.suite, &interpreted(FIXTURE));
    assert_eq!(
        status(&honest, DROP),
        Status::Passed,
        "{:?}",
        not_passed(&honest)
    );

    // Zero attempts: the dispatcher never delivers.
    let zero = run_against(
        &synthesis.suite,
        &interpreted(&rewrite(FIXTURE, KICKED_STARTS, "")),
    );
    assert_eq!(status(&zero, DROP), Status::Failed);
    // Two attempts: the dispatcher retries what it should drop, and the retry succeeds.
    let retry = rewrite(
        FIXTURE,
        KICKED_STARTS,
        &KICKED_STARTS.replace("on_failure: drop", "on_failure: retry"),
    );
    let two = run_against(&synthesis.suite, &interpreted(&retry));
    assert_eq!(status(&two, DROP), Status::Failed);
    assert!(failed_checks(&two, DROP)
        .iter()
        .any(|(code, about)| *code == CheckCode::Invocation && about.contains("exactly 1")));
    // One correct attempt and one carrying the wrong job.
    let malformed = run_against(
        &synthesis.suite,
        &Faulty::new(FIXTURE, Fault::MalformedRetry),
    );
    assert_eq!(status(&malformed, DROP), Status::Failed);
    assert!(failed_checks(&malformed, DROP)
        .iter()
        .any(|(code, about)| *code == CheckCode::Invocation && about.contains("every invocation")));
    // The forced refusal is ignored and the job starts.
    let succeeded = run_against(&synthesis.suite, &Faulty::new(FIXTURE, Fault::SwallowForce));
    assert_eq!(status(&succeeded, DROP), Status::Failed);
    assert_eq!(
        failed_checks(&succeeded, DROP)
            .iter()
            .map(|(code, _)| *code)
            .collect::<Vec<_>>(),
        [CheckCode::View],
        "only the unchanged row tells this one apart"
    );
}

#[test]
fn binding_drop_late_retry_fails() {
    let suite = synth(FIXTURE).suite;
    // The every-invocation step asks about fifty times; the count step's first answer is one.
    let late = run_against(&suite, &Faulty::new(FIXTURE, Fault::LateRetry(90)));
    assert_eq!(
        status(&late, DROP),
        Status::Failed,
        "{:?}",
        not_passed(&late)
    );
    assert!(failed_checks(&late, DROP)
        .iter()
        .any(|(code, about)| *code == CheckCode::Invocation && about.contains("exactly 1")));
}

/// The cumulative, non-consuming, correlation-scoped history the drop check relies on.
fn history_contract<T: ConformanceTarget>(target: &T) -> Result<(), String> {
    let own = CorrelationId::new("history").unwrap();
    let context = ScenarioContext::new(
        ScenarioId::parse("kicked-starts/binding/on-failure").unwrap(),
        own.clone(),
    );
    target.begin_scenario(&context).map_err(|e| e.to_string())?;
    let kick = |job: &str| {
        target
            .execute_command(SemanticCommandRequest {
                command: CommandRef::new("jobs.job.Kick".parse().unwrap()),
                actor: None,
                caller: None,
                input: [("job_id".to_owned(), Node::Text(job.into()))].into(),
                correlation: own.clone(),
            })
            .map_err(|e| e.to_string())
    };
    let (a, b) = (
        "00000000-0000-4000-8000-00000000000a",
        "00000000-0000-4000-8000-00000000000b",
    );
    kick(a)?;
    kick(a)?;
    kick(b)?;
    let read = |correlation: &CorrelationId, at: u64| {
        target
            .observe_invocations(InvocationObservationRequest {
                binding: BindingRef::new(
                    ess_domain::binding::BindingName::new("kicked-starts").unwrap(),
                ),
                command: CommandRef::new("jobs.job.Start".parse().unwrap()),
                correlation: correlation.clone(),
                deadline: Deadline::at(Timestamp::from_epoch_millis(at)),
            })
            .map(|seen| {
                let mut jobs: Vec<String> = seen
                    .iter()
                    .map(|invocation| match &invocation.input["job_id"] {
                        Node::Text(job) => job.clone(),
                        other => format!("{other:?}"),
                    })
                    .collect();
                jobs.sort();
                jobs
            })
            .map_err(|e| e.to_string())
    };
    let all = vec![a.to_owned(), a.to_owned(), b.to_owned()];
    let first = read(&own, 1_000)?;
    if first != all {
        return Err(format!("first read {first:?}"));
    }
    let again = read(&own, 1_000)?;
    if again != all {
        return Err(format!("repeated read {again:?}"));
    }
    kick(b)?;
    let later = read(&own, 9_000)?;
    let grown = vec![a.to_owned(), a.to_owned(), b.to_owned(), b.to_owned()];
    if later != grown {
        return Err(format!("later read under a new deadline {later:?}"));
    }
    let other = read(&CorrelationId::new("someone-else").unwrap(), 9_000)?;
    if !other.is_empty() {
        return Err(format!("another correlation saw {other:?}"));
    }
    target.end_scenario(&context).map_err(|e| e.to_string())
}

#[test]
fn binding_invocation_history_is_cumulative() {
    assert_eq!(history_contract(&interpreted(FIXTURE)), Ok(()));
    for fault in [
        Fault::Draining,
        Fault::DeadlineReset,
        Fault::WrongCorrelation,
        Fault::FilterExpected,
    ] {
        assert!(
            history_contract(&Faulty::new(FIXTURE, fault)).is_err(),
            "{fault:?} breaks the history contract"
        );
    }
    // The drop check itself fails against the draining and the resetting adapters.
    let suite = synth(FIXTURE).suite;
    for fault in [
        Fault::Draining,
        Fault::DeadlineReset,
        Fault::WrongCorrelation,
    ] {
        let report = run_against(&suite, &Faulty::new(FIXTURE, fault));
        if fault == Fault::WrongCorrelation {
            // One scenario at a time: the interpreter holds only its own correlation's attempts,
            // so ignoring the requested correlation changes nothing a scenario can see.
            continue;
        }
        assert_eq!(status(&report, DROP), Status::Failed, "{fault:?}");
    }
}

#[test]
fn binding_generated_destination_identity_is_not_guessed() {
    // `Kicked` carries an identity the implementation mints after the trigger: no job can be
    // arranged for it beforehand.
    let generated = rewrite(
        FIXTURE,
        "payload: {jobs.job.Kicked: {job_id: input.job_id}}",
        "payload: {jobs.job.Kicked: {job_id: {generated: true}}}",
    );
    let synthesis = synth(&generated);
    for aspect in ["flow", "delivery", "on-failure"] {
        let id = format!("kicked-starts/binding/{aspect}");
        let named = refused_as(&synthesis, &id);
        assert!(
            named
                .iter()
                .any(|text| text.contains("DestinationIdentityUnavailable")
                    && text.contains("job_id")),
            "{id}: {named:?}"
        );
    }
    steps_of(&synthesis, "kicked-starts/binding/mapping");

    // Input-derived: the fixture itself.
    let synthesis = synth(FIXTURE);
    steps_of(&synthesis, "kicked-starts/binding/flow");

    // Earlier-captured: the trigger acts on a job arranged and captured before it.
    let captured = rewrite(
        FIXTURE,
        "      - name: kicked\n        emits: [jobs.job.Kicked]",
        "      - name: kicked\n        updates: jobs.job.Job\n        instance: job_id\n        sets: {label: kicked}\n        emits: [jobs.job.Kicked]",
    );
    let synthesis = synth(&captured);
    let flow = steps_of(&synthesis, "kicked-starts/binding/flow");
    let sent: Vec<&str> = flow
        .iter()
        .filter(|step| step["step"] == "execute_command")
        .map(|step| step["command"].as_str().unwrap())
        .collect();
    assert_eq!(sent, ["jobs.job.Register", "jobs.job.Kick"], "{flow:#?}");
    let honest = run_against(&synthesis.suite, &interpreted(&captured));
    assert_eq!(not_passed(&honest), BTreeMap::new());
}

const STARTED_STOPS: &str = "  - id: started-stops
    when: {event: jobs.job.Started}
    invoke: {command: jobs.job.Stop}
    mapping: {job_id: event.job_id}
    delivery: at_least_once
    on_failure: drop
";

#[test]
fn binding_chain_arrangement_settles() {
    let chained = format!("{FIXTURE}{STARTED_STOPS}");
    let synthesis = synth(&chained);
    assert!(
        eventual_states(&synthesis.suite, "Stopped")
            .contains(&"jobs.job.Create/outcome/created".to_owned()),
        "{:#?}",
        steps_of(&synthesis, "jobs.job.Create/outcome/created")
    );
    // Nothing rests in `Started`: every way there sets `started-stops` off.
    let named = refused_as(&synthesis, "jobs.job.Stop/outcome/stopped");
    assert!(
        named.iter().any(|text| text.contains("started-stops")),
        "{named:?}\n{}",
        refusals(&synthesis).join("\n")
    );
    let honest = run_against(&synthesis.suite, &interpreted(&chained));
    assert_eq!(not_passed(&honest), BTreeMap::new());

    // A chain that never settles is named, not invented.
    let cyclic = rewrite(
        &format!(
            "{chained}  - id: stopped-restarts
    when: {{event: jobs.job.Stopped}}
    invoke: {{command: jobs.job.Restart}}
    mapping: {{job_id: event.job_id}}
    delivery: at_least_once
    on_failure: drop
"
        ),
        "      terminal: [Stopped]\n      transitions:\n",
        "      terminal: []\n      transitions:\n        - {name: restart, from: [Stopped], to: Started}\n",
    );
    let cyclic = rewrite(
        &cyclic,
        "  - name: jobs.job.Stop\n",
        "  - name: jobs.job.Restart
    input:
      - {name: job_id, type: jobs.job.JobId}
    outcomes:
      - name: restarted
        moves: jobs.job.Job.restart
        instance: job_id
        emits: [jobs.job.Started]
        payload: {jobs.job.Started: {job_id: input.job_id}}
      - {name: wrong-state, wrong_state: true, error: jobs.job.JobStateConflict}
  - name: jobs.job.Stop
",
    );
    let synthesis = synth(&cyclic);
    let unsettled: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|text| text.contains("EffectUnsettled") || text.contains("BindingUnsettled"))
        .collect();
    assert!(!unsettled.is_empty(), "{}", refusals(&synthesis).join("\n"));
    assert!(
        eventual_states(&synthesis.suite, "Stopped").is_empty()
            && eventual_states(&synthesis.suite, "Started").is_empty(),
        "no settled state is invented for a cycle"
    );
}

/// Every `.yaml` under `models/<name>`, compiled with its original labels.
fn repository_model(name: &str) -> ess_compiler::EssIr {
    use ess_compiler::{resolve::compile, source::SourceMap};
    use ess_domain::{spec::RawSpecFile, system::Source, Specification};
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../models")
        .join(name);
    let mut pending = vec![base.clone()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "yaml") {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in files {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let text = std::fs::read_to_string(path).unwrap();
        let raw = RawSpecFile::parse(&text).unwrap();
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}

#[test]
fn toolchain_check_on_tag_flow_is_no_longer_refused_for_its_wrong_state_branch() {
    // `models/toolchain/README.md` recorded `check-on-tag/binding/flow` and `.../delivery` as
    // ESS-SYNTH-010 only because `RunReleaseChecks` declares a `wrong_state` branch.
    let synthesis = synthesize(&repository_model("toolchain"));
    for id in ["check-on-tag/binding/flow", "check-on-tag/binding/delivery"] {
        assert!(
            refused_as(&synthesis, id).is_empty(),
            "{id}: {:?}",
            refused_as(&synthesis, id)
        );
        let steps = steps_of(&synthesis, id);
        assert!(
            steps.iter().any(|step| step["step"] == "eventually_event"),
            "{id}: {steps:#?}"
        );
    }
    // The two declarations stay what they were.
    assert!(
        refused_as(&synthesis, "record-on-completion/binding/delivery")
            .iter()
            .any(|text| text.contains("DeliverySingleAttempt"))
    );
    let dropped = refused_as(&synthesis, "record-on-completion/binding/on-failure");
    assert!(!dropped.is_empty(), "{}", refusals(&synthesis).join("\n"));
}

// ---- the trigger's whole fan-out on the row (adversary pass 1, F1 and F2) -----------------------

#[test]
fn binding_drop_beside_a_sibling_reads_where_the_sibling_leaves_the_row() {
    // `Kicked` also relabels the job: the row is not unchanged, it is relabelled and still `New`.
    let text = relabelled_on("Kicked");
    let synthesis = synth(&text);
    let steps = steps_of(&synthesis, DROP);
    let kinds = kinds(&steps);
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| kind.contains("snapshot") || kind.contains("unchanged"))
            .count(),
        0,
        "{steps:#?}"
    );
    let last = steps.last().unwrap();
    assert_eq!(last["step"], "eventually_view", "{last}");
    assert_eq!(
        last["expectation"]["fields"]["state"]["value"], "New",
        "{last}"
    );
    assert_eq!(
        last["expectation"]["fields"]["label"]["value"], "relabelled",
        "{last}"
    );
    let honest = run_against(&synthesis.suite, &interpreted(&text));
    assert_eq!(
        status(&honest, DROP),
        Status::Passed,
        "{:?}",
        not_passed(&honest)
    );
    // The forced refusal ignored starts the job, and the settled read sees it.
    let started = run_against(&synthesis.suite, &Faulty::new(&text, Fault::SwallowForce));
    assert_eq!(status(&started, DROP), Status::Failed);
    let retried = run_against(
        &synthesis.suite,
        &interpreted(&rewrite(
            &text,
            KICKED_STARTS,
            &KICKED_STARTS.replace("on_failure: drop", "on_failure: retry"),
        )),
    );
    assert_eq!(status(&retried, DROP), Status::Failed);

    // A sibling that invokes the forced command itself could take the forced refusal: refused.
    let twice = format!(
        "{FIXTURE}{}",
        KICKED_STARTS.replace("kicked-starts", "kicked-starts-again")
    );
    let named = refused_as(&synth(&twice), DROP);
    assert!(
        named
            .iter()
            .any(|text| text.contains("UnchangedUnobservable")),
        "{named:?}"
    );
}

#[test]
fn binding_flow_beside_a_sibling_asserts_no_state_the_fan_out_does_not_settle() {
    // `Created` sets off `created-starts` and a relabel on one row at once: no rest is named, so
    // neither flow asserts a state, and each says why.
    let text = relabelled_on("Created");
    let synthesis = synth(&text);
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/delivery",
        "relabels-on-created/binding/flow",
        "relabels-on-created/binding/delivery",
    ] {
        let steps = steps_of(&synthesis, id);
        assert_eq!(
            steps
                .iter()
                .filter(|step| step["step"] == "eventually_view")
                .count(),
            0,
            "{id}: {steps:#?}"
        );
        let named = refused_as(&synthesis, id);
        assert!(
            named.iter().any(|text| text.contains("EffectUnsettled")),
            "{id}: {named:?}"
        );
    }
    let honest = run_against(&synthesis.suite, &interpreted(&text));
    assert_eq!(not_passed(&honest), BTreeMap::new());
    // The rest is still observed where the fan-out is one binding.
    let plain = synth(FIXTURE);
    assert!(eventual_states(&plain.suite, "Started")
        .contains(&"created-starts/binding/flow".to_owned()));
}
