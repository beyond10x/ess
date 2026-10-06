//! Calendar-window guards in synthesis, the native interpreter and the Rust runner (beyond10x/ess#244
//! part b, `docs/design/calendar-window-guards.md`).
//!
//! `tests/fixtures/calendar-windows.yaml` holds four windows: `Schedule` over an input at `+01:00`,
//! `Maintain` over an input across midnight at `-05:00`, `Promote` over a stored instant at `+01:00`,
//! and `Deploy` over `now`. Synthesis witnesses the first three a second either side of every
//! boundary; it refuses the fourth by name, because no suite step sets a target's clock, and the
//! interpreter's scripted command clock accepts it instead. The healthy interpreter passes the
//! synthesized suite; each faulty target — local host time, the offset ignored, `to` inclusive, the
//! midnight crossing attributed to the instant's own day — fails named scenarios.

mod support_go;
mod support_occurrence_clock;

use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, MutantClass, Mutation, Verdict};
use ess_conformance::synthesize::{synthesize, RefusalCause};
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use ess_primitives::time::Rfc3339Instant;
use support_occurrence_clock::{begin, model, outcome, request, text, Scripted};

const RELEASES: &str = include_str!("fixtures/calendar-windows.yaml");

/// Every instant `command` is sent at `field` in the scenario `id`, written back in UTC: a further
/// witness is spelled at an offset that tells a reader of its written clock apart, and it is the
/// instant that is claimed here.
fn sent(suite: &ConformanceSuite, id: &str, command: &str, field: &str) -> BTreeSet<String> {
    spelled(suite, id, command, field)
        .iter()
        .map(|text| {
            Rfc3339Instant::parse_rfc3339(text)
                .map_or_else(|| text.clone(), Rfc3339Instant::to_rfc3339)
        })
        .collect()
}

