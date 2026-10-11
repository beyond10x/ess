//! Adversary pass 1 on `mutate --scenarios` (<https://github.com/beyond10x/ess/issues/515>):
//! `mutate::audit_with_authored`, attacked on what a kill, a survivor and an equivalent mutant
//! mean once authored scenarios run against the interpreter of each mutant.
//!
//! The models are inline rather than fixture files, so the repository-wide fixture tables do not
//! move.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_conformance::{ConformanceScenario, ScenarioId};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn documents(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

fn authored(ir: &EssIr, name: &str, text: &str) -> BTreeMap<ScenarioId, ConformanceScenario> {
    let authoring = ess_conformance::authored::compile(
        ir,
        &[ess_conformance::authored::Source::new(name, text)],
    );
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    authoring.scenarios
}

/// The audit as `ess verify conform mutate --target interpreted --scenarios` runs it: the
/// synthesized suites on the interpreter of the unchanged specification.
fn audit(
    label: &str,
    text: &str,
    classes: &[MutantClass],
    scenarios: &[(&str, &str)],
) -> MutationReport {
    let (documents, texts) = documents(label, text);
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let mut set = BTreeMap::new();
    for (name, scenario) in scenarios {
        set.extend(authored(&ir, name, scenario));
    }
    mutate::audit_with_authored(
        &documents,
        &texts,
        classes,
        || Interpreted::for_model(ir.clone()),
        None,
        &set,
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn entry<'a>(report: &'a MutationReport, id: &str) -> &'a mutate::MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("{id}: {}", report.render_text()))
}

// ---- a `now` guard ------------------------------------------------------------------------------

/// `Present` refuses a token whose `exp` is more than ten minutes before `now`. `Ping` is there so
/// the baseline executes something without a clock.
const CLOCKED: &str = r"format: ess/23
system: demo
version: v1
domain: demo.tok
types:
  - {name: demo.tok.TokenId, kind: newtype, of: String}
errors:
  - name: demo.tok.Expired
  - name: demo.tok.Revoked
commands:
  - name: demo.tok.Ping
    input:
      - {name: token_id, type: demo.tok.TokenId}
    outcomes:
      - name: ponged
        accepts: nothing
  - name: demo.tok.Present
    input:
      - {name: token_id, type: demo.tok.TokenId}
      - {name: exp, type: Timestamp}
    outcomes:
      - name: expired
        when: exp < now - 10m
        error: demo.tok.Expired
      - name: accepted
        accepts: nothing
";

/// A token that expired in 2000 is refused at every moment a target may run it, so `authored`
/// admits the outcome, and `ess verify conform run --target interpreted --scenarios` passes it
/// (`crates/edge/ess-cli/tests/adversary_j3_pass1.rs` shows that end to end).
const LONG_EXPIRED: &str = r"type: ess-scenario/1
domain: demo.tok
scenario: long-expired
summary: A token that expired long ago is refused as expired.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.tok.Present
    input: {token_id: t-1, exp: 2000-01-01T00:00:00Z}
    outcome: expired
    error: {name: demo.tok.Expired}
";

const CLOCKED_SWAP: &str = "error-swap/demo.tok.Present/expired";

/// Acceptance: "a mutant only an authored scenario pins is killed by it". The authored scenario
/// requires `Expired`; the interpreter of the `error-swap` mutant answers `Revoked`, as `run
/// --target interpreted` shows once it is handed the step clock. `mutate` builds the mutant's
/// interpreter with no command clock (`AuthoredSuite::on`, `mutate.rs`), so the `now` guard is
/// Unknown, the scenario is `unsupported` on the baseline and on every mutant, and the mutant is
/// not killed.
#[test]
fn an_authored_scenario_on_a_now_guard_kills_the_mutant_it_pins() {
    let report = audit(
        "clocked.yaml",
        CLOCKED,
        &[MutantClass::ErrorSwap],
        &[("long-expired.yaml", LONG_EXPIRED)],
    );
    let entry = entry(&report, CLOCKED_SWAP);
    assert_eq!(
        entry.verdict,
        Verdict::Killed,
        "{entry:#?}\n{}",
        report.render_text()
    );
    assert_eq!(
        entry.killers.as_deref(),
        Some(["demo.tok/authored/long-expired".to_owned()].as_slice()),
        "{entry:#?}"
    );
}

