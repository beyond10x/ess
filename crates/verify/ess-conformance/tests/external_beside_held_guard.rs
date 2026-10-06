//! An external branch beside a sibling guard that reads a stored row (beyond10x/ess#464).
//!
//! An external branch's scenario forces its provider and sends the command. A sibling guarded by
//! the held state (`when_subject`, `when_subject_state`) answers before it in the precedence order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order", step 4), and a
//! `when_related` sibling reads its row in declaration order beside it. So the witness is a row and
//! an input that no such sibling claims, whatever the order it is declared in: then the guide's
//! order and the model interpreter's declaration order answer alike. Where no row and input miss
//! every such guard, the scenario is refused naming the guard, and never written for the reference
//! interpreter to fail.
//!
//! `tests/fixtures/external-beside-held-guard.yaml`: `CheckPick` refuses a pick whose stored
//! `revision` differs from the input's (`stale`, a `when_subject` guard) and declares the external
//! `unlisted` after it. Before this change, `unlisted` was sent a revision the arranged row did not
//! hold, so the interpreter answered `stale`, two scenarios failed and `mutate` refused its baseline
//! (ESS-MUTATE-001).
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::ir::{ResolvedCommand, ResolvedCondition};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, AuditRefusal, Document, MutantClass};
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceSuite, InstanceName};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

const UNLISTED: &str = "demo.desk.CheckPick/outcome/unlisted";
const REFUSED_BY_UNLISTED: &str =
    "demo.desk.Pick/transition/refuse/by/demo.desk.CheckPick/unlisted";
const REFUSED_STATE: &str = "demo.desk.Pick/state/Refused/refuses/demo.desk.CheckPick";

/// The `unlisted` branch of the fixture, as declared.
const UNLISTED_BRANCH: &str = "      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
";

/// The `stale` branch of the fixture, as declared.
const STALE_BRANCH: &str = "      - name: stale
        when_subject:
          predicate: revision != input.revision
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";

/// The fixture's shape with the guard over a related row instead (`when_related`): `CheckPick`
/// refuses a list no row carries (`no-list`) and a list whose `revision` differs from the input's
/// (`stale`), and declares the external `unlisted` after both. `unlisted` is the only move to
/// `Refused`, so the state refusal there is arranged through it.
const RELATED: &str = r"format: ess/20
system: demo
version: v1
domain: demo.desk
summary: A pick checked against its list's revision before it is accepted.
types:
  - {name: demo.desk.PickId, kind: newtype, of: Uuid}
  - {name: demo.desk.ListId, kind: newtype, of: Uuid}
entities:
  - name: demo.desk.List
    identity: {name: list_id, type: demo.desk.ListId}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.desk.Pick
    identity: {name: pick_id, type: demo.desk.PickId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Picked
      states: [Picked, Accepted, Refused]
      terminal: [Accepted, Refused]
      transitions:
        - {name: accept, from: [Picked], to: Accepted}
        - {name: refuse, from: [Picked], to: Refused}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.OpenList, demo.desk.MakePick, demo.desk.CheckPick]
commands:
  - name: demo.desk.OpenList
    input:
      - {name: revision, type: Integer}
    outcomes:
      - name: opened
        creates: demo.desk.List
        instance: list_id
        sets: {revision: input.revision}
        emits: [demo.desk.ListOpened]
        payload:
          demo.desk.ListOpened: {list_id: {generated: true}}
  - name: demo.desk.MakePick
    input:
      - {name: note, type: String}
    outcomes:
      - name: picked
        creates: demo.desk.Pick
        instance: pick_id
        sets: {note: input.note}
        emits: [demo.desk.Picked]
        payload:
          demo.desk.Picked: {pick_id: {generated: true}}
  - name: demo.desk.CheckPick
    input:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: list_id, type: demo.desk.ListId}
      - {name: revision, type: Integer}
    outcomes:
      - name: no-list
        when_related: {via: input.list_id, exists: false}
        error: demo.desk.NoList
      - name: stale
        when_related:
          via: input.list_id
          predicate: revision != input.revision
        error: demo.desk.Stale
      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
      - name: accepted
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
events:
  - name: demo.desk.ListOpened
    fields: [{name: list_id, type: demo.desk.ListId}]
  - name: demo.desk.Picked
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickUnlisted
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickAccepted
    fields: [{name: pick_id, type: demo.desk.PickId}]
