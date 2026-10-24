//! A NaN or an infinity written where an authored scenario expects a value (beyond10x/ess#676).
//!
//! No ESS type has a non-finite value: `Decimal` is a patterned decimal string and `Binary64` a
//! finite bit pattern. YAML spells both anyway (`.nan`, `.inf`, `-.inf`), and [`Node`] — like
//! `serde_json::Value` — has nothing to hold them in, so its reader turns each into `null`. A
//! scenario then said "expected Decimal, found null" for a number the author did write, and at an
//! `Optional` it admitted the number as the absent value.
//!
//! [`read`] reads a value as [`Node`] does, and says where in it a non-finite number stood, so the
//! refusal can name the number and the reason.

use std::fmt;

use ess_primitives::json::{
    from_arbitrary_precision, number, CanonicalValue, ARBITRARY_PRECISION_NUMBER,
};
use ess_primitives::node::Node;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize as _;
use serde_json::{Map, Value};

/// Each non-finite number in a value: its position below the value (empty for the value itself,
/// `.member` or `[index]` below it) and the number.
pub(super) type Found = Vec<(String, f64)>;

/// A value as [`Node`] reads it, and each non-finite number in it.
pub(super) fn read<'de, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<(Node, Found), D::Error> {
    let mut found = Vec::new();
    let value = Walk {
        at: String::new(),
        found: &mut found,
    }
    .deserialize(deserializer)?;
    let node = Node::deserialize(value).map_err(<D::Error as de::Error>::custom)?;
    Ok((node, found))
}

/// How YAML spells `value`, a non-finite number.
pub(super) fn spelling(value: f64) -> &'static str {
    if value.is_nan() {
        ".nan"
    } else if value.is_sign_negative() {
        "-.inf"
    } else {
        ".inf"
    }
}

/// [`CanonicalValue`], recording each non-finite number where it stands rather than only turning
/// it into `null`.
struct Walk<'a> {
    at: String,
    found: &'a mut Found,
}

impl Walk<'_> {
    fn below(&mut self, step: &str) -> Walk<'_> {
        Walk {
            at: format!("{}{step}", self.at),
            found: self.found,
        }
    }
}

impl<'de> DeserializeSeed<'de> for Walk<'_> {
    type Value = Value;

    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Walk<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Visitor::expecting(&CanonicalValue, f)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        CanonicalValue.visit_bool(value)
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        CanonicalValue.visit_i64(value)
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        CanonicalValue.visit_u64(value)
    }

    fn visit_i128<E: de::Error>(self, value: i128) -> Result<Value, E> {
        CanonicalValue.visit_i128(value)
    }

    fn visit_u128<E: de::Error>(self, value: u128) -> Result<Value, E> {
        CanonicalValue.visit_u128(value)
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        if !value.is_finite() {
            self.found.push((self.at, value));
        }
        CanonicalValue.visit_f64(value)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        CanonicalValue.visit_str(value)
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        CanonicalValue.visit_string(value)
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        CanonicalValue.visit_none()
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        CanonicalValue.visit_unit()
    }

    fn visit_some<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        self.deserialize(deserializer)
    }

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut access: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) =
            access.next_element_seed(self.below(&format!("[{}]", items.len())))?
        {
            items.push(item);
        }
        Ok(Value::Array(items))
    }

    // As `CanonicalValue::visit_map`, member by member.
    fn visit_map<A: MapAccess<'de>>(mut self, mut access: A) -> Result<Value, A::Error> {
        let mut entries = Map::new();
        while let Some(key) = access.next_key::<String>()? {
            if key == ARBITRARY_PRECISION_NUMBER
                && entries.is_empty()
                && from_arbitrary_precision::<A::Error>()
            {
                let spelling: String = access.next_value()?;
                return number(&spelling)
                    .map(Value::Number)
                    .map_err(|_| de::Error::custom("number out of range"));
            }
            let value = access.next_value_seed(self.below(&format!(".{key}")))?;
            entries.insert(key, value);
        }
        Ok(Value::Object(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Read(Node, Found);

    impl<'de> serde::Deserialize<'de> for Read {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            read(deserializer).map(|(node, found)| Self(node, found))
        }
    }

    fn read_yaml(text: &str) -> (Node, Found) {
        let Read(node, found) = ess_yaml::from_str(text).expect("readable");
        (node, found)
    }

    #[test]
    fn a_non_finite_number_is_found_where_it_stands_and_read_as_node_reads_it() {
        let (node, found) = read_yaml("{a: .nan, b: [1, -.inf], c: {d: .inf}, e: 1.5}");
        let spelled: Vec<(String, &str)> = found
            .iter()
            .map(|(at, value)| (at.clone(), spelling(*value)))
            .collect();
        assert_eq!(
            spelled,
            [
                (".a".to_owned(), ".nan"),
                (".b[1]".to_owned(), "-.inf"),
                (".c.d".to_owned(), ".inf"),
            ]
        );
        let plain: Node =
            ess_yaml::from_str("{a: .nan, b: [1, -.inf], c: {d: .inf}, e: 1.5}").expect("read");
        assert_eq!(node, plain);
    }

    #[test]
    fn a_whole_value_is_found_at_no_position_and_a_finite_one_finds_nothing() {
        let (node, found) = read_yaml(".inf");
        assert_eq!(node, Node::Null);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "");
        let (_, found) = read_yaml("{a: 1, b: [x, 2.5]}");
        assert_eq!(found, Found::new());
    }
}
