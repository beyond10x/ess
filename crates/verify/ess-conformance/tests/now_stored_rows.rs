//! `now` over stored and related rows, on the native interpreter
//! (`docs/design/expression-family-source22.md`, "A3: current time over stored and related rows";
//! beyond10x/ess#244 part a, unit U5).
//!
//! `tests/fixtures/now-stored-rows.yaml` (`ess/22`) orders stored instants against the current
//! time in every unit: a lease is renewed while its stored expiry is not more than an hour past
//! (`when_subject:`, `h`), or graced while its stored grace is more than five minutes ahead
//! (`when_subject:`, `m`); a member is refused while the ban the member's row stores ends more than
//! thirty seconds ahead (`when_related:`, `s`), and a book is refused while its stored embargo ended
//! less than two minutes ago (`when_related:` over a second row, `m`).
//!
//! The scripted provider answers the setup instant [`T0`] for every arranging command and the
//! decision instant [`T1`] for the command under test. Every stored instant sits on a boundary that
//! lies strictly between the two, so a decision made with the setup reading answers differently,
//! and the boundaries are tried to the nanosecond, so a `>` read as `>=` or a truncated instant is
//! told apart.
//!
//! Cases:
//!
//! - `a3_subject_rows_are_decided_at_the_decision_instant` and
//!   `a3_related_rows_are_decided_at_the_decision_instant`: every boundary side of the subject's
//!   two guards and of both related rows' guards, the read count, and the receipt's instant.
//! - `a3_one_reading_decides_every_row_of_the_decision`: two rows, each straddling the decision
//!   instant, read in one decision with one reading.
//! - `a3_missing_clock_is_unknown_only_where_a_row_needs_it`: with no provider, every decision
//!   reaching a stored or related `now` leaf is Unknown and licenses no effect; every earlier
//!   definitive refusal (an input refusal, an unknown identity, a missing related row) answers.
//! - `a3_setup_time_reuse_and_reread_faults_fail`: a provider reused from the setup reading, and a
//!   provider read once per row, are told apart from the healthy target (rule 18).

mod support_occurrence_clock;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::target::{
    ConformanceTarget, SemanticCommandRequest, SemanticViewRequest, TargetError,
};
use ess_primitives::consistency::QueryConsistency;
use ess_primitives::node::Node;
use support_occurrence_clock::{instant, model, outcome, request, text, unsupported, T0, T1};

const LEASES: &str = include_str!("fixtures/now-stored-rows.yaml");

/// Far before every boundary, and far after: rows on which a guard is decided whatever the time.
const PAST: &str = "1990-01-01T00:00:00Z";
const FUTURE: &str = "2090-01-01T00:00:00Z";

/// [`T1`] less an hour: the renewal boundary, inclusive (`expires_at >= now - 1h`).
const T1_LESS_1H: &str = "2026-10-04T11:00:00.123456789Z";
/// The same instant in another offset: a target ordering the spelling answers it the other way.
const T1_LESS_1H_ELSEWHERE: &str = "2026-10-04T12:00:00.123456789+01:00";
const T1_LESS_1H_LESS_1NS: &str = "2026-10-04T11:00:00.123456788Z";
/// [`T1`] plus five minutes: the grace boundary, exclusive (`grace_until > now + 5m`).
const T1_PLUS_5M: &str = "2026-10-04T12:05:00.123456789Z";
const T1_PLUS_5M_PLUS_1NS: &str = "2026-10-04T12:05:00.12345679Z";
/// [`T1`] plus thirty seconds: the ban boundary, exclusive (`banned_until > now + 30s`).
const T1_PLUS_30S: &str = "2026-10-04T12:00:30.123456789Z";
const T1_PLUS_30S_PLUS_1NS: &str = "2026-10-04T12:00:30.12345679Z";
/// [`T1`] less two minutes: the embargo boundary, inclusive (`embargo_until >= now - 2m`).
const T1_LESS_2M: &str = "2026-10-04T11:58:00.123456789Z";
const T1_LESS_2M_LESS_1NS: &str = "2026-10-04T11:58:00.123456788Z";

const NOTE: &str = "renewal";

/// A provider answering [`T0`] while it is told the commands it is read for arrange, and the
/// decision instant while the command under test decides — each further read within one decision
/// an hour later than the one before, so a target reading it more than once decides with another
/// instant. It counts every read.
#[derive(Debug)]
struct Phased {
    state: Mutex<PhaseState>,
}