errors:
  - name: demo.desk.NotPicked
  - name: demo.desk.NoList
  - name: demo.desk.Stale
views:
  - name: demo.desk.Picks
    source: demo.desk.Pick
    consistency: read_your_writes
    fields:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: note, type: String}
      - {name: state, type: demo.desk.Pick.State}
  - name: demo.desk.Lists
    source: demo.desk.List
    consistency: read_your_writes
    fields:
      - {name: list_id, type: demo.desk.ListId}
      - {name: revision, type: Integer}
";

/// A held-state guard (`when_subject_state`) beside an external branch: `Operate` closes an open
/// door pushed as `PUSH` names (`shut`), the external `stalled` jams an open door, and
/// everything else is `refused`. `stalled` is the only move to `Jammed`, so a jammed door is
/// arranged through it.
const DOOR: &str = r"format: ess/20
system: demo
version: v1
domain: demo.doors
summary: A door closed gently, or jammed when its motor stalls.
types:
  - {name: demo.doors.DoorId, kind: newtype, of: Uuid}
  - {name: demo.doors.Push, kind: enum, variants: [Gentle, Rough]}
entities:
  - name: demo.doors.Door
    identity: {name: door_id, type: demo.doors.DoorId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed, Jammed]
      terminal: [Closed, Jammed]
      transitions:
        - {name: close, from: [Open], to: Closed}
        - {name: jam, from: [Open], to: Jammed}
actors:
  - name: demo.doors.Keeper
    may: [demo.doors.Install, demo.doors.Operate]
commands:
  - name: demo.doors.Install
    input:
      - {name: label, type: String}
    outcomes:
      - name: installed
        creates: demo.doors.Door
        instance: door_id
        sets: {label: input.label}
        emits: [demo.doors.Installed]
        payload:
          demo.doors.Installed: {door_id: {generated: true}}
  - name: demo.doors.Operate
    input:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: push, type: demo.doors.Push}
    outcomes:
      - name: shut
        when_subject_state: Open
        when: push == PUSH
        moves: demo.doors.Door.close
        instance: door_id
        emits: [demo.doors.Shut]
        payload:
          demo.doors.Shut: {door_id: input.door_id}
      - name: stalled
        external: the motor stalls
        moves: demo.doors.Door.jam
        instance: door_id
        emits: [demo.doors.Stalled]
        payload:
          demo.doors.Stalled: {door_id: input.door_id}
      - name: refused
        error: demo.doors.NotOpen
events:
  - name: demo.doors.Installed
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Shut
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Stalled
    fields: [{name: door_id, type: demo.doors.DoorId}]
errors:
  - name: demo.doors.NotOpen
views:
  - name: demo.doors.Doors
    source: demo.doors.Door
    consistency: read_your_writes
    fields:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: label, type: String}
      - {name: state, type: demo.doors.Door.State}
";

/// A row-set guard (`ess/22`) beside an external branch: `Close` refuses while the attempt's
/// worker and batch hold more than `LIMIT` attempts (`crowded`), and the external `stuck` closes
/// the attempt when its worker does not answer.
const ROW_SET: &str = r"format: ess/22
system: demo
version: v1
domain: demo.jobs
summary: Attempts closed unless their worker and batch are crowded.
types:
  - {name: demo.jobs.AttemptId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Attempt
    identity: {name: attempt_id, type: demo.jobs.AttemptId}
    fields:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - {name: demo.jobs.Crowded, summary: The worker and batch hold too many attempts., fields: []}
  - {name: demo.jobs.UnknownAttempt, summary: No attempt carries the identity., fields: []}
  - {name: demo.jobs.AlreadyClosed, summary: The attempt is closed already., fields: []}
events:
  - name: demo.jobs.AttemptRecorded
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
  - name: demo.jobs.AttemptStuck
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
  - name: demo.jobs.AttemptClosed
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
commands:
  - name: demo.jobs.Record
    input:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
    outcomes:
      - name: recorded
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets: {worker_id: input.worker_id, batch_id: input.batch_id}
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {attempt_id: {generated: true}}
  - name: demo.jobs.Close
    input:
      - {name: attempt_id, type: demo.jobs.AttemptId}
    outcomes:
      - name: unknown-attempt
        unknown_instance: true
        error: demo.jobs.UnknownAttempt
      - name: already-closed
        wrong_state: true
        error: demo.jobs.AlreadyClosed
      - name: crowded
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == subject.worker_id, batch_id == subject.batch_id]}
          count: {gt: LIMIT}
        error: demo.jobs.Crowded
      - name: stuck
        external: the worker does not answer
        moves: demo.jobs.Attempt.close
        instance: attempt_id
        emits: [demo.jobs.AttemptStuck]
        payload:
          demo.jobs.AttemptStuck: {attempt_id: input.attempt_id}
      - name: closed
        moves: demo.jobs.Attempt.close
        instance: attempt_id
        emits: [demo.jobs.AttemptClosed]
        payload:
          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}
