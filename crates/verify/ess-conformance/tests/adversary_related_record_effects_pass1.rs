//! Adversary pass 1 against `moves:` inside `affects:` (beyond10x/ess#229, ess/22,
//! `story:related-record-effects`).
//!
//! Each case states the claim it attacks. The models are the unit's own `demo.users` model with
//! one edit each.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Store, Undetermined,
};
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/22
system: demo
version: v1
domain: demo.users
types:
  - {name: demo.users.UserId, kind: newtype, of: String}
  - {name: demo.users.SessionId, kind: newtype, of: String}
entities:
  - name: demo.users.User
    identity: {name: user_id, type: demo.users.UserId}
    fields: [{name: team, type: String}]
    lifecycle:
      initial: Active
      states: [Active, Inactive]
      terminal: [Inactive]
      transitions: [{name: deactivate, from: [Active], to: Inactive}]
  - name: demo.users.Session
    identity: {name: session_id, type: demo.users.SessionId}
    fields:
      - {name: user_id, type: demo.users.UserId}
      - {name: revoked, type: Boolean}
    lifecycle:
      initial: Live
      states: [Live, Ended, Expired]
      terminal: [Ended, Expired]
      transitions:
        - {name: end, from: [Live], to: Ended}
        - {name: expire, from: [Live], to: Expired}
events:
  - name: demo.users.UserAdded
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.UserDeactivated
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.SessionStarted
    fields: [{name: session_id, type: demo.users.SessionId}]
  - name: demo.users.SessionExpired
    fields: [{name: session_id, type: demo.users.SessionId}]
errors: [{name: demo.users.NotActive}, {name: demo.users.NotLive}]
actors:
  - name: demo.users.Admin
    may: [demo.users.AddUser, demo.users.StartSession, demo.users.ExpireSession, demo.users.DeactivateUser]
commands:
  - name: demo.users.AddUser
    input: [{name: team, type: String}]
    outcomes:
      - name: added
        creates: demo.users.User
        instance: user_id
        emits: [demo.users.UserAdded]
        payload: {demo.users.UserAdded: {user_id: {generated: true}}}
        sets: {team: input.team}
  - name: demo.users.StartSession
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: started
        creates: demo.users.Session
        instance: session_id
        emits: [demo.users.SessionStarted]
        payload: {demo.users.SessionStarted: {session_id: {generated: true}}}
        sets: {user_id: input.user_id, revoked: false}
  - name: demo.users.ExpireSession
    input: [{name: session_id, type: demo.users.SessionId}]
    outcomes:
      - name: expired
        moves: demo.users.Session.expire
        instance: session_id
        emits: [demo.users.SessionExpired]
        payload: {demo.users.SessionExpired: {session_id: input.session_id}}
      - {name: not-live, wrong_state: true, error: demo.users.NotLive}
  - name: demo.users.DeactivateUser
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: deactivated
        moves: demo.users.User.deactivate
        instance: user_id
        emits: [demo.users.UserDeactivated]
        payload: {demo.users.UserDeactivated: {user_id: input.user_id}}
        affects:
          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.end
            sets: {revoked: true}
      - {name: not-active, wrong_state: true, error: demo.users.NotActive}
views:
  - name: demo.users.Users
    source: demo.users.User
    consistency: read_your_writes
    fields: [{name: user_id, type: demo.users.UserId}, {name: team, type: String}, {name: state, type: demo.users.User.State}]
  - name: demo.users.Sessions
    source: demo.users.Session
    consistency: read_your_writes
    fields: [{name: session_id, type: demo.users.SessionId}, {name: user_id, type: demo.users.UserId}, {name: revoked, type: Boolean}, {name: state, type: demo.users.Session.State}]
";

const DEACTIVATED: &str = "demo.users.DeactivateUser/outcome/deactivated";

const ENTRY: &str = "          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.end
            sets: {revoked: true}
";

fn edited(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model holds:\n{from}");
    model.replacen(from, to, 1)
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("users.yaml"), raw)])
}