#[derive(Debug)]
struct PhaseState {
    testing: bool,
    in_decision: i64,
    reads: usize,
}

impl Phased {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(PhaseState {
                testing: false,
                in_decision: 0,
                reads: 0,
            }),
        })
    }

    /// The commands read for from here on are the command under test.
    fn testing(&self) {
        let mut state = self.state.lock().unwrap();
        state.testing = true;
        state.in_decision = 0;
    }

    /// One decision begins: its first read answers the phase's instant again.
    fn decision(&self) {
        self.state.lock().unwrap().in_decision = 0;
    }

    fn reads(&self) -> usize {
        self.state.lock().unwrap().reads
    }
}

impl CommandClock for Phased {
    fn read(&self) -> Option<DecisionInstant> {
        let mut state = self.state.lock().unwrap();
        state.reads += 1;
        let base = if state.testing {
            instant(T1)
        } else {
            instant(T0)
        };
        let later = state.in_decision;
        state.in_decision += 1;
        Some(DecisionInstant::from_instant(
            base.instant().plus_seconds(3600 * later).expect("spelled"),
        ))
    }
}

/// The leases model executed by the interpreter, deciding with `clock` where one is given.
fn leases(clock: Option<Box<dyn CommandClock + Send>>) -> Interpreted {
    let target = Interpreted::for_model(model(LEASES));
    let target = match clock {
        Some(clock) => target.with_command_clock(clock),
        None => target,
    };
    support_occurrence_clock::begin(&target);
    target
}

fn send(target: &dyn ConformanceTarget, clock: &Phased, request: SemanticCommandRequest) -> String {
    clock.decision();
    let answer = target
        .execute_command(request)
        .unwrap_or_else(|error| panic!("answered: {error}"));
    outcome(&answer)
}

/// The identity a creation published in its one event, as `field`.
fn created(
    target: &dyn ConformanceTarget,
    command: &str,
    input: &[(&str, &str)],
    field: &str,
) -> String {
    let input: Vec<(&str, Node)> = input
        .iter()
        .map(|(name, value)| (*name, text(value)))
        .collect();
    let answer = target
        .execute_command(request(command, &input))
        .unwrap_or_else(|error| panic!("{command} creates: {error}"));
    answer.direct_events[0].payload[field]
        .as_text()
        .expect("a text identity")
        .to_owned()
}

fn open(target: &dyn ConformanceTarget, expires_at: &str, grace_until: &str) -> String {
    created(
        target,
        "demo.leases.OpenLease",
        &[("expires_at", expires_at), ("grace_until", grace_until)],
        "lease_id",
    )
}

fn register(target: &dyn ConformanceTarget, banned_until: &str) -> String {
    created(
        target,
        "demo.leases.RegisterMember",
        &[("banned_until", banned_until)],
        "member_id",
    )
}

fn add_book(target: &dyn ConformanceTarget, embargo_until: &str) -> String {
    created(
        target,
        "demo.leases.AddBook",
        &[("embargo_until", embargo_until)],
        "book_id",
    )
}

fn renew(lease: &str, note: &str) -> SemanticCommandRequest {
    request(
        "demo.leases.RenewLease",
        &[
            ("lease_id", text(lease)),
            ("new_expires_at", text(FUTURE)),
            ("note", text(note)),
        ],
    )
}

fn join(member: &str) -> SemanticCommandRequest {
    request("demo.leases.Join", &[("member_id", text(member))])
}

fn lend(member: &str, book: &str, copies: i64) -> SemanticCommandRequest {
    request(
        "demo.leases.Lend",
        &[
            ("member_id", text(member)),
            ("book_id", text(book)),
            (
                "copies",
                Node::Number(ess_primitives::facts::Number::from(copies)),
            ),
        ],
    )
}

/// The `Leases` view's state of `lease`.
fn lease_state(target: &dyn ConformanceTarget, lease: &str) -> String {
    let view = target
        .query_view(SemanticViewRequest {
            view: "demo.leases.Leases".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: ess_primitives::ids::CorrelationId::new("now-stored-rows").unwrap(),
            deadline: ess_conformance::target::Deadline::at(
                ess_primitives::time::Timestamp::from_epoch_millis(0),
            ),
        })
        .expect("the view answers");
    let row = view
        .rows
        .iter()
        .find(|row| row["lease_id"].as_text() == Some(lease))
        .expect("the lease is listed");
    row["state"].as_text().expect("a state").to_owned()
}

