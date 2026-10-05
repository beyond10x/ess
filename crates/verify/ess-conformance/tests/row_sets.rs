//! Row sets and filtered related reads on the native interpreter (`docs/design/filtered-related-reads.md`,
//! "Decisive acceptance"; `docs/design/expression-family-source22.md`, final decision 13;
//! beyond10x/ess#228, #299).
//!
//! `tests/fixtures/filtered-related-reads.yaml`: `Retry` refuses when more than one earlier attempt
//! of the worker and batch matches, starts with delay 0 when none does, and otherwise copies the one
//! match's delay and note; `CheckLimit` holds when every matching attempt waited at most the limit;
//! `Close` refuses while another attempt of the subject's own worker and batch is recorded, after
//! the addressed row's existence and held state. The selected value is 17; the decoys, one per
//! conjunct, hold 31 and 47.
//!
//! `tests/fixtures/unique-within-scope.yaml`: `BindIdentity` refuses claims another user of the
//! tenant carries; an Optional `org_id` key is guarded by its `defined(org_id)` conjunct.

mod support_row_sets;

use ess_conformance::target::ConformanceTarget;
use ess_primitives::node::Node;
use support_row_sets::*;

const SELECTED: i64 = 17;
const DECOY_WORKER: i64 = 31;
const DECOY_BATCH: i64 = 47;

/// The selected attempt between its two decoys, each refuting one conjunct.
fn arranged(target: &dyn ConformanceTarget, note: Option<&str>) -> String {
    record(target, "w2", "b1", DECOY_WORKER, Some("decoy-worker"));
    let selected = record(target, "w1", "b1", SELECTED, note);
    record(target, "w1", "b2", DECOY_BATCH, Some("decoy-batch"));
    selected
}

fn answer(
    target: &dyn ConformanceTarget,
    request: ess_conformance::target::SemanticCommandRequest,
) -> ess_conformance::target::SemanticCommandResult {
    target
        .execute_command(request)
        .unwrap_or_else(|error| panic!("answered: {error}"))
}

#[test]
fn one_match_reads_value() {
    let target = target(READS);
    arranged(&target, Some("selected"));
    let before = attempts(&target).len();
    let retried = answer(&target, retry("w1", "b1"));
    assert_eq!(outcome(&retried), "retried");
    let event = &retried.direct_events[0].payload;
    assert_eq!(event["delay"], number(SELECTED), "{event:?}");
    assert_eq!(event["note"], text("selected"), "{event:?}");
    let rows = attempts(&target);
    assert_eq!(rows.len(), before + 1);
    let created = event["attempt_id"].as_text().expect("an identity");
    let row = rows
        .iter()
        .find(|row| row["attempt_id"].as_text() == Some(created))
        .expect("the retried attempt is stored");
    assert_eq!(row["delay"], number(SELECTED));
    assert_eq!(row["note"], text("selected"));
}

#[test]
fn zero_match_branch() {
    let target = target(READS);
    record(&target, "w2", "b1", DECOY_WORKER, None);
    record(&target, "w1", "b2", DECOY_BATCH, None);
    let started = answer(&target, retry("w1", "b1"));
    assert_eq!(outcome(&started), "started");
    let created = started.direct_events[0].payload["attempt_id"]
        .as_text()
        .expect("an identity")
        .to_owned();
    let row = attempts(&target)
        .into_iter()
        .find(|row| row["attempt_id"].as_text() == Some(created.as_str()))
        .expect("stored");
    assert_eq!(row["delay"], number(0));
}

#[test]
fn many_matches_refuse() {
    let target = target(READS);
    arranged(&target, None);
    record(&target, "w1", "b1", 23, None);
    let before = attempts(&target);
    let ambiguous = answer(&target, retry("w1", "b1"));
    assert_eq!(outcome(&ambiguous), "ambiguous");
    assert_eq!(ambiguous.direct_events.len(), 0);
    assert_eq!(attempts(&target), before, "a refusal changes no row");
}

#[test]
fn unguarded_zero_and_many_do_not_mutate() {
    // `Retry` with its guards taken away: the read alone, on whatever the store holds.
    let (head, tail) = READS.split_once("      - name: ambiguous").unwrap();
    let (_, rest) = tail.split_once("      - name: retried").unwrap();
    let unguarded = format!("{head}      - name: retried{rest}");
    for matching in [0, 2] {
        let target = target(&unguarded);
        record(&target, "w2", "b1", DECOY_WORKER, None);
        for delay in [SELECTED, 23].into_iter().take(matching) {
            record(&target, "w1", "b1", delay, None);
        }
        let before = attempts(&target);
        let answered = target.execute_command(retry("w1", "b1"));
        assert!(
            answered
                .as_ref()
                .map_or(true, |result| result.outcome.is_none()),
            "{matching} matches supply no value: {answered:?}"
        );
        assert_eq!(
            attempts(&target),
            before,
            "{matching} matches change no row"
        );
    }
}

