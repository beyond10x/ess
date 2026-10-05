//! One observed decision instant per command occurrence, on the native interpreter
//! (beyond10x/ess#244 part a, unit U4; `docs/design/expression-family-source22.md`, "One observed
//! decision instant per command occurrence").
//!
//! The scripted provider answers the setup instant [`T0`], then the decision instant [`T1`], then a
//! later instant on every further read. The deadline sits strictly between the two, so a target that
//! decided the tested command with the setup reading answers `accepted` where the decision reading
//! answers `lapsed`, and a target that reads the provider more than once per decision is counted.
//!
//! Cases:
//!
//! - `clock_read_once_per_decision`: one read per executed decision, the receipt carrying exactly
//!   that reading at full precision; none for a command refused before its decision edge.
//! - `clock_absent_unknown_only_when_needed`: with no provider, a decision whose guard reads `now`
//!   is Unknown (reported unsupported, never a branch), and every decision that does not reach such
//!   a leaf is answered.
//! - `clock_absent_earlier_refusal_wins`: an input refusal, an unknown identity and a held state
//!   that decide the command before any `now` leaf answer with no provider.
//! - `setup_time_reuse_fault_fails` and `per_read_reread_fault_fails`: the scripted-provider control
//!   passes the interpreter and fails a target deciding with the setup reading, and one reading the
//!   provider once per leaf (rule 18: clock-edge faults are this control's authority).

mod support_occurrence_clock;

use std::collections::BTreeMap;
use std::sync::Arc;

use ess_conformance::interpret::execute::{self, Externals, Generated, Store, Undetermined};
use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::target::{ConformanceTarget, EntitySetupRequest};
use ess_domain::name::QualifiedName;
use ess_primitives::ids::CorrelationId;
use support_occurrence_clock::*;

/// The control a target passes only by reading the provider once per decision and deciding with
/// that reading: setup at [`T0`], then the tested decision at [`T1`].
fn decision_control(target: &dyn ConformanceTarget, clock: &Scripted) -> Result<(), String> {
    let opened = target
        .execute_command(request(
            "demo.offers.OpenOffer",
            &[("expires_at", text(DEADLINE))],
        ))
        .map_err(|error| format!("the setup decision failed: {error}"))?;
    if clock.reads() != 1 {
        return Err(format!(
            "the setup decision read the provider {} times",
            clock.reads()
        ));
    }
    if outcome(&opened) != "opened" {
        return Err(format!(
            "the setup decision answered `{}`, not `opened`",
            outcome(&opened)
        ));
    }
    let offer = opened.direct_events[0].payload["offer_id"]
        .as_text()
        .ok_or("the opened offer has a text identity")?
        .to_owned();
    let receipt = target.execute_command_recorded(accept(&offer, DEADLINE));
    if clock.reads() != 2 {
        return Err(format!(
            "the setup and the tested decision read the provider {} times, not twice",
            clock.reads()
        ));
    }
    if receipt.decision_time != Some(clock.reading(1)) {
        return Err(format!(
            "the receipt carries {:?}, not the decision reading {}",
            receipt.decision_time,
            clock.reading(1)
        ));
    }
    match receipt.answer {
        Ok(result) if outcome(&result) == "lapsed" => Ok(()),
        Ok(result) => Err(format!(
            "the tested decision answered `{}`, not `lapsed`",
            outcome(&result)
        )),
        Err(error) => Err(format!("the tested decision failed: {error}")),
    }
}

