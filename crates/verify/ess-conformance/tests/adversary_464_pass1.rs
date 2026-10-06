//! Adversary pass 1 against beyond10x/ess#464: an external branch's witness beside a sibling guard
//! that reads a stored row.
//!
//! The story's acceptance: synthesis witnesses an external branch, and every run that drives a
//! row through one, on a row and an input that no input refusal, held-state guard
//! (`when_subject`, `when_subject_state`) or related guard claims; where none exists the scenario
//! is refused naming the guard, and a suite the reference model fails is never written.
//!
//! Each model below is the unit's fixture shape with one thing added that the unit's own cases do
//! not combine: an external driver beside a `when_subject` guard, a held-state predicate with an
//! input guard, an external refusal that names no subject, a stored related reference, an
//! external guard overlapping a sibling's, an accepting `when_subject` declared after the
//! external, an external guard over the field the stored guard compares, two externals, a
//! two-field stored guard, an ess/6 `{field, equals}` sibling, an external moving from a later
//! state, a move-less external, a re-witnessed external that writes a field, and the base bytes of
//! external witnesses no sibling claims. Every suite is run on the model interpreter, as the
//! unit's own tests do.
//!
//! `when_subject_state:` beside `when_subject:`, `when_subject_state:` beside `when_related:`, and
//! `wrong_state:` beside `when_subject_state:` are refused by validation, so no case builds them.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceSuite, InstanceName};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble(vec![(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
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

fn refusal_of(result: &Synthesis, id: &str) -> Option<String> {
    result
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|it| it.to_string() == id)
        })
        .map(ToString::to_string)
}

/// Every scenario of `suite` run on the model interpreter of `text`, by id, with what each
/// diagnosed.
fn run(text: &str, suite: &ConformanceSuite) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnosed = result
                .diagnostics()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            (result.scenario.to_string(), (result.status, diagnosed))
        })
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, (Status, String)>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, diagnosed))| format!("{id}: {status:?}\n{diagnosed}"))
        .collect()
}

/// The synthesized suite of `text` passes the model interpreter of `text`, every scenario.
fn assert_reference_passes(label: &str, text: &str) -> Synthesis {
    let result = synthesis(text);
    let statuses = run(text, &result.suite);
    assert!(
        !statuses.is_empty(),
        "{label}: an empty suite proves nothing"
    );
    let failing = not_passed(&statuses);
    let steps: Vec<String> = result
        .suite
        .scenarios
        .iter()
        .filter(|(id, _)| {
            statuses
                .get(&id.to_string())
                .is_some_and(|(s, _)| *s != Status::Passed)
        })
        .map(|(id, scenario)| {
            let lines: Vec<String> = scenario
                .steps
                .iter()
                .map(|step| format!("    {step:?}"))
                .collect();
            format!("{id} steps:\n{}", lines.join("\n"))
        })
        .collect();
    assert_eq!(
        failing.len(),
        0,
        "{label}: a suite the reference model fails is written\n{}\n{}\n refusals: {:#?}",
        failing.join("\n"),
        steps.join("\n"),
        refusals(&result)
    );
    result
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

/// The `field` each instance a send of `creator` captured was created with, by instance.
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

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "anchor not found: {from}");
    out
}

// ---- 1. an external driver beside a `when_subject` guard ----------------------------------------

/// The fixture's `CheckPick`, with `unlisted` the only move into `Unlisted` and an `Archive`
/// command that runs from `Accepted` and `Refused`: so the `Unlisted` state refusals of
/// `CheckPick` and `Archive` are arranged by driving a pick through the external `unlisted`, on a
/// row `stale` (a `when_subject` guard) reads.
const DRIVER_WHEN_SUBJECT: &str = r"format: ess/20
system: demo
version: v1
domain: demo.desk
summary: A pick checked against the current revision, delisted by the provider.
types:
  - {name: demo.desk.PickId, kind: newtype, of: Uuid}
entities:
  - name: demo.desk.Pick
    identity: {name: pick_id, type: demo.desk.PickId}
    fields:
      - {name: revision, type: Integer}
    lifecycle:
      initial: Picked
      states: [Picked, Accepted, Refused, Unlisted, Archived]
      terminal: [Unlisted, Archived]
      transitions:
        - {name: accept, from: [Picked], to: Accepted}
        - {name: refuse, from: [Picked], to: Refused}
        - {name: unlist, from: [Picked], to: Unlisted}
        - {name: archive, from: [Accepted, Refused], to: Archived}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.MakePick, demo.desk.CheckPick, demo.desk.Archive]
