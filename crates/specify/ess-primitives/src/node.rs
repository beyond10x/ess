//! A format-neutral dynamic value.
//!
//! Some document fragments are genuinely untyped: a counterexample's input, an artifact's
//! metadata, the parameters of an external verifier. [`Node`] carries those without pulling a
//! specific serialization format into the domain crate, and it is the intermediate
//! representation the predicate parser works on, which is what lets one parser accept both
//! the string form (`tests.unit.failed == 0`) and the map form (`{all: [...]}`).

use std::collections::BTreeMap;
use std::fmt;

use crate::facts::Number;

/// A dynamic document value: null, boolean, number, string, sequence or mapping.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(untagged)]
pub enum Node {
    /// An absent value.
    #[default]
    Null,
    /// A boolean.
    Bool(bool),
    /// A number.
    Number(Number),
    /// A string.
    Text(String),
    /// An ordered sequence.
    Seq(Vec<Node>),
    /// A mapping with string keys, ordered for deterministic output.
    Map(BTreeMap<String, Node>),
}

impl<'de> serde::Deserialize<'de> for Node {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Not `serde_json::Value::deserialize`: under serde_json's `arbitrary_precision` that keeps
        // a number's spelling, reads `-0` as `0`, and admits `1e400` (`crate::json`).
        let value =
            serde::de::DeserializeSeed::deserialize(crate::json::CanonicalValue, deserializer)?;
        Self::from_value(value).map_err(serde::de::Error::custom)
    }
}

impl Node {
    fn from_value(value: serde_json::Value) -> Result<Self, String> {
        match value {
            serde_json::Value::Null => Ok(Self::Null),
            serde_json::Value::Bool(value) => Ok(Self::Bool(value)),
            // **An integer token is read as the integer, because the writer writes one.**
            //
            // The two doors are one decision (`docs/design/review-primitive-semantics.md`, *The
            // round-trip law*). At the base the writer emitted a binary64 for every value, so a
            // reader that kept the exact integer made `9223372036854775807` a value that was
            // admitted, written as `9.223372036854776e+18`, and then refused. `Serialize` now
            // writes the integer for a value binary64 does not carry, so reading it back exactly is
            // what closes the loop rather than what opens it.
            serde_json::Value::Number(value) => match value.as_i64() {
                Some(exact) => Ok(Self::Number(Number::from(exact))),
                None => match value.as_u64() {
                    Some(exact) => Ok(Self::Number(Number::from_integer(i128::from(exact)))),
                    None => value
                        .as_f64()
                        .ok_or_else(|| format!("number `{value}` does not fit the domain number"))
                        .and_then(|value| {
                            Number::new(value)
                                .map(Self::Number)
                                .map_err(|e| e.to_string())
                        }),
                },
            },
            serde_json::Value::String(value) => Ok(Self::Text(value)),
            serde_json::Value::Array(values) => values
                .into_iter()
                .map(Self::from_value)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Seq),
            serde_json::Value::Object(values) => values
                .into_iter()
                .map(|(key, value)| Self::from_value(value).map(|value| (key, value)))
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map(Self::Map),
        }
    }
}

/// The deepest [`Node::from_json_exact`] descends.
const EXACT_DEPTH: usize = 128;

impl Node {
    /// Reads one JSON document keeping the exact value of every number token.
    ///
    /// What a target adapter calls to hand the conformance runner a response it decoded from JSON:
    /// the runner compares a `Decimal` by exact value, so `0.1` and `0.10000000000000001` are two
    /// values there, as in the Go and TypeScript runners. [`Deserialize`](serde::Deserialize) reads
    /// every non-integer through binary64 because `serde_json` has parsed the token before ESS
    /// sees it. Each number is read by [`Number::from_json_token`] and so writes as it always
    /// wrote; the keys and the other values read as they do through `Deserialize`.
    ///
    /// It is for the exact observers, a complete subject and a retained result. A number it keeps
    /// exactly makes `Number ==` tell `0.1` from `0.10000000000000001` wherever the `Node` is
    /// compared, so do not use it for a view row compared as a `Binary64`, which is compared by
    /// its nearest binary64.
    ///
    /// # Errors
    ///
    /// When `text` is not one JSON value, nests past 128 levels, or holds a number
    /// [`Number::from_json_token`] refuses: past binary64, a token over 512 characters, or an
    /// exponent past ±512. Go and TypeScript refuse the same numbers.
    pub fn from_json_exact(text: &str) -> Result<Self, String> {
        let raw: &serde_json::value::RawValue =
            serde_json::from_str(text).map_err(|e| e.to_string())?;
        Self::from_raw(raw, 0)
    }

