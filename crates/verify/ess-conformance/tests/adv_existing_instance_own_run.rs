//! Adversary probes for beyond10x/ess#733: the existence scenario's first call is the creating
//! branch's own run (`existence::own_run`), and the refusal's help asks for `sets:` only where a
//! creator of the related row leaves a compared field unset from its input.
//!
//! Every shape here is a model a user writes: the related row rests two moves away, its state is
//! reached through a command guarded on yet another row, two commands create it, the command has two
//! creating branches, and the related row owns the created one. Each must synthesize the existence
//! scenario with no refusal, and every scenario must pass against the interpreter.
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| {
            let scenario = refusal
                .scenario
                .as_ref()
                .map_or_else(String::new, ToString::to_string);
            format!("{scenario} {}: {}", refusal.cause.code(), refusal.cause)
        })
        .collect()
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn run(model: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// No refusal for `exists`, the scenario is in the suite, and every scenario passes on the
/// interpreter.
fn witnessed(label: &str, text: &str, exists: &str) {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let refused: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.starts_with(exists))
        .collect();
    assert_eq!(refused, Vec::<String>::new(), "{label}: refusals");
    assert!(
        ids(&synthesis).iter().any(|id| id == exists),
        "{label}: no `{exists}` in {:?}",
        ids(&synthesis)
    );
    let statuses = run(model, &synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failing.is_empty(),
        "{label}: not passed against interpreted: {failing:?}"
    );
    assert_eq!(statuses.get(exists), Some(&Status::Passed), "{label}");
}

/// The reproduction of beyond10x/ess#733, as the unit's own test file has it.
const CONFIRM: &str = "format: ess/35
system: example
version: v1
domain: example.review
types:
  - {name: example.review.DecisionId, kind: newtype, of: Uuid}
entities:
  - name: example.review.Decision
    identity: {name: decision_id, type: example.review.DecisionId}
    fields:
      - {name: item, type: String}
    lifecycle:
      initial: Current
      states: [Current, Stale]
      terminal: [Stale]
      transitions:
        - {name: expire, from: [Current], to: Stale}
events:
  - name: example.review.DecisionRecorded
    fields:
      - {name: decision_id, type: example.review.DecisionId}
  - name: example.review.DecisionExpired
    fields:
      - {name: decision_id, type: example.review.DecisionId}
  - name: example.review.DecisionConfirmed
    fields:
      - {name: decision_id, type: example.review.DecisionId}
errors:
  - {name: example.review.AlreadyExists, fields: []}
  - {name: example.review.Missing, fields: []}
  - {name: example.review.OtherItem, fields: []}
  - {name: example.review.NotStale, fields: []}
  - {name: example.review.NotCurrent, fields: []}
commands:
  - name: example.review.RecordDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: item, type: String}
    outcomes:
      - name: recorded
        creates: example.review.Decision
        instance: decision_id
        sets: {item: input.item}
        emits: [example.review.DecisionRecorded]
        payload:
          example.review.DecisionRecorded: {decision_id: input.decision_id}
  - name: example.review.ExpireDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Stale], error: example.review.NotCurrent}
      - name: expired
        moves: example.review.Decision.expire
        instance: decision_id
        emits: [example.review.DecisionExpired]
        payload:
          example.review.DecisionExpired: {decision_id: input.decision_id}
  - name: example.review.ConfirmDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: confirms, type: example.review.DecisionId}
      - {name: item, type: String}
    outcomes:
      - {name: already-exists, existing_instance: true, error: example.review.AlreadyExists}
      - {name: missing, when_related: {via: input.confirms, exists: false}, error: example.review.Missing}
      - {name: other-item, when_related: {via: input.confirms, predicate: item != input.item}, error: example.review.OtherItem}
      - {name: not-stale, when_related: {via: input.confirms, predicate: state != Stale}, error: example.review.NotStale}
      - name: confirmed
        creates: example.review.Decision
        instance: decision_id
        sets: {item: input.item}
        emits: [example.review.DecisionConfirmed]
        payload:
          example.review.DecisionConfirmed: {decision_id: input.decision_id}
views:
  - name: example.review.Decisions
    source: example.review.Decision
    consistency: read_your_writes
    fields:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: item, type: String}
      - {name: state, type: example.review.Decision.State}
";

const CONFIRM_EXISTS: &str = "example.review.ConfirmDecision/outcome/already-exists";