fn ir_of(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn run(suite: &ConformanceSuite, target_model: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(target_model),
        )
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Every diagnostic `id` produced when `suite` ran against `target_model`'s interpreter.
fn diagnostics(suite: &ConformanceSuite, target_model: EssIr, id: &str) -> String {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(target_model),
        )
        .into_report()
        .scenarios
        .iter()
        .filter(|result| result.scenario.to_string() == id)
        .flat_map(|result| result.diagnostics().cloned().collect::<Vec<_>>())
        .map(|diagnostic| {
            format!(
                "{:?}\n  expected: {:#?}\n  observed: {:#?}",
                diagnostic.code, diagnostic.expected, diagnostic.observed
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Runs the model's own synthesized suite against the model's own interpreter, and answers the
/// status of `id`, or `None` where synthesis wrote no such scenario, with the synthesis refusals
/// and the scenario's diagnostics.
fn self_run(text: &str, id: &str) -> (Option<Status>, Vec<String>) {
    let ir = ir_of(text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let mut refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .collect();
    let statuses = run(&synthesis.suite, ir.clone());
    refusals.push(format!(
        "DIAGNOSTICS:\n{}",
        diagnostics(&synthesis.suite, ir, id)
    ));
    (statuses.get(id).copied(), refusals)
}

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

fn given(event: &str, field: &str, value: &str) -> Generated {
    Generated::Given(BTreeMap::from([(
        GeneratedSlot::new(event.parse().unwrap(), field),
        text(value),
    )]))
}

fn create(
    ir: &EssIr,
    store: &mut Store,
    command: &str,
    input: &BTreeMap<String, Node>,
    generated: &Generated,
) {
    let mut steps = execute_generating(
        ir,
        store,
        &command.parse().unwrap(),
        input,
        &Externals::Withheld,
        generated,
    )
    .unwrap_or_else(|why| panic!("{command}: {why}"));
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    assert!(step.outcome.is_some(), "{command} takes a declared branch");
    *store = step.next;
}

fn add_user(ir: &EssIr, store: &mut Store, id: &str, team: &str) {
    create(
        ir,
        store,
        "demo.users.AddUser",
        &BTreeMap::from([("team".to_owned(), text(team))]),
        &given("demo.users.UserAdded", "user_id", id),
    );
}

fn start_session(ir: &EssIr, store: &mut Store, id: &str, user: Option<&str>) {
    let mut input = BTreeMap::new();
    if let Some(user) = user {
        input.insert("user_id".to_owned(), text(user));
    }
    create(
        ir,
        store,
        "demo.users.StartSession",
        &input,
        &given("demo.users.SessionStarted", "session_id", id),
    );
}

fn state(store: &Store, entity: &str, id: &str) -> String {
    store
        .instance(&entity.parse().unwrap(), id)
        .unwrap_or_else(|| panic!("{entity} {id} is held"))
        .state
        .to_string()
}

// ---- 1. two moving entries choosing the same row ------------------------------------------------

/// Two entries over `Session`, both selecting the user's sessions: one ends them, one expires
/// them. The model is admitted (nothing refuses two moving entries over one entity), so the
/// reference interpreter must determine an answer — any answer — for a store holding one such
/// session. The implementor's note says it answers "cannot undergo its declared effect".
const TWO_MOVES: &str = "          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.end
            sets: {revoked: true}
          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.expire
";

#[test]
fn adv229_two_moving_entries_over_one_row_are_refused_or_determined() {
    let model = edited(MODEL, ENTRY, TWO_MOVES);
    let Ok(spec) = assemble(&model) else {
        return; // refused statically: the claim holds
    };
    let ir = compile(&spec, &SourceMap::new()).expect("the model compiles");
    let mut store = Store::default();
    add_user(&ir, &mut store, "u1", "red");
    start_session(&ir, &mut store, "s1", Some("u1"));
    let answer = execute(
        &ir,
        &store,
        &"demo.users.DeactivateUser".parse().unwrap(),
        &BTreeMap::from([("user_id".to_owned(), text("u1"))]),
        &Externals::Withheld,
    );
    assert!(
        answer.is_ok(),
        "an admitted model leaves the reference interpreter without an answer: {:?}",
        answer.err()
    );
}

#[test]
fn adv229_two_moving_entries_suite_passes_its_own_interpreter_or_is_refused() {
    let model = edited(MODEL, ENTRY, TWO_MOVES);
    if assemble(&model).is_err() {
        return;
    }
    let (status, refusals) = self_run(&model, DEACTIVATED);
    if status.is_none() {
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.contains("DeactivateUser")),
            "no scenario and no refusal for the branch: {refusals:#?}"
        );
        return;
    }
    assert_eq!(
        status,
        Some(Status::Passed),
        "the suite synthesized for an admitted model fails the model's own interpreter: \
         {refusals:#?}"
    );
}

/// One entry ends the user's sessions, a second over the same rows only marks them revoked. The
/// interpreter applies both; the synthesized scenario must agree with it.
const MOVE_THEN_SETS: &str = "          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.end
          - entity: demo.users.Session
            where: user_id == subject.user_id
            sets: {revoked: true}
";

#[test]
fn adv229_a_moving_and_a_setting_entry_over_one_row_pass_their_own_interpreter() {
    let model = edited(MODEL, ENTRY, MOVE_THEN_SETS);
    if assemble(&model).is_err() {
        return;
    }
    let (status, refusals) = self_run(&model, DEACTIVATED);
    if status.is_none() {
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.contains("DeactivateUser")),
            "no scenario and no refusal for the branch: {refusals:#?}"
        );
        return;
    }
    assert_eq!(
        status,
        Some(Status::Passed),
        "the suite synthesized for an admitted model fails the model's own interpreter: \
         {refusals:#?}"
    );
}

// ---- 2. a move whose `from` holds its own arrival state ------------------------------------------

/// `Session.reset` starts from `Live` and `Locked` and arrives in `Live`. `DeactivateUser` resets
/// the user's sessions. A target that never resets anything must fail the branch's scenario.
const RESETTING: &str = r"format: ess/22
system: demo
version: v1
domain: demo.users
types:
  - {name: demo.users.UserId, kind: newtype, of: String}
  - {name: demo.users.SessionId, kind: newtype, of: String}
entities:
  - name: demo.users.User
    identity: {name: user_id, type: demo.users.UserId}
    fields: [{name: team, type: String}]
    lifecycle:
      initial: Active
      states: [Active, Inactive]
      terminal: [Inactive]
      transitions: [{name: deactivate, from: [Active], to: Inactive}]
  - name: demo.users.Session
    identity: {name: session_id, type: demo.users.SessionId}
    fields:
      - {name: user_id, type: demo.users.UserId}
    lifecycle:
      initial: Live
      states: [Live, Locked, Ended]
      terminal: [Ended]
      transitions:
        - {name: lock, from: [Live], to: Locked}
        - {name: reset, from: [Live, Locked], to: Live}
        - {name: end, from: [Live, Locked], to: Ended}
events:
  - name: demo.users.UserAdded
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.UserDeactivated
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.SessionStarted
    fields: [{name: session_id, type: demo.users.SessionId}]
  - name: demo.users.SessionChanged
    fields: [{name: session_id, type: demo.users.SessionId}]
errors: [{name: demo.users.NotActive}, {name: demo.users.Unchangeable}]
actors:
  - name: demo.users.Admin
    may: [demo.users.AddUser, demo.users.StartSession, demo.users.LockSession, demo.users.ResetSession, demo.users.EndSession, demo.users.DeactivateUser]
commands:
  - name: demo.users.AddUser
    input: [{name: team, type: String}]
    outcomes:
      - name: added
        creates: demo.users.User
        instance: user_id
        emits: [demo.users.UserAdded]
        payload: {demo.users.UserAdded: {user_id: {generated: true}}}
        sets: {team: input.team}
  - name: demo.users.StartSession
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: started
        creates: demo.users.Session
        instance: session_id
        emits: [demo.users.SessionStarted]
        payload: {demo.users.SessionStarted: {session_id: {generated: true}}}
        sets: {user_id: input.user_id}
  - name: demo.users.LockSession
    input: [{name: session_id, type: demo.users.SessionId}]
    outcomes:
      - name: locked
        moves: demo.users.Session.lock
        instance: session_id
        emits: [demo.users.SessionChanged]
        payload: {demo.users.SessionChanged: {session_id: input.session_id}}
      - {name: unchangeable, wrong_state: true, error: demo.users.Unchangeable}
  - name: demo.users.ResetSession
    input: [{name: session_id, type: demo.users.SessionId}]
    outcomes:
      - name: reset
        moves: demo.users.Session.reset
        instance: session_id
        emits: [demo.users.SessionChanged]
        payload: {demo.users.SessionChanged: {session_id: input.session_id}}
      - {name: unchangeable, wrong_state: true, error: demo.users.Unchangeable}
  - name: demo.users.EndSession
    input: [{name: session_id, type: demo.users.SessionId}]
    outcomes:
      - name: ended
        moves: demo.users.Session.end
        instance: session_id
        emits: [demo.users.SessionChanged]
        payload: {demo.users.SessionChanged: {session_id: input.session_id}}
      - {name: unchangeable, wrong_state: true, error: demo.users.Unchangeable}
  - name: demo.users.DeactivateUser
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: deactivated
        moves: demo.users.User.deactivate
        instance: user_id
        emits: [demo.users.UserDeactivated]
        payload: {demo.users.UserDeactivated: {user_id: input.user_id}}
        affects:
          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.reset
      - {name: not-active, wrong_state: true, error: demo.users.NotActive}
views:
  - name: demo.users.Users
    source: demo.users.User
    consistency: read_your_writes
    fields: [{name: user_id, type: demo.users.UserId}, {name: team, type: String}, {name: state, type: demo.users.User.State}]
  - name: demo.users.Sessions
    source: demo.users.Session
    consistency: read_your_writes
    fields: [{name: session_id, type: demo.users.SessionId}, {name: user_id, type: demo.users.UserId}, {name: state, type: demo.users.Session.State}]
";

const RESET_ENTRY: &str = "        affects:
          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.reset
";

#[test]
fn adv229_a_target_skipping_a_move_whose_from_holds_its_arrival_fails_the_branch() {
    let ir = ir_of(RESETTING);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    // The model's own interpreter passes the branch: the suite is sound.
    let own = run(&synthesis.suite, ir);
    assert_eq!(own.get(DEACTIVATED), Some(&Status::Passed), "{own:#?}");
    // The same model with the effect left out: a target that deactivates and resets nothing.
    let skipping = ir_of(&edited(RESETTING, RESET_ENTRY, ""));
    let statuses = run(&synthesis.suite, skipping);
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == DEACTIVATED)
        .map(|(_, scenario)| format!("{:#?}", scenario.steps))
        .unwrap_or_default();
    assert_eq!(
        statuses.get(DEACTIVATED),
        Some(&Status::Failed),
        "a target that never takes the related move passes the branch's scenario; it sends \
         `LockSession` {} times:\n{scenario}",
        scenario.matches("demo.users.LockSession").count()
    );
}