#[test]
fn a3_subject_rows_are_decided_at_the_decision_instant() {
    for (expires_at, grace_until, expected) in [
        // `expires_at >= now - 1h`: on the boundary, in either spelling, and a nanosecond before.
        (T1_LESS_1H, PAST, "renewed"),
        (T1_LESS_1H_ELSEWHERE, PAST, "renewed"),
        (T1_LESS_1H_LESS_1NS, PAST, "lapsed"),
        // `grace_until > now + 5m`: a nanosecond past the boundary, and on it.
        (PAST, T1_PLUS_5M_PLUS_1NS, "graced"),
        (PAST, T1_PLUS_5M, "lapsed"),
        // Declaration order: a row both guards select is renewed.
        (FUTURE, FUTURE, "renewed"),
    ] {
        let clock = Phased::new();
        let target = leases(Some(Box::new(clock.clone())));
        let lease = open(&target, expires_at, grace_until);
        assert_eq!(
            clock.reads(),
            1,
            "the setup decision reads the provider once"
        );
        clock.testing();
        clock.decision();
        let receipt = target.execute_command_recorded(renew(&lease, NOTE));
        assert_eq!(
            clock.reads(),
            2,
            "the tested decision reads the provider once"
        );
        assert_eq!(receipt.decision_time, Some(instant(T1)));
        let answer = receipt
            .answer
            .unwrap_or_else(|error| panic!("{expires_at}/{grace_until}: {error}"));
        assert_eq!(
            outcome(&answer),
            expected,
            "expires_at {expires_at}, grace_until {grace_until}"
        );
        let moved = if expected == "lapsed" {
            "Active"
        } else {
            "Renewed"
        };
        assert_eq!(lease_state(&target, &lease), moved);
    }
}

#[test]
fn a3_related_rows_are_decided_at_the_decision_instant() {
    for (banned_until, expected) in [
        (T1_PLUS_30S_PLUS_1NS, "banned-from-joining"),
        (T1_PLUS_30S, "joined"),
        (PAST, "joined"),
    ] {
        let clock = Phased::new();
        let target = leases(Some(Box::new(clock.clone())));
        let member = register(&target, banned_until);
        clock.testing();
        assert_eq!(
            send(&target, &clock, join(&member)),
            expected,
            "{banned_until}"
        );
        assert_eq!(clock.reads(), 2);
    }
    for (banned_until, embargo_until, expected) in [
        (T1_PLUS_30S_PLUS_1NS, FUTURE, "banned"),
        (T1_PLUS_30S, T1_LESS_2M, "embargoed"),
        (T1_PLUS_30S, T1_LESS_2M_LESS_1NS, "lent"),
        (PAST, PAST, "lent"),
    ] {
        let clock = Phased::new();
        let target = leases(Some(Box::new(clock.clone())));
        let member = register(&target, banned_until);
        let book = add_book(&target, embargo_until);
        clock.testing();
        assert_eq!(
            send(&target, &clock, lend(&member, &book, 1)),
            expected,
            "{banned_until}/{embargo_until}"
        );
        assert_eq!(clock.reads(), 3, "two setups and one decision");
    }
}

#[test]
fn a3_one_reading_decides_every_row_of_the_decision() {
    // Both rows straddle the decision instant: the member's ban ends a nanosecond inside its
    // bound and the book's embargo a nanosecond outside its own. Read with the decision's one
    // instant, neither refuses; a second reading an hour later would refuse neither either, but a
    // reading an hour earlier — the setup's, or any row reread from it — refuses the book.
    let clock = Phased::new();
    let target = leases(Some(Box::new(clock.clone())));
    let member = register(&target, T1_PLUS_30S);
    let book = add_book(&target, T1_LESS_2M_LESS_1NS);
    clock.testing();
    clock.decision();
    let receipt = target.execute_command_recorded(lend(&member, &book, 2));
    assert_eq!(outcome(&receipt.answer.expect("answered")), "lent");
    assert_eq!(receipt.decision_time, Some(instant(T1)));
    assert_eq!(
        clock.reads(),
        3,
        "one reading for the decision over both rows"
    );
}