views:
  - name: demo.jobs.Attempts
    source: demo.jobs.Attempt
    consistency: read_your_writes
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
      - {name: state, type: demo.jobs.Attempt.State}
";

fn parsed(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("model.yaml"), raw)], texts)
}

fn ir(text: &str) -> EssIr {
    let (documents, texts) = parsed(text);
    let spec =
        Specification::assemble(documents).unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
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
                    ids(result),
                    refusals(result)
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every scenario of `suite` run on the model interpreter of `text`, by id.
fn run(text: &str, suite: &ConformanceSuite) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// Every send of `command` in `scenario`: its index and its input.
fn sends<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
) -> Vec<(usize, &'a BTreeMap<String, ScenarioValue>)> {
    scenario
        .steps
        .iter()
        .enumerate()
        .filter_map(|(at, step)| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.name().to_string() == command => Some((at, input)),
            _ => None,
        })
        .collect()
}

/// Whether the step before `at` forces `outcome` of `command`.
fn forced_before(scenario: &ConformanceScenario, at: usize, command: &str, outcome: &str) -> bool {
    at.checked_sub(1)
        .and_then(|before| scenario.steps.get(before))
        .is_some_and(|step| {
            matches!(step, ScenarioStep::ConfigureExternalOutcome { force, .. }
                if force.command.name().to_string() == command && force.outcome.to_string() == outcome)
        })
}

/// The send of `command` that `outcome`'s provider is forced for, in `scenario`.
fn forced_send<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
    outcome: &str,
) -> &'a BTreeMap<String, ScenarioValue> {
    let Some((_, input)) = sends(scenario, command)
        .into_iter()
        .find(|(at, _)| forced_before(scenario, *at, command, outcome))
    else {
        panic!("no forced send of {command}/{outcome}: {scenario:#?}");
    };
    input
}

