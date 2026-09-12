//! `uniqueItems`, decided the same way at every array length.
//!
//! Schema equality treats `0`, `0.0` and `-0.0` as one number. The dependency's own decision
//! changes with array length: it compares pairwise up to fifteen elements and switches to a hash
//! set beyond that, and its hash of a number is the raw `f64` bit pattern, which the two signed
//! zeros do not share. A sixteenth element therefore made an array holding both zeros unique.
//! Here the hash only groups candidates and every candidate pair is settled by the dependency's
//! own exact comparison, so no hash can decide an array's uniqueness.
//!
//! This file is compiled by this crate **and emitted verbatim** into every Rust normalization
//! runtime it generates, so a projection cannot decide `uniqueItems` differently from the
//! reference it was generated from. It builds no validator itself: it supplies the keyword, and
//! each side attaches it to its own options.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use jsonschema::json::cmp;
use jsonschema::paths::Location;
use jsonschema::{Keyword, ValidationError};
use serde_json::{Map, Number, Value};

/// The `uniqueItems` keyword, for `with_keyword("uniqueItems", compile)`.
///
/// Attaching it replaces the dependency's own `uniqueItems`, whose equality and hash disagree
/// about signed zero and which therefore decides an array by its length.
// The `Result` and the two unread arguments are the factory signature `with_keyword` imposes.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn compile<'a>(
    _parent: &'a Map<String, Value>,
    value: &'a Value,
    _path: Location,
) -> Result<Box<dyn for<'i> Keyword<'i>>, ValidationError<'a>> {
    Ok(Box::new(UniqueItems {
        asserted: value == &Value::Bool(true),
    }))
}

/// The keyword as the dependency itself compiles it: only a literal `true` restricts anything.
struct UniqueItems {
    asserted: bool,
}

impl<'i> Keyword<'i> for UniqueItems {
    fn validate(&self, instance: &'i Value) -> Result<(), ValidationError<'i>> {
        if self.is_valid(instance) {
            Ok(())
        } else {
            // The message the dependency's own `uniqueItems` error prints, unchanged.
            Err(ValidationError::custom(format!(
                "{instance} has non-unique elements"
            )))
        }
    }

    fn is_valid(&self, instance: &'i Value) -> bool {
        match instance {
            Value::Array(items) if self.asserted => unique(items),
            _ => true,
        }
    }
}

/// Whether no two elements are equal under JSON Schema equality.
///
/// The key groups elements that *may* be equal; equality itself is always the dependency's
/// comparison, so a key collision costs a comparison and can never decide the outcome.
fn unique(items: &[Value]) -> bool {
    let mut candidates: HashMap<u64, Vec<&Value>> = HashMap::with_capacity(items.len());
    for item in items {
        let bucket = candidates.entry(key(item)).or_default();
        if bucket.iter().any(|seen| cmp::equal(seen, item)) {
            return false;
        }
        bucket.push(item);
    }
    true
}

/// A key equal values always share.
fn key(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash(value, &mut hasher);
    hasher.finish()
}

fn hash<H: Hasher>(value: &Value, state: &mut H) {
    match value {
        Value::Null => state.write_u8(0),
        Value::Bool(item) => {
            state.write_u8(1);
            item.hash(state);
        }
        Value::Number(item) => {
            state.write_u8(2);
            number_key(item).hash(state);
        }
        Value::String(item) => {
            state.write_u8(3);
            item.hash(state);
        }
        Value::Array(items) => {
            state.write_u8(4);
            state.write_usize(items.len());
            for item in items {
                hash(item, state);
            }
        }
        Value::Object(entries) => {
            // Combined without regard to order, so two equal objects agree however they are held.
            state.write_u8(5);
            let mut combined = 0_u64;
            for (name, item) in entries {
                let mut entry = DefaultHasher::new();
                name.hash(&mut entry);
                hash(item, &mut entry);
                combined ^= entry.finish();
            }
            state.write_u64(combined);
        }
    }
}

/// The exact value of a number where it is an integer, and its bits otherwise.
///
/// Equal numbers share a key: an integer and a floating spelling of the same integer both key on
/// that integer, `-0.0` keys on `0`, and two equal non-integral doubles have one bit pattern.
#[derive(Hash)]
enum NumberKey {
    Integer(i128),
    Bits(u64),
}

fn number_key(number: &Number) -> NumberKey {
    if let Some(value) = number.as_u64() {
        return NumberKey::Integer(i128::from(value));
    }
    if let Some(value) = number.as_i64() {
        return NumberKey::Integer(i128::from(value));
    }
    let Some(value) = number.as_f64() else {
        // Unreachable without `serde_json/arbitrary_precision`, which this graph does not select.
        // One shared key keeps grouping correct at the cost of comparisons.
        return NumberKey::Bits(0);
    };
    if value.is_finite() && value.fract() == 0.0 && value.abs() < 2f64.powi(127) {
        // Exact: the fractional part is zero and the value is inside `i128`.
        #[allow(clippy::cast_possible_truncation)]
        NumberKey::Integer(value as i128)
    } else {
        NumberKey::Bits(value.to_bits())
    }
}

#[cfg(test)]
mod tests {
    use super::{key, unique};
    use serde_json::json;

    #[test]
    fn equal_numbers_key_alike_however_they_are_spelled() {
        for (left, right) in [
            (json!(0), json!(-0.0)),
            (json!(0), json!(0.0)),
            (json!(0.0), json!(-0.0)),
            (json!(1), json!(1.0)),
            (json!(-1), json!(-1.0)),
            (json!(i64::MIN), json!(-9_223_372_036_854_775_808.0)),
            (json!({"z": [0]}), json!({"z": [-0.0]})),
        ] {
            assert_eq!(key(&left), key(&right), "{left} {right}");
            assert!(!unique(&[left, right]));
        }
    }

    #[test]
    fn exact_neighbours_and_other_types_stay_distinct() {
        for (left, right) in [
            (
                json!(9_007_199_254_740_992_i64),
                json!(9_007_199_254_740_993_i64),
            ),
            (json!(i64::MIN), json!(u64::MAX)),
            (json!(0), json!(0.5)),
            (json!(0), json!("0")),
            (json!([0]), json!([0, 0])),
            (json!({"z": 0}), json!({"y": 0})),
        ] {
            assert!(unique(&[left, right]));
        }
    }
}
