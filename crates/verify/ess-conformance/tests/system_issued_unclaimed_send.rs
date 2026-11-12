//! A send of a system-issued command that claims no branch is not awaited (workbench report item
//! 38, `docs/design/system-issued-commands.md`, *Which branches a scenario can claim*).
//!
//! `TakeDefault` of the committed example 28 is issued by the system actor `gap.demo.Clock`.
//! Its `no-default` scenario (`ObserveSubjectFact`) sends it before any row carries the identity
//! and checks, after it, that the created item is untouched. The send claims no branch, and the
//! rewrite made it an accepting `await_outcome`, which the interpreted target answers `NotIssued`
//! for an identity no record carries: the scenario ended at the runner's deadline. There is no
//! branch for an await to witness, so the send becomes an `expect_not_issued` and the check after
//! it stays.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SYSTEM: &str = include_str!(
    "../../../../.engineering/repro/workbench-2026-11-09/examples/28/spec/system.yaml"
);
const COMPONENTS: &str = include_str!(
    "../../../../.engineering/repro/workbench-2026-11-09/examples/28/spec/components.yaml"
);
const DEMO: &str =
    include_str!("../../../../.engineering/repro/workbench-2026-11-09/examples/28/spec/demo.yaml");

const NO_DEFAULT: &str = "gap.demo.TakeDefault/outcome/no-default";
const TAKE_DEFAULT: &str = "gap.demo.TakeDefault";

fn ir() -> EssIr {
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("demo.yaml", DEMO),
    ] {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        texts.insert(label.to_owned(), text.to_owned());
        parsed.push((Source::new(label), raw));
    }
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

#[test]
fn an_unclaimed_send_of_the_system_issued_step_is_not_awaited() {
    let result = synthesize(&ir());
    let scenario = scenario(&result, NO_DEFAULT);
    let awaits = scenario
        .steps
        .iter()
        .filter(|step| matches!(step, ScenarioStep::AwaitOutcome { .. }))
        .count();
    assert_eq!(awaits, 0, "{scenario:#?}");
    let on_item = scenario
        .steps
        .iter()
        .filter(|step| match step {
            ScenarioStep::ExpectNotIssued {
                command,
                instance: ScenarioValue::Instance { instance },
                ..
            } => command.to_string() == TAKE_DEFAULT && instance.as_str() == "item",
            _ => false,
        })
        .count();
    assert_eq!(on_item, 1, "{scenario:#?}");
}

#[test]
fn the_unclaimed_send_scenario_passes_on_the_interpreted_target() {
    let result = synthesize(&ir());
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report();
    let ran = report
        .scenarios
        .iter()
        .find(|ran| ran.scenario.to_string() == NO_DEFAULT)
        .unwrap_or_else(|| panic!("{NO_DEFAULT} did not run"));
    assert_eq!(ran.status, Status::Passed, "{ran:#?}");
}

// ---- adversary cases (uncommitted) -----------------------------------------------------------

fn adversary_ir_of(files: &[(&str, &str)]) -> EssIr {
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        texts.insert((*label).to_owned(), (*text).to_owned());
        parsed.push((Source::new(*label), raw));
    }
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Runs only the scenarios `keep` selects, on `target`, and returns `(id, status, result)`.
fn adversary_run<T: ess_conformance::target::ConformanceTarget>(
    result: &Synthesis,
    keep: impl Fn(&str) -> bool,
    target: &T,
) -> Vec<(String, Status, String)> {
    let mut suite = result.suite.clone();
    suite.scenarios.retain(|id, _| keep(&id.to_string()));
    assert!(!suite.scenarios.is_empty(), "no scenario selected");
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|e| panic!("{e}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|ran| (ran.scenario.to_string(), ran.status, format!("{ran:#?}")))
        .collect()
}

/// An entry the caller enqueues and finishes; the system expires a waiting one. `Expire` declares
/// no `wrong_state:` branch, so its wrong-state scenario sends it with no `expect_outcome` after.
const ADV_NO_WRONG_STATE: &str = "format: ess/27
system: queueing
version: v1
domain: queueing.entry
types:
  - {name: queueing.entry.EntryId, kind: newtype, of: Uuid}
entities:
  - name: queueing.entry.Entry
    identity: {name: entry_id, type: queueing.entry.EntryId}
    fields: []
    lifecycle:
      initial: Waiting
      states: [Waiting, Done, Expired]
      terminal: [Done, Expired]
      transitions:
        - {name: finish, from: [Waiting], to: Done}
        - {name: expire, from: [Waiting], to: Expired}
errors:
  - {name: queueing.entry.NoEntry, summary: No entry carries the identity., fields: []}
events:
  - name: queueing.entry.Enqueued
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
  - name: queueing.entry.Finished
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
  - name: queueing.entry.Expired
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.Enqueue, queueing.entry.Finish, queueing.entry.Entries]
  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Expire]
