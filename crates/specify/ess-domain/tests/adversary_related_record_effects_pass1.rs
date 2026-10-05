//! Adversary pass 1 against `moves:` inside `affects:` (beyond10x/ess#229, ess/22): what is
//! refused, and that a refusal of the move comes alone (`story:related-record-effects`: "the
//! spurious ESS-COMMAND-007 cascade beside the `affects[].moves` refusal").

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

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
  - name: demo.users.SessionEnded
    fields: [{name: session_id, type: demo.users.SessionId}]
errors: [{name: demo.users.NotActive}, {name: demo.users.NotLive}]
actors:
  - name: demo.users.Admin
    may: [demo.users.AddUser, demo.users.StartSession, demo.users.ExpireSession, demo.users.EndSession, demo.users.DeactivateUser]
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
  - name: demo.users.EndSession
    input: [{name: session_id, type: demo.users.SessionId}]
    outcomes:
      - name: ended
        moves: demo.users.Session.end
        instance: session_id
        emits: [demo.users.SessionEnded]
        payload: {demo.users.SessionEnded: {session_id: input.session_id}}
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

const MOVE: &str = "            moves: demo.users.Session.end\n";

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the model parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("users.yaml"), raw)])
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn edited(from: &str, to: &str) -> String {
    assert!(MODEL.contains(from), "the model holds:\n{from}");
    MODEL.replacen(from, to, 1)
}

#[test]
fn adv229_control_the_model_is_admitted() {
    // `Session.end` is also taken by `EndSession`, so no refusal below is the entry's transition
    // left without a cause.
    assemble(MODEL).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn adv229_a_move_of_another_entity_is_refused_alone() {
    let errors = refused(&edited(
        MOVE,
        "            moves: demo.users.User.deactivate\n",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration),
        "{errors}"
    );
    assert_eq!(
        errors.len(),
        1,
        "one refusal of the misnamed move, and no cascade beside it:\n{errors}"
    );
}

#[test]
fn adv229_a_move_naming_no_entity_is_refused_alone() {
    let errors = refused(&edited(MOVE, "            moves: end\n"));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::MissingDeclaration),
        "{errors}"
    );
    assert_eq!(
        errors.len(),
        1,
        "one refusal of the unqualified move, and no cascade beside it:\n{errors}"
    );
}

#[test]
fn adv229_sets_cannot_write_the_lifecycle_state_beside_a_move() {
    let errors = refused(&edited(
        "            sets: {revoked: true}\n",
        "            sets: {revoked: true, state: Expired}\n",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("state")),
        "{errors}"
    );
}
