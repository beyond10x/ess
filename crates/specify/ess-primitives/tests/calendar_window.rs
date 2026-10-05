//! Calendar-window guards (beyond10x/ess#244 part b, `docs/design/calendar-window-guards.md`).
//!
//! `{window: {at, days, from, to, offset}}` holds where the instant at `at`, shifted by a fixed UTC
//! offset, falls on a listed weekday between `from` (inclusive) and `to` (exclusive); a window with
//! `from` after `to` crosses midnight and belongs to the day it opens. No zone database: a named
//! zone is refused by name, and Rust, Go and TypeScript answer the shared vectors alike.

use ess_primitives::facts::{FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{reading_source22_operands, Predicate, Truth, WithNow};
use ess_primitives::time::Rfc3339Instant;
use ess_primitives::window::{CalendarWindow, WindowInstant};

fn read(value: serde_json::Value) -> Result<Predicate, String> {
    serde_json::from_value::<Predicate>(value).map_err(|error| error.to_string())
}

fn window(fields: &serde_json::Value) -> Predicate {
    read(serde_json::json!({ "window": fields })).expect("the window reads")
}

fn instant(text: &str) -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339(text).expect("an instant")
}

fn at(text: &str) -> FactStore {
    let mut store = FactStore::new();
    store.set_path("t", FactValue::text(text));
    store
}

fn mon_to_thu() -> Predicate {
    window(&serde_json::json!({
        "at": "t", "days": ["mon", "tue", "wed", "thu"], "from": "08:00", "to": "16:00",
        "offset": "+01:00"
    }))
}

#[test]
fn window_each_side_of_each_boundary_at_a_fixed_offset() {
    let guard = mon_to_thu();
    for (t, expected) in [
        // `from` is inclusive: Monday 08:00 at +01:00 is 07:00 UTC.
        ("2020-01-06T07:00:00Z", Truth::True),
        ("2020-01-06T06:59:59Z", Truth::False),
        // `to` is exclusive: Thursday 16:00 at +01:00 is 15:00 UTC.
        ("2020-01-09T14:59:59Z", Truth::True),
        ("2020-01-09T15:00:00Z", Truth::False),
        // The day boundary: Friday at `from` is outside, Wednesday noon inside.
        ("2020-01-10T07:00:00Z", Truth::False),
        ("2020-01-08T11:00:00Z", Truth::True),
        // Sunday 23:59:59 local and Monday 00:00 local are both before `from`.
        ("2020-01-05T22:59:59Z", Truth::False),
        ("2020-01-05T23:00:00Z", Truth::False),
    ] {
        assert_eq!(guard.evaluate(&at(t)), expected, "{t}");
    }
}

#[test]
fn window_one_instant_lands_on_different_days_under_two_offsets() {
    let t = "2020-01-09T23:30:00Z"; // Thursday 23:30 UTC, Friday 00:30 at +01:00.
    let friday = |offset: &str| {
        window(&serde_json::json!({
            "at": "t", "days": ["fri"], "from": "00:00", "to": "24:00", "offset": offset
        }))
    };
    assert_eq!(friday("+01:00").evaluate(&at(t)), Truth::True);
    assert_eq!(friday("Z").evaluate(&at(t)), Truth::False);
    assert_eq!(friday("-05:00").evaluate(&at(t)), Truth::False);
}

#[test]
fn window_compares_the_instant_never_the_offset_it_was_written_with() {
    let guard = mon_to_thu();
    // 02:30 at -05:00 is 07:30 UTC, 08:30 at +01:00: inside, though its written clock reads 02:30.
    assert_eq!(
        guard.evaluate(&at("2020-01-06T02:30:00-05:00")),
        Truth::True
    );
    // 08:30 at +08:00 is 00:30 UTC, 01:30 at +01:00: outside, though its written clock reads 08:30.
    assert_eq!(
        guard.evaluate(&at("2020-01-06T08:30:00+08:00")),
        Truth::False
    );
    // A fraction is dropped exactly: every boundary is a whole minute.
    assert_eq!(guard.evaluate(&at("2020-01-09T14:59:59.999Z")), Truth::True);
    assert_eq!(
        guard.evaluate(&at("2020-01-09T15:00:00.001Z")),
        Truth::False
    );
}

#[test]
fn window_crossing_midnight_belongs_to_the_day_it_opens() {
    let guard = window(&serde_json::json!({
        "at": "t", "days": ["fri"], "from": "22:00", "to": "02:00", "offset": "Z"
    }));
    for (t, expected) in [
        ("2020-01-10T21:59:59Z", Truth::False),
        ("2020-01-10T22:00:00Z", Truth::True),
        ("2020-01-11T01:59:59Z", Truth::True),
        ("2020-01-11T02:00:00Z", Truth::False),
        // Friday's own early hours belong to Thursday's window, which is not listed.
        ("2020-01-10T01:00:00Z", Truth::False),
        // Saturday's evening opens Saturday's window, which is not listed.
        ("2020-01-11T22:30:00Z", Truth::False),
    ] {
        assert_eq!(guard.evaluate(&at(t)), expected, "{t}");
    }
}