commands:
  - name: demo.desk.MakePick
    input:
      - {name: revision, type: Integer}
    outcomes:
      - name: picked
        creates: demo.desk.Pick
        instance: pick_id
        sets: {revision: input.revision}
        emits: [demo.desk.Picked]
        payload:
          demo.desk.Picked: {pick_id: {generated: true}}
  - name: demo.desk.CheckPick
    input:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: revision, type: Integer}
    outcomes:
      - name: stale
        when_subject:
          predicate: revision != input.revision
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.unlist
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
      - name: wrong-state
        wrong_state: true
        error: demo.desk.NotPicked
  - name: demo.desk.Archive
    input:
      - {name: pick_id, type: demo.desk.PickId}
    outcomes:
      - name: archived
        moves: demo.desk.Pick.archive
        instance: pick_id
        emits: [demo.desk.PickArchived]
        payload:
          demo.desk.PickArchived: {pick_id: input.pick_id}
      - name: not-archivable
        wrong_state: true
        error: demo.desk.NotArchivable
events:
  - name: demo.desk.Picked
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickStale
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickUnlisted
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickAccepted
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickArchived
    fields: [{name: pick_id, type: demo.desk.PickId}]
errors:
  - name: demo.desk.NotPicked
  - name: demo.desk.NotArchivable
views:
  - name: demo.desk.Picks
    source: demo.desk.Pick
    consistency: read_your_writes
    fields:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: revision, type: Integer}
      - {name: state, type: demo.desk.Pick.State}
";

#[test]
fn adversary_464_external_driver_beside_when_subject_reaches_its_state() {
    let result = synthesis(DRIVER_WHEN_SUBJECT);
    let mut driven = 0;
    let mut mismatched = Vec::new();
    for (id, found) in &result.suite.scenarios {
        let sent = sends(found, "demo.desk.CheckPick");
        let Some((last, _)) = sent.last() else {
            continue;
        };
        let rows = created_with(found, "demo.desk.MakePick", "revision");
        for (at, input) in &sent {
            if at == last || !forced_before(found, *at, "demo.desk.CheckPick", "unlisted") {
                continue;
            }
            driven += 1;
            let ScenarioValue::Instance { instance } = &input["pick_id"] else {
                continue;
            };
            if rows.get(instance) != Some(&input["revision"]) {
                mismatched.push(format!(
                    "{id}: driver sent revision {:?} to a row created with {:?}",
                    input["revision"],
                    rows.get(instance)
                ));
            }
        }
    }
    assert!(
        driven > 0,
        "an arrangement runs `unlisted` as a driver: {:#?}\n refusals: {:#?}",
        ids(&result),
        refusals(&result)
    );
    let statuses = run(DRIVER_WHEN_SUBJECT, &result.suite);
    assert_eq!(
        (mismatched, not_passed(&statuses)),
        (Vec::<String>::new(), Vec::<String>::new()),
        "the external driver is sent an input `stale` claims on the row it drives"
    );
}

// ---- 2. a held-state guard read through `when_subject` beside a stored-field guard ---------------

/// The fixture with a refusal `held` declared first that reads the held state through the
/// predicate (`state == Picked`, the form validation offers in place of `when_subject_state:` on
/// a command that reads stored fields) beside an input guard: every pick `unlisted` moves from
/// satisfies its stored half, so only an input refuting `guard` leaves `unlisted` to answer.
fn when_subject_and_state(guard: &str) -> String {
    let text = replaced(
        MODEL,
        "      - {name: revision, type: Integer}\n    outcomes:\n      - name: stale\n",
        &format!(
            "      - {{name: revision, type: Integer}}\n      - {{name: channel, type: String}}\n    outcomes:\n      - name: held\n        when_subject:\n          predicate: state == Picked\n        when: {guard}\n        error: demo.desk.Held\n      - name: stale\n"
        ),
    );
    replaced(
        &text,
        "  - name: demo.desk.NotPicked\n",
        "  - name: demo.desk.NotPicked\n  - name: demo.desk.Held\n",
    )
}

