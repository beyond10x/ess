//! A non-creating command's effect on related records (beyond10x/ess#229, the effect half;
//! `story:related-record-effects`, ess/22): an `affects:` entry may declare
//! `moves: <Entity>.<transition>`, with the `instances:` set-move semantics.
//!
//! `DeactivateUser` moves its user to `Inactive` and ends every live session of that user. Only
//! that entry takes `Session.end`, so the entry is the transition's cause. Below ess/22 the entry's
//! move is refused naming ess/22, with nothing refused beside it.

use ess_domain::command::RawOutcome;
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

const MOVE: &str = "            moves: demo.users.Session.end\n";

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).unwrap_or_else(|error| {
        panic!("the model parses: {error}\n{body}");
    });
    Specification::assemble([(ess_domain::system::Source::new("users.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
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

fn deactivated(spec: &Specification) -> &ess_domain::command::Outcome {
    spec.commands()
        .values()
        .find(|command| command.name.to_string() == "demo.users.DeactivateUser")
        .and_then(|command| {
            command
                .outcomes
                .iter()
                .find(|outcome| outcome.name.as_str() == "deactivated")
        })
        .expect("the branch is declared")
}

#[test]
fn issue_229_a_move_inside_affects_is_admitted_from_ess_22() {
    let spec = admitted(MODEL);
    let outcome = deactivated(&spec);
    let [affect] = outcome.set_effects.affects.as_slice() else {
        panic!("one secondary effect: {:#?}", outcome.set_effects.affects)
    };
    assert_eq!(affect.entity.to_string(), "demo.users.Session");
    assert_eq!(affect.moves.as_deref(), Some("end"));
    assert!(affect.sets.contains_key("revoked"));
}

#[test]
fn issue_229_the_affects_move_is_the_cause_of_its_transition() {
    // The model above is admitted with no other outcome taking `Session.end`; without the entry's
    // move the same model is refused for a transition nothing takes, so the admission is the
    // entry being counted, not the check being absent.
    admitted(MODEL);
    let errors = refused(&edited(MOVE, ""));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("end")),
        "a transition nothing takes is refused: {errors}"
    );
}

#[test]
fn issue_229_a_moving_entry_needs_no_sets() {
    let spec = admitted(&edited(
        "            sets: {revoked: true}\n      - {name: not-active",
        "      - {name: not-active",
    ));
    let [affect] = deactivated(&spec).set_effects.affects.as_slice() else {
        panic!("one secondary effect")
    };
    assert_eq!(affect.moves.as_deref(), Some("end"));
    assert_eq!(affect.sets.len(), 0, "{affect:#?}");
}

#[test]
fn issue_229_a_move_inside_affects_is_refused_below_ess_22_naming_it() {
    for format in ["ess/16", "ess/20", "ess/21"] {
        let below = MODEL.replacen("format: ess/22", &format!("format: {format}"), 1);
        let errors = refused(&below);
        assert_eq!(
            errors.len(),
            1,
            "{format}: one refusal, and no cascade (`empty_declaration`, a transition nothing \
             takes) beside it:\n{errors}"
        );
        let error = &errors.as_slice()[0];
        let text = error.to_string();
        assert!(text.contains("affects[0].moves"), "{format}: {text}");
        assert!(text.contains("ess/22"), "{format}: {text}");
        // The code every earlier format gave a move inside `affects:` is kept.
        assert_eq!(
            error.code,
            ValidationCode::UnsupportedConstruct,
            "{format}: {text}"
        );
    }
}

#[test]
fn issue_229_the_move_must_be_a_transition_of_the_affected_entity() {
    let errors = refused(&edited(
        MOVE,
        "            moves: demo.users.User.deactivate\n",
    ));
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
                && error.to_string().contains("affects[0].moves")
        }),
        "{errors}"
    );
}

#[test]
fn issue_229_an_undeclared_transition_is_refused() {
    let errors = refused(&edited(
        MOVE,
        "            moves: demo.users.Session.vanish\n",
    ));
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::UndeclaredReference
                && error.to_string().contains("vanish")
        }),
        "{errors}"
    );
}

#[test]
fn issue_229_the_written_form_round_trips() {
    let spec = admitted(MODEL);
    let outcome = deactivated(&spec);
    let written =
        serde_yaml::to_string(&RawOutcome::from(outcome.clone())).expect("an outcome serializes");
    assert!(
        written.contains("moves: demo.users.Session.end"),
        "{written}"
    );
    let read: RawOutcome =
        serde_yaml::from_str(&written).unwrap_or_else(|error| panic!("{error}\n{written}"));
    let back = ess_domain::command::Outcome::try_from(read)
        .unwrap_or_else(|errors| panic!("{errors}\n{written}"));
    assert_eq!(&back, outcome, "{written}");
}

#[test]
fn issue_229_two_moving_entries_over_one_entity_are_refused_at_the_second() {
    let errors = refused(&edited(
        "            sets: {revoked: true}\n",
        "            sets: {revoked: true}\n          - entity: demo.users.Session\n            where: user_id == subject.user_id\n            moves: demo.users.Session.expire\n",
    ));
    assert_eq!(errors.len(), 1, "{errors}");
    let error = &errors.as_slice()[0];
    assert_eq!(
        error.code,
        ValidationCode::ConflictingDeclaration,
        "{error}"
    );
    let text = error.to_string();
    assert!(text.contains("affects[1].moves"), "{text}");
    assert!(text.contains("one transition per entity"), "{text}");
}

#[test]
fn issue_229_a_moving_and_a_setting_entry_over_one_entity_are_admitted() {
    admitted(&edited(
        "            sets: {revoked: true}\n",
        "          - entity: demo.users.Session\n            where: user_id == subject.user_id\n            sets: {revoked: true}\n",
    ));
}