// ---- 3. an Optional filter field absent on a row ---------------------------------------------------

/// `Session.user_id` is Optional; a session that names no user rests `Live` beside one that names
/// the subject. Deactivating the user ends the one that names it and leaves the other.
#[test]
fn adv229_a_row_without_the_optional_filter_field_is_not_selected() {
    let model = MODEL
        .replacen(
            "      - {name: user_id, type: demo.users.UserId}\n      - {name: revoked",
            "      - {name: user_id, type: Optional<demo.users.UserId>}\n      - {name: revoked",
            1,
        )
        .replacen(
            "    input: [{name: user_id, type: demo.users.UserId}]\n    outcomes:\n      - name: started",
            "    input: [{name: user_id, type: Optional<demo.users.UserId>}]\n    outcomes:\n      - name: started",
            1,
        )
        .replacen(
            "{name: user_id, type: demo.users.UserId}, {name: revoked, type: Boolean}",
            "{name: user_id, type: Optional<demo.users.UserId>}, {name: revoked, type: Boolean}",
            1,
        );
    assert_ne!(model, MODEL);
    let Ok(spec) = assemble(&model) else {
        return; // a filter over an Optional field refused statically
    };
    let ir = compile(&spec, &SourceMap::new()).expect("the model compiles");
    let mut store = Store::default();
    add_user(&ir, &mut store, "u1", "red");
    start_session(&ir, &mut store, "mine", Some("u1"));
    start_session(&ir, &mut store, "nobody", None);
    let answer = execute(
        &ir,
        &store,
        &"demo.users.DeactivateUser".parse().unwrap(),
        &BTreeMap::from([("user_id".to_owned(), text("u1"))]),
        &Externals::Withheld,
    );
    let mut steps = match answer {
        Ok(steps) => steps,
        Err(Undetermined::Undecidable { outcome, guard }) => {
            panic!("an absent Optional field leaves the filter undecidable: {outcome}: {guard}")
        }
        Err(other) => panic!("{other}"),
    };
    let next = steps.remove(0).next;
    assert_eq!(state(&next, "demo.users.Session", "mine"), "Ended");
    assert_eq!(state(&next, "demo.users.Session", "nobody"), "Live");
}

