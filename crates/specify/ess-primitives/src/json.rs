//! JSON values whose numbers mean one thing in every `serde_json` build.
//!
//! `serde_json`'s `arbitrary_precision` feature changes what a number *is*. Without it a
//! `serde_json::Value` holds a `u64`, an `i64` or a binary64; with it, the literal spelling. The
//! feature is not ESS's choice: `entity-core` enables it, and Cargo unifies features across a build,
//! so a workspace or an adopter that builds ESS beside Entity Runtime gets it everywhere. With it on:
//!
//! * `1.50`, `1E2` and `1e400` are held as written, so they print, compare and hash differently,
//!   and `1e400` is admitted where the build without the feature refuses it;
//! * `-0` is the integer `0`, where it was the binary64 `-0.0`;
//! * an integer past 64 bits is an integer, where it was a binary64;
//! * a number reaches a [`Visitor`] as a one-entry map under [`ARBITRARY_PRECISION_NUMBER`] — or,
//!   for `-0`, as `visit_i64(0)` — rather than through `visit_f64`;
//! * a number serialises as a one-entry struct, which every format but JSON writes as a mapping.
//!
//! The functions here answer as the build **without** the feature does, in both builds, so ESS
//! reads, compares and writes a number the same way whichever `serde_json` it was linked against.

use std::fmt;
use std::sync::OnceLock;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserializer, Serialize};
use serde_json::{Map, Number, Value};

/// The key `serde_json` hands a visitor a number under when `arbitrary_precision` is on.
pub const ARBITRARY_PRECISION_NUMBER: &str = "$serde_json::private::Number";

/// Whether the linked `serde_json` holds numbers as their spelling.
pub fn arbitrary_precision() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        serde_json::from_str::<Value>("1.50").is_ok_and(|value| {
            value
                .as_number()
                .is_some_and(|number| number.to_string() == "1.50")
        })
    })
}

/// Whether a visitor is being driven by a `serde_json` that has `arbitrary_precision` on.
///
/// That door, and only that one, sends a number as a map under [`ARBITRARY_PRECISION_NUMBER`],
/// sends `-0` as `visit_i64(0)`, and sends an integer past 64 bits as `visit_u128`/`visit_i128`.
/// Without the feature `serde_json` never calls any of those, so a visitor that answers them — when
/// this holds — as the build without the feature does is exact in both builds, and leaves every
/// other format's integers and maps alone. The door is recognised by its error type.
pub fn from_arbitrary_precision<E>() -> bool {
    arbitrary_precision()
        && std::any::type_name::<E>() == std::any::type_name::<serde_json::Error>()
}

/// The number the build without `arbitrary_precision` reads for a JSON number token.
///
/// A non-negative integer that fits a `u64` and a negative one that fits an `i64` are those
/// integers; everything else — a fraction, an exponent, `-0`, an integer past 64 bits — is the
/// binary64 `serde_json`'s own float parser gives, which is the parser that build uses. A token
/// past binary64 is refused, as that build refuses it.
///
/// # Errors
///
/// When `spelling` is not a JSON number or is outside finite binary64.
pub fn number(spelling: &str) -> Result<Number, serde_json::Error> {
    if !spelling.contains(['.', 'e', 'E']) {
        if let Ok(value) = spelling.parse::<u64>() {
            return Ok(Number::from(value));
        }
        if let Ok(value) = spelling.parse::<i64>() {
            if value < 0 {
                return Ok(Number::from(value));
            }
        }
    }
    // `deserialize_f64` goes through `serde_json`'s number parser whichever features are on.
    let value: f64 = serde_json::from_str(spelling)?;
    Number::from_f64(value).ok_or_else(|| de::Error::custom("number out of range"))
}

/// Reads one JSON document as the build without `arbitrary_precision` reads it.
///
/// # Errors
///
/// As [`serde_json::from_str`], and for a number outside finite binary64 in either build.
pub fn from_str(text: &str) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let value = CanonicalValue.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

/// [`from_str`] over bytes.
///
/// # Errors
///
/// As [`serde_json::from_slice`], and for a number outside finite binary64 in either build.
pub fn from_slice(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = CanonicalValue.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

/// A value someone else parsed, with every number as the build without `arbitrary_precision`
/// holds it.
///
/// Exact for every value except `-0`: `serde_json` with the feature reads that as the integer `0`
/// and nothing downstream can tell the two apart, which is why a document should come through
/// [`from_str`] rather than through `serde_json::from_str` and then here.
///
/// # Errors
///
/// For a number outside finite binary64.
pub fn canonical(value: &Value) -> Result<Value, serde_json::Error> {
    Ok(match value {
        Value::Number(number) => Value::Number(self::number(&number.to_string())?),
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect::<Result<_, _>>()?),
        Value::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| Ok((key.clone(), canonical(value)?)))
                .collect::<Result<_, serde_json::Error>>()?,
        ),
        other => other.clone(),
    })
}