/// The `field` each instance of `entity` a send of `creator` captured was created with, by
/// instance: the value the row holds, read from the creating send.
fn created_with(
    scenario: &ConformanceScenario,
    creator: &str,
    field: &str,
) -> BTreeMap<InstanceName, ScenarioValue> {
    let mut found = BTreeMap::new();
    for (at, input) in sends(scenario, creator) {
        let captured = scenario.steps[at + 1..]
            .iter()
            .take_while(|step| !matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .find_map(|step| match step {
                ScenarioStep::CaptureInstance { instance, .. } => Some(instance.clone()),
                _ => None,
            });
        if let (Some(instance), Some(value)) = (captured, input.get(field)) {
            found.insert(instance, value.clone());
        }
    }
    found
}

/// The `revision` the `Pick` the forced `unlisted` send names was created with, and the one sent.
fn sent_against_row(scenario: &ConformanceScenario) -> (ScenarioValue, ScenarioValue) {
    let sent = forced_send(scenario, "demo.desk.CheckPick", "unlisted");
    let ScenarioValue::Instance { instance } = &sent["pick_id"] else {
        panic!("the pick is an arranged one: {sent:#?}");
    };
    let rows = created_with(scenario, "demo.desk.MakePick", "revision");
    (rows[instance].clone(), sent["revision"].clone())
}

fn moved_before(text: &str, moved: &str, before: &str) -> String {
    let without = text.replacen(moved, "", 1);
    assert_ne!(without, text, "the branch is declared");
    let moved_text = without.replacen(before, &format!("{moved}{before}"), 1);
    assert_ne!(moved_text, without, "the anchor is declared");
    moved_text
}

#[test]
fn external_witness_refutes_when_subject() {
    let result = synthesis(MODEL);
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        let (held, sent) = sent_against_row(scenario(&result, id));
        assert_eq!(
            held, sent,
            "{id}: `unlisted` is sent the revision its row holds, so `stale` does not claim it"
        );
    }
    let statuses = run(MODEL, &result.suite);
    assert_eq!(statuses.len(), 10, "{:#?}", ids(&result));
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

#[test]
fn external_witness_mutate_baseline_passes() {
    let (documents, texts) = parsed(MODEL);
    let ir = ir(MODEL);
    match mutate::audit(&documents, &texts, MutantClass::ALL, || {
        Interpreted::for_model(ir.clone())
    }) {
        Ok(report) => assert!(report.baseline.scenarios > 0, "{report:#?}"),
        Err(refusal @ AuditRefusal::BaselineFailed { .. }) => {
            panic!("the unmutated suite fails its baseline: {refusal}")
        }
        Err(refusal) => panic!("the audit refused: {refusal}"),
    }
}

#[test]
fn external_declared_first_still_refutes() {
    let first = moved_before(MODEL, UNLISTED_BRANCH, STALE_BRANCH);
    let result = synthesis(&first);
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        let (held, sent) = sent_against_row(scenario(&result, id));
        assert_eq!(
            held, sent,
            "{id}: declared first, `unlisted` is still sent an input `stale` does not claim"
        );
    }
    let statuses = run(&first, &result.suite);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

/// The `revision` of the `List` the forced `unlisted` send names, and the one sent; `None` for a
/// list the scenario did not arrange.
fn sent_against_list(
    scenario: &ConformanceScenario,
    at: usize,
) -> (Option<ScenarioValue>, ScenarioValue) {
    let ScenarioStep::ExecuteCommand { input: sent, .. } = &scenario.steps[at] else {
        panic!("a send");
    };
    let lists = created_with(scenario, "demo.desk.OpenList", "revision");
    let held = match &sent["list_id"] {
        ScenarioValue::Instance { instance } => lists.get(instance).cloned(),
        _ => None,
    };
    (held, sent["revision"].clone())
}

#[test]
fn external_witness_arranges_related_row() {
    let result = synthesis(RELATED);
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        let found = scenario(&result, id);
        let (at, _) = sends(found, "demo.desk.CheckPick")
            .into_iter()
            .find(|(at, _)| forced_before(found, *at, "demo.desk.CheckPick", "unlisted"))
            .unwrap_or_else(|| panic!("{id}: no forced send"));
        let (held, sent) = sent_against_list(found, at);
        assert_eq!(
            held,
            Some(sent),
            "{id}: `unlisted` names a present list holding the revision it is sent"
        );
    }
    let statuses = run(RELATED, &result.suite);
    for id in [UNLISTED, REFUSED_BY_UNLISTED, REFUSED_STATE] {
        assert_eq!(
            statuses.get(id),
            Some(&Status::Passed),
            "{id}: {:#?}\n refusals: {:#?}",
            statuses,
            refusals(&result)
        );
    }
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

#[test]
fn external_driver_refutes_sibling_guards() {
    // A related row: the state refusal in `Refused` is arranged through `unlisted`.
    let result = synthesis(RELATED);
    let found = scenario(&result, REFUSED_STATE);
    let (at, _) = sends(found, "demo.desk.CheckPick")
        .into_iter()
        .find(|(at, _)| forced_before(found, *at, "demo.desk.CheckPick", "unlisted"))
        .unwrap_or_else(|| panic!("the arrangement runs `unlisted`: {found:#?}"));
    let (held, sent) = sent_against_list(found, at);
    assert_eq!(
        held,
        Some(sent),
        "the driver names a present list it does not refuse"
    );
    let statuses = run(RELATED, &result.suite);
    assert_eq!(
        statuses.get(REFUSED_STATE),
        Some(&Status::Passed),
        "{statuses:#?}"
    );

    // A held state: a jammed door is arranged through `stalled`, sent an input `shut` does not
    // claim on the open door, whichever push selects `shut`.
    for (push, other) in [("Gentle", "Rough"), ("Rough", "Gentle")] {
        let door = DOOR.replace("PUSH", push);
        let result = synthesis(&door);
        let mut driven = 0;
        for (id, found) in &result.suite.scenarios {
            let sent = sends(found, "demo.doors.Operate");
            let Some((last, _)) = sent.last() else {
                continue;
            };
            for (at, input) in &sent {
                if at == last || !forced_before(found, *at, "demo.doors.Operate", "stalled") {
                    continue;
                }
                driven += 1;
                assert_eq!(
                    input["push"],
                    variant(other),
                    "{push}: {id}: the driver is sent an input `shut` does not claim"
                );
            }
        }
        assert!(
            driven > 0,
            "{push}: an arrangement runs `stalled`: {:#?}",
            ids(&result)
        );
        let statuses = run(&door, &result.suite);
        assert_eq!(not_passed(&statuses), Vec::<String>::new(), "{push}");
    }
}

/// An enum input's value as a scenario sends it.
fn variant(name: &str) -> ScenarioValue {
    ScenarioValue::literal(ess_primitives::node::Node::Text(name.into()))
}

#[test]
fn external_witness_refutes_when_subject_state() {
    for (push, other) in [("Gentle", "Rough"), ("Rough", "Gentle")] {
        let door = DOOR.replace("PUSH", push);
        let result = synthesis(&door);
        for id in [
            "demo.doors.Operate/outcome/stalled",
            "demo.doors.Door/transition/jam/by/demo.doors.Operate/stalled",
        ] {
            let sent = forced_send(scenario(&result, id), "demo.doors.Operate", "stalled");
            assert_eq!(
                sent["push"],
                variant(other),
                "{push}: {id} is sent an input `shut` does not claim on the open door"
            );
        }
        let statuses = run(&door, &result.suite);
        assert_eq!(not_passed(&statuses), Vec::<String>::new(), "{push}");
    }
}

#[test]
fn external_guarded_by_when_keeps_own_guard() {
    let guarded = MODEL
        .replace(
            "      - {name: revision, type: Integer}\n    outcomes:\n      - name: stale",
            "      - {name: revision, type: Integer}\n      - {name: channel, type: String}\n    outcomes:\n      - name: stale",
        )
        .replace(
            "        external: the current list does not name the pick\n",
            "        external: the current list does not name the pick\n        when: channel == \"fax\"\n",
        );
    assert_ne!(guarded, MODEL);
    let result = synthesis(&guarded);
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        let found = scenario(&result, id);
        let (held, sent) = sent_against_row(found);
        assert_eq!(held, sent, "{id}: `stale` does not claim the input");
        let input = forced_send(found, "demo.desk.CheckPick", "unlisted");
        assert_eq!(
            input["channel"],
            ScenarioValue::literal(ess_primitives::node::Node::Text("fax".into())),
            "{id}: the input is one `unlisted`'s own guard admits"
        );
    }
    let statuses = run(&guarded, &result.suite);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

#[test]
fn unrefutable_sibling_refused_by_name() {
    // Every pick rests in `Picked` where `unlisted` moves from, and `frozen` holds of every one.
    let frozen = MODEL.replace(
        "        when_subject:\n          predicate: revision != input.revision\n",
        "        when_subject:\n          predicate: state == Picked\n",
    );
    assert_ne!(frozen, MODEL);
    let result = synthesis(&frozen);
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        assert!(
            !ids(&result).contains(&id.to_owned()),
            "{id} is not written for the interpreter to fail"
        );
        let named = result
            .refusals
            .iter()
            .filter(|refusal| {
                refusal
                    .scenario
                    .as_ref()
                    .is_some_and(|it| it.to_string() == id)
            })
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(
            named
                .iter()
                .any(|text| text.starts_with("refusal[ESS-SYNTH-003]") && text.contains("`stale`")),
            "{id} is refused naming the guard that claims it: {named:#?}\n all: {:#?}",
            refusals(&result)
        );
    }
    let statuses = run(&frozen, &result.suite);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

#[test]
fn row_set_sibling_witnessed_or_refused() {
    let stuck = "demo.jobs.Close/outcome/stuck";
    for limit in ["1", "0"] {
        let text = ROW_SET.replace("LIMIT", limit);
        let result = synthesis(&text);
        let statuses = run(&text, &result.suite);
        assert_eq!(not_passed(&statuses), Vec::<String>::new(), "gt {limit}");
        let written = ids(&result).contains(&stuck.to_owned());
        let named = result.refusals.iter().any(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|it| it.to_string() == stuck)
                && refusal.to_string().contains("crowded")
        });
        assert!(
            written != named,
            "gt {limit}: `stuck` is witnessed or refused naming `crowded`, not both, not neither: \
             {:#?}",
            refusals(&result)
        );
        // Over one attempt the guard is refuted; every attempt is in its own scope, so over none
        // it never is.
        assert_eq!(
            written,
            limit == "1",
            "gt {limit}: {:#?}",
            refusals(&result)
        );
    }
}