/// The related row rests two moves away: `Current -> Reviewed -> Stale`, each move its own command.
#[test]
fn own_run_reaches_a_related_state_two_moves_away() {
    let text = replaced(
        &replaced(
            &replaced(
                CONFIRM,
                "      states: [Current, Stale]\n      terminal: [Stale]\n      transitions:\n        - {name: expire, from: [Current], to: Stale}\n",
                "      states: [Current, Reviewed, Stale]\n      terminal: [Stale]\n      transitions:\n        - {name: review, from: [Current], to: Reviewed}\n        - {name: expire, from: [Reviewed], to: Stale}\n",
            ),
            "      - {name: wrong-state, when_subject_state: [Stale], error: example.review.NotCurrent}\n",
            "      - {name: wrong-state, when_subject_state: [Current, Stale], error: example.review.NotCurrent}\n",
        ),
        "  - name: example.review.ConfirmDecision\n",
        "  - name: example.review.ReviewDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Reviewed, Stale], error: example.review.NotCurrent}
      - name: reviewed
        moves: example.review.Decision.review
        instance: decision_id
        emits: [example.review.DecisionReviewed]
        payload:
          example.review.DecisionReviewed: {decision_id: input.decision_id}
  - name: example.review.ConfirmDecision
",
    );
    let text = replaced(
        &text,
        "  - name: example.review.DecisionConfirmed\n",
        "  - name: example.review.DecisionReviewed
    fields:
      - {name: decision_id, type: example.review.DecisionId}
  - name: example.review.DecisionConfirmed
",
    );
    witnessed("two moves away", &text, CONFIRM_EXISTS);
}

/// The move that makes the related row `Stale` is itself guarded on a row of another entity, so
/// the creating branch's own run arranges two prior rows of two entities.
#[test]
fn own_run_arranges_prior_rows_of_two_entities() {
    let mut text = replaced(
        CONFIRM,
        "  - {name: example.review.DecisionId, kind: newtype, of: Uuid}\n",
        "  - {name: example.review.DecisionId, kind: newtype, of: Uuid}\n  - {name: example.review.ReviewerId, kind: newtype, of: Uuid}\n",
    );
    text = replaced(
        &text,
        "events:\n",
        "  - name: example.review.Reviewer
    identity: {name: reviewer_id, type: example.review.ReviewerId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: example.review.ReviewerAdded
    fields:
      - {name: reviewer_id, type: example.review.ReviewerId}
",
    );
    text = replaced(
        &text,
        "  - {name: example.review.NotCurrent, fields: []}\n",
        "  - {name: example.review.NotCurrent, fields: []}\n  - {name: example.review.NoReviewer, fields: []}\n",
    );
    text = replaced(
        &text,
        "  - name: example.review.ExpireDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Stale], error: example.review.NotCurrent}
",
        "  - name: example.review.AddReviewer
    input:
      - {name: reviewer_id, type: example.review.ReviewerId}
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: example.review.Reviewer
        instance: reviewer_id
        sets: {label: input.label}
        emits: [example.review.ReviewerAdded]
        payload:
          example.review.ReviewerAdded: {reviewer_id: input.reviewer_id}
  - name: example.review.ExpireDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: reviewer_id, type: example.review.ReviewerId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Stale], error: example.review.NotCurrent}
      - {name: no-reviewer, when_related: {via: input.reviewer_id, exists: false}, error: example.review.NoReviewer}
",
    );
    text.push_str(
        "  - name: example.review.Reviewers
    source: example.review.Reviewer
    consistency: read_your_writes
    fields:
      - {name: reviewer_id, type: example.review.ReviewerId}
      - {name: label, type: String}
",
    );
    witnessed("prior rows of two entities", &text, CONFIRM_EXISTS);
}