#[test]
fn empty_forall_is_true() {
    let target = target(READS);
    record(&target, "w2", "b1", 900, None);
    record(&target, "w1", "b2", 900, None);
    let held = answer(&target, check_limit("w1", "b1", 5));
    assert_eq!(outcome(&held), "within-limit");
}

#[test]
fn forall_has_counterexample() {
    let target = target(READS);
    record(&target, "w1", "b1", 3, None);
    record(&target, "w1", "b1", 5, None);
    assert_eq!(
        outcome(&answer(&target, check_limit("w1", "b1", 5))),
        "within-limit"
    );
    record(&target, "w1", "b1", 9, None);
    let refused = answer(&target, check_limit("w1", "b1", 5));
    assert_eq!(outcome(&refused), "over-limit");
}

#[test]
fn count_boundaries() {
    for (matching, expected) in [
        (0, "started"),
        (1, "retried"),
        (2, "ambiguous"),
        (3, "ambiguous"),
    ] {
        let target = target(READS);
        record(&target, "w2", "b1", DECOY_WORKER, None);
        for _ in 0..matching {
            record(&target, "w1", "b1", SELECTED, None);
        }
        assert_eq!(
            outcome(&answer(&target, retry("w1", "b1"))),
            expected,
            "{matching} matching rows"
        );
    }
}

#[test]
fn optional_selected_value() {
    let target = target(READS);
    arranged(&target, None);
    let retried = answer(&target, retry("w1", "b1"));
    assert_eq!(outcome(&retried), "retried");
    let event = &retried.direct_events[0].payload;
    assert_eq!(event["delay"], number(SELECTED));
    assert!(
        event.get("note").is_none_or(|note| *note == Node::Null),
        "an absent note is copied as absent: {event:?}"
    );
}

#[test]
fn subject_borrowing_and_precedence() {
    // An identity no attempt carries: the addressed row's existence answers first.
    let target = target(READS);
    let other = record(&target, "w1", "b1", 1, None);
    let unknown = answer(&target, close("00000000-0000-4000-8000-000000000000"));
    assert_eq!(outcome(&unknown), "unknown-attempt");
    // Alone with its worker and batch: closed.
    assert_eq!(outcome(&answer(&target, close(&other))), "closed");
    // Closed already, beside a sibling: the held state answers before the row set.
    record(&target, "w1", "b1", 2, None);
    assert_eq!(outcome(&answer(&target, close(&other))), "already-closed");
    // Open, beside a sibling of its own worker and batch: the subject's own row is among the rows
    // its selector reads, so two match.
    let crowded = record(&target, "w3", "b3", 1, None);
    record(&target, "w3", "b3", 2, None);
    assert_eq!(outcome(&answer(&target, close(&crowded))), "crowded");
}

#[test]
fn held_state_answers_before_a_row_set_guarded_accepting_branch() {
    // `Close` taken only while at most one attempt of its worker and batch is recorded: the
    // addressed row's held state still answers before that test is read.
    const FROM: &str = "      - name: closed\n        moves: demo.jobs.Attempt.close\n";
    let guarded = READS.replacen(
        FROM,
        "      - name: closed
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == subject.worker_id, batch_id == subject.batch_id]}
          count: {lte: 1}
        moves: demo.jobs.Attempt.close
",
        1,
    );
    assert_ne!(guarded, READS, "the fixture's accepting `Close` branch");
    let target = target(&guarded);
    let attempt = record(&target, "w1", "b1", 1, None);
    assert_eq!(outcome(&answer(&target, close(&attempt))), "closed");
    // Closed, beside two siblings: the row set would answer `crowded`, the held state first.
    record(&target, "w1", "b1", 2, None);
    record(&target, "w1", "b1", 3, None);
    assert_eq!(outcome(&answer(&target, close(&attempt))), "already-closed");
    // Open among them: the row set answers.
    let open = record(&target, "w1", "b1", 4, None);
    assert_eq!(outcome(&answer(&target, close(&open))), "crowded");
}