// ---- an external witness no sibling claims keeps its bytes --------------------------------------

/// The canonical bytes of the one-scenario suite holding `id`, as `<length>:<hash>`: the digest
/// adversary pass 1 of beyond10x/ess#464 pinned these scenarios with at base (bc4884203).
fn scenario_digest(result: &Synthesis, id: &str) -> String {
    let mut one = result.suite.clone();
    one.scenarios.retain(|other, _| other.to_string() == id);
    assert_eq!(one.scenarios.len(), 1, "no scenario {id}");
    let json = one.to_canonical_json().unwrap();
    let hash = json.bytes().fold(0u64, |h, b| {
        h.wrapping_mul(1_099_511_628_211).wrapping_add(u64::from(b))
    });
    format!("{}:{hash:x}", json.len())
}

/// External witnesses no sibling claimed at base (bc4884203), each passing the model interpreter
/// there, keep their bytes: the plain witness is kept wherever no sibling claims it. `stalled` is
/// sent `Gentle` beside `shut`, which claims only `Rough`; `unlisted` beside a `stale` that claims
/// only a stored revision above the input's. [`repository_model_suites_change_only_where_claimed`]
/// leaves every external witness beside such a guard out, claimed or not, so this is where an
/// unclaimed one is held.
#[test]
fn unclaimed_external_witnesses_keep_their_bytes() {
    let door = DOOR.replace("PUSH", "Rough");
    let gt = MODEL.replace("revision != input.revision", "revision > input.revision");
    let pinned: [(&str, &str, &str); 5] = [
        (
            "door",
            "demo.doors.Operate/outcome/stalled",
            "4856:38de8940626a100",
        ),
        (
            "door",
            "demo.doors.Door/transition/jam/by/demo.doors.Operate/stalled",
            "4879:a00983a30a269bec",
        ),
        (
            "door",
            "demo.doors.Door/state/Jammed/refuses/demo.doors.Operate",
            "5366:e6d3404ee10ab81d",
        ),
        (
            "gt",
            "demo.desk.CheckPick/outcome/unlisted",
            "4501:86dd28889c3dcb9b",
        ),
        (
            "gt",
            "demo.desk.Pick/transition/refuse/by/demo.desk.CheckPick/unlisted",
            "4529:3cea3716e45084e9",
        ),
    ];
    let door_result = synthesis(&door);
    let gt_result = synthesis(&gt);
    for (label, text, result) in [("door", &door, &door_result), ("gt", &gt, &gt_result)] {
        assert_eq!(
            not_passed(&run(text, &result.suite)),
            Vec::<String>::new(),
            "{label}"
        );
    }
    let moved: Vec<String> = pinned
        .iter()
        .filter_map(|(model, id, base)| {
            let result = if *model == "door" {
                &door_result
            } else {
                &gt_result
            };
            let here = scenario_digest(result, id);
            (here != *base).then(|| format!("{model}: {id}: base {base}, here {here}"))
        })
        .collect();
    assert_eq!(
        moved,
        Vec::<String>::new(),
        "an unclaimed external witness moved"
    );
}

