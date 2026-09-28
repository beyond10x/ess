//! The current-time operand of a `Timestamp` guard (beyond10x/ess#171, `ess/16`,
//! `docs/design/current-time-guards.md`): `starts_at < now - 60s`.
//!
//! The operand is a text literal on the right of an ordering, read as the instant the evaluator is
//! told it is now plus a signed whole-unit offset. Only a source that knows the time can answer it;
//! every other source keeps the answer it had (`Unknown`, because the text is no RFC 3339 instant).
use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue};
use ess_primitives::predicate::{Predicate, Truth, WithNow};
use ess_primitives::time::{CurrentTime, Rfc3339Instant};

fn at(text: &str) -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339(text).unwrap_or_else(|| panic!("{text} is an instant"))
}

/// A source that declares `starts_at` a `Timestamp` and `label` a plain text.
struct Declared(FactStore);

impl FactSource for Declared {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.0.fact(path)
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        path.namespace() == "starts_at"
    }

    fn orders_text_by_bytes(&self, _path: &FactPath) -> bool {
        true
    }
}

fn declared(starts_at: &str) -> Declared {
    let mut store = FactStore::new();
    store.set(
        FactPath::new("starts_at").unwrap(),
        FactValue::text(starts_at),
    );
    store.set(FactPath::new("label").unwrap(), FactValue::text("now"));
    Declared(store)
}

fn parse(expression: &str) -> Predicate {
    Predicate::parse_expression(expression).unwrap_or_else(|e| panic!("{expression}: {e}"))
}

#[test]
fn the_operand_is_now_with_an_optional_signed_whole_unit_offset() {
    for (text, seconds) in [
        ("now", 0),
        ("now - 60s", -60),
        ("now + 5m", 300),
        ("now - 1h", -3600),
        ("now-60s", -60),
        ("now +5m", 300),
    ] {
        assert_eq!(
            CurrentTime::parse(text).map(CurrentTime::offset_seconds),
            Some(seconds),
            "{text}"
        );
    }
    for text in [
        "Now",
        "now - 60",
        "now - 1d",
        "now - 060s",
        "now - -60s",
        "now - 1.5h",
        "now 60s",
        "now - ",
        "nowadays",
        "now - 99999999999h",
        "2020-01-01T00:00:00Z",
    ] {
        assert_eq!(CurrentTime::parse(text), None, "{text}");
    }
    assert!(
        CurrentTime::mentions("now - 60"),
        "a malformed offset is still an attempt at now"
    );
    assert!(!CurrentTime::mentions("nowadays"));
    assert!(!CurrentTime::mentions("2020-01-01T00:00:00Z"));
}

#[test]
fn a_timestamp_ordered_against_now_is_decided_by_the_clock_the_evaluator_is_given() {
    let now = at("2026-09-27T12:00:00Z");
    let guard = parse("starts_at < now - 60s");
    for (starts_at, truth) in [
        ("2026-09-27T11:58:59Z", Truth::True),
        ("2026-09-27T11:59:00Z", Truth::False),
        ("2026-09-27T11:59:01Z", Truth::False),
        // Ordered by the instant, not the spelling.
        ("2026-09-27T13:58:59+02:00", Truth::True),
    ] {
        let facts = declared(starts_at);
        assert_eq!(
            guard.evaluate(&WithNow::new(&facts, now)),
            truth,
            "{starts_at}"
        );
    }
    let facts = declared("2026-09-27T12:04:59Z");
    assert_eq!(
        parse("starts_at > now + 5m").evaluate(&WithNow::new(&facts, now)),
        Truth::False
    );
    assert_eq!(
        parse("starts_at >= now").evaluate(&WithNow::new(&facts, now)),
        Truth::True
    );
}

#[test]
fn a_source_without_a_clock_keeps_the_answer_it_had() {
    let facts = declared("2020-01-01T00:00:00Z");
    assert_eq!(
        parse("starts_at < now - 60s").evaluate(&facts),
        Truth::Unknown,
        "no clock, so the operand names no instant"
    );
}

#[test]
fn only_a_literal_is_read_as_now() {
    let now = at("2026-09-27T12:00:00Z");
    // A caller who sends the text `now` sent no instant.
    let facts = declared("now");
    assert_eq!(
        parse("starts_at < \"2030-01-01T00:00:00Z\"").evaluate(&WithNow::new(&facts, now)),
        Truth::Unknown
    );
    // A text that is not a declared Timestamp compares with the four characters, as before.
    let facts = declared("2020-01-01T00:00:00Z");
    assert_eq!(
        parse("label == now").evaluate(&WithNow::new(&facts, now)),
        Truth::True
    );
}

#[test]
fn the_compact_form_round_trips_the_operand_as_written() {
    for expression in [
        "starts_at < now - 60s",
        "starts_at >= now",
        "starts_at > now + 5m",
    ] {
        let predicate = parse(expression);
        assert_eq!(predicate.to_string(), expression);
        let node = predicate.to_node();
        assert_eq!(
            Predicate::from_node(&node).unwrap(),
            predicate,
            "{expression}"
        );
    }
}

#[test]
fn instant_arithmetic_for_a_clock() {
    let instant = Rfc3339Instant::from_epoch_millis(1_790_510_400_250).expect("in range");
    assert_eq!(instant.to_rfc3339(), "2026-09-27T12:00:00.25Z");
    assert_eq!(
        instant.ceil_to_second().to_rfc3339(),
        "2026-09-27T12:00:01Z"
    );
    assert_eq!(
        at("2026-09-27T12:00:00Z").ceil_to_second(),
        at("2026-09-27T12:00:00Z")
    );
    assert_eq!(
        at("2026-09-27T11:59:00Z").whole_seconds_since(at("2026-09-27T12:00:00Z")),
        Some(-60)
    );
    assert_eq!(
        at("2026-09-27T11:59:00.5Z").whole_seconds_since(at("2026-09-27T12:00:00Z")),
        None,
        "a fraction of a second is no whole offset"
    );
    assert_eq!(
        CurrentTime::parse("now - 60s")
            .and_then(|operand| operand.at(at("2026-09-27T12:00:00Z")))
            .map(Rfc3339Instant::to_rfc3339),
        Some("2026-09-27T11:59:00Z".to_owned())
    );
}