#[test]
fn adversary_464_held_state_predicate_with_input_guard_is_refuted() {
    for guard in ["channel == \"fax\"", "channel != \"fax\""] {
        let text = when_subject_and_state(guard);
        let result = assert_reference_passes(guard, &text);
        for id in [
            "demo.desk.CheckPick/outcome/unlisted",
            "demo.desk.Pick/transition/refuse/by/demo.desk.CheckPick/unlisted",
        ] {
            assert!(
                ids(&result).contains(&id.to_owned()) || refusal_of(&result, id).is_some(),
                "{guard}: {id} is witnessed or refused by name"
            );
        }
    }
}

// ---- 4. an external refusal that names no subject ------------------------------------------------

/// The unit's door shape with an external refusal `offline` that names no subject of its own,
/// declared after `shut`: its scenario still sends `Operate` for an open door.
const DOOR_OFFLINE: &str = r"format: ess/20
system: demo
version: v1
domain: demo.doors
summary: A door closed gently, refused while its controller is offline.
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
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
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
      - name: offline
        external: the door controller is offline
        error: demo.doors.Offline
      - name: refused
        error: demo.doors.NotOpen
events:
  - name: demo.doors.Installed
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Shut
    fields: [{name: door_id, type: demo.doors.DoorId}]
errors:
  - name: demo.doors.NotOpen
  - name: demo.doors.Offline
views:
  - name: demo.doors.Doors
    source: demo.doors.Door
    consistency: read_your_writes
    fields:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: label, type: String}
      - {name: state, type: demo.doors.Door.State}
";

#[test]
fn adversary_464_subjectless_external_refusal_beside_when_subject_state() {
    for push in ["Gentle", "Rough"] {
        let text = DOOR_OFFLINE.replace("PUSH", push);
        assert_reference_passes(push, &text);
    }
}

#[test]
fn adversary_464_subjectless_external_refusal_beside_when_subject() {
    let text = replaced(
        MODEL,
        "      - name: unlisted\n",
        "      - name: unavailable\n        external: the catalogue is down\n        error: demo.desk.Unavailable\n      - name: unlisted\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.desk.NotPicked\n",
        "  - name: demo.desk.NotPicked\n  - name: demo.desk.Unavailable\n",
    );
    assert_reference_passes("unavailable", &text);
}

// ---- 5. `when_related` through a stored reference (the known gap) --------------------------------

const STORED: &str = include_str!("fixtures/related-guard-stored-reference.yaml");

/// The stored-reference fixture with an external branch after `blocked`: a refusal, or a move.
fn stored_with(branch: &str) -> String {
    let text = replaced(
        STORED,
        "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n",
        &format!("{branch}      - {{name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}}\n"),
    );
    replaced(
        &text,
        "  - {name: demo.tasks.TaskStateConflict,",
        "  - {name: demo.tasks.SchedulerDown, summary: The scheduler is down., fields: []}\n  - {name: demo.tasks.TaskStateConflict,",
    )
}

/// Recast by the correction pass of beyond10x/ess#464 (finding 7): an external witness beside a
/// `when_related` guard reading a stored reference is not arranged, so the scenario is refused
/// naming the guards and the stored reference, never written for the interpreter to fail.
fn assert_stored_reference_refused(result: &Synthesis, id: &str) {
    assert!(!ids(result).contains(&id.to_owned()), "{id} is not written");
    let refused = refusal_of(result, id);
    assert!(
        refused
            .as_ref()
            .is_some_and(|text| text.starts_with("refusal[ESS-SYNTH-003]")
                && text.contains("`subject.blocked_by`")
                && text.contains("`blocker-missing`")
                && text.contains("`blocked`")),
        "{id} is refused naming the guards over the stored reference: {refused:?}\n all: {:#?}",
        refusals(result)
    );
}

#[test]
fn adversary_464_external_refusal_beside_stored_reference_guard_is_not_a_failing_suite() {
    let result = assert_reference_passes(
        "refusal",
        &stored_with(
            "      - {name: scheduler-down, external: the scheduler is down, error: demo.tasks.SchedulerDown}\n",
        ),
    );
    assert_stored_reference_refused(&result, "demo.tasks.CompleteTask/outcome/scheduler-down");
}