/// Two commands create the related row; the second sets `item` from a literal.
#[test]
fn own_run_with_two_creators_of_the_related_row() {
    let text = replaced(
        &replaced(
            CONFIRM,
            "  - name: example.review.ExpireDecision\n",
            "  - name: example.review.ImportDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
    outcomes:
      - name: imported
        creates: example.review.Decision
        instance: decision_id
        sets: {item: \"imported\"}
        emits: [example.review.DecisionImported]
        payload:
          example.review.DecisionImported: {decision_id: input.decision_id}
  - name: example.review.ExpireDecision
",
        ),
        "  - name: example.review.DecisionExpired\n",
        "  - name: example.review.DecisionImported
    fields:
      - {name: decision_id, type: example.review.DecisionId}
  - name: example.review.DecisionExpired
",
    );
    witnessed("two creators", &text, CONFIRM_EXISTS);
}

/// The command has two creating branches, an urgent one decided by an input guard and the plain
/// one, so the existence scenario holds two segments, each arranging its own related row.
#[test]
fn own_run_for_two_creating_branches() {
    let text = replaced(
        &replaced(
            CONFIRM,
            "      - {name: item, type: String}\n    outcomes:\n      - {name: already-exists,",
            "      - {name: item, type: String}\n      - {name: priority, type: Integer}\n    outcomes:\n      - {name: already-exists,",
        ),
        "      - name: confirmed\n",
        "      - name: confirmed-urgent
        when: priority > 5
        creates: example.review.Decision
        instance: decision_id
        sets: {item: input.item}
        emits: [example.review.DecisionConfirmed]
        payload:
          example.review.DecisionConfirmed: {decision_id: input.decision_id}
      - name: confirmed
",
    );
    witnessed("two creating branches", &text, CONFIRM_EXISTS);
}

/// The second adopter of the story's fit review: a booking refused where its schedule is not
/// `Published`, and a booking id already taken.
const BOOK: &str = "format: ess/35
system: demo
version: v1
domain: demo.book
types:
  - {name: demo.book.ScheduleId, kind: newtype, of: Uuid}
  - {name: demo.book.BookingId, kind: newtype, of: String}
entities:
  - name: demo.book.Schedule
    identity: {name: schedule_id, type: demo.book.ScheduleId}
    fields:
      - {name: title, type: String}
    lifecycle:
      initial: Draft
      states: [Draft, Published]
      terminal: [Published]
      transitions:
        - {name: publish, from: [Draft], to: Published}
  - name: demo.book.Booking
    identity: {name: booking_id, type: demo.book.BookingId}
    fields:
      - {name: schedule_id, type: demo.book.ScheduleId}
    lifecycle: {initial: Booked, states: [Booked], terminal: [Booked]}
events:
  - name: demo.book.ScheduleOpened
    fields: [{name: schedule_id, type: demo.book.ScheduleId}]
  - name: demo.book.SchedulePublished
    fields: [{name: schedule_id, type: demo.book.ScheduleId}]
  - name: demo.book.Booked
    fields: [{name: booking_id, type: demo.book.BookingId}]
errors:
  - {name: demo.book.BookingTaken, fields: []}
  - {name: demo.book.NoSchedule, fields: []}
  - {name: demo.book.NotPublished, fields: []}
  - {name: demo.book.AlreadyPublished, fields: []}
commands:
  - name: demo.book.OpenSchedule
    input:
      - {name: schedule_id, type: demo.book.ScheduleId}
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.book.Schedule
        instance: schedule_id
        sets: {title: input.title}
        emits: [demo.book.ScheduleOpened]
        payload:
          demo.book.ScheduleOpened: {schedule_id: input.schedule_id}
  - name: demo.book.PublishSchedule
    input:
      - {name: schedule_id, type: demo.book.ScheduleId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Published], error: demo.book.AlreadyPublished}
      - name: published
        moves: demo.book.Schedule.publish
        instance: schedule_id
        emits: [demo.book.SchedulePublished]
        payload:
          demo.book.SchedulePublished: {schedule_id: input.schedule_id}
  - name: demo.book.Book
    input:
      - {name: booking_id, type: demo.book.BookingId}
      - {name: schedule_id, type: demo.book.ScheduleId}
    outcomes:
      - {name: taken, existing_instance: true, error: demo.book.BookingTaken}
      - {name: no-schedule, when_related: {via: input.schedule_id, exists: false}, error: demo.book.NoSchedule}
      - {name: unpublished, when_related: {via: input.schedule_id, predicate: state != Published}, error: demo.book.NotPublished}
      - name: booked
        creates: demo.book.Booking
        instance: booking_id
        sets: {schedule_id: input.schedule_id}
        emits: [demo.book.Booked]
        payload:
          demo.book.Booked: {booking_id: input.booking_id}
views:
  - name: demo.book.Schedules
    source: demo.book.Schedule
    consistency: read_your_writes
    fields:
      - {name: schedule_id, type: demo.book.ScheduleId}
      - {name: title, type: String}
      - {name: state, type: demo.book.Schedule.State}
  - name: demo.book.Bookings
    source: demo.book.Booking
    consistency: read_your_writes
    fields:
      - {name: booking_id, type: demo.book.BookingId}
      - {name: schedule_id, type: demo.book.ScheduleId}