#[test]
fn clock_read_once_per_decision() {
    let clock = Scripted::setup_then_decision();
    let target = offers(Some(clock.clone()));
    let offer = open(&target, DEADLINE);
    assert_eq!(
        clock.reads(),
        1,
        "the setup decision reads the provider once"
    );

    let receipt = target.execute_command_recorded(accept(&offer, DEADLINE));
    assert_eq!(
        clock.reads(),
        2,
        "the tested decision reads the provider once"
    );
    let decided = receipt
        .decision_time
        .expect("the decision edge was reached");
    assert_eq!(decided, instant(T1));
    assert_eq!(
        decided.to_rfc3339(),
        T1,
        "the receipt keeps the reading at full precision"
    );
    assert_eq!(outcome(&receipt.answer.expect("answered")), "lapsed");

    // The ordinary entrypoint runs the same command core and discards the receipt: one more read,
    // never two.
    let later = target
        .execute_command(accept(&offer, "2030-01-01T00:00:00Z"))
        .expect("answered");
    assert_eq!(clock.reads(), 3);
    assert_eq!(outcome(&later), "accepted", "decided at T1 + 1s");

    // A command refused before its decision edge reads nothing and carries no time.
    let unheld = Interpreted::new().with_command_clock(clock.clone());
    let refused = unheld.execute_command_recorded(accept(&offer, DEADLINE));
    assert!(refused.answer.is_err());
    assert_eq!(refused.decision_time, None);
    assert_eq!(clock.reads(), 3, "no decision, no read");
}

/// `offer` arranged in `state` with no command and no clock.
fn establish(target: &Interpreted, offer: &str, state: &str) {
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.offers.Offer".parse().unwrap(),
            identity: text(offer),
            fields: BTreeMap::from([("expires_at".to_owned(), text(DEADLINE))]),
            state: state.parse().unwrap(),
            correlation: CorrelationId::new("occurrence-clock").unwrap(),
        })
        .unwrap_or_else(|error| panic!("the offer is established: {error}"));
}

const OPEN: &str = "00000000-0000-4000-8000-00000000a001";
const ACCEPTED: &str = "00000000-0000-4000-8000-00000000a002";
const NOBODY: &str = "00000000-0000-4000-8000-00000000a003";

#[test]
fn clock_absent_unknown_only_when_needed() {
    let target = offers(None);
    establish(&target, OPEN, "Open");
    establish(&target, ACCEPTED, "Accepted");

    // The guard of `accepted` reads `now`, and the held state `Open` reaches it: Unknown, reported
    // as a capability the target lacks, never a branch and never an effect.
    let receipt = target.execute_command_recorded(accept(OPEN, DEADLINE));
    assert!(unsupported(&receipt.answer), "{:?}", receipt.answer);
    assert_eq!(receipt.decision_time, None, "no provider, no reading");
    let again = target.execute_command_recorded(accept(OPEN, "2030-01-01T00:00:00Z"));
    assert!(
        unsupported(&again.answer),
        "the offer was not moved by the Unknown decision: {:?}",
        again.answer
    );
    let unmoved = target
        .execute_command(request(
            "demo.offers.ArchiveOffer",
            &[("offer_id", text(OPEN))],
        ))
        .expect("answered");
    assert_eq!(outcome(&unmoved), "not-accepted", "no effect was licensed");

    // A creation guarded by `now` is Unknown too.
    assert!(unsupported(&target.execute_command(request(
        "demo.offers.OpenOffer",
        &[("expires_at", text(DEADLINE))],
    ))));

    // A command reading no clock is answered with none.
    let archived = target
        .execute_command(request(
            "demo.offers.ArchiveOffer",
            &[("offer_id", text(ACCEPTED))],
        ))
        .expect("answered");
    assert_eq!(outcome(&archived), "archived");

    // The clock-free library entrypoint delegates with no instant: the guard is Unknown, named
    // as such, and no longer refused before selection as a construct not interpreted.
    let ir = model(OFFERS);
    let command = QualifiedName::new("demo.offers.OpenOffer").unwrap();
    let input = BTreeMap::from([("expires_at".to_owned(), text(DEADLINE))]);
    let clock_free = execute::execute(
        &ir,
        &Store::default(),
        &command,
        &input,
        &Externals::Withheld,
    );
    assert!(
        matches!(clock_free, Err(Undetermined::Undecidable { .. })),
        "{clock_free:?}"
    );
    // The typed entrypoint decides it with the instant it is handed.
    for (at, expected) in [(T0, "opened"), (T1, "too-late-to-open")] {
        let steps = execute::execute_at(
            &ir,
            &Store::default(),
            &command,
            &input,
            &Externals::Withheld,
            &Generated::Counter,
            Some(instant(at)),
        )
        .expect("decided");
        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0]
                .outcome
                .as_ref()
                .map(|taken| taken.outcome.to_string()),
            Some(expected.to_owned()),
            "at {at}"
        );
    }
}

