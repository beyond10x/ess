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

/// One step of a path into a document: a mapping key, or a position in a sequence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    /// A mapping's string key.
    Key(String),
    /// A sequence's position.
    Index(usize),
}

/// The spelling of every plain number `document` holds as a float, by its path, read back out of
/// `text`, the text `document` was read from.
///
/// A YAML number reaches every reader as an `f64`, so `0.1000000000000000000001` and `0.1` are one
/// value by then. The digits survive only in the text, and only through a scalar read as a string,
/// which is what this does: it walks `text` beside `document`, which says which scalars are
/// floats, and reads each of those as the text it was written as. A number under a key that is not
/// text has no path and is left out.
///
/// Empty where the two disagree, which a `document` read from `text` never does.
pub fn float_spellings(text: &str, document: &Value) -> Vec<(Vec<Step>, String)> {
    let found = std::cell::RefCell::new(Vec::new());
    let guide = Guide {
        value: document,
        path: Vec::new(),
        found: &found,
    };
    match serde::de::DeserializeSeed::deserialize(guide, Deserializer::from_str(text)) {
        Ok(()) => found.into_inner(),
        Err(_) => Vec::new(),
    }
}

/// A walk of the text that follows the document it was read into.
struct Guide<'a> {
    value: &'a Value,
    path: Vec<Step>,
    found: &'a std::cell::RefCell<Vec<(Vec<Step>, String)>>,
}

impl<'de> serde::de::DeserializeSeed<'de> for Guide<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        match self.value {
            Value::Number(number) if number.is_f64() => deserializer.deserialize_str(self),
            Value::Mapping(_) => deserializer.deserialize_map(self),
            Value::Sequence(_) => deserializer.deserialize_seq(self),
            _ => deserializer
                .deserialize_ignored_any(serde::de::IgnoredAny)
                .map(drop),
        }
    }
}

impl<'de> serde::de::Visitor<'de> for Guide<'_> {
    type Value = ();

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the document it was read into")
    }

    fn visit_str<E: serde::de::Error>(self, spelling: &str) -> Result<(), E> {
        self.found
            .borrow_mut()
            .push((self.path, spelling.to_owned()));
        Ok(())
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        use serde::de::IgnoredAny;
        let Value::Mapping(entries) = self.value else {
            return Err(serde::de::Error::custom(
                "not the document it was read into",
            ));
        };
        for (key, value) in entries {
            if map.next_key::<IgnoredAny>()?.is_none() {
                break;
            }
            let Some(key) = key.as_str() else {
                map.next_value::<IgnoredAny>()?;
                continue;
            };
            let mut path = self.path.clone();
            path.push(Step::Key(key.to_owned()));
            map.next_value_seed(Guide {
                value,
                path,
                found: self.found,
            })?;
        }
        while map.next_key::<IgnoredAny>()?.is_some() {
            map.next_value::<IgnoredAny>()?;
        }
        Ok(())
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let Value::Sequence(items) = self.value else {
            return Err(serde::de::Error::custom(
                "not the document it was read into",
            ));
        };
        for (index, value) in items.iter().enumerate() {
            let mut path = self.path.clone();
            path.push(Step::Index(index));
            let more = seq.next_element_seed(Guide {
                value,
                path,
                found: self.found,
            })?;
            if more.is_none() {
                break;
            }
        }
        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
        Ok(())
    }
}

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

    #[test]
    fn a_float_is_read_back_as_the_digits_it_was_written_with() {
        let text = "a: 0.1000000000000000000001\n\
                    b:\n  - 2\n  - 0.5\n  - {c: 1e2}\n  - '0.25'\n\
                    d: [x, 3.0]\n\
                    ? [1]\n: 0.7\n";
        let document: Value = from_str(text).unwrap();
        let key = |name: &str| Step::Key(name.to_owned());
        assert_eq!(
            float_spellings(text, &document),
            vec![
                (vec![key("a")], "0.1000000000000000000001".to_owned()),
                (vec![key("b"), Step::Index(1)], "0.5".to_owned()),
                (vec![key("b"), Step::Index(2), key("c")], "1e2".to_owned()),
                (vec![key("d"), Step::Index(1)], "3.0".to_owned()),
            ]
        );
    }

    #[test]
    fn a_document_that_is_not_the_text_it_was_read_from_reads_no_spelling() {
        let document: Value = from_str("a: 0.5\n").unwrap();
        assert_eq!(float_spellings("a: [0.5]\n", &document).first(), None);
    }
}
