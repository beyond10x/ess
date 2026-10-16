//! YAML for ESS.
//!
//! The only crate in this workspace that names a YAML library. Every other crate reads and writes
//! YAML through it (`infra-*` through `ess_primitives::yaml`, so that crate stays their only ESS
//! dependency), so the backend is this crate's business alone
//! (`docs/design/yaml-parser-replacement.md`, beyond10x/ess#541).
//!
//! The API is the subset of `serde_yaml` 0.9 that ESS uses, with the same names and meanings:
//! scalar resolution, duplicate-key behaviour per target, alias limits, `<<` as an ordinary key,
//! tags, the tag form of enums, error wording with its ` at line L column C` suffix, and the bytes
//! [`to_string`] writes.
//!
//! The backend today is `serde_norway`, a fork of `serde_yaml` 0.9 that is byte- and
//! behaviour-identical by measurement and does not depend on `unsafe-libyaml`. It is a stopgap:
//! an ESS-owned parser and emitter are planned behind this same API.
//!
//! One departure from the backend, on the writing side: a `serde_json::Number` is written as the
//! number it denotes in every build (beyond10x/ess#579). With `serde_json`'s
//! `arbitrary_precision` feature unified into a build, a `Number` serializes as a one-key map
//! under [`SERDE_JSON_NUMBER`] for every serializer but `serde_json`'s own, and the backend writes
//! that map faithfully. [`to_value`] and [`to_string`] write the number instead, held as the build
//! without the feature holds it, so no emitted byte depends on an adopter's dependency graph.

#![forbid(unsafe_code)]

pub use serde_norway::{
    from_slice, from_str, from_value, Deserializer, Error, Mapping, Number, Value,
};

use serde::Serialize;

/// The key a `serde_json::Number` is serialized under when `arbitrary_precision` is on.
pub const SERDE_JSON_NUMBER: &str = "$serde_json::private::Number";

/// `value` as a YAML value, with every `serde_json` number token written as its number.
///
/// # Errors
///
/// What the backend's `to_value` refuses.
pub fn to_value<T: ?Sized + Serialize>(value: &T) -> Result<Value, Error> {
    serde_norway::to_value(value).map(plain_numbers)
}

/// `value` as YAML text, with every `serde_json` number token written as its number.
///
/// The backend's bytes wherever no token appears, so nothing that was right moves.
///
/// # Errors
///
/// What the backend's `to_string` refuses.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String, Error> {
    let text = serde_norway::to_string(value)?;
    if !text.contains(SERDE_JSON_NUMBER) {
        return Ok(text);
    }
    serde_norway::to_string(&to_value(value)?)
}

/// `value` with each one-key map under [`SERDE_JSON_NUMBER`] replaced by the number it spells.
fn plain_numbers(value: Value) -> Value {
    match value {
        Value::Mapping(mapping) => {
            if let Some(number) = token(&mapping) {
                return Value::Number(number);
            }
            Value::Mapping(
                mapping
                    .into_iter()
                    .map(|(key, value)| (plain_numbers(key), plain_numbers(value)))
                    .collect(),
            )
        }
        Value::Sequence(items) => Value::Sequence(items.into_iter().map(plain_numbers).collect()),
        Value::Tagged(mut tagged) => {
            let inner = std::mem::replace(&mut tagged.value, Value::Null);
            tagged.value = plain_numbers(inner);
            Value::Tagged(tagged)
        }
        other => other,
    }
}

/// The number a `serde_json` token map spells, where `mapping` is one.
fn token(mapping: &Mapping) -> Option<Number> {
    if mapping.len() != 1 {
        return None;
    }
    let (Value::String(key), Value::String(spelling)) = mapping.iter().next()? else {
        return None;
    };
    if key != SERDE_JSON_NUMBER {
        return None;
    }
    number(spelling)
}

/// A JSON number's spelling as `serde_json` without `arbitrary_precision` holds it: an integer
/// that fits as `u64` or `i64`, `-0` and every other number as the nearest `f64`.
fn number(spelling: &str) -> Option<Number> {
    let integer = spelling
        .strip_prefix('-')
        .unwrap_or(spelling)
        .bytes()
        .all(|byte| byte.is_ascii_digit());
    if integer {
        if let Ok(value) = spelling.parse::<u64>() {
            return Some(Number::from(value));
        }
        if let Ok(value) = spelling.parse::<i64>() {
            if value != 0 {
                return Some(Number::from(value));
            }
        }
    }
    spelling
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(Number::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The map `serde_json` hands a foreign serializer for a number under `arbitrary_precision`.
    fn token_map(spelling: &str) -> Value {
        let mut mapping = Mapping::new();
        mapping.insert(SERDE_JSON_NUMBER.into(), spelling.into());
        Value::Mapping(mapping)
    }

    #[test]
    fn a_number_token_is_written_as_its_number() {
        let document = Value::Sequence(vec![
            token_map("2"),
            token_map("-3"),
            token_map("18446744073709551615"),
            token_map("2.50"),
            token_map("-0"),
        ]);
        assert_eq!(
            to_string(&document).unwrap(),
            "- 2\n- -3\n- 18446744073709551615\n- 2.5\n- -0.0\n"
        );
        assert_eq!(
            to_value(&document).unwrap(),
            Value::Sequence(vec![
                Value::Number(2u64.into()),
                Value::Number((-3i64).into()),
                Value::Number(u64::MAX.into()),
                Value::Number(2.5f64.into()),
                Value::Number((-0.0f64).into()),
            ])
        );
    }

    #[test]
    fn a_map_that_is_not_a_number_token_is_written_as_it_is() {
        let mut two = Mapping::new();
        two.insert(SERDE_JSON_NUMBER.into(), "2".into());
        two.insert("other".into(), "x".into());
        let document = Value::Sequence(vec![Value::Mapping(two), token_map("not a number")]);
        assert_eq!(
            to_string(&document).unwrap(),
            serde_norway::to_string(&document).unwrap()
        );
    }
}
