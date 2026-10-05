//! Adversary cases, beyond10x/ess#244 part b, pass 1: the shared Rust window evaluator and reader
//! (`docs/design/calendar-window-guards.md`) against an independent oracle, and the spellings the
//! design admits or refuses.

use ess_primitives::facts::{FactStore, FactValue};
use ess_primitives::predicate::{Predicate, Truth};
use ess_primitives::time::Rfc3339Instant;

fn read(value: serde_json::Value) -> Result<Predicate, String> {
    serde_json::from_value::<Predicate>(value).map_err(|error| error.to_string())
}

fn fields(offset: &str, from: &str, to: &str, days: &[&str]) -> serde_json::Value {
    serde_json::json!({"window": {"at": "t", "days": days, "from": from, "to": to, "offset": offset}})
}

fn at(text: &str) -> FactStore {
    let mut store = FactStore::new();
    store.set_path("t", FactValue::text(text));
    store
}

/// A deterministic generator (64-bit LCG, fixed seed).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 17
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// The oracle: the instant shifted by the offset, written back as UTC text, and the weekday and
/// time of day read off that text with Sakamoto's day-of-week formula. Shares nothing with
/// `CalendarWindow::contains` but the RFC 3339 writer.
fn oracle(seconds: i64, days: [bool; 7], from: i64, to: i64, offset_minutes: i64) -> Option<bool> {
    let local = Rfc3339Instant::from_epoch_seconds(seconds + offset_minutes * 60)?.to_rfc3339();
    let number = |range: std::ops::Range<usize>| local[range].parse::<i64>().unwrap();
    let (year, month, day) = (number(0..4), number(5..7), number(8..10));
    let tod = number(11..13) * 3600 + number(14..16) * 60 + number(17..19);
    // Sakamoto: 0 = Sunday.
    let table = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let sunday_first =
        (y + y / 4 - y / 100 + y / 400 + table[usize::try_from(month - 1).unwrap()] + day) % 7;
    let monday_first = usize::try_from((sunday_first + 6) % 7).unwrap();
    let previous = (monday_first + 6) % 7;
    Some(if from > to {
        (days[monday_first] && tod >= from * 60) || (days[previous] && tod < to * 60)
    } else {
        days[monday_first] && from * 60 <= tod && tod < to * 60
    })
}

const NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

fn clock(minutes: i64) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

fn offset_text(minutes: i64) -> String {
    if minutes == 0 {
        return "Z".to_owned();
    }
    let sign = if minutes < 0 { '-' } else { '+' };
    format!("{sign}{:02}:{:02}", minutes.abs() / 60, minutes.abs() % 60)
}

/// Property, fixed seed: for 400 windows (every offset from -14:00 to +14:00 in reach, odd minutes,
/// crossing and not, `to` 24:00) and, per window, its own synthesis boundaries, a second either side
/// of each, half-second instants and random instants from 1900 to 2100 written at random offsets,
/// the evaluator agrees with the oracle.
#[test]
fn adv244b_window_evaluation_agrees_with_an_independent_oracle() {
    let mut random = Lcg(0x0244_b0a1_2020_0106);
    let mut disagreements = Vec::new();
    let mut checked = 0usize;
    for _ in 0..400 {
        let mut days = [false; 7];
        while !days.iter().any(|day| *day) {
            for day in &mut days {
                *day = random.below(3) == 0;
            }
        }
        let from = i64::try_from(random.below(1440)).unwrap();
        let mut to = 1 + i64::try_from(random.below(1440)).unwrap();
        if to == from {
            to = if from == 1439 { 1440 } else { from + 1 };
        }
        let offset = i64::try_from(random.below(1681)).unwrap() - 840;
        let listed: Vec<&str> = (0..7).filter(|i| days[*i]).map(|i| NAMES[i]).collect();
        let predicate = read(fields(
            &offset_text(offset),
            &clock(from),
            &clock(to),
            &listed,
        ))
        .unwrap_or_else(|error| panic!("{} {} {}: {error}", clock(from), clock(to), offset));
        let Predicate::Window(window) = &predicate else {
            panic!("a window");
        };
        let mut instants: Vec<i64> = Vec::new();
        for boundary in window.boundaries() {
            let second = boundary.epoch_seconds();
            instants.extend([second - 1, second, second + 1]);
        }
        for _ in 0..40 {
            // 1900-01-01 .. 2100-01-01
            instants.push(-2_208_988_800 + i64::try_from(random.below(6_311_433_600)).unwrap());
        }
        for second in instants {
            let expected = oracle(second, days, from, to, offset);
            let Some(expected) = expected else { continue };
            // Written at a random offset, with a half second on every other instant.
            let written_offset = i64::try_from(random.below(1681)).unwrap() - 840;
            let mut text = Rfc3339Instant::from_epoch_seconds(second + written_offset * 60)
                .unwrap()
                .to_rfc3339();
            text.truncate(19);
            if second % 2 == 0 {
                text.push_str(".5");
            }
            text.push_str(
                if written_offset == 0 {
                    "Z".to_owned()
                } else {
                    offset_text(written_offset)
                }
                .as_str(),
            );
            let parsed = Rfc3339Instant::parse_rfc3339(&text).expect("written back");
            assert_eq!(parsed.epoch_seconds(), second, "{text}");
            let truth = predicate.evaluate(&at(&text));
            checked += 1;
            if truth != Truth::from_bool(expected) {
                disagreements.push(format!(
                    "{window}: {text} oracle {expected} evaluator {truth:?}"
                ));
            }
        }
    }
    assert!(checked > 20_000, "{checked}");
    assert_eq!(
        disagreements.len(),
        0,
        "{:#?}",
        &disagreements[..disagreements.len().min(10)]
    );
}

