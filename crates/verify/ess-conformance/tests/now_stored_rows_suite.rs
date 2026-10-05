//! The suite `tests/fixtures/now-stored-rows.yaml` synthesizes, and what it decides
//! (`docs/design/expression-family-source22.md`, "A3: current time over stored and related rows";
//! beyond10x/ess#244 part a, unit U5).
//!
//! A stored instant ordered against `now` names no value a suite can carry, so synthesis decides its
//! witnesses at the fixed reference instant and arranges the row through the creator's input: the
//! value chosen for the row is sent to the creator as a `now_offset`, and travels through its
//! `sets:` into the row. The runner resolves it once per scenario from its wall clock; the target
//! decides the command under test by its own clock. A stored instant no creator input carries — one
//! the implementation generates, or a literal — is refused by name, never decided at the reference.
//!
//! The run controls hand the runner a wall clock fixed at `WALL_MS`, and the interpreter a
//! provider that answers `T0` for every arranging command and `T1`, a fraction of a second after
//! the wall, for the command under test. Every witness's boundary lies strictly between the two, so:
//!
//! - the healthy target passes every scenario, with one reading per decision;
//! - a target deciding with the scenario's first reading — the setup instant — fails every scenario
//!   whose witness lies on the other side of a boundary at `T0`;
//! - a target reading the provider a second time within one decision, two days later, fails every
//!   scenario whose witness lies on the other side two days later;
//! - a target with no provider is Unknown, and unsupported, exactly where a row's `now` leaf is
//!   needed, and answers every scenario decided before one.

mod support_now_stored;
mod support_occurrence_clock;

use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{
    now_offset, AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig,
    ScenarioStep, ScenarioValue,
};
use ess_primitives::time::Timestamp;
use support_now_stored::{Clocking, Staged, LEASES, WALL_MS};
use support_occurrence_clock::model;

const RENEWED: &str = "demo.leases.RenewLease/outcome/renewed";
const GRACED: &str = "demo.leases.RenewLease/outcome/graced";
const LAPSED: &str = "demo.leases.RenewLease/outcome/lapsed";
const BANNED_FROM_JOINING: &str = "demo.leases.Join/outcome/banned-from-joining";
const JOINED: &str = "demo.leases.Join/outcome/joined";
const BANNED: &str = "demo.leases.Lend/outcome/banned";
const EMBARGOED: &str = "demo.leases.Lend/outcome/embargoed";
const LENT: &str = "demo.leases.Lend/outcome/lent";

fn synthesis_of(text: &str) -> Synthesis {
    synthesize(&model(text))
}

/// The fixture with every ordering against `now` written against the fixed instant it names at the
/// synthesis reference instead: what synthesis refuses for it, it refuses for reasons of its own.
const FIXED: [(&str, &str); 4] = [
    (
        "expires_at >= now - 1h",
        "expires_at >= '2019-12-30T22:59:59Z'",
    ),
    (
        "grace_until > now + 5m",
        "grace_until > '2019-12-31T00:04:59Z'",
    ),
    (
        "banned_until > now + 30s",
        "banned_until > '2019-12-31T00:00:29Z'",
    ),
    (
        "embargo_until >= now - 2m",
        "embargo_until >= '2019-12-30T23:57:59Z'",
    ),
];

fn refusals_of(synthesis: &Synthesis) -> BTreeSet<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {:?}", refusal.scenario, refusal.cause))
        .collect()
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesis_of(LEASES);
    let fixed = FIXED
        .iter()
        .fold(LEASES.to_owned(), |text, (now, instant)| {
            assert!(text.contains(now), "{now}");
            text.replace(now, instant)
        });
    let control = refusals_of(&synthesis_of(&fixed));
    let refused = refusals_of(&synthesis);
    assert!(
        refused.is_subset(&control),
        "every refusal is the fixed-instant control's too:\n{:#?}\ncontrol:\n{control:#?}",
        refused.difference(&control).collect::<Vec<_>>()
    );
    synthesis.suite
}