// ---- 4. an entry over the subject's own entity ------------------------------------------------------

/// `DeactivateUser` deactivates its user and suspends every other active user of the same team,
/// flagging each. The subject matches the filter and is not among the rows. The model's own
/// suite must pass the model's own interpreter.
const SUSPENDING: &str = r"format: ess/22
system: demo
version: v1
domain: demo.users
types:
  - {name: demo.users.UserId, kind: newtype, of: String}
entities:
  - name: demo.users.User
    identity: {name: user_id, type: demo.users.UserId}
    fields:
      - {name: team, type: String}
      - {name: flagged, type: Boolean}
    lifecycle:
      initial: Active
      states: [Active, Suspended, Inactive]
      terminal: [Inactive]
      transitions:
        - {name: suspend, from: [Active], to: Suspended}
        - {name: deactivate, from: [Active, Suspended], to: Inactive}
events:
  - name: demo.users.UserAdded
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.UserDeactivated
    fields: [{name: user_id, type: demo.users.UserId}]
errors: [{name: demo.users.NotActive}]
actors:
  - name: demo.users.Admin
    may: [demo.users.AddUser, demo.users.DeactivateUser]
commands:
  - name: demo.users.AddUser
    input: [{name: team, type: String}]
    outcomes:
      - name: added
        creates: demo.users.User
        instance: user_id
        emits: [demo.users.UserAdded]
        payload: {demo.users.UserAdded: {user_id: {generated: true}}}
        sets: {team: input.team, flagged: false}
  - name: demo.users.DeactivateUser
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: deactivated
        moves: demo.users.User.deactivate
        instance: user_id
        emits: [demo.users.UserDeactivated]
        payload: {demo.users.UserDeactivated: {user_id: input.user_id}}
        affects:
          - entity: demo.users.User
            where: team == subject.team
            moves: demo.users.User.suspend
            sets: {flagged: true}
      - {name: not-active, wrong_state: true, error: demo.users.NotActive}