/// Design, Source: `offset` is `Z` or `±HH:MM`, "at most `14:00` either way", `+00:00` reads as `Z`,
/// `-00:00` refused; a time is quoted `HH:MM`, `from` up to 23:59 and `to` from 00:01 to 24:00;
/// days at least one, no repeats.
#[test]
fn adv244b_window_spellings_admitted_and_refused() {
    let monday = ["mon"];
    for (offset, canonical) in [
        ("+14:00", "+14:00"),
        ("-14:00", "-14:00"),
        ("+05:45", "+05:45"),
        ("-09:30", "-09:30"),
        ("+00:00", "Z"),
        ("Z", "Z"),
    ] {
        let predicate = read(fields(offset, "08:00", "16:00", &monday))
            .unwrap_or_else(|error| panic!("{offset}: {error}"));
        let Predicate::Window(window) = predicate else {
            panic!("a window")
        };
        assert_eq!(window.offset_text(), canonical, "{offset}");
    }
    for offset in [
        "+14:01",
        "-14:01",
        "+5:00",
        "+0500",
        "UTC+1",
        "z",
        "-00:00",
        "+05:60",
        "GMT",
        "UTC",
        "Europe/Berlin",
        "+01",
        "+01:00:00",
        " +01:00",
        "+01:00 ",
        "\u{2212}01:00",
    ] {
        assert!(
            read(fields(offset, "08:00", "16:00", &monday)).is_err(),
            "offset {offset:?} admitted"
        );
    }
    for (from, to) in [
        ("7:00", "16:00"),
        ("07:00:00", "16:00"),
        ("08:00", "24:01"),
        ("23:60", "24:00"),
        ("0800", "16:00"),
        ("08.00", "16:00"),
        ("24:00", "16:00"),
        ("08:00", "00:00"),
        ("08:00", "08:00"),
        ("\u{ff10}8:00", "16:00"),
        ("+8:00", "16:00"),
        ("-1:00", "16:00"),
    ] {
        assert!(
            read(fields("Z", from, to, &monday)).is_err(),
            "{from}-{to} admitted"
        );
    }
    for days in [&[][..], &["mon", "mon"][..], &["Mon"][..], &["monday"][..]] {
        assert!(
            read(fields("Z", "08:00", "16:00", days)).is_err(),
            "{days:?} admitted"
        );
    }
    // Unordered days are written back Monday first, and the whole day is 00:00-24:00.
    let predicate = read(fields("Z", "00:00", "24:00", &["sun", "wed", "mon"])).unwrap();
    assert_eq!(
        serde_json::to_string(&predicate).unwrap(),
        r#"{"window":{"at":"t","days":["mon","wed","sun"],"from":"00:00","offset":"Z","to":"24:00"}}"#
    );
}

/// Design, Unknown: text at `at` that is no RFC 3339 instant is Unknown. A leap second and a
/// lowercase `z` are the two spellings at the edge of the production; neither may decide a window
/// differently from its neighbour by accident.
#[test]
fn adv244b_window_leap_second_and_lowercase_z() {
    let predicate = read(fields("Z", "23:00", "24:00", &["mon"])).unwrap();
    assert_eq!(
        predicate.evaluate(&at("2020-01-06T23:59:60Z")),
        Truth::Unknown,
        "a leap second names no instant on this line"
    );
    assert_eq!(predicate.evaluate(&at("2020-01-06T23:59:59z")), Truth::True);
    assert_eq!(
        predicate.evaluate(&at("2020-01-06t23:59:59.999999999Z")),
        Truth::True
    );
    assert_eq!(
        predicate.evaluate(&at("2020-01-07T00:00:00.000000001Z")),
        Truth::False
    );
    // Pre-epoch: 1969-12-29 is a Monday.
    assert_eq!(
        predicate.evaluate(&at("1969-12-29T23:30:00.5Z")),
        Truth::True
    );
    assert_eq!(
        predicate.evaluate(&at("1969-12-29T22:59:59.999Z")),
        Truth::False
    );
}

/// Design, Semantics: a crossing window "belongs to the day it opens". The Sunday-to-Monday wrap at
/// every offset extreme: `days: [sun], from: 22:00, to: 02:00` holds Sunday 22:00 to Monday 02:00
/// local, never Sunday 01:00 and never Monday 22:00.
#[test]
fn adv244b_window_sunday_to_monday_wrap_at_offset_extremes() {
    for offset in [-840i64, -345, 0, 345, 840] {
        let predicate = read(fields(&offset_text(offset), "22:00", "02:00", &["sun"])).unwrap();
        // Local Sunday 2020-01-12 and Monday 2020-01-13, as UTC seconds.
        let sunday = 1_578_787_200i64 - offset * 60;
        for (local, expected) in [
            (sunday + 79_199, false),          // Sun 21:59:59
            (sunday + 79_200, true),           // Sun 22:00
            (sunday + 86_399, true),           // Sun 23:59:59
            (sunday + 86_400, true),           // Mon 00:00
            (sunday + 93_599, true),           // Mon 01:59:59
            (sunday + 93_600, false),          // Mon 02:00
            (sunday + 3_600, false),           // Sun 01:00 (Saturday unlisted)
            (sunday + 86_400 + 79_200, false), // Mon 22:00
        ] {
            let text = Rfc3339Instant::from_epoch_seconds(local)
                .unwrap()
                .to_rfc3339();
            assert_eq!(
                predicate.evaluate(&at(&text)),
                Truth::from_bool(expected),
                "offset {offset}: {text}"
            );
        }
    }
}