#[test]
fn reads_pre_outcome_snapshot() {
    // The creation would match the selector it is selected by: it is not among the rows read.
    let target = target(READS);
    assert_eq!(outcome(&answer(&target, retry("w1", "b1"))), "started");
    let retried = answer(&target, retry("w1", "b1"));
    assert_eq!(outcome(&retried), "retried");
    assert_eq!(retried.direct_events[0].payload["delay"], number(0));
    // Now two attempts of the worker and batch are stored.
    assert_eq!(outcome(&answer(&target, retry("w1", "b1"))), "ambiguous");
}

#[test]
fn reverse_row_order_and_change_only_decoy_keep_the_value() {
    let mut copied = Vec::new();
    for (first, second) in [
        ((31, "w2", "b1"), (47, "w1", "b2")),
        ((470, "w1", "b2"), (310, "w2", "b1")),
    ] {
        for selected_first in [true, false] {
            let target = target(READS);
            if selected_first {
                record(&target, "w1", "b1", SELECTED, None);
            }
            record(&target, first.1, first.2, first.0, None);
            record(&target, second.1, second.2, second.0, None);
            if !selected_first {
                record(&target, "w1", "b1", SELECTED, None);
            }
            let retried = answer(&target, retry("w1", "b1"));
            copied.push(retried.direct_events[0].payload["delay"].clone());
        }
    }
    assert!(
        copied.iter().all(|value| *value == number(SELECTED)),
        "{copied:?}"
    );
}

// ---- beyond10x/ess#228: uniqueness within a scope ------------------------------------------------

const ALICE: &str = "11111111-1111-4111-8111-111111111111";
const BOB: &str = "22222222-2222-4222-8222-222222222222";
const CAROL: &str = "33333333-3333-4333-8333-333333333333";
const DAN: &str = "44444444-4444-4444-8444-444444444444";
const ERIN: &str = "55555555-5555-4555-8555-555555555555";

#[test]
fn unique_absent_accepts() {
    let target = target(UNIQUE);
    assert_eq!(
        outcome(&answer(&target, bind(ALICE, "t1", "s1", "o1"))),
        "bound"
    );
    assert_eq!(identities(&target).len(), 1);
}

#[test]
fn unique_present_refuses() {
    let target = target(UNIQUE);
    answer(&target, bind(ALICE, "t1", "s1", "o1"));
    let refused = answer(&target, bind(BOB, "t1", "s1", "o1"));
    assert_eq!(outcome(&refused), "claims-taken");
    assert_eq!(identities(&target).len(), 1, "a refusal stores no row");
}

#[test]
fn second_create_refuses() {
    // The row is created by the very command whose uniqueness it then decides.
    let target = target(UNIQUE);
    for (user, expected) in [
        (ALICE, "bound"),
        (BOB, "claims-taken"),
        (CAROL, "claims-taken"),
    ] {
        assert_eq!(
            outcome(&answer(&target, bind(user, "t1", "s1", "o1"))),
            expected
        );
    }
}

#[test]
fn decoy_per_conjunct_accepts() {
    let target = target(UNIQUE);
    answer(&target, bind(ALICE, "t2", "s1", "o1"));
    answer(&target, bind(BOB, "t1", "s2", "o1"));
    answer(&target, bind(CAROL, "t1", "s1", "o2"));
    assert_eq!(
        outcome(&answer(&target, import(DAN, "t1", "s1"))),
        "imported"
    );
    assert_eq!(
        outcome(&answer(&target, bind(ERIN, "t1", "s1", "o1"))),
        "bound"
    );
}

#[test]
fn composes_with_existing_instance() {
    // The command's own identity is checked before the row set: the second bind of one user with
    // the claims it already carries is `already-bound`, not `claims-taken`.
    let target = target(UNIQUE);
    answer(&target, bind(ALICE, "t1", "s1", "o1"));
    assert_eq!(
        outcome(&answer(&target, bind(ALICE, "t1", "s1", "o1"))),
        "already-bound"
    );
}

#[test]
fn optional_key_needs_defined() {
    // With `defined(org_id)`, a row holding no `org_id` is decidedly not a match.
    let target = target(UNIQUE);
    answer(&target, import(DAN, "t1", "s1"));
    assert_eq!(
        outcome(&answer(&target, bind(ERIN, "t1", "s1", "o1"))),
        "bound"
    );
    // Without it, that row's membership is unknown, and so is the decision: no branch is taken.
    let undefined = UNIQUE.replace("defined(org_id), ", "");
    let unguarded = support_row_sets::target(&undefined);
    answer(&unguarded, import(DAN, "t1", "s1"));
    let before = identities(&unguarded);
    let answered = unguarded.execute_command(bind(ERIN, "t1", "s1", "o1"));
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "an unknown member decides nothing: {answered:?}"
    );
    assert_eq!(identities(&unguarded), before);
}