fn steps<'s>(suite: &'s ConformanceSuite, id: &str) -> &'s [ScenarioStep] {
    &suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {id}: {:#?}",
                suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1
        .steps
}

/// The `now_offset` seconds the creator `command` is sent for `field` in scenario `id`, in order.
fn sent(suite: &ConformanceSuite, id: &str, command: &str, field: &str) -> Vec<i64> {
    steps(suite, id)
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(match &input[field] {
                ScenarioValue::NowOffset { seconds } => *seconds,
                other => panic!("{id}: {command}.{field} is sent as {other:?}, not a now_offset"),
            }),
            _ => None,
        })
        .collect()
}

#[test]
fn a3_every_stored_now_branch_is_witnessed() {
    let suite = suite();
    for id in [
        RENEWED,
        GRACED,
        LAPSED,
        BANNED_FROM_JOINING,
        JOINED,
        BANNED,
        EMBARGOED,
        LENT,
    ] {
        steps(&suite, id);
    }
    assert!(now_offset::used_by(&suite));
    // A3 adds no persisted predicate: the suite takes the `now_offset` pair, never 40/41.
    let major = suite.provenance.suite_version.major();
    assert!((now_offset::ORDINARY..40).contains(&major), "suite/{major}");
}

/// A witness a second either side of its boundary, never on it, or the plain witness — or a further
/// instance's — a whole number of days and a second past the reference.
fn beside(seconds: i64, boundary: i64) -> bool {
    seconds == boundary - 1 || seconds == boundary + 1 || (seconds > 0 && seconds % 86_400 == 1)
}

#[test]
fn a3_stored_instants_are_sent_through_the_creator_as_now_offsets() {
    let suite = suite();
    let lease = |id: &str, field: &str| sent(&suite, id, "demo.leases.OpenLease", field);
    let member = |id: &str| sent(&suite, id, "demo.leases.RegisterMember", "banned_until");
    let book = |id: &str| sent(&suite, id, "demo.leases.AddBook", "embargo_until");
    // `expires_at >= now - 1h`, `grace_until > now + 5m`.
    for (id, renews, graces) in [
        (RENEWED, Some(true), None),
        (GRACED, Some(false), Some(true)),
        (LAPSED, Some(false), Some(false)),
    ] {
        let expires = lease(id, "expires_at");
        let grace = lease(id, "grace_until");
        assert_ne!(expires.len(), 0, "{id}: the lease is arranged");
        let (expires, grace) = (*expires.last().unwrap(), *grace.last().unwrap());
        assert!(beside(expires, -3600), "{id}: expires_at {expires}");
        assert!(beside(grace, 300), "{id}: grace_until {grace}");
        if let Some(renews) = renews {
            assert_eq!(expires >= -3600, renews, "{id}: expires_at {expires}");
        }
        if let Some(graces) = graces {
            assert_eq!(grace > 300, graces, "{id}: grace_until {grace}");
        }
    }
    // `banned_until > now + 30s`. A related row the scenario names sits between two decoys, each
    // a witness of its own: every member sent is a witness, and one is on the branch's side.
    for (id, bans) in [(BANNED_FROM_JOINING, true), (JOINED, false), (BANNED, true)] {
        let sent = member(id);
        assert!(
            sent.iter().all(|banned| beside(*banned, 30)),
            "{id}: banned_until {sent:?}"
        );
        assert!(
            sent.iter().any(|banned| (*banned > 30) == bans),
            "{id}: banned_until {sent:?}"
        );
    }
    // `embargo_until >= now - 2m`, behind a member who is not banned.
    for (id, embargoes) in [(EMBARGOED, true), (LENT, false)] {
        let members = member(id);
        assert!(
            members.iter().any(|banned| *banned <= 30),
            "{id}: banned_until {members:?}"
        );
        let books = book(id);
        assert!(
            books.iter().all(|embargo| beside(*embargo, -120)),
            "{id}: embargo_until {books:?}"
        );
        assert!(
            books.iter().any(|embargo| (*embargo >= -120) == embargoes),
            "{id}: embargo_until {books:?}"
        );
    }
}

