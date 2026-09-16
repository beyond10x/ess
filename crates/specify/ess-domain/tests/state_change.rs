//! `when_state_changes:` — a branch saying the state it arrives at is, or is not, the one held.
//!
//! One rule about a *place* and one about the *lifecycle*. The key is admitted only beside
//! `moves:`, because nothing else declares a state to arrive at; and the side of the transition's
//! own `from` set it picks must be inhabited, because an empty side is a branch no held state
//! reaches.
//!
//! The model under test is the adopter shape the construct exists for: a push carries a state, the
//! reducer re-reports the state a row already holds, and the specification has to tell the first
//! report of `Added` from every later one.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("state.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// A member that the backend re-reports on every push, and the command that reads those pushes.
///
/// Every transition runs from every state, which is the adopter's own lifecycle and the reason a
/// `from` set cannot stand in for this construct: `join` is legal from `Added`, so the lifecycle
/// alone cannot say that a push re-reporting `Added` is not the join.
fn model(joined_guard: &str, refreshed: &str) -> String {
    format!(
        "format: ess/4
system: demo
version: v1
domain: demo.room
types:
  - name: demo.room.MemberId
    kind: newtype
    of: Uuid
  - name: demo.room.Reported
    kind: enum
    variants: [Pending, Added, Gone]
entities:
  - name: demo.room.Member
    identity:
      name: member_id
      type: demo.room.MemberId
    fields:
      - name: on_hold
        type: Boolean
    lifecycle:
      initial: Pending
      states: [Pending, Added, Gone]
      terminal: [Gone]
      transitions:
        - name: join
          from: [Pending, Added, Gone]
          to: Added
        - name: leave
          from: [Pending, Added, Gone]
          to: Gone
events:
  - name: demo.room.MemberUpdated
    fields:
      - name: member_id
        type: demo.room.MemberId
commands:
  - name: demo.room.Report
    input:
      - name: member_id
        type: demo.room.MemberId
      - name: state
        type: demo.room.Reported
      - name: on_hold
        type: Boolean
    outcomes:
      - name: joined
        when: state == Added
{joined_guard}        moves: demo.room.Member.join
        instance: member_id
        emits:
          - demo.room.MemberUpdated
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
      - name: gone
        when: state == Gone
        when_state_changes: true
        moves: demo.room.Member.leave
        instance: member_id
        emits:
          - demo.room.MemberUpdated
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
{refreshed}"
    )
}

const FIRST_REPORT: &str = "        when_state_changes: true\n";

/// The branch every later report falls to: fields only, no move, no guard.
const REFRESHED: &str = "      - name: refreshed
        updates: demo.room.Member
        instance: member_id
        sets:
          on_hold: input.on_hold
        emits:
          - demo.room.MemberUpdated
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
";

#[test]
fn the_first_report_of_a_state_is_a_declarable_branch() {
    let parsed = spec(&model(FIRST_REPORT, REFRESHED)).expect("the adopter shape is admitted");
    let command = parsed
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.room.Report")
        .expect("the command");
    let joined = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "joined")
        .expect("the branch");
    assert!(
        matches!(
            joined.condition,
            ess_domain::command::OutcomeCondition::StateChange { changes: true, .. }
        ),
        "{:?}",
        joined.condition
    );
    assert!(joined.condition.reads_held_state());
    assert!(!joined.is_unconditional());
    // The input predicate survives beside it: the two conditions are conjunctive, not alternatives.
    assert_eq!(
        joined.condition.predicate().map(ToString::to_string),
        Some("state == Added".to_owned())
    );
    assert_eq!(
        joined.test_strategy().as_str(),
        "construct_input_in_state",
        "a scenario has to establish a held state, not only build an input"
    );
}

#[test]
fn the_construct_round_trips_through_the_document_it_was_written_in() {
    let parsed = spec(&model(FIRST_REPORT, REFRESHED)).unwrap();
    let command = parsed
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.room.Report")
        .unwrap();
    let encoded = serde_yaml::to_string(command).unwrap();
    assert!(encoded.contains("when_state_changes: true"), "{encoded}");
    let raw: ess_domain::command::RawCommandSpec = serde_yaml::from_str(&encoded).unwrap();
    let roundtrip = ess_domain::command::CommandSpec::try_from(raw).unwrap();
    assert_eq!(&roundtrip, command, "the construct survives a round trip");
}

#[test]
fn both_answers_are_writable_and_false_is_not_the_same_document_as_absent() {
    // `false` selects the restatement, which this lifecycle admits because `join` runs from `Added`.
    // Without the default branch the partition is closed by the two answers together.
    let restating = model("        when_state_changes: false\n", REFRESHED);
    spec(&restating).expect("a branch may answer the restatement instead");

    let absent = spec(&model("", REFRESHED)).expect("no held-state condition at all still parses");
    let guarded = spec(&restating).unwrap();
    assert_ne!(
        absent.commands().values().next().unwrap().outcomes,
        guarded.commands().values().next().unwrap().outcomes,
        "`when_state_changes: false` says something the key's absence does not"
    );
}

#[test]
fn a_restated_state_no_longer_matches_the_branch_that_moves_to_it() {
    // The whole point: held `Added` with a push reporting `Added` selects exactly one branch, and
    // it is not `joined`. The joint state/input partition proof is what says so, and it says it by
    // refusing the same model with the default removed.
    spec(&model(FIRST_REPORT, REFRESHED)).expect("with a default, every pair is covered");
    let error = spec(&model(FIRST_REPORT, "")).expect_err("without one, the restatements are open");
    assert!(
        error.contains("select 0 branches"),
        "the uncovered pairs are named: {error}"
    );
    assert!(
        error.contains("held state Added") && error.contains("state = Added"),
        "and the pair that is uncovered is the restatement: {error}"
    );
}

