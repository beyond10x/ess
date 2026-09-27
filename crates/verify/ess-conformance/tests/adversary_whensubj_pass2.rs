//! Adversary pass 2 against story:when-subject-witness-and-diagnostics (beyond10x/ess#172,
//! beyond10x/ess#173): the wrong-state input beside a refusal guarded by stored fields and input,
//! and the post-command observations the unit's own suite leaves unasserted.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::Synthesis, ConformanceScenario, ConformanceSuite, ScenarioStep};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const EVENTUAL: &str = include_str!("fixtures/when-subject-eventual.yaml");

const REFUSED_DIRECT: &str = "demo.jobs.Job/state/Direct/refuses/demo.jobs.AdvanceJob";

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
    ess_conformance::synthesize::synthesize(&ir)
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {} :: {} / {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause,
                refusal.cause.hint()
            )
        })
        .collect()
}

fn ids(suite: &ConformanceSuite) -> Vec<String> {
    suite.scenarios.keys().map(ToString::to_string).collect()
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// The steps after the last `AdvanceJob` the scenario sends.
fn after_advance(scenario: &ConformanceScenario) -> &[ScenarioStep] {
    let sent = scenario
        .steps
        .iter()
        .rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.jobs.AdvanceJob")
        })
        .expect("the command under test is sent");
    &scenario.steps[sent + 1..]
}

/// The mutant `leaves_changed` → `false`: every moving branch read through an eventual view would
/// fall to `observe_unchanged`, find no immediate view, and assert nothing about the row it left.
/// The unit's own suite checks only what is observed *before* the command. The generic view
/// assertion awaits the row once through every view of the entity; `around_row` awaits it a second
/// time, and that second block is the one the mutant removes. A move to `Running` or
/// `Direct` is awaited in an `eventually` block after it.
#[test]
fn an_eventual_moving_branch_awaits_the_state_it_arrives_at() {
    let result = synthesis(EVENTUAL);
    for (id, arrives) in [
        ("demo.jobs.AdvanceJob/outcome/started", "Running"),
        ("demo.jobs.AdvanceJob/outcome/direct", "Direct"),
    ] {
        let found = scenario(&result.suite, id)
            .unwrap_or_else(|| panic!("no {id}: {:#?}", refusals(&result)));
        let after = after_advance(found);
        let awaited = after
            .iter()
            .filter(|step| {
                matches!(step, ScenarioStep::EventuallyView { .. })
                    && serde_json::to_string(step).unwrap().contains(arrives)
            })
            .count();
        assert!(
            awaited >= 2,
            "{id} awaits the row in `{arrives}` {awaited} time(s) after the command, not in both \
             the generic view assertion and `around_row`: {}",
            serde_json::to_string(after).unwrap()
        );
    }
}

/// The mutant `leaves_changed` → `true`: an `updates:` branch writing the value the row already
/// held would be awaited through the eventual view after the command, which passes on a
/// projection that never saw the command. The unit documents that such a row "is read as a
/// refusal's row is", through an immediate view only; with none, nothing is awaited after it.
#[test]
fn an_update_writing_the_held_value_is_not_awaited_through_an_eventual_view() {
    let text = EVENTUAL
        .replace("format: ess/14", "format: ess/16")
        .replace(
            "      states: [Queued, Running, Direct]
      terminal: [Running, Direct]
      transitions:
        - {name: start, from: [Queued], to: Running}
",
            "      states: [Queued, Direct]
      terminal: [Direct]
      transitions:
",
        )
        .replace(
            "      - name: started
        when_subject:
          predicate: fast == true
        moves: demo.jobs.Job.start
        instance: job_id
        emits: [demo.jobs.JobMoved]
        payload:
          demo.jobs.JobMoved: {job_id: input.job_id, state: Running}
",
            "      - name: touched
        when_subject:
          predicate: fast == true
        updates: demo.jobs.Job
        instance: job_id
        sets: {fast: true}
        emits: [demo.jobs.JobMoved]
        payload:
          demo.jobs.JobMoved: {job_id: input.job_id, state: Queued}
",
        );
    assert!(
        text.contains("name: touched"),
        "the fixture was not rewritten"
    );
    let result = synthesis(&text);
    let id = "demo.jobs.AdvanceJob/outcome/touched";
    let found = scenario(&result.suite, id).unwrap_or_else(|| {
        panic!(
            "no {id}: {:#?}\n{:#?}",
            refusals(&result),
            ids(&result.suite)
        )
    });
    let after = after_advance(found);
    assert!(
        !after
            .iter()
            .any(|step| matches!(step, ScenarioStep::EventuallyView { .. })),
        "{id} awaits an unchanged row through an eventual view: {}",
        serde_json::to_string(after).unwrap()
    );
}

