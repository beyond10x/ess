//! Adversary pass 1 against story:when-subject-witness-and-diagnostics (beyond10x/ess#172,
//! beyond10x/ess#173): variants of the two repros the unit's own suite does not synthesize.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::Synthesis, ConformanceScenario, ConformanceSuite, ScenarioStep};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const REFUSED_RUNNING: &str = "demo.jobs.Job/state/Running/refuses/demo.jobs.AdvanceJob";
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

fn assert_no_gap(result: &Synthesis) {
    let all = refusals(result);
    let gaps: Vec<_> = all
        .iter()
        .filter(|line| {
            ["ESS-SYNTH-001 ", "ESS-SYNTH-004 ", "ESS-SYNTH-008 "]
                .iter()
                .any(|banned| line.starts_with(banned))
        })
        .collect();
    assert!(
        gaps.is_empty(),
        "{gaps:#?}\nall refusals: {all:#?}\nscenarios: {:#?}",
        ids(&result.suite)
    );
}

fn assert_scenarios(result: &Synthesis, wanted: &[&str]) {
    let missing: Vec<_> = wanted
        .iter()
        .filter(|id| scenario(&result.suite, id).is_none())
        .collect();
    assert!(
        missing.is_empty(),
        "missing {missing:#?}\nrefusals: {:#?}\nscenarios: {:#?}",
        refusals(result),
        ids(&result.suite)
    );
}

/// The number the scenario sends as `force` in its last `AdvanceJob` step, the command under test.
fn sent_force(scenario: &ConformanceScenario) -> Option<f64> {
    fn first_int(value: &serde_json::Value) -> Option<f64> {
        match value {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse().ok(),
            serde_json::Value::Array(items) => items.iter().find_map(first_int),
            serde_json::Value::Object(map) => map.values().find_map(first_int),
            _ => None,
        }
    }
    scenario.steps.iter().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { command, .. }
            if command.to_string() == "demo.jobs.AdvanceJob" =>
        {
            let json = serde_json::to_value(step).ok()?;
            let force = json.pointer("/input/force").or_else(|| {
                json.as_object()?
                    .values()
                    .find_map(|inner| inner.pointer("/input/force"))
            })?;
            first_int(force)
        }
        _ => None,
    })
}

const HEAD: &str = "format: ess/16
system: demo
version: v1
domain: demo.jobs
summary: Jobs that leave the queue by a stored flag.
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: fast, type: Boolean}
    lifecycle:
      initial: Queued
      states: [Queued, Running, Direct]
      terminal: [Running, Direct]
      transitions:
        - {name: start, from: [Queued], to: Running}
        - {name: go-direct, from: [Queued], to: Direct}
actors:
  - {name: demo.jobs.Clerk, may: [demo.jobs.SubmitJob, demo.jobs.AdvanceJob]}
errors:
  - name: demo.jobs.Jammed
    summary: The force was too high.
  - name: demo.jobs.JobStateConflict
    summary: The job has left the queue.
  - name: demo.jobs.JobNotFound
    summary: No job carries that identity.
    fields:
      - {name: job_id, type: demo.jobs.JobId}
events:
  - name: demo.jobs.JobSubmitted
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: fast, type: Boolean}
  - name: demo.jobs.JobMoved
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: state, type: demo.jobs.Job.State}
views:
  - name: demo.jobs.JobDetails
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: fast, type: Boolean}
      - {name: state, type: demo.jobs.Job.State}
commands:
  - name: demo.jobs.SubmitJob
    input:
      - {name: fast, type: Boolean}
    outcomes:
      - name: submitted
        creates: demo.jobs.Job
        instance: job_id
        sets: {fast: input.fast}
        emits: [demo.jobs.JobSubmitted]
        payload:
          demo.jobs.JobSubmitted: {job_id: {generated: true}, fast: input.fast}
";

const STARTED: &str = "      - name: started
        when_subject:
          predicate: fast == true
        moves: demo.jobs.Job.start
        instance: job_id
        emits: [demo.jobs.JobMoved]
        payload:
          demo.jobs.JobMoved: {job_id: input.job_id, state: Running}
";

const DIRECT_FALLBACK: &str = "      - name: direct
        moves: demo.jobs.Job.go-direct
        instance: job_id
        emits: [demo.jobs.JobMoved]
        payload:
          demo.jobs.JobMoved: {job_id: input.job_id, state: Direct}
";

const WRONG_STATE: &str = "      - name: left-queue
        wrong_state: true
        error: demo.jobs.JobStateConflict
";

