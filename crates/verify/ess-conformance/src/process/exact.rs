//! The exact reader and writer the protocol's lines go through.
//!
//! **Reading.** [`parse`] reads one line into a [`Value`] tree that keeps every number as the value
//! its token spells ([`Number::from_json_token`], the door [`Node::from_json_exact`] uses), refuses a
//! member name an object repeats (a map visitor keeps the last value, so a repeated name would
//! otherwise be silently read as one), and remembers the byte offset of every value. A refusal is a
//! [`Bad`]: a kind and an offset, never text from the line, because `serde_json`'s error text can
//! quote a value and `Node::from_json_exact`'s quotes a number token.
//!
//! **Writing.** [`ExactNode`] writes a [`Node`] with every number as [`Number::exact_text`], through
//! `serde_json`'s raw value, never through `Number`'s `Serialize`, which writes a binary64 and
//! would round a decimal whose digits binary64 does not carry.

use std::collections::BTreeMap;
use std::fmt;

use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use serde::de::{DeserializeOwned, MapAccess, Visitor};
use serde::Serialize;
use serde_json::value::RawValue;

/// The deepest a line nests, as [`Node::from_json_exact`] bounds it.
const DEPTH: usize = 128;

/// Why a line or a member of it was refused: what, and at which byte of the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bad {
    /// Not one JSON value.
    NotJson(usize),
    /// A number token [`Number::from_json_token`] refuses.
    Number(usize),
    /// Valid JSON of the wrong shape: an unknown, missing or repeated member, or a wrong type.
    Shape(usize),
}

/// One value of a line, with the byte offset it starts at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Value {
    /// Where the value starts in the line.
    pub at: usize,
    /// What it is.
    pub kind: Kind,
}

/// The shape of a [`Value`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    Null,
    Bool(bool),
    Number(Number),
    Text(String),
    Seq(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

/// Reads one line exactly. See the [module documentation](self).
pub(crate) fn parse(line: &str) -> Result<Value, Bad> {
    if line.trim().is_empty() {
        return Err(Bad::NotJson(0));
    }
    let raw: &RawValue = serde_json::from_str(line).map_err(|e| Bad::NotJson(column(&e)))?;
    build(line, raw, 0)
}

/// The byte offset a `serde_json` error on a one-line document names. Nothing else of the error is
/// kept: its text can quote a value.
fn column(error: &serde_json::Error) -> usize {
    error.column().saturating_sub(1)
}

/// Where `part`, a slice of `line`, starts in it.
fn offset(line: &str, part: &str) -> usize {
    part.as_ptr().addr().saturating_sub(line.as_ptr().addr())
}

fn build(line: &str, raw: &RawValue, depth: usize) -> Result<Value, Bad> {
    let text = raw.get().trim();
    let at = offset(line, text);
    if depth > DEPTH {
        return Err(Bad::Shape(at));
    }
    let again = |e: serde_json::Error| Bad::NotJson(at + column(&e));
    let kind = match text.as_bytes().first() {
        Some(b'{') => {
            let Entries(entries) = serde_json::from_str(text).map_err(again)?;
            let mut map = BTreeMap::new();
            for (name, item) in entries {
                if map.contains_key(&name) {
                    return Err(Bad::Shape(offset(line, item.get())));
                }
                let item = build(line, item, depth + 1)?;
                map.insert(name, item);
            }
            Kind::Map(map)
        }
        Some(b'[') => {
            let items: Vec<&RawValue> = serde_json::from_str(text).map_err(again)?;
            Kind::Seq(
                items
                    .into_iter()
                    .map(|item| build(line, item, depth + 1))
                    .collect::<Result<_, _>>()?,
            )
        }
        Some(b'"') => Kind::Text(serde_json::from_str(text).map_err(again)?),
        Some(b'n') => Kind::Null,
        Some(b't') => Kind::Bool(true),
        Some(b'f') => Kind::Bool(false),
        _ => Kind::Number(Number::from_json_token(text).ok_or(Bad::Number(at))?),
    };
    Ok(Value { at, kind })
}

/// An object's members in the order the line writes them, repeated names included.
struct Entries<'a>(Vec<(String, &'a RawValue)>);

impl<'de: 'a, 'a> serde::Deserialize<'de> for Entries<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntryVisitor<'a>(std::marker::PhantomData<&'a ()>);
        impl<'de: 'a, 'a> Visitor<'de> for EntryVisitor<'a> {
            type Value = Entries<'a>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, &'a RawValue>()? {
                    entries.push(entry);
                }
                Ok(Entries(entries))
            }
        }
        deserializer.deserialize_map(EntryVisitor(std::marker::PhantomData))
    }
}

// ---- typed reading -------------------------------------------------------------------------------

/// The members of a closed object, checked: every name known, every required one present.
pub(crate) struct Members<'v> {
    map: &'v BTreeMap<String, Value>,
}

impl<'v> Members<'v> {
    /// A required member. Only call it for a name passed as required to [`object`].
    pub fn req(&self, name: &str) -> &'v Value {
        &self.map[name]
    }

    /// An optional member: `None` when it is absent or `null`.
    pub fn opt(&self, name: &str) -> Option<&'v Value> {
        self.map
            .get(name)
            .filter(|value| !matches!(value.kind, Kind::Null))
    }
}