/// The number the scenario sends as `force` in its last `AdvanceJob` step.
fn sent_force(scenario: &ConformanceScenario) -> Option<f64> {
    let step = scenario.steps.iter().rev().find(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command, .. }
            if command.to_string() == "demo.jobs.AdvanceJob")
    })?;
    let json = serde_json::to_value(step).ok()?;
    number(find(&json)?)
}

fn number(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse().ok(),
        serde_json::Value::Array(items) => items.iter().find_map(number),
        serde_json::Value::Object(map) => map.values().find_map(number),
        _ => None,
    }
}

fn find(value: &serde_json::Value) -> Option<&serde_json::Value> {
    match value {
        serde_json::Value::Object(map) => map.get("force").or_else(|| map.values().find_map(find)),
        serde_json::Value::Array(items) => items.iter().find_map(find),
        _ => None,
    }
}

/// Pass 1's corrected case, with the refusal sibling guarded by the stored flag as well as by the
/// input: `jammed` answers a job that is not fast and pushed with `force < 5`. `refusal_input`
/// refutes the input guard of every sibling it finds through `when`, which answers `None` for a
/// `when_subject:` branch, so `jammed`'s `when: force < 5` is never refuted. The `Direct` row is
/// arranged through `direct`, with `fast == false`, so an input with `force < 5` satisfies both
/// halves of `jammed` on the very row the wrong-state scenario sends it to: the scenario then
/// depends on whether the implementation decides the state or `jammed` first — the dependence the
/// function's own documentation says it removes.
#[test]
fn the_wrong_state_input_refutes_the_input_half_of_a_stored_field_refusal_sibling() {
    let text = EVENTUAL
        .replace("format: ess/14", "format: ess/16")
        .replace("consistency: eventual", "consistency: read_your_writes")
        .replace(
            "      - {name: job_id, type: demo.jobs.JobId}
    outcomes:
      - name: started",
            "      - {name: job_id, type: demo.jobs.JobId}
      - {name: force, type: Integer}
    outcomes:
      - name: jammed
        when_subject:
          predicate: fast == false
        when: force < 5
        error: demo.jobs.Jammed
      - name: started",
        )
        .replace(
            "events:\n",
            "errors:
  - name: demo.jobs.Jammed
    summary: The force was too low for a slow job.
  - name: demo.jobs.JobStateConflict
    summary: The job has left the queue.
events:\n",
        )
        .replace(
            "          demo.jobs.JobMoved: {job_id: input.job_id, state: Direct}\n",
            "          demo.jobs.JobMoved: {job_id: input.job_id, state: Direct}
      - name: left-queue
        wrong_state: true
        error: demo.jobs.JobStateConflict\n",
        );
    assert!(
        text.contains("name: jammed") && text.contains("name: left-queue"),
        "the fixture was not rewritten"
    );
    let result = synthesis(&text);
    // The `Running` row was arranged through `started`, so `fast == true` refutes `jammed` there
    // whatever the input; the `Direct` row was arranged through `direct`, with `fast == false`.
    {
        let id = REFUSED_DIRECT;
        let found = scenario(&result.suite, id).unwrap_or_else(|| {
            panic!(
                "no {id}: {:#?}\n{:#?}",
                refusals(&result),
                ids(&result.suite)
            )
        });
        let force = sent_force(found);
        assert!(
            force.is_some_and(|force| force >= 5.0),
            "{id} sends force {force:?}, which the input half of the refusal sibling `jammed` \
             admits: {}",
            serde_json::to_string(&found.steps).unwrap()
        );
    }
}