fn advance(input: &str, outcomes: &[&str]) -> String {
    format!(
        "{HEAD}  - name: demo.jobs.AdvanceJob
    input:
      - {{name: job_id, type: demo.jobs.JobId}}
{input}    outcomes:
{}",
        outcomes.concat()
    )
}

/// An input-guarded refusal beside the subject-fact branch and the guard-less fallback. The
/// wrong-state scenario must send the input that reaches a *moving* branch, as `plain_guards`' own
/// comment says `refused_here` does: in the arranging step the same scenario sends `force: 5`, the
/// input `selects` takes `started` with, and the command under test must not send one that selects
/// `jammed` instead. Without a declared `wrong_state` branch the scenario asserts only that nothing
/// is published, which an implementation that answers `jammed` and ignores the state then passes;
/// with one, the precedence of `wrong_state` over an input refusal is not settled by the docs
/// (`mutation-audit-and-model-runner.md` puts it first, `typed-literals-and-unknown-instances.md`
/// decides input guards first).
#[test]
fn the_wrong_state_input_does_not_select_an_input_guarded_refusal_sibling() {
    let jammed = "      - name: jammed
        when: force < 5
        error: demo.jobs.Jammed
";
    for declared in [&[WRONG_STATE][..], &[]] {
        let outcomes = [&[STARTED, jammed, DIRECT_FALLBACK][..], declared].concat();
        let text = advance("      - {name: force, type: Integer}\n", &outcomes);
        let result = synthesis(&text);
        assert_scenarios(&result, &[REFUSED_RUNNING, REFUSED_DIRECT]);
        for id in [REFUSED_RUNNING, REFUSED_DIRECT] {
            let refused = scenario(&result.suite, id).unwrap();
            let force = sent_force(refused);
            assert!(
                force.is_some_and(|force| force >= 5.0),
                "{id} (wrong_state declared: {}) sends force {force:?}, which selects the \
                 input-guarded refusal `jammed` rather than a moving branch: {}",
                !declared.is_empty(),
                serde_json::to_string(&refused.steps).unwrap()
            );
        }
    }
}

/// A guard-less fallback and a declared unknown-instance answer: the unknown-identity scenario is
/// written beside the wrong-state ones.
#[test]
fn an_unknown_identity_is_witnessed_beside_the_wrong_state_refusals() {
    let unknown = "      - name: not-found
        unknown_instance: true
        error: demo.jobs.JobNotFound
";
    let text = advance("", &[STARTED, DIRECT_FALLBACK, WRONG_STATE, unknown]);
    let result = synthesis(&text);
    assert_no_gap(&result);
    assert_scenarios(
        &result,
        &[
            REFUSED_RUNNING,
            REFUSED_DIRECT,
            "demo.jobs.AdvanceJob/outcome/not-found",
        ],
    );
}

/// Several guarded branches read through an eventual view only: every branch, transition and
/// refusal is witnessed.
#[test]
fn several_guarded_branches_through_an_eventual_view_are_all_witnessed() {
    let text = HEAD
        .replace(
            "      - {name: fast, type: Boolean}\n    lifecycle:",
            "      - {name: fast, type: Boolean}\n      - {name: urgent, type: Boolean}\n    lifecycle:",
        )
        .replace(
            "      states: [Queued, Running, Direct]\n      terminal: [Running, Direct]",
            "      states: [Queued, Running, Rushed, Direct]\n      terminal: [Running, Rushed, Direct]",
        )
        .replace(
            "        - {name: go-direct, from: [Queued], to: Direct}",
            "        - {name: go-direct, from: [Queued], to: Direct}\n        - {name: rush, from: [Queued], to: Rushed}",
        )
        .replace("consistency: read_your_writes", "consistency: eventual")
        .replace(
            "      - {name: fast, type: Boolean}\n      - {name: state, type: demo.jobs.Job.State}",
            "      - {name: fast, type: Boolean}\n      - {name: urgent, type: Boolean}\n      - {name: state, type: demo.jobs.Job.State}",
        )
        .replace(
            "      - {name: fast, type: Boolean}\n    outcomes:",
            "      - {name: fast, type: Boolean}\n      - {name: urgent, type: Boolean}\n    outcomes:",
        )
        .replace(
            "        sets: {fast: input.fast}",
            "        sets: {fast: input.fast, urgent: input.urgent}",
        );
    assert!(text.contains("rush, from"), "the head was not rewritten");
    let rushed = "      - name: rushed
        when_subject:
          predicate:
            all:
              - urgent == true
              - fast == false
        moves: demo.jobs.Job.rush
        instance: job_id
        emits: [demo.jobs.JobMoved]
        payload:
          demo.jobs.JobMoved: {job_id: input.job_id, state: Rushed}
";
    let text = format!(
        "{text}  - name: demo.jobs.AdvanceJob
    input:
      - {{name: job_id, type: demo.jobs.JobId}}
    outcomes:
{STARTED}{rushed}{DIRECT_FALLBACK}{WRONG_STATE}"
    );
    let result = synthesis(&text);
    assert_no_gap(&result);
    assert_scenarios(
        &result,
        &[
            "demo.jobs.AdvanceJob/outcome/started",
            "demo.jobs.AdvanceJob/outcome/rushed",
            "demo.jobs.AdvanceJob/outcome/direct",
            "demo.jobs.Job/transition/rush/by/demo.jobs.AdvanceJob/rushed",
            REFUSED_RUNNING,
            REFUSED_DIRECT,
            "demo.jobs.Job/state/Rushed/refuses/demo.jobs.AdvanceJob",
        ],
    );
}

