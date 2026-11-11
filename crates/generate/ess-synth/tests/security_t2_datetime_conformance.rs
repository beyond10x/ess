//! Lives in ess-synth, not schema-contract: it builds the dependency's own `date-time` checker as a
//! reference, which schema-contract's constructor guard (`schema_unique_items`) forbids in its tests.
//!
//! Conformance checks for the crate's own `date-time` format checker (security review, turn 2).
//!
//! 1. No input makes it panic: arbitrary short strings over a structural and multibyte alphabet,
//!    and very long inputs.
//! 2. It agrees with the dependency's own checker on every string with ASCII digits at every
//!    digit position (fixed seed, structured generation, every leap-second hour/minute/offset).
//! 3. Every non-digit, ASCII or not, at every digit position of several valid stamps is refused,
//!    by the function and by a validator built from `uniqueness::options()`.
//! 4. `validate` (behind `ess generate schema validate`) reaches the checker for a draft-07
//!    document, a document without `$schema`, a `$ref` into another document and array items.

use schema_contract::date_time::is_date_time;
use schema_contract::uniqueness::options;
use schema_contract::validate::{validate, IssueCode, JsonDocument};
use serde_json::{json, Value};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
    fn digit(&mut self) -> char {
        char::from(b'0' + u8::try_from(self.below(10)).expect("a digit"))
    }
    fn two(&mut self, in_range_below: u32) -> String {
        if self.below(4) == 0 {
            format!("{}{}", self.digit(), self.digit())
        } else {
            format!("{:02}", self.below(u64::from(in_range_below)))
        }
    }
}

fn dependency() -> jsonschema::Validator {
    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&json!({"type": "string", "format": "date-time"}))
        .expect("the schema compiles")
}

fn ours() -> jsonschema::Validator {
    options()
        .should_validate_formats(true)
        .build(&json!({"type": "string", "format": "date-time"}))
        .expect("the schema compiles")
}

fn admits(validator: &jsonschema::Validator, text: &str) -> bool {
    validator.is_valid(&Value::String(text.to_owned()))
}

/// Whether every position RFC 3339 spells with a digit holds an ASCII digit (fixed fields and the
/// numeric offset; fractional digits the dependency already checks byte by byte).
fn digits_where_rfc3339_puts_them(text: &str) -> bool {
    let bytes = text.as_bytes();
    let all = |from: usize, to: usize| {
        bytes
            .get(from..to)
            .is_some_and(|part| part.iter().all(u8::is_ascii_digit))
    };
    let fixed = [(0, 4), (5, 7), (8, 10), (11, 13), (14, 16), (17, 19)]
        .iter()
        .all(|&(from, to)| all(from, to));
    let offset = matches!(bytes.last(), Some(b'Z' | b'z'))
        || (bytes.len() >= 5
            && all(bytes.len() - 5, bytes.len() - 3)
            && all(bytes.len() - 2, bytes.len()));
    fixed && offset
}

#[test]
fn security_t2_datetime_no_input_panics_and_refusals_only_narrow_the_dependency() {
    let dependency = dependency();
    let ours = ours();
    let alphabet: Vec<&str> = vec![
        "0", "1", "2", "5", "9", "-", ":", "T", "t", "Z", "z", "+", ".", " ", "x", "\u{0}", "é",
        "２", "٣", "𝟘", "\u{feff}",
    ];
    let mut rng = Rng(0x5eed_2026_1110_0001);
    let mut cases = Vec::new();
    for _ in 0..400_000 {
        let length = rng.below(34);
        let mut text = String::new();
        for _ in 0..length {
            text.push_str(alphabet[usize::try_from(rng.below(alphabet.len() as u64)).unwrap()]);
        }
        cases.push(text);
    }
    // Mutations of a valid stamp: each byte position replaced by each alphabet entry, deleted or
    // doubled, so the structural checks see every length around the valid one.
    let base = "2026-10-29T12:00:00.5+01:00";
    for at in 0..base.len() {
        for entry in &alphabet {
            cases.push(format!("{}{}{}", &base[..at], entry, &base[at + 1..]));
        }
        cases.push(format!("{}{}", &base[..at], &base[at + 1..]));
        cases.push(format!("{}{}", &base[..=at], &base[at..]));
        cases.push(base[..at].to_owned());
    }
    let long = 1_000_000;
    cases.push(format!("2026-10-29T12:00:00.{}Z", "1".repeat(long)));
    cases.push(format!("2026-10-29T12:00:00.{}", "1".repeat(long)));
    cases.push("T".repeat(long));
    cases.push(format!("2026-10-29T{}", ":".repeat(long)));
    cases.push(format!("{}T12:00:00Z", "9".repeat(long)));
    cases.push(format!("2026-10-29T12:00:00+{}", "0".repeat(long)));
    cases.push("é".repeat(long));
    let mut widened = Vec::new();
    for text in &cases {
        let checked = is_date_time(text);
        assert_eq!(
            checked,
            admits(&ours, text),
            "{text:?}: options() and is_date_time"
        );
        let reference = admits(&dependency, text);
        if checked && !reference {
            widened.push(text.clone());
        }
        if reference && !checked && digits_where_rfc3339_puts_them(text) {
            widened.push(format!("narrowed with digits in place: {text}"));
        }
    }
    assert!(
        is_date_time(&cases[cases.len() - 7]),
        "a million fractional digits"
    );
    assert!(
        widened.is_empty(),
        "{} disagreement(s): {:?}",
        widened.len(),
        &widened[..widened.len().min(20)]
    );
}