views:
  - name: demo.users.Users
    source: demo.users.User
    consistency: read_your_writes
    fields: [{name: user_id, type: demo.users.UserId}, {name: team, type: String}, {name: flagged, type: Boolean}, {name: state, type: demo.users.User.State}]
";

#[test]
fn adv229_the_interpreter_never_moves_the_subject_with_its_own_entry() {
    let ir = ir_of(SUSPENDING);
    let mut store = Store::default();
    add_user(&ir, &mut store, "me", "red");
    add_user(&ir, &mut store, "mate", "red");
    add_user(&ir, &mut store, "other", "blue");
    let mut steps = execute(
        &ir,
        &store,
        &"demo.users.DeactivateUser".parse().unwrap(),
        &BTreeMap::from([("user_id".to_owned(), text("me"))]),
        &Externals::Withheld,
    )
    .unwrap_or_else(|why| panic!("{why}"));
    let next = steps.remove(0).next;
    assert_eq!(state(&next, "demo.users.User", "me"), "Inactive");
    assert_eq!(state(&next, "demo.users.User", "mate"), "Suspended");
    assert_eq!(state(&next, "demo.users.User", "other"), "Active");
}

#[test]
fn adv229_an_entry_over_the_subjects_entity_passes_its_own_interpreter() {
    let (status, refusals) = self_run(SUSPENDING, "demo.users.DeactivateUser/outcome/deactivated");
    assert_eq!(
        status,
        Some(Status::Passed),
        "the branch's synthesized scenario fails the model's own interpreter; refusals: \
         {refusals:#?}"
    );
}

// ---- origin probes: the same attacks with no `moves:` inside `affects:` (ess/16 behaviour) -------
//
// Red here means the defect does not depend on the unit's construct.