views:
  - name: queueing.entry.Entries
    source: queueing.entry.Entry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
      - {name: state, type: queueing.entry.Entry.State}
commands:
  - name: queueing.entry.Enqueue
    input: []
    outcomes:
      - name: enqueued
        creates: queueing.entry.Entry
        instance: entry_id
        emits: [queueing.entry.Enqueued]
        payload:
          queueing.entry.Enqueued: {entry_id: {generated: true}}
  - name: queueing.entry.Finish
    input:
      - {name: entry_id, type: queueing.entry.EntryId}
    outcomes:
      - name: finished
        moves: queueing.entry.Entry.finish
        instance: entry_id
        emits: [queueing.entry.Finished]
        payload:
          queueing.entry.Finished: {entry_id: input.entry_id}
  - name: queueing.entry.Expire
    await_within: PT1S
    input:
      - {name: entry_id, type: queueing.entry.EntryId}
    outcomes:
      - name: expired
        moves: queueing.entry.Entry.expire
        instance: entry_id
        emits: [queueing.entry.Expired]
        payload:
          queueing.entry.Expired: {entry_id: input.entry_id}
      - {name: unknown-entry, unknown_instance: true, error: queueing.entry.NoEntry}
";

const ADV_EXPIRE: &str = "queueing.entry.Expire";

fn adversary_wrong_state_ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.contains("/state/") && id.ends_with(&format!("/refuses/{ADV_EXPIRE}")))
        .collect()
}

/// The wrong-state scenario of a system-issued command that declares no `wrong_state:` branch is
/// about the system not taking the step from that state. Its send claims no branch, and the unit
/// drops it: what is left holds no step of `Expire` at all. Either it is claimed as
/// `expect_not_issued` (the observation the scenario exists for) or it is withdrawn; a scenario
/// named `.../refuses/Expire` that never observes `Expire` is neither.
#[test]
fn adversary_an_undeclared_wrong_state_scenario_still_observes_the_system_step() {
    let ir = adversary_ir_of(&[("queueing.yaml", ADV_NO_WRONG_STATE)]);
    let result = synthesize(&ir);
    let ids = adversary_wrong_state_ids(&result);
    assert!(
        !ids.is_empty(),
        "no wrong-state scenario of {ADV_EXPIRE}: {:#?}",
        result.suite.scenarios.keys().collect::<Vec<_>>()
    );
    let silent: Vec<String> = ids
        .iter()
        .filter_map(|id| {
            let scenario = scenario(&result, id);
            let observed = scenario.steps.iter().any(|step| match step {
                ScenarioStep::ExpectNotIssued { command, .. }
                | ScenarioStep::AwaitOutcome { command, .. } => command.to_string() == ADV_EXPIRE,
                _ => false,
            });
            (!observed).then(|| format!("{id}: {scenario:#?}"))
        })
        .collect();
    assert!(
        silent.is_empty(),
        "a wrong-state scenario of the system-issued step observes no step of it:\n{}",
        silent.join("\n")
    );
}

/// The mutant: a target whose system takes `Expire` from a state it has no transition from. The
/// wrong-state scenario exists to catch exactly that, and must not pass on it.
#[test]
fn adversary_a_target_taking_the_step_from_a_wrong_state_fails_its_scenario() {
    use ess_conformance::faulty::{AwaitFault, AwaitFaulty};
    let ir = adversary_ir_of(&[("queueing.yaml", ADV_NO_WRONG_STATE)]);
    let result = synthesize(&ir);
    let ids = adversary_wrong_state_ids(&result);
    let target = AwaitFaulty::new(Interpreted::for_model(ir), AwaitFault::TakesRefusedStep);
    let ran = adversary_run(&result, |id| ids.iter().any(|kept| kept == id), &target);
    let passed: Vec<_> = ran
        .iter()
        .filter(|(_, status, _)| *status == Status::Passed)
        .map(|(id, _, detail)| format!("{id}: {detail}"))
        .collect();
    assert!(
        passed.is_empty(),
        "a target that takes the refused system step passes:\n{}",
        passed.join("\n")
    );
}