/// Every literal text `command` is sent at `field` in the scenario `id`, as written.
fn spelled(suite: &ConformanceSuite, id: &str, command: &str, field: &str) -> BTreeSet<String> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1;
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => match input.get(field) {
                Some(ScenarioValue::Literal {
                    value: Node::Text(text),
                }) => Some(text.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn window_of(text: &str) -> ess_primitives::window::CalendarWindow {
    let node: Node = serde_yaml::from_str(text).expect("yaml");
    let Predicate::Window(window) = Predicate::from_node(&node).expect("a window") else {
        panic!("a window");
    };
    *window
}

#[test]
fn window_synthesis_witnesses_each_side_of_each_boundary_of_an_input_window() {
    let synthesis = synthesize(&model(RELEASES));
    let suite = &synthesis.suite;
    let scheduled = sent(
        suite,
        "demo.releases.Schedule/outcome/scheduled",
        "demo.releases.Schedule",
        "starts_at",
    );
    // Monday 08:00 and Thursday 15:59:59 at +01:00, spelled in UTC.
    for inside in ["2020-01-06T07:00:00Z", "2020-01-09T14:59:59Z"] {
        assert!(scheduled.contains(inside), "{inside}: {scheduled:?}");
    }
    let outside = sent(
        suite,
        "demo.releases.Schedule/outcome/outside-hours",
        "demo.releases.Schedule",
        "starts_at",
    );
    // Monday 07:59:59, Thursday 16:00 and Friday 08:00 at +01:00.
    for refused in [
        "2020-01-06T06:59:59Z",
        "2020-01-09T15:00:00Z",
        "2020-01-10T07:00:00Z",
    ] {
        assert!(outside.contains(refused), "{refused}: {outside:?}");
    }
    // Friday 22:00 to Saturday 02:00 at -05:00 is Saturday 03:00 to 07:00 UTC.
    let allowed = sent(
        suite,
        "demo.releases.Maintain/outcome/allowed",
        "demo.releases.Maintain",
        "requested_at",
    );
    for inside in ["2020-01-11T03:00:00Z", "2020-01-11T06:59:59Z"] {
        assert!(allowed.contains(inside), "{inside}: {allowed:?}");
    }
    let closed = sent(
        suite,
        "demo.releases.Maintain/outcome/outside-maintenance",
        "demo.releases.Maintain",
        "requested_at",
    );
    for refused in ["2020-01-11T02:59:59Z", "2020-01-11T07:00:00Z"] {
        assert!(closed.contains(refused), "{refused}: {closed:?}");
    }
    // Further witnesses are spelled at an offset under which their written clock, read as UTC, is
    // on the other side: a target comparing spellings is told apart.
    let offsets = spelled(
        suite,
        "demo.releases.Schedule/outcome/outside-hours",
        "demo.releases.Schedule",
        "starts_at",
    );
    assert!(
        offsets.iter().any(|text| !text.ends_with('Z')),
        "{offsets:?}"
    );
    assert!(
        !ess_conformance::expression_format::used_by(suite),
        "a window in a guard adds no suite vocabulary"
    );
}

#[test]
fn window_a_suite_carrying_one_is_read_from_suite_40() {
    use ess_conformance::expression_format::admit_predicate;
    let window =
        r#"{"window":{"at":"ready_at","days":["mon"],"from":"08:00","offset":"Z","to":"16:00"}}"#;
    let refused = admit_predicate(window, 39).expect_err("a window relabelled /39 is refused");
    assert!(
        format!("{refused:?}").contains("calendar window"),
        "{refused:?}"
    );
    admit_predicate(window, 40).expect("read from /40");
    admit_predicate(r#"{"window":{"gte":3}}"#, 39).expect("a fact named `window` is no window");
}

#[test]
fn window_synthesis_arranges_a_stored_instant_through_its_creator() {
    let synthesis = synthesize(&model(RELEASES));
    let window = window_of(
        r#"{window: {at: ready_at, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}}"#,
    );
    let inside = |text: &String| {
        window.contains(Rfc3339Instant::parse_rfc3339(text).expect("an instant written"))
    };
    let promoted = sent(
        &synthesis.suite,
        "demo.releases.Promote/outcome/promoted",
        "demo.releases.Prepare",
        "ready_at",
    );
    assert!(promoted.iter().any(inside), "{promoted:?}");
    let not_ready = sent(
        &synthesis.suite,
        "demo.releases.Promote/outcome/not-ready",
        "demo.releases.Prepare",
        "ready_at",
    );
    assert!(not_ready.iter().any(|text| !inside(text)), "{not_ready:?}");
    // No scenario a window decides is refused. (`Promote/outcome/unknown-release` is: an
    // `unknown_instance:` branch beside a `when_subject:` predicate is a refusal of its own,
    // with or without a window.)
    let decided = [
        "demo.releases.Schedule/outcome/scheduled",
        "demo.releases.Schedule/outcome/outside-hours",
        "demo.releases.Maintain/outcome/allowed",
        "demo.releases.Maintain/outcome/outside-maintenance",
        "demo.releases.Promote/outcome/promoted",
        "demo.releases.Promote/outcome/not-ready",
    ];
    for refusal in &synthesis.refusals {
        let named = refusal
            .scenario
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        assert!(!decided.contains(&named.as_str()), "{refusal:#?}");
    }
    for id in decided {
        assert!(
            synthesis
                .suite
                .scenarios
                .keys()
                .any(|scenario| scenario.to_string() == id),
            "{id} is synthesized"
        );
    }
}

#[test]
fn window_over_now_is_refused_by_name_and_sent_by_no_scenario() {
    let synthesis = synthesize(&model(RELEASES));
    // Every refused scenario that would send `Deploy`: its own outcomes and the state refusals.
    let refused: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal.scenario.as_ref().is_some_and(|id| {
                id.to_string()
                    .split('/')
                    .any(|segment| segment == "demo.releases.Deploy")
            })
        })
        .collect();
    assert!(refused.len() >= 4, "{:#?}", synthesis.refusals);
    for refusal in &refused {
        let RefusalCause::NoWitness(gap) = &refusal.cause else {
            panic!("a named no-witness refusal: {refusal:#?}");
        };
        assert!(gap.path.contains("calendar window"), "{gap:#?}");
        assert!(gap.path.contains("at now"), "{gap:#?}");
        assert!(gap.reason.contains("clock"), "{gap:#?}");
    }
    for (id, scenario) in &synthesis.suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { command, .. } = step {
                assert_ne!(command.to_string(), "demo.releases.Deploy", "{id}");
            }
        }
    }
}

