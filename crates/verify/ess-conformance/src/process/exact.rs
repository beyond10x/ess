//! The exact reader and writer the protocol's lines go through.
//!
//! **Reading.** [`parse`] reads one line into a [`Value`] tree that keeps every number as the value
//! its token spells ([`Number::from_json_token`], the door [`Node::from_json_exact`] uses), refuses a
//! member name an object repeats (a map visitor keeps the last value, so a repeated name would
//! otherwise be silently read as one), and remembers the byte offset of every value. A refusal is a
//! [`Bad`]: a kind and an offset, never text from the line, because `serde_json`'s error text can
//! quote a value and `Node::from_json_exact`'s quotes a number token. It is linear in the line:
//! `serde_json` checks the syntax in one pass and the tree is built in one more, so a line nested
//! deep is not read again at every level. Nesting past [`DEPTH`] (128) levels below the line's own
//! value is refused, [`Bad::Shape`] at the first byte of the value too deep, by [`Reader::value`].
//!
//! **Writing.** [`ExactNode`] writes a [`Node`] with every number as [`Number::exact_text`], through
//! `serde_json`'s raw value, never through `Number`'s `Serialize`, which writes a binary64 and
//! would round a decimal whose digits binary64 does not carry.

use std::collections::BTreeMap;

use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::RawValue;

/// The deepest a line nests, as [`Node::from_json_exact`] bounds it.
const DEPTH: usize = 128;

#[cfg(test)]
mod tests;

#[cfg(test)]
thread_local! {
    /// The bytes of the line the reader has gone over since [`tests`] last reset it.
    static VISITED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Counts `bytes` gone over, under `cfg(test)` only.
#[cfg(test)]
fn visit(bytes: usize) {
    VISITED.with(|visited| visited.set(visited.get() + bytes));
}

#[cfg(not(test))]
fn visit(_: usize) {}

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
///
/// Two passes, each linear in the line: `serde_json` refuses anything that is not one JSON value,
/// at the byte it names, skipping the value without building it; then [`Reader`] builds the tree,
/// going over each byte once however deep the line nests.
pub(crate) fn parse(line: &str) -> Result<Value, Bad> {
    if line.trim().is_empty() {
        return Err(Bad::NotJson(0));
    }
    let raw: &RawValue = serde_json::from_str(line).map_err(|e| Bad::NotJson(column(&e)))?;
    let start = raw
        .get()
        .as_ptr()
        .addr()
        .saturating_sub(line.as_ptr().addr());
    Reader { line, at: start }.value(0)
}

/// The byte offset a `serde_json` error on a one-line document names. Nothing else of the error is
/// kept: its text can quote a value.
fn column(error: &serde_json::Error) -> usize {
    error.column().saturating_sub(1)
}

/// The building pass of [`parse`], over a line `serde_json` has found to be one JSON value, so it
/// meets no syntax it must refuse.
///
/// It refuses what the reader before it refused, at the same byte and in the same order: a value
/// nested past [`DEPTH`] at its first byte; a number [`Number::from_json_token`] refuses; a string
/// `serde_json` will not decode (a lone surrogate escape), where `serde_json` says; a repeated
/// member name at its value. Within one object every member name is decoded before any member is
/// read, so a name that does not decode outranks every refusal in that object's members; then
/// members are read in order, a repeated name before its value. Every value, read or refused,
/// leaves [`at`](Self::at) just past it, so the object around it can still decode the names after
/// it without going back.
struct Reader<'l> {
    line: &'l str,
    at: usize,
}

impl<'l> Reader<'l> {
    fn byte(&self) -> Option<u8> {
        self.line.as_bytes().get(self.at).copied()
    }

    fn step(&mut self, bytes: usize) {
        self.at += bytes;
        visit(bytes);
    }

