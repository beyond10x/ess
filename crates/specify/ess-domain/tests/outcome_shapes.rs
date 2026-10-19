//! Outcome shapes beyond `ess/14` (`docs/design/outcome-shapes.md`, beyond10x/ess#144, #145,
//! #150, #151, #152).
//!
//! Five constructs, each admitted under `ess/15` and refused below it with
//! `unsupported_format_version`, each with the rules the design page states for it. The model is
//! the neutral shape the issues describe: a call announced by an external plane, answered, and
//! removed at the end of its lifecycle.

use ess_domain::command::{Effect, OutcomeCondition, RawOutcome};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("calls.yaml"), raw)])
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

fn assert_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

/// The call model, with `extra` spliced into the command list, `header` into the system header and
/// `format` as the source format.
fn calls(format: u32, header: &str, outcomes_of_answer: &str, extra_commands: &str) -> String {
    format!(
        "format: ess/{format}
system: example
version: v1
{header}domain: example.call
types:
  - {{name: example.call.CallId, kind: newtype, of: Uuid}}
  - {{name: example.call.UserId, kind: newtype, of: Uuid}}
entities:
  - name: example.call.Call
    identity: {{name: call_id, type: example.call.CallId}}
    fields:
      - {{name: caller, type: String}}
    lifecycle:
      initial: Dialing
      states: [Dialing, Ringing, Connected, Ended]
      terminal: [Ended]
      transitions:
        - {{name: ring, from: [Dialing], to: Ringing}}
        - {{name: answer, from: [Ringing], to: Connected}}
        - {{name: hang-up, from: [Dialing, Ringing, Connected], to: Ended}}
  - name: example.call.User
    identity: {{name: user_id, type: example.call.UserId}}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
events:
  - name: example.call.CallPlaced
    fields: [{{name: call_id, type: example.call.CallId}}]
  - name: example.call.CallOffered
    fields: [{{name: call_id, type: example.call.CallId}}]
  - name: example.call.CallRang
    fields: [{{name: call_id, type: example.call.CallId}}]
  - name: example.call.CallAnswered
    fields: [{{name: call_id, type: example.call.CallId}}]
  - name: example.call.CallHungUp
    fields: [{{name: call_id, type: example.call.CallId}}]
  - name: example.call.SessionOpened
    fields: [{{name: user_id, type: example.call.UserId}}]
errors:
  - name: example.call.CallNotFound
  - name: example.call.CallStateConflict
actors:
  - name: example.call.Agent
    may: [example.call.PlaceCall, example.call.RingCall, example.call.AnswerCall,
          example.call.HangUp, example.call.OpenSession]
commands:
  - name: example.call.PlaceCall
    input:
      - {{name: call_id, type: example.call.CallId}}
    outcomes:
      - name: placed
        creates: example.call.Call
        instance: call_id
        emits: [example.call.CallPlaced]
        payload:
          example.call.CallPlaced: {{call_id: input.call_id}}
  - name: example.call.RingCall
    input:
      - {{name: call_id, type: example.call.CallId}}
    outcomes:
      - name: rang
        moves: example.call.Call.ring
        instance: call_id
        emits: [example.call.CallRang]
        payload:
          example.call.CallRang: {{call_id: input.call_id}}
  - name: example.call.AnswerCall
    input:
      - {{name: call_id, type: example.call.CallId}}
    outcomes:
      - name: answered
        moves: example.call.Call.answer
        instance: call_id
        emits: [example.call.CallAnswered]
        payload:
          example.call.CallAnswered: {{call_id: input.call_id}}
{outcomes_of_answer}  - name: example.call.HangUp
    input:
      - {{name: call_id, type: example.call.CallId}}
    outcomes:
      - name: hung-up
        moves: example.call.Call.hang-up
        instance: call_id
        emits: [example.call.CallHungUp]
        payload:
          example.call.CallHungUp: {{call_id: input.call_id}}
  - name: example.call.OpenSession
    input:
      - {{name: user_id, type: example.call.UserId}}
    outcomes:
      - name: opened
        creates: example.call.User
        instance: user_id
        emits: [example.call.SessionOpened]
        payload:
          example.call.SessionOpened: {{user_id: input.user_id}}
{extra_commands}"
    )
}

