//! Adversary, story `the-date-time-agreement-oracle-refuses-non-digits`.
//!
//! `schema_contract::date_time` is checked against an RFC 3339 `date-time` reference written
//! here from the grammar (§5.6, with the leap-second rule of §5.7 read at UTC), over every
//! one-byte edit, deletion, insertion and truncation of a set of valid seeds (multibyte bytes
//! included, so a panic on odd input shows) and over the value range of each field. A set of
//! literals pins the ranges a mutant of the checker would move (month, day, minute, second,
//! offset, century leap years) and that the crate's own unit test does not name.
//!
//! The same candidates were also compared against the dependency's own checker restricted to
//! ASCII digits, and agreed; that comparison is not kept here because this crate's tests may not
//! build the dependency's own validator (`schema_unique_items.rs`).

use schema_contract::date_time::is_date_time;
use serde_json::{json, Value};

fn wired() -> jsonschema::Validator {
    schema_contract::uniqueness::options()
        .should_validate_formats(true)
        .build(&json!({"type": "string", "format": "date-time"}))
        .expect("schema")
}

/// RFC 3339 `date-time`, read from the grammar: `full-date "T" full-time`, `T` and `Z` in either
/// case, `time-secfrac` one or more digits, `time-numoffset` `+`/`-` `HH:MM` with HH <= 23 and
/// MM <= 59, and second 60 only where the instant is 23:59:60 UTC.
fn reference(text: &str) -> bool {
    if !text.is_ascii() || text.len() < 20 {
        return false;
    }
    let number = |part: &str| -> Option<i64> {
        part.bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| part.parse().unwrap())
    };
    let (date, rest) = text.split_at(10);
    let (separator, time) = rest.split_at(1);
    if separator != "T" && separator != "t" {
        return false;
    }
    if &date[4..5] != "-" || &date[7..8] != "-" || &time[2..3] != ":" || &time[5..6] != ":" {
        return false;
    }
    let fields = [
        number(&date[..4]),
        number(&date[5..7]),
        number(&date[8..10]),
        number(&time[..2]),
        number(&time[3..5]),
        number(&time[6..8]),
    ];
    let [Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)] = fields
    else {
        return false;
    };
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap_year { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month) || day < 1 || day > month_days[(month - 1) as usize] {
        return false;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return false;
    }
    let mut zone = &time[8..];
    if let Some(fraction) = zone.strip_prefix('.') {
        let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        zone = &fraction[digits..];
    }
    let east = match zone {
        "Z" | "z" => 0,
        _ if zone.len() == 6 && &zone[3..4] == ":" => {
            let sign = match &zone[..1] {
                "+" => 1,
                "-" => -1,
                _ => return false,
            };
            let (Some(offset_hour), Some(offset_minute)) =
                (number(&zone[1..3]), number(&zone[4..]))
            else {
                return false;
            };
            if offset_hour > 23 || offset_minute > 59 {
                return false;
            }
            sign * (offset_hour * 60 + offset_minute)
        }
        _ => return false,
    };
    second != 60 || (hour * 60 + minute - east).rem_euclid(1440) == 1439
}

const SEEDS: &[&str] = &[
    "2026-10-29T12:00:00Z",
    "2026-10-29t12:00:00.250+02:00",
    "2024-02-29T00:00:00-05:30",
    "2016-12-31T23:59:60Z",
    "2016-12-31T18:29:60-05:30",
    "2017-01-01T00:59:60+01:00",
    "0000-01-01T00:00:00Z",
    "9999-12-31T23:59:59.999999999z",
    "1900-02-28T00:00:00+23:59",
    "2000-02-29T00:00:00-23:59",
];