#[test]
fn adv229_origin_probe_optional_filter_field_absent_without_a_move() {
    let model = MODEL
        .replacen("            moves: demo.users.Session.end\n", "", 1)
        .replacen(
            "      - {name: user_id, type: demo.users.UserId}\n      - {name: revoked",
            "      - {name: user_id, type: Optional<demo.users.UserId>}\n      - {name: revoked",
            1,
        )
        .replacen(
            "    input: [{name: user_id, type: demo.users.UserId}]\n    outcomes:\n      - name: started",
            "    input: [{name: user_id, type: Optional<demo.users.UserId>}]\n    outcomes:\n      - name: started",
            1,
        )
        .replacen(
            "{name: user_id, type: demo.users.UserId}, {name: revoked, type: Boolean}",
            "{name: user_id, type: Optional<demo.users.UserId>}, {name: revoked, type: Boolean}",
            1,
        )
        // `Session.end` needs a cause once the entry no longer takes it.
        .replacen(
            "      - {name: not-live, wrong_state: true, error: demo.users.NotLive}\n",
            "      - {name: not-live, wrong_state: true, error: demo.users.NotLive}\n  - name: demo.users.EndSession\n    input: [{name: session_id, type: demo.users.SessionId}]\n    outcomes:\n      - name: ended\n        moves: demo.users.Session.end\n        instance: session_id\n        emits: [demo.users.SessionExpired]\n        payload: {demo.users.SessionExpired: {session_id: input.session_id}}\n      - {name: not-live, wrong_state: true, error: demo.users.NotLive}\n",
            1,
        )
        .replacen(
            "demo.users.ExpireSession, demo.users.DeactivateUser]",
            "demo.users.ExpireSession, demo.users.EndSession, demo.users.DeactivateUser]",
            1,
        );
    let spec = assemble(&model).unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).expect("the model compiles");
    let mut store = Store::default();
    add_user(&ir, &mut store, "u1", "red");
    start_session(&ir, &mut store, "mine", Some("u1"));
    start_session(&ir, &mut store, "nobody", None);
    let answer = execute(
        &ir,
        &store,
        &"demo.users.DeactivateUser".parse().unwrap(),
        &BTreeMap::from([("user_id".to_owned(), text("u1"))]),
        &Externals::Withheld,
    );
    assert!(answer.is_ok(), "{:?}", answer.err());
}

#[test]
fn adv229_origin_probe_two_setting_entries_over_one_row_pass_their_own_interpreter() {
    let model = edited(
        MODEL,
        ENTRY,
        "          - entity: demo.users.Session
            where: user_id == subject.user_id
            sets: {revoked: true}
          - entity: demo.users.Session
            where: user_id == subject.user_id
            sets: {archived: true}
",
    )
    .replacen(
        "      - {name: revoked, type: Boolean}\n",
        "      - {name: revoked, type: Boolean}\n      - {name: archived, type: Boolean}\n",
        1,
    )
    .replacen(
        "sets: {user_id: input.user_id, revoked: false}",
        "sets: {user_id: input.user_id, revoked: false, archived: false}",
        1,
    )
    .replacen(
        "{name: revoked, type: Boolean}, {name: state, type: demo.users.Session.State}",
        "{name: revoked, type: Boolean}, {name: archived, type: Boolean}, {name: state, type: demo.users.Session.State}",
        1,
    )
    .replacen(
        "      - {name: not-live, wrong_state: true, error: demo.users.NotLive}\n",
        "      - {name: not-live, wrong_state: true, error: demo.users.NotLive}\n  - name: demo.users.EndSession\n    input: [{name: session_id, type: demo.users.SessionId}]\n    outcomes:\n      - name: ended\n        moves: demo.users.Session.end\n        instance: session_id\n        emits: [demo.users.SessionExpired]\n        payload: {demo.users.SessionExpired: {session_id: input.session_id}}\n      - {name: not-live, wrong_state: true, error: demo.users.NotLive}\n",
        1,
    )
    .replacen(
        "demo.users.ExpireSession, demo.users.DeactivateUser]",
        "demo.users.ExpireSession, demo.users.EndSession, demo.users.DeactivateUser]",
        1,
    );
    assemble(&model).unwrap_or_else(|errors| panic!("{errors}"));
    let (status, refusals) = self_run(&model, DEACTIVATED);
    if status.is_none() {
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.contains("DeactivateUser")),
            "no scenario and no refusal for the branch: {refusals:#?}"
        );
        return;
    }
    assert_eq!(status, Some(Status::Passed), "{refusals:#?}");
}

#[test]
fn adv229_origin_probe_a_setting_entry_over_the_subjects_entity_passes_its_own_interpreter() {
    let model = SUSPENDING
        .replacen("            moves: demo.users.User.suspend\n", "", 1)
        .replacen(
            "        - {name: suspend, from: [Active], to: Suspended}\n",
            "",
            1,
        )
        .replacen(
            "      states: [Active, Suspended, Inactive]",
            "      states: [Active, Inactive]",
            1,
        )
        .replacen(
            "from: [Active, Suspended], to: Inactive",
            "from: [Active], to: Inactive",
            1,
        );
    assemble(&model).unwrap_or_else(|errors| panic!("{errors}"));
    let (status, refusals) = self_run(&model, "demo.users.DeactivateUser/outcome/deactivated");
    assert_eq!(status, Some(Status::Passed), "{refusals:#?}");
}