#[test]
fn security_t2_datetime_agrees_with_the_dependency_wherever_digits_are_in_place() {
    let dependency = dependency();
    let mut rng = Rng(0x5eed_2026_1110_0002);
    let mut disagreements = Vec::new();
    let mut admitted = 0;
    let mut check = |text: String, disagreements: &mut Vec<String>| {
        let checked = is_date_time(&text);
        if checked {
            admitted += 1;
        }
        if checked != admits(&dependency, &text) {
            disagreements.push(text);
        }
    };
    for _ in 0..300_000 {
        let year = if rng.below(4) == 0 {
            format!(
                "{}{}{}{}",
                rng.digit(),
                rng.digit(),
                rng.digit(),
                rng.digit()
            )
        } else {
            ["0000", "1900", "2000", "2024", "2026", "2100", "9999"]
                [usize::try_from(rng.below(7)).unwrap()]
            .to_owned()
        };
        let month = rng.two(13);
        let day = rng.two(32);
        let separator = if rng.below(2) == 0 { "T" } else { "t" };
        let hour = rng.two(24);
        let minute = if rng.below(3) == 0 {
            "59".to_owned()
        } else {
            rng.two(60)
        };
        let second = if rng.below(3) == 0 {
            "60".to_owned()
        } else {
            rng.two(61)
        };
        let fraction = match rng.below(4) {
            0 => String::new(),
            1 => ".".to_owned(),
            _ => {
                let mut text = ".".to_owned();
                for _ in 0..=rng.below(9) {
                    text.push(rng.digit());
                }
                text
            }
        };
        let offset = match rng.below(4) {
            0 => "Z".to_owned(),
            1 => "z".to_owned(),
            sign => format!(
                "{}{}:{}",
                if sign == 2 { '+' } else { '-' },
                rng.two(24),
                rng.two(60)
            ),
        };
        check(
            format!("{year}-{month}-{day}{separator}{hour}:{minute}:{second}{fraction}{offset}"),
            &mut disagreements,
        );
    }
    for hour in 0..24 {
        for minute in 0..60 {
            check(
                format!("2026-10-29T{hour:02}:{minute:02}:60Z"),
                &mut disagreements,
            );
            for offset_hour in 0..24 {
                for offset_minute in [0, 1, 29, 30, 59] {
                    for sign in ['+', '-'] {
                        check(
                            format!(
                                "2026-10-29T{hour:02}:{minute:02}:60{sign}{offset_hour:02}:{offset_minute:02}"
                            ),
                            &mut disagreements,
                        );
                    }
                }
            }
        }
    }
    for year in 0..10_000 {
        check(format!("{year:04}-02-29T00:00:00Z"), &mut disagreements);
    }
    assert!(
        admitted > 10_000,
        "the generator reaches valid stamps: {admitted}"
    );
    assert!(
        disagreements.is_empty(),
        "{} disagreement(s), the first: {:?}",
        disagreements.len(),
        &disagreements[..disagreements.len().min(20)]
    );
}