#[test]
fn window_a_stored_field_also_ordered_against_now_is_refused_by_name() {
    // `Deploy` decided by a window over its own input instead of `now` is sent, so its
    // `ready_at <= now - 1h` orders the stored field `Promote` holds to a window. A value chosen at a
    // window boundary is a 2020 instant, which `now` reads one way at the synthesis reference and the
    // other at every run: refused by name, never sent as a `now_offset` on another weekday.
    let unclocked = RELEASES
        .replacen(
            "  - name: demo.releases.Deploy\n    input:\n      - {name: release_id, type: demo.releases.ReleaseId}\n",
            "  - name: demo.releases.Deploy\n    input:\n      - {name: release_id, type: demo.releases.ReleaseId}\n      - {name: requested_at, type: Timestamp}\n",
            1,
        )
        .replacen(
            r#"window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#,
            r#"window: {at: requested_at, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#,
            1,
        );
    assert!(!unclocked.contains("window: {at: now"));
    let synthesis = synthesize(&model(&unclocked));
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == "demo.releases.Promote/outcome/promoted")
        })
        .unwrap_or_else(|| panic!("{:#?}", synthesis.refusals));
    let RefusalCause::NoWitness(gap) = &refusal.cause else {
        panic!("a named no-witness refusal: {refusal:#?}");
    };
    assert!(
        gap.reason.contains("fixed instant") && gap.path.contains("ready_at"),
        "{gap:#?}"
    );
    for (id, scenario) in &synthesis.suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
                assert!(
                    command.to_string() != "demo.releases.Prepare"
                        || !matches!(input.get("ready_at"), Some(ScenarioValue::NowOffset { .. })),
                    "{id}"
                );
            }
        }
    }
}

// ---- the Rust runner, healthy and faulty ------------------------------------------------------

/// One fault written into the fixture's windows, the same position a wrong implementation is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// Every offset replaced by a host zone, `-07:00`.
    HostTime,
    /// Every offset replaced by `Z`.
    OffsetIgnored,
    /// Every `to` a minute later: inclusive at `to` itself.
    InclusiveTo,
    /// The midnight-crossing window split into `from`–`24:00` and `00:00`–`to` on the same day.
    CrossingOwnDay,
}

const FAULTS: [Fault; 4] = [
    Fault::HostTime,
    Fault::OffsetIgnored,
    Fault::InclusiveTo,
    Fault::CrossingOwnDay,
];