#[test]
fn window_unknown_where_no_instant_is_read() {
    let guard = mon_to_thu();
    assert_eq!(guard.evaluate(&FactStore::new()), Truth::Unknown);
    let outcome = guard.outcome(&at("Monday 09:00"));
    assert_eq!(outcome.truth, Truth::Unknown);
    let note = outcome.causes[0].note.clone().expect("a note");
    assert!(note.contains("RFC 3339"), "{note}");
    assert_eq!(
        Predicate::not(guard.clone()).evaluate(&FactStore::new()),
        Truth::Unknown,
        "negation keeps Unknown"
    );
}

#[test]
fn window_over_now_reads_the_clock_it_is_given_and_nothing_else() {
    let guard = window(&serde_json::json!({
        "at": "now", "days": ["mon", "tue", "wed", "thu"], "from": "08:00", "to": "16:00",
        "offset": "+01:00"
    }));
    let store = FactStore::new();
    assert_eq!(guard.evaluate(&store), Truth::Unknown, "no clock");
    for (now, expected) in [
        ("2020-01-06T07:00:00Z", Truth::True),
        ("2020-01-06T06:59:59Z", Truth::False),
        ("2020-01-09T14:59:59Z", Truth::True),
        ("2020-01-09T15:00:00Z", Truth::False),
        ("2020-01-10T07:00:00Z", Truth::False),
    ] {
        assert_eq!(
            guard.evaluate(&WithNow::new(&store, instant(now))),
            expected,
            "{now}"
        );
    }
    // A fact named `now` is not the clock.
    let mut named = FactStore::new();
    named.set_path("now", FactValue::text("2020-01-06T07:00:00Z"));
    assert_eq!(guard.evaluate(&named), Truth::Unknown);
}

#[test]
fn window_canonical_form_round_trips() {
    let written = serde_json::json!({"window": {
        "at": "now", "days": ["thu", "mon", "wed", "tue"], "from": "08:00", "to": "16:00",
        "offset": "+01:00"
    }});
    let predicate = read(written).expect("reads");
    let text = serde_json::to_string(&predicate).expect("writes");
    assert_eq!(
        text,
        r#"{"window":{"at":"now","days":["mon","tue","wed","thu"],"from":"08:00","offset":"+01:00","to":"16:00"}}"#
    );
    assert_eq!(
        read(serde_json::from_str(&text).expect("json")),
        Ok(predicate.clone())
    );
    let Predicate::Window(parsed) = &predicate else {
        panic!("a window: {predicate:?}");
    };
    assert_eq!(parsed.at, WindowInstant::Now);
    assert!(!parsed.crosses_midnight());
    assert_eq!(
        predicate.to_string(),
        "window(at now, mon tue wed thu, 08:00-16:00, +01:00)"
    );
    let utc = window(&serde_json::json!({
        "at": "a.b", "days": ["sun", "sat"], "from": "22:00", "to": "02:00", "offset": "+00:00"
    }));
    assert_eq!(
        serde_json::to_string(&utc).expect("writes"),
        r#"{"window":{"at":"a.b","days":["sat","sun"],"from":"22:00","offset":"Z","to":"02:00"}}"#
    );
    assert_eq!(
        utc.fact_paths()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["a.b"]
    );
    assert!(utc.reads_window());
    assert!(!Predicate::parse_expression("a < 3")
        .expect("parses")
        .reads_window());
}

#[test]
fn window_a_window_key_without_at_is_still_a_fact() {
    let predicate = read(serde_json::json!({"window": {"gte": 3}})).expect("reads");
    assert_eq!(
        predicate,
        Predicate::parse_expression("window >= 3").expect("parses")
    );
    assert!(!predicate.reads_window());
}

#[test]
fn window_named_zones_are_refused_by_name() {
    for (fields, needle) in [
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": "08:00", "to": "16:00",
                "offset": "Europe/Berlin"}),
            "`Europe/Berlin` names a time zone",
        ),
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": "08:00", "to": "16:00",
                "zone": "Europe/Berlin"}),
            "`zone`",
        ),
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": "08:00", "to": "16:00",
                "offset": "UTC"}),
            "`UTC` names a time zone",
        ),
    ] {
        let refused = read(serde_json::json!({ "window": fields })).expect_err("a zone is refused");
        assert!(refused.contains(needle), "{refused}");
        assert!(refused.contains("fixed offset"), "{refused}");
        assert!(refused.contains("daylight saving"), "{refused}");
    }
}