#[test]
fn adversary_464_external_move_beside_stored_reference_guard_is_not_a_failing_suite() {
    let result = assert_reference_passes(
        "move",
        &stored_with(
            "      - name: forced\n        external: the scheduler closes it\n        moves: demo.tasks.Task.complete\n        instance: task_id\n        emits: [demo.tasks.TaskCompleted]\n        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}\n",
        ),
    );
    assert_stored_reference_refused(&result, "demo.tasks.CompleteTask/outcome/forced");
}

// ---- 6. an external guard that overlaps the held-state sibling's ---------------------------------

#[test]
fn adversary_464_external_guard_overlapping_held_sibling_is_refused_by_name() {
    let door = include_str!("external_beside_held_guard.rs");
    let start = door.find("const DOOR: &str = r\"").unwrap() + "const DOOR: &str = r\"".len();
    let end = start + door[start..].find("\";\n").unwrap();
    let door = &door[start..end];
    for push in ["Gentle", "Rough"] {
        let text = replaced(
            &door.replace("PUSH", push),
            "        external: the motor stalls\n",
            &format!("        external: the motor stalls\n        when: push == {push}\n"),
        );
        let result = assert_reference_passes(push, &text);
        for id in [
            "demo.doors.Operate/outcome/stalled",
            "demo.doors.Door/transition/jam/by/demo.doors.Operate/stalled",
        ] {
            let refused = refusal_of(&result, id);
            assert!(
                !ids(&result).contains(&id.to_owned())
                    && refused.as_ref().is_some_and(|text| text.contains("`shut`")),
                "{push}: {id} is refused naming `shut`: {refused:?}\n all: {:#?}",
                refusals(&result)
            );
        }
    }
}

// ---- 7. an accepting `when_subject` declared after the external ----------------------------------

#[test]
fn adversary_464_accepting_when_subject_after_external_is_refuted_or_named() {
    let text = replaced(
        MODEL,
        "      - name: accepted\n        moves: demo.desk.Pick.accept\n",
        "      - name: same\n        when_subject:\n          predicate: revision == input.revision\n        moves: demo.desk.Pick.accept\n        instance: pick_id\n        emits: [demo.desk.PickAccepted]\n        payload:\n          demo.desk.PickAccepted: {pick_id: input.pick_id}\n      - name: accepted\n        moves: demo.desk.Pick.accept\n",
    );
    let result = assert_reference_passes("same", &text);
    let id = "demo.desk.CheckPick/outcome/unlisted";
    let refused = refusal_of(&result, id);
    assert!(
        !ids(&result).contains(&id.to_owned())
            && refused
                .as_ref()
                .is_some_and(|text| text.contains("`stale`") && text.contains("`same`")),
        "every row and input is claimed by `stale` or `same`, so {id} is refused naming both: \
         {refused:?}"
    );
}

// ---- 8. the external's own guard reads the field the stored guard compares -----------------------

#[test]
fn adversary_464_external_guard_over_compared_field_is_witnessed() {
    let text = replaced(
        MODEL,
        "        external: the current list does not name the pick\n",
        "        external: the current list does not name the pick\n        when: revision > 5\n",
    );
    let result = assert_reference_passes("revision > 5", &text);
    let id = "demo.desk.CheckPick/outcome/unlisted";
    assert!(
        ids(&result).contains(&id.to_owned()),
        "a pick created at revision 6 and checked at 6 is a witness, so {id} is written: {:?}",
        refusal_of(&result, id)
    );
}

// ---- 9. two externals; a two-field stored guard -------------------------------------------------

#[test]
fn adversary_464_two_externals_beside_when_subject() {
    let text = replaced(
        MODEL,
        "      - name: accepted\n        moves: demo.desk.Pick.accept\n",
        "      - name: expired\n        external: the pick expired upstream\n        moves: demo.desk.Pick.refuse\n        instance: pick_id\n        emits: [demo.desk.PickUnlisted]\n        payload:\n          demo.desk.PickUnlisted: {pick_id: input.pick_id}\n      - name: accepted\n        moves: demo.desk.Pick.accept\n",
    );
    let result = assert_reference_passes("two externals", &text);
    for id in [
        "demo.desk.CheckPick/outcome/unlisted",
        "demo.desk.CheckPick/outcome/expired",
    ] {
        assert!(ids(&result).contains(&id.to_owned()), "{id} is written");
    }
}