#[test]
fn a3_a_row_read_back_holds_the_instant_the_creator_was_sent() {
    let suite = suite();
    let expires = *sent(&suite, LAPSED, "demo.leases.OpenLease", "expires_at")
        .last()
        .unwrap();
    let viewed = steps(&suite, LAPSED).iter().any(|step| match step {
        ScenarioStep::ExpectView { expectation, .. }
        | ScenarioStep::EventuallyView { expectation, .. } => {
            format!("{expectation:?}").contains(&format!("NowOffset {{ seconds: {expires} }}"))
        }
        _ => false,
    });
    assert!(viewed, "{:#?}", steps(&suite, LAPSED));
}

/// `LEASES` with `before` replaced by `after`, once.
fn edited(before: &str, after: &str) -> String {
    assert!(LEASES.contains(before), "{before}");
    LEASES.replacen(before, after, 1)
}

/// `LEASES` with a `Window` structure stored beside the expiry, and the renewal guarded by the
/// instant inside it: a member no `now_offset` can carry, which replaces a whole input field.
fn nested() -> String {
    [
        (
            "  - {name: demo.leases.LeaseId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.leases.LeaseId, kind: newtype, of: Uuid}\n  - name: demo.leases.Window\n    kind: struct\n    fields:\n      - {name: ends_at, type: Timestamp}\n",
        ),
        (
            "      - {name: grace_until, type: Timestamp}\n    lifecycle:",
            "      - {name: grace_until, type: Timestamp}\n      - {name: window, type: demo.leases.Window}\n    lifecycle:",
        ),
        (
            "      - {name: grace_until, type: Timestamp}\n    outcomes:",
            "      - {name: grace_until, type: Timestamp}\n      - {name: window, type: demo.leases.Window}\n    outcomes:",
        ),
        (
            "sets: {expires_at: input.expires_at, grace_until: input.grace_until}",
            "sets: {expires_at: input.expires_at, grace_until: input.grace_until, window: input.window}",
        ),
        (
            "when_subject: {predicate: expires_at >= now - 1h}",
            "when_subject: {predicate: window.ends_at >= now - 1h}",
        ),
    ]
    .iter()
    .fold(LEASES.to_owned(), |text, (before, after)| {
        assert!(text.contains(before), "{before}");
        text.replacen(before, after, 1)
    })
}

#[test]
fn a3_a_stored_instant_no_creator_input_carries_is_refused_by_name() {
    for (case, text, path) in [
        (
            "generated",
            edited(
                "sets: {expires_at: input.expires_at, grace_until: input.grace_until}",
                "sets: {expires_at: {generated: true}, grace_until: input.grace_until}",
            ),
            "demo.leases.Lease.expires_at",
        ),
        (
            "inside a structure",
            nested(),
            "demo.leases.Lease.window.ends_at",
        ),
    ] {
        let synthesis = synthesis_of(&text);
        let refusals: Vec<String> = synthesis
            .refusals
            .iter()
            .map(|refusal| format!("{:?} {:?}", refusal.scenario, refusal.cause))
            .collect();
        let named: Vec<&String> = refusals
            .iter()
            .filter(|line| {
                line.contains("RenewLease") && line.contains(path) && line.contains("now_offset")
            })
            .collect();
        assert_ne!(
            named.len(),
            0,
            "{case}: a renewal refused naming the stored path: {refusals:#?}"
        );
        // No scenario decides the stored instant at the reference: none sends the renewal.
        for (id, scenario) in &synthesis.suite.scenarios {
            let sends_renewal = scenario.steps.iter().any(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "demo.leases.RenewLease")
            });
            let decided_by_row = [RENEWED, GRACED, LAPSED].contains(&id.to_string().as_str());
            assert!(
                !(sends_renewal && decided_by_row),
                "{case}: {id} decides a stored instant nobody can carry"
            );
        }
        // The rows the generated or nested field does not touch are still witnessed.
        steps(&synthesis.suite, BANNED);
        steps(&synthesis.suite, LENT);
    }
}