    fn space(&mut self) {
        while matches!(self.byte(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.step(1);
        }
    }

    /// One value at `depth`, from the next non-space byte.
    fn value(&mut self, depth: usize) -> Result<Value, Bad> {
        self.space();
        let at = self.at;
        if depth > DEPTH {
            self.skip();
            return Err(Bad::Shape(at));
        }
        let kind = match self.byte() {
            Some(b'{') => self.object(depth)?,
            Some(b'[') => self.array(depth)?,
            Some(b'"') => Kind::Text(self.string()?),
            Some(b'n') => {
                self.step(4);
                Kind::Null
            }
            Some(b't') => {
                self.step(4);
                Kind::Bool(true)
            }
            Some(b'f') => {
                self.step(5);
                Kind::Bool(false)
            }
            _ => Kind::Number(Number::from_json_token(self.number()).ok_or(Bad::Number(at))?),
        };
        Ok(Value { at, kind })
    }

    fn object(&mut self, depth: usize) -> Result<Kind, Bad> {
        self.step(1);
        let mut map = BTreeMap::new();
        let mut refused = None;
        self.space();
        if self.byte() == Some(b'}') {
            self.step(1);
            return Ok(Kind::Map(map));
        }
        loop {
            self.space();
            let name = match self.string() {
                Ok(name) => name,
                Err(bad) => {
                    self.leave();
                    return Err(bad);
                }
            };
            self.space();
            self.step(1);
            self.space();
            if refused.is_some() {
                self.skip();
            } else {
                match map.entry(name) {
                    std::collections::btree_map::Entry::Occupied(_) => {
                        refused = Some(Bad::Shape(self.at));
                        self.skip();
                    }
                    std::collections::btree_map::Entry::Vacant(slot) => {
                        match self.value(depth + 1) {
                            Ok(item) => {
                                slot.insert(item);
                            }
                            Err(bad) => refused = Some(bad),
                        }
                    }
                }
            }
            if self.end_of_member(b'}') {
                return refused.map_or(Ok(Kind::Map(map)), Err);
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Kind, Bad> {
        self.step(1);
        let mut items = Vec::new();
        let mut refused = None;
        self.space();
        if self.byte() == Some(b']') {
            self.step(1);
            return Ok(Kind::Seq(items));
        }
        loop {
            if refused.is_some() {
                self.skip();
            } else {
                match self.value(depth + 1) {
                    Ok(item) => items.push(item),
                    Err(bad) => refused = Some(bad),
                }
            }
            if self.end_of_member(b']') {
                return refused.map_or(Ok(Kind::Seq(items)), Err);
            }
        }
    }

    /// Past the `,` after a member, or past `close`; whether it was `close`.
    fn end_of_member(&mut self, close: u8) -> bool {
        self.space();
        let closed = self.byte() == Some(close);
        self.step(1);
        closed
    }

    /// A string token, decoded by `serde_json` as the reader before decoded it.
    fn string(&mut self) -> Result<String, Bad> {
        let start = self.at;
        self.skip_string();
        let token = &self.line[start..self.at];
        visit(token.len());
        serde_json::from_str(token).map_err(|e| Bad::NotJson(start + column(&e)))
    }

    /// A number token.
    fn number(&mut self) -> &'l str {
        let start = self.at;
        while matches!(
            self.byte(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.step(1);
        }
        let line = self.line;
        &line[start..self.at]
    }

    fn skip_string(&mut self) {
        self.step(1);
        loop {
            match self.byte() {
                Some(b'\\') => self.step(2),
                Some(b'"') => {
                    self.step(1);
                    return;
                }
                Some(_) => self.step(1),
                None => return,
            }
        }
    }

    /// Past one value, building nothing.
    fn skip(&mut self) {
        self.space();
        match self.byte() {
            Some(b'"') => self.skip_string(),
            Some(b'{' | b'[') => {
                self.step(1);
                self.leave();
            }
            _ => {
                while !matches!(
                    self.byte(),
                    None | Some(b',' | b']' | b'}' | b' ' | b'\t' | b'\n' | b'\r')
                ) {
                    self.step(1);
                }
            }
        }
    }

    /// Past the end of the object or array [`at`](Self::at) is inside, outside any string.
    fn leave(&mut self) {
        let mut open = 1_usize;
        while open > 0 {
            match self.byte() {
                Some(b'"') => self.skip_string(),
                Some(b'{' | b'[') => {
                    open += 1;
                    self.step(1);
                }
                Some(b'}' | b']') => {
                    open -= 1;
                    self.step(1);
                }
                Some(_) => self.step(1),
                None => return,
            }
        }
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

/// The members one request may carry that the types reading it do not know.
const UNKNOWN: usize = 256;

/// The reads one request may take to find those members.
const PROBES: usize = 4096;

/// A request read by `T`'s own `Deserialize`, open at every depth (design "Requests are open"): a
/// member a type in it does not know, wherever it sits, is dropped, and every other member is read
/// as [`serde`] reads it. The types keep denying unknown members, because they also read closed
/// suite documents; a request is read by those same types, so their own validation, a `try_from`
/// included, runs on what is kept.
///
/// A type that denies a member refuses it as ``unknown field `name` ``, the spelling serde's
/// derive and `serde_json` give that refusal. Which object of the request held it is found by
/// renaming the member in one object at a time, the last in the line first, to a name that sorts
/// where it sat: the object whose renamed member is refused under the new name is the one. The
/// last first, because the line is read in order and a refusal is the first thing it met, so an
/// object after the one refused is never read and renaming there changes nothing, while an object
/// before it, whose type knows the name, would refuse the new one. Anything else is refused as
/// before: a type that is not a refusal of an unknown member, more than [`UNKNOWN`] such members,
/// or more than [`PROBES`] reads.
pub(crate) fn request<T: DeserializeOwned>(value: &Value) -> Result<T, Bad> {
    let read = |tree: &Value| serde_json::from_str::<T>(&normal(tree));
    let mut tree = value.clone();
    let mut probes = 0;
    for _ in 0..=UNKNOWN {
        let refusal = match read(&tree) {
            Ok(request) => return Ok(request),
            Err(error) => error.to_string(),
        };
        let Some((path, name)) = unknown_member(&tree, &refusal, &mut probes, |probe| {
            read(probe).err().map(|error| error.to_string())
        }) else {
            break;
        };
        if let Some(map) = object_at(&mut tree, &path) {
            map.remove(&name);
        }
    }
    Err(Bad::Shape(value.at))
}

/// One step from a value to a value inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Member(String),
    Item(usize),
}

/// Whether `refusal` is the refusal of the unknown member `name`.
fn refuses_unknown(refusal: &str, name: &str) -> bool {
    refusal
        .strip_prefix("unknown field `")
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.strip_prefix('`'))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(", ") || rest.starts_with(" at "))
}

/// The object holding the member `refusal` names unknown, and the member's name: see [`request`].
fn unknown_member(
    tree: &Value,
    refusal: &str,
    probes: &mut usize,
    read: impl Fn(&Value) -> Option<String>,
) -> Option<(Vec<Step>, String)> {
    let mut names = std::collections::BTreeSet::new();
    member_names(tree, &mut names);
    for name in names.iter().filter(|name| refuses_unknown(refusal, name)) {
        // The name a member renamed takes: nothing sorts between a name and itself followed by
        // NUL, so the member keeps its place among its object's members.
        let renamed = format!("{name}\u{0}");
        if names.contains(&renamed) {
            continue;
        }
        let mut holders = Vec::new();
        holding(tree, name, &mut Vec::new(), &mut holders);
        for path in holders.into_iter().rev() {
            *probes += 1;
            if *probes > PROBES {
                return None;
            }
            let mut probe = tree.clone();
            let map = object_at(&mut probe, &path)?;
            let member = map.remove(name)?;
            map.insert(renamed.clone(), member);
            if read(&probe).is_some_and(|refusal| refuses_unknown(&refusal, &renamed)) {
                return Some((path, name.clone()));
            }
        }
    }
    None
}

/// Every member name in `value`, however deep.
fn member_names(value: &Value, out: &mut std::collections::BTreeSet<String>) {
    match &value.kind {
        Kind::Seq(items) => items.iter().for_each(|item| member_names(item, out)),
        Kind::Map(map) => {
            for (name, item) in map {
                out.insert(name.clone());
                member_names(item, out);
            }
        }
        _ => {}
    }
}

/// The path of every object in `value` holding a member `name`, in the order the line reaches
/// that member.
fn holding(value: &Value, name: &str, path: &mut Vec<Step>, out: &mut Vec<Vec<Step>>) {
    match &value.kind {
        Kind::Seq(items) => {
            for (index, item) in items.iter().enumerate() {
                path.push(Step::Item(index));
                holding(item, name, path, out);
                path.pop();
            }
        }
        Kind::Map(map) => {
            for (member, item) in map {
                if member == name {
                    out.push(path.clone());
                }
                path.push(Step::Member(member.clone()));
                holding(item, name, path, out);
                path.pop();
            }
        }
        _ => {}
    }
}

/// The object at `path` in `value`.
fn object_at<'v>(value: &'v mut Value, path: &[Step]) -> Option<&'v mut BTreeMap<String, Value>> {
    let mut at = value;
    for step in path {
        at = match (&mut at.kind, step) {
            (Kind::Map(map), Step::Member(name)) => map.get_mut(name)?,
            (Kind::Seq(items), Step::Item(index)) => items.get_mut(*index)?,
            _ => return None,
        };
    }
    match &mut at.kind {
        Kind::Map(map) => Some(map),
        _ => None,
    }
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