fn answer_outcome<'s>(
    spec: &'s Specification,
    command: &str,
    name: &str,
) -> &'s ess_domain::command::Outcome {
    spec.commands()
        .get(&command.parse().unwrap())
        .unwrap_or_else(|| panic!("{command} is declared"))
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("{command}/{name} is declared"))
}

// ---- #145: `unknown_instance:` ----------------------------------------------------------------

const UNKNOWN: &str =
    "      - {name: no-such-call, unknown_instance: true, error: example.call.CallNotFound}
      - {name: already-ended, wrong_state: true, refuses: false}
";

#[test]
fn an_unknown_instance_has_its_own_outcome_beside_an_accepting_wrong_state() {
    let spec = admitted(&calls(15, "", UNKNOWN, ""));
    let outcome = answer_outcome(&spec, "example.call.AnswerCall", "no-such-call");
    assert_eq!(outcome.condition, OutcomeCondition::UnknownInstance);
    assert!(outcome.refuses);
    assert_eq!(
        outcome.error.as_ref().map(ToString::to_string).as_deref(),
        Some("example.call.CallNotFound")
    );
    let raw = RawOutcome::from(outcome.clone());
    assert!(raw.unknown_instance, "the marker is written back");
    assert_eq!(raw.refuses, None, "a refusing marker writes no `refuses:`");
}

#[test]
fn an_unknown_instance_may_be_an_accepted_no_op() {
    let spec = admitted(&calls(
        15,
        "",
        "      - {name: no-such-call, unknown_instance: true, refuses: false}
      - {name: already-ended, wrong_state: true, error: example.call.CallStateConflict}
",
        "",
    ));
    let outcome = answer_outcome(&spec, "example.call.AnswerCall", "no-such-call");
    assert!(!outcome.refuses);
    assert_eq!(RawOutcome::from(outcome.clone()).refuses, Some(false));
}

#[test]
fn unknown_instance_is_refused_below_ess_15() {
    let errors = refused(&calls(14, "", UNKNOWN, ""));
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "unknown_instance",
    );
}