#[test]
fn adversary_464_two_field_stored_guard_beside_external() {
    let text = replaced(
        MODEL,
        "      - {name: revision, type: Integer}\n    lifecycle:",
        "      - {name: revision, type: Integer}\n      - {name: owner, type: String}\n    lifecycle:",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n    outcomes:\n      - name: picked\n",
        "      - {name: revision, type: Integer}\n      - {name: owner, type: String}\n    outcomes:\n      - name: picked\n",
    );
    let text = replaced(
        &text,
        "        sets: {revision: input.revision}\n",
        "        sets: {revision: input.revision, owner: input.owner}\n",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n    outcomes:\n      - name: stale\n",
        "      - {name: revision, type: Integer}\n      - {name: owner, type: String}\n    outcomes:\n      - name: stale\n",
    );
    let text = replaced(
        &text,
        "          predicate: revision != input.revision\n",
        "          predicate: {any: [revision != input.revision, owner != input.owner]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n      - {name: state, type: demo.desk.Pick.State}\n",
        "      - {name: revision, type: Integer}\n      - {name: owner, type: String}\n      - {name: state, type: demo.desk.Pick.State}\n",
    );
    let result = assert_reference_passes("two fields", &text);
    assert!(
        ids(&result).contains(&"demo.desk.CheckPick/outcome/unlisted".to_owned()),
        "{:#?}",
        refusals(&result)
    );
}

// ---- 10. an ess/6 `{field, equals}` sibling; 11. an external moving from a later state ----------

#[test]
fn adversary_464_enum_field_equals_sibling_beside_external() {
    let text = replaced(
        MODEL,
        "  - {name: demo.desk.PickId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.desk.PickId, kind: newtype, of: Uuid}\n  - {name: demo.desk.Tier, kind: enum, variants: [Gold, Silver]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n    lifecycle:",
        "      - {name: revision, type: Integer}\n      - {name: tier, type: demo.desk.Tier}\n    lifecycle:",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n    outcomes:\n      - name: picked\n",
        "      - {name: revision, type: Integer}\n      - {name: tier, type: demo.desk.Tier}\n    outcomes:\n      - name: picked\n",
    );
    let text = replaced(
        &text,
        "        sets: {revision: input.revision}\n",
        "        sets: {revision: input.revision, tier: input.tier}\n",
    );
    let text = replaced(
        &text,
        "        when_subject:\n          predicate: revision != input.revision\n",
        "        when_subject: {field: tier, equals: Gold}\n",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n      - {name: state, type: demo.desk.Pick.State}\n",
        "      - {name: revision, type: Integer}\n      - {name: tier, type: demo.desk.Tier}\n      - {name: state, type: demo.desk.Pick.State}\n",
    );
    let result = assert_reference_passes("tier", &text);
    assert!(
        ids(&result).contains(&"demo.desk.CheckPick/outcome/unlisted".to_owned()),
        "a Silver pick is a witness: {:#?}",
        refusals(&result)
    );
}

#[test]
fn adversary_464_external_moving_from_a_later_state_beside_when_subject() {
    let text = replaced(
        MODEL,
        "      states: [Picked, Accepted, Refused]\n      terminal: [Accepted, Refused]\n      transitions:\n        - {name: accept, from: [Picked], to: Accepted}\n        - {name: refuse, from: [Picked], to: Refused}\n",
        "      states: [Picked, Checked, Accepted, Refused]\n      terminal: [Accepted, Refused]\n      transitions:\n        - {name: review, from: [Picked], to: Checked}\n        - {name: accept, from: [Checked], to: Accepted}\n        - {name: refuse, from: [Checked], to: Refused}\n",
    );
    let text = replaced(
        &text,
        "    may: [demo.desk.MakePick, demo.desk.CheckPick]\n",
        "    may: [demo.desk.MakePick, demo.desk.CheckPick, demo.desk.Review]\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "  - name: demo.desk.Review\n    input:\n      - {name: pick_id, type: demo.desk.PickId}\n    outcomes:\n      - name: reviewed\n        moves: demo.desk.Pick.review\n        instance: pick_id\n        emits: [demo.desk.PickReviewed]\n        payload:\n          demo.desk.PickReviewed: {pick_id: input.pick_id}\n      - name: not-picked\n        wrong_state: true\n        error: demo.desk.NotPicked\nevents:\n  - name: demo.desk.PickReviewed\n    fields: [{name: pick_id, type: demo.desk.PickId}]\n",
    );
    let result = assert_reference_passes("checked", &text);
    assert!(
        ids(&result).contains(&"demo.desk.CheckPick/outcome/unlisted".to_owned()),
        "{:#?}",
        refusals(&result)
    );
}