";

const BOOK_EXISTS: &str = "demo.book.Book/outcome/taken";

#[test]
fn own_run_for_the_second_adopter() {
    witnessed("schedule and booking", BOOK, BOOK_EXISTS);
}

/// The same, where the schedule owns its bookings (`kind: owns`): the related row is the created
/// row's owner, which `steered` returns early for.
#[test]
fn own_run_where_the_related_row_owns_the_created_one() {
    let text = replaced(
        BOOK,
        "      - {name: title, type: String}\n    lifecycle:\n",
        "      - {name: title, type: String}\n    relations:\n      - {name: bookings, kind: owns, target: demo.book.Booking, cardinality: many, via: schedule_id}\n    lifecycle:\n",
    );
    witnessed("schedule owns booking", &text, BOOK_EXISTS);
}

/// `PICK` of the unit's own test file: no catalogue row passes both revision guards, so the
/// existence scenario is refused with ESS-SYNTH-004 (`RelatedUnmet`).
const PICK: &str = "format: ess/35
system: demo
version: v1
domain: demo.pick
types:
  - {name: demo.pick.CatalogueId, kind: newtype, of: Uuid}
  - {name: demo.pick.SelectionId, kind: newtype, of: Uuid}
entities:
  - name: demo.pick.Catalogue
    identity: {name: catalogue_id, type: demo.pick.CatalogueId}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Projected, states: [Projected], terminal: [Projected]}
  - name: demo.pick.Selection
    identity: {name: selection_id, type: demo.pick.SelectionId}
    fields:
      - {name: revision, type: Integer}
    lifecycle:
      initial: Draft
      states: [Draft, Selected]
      terminal: [Selected]
      transitions:
        - {name: settle, from: [Draft], to: Selected}
errors:
  - {name: demo.pick.SelectionExists, fields: []}
  - {name: demo.pick.CatalogueNotFound, fields: []}
  - {name: demo.pick.RevisionMismatch, fields: []}
  - {name: demo.pick.RevisionMatch, fields: []}
  - {name: demo.pick.AlreadySelected, fields: []}
events:
  - name: demo.pick.CatalogueProjected
    fields: [{name: catalogue_id, type: demo.pick.CatalogueId}]
  - name: demo.pick.SelectionMade
    fields: [{name: selection_id, type: demo.pick.SelectionId}]
  - name: demo.pick.SelectionSettled
    fields: [{name: selection_id, type: demo.pick.SelectionId}]
commands:
  - name: demo.pick.Project
    input:
      - {name: catalogue_id, type: demo.pick.CatalogueId}
      - {name: revision, type: Integer}
    outcomes:
      - name: projected
        creates: demo.pick.Catalogue
        instance: catalogue_id
        sets: {revision: input.revision}
        emits: [demo.pick.CatalogueProjected]
        payload:
          demo.pick.CatalogueProjected: {catalogue_id: input.catalogue_id}
  - name: demo.pick.Settle
    input:
      - {name: selection_id, type: demo.pick.SelectionId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Selected], error: demo.pick.AlreadySelected}
      - name: settled
        moves: demo.pick.Selection.settle
        instance: selection_id
        emits: [demo.pick.SelectionSettled]
        payload:
          demo.pick.SelectionSettled: {selection_id: input.selection_id}
  - name: demo.pick.Select
    input:
      - {name: selection_id, type: demo.pick.SelectionId}
      - {name: catalogue_id, type: demo.pick.CatalogueId}
      - {name: revision, type: Integer}
    outcomes:
      - {name: selection-exists, existing_instance: true, error: demo.pick.SelectionExists}
      - {name: catalogue-unknown, when_related: {via: input.catalogue_id, exists: false}, error: demo.pick.CatalogueNotFound}
      - {name: revision-mismatch, when_related: {via: input.catalogue_id, predicate: revision != input.revision}, error: demo.pick.RevisionMismatch}
      - {name: revision-match, when_related: {via: input.catalogue_id, predicate: revision == input.revision}, error: demo.pick.RevisionMatch}
      - name: selected
        creates: demo.pick.Selection
        instance: selection_id
        into: Selected
        sets: {revision: input.revision}
        emits: [demo.pick.SelectionMade]
        payload:
          demo.pick.SelectionMade: {selection_id: input.selection_id}
