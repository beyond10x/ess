//! Decode at the text boundary without hiding precision loss in a caller's JSON parser.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{value::RawValue, Number, Value};

use super::Refused;
use super::{finding, path};

type Result<T> = std::result::Result<T, Refused>;

pub(super) fn parse(text: &str) -> Result<Value> {
    let raw: Box<RawValue> = serde_json::from_str(text)
        .map_err(|_| error("/input", "input_syntax", "expected one complete JSON value"))?;
    decode(&raw, "/input", 0)
}

fn decode(raw: &RawValue, at: &str, depth: usize) -> Result<Value> {
    if depth > 64 {
        return Err(error(at, "input_depth", "JSON input exceeds 64 levels"));
    }
    let text = raw.get();
    match text.as_bytes()[0] {
        b'{' => {
            #[derive(Deserialize)]
            struct Object(
                #[serde(deserialize_with = "super::unique_map")] BTreeMap<String, Box<RawValue>>,
            );
            let object: Object = serde_json::from_str(text)
                .map_err(|_| error(at, "input_syntax", "JSON object keys must be unique"))?;
            object
                .0
                .iter()
                .map(|(key, value)| {
                    decode(value, &path(at, key), depth + 1).map(|value| (key.clone(), value))
                })
                .collect::<Result<serde_json::Map<_, _>>>()
                .map(Value::Object)
        }
        b'[' => {
            let values: Vec<Box<RawValue>> = serde_json::from_str(text)
                .map_err(|_| error(at, "input_syntax", "invalid JSON list"))?;
            values
                .iter()
                .enumerate()
                .map(|(index, value)| decode(value, &format!("{at}/{index}"), depth + 1))
                .collect::<Result<_>>()
                .map(Value::Array)
        }
        b'-' | b'0'..=b'9' => number(text, at).map(Value::Number),
        _ => {
            serde_json::from_str(text).map_err(|_| error(at, "input_syntax", "invalid JSON scalar"))
        }
    }
}

fn number(text: &str, at: &str) -> Result<Number> {
    if !text.contains(['.', 'e', 'E']) {
        if let Ok(integer) = text.parse::<i64>() {
            return Ok(integer.into());
        }
        if let Ok(integer) = text.parse::<u64>() {
            return Ok(integer.into());
        }
    }
    let float: f64 = serde_json::from_str(text).map_err(|_| {
        error(
            at,
            "input_number",
            "JSON number is outside the supported representation",
        )
    })?;
    let number = Number::from_f64(float).ok_or_else(|| {
        error(
            at,
            "input_number",
            "JSON number is outside the supported representation",
        )
    })?;
    let original = decimal(text);
    if original.is_none() || original != decimal(&number.to_string()) {
        return Err(error(
            at,
            "input_number",
            "JSON number cannot round-trip without precision loss",
        ));
    }
    Ok(number)
}

// JSON grammar is already checked by RawValue. This key compares decimal values exactly,
// without using binary floating point to decide whether binary decoding lost information.
fn decimal(text: &str) -> Option<(bool, String, i128)> {
    let negative = text.starts_with('-');
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (significand, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
    let (whole, fraction) = significand.split_once('.').unwrap_or((significand, ""));
    let digits = format!("{whole}{fraction}");
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Some((false, String::new(), 0));
    }
    let trimmed = digits.trim_end_matches('0');
    let exponent = exponent
        .parse::<i128>()
        .ok()?
        .checked_sub(i128::try_from(fraction.len()).ok()?)?
        .checked_add(i128::try_from(digits.len() - trimmed.len()).ok()?)?;
    Some((negative, trimmed.to_owned(), exponent))
}

fn error(at: &str, rule: &str, detail: &str) -> Refused {
    Refused(vec![finding(at, rule, detail)])
}