// ---- 12. an external that moves nothing, two of its held states claimed outright ----------------

/// `Operate` refuses an open door (`locked`, no input guard) and shuts an ajar one (`shut`, no
/// input guard); the external `relabelled` moves nothing, so it is taken from every state, and only
/// a `Closed` door leaves it to answer (validation refuses `wrong_state:` beside
/// `when_subject_state:`, so `Closed` answers `relabelled`, not a wrong-state refusal).
const DOOR_RELABEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.doors
summary: A door shut when ajar, refused when open, relabelled by a printer.
types:
  - {name: demo.doors.DoorId, kind: newtype, of: Uuid}
entities:
  - name: demo.doors.Door
    identity: {name: door_id, type: demo.doors.DoorId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Ajar, Closed]
      terminal: [Closed]
      transitions:
        - {name: nudge, from: [Open], to: Ajar}
        - {name: close, from: [Open, Ajar], to: Closed}
actors:
  - name: demo.doors.Keeper
    may: [demo.doors.Install, demo.doors.Nudge, demo.doors.Operate]
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
  - name: demo.doors.Nudge
    input:
      - {name: door_id, type: demo.doors.DoorId}
    outcomes:
      - name: nudged
        moves: demo.doors.Door.nudge
        instance: door_id
        emits: [demo.doors.Nudged]
        payload:
          demo.doors.Nudged: {door_id: input.door_id}
      - name: not-open
        wrong_state: true
        error: demo.doors.NotOpen
  - name: demo.doors.Operate
    input:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: label, type: String}
    outcomes:
      - name: locked
        when_subject_state: Open
        error: demo.doors.Locked
      - name: shut
        when_subject_state: Ajar
        moves: demo.doors.Door.close
        instance: door_id
        emits: [demo.doors.Shut]
        payload:
          demo.doors.Shut: {door_id: input.door_id}
      - name: relabelled
        external: the label printer answers
        updates: demo.doors.Door
        instance: door_id
        sets: {label: input.label}
        emits: [demo.doors.Relabelled]
        payload:
          demo.doors.Relabelled: {door_id: input.door_id}
      - name: refused
        error: demo.doors.Refused
events:
  - name: demo.doors.Installed
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Nudged
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Shut
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Relabelled
    fields: [{name: door_id, type: demo.doors.DoorId}]
errors:
  - name: demo.doors.NotOpen
  - name: demo.doors.Locked
  - name: demo.doors.Refused
views:
  - name: demo.doors.Doors
    source: demo.doors.Door
    consistency: read_your_writes
    fields:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: label, type: String}
      - {name: state, type: demo.doors.Door.State}
";

#[test]
fn adversary_464_moveless_external_beside_unguarded_held_siblings() {
    let result = assert_reference_passes("relabel", DOOR_RELABEL);
    let id = "demo.doors.Operate/outcome/relabelled";
    assert!(
        ids(&result).contains(&id.to_owned()) || refusal_of(&result, id).is_some(),
        "{id} is witnessed or refused by name"
    );
}

// ---- base/head comparison: written only with ADV464_DUMP=<dir> ----------------------------------

/// A `const NAME: &str = r"…";` model of the unit's own test file, by name.
fn unit_model(name: &str) -> String {
    let file = include_str!("external_beside_held_guard.rs");
    let open = format!("const {name}: &str = r\"");
    let start = file.find(&open).unwrap() + open.len();
    let end = start + file[start..].find("\";\n").unwrap();
    file[start..end].to_owned()
}