// ---- every repository model, before and after ---------------------------------------------------

/// The digests of every model of the base of beyond10x/ess#464 (bc4884203), written by this file
/// with `EXTERNAL_WITNESS_BASE_WRITE=<path>` before the change: one line per model, as
/// [`model_line`] writes it.
const BASE: &str = include_str!("fixtures/external-beside-held-guard-base.tsv");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &str) -> String {
    let hex =
        Sha256::digest(bytes.as_bytes())
            .iter()
            .take(10)
            .fold(String::new(), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            });
    format!("{hex}:{}", bytes.len())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !["target", "node_modules", ".git", ".engineering"].contains(&name.as_str()) {
                walk(&path, out);
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

/// Whether `command` declares a branch guarded by the stored row it addresses, its held state, a
/// related row or a row set: a sibling that may claim an external branch's witness.
fn reads_rows(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectField { .. }
                | ResolvedCondition::SubjectPredicate { .. }
                | ResolvedCondition::SubjectState { .. }
                | ResolvedCondition::StateChange { .. }
                | ResolvedCondition::Related { .. }
                | ResolvedCondition::RelatedSet { .. }
        )
    })
}

/// Whether `scenario` forces an external branch of a command [`reads_rows`] beside: the scenarios
/// whose bytes this change may move, and only where a sibling claimed them.
fn claimable(ir: &EssIr, scenario: &ConformanceScenario) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ConfigureExternalOutcome { force, .. }
            if ir.commands().get(force.command.name()).is_some_and(reads_rows))
    })
}