// ---- an authored scenario the baseline did not execute ------------------------------------------

/// `mutation-authored.yaml`'s limit, with a `now` guard ahead of it: every `RetryTask` decision
/// reads `now` first. The synthesized `at-limit` scenario is refused (`ESS-SYNTH-003`), so a
/// mutant on `at-limit` is unwitnessed without authored scenarios.
const STALE_LIMIT: &str = r"format: ess/23
system: work
version: v1
domain: work.tasks
types:
  - {name: work.tasks.TaskId, kind: newtype, of: Uuid}
entities:
  - name: work.tasks.Task
    identity: {name: task_id, type: work.tasks.TaskId}
    fields:
      - {name: retries, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: work.tasks.Stale, fields: []}
  - {name: work.tasks.Exhausted, fields: []}
  - {name: work.tasks.Blocked, fields: []}
events:
  - name: work.tasks.TaskCreated
    fields: [{name: task_id, type: work.tasks.TaskId}]
  - name: work.tasks.TaskRetried
    fields: [{name: task_id, type: work.tasks.TaskId}]
commands:
  - name: work.tasks.CreateTask
    input: []
    outcomes:
      - name: created
        creates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskCreated]
        payload:
          work.tasks.TaskCreated: {task_id: {generated: true}}
        sets: {retries: 0}
  - name: work.tasks.RetryTask
    input:
      - {name: task_id, type: work.tasks.TaskId}
      - {name: until, type: Timestamp}
    outcomes:
      - name: stale
        when: until < now
        error: work.tasks.Stale
      - name: at-limit
        when_subject: {predicate: retries >= 18}
        error: work.tasks.Exhausted
      - name: retried
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskRetried]
        payload:
          work.tasks.TaskRetried: {task_id: input.task_id}
        sets: {retries: {increment: 1}}
";

const STALE_SWAP: &str = "error-swap/work.tasks.RetryTask/at-limit";

/// Eighteen retries valid until 2999, then a nineteenth refused as exhausted.
fn stale_limit_scenario() -> String {
    let mut text = String::from(
        "type: ess-scenario/1\ndomain: work.tasks\nscenario: retries-reach-the-limit\nsummary: \
         Eighteen retries, and the nineteenth is refused as exhausted.\narrange:\n  - instance: \
         task\n    entity: work.tasks.Task\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    command: \
         work.tasks.CreateTask\n    input: {}\n    outcome: created\n    capture: {instance: task, \
         event: work.tasks.TaskCreated, field: task_id}\n",
    );
    for second in 1..=19 {
        let outcome = if second == 19 {
            "at-limit, error: {name: work.tasks.Exhausted}"
        } else {
            "retried"
        };
        let _ = writeln!(
            text,
            "  - {{at: 2026-01-05T09:00:{second:02}Z, command: work.tasks.RetryTask, input: \
             {{task_id: {{$instance: task}}, until: 2999-01-01T00:00:00Z}}, outcome: {outcome}}}"
        );
    }
    text
}

/// The `hidden` rule in `Ruler::judge` says an excluded scenario the mutant holds unchanged "asks
/// the target what the baseline asked, so it cannot hide a kill". An authored scenario is held
/// unchanged by construction but runs against a different target, the mutant's interpreter. So an
/// authored scenario the baseline did not execute is excluded and never hides anything, while its
/// `ExpectOutcome` still counts in `witnessed`: the mutant stops being unwitnessed and is reported
/// `survived`, a claim that an executed scenario missed it, though nothing executed observed the
/// outcome. Without `--scenarios` it is `unwitnessed`; with one that executed nothing it must not
/// become a survivor.
#[test]
fn an_authored_scenario_nothing_executed_does_not_turn_an_unwitnessed_mutant_into_a_survivor() {
    let without = audit("stale.yaml", STALE_LIMIT, &[MutantClass::ErrorSwap], &[]);
    assert_eq!(
        entry(&without, STALE_SWAP).verdict,
        Verdict::Unwitnessed,
        "precondition: {}",
        without.render_text()
    );
    let scenario = stale_limit_scenario();
    let with = audit(
        "stale.yaml",
        STALE_LIMIT,
        &[MutantClass::ErrorSwap],
        &[("retries-reach-the-limit.yaml", &scenario)],
    );
    let entry = entry(&with, STALE_SWAP);
    assert_ne!(
        entry.verdict,
        Verdict::Survived,
        "an authored scenario neither run judged it: {entry:#?}\n{}",
        with.render_text()
    );
}