#[test]
fn adversary_464_dump_scenarios_for_base_comparison() {
    let Ok(dir) = std::env::var("ADV464_DUMP") else {
        return;
    };
    let door = unit_model("DOOR");
    let related = unit_model("RELATED");
    let row_set = unit_model("ROW_SET");
    let mut models: Vec<(String, String)> = vec![
        ("model".into(), MODEL.to_owned()),
        ("related".into(), related),
        ("driver-when-subject".into(), DRIVER_WHEN_SUBJECT.to_owned()),
        ("door-relabel".into(), DOOR_RELABEL.to_owned()),
        (
            "model-gt".into(),
            MODEL.replace("revision != input.revision", "revision > input.revision"),
        ),
    ];
    for push in ["Gentle", "Rough"] {
        models.push((format!("door-{push}"), door.replace("PUSH", push)));
        models.push((
            format!("door-offline-{push}"),
            DOOR_OFFLINE.replace("PUSH", push),
        ));
    }
    for limit in ["0", "1"] {
        models.push((format!("row-set-{limit}"), row_set.replace("LIMIT", limit)));
    }
    let mut out = String::new();
    for (label, text) in models {
        let result = synthesis(&text);
        let statuses = run(&text, &result.suite);
        for id in result.suite.scenarios.keys() {
            let status = statuses
                .get(&id.to_string())
                .map_or_else(|| "none".to_owned(), |(status, _)| format!("{status:?}"));
            let hash = scenario_digest(&result, &id.to_string());
            writeln!(out, "{label}\t{id}\t{status}\t{hash}").unwrap();
        }
        for refusal in &result.refusals {
            let id = refusal
                .scenario
                .as_ref()
                .map_or_else(String::new, ToString::to_string);
            writeln!(out, "{label}\t{id}\tREFUSED\t-").unwrap();
        }
    }
    std::fs::write(std::path::Path::new(&dir).join("scenarios.tsv"), out).unwrap();
}

/// Control for [`adversary_464_subjectless_external_refusal_beside_when_subject`]: the same
/// external refusal on the fixture without its `stale` guard is sent for an arranged pick.
#[test]
fn adversary_464_control_subjectless_external_refusal_without_stored_guard() {
    let text = replaced(
        MODEL,
        "      - name: stale\n        when_subject:\n          predicate: revision != input.revision\n        moves: demo.desk.Pick.refuse\n        instance: pick_id\n        emits: [demo.desk.PickStale]\n        payload:\n          demo.desk.PickStale: {pick_id: input.pick_id}\n",
        "      - name: unavailable\n        external: the catalogue is down\n        error: demo.desk.Unavailable\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.desk.NotPicked\n",
        "  - name: demo.desk.NotPicked\n  - name: demo.desk.Unavailable\n",
    );
    assert_reference_passes("control", &text);
}

// ---- 13. a re-witnessed external that writes a field still writes something new (#161) ----------

/// The `field` the forced `outcome` send of `command` names in `id`, and the one its row was
/// created with by `creator`.
fn written_against_row(
    result: &Synthesis,
    id: &str,
    (command, outcome, identity): (&str, &str, &str),
    (creator, field): (&str, &str),
) -> (Option<ScenarioValue>, ScenarioValue) {
    let found = result
        .suite
        .scenarios
        .iter()
        .find_map(|(key, scenario)| (key.to_string() == id).then_some(scenario))
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(result)));
    let (_, sent) = sends(found, command)
        .into_iter()
        .find(|(at, _)| forced_before(found, *at, command, outcome))
        .unwrap_or_else(|| panic!("{id}: no forced send"));
    let ScenarioValue::Instance { instance } = &sent[identity] else {
        panic!("{id}: the send names an arranged row: {sent:#?}");
    };
    let rows = created_with(found, creator, field);
    (rows.get(instance).cloned(), sent[field].clone())
}