/// Whether `id` names the outcome or transition scenario of an external branch of a command
/// [`reads_rows`] beside: a refusal of one is a scenario such a sibling claimed.
fn claimable_id(ir: &EssIr, id: &str) -> bool {
    ir.commands()
        .values()
        .filter(|command| reads_rows(command))
        .any(|command| {
            command
                .outcomes
                .iter()
                .filter(|outcome| {
                    matches!(
                        outcome.condition,
                        ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
                    )
                })
                .any(|outcome| {
                    id == format!("{}/outcome/{}", command.name, outcome.name)
                        || id.ends_with(&format!("/by/{}/{}", command.name, outcome.name))
                })
        })
}

/// One model's line: its suite without the [`claimable`] scenarios, its refusals without those
/// about a [`claimable_id`] or about a scenario `exempt` names, and the claimable scenarios.
fn model_line(
    documents: Vec<(Source, RawSpecFile)>,
    sources: &SourceMap,
    exempt: &[String],
) -> String {
    let spec = match Specification::assemble(documents) {
        Ok(spec) => spec,
        Err(errors) => return format!("ASSEMBLE {}", digest(&errors.to_string())),
    };
    let ir = match compile(&spec, sources) {
        Ok(ir) => ir,
        Err(error) => return format!("COMPILE {}", digest(&format!("{error:?}"))),
    };
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let mut suite = synthesis.suite.clone();
    let claimed: Vec<String> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| claimable(&ir, scenario))
        .map(|(id, _)| id.to_string())
        .collect();
    suite
        .scenarios
        .retain(|id, _| !claimed.contains(&id.to_string()));
    let kept = suite
        .to_canonical_json()
        .unwrap_or_else(|error| format!("SUITE-ERROR {error:?}"));
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal.scenario.as_ref().is_none_or(|id| {
                let id = id.to_string();
                !claimable_id(&ir, &id) && !claimed.contains(&id) && !exempt.contains(&id)
            })
        })
        .map(ToString::to_string)
        .collect();
    format!(
        "suite={} refusals={} claimable={}",
        digest(&kept),
        digest(&refusals.join("\n")),
        claimed.join(",")
    )
}

/// The documents and sources of every model this tree holds, by label: each ESS source file alone,
/// and each `examples/<name>` directory as one system. `None` for one that does not parse.
type Model = Option<(Vec<(Source, RawSpecFile)>, SourceMap)>;