#[test]
fn clock_absent_earlier_refusal_wins() {
    for clock in [None, Some(Scripted::setup_then_decision())] {
        let target = offers(clock.clone());
        establish(&target, OPEN, "Open");
        establish(&target, ACCEPTED, "Accepted");
        let cases = [
            // An input refusal declared before the `now` leaf.
            (OPEN, "1999-01-01T00:00:00Z", "malformed"),
            // An identity nobody holds, answered before the guarded branch is read.
            (NOBODY, DEADLINE, "unknown"),
            // A held state that rules the `now`-guarded branch out whatever the time.
            (ACCEPTED, DEADLINE, "lapsed"),
        ];
        for (index, (offer, expires_at, expected)) in cases.into_iter().enumerate() {
            let receipt = target.execute_command_recorded(accept(offer, expires_at));
            let answer = receipt
                .answer
                .unwrap_or_else(|error| panic!("{expected} with clock {clock:?}: {error}"));
            assert_eq!(outcome(&answer), expected, "with clock {clock:?}");
            match &clock {
                None => assert_eq!(receipt.decision_time, None),
                Some(clock) => {
                    assert_eq!(clock.reads(), index + 1, "one read per decision");
                    assert_eq!(receipt.decision_time, Some(clock.reading(index)));
                }
            }
        }
    }
}

/// A provider that answers its first reading forever after: the setup instant reused.
struct ReuseFirst {
    inner: Arc<Scripted>,
    first: std::sync::Mutex<Option<DecisionInstant>>,
}

impl CommandClock for ReuseFirst {
    fn read(&self) -> Option<DecisionInstant> {
        let mut first = self.first.lock().expect("one reader at a time");
        if first.is_none() {
            *first = self.inner.read();
        }
        *first
    }
}

/// A provider read once per leaf: two reads for a decision, the second deciding.
struct EveryRead(Arc<Scripted>);

impl CommandClock for EveryRead {
    fn read(&self) -> Option<DecisionInstant> {
        let _first_leaf = self.0.read();
        self.0.read()
    }
}

#[test]
fn setup_time_reuse_fault_fails() {
    let clock = Scripted::setup_then_decision();
    assert_eq!(
        decision_control(&offers(Some(clock.clone())), &clock),
        Ok(())
    );

    let clock = Scripted::setup_then_decision();
    let faulty = Interpreted::for_model(model(OFFERS)).with_command_clock(ReuseFirst {
        inner: clock.clone(),
        first: std::sync::Mutex::new(None),
    });
    begin(&faulty);
    let verdict = decision_control(&faulty, &clock);
    assert!(verdict.is_err(), "the setup-time target passed the control");
    // And the reason is the clock edge: the deadline between T0 and T1 is decided the other way.
    let offer = open(&faulty, DEADLINE);
    let decided = faulty.execute_command(accept(&offer, DEADLINE)).unwrap();
    assert_eq!(
        outcome(&decided),
        "accepted",
        "decided with the setup reading"
    );
}

#[test]
fn per_read_reread_fault_fails() {
    let clock = Scripted::setup_then_decision();
    let faulty = Interpreted::for_model(model(OFFERS)).with_command_clock(EveryRead(clock.clone()));
    begin(&faulty);
    let verdict = decision_control(&faulty, &clock);
    assert!(
        verdict
            .as_ref()
            .is_err_and(|why| why.contains("read the provider")),
        "{verdict:?}"
    );
}