/// Every scenario's status against the interpreter wired as `clocking`, with the runner's wall at
/// [`WALL_MS`]; and the provider's reads and the decisions it was read for.
fn run(clocking: Clocking) -> (BTreeMap<String, Status>, (usize, usize)) {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Staged::new(model(LEASES), &suite, clocking);
    let runner = Runner::new(
        RunnerConfig::default(),
        now_offset::WithWall::new(AdvancingClock::default(), || {
            Timestamp::from_epoch_millis(WALL_MS)
        }),
        Ids::for_suite(&suite),
    );
    let report = runner.run_admitted(&admitted, &target).into_report();
    for scenario in &report.scenarios {
        if scenario.status != Status::Passed {
            let diagnostics: Vec<String> = scenario
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            eprintln!("{clocking:?} {}: {diagnostics:#?}", scenario.scenario);
        }
    }
    let statuses = report
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    (statuses, target.stage.reads())
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> BTreeSet<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

#[test]
fn a3_the_healthy_target_passes_reading_once_per_decision() {
    let (statuses, (reads, decisions)) = run(Clocking::Healthy);
    assert_eq!(not_passed(&statuses), BTreeSet::new(), "{statuses:#?}");
    assert_eq!(reads, decisions, "one reading per decision");
    assert_ne!(decisions, 0);
}

#[test]
fn a3_a_target_deciding_with_the_setup_reading_fails() {
    let (statuses, _) = run(Clocking::SetupReused);
    let failed = not_passed(&statuses);
    // At the setup instant every lease is renewable and every member banned. (`lapsed` sends the
    // renewal for a lease nobody holds first, so its first reading is already a decision's.)
    for id in [GRACED, JOINED, EMBARGOED, LENT] {
        assert!(failed.contains(id), "{id} passed: {failed:#?}");
    }
    for id in [RENEWED, BANNED_FROM_JOINING, BANNED] {
        assert!(!failed.contains(id), "{id} failed: {failed:#?}");
    }
}

#[test]
fn a3_a_target_rereading_within_a_decision_fails() {
    let (statuses, (reads, decisions)) = run(Clocking::Reread);
    assert_eq!(reads, 2 * decisions);
    let failed = not_passed(&statuses);
    // Two days later no lease is renewable or graced, and no ban or embargo holds.
    for id in [RENEWED, GRACED, BANNED_FROM_JOINING, BANNED, EMBARGOED] {
        assert!(failed.contains(id), "{id} passed: {failed:#?}");
    }
    for id in [LAPSED, JOINED, LENT] {
        assert!(!failed.contains(id), "{id} failed: {failed:#?}");
    }
}

#[test]
fn a3_a_target_with_no_clock_is_unsupported_exactly_where_a_row_needs_one() {
    let (statuses, (reads, _)) = run(Clocking::Absent);
    assert_eq!(reads, 0);
    let failed = not_passed(&statuses);
    for id in [
        RENEWED,
        GRACED,
        LAPSED,
        BANNED_FROM_JOINING,
        JOINED,
        BANNED,
        EMBARGOED,
        LENT,
    ] {
        assert_eq!(statuses[id], Status::Unsupported, "{id}");
    }
    // Every scenario decided before a row's `now` leaf passes without a clock.
    for (id, status) in &statuses {
        if !failed.contains(id.as_str()) {
            assert_eq!(*status, Status::Passed);
        }
    }
    for id in [
        "demo.leases.RenewLease/outcome/blank-note",
        "demo.leases.Join/outcome/no-member-to-join",
        "demo.leases.Lend/outcome/no-member",
        "demo.leases.Lend/outcome/no-book",
        "demo.leases.Lend/outcome/too-few",
    ] {
        assert_eq!(statuses.get(id), Some(&Status::Passed), "{id}: {failed:#?}");
    }
}