views:
  - name: demo.pick.Selections
    source: demo.pick.Selection
    consistency: read_your_writes
    fields:
      - {name: selection_id, type: demo.pick.SelectionId}
      - {name: revision, type: Integer}
";

const PICK_EXISTS: &str = "demo.pick.Select/outcome/selection-exists";

const PROJECT_FROM_INPUT: &str = "      - {name: revision, type: Integer}
    outcomes:
      - name: projected
        creates: demo.pick.Catalogue
        instance: catalogue_id
        sets: {revision: input.revision}
";

fn pick_help(text: &str) -> String {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    let Some(refusal) = synthesis.refusals.iter().find(|refusal| {
        refusal
            .scenario
            .as_ref()
            .is_some_and(|id| id.to_string() == PICK_EXISTS)
    }) else {
        panic!("a refusal of `{PICK_EXISTS}`: {:?}", refusals(&synthesis));
    };
    assert_eq!(
        refusal.cause.code().to_string(),
        "ESS-SYNTH-004",
        "{}",
        refusal.cause
    );
    refusal.hint().to_owned()
}

/// `Project` sets the compared revision from a literal: the help asks for `sets:`. Nothing else in
/// the suite pins the `unset: true` arm, so `creators_leave_unset` answering `false` always would
/// pass every other test.
#[test]
fn help_asks_for_sets_where_the_creator_sets_a_literal() {
    let text = replaced(
        PICK,
        PROJECT_FROM_INPUT,
        "    outcomes:
      - name: projected
        creates: demo.pick.Catalogue
        instance: catalogue_id
        sets: {revision: 0}
",
    );
    let help = pick_help(&text);
    assert!(help.contains("sets: <field>: input.<field>"), "{help}");
}

/// One creator sets the revision from its input, a second from a literal: a creator leaves it
/// unset, so the help asks.
#[test]
fn help_asks_for_sets_where_one_of_two_creators_sets_a_literal() {
    let text = replaced(
        PICK,
        "  - name: demo.pick.Settle\n",
        "  - name: demo.pick.Import
    input:
      - {name: catalogue_id, type: demo.pick.CatalogueId}
    outcomes:
      - name: imported
        creates: demo.pick.Catalogue
        instance: catalogue_id
        sets: {revision: 0}
        emits: [demo.pick.CatalogueProjected]
        payload:
          demo.pick.CatalogueProjected: {catalogue_id: input.catalogue_id}
  - name: demo.pick.Settle
",
    );
    let help = pick_help(&text);
    assert!(help.contains("sets: <field>: input.<field>"), "{help}");
}

/// `Project` sets the revision from an input of another name: it already sets it from its input.
#[test]
fn help_names_no_fix_where_the_creator_sets_from_another_input_name() {
    let text = replaced(
        PICK,
        PROJECT_FROM_INPUT,
        "      - {name: rev, type: Integer}
    outcomes:
      - name: projected
        creates: demo.pick.Catalogue
        instance: catalogue_id
        sets: {revision: input.rev}
",
    );
    let help = pick_help(&text);
    assert!(!help.contains("sets: <field>: input.<field>"), "{help}");
}

/// `BOOK` with an external refusal answered before the row (`before_row: true`, ess/25): its
/// scenario gains a segment storing a booking first (`external_before_row`), whose creation is the
/// creating branch's own run through the related schedule. Every scenario passes on the
/// interpreter.
#[test]
fn own_run_in_the_segment_of_an_external_refusal_before_the_row() {
    let text = replaced(
        &replaced(
            BOOK,
            "      - {name: schedule_id, type: demo.book.ScheduleId}\n    outcomes:\n      - {name: taken,",
            "      - {name: schedule_id, type: demo.book.ScheduleId}\n      - {name: secret, type: String}\n    outcomes:\n      - name: bad-credential\n        external: secret is not the digest the service derives from booking_id\n        before_row: true\n        error: demo.book.BadCredential\n      - {name: taken,",
        ),
        "  - {name: demo.book.BookingTaken, fields: []}\n",
        "  - {name: demo.book.BookingTaken, fields: []}\n  - {name: demo.book.BadCredential, fields: []}\n",
    );
    witnessed("external before the row", &text, BOOK_EXISTS);
    witnessed(
        "external before the row",
        &text,
        "demo.book.Book/outcome/bad-credential",
    );
}