#[test]
fn an_answer_no_held_state_reaches_is_refused_as_unreachable() {
    // `join` narrowed to `from: [Pending]` never restates `Added`, so `false` admits nothing.
    let narrowed = model("        when_state_changes: false\n", REFRESHED).replace(
        "from: [Pending, Added, Gone]\n          to: Added",
        "from: [Pending]\n          to: Added",
    );
    let error = spec(&narrowed).expect_err("a side of the from set that is empty is no branch");
    for required in [
        "when_state_changes: false",
        "unreachable_branch",
        "join",
        "Added",
        "demo.room.Member",
    ] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
    assert!(
        error.contains("default branch"),
        "the hint names the repair:\n{error}"
    );
}

#[test]
fn an_answer_that_never_changes_the_state_is_refused_the_other_way() {
    // The other empty side, on its own model: `again` is a declared self-transition, so `true`
    // admits nothing. It needs a second transition into `Added` or the state is unreachable and the
    // entity is refused before any command is read — which is why this is not a mutation of the
    // model above.
    let error = spec(
        "format: ess/4
system: demo
version: v1
domain: demo.loop
entities:
  - name: demo.loop.Row
    identity: {name: row_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Pending
      states: [Pending, Added, Gone]
      terminal: [Gone]
      transitions:
        - {name: arrive, from: [Pending], to: Added}
        - {name: again, from: [Added], to: Added}
        - {name: leave, from: [Added], to: Gone}
events:
  - name: demo.loop.Seen
    fields: []
commands:
  - name: demo.loop.Push
    input:
      - {name: row_id, type: Uuid}
    outcomes:
      - name: arrived
        when_state_changes: true
        moves: demo.loop.Row.arrive
        instance: row_id
        emits: [demo.loop.Seen]
      - name: restated
        when_state_changes: true
        moves: demo.loop.Row.again
        instance: row_id
        emits: [demo.loop.Seen]
      - name: left
        when_state_changes: true
        moves: demo.loop.Row.leave
        instance: row_id
        emits: [demo.loop.Seen]
",
    )
    .expect_err("a move that only ever restates never changes the state");
    for required in [
        "when_state_changes: true",
        "unreachable_branch",
        "it never changes one",
        "again",
    ] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
    assert!(
        error.contains("drop the key"),
        "the hint names the repair:\n{error}"
    );
}

#[test]
fn the_key_is_refused_where_no_state_is_arrived_at() {
    // `updates:` takes no transition, `creates:` starts at the initial state, and a branch with no
    // subject has nothing resting anywhere. None of the three has an arrival state to be about.
    for (label, replacement, expected) in [
        (
            "updates",
            "        updates: demo.room.Member\n        instance: member_id\n",
            "must take a declared move",
        ),
        (
            "creates",
            "        creates: demo.room.Member\n        instance: member_id\n",
            "requires an existing moves or updates subject",
        ),
        (
            "nothing",
            "",
            "requires an existing moves or updates subject",
        ),
    ] {
        let source = model(FIRST_REPORT, REFRESHED).replace(
            "        moves: demo.room.Member.join\n        instance: member_id\n",
            replacement,
        );
        let error = spec(&source).expect_err("no arrival state is declared here");
        assert!(
            !error.contains("unknown field"),
            "{label}: expected a semantic refusal, not a parse error:\n{error}"
        );
        assert!(
            error.contains("conflicting_declaration") && error.contains(expected),
            "{label}: the refusal names what is missing:\n{error}"
        );
        assert!(
            error.contains("when_state_changes"),
            "{label}: the refusal is located on the key an author would delete:\n{error}"
        );
    }
}

#[test]
fn it_cannot_borrow_another_branchs_condition_authority() {
    for (label, extra, expected) in [
        (
            "a second held-state authority",
            "        when_subject_state: Pending\n",
            "one way",
        ),
        (
            "an external cause",
            "        external: the provider decided\n",
            "external or wrong_state",
        ),
        (
            "wrong-state precedence",
            "        wrong_state: true\n",
            "external or wrong_state",
        ),
    ] {
        let source = model(&format!("{FIRST_REPORT}{extra}"), REFRESHED);
        let error = spec(&source).expect_err("a second authority over one branch");
        assert!(
            error.contains("conflicting_declaration") && error.contains(expected),
            "{label}: {error}"
        );
    }
}

#[test]
fn a_document_older_than_the_construct_refuses_it_by_name() {
    for format in ["ess/1", "ess/2", "ess/3"] {
        let source =
            model(FIRST_REPORT, REFRESHED).replace("format: ess/4", &format!("format: {format}"));
        let error = spec(&source).expect_err("the construct is newer than this format");
        assert!(
            error.contains("state-change outcome guards require specification format ess/4"),
            "{format}: {error}"
        );
    }
}

#[test]
fn a_malformed_answer_is_not_read_as_one() {
    // The key carries an answer, not a state and not a list. Each of these used to be the shape an
    // author reached for, and each has to be refused rather than coerced into a reading.
    for bad in [
        "        when_state_changes: Added\n",
        "        when_state_changes: [Pending]\n",
        "        when_state_changes: \"true\"\n",
        "        when_state_change: true\n",
    ] {
        let error = spec(&model(bad, REFRESHED)).expect_err("this is not an answer");
        assert!(
            error.contains("invalid type")
                || error.contains("unknown field")
                || error.contains("expected"),
            "`{bad}`: {error}"
        );
    }
}
