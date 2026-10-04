//! Adversary, E-U4 pass 1: the decision receipt through the crate's own wrapper targets, and an
//! instant the recorder writes that its own reader refuses.
//!
//! - `untraced_forwards_the_decision_receipt`: `Untraced` documents itself as "the same
//!   implementation asked one fewer question" (`src/reference.rs`), forwarding every method but
//!   `observe_invocations`. It does not forward `execute_command_recorded`, so the trait default
//!   runs `execute_command` and answers no time: the wrapped target read its clock and the instant
//!   is silently lost.
//! - `faulty_keeps_the_decision_receipt_through_atomic`: `Faulty` is "wrong in exactly one way"
//!   (`src/faulty.rs`). Wrapping a clock-bearing interpreter with a fault that touches none of its
//!   commands, a history recorded through `Atomic` carries no `decision_time` and is written as
//!   `ess-history/1`: a second, undeclared defect.
//! - `the_recorder_never_writes_an_instant_its_reader_refuses`: `DecisionInstant::from_instant`
//!   admits any `Rfc3339Instant`, including one before `0000-01-01T00:00:00Z` that an offset
//!   spelling parses to. The recorder writes it, `serde_json` serializes it, and `history::read`
//!   refuses the bytes.

mod support_occurrence_clock;

use std::collections::BTreeMap;

use ess_conformance::faulty::{Fault, Faulty};
use ess_conformance::history::{self, HistoryFormat};
use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::reference::Untraced;
use ess_conformance::scenario::SuiteProvenance;
use ess_conformance::target::ConformanceTarget;
use ess_primitives::time::Rfc3339Instant;
use support_occurrence_clock::*;

fn call(command: &str, input: &[(&str, &str)], subject: Subject) -> Call {
    Call::new(
        command,
        input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), text(value)))
            .collect::<BTreeMap<_, _>>(),
        subject,
    )
}

/// One offer opened in the prefix, one acceptance of it on one client.
fn open_then_accept() -> Workload {
    Workload {
        prefix: vec![call(
            "demo.offers.OpenOffer",
            &[("expires_at", DEADLINE)],
            Subject::Creates,
        )],
        clients: vec![vec![call(
            "demo.offers.AcceptOffer",
            &[("expires_at", DEADLINE)],
            Subject::Created(0),
        )]],
    }
}

#[test]
fn untraced_forwards_the_decision_receipt() {
    let clock = Scripted::setup_then_decision();
    let untraced = Untraced(offers(Some(clock.clone())));
    let offer = open(&untraced, DEADLINE);
    assert_eq!(
        clock.reads(),
        1,
        "the setup decision read the provider once"
    );

    let receipt = untraced.execute_command_recorded(accept(&offer, DEADLINE));
    assert_eq!(
        clock.reads(),
        2,
        "the wrapped target decided with its clock"
    );
    assert_eq!(
        outcome(&receipt.answer.expect("answered")),
        "lapsed",
        "decided at T1"
    );
    assert_eq!(
        receipt.decision_time,
        Some(instant(T1)),
        "the wrapped target read T1 at its decision edge; `Untraced` answered the receipt without it"
    );
}

#[test]
fn faulty_keeps_the_decision_receipt_through_atomic() {
    let ir = model(OFFERS);
    let clock = Scripted::new(&[T0, "2000-07-01T00:00:00.5Z"]);
    // `WrongEvent` rewrites `billing.invoice.CreateInvoice` only: no offers command is touched.
    let faulty = Faulty::new(
        Interpreted::for_model(model(OFFERS)).with_command_clock(clock.clone()),
        Fault::WrongEvent,
    );
    begin(&faulty);
    let history = record::record(&ir, &Atomic(&faulty), &open_then_accept(), 0).expect("recorded");
    assert_eq!(clock.reads(), 2, "both decisions read the provider");
    let recorded: Vec<Option<DecisionInstant>> = history
        .operations
        .iter()
        .map(|operation| operation.decision_time)
        .collect();
    assert_eq!(
        recorded,
        vec![Some(instant(T0)), Some(instant("2000-07-01T00:00:00.5Z"))],
        "each operation carries the reading its decision used"
    );
    assert_eq!(history.format, HistoryFormat::EssHistory2);
}

/// A clock answering `first` once, then `then` on every further read.
struct FirstThen {
    first: DecisionInstant,
    then: DecisionInstant,
    read: std::sync::atomic::AtomicBool,
}

impl CommandClock for FirstThen {
    fn read(&self) -> Option<DecisionInstant> {
        if self.read.swap(true, std::sync::atomic::Ordering::SeqCst) {
            Some(self.then)
        } else {
            Some(self.first)
        }
    }
}

#[test]
fn the_recorder_never_writes_an_instant_its_reader_refuses() {
    // One minute before the first second RFC 3339 spells, reached through an offset spelling the
    // instant parser admits.
    let before_year_zero =
        Rfc3339Instant::parse_rfc3339("0000-01-01T00:00:00+00:01").expect("an RFC 3339 date-time");
    let instant = DecisionInstant::from_instant(before_year_zero);

    let ir = model(OFFERS);
    let target = Interpreted::for_model(model(OFFERS)).with_command_clock(FirstThen {
        first: support_occurrence_clock::instant(T0),
        then: instant,
        read: std::sync::atomic::AtomicBool::new(false),
    });
    begin(&target);
    let history = record::record(&ir, &Atomic(&target), &open_then_accept(), 0).expect("recorded");
    // The acceptance decided at that reading; its guard could not use it, and the receipt keeps it.
    assert_eq!(
        history.operations[1].decision_time,
        Some(instant),
        "{history:?}"
    );

    match serde_json::to_vec(&history) {
        // Refusing to write it is one honest answer.
        Err(_) => {}
        // Writing it is the other only if the reader reads it back.
        Ok(bytes) => {
            let read = history::read(&bytes, &SuiteProvenance::of(&ir).spec_digest);
            assert!(
                read.is_ok(),
                "the recorder wrote `{}`, which its own reader refuses: {}",
                instant,
                read.unwrap_err()
            );
        }
    }
}