/// The fixture with `fault` written in.
pub fn faulty(fault: Fault) -> String {
    let crossing =
        r#"window: {at: requested_at, days: [fri], from: "22:00", to: "02:00", offset: "-05:00"}"#;
    assert!(RELEASES.contains(crossing));
    match fault {
        Fault::HostTime => RELEASES
            .replace(r#"offset: "+01:00""#, r#"offset: "-07:00""#)
            .replace(r#"offset: "-05:00""#, r#"offset: "-07:00""#),
        Fault::OffsetIgnored => RELEASES
            .replace(r#"offset: "+01:00""#, "offset: Z")
            .replace(r#"offset: "-05:00""#, "offset: Z"),
        Fault::InclusiveTo => RELEASES
            .replace(r#"to: "16:00""#, r#"to: "16:01""#)
            .replace(r#"to: "02:00""#, r#"to: "02:01""#),
        Fault::CrossingOwnDay => RELEASES.replace(
            crossing,
            "any:\n            - window: {at: requested_at, days: [fri], from: \"22:00\", to: \"24:00\", offset: \"-05:00\"}\n            - window: {at: requested_at, days: [fri], from: \"00:00\", to: \"02:00\", offset: \"-05:00\"}",
        ),
    }
}

/// The scenarios each fault fails (read off the run, then held).
pub fn killed(fault: Fault) -> &'static [&'static str] {
    match fault {
        // Either way the window is read in another offset than its own: every accepting boundary
        // lands outside it.
        Fault::HostTime | Fault::OffsetIgnored => &[
            "demo.releases.Schedule/outcome/scheduled",
            "demo.releases.Maintain/outcome/allowed",
            "demo.releases.Promote/outcome/promoted",
        ],
        Fault::InclusiveTo => &[
            "demo.releases.Schedule/outcome/outside-hours",
            "demo.releases.Maintain/outcome/outside-maintenance",
            "demo.releases.Promote/outcome/not-ready",
        ],
        Fault::CrossingOwnDay => &["demo.releases.Maintain/outcome/allowed"],
    }
}

fn verdicts<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, String> {
    support_go::rust_outcomes(suite, target)
}

#[test]
fn window_the_healthy_interpreter_passes_and_each_fault_fails_named_scenarios() {
    let suite = synthesize(&model(RELEASES)).suite;
    let healthy = verdicts(&suite, &Interpreted::for_model(model(RELEASES)));
    assert!(healthy.len() >= 8, "{healthy:#?}");
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    for fault in FAULTS {
        let run = verdicts(&suite, &Interpreted::for_model(model(&faulty(fault))));
        let failed = support_go::not_passed(&run);
        for id in killed(fault) {
            assert!(failed.contains(id), "{fault:?}: {id} passed: {run:#?}");
        }
        // Only scenarios sending a command a window decides fail: every other one still passes.
        for id in &failed {
            assert!(
                id.split('/').any(|segment| [
                    "demo.releases.Schedule",
                    "demo.releases.Maintain",
                    "demo.releases.Promote"
                ]
                .contains(&segment)),
                "{fault:?}: {id} failed: {run:#?}"
            );
        }
    }
}

// ---- `at: now`, at the interpreter's scripted command clock -----------------------------------

/// A release prepared ready at `ready_at` and promoted, deployed at the clock reading `now`: the
/// outcome, and how many times the clock was read while `Deploy` decided.
fn deploy(text_of_model: &str, ready_at: &str, now: &str) -> (String, usize) {
    // Prepare and Promote read the clock once each; Deploy reads it third.
    let clock = Scripted::new(&["2020-01-06T09:00:00Z", "2020-01-06T09:00:00Z", now]);
    let target = Interpreted::for_model(model(text_of_model)).with_command_clock(clock.clone());
    begin(&target);
    let prepared = target
        .execute_command(request(
            "demo.releases.Prepare",
            &[("ready_at", text(ready_at))],
        ))
        .unwrap_or_else(|error| panic!("prepared: {error}"));
    assert_eq!(outcome(&prepared), "prepared");
    let id = prepared.direct_events[0].payload["release_id"].clone();
    let promoted = target
        .execute_command(request(
            "demo.releases.Promote",
            &[("release_id", id.clone())],
        ))
        .unwrap_or_else(|error| panic!("promoted: {error}"));
    assert_eq!(outcome(&promoted), "promoted", "ready at {ready_at}");
    let before = clock.reads();
    let deployed = target
        .execute_command(request("demo.releases.Deploy", &[("release_id", id)]))
        .unwrap_or_else(|error| panic!("deployed: {error}"));
    (outcome(&deployed), clock.reads() - before)
}

/// Monday 2020-01-06 10:00 at +01:00: inside the promotion window.
const READY: &str = "2020-01-06T09:00:00Z";

#[test]
fn window_a_stored_instant_written_at_another_offset_is_read_as_its_instant() {
    // `Promote` holds the stored `ready_at` to Monday to Thursday 08:00-16:00 at +01:00. Its written
    // clock is never what is compared: 02:30 at -05:00 is 08:30 at +01:00 (inside), and 08:30 at
    // +08:00 is 01:30 at +01:00 (outside).
    for (ready_at, expected) in [
        ("2020-01-06T02:30:00-05:00", "promoted"),
        ("2020-01-06T08:30:00+08:00", "not-ready"),
        ("2020-01-06T07:59:59+01:00", "not-ready"),
        ("2020-01-06T08:00:00+01:00", "promoted"),
    ] {
        let target = Interpreted::for_model(model(RELEASES));
        begin(&target);
        let prepared = target
            .execute_command(request(
                "demo.releases.Prepare",
                &[("ready_at", text(ready_at))],
            ))
            .unwrap_or_else(|error| panic!("prepared: {error}"));
        let id = prepared.direct_events[0].payload["release_id"].clone();
        let promoted = target
            .execute_command(request("demo.releases.Promote", &[("release_id", id)]))
            .unwrap_or_else(|error| panic!("{ready_at}: {error}"));
        assert_eq!(outcome(&promoted), expected, "{ready_at}");
    }
}

#[test]
fn window_over_now_each_side_of_each_boundary_at_the_scripted_clock() {
    for (now, expected) in [
        // Monday 2020-01-13 08:00 at +01:00, and a second before.
        ("2020-01-13T07:00:00Z", "deployed"),
        ("2020-01-13T06:59:59Z", "frozen"),
        // Thursday 2020-01-16 15:59:59 at +01:00, and 16:00.
        ("2020-01-16T14:59:59Z", "deployed"),
        ("2020-01-16T15:00:00Z", "frozen"),
        // Friday 2020-01-17 08:00, a day the window does not list.
        ("2020-01-17T07:00:00Z", "frozen"),
        // Sunday 2020-01-12 23:30 UTC is Monday 00:30 at +01:00: before `from`.
        ("2020-01-12T23:30:00Z", "frozen"),
    ] {
        let (taken, reads) = deploy(RELEASES, READY, now);
        assert_eq!(taken, expected, "{now}");
        assert_eq!(reads, 1, "one clock reading per decision at {now}");
    }
}

#[test]
fn window_over_now_the_same_instant_lands_on_different_sides_under_two_offsets() {
    // Monday 2020-01-13 07:30 UTC is 08:30 at +01:00 (inside) and 07:30 at Z (outside).
    let now = "2020-01-13T07:30:00Z";
    assert_eq!(deploy(RELEASES, READY, now).0, "deployed");
    let utc = RELEASES.replacen(
        r#"window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#,
        r#"window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: Z}"#,
        1,
    );
    assert_ne!(utc, RELEASES);
    assert_eq!(deploy(&utc, READY, now).0, "frozen");
}

#[test]
fn window_over_now_and_a_stored_now_read_one_instant() {
    // Monday 2020-01-13 09:30 at +01:00: inside the window. A release ready at 08:45 (inside the
    // promotion window too) became ready 45 minutes earlier, so `ready_at <= now - 1h` refuses it
    // at that same instant; one ready at 08:15 deploys.
    let (taken, reads) = deploy(RELEASES, "2020-01-13T07:45:00Z", "2020-01-13T08:30:00Z");
    assert_eq!(taken, "too-early");
    assert_eq!(reads, 1);
    let (taken, reads) = deploy(RELEASES, "2020-01-13T07:15:00Z", "2020-01-13T08:30:00Z");
    assert_eq!(taken, "deployed");
    assert_eq!(reads, 1);
}

/// The `Deploy` window over `now` as the fixture writes it.
const DEPLOY_WINDOW: &str = r#"window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#;

#[test]
fn window_over_now_faulty_targets_decide_otherwise_at_the_scripted_clock() {
    // Each fault written into the `Deploy` window alone, at an instant the healthy model decides
    // one way and the fault the other; the promotion window stays healthy.
    for (fault, written, now) in [
        // Host time at -07:00: Monday 08:00 at +01:00 is Monday 00:00 at -07:00.
        (
            Fault::HostTime,
            DEPLOY_WINDOW.replace(r#""+01:00""#, r#""-07:00""#),
            "2020-01-13T07:00:00Z",
        ),
        // The offset ignored: Monday 08:00 at +01:00 is 07:00 UTC.
        (
            Fault::OffsetIgnored,
            DEPLOY_WINDOW.replace(r#""+01:00""#, "Z"),
            "2020-01-13T07:00:00Z",
        ),
        // `to` inclusive: Thursday 16:00 at +01:00.
        (
            Fault::InclusiveTo,
            DEPLOY_WINDOW.replace(r#"to: "16:00""#, r#"to: "16:01""#),
            "2020-01-16T15:00:00Z",
        ),
    ] {
        assert!(RELEASES.contains(DEPLOY_WINDOW));
        let wrong = RELEASES.replacen(DEPLOY_WINDOW, &written, 1);
        let healthy = deploy(RELEASES, READY, now).0;
        let faulted = deploy(&wrong, READY, now).0;
        assert_ne!(healthy, faulted, "{fault:?} at {now}");
    }
}

#[test]
fn window_over_now_without_a_clock_is_unsupported_not_decided() {
    let target = Interpreted::for_model(model(RELEASES));
    begin(&target);
    let prepared = target
        .execute_command(request(
            "demo.releases.Prepare",
            &[("ready_at", text(READY))],
        ))
        .expect("a creation reads no clock");
    let id = prepared.direct_events[0].payload["release_id"].clone();
    let promoted = target
        .execute_command(request(
            "demo.releases.Promote",
            &[("release_id", id.clone())],
        ))
        .expect("a window over a stored instant reads no clock");
    assert_eq!(outcome(&promoted), "promoted");
    // Without a command clock the interpreter reads none, host or otherwise: the guard is Unknown
    // and the decision unsupported, as for any other `now` (`Interpreted::with_command_clock`).
    let error = target
        .execute_command(request("demo.releases.Deploy", &[("release_id", id)]))
        .expect_err("a window over now without a clock is not decided");
    let said = error.to_string();
    assert!(said.contains("is Unknown over this input"), "{said}");
    assert!(said.contains("Window"), "{said}");
}

// ---- mutation -----------------------------------------------------------------------------------

#[test]
fn window_mutants_move_a_boundary_and_the_synthesized_suite_kills_them() {
    let mut texts = ess_compiler::source::SourceMap::new();
    texts.insert("releases.yaml".to_owned(), RELEASES.to_owned());
    let raw = ess_domain::spec::RawSpecFile::parse(RELEASES).expect("parses");
    let documents = vec![(ess_domain::system::Source::new("releases.yaml"), raw)];
    let original = mutate::compile(documents.clone(), &texts).expect("compiles");
    let mutants = mutate::mutants(&documents, &[MutantClass::GuardBoundary]);
    let windows: Vec<_> = mutants
        .iter()
        .filter(|mutant| mutant.change.contains("window("))
        .collect();
    let changes: Vec<&str> = windows
        .iter()
        .map(|mutant| mutant.change.as_str())
        .collect();
    for expected in [
        "becomes `window(at starts_at, mon tue wed thu, 08:01-16:00, +01:00)`",
        "becomes `window(at starts_at, mon tue wed thu, 08:00-16:01, +01:00)`",
        "becomes `window(at requested_at, fri, 22:01-02:00, -05:00)`",
        "becomes `window(at requested_at, fri, 22:00-02:01, -05:00)`",
    ] {
        assert!(
            changes.iter().any(|change| change.contains(expected)),
            "{expected}: {changes:#?}"
        );
    }
    assert!(windows
        .iter()
        .any(|mutant| matches!(mutant.mutation, Mutation::GuardOutward { .. })));
    for mutant in &windows {
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        // A window over an input is witnessed at each boundary, so moving one is caught. One over
        // `now` has no scenario (no suite step sets a target's clock): never a survivor, and
        // never a kill either.
        let expected = if mutant.id.contains("demo.releases.Deploy/") {
            Verdict::Unwitnessed
        } else {
            Verdict::Killed
        };
        assert_eq!(entry.verdict, expected, "{}: {entry:#?}", mutant.id);
    }
}

/// A window a caller resolved is two `Timestamp` parameters (beyond10x/ess#439,
/// `docs/design/read-api-view-idioms.md`); the view filter itself still reads no clock. `now` and
/// `window:` there keep their `type_mismatch` refusals beside the parameters.
#[test]
fn window_parameters_stay_refused() {
    use ess_domain::spec::{RawSpecFile, Specification};
    use ess_domain::system::Source;
    const RANGE: &str = include_str!("fixtures/aggregate-timestamp-range.yaml");
    const FILTER: &str = "filter: [started_at >= param.from, started_at < param.to]";
    for (filter, phrase) in [
        (
            "filter: [started_at >= param.from, started_at < param.to, started_at >= now - 1h]",
            "with the current time, `now - 1h`, which is admitted only in a command outcome's",
        ),
        (
            "filter: [started_at >= param.from, started_at < param.to, {window: {at: started_at, \
             days: [mon], from: \"08:00\", to: \"16:00\", offset: Z}}]",
            "is a calendar window, which is admitted only in a command outcome's guard",
        ),
    ] {
        assert!(RANGE.contains(FILTER), "the fixture reads {FILTER}");
        let text = RANGE.replacen(FILTER, filter, 1);
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
        let errors = Specification::assemble([(Source::new("range.yaml"), raw)])
            .err()
            .unwrap_or_else(|| panic!("refused: {filter}"))
            .to_string();
        assert!(errors.contains("[type_mismatch]"), "{errors}");
        assert!(errors.contains(phrase), "{phrase}:\n{errors}");
    }
}