#[test]
fn unknown_instance_names_its_answer_and_is_declared_once() {
    let errors = refused(&calls(
        15,
        "",
        "      - {name: no-such-call, unknown_instance: true}
",
        "",
    ));
    assert_code(&errors, ValidationCode::MissingDeclaration, "no-such-call");

    let errors = refused(&calls(
        15,
        "",
        "      - {name: no-such-call, unknown_instance: true, error: example.call.CallNotFound}
      - {name: no-call-at-all, unknown_instance: true, error: example.call.CallNotFound}
",
        "",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "unknown_instance",
    );

    let errors = refused(&calls(
        15,
        "",
        "      - {name: no-such-call, unknown_instance: true, refuses: false, error: example.call.CallNotFound}
",
        "",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "no-such-call",
    );
}

#[test]
fn unknown_instance_is_one_condition_and_changes_nothing() {
    for shape in [
        "{name: no-such-call, unknown_instance: true, when: call_id == call_id, error: example.call.CallNotFound}",
        "{name: no-such-call, unknown_instance: true, wrong_state: true, error: example.call.CallNotFound}",
        "{name: no-such-call, unknown_instance: true, external: the plane forgot it, error: example.call.CallNotFound}",
    ] {
        let raw = ess_domain::spec::RawSpecFile::parse(&calls(15, "", &format!("      - {shape}\n"), ""))
            .unwrap();
        let errors = Specification::assemble([(ess_domain::system::Source::new("calls.yaml"), raw)])
            .expect_err(shape);
        assert_code(&errors, ValidationCode::ConflictingDeclaration, "unknown_instance");
    }
    let errors = refused(&calls(
        15,
        "",
        "      - {name: no-such-call, unknown_instance: true, refuses: false, updates: example.call.Call, instance: call_id}
",
        "",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "unknown_instance",
    );
}

#[test]
fn unknown_instance_needs_a_branch_acting_on_an_input_named_instance() {
    let errors = refused(&calls(
        15,
        "",
        "",
        "  - name: example.call.Touch
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - name: offered
        creates: example.call.Call
        instance: call_id
        emits: [example.call.CallOffered]
        payload:
          example.call.CallOffered: {call_id: input.call_id}
      - {name: no-such-call, unknown_instance: true, error: example.call.CallNotFound}
",
    ));
    assert_code(&errors, ValidationCode::UnreachableBranch, "no-such-call");
}

/// beyond10x/ess#535: the issue's read-only `Ring` — a `when_subject:` refusal and a `preserves:`
/// branch addressing the seat its input names — beside the commands that make and close a seat.
const READ_ONLY: &str =
    include_str!("../../ess-compiler/tests/fixtures/read-only-unknown-instance.yaml");

#[test]
fn a_read_only_command_declares_an_unknown_instance_answer() {
    let spec = admitted(READ_ONLY);
    let outcome = answer_outcome(&spec, "demo.desk.Ring", "no-seat");
    assert_eq!(outcome.condition, OutcomeCondition::UnknownInstance);
    assert!(outcome.refuses);
    assert_eq!(
        outcome.error.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.desk.NoSeat")
    );
}

#[test]
fn a_command_naming_no_input_instance_still_refuses_unknown_instance() {
    // No branch addresses an instance from input: an external refusal and an accepted no-op.
    let errors = refused(&READ_ONLY.replace(
        "      - name: closed
        when_subject: {predicate: state == Closed}
        error: demo.desk.NoSeat
      - name: rung
        preserves: demo.desk.Seat
        instance: user_id
",
        "      - {name: closed, external: the desk is closed, error: demo.desk.NoSeat}
      - {name: rung, accepts: nothing}
",
    ));
    assert_code(&errors, ValidationCode::UnreachableBranch, "no-seat");
}

#[test]
fn a_creating_unknown_instance_beside_only_a_preserving_branch_is_not_a_create_or_update() {
    // The create-or-update pairing stays on `moves:` and `updates:`: a creation for an unknown
    // identity beside a branch that only preserves the existing row is refused, as before.
    let errors = refused(&READ_ONLY.replace(
        "      - name: no-seat
        unknown_instance: true
        error: demo.desk.NoSeat
",
        "      - name: no-seat
        unknown_instance: true
        creates: demo.desk.Seat
        instance: user_id
        emits: [demo.desk.SeatTaken]
        payload:
          demo.desk.SeatTaken: {user_id: input.user_id}
",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "no-seat");
}

// ---- #151: `deletes:` -----------------------------------------------------------------------

const DELETES: &str = "  - name: example.call.EndCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - name: ended
        deletes: example.call.Call
        instance: call_id
        when_subject_state: Connected
        emits: [example.call.CallHungUp]
        payload:
          example.call.CallHungUp: {call_id: input.call_id}
      - {name: not-connected, error: example.call.CallStateConflict}
";

#[test]
fn an_outcome_deletes_its_subject() {
    let spec = admitted(&calls(15, "", "", DELETES));
    let outcome = answer_outcome(&spec, "example.call.EndCall", "ended");
    let subject = outcome.subject.as_ref().expect("a subject");
    assert_eq!(subject.effect, Effect::Deletes);
    assert_eq!(subject.instance, "call_id");
    assert_eq!(
        subject.surface(),
        ess_domain::command::InstanceSurface::CommandInput
    );
    let raw = RawOutcome::from(outcome.clone());
    assert_eq!(
        raw.deletes.as_ref().map(ToString::to_string).as_deref(),
        Some("example.call.Call")
    );
}

#[test]
fn a_deleting_outcome_may_emit_nothing() {
    admitted(&calls(
        15,
        "",
        "",
        "  - name: example.call.EndCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - {name: ended, deletes: example.call.Call, instance: call_id}
",
    ));
}

#[test]
fn deletes_is_refused_below_ess_15() {
    assert_code(
        &refused(&calls(14, "", "", DELETES)),
        ValidationCode::UnsupportedFormatVersion,
        "deletes",
    );
}

#[test]
fn a_deleting_outcome_sets_nothing_and_does_one_thing() {
    let errors = refused(&calls(
        15,
        "",
        "",
        "  - name: example.call.EndCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - {name: ended, deletes: example.call.Call, instance: call_id, sets: {caller: gone}}
",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "deletes");

    let errors = refused(&calls(
        15,
        "",
        "",
        "  - name: example.call.EndCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - {name: ended, deletes: example.call.Call, moves: example.call.Call.hang-up, instance: call_id}
",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "deletes");

    let errors = refused(&calls(
        15,
        "",
        "",
        "  - name: example.call.EndCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - {name: ended, deletes: example.call.Call, instance: call_id, error: example.call.CallStateConflict}
",
    ));
    assert_code(&errors, ValidationCode::RefusalMutatedState, "ended");
}

// ---- #150: `into:` --------------------------------------------------------------------------

const OFFER: &str = "  - name: example.call.OfferCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - name: offered
        creates: example.call.Call
        instance: call_id
        into: Ringing
        emits: [example.call.CallOffered]
        payload:
          example.call.CallOffered: {call_id: input.call_id}
";

#[test]
fn a_creation_lands_in_a_declared_state() {
    let spec = admitted(&calls(15, "", "", OFFER));
    let outcome = answer_outcome(&spec, "example.call.OfferCall", "offered");
    let subject = outcome.subject.as_ref().expect("a subject");
    assert_eq!(subject.effect, Effect::Creates);
    assert_eq!(
        subject.into.as_ref().map(ToString::to_string).as_deref(),
        Some("Ringing")
    );
    let raw = RawOutcome::from(outcome.clone());
    assert_eq!(
        raw.into.as_ref().map(ToString::to_string).as_deref(),
        Some("Ringing")
    );
    // Omitted, creation still lands in `initial`, and nothing is written back.
    let placed = answer_outcome(&spec, "example.call.PlaceCall", "placed");
    assert_eq!(placed.subject.as_ref().unwrap().into, None);
    assert_eq!(RawOutcome::from(placed.clone()).into, None);
}

#[test]
fn a_creation_may_land_in_a_terminal_state() {
    admitted(&calls(
        15,
        "",
        "",
        &OFFER.replace("into: Ringing", "into: Ended"),
    ));
}

#[test]
fn into_is_refused_below_ess_15() {
    assert_code(
        &refused(&calls(14, "", "", OFFER)),
        ValidationCode::UnsupportedFormatVersion,
        "into",
    );
}

#[test]
fn into_names_a_declared_state_and_sits_only_beside_creates() {
    assert_code(
        &refused(&calls(
            15,
            "",
            "",
            &OFFER.replace("into: Ringing", "into: Queued"),
        )),
        ValidationCode::UnknownState,
        "Queued",
    );
    let errors = refused(&calls(
        15,
        "",
        "",
        "  - name: example.call.OfferCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - {name: offered, updates: example.call.Call, instance: call_id, into: Ringing, emits: [example.call.CallOffered], payload: {example.call.CallOffered: {call_id: input.call_id}}}
",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "into");
}

// ---- #144: `accepts: nothing` ---------------------------------------------------------------

const TOUCH: &str = "domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Touch]}
commands:
  - name: demo.orders.Touch
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: accepted
        summary: The system accepts the request and publishes nothing.
";

fn touch(format: u32, accepted_extra: &str) -> String {
    format!("format: ess/{format}\nsystem: demo\nversion: v1\n{TOUCH}{accepted_extra}")
}

#[test]
fn the_issue_repro_is_refused_until_it_says_it_accepts_nothing() {
    assert_code(
        &refused(&touch(15, "")),
        ValidationCode::EmptyChange,
        "accepted",
    );
    let spec = admitted(&touch(15, "        accepts: nothing\n"));
    let outcome = answer_outcome(&spec, "demo.orders.Touch", "accepted");
    assert!(outcome.accepts_nothing);
    assert_eq!(outcome.condition, OutcomeCondition::Otherwise);
    assert!(RawOutcome::from(outcome.clone()).accepts.is_some());
}

#[test]
fn accepts_nothing_is_refused_below_ess_15() {
    assert_code(
        &refused(&touch(14, "        accepts: nothing\n")),
        ValidationCode::UnsupportedFormatVersion,
        "accepts",
    );
}

#[test]
fn accepts_nothing_is_admitted_under_a_when() {
    admitted(
        "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
events:
  - {name: demo.orders.Touched, fields: [{name: order_id, type: demo.orders.OrderId}]}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Touch]}
commands:
  - name: demo.orders.Touch
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: flag, type: Boolean}
    outcomes:
      - {name: acknowledged, when: flag == true, accepts: nothing}
      - name: touched
        emits: [demo.orders.Touched]
        payload:
          demo.orders.Touched: {order_id: input.order_id}
",
    );
}

#[test]
fn accepts_nothing_carries_nothing_else() {
    for extra in [
        "        accepts: nothing\n        error: demo.orders.Gone\n",
        "        accepts: nothing\n        wrong_state: true\n",
        "        accepts: nothing\n        external: the clerk walked away\n",
    ] {
        let body = touch(15, extra).replace(
            "commands:",
            "errors:\n  - name: demo.orders.Gone\ncommands:",
        );
        let errors = refused(&body);
        assert_code(&errors, ValidationCode::ConflictingDeclaration, "accepts");
    }
    assert!(
        ess_domain::spec::RawSpecFile::parse(&touch(15, "        accepts: everything\n")).is_err(),
        "`accepts:` takes the one word `nothing`"
    );
}

// ---- #152: `preconditions:` -----------------------------------------------------------------

const SESSION: &str = "preconditions:
  - command: example.call.OpenSession
    as: example.call.Agent
    input: {user_id: 00000000-0000-4000-8000-000000000152}
";

#[test]
fn a_system_declares_ambient_preconditions() {
    let spec = admitted(&calls(15, SESSION, "", ""));
    let preconditions = &spec.system().preconditions;
    assert_eq!(preconditions.len(), 1);
    assert_eq!(
        preconditions[0].command.to_string(),
        "example.call.OpenSession"
    );
    assert_eq!(
        preconditions[0]
            .actor
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("example.call.Agent")
    );
    assert!(preconditions[0].input.contains_key("user_id"));
}

#[test]
fn preconditions_are_refused_below_ess_15() {
    assert_code(
        &refused(&calls(14, SESSION, "", "")),
        ValidationCode::UnsupportedFormatVersion,
        "preconditions",
    );
}

#[test]
fn a_precondition_invokes_a_declared_command_as_a_permitted_actor_with_its_input() {
    assert_code(
        &refused(&calls(
            15,
            &SESSION.replace("OpenSession", "OpenDoor"),
            "",
            "",
        )),
        ValidationCode::UndeclaredReference,
        "example.call.OpenDoor",
    );
    assert_code(
        &refused(&calls(
            15,
            &SESSION.replace("as: example.call.Agent", "as: example.call.Nobody"),
            "",
            "",
        )),
        ValidationCode::UndeclaredReference,
        "example.call.Nobody",
    );
    assert_code(
        &refused(&calls(
            15,
            &SESSION.replace("user_id:", "session_id:"),
            "",
            "",
        )),
        ValidationCode::UndeclaredReference,
        "session_id",
    );
    assert_code(
        &refused(&calls(
            15,
            "preconditions:\n  - {command: example.call.OpenSession, as: example.call.Agent}\n",
            "",
            "",
        )),
        ValidationCode::MissingDeclaration,
        "user_id",
    );
    assert_code(
        &refused(&calls(
            15,
            &SESSION.replace("00000000-0000-4000-8000-000000000152", "not-a-uuid"),
            "",
            "",
        )),
        ValidationCode::TypeMismatch,
        "user_id",
    );
}

#[test]
fn a_precondition_must_select_exactly_one_branch_that_reports_no_error() {
    let guarded = |input: &str| {
        calls(
            15,
            &format!(
                "preconditions:\n  - command: example.call.Login\n    input: {{user_id: 00000000-0000-4000-8000-000000000152{input}}}\n"
            ),
            "",
            "  - name: example.call.Login
    input:
      - {name: user_id, type: example.call.UserId}
      - {name: locked, type: Boolean}
    outcomes:
      - {name: locked-out, when: locked == true, error: example.call.CallStateConflict}
      - {name: resumed, emits: [example.call.CallRang], payload: {example.call.CallRang: {call_id: {generated: true}}}}
",
        )
    };
    admitted(&guarded(", locked: false"));
    assert_code(
        &refused(&guarded(", locked: true")),
        ValidationCode::ConflictingDeclaration,
        "locked-out",
    );
    assert_code(
        &refused(&guarded("")),
        ValidationCode::MissingDeclaration,
        "locked",
    );
}

// ---- create-or-preserve under the one precedence rule (G4) --------------------------------

/// `Ring` with its `unknown_instance:` branch creating the seat it would otherwise refuse: "ensure
/// the seat exists", the creation beside a branch that only preserves the seat, in `format`.
fn create_or_preserve(format: &str) -> String {
    let text = READ_ONLY.replacen("format: ess/24\n", &format!("format: {format}\n"), 1);
    let text = text.replacen(
        "      - name: no-seat\n        unknown_instance: true\n        error: demo.desk.NoSeat\n      - name: closed\n",
        "      - name: no-seat\n        unknown_instance: true\n        creates: demo.desk.Seat\n        instance: user_id\n        emits: [demo.desk.SeatTaken]\n        payload:\n          demo.desk.SeatTaken: {user_id: input.user_id}\n      - name: closed\n",
        1,
    );
    assert!(text.contains("creates: demo.desk.Seat\n        instance: user_id\n        emits: [demo.desk.SeatTaken]\n        payload:\n          demo.desk.SeatTaken: {user_id: input.user_id}\n      - name: closed"));
    text
}

/// Under the one rule (`ess/27`) the creation answers at step 2 for an identity no seat carries,
/// and the preserving branch every stored one (beyond10x/ess#535's residue).
#[test]
fn format_a_admits_a_creating_unknown_instance_beside_a_preserving_branch() {
    let spec = admitted(&create_or_preserve("ess/27"));
    let created = answer_outcome(&spec, "demo.desk.Ring", "no-seat");
    assert_eq!(created.condition, OutcomeCondition::UnknownInstance);
    assert_eq!(
        created.subject.as_ref().map(|subject| &subject.effect),
        Some(&Effect::Creates)
    );
}

/// A create-or-delete toggle is not the construct, in every format.
#[test]
fn a_creating_unknown_instance_beside_a_deleting_branch_stays_refused() {
    let deleting = create_or_preserve("ess/27").replacen(
        "      - name: rung\n        preserves: demo.desk.Seat\n",
        "      - name: rung\n        deletes: demo.desk.Seat\n",
        1,
    );
    assert!(deleting.contains("deletes: demo.desk.Seat"));
    let errors = refused(&deleting);
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "no-seat");
}

/// Below `ess/27` the creation beside a preserving branch keeps its refusal, the hint naming
/// `ess/27`.
#[test]
fn below_format_a_create_or_preserve_keeps_its_refusal_naming_format_a() {
    let errors = refused(&create_or_preserve("ess/26"));
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
                && error.to_string().contains("no-seat")
                && error.to_string().contains("format: ess/27")
        }),
        "{errors}"
    );
}