#[test]
fn window_below_source22_is_refused_naming_the_format() {
    let node: Node = serde_json::from_value(serde_json::json!({"window": {
        "at": "now", "days": ["mon"], "from": "08:00", "to": "16:00", "offset": "Z"
    }}))
    .expect("json");
    let refused = reading_source22_operands(false, || Predicate::from_node(&node))
        .expect_err("refused below ess/22");
    assert!(refused.to_string().contains("ess/22"), "{refused}");
    assert!(
        reading_source22_operands(true, || Predicate::from_node(&node)).is_ok(),
        "read from ess/22"
    );
}

#[test]
fn window_refusals_name_what_is_wrong() {
    for (fields, needle) in [
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": "08:00", "to": "08:00",
                "offset": "Z"}),
            "`from` and `to` are both 08:00",
        ),
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": "08:00", "to": "00:00",
                "offset": "Z"}),
            "24:00",
        ),
        (
            serde_json::json!({"at": "t", "days": [], "from": "08:00", "to": "16:00",
                "offset": "Z"}),
            "no day",
        ),
        (
            serde_json::json!({"at": "t", "days": ["mon", "mon"], "from": "08:00", "to": "16:00",
                "offset": "Z"}),
            "`mon` twice",
        ),
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": 480, "to": "16:00",
                "offset": "Z"}),
            "quoted",
        ),
        (
            serde_json::json!({"at": "t", "days": ["mon"], "from": "08:00", "to": "16:00",
                "offset": "-00:00"}),
            "-00:00",
        ),
    ] {
        let refused = read(serde_json::json!({ "window": fields })).expect_err("refused");
        assert!(refused.contains(needle), "{needle}: {refused}");
    }
}

// ---- the shared vectors -----------------------------------------------------------------------

/// The shared vectors the Go and TypeScript readers answer too.
const VECTORS: &str = include_str!("vectors/calendar-window.json");

#[test]
fn window_the_rust_evaluator_answers_the_shared_vectors() {
    let vectors: serde_json::Value = serde_json::from_str(VECTORS).expect("json");
    let mut answered = 0;
    for vector in vectors["evaluate"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let predicate = read(serde_json::json!({"window": vector["window"].clone()}))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let mut store = FactStore::new();
        match &vector["t"] {
            serde_json::Value::String(text) => store.set_path("t", FactValue::text(text.clone())),
            serde_json::Value::Number(_) => store.set_path(
                "t",
                FactValue::Number(serde_json::from_value(vector["t"].clone()).expect("a number")),
            ),
            _ => {}
        }
        assert_eq!(
            predicate.evaluate(&store).as_str(),
            vector["truth"].as_str().expect("a truth"),
            "{name}"
        );
        answered += 1;
    }
    for vector in vectors["refused"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        assert!(
            read(serde_json::json!({"window": vector["window"].clone()})).is_err(),
            "{name} is refused"
        );
        answered += 1;
    }
    for vector in vectors["canonical"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let predicate = read(serde_json::json!({"window": vector["written"].clone()}))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            serde_json::to_value(&predicate).expect("writes"),
            serde_json::json!({"window": vector["canonical"].clone()}),
            "{name}"
        );
        answered += 1;
    }
    assert!(answered >= 60, "{answered} vectors answered");
}

#[test]
fn window_boundaries_are_each_side_of_from_and_to_on_the_reference_week() {
    let Predicate::Window(guard) = mon_to_thu() else {
        panic!("a window");
    };
    let boundaries: Vec<String> = guard
        .boundaries()
        .into_iter()
        .map(Rfc3339Instant::to_rfc3339)
        .collect();
    for expected in [
        "2020-01-06T06:59:59Z",
        "2020-01-06T07:00:00Z",
        "2020-01-09T14:59:59Z",
        "2020-01-09T15:00:00Z",
        // Friday follows a listed day: its `from` is tried, and refuses.
        "2020-01-10T07:00:00Z",
    ] {
        assert!(
            boundaries.contains(&expected.to_owned()),
            "{expected}: {boundaries:?}"
        );
    }
    let classified = |inside: bool| {
        guard
            .boundaries()
            .into_iter()
            .filter(|candidate| guard.contains(*candidate) == inside)
            .count()
    };
    assert!(classified(true) >= 2 && classified(false) >= 2);
    let crossing = CalendarWindow::parse_mapping(
        &serde_json::from_value(serde_json::json!({
            "at": "t", "days": ["fri"], "from": "22:00", "to": "02:00", "offset": "Z"
        }))
        .expect("node"),
    )
    .expect("reads");
    assert!(crossing.crosses_midnight());
    let spelled: Vec<String> = crossing
        .boundaries()
        .into_iter()
        .map(Rfc3339Instant::to_rfc3339)
        .collect();
    for expected in [
        "2020-01-10T21:59:59Z",
        "2020-01-10T22:00:00Z",
        "2020-01-11T01:59:59Z",
        "2020-01-11T02:00:00Z",
    ] {
        assert!(
            spelled.contains(&expected.to_owned()),
            "{expected}: {spelled:?}"
        );
    }
}
