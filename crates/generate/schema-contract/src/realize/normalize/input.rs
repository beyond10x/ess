//! Decode at the text boundary without hiding precision loss in a caller's JSON parser.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{value::RawValue, Number, Value};

use super::{finding, path};
use super::{NumberPath, Refused};

type Result<T> = std::result::Result<T, Refused>;

pub(super) fn parse(text: &str, paths: &[Vec<NumberPath>]) -> Result<Value> {
    let raw: Box<RawValue> = serde_json::from_str(text)
        .map_err(|_| error("/input", "input_syntax", "expected one complete JSON value"))?;
    decode(
        &raw,
        "/input",
        0,
        &paths.iter().map(Vec::as_slice).collect::<Vec<_>>(),
    )
}

fn decode(raw: &RawValue, at: &str, depth: usize, paths: &[&[NumberPath]]) -> Result<Value> {
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
                    decode(value, &path(at, key), depth + 1, &descend(paths, Some(key)))
                        .map(|value| (key.clone(), value))
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
                .map(|(index, value)| {
                    decode(
                        value,
                        &format!("{at}/{index}"),
                        depth + 1,
                        &descend(paths, None),
                    )
                })
                .collect::<Result<_>>()
                .map(Value::Array)
        }
        b'-' | b'0'..=b'9' => {
            if paths.iter().any(|path| path.is_empty()) {
                binary64(text, at).map(Value::Number)
            } else {
                number(text, at).map(Value::Number)
            }
        }
        _ => {
            serde_json::from_str(text).map_err(|_| error(at, "input_syntax", "invalid JSON scalar"))
        }
    }
}

fn descend<'a>(paths: &[&'a [NumberPath]], field: Option<&str>) -> Vec<&'a [NumberPath]> {
    paths
        .iter()
        .filter_map(|path| match (path.split_first(), field) {
            (Some((NumberPath::Field { name }, rest)), Some(field)) if name == field => Some(rest),
            (Some((NumberPath::Items, rest)), None) => Some(rest),
            _ => None,
        })
        .collect()
}

pub(super) fn prepare(value: &Value, paths: &[Vec<NumberPath>]) -> Result<Value> {
    convert(
        value,
        "/input",
        &paths.iter().map(Vec::as_slice).collect::<Vec<_>>(),
    )
}

fn convert(value: &Value, at: &str, paths: &[&[NumberPath]]) -> Result<Value> {
    if paths.is_empty() {
        return Ok(value.clone());
    }
    match value {
        Value::Object(fields) => fields
            .iter()
            .map(|(key, value)| {
                convert(value, &path(at, key), &descend(paths, Some(key)))
                    .map(|value| (key.clone(), value))
            })
            .collect::<Result<serde_json::Map<_, _>>>()
            .map(Value::Object),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .map(|(index, value)| convert(value, &format!("{at}/{index}"), &descend(paths, None)))
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Value::Number(number) if paths.iter().any(|path| path.is_empty()) => {
            binary64(&number.to_string(), at).map(Value::Number)
        }
        _ => Ok(value.clone()),
    }
}

fn binary64(text: &str, at: &str) -> Result<Number> {
    super::numeric::finite(text)
        .and_then(Number::from_f64)
        .ok_or_else(|| {
            error(
                at,
                "input_number",
                "JSON number is outside finite binary64 representation",
            )
        })
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