// ---- an identical answer ------------------------------------------------------------------------

const DESK_FIXTURE: &str = include_str!("fixtures/mutation-identical-answer.yaml");

/// The identical-answer fixture without its `components:` block: on a served surface an authored
/// act must be sent as a granted actor, and the fixture grants none. Nothing `identical_answer`
/// reads is in that block.
fn desk() -> String {
    let start = DESK_FIXTURE
        .find("components:")
        .expect("the fixture has components");
    let end = DESK_FIXTURE
        .find("errors:")
        .expect("the fixture has errors");
    format!("{}{}", &DESK_FIXTURE[..start], &DESK_FIXTURE[end..])
}
const OPEN_SWAP: &str = "precedence-swap/desk.entry.Open/leading-hold/trailing-hold";

/// `hold me hold` satisfies both of `Open`'s refusals; both refuse with `Invalid` and no payload.
const BOTH_HOLDS: &str = r"type: ess-scenario/1
domain: desk.entry
scenario: both-holds
summary: A prompt that both starts and ends with hold is refused as invalid.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: desk.entry.Open
    input: {prompt: hold me hold}
    outcome: leading-hold
    error: {name: desk.entry.Invalid}
";

/// <https://github.com/beyond10x/ess/issues/517> reports this swap `equivalent`: "either order
/// answers alike wherever both guards hold", the outcome's name being among what "only names or
/// documents the branch" (`identical_answer`, `mutate.rs`). An authored scenario that names the
/// outcome it expects fails on the swapped interpreter by that name alone, and the entry comes out
/// `killed` while still carrying `identical_answer`: one entry asserting both that the caller
/// cannot tell the two apart and that a scenario did.
#[test]
fn an_identical_answer_swap_is_not_killed_by_the_outcome_label_alone() {
    let without = audit("desk.yaml", &desk(), &[MutantClass::PrecedenceSwap], &[]);
    let before = entry(&without, OPEN_SWAP);
    assert_eq!(
        before.verdict,
        Verdict::Equivalent,
        "precondition: {before:#?}"
    );
    assert!(
        before.identical_answer.is_some(),
        "precondition: {before:#?}"
    );
    let report = audit(
        "desk.yaml",
        &desk(),
        &[MutantClass::PrecedenceSwap],
        &[("both-holds.yaml", BOTH_HOLDS)],
    );
    let entry = entry(&report, OPEN_SWAP);
    assert!(
        !(entry.verdict == Verdict::Killed && entry.identical_answer.is_some()),
        "killed by the label of an answer it records as identical: {entry:#?}"
    );
}

// ---- determinism and the empty set --------------------------------------------------------------

/// Two audits with the same authored scenarios write the same report bytes.
#[test]
fn two_audits_with_authored_scenarios_write_the_same_bytes() {
    let run = || {
        audit(
            "desk.yaml",
            &desk(),
            &[MutantClass::PrecedenceSwap, MutantClass::ErrorSwap],
            &[("both-holds.yaml", BOTH_HOLDS)],
        )
    };
    let (first, second) = (run(), run());
    assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    assert_eq!(first.render_text(), second.render_text());
}

/// `audit_with_authored` with no authored scenario is `audit_with`, on a model with a `now` guard
/// and every class.
#[test]
fn an_empty_authored_set_is_audit_with_on_a_clocked_model() {
    let (documents, texts) = documents("clocked.yaml", CLOCKED);
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let classes = [
        MutantClass::ErrorSwap,
        MutantClass::GuardBoundary,
        MutantClass::GuardNegate,
    ];
    let before = mutate::audit_with(
        &documents,
        &texts,
        &classes,
        || Interpreted::for_model(ir.clone()),
        None,
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let after = mutate::audit_with_authored(
        &documents,
        &texts,
        &classes,
        || Interpreted::for_model(ir.clone()),
        None,
        &BTreeMap::new(),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(before.to_canonical_json(), after.to_canonical_json());
}