/// YAML for a serialisable value whose numbers are plain YAML numbers in both builds.
///
/// With `arbitrary_precision` on, every `serde_json::Number` inside `value` serialises as a
/// one-entry struct, which `serde_yaml` writes as a mapping. This writes each such number as the
/// number [`number`] reads for its spelling. Without the feature it is `serde_yaml::to_string`.
///
/// # Errors
///
/// As [`serde_yaml::to_string`].
pub fn to_yaml_string<T: Serialize + ?Sized>(value: &T) -> Result<String, serde_yaml::Error> {
    if !arbitrary_precision() {
        return serde_yaml::to_string(value);
    }
    serde_yaml::to_string(&to_yaml_value(value)?)
}

/// [`to_yaml_string`]'s value, for a caller that goes on editing it.
///
/// # Errors
///
/// As [`serde_yaml::to_value`].
pub fn to_yaml_value<T: Serialize + ?Sized>(
    value: &T,
) -> Result<serde_yaml::Value, serde_yaml::Error> {
    let mut value = serde_yaml::to_value(value)?;
    if arbitrary_precision() {
        plain_numbers(&mut value)?;
    }
    Ok(value)
}

fn plain_numbers(value: &mut serde_yaml::Value) -> Result<(), serde_yaml::Error> {
    match value {
        serde_yaml::Value::Mapping(entries) => {
            if entries.len() == 1 {
                if let Some(serde_yaml::Value::String(spelling)) =
                    entries.get(ARBITRARY_PRECISION_NUMBER)
                {
                    let number = number(spelling)
                        .map_err(<serde_yaml::Error as serde::ser::Error>::custom)?;
                    *value = yaml_number(&number);
                    return Ok(());
                }
            }
            for (_, entry) in entries.iter_mut() {
                plain_numbers(entry)?;
            }
        }
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                plain_numbers(item)?;
            }
        }
        serde_yaml::Value::Tagged(tagged) => plain_numbers(&mut tagged.value)?,
        _ => {}
    }
    Ok(())
}

/// The YAML number the build without `arbitrary_precision` writes for a `serde_json::Number`.
fn yaml_number(number: &Number) -> serde_yaml::Value {
    serde_yaml::Value::Number(if let Some(value) = number.as_u64() {
        value.into()
    } else if let Some(value) = number.as_i64() {
        value.into()
    } else {
        number
            .as_f64()
            .expect("a canonical number is finite")
            .into()
    })
}

/// A [`Value`] read as the build without `arbitrary_precision` reads it, from any deserializer.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalValue;

impl<'de> DeserializeSeed<'de> for CanonicalValue {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for CanonicalValue {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        // `serde_json` sends a negative integer here, and `0` only for the token `-0`, which only
        // `arbitrary_precision` reads as an integer.
        if value == 0 && from_arbitrary_precision::<E>() {
            return spelled("-0");
        }
        Ok(Value::from(value))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::from(value))
    }

    // From `serde_json` these come only with `arbitrary_precision`, for an integer past 64 bits,
    // which the build without it reads as a binary64. From another format they are that format's
    // integers, which `serde_json::Value` holds only within 64 bits: past them it refuses with
    // this message, which is the one the build without the feature gives.
    fn visit_i128<E: de::Error>(self, value: i128) -> Result<Value, E> {
        if from_arbitrary_precision::<E>() {
            return spelled(&value.to_string());
        }
        u64::try_from(value)
            .map(Value::from)
            .or_else(|_| i64::try_from(value).map(Value::from))
            .map_err(|_| E::custom("JSON number out of range"))
    }

    fn visit_u128<E: de::Error>(self, value: u128) -> Result<Value, E> {
        if from_arbitrary_precision::<E>() {
            return spelled(&value.to_string());
        }
        u64::try_from(value)
            .map(Value::from)
            .map_err(|_| E::custom("JSON number out of range"))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        // As `serde_json::Value` does: only a non-JSON format can hand over a non-finite value.
        Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        self.deserialize(deserializer)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = access.next_element_seed(CanonicalValue)? {
            items.push(item);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut entries = Map::new();
        while let Some(key) = access.next_key::<String>()? {
            if key == ARBITRARY_PRECISION_NUMBER
                && entries.is_empty()
                && from_arbitrary_precision::<A::Error>()
            {
                let spelling: String = access.next_value()?;
                return spelled(&spelling);
            }
            let value = access.next_value_seed(CanonicalValue)?;
            entries.insert(key, value);
        }
        Ok(Value::Object(entries))
    }
}

fn spelled<E: de::Error>(spelling: &str) -> Result<Value, E> {
    number(spelling)
        .map(Value::Number)
        .map_err(|_| E::custom("number out of range"))
}