/// The kept `expect_not_issued` of `no-default` fails a target that takes the step anyway.
#[test]
fn adversary_a_target_taking_the_refused_default_step_fails_no_default() {
    use ess_conformance::faulty::{AwaitFault, AwaitFaulty};
    let result = synthesize(&ir());
    let target = AwaitFaulty::new(Interpreted::for_model(ir()), AwaitFault::TakesRefusedStep);
    let ran = adversary_run(&result, |id| id == NO_DEFAULT, &target);
    assert_eq!(ran.len(), 1);
    assert_eq!(ran[0].1, Status::Failed, "{}", ran[0].2);
}

/// The mutant: a target whose `no-default` guard is inverted (`defined(dflt)`), so it takes
/// `TakeDefault` on an item without a default. The `no-default` scenario claims that branch; its
/// kept `expect_not_issued` must fail such a target. The arranged item's `parent_id` names no
/// parent, so `no-parent` refuses the step for both targets and the claim of `no-default` is
/// never witnessed.
#[test]
#[ignore = "story:a-subject-fact-scenario-beside-a-stored-related-guard-arranges-the-related-row (backlog)"]
fn adversary_a_target_wrong_about_the_no_default_guard_fails_no_default() {
    let mutant = DEMO.replace("predicate: not defined(dflt)", "predicate: defined(dflt)");
    assert_ne!(mutant, DEMO, "the no-default guard was not found");
    let mutant_ir = adversary_ir_of(&[
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("demo.yaml", &mutant),
    ]);
    let result = synthesize(&ir());
    let ran = adversary_run(
        &result,
        |id| id == NO_DEFAULT,
        &Interpreted::for_model(mutant_ir),
    );
    assert_eq!(ran.len(), 1);
    assert_ne!(ran[0].1, Status::Passed, "{}", ran[0].2);
}

/// Example 28 with `TakeDefault` driven by a caller: its refusals through the stored reference,
/// whose payloads read `subject: parent_id`, pass on the interpreted target through the changed
/// `missing_row`.
#[test]
fn adversary_driven_no_parent_refusal_payload_agrees_with_the_interpreted_target() {
    let demo = DEMO.replace("\n  system: true", "");
    assert_ne!(demo, DEMO, "the system actor line was not found");
    let ir = || {
        adversary_ir_of(&[
            ("system.yaml", SYSTEM),
            ("components.yaml", COMPONENTS),
            ("demo.yaml", &demo),
        ])
    };
    let result = synthesize(&ir());
    let no_parent = format!("{TAKE_DEFAULT}/outcome/no-parent");
    let steps = &scenario(&result, &no_parent).steps;
    // What the suite expects `NoParent.parent_id` (`subject: parent_id`) to be, and what the
    // arrangement stored as the addressed item's `parent_id`.
    let expected = steps.iter().find_map(|step| match step {
        ScenarioStep::ExpectError { fields, .. } => fields.get("parent_id").cloned(),
        _ => None,
    });
    let stored = steps.iter().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "gap.demo.CreateItem" =>
        {
            input.get("parent_id").cloned()
        }
        _ => None,
    });
    let ran = adversary_run(&result, |id| id == no_parent, &Interpreted::for_model(ir()));
    assert_eq!(
        ran[0].1,
        Status::Passed,
        "expected {expected:?}, the item stored {stored:?}\n{}\n{steps:#?}",
        ran[0].2
    );
}

/// Two other system-issued fixtures, run whole on the interpreted target: the unit claims no
/// other repository model's interpreted run changes.
#[test]
fn adversary_two_system_issued_fixtures_pass_on_the_interpreted_target() {
    for (label, text) in [
        (
            "system-guard.yaml",
            include_str!("fixtures/system-guard.yaml"),
        ),
        (
            "system-issued-queue.yaml",
            include_str!("fixtures/system-issued-queue.yaml"),
        ),
    ] {
        let ir = || adversary_ir_of(&[(label, text)]);
        let result = synthesize(&ir());
        let ran = adversary_run(&result, |_| true, &Interpreted::for_model(ir()));
        let failing: Vec<_> = ran
            .iter()
            .filter(|(_, status, _)| *status != Status::Passed)
            .map(|(id, status, detail)| format!("{id} {status:?}: {detail}"))
            .collect();
        assert!(failing.is_empty(), "{label}:\n{}", failing.join("\n"));
        assert_eq!(ran.len(), result.suite.scenarios.len(), "{label}");
    }
}