fn models(root: &Path) -> Vec<(String, Model)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        let model = RawSpecFile::parse(&text).ok().map(|raw| {
            let mut sources = SourceMap::new();
            sources.insert(label.clone(), text.clone());
            (vec![(Source::new(label.clone()), raw)], sources)
        });
        found.push((label, model));
    }
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("examples"))
        .map(|entries| {
            entries
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    for dir in dirs {
        let mut members = Vec::new();
        walk(&dir, &mut members);
        members.sort();
        let mut sources = SourceMap::new();
        let mut documents = Vec::new();
        let mut unparsed = false;
        for path in members {
            let text = std::fs::read_to_string(&path).unwrap();
            if !text.lines().any(|line| line.starts_with("format: ess/")) {
                continue;
            }
            let label = path.strip_prefix(&dir).unwrap().display().to_string();
            match RawSpecFile::parse(&text) {
                Ok(raw) => {
                    sources.insert(label.clone(), text);
                    documents.push((Source::new(label), raw));
                }
                Err(_) => unparsed = true,
            }
        }
        if documents.is_empty() && !unparsed {
            continue;
        }
        let name = dir.strip_prefix(root).unwrap().display().to_string();
        found.push((
            format!("examples-dir:{name}"),
            (!unparsed).then_some((documents, sources)),
        ));
    }
    found
}

/// The models this test does not synthesize, each with why that loses nothing: synthesis of
/// `conditional-measures-generated.yaml` takes minutes in a debug build, and it declares no
/// external branch, so no path this change touches is entered for it. Its suite bytes are pinned
/// by the slow probe `adversary_244b_window_free_bytes` (`task test-slow-probes`).
const UNAFFORDABLE: &[&str] =
    &["crates/generate/ess-synth/tests/fixtures/conditional-measures-generated.yaml"];

/// Whether the model `documents` compile to declares an external branch.
fn declares_external(documents: Vec<(Source, RawSpecFile)>, sources: &SourceMap) -> bool {
    let Ok(spec) = Specification::assemble(documents) else {
        return true;
    };
    let Ok(ir) = compile(&spec, sources) else {
        return true;
    };
    ir.commands().values().any(|command| {
        command.outcomes.iter().any(|outcome| {
            matches!(
                outcome.condition,
                ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
            )
        })
    })
}

#[test]
fn repository_model_suites_change_only_where_claimed() {
    let root = root();
    let here = models(&root);
    if let Ok(path) = std::env::var("EXTERNAL_WITNESS_BASE_WRITE") {
        let mut text = String::new();
        for (label, model) in here {
            let line = model.map_or_else(
                || "PARSE".to_owned(),
                |(documents, sources)| model_line(documents, &sources, &[]),
            );
            writeln!(text, "{label}\t{line}").unwrap();
        }
        std::fs::write(path, text).unwrap();
        return;
    }
    let mut moved = Vec::new();
    let mut compared = 0usize;
    let mut skipped = 0usize;
    let mut claimable_seen = 0usize;
    for pinned in BASE.lines().filter(|line| !line.is_empty()) {
        let (label, base) = pinned.split_once('\t').expect("label, then digests");
        compared += 1;
        let exempt: Vec<String> = base
            .rsplit_once("claimable=")
            .map(|(_, ids)| {
                ids.split(',')
                    .filter(|id| !id.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        claimable_seen += exempt.len();
        let Some((_, model)) = here.iter().find(|(name, _)| name == label) else {
            moved.push(format!("{label}: no longer found"));
            continue;
        };
        if UNAFFORDABLE.contains(&label) {
            let (documents, sources) = model.clone().expect("an unaffordable model parses");
            assert!(
                !declares_external(documents, &sources),
                "{label} declares an external branch, so it is synthesized like every other model"
            );
            skipped += 1;
            continue;
        }
        let line = model.clone().map_or_else(
            || "PARSE".to_owned(),
            |(documents, sources)| model_line(documents, &sources, &exempt),
        );
        // The claimable scenarios may differ: each is an external branch's witness a sibling
        // guard may have claimed, now re-witnessed or refused by name.
        let without_claimable = |line: &str| {
            line.rsplit_once(" claimable=")
                .map_or_else(|| line.to_owned(), |(kept, _)| kept.to_owned())
        };
        if without_claimable(&line) != without_claimable(base) {
            moved.push(format!("{label}\n  base {base}\n  here {line}"));
        }
    }
    assert_eq!(compared, 191, "every pinned model is read");
    assert_eq!(
        skipped,
        UNAFFORDABLE.len(),
        "every unaffordable model is pinned"
    );
    assert_eq!(
        moved,
        Vec::<String>::new(),
        "bytes moved outside the external witnesses a sibling guard claims ({claimable_seen} \
         claimable at base)"
    );
}