    fn from_raw(raw: &serde_json::value::RawValue, depth: usize) -> Result<Self, String> {
        if depth > EXACT_DEPTH {
            return Err("JSON nests too deeply".to_owned());
        }
        let text = raw.get().trim();
        let invalid = |e: serde_json::Error| e.to_string();
        match text.as_bytes().first() {
            Some(b'[') => serde_json::from_str::<Vec<&serde_json::value::RawValue>>(text)
                .map_err(invalid)?
                .into_iter()
                .map(|item| Self::from_raw(item, depth + 1))
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Seq),
            Some(b'{') => {
                serde_json::from_str::<BTreeMap<String, &serde_json::value::RawValue>>(text)
                    .map_err(invalid)?
                    .into_iter()
                    .map(|(key, item)| Self::from_raw(item, depth + 1).map(|item| (key, item)))
                    .collect::<Result<BTreeMap<_, _>, _>>()
                    .map(Self::Map)
            }
            Some(b'"') => serde_json::from_str(text).map(Self::Text).map_err(invalid),
            Some(b'n') => Ok(Self::Null),
            Some(b't') => Ok(Self::Bool(true)),
            Some(b'f') => Ok(Self::Bool(false)),
            _ => Number::from_json_token(text)
                .map(Self::Number)
                .ok_or_else(|| format!("number `{text}` does not fit the domain number")),
        }
    }

    /// The name of this node's shape, for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "a boolean",
            Self::Number(_) => "a number",
            Self::Text(_) => "a string",
            Self::Seq(_) => "a sequence",
            Self::Map(_) => "a mapping",
        }
    }

    /// The string contents, when this is a string.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The sequence contents, when this is a sequence.
    pub fn as_seq(&self) -> Option<&[Node]> {
        match self {
            Self::Seq(items) => Some(items),
            _ => None,
        }
    }

    /// The mapping contents, when this is a mapping.
    pub fn as_map(&self) -> Option<&BTreeMap<String, Node>> {
        match self {
            Self::Map(entries) => Some(entries),
            _ => None,
        }
    }

    /// A single-entry mapping's key and value, when this is a mapping of exactly one entry.
    ///
    /// This shape carries most of AEP's document syntax: `{all: [...]}`, `{derived_from: x}`,
    /// `{path: docs/designs/passkeys.md}`.
    pub fn as_single_entry(&self) -> Option<(&str, &Node)> {
        let entries = self.as_map()?;
        if entries.len() == 1 {
            entries
                .iter()
                .next()
                .map(|(key, value)| (key.as_str(), value))
        } else {
            None
        }
    }

    /// Interprets this node as a sequence, treating a single scalar as a one-element sequence.
    ///
    /// Documents are written by humans, who reasonably write `add: clean-room` where the
    /// schema says a list.
    pub fn as_seq_or_single(&self) -> Vec<&Node> {
        match self {
            Self::Seq(items) => items.iter().collect(),
            Self::Null => Vec::new(),
            other => vec![other],
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Number(value) => write!(f, "{value}"),
            Self::Text(value) => f.write_str(value),
            Self::Seq(items) => {
                f.write_str("[")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
            Self::Map(entries) => {
                f.write_str("{")?;
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                }
                f.write_str("}")
            }
        }
    }
}

impl schemars::JsonSchema for Node {
    fn schema_name() -> String {
        "Node".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject::default();
        schema.metadata().description =
            Some("Any JSON/YAML value: null, boolean, number, string, array or object.".to_owned());
        schema.into()
    }
}

impl From<bool> for Node {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Number> for Node {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_stays_a_number_when_serde_json_arbitrary_precision_is_unified() {
        let node: Node =
            serde_json::from_str(r#"{"amount":10.5,"count":2}"#).expect("the object deserializes");
        let fields = node.as_map().expect("a map");
        assert!(matches!(fields.get("amount"), Some(Node::Number(_))));
        assert!(matches!(fields.get("count"), Some(Node::Number(_))));
    }

    /// A token keeps its digits through [`Node::from_json_exact`], and a spelling of one value
    /// stays one value.
    #[test]
    fn an_exact_node_compares_by_the_digits_of_its_tokens() {
        for (left, right, equal) in [
            ("0.1", "0.10000000000000001", false),
            ("1", "1.0000000000000000001", false),
            (
                "123456789012345678901234567890",
                "123456789012345678901234567891",
                false,
            ),
            ("-9223372036854775809", "-9223372036854775810", false),
            ("1.5e0", "1.50", true),
            ("0", "-0.0", true),
            ("1E2", "100", true),
            ("9007199254740993", "9007199254740992", false),
            (
                "0.10000000000000001",
                "0.100000000000000010000000000000000000000000",
                true,
            ),
            ("0.1", "0.100000000000000010000000000000000000000000", false),
            ("1", "1.0000000000000000000000000000000000000001", false),
            ("0", "1e-400", false),
            ("1e-300", "1.00000000000000000001e-300", false),
            ("1e300", "1.00000000000000000001e300", false),
            ("-1e-400", "1e-400", false),
        ] {
            let (left, right) = (
                Node::from_json_exact(left).unwrap(),
                Node::from_json_exact(right).unwrap(),
            );
            assert_eq!(left == right, equal, "{left:?} {right:?}");
        }
        assert_eq!(
            serde_json::to_string(&Node::from_json_exact("-0.0").unwrap()).unwrap(),
            "-0.0"
        );
        assert!(Node::from_json_exact("1e400").is_err());
        for refused in ["1e513", "1e-513", "01", "+1", "-01", "00", "+0", ".5", "5."] {
            assert!(Number::from_json_token(refused).is_none(), "{refused}");
        }
        assert!(Number::from_json_token("1e-512").is_some());
        assert!(Number::from_json_token(&format!("1.{}", "0".repeat(511))).is_none());
        assert!(Number::from_json_token(&format!("1.{}", "0".repeat(510))).is_some());
        assert!(Node::from_json_exact("[1,").is_err());
        assert!(
            Node::from_json_exact(&format!("{}1{}", "[".repeat(200), "]".repeat(200))).is_err()
        );
    }
}