/// `value` as a closed object of `required` and `optional` members.
pub(crate) fn object<'v>(
    value: &'v Value,
    required: &[&str],
    optional: &[&str],
) -> Result<Members<'v>, Bad> {
    let Kind::Map(map) = &value.kind else {
        return Err(Bad::Shape(value.at));
    };
    for (name, member) in map {
        if !required.contains(&name.as_str()) && !optional.contains(&name.as_str()) {
            return Err(Bad::Shape(member.at));
        }
    }
    if required.iter().any(|name| !map.contains_key(*name)) {
        return Err(Bad::Shape(value.at));
    }
    Ok(Members { map })
}

/// `value` as an object, open: for a request, whose unknown members a reader ignores.
pub(crate) fn open_object(value: &Value) -> Result<&BTreeMap<String, Value>, Bad> {
    match &value.kind {
        Kind::Map(map) => Ok(map),
        _ => Err(Bad::Shape(value.at)),
    }
}

/// `value` as a string.
pub(crate) fn text(value: &Value) -> Result<String, Bad> {
    match &value.kind {
        Kind::Text(text) => Ok(text.clone()),
        _ => Err(Bad::Shape(value.at)),
    }
}

/// `value` as a boolean.
pub(crate) fn boolean(value: &Value) -> Result<bool, Bad> {
    match value.kind {
        Kind::Bool(flag) => Ok(flag),
        _ => Err(Bad::Shape(value.at)),
    }
}

/// `value` as a non-negative integer, by value: `20` and `20.0` are one number.
pub(crate) fn unsigned(value: &Value) -> Result<u64, Bad> {
    match value.kind {
        Kind::Number(number) => match number.exact_parts() {
            Some((units, 0)) => u64::try_from(units).map_err(|_| Bad::Shape(value.at)),
            _ => Err(Bad::Shape(value.at)),
        },
        _ => Err(Bad::Shape(value.at)),
    }
}

/// `value` as a sequence.
pub(crate) fn seq(value: &Value) -> Result<&[Value], Bad> {
    match &value.kind {
        Kind::Seq(items) => Ok(items),
        _ => Err(Bad::Shape(value.at)),
    }
}

/// `value` as a [`Node`], every number exact.
pub(crate) fn node(value: &Value) -> Node {
    match &value.kind {
        Kind::Null => Node::Null,
        Kind::Bool(flag) => Node::Bool(*flag),
        Kind::Number(number) => Node::Number(*number),
        Kind::Text(text) => Node::Text(text.clone()),
        Kind::Seq(items) => Node::Seq(items.iter().map(node).collect()),
        Kind::Map(map) => Node::Map(
            map.iter()
                .map(|(name, item)| (name.clone(), node(item)))
                .collect(),
        ),
    }
}

/// `value` as a value map: an object of [`Node`]s, whatever names it holds.
pub(crate) fn node_map(value: &Value) -> Result<BTreeMap<String, Node>, Bad> {
    match &value.kind {
        Kind::Map(map) => Ok(map
            .iter()
            .map(|(name, item)| (name.clone(), node(item)))
            .collect()),
        _ => Err(Bad::Shape(value.at)),
    }
}

/// `value` read by `T`'s own `Deserialize`, for a part that holds no [`Node`]: a reference, a
/// token, a tagged enum. Its numbers are handed over as their exact text, so an integer reads by
/// value whatever its spelling. The error, which can quote the value, is dropped.
pub(crate) fn serde<T: DeserializeOwned>(value: &Value) -> Result<T, Bad> {
    serde_json::from_str(&normal(value)).map_err(|_| Bad::Shape(value.at))
}

/// `value` written back as JSON, every number as its exact text.
fn normal(value: &Value) -> String {
    let mut out = String::new();
    write_normal(value, &mut out);
    out
}

fn write_normal(value: &Value, out: &mut String) {
    match &value.kind {
        Kind::Null => out.push_str("null"),
        Kind::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Kind::Number(number) => out.push_str(&number.exact_text()),
        Kind::Text(text) => {
            out.push_str(&serde_json::to_string(text).unwrap_or_else(|_| "null".to_owned()));
        }
        Kind::Seq(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_normal(item, out);
            }
            out.push(']');
        }
        Kind::Map(map) => {
            out.push('{');
            for (index, (name, item)) in map.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(name).unwrap_or_else(|_| "\"\"".to_owned()));
                out.push(':');
                write_normal(item, out);
            }
            out.push('}');
        }
    }
}

// ---- exact writing -------------------------------------------------------------------------------

/// A [`Node`] written with every number as [`Number::exact_text`].
pub(crate) struct ExactNode<'a>(pub &'a Node);

impl Serialize for ExactNode<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Node::Null => serializer.serialize_unit(),
            Node::Bool(flag) => serializer.serialize_bool(*flag),
            Node::Number(number) => RawValue::from_string(number.exact_text())
                .map_err(<S::Error as serde::ser::Error>::custom)?
                .serialize(serializer),
            Node::Text(text) => serializer.serialize_str(text),
            Node::Seq(items) => serializer.collect_seq(items.iter().map(ExactNode)),
            Node::Map(map) => serializer.collect_map(map.iter().map(|(k, v)| (k, ExactNode(v)))),
        }
    }
}

/// A value map written exactly.
pub(crate) struct ExactMap<'a>(pub &'a BTreeMap<String, Node>);

impl Serialize for ExactMap<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.0.iter().map(|(k, v)| (k, ExactNode(v))))
    }
}

/// An optional value map written exactly, `null` when absent.
pub(crate) fn exact_opt(map: Option<&BTreeMap<String, Node>>) -> Option<ExactMap<'_>> {
    map.map(ExactMap)
}