#[test]
fn security_t2_datetime_refuses_every_non_digit_at_every_digit_position() {
    let ours = ours();
    let mut replacements: Vec<String> = (0_u8..0x80)
        .filter(|byte| !byte.is_ascii_digit())
        .map(|byte| char::from(byte).to_string())
        .collect();
    replacements.extend(["é", "２", "٣", "𝟘", "\u{feff}", "\u{0660}"].map(str::to_owned));
    let mut admitted = Vec::new();
    let mut checked = 0;
    for stamp in [
        "2026-10-29T12:00:00Z",
        "2024-02-29T23:59:60Z",
        "2026-10-29T00:59:60+01:00",
        "2026-10-29t12:00:00.123-05:30",
        "0000-01-01T00:00:00.0z",
        "9999-12-31T23:59:59.999999999+23:59",
    ] {
        assert!(is_date_time(stamp), "{stamp}");
        assert!(admits(&ours, stamp), "{stamp}");
        for (at, byte) in stamp.bytes().enumerate() {
            if !byte.is_ascii_digit() {
                continue;
            }
            for replacement in &replacements {
                let text = format!("{}{}{}", &stamp[..at], replacement, &stamp[at + 1..]);
                checked += 1;
                if is_date_time(&text) || admits(&ours, &text) {
                    admitted.push(text);
                }
            }
        }
    }
    assert!(checked > 10_000, "{checked}");
    assert!(admitted.is_empty(), "admitted: {admitted:?}");
}

fn report(schemas: &[(&str, Value)], instance: &Value) -> (bool, Vec<(IssueCode, String)>) {
    let documents: Vec<JsonDocument<'_>> = schemas
        .iter()
        .map(|(location, value)| JsonDocument::new(location, value))
        .collect();
    let report = validate(
        &documents,
        &[JsonDocument::new("instances/at.json", instance)],
    );
    let issues = report
        .issues
        .iter()
        .map(|issue| (issue.code, issue.instance_path.clone()))
        .collect();
    (report.is_valid(), issues)
}

#[test]
fn security_t2_datetime_schema_validate_reaches_the_checker_on_every_dialect_and_reference() {
    let draft7 = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "$id": "urn:example:seven",
        "type": "object",
        "properties": {"at": {"type": "string", "format": "date-time"}}
    });
    let bare = json!({
        "$id": "urn:example:bare",
        "type": "object",
        "properties": {"at": {"type": "string", "format": "date-time"}}
    });
    let shared = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "urn:example:shared",
        "$defs": {"stamp": {"type": "string", "format": "date-time"}}
    });
    let referring = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "urn:example:referring",
        "type": "object",
        "properties": {"at": {"$ref": "urn:example:shared#/$defs/stamp"}}
    });
    let items = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "urn:example:items",
        "type": "object",
        "properties": {"at": {"type": "array", "items": {"type": "string", "format": "date-time"}}}
    });
    let schemas = [
        ("schemas/seven.json", draft7),
        ("schemas/bare.json", bare),
        ("schemas/shared.json", shared),
        ("schemas/referring.json", referring),
        ("schemas/items.json", items),
    ];
    let mut failures = Vec::new();
    for selector in [
        "urn:example:seven",
        "urn:example:bare",
        "urn:example:referring",
        "urn:example:items",
    ] {
        let wrap = |text: &str| {
            if selector == "urn:example:items" {
                json!({"schema": selector, "at": ["2026-10-29T12:00:00Z", text]})
            } else {
                json!({"schema": selector, "at": text})
            }
        };
        let path = if selector == "urn:example:items" {
            "/at/1"
        } else {
            "/at"
        };
        let (valid, issues) = report(&schemas, &wrap("2026-10-29T12:00:00Z"));
        if !valid {
            failures.push(format!(
                "{selector}: the valid stamp was refused: {issues:?}"
            ));
        }
        for bad in [
            "202+-10-29T12:00:00Z",
            "2026-10-29T1-:00:00Z",
            "2026-1.-29T12:00:00Z",
        ] {
            let (valid, issues) = report(&schemas, &wrap(bad));
            if valid || issues != vec![(IssueCode::InvalidInstance, path.to_owned())] {
                failures.push(format!("{selector}: {bad}: valid={valid} {issues:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