fn candidates() -> Vec<String> {
    let alphabet: Vec<String> = "0123456789+-.:TtZz x/"
        .chars()
        .map(String::from)
        .chain(["é", "５", "\u{0}", "\u{1F600}"].map(String::from))
        .collect();
    let mut out = Vec::new();
    for seed in SEEDS {
        out.push((*seed).to_owned());
        for at in 0..seed.len() {
            let mut deleted = (*seed).to_owned();
            deleted.remove(at);
            out.push(deleted);
            for byte in &alphabet {
                let mut replaced = (*seed).to_owned();
                replaced.replace_range(at..=at, byte);
                out.push(replaced);
                let mut inserted = (*seed).to_owned();
                inserted.insert_str(at, byte);
                out.push(inserted);
            }
        }
        for cut in 0..seed.len() {
            out.push(seed[..cut].to_owned());
        }
    }
    for year in ["0000", "1900", "2000", "2024", "2026"] {
        for month in 0..=13 {
            for day in 0..=32 {
                out.push(format!("{year}-{month:02}-{day:02}T12:00:00Z"));
            }
        }
    }
    for hour in 0..=24 {
        for minute in [0, 1, 29, 30, 58, 59, 60] {
            for second in [0, 59, 60, 61] {
                out.push(format!("2026-10-29T{hour:02}:{minute:02}:{second:02}Z"));
                out.push(format!("2026-10-29T{hour:02}:{minute:02}:{second:02}.5z"));
                for sign in ['+', '-'] {
                    for offset_hour in [0, 1, 5, 12, 23, 24] {
                        for offset_minute in [0, 1, 30, 59, 60] {
                            out.push(format!(
                                "2026-10-29T{hour:02}:{minute:02}:{second:02}{sign}{offset_hour:02}:{offset_minute:02}"
                            ));
                        }
                    }
                }
            }
        }
    }
    out
}

#[test]
fn the_checker_and_the_options_validator_are_rfc_3339_on_every_candidate() {
    let wired = wired();
    let mut wrong = Vec::new();
    let all = candidates();
    let mut admitted = 0;
    for text in &all {
        let expected = reference(text);
        let ours = is_date_time(text);
        let through_options = wired.is_valid(&Value::from(text.as_str()));
        if ours {
            admitted += 1;
        }
        if ours != expected || through_options != expected {
            wrong.push(format!(
                "{text:?}: RFC 3339 {expected}, checker {ours}, options() {through_options}"
            ));
        }
    }
    assert!(
        admitted > 2_000,
        "only {admitted} of {} admitted",
        all.len()
    );
    assert!(
        wrong.is_empty(),
        "{} disagreements:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn the_field_ranges_are_rfc_3339s() {
    for good in [
        "2000-02-29T00:00:00Z",
        "0000-02-29T00:00:00Z",
        "2026-12-31T23:59:59Z",
        "2026-01-01T00:00:00+23:59",
        "2026-01-01T00:00:00-23:59",
        "2016-12-31T18:29:60-05:30",
        "2017-01-01T00:59:60+01:00",
        "2016-12-31T23:59:60+00:00",
        "2016-12-31T23:59:60-00:00",
        "2026-10-29T12:00:00.000000000000000000001Z",
    ] {
        assert!(is_date_time(good), "{good} refused");
    }
    for bad in [
        "1900-02-29T00:00:00Z",
        "2100-02-29T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-00-01T00:00:00Z",
        "2026-01-00T00:00:00Z",
        "2026-01-32T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-01-01T00:60:00Z",
        "2026-01-01T23:59:61Z",
        "2026-01-01T12:00:00+24:00",
        "2026-01-01T12:00:00+00:60",
        "2026-01-01T12:00:00-24:00",
        "2016-12-31T23:59:60+00:01",
        "2016-12-31T23:59:60-01:00",
        "2016-12-31T22:59:60+01:00",
        "2026-10-29T12:00:00+01:00Z",
        "2026-10-29T12:00:00Z ",
        " 2026-10-29T12:00:00Z",
        "2026-10-29 12:00:00Z",
        "2026-10-29T12:00:00.5",
        "2026-10-29T12:00:00+01",
        "2026-10-29T12:00:00+01:0",
        "2026-10-29T12:00:00+01:000",
        "2026-10-29T12:00:00.５Z",
        "2026-10-29T12:00:00Ｚ",
        "2026-10-29Ｔ12:00:00Z",
        "T",
        "t",
        "2026-10-29T",
        "2026-10-29T12:00:00",
        "2026-10-29T12:00:0",
    ] {
        assert!(!is_date_time(bad), "{bad} admitted");
    }
}