#[test]
fn a3_missing_clock_is_unknown_only_where_a_row_needs_it() {
    // A fresh target per case with no provider, its rows made by its own creations, which read no
    // clock-dependent leaf.
    for (case, expected) in [
        ("blank-note", Some("blank-note")),
        ("unknown-lease", Some("unknown-lease")),
        ("renew", None),
        ("no-member-to-join", Some("no-member-to-join")),
        ("join", None),
        ("no-member", Some("no-member")),
        ("no-book", Some("no-book")),
        ("too-few", Some("too-few")),
        ("lend", None),
    ] {
        let target = leases(None);
        let lease = open(&target, FUTURE, FUTURE);
        let member = register(&target, PAST);
        let book = add_book(&target, PAST);
        let nobody = "00000000-0000-4000-8000-00000000dead";
        let sent = match case {
            "blank-note" => renew(&lease, ""),
            "unknown-lease" => renew(nobody, NOTE),
            "renew" => renew(&lease, NOTE),
            "no-member-to-join" => join(nobody),
            "join" => join(&member),
            "no-member" => lend(nobody, &book, 1),
            "no-book" => lend(&member, nobody, 1),
            "too-few" => lend(&member, &book, 0),
            "lend" => lend(&member, &book, 1),
            _ => unreachable!(),
        };
        let receipt = target.execute_command_recorded(sent);
        assert_eq!(
            receipt.decision_time, None,
            "{case}: no provider, no reading"
        );
        match expected {
            Some(expected) => {
                let answer = receipt
                    .answer
                    .unwrap_or_else(|error| panic!("{case} answers without a clock: {error}"));
                assert_eq!(outcome(&answer), expected, "{case}");
            }
            None => {
                assert!(
                    unsupported(&receipt.answer),
                    "{case}: a row's `now` leaf is needed and there is no clock: {:?}",
                    receipt.answer
                );
            }
        }
        // No Unknown decision moved the lease.
        assert_eq!(lease_state(&target, &lease), "Active", "{case}");
    }
}

/// A provider answering its first reading in a scenario forever after: the setup reading reused
/// for the decision under test.
struct ReuseFirst {
    inner: Arc<Phased>,
    first: Mutex<Option<DecisionInstant>>,
}

impl CommandClock for ReuseFirst {
    fn read(&self) -> Option<DecisionInstant> {
        let mut first = self.first.lock().unwrap();
        if first.is_none() {
            *first = self.inner.read();
        }
        *first
    }
}

/// A provider read once more for every row a decision reads after its first: the decision under
/// test decided with its second reading.
struct PerRow(Arc<Phased>);

impl CommandClock for PerRow {
    fn read(&self) -> Option<DecisionInstant> {
        let _first_row = self.0.read();
        self.0.read()
    }
}

/// The lending of a member and a book whose bounds straddle the decision instant, on `target`
/// read through `clock`: `lent` where the decision reads once at [`T1`].
fn straddling(target: &Interpreted, clock: &Phased) -> Result<String, TargetError> {
    let member = register(target, T1_PLUS_30S);
    let book = add_book(target, T1_LESS_2M_LESS_1NS);
    clock.testing();
    clock.decision();
    target
        .execute_command(lend(&member, &book, 1))
        .map(|answer| outcome(&answer))
}

#[test]
fn a3_setup_time_reuse_and_reread_faults_fail() {
    let clock = Phased::new();
    let healthy = leases(Some(Box::new(clock.clone())));
    assert_eq!(straddling(&healthy, &clock).unwrap(), "lent");
    assert_eq!(clock.reads(), 3);

    // Decided with the setup reading: both rows' bounds lie after it, and the member is banned.
    let clock = Phased::new();
    let reused = leases(Some(Box::new(ReuseFirst {
        inner: clock.clone(),
        first: Mutex::new(None),
    })));
    assert_eq!(straddling(&reused, &clock).unwrap(), "banned");

    // Decided with a second reading an hour later: the book's embargo is long over and the member's
    // ban too; the decision still answers `lent`, but the provider was read twice for it.
    let clock = Phased::new();
    let reread = leases(Some(Box::new(PerRow(clock.clone()))));
    let answered = straddling(&reread, &clock).unwrap();
    assert_eq!(clock.reads(), 6, "{answered}: two reads per decision");

    // And over the subject: renewed at T1, lapsed when decided an hour later.
    let clock = Phased::new();
    let reread = leases(Some(Box::new(PerRow(clock.clone()))));
    let lease = open(&reread, T1_LESS_1H, PAST);
    clock.testing();
    assert_eq!(send(&reread, &clock, renew(&lease, NOTE)), "lapsed");
    let clock = Phased::new();
    let healthy = leases(Some(Box::new(clock.clone())));
    let lease = open(&healthy, T1_LESS_1H, PAST);
    clock.testing();
    assert_eq!(send(&healthy, &clock, renew(&lease, NOTE)), "renewed");
}