/// Only a filtered eventual view: the refusal names the change, and says the filter is the reason.
#[test]
fn a_filtered_eventual_view_is_refused_with_a_hint_naming_the_filter() {
    let text = advance("", &[STARTED, DIRECT_FALLBACK, WRONG_STATE]).replace(
        "consistency: read_your_writes",
        "consistency: eventual\n    filter: fast == true",
    );
    let result = synthesis(&text);
    let about: Vec<_> = refusals(&result)
        .into_iter()
        .filter(|line| line.starts_with("ESS-SYNTH-001 demo.jobs.AdvanceJob/outcome/started"))
        .collect();
    assert_eq!(about.len(), 1, "{:#?}", refusals(&result));
    assert!(about[0].contains("no filter"), "{}", about[0]);
    assert!(!about[0].contains("drifted apart"), "{}", about[0]);
}

/// #173 says the refusal fires "with and without a `wrong_state` outcome"; the unit's suite
/// synthesizes only the model without one.
#[test]
fn the_repro_with_a_declared_wrong_state_branch_gets_its_refusal_scenarios() {
    let text = advance("", &[STARTED, DIRECT_FALLBACK, WRONG_STATE]);
    let result = synthesis(&text);
    assert_no_gap(&result);
    assert_scenarios(&result, &[REFUSED_RUNNING, REFUSED_DIRECT]);
    for id in [REFUSED_RUNNING, REFUSED_DIRECT] {
        let steps = serde_json::to_string(&scenario(&result.suite, id).unwrap().steps).unwrap();
        assert!(
            steps.contains("demo.jobs.JobStateConflict"),
            "{id}: {steps}"
        );
    }
}

/// The same with only an eventual view: both repros at once.
#[test]
fn the_eventual_repro_with_a_declared_wrong_state_branch_gets_its_refusal_scenarios() {
    let text = advance("", &[STARTED, DIRECT_FALLBACK, WRONG_STATE])
        .replace("consistency: read_your_writes", "consistency: eventual");
    let result = synthesis(&text);
    assert_no_gap(&result);
    assert_scenarios(
        &result,
        &[
            "demo.jobs.AdvanceJob/outcome/started",
            "demo.jobs.AdvanceJob/outcome/direct",
            REFUSED_RUNNING,
            REFUSED_DIRECT,
        ],
    );
}

/// An immediate and an eventual view both qualify: the immediate one is observed, as `observer`
/// documents, and the arranged row is never awaited through an `eventually` block before the
/// command.
#[test]
fn an_immediate_view_is_preferred_over_an_eventual_one_that_also_qualifies() {
    let both = "  - name: demo.jobs.JobFeed
    source: demo.jobs.Job
    consistency: eventual
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: fast, type: Boolean}
      - {name: state, type: demo.jobs.Job.State}
commands:
";
    let text = advance("", &[STARTED, DIRECT_FALLBACK]).replacen("commands:\n", both, 1);
    let result = synthesis(&text);
    assert_no_gap(&result);
    for id in [
        "demo.jobs.AdvanceJob/outcome/started",
        "demo.jobs.AdvanceJob/outcome/direct",
    ] {
        let steps = &scenario(&result.suite, id)
            .unwrap_or_else(|| panic!("no {id}: {:#?}", refusals(&result)))
            .steps;
        assert!(
            !steps
                .iter()
                .take_while(
                    |step| !matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "demo.jobs.AdvanceJob")
                )
                .any(|step| matches!(step, ScenarioStep::EventuallyView { .. })),
            "{id} waits on the eventual view where an immediate one qualifies: {}",
            serde_json::to_string(steps).unwrap()
        );
    }
}