#[test]
fn adversary_464_rewitnessed_held_state_external_writes_a_new_value() {
    let door = unit_model("DOOR");
    let mut same = Vec::new();
    for push in ["Gentle", "Rough"] {
        let text = replaced(
            &door.replace("PUSH", push),
            "      - {name: push, type: demo.doors.Push}\n",
            "      - {name: push, type: demo.doors.Push}\n      - {name: label, type: String}\n",
        );
        let text = replaced(
            &text,
            "        moves: demo.doors.Door.jam\n        instance: door_id\n",
            "        moves: demo.doors.Door.jam\n        instance: door_id\n        sets: {label: input.label}\n",
        );
        let result = assert_reference_passes(push, &text);
        for id in [
            "demo.doors.Operate/outcome/stalled",
            "demo.doors.Door/transition/jam/by/demo.doors.Operate/stalled",
        ] {
            let (held, sent) = written_against_row(
                &result,
                id,
                ("demo.doors.Operate", "stalled", "door_id"),
                ("demo.doors.Install", "label"),
            );
            if held.as_ref() == Some(&sent) {
                same.push(format!(
                    "{push}: {id}: label {sent:?} is what the door already holds"
                ));
            }
        }
    }
    assert_eq!(
        same,
        Vec::<String>::new(),
        "`stalled` writes `label`; a witness sending the value the row holds cannot tell a write \
         from none (beyond10x/ess#161)"
    );
}

#[test]
fn adversary_464_rewitnessed_stored_guard_external_writes_a_new_value() {
    let text = replaced(
        MODEL,
        "      - {name: revision, type: Integer}\n    lifecycle:",
        "      - {name: revision, type: Integer}\n      - {name: note, type: String}\n    lifecycle:",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n    outcomes:\n      - name: picked\n",
        "      - {name: revision, type: Integer}\n      - {name: note, type: String}\n    outcomes:\n      - name: picked\n",
    );
    let text = replaced(
        &text,
        "        sets: {revision: input.revision}\n",
        "        sets: {revision: input.revision, note: input.note}\n",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n    outcomes:\n      - name: stale\n",
        "      - {name: revision, type: Integer}\n      - {name: note, type: String}\n    outcomes:\n      - name: stale\n",
    );
    let text = replaced(
        &text,
        "        moves: demo.desk.Pick.refuse\n        instance: pick_id\n        emits: [demo.desk.PickUnlisted]\n",
        "        moves: demo.desk.Pick.refuse\n        instance: pick_id\n        sets: {note: input.note}\n        emits: [demo.desk.PickUnlisted]\n",
    );
    let text = replaced(
        &text,
        "      - {name: revision, type: Integer}\n      - {name: state, type: demo.desk.Pick.State}\n",
        "      - {name: revision, type: Integer}\n      - {name: note, type: String}\n      - {name: state, type: demo.desk.Pick.State}\n",
    );
    let result = assert_reference_passes("note", &text);
    let mut same = Vec::new();
    for id in [
        "demo.desk.CheckPick/outcome/unlisted",
        "demo.desk.Pick/transition/refuse/by/demo.desk.CheckPick/unlisted",
    ] {
        let (held, sent) = written_against_row(
            &result,
            id,
            ("demo.desk.CheckPick", "unlisted", "pick_id"),
            ("demo.desk.MakePick", "note"),
        );
        if held.as_ref() == Some(&sent) {
            same.push(format!(
                "{id}: note {sent:?} is what the pick already holds"
            ));
        }
    }
    assert_eq!(
        same,
        Vec::<String>::new(),
        "`unlisted` writes `note`; a witness sending the value the row holds cannot tell a write \
         from none (beyond10x/ess#161)"
    );
}

/// The canonical bytes of the one-scenario suite holding `id`, as `<length>:<FNV-1a-like hash>`.
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

// ---- 14. an external witness no sibling claims keeps its base bytes ----------------------------

/// The story: "Suite bytes change ... only for external-branch scenarios a sibling guard claims
/// today", and the commit: "The old witness is computed first and kept wherever no sibling claims
/// it". `repository_model_suites_change_only_where_claimed` leaves out every scenario forcing an
/// external branch of a command that reads rows, claimed or not, and its base table holds none
/// outside the unit's own fixture, so that claim is not compared anywhere. These are external
/// scenarios no sibling claimed at base (bc4884203), each passing the interpreter there, with the
/// digests of their canonical bytes at base: `stalled` sent `Gentle` beside `shut` guarded by
/// `push == Rough`, and `unlisted` beside a `stale` that claims only a stored revision above the
/// input.
#[test]
fn adversary_464_unclaimed_external_witnesses_keep_their_base_bytes() {
    let door = unit_model("DOOR").replace("PUSH", "Rough");
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
    let door_result = assert_reference_passes("door", &door);
    let gt_result = assert_reference_passes("gt", &gt);
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
