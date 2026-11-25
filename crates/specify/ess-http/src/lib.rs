//! Where the commands and views of one exact ESS are served over HTTP: method and path.
//!
//! A command is a request and a view is a read; neither holds a route, so the model keeps no path
//! or verb (`ess_domain::component`, "No variant names a wire, a port, a path or a verb"). A
//! published HTTP protocol fixes some routes itself and leaves others to each server, and that
//! has to be stated somewhere a projection and a conformance adapter can read. [`HttpSpec`] is
//! that place: adopter-authored `ess-http/1`, pinned to one specification's source digest the way
//! `ess-transport/1` is, and compiled by [`compile`] against that specification's [`EssIr`] into a
//! deterministic [`HttpIr`]. The design is `docs/design/http-presentation-binding.md`.
//!
//! Nothing here serves a request. An operation the binding does not name keeps the route ESS
//! derives for it.

use std::collections::BTreeMap;
use std::fmt;

use ess_compiler::ir::{
    ResolvedBody, ResolvedCommand, ResolvedField, ResolvedOutcome, ResolvedTypeRef, ResolvedView,
};
use ess_compiler::unknown_instance::unknown_instance_answer;
use ess_compiler::EssIr;
use ess_domain::component::Reach;
use ess_domain::types::Primitive;
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};

pub mod status;

/// The adopter-authored HTTP binding format.
pub const HTTP_FORMAT: &str = "ess-http/1";

/// The compiled HTTP binding format.
pub const HTTP_IR_FORMAT: &str = "ess-http-ir/1";

/// Every code [`compile`] and [`compile_refining`] can refuse with, in code order.
pub const CODES: &[&str] = &[
    "ESS-HTTP-001",
    "ESS-HTTP-002",
    "ESS-HTTP-003",
    "ESS-HTTP-004",
    "ESS-HTTP-005",
    "ESS-HTTP-006",
    "ESS-HTTP-007",
    "ESS-HTTP-008",
    "ESS-HTTP-009",
    "ESS-HTTP-010",
    "ESS-HTTP-011",
    "ESS-HTTP-012",
    "ESS-HTTP-013",
    "ESS-HTTP-014",
    "ESS-HTTP-020",
    "ESS-HTTP-021",
    "ESS-HTTP-022",
    "ESS-HTTP-023",
    "ESS-HTTP-024",
    "ESS-HTTP-025",
    "ESS-HTTP-026",
    "ESS-HTTP-027",
    "ESS-HTTP-028",
    "ESS-HTTP-029",
    "ESS-HTTP-030",
    "ESS-HTTP-031",
    "ESS-HTTP-032",
    "ESS-HTTP-033",
    "ESS-HTTP-034",
    "ESS-HTTP-035",
    "ESS-HTTP-036",
    "ESS-HTTP-037",
    "ESS-HTTP-038",
];

/// The wording of a route collision, the same as the `OpenAPI` import's refusal of two operations
/// on one location, so the two read alike.
pub const COLLISION: &str = "two operations claim the same method and path";

/// The largest request body both generated servers read when a binding declares no `limits`, in
/// bytes: 1 MiB.
pub const FIXED_REQUEST_BYTES: u64 = 1_048_576;

/// The largest `limits.request_bytes` a binding may declare, in bytes: 16 MiB.
pub const MAX_REQUEST_BYTES: u64 = 16_777_216;

/// The exact ESS an HTTP binding binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificationIdentity {
    /// The system's qualified name.
    pub system: String,
    /// The specification version.
    pub version: String,
    /// `sha256:<hex>` of the specification's source.
    pub source_digest: String,
}

/// The HTTP methods a route may name: the closed lower-case set the `OpenAPI` import reads. Which
/// of them a route may use depends on what it serves; see [`compile`].
///
/// Declared in name order, which is the order routes on one path are compiled in.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    /// `DELETE`.
    Delete,
    /// `GET`.
    Get,
    /// `HEAD`.
    Head,
    /// `OPTIONS`.
    Options,
    /// `PATCH`.
    Patch,
    /// `POST`.
    Post,
    /// `PUT`.
    Put,
    /// `TRACE`.
    Trace,
}

impl Method {
    /// Every method, in name order.
    pub const ALL: [Self; 8] = [
        Self::Delete,
        Self::Get,
        Self::Head,
        Self::Options,
        Self::Patch,
        Self::Post,
        Self::Put,
        Self::Trace,
    ];

    /// The method as a binding and an `OpenAPI` path item spell it: lower case.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Get => "get",
            Self::Head => "head",
            Self::Options => "options",
            Self::Patch => "patch",
            Self::Post => "post",
            Self::Put => "put",
            Self::Trace => "trace",
        }
    }

    /// The method a binding spells `text`, if it is one: exactly, in lower case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|method| method.as_str() == text)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How a command's request carries its input (`docs/design/http-presentation-binding.md`,
/// "Request encoding").
///
/// Declared in the order the format lists them; `json` is the default.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    /// An `application/json` body, ESS's own, less any field mapped to the path.
    #[default]
    Json,
    /// An `application/x-www-form-urlencoded` body, UTF-8.
    Form,
    /// No body: every input field the path does not map is in the query string. The only
    /// encoding a command on `get` may use, and one a command on `delete` may use too.
    Query,
    /// A `multipart/form-data` body (RFC 7578): one part per input field the path does not map, a
    /// `Bytes` field a file part, its bytes raw, every other field a text part spelled as `form`
    /// spells it (beyond10x/ess#754).
    Multipart,
}

impl Encoding {
    /// Every encoding, in the order the format lists them.
    pub const ALL: [Self; 4] = [Self::Json, Self::Form, Self::Query, Self::Multipart];

    /// The encoding as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Form => "form",
            Self::Query => "query",
            Self::Multipart => "multipart",
        }
    }

    /// The encoding a binding spells `text`, if it is one: exactly, in lower case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|encoding| encoding.as_str() == text)
    }

    // `skip_serializing_if` passes the field by reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// What a `form` or `query` request does with a parameter no input field names; `refuse` is the
/// default, and the only answer a `json` body gives.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Unknown {
    /// A parameter or member no field names, or one the path carries, makes the request malformed.
    #[default]
    Refuse,
    /// A parameter or member no field names is ignored (RFC 6749 §3.1), and one the path carries
    /// is admitted when its value is the path's (beyond10x/ess#752).
    Ignore,
}

impl Unknown {
    /// Every value, in the order the format lists them.
    pub const ALL: [Self; 2] = [Self::Refuse, Self::Ignore];

    /// The value as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Refuse => "refuse",
            Self::Ignore => "ignore",
        }
    }

    /// The value a binding spells `text`, if it is one: exactly, in lower case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|unknown| unknown.as_str() == text)
    }

    // `skip_serializing_if` passes the field by reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl fmt::Display for Unknown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A literal a `select` tests a member of the body for: a JSON string, Boolean or integer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemberLiteral {
    /// `true` or `false`.
    Boolean(bool),
    /// An integer within 64 bits.
    Integer(i64),
    /// A string.
    String(String),
}

impl MemberLiteral {
    /// The literal `value` is, if it is one: a string, a Boolean or an integer within 64 bits.
    pub fn of(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::Bool(value) => Some(Self::Boolean(*value)),
            serde_json::Value::String(value) => Some(Self::String(value.clone())),
            serde_json::Value::Number(value) => value.as_i64().map(Self::Integer),
            _ => None,
        }
    }

    /// The literal as a JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Boolean(value) => serde_json::Value::Bool(*value),
            Self::Integer(value) => serde_json::Value::from(*value),
            Self::String(value) => serde_json::Value::String(value.clone()),
        }
    }
}

impl fmt::Display for MemberLiteral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::Integer(value) => write!(formatter, "{value}"),
            Self::String(value) => formatter.write_str(value),
        }
    }
}

impl serde::Serialize for MemberLiteral {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Boolean(value) => serializer.serialize_bool(*value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::String(value) => serializer.serialize_str(value),
        }
    }
}

impl<'de> serde::Deserialize<'de> for MemberLiteral {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::of(&value).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "`{value}` is not a literal a `select` tests for: a string, a Boolean or an integer"
            ))
        })
    }
}

/// One alternative of a route's `select`: the members of the `json` body it tests, each with its
/// literal, in member order, none an object above another. A body matches it when every tested
/// member holds its literal; a member it does not test matches any value, absent included. With no
/// test it is the catch-all, which matches every body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Select(pub Vec<(MemberPath, MemberLiteral)>);

/// How two alternatives meet, when one body can match both.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Meeting {
    /// They test the same members for the same literals.
    Same,
    /// One tests every member the other tests, for the same literals, and more: it is the more
    /// specific, and it answers.
    Nested,
    /// Each tests a member the other leaves open: the first's and the second's.
    Crossed(MemberPath, MemberPath),
}

impl Select {
    /// Each tested member with its literal, in member order.
    pub fn tests(&self) -> impl Iterator<Item = (&MemberPath, &MemberLiteral)> {
        self.0.iter().map(|(member, literal)| (member, literal))
    }

    /// The literal `member` is tested for, if it is tested.
    pub fn literal(&self, member: &MemberPath) -> Option<&MemberLiteral> {
        self.tests()
            .find(|(tested, _)| *tested == member)
            .map(|(_, literal)| literal)
    }

    /// How `self` and `other` meet, `None` when no body matches both: they test one member for two
    /// literals, or a member one tests is an object above one the other tests, which a literal is
    /// not.
    fn meet(&self, other: &Self) -> Option<Meeting> {
        for (member, literal) in self.tests() {
            for (theirs, their_literal) in other.tests() {
                let apart = if member == theirs {
                    literal != their_literal
                } else {
                    member.covers(theirs) || theirs.covers(member)
                };
                if apart {
                    return None;
                }
            }
        }
        let open = |one: &Self, other: &Self| {
            one.tests()
                .find(|(member, _)| other.literal(member).is_none())
                .map(|(member, _)| member.clone())
        };
        Some(match (open(self, other), open(other, self)) {
            (None, None) => Meeting::Same,
            (Some(mine), Some(theirs)) => Meeting::Crossed(mine, theirs),
            _ => Meeting::Nested,
        })
    }

    /// A body that matches both `self` and `other`, which meet, written as JSON with a space after
    /// each `:` and `,`.
    fn common_body(&self, other: &Self) -> String {
        fn put(
            level: &mut serde_json::Map<String, serde_json::Value>,
            names: &[String],
            value: serde_json::Value,
        ) {
            let [first, rest @ ..] = names else {
                return;
            };
            if rest.is_empty() {
                level.insert(first.clone(), value);
            } else if let serde_json::Value::Object(next) = level
                .entry(first.clone())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
            {
                put(next, rest, value);
            }
        }
        let mut body = serde_json::Map::new();
        for (member, literal) in self.tests().chain(other.tests()) {
            put(&mut body, member.segments(), literal.to_json());
        }
        spaced(&serde_json::Value::Object(body))
    }
}

/// `value` as JSON with a space after each `:` and `,`, each object's members in name order.
fn spaced(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let members: BTreeMap<&String, &serde_json::Value> = map.iter().collect();
            let shown: Vec<String> = members
                .into_iter()
                .map(|(name, value)| {
                    format!(
                        "{}: {}",
                        serde_json::Value::from(name.as_str()),
                        spaced(value)
                    )
                })
                .collect();
            format!("{{{}}}", shown.join(", "))
        }
        other => other.to_string(),
    }
}

impl fmt::Display for Select {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tests: Vec<String> = self
            .tests()
            .map(|(member, literal)| format!("{member}: {literal}"))
            .collect();
        write!(formatter, "{{{}}}", tests.join(", "))
    }
}

impl serde::Serialize for Select {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (member, literal) in &self.0 {
            map.serialize_entry(member, literal)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Select {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let Entries(entries) = Entries::<MemberLiteral>::deserialize(deserializer)?;
        let mut select = Vec::new();
        for (text, literal) in entries {
            let member = MemberPath::parse(&text).ok_or_else(|| {
                serde::de::Error::custom(format!("`select` tests `{text}`, not a member path"))
            })?;
            select.push((member, literal));
        }
        Self::checked(select).map_err(serde::de::Error::custom)
    }
}

impl Select {
    /// The alternative testing `tests`, once it is held to its rules: at least one test, in member
    /// order, each member once and none an object above another.
    fn checked(tests: Vec<(MemberPath, MemberLiteral)>) -> Result<Self, String> {
        if tests.is_empty() {
            return Err("a `select` map is empty: it tests at least one member".to_owned());
        }
        for (index, (member, _)) in tests.iter().enumerate().skip(1) {
            let (before, _) = &tests[index - 1];
            if before >= member {
                return Err(format!(
                    "`select` is out of member order: `{member}` follows `{before}`; compiled IR \
                     orders a select's members, each once"
                ));
            }
        }
        if let Some((one, other)) = tests.iter().enumerate().find_map(|(index, (one, _))| {
            tests[..index]
                .iter()
                .find(|(other, _)| other.covers(one) || one.covers(other))
                .map(|(other, _)| (one, other))
        }) {
            return Err(format!(
                "`select` tests `{other}` and `{one}`, one an object above the other, which a \
                 literal is not"
            ));
        }
        Ok(Self(tests))
    }
}

/// A route's compiled `select`: its alternatives, each a [`Select`], in the order written, no two
/// meeting. Compiled IR writes one alternative as its map and several as a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection(pub Vec<Select>);

impl Selection {
    /// The alternatives, in the order written.
    pub fn alternatives(&self) -> &[Select] {
        &self.0
    }

    /// The first two alternatives, by index, that one body can match, if any.
    fn meeting(alternatives: &[Select]) -> Option<(usize, usize)> {
        (0..alternatives.len()).find_map(|later| {
            (0..later)
                .find(|&earlier| alternatives[earlier].meet(&alternatives[later]).is_some())
                .map(|earlier| (earlier, later))
        })
    }
}

impl fmt::Display for Selection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_slice() {
            [one] => write!(formatter, "{one}"),
            many => {
                let shown: Vec<String> = many.iter().map(ToString::to_string).collect();
                write!(formatter, "[{}]", shown.join(", "))
            }
        }
    }
}

impl serde::Serialize for Selection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0.as_slice() {
            [one] => one.serialize(serializer),
            many => many.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Selection {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Written {
            One(Select),
            Many(Vec<Select>),
        }
        let alternatives = match Written::deserialize(deserializer)? {
            Written::One(one) => vec![one],
            Written::Many(many) if many.len() >= 2 => many,
            Written::Many(_) => return Err(serde::de::Error::custom(
                "`select` is a list of fewer than two maps; compiled IR writes one alternative \
                     as its map",
            )),
        };
        if let Some((earlier, later)) = Self::meeting(&alternatives) {
            return Err(serde::de::Error::custom(format!(
                "`select` alternatives {earlier} and {later} meet; alternatives are disjoint"
            )));
        }
        Ok(Self(alternatives))
    }
}

/// A route's `request`, as written. Each value is read as text so that one outside its set is
/// refused with a code and a path (`ESS-HTTP-020`) rather than as a parse error.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestSpec {
    /// `json`, `form`, `query` or `multipart`; absent, `json`.
    #[serde(default)]
    pub encoding: Option<String>,
    /// `refuse` or `ignore`; absent, `refuse`.
    #[serde(default)]
    pub unknown: Option<String>,
    /// `input.<field>`: the one input field a `json` body carries bare, its JSON value the whole
    /// body; absent, the body is the input object.
    #[serde(default)]
    pub body: Option<String>,
    /// The attributes of a `multipart` request's file parts, by the `Bytes` input field each part
    /// carries (beyond10x/ess#754); held to their rules by [`compile`] (`ESS-HTTP-036`).
    #[serde(default)]
    pub parts: Option<BTreeMap<String, PartSpec>>,
    /// `input.<field>` each: the input fields read from the query string beside the body, each under
    /// its wire name with the flat spelling `encoding: query` reads (beyond10x/ess#754); held to
    /// their rules by [`compile`] (`ESS-HTTP-020`, `ESS-HTTP-021`).
    #[serde(default)]
    pub query: Option<Vec<String>>,
    /// `<input field>: <member path>` each: the input fields a `json` body reads from a nested
    /// member, the path one or more member names joined by `.`; held to their rules by [`compile`]
    /// (`ESS-HTTP-020`).
    #[serde(default)]
    pub members: Option<BTreeMap<String, String>>,
}

/// A path of JSON object members, outermost first: one or more names, each non-empty and holding no
/// `.`, written joined by `.` (`data.ticket_id`). `request.members` reads an input field from one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberPath(Vec<String>);

impl MemberPath {
    /// The path `text` spells, if it is one: one or more non-empty names joined by `.`.
    pub fn parse(text: &str) -> Option<Self> {
        let segments: Vec<String> = text.split('.').map(str::to_owned).collect();
        segments
            .iter()
            .all(|segment| !segment.is_empty())
            .then_some(Self(segments))
    }

    /// The member names, outermost first; at least one.
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// The top-level member the path starts at.
    pub fn top(&self) -> &str {
        &self.0[0]
    }

    /// Whether `self` is `other` or an object above it: every name of `self` starts `other`.
    pub fn covers(&self, other: &Self) -> bool {
        other.0.starts_with(&self.0)
    }
}

impl fmt::Display for MemberPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.join("."))
    }
}

impl serde::Serialize for MemberPath {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for MemberPath {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "`{text}` is not a member path: one or more member names, each non-empty, joined \
                 by `.`"
            ))
        })
    }
}

/// A compiled request's `members`: each input field a `json` body reads from a nested member, with
/// its path, in the command's field order. Two fields never share a path, and no path is an object
/// above another.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Members(pub Vec<(String, MemberPath)>);

impl Members {
    /// Whether it maps no field.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Each field with its path, in field order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &MemberPath)> {
        self.0.iter().map(|(field, path)| (field.as_str(), path))
    }

    /// The path `field` is read from, if it is mapped.
    pub fn get(&self, field: &str) -> Option<&MemberPath> {
        self.iter()
            .find(|(name, _)| *name == field)
            .map(|(_, path)| path)
    }

    /// The first rule the mapped paths break, if one: a path twice, or one an object above another.
    fn clash(&self) -> Option<(usize, usize)> {
        (0..self.0.len()).find_map(|later| {
            (0..later)
                .find(|&earlier| {
                    let (one, other) = (&self.0[earlier].1, &self.0[later].1);
                    one.covers(other) || other.covers(one)
                })
                .map(|earlier| (earlier, later))
        })
    }
}

impl serde::Serialize for Members {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (field, path) in &self.0 {
            map.serialize_entry(field, path)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Members {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let Entries(entries) = Entries::<MemberPath>::deserialize(deserializer)?;
        if entries.is_empty() {
            return Err(serde::de::Error::custom(
                "`request.members` is written empty, which compiled IR omits",
            ));
        }
        if let Some((index, (field, _))) = entries
            .iter()
            .enumerate()
            .find(|(index, (field, _))| entries[..*index].iter().any(|(seen, _)| seen == field))
        {
            return Err(serde::de::Error::custom(format!(
                "`request.members` maps `{field}` twice (entry {index}); a field is read from one \
                 member"
            )));
        }
        let members = Self(entries);
        if let Some((earlier, later)) = members.clash() {
            return Err(serde::de::Error::custom(format!(
                "`request.members` reads `{}` from `{}` and `{}` from `{}`: two fields never share \
                 a member, and no member is an object above another",
                members.0[earlier].0, members.0[earlier].1, members.0[later].0, members.0[later].1
            )));
        }
        Ok(members)
    }
}

/// One file part's attributes, as written: what reads its filename and its media type into the
/// input, and the media types it may carry. Each source is read as text, so that one outside its
/// rules is refused with a code and a path (`ESS-HTTP-036`).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartSpec {
    /// `input.<field>`: the `String` input field the part's filename is read into.
    #[serde(default)]
    pub filename: Option<String>,
    /// `input.<field>`: the `String` input field the part's media type is read into.
    #[serde(default)]
    pub media_type: Option<String>,
    /// The media types the part may carry, RFC 6838 `type/subtype`; another is malformed.
    #[serde(default)]
    pub media_types: Option<Vec<String>>,
}

/// One file part of a compiled `multipart` request: what reads its filename and media type into
/// the input, and the media types it may carry, each omitted when the binding states none.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    /// The `String` input field the part's filename is read into.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "part_source"
    )]
    pub filename: Option<Source>,
    /// The `String` input field the part's media type is read into.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "part_source"
    )]
    pub media_type: Option<Source>,
    /// The media types the part may carry, in the order written; empty, any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media_types: Vec<String>,
}

/// Reads a compiled part attribute's source, which names a command's input field.
fn part_source<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Source>, D::Error> {
    match <Source as serde::Deserialize>::deserialize(deserializer)? {
        source @ Source::Input(_) => Ok(Some(source)),
        source @ Source::Param(_) => Err(serde::de::Error::custom(format!(
            "a part's `filename` or `media_type` is `{source}`; it reads into one input field of \
             a command, `input.<field>`"
        ))),
    }
}

/// Reads a compiled request's `parts`, refusing them written empty: compiled IR omits them.
fn written_parts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Part>, D::Error> {
    let parts = <BTreeMap<String, Part> as serde::Deserialize>::deserialize(deserializer)?;
    if parts.is_empty() {
        return Err(serde::de::Error::custom(
            "`request.parts` is written empty, which compiled IR omits",
        ));
    }
    Ok(parts)
}

/// Reads a compiled request's `query`, refusing it written empty, which compiled IR omits, and an
/// entry that is not `input.<field>`.
fn written_query<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Source>, D::Error> {
    let query = <Vec<Source> as serde::Deserialize>::deserialize(deserializer)?;
    if query.is_empty() {
        return Err(serde::de::Error::custom(
            "`request.query` is written empty, which compiled IR omits",
        ));
    }
    if let Some(source) = query
        .iter()
        .find(|source| matches!(source, Source::Param(_)))
    {
        return Err(serde::de::Error::custom(format!(
            "`request.query` lists `{source}`; it lists input fields of a command, `input.<field>`"
        )));
    }
    Ok(query)
}

/// Whether `text` is an RFC 6838 media type without parameters, `type/subtype`: each name 1 to 127
/// characters, starting with a letter or a digit, of letters, digits and ``!#$&-^_.+``.
pub fn is_media_type(text: &str) -> bool {
    let name = |part: &str| {
        (1..=127).contains(&part.len())
            && part
                .bytes()
                .next()
                .is_some_and(|first| first.is_ascii_alphanumeric())
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&byte))
    };
    text.split_once('/')
        .is_some_and(|(kind, subtype)| name(kind) && name(subtype))
}

/// A compiled route's `request`. Each key whose value is its default is omitted from the IR, and
/// a route whose request is all defaults carries none, so two documents that mean the same compile
/// to the same bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// The one input field a `json` body carries bare (`input.<field>`), every other input field
    /// being carried by the path; absent, the body is the input object less what the path
    /// carries.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "input_body"
    )]
    pub body: Option<Source>,
    /// How the request carries the input.
    #[serde(
        default,
        skip_serializing_if = "Encoding::is_default",
        deserialize_with = "omitted_default"
    )]
    pub encoding: Encoding,
    /// The input fields a `json` body reads from a nested member, each with its member path, in
    /// the command's field order; empty, and omitted, when the binding maps none. Every other
    /// input field the path and the query string do not carry is a top-level member under its wire
    /// name.
    #[serde(default, skip_serializing_if = "Members::is_empty")]
    pub members: Members,
    /// The attributes of a `multipart` request's file parts, by the `Bytes` input field each part
    /// carries, in name order; empty, and omitted, when the binding states none.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "written_parts"
    )]
    pub parts: BTreeMap<String, Part>,
    /// The input fields read from the query string beside the body (`input.<field>`), in the order
    /// written; empty, and omitted, when the binding lists none (beyond10x/ess#754).
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "written_query"
    )]
    pub query: Vec<Source>,
    /// What a request parameter no field names does.
    #[serde(
        default,
        skip_serializing_if = "Unknown::is_default",
        deserialize_with = "omitted_default"
    )]
    pub unknown: Unknown,
}

impl Request {
    /// The input field the body carries bare, by its local name, if it carries one.
    pub fn body_field(&self) -> Option<&str> {
        self.body.as_ref().map(Source::field)
    }

    /// The input fields read from the query string beside the body, by their local names, in the
    /// order written.
    pub fn query_fields(&self) -> impl Iterator<Item = &str> {
        self.query.iter().map(Source::field)
    }

    /// The member path a `json` body reads the input field `field` from, by its local name, when
    /// `request.members` maps it; `None` when it is a top-level member under its wire name, or
    /// travels elsewhere.
    pub fn member(&self, field: &str) -> Option<&MemberPath> {
        self.members.get(field)
    }
}

/// Reads a compiled request's `body`, which names a command's input field and never a view
/// parameter.
fn input_body<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Source>, D::Error> {
    match <Source as serde::Deserialize>::deserialize(deserializer)? {
        source @ Source::Input(_) => Ok(Some(source)),
        source @ Source::Param(_) => Err(serde::de::Error::custom(format!(
            "`request.body` is `{source}`; a request body is one input field of a command, \
             `input.<field>`"
        ))),
    }
}

/// A request key compiled IR writes only when it differs from its default.
trait RequestKey: Default + PartialEq + fmt::Display + for<'de> serde::Deserialize<'de> {
    const KEY: &'static str;
}

impl RequestKey for Encoding {
    const KEY: &'static str = "encoding";
}

impl RequestKey for Unknown {
    const KEY: &'static str = "unknown";
}

/// Reads a request key of compiled IR, refusing its default written out: compiled IR omits it, and
/// a reader that took it would write back bytes it was not given.
fn omitted_default<'de, D: serde::Deserializer<'de>, T: RequestKey>(
    deserializer: D,
) -> Result<T, D::Error> {
    let value = T::deserialize(deserializer)?;
    if value == T::default() {
        return Err(serde::de::Error::custom(format!(
            "`request.{}` is `{value}`, its default, which compiled IR omits",
            T::KEY
        )));
    }
    Ok(value)
}

/// The statuses a `3xx` outcome may answer: the redirects (decided for stage 2, answer 15).
/// `300`, `304`, `305` and `306` name no answer an outcome gives.
pub const REDIRECTS: [u16; 5] = [301, 302, 303, 307, 308];

/// A response body as written (`docs/design/http-presentation-binding.md`, "Response body per
/// outcome"): a word, `{<member>: error}` or `{<member>: {literal: <text>}}`. A word is read as
/// text, whatever it is, so that one no body form spells is refused with a code and a path
/// (`ESS-HTTP-024`, `ESS-HTTP-029`) rather than as a parse error; which forms a key admits is
/// [`compile`]'s to check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodySpec {
    /// `envelope`, `response`, `none`, `rows`, `single`, or a word that is none of them.
    Word(String),
    /// `{<member>: error}`: the error's wire name under `<member>`, beside the error's fields.
    Error(String),
    /// `{<member>: {literal: <text>}}`: one fixed member.
    Literal {
        /// The member's name.
        member: String,
        /// Its text.
        text: String,
    },
    /// `{<member>: <form>}`, any word but `error`: what the bare form answers, under `<member>`
    /// (beyond10x/ess#755). Which forms a key admits is [`compile`]'s to check.
    Wrapped {
        /// The member's name.
        member: String,
        /// The bare form it holds, as written.
        form: String,
    },
}

impl fmt::Display for BodySpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Word(word) => formatter.write_str(word),
            Self::Error(member) => write!(formatter, "{{{member}: error}}"),
            Self::Literal { member, text } => {
                write!(formatter, "{{{member}: {{literal: {text}}}}}")
            }
            Self::Wrapped { member, form } => write!(formatter, "{{{member}: {form}}}"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for BodySpec {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Form;

        impl<'de> serde::de::Visitor<'de> for Form {
            type Value = BodySpec;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(
                    "a body: a word such as `envelope`, `{<member>: error}`, or \
                     `{<member>: {literal: <text>}}`",
                )
            }

            fn visit_str<E: serde::de::Error>(self, word: &str) -> Result<Self::Value, E> {
                Ok(BodySpec::Word(word.to_owned()))
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let one = "a body object holds exactly one member";
                let Some((member, value)) = map.next_entry::<String, MemberValue>()? else {
                    return Err(serde::de::Error::custom(one));
                };
                if map.next_key::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom(one));
                }
                Ok(match value {
                    MemberValue::Error => BodySpec::Error(member),
                    MemberValue::Literal(text) => BodySpec::Literal { member, text },
                    MemberValue::Form(form) => BodySpec::Wrapped { member, form },
                })
            }
        }

        deserializer.deserialize_any(Form)
    }
}

impl serde::Serialize for BodySpec {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;
        match self {
            Self::Word(word) => serializer.serialize_str(word),
            Self::Error(member) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(member, "error")?;
                map.end()
            }
            Self::Literal { member, text } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    member,
                    &Literal {
                        literal: text.clone(),
                    },
                )?;
                map.end()
            }
            Self::Wrapped { member, form } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(member, form)?;
                map.end()
            }
        }
    }
}

/// The value of a body object's one member: `error`, `{literal: <text>}`, or another word, the
/// bare form it wraps.
enum MemberValue {
    Error,
    Literal(String),
    Form(String),
}

/// `{literal: <text>}`, closed.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Literal {
    literal: String,
}

impl<'de> serde::Deserialize<'de> for MemberValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Value;

        impl<'de> serde::de::Visitor<'de> for Value {
            type Value = MemberValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("`error`, a bare body form, or `{literal: <text>}`")
            }

            fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<Self::Value, E> {
                Ok(if text == "error" {
                    MemberValue::Error
                } else {
                    MemberValue::Form(text.to_owned())
                })
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let literal: Literal = serde::Deserialize::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                Ok(MemberValue::Literal(literal.literal))
            }
        }

        deserializer.deserialize_any(Value)
    }
}

/// What a command's outcome answers with (`docs/design/http-presentation-binding.md`, "Response
/// body per outcome"). `envelope` is the default and compiled IR omits it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Body {
    /// ESS's own body: `outcome`, `error`, `payload`, `published`, `response`.
    #[default]
    Envelope,
    /// The command's declared response object, bare.
    Response,
    /// The value of one top-level field of the command's declared response, bare, named by its
    /// local name and written `response.<field>`.
    ResponseField(String),
    /// No body.
    None,
    /// An object whose `member` is the error's wire name, beside the error's own fields under
    /// their wire names.
    Error {
        /// The member that carries the error's wire name.
        member: String,
    },
    /// `{<member>: response}` or `{<member>: response.<field>}`: an object whose one `member`
    /// holds what `response`, or `response.<field>`, answers bare (beyond10x/ess#755).
    Wrapped {
        /// The member that holds the value.
        member: String,
        /// The response field it holds, by local name; `None` for the whole response.
        field: Option<String>,
    },
}

impl Body {
    fn spec(&self) -> BodySpec {
        match self {
            Self::Envelope => BodySpec::Word("envelope".into()),
            Self::Response => BodySpec::Word("response".into()),
            Self::ResponseField(field) => {
                BodySpec::Word(AnswerSource::Response(field.clone()).to_string())
            }
            Self::None => BodySpec::Word("none".into()),
            Self::Error { member } => BodySpec::Error(member.clone()),
            Self::Wrapped { member, field } => BodySpec::Wrapped {
                member: member.clone(),
                form: field.as_ref().map_or_else(
                    || "response".to_owned(),
                    |field| AnswerSource::Response(field.clone()).to_string(),
                ),
            },
        }
    }

    /// Whether it answers the command's response, or a field of it, bare or wrapped.
    pub fn reads_response(&self) -> bool {
        matches!(
            self,
            Self::Response | Self::ResponseField(_) | Self::Wrapped { .. }
        )
    }

    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl fmt::Display for Body {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.spec().fmt(formatter)
    }
}

impl serde::Serialize for Body {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.spec().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Body {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = <BodySpec as serde::Deserialize>::deserialize(deserializer)?;
        match &written {
            BodySpec::Word(word) => match word.as_str() {
                "envelope" => Ok(Self::Envelope),
                "response" => Ok(Self::Response),
                "none" => Ok(Self::None),
                _ => match AnswerSource::parse(word) {
                    Some(AnswerSource::Response(field)) => Ok(Self::ResponseField(field)),
                    _ => Err(serde::de::Error::custom(format!(
                        "`{written}` is not an outcome's body: compiled IR writes `response`, \
                         `response.<field>`, `none` or `{{<member>: error}}`"
                    ))),
                },
            },
            BodySpec::Error(member) => Ok(Self::Error {
                member: member.clone(),
            }),
            BodySpec::Literal { .. } => Err(serde::de::Error::custom(format!(
                "`{written}` is a literal body, which only a standard refusal answers"
            ))),
            BodySpec::Wrapped { member, .. } if member.is_empty() => Err(serde::de::Error::custom(
                format!("`{written}` names an empty member; a wrapped body has one named member"),
            )),
            BodySpec::Wrapped { member, form } => {
                let field = match AnswerSource::parse(form) {
                    _ if form == "response" => None,
                    Some(AnswerSource::Response(field)) => Some(field),
                    _ => {
                        return Err(serde::de::Error::custom(format!(
                            "`{written}` is not an outcome's body: compiled IR wraps `response` \
                             or `response.<field>`"
                        )))
                    }
                };
                Ok(Self::Wrapped {
                    member: member.clone(),
                    field,
                })
            }
        }
    }
}

/// Reads an outcome's `body` of compiled IR, refusing `envelope` written out: compiled IR omits it.
fn omitted_envelope<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Body, D::Error> {
    let body = <Body as serde::Deserialize>::deserialize(deserializer)?;
    if body.is_default() {
        return Err(serde::de::Error::custom(
            "an outcome's `body` is `envelope`, its default, which compiled IR omits",
        ));
    }
    Ok(body)
}

/// What a view route answers with; `rows` is the default and compiled IR omits it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewBody {
    /// `{rows: [...]}`, ESS's own.
    #[default]
    Rows,
    /// The one row, bare, and the route's `empty` status when the view holds none.
    Single,
    /// The rows, bare: a JSON array with no object around it.
    Array,
}

impl ViewBody {
    /// The word that spells it.
    fn word(self) -> &'static str {
        match self {
            Self::Rows => "rows",
            Self::Single => "single",
            Self::Array => "array",
        }
    }
}

/// A view route's answer: its form, bare, or under one named member (`{<member>: single}`,
/// `{<member>: array}`, beyond10x/ess#755). `rows` is the default and compiled IR omits it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewAnswer {
    /// What the route answers: `rows`, `single` or `array`.
    pub form: ViewBody,
    /// The member that holds the `single` or `array` answer; `None` where it is bare. Never
    /// beside `rows`, which `{rows: array}` is.
    pub member: Option<String>,
}

impl ViewAnswer {
    /// The bare `form`.
    pub fn bare(form: ViewBody) -> Self {
        Self { form, member: None }
    }

    fn is_default(&self) -> bool {
        *self == Self::default()
    }

    fn spec(&self) -> BodySpec {
        match &self.member {
            None => BodySpec::Word(self.form.word().to_owned()),
            Some(member) => BodySpec::Wrapped {
                member: member.clone(),
                form: self.form.word().to_owned(),
            },
        }
    }
}

impl fmt::Display for ViewAnswer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.spec().fmt(formatter)
    }
}

impl serde::Serialize for ViewAnswer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.spec().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ViewAnswer {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = <BodySpec as serde::Deserialize>::deserialize(deserializer)?;
        let form = |word: &str| match word {
            "rows" => Some(ViewBody::Rows),
            "single" => Some(ViewBody::Single),
            "array" => Some(ViewBody::Array),
            _ => None,
        };
        let refused = || {
            serde::de::Error::custom(format!(
                "`{written}` is not a view route's body: compiled IR writes `single`, `array`, \
                 `{{<member>: single}}` or `{{<member>: array}}`"
            ))
        };
        match &written {
            BodySpec::Word(word) => form(word).map(Self::bare).ok_or_else(refused),
            BodySpec::Wrapped { member, .. } if member.is_empty() => Err(serde::de::Error::custom(
                format!("`{written}` names an empty member; a wrapped body has one named member"),
            )),
            BodySpec::Wrapped { member, form: word } => match form(word) {
                Some(ViewBody::Array) if member == "rows" => Err(serde::de::Error::custom(
                    format!("`{written}` is `rows`, ESS's own answer, which compiled IR omits"),
                )),
                Some(form @ (ViewBody::Single | ViewBody::Array)) => Ok(Self {
                    form,
                    member: Some(member.clone()),
                }),
                Some(ViewBody::Rows) | None => Err(refused()),
            },
            BodySpec::Error(_) | BodySpec::Literal { .. } => Err(refused()),
        }
    }
}

/// Reads a view route's `body` of compiled IR, refusing `rows` written out: compiled IR omits it.
fn omitted_rows<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<ViewAnswer, D::Error> {
    let body = <ViewAnswer as serde::Deserialize>::deserialize(deserializer)?;
    if body.is_default() {
        return Err(serde::de::Error::custom(
            "a view route's `body` is `rows`, its default, which compiled IR omits",
        ));
    }
    Ok(body)
}

/// One listed outcome, as written. `status` is read as any integer, so one outside `100..=599` is
/// refused with a code (`ESS-HTTP-023`) rather than as a parse error.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeSpec {
    /// The status the outcome answers; required.
    pub status: i64,
    /// The body; absent, `envelope`.
    #[serde(default)]
    pub body: Option<BodySpec>,
    /// How the body is written, `json` or `text`, as written. Read as text so that one outside the
    /// set is refused with a code and a path (`ESS-HTTP-024`) rather than as a parse error; absent,
    /// `json`.
    #[serde(default)]
    pub encoding: Option<String>,
    /// Response headers by name, as written; absent, none.
    #[serde(default)]
    pub headers: Option<Entries<HeaderSpec>>,
    /// Where a `3xx` outcome sends the caller; required on one, refused on any other.
    #[serde(default)]
    pub redirect: Option<RedirectSpec>,
}

/// How a declared outcome writes its body (`docs/design/http-presentation-binding.md`, "Response
/// body per outcome"); `json` is the default and compiled IR omits it.
///
/// Declared in the order the format lists them.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ResponseEncoding {
    /// `application/json`, ESS's own: every body form.
    #[default]
    Json,
    /// `text/plain; charset=utf-8`: the characters of one `String` response field, unquoted. Only
    /// a `response.<field>` body whose field is a `String` or a newtype of one is written so.
    Text,
}

impl ResponseEncoding {
    /// Every encoding, in the order the format lists them.
    pub const ALL: [Self; 2] = [Self::Json, Self::Text];

    /// The encoding as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Text => "text",
        }
    }

    /// The encoding a binding spells `text`, if it is one: exactly, in lower case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|encoding| encoding.as_str() == text)
    }

    // `skip_serializing_if` passes the field by reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl fmt::Display for ResponseEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Reads an outcome's `encoding` of compiled IR, refusing `json` written out: compiled IR omits it.
fn omitted_json<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ResponseEncoding, D::Error> {
    let encoding = <ResponseEncoding as serde::Deserialize>::deserialize(deserializer)?;
    if encoding.is_default() {
        return Err(serde::de::Error::custom(
            "an outcome's `encoding` is `json`, its default, which compiled IR omits",
        ));
    }
    Ok(encoding)
}

/// One listed outcome, compiled.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    /// The status it answers, always written.
    pub status: u16,
    /// The body; omitted when it is `envelope`.
    #[serde(
        default,
        skip_serializing_if = "Body::is_default",
        deserialize_with = "omitted_envelope"
    )]
    pub body: Body,
    /// How the body is written; omitted when it is `json`. `text` only beside a
    /// `response.<field>` body.
    #[serde(
        default,
        skip_serializing_if = "ResponseEncoding::is_default",
        deserialize_with = "omitted_json"
    )]
    pub encoding: ResponseEncoding,
    /// Response headers by name, in name order, each name as authored. Omitted when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_headers"
    )]
    pub headers: BTreeMap<String, Header>,
    /// Where the outcome sends the caller; present exactly on a `3xx` status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<Redirect>,
}

/// A `single` view route's answer for a view that holds no row, as written.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptySpec {
    /// A `4xx` status; required.
    pub status: i64,
    /// `none` or `{<member>: {literal: <text>}}`, read as any body form so that another is refused
    /// with a code (`ESS-HTTP-030`) rather than as a parse error; absent, `none`.
    #[serde(default)]
    pub body: Option<BodySpec>,
}

/// A `single` view route's answer for a view that holds no row, compiled: a `4xx` status and no
/// body, or one literal member.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Empty {
    /// The status.
    pub status: u16,
    /// The body; `none` is the default and compiled IR omits it.
    #[serde(
        default = "RefusalBody::none",
        skip_serializing_if = "RefusalBody::is_none",
        deserialize_with = "omitted_none"
    )]
    pub body: RefusalBody,
}

/// Reads an empty answer's `body` of compiled IR, refusing `none` written out: compiled IR omits
/// it.
fn omitted_none<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<RefusalBody, D::Error> {
    let body = <RefusalBody as serde::Deserialize>::deserialize(deserializer)?;
    if body.is_none() {
        return Err(serde::de::Error::custom(
            "an empty answer's `body` is `none`, its default, which compiled IR omits",
        ));
    }
    Ok(body)
}

/// A standard refusal the served surface answers without a declared outcome
/// (`docs/design/http-presentation-binding.md`, "Standard refusals").
///
/// Declared in name order, the order compiled IR writes them in.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum RefusalKey {
    /// A request whose input cannot be read; today `400`. Answered with a `4xx`.
    Malformed,
    /// A caller no grant admits; today `403`. Answered with a `4xx`.
    NotGranted,
    /// A request whose body is larger than the route reads; today `413`. Answered with a `4xx`.
    TooLarge,
    /// A call without the credential its route's `authentication` requires (beyond10x/ess#681);
    /// `401` with the schemes' challenge when not declared. Answered with a `4xx`.
    Unauthenticated,
    /// An unfinished realization; today `501`. Answered with a `5xx`.
    Unfinished,
}

impl RefusalKey {
    /// Every key, in name order.
    pub const ALL: [Self; 5] = [
        Self::Malformed,
        Self::NotGranted,
        Self::TooLarge,
        Self::Unauthenticated,
        Self::Unfinished,
    ];

    /// The key as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::NotGranted => "not_granted",
            Self::TooLarge => "too_large",
            Self::Unauthenticated => "unauthenticated",
            Self::Unfinished => "unfinished",
        }
    }

    /// The key a binding spells `text`, if it is one.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.as_str() == text)
    }

    /// The first digit of every status it may answer: `4` for a refusal of the request, `5` for
    /// the unfinished realization.
    fn class(self) -> u16 {
        match self {
            Self::Malformed | Self::NotGranted | Self::TooLarge | Self::Unauthenticated => 4,
            Self::Unfinished => 5,
        }
    }
}

impl fmt::Display for RefusalKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One standard refusal's override, as written.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefusalSpec {
    /// The status; required.
    pub status: i64,
    /// `{<member>: {literal: <text>}}` or `none`; required.
    pub body: BodySpec,
    /// Response headers by name, as written: literals and challenges whose params are literals,
    /// since a standard refusal has no outcome to read a source from; absent, none.
    #[serde(default)]
    pub headers: Option<Entries<HeaderSpec>>,
}

/// A standard refusal's body: one literal member, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalBody {
    /// No body.
    None,
    /// An object with one fixed member.
    Literal {
        /// The member's name.
        member: String,
        /// Its text.
        text: String,
    },
}

impl RefusalBody {
    /// No body, the default of an empty answer.
    pub fn none() -> Self {
        Self::None
    }

    /// Whether this is no body.
    pub fn is_none(&self) -> bool {
        *self == Self::None
    }

    fn spec(&self) -> BodySpec {
        match self {
            Self::None => BodySpec::Word("none".into()),
            Self::Literal { member, text } => BodySpec::Literal {
                member: member.clone(),
                text: text.clone(),
            },
        }
    }
}

impl serde::Serialize for RefusalBody {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.spec().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RefusalBody {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match <BodySpec as serde::Deserialize>::deserialize(deserializer)? {
            BodySpec::Word(word) if word == "none" => Ok(Self::None),
            BodySpec::Literal { member, text } => Ok(Self::Literal { member, text }),
            written => Err(serde::de::Error::custom(format!(
                "`{written}` is not a refusal's body: compiled IR writes `none` or \
                 `{{<member>: {{literal: <text>}}}}`"
            ))),
        }
    }
}

/// One standard refusal's override, compiled: its status and body, both always written, and its
/// headers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refusal {
    /// The status.
    pub status: u16,
    /// The body.
    pub body: RefusalBody,
    /// Response headers by name, in name order, each name as authored; never a source. Omitted
    /// when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_headers"
    )]
    pub headers: BTreeMap<String, Header>,
}

/// The one header a challenge is declared under (decided for stage 2, answer 18).
pub const CHALLENGE_HEADER: &str = "WWW-Authenticate";

/// The headers the server, the encoding and `redirect` write, which a binding does not declare
/// (`ESS-HTTP-026`), each with what writes it. Compared ignoring case.
const RESERVED_HEADERS: [(&str, &str); 6] = [
    ("Connection", "the server"),
    ("Content-Encoding", "the encoding"),
    ("Content-Length", "the encoding"),
    ("Content-Type", "the encoding"),
    ("Location", "`redirect`"),
    ("Transfer-Encoding", "the encoding"),
];

/// Whether `text` is an RFC 9110 token (§5.6.2): one or more letters, digits and
/// ``!#$%&'*+-.^_`|~``.
fn is_token(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
}

/// Why `text` is not a field value a literal header may hold (RFC 9110 §5.5): visible US-ASCII,
/// spaces and horizontal tabs, and no space or tab at either end.
fn field_value_reason(text: &str) -> Option<String> {
    let allowed = |c: char| c == ' ' || c == '\t' || c.is_ascii_graphic();
    if let Some(c) = text.chars().find(|c| !allowed(*c)) {
        return Some(format!(
            "{text:?} is not a field value: it holds {c:?}, and a field value holds visible \
             US-ASCII, spaces and horizontal tabs only, so a value cannot end its header line or \
             start another"
        ));
    }
    if text.starts_with([' ', '\t']) || text.ends_with([' ', '\t']) {
        return Some(format!(
            "{text:?} is not a field value: it starts or ends with a space or a tab, which a \
             reader strips"
        ));
    }
    None
}

/// Whether an RFC 9110 quoted string carries `text` (§5.6.4): horizontal tabs, spaces and visible
/// US-ASCII, `"` and `\` escaped by the generator.
fn quotable(text: &str) -> bool {
    text.chars()
        .all(|c| c == ' ' || c == '\t' || c.is_ascii_graphic())
}

/// Whether `name` is one or more RFC 3986 `unreserved` characters (§2.3): letters, digits and
/// `-._~`, which a redirect writes into a query or fragment as they are.
fn unreserved(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._~".contains(&byte))
}

/// Why `name` is no redirect parameter name ([`unreserved`]).
fn unreserved_reason(name: &str) -> String {
    format!(
        "{name:?} is not a redirect parameter name: a parameter name is one or more RFC 3986 \
         `unreserved` characters, letters, digits and `-._~`, which no encoding of the query or \
         fragment changes"
    )
}

/// The header `name` is, ignoring case, among `names`.
fn same_header<'a>(name: &str, names: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    names
        .into_iter()
        .find(|other| other.eq_ignore_ascii_case(name))
}

/// One response header's value, as written: exactly one of `{literal: <text>}`,
/// `{source: <path>}` and `{challenge: {scheme, params}}`. A source is read as text, so that one
/// that names nothing is refused with a code (`ESS-HTTP-026`) rather than as a parse error.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(try_from = "HeaderFields")]
pub enum HeaderSpec {
    /// A fixed text.
    Literal(String),
    /// A field the outcome supplies, as written.
    Source(String),
    /// One RFC 9110 §11.6.1 challenge.
    Challenge(ChallengeSpec),
}

/// A challenge param's value, as written: `{literal: <text>}` or `{source: <path>}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(try_from = "ParamFields")]
pub enum ParamSpec {
    /// A fixed text.
    Literal(String),
    /// A field the outcome supplies, as written.
    Source(String),
}

/// The keys of a header's value object, each optional so that exactly one can be required with a
/// sentence rather than an enum's parse error.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct HeaderFields {
    #[serde(default)]
    literal: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    challenge: Option<ChallengeSpec>,
}

/// The keys of a challenge param's value object, as [`HeaderFields`] without `challenge`.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamFields {
    #[serde(default)]
    literal: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

impl TryFrom<HeaderFields> for HeaderSpec {
    type Error = String;

    fn try_from(fields: HeaderFields) -> Result<Self, Self::Error> {
        match (fields.literal, fields.source, fields.challenge) {
            (Some(text), None, None) => Ok(Self::Literal(text)),
            (None, Some(text), None) => Ok(Self::Source(text)),
            (None, None, Some(challenge)) => Ok(Self::Challenge(challenge)),
            _ => Err(
                "a header's value is exactly one of `{literal: <text>}`, `{source: <path>}` and \
                 `{challenge: {scheme, params}}`"
                    .to_owned(),
            ),
        }
    }
}

impl TryFrom<ParamFields> for ParamSpec {
    type Error = String;

    fn try_from(fields: ParamFields) -> Result<Self, Self::Error> {
        match (fields.literal, fields.source) {
            (Some(text), None) => Ok(Self::Literal(text)),
            (None, Some(text)) => Ok(Self::Source(text)),
            _ => Err(
                "a challenge param's value is exactly one of `{literal: <text>}` and \
                 `{source: <path>}`"
                    .to_owned(),
            ),
        }
    }
}

impl HeaderSpec {
    /// The value a refinement compares (`ESS-HTTP-012`): structure, never text, so that one
    /// literal cannot spell another header.
    fn compared(&self) -> serde_json::Value {
        match self {
            Self::Literal(text) => serde_json::json!({"literal": text}),
            Self::Source(text) => serde_json::json!({"source": text}),
            Self::Challenge(challenge) => {
                let mut compared = serde_json::json!({"scheme": challenge.scheme});
                if let Some(params) =
                    compared_entries(challenge.params.as_ref(), ParamSpec::compared)
                {
                    compared["params"] = params;
                }
                serde_json::json!({"challenge": compared})
            }
        }
    }
}

impl ParamSpec {
    /// The value a refinement compares (`ESS-HTTP-012`).
    fn compared(&self) -> serde_json::Value {
        match self {
            Self::Literal(text) => serde_json::json!({"literal": text}),
            Self::Source(text) => serde_json::json!({"source": text}),
        }
    }
}

/// One challenge, as written.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeSpec {
    /// The auth scheme, a token; required.
    pub scheme: String,
    /// The auth params by name, as written; absent, none.
    #[serde(default)]
    pub params: Option<Entries<ParamSpec>>,
}

/// A redirect, as written. `mode` is read as text, so that one outside the set, or none, is
/// refused with a code (`ESS-HTTP-028`) rather than as a parse error.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedirectSpec {
    /// The source holding the absolute URI the caller is sent to; required.
    pub location: String,
    /// `query` or `fragment`; required, and refused with a code when absent.
    #[serde(default)]
    pub mode: Option<String>,
    /// The parameters written into the URI, by name, each a source; absent, none.
    #[serde(default)]
    pub parameters: Option<Entries<String>>,
}

impl RedirectSpec {
    /// The value a refinement compares (`ESS-HTTP-012`).
    fn compared(&self) -> serde_json::Value {
        let mut compared = serde_json::json!({"location": self.location});
        if let Some(mode) = &self.mode {
            compared["mode"] = serde_json::json!(mode);
        }
        if let Some(parameters) = compared_entries(self.parameters.as_ref(), |source: &String| {
            serde_json::json!(source)
        }) {
            compared["parameters"] = parameters;
        }
        compared
    }
}

/// Written entries as one object, each value by `value`, the first of a name written twice;
/// `None` when there are none. What a refinement compares (`ESS-HTTP-012`): an object compares
/// by name, whatever the authored order, and an empty map is none.
fn compared_entries<T>(
    entries: Option<&Entries<T>>,
    value: impl Fn(&T) -> serde_json::Value,
) -> Option<serde_json::Value> {
    let mut object = serde_json::Map::new();
    for (name, written) in entries.iter().flat_map(|entries| &entries.0) {
        if !object.contains_key(name) {
            object.insert(name.clone(), value(written));
        }
    }
    (!object.is_empty()).then_some(serde_json::Value::Object(object))
}

/// What a header, a challenge param or a redirect reads: a field of the command's input, of its
/// response, of the outcome's error, or the error's wire name. Written `input.<field>`,
/// `response.<field>`, `error.<field>` and `error.wire`, in the authored binding and in the IR
/// alike; `error.wire` always names the wire name (decided for stage 2, answer 19).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum AnswerSource {
    /// `input.<field>`.
    Input(String),
    /// `response.<field>`: the outcome declares `returns: true`.
    Response(String),
    /// `error.<field>`: the outcome reports an error.
    Error(String),
    /// `error.wire`: the wire name of the outcome's error.
    ErrorWire,
}

impl AnswerSource {
    /// The source `text` spells, if it is one: a namespace, one `.`, and one non-empty name.
    pub fn parse(text: &str) -> Option<Self> {
        let (namespace, field) = text.split_once('.')?;
        if field.is_empty() || field.contains('.') {
            return None;
        }
        match (namespace, field) {
            ("error", "wire") => Some(Self::ErrorWire),
            ("input", _) => Some(Self::Input(field.to_owned())),
            ("response", _) => Some(Self::Response(field.to_owned())),
            ("error", _) => Some(Self::Error(field.to_owned())),
            _ => None,
        }
    }
}

impl fmt::Display for AnswerSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(field) => write!(formatter, "input.{field}"),
            Self::Response(field) => write!(formatter, "response.{field}"),
            Self::Error(field) => write!(formatter, "error.{field}"),
            Self::ErrorWire => formatter.write_str("error.wire"),
        }
    }
}

impl From<AnswerSource> for String {
    fn from(source: AnswerSource) -> Self {
        source.to_string()
    }
}

impl TryFrom<String> for AnswerSource {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text).ok_or_else(|| {
            format!(
                "`{text}` is not a source an answer reads: `input.<field>`, `response.<field>`, \
                 `error.<field>` or `error.wire`"
            )
        })
    }
}

/// One response header's value, compiled.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Header {
    /// A fixed field value.
    Literal(String),
    /// A scalar the outcome supplies, written as text; an absent optional omits the header.
    Source(AnswerSource),
    /// One challenge, under `WWW-Authenticate`.
    Challenge(Challenge),
}

/// One RFC 9110 §11.6.1 challenge, compiled: the generator writes each param as a quoted string.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Challenge {
    /// The auth scheme, as authored.
    pub scheme: String,
    /// The auth params by name, in name order, each name as authored. Omitted when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_params"
    )]
    pub params: BTreeMap<String, Param>,
}

/// A challenge param's value, compiled.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Param {
    /// A fixed text a quoted string carries.
    Literal(String),
    /// A scalar the outcome supplies; an absent optional omits the param.
    Source(AnswerSource),
}

/// Where a redirect writes its parameters into the location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedirectMode {
    /// Appended to the location's query, keeping a query it already has.
    Query,
    /// Written as the location's fragment.
    Fragment,
}

impl RedirectMode {
    /// The mode `text` spells, if it is one.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "query" => Some(Self::Query),
            "fragment" => Some(Self::Fragment),
            _ => None,
        }
    }
}

/// A redirect, compiled.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Redirect {
    /// The `String` source holding the absolute URI the caller is sent to.
    pub location: AnswerSource,
    /// Where the parameters go.
    pub mode: RedirectMode,
    /// The parameters by name, in name order, each a scalar source. Omitted when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_redirect_parameters"
    )]
    pub parameters: BTreeMap<String, AnswerSource>,
}

/// A map as written: every entry in document order, a key written twice kept twice, so that
/// [`compile`] refuses the second with a code rather than one entry silently replacing the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entries<T>(pub Vec<(String, T)>);

impl<T> Default for Entries<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Entries<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Entries<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, T>()? {
                    entries.push(entry);
                }
                Ok(Entries(entries))
            }
        }

        deserializer.deserialize_map(Visitor(std::marker::PhantomData))
    }
}

/// Reads a map of compiled IR, refusing a key that does not sort strictly after the one before it,
/// and an empty map: compiled IR orders `what` by key, each once, and omits it when empty, and a
/// reader that sorted the keys or read `{}` silently would write back bytes it was not given.
fn canonical_map<'de, D, K, V>(
    deserializer: D,
    what: &'static str,
) -> Result<BTreeMap<K, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    K: Ord + fmt::Display + serde::Deserialize<'de>,
    V: serde::Deserialize<'de>,
{
    struct Visitor<K, V>(&'static str, std::marker::PhantomData<(K, V)>);

    impl<'de, K, V> serde::de::Visitor<'de> for Visitor<K, V>
    where
        K: Ord + fmt::Display + serde::Deserialize<'de>,
        V: serde::Deserialize<'de>,
    {
        type Value = BTreeMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "`{}`, ordered by key", self.0)
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries: BTreeMap<K, V> = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<K, V>()? {
                if let Some((last, _)) = entries.last_key_value() {
                    if *last >= key {
                        return Err(serde::de::Error::custom(format!(
                            "`{}` is out of canonical order: `{key}` follows `{last}`; compiled IR \
                             orders `{}` by key, each once",
                            self.0, self.0
                        )));
                    }
                }
                entries.insert(key, value);
            }
            if entries.is_empty() {
                return Err(serde::de::Error::custom(format!(
                    "`{}` is empty; compiled IR omits an empty `{}`",
                    self.0, self.0
                )));
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_map(Visitor(what, std::marker::PhantomData))
}

fn canonical_outcomes<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Outcome>, D::Error> {
    canonical_map(deserializer, "outcomes")
}

/// Reads a route's `alike` of compiled IR, refusing it written empty: compiled IR omits it. The
/// order of its groups and names is [`answer_invariants`]' to check.
fn canonical_alike<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Vec<String>>, D::Error> {
    let groups = <Vec<Vec<String>> as serde::Deserialize>::deserialize(deserializer)?;
    if groups.is_empty() {
        return Err(serde::de::Error::custom(
            "`alike` is empty; compiled IR omits a route's `alike` when it holds no group",
        ));
    }
    Ok(groups)
}

fn canonical_refusals<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<RefusalKey, Refusal>, D::Error> {
    canonical_map(deserializer, "refusals")
}

/// Reads a route's `refusals` of compiled IR, refusing them written empty: compiled IR omits a
/// route's `refusals` when it overrides none.
fn route_refusals<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<RefusalKey, Refusal>, D::Error> {
    let refusals = canonical_refusals(deserializer)?;
    if refusals.is_empty() {
        return Err(serde::de::Error::custom(
            "a route's `refusals` is empty; compiled IR omits it when the route overrides none",
        ));
    }
    Ok(refusals)
}

fn canonical_headers<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Header>, D::Error> {
    canonical_map(deserializer, "headers")
}

fn canonical_params<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Param>, D::Error> {
    canonical_map(deserializer, "params")
}

fn canonical_servers<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Vec<Server>>, D::Error> {
    canonical_map(deserializer, "servers")
}

fn canonical_redirect_parameters<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, AnswerSource>, D::Error> {
    canonical_map(deserializer, "parameters")
}

fn canonical_lists<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, ListForm>, D::Error> {
    canonical_map(deserializer, "lists")
}

/// Why `status` is no status any outcome may answer, whatever it reports: outside `100..=599`, a
/// `1xx`, or a `3xx` other than the [`REDIRECTS`].
fn status_reason(status: i64) -> Option<String> {
    if !(100..=599).contains(&status) {
        return Some(format!(
            "`{status}` is not an HTTP status, an integer from 100 to 599"
        ));
    }
    if status < 200 {
        return Some(format!(
            "`{status}` is a `1xx` status, an interim answer no outcome gives"
        ));
    }
    let redirect = u16::try_from(status).is_ok_and(|status| REDIRECTS.contains(&status));
    if (300..400).contains(&status) && !redirect {
        return Some(format!(
            "`{status}` is a `3xx` status that is not a redirect; an outcome answers `3xx` with \
             `301`, `302`, `303`, `307` or `308` only"
        ));
    }
    None
}

/// Why `status` is in a class an outcome that does (`error`) or does not report an error may not
/// answer: `2xx` says the request succeeded, `4xx` and `5xx` that it failed; `3xx` is admitted for
/// both.
fn class_reason(status: i64, error: bool) -> Option<String> {
    match (status / 100, error) {
        (2, true) => Some(format!(
            "`{status}` is a `2xx` status, which says the request succeeded, and the outcome \
             reports an error; an outcome with an error answers `3xx`, `4xx` or `5xx`"
        )),
        (class @ (4 | 5), false) => Some(format!(
            "`{status}` is a `{class}xx` status, which says the request failed, and the outcome \
             reports no error; an outcome without an error answers `2xx` or `3xx`"
        )),
        _ => None,
    }
}

/// One route, before resolution.
///
/// Every key is closed: a key the format does not have is refused by name as an unknown field,
/// never read and dropped.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteSpec {
    /// The qualified name of the command it serves; exactly one of this and `view`.
    #[serde(default)]
    pub command: Option<String>,
    /// The qualified name of the view it serves; exactly one of this and `command`.
    #[serde(default)]
    pub view: Option<String>,
    /// The `operationId` the route is published under; absent, the operation's qualified name.
    #[serde(default)]
    pub operation_id: Option<String>,
    /// The schemes a caller of this route authenticates with, any one sufficing, over the
    /// binding's `authentication.required`; `[]` makes the route open; absent, the binding's.
    #[serde(default)]
    pub authentication: Option<Vec<String>>,
    /// The method, as written. Read as text so that one outside the closed set is refused with a
    /// code and a path rather than as a parse error.
    pub method: String,
    /// The path template. Absent, the route is open: each server chooses its path.
    #[serde(default)]
    pub path: Option<String>,
    /// Path template variable to the source it reads: `input.<field>` or `param.<name>`.
    #[serde(default)]
    pub parameters: Parameters,
    /// How a command's request is encoded; absent, a `json` body that refuses an unknown key.
    #[serde(default)]
    pub request: Option<RequestSpec>,
    /// The literal values of members of a command's `json` body that choose this route among the
    /// routes of its component on its method and path: a map of member paths to literals, or a list
    /// of such maps, each an alternative. Read as any value, so that one outside its rules is
    /// refused with a code and a path (`ESS-HTTP-020`).
    #[serde(default)]
    pub select: Option<serde_json::Value>,
    /// The `List` input fields or view parameters the route writes as one comma-joined value, each
    /// name mapped to `comma`; absent, a list repeats its key once per element. Kept as written, so
    /// that a name written twice or a word other than `comma` is refused with a code and a path
    /// (`ESS-HTTP-021`).
    #[serde(default)]
    pub lists: Option<Entries<String>>,
    /// A command's answer per outcome, keyed by the outcome's local name; an outcome not listed
    /// keeps its derived status and the envelope.
    #[serde(default)]
    pub outcomes: Option<Entries<OutcomeSpec>>,
    /// A command's answer for every outcome that reports an error and is not listed under
    /// `outcomes`, as if listed under each; absent, each keeps its derived status and the envelope.
    #[serde(default)]
    pub errors: Option<OutcomeSpec>,
    /// Groups of a command's outcomes the route answers alike, each two or more local names of
    /// outcomes with one declared answer, so that the wire need not tell them apart; absent, none.
    #[serde(default)]
    pub alike: Option<Vec<Vec<String>>>,
    /// A view route's body, `rows`, `array` or `single`; absent, `rows`.
    #[serde(default)]
    pub body: Option<BodySpec>,
    /// A `single` view route's answer for a view that holds no row.
    #[serde(default)]
    pub empty: Option<EmptySpec>,
    /// The standard refusals this route answers its own way, by key, over the binding's
    /// top-level `refusals:`; absent, the binding's.
    #[serde(default)]
    pub refusals: Option<Entries<RefusalSpec>>,
    /// The largest request body this route reads, over the binding's top-level `limits:`; absent,
    /// the binding's. Only a command route that reads a body takes it.
    #[serde(default)]
    pub limits: Option<LimitsSpec>,
    /// Which origins may read this route's answers: `none`, or a policy in place of the binding's
    /// `cors`; absent, the binding's. Read as any value, so that one outside its rules is refused
    /// with a code and a path (`ESS-HTTP-038`).
    #[serde(default)]
    pub cors: Option<serde_json::Value>,
    /// A view route's other representations of its rows, each selected by one literal value of one
    /// query parameter (beyond10x/ess#753); absent, the route answers its `body` only.
    #[serde(default)]
    pub representations: Option<Vec<RepresentationSpec>>,
}

/// One `representations` entry of a view route, as written (beyond10x/ess#753).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepresentationSpec {
    /// The query parameter and the literal value that select it, as written; [`compile`] holds it
    /// to exactly one entry.
    pub query: Entries<String>,
    /// How the rows are written; required.
    pub encoding: RepresentationEncodingSpec,
    /// The headers the representation adds to the route's answer, literals only; absent, none.
    #[serde(default)]
    pub headers: Option<Entries<HeaderSpec>>,
}

/// A representation's `encoding`, as written: `{csv: {separator, header}}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepresentationEncodingSpec {
    /// Delimited text, one line per row.
    pub csv: CsvSpec,
}

/// A `csv` encoding, as written. `separator` is read as text, so that one outside the set is
/// refused with a code (`ESS-HTTP-035`) rather than as a parse error.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsvSpec {
    /// `","`, `";"` or a tab; absent, `","`.
    #[serde(default)]
    pub separator: Option<String>,
    /// Whether a header line of wire names comes first; absent, `true`.
    #[serde(default)]
    pub header: Option<bool>,
}

impl RepresentationSpec {
    /// The value a refinement compares (`ESS-HTTP-012`): a default written out or left out counts
    /// as that default, and headers compare by name.
    fn compared(&self) -> serde_json::Value {
        let mut compared = serde_json::json!({
            "query": compared_entries(Some(&self.query), |value: &String| serde_json::json!(value)),
            "separator": self.encoding.csv.separator.as_deref().unwrap_or(","),
            "header": self.encoding.csv.header.unwrap_or(true),
        });
        if let Some(headers) = compared_entries(self.headers.as_ref(), HeaderSpec::compared) {
            compared["headers"] = headers;
        }
        compared
    }
}

/// A `limits:` entry, as written: the largest request body a route reads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimitsSpec {
    /// The largest request body, in bytes, from 1 to [`MAX_REQUEST_BYTES`]; required.
    pub request_bytes: i64,
}

/// A `cors:` policy, as written: which origins may read a route's answers
/// (`docs/design/http-presentation-binding.md`, "Cross-origin reads").
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorsSpec {
    /// `any`, or the origins, each `scheme://host[:port]`; required. Read as any value, so that one
    /// outside its rules is refused with a code and a path (`ESS-HTTP-038`).
    pub origins: serde_json::Value,
    /// Whether a page may send credentials and read the answer; absent, `false`.
    #[serde(default)]
    pub credentials: Option<bool>,
    /// The request headers a page may send; absent, none.
    #[serde(default)]
    pub headers: Option<Vec<String>>,
    /// The response headers a page may read; absent, none.
    #[serde(default)]
    pub expose: Option<Vec<String>>,
    /// How long, in seconds, a preflight answer is held, from 0 to [`MAX_CORS_AGE`]; absent, the
    /// browser's own.
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// The longest a preflight answer may be held, in seconds: one day.
pub const MAX_CORS_AGE: u32 = 86_400;

/// A route's `parameters`, as written: every entry in document order, a key written twice kept
/// twice, so that [`compile`] refuses a variable mapped twice with a code (`ESS-HTTP-008`) rather
/// than one entry silently replacing the other, which is what a JSON object read into a map does.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Parameters(pub Vec<(String, String)>);

impl Parameters {
    /// The source text `variable` is mapped to, the first time it is.
    pub fn get(&self, variable: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(key, _)| key == variable)
            .map(|(_, source)| source.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Parameters {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Entries;

        impl<'de> serde::de::Visitor<'de> for Entries {
            type Value = Parameters;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map from path variable to source")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, String>()? {
                    entries.push(entry);
                }
                Ok(Parameters(entries))
            }
        }

        deserializer.deserialize_map(Entries)
    }
}

/// Where a path variable's value comes from: one input field of a command, or one parameter of a
/// view. Written `input.<field>` and `param.<name>`, in the authored binding and in the IR alike.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Source {
    /// `input.<field>`: an input field of the command the route serves.
    Input(String),
    /// `param.<name>`: a parameter of the view the route serves.
    Param(String),
}

impl Source {
    /// The field or parameter it names.
    pub fn field(&self) -> &str {
        match self {
            Self::Input(field) | Self::Param(field) => field,
        }
    }

    /// `input` or `param`.
    pub fn namespace(&self) -> &'static str {
        match self {
            Self::Input(_) => "input",
            Self::Param(_) => "param",
        }
    }

    /// The source `text` spells, if it is one: a namespace, one `.`, and one non-empty name.
    pub fn parse(text: &str) -> Option<Self> {
        let (namespace, field) = text.split_once('.')?;
        if field.is_empty() || field.contains('.') {
            return None;
        }
        match namespace {
            "input" => Some(Self::Input(field.to_owned())),
            "param" => Some(Self::Param(field.to_owned())),
            _ => None,
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.namespace(), self.field())
    }
}

impl From<Source> for String {
    fn from(source: Source) -> Self {
        source.to_string()
    }
}

impl TryFrom<String> for Source {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::parse(&text)
            .ok_or_else(|| format!("`{text}` is not a source: `input.<field>` or `param.<name>`"))
    }
}

/// One segment of a path template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Text the request path holds exactly.
    Literal(String),
    /// `{name}`: a whole segment whose text is the value of the source `name` is mapped to.
    Variable(String),
}

impl fmt::Display for Segment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(text) => formatter.write_str(text),
            Self::Variable(name) => write!(formatter, "{{{name}}}"),
        }
    }
}

/// The segments of a path template, or every reason it breaks the grammar
/// (`docs/design/http-presentation-binding.md`, "Path templates"):
///
/// - it starts with `/`, and `/` alone is the root, with no segment;
/// - segments are `/`-separated and none is empty, so there is no trailing `/` but the root's;
/// - no `?`, `#` or `%`, and no segment that is `.` or `..`, which a client removes before it asks
///   (RFC 3986 §5.2.4) and so asks for another route;
/// - a literal segment is RFC 3986 `pchar` text without `%`;
/// - a variable is a whole segment `{name}`, its name an identifier: an ASCII letter or `_`, then
///   ASCII letters, digits and `_`. `/items/{id}.json` is refused: a partial-segment template needs
///   an escaping rule this format does not state.
///
/// Each reason names the segment it is about.
pub fn template(path: &str) -> Result<Vec<Segment>, Vec<String>> {
    let Some(rest) = path.strip_prefix('/') else {
        return Err(vec![format!(
            "`{path}` does not start with `/`; a path template is absolute"
        )]);
    };
    if rest.is_empty() {
        return Ok(Vec::new());
    }
    let mut segments = Vec::new();
    let mut reasons = Vec::new();
    for (index, text) in rest.split('/').enumerate() {
        match segment(text) {
            Ok(segment) => segments.push(segment),
            Err(reason) => reasons.push(format!("segment {} of `{path}`{}", index + 1, reason)),
        }
    }
    if reasons.is_empty() {
        Ok(segments)
    } else {
        Err(reasons)
    }
}

fn segment(text: &str) -> Result<Segment, String> {
    if text.is_empty() {
        return Err(
            " is empty; segments are separated by one `/` and a path ends without one".into(),
        );
    }
    if text == "." || text == ".." {
        return Err(format!(
            ", `{text}`, is a dot-segment, which a client removes before it asks (RFC 3986 §5.2.4)"
        ));
    }
    if let Some(found) = text.chars().find(|c| matches!(c, '?' | '#')) {
        return Err(format!(
            ", `{text}`, holds `{found}`; a path template holds no query and no fragment"
        ));
    }
    if text.contains('%') {
        return Err(format!(
            ", `{text}`, holds `%`; a path template is written without percent-encoding"
        ));
    }
    if text.contains(['{', '}']) {
        return match text
            .strip_prefix('{')
            .and_then(|inner| inner.strip_suffix('}'))
        {
            Some(name) if is_identifier(name) => Ok(Segment::Variable(name.to_owned())),
            _ => Err(format!(
                ", `{text}`, is not one whole variable `{{name}}`: a variable is a whole segment and \
                 its name an identifier (an ASCII letter or `_`, then letters, digits and `_`)"
            )),
        };
    }
    if let Some(found) = text.chars().find(|&c| !is_pchar(c)) {
        return Err(format!(
            ", `{text}`, holds `{found}`, which is not RFC 3986 path text: a literal segment is \
             letters, digits and `-._~!$&'()*+,;=:@`"
        ));
    }
    Ok(Segment::Literal(text.to_owned()))
}

fn is_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// RFC 3986 `pchar` without `pct-encoded`: `unreserved`, `sub-delims`, `:` and `@`.
fn is_pchar(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '-' | '.'
                | '_'
                | '~'
                | '!'
                | '$'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | ';'
                | '='
                | ':'
                | '@'
        )
}

/// How a caller of the binding's routes proves who it is, as written (beyond10x/ess#681,
/// `docs/design/http-presentation-binding.md`, "Authentication").
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticationSpec {
    /// The schemes by name, each an RFC 9110 token; required.
    pub schemes: Entries<SchemeSpec>,
    /// The schemes a route requires when it states none of its own, any one sufficing; absent,
    /// none: such a route is open.
    #[serde(default)]
    pub required: Option<Vec<String>>,
}

/// One scheme, as written. `kind` and `in` are read as text, so that one outside the set is
/// refused with a code (`ESS-HTTP-037`) rather than as a parse error.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeSpec {
    /// `bearer`, `basic`, `api_key` or `mutual_tls`; required.
    pub kind: String,
    /// Where an `api_key` travels: `header`, `query` or `cookie`.
    #[serde(default, rename = "in")]
    pub location: Option<String>,
    /// The header, query parameter or cookie an `api_key` travels in.
    #[serde(default)]
    pub name: Option<String>,
    /// The realm of a `basic` scheme's challenge.
    #[serde(default)]
    pub realm: Option<String>,
}

impl SchemeSpec {
    /// The value a refinement compares (`ESS-HTTP-012`).
    fn compared(&self) -> serde_json::Value {
        serde_json::json!([self.kind, self.location, self.name, self.realm])
    }
}

/// How a caller proves who it is, compiled: the schemes by name, in name order, and the ones a
/// route that states none requires.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authentication {
    /// The schemes by name, in name order.
    #[serde(deserialize_with = "canonical_schemes")]
    pub schemes: BTreeMap<String, Scheme>,
    /// The schemes a route that states none requires, in the order written, any one sufficing.
    /// Absent when empty: such a route is open.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

/// One compiled scheme.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scheme {
    /// RFC 6750: a bearer token in `Authorization`; its challenge is `Bearer`.
    Bearer,
    /// RFC 7617: a user and password in `Authorization`; its challenge is `Basic` with the realm.
    Basic {
        /// The realm the challenge names.
        realm: String,
    },
    /// A key in a header, a query parameter or a cookie; it has no challenge.
    ApiKey {
        /// Where the key travels.
        #[serde(rename = "in")]
        location: KeyLocation,
        /// The header, query parameter or cookie it travels in.
        name: String,
    },
    /// A client certificate presented in the TLS handshake; it has no challenge.
    MutualTls,
}

impl Scheme {
    /// Whether a `401` for a call without this scheme's credential can carry its challenge.
    pub fn challenges(&self) -> bool {
        matches!(self, Self::Bearer | Self::Basic { .. })
    }
}

/// Where an `api_key` travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyLocation {
    /// A request header.
    Header,
    /// A query parameter.
    Query,
    /// A cookie.
    Cookie,
}

fn canonical_schemes<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Scheme>, D::Error> {
    canonical_map(deserializer, "schemes")
}

/// An adopter-authored `ess-http/1` document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpSpec {
    /// The format, as written; [`compile`] refuses anything but [`HTTP_FORMAT`].
    #[serde(rename = "type")]
    pub format: String,
    /// The exact ESS the routes belong to.
    pub specification: SpecificationIdentity,
    /// `sha256:<hex>` of the bytes of the binding this one refines; absent, it refines nothing.
    #[serde(default)]
    pub refines: Option<String>,
    /// The standard refusals this binding answers its own way, keyed `malformed`, `not_granted`,
    /// `too_large` or `unfinished`; one not listed keeps today's answer.
    #[serde(default)]
    pub refusals: Option<Entries<RefusalSpec>>,
    /// The largest request body a route that reads one takes; absent, [`FIXED_REQUEST_BYTES`].
    #[serde(default)]
    pub limits: Option<LimitsSpec>,
    /// The base URLs a `reached_by: network` component's paths resolve against, keyed by the
    /// component, each list in the order written: the first is the default.
    #[serde(default)]
    pub servers: Option<Entries<Vec<ServerSpec>>>,
    /// How a caller proves who it is; absent, every route is open.
    #[serde(default)]
    pub authentication: Option<AuthenticationSpec>,
    /// Which origins may read the answers of every route that states no policy of its own;
    /// absent, no cross-origin read.
    #[serde(default)]
    pub cors: Option<CorsSpec>,
    /// One route per bound command or view, in document order.
    pub routes: Vec<RouteSpec>,
}

/// One base URL of a component, before resolution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerSpec {
    /// An absolute `http://` or `https://` URL, or a `/`-rooted path.
    pub url: String,
    /// What the server is, for a reader.
    #[serde(default)]
    pub description: Option<String>,
}

/// One compiled base URL of a component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// An absolute `http://` or `https://` URL, or a `/`-rooted path, with no whitespace, no `{`
    /// or `}`, no query, no fragment and no userinfo.
    pub url: String,
    /// What the server is, for a reader. Absent when not declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Why `url` is no server URL: neither absolute `http(s)://` with a host (the authority before any
/// `:port`, an IPv6 literal bracketed) nor `/`-rooted, or holding whitespace, `{`, `}`, `?`, `#` or
/// userinfo.
fn server_url_reason(url: &str) -> Option<String> {
    if let Some(found) = url
        .chars()
        .find(|c| c.is_whitespace() || c.is_control() || matches!(c, '{' | '}' | '?' | '#'))
    {
        let what = match found {
            '{' | '}' => "a server variable, which a binding does not offer",
            '?' => "a query",
            '#' => "a fragment",
            _ => "whitespace or a control character",
        };
        return Some(format!(
            "`{url}` holds {what}; a server URL is a base its paths are appended to"
        ));
    }
    let authority = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    match authority {
        Some(rest) => {
            let authority = rest.split('/').next().unwrap_or_default();
            if authority.contains('@') {
                return Some(format!(
                    "`{url}` holds userinfo; a server URL carries no credentials"
                ));
            }
            // The host is the authority before any `:port`; an IPv6 literal is bracketed and
            // holds colons of its own.
            let host = match authority.strip_prefix('[') {
                Some(literal) => match literal.split_once(']') {
                    Some((inside, after)) if after.is_empty() || after.starts_with(':') => inside,
                    _ => {
                        return Some(format!(
                            "`{url}` opens an IP literal with `[` and does not close it before \
                             the port or the path"
                        ))
                    }
                },
                None => authority.split(':').next().unwrap_or_default(),
            };
            host.is_empty().then(|| format!("`{url}` names no host"))
        }
        None if url.starts_with('/') && !url.starts_with("//") => None,
        None => Some(format!(
            "`{url}` is neither an absolute `http://` or `https://` URL nor a `/`-rooted path"
        )),
    }
}

/// Whether `id` is an operation id: `[A-Za-z_][A-Za-z0-9_.-]*`.
fn is_operation_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// The refusal of an operation id outside the grammar.
fn operation_id_reason(id: &str) -> String {
    format!(
        "`{id}` is not an operation id: one starts with an ASCII letter or `_` and continues with \
         ASCII letters, digits, `_`, `.` and `-`"
    )
}

/// `sha256:<hex>` of `bytes`: what `refines:` pins a refined binding by.
pub fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;

    let mut out = String::with_capacity(71);
    out.push_str("sha256:");
    for byte in Sha256::digest(bytes) {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Whether `text` is a digest as [`digest`] writes one: `sha256:` and 64 lower-case hex digits.
fn is_digest(text: &str) -> bool {
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    })
}

/// The path of the first key or element where `given` and `written` differ, under `at`; `None` when
/// they are equal. Object keys are compared as sets, in key order, so it reports the first by name.
fn first_difference(
    given: &serde_json::Value,
    written: &serde_json::Value,
    at: &str,
) -> Option<String> {
    use serde_json::Value;
    let under = |key: &str| {
        if at.is_empty() {
            key.to_owned()
        } else {
            format!("{at}.{key}")
        }
    };
    match (given, written) {
        (Value::Object(given), Value::Object(written)) => {
            let keys: std::collections::BTreeSet<&String> =
                given.keys().chain(written.keys()).collect();
            keys.into_iter()
                .find_map(|key| match (given.get(key), written.get(key)) {
                    (Some(one), Some(other)) => first_difference(one, other, &under(key)),
                    _ => Some(under(key)),
                })
        }
        (Value::Array(given), Value::Array(written)) => {
            let found = given
                .iter()
                .zip(written)
                .enumerate()
                .find_map(|(index, (one, other))| {
                    first_difference(one, other, &format!("{at}[{index}]"))
                });
            found.or_else(|| (given.len() != written.len()).then(|| at.to_owned()))
        }
        _ => (given != written).then(|| at.to_owned()),
    }
}

/// A binding another one refines: the document as read, and the digest of the exact bytes it was
/// read from, which is what the refining document's `refines:` pins. Reformatting the file changes
/// the digest, so it is a deliberate re-pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refined {
    digest: String,
    spec: HttpSpec,
}

impl Refined {
    /// Reads a refined binding's file bytes as strict YAML.
    pub fn from_yaml(bytes: &[u8]) -> Result<Self, ess_yaml::Error> {
        Ok(Self {
            digest: digest(bytes),
            spec: ess_yaml::from_slice(bytes)?,
        })
    }

    /// Reads a refined binding's file bytes as strict JSON.
    pub fn from_json(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        Ok(Self {
            digest: digest(bytes),
            spec: serde_json::from_slice(bytes)?,
        })
    }

    /// `sha256:<hex>` of the bytes it was read from.
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// The document.
    pub fn spec(&self) -> &HttpSpec {
        &self.spec
    }
}

impl HttpSpec {
    /// Reads strict YAML. Semantic validation happens in [`compile`].
    pub fn from_yaml(text: &str) -> Result<Self, ess_yaml::Error> {
        ess_yaml::from_str(text)
    }

    /// Reads strict JSON. Semantic validation happens in [`compile`].
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// What one route serves, by qualified name.
///
/// Serialised as the authored key, `command: <name>` or `view: <name>`, beside the route's other
/// keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// A command a component accepts.
    Command(String),
    /// A view a `reached_by: network` component serves.
    View(String),
}

impl Operation {
    /// The qualified name.
    pub fn name(&self) -> &str {
        match self {
            Self::Command(name) | Self::View(name) => name,
        }
    }

    /// `command` or `view`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Command(_) => "command",
            Self::View(_) => "view",
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} `{}`", self.kind(), self.name())
    }
}

/// How a route writes one `List` in the query string or a `form` body other than by repeating its
/// key once per element.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
// One byte, so that a map to the form is no map to a zero-sized value while `comma` is the only one.
#[repr(u8)]
pub enum ListForm {
    /// One key, its value the elements' texts joined by `,`: the empty value is the empty list, an
    /// absent key an absent `Optional` list, and a second occurrence of the key is malformed. An
    /// element holding `,` cannot be written.
    Comma,
}

impl ListForm {
    /// Every form, in the order the format lists them.
    pub const ALL: [Self; 1] = [Self::Comma];

    /// The form as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Comma => "comma",
        }
    }

    /// The form a binding spells `text`, if it is one: exactly, in lower case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|form| form.as_str() == text)
    }
}

impl fmt::Display for ListForm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A resolved route.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    /// What it serves.
    #[serde(flatten)]
    pub operation: Operation,
    /// The `operationId` the binding publishes the route under. Absent when it declares none; then
    /// the operation's qualified name is its `operationId`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The schemes the route states for itself, in the order written, over the binding's
    /// [`Authentication::required`]; `[]` when it is open. Absent when it states none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Vec<String>>,
    /// The method.
    pub method: Method,
    /// The path template; absent for an open route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Path template variable to the typed source it reads, ordered by variable. Absent when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_parameters"
    )]
    pub parameters: BTreeMap<String, Source>,
    /// How a command's request is encoded. Absent when it is a `json` body that refuses an unknown
    /// key, ESS's own, and always absent on a view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    /// The literal values of members of the `json` body that choose this route among the routes of
    /// its component sharing its method and path; absent, the route takes every body the others
    /// do not select (the catch-all), or it shares its template with none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<Selection>,
    /// The `List` input fields or view parameters written as one comma-joined value, by name in
    /// name order. Absent when empty: every list then repeats its key once per element.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_lists"
    )]
    pub lists: BTreeMap<String, ListForm>,
    /// A command's answer for each outcome the binding lists, keyed by the outcome's local name in
    /// name order. Absent when empty, and always absent on a view; an outcome not listed keeps its
    /// derived status and the envelope.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_outcomes"
    )]
    pub outcomes: BTreeMap<String, Outcome>,
    /// A command's answer for every outcome that reports an error and is not in `outcomes`.
    /// Absent when the binding declares none, and always absent on a view; then each such
    /// outcome keeps its derived status and the envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Outcome>,
    /// The groups of a command's outcomes the route answers alike: each two or more local names in
    /// name order, every one with one declared answer, the groups ordered by their first name.
    /// Absent when empty, and always absent on a view.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "canonical_alike"
    )]
    pub alike: Vec<Vec<String>>,
    /// A view route's body. Absent when it is `rows`, and always absent on a command.
    #[serde(
        default,
        skip_serializing_if = "ViewAnswer::is_default",
        deserialize_with = "omitted_rows"
    )]
    pub body: ViewAnswer,
    /// A `single` view route's answer for a view that holds no row; present exactly with `single`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty: Option<Empty>,
    /// The standard refusals this route answers its own way, in key order, over the binding's
    /// [`HttpIr::refusals`]. Absent when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "route_refusals"
    )]
    pub refusals: BTreeMap<RefusalKey, Refusal>,
    /// The largest request body this route reads, in place of the binding's [`HttpIr::limits`].
    /// Absent when the binding's stands, and always absent on a route that reads no body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    /// Which origins may read this route's answers, in place of the binding's [`HttpIr::cors`]:
    /// `none` or its own policy. Absent when the binding's stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cors: Option<RouteCors>,
    /// A `rows` or `array` view route's other representations of its rows, in the order written,
    /// each selected by one value of one query parameter (beyond10x/ess#753). Absent when empty,
    /// and always absent on a command; without the parameter the route answers its `body`.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "canonical_representations"
    )]
    pub representations: Vec<Representation>,
}

/// A compiled `limits`: the largest request body a route reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// The largest request body, in bytes, from 1 to [`MAX_REQUEST_BYTES`].
    pub request_bytes: u64,
}

/// The origins a compiled `cors` policy admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origins {
    /// Every origin, written `any`; never beside credentials.
    Any,
    /// These origins, each `scheme://host[:port]`, in the order written, none twice.
    Listed(Vec<String>),
}

impl Origins {
    /// Whether every origin `other` admits, this admits too.
    pub fn covers(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) => true,
            (Self::Listed(_), Self::Any) => false,
            (Self::Listed(mine), Self::Listed(theirs)) => {
                theirs.iter().all(|origin| mine.contains(origin))
            }
        }
    }
}

impl serde::Serialize for Origins {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Any => serializer.serialize_str("any"),
            Self::Listed(origins) => origins.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Origins {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(word) if word == "any" => Ok(Self::Any),
            value => serde_json::from_value::<Vec<String>>(value)
                .map(Self::Listed)
                .map_err(|_| {
                    serde::de::Error::custom("`origins` is `any` or a list of origin strings")
                }),
        }
    }
}

/// A compiled `cors` policy: which origins may read a route's answers, and what the preflight
/// answers (`docs/design/http-presentation-binding.md`, "Cross-origin reads").
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cors {
    /// The origins that may read the answers.
    pub origins: Origins,
    /// Whether a page may send credentials and read the answer. Absent when `false`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub credentials: bool,
    /// The request headers a page may send, in the order written. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<String>,
    /// The response headers a page may read, in the order written, each one a covered route
    /// declares. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expose: Vec<String>,
    /// How long, in seconds, a preflight answer is held. Absent when not declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
}

/// Whether `value` is `false`, the default a compiled policy omits.
#[allow(clippy::trivially_copy_pass_by_ref)] // serde's `skip_serializing_if` passes a reference
fn is_false(value: &bool) -> bool {
    !*value
}

/// A route's own compiled `cors`, in place of the binding's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteCors {
    /// `none`: no origin but the route's own may read its answers.
    None,
    /// This policy.
    Own(Cors),
}

impl serde::Serialize for RouteCors {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::None => serializer.serialize_str("none"),
            Self::Own(cors) => cors.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RouteCors {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(word) if word == "none" => Ok(Self::None),
            value @ serde_json::Value::Object(_) => serde_json::from_value(value)
                .map(Self::Own)
                .map_err(serde::de::Error::custom),
            _ => Err(serde::de::Error::custom(
                "a route's `cors` is `none` or a policy",
            )),
        }
    }
}

/// Why `origin` is not an origin a policy lists: `scheme://host` or `scheme://host:port`, with no
/// path, query, fragment, userinfo or `*`.
fn origin_reason(origin: &str) -> Option<String> {
    origin_fault(origin).map(|why| {
        format!(
            "`{origin}` is not an origin: {why}; an origin is `scheme://host` or \
             `scheme://host:port`, as a browser sends it in `Origin`"
        )
    })
}

/// What keeps `origin` from being `scheme://host[:port]`.
fn origin_fault(origin: &str) -> Option<String> {
    let Some((scheme, rest)) = origin.split_once("://") else {
        return Some("it has no `scheme://`".to_owned());
    };
    let scheme_char = |c: char| c.is_ascii_alphanumeric() || "+-.".contains(c);
    if !scheme.starts_with(|c: char| c.is_ascii_alphabetic()) || !scheme.chars().all(scheme_char) {
        return Some(format!("`{scheme}` is no scheme"));
    }
    for (c, what) in [
        ('/', "a path"),
        ('?', "a query"),
        ('#', "a fragment"),
        ('@', "userinfo"),
        ('*', "a `*`"),
    ] {
        if rest.contains(c) {
            return Some(format!("it holds {what}"));
        }
    }
    if rest.chars().any(|c| !c.is_ascii_graphic()) {
        return Some("it holds a character outside visible US-ASCII".to_owned());
    }
    let (host, port) = match rest.strip_prefix('[') {
        Some(bracketed) => {
            let Some((inner, after)) = bracketed.split_once(']') else {
                return Some("its IPv6 host has no `]`".to_owned());
            };
            if after.is_empty() {
                (inner, None)
            } else if let Some(port) = after.strip_prefix(':') {
                (inner, Some(port))
            } else {
                return Some("text follows its IPv6 host".to_owned());
            }
        }
        None => match rest.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (rest, None),
        },
    };
    if host.is_empty() {
        return Some("it has no host".to_owned());
    }
    if !rest.starts_with('[') && host.contains(':') {
        return Some("its host holds `:`".to_owned());
    }
    if let Some(port) = port {
        let number = (port.len() <= 5 && port.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| port.parse::<u32>().ok())
            .flatten();
        if number.is_none_or(|number| number > 65_535) {
            return Some(format!("`{port}` is no port"));
        }
    }
    None
}

/// What breaks the rules of `cors` that need no route: each with the key under the policy it is at
/// (`credentials`, `origins`, `origins[j]`, `headers[j]`, `expose[j]`, `max_age`) and why.
fn cors_problems(cors: &Cors) -> Vec<(String, String)> {
    let mut problems = Vec::new();
    match &cors.origins {
        Origins::Any if cors.credentials => problems.push((
            "credentials".to_owned(),
            "`credentials: true` beside `origins: any` is refused: the Fetch standard forbids the \
             wildcard origin on an answer a page reads with credentials; list the origins"
                .to_owned(),
        )),
        Origins::Any => {}
        Origins::Listed(origins) => {
            if origins.is_empty() {
                problems.push((
                    "origins".to_owned(),
                    "`origins` is empty: list one origin or more, or leave `cors` out, which \
                     admits no cross-origin read"
                        .to_owned(),
                ));
            }
            for (index, origin) in origins.iter().enumerate() {
                if let Some(reason) = origin_reason(origin) {
                    problems.push((format!("origins[{index}]"), reason));
                } else if origins[..index]
                    .iter()
                    .any(|earlier| earlier.eq_ignore_ascii_case(origin))
                {
                    problems.push((
                        format!("origins[{index}]"),
                        format!("the origin `{origin}` is given twice"),
                    ));
                }
            }
        }
    }
    for (key, names) in [("headers", &cors.headers), ("expose", &cors.expose)] {
        for (index, name) in names.iter().enumerate() {
            if !is_token(name) {
                problems.push((
                    format!("{key}[{index}]"),
                    format!("{name:?} is not a header name: a header name is an RFC 9110 token"),
                ));
            } else if same_header(name, names[..index].iter().map(String::as_str)).is_some() {
                problems.push((
                    format!("{key}[{index}]"),
                    format!("the header `{name}` is given twice, ignoring case"),
                ));
            }
        }
    }
    if let Some(max_age) = cors.max_age.filter(|age| *age > MAX_CORS_AGE) {
        problems.push(("max_age".to_owned(), max_age_reason(i64::from(max_age))));
    }
    problems
}

/// Why `max_age` is refused.
fn max_age_reason(max_age: i64) -> String {
    format!(
        "`max_age` {max_age} is outside 0 to {MAX_CORS_AGE} seconds, the longest a preflight \
         answer is held"
    )
}

/// Every header name `route` declares: on its outcomes, its `errors`, its own refusals and its
/// representations.
fn route_headers(route: &Route) -> impl Iterator<Item = &str> {
    route
        .outcomes
        .values()
        .chain(route.errors.iter())
        .flat_map(|outcome| outcome.headers.keys())
        .chain(
            route
                .refusals
                .values()
                .flat_map(|refusal| refusal.headers.keys()),
        )
        .chain(
            route
                .representations
                .iter()
                .flat_map(|representation| representation.headers.keys()),
        )
        .map(String::as_str)
}

/// Each name an `expose` lists that no route its policy covers declares, with its key and why: the
/// binding's `cors` (at `cors`) covers every route that states no policy of its own, a route's own
/// (at `routes[i].cors`, `at` naming the route) covers that route. A standard refusal the binding
/// overrides answers on every route that declares one.
fn unexposed<'a>(
    cors: Option<&Cors>,
    standard: &BTreeMap<RefusalKey, Refusal>,
    routes: impl Iterator<Item = (String, &'a Route)>,
) -> Vec<(String, String)> {
    let binding: Vec<&str> = standard
        .values()
        .flat_map(|refusal| refusal.headers.keys().map(String::as_str))
        .collect();
    let mut problems = Vec::new();
    let check =
        |policy: &Cors, declared: &[&str], at: &str, problems: &mut Vec<(String, String)>| {
            for (index, name) in policy.expose.iter().enumerate() {
                if same_header(name, declared.iter().copied()).is_none() {
                    problems.push((
                        format!("{at}.expose[{index}]"),
                        format!(
                            "`{name}` is exposed, and no route this policy covers declares it; \
                         `expose` names a header a covered route answers with"
                        ),
                    ));
                }
            }
        };
    let mut covered: Vec<&str> = Vec::new();
    for (at, route) in routes {
        match &route.cors {
            None => {
                if covered.is_empty() {
                    covered.extend(binding.iter().copied());
                }
                covered.extend(route_headers(route));
            }
            Some(RouteCors::None) => {}
            Some(RouteCors::Own(own)) => {
                let declared: Vec<&str> = route_headers(route)
                    .chain(binding.iter().copied())
                    .collect();
                check(own, &declared, &format!("{at}.cors"), &mut problems);
            }
        }
    }
    if let Some(cors) = cors {
        check(cors, &covered, "cors", &mut problems);
    }
    problems
}

/// A compiled representation of a view route's rows: what answers when the query holds its
/// selector (beyond10x/ess#753).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Representation {
    /// The query parameter and the value that select it.
    pub query: Selector,
    /// How the rows are written.
    pub encoding: RepresentationEncoding,
    /// The literal headers it adds to the route's answer, in name order, each name as authored.
    /// Absent when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_headers"
    )]
    pub headers: BTreeMap<String, Header>,
}

/// One query parameter and the one literal value of it that selects a representation, written
/// `{<parameter>: <value>}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    /// The query parameter's name.
    pub parameter: String,
    /// The value that selects the representation.
    pub value: String,
}

impl serde::Serialize for Selector {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.parameter, &self.value)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Selector {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let Entries(mut entries) =
            <Entries<String> as serde::Deserialize>::deserialize(deserializer)?;
        if entries.len() != 1 {
            return Err(serde::de::Error::custom(format!(
                "a representation's `query` holds {} parameters; compiled IR writes exactly one",
                entries.len()
            )));
        }
        let (parameter, value) = entries.remove(0);
        Ok(Self { parameter, value })
    }
}

/// A compiled representation's `encoding`: `{csv: {…}}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepresentationEncoding {
    /// Delimited text, one line per row.
    pub csv: Csv,
}

/// A compiled `csv` encoding: each default omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Csv {
    /// What separates two fields of a line. Absent when `,`.
    #[serde(
        default,
        skip_serializing_if = "Separator::is_default",
        deserialize_with = "omitted_comma"
    )]
    pub separator: Separator,
    /// Whether a header line of the columns' wire names comes first. Absent when `true`.
    #[serde(
        default = "header_line",
        skip_serializing_if = "is_header_line",
        deserialize_with = "omitted_header_line"
    )]
    pub header: bool,
}

impl Default for Csv {
    fn default() -> Self {
        Self {
            separator: Separator::default(),
            header: true,
        }
    }
}

/// What separates two fields of a CSV line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Separator {
    /// `,`, the default.
    #[default]
    #[serde(rename = ",")]
    Comma,
    /// `;`.
    #[serde(rename = ";")]
    Semicolon,
    /// A horizontal tab.
    #[serde(rename = "\t")]
    Tab,
}

impl Separator {
    /// The separator `text` spells, if it is one.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "," => Some(Self::Comma),
            ";" => Some(Self::Semicolon),
            "\t" => Some(Self::Tab),
            _ => None,
        }
    }

    /// The character it is.
    pub fn as_char(self) -> char {
        match self {
            Self::Comma => ',',
            Self::Semicolon => ';',
            Self::Tab => '\t',
        }
    }

    // `skip_serializing_if` passes the field by reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

fn header_line() -> bool {
    true
}

// `skip_serializing_if` passes the field by reference.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_header_line(header: &bool) -> bool {
    *header
}

/// Reads a `csv` encoding's `separator` of compiled IR, refusing `,` written out.
fn omitted_comma<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Separator, D::Error> {
    let separator = <Separator as serde::Deserialize>::deserialize(deserializer)?;
    if separator.is_default() {
        return Err(serde::de::Error::custom(
            "a `csv` encoding's `separator` is `,`, its default, which compiled IR omits",
        ));
    }
    Ok(separator)
}

/// Reads a `csv` encoding's `header` of compiled IR, refusing `true` written out.
fn omitted_header_line<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    let header = <bool as serde::Deserialize>::deserialize(deserializer)?;
    if header {
        return Err(serde::de::Error::custom(
            "a `csv` encoding's `header` is `true`, its default, which compiled IR omits",
        ));
    }
    Ok(header)
}

/// Reads a route's `representations` of compiled IR, refusing them written empty: compiled IR
/// omits a route's `representations` when it declares none.
fn canonical_representations<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Representation>, D::Error> {
    let representations = <Vec<Representation> as serde::Deserialize>::deserialize(deserializer)?;
    if representations.is_empty() {
        return Err(serde::de::Error::custom(
            "a route's `representations` is empty; compiled IR omits it when the route declares \
             none",
        ));
    }
    Ok(representations)
}

impl Route {
    /// Whether the route reads a request body: it serves a command, its method is not `get` and
    /// its request is not encoded `query`. A view, a `get` command and a `query`-encoded command
    /// carry their input in the path or the query string.
    pub fn reads_body(&self) -> bool {
        reads_body(
            &self.operation,
            self.method,
            self.request.as_ref().map(|request| request.encoding),
        )
    }

    /// The path's segments; `None` for an open route.
    ///
    /// A compiled path always parses: [`compile`] refuses one that does not, and
    /// [`HttpIr::from_json`] refuses compiled IR holding one.
    pub fn segments(&self) -> Option<Vec<Segment>> {
        self.path.as_deref().and_then(|path| template(path).ok())
    }

    /// The declared answer of the command's outcome `name`: the one listed under `outcomes`, else
    /// `errors` when the outcome reports an error (`reports_error`); `None` when it keeps its
    /// derived status and the envelope.
    pub fn answer(&self, name: &str, reports_error: bool) -> Option<&Outcome> {
        self.outcomes.get(name).or(if reports_error {
            self.errors.as_ref()
        } else {
            None
        })
    }

    /// Whether the route declares any answer of a command's outcomes, under `outcomes` or
    /// `errors`.
    pub fn declares_answers(&self) -> bool {
        !self.outcomes.is_empty() || self.errors.is_some()
    }
}

/// One route ESS derives for an operation of one component, without a binding.
///
/// [`compile`] takes the whole derived table as an input rather than deriving it, so that this
/// crate does not depend on the generator that reads the compiled binding: the caller fills it from
/// `ess_gen::http::routes`, one entry per route of each component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedRoute {
    /// The component whose surface it is on.
    pub component: String,
    /// What it serves.
    #[serde(flatten)]
    pub operation: Operation,
    /// The method.
    pub method: Method,
    /// The derived path, which holds no variable.
    pub path: String,
}

/// A compiled `ess-http-ir/1`: every route resolved, ordered by path, then method, then operation,
/// and the derived route of every operation the binding does not name, which together are the
/// whole surface the routes were checked against.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpIr {
    format: String,
    specification: SpecificationIdentity,
    /// The digest of the refined binding's bytes. Absent when the binding refines nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refines: Option<String>,
    /// The largest request body a route that reads one takes. Absent when not declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    limits: Option<Limits>,
    /// The standard refusals the binding overrides, in key order. Absent when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_refusals"
    )]
    refusals: BTreeMap<RefusalKey, Refusal>,
    /// Each `reached_by: network` component's base URLs, in component order, each list in the
    /// order written. Absent when empty.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "canonical_servers"
    )]
    servers: BTreeMap<String, Vec<Server>>,
    /// How a caller of the routes proves who it is. Absent when not declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authentication: Option<Authentication>,
    /// Which origins may read the answers of every route without its own policy. Absent when not
    /// declared: no cross-origin read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cors: Option<Cors>,
    routes: Vec<Route>,
    /// Ordered by component, then path, then method, then operation. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    derived_routes: Vec<DerivedRoute>,
}

impl HttpIr {
    /// The specification this binding binds.
    pub fn specification(&self) -> &SpecificationIdentity {
        &self.specification
    }

    /// `sha256:<hex>` of the bytes of the binding this one refines, if it refines one.
    pub fn refines(&self) -> Option<&str> {
        self.refines.as_deref()
    }

    /// The standard refusals the binding answers its own way, in key order.
    pub fn refusals(&self) -> &BTreeMap<RefusalKey, Refusal> {
        &self.refusals
    }

    /// The binding's own `limits`, if it declares them.
    pub fn limits(&self) -> Option<&Limits> {
        self.limits.as_ref()
    }

    /// The largest request body `route` reads, in bytes: `None` when it reads no body, else the
    /// route's own `limits` over the binding's, and `None` too when neither declares one (the
    /// server's [`FIXED_REQUEST_BYTES`] then applies).
    pub fn request_bytes(&self, route: &Route) -> Option<u64> {
        if !route.reads_body() {
            return None;
        }
        route
            .limits
            .as_ref()
            .or(self.limits.as_ref())
            .map(|limits| limits.request_bytes)
    }

    /// Whether the binding bounds the request of `route`: it reads a body and a `limits` or a
    /// `too_large` refusal, its own or the binding's, applies to it.
    pub fn bounds_request(&self, route: &Route) -> bool {
        route.reads_body()
            && (self.request_bytes(route).is_some()
                || route.refusals.contains_key(&RefusalKey::TooLarge)
                || self.refusals.contains_key(&RefusalKey::TooLarge))
    }

    /// Whether the binding bounds the request of any of its routes.
    pub fn bounds_requests(&self) -> bool {
        self.routes.iter().any(|route| self.bounds_request(route))
    }

    /// Each `reached_by: network` component's base URLs the binding declares, by component name,
    /// each list in the order written: the first is the default.
    pub fn servers(&self) -> &BTreeMap<String, Vec<Server>> {
        &self.servers
    }

    /// How a caller proves who it is, if the binding declares it.
    pub fn authentication(&self) -> Option<&Authentication> {
        self.authentication.as_ref()
    }

    /// Which origins may read the answers of every route without its own policy, if the binding
    /// declares it.
    pub fn cors(&self) -> Option<&Cors> {
        self.cors.as_ref()
    }

    /// The policy that stands on `route`, a declared route or `None` for a derived one: the
    /// route's own, none under `cors: none`, else the binding's [`HttpIr::cors`]. `None` means no
    /// cross-origin read.
    pub fn cors_on<'a>(&'a self, route: Option<&'a Route>) -> Option<&'a Cors> {
        match route.and_then(|route| route.cors.as_ref()) {
            Some(RouteCors::None) => None,
            Some(RouteCors::Own(cors)) => Some(cors),
            None => self.cors.as_ref(),
        }
    }

    /// The schemes a caller of `route` authenticates with, any one of them sufficing: the route's
    /// own, else the binding's `required`; empty when the route is open.
    pub fn required_schemes<'a>(&'a self, route: &'a Route) -> &'a [String] {
        match (&route.authentication, &self.authentication) {
            (Some(own), _) => own,
            (None, Some(authentication)) => &authentication.required,
            (None, None) => &[],
        }
    }

    /// Every route, ordered by path (an open route first), then method, then operation.
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// The route of one operation, if the binding names it.
    pub fn route(&self, operation: &Operation) -> Option<&Route> {
        self.routes
            .iter()
            .find(|route| &route.operation == operation)
    }

    /// The derived route of every operation the binding does not name, on each component that
    /// serves it, ordered by component, then path, then method, then operation.
    pub fn derived_routes(&self) -> &[DerivedRoute] {
        &self.derived_routes
    }

    /// The canonical JSON bytes, with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("HTTP IR serializes: {error}"));
        json.push('\n');
        json
    }

    /// Reads compiled IR back: canonical or compact JSON, in canonical order, which writes back
    /// canonical bytes ([`HttpIr::to_canonical_json`]).
    ///
    /// It refuses, naming the key, any format but [`HTTP_IR_FORMAT`], any unknown key, and IR that
    /// breaks an invariant [`compile`] guarantees and that needs no specification to check: a
    /// `refines` that is not a `sha256:` digest of 64 lower-case hex digits; `routes`, `derived_routes` or a route's `parameters` out of canonical order; one operation
    /// routed twice; a method not admitted for the kind of operation; a route path that breaks the
    /// template grammar, or whose `parameters` leave a variable unmapped, map one the path does
    /// not have, or read one source from two variables; a source outside the operation's
    /// namespace; a `request` on a view, holding only defaults or writing a default out, encoded
    /// `query` on any method but `get` and `delete` or `json` or `form` on `get`, or `ignore`
    /// beside a bare `body`; `outcomes` on a view, or `body` or `empty` on a command; `outcomes` or `refusals` out of key
    /// order or empty; an outcome status outside `100..=599`, a `1xx` or a `3xx` other than the
    /// [`REDIRECTS`]; an outcome body that is `envelope` written out, a literal, a view's body, an
    /// error object under an empty member, or anything but `none` on `204`; an outcome `encoding`
    /// `json` written out, or `text` beside any body but `response.<field>`; `alike` on a view,
    /// written empty, a group of fewer than two names, a name in two groups or twice, names out of
    /// name order or groups out of the order of their first names, a name no listed outcome or
    /// `errors` answers, or a group whose answers differ or hold an `envelope`; a view body `rows`
    /// written out, `single` without `empty` or `empty` without `single` (beside `rows` or
    /// `array`), an `empty` status outside `4xx`, or an `empty` body `none` written out or a
    /// literal under an empty member; a refusal key outside `malformed`, `not_granted`,
    /// `too_large`, `unauthenticated` and `unfinished`, a refusal status outside its class (`4xx`, `4xx`, `4xx`, `4xx`, `5xx`), or a refusal body
    /// other than `none` or one literal member with a name, the binding's or a route's own, and a
    /// route's `refusals` out of key order or written empty; `headers`, challenge `params` or redirect `parameters` out of
    /// name order or empty; a header name that is not a token, one the server, the encoding or
    /// `redirect` writes, or one declared twice ignoring case; a literal header that is not a field
    /// value; a challenge under any header but `WWW-Authenticate`, with a scheme or param name
    /// that is not a token, a param twice ignoring case or a literal param a quoted string cannot
    /// carry; a `401` without a `WWW-Authenticate` challenge; a source in a refusal's header; a
    /// `redirect` on any status but the [`REDIRECTS`], a redirect status without one, or a body
    /// other than `none` beside one; a redirect `mode` outside `query` and `fragment`, a
    /// `location` reading `error.wire`, a parameter name outside RFC 3986 `unreserved`, and any
    /// source outside the four forms; a derived route for an operation the IR routes; a derived path that is
    /// not a grammar-valid path without variables; a route's `operation_id` outside
    /// `[A-Za-z_][A-Za-z0-9_.-]*`, given to two routes, or equal to the qualified name of another
    /// operation the IR routes or derives; and `servers` out of component order or empty, a
    /// component's list empty, or a server URL that is neither absolute `http(s)://` with a host
    /// nor `/`-rooted, holds whitespace, `{`, `}`, `?`, `#` or userinfo, or is given twice for one
    /// component; and a route's `representations` beside a command or a `single` body, a
    /// `query` of other than one parameter, two on different parameters or one value, a name
    /// that is a path variable, a name or value outside RFC 3986 `unreserved`, a `separator`
    /// outside `,`, `;` and a tab, or a header that is not a literal a binding may declare
    /// (beyond10x/ess#753); and an `authentication` scheme whose name is not a token, of a kind
    /// outside the four, with an `api_key` name its location cannot carry or a `basic` realm no
    /// quoted string carries, `schemes` out of name order or empty, and an
    /// `authentication.required` or a route's `authentication` naming a scheme not declared or
    /// one twice (beyond10x/ess#681); and a `cors` policy, the binding's or a route's own, holding `any` beside credentials, an empty `origins`, an origin that is not `scheme://host[:port]` or one twice, a header name that is not a token or one twice, an `expose` name no covered route declares, or a `max_age` over one day, and a route `cors` other than `none` or a policy; and a route's `lists` out of name order, empty, with a form other than `comma`, naming a source a path variable reads, or naming on a command a field neither a `form` or `query` encoding nor `request.query` carries. Beyond these, it refuses, naming
    /// the key, any IR that would not write back what it was given: a key compiled IR omits,
    /// written as an empty map or list, a `null` or a default.
    ///
    /// What needs the specification is not checked: that each operation, field and outcome exists,
    /// its type, the `get` input, an outcome's status class and body against its error and
    /// `returns`, whether a `text` field is a `String`, whether an `alike` name is an outcome of
    /// the command and whether error objects alike share one wire name, the distinguishability of
    /// outcomes and overridden refusals on one status, a
    /// `single` view's identity selection and paging, an `array` view's paging, a representation's
    /// view parameters and flat fields, whether the outcome supplies each header,
    /// param and redirect source and its type, and collisions on each component's surface.
    /// A reader that needs those compiles the authored binding instead.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let ir: Self = serde_json::from_str(text)?;
        if ir.format != HTTP_IR_FORMAT {
            return Err(serde::de::Error::custom(format!(
                "expected `{HTTP_IR_FORMAT}`, found `{}`",
                ir.format
            )));
        }
        ir.invariants().map_err(serde::de::Error::custom)?;
        // Whatever a field's reader admits, the IR read writes back exactly what it was given: a
        // key compiled IR omits (an empty map or list, a `null`, a default) is refused by name.
        let given: serde_json::Value = serde_json::from_str(text)?;
        let written = serde_json::to_value(&ir)?;
        if let Some(at) = first_difference(&given, &written, "") {
            return Err(serde::de::Error::custom(format!(
                "`{at}` is not as compiled IR writes it: compiled IR omits an empty map or list, a \
                 `null` and a default, and a reader writes back only the bytes it was given"
            )));
        }
        Ok(ir)
    }

    /// The invariants of compiled IR that need no specification, each refusal naming its key.
    fn invariants(&self) -> Result<(), String> {
        if let Some(refines) = &self.refines {
            if !is_digest(refines) {
                return Err(format!(
                    "`refines` `{refines}` is not `sha256:` and 64 lower-case hex digits, the \
                     digest compiled IR records of the refined binding's bytes"
                ));
            }
        }
        if let Some(limits) = &self.limits {
            limits_invariants(*limits, "limits")?;
        }
        for (key, refusal) in &self.refusals {
            refusal_invariants(*key, refusal, "refusals")?;
        }
        let route_key = |route: &Route| (route.path.clone(), route.method, route.operation.clone());
        if let Some(at) = out_of_order(&self.routes, |left, right| {
            route_key(left).cmp(&route_key(right))
        }) {
            return Err(format!(
                "`routes[{at}]` is out of canonical order: compiled IR orders `routes` by path, \
                 then method, then operation, each once"
            ));
        }
        if let Some(at) = out_of_order(&self.derived_routes, derived_order) {
            return Err(format!(
                "`derived_routes[{at}]` is out of canonical order: compiled IR orders \
                 `derived_routes` by component, then path, then method, then operation, each once"
            ));
        }
        let mut routed: BTreeMap<&Operation, usize> = BTreeMap::new();
        for (index, route) in self.routes.iter().enumerate() {
            let at = format!("routes[{index}]");
            if let Some(first) = routed.insert(&route.operation, index) {
                return Err(format!(
                    "`{at}` routes {}, which `routes[{first}]` already routes; compiled IR routes \
                     an operation once",
                    route.operation
                ));
            }
            route_invariants(route, &at)?;
            for (key, refusal) in &route.refusals {
                refusal_invariants(*key, refusal, &format!("{at}.refusals"))?;
            }
            if let Some(RouteCors::Own(cors)) = &route.cors {
                cors_invariants(cors, &format!("{at}.cors"))?;
            }
        }
        if let Some(cors) = &self.cors {
            cors_invariants(cors, "cors")?;
        }
        let indexed = self
            .routes
            .iter()
            .enumerate()
            .map(|(index, route)| (format!("routes[{index}]"), route));
        if let Some((at, why)) = unexposed(self.cors.as_ref(), &self.refusals, indexed)
            .into_iter()
            .next()
        {
            return Err(format!("`{at}`: {why}"));
        }
        for (index, derived) in self.derived_routes.iter().enumerate() {
            let at = format!("derived_routes[{index}]");
            if let Some(routed) = routed.get(&derived.operation) {
                return Err(format!(
                    "`{at}` is a derived route of {}, which `routes[{routed}]` routes; \
                     `derived_routes` holds only operations the binding does not name",
                    derived.operation
                ));
            }
            match template(&derived.path) {
                Err(reasons) => {
                    return Err(format!("`{at}.path`: {}", reasons.join("; ")));
                }
                Ok(segments) => {
                    if segments
                        .iter()
                        .any(|segment| matches!(segment, Segment::Variable(_)))
                    {
                        return Err(format!(
                            "`{at}.path` `{}` holds a variable; a derived path holds none",
                            derived.path
                        ));
                    }
                }
            }
        }
        self.authentication_invariants()?;
        self.presentation_invariants()
    }

    /// The invariants of `authentication` and of each route's list (see [`HttpIr::from_json`]):
    /// scheme names that are tokens, an `api_key` name a header, query or cookie can carry, a
    /// `basic` realm a quoted string carries, and lists that name declared schemes, each once.
    fn authentication_invariants(&self) -> Result<(), String> {
        let declared = self
            .authentication
            .as_ref()
            .map(|authentication| &authentication.schemes);
        let listed = |list: &[String], at: &str| -> Result<(), String> {
            for (index, name) in list.iter().enumerate() {
                if !declared.is_some_and(|schemes| schemes.contains_key(name)) {
                    return Err(format!(
                        "`{at}` names `{name}`, which `authentication.schemes` does not declare"
                    ));
                }
                if list[..index].contains(name) {
                    return Err(format!("`{at}` names `{name}` twice"));
                }
            }
            Ok(())
        };
        if let Some(authentication) = &self.authentication {
            for (name, scheme) in &authentication.schemes {
                if let Some(reason) = scheme_reason(name, scheme) {
                    return Err(format!("`authentication.schemes.{name}`: {reason}"));
                }
            }
            listed(&authentication.required, "authentication.required")?;
        }
        for (index, route) in self.routes.iter().enumerate() {
            if let Some(list) = &route.authentication {
                listed(list, &format!("routes[{index}].authentication"))?;
            }
        }
        Ok(())
    }

    /// The invariants of the operation ids and servers of compiled IR (see [`HttpIr::from_json`]).
    fn presentation_invariants(&self) -> Result<(), String> {
        let operations: Vec<&Operation> = self
            .routes
            .iter()
            .map(|route| &route.operation)
            .chain(self.derived_routes.iter().map(|derived| &derived.operation))
            .collect();
        let mut given: BTreeMap<&str, usize> = BTreeMap::new();
        for (index, route) in self.routes.iter().enumerate() {
            let Some(id) = route.operation_id.as_deref() else {
                continue;
            };
            let at = format!("routes[{index}].operation_id");
            if !is_operation_id(id) {
                return Err(format!("`{at}`: {}", operation_id_reason(id)));
            }
            if let Some(first) = given.insert(id, index) {
                return Err(format!(
                    "`{at}` `{id}` is the operation id of `routes[{first}]` too; compiled IR gives \
                     an operation id to one route"
                ));
            }
            if let Some(other) = operations
                .iter()
                .find(|other| other.name() == id && **other != &route.operation)
            {
                return Err(format!(
                    "`{at}` `{id}` is the qualified name of {other}, its operation id; compiled IR \
                     gives no route another operation's name"
                ));
            }
        }
        for (component, servers) in &self.servers {
            let at = format!("servers.{component}");
            if servers.is_empty() {
                return Err(format!(
                    "`{at}` is empty; compiled IR omits a component without servers"
                ));
            }
            for (index, server) in servers.iter().enumerate() {
                let key = format!("{at}[{index}].url");
                if let Some(reason) = server_url_reason(&server.url) {
                    return Err(format!("`{key}`: {reason}"));
                }
                if servers[..index]
                    .iter()
                    .any(|earlier| earlier.url == server.url)
                {
                    return Err(format!(
                        "`{key}` `{}` is given twice; compiled IR lists a URL once per component",
                        server.url
                    ));
                }
            }
        }
        Ok(())
    }
}

/// The invariants of one compiled route that need no specification (see [`HttpIr::from_json`]).
fn route_invariants(route: &Route, at: &str) -> Result<(), String> {
    if route.select.is_some() {
        let body = route
            .request
            .as_ref()
            .is_none_or(|request| request.encoding == Encoding::Json && request.body.is_none());
        if !matches!(route.operation, Operation::Command(_)) || route.method == Method::Get || !body
        {
            return Err(format!(
                "`{at}.select` is on {} on `{}`; a select tests members of a command's `json` \
                 body that is the input object, off `get`",
                route.operation, route.method
            ));
        }
    }
    let admitted = match route.operation {
        Operation::View(_) => route.method == Method::Get,
        Operation::Command(_) => {
            !matches!(route.method, Method::Head | Method::Options | Method::Trace)
        }
    };
    if !admitted {
        return Err(format!(
            "`{at}.method` `{}` is not admitted for a {}",
            route.method,
            route.operation.kind()
        ));
    }
    let namespace = match route.operation {
        Operation::Command(_) => "input",
        Operation::View(_) => "param",
    };
    let mut variables: Vec<String> = Vec::new();
    if let Some(path) = &route.path {
        let segments =
            template(path).map_err(|reasons| format!("`{at}.path`: {}", reasons.join("; ")))?;
        for segment in &segments {
            if let Segment::Variable(name) = segment {
                if variables.contains(name) {
                    return Err(format!(
                        "`{at}.path` `{path}` names variable `{name}` twice; a variable names one \
                         segment"
                    ));
                }
                variables.push(name.clone());
            }
        }
    }
    let written = route.path.as_deref().unwrap_or("(open)");
    for name in &variables {
        if !route.parameters.contains_key(name) {
            return Err(format!(
                "`{at}.parameters.{name}` is missing: variable `{name}` of `{written}` is not \
                 mapped, and compiled IR maps each variable of the path"
            ));
        }
    }
    let mut read: Vec<(&Source, &str)> = Vec::new();
    for (name, source) in &route.parameters {
        let key = format!("{at}.parameters.{name}");
        if !variables.contains(name) {
            return Err(format!(
                "`{key}` maps no variable of `{written}`; compiled IR maps only variables of the \
                 path"
            ));
        }
        if source.namespace() != namespace {
            return Err(format!(
                "`{key}` reads `{source}`, outside `{namespace}.`, the namespace of a {}",
                route.operation.kind()
            ));
        }
        if let Some((_, earlier)) = read.iter().find(|(seen, _)| *seen == source) {
            return Err(format!(
                "`{key}` reads `{source}`, which `{earlier}` already reads; compiled IR reads a \
                 source from one variable"
            ));
        }
        read.push((source, name));
    }
    if let Some(request) = &route.request {
        request_invariants(route, request, at)?;
    }
    lists_invariants(route, at)?;
    if let Some(limits) = &route.limits {
        if !route.reads_body() {
            return Err(format!(
                "`{at}.limits` is on {}, which reads no request body",
                route.operation
            ));
        }
        limits_invariants(*limits, &format!("{at}.limits"))?;
    }
    representations_invariants(route, &variables, at)?;
    answer_invariants(route, at)
}

/// Holds a route's compiled `lists` to what compiling guarantees without the specification: no
/// name a variable of the path reads, and on a command only a name the query string or a `form`
/// body carries, the route encoded `form` or `query` or the name listed in `request.query`.
fn lists_invariants(route: &Route, at: &str) -> Result<(), String> {
    let request = route.request.as_ref();
    let encoding = request.map_or(Encoding::Json, |request| request.encoding);
    for name in route.lists.keys() {
        let key = format!("{at}.lists.{name}");
        if route
            .parameters
            .values()
            .any(|source| source.field() == name.as_str())
        {
            return Err(format!(
                "`{key}` is carried by the path, which holds one value and no list"
            ));
        }
        let queried =
            request.is_some_and(|request| request.query_fields().any(|field| field == name));
        if matches!(route.operation, Operation::Command(_))
            && !queried
            && matches!(encoding, Encoding::Json | Encoding::Multipart)
        {
            return Err(format!(
                "`{key}` travels in a `{}` body; compiled IR writes comma-joined only a list the \
                 query string or a `form` body carries",
                encoding.as_str()
            ));
        }
    }
    Ok(())
}

/// Holds a route's compiled `representations` to what compiling guarantees without the
/// specification (beyond10x/ess#753): only on a view route that answers `rows` or `array`, every
/// selector on one parameter that is no path variable, each value once, names and values of
/// `unreserved` characters, and literal headers only, under names a binding may declare.
fn representations_invariants(route: &Route, variables: &[String], at: &str) -> Result<(), String> {
    let Some(first) = route.representations.first() else {
        return Ok(());
    };
    let key = format!("{at}.representations");
    if !matches!(route.operation, Operation::View(_)) || route.body.form == ViewBody::Single {
        return Err(format!(
            "`{key}` is on {} answering `{}`; only a view route answering `rows` or `array` \
             has representations",
            route.operation, route.body
        ));
    }
    let parameter = &first.query.parameter;
    if !unreserved(parameter) || variables.contains(parameter) {
        return Err(format!(
            "`{key}[0].query` selects by `{parameter}`, which is not one or more `unreserved` \
             characters or is a variable of the path"
        ));
    }
    let mut values: Vec<&str> = Vec::new();
    for (index, representation) in route.representations.iter().enumerate() {
        let at = format!("{key}[{index}]");
        let Selector {
            parameter: name,
            value,
        } = &representation.query;
        if name != parameter {
            return Err(format!(
                "`{at}.query` selects by `{name}`, and `{key}[0]` by `{parameter}`; every \
                 representation of a route is selected by one parameter"
            ));
        }
        if !unreserved(value) || values.contains(&value.as_str()) {
            return Err(format!(
                "`{at}.query` selects by the value `{value}`, which is not one or more \
                 `unreserved` characters or selects another representation too"
            ));
        }
        values.push(value);
        for (name, header) in &representation.headers {
            if !is_token(name)
                || RESERVED_HEADERS
                    .iter()
                    .any(|(owned, _)| owned.eq_ignore_ascii_case(name))
                || !matches!(header, Header::Literal(text) if field_value_reason(text).is_none())
            {
                return Err(format!(
                    "`{at}.headers.{name}` is not a literal header a binding may declare"
                ));
            }
        }
    }
    Ok(())
}

/// The rules of a compiled policy at `at` that need no route ([`cors_problems`]).
fn cors_invariants(cors: &Cors, at: &str) -> Result<(), String> {
    match cors_problems(cors).into_iter().next() {
        Some((key, why)) => Err(format!("`{at}.{key}`: {why}")),
        None => Ok(()),
    }
}

/// Holds a compiled `limits` to its range.
fn limits_invariants(limits: Limits, at: &str) -> Result<(), String> {
    if !(1..=MAX_REQUEST_BYTES).contains(&limits.request_bytes) {
        return Err(format!(
            "`{at}.request_bytes` `{}` is not a whole number of bytes from 1 to {MAX_REQUEST_BYTES}",
            limits.request_bytes
        ));
    }
    Ok(())
}

/// The invariants of one compiled route's answers (see [`HttpIr::from_json`]): `outcomes` on a
/// command, each with a status an outcome may answer and a body admitted beside it; `body` and
/// `empty` on a view, `single` exactly with an `empty` holding a `4xx` and no body or one named
/// literal member.
fn answer_invariants(route: &Route, at: &str) -> Result<(), String> {
    match route.operation {
        Operation::View(_) => {
            if !route.outcomes.is_empty() {
                return Err(format!(
                    "`{at}.outcomes` is on {}; a view answers rows, or one row, not outcomes",
                    route.operation
                ));
            }
            if route.errors.is_some() {
                return Err(format!(
                    "`{at}.errors` is on {}; a view answers rows, or one row, not outcomes",
                    route.operation
                ));
            }
            if !route.alike.is_empty() {
                return Err(format!(
                    "`{at}.alike` is on {}; a view has no outcomes to answer alike",
                    route.operation
                ));
            }
            match (route.body.form, &route.empty) {
                (ViewBody::Single, None) => Err(format!(
                    "`{at}.body` is `{}`, and `{at}.empty` is missing; compiled IR writes \
                     the empty answer of every `single` route",
                    route.body
                )),
                (ViewBody::Rows | ViewBody::Array, Some(_)) => Err(format!(
                    "`{at}.empty` is on a route that answers `{}`; only a `single` route has \
                     an empty answer",
                    route.body
                )),
                (_, Some(empty)) if !(400..500).contains(&empty.status) => Err(format!(
                    "`{at}.empty.status` `{}` is not a `4xx` status",
                    empty.status
                )),
                (_, Some(empty)) if matches!(&empty.body, RefusalBody::Literal { member, .. } if member.is_empty()) => {
                    Err(format!(
                        "`{at}.empty.body` names an empty member; a literal body has one named \
                         member"
                    ))
                }
                _ => Ok(()),
            }
        }
        Operation::Command(_) => {
            if !route.body.is_default() {
                return Err(format!(
                    "`{at}.body` is on {}; a command answers per outcome, under `outcomes`",
                    route.operation
                ));
            }
            if route.empty.is_some() {
                return Err(format!(
                    "`{at}.empty` is on {}; only a `single` view route has an empty answer",
                    route.operation
                ));
            }
            for (name, outcome) in &route.outcomes {
                outcome_invariants(outcome, &format!("{at}.outcomes.{name}"))?;
            }
            if let Some(errors) = &route.errors {
                let key = format!("{at}.errors");
                if !(300..600).contains(&errors.status) {
                    return Err(format!(
                        "`{key}.status` `{}` is not a status an outcome that reports an error \
                         answers: a redirect, a `4xx` or a `5xx`",
                        errors.status
                    ));
                }
                if errors.body.reads_response() {
                    return Err(format!(
                        "`{key}.body` is `{}`, the command's response, and an outcome that \
                         reports an error answers none",
                        errors.body
                    ));
                }
                outcome_invariants(errors, &key)?;
            }
            alike_invariants(route, at)
        }
    }
}

/// The invariants of one compiled command route's `alike` (see [`HttpIr::from_json`]): each group
/// two or more names in name order, a name in one group once, the groups ordered by their first
/// name, and every name with a declared answer (under `outcomes`, else `errors`), one answer for
/// the whole group and never `envelope`. That a name is an outcome of the command, and that
/// `errors` covers it, needs the specification.
fn alike_invariants(route: &Route, at: &str) -> Result<(), String> {
    let key = format!("{at}.alike");
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, group) in route.alike.iter().enumerate() {
        let at = format!("{key}[{index}]");
        if group.len() < 2 {
            return Err(format!(
                "`{at}` names {} outcome; compiled IR writes a group of two or more",
                group.len()
            ));
        }
        for name in group {
            if let Some(first) = seen.insert(name, index) {
                return Err(format!(
                    "`{at}` names `{name}`, which `{key}[{first}]` names too; compiled IR names an \
                     outcome once in `alike`"
                ));
            }
        }
        if out_of_order(group, Ord::cmp).is_some() {
            return Err(format!(
                "`{at}` is out of canonical order: compiled IR writes a group's names in name \
                 order"
            ));
        }
    }
    if let Some(index) = out_of_order(&route.alike, |left, right| left[0].cmp(&right[0])) {
        return Err(format!(
            "`{key}[{index}]` is out of canonical order: compiled IR orders the groups of `alike` \
             by their first name"
        ));
    }
    for (index, group) in route.alike.iter().enumerate() {
        let at = format!("{key}[{index}]");
        let mut answers = Vec::new();
        for name in group {
            let Some(answer) = route.outcomes.get(name).or(route.errors.as_ref()) else {
                return Err(format!(
                    "`{at}` names `{name}`, which has no declared answer: it is not under \
                     `outcomes`, and the route declares no `errors`"
                ));
            };
            answers.push((name, answer));
        }
        let (first, answer) = answers[0];
        if let Some((other, _)) = answers.iter().find(|(_, other)| *other != answer) {
            return Err(format!(
                "`{at}`: the answers of `{first}` and `{other}` differ; compiled IR groups only \
                 outcomes with one declared answer"
            ));
        }
        if answer.body == Body::Envelope {
            return Err(format!(
                "`{at}` answers `envelope`, whose `outcome` member tells the outcomes apart; \
                 compiled IR groups only outcomes the wire cannot tell apart"
            ));
        }
    }
    Ok(())
}

/// The invariants of one compiled outcome's answer at `key` (see [`HttpIr::from_json`]): a status
/// an outcome may answer, an error object under a named member, `text` only beside a
/// `response.<field>` body, `none` on `204` and beside a redirect, a redirect exactly on a
/// redirect status, and its headers.
fn outcome_invariants(outcome: &Outcome, key: &str) -> Result<(), String> {
    if let Some(reason) = status_reason(i64::from(outcome.status)) {
        return Err(format!("`{key}.status`: {reason}"));
    }
    if outcome.encoding == ResponseEncoding::Text && !matches!(outcome.body, Body::ResponseField(_))
    {
        return Err(format!(
            "`{key}.encoding` is `text` beside body `{}`; compiled IR writes `text` only beside \
             a `response.<field>` body",
            outcome.body
        ));
    }
    if let Body::Error { member } = &outcome.body {
        if member.is_empty() {
            return Err(format!(
                "`{key}.body` names an empty member; the error object carries the error's wire \
                 name under a named member"
            ));
        }
    }
    if outcome.status == 204 && outcome.body != Body::None {
        return Err(format!(
            "`{key}.body` is `{}` on `204`, which carries no content; compiled IR writes `none` \
             there",
            outcome.body
        ));
    }
    let redirects = REDIRECTS.contains(&outcome.status);
    match (&outcome.redirect, redirects) {
        (Some(_), false) => {
            return Err(format!(
                "`{key}.redirect` is on `{}`, which is not a redirect status; compiled IR writes \
                 a `redirect` on `301`, `302`, `303`, `307` and `308` only",
                outcome.status
            ));
        }
        (None, true) => {
            return Err(format!(
                "`{key}` answers `{}`, a redirect status, without a `redirect`; compiled IR \
                 writes the `redirect` of every redirect status",
                outcome.status
            ));
        }
        _ => {}
    }
    if outcome.redirect.is_some() && outcome.body != Body::None {
        return Err(format!(
            "`{key}.body` is `{}` beside a `redirect`; compiled IR writes `none` there",
            outcome.body
        ));
    }
    if let Some(redirect) = &outcome.redirect {
        redirect_invariants(redirect, key)?;
    }
    headers_invariants(&outcome.headers, outcome.status, key, false)
}

/// The invariants of one compiled standard refusal (see [`HttpIr::from_json`]): a status of its
/// key's class, and a literal body under a named member when it has one. `prefix` is `refusals`,
/// or a route's `routes[i].refusals`.
fn refusal_invariants(key: RefusalKey, refusal: &Refusal, prefix: &str) -> Result<(), String> {
    if refusal.status / 100 != key.class() {
        return Err(format!(
            "`{prefix}.{key}.status` `{}` is not a `{}xx` status, the class of `{key}`",
            refusal.status,
            key.class()
        ));
    }
    if let RefusalBody::Literal { member, .. } = &refusal.body {
        if member.is_empty() {
            return Err(format!(
                "`{prefix}.{key}.body` names an empty member; a literal body has one named member"
            ));
        }
    }
    headers_invariants(
        &refusal.headers,
        refusal.status,
        &format!("{prefix}.{key}"),
        true,
    )
}

/// The invariants of one compiled outcome's `redirect` that need no specification (see
/// [`HttpIr::from_json`]), under the outcome's `key`: a `location` other than `error.wire`, and
/// parameter names in RFC 3986 `unreserved` characters.
fn redirect_invariants(redirect: &Redirect, key: &str) -> Result<(), String> {
    if redirect.location == AnswerSource::ErrorWire {
        return Err(format!(
            "`{key}.redirect.location` reads `error.wire`, the error's wire name; compiled IR \
             reads a location holding an absolute URI"
        ));
    }
    if let Some(name) = redirect.parameters.keys().find(|name| !unreserved(name)) {
        return Err(format!(
            "`{key}.redirect.parameters.{name}`: {}",
            unreserved_reason(name)
        ));
    }
    Ok(())
}

/// The invariants of one compiled answer's `headers` (see [`HttpIr::from_json`]), under `at`:
/// token names, none the server, the encoding or `redirect` writes, none twice ignoring case,
/// literals that are field values, challenges under `WWW-Authenticate` only with a token scheme
/// and token params once each whose literals a quoted string carries, a `WWW-Authenticate`
/// challenge on `401`, and, on a standard refusal (`refusal`), no source.
fn headers_invariants(
    headers: &BTreeMap<String, Header>,
    status: u16,
    at: &str,
    refusal: bool,
) -> Result<(), String> {
    let mut challenged = false;
    for (index, (name, header)) in headers.iter().enumerate() {
        let key = format!("{at}.headers.{name}");
        if !is_token(name) {
            return Err(format!(
                "`{key}`: `{name}` is not a header name, an RFC 9110 token"
            ));
        }
        if let Some((owned, owner)) = RESERVED_HEADERS
            .iter()
            .find(|(owned, _)| owned.eq_ignore_ascii_case(name))
        {
            return Err(format!(
                "`{key}`: `{owned}` is written by {owner}, and compiled IR declares no header it \
                 writes"
            ));
        }
        if let Some(first) = same_header(name, headers.keys().take(index).map(String::as_str)) {
            return Err(format!(
                "`{key}` is `{first}` declared twice, ignoring case; compiled IR declares each \
                 header once"
            ));
        }
        match header {
            Header::Literal(text) => {
                if let Some(reason) = field_value_reason(text) {
                    return Err(format!("`{key}`: {reason}"));
                }
            }
            Header::Source(source) => {
                if refusal {
                    return Err(format!(
                        "`{key}` reads `{source}`, and a standard refusal has no outcome to read a \
                         source from"
                    ));
                }
            }
            Header::Challenge(challenge) => {
                if !name.eq_ignore_ascii_case(CHALLENGE_HEADER) {
                    return Err(format!(
                        "`{key}` holds a challenge, which compiled IR writes under \
                         `{CHALLENGE_HEADER}` only"
                    ));
                }
                challenged = true;
                if !is_token(&challenge.scheme) {
                    return Err(format!(
                        "`{key}.challenge.scheme` `{}` is not a token",
                        challenge.scheme
                    ));
                }
                for (position, (param, value)) in challenge.params.iter().enumerate() {
                    let at = format!("{key}.challenge.params.{param}");
                    if !is_token(param) {
                        return Err(format!("`{at}`: `{param}` is not a token"));
                    }
                    let earlier = challenge.params.keys().take(position).map(String::as_str);
                    if let Some(first) = same_header(param, earlier) {
                        return Err(format!(
                            "`{at}` is `{first}` declared twice, ignoring case; a challenge names \
                             each param once"
                        ));
                    }
                    match value {
                        Param::Literal(text) if !quotable(text) => {
                            return Err(format!(
                                "`{at}`: {text:?} is not text a quoted string carries"
                            ));
                        }
                        Param::Source(source) if refusal => {
                            return Err(format!(
                                "`{at}` reads `{source}`, and a standard refusal has no outcome to \
                                 read a source from"
                            ));
                        }
                        Param::Literal(_) | Param::Source(_) => {}
                    }
                }
            }
        }
    }
    if status == 401 && !challenged {
        return Err(format!(
            "`{at}` answers `401` without a `{CHALLENGE_HEADER}` challenge, which compiled IR \
             writes on every `401` (RFC 9110 §15.5.2)"
        ));
    }
    Ok(())
}

/// The invariants of one compiled route's `request` (see [`HttpIr::from_json`]): it is on a
/// command, not all defaults, and admitted for the route's method, with `ignore` beside no
/// bare `body`, and a bare `body` beside `json` only, naming no field the path carries.
fn request_invariants(route: &Route, request: &Request, at: &str) -> Result<(), String> {
    if let Operation::View(_) = route.operation {
        return Err(format!(
            "`{at}.request` is on {}; a view takes no request",
            route.operation
        ));
    }
    if let Some(body) = &request.body {
        if request.encoding != Encoding::Json || route.method == Method::Get {
            return Err(format!(
                "`{at}.request.body` is `{body}` beside `{}` on `{}`; a bare body is a `json` \
                 body, which `get` carries none of",
                request.encoding, route.method
            ));
        }
        if route.parameters.values().any(|source| source == body) {
            return Err(format!(
                "`{at}.request.body` is `{body}`, which the path carries; a field travels in one \
                 place"
            ));
        }
    }
    if *request == Request::default() {
        return Err(format!(
            "`{at}.request` holds only defaults; compiled IR omits a request that is a `json` body \
             refusing an unknown key"
        ));
    }
    let admitted = match request.encoding {
        Encoding::Query => matches!(route.method, Method::Get | Method::Delete),
        Encoding::Json | Encoding::Form | Encoding::Multipart => route.method != Method::Get,
    };
    if !admitted {
        return Err(format!(
            "`{at}.request.encoding` `{}` is not admitted on `{}`: `query` is the only encoding \
             of a command on `get`, and an encoding of one on `delete`, and of no other",
            request.encoding, route.method
        ));
    }
    if request.body.is_some() && request.unknown == Unknown::Ignore {
        return Err(format!(
            "`{at}.request.unknown` is `ignore` beside a bare `body`, which holds no member of the \
             input to ignore"
        ));
    }
    parts_invariants(route, request, at)?;
    query_invariants(route, request, at)?;
    members_invariants(route, request, at)
}

/// The invariants of a compiled request's `members` (see [`HttpIr::from_json`]): beside a `json`
/// body off `get` without a bare `body` only, each field carried neither by the path nor by
/// `request.query`. The paths' own rules (none empty, twice or above another) hold when read.
fn members_invariants(route: &Route, request: &Request, at: &str) -> Result<(), String> {
    if request.members.is_empty() {
        return Ok(());
    }
    if request.encoding != Encoding::Json || route.method == Method::Get || request.body.is_some() {
        return Err(format!(
            "`{at}.request.members` is beside `{}` on `{}`{}; members are read from a `json` body \
             that is the input object",
            request.encoding,
            route.method,
            if request.body.is_some() {
                " and a bare `body`"
            } else {
                ""
            }
        ));
    }
    for (field, _) in request.members.iter() {
        let by_path = route
            .parameters
            .values()
            .any(|source| matches!(source, Source::Input(name) if name == field));
        if by_path || request.query_fields().any(|listed| listed == field) {
            return Err(format!(
                "`{at}.request.members.{field}` is carried by the {} already; a field travels in \
                 one place",
                if by_path { "path" } else { "query string" }
            ));
        }
    }
    Ok(())
}

/// The invariants of a compiled request's `query` (see [`HttpIr::from_json`]): beside a body
/// encoding off `get` only; each field listed once and carried nowhere else, neither by the path,
/// the bare `body`, a part nor a part attribute.
fn query_invariants(route: &Route, request: &Request, at: &str) -> Result<(), String> {
    if request.query.is_empty() {
        return Ok(());
    }
    if request.encoding == Encoding::Query || route.method == Method::Get {
        return Err(format!(
            "`{at}.request.query` is beside `{}` on `{}`; it lists the input fields read from the \
             query string beside a body",
            request.encoding, route.method
        ));
    }
    for (index, source) in request.query.iter().enumerate() {
        let attribute = request.parts.values().any(|part| {
            part.filename.as_ref() == Some(source) || part.media_type.as_ref() == Some(source)
        });
        let carrier = if request.query[..index].contains(source) {
            Some("listed twice")
        } else if route.parameters.values().any(|carried| carried == source) {
            Some("carried by the path")
        } else if request.body.as_ref() == Some(source) {
            Some("the bare body")
        } else if request.parts.contains_key(source.field()) || attribute {
            Some("read from a part")
        } else {
            None
        };
        if let Some(carrier) = carrier {
            return Err(format!(
                "`{at}.request.query` lists `{source}`, which is {carrier}; a field travels in \
                 one place"
            ));
        }
    }
    Ok(())
}

/// The invariants of a compiled request's `parts` (see [`HttpIr::from_json`]): beside `multipart`
/// only; each part stating something; no attribute reading the path's field, a part's field or
/// one another attribute reads; `media_types` distinct RFC 6838 `type/subtype` names.
fn parts_invariants(route: &Route, request: &Request, at: &str) -> Result<(), String> {
    if request.parts.is_empty() {
        return Ok(());
    }
    if request.encoding != Encoding::Multipart {
        return Err(format!(
            "`{at}.request.parts` is beside `{}`; parts are a `multipart` request's",
            request.encoding
        ));
    }
    let mut read: Vec<&Source> = Vec::new();
    for (key, part) in &request.parts {
        let here = format!("{at}.request.parts.{key}");
        if *part == Part::default() {
            return Err(format!("`{here}` states nothing, which compiled IR omits"));
        }
        for (attribute, source) in [
            ("filename", &part.filename),
            ("media_type", &part.media_type),
        ] {
            let Some(source) = source else {
                continue;
            };
            let carrier = if route.parameters.values().any(|carried| carried == source) {
                Some("which the path carries")
            } else if request.parts.contains_key(source.field()) {
                Some("which is a part")
            } else if read.contains(&source) {
                Some("which another attribute reads")
            } else {
                None
            };
            if let Some(carrier) = carrier {
                return Err(format!("`{here}.{attribute}` is `{source}`, {carrier}"));
            }
            read.push(source);
        }
        // An empty list is one left out, which admits any media type.
        if part.media_types.is_empty() {
            continue;
        }
        if let Some(rule) = media_types_rule(&part.media_types) {
            return Err(format!("`{here}.media_types`: {rule}"));
        }
    }
    Ok(())
}

/// The index of the first item that does not sort strictly after the one before it.
fn out_of_order<T>(items: &[T], order: impl Fn(&T, &T) -> std::cmp::Ordering) -> Option<usize> {
    items
        .windows(2)
        .position(|pair| order(&pair[0], &pair[1]) != std::cmp::Ordering::Less)
        .map(|position| position + 1)
}

/// Reads a compiled route's `parameters`, refusing a key that does not sort strictly after the one
/// before it: compiled IR orders them by variable, each once, and a reader that sorted them
/// silently would write back bytes it was not given.
fn canonical_parameters<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Source>, D::Error> {
    struct Entries;

    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = BTreeMap<String, Source>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map from path variable to source, ordered by variable")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = BTreeMap::new();
            let mut last: Option<String> = None;
            while let Some((variable, source)) = map.next_entry::<String, Source>()? {
                if let Some(before) = &last {
                    if *before >= variable {
                        return Err(serde::de::Error::custom(format!(
                            "`parameters` is out of canonical order: `{variable}` follows \
                             `{before}`; compiled IR orders `parameters` by variable, each once"
                        )));
                    }
                }
                last = Some(variable.clone());
                entries.insert(variable, source);
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_map(Entries)
}

/// One refusal, located at the key that caused it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct HttpDiagnostic {
    /// A stable code, `ESS-HTTP-NNN`.
    pub code: &'static str,
    /// The path of the offending key, such as `routes[0].method`.
    pub path: String,
    /// What is wrong, in a sentence.
    pub message: String,
}

impl fmt::Display for HttpDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "error[{}]: {}: {}",
            self.code, self.path, self.message
        )
    }
}

/// Every refusal a document earned, in document order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HttpDiagnostics(pub Vec<HttpDiagnostic>);

impl fmt::Display for HttpDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.0.iter().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }
            write!(formatter, "{diagnostic}")?;
        }
        Ok(())
    }
}

impl std::error::Error for HttpDiagnostics {}

struct Refusals(Vec<HttpDiagnostic>);

impl Refusals {
    fn refuse(&mut self, code: &'static str, path: impl Into<String>, message: impl Into<String>) {
        self.0.push(HttpDiagnostic {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
}

/// Resolves an HTTP binding against the specification it names and the routes ESS derives for it.
///
/// `derived` is the route table ESS derives without a binding, every route of every component
/// (`ess_gen::http::routes`); the caller supplies it, so this crate never depends on the generator.
/// An operation the binding names replaces its derived route on every component that serves it;
/// every other derived route stays, and the compiled IR records it.
///
/// Every check of `docs/design/http-presentation-binding.md` this build implements runs, stage 1's
/// and stage 2's request encoding, statuses, bodies, headers and redirects, and every refusal is
/// returned at once rather than the first:
///
/// | code | refused |
/// |---|---|
/// | `ESS-HTTP-001` | `type` is not `ess-http/1` |
/// | `ESS-HTTP-002` | a `specification` key disagrees with the IR |
/// | `ESS-HTTP-004` | a route names no operation, names both a command and a view, or names one the specification does not declare |
/// | `ESS-HTTP-005` | one operation is routed twice |
/// | `ESS-HTTP-006` | a method outside the closed set, or one not admitted for the operation: a view is bound to `get` only, a command to `post`, `put`, `patch`, `delete` or `get` |
/// | `ESS-HTTP-007` | a path that breaks the template grammar ([`template`]), naming the segment |
/// | `ESS-HTTP-008` | a path variable not mapped, mapped twice (two segments or two keys), reading a source another variable of the route already reads, or a mapping for no variable of the path |
/// | `ESS-HTTP-009` | a source in the wrong namespace (`input.` for a command, `param.` for a view), naming no field, or of a type a segment cannot carry: `String`, `Integer`, an enum or an unconstrained newtype of one of them |
/// | `ESS-HTTP-010` | two operations of one component on one method and template, variable names ignored, declared or derived; two declared command routes on one template whose `select` tests the same members for the same literals, two of them without `select`, or one `select` beside a route whose body is not a `json` input object (the message names `select:` as the way out when both are declared) |
/// | `ESS-HTTP-011` | two templates of one component and method that cross: a request path matches both, and each holds a literal at some position the other holds a variable (`/{a}/x` beside `/y/{b}`), naming both operations, a path both match and the segments. From 0.89.0 a nested pair, where one template holds the literal at every position the two differ (`/items/current` beside `/items/{item_id}`), is admitted and the more specific answers; two declared routes on one template whose selects cross: each tests a body member the other leaves open, naming both members and a body both select |
/// | `ESS-HTTP-013` | an operation with no HTTP surface: a command no component accepts, a view no `reached_by: network` component serves |
/// | `ESS-HTTP-014` | a command on `get` with an input field no variable of the path maps, unless its request is encoded `query` |
/// | `ESS-HTTP-020` | at `routes[i].request.encoding`: an encoding outside `json`, `form`, `query` and `multipart`, `json`, `form` or `multipart` written on a `get` command, or `query` on a method other than `get` and `delete`; at `routes[i].request.unknown`: a value outside `refuse` and `ignore`, or `ignore` beside a bare `body`; at `routes[i].request`: `request` on a view; at `routes[i].request.body`: `body` on a view, beside `form` or `query`, or on a `get` command, a body that is not `input.<field>` naming an input field of the command, one that is `Optional` or that the path carries, and, on a route with a path, each other input field neither the path nor `request.query` carries; at `routes[i].request.query`: `query` beside `encoding: query`, on a `get` command, or written empty, an entry that is not `input.<field>` naming an input field of the command, one listed twice, and one the path, the bare `body` or a part of a `multipart` request carries; at `routes[i].request.members`: `members` written empty; at `routes[i].request.members.<field>`: `members` on a view, beside `form`, `query`, `multipart` or a bare `body`, or on a `get` command, a key that is not an input field of the command, a field the path or `request.query` carries, a path that is not one or more non-empty member names joined by `.`, two fields on one path, one path an object above another, and a path starting at the top-level member an unmapped input field is carried under (its wire name); at `routes[i].select`: `select` on a view, beside `form`, `query`, `multipart` or a bare `body`, or on a `get` command, a value that is not a map or a list of maps, an empty map or list, a member path with an empty name, a literal that is not a string, a Boolean or an integer, two members one an object above the other, a literal the type of the input field on that member does not admit (naming the field and its type), a member above or inside the member an input field is read from, and two alternatives of one route that meet |
/// | `ESS-HTTP-021` | at `routes[i].request.encoding`: a `form`, `query` or `multipart` request carrying an input field whose type has no flat spelling, naming the field and its type. A flat field is a scalar (a primitive other than `Json`, an enum, or a newtype of a scalar), an `Optional` scalar or a `List` of scalars; a field mapped to the path is not carried; at `routes[i].request.query`: a field `query` lists whose type has no flat spelling, beside any encoding; at `routes[i].lists.<name>`: a name that is no input field of the command or parameter of the view, one named twice, one the path carries, one whose type is not a `List` of scalars (`Optional` or not), a command's field carried in a `json` or `multipart` body rather than the query string or a `form` body, or a form other than `comma`; at `routes[i].lists`: `lists` written empty |
/// | `ESS-HTTP-022` | at `routes[i].outcomes.<name>`: a key that is not the local name of an outcome of the command, or one listed twice; at `routes[i].outcomes`: `outcomes` on a view; at `routes[i].errors`: `errors` on a view, or on a command none of whose outcomes reports an error without being listed under `outcomes`; at `routes[i].alike`: `alike` on a view, a group of fewer than two names, a name that is not the local name of an outcome of the command, one in two groups or twice in one, or one with no declared answer (neither listed under `outcomes` nor covered by `errors`) |
/// | `ESS-HTTP-023` | at `routes[i].outcomes.<name>.status`, or `routes[i].errors.status` once for each outcome `errors` covers: a status outside `100..=599`, a `1xx`, a `3xx` other than `301`, `302`, `303`, `307` and `308`, a `2xx` on an outcome with an error, or a `4xx` or `5xx` on one without |
/// | `ESS-HTTP-024` | at `routes[i].outcomes.<name>.body`, or `routes[i].errors.body` once for each outcome `errors` covers: a word no body form spells, `response` or `response.<field>` on an outcome without `returns: true` or with an error, `response.<field>` naming no field of the response or an `Optional` one, `input.<field>`, `error.<field>` or `error.wire` as a body, `{<member>: error}` on one without an error, under an empty `<member>` or one equal to the wire name of a field of the error, a literal body, `rows`, `array` or `single`, or any body but `none` on `204` when that status compiled, or beside a `redirect` on a redirect status; at `routes[i].body`: `body` on a command, a body other than `rows`, `array` and `single` on a view, or `single` or `array` on a paged view; at `routes[i].outcomes.<name>.encoding`, or `routes[i].errors.encoding` once for each outcome `errors` covers: a word other than `json` and `text`, or `text` beside any body but `response.<field>` or on a field that is not a `String` (a newtype of one counts) |
/// | `ESS-HTTP-025` | at `routes[i].outcomes.<name>`, or `routes[i].errors` for an outcome `errors` covers: two outcomes of the command on one status whose bodies do not tell them apart (both `envelope`, or both `{<member>: error}` on one member with distinct error wire names, or with one wire name and one error always writing a field, its type not `Optional` through any newtype and its outcome not a `wrong_state` one that can answer an unknown instance, that the other never writes, do; one function decides the field rule for this check and for `alike`) and whose redirects do not either (both declare one and one always writes a parameter, its source not `Optional`, the other never writes, do), an outcome `errors` covers counted at the status and body of `errors`, any other the route does not list at its derived status ([`status::outcome_status`]) with `envelope`. Reported once, at the first listed outcome in document order, then the first covered one in declaration order, naming the others and the status; a pair already named together is not reported again, and an outcome left with none is not reported; at `refusals.<key>`: an override on a route the binding names, on the status of an outcome of its command, whose body does not tell the two apart (only `{<member>: {literal: <text>}}` beside `{<member>: error}` on one member, the text differing from the error's wire name, does), once per route, naming the route, the outcomes and the status; at `routes[i].refusals.<key>`: the same for a route's own override, which stands on that route in place of the binding's; two outcomes in one `alike` group are not a pair; at `routes[i].alike`: a group whose answers differ (the first differing key among `status`, `body`, `encoding`, `headers` and `redirect` named), or that the wire tells apart: `envelope` bodies, `{<member>: error}` bodies under distinct error wire names or over errors one of which always writes a field the other never writes (the message names the field), or that are not one answer, the fields of the errors differing otherwise (in an optional field, a type or an order), a header or challenge param reading `error.wire` under distinct error wire names, or redirects one of which always writes a parameter the other never does |
/// | `ESS-HTTP-026` | at `routes[i].outcomes.<name>.headers.<Name>`, `routes[i].errors.headers.<Name>` (once for each outcome `errors` covers), `refusals.<key>.headers.<Name>` or `routes[i].refusals.<key>.headers.<Name>` or `routes[i].representations[j].headers.<Name>`: a name that is not an RFC 9110 token; `Content-Type`, `Content-Length`, `Transfer-Encoding` or `Location` in any case, which the encoding and `redirect` write; a name declared again ignoring case; a literal that is not a field value (visible US-ASCII, spaces and tabs, no space or tab at either end); a source outside `input.<field>`, `response.<field>`, `error.<field>` and `error.wire`, naming no field, naming one of a type other than a scalar or an `Optional` scalar, or one the outcome cannot supply (`response.` without `returns: true`, `error.` without an error), or `error.wire` where the error declares a field named `wire`; any source in a standard refusal's header; any source or challenge in a representation's header. At `….challenge.params.<param>`: the same source refusals for a sourced challenge param |
/// | `ESS-HTTP-027` | at `….headers.<Name>`: a challenge under any header but `WWW-Authenticate`; at `….challenge.scheme`: a scheme that is not a token; at `….challenge.params.<param>`: a param name that is not a token, one declared again ignoring case, or a literal a quoted string cannot carry; at `….headers`: an outcome or standard refusal on `401` without a `WWW-Authenticate` challenge (one refused for its scheme or params counts as declared) |
/// | `ESS-HTTP-028` | at `routes[i].outcomes.<name>.redirect` or `routes[i].errors.redirect` (once for each outcome `errors` covers): a `redirect` on a compiled status other than `301`, `302`, `303`, `307` and `308`, or one of those without a `redirect`; at `….redirect.location`: a source the outcome cannot supply (as for `ESS-HTTP-026`), or of a type other than `String` or a newtype of `String`, `Optional` included; at `….redirect.mode`: none, or one outside `query` and `fragment`; at `….redirect.parameters.<name>`: a parameter written twice, or a source refused as a header source is |
/// | `ESS-HTTP-029` | at `refusals.<key>`, or a route's own `routes[i].refusals.<key>`: a key outside `malformed`, `not_granted`, `too_large`, `unauthenticated` and `unfinished`, or one written twice; at `….<key>.status`: a status outside its class, `4xx` for `malformed`, `not_granted`, `too_large` and `unauthenticated` (a `401` carries a `WWW-Authenticate` challenge, `ESS-HTTP-027`), `5xx` for `unfinished`; at `….<key>.body`: a body other than `none` or `{<member>: {literal: <text>}}` under a non-empty member |
/// | `ESS-HTTP-030` | at `routes[i]`: `body: single` without `empty`, `empty` without `single` (on a command too), or an `empty` status outside `4xx`, unless the route answers `rows` or `array`, where `empty` is refused once, for itself; at `routes[i].empty.body`: a body other than `none` or `{<member>: {literal: <text>}}` under a non-empty member |
/// | `ESS-HTTP-031` | at `routes[i].body`: `body: single` on a view whose filter does not select by its source entity's identity: neither the filter nor a member of a top-level `all` (nested `all` included) is `<identity field> == param.<name>`, compared by value, in either operand order; naming the view, its entity and the identity field |
/// | `ESS-HTTP-032` | at `routes[i].operation_id`: an id outside `[A-Za-z_][A-Za-z0-9_.-]*`; one an earlier route already gives, naming both routes; or the qualified name of another command or view of the specification, which is that operation's derived `operationId` |
/// | `ESS-HTTP-033` | at `servers.<component>`: a component the specification does not declare, one not `reached_by: network`, one written twice, or an empty list; at `servers.<component>[j].url`: a URL that is neither absolute `http://` or `https://` with a host nor `/`-rooted, one holding whitespace, `{`, `}`, `?`, `#` or userinfo, or one given twice for the component |
/// | `ESS-HTTP-034` | at `limits.request_bytes` or `routes[i].limits.request_bytes`: a request size outside 1 to 16777216 bytes; at `routes[i].limits`: a limit on a route that reads no request body (a view, a `get` command or a `request: {encoding: query}` command) |
/// | `ESS-HTTP-035` | at `routes[i].representations`: `representations` on a command route, on a `single` view or on a paged view, or an empty list; at `routes[i].representations[j].query`: a `query` holding other than exactly one parameter, a parameter other than the first representation's, a value an earlier representation has, a parameter name that is a view parameter's wire name or a path variable or is not RFC 3986 `unreserved` characters, or a value that is empty or not `unreserved` characters; at `routes[i].representations[j].encoding.csv.separator`: a separator other than `","`, `";"` and `"\t"`; at `routes[i].representations[j].encoding`: a view field whose type has no flat spelling (a struct, a union, a `List`, a `Map`, `Json`), naming the field (beyond10x/ess#753) |
/// | `ESS-HTTP-036` | at `routes[i].request.parts`: `parts` beside an encoding other than `multipart`; at `routes[i].request.parts.<field>`: a key naming no input field, or one whose type is not `Bytes` (a newtype of it, or an `Optional` of either), or a part stating nothing; at `routes[i].request.parts.<field>.filename` or `….media_type`: a source that is not `input.<field>` naming a `String` input field (a newtype of it, or an `Optional` of either), or one the path carries, that is a part, or that another attribute reads; at `routes[i].request.parts.<field>.media_types`: an empty list, an entry that is not an RFC 6838 `type/subtype`, or one entry twice (beyond10x/ess#754) |
/// | `ESS-HTTP-037` | at `authentication`: `authentication` over a specification that declares no actor; at `authentication.schemes`: no scheme declared; at `authentication.schemes.<name>`: a name that is not an RFC 9110 token, one declared twice, or one no route requires (its own `authentication` or, where it states none, `authentication.required`); at `….<name>.kind`: a kind outside `bearer`, `basic`, `api_key` and `mutual_tls` (`oauth2` and `open_id_connect` named as not yet admitted); at `….<name>.in`, `….name` or `….realm`: a key the kind does not take, one it requires missing (`realm` for `basic`, `in` and `name` for `api_key`), an `in` outside `header`, `query` and `cookie`, a header or cookie name that is not a token, a query name that is not RFC 3986 `unreserved` characters, or a realm no quoted string carries; at `authentication.required` or `routes[i].authentication`: a scheme no `authentication.schemes` declares, or one listed twice; at `routes[i].authentication`: a route requiring only schemes without a challenge (`api_key`, `mutual_tls`) and no `refusals.unauthenticated`, its own or the binding's (beyond10x/ess#681) |
/// | `ESS-HTTP-038` | at `cors.credentials` or `routes[i].cors.credentials`: `credentials: true` beside `origins: any`; at `….origins`: a value that is neither `any` nor a list, or an empty list; at `….origins[j]`: an entry that is not a string, an origin that is not `scheme://host[:port]` (a path, a trailing `/` included, a query, a fragment, userinfo or a `*` in it, no host, or a port outside `0..=65535`), or one an earlier entry gives, ignoring case; at `….headers[j]` or `….expose[j]`: a name that is not an RFC 9110 token, or one given twice ignoring case; at `….expose[j]`: a name no route the policy covers declares (the binding's `cors` covers every route without its own, a route's own covers that route); at `….max_age`: a value outside `0..=86400`; at `routes[i].cors`: a value other than `none` or a map, or a map that is no policy |
///
/// A listed outcome, a `single` view and a standard refusal are written into the IR with every
/// default omitted: an outcome's `envelope`, a view's `rows`, an empty answer's `none` body, an
/// empty `outcomes`, `refusals` (the binding's or a route's), `headers`, challenge `params` or
/// redirect `parameters`. Header, param and parameter maps are
/// written in name order, each name as authored.
///
/// A route without a path is open, and valid: the artifacts that need a path refuse it. It takes
/// no part in collision checks, because no path is claimed.
///
/// `compile` supplies no refined binding, so a document with `refines:` is refused
/// (`ESS-HTTP-003`); [`compile_refining`] takes the refined binding and adds `ESS-HTTP-003` and
/// `ESS-HTTP-012`.
pub fn compile(
    spec: &HttpSpec,
    ir: &EssIr,
    derived: &[DerivedRoute],
) -> Result<HttpIr, HttpDiagnostics> {
    compile_refining(spec, None, ir, derived)
}

/// [`compile`], with the binding `spec` refines, read from its file's bytes ([`Refined`]).
///
/// A **server binding** names a **protocol binding** in `refines:` and fills in its open paths
/// (`docs/design/http-presentation-binding.md`, "Two levels in one format"). Beside every check
/// [`compile`] runs on `spec`:
///
/// | code | refused |
/// |---|---|
/// | `ESS-HTTP-003` | at `refines`: a value that is not `sha256:` and 64 lower-case hex digits; `refines` given and no refined binding supplied; the supplied binding's byte digest differs; the refined binding itself refines (a refinement of a refinement is refused in `/1`); or a refined binding supplied to a document that refines nothing |
/// | `ESS-HTTP-012` | at `limits` or `routes[i].limits`: a request limit added, dropped or restated; at `routes[i].representations`: a refined route's representations added, dropped or changed, a default written out or left out counting as that default; at `routes[i].method` or `routes[i].path`: a method or path the refined binding fixed is changed, or a fixed path left open; at `routes[i].request.encoding`, `routes[i].request.unknown` or `routes[i].request.body`: a refined route's request restated with another value, a default written out or left out counting as that default, and a body added or dropped; at `routes[i].request.parts`: a refined route's `parts` added, dropped or restated with another attribute; at `routes[i].request.query`: a refined route's `query` added, dropped or restated with other fields or another order; at `routes[i].request.members`: a refined route's `members` added, dropped or restated with another path; at `routes[i].select`: a refined route's `select` added, dropped or written otherwise; at `routes[i].lists`: a refined route's `lists` added, dropped or restated with another name or form, the order written not counting; at `routes[i].outcomes.<name>`, `routes[i].errors`, `routes[i].body` or `routes[i].empty`: a refined route's outcome, `errors` (its headers and redirect included), view body or empty answer (its body included, none written out or left out counting as none) added, dropped or restated with another status or body, by the same counting; at `routes[i].outcomes.<name>.headers` or `….redirect`: an outcome both list whose headers or redirect differ, compared in name order with an empty map counting as none; at `refusals.<key>`: a standard refusal added, dropped or changed; at `refusals.<key>.headers`: one both declare whose headers differ; at `routes[i].refusals.<key>` and `routes[i].refusals.<key>.headers`: the same for a route's own refusals; at `routes`: a route of the refined binding is dropped; at `routes[i].operation_id`: an operation id the refined route fixes changed or left out (one may be added where the refined route has none); at `routes[i].outcomes.<name>`: an outcome both list restated with another `encoding`, `json` written out or left out counting as `json`; at `routes[i].alike`: the groups added, dropped or restated, each compared in name order; at `routes[i].authentication`: the schemes a refined route requires (its own or `authentication.required`) changed, in order, each with its declaration; at `cors` or `cors.origins`, and at `routes[i].cors` or `routes[i].cors.origins` where either binding gives the route its own: a policy where the refined binding has none, none where it has one, an origin it does not list (`any` over a list included), or `credentials`, `headers`, `expose` or `max_age` restated otherwise |
///
/// `servers` is not compared: each server binding states its own deployment.
///
/// The refined binding is checked against the same specification and derived routes, and its own
/// refusals are reported with their path under `refined.` (`refined.routes[0].method`). It is
/// compared route by route only when its digest is the one pinned. Adding a route for an
/// operation the refined binding does not name is admitted, and so is filling in, or leaving, an
/// open path. The compiled IR records the pinned digest.
pub fn compile_refining(
    spec: &HttpSpec,
    refined: Option<&Refined>,
    ir: &EssIr,
    derived: &[DerivedRoute],
) -> Result<HttpIr, HttpDiagnostics> {
    let mut refusals = Refusals(Vec::new());
    let standard = envelope(spec, ir, &mut refusals);
    let limits = limits(spec.limits.as_ref(), "limits", &mut refusals);
    let cors = spec
        .cors
        .as_ref()
        .and_then(|written| self::cors(written, "cors", &mut refusals));
    let compared = refinement(spec, refined, ir, derived, &mut refusals);
    let (routes, derived_routes) = body(spec, ir, derived, &standard, &mut refusals);
    for (at, why) in unexposed(
        cors.as_ref(),
        &standard,
        routes.iter().map(|route| (declared_at(spec, route), route)),
    ) {
        refusals.refuse(CORS_CODE, at, why);
    }
    let servers = presentation(spec, ir, &mut refusals);
    let authentication = authentication(spec, ir, &mut refusals);
    if let Some(refined) = compared {
        changes(spec, refined, &mut refusals);
        operation_id_changes(spec, refined, &mut refusals);
        authentication_changes(spec, refined, &mut refusals);
        cors_changes(spec, refined, &mut refusals);
    }
    if refusals.0.is_empty() {
        Ok(HttpIr {
            format: HTTP_IR_FORMAT.to_owned(),
            specification: spec.specification.clone(),
            refines: spec.refines.clone(),
            limits,
            refusals: standard,
            servers,
            authentication,
            cors,
            routes,
            derived_routes,
        })
    } else {
        Err(HttpDiagnostics(refusals.0))
    }
}

/// `type` (`ESS-HTTP-001`), `specification` (`ESS-HTTP-002`) and `refusals` (`ESS-HTTP-029`),
/// returning the standard refusals compiled.
fn envelope(spec: &HttpSpec, ir: &EssIr, refusals: &mut Refusals) -> BTreeMap<RefusalKey, Refusal> {
    if spec.format != HTTP_FORMAT {
        refusals.refuse(
            "ESS-HTTP-001",
            "type",
            format!("expected `{HTTP_FORMAT}`, found `{}`", spec.format),
        );
    }
    specification(&spec.specification, ir, refusals);
    standard(spec.refusals.as_ref(), "refusals", refusals)
}

/// The standard refusals one route answers its own way, each with the path it is declared at: the
/// route's own (`routes[i].refusals.<key>`) over the binding's (`refusals.<key>`).
type Standing = BTreeMap<RefusalKey, (Refusal, String)>;

/// The refusals that stand on the route at `path`: `own` over the binding's `standard`.
fn standing(
    standard: &BTreeMap<RefusalKey, Refusal>,
    own: &BTreeMap<RefusalKey, Refusal>,
    path: &str,
) -> Standing {
    let mut standing: Standing = standard
        .iter()
        .map(|(key, refusal)| (*key, (refusal.clone(), format!("refusals.{key}"))))
        .collect();
    for (key, refusal) in own {
        standing.insert(*key, (refusal.clone(), format!("{path}.refusals.{key}")));
    }
    standing
}

/// Holds a written `limits` to its range (`ESS-HTTP-034`), at `{at}.request_bytes`; `None` when
/// refused or not written.
fn limits(written: Option<&LimitsSpec>, at: &str, refusals: &mut Refusals) -> Option<Limits> {
    let written = written?;
    let Some(request_bytes) = u64::try_from(written.request_bytes)
        .ok()
        .filter(|bytes| (1..=MAX_REQUEST_BYTES).contains(bytes))
    else {
        refusals.refuse(
            "ESS-HTTP-034",
            format!("{at}.request_bytes"),
            format!(
                "`{}` is not a request size: `limits.request_bytes` is a whole number of bytes \
                 from 1 to {MAX_REQUEST_BYTES}",
                written.request_bytes
            ),
        );
        return None;
    };
    Some(Limits { request_bytes })
}

/// The code of a `cors` policy outside its rules.
const CORS_CODE: &str = "ESS-HTTP-038";

/// `routes[i]`, `i` the index in `spec` of the route that compiled to `route`.
fn declared_at(spec: &HttpSpec, route: &Route) -> String {
    let index = spec
        .routes
        .iter()
        .position(|written| named(written).as_ref() == Some(&route.operation))
        .unwrap_or_default();
    format!("routes[{index}]")
}

/// The policy `written` states, if it can be read as one: `origins` `any` or a list of strings,
/// and `max_age` in range. Nothing else is checked.
fn written_cors(written: &CorsSpec) -> Option<Cors> {
    let origins = match &written.origins {
        serde_json::Value::String(word) if word == "any" => Origins::Any,
        value => Origins::Listed(serde_json::from_value(value.clone()).ok()?),
    };
    let max_age = match written.max_age {
        Some(age) => Some(u32::try_from(age).ok().filter(|age| *age <= MAX_CORS_AGE)?),
        None => None,
    };
    Some(Cors {
        origins,
        credentials: written.credentials.unwrap_or(false),
        headers: written.headers.clone().unwrap_or_default(),
        expose: written.expose.clone().unwrap_or_default(),
        max_age,
    })
}

/// Holds a written policy at `at` to the rules that need no route (`ESS-HTTP-038`); `None` when
/// refused. Which names `expose` may list is checked against the routes ([`unexposed`]).
fn cors(written: &CorsSpec, at: &str, refusals: &mut Refusals) -> Option<Cors> {
    let mut refused = false;
    let origins_read = match &written.origins {
        serde_json::Value::String(word) if word == "any" => true,
        serde_json::Value::Array(items) => {
            for (index, item) in items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.is_string())
            {
                refusals.refuse(
                    CORS_CODE,
                    format!("{at}.origins[{index}]"),
                    format!(
                        "`{item}` is not an origin: an origin is a string `scheme://host[:port]`"
                    ),
                );
                refused = true;
            }
            !refused
        }
        other => {
            refusals.refuse(
                CORS_CODE,
                format!("{at}.origins"),
                format!("`{other}` is not `any` or a list of origins"),
            );
            refused = true;
            false
        }
    };
    if let Some(age) = written
        .max_age
        .filter(|age| u32::try_from(*age).map_or(true, |age| age > MAX_CORS_AGE))
    {
        refusals.refuse(CORS_CODE, format!("{at}.max_age"), max_age_reason(age));
        refused = true;
    }
    let checked = Cors {
        origins: if origins_read {
            match &written.origins {
                serde_json::Value::Array(items) => Origins::Listed(
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_owned))
                        .collect(),
                ),
                _ => Origins::Any,
            }
        } else {
            Origins::Listed(Vec::new())
        },
        credentials: written.credentials.unwrap_or(false),
        headers: written.headers.clone().unwrap_or_default(),
        expose: written.expose.clone().unwrap_or_default(),
        max_age: written
            .max_age
            .and_then(|age| u32::try_from(age).ok())
            .filter(|age| *age <= MAX_CORS_AGE),
    };
    for (key, why) in cors_problems(&checked) {
        if !origins_read && key == "origins" {
            continue;
        }
        refusals.refuse(CORS_CODE, format!("{at}.{key}"), why);
        refused = true;
    }
    (!refused).then_some(checked)
}

/// Holds a route's written `cors` at `{path}.cors`: `none`, or a policy held as [`cors`] holds the
/// binding's (`ESS-HTTP-038`); `None` when absent or refused.
fn route_cors(spec: &RouteSpec, path: &str, refusals: &mut Refusals) -> Option<RouteCors> {
    let at = format!("{path}.cors");
    match spec.cors.as_ref()? {
        serde_json::Value::String(word) if word == "none" => Some(RouteCors::None),
        value @ serde_json::Value::Object(_) => {
            match serde_json::from_value::<CorsSpec>(value.clone()) {
                Ok(written) => cors(&written, &at, refusals).map(RouteCors::Own),
                Err(error) => {
                    refusals.refuse(
                        CORS_CODE,
                        at,
                        format!("a route's `cors` policy is not one: {error}"),
                    );
                    None
                }
            }
        }
        other => {
            refusals.refuse(
                CORS_CODE,
                at,
                format!(
                    "`{other}` is not a route's `cors`: it is `none`, or a policy in place of \
                     the binding's"
                ),
            );
            None
        }
    }
}

/// What may read a route's answers, as the written bindings state it, for comparing a refinement.
enum Reads {
    /// No cross-origin read.
    Closed,
    /// This policy.
    Open(Cors),
    /// A policy that is no policy, refused where it is written.
    Unread,
}

impl Reads {
    fn of(written: Option<&CorsSpec>) -> Self {
        match written {
            None => Self::Closed,
            Some(written) => written_cors(written).map_or(Self::Unread, Self::Open),
        }
    }

    /// What stands on `route` of `binding`: its own `cors`, else the binding's.
    fn on(binding: &HttpSpec, route: &RouteSpec) -> Self {
        match &route.cors {
            None => Self::of(binding.cors.as_ref()),
            Some(serde_json::Value::String(word)) if word == "none" => Self::Closed,
            Some(value) => serde_json::from_value::<CorsSpec>(value.clone())
                .ok()
                .and_then(|written| written_cors(&written))
                .map_or(Self::Unread, Self::Open),
        }
    }

    /// Why `self` is not the refined binding's `old` or a narrowing of it, and whether the
    /// origins are why: a refinement lists the same origins or fewer and restates every other
    /// key.
    fn widens(&self, old: &Self) -> Option<(bool, String)> {
        match (self, old) {
            (Self::Unread, _) | (_, Self::Unread) | (Self::Closed, Self::Closed) => None,
            (Self::Open(_), Self::Closed) => Some((
                false,
                "a policy admits cross-origin reads here, and the refined binding admits none; a \
                 refinement may narrow the origins and may not widen them"
                    .to_owned(),
            )),
            (Self::Closed, Self::Open(_)) => Some((
                false,
                "no policy stands here, and the refined binding states one; a refinement \
                 restates `cors`, with the same origins or fewer"
                    .to_owned(),
            )),
            (Self::Open(new), Self::Open(old)) => {
                if !old.origins.covers(&new.origins) {
                    Some((
                        true,
                        "the origins admit one the refined binding does not; a refinement may \
                         narrow the origins and may not widen them"
                            .to_owned(),
                    ))
                } else if (new.credentials, &new.headers, &new.expose, new.max_age)
                    != (old.credentials, &old.headers, &old.expose, old.max_age)
                {
                    Some((
                        false,
                        "the policy is restated otherwise than the refined binding's: a \
                         refinement restates `credentials`, `headers`, `expose` and `max_age` \
                         exactly"
                            .to_owned(),
                    ))
                } else {
                    None
                }
            }
        }
    }
}

/// Refuses (`ESS-HTTP-012`) a refinement whose policies widen the refined binding's or restate
/// them otherwise: the binding's `cors` at `cors`, and, for each route either binding gives a
/// policy of its own, the policy that stands on the route at `routes[i].cors` (`.origins` when the
/// origins widen).
fn cors_changes(spec: &HttpSpec, refined: &HttpSpec, refusals: &mut Refusals) {
    let refuse = |at: String, (origins, why): (bool, String), refusals: &mut Refusals| {
        let at = if origins { format!("{at}.origins") } else { at };
        refusals.refuse("ESS-HTTP-012", at, why);
    };
    if let Some(widened) = Reads::of(spec.cors.as_ref()).widens(&Reads::of(refined.cors.as_ref())) {
        refuse("cors".to_owned(), widened, refusals);
    }
    for (index, route) in spec.routes.iter().enumerate() {
        let Some(operation) = named(route) else {
            continue;
        };
        let Some(old) = refined
            .routes
            .iter()
            .find(|old| named(old).as_ref() == Some(&operation))
        else {
            continue;
        };
        if route.cors.is_none() && old.cors.is_none() {
            continue;
        }
        if let Some(widened) = Reads::on(spec, route).widens(&Reads::on(refined, old)) {
            refuse(format!("routes[{index}].cors"), widened, refusals);
        }
    }
}

/// Whether an operation reads a request body: a command whose method is not `get` and whose
/// request is not encoded `query`.
fn reads_body(operation: &Operation, method: Method, encoding: Option<Encoding>) -> bool {
    matches!(operation, Operation::Command(_))
        && method != Method::Get
        && encoding != Some(Encoding::Query)
}

/// Holds a route's `limits` to its range and to the kind of route it is on (`ESS-HTTP-034`): a
/// command that reads a request body takes it.
fn route_limits(
    spec: &RouteSpec,
    operation: &Operation,
    method: Option<Method>,
    request: &Result<Option<Request>, Refused>,
    path: &str,
    refusals: &mut Refusals,
) -> Option<Limits> {
    let compiled = limits(spec.limits.as_ref(), &format!("{path}.limits"), refusals);
    let (Some(_), Some(method)) = (&spec.limits, method) else {
        return compiled;
    };
    let encoding = request
        .as_ref()
        .ok()
        .and_then(|request| request.as_ref())
        .map(|request| request.encoding);
    if !reads_body(operation, method, encoding) {
        let why = if matches!(operation, Operation::View(_)) {
            "a view takes no request"
        } else if method == Method::Get {
            "`get` carries none"
        } else {
            "`request: {encoding: query}` carries the input in the query string"
        };
        refusals.refuse(
            "ESS-HTTP-034",
            format!("{path}.limits"),
            format!(
                "{operation} reads no request body ({why}); `limits.request_bytes` belongs on a \
                 command route that carries its input in a `json` or `form` body"
            ),
        );
    }
    compiled
}

/// Refuses (`ESS-HTTP-012`) a `limits` that differs from the refined binding's, at `at`.
fn limit_changes(
    (new, old): (Option<&LimitsSpec>, Option<&LimitsSpec>),
    at: String,
    message: impl FnOnce(&str, &str) -> String,
    refusals: &mut Refusals,
) {
    if new == old {
        return;
    }
    let shown = |limits: Option<&LimitsSpec>| {
        limits.map_or_else(
            || "no limit".to_owned(),
            |limits| format!("`request_bytes: {}`", limits.request_bytes),
        )
    };
    refusals.refuse("ESS-HTTP-012", at, message(&shown(new), &shown(old)));
}

/// Holds each `refusals:` entry under `prefix` (`refusals`, or a route's `routes[i].refusals`) to
/// its rules (`ESS-HTTP-029`): a key of the set, written once, a status of the key's class and a
/// body that is `none` or one literal member with a name.
/// Refuses (`ESS-HTTP-012`) a refining document whose top-level `limits` differ from the refined
/// binding's.
fn binding_limit_changes(spec: &HttpSpec, refined: &HttpSpec, refusals: &mut Refusals) {
    limit_changes(
        (spec.limits.as_ref(), refined.limits.as_ref()),
        "limits".to_owned(),
        |new, old| {
            format!(
                "`limits` is {new} here, and the refined binding fixes {old}; a refinement \
                 restates the refined binding's limits exactly"
            )
        },
        refusals,
    );
}

fn standard(
    written: Option<&Entries<RefusalSpec>>,
    prefix: &str,
    refusals: &mut Refusals,
) -> BTreeMap<RefusalKey, Refusal> {
    let mut compiled = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for (name, spec) in written.map_or(&[][..], |entries| &entries.0[..]) {
        let at = format!("{prefix}.{name}");
        if !seen.insert(name.as_str()) {
            refusals.refuse(
                "ESS-HTTP-029",
                at,
                format!("`{name}` is written twice; each standard refusal is overridden once"),
            );
            continue;
        }
        let Some(key) = RefusalKey::parse(name) else {
            refusals.refuse(
                "ESS-HTTP-029",
                at,
                format!(
                    "`{name}` is not a standard refusal; `refusals:` takes `malformed`, \
                     `not_granted`, `too_large`, `unauthenticated` and `unfinished`"
                ),
            );
            continue;
        };
        let class = i64::from(key.class());
        let status = u16::try_from(spec.status)
            .ok()
            .filter(|_| spec.status / 100 == class);
        if status.is_none() {
            refusals.refuse(
                "ESS-HTTP-029",
                format!("{at}.status"),
                format!(
                    "`{}` is not a `{class}xx` status, and `{key}` answers a `{class}xx`: \
                     `malformed`, `not_granted`, `too_large` and `unauthenticated` refuse the \
                     request (`4xx`), `unfinished` is the server's (`5xx`)",
                    spec.status
                ),
            );
        }
        let body = match &spec.body {
            BodySpec::Word(word) if word == "none" => Some(RefusalBody::None),
            BodySpec::Literal { member, text } if !member.is_empty() => {
                Some(RefusalBody::Literal {
                    member: member.clone(),
                    text: text.clone(),
                })
            }
            written => {
                refusals.refuse(
                    "ESS-HTTP-029",
                    format!("{at}.body"),
                    format!(
                        "`{written}` is not a standard refusal's body: it is `none`, or \
                         `{{<member>: {{literal: <text>}}}}` under a member with a name"
                    ),
                );
                None
            }
        };
        let headers = answer_headers(spec.headers.as_ref(), None, status, &at, refusals);
        if let (Some(status), Some(body), Ok(headers)) = (status, body, headers) {
            compiled.insert(
                key,
                Refusal {
                    status,
                    body,
                    headers,
                },
            );
        }
    }
    compiled
}

/// Every route checked and resolved, ordered by path, then method, then operation, with the
/// derived routes that stay.
fn body(
    spec: &HttpSpec,
    ir: &EssIr,
    derived: &[DerivedRoute],
    standard: &BTreeMap<RefusalKey, Refusal>,
    refusals: &mut Refusals,
) -> (Vec<Route>, Vec<DerivedRoute>) {
    let resolved = routes(&spec.routes, ir, standard, refusals);
    let derived_routes = surface(&resolved, derived, refusals);
    let mut routes: Vec<Route> = resolved
        .into_iter()
        .filter_map(|bound| bound.route)
        .collect();
    routes.sort_by(|left, right| {
        (&left.path, left.method, &left.operation).cmp(&(
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    (routes, derived_routes)
}

/// Holds `refines` to the refined binding supplied (`ESS-HTTP-003`), and checks that binding
/// against the same specification, reporting its refusals under `refined.`. Returns the refined
/// document to compare route by route when its digest is the one pinned.
fn refinement<'a>(
    spec: &HttpSpec,
    refined: Option<&'a Refined>,
    ir: &EssIr,
    derived: &[DerivedRoute],
    refusals: &mut Refusals,
) -> Option<&'a HttpSpec> {
    let pin = match (spec.refines.as_deref(), refined) {
        (None, None) => return None,
        (None, Some(refined)) => {
            refusals.refuse(
                "ESS-HTTP-003",
                "refines",
                format!(
                    "a refined binding (`{}`) was supplied, and this document refines nothing; a \
                     document names the binding it refines in `refines:`",
                    refined.digest
                ),
            );
            return None;
        }
        (Some(pin), _) if !is_digest(pin) => {
            refusals.refuse(
                "ESS-HTTP-003",
                "refines",
                format!(
                    "`{pin}` is not a digest: `refines` is `sha256:` and 64 lower-case hex digits \
                     of the refined binding's bytes"
                ),
            );
            return None;
        }
        (Some(pin), None) => {
            refusals.refuse(
                "ESS-HTTP-003",
                "refines",
                format!("refines `{pin}`, and no refined binding was supplied to check it against"),
            );
            return None;
        }
        (Some(pin), Some(_)) => pin,
    };
    let refined = refined?;
    if refined.digest != pin {
        refusals.refuse(
            "ESS-HTTP-003",
            "refines",
            format!(
                "refines `{pin}`, and the refined binding supplied is `{}`: `refines` pins the \
                 bytes of the refined file, so a file edited or reformatted since is another \
                 binding",
                refined.digest
            ),
        );
        return None;
    }
    if let Some(further) = &refined.spec.refines {
        refusals.refuse(
            "ESS-HTTP-003",
            "refines",
            format!(
                "the refined binding `{pin}` itself refines `{further}`; in `{HTTP_FORMAT}` a \
                 binding refines one that refines nothing"
            ),
        );
    }
    let mut own = Refusals(Vec::new());
    let standard = envelope(&refined.spec, ir, &mut own);
    limits(refined.spec.limits.as_ref(), "limits", &mut own);
    body(&refined.spec, ir, derived, &standard, &mut own);
    presentation(&refined.spec, ir, &mut own);
    authentication(&refined.spec, ir, &mut own);
    for diagnostic in own.0 {
        refusals.refuse(
            diagnostic.code,
            format!("refined.{}", diagnostic.path),
            diagnostic.message,
        );
    }
    Some(&refined.spec)
}

/// The operation a route names, when it names exactly one.
fn named(route: &RouteSpec) -> Option<Operation> {
    match (&route.command, &route.view) {
        (Some(name), None) => Some(Operation::Command(name.clone())),
        (None, Some(name)) => Some(Operation::View(name.clone())),
        _ => None,
    }
}

/// A route's operation, as [`Operation`]'s display writes it, or `a route` when it names none.
fn described(route: &RouteSpec) -> String {
    named(route).map_or_else(|| "a route".to_owned(), |operation| operation.to_string())
}

/// Holds each route's `operation_id` to its rules (`ESS-HTTP-032`) and `servers` to theirs
/// (`ESS-HTTP-033`), returning the servers compiled, by component.
fn presentation(
    spec: &HttpSpec,
    ir: &EssIr,
    refusals: &mut Refusals,
) -> BTreeMap<String, Vec<Server>> {
    operation_ids(spec, ir, refusals);
    servers(spec.servers.as_ref(), ir, refusals)
}

/// Holds `authentication` and each route's `authentication` to their rules (`ESS-HTTP-037`),
/// returning them compiled; `None` when the binding declares none or any of it is refused.
fn authentication(spec: &HttpSpec, ir: &EssIr, refusals: &mut Refusals) -> Option<Authentication> {
    let before = refusals.0.len();
    let written = spec.authentication.as_ref();
    let (declared, schemes) = declared_schemes(written, ir, refusals);
    routes_authentication(spec, &declared, &schemes, refusals);
    let written = written?;
    (refusals.0.len() == before).then(|| Authentication {
        schemes,
        required: written.required.clone().unwrap_or_default(),
    })
}

/// Every scheme name written, first occurrences only, and the schemes that compiled, refusing
/// `authentication` over a model without an actor, an empty `schemes`, a name twice or not a
/// token, and a scheme [`scheme`] refuses (`ESS-HTTP-037`).
fn declared_schemes<'a>(
    written: Option<&'a AuthenticationSpec>,
    ir: &EssIr,
    refusals: &mut Refusals,
) -> (Vec<&'a str>, BTreeMap<String, Scheme>) {
    let mut declared: Vec<&str> = Vec::new();
    let mut schemes: BTreeMap<String, Scheme> = BTreeMap::new();
    if let Some(written) = written {
        if ir.actors().is_empty() {
            refusals.refuse(
                "ESS-HTTP-037",
                "authentication",
                "the specification declares no actor, so it tells no caller apart by who it is; \
                 `authentication` binds a model whose actors `may:` call its operations",
            );
        }
        if written.schemes.0.is_empty() {
            refusals.refuse(
                "ESS-HTTP-037",
                "authentication.schemes",
                "`schemes` declares no scheme; `authentication` names at least one, as \
                 `<name>: {kind: bearer}`",
            );
        }
        for (name, scheme) in &written.schemes.0 {
            let at = format!("authentication.schemes.{name}");
            if declared.contains(&name.as_str()) {
                refusals.refuse(
                    "ESS-HTTP-037",
                    at,
                    format!("scheme `{name}` is declared twice; each scheme is declared once"),
                );
                continue;
            }
            declared.push(name);
            if !is_token(name) {
                refusals.refuse(
                    "ESS-HTTP-037",
                    at,
                    format!(
                        "{name:?} is not a scheme name: a scheme name is an RFC 9110 token, one \
                         or more letters, digits and ``!#$%&'*+-.^_`|~``"
                    ),
                );
                continue;
            }
            if let Some(compiled) = self::scheme(scheme, &at, refusals) {
                schemes.insert(name.clone(), compiled);
            }
        }
    }
    (declared, schemes)
}

/// Holds `authentication.required` and each route's list to the schemes `declared`, each named
/// once; a route whose schemes have no challenge to its `refusals.unauthenticated`; and every
/// declared scheme to a route that requires it (`ESS-HTTP-037`).
fn routes_authentication(
    spec: &HttpSpec,
    declared: &[&str],
    schemes: &BTreeMap<String, Scheme>,
    refusals: &mut Refusals,
) {
    let listed = |list: &[String], at: &str, refusals: &mut Refusals| -> bool {
        let mut fine = true;
        for (index, name) in list.iter().enumerate() {
            if list[..index].contains(name) {
                refusals.refuse(
                    "ESS-HTTP-037",
                    at,
                    format!("`{at}` names `{name}` twice; each scheme is listed once"),
                );
                fine = false;
            } else if !declared.contains(&name.as_str()) {
                refusals.refuse(
                    "ESS-HTTP-037",
                    at,
                    format!(
                        "`{at}` names `{name}`, which `authentication.schemes` does not declare"
                    ),
                );
                fine = false;
            }
        }
        fine
    };
    let required: &[String] = spec
        .authentication
        .as_ref()
        .and_then(|written| written.required.as_deref())
        .unwrap_or(&[]);
    let required_fine = listed(required, "authentication.required", refusals);
    let answers_unauthenticated = |written: Option<&Entries<RefusalSpec>>| {
        written.is_some_and(|entries| {
            entries
                .0
                .iter()
                .any(|(key, _)| key == RefusalKey::Unauthenticated.as_str())
        })
    };
    let mut used: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for (index, route) in spec.routes.iter().enumerate() {
        let at = format!("routes[{index}].authentication");
        let (list, fine) = match &route.authentication {
            Some(own) => (own.as_slice(), listed(own, &at, refusals)),
            None => (required, required_fine),
        };
        used.extend(list.iter().map(String::as_str));
        let compiled = list.iter().all(|name| schemes.contains_key(name));
        if fine
            && compiled
            && !list.is_empty()
            && !list.iter().any(|name| schemes[name].challenges())
            && !answers_unauthenticated(route.refusals.as_ref())
            && !answers_unauthenticated(spec.refusals.as_ref())
        {
            refusals.refuse(
                "ESS-HTTP-037",
                at,
                format!(
                    "{} requires {}, and no scheme of them has a challenge a `401` could carry \
                     (`api_key` and `mutual_tls` have none); state the answer to a call without \
                     the credential as `refusals.unauthenticated`, the route's own or the \
                     binding's",
                    described(route),
                    list.iter()
                        .map(|name| format!("`{name}`"))
                        .collect::<Vec<_>>()
                        .join(" or ")
                ),
            );
        }
    }
    for name in declared {
        if is_token(name) && !used.contains(name) {
            refusals.refuse(
                "ESS-HTTP-037",
                format!("authentication.schemes.{name}"),
                format!(
                    "scheme `{name}` is declared, and no route requires it, neither in its own \
                     `authentication` nor through `authentication.required`"
                ),
            );
        }
    }
}

/// Refuses a `kind` scheme at `at` without the `key` it requires (`ESS-HTTP-037`).
fn missing(kind: &str, key: &str, at: &str, refusals: &mut Refusals) {
    refusals.refuse(
        "ESS-HTTP-037",
        format!("{at}.{key}"),
        format!("a `{kind}` scheme requires `{key}`"),
    );
}

/// An `api_key` scheme compiled, at `at`, or `None` when its `in` or `name` is missing or `in` is
/// outside the set (`ESS-HTTP-037`).
fn api_key(written: &SchemeSpec, at: &str, refusals: &mut Refusals) -> Option<Scheme> {
    let location = match written.location.as_deref() {
        None => {
            missing("api_key", "in", at, refusals);
            None
        }
        Some("header") => Some(KeyLocation::Header),
        Some("query") => Some(KeyLocation::Query),
        Some("cookie") => Some(KeyLocation::Cookie),
        Some(other) => {
            refusals.refuse(
                "ESS-HTTP-037",
                format!("{at}.in"),
                format!(
                    "`{other}` is not where an API key travels: `in` is `header`, `query` or \
                     `cookie`"
                ),
            );
            None
        }
    };
    if written.name.is_none() {
        missing("api_key", "name", at, refusals);
    }
    Some(Scheme::ApiKey {
        location: location?,
        name: written.name.clone()?,
    })
}

/// One scheme compiled, at `at`, or `None` once refused (`ESS-HTTP-037`): a `kind` outside the
/// set, a key the kind does not take, or one it requires missing or out of its rule.
fn scheme(written: &SchemeSpec, at: &str, refusals: &mut Refusals) -> Option<Scheme> {
    let before = refusals.0.len();
    let kind = written.kind.as_str();
    let keys = [
        ("in", written.location.is_some()),
        ("name", written.name.is_some()),
        ("realm", written.realm.is_some()),
    ];
    let takes: &[&str] = match kind {
        "basic" => &["realm"],
        "api_key" => &["in", "name"],
        _ => &[],
    };
    let compiled = match kind {
        "bearer" => Some(Scheme::Bearer),
        "mutual_tls" => Some(Scheme::MutualTls),
        "basic" => match &written.realm {
            None => {
                missing(kind, "realm", at, refusals);
                None
            }
            Some(realm) => Some(Scheme::Basic {
                realm: realm.clone(),
            }),
        },
        "api_key" => api_key(written, at, refusals),
        "oauth2" | "open_id_connect" => {
            refusals.refuse(
                "ESS-HTTP-037",
                format!("{at}.kind"),
                format!(
                    "`{kind}` is not yet admitted: OAuth 2.0 and OpenID Connect wait for a need \
                     that names a flow; `kind` is `bearer`, `basic`, `api_key` or `mutual_tls`"
                ),
            );
            None
        }
        other => {
            refusals.refuse(
                "ESS-HTTP-037",
                format!("{at}.kind"),
                format!(
                    "`{other}` is not a scheme kind: `kind` is `bearer` (RFC 6750), `basic` (RFC \
                     7617), `api_key` or `mutual_tls`"
                ),
            );
            None
        }
    };
    if compiled.is_some() {
        for (key, present) in keys {
            if present && !takes.contains(&key) {
                refusals.refuse(
                    "ESS-HTTP-037",
                    format!("{at}.{key}"),
                    format!("a `{kind}` scheme takes no `{key}`"),
                );
            }
        }
    }
    let compiled = compiled?;
    let name = at.rsplit_once('.').map_or(at, |(_, name)| name);
    if let Some(reason) = scheme_reason(name, &compiled) {
        let key = match compiled {
            Scheme::Basic { .. } => "realm",
            _ => "name",
        };
        refusals.refuse("ESS-HTTP-037", format!("{at}.{key}"), reason);
    }
    (refusals.0.len() == before).then_some(compiled)
}

/// Why a compiled scheme named `name` breaks its rule: a name that is not a token, a `basic` realm
/// no quoted string carries, or an `api_key` name its header or cookie (a token) or its query
/// parameter (RFC 3986 `unreserved` characters) cannot carry.
fn scheme_reason(name: &str, scheme: &Scheme) -> Option<String> {
    if !is_token(name) {
        return Some(format!("{name:?} is not a scheme name, an RFC 9110 token"));
    }
    match scheme {
        Scheme::Basic { realm } if !quotable(realm) => Some(format!(
            "{realm:?} is not a realm a quoted string carries: horizontal tabs, spaces and \
             visible US-ASCII only"
        )),
        Scheme::ApiKey {
            location: KeyLocation::Query,
            name,
        } if !unreserved(name) => Some(format!(
            "{name:?} is not a query parameter name an API key travels in: one or more RFC 3986 \
             `unreserved` characters, letters, digits and `-._~`"
        )),
        Scheme::ApiKey {
            location: KeyLocation::Header | KeyLocation::Cookie,
            name,
        } if !is_token(name) => Some(format!(
            "{name:?} is not a header or cookie name an API key travels in: an RFC 9110 token, \
             one or more letters, digits and ``!#$%&'*+-.^_`|~``"
        )),
        _ => None,
    }
}

/// Refuses a refinement that changes what a refined route requires (`ESS-HTTP-012`), at
/// `routes[i].authentication`: the schemes it requires, its own or through
/// `authentication.required`, compared in order with the declaration of each.
fn authentication_changes(spec: &HttpSpec, refined: &HttpSpec, refusals: &mut Refusals) {
    let effective = |document: &HttpSpec, route: &RouteSpec| -> serde_json::Value {
        let declared = document.authentication.as_ref();
        let list: &[String] = route
            .authentication
            .as_deref()
            .or_else(|| declared.and_then(|written| written.required.as_deref()))
            .unwrap_or(&[]);
        serde_json::Value::Array(
            list.iter()
                .map(|name| {
                    let scheme = declared
                        .and_then(|written| written.schemes.0.iter().find(|(key, _)| key == name))
                        .map(|(_, scheme)| scheme.compared());
                    serde_json::json!([name, scheme])
                })
                .collect(),
        )
    };
    let mut routed: BTreeMap<Operation, usize> = BTreeMap::new();
    for (index, route) in spec.routes.iter().enumerate() {
        if let Some(operation) = named(route) {
            routed.entry(operation).or_insert(index);
        }
    }
    let mut compared = std::collections::BTreeSet::new();
    for (at, fixed) in refined.routes.iter().enumerate() {
        let Some(operation) = named(fixed) else {
            continue;
        };
        if !compared.insert(operation.clone()) {
            continue;
        }
        let Some(&index) = routed.get(&operation) else {
            continue;
        };
        let (here, there) = (
            effective(spec, &spec.routes[index]),
            effective(refined, fixed),
        );
        if here != there {
            refusals.refuse(
                "ESS-HTTP-012",
                format!("routes[{index}].authentication"),
                format!(
                    "{operation} requires `{here}` here, and the refined binding requires \
                     `{there}` at `routes[{at}]`; a refinement restates the authentication of \
                     every route it restates, each scheme as declared"
                ),
            );
        }
    }
}

/// Refuses an `operation_id` outside the grammar, one an earlier route already gives, or one that
/// is the qualified name of another operation, its derived `operationId` (`ESS-HTTP-032`).
fn operation_ids(spec: &HttpSpec, ir: &EssIr, refusals: &mut Refusals) {
    let operations: Vec<Operation> = ir
        .commands()
        .keys()
        .map(|name| Operation::Command(name.to_string()))
        .chain(
            ir.views()
                .keys()
                .map(|name| Operation::View(name.to_string())),
        )
        .collect();
    let mut given: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, route) in spec.routes.iter().enumerate() {
        let Some(id) = route.operation_id.as_deref() else {
            continue;
        };
        let at = format!("routes[{index}].operation_id");
        if !is_operation_id(id) {
            refusals.refuse("ESS-HTTP-032", at, operation_id_reason(id));
            continue;
        }
        if let Some(&first) = given.get(id) {
            refusals.refuse(
                "ESS-HTTP-032",
                at,
                format!(
                    "operation id `{id}` of {} is already the operation id of {} at \
                     `routes[{first}]`; an operation id names one operation",
                    described(route),
                    described(&spec.routes[first])
                ),
            );
            continue;
        }
        given.insert(id, index);
        let own = named(route);
        if let Some(other) = operations
            .iter()
            .find(|operation| operation.name() == id && Some(*operation) != own.as_ref())
        {
            refusals.refuse(
                "ESS-HTTP-032",
                at,
                format!(
                    "operation id `{id}` of {} is the qualified name of {other}, which is that \
                     operation's derived operation id; an operation id names one operation",
                    described(route)
                ),
            );
        }
    }
}

/// Holds each `servers:` entry to its rules (`ESS-HTTP-033`): a `reached_by: network` component
/// of the specification, written once, with a list that is not empty of URLs each admitted and
/// each given once.
fn servers(
    written: Option<&Entries<Vec<ServerSpec>>>,
    ir: &EssIr,
    refusals: &mut Refusals,
) -> BTreeMap<String, Vec<Server>> {
    let mut compiled: BTreeMap<String, Vec<Server>> = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for (name, list) in written
        .map(|entries| entries.0.as_slice())
        .unwrap_or_default()
    {
        let at = format!("servers.{name}");
        if !seen.insert(name.as_str()) {
            refusals.refuse(
                "ESS-HTTP-033",
                at,
                format!("component `{name}` is written twice; its servers are one list"),
            );
            continue;
        }
        let Some(component) = ir
            .components()
            .values()
            .find(|component| component.name.to_string() == *name)
        else {
            refusals.refuse(
                "ESS-HTTP-033",
                at,
                format!("the specification declares no component `{name}`"),
            );
            continue;
        };
        if component.reached_by != Reach::Network {
            refusals.refuse(
                "ESS-HTTP-033",
                at,
                format!(
                    "component `{name}` is not declared `reached_by: network`: its callers are \
                     deployed with it, so no request reaches its paths at a server URL"
                ),
            );
            continue;
        }
        if list.is_empty() {
            refusals.refuse(
                "ESS-HTTP-033",
                at,
                format!(
                    "component `{name}` lists no server; a component without servers is left out"
                ),
            );
            continue;
        }
        for (index, server) in list.iter().enumerate() {
            let key = format!("{at}[{index}].url");
            if let Some(reason) = server_url_reason(&server.url) {
                refusals.refuse("ESS-HTTP-033", key, reason);
            } else if list[..index]
                .iter()
                .any(|earlier| earlier.url == server.url)
            {
                refusals.refuse(
                    "ESS-HTTP-033",
                    key,
                    format!(
                        "`{}` is already a server of component `{name}`; a URL is listed once",
                        server.url
                    ),
                );
            }
        }
        compiled.insert(
            name.clone(),
            list.iter()
                .map(|server| Server {
                    url: server.url.clone(),
                    description: server.description.clone(),
                })
                .collect(),
        );
    }
    compiled
}

/// Refuses a refinement that changes or leaves out an `operation_id` the refined binding fixed
/// (`ESS-HTTP-012`), at `routes[i].operation_id`. Each operation is compared at the first route
/// that names it in each document, as [`changes`] compares the rest; a dropped route is refused
/// there. An id added where the refined route has none is admitted.
fn operation_id_changes(spec: &HttpSpec, refined: &HttpSpec, refusals: &mut Refusals) {
    let mut routed: BTreeMap<Operation, usize> = BTreeMap::new();
    for (index, route) in spec.routes.iter().enumerate() {
        if let Some(operation) = named(route) {
            routed.entry(operation).or_insert(index);
        }
    }
    let mut compared = std::collections::BTreeSet::new();
    for (at, fixed) in refined.routes.iter().enumerate() {
        let Some(operation) = named(fixed) else {
            continue;
        };
        if !compared.insert(operation.clone()) {
            continue;
        }
        let (Some(fixed_id), Some(&index)) =
            (fixed.operation_id.as_deref(), routed.get(&operation))
        else {
            continue;
        };
        let here = spec.routes[index].operation_id.as_deref();
        if here != Some(fixed_id) {
            let found = here.map_or_else(|| "none".to_owned(), |id| format!("`{id}`"));
            refusals.refuse(
                "ESS-HTTP-012",
                format!("routes[{index}].operation_id"),
                format!(
                    "{operation} has operation id {found} here, and the refined binding fixes \
                     `{fixed_id}` at `routes[{at}]`; a refinement keeps every operation id the \
                     refined binding fixes"
                ),
            );
        }
    }
}

/// Refuses a refinement that changes `refusals:`, a method or path the refined binding fixed,
/// leaves a fixed path open, restates a route's `request` or answers with another value, or drops
/// a route of the refined binding (`ESS-HTTP-012`). Each operation is compared at the first route
/// that names it in each document; a second one is `ESS-HTTP-005` already.
fn changes(spec: &HttpSpec, refined: &HttpSpec, refusals: &mut Refusals) {
    refusal_changes(
        spec.refusals.as_ref(),
        refined.refusals.as_ref(),
        "refusals",
        refusals,
    );
    binding_limit_changes(spec, refined, refusals);
    let mut routed: BTreeMap<Operation, usize> = BTreeMap::new();
    for (index, route) in spec.routes.iter().enumerate() {
        if let Some(operation) = named(route) {
            routed.entry(operation).or_insert(index);
        }
    }
    let mut compared = std::collections::BTreeSet::new();
    for (at, fixed) in refined.routes.iter().enumerate() {
        let Some(operation) = named(fixed) else {
            continue;
        };
        if !compared.insert(operation.clone()) {
            continue;
        }
        let fixed_path = fixed.path.as_deref();
        let Some(&index) = routed.get(&operation) else {
            refusals.refuse(
                "ESS-HTTP-012",
                "routes",
                format!(
                    "drops {operation}, which the refined binding routes at `routes[{at}]` \
                     (`{} {}`); a refinement restates every route of the binding it refines",
                    fixed.method,
                    fixed_path.unwrap_or("(open)")
                ),
            );
            continue;
        };
        let route = &spec.routes[index];
        if route.method != fixed.method {
            refusals.refuse(
                "ESS-HTTP-012",
                format!("routes[{index}].method"),
                format!(
                    "{operation} is on `{}` here, and the refined binding fixes `{}` at \
                     `routes[{at}]`; a refinement keeps the method of every route it restates",
                    route.method, fixed.method
                ),
            );
        }
        if let Some(fixed_path) = fixed_path {
            let found = match route.path.as_deref() {
                Some(path) if path == fixed_path => None,
                Some(path) => Some(format!("is on `{path}` here")),
                None => Some("is left open here".to_owned()),
            };
            if let Some(found) = found {
                refusals.refuse(
                    "ESS-HTTP-012",
                    format!("routes[{index}].path"),
                    format!(
                        "{operation} {found}, and the refined binding fixes `{fixed_path}` at \
                         `routes[{at}]`; a refinement fills in only a path the refined binding \
                         left open"
                    ),
                );
            }
        }
        request_changes(route, fixed, &operation, [index, at], refusals);
        answer_changes(route, fixed, &operation, index, at, refusals);
        representation_changes(route, fixed, &operation, [index, at], refusals);
        refusal_changes(
            route.refusals.as_ref(),
            fixed.refusals.as_ref(),
            &format!("routes[{index}].refusals"),
            refusals,
        );
        limit_changes(
            (route.limits.as_ref(), fixed.limits.as_ref()),
            format!("routes[{index}].limits"),
            |new, old| {
                format!(
                    "{operation} bounds its request at {new} here, and the refined binding fixes \
                     {old} at `routes[{at}]`; a refinement restates the limits of every route it \
                     restates"
                )
            },
            refusals,
        );
    }
}

/// Refuses a refining document whose refusals under `prefix` differ from the refined binding's
/// (`ESS-HTTP-012`), entry by entry, at `<prefix>.<key>`: an entry added, dropped, or restated with
/// another status, body or headers. `prefix` is `refusals` for the binding's, `routes[i].refusals`
/// for a route's own. The first entry of each key is compared; a second is `ESS-HTTP-029`. Bodies
/// are compared as written values, never as text, so that one literal cannot spell another.
fn refusal_changes(
    here: Option<&Entries<RefusalSpec>>,
    there: Option<&Entries<RefusalSpec>>,
    prefix: &str,
    refusals: &mut Refusals,
) {
    type Compared = (i64, BodySpec, Option<serde_json::Value>);
    let entries = |written: Option<&Entries<RefusalSpec>>| -> BTreeMap<String, Compared> {
        let mut entries = BTreeMap::new();
        for (key, refusal) in written.iter().flat_map(|entries| &entries.0) {
            entries.entry(key.clone()).or_insert_with(|| {
                (
                    refusal.status,
                    refusal.body.clone(),
                    compared_entries(refusal.headers.as_ref(), HeaderSpec::compared),
                )
            });
        }
        entries
    };
    let (new, old) = (entries(here), entries(there));
    let keys: std::collections::BTreeSet<&String> = new.keys().chain(old.keys()).collect();
    for key in keys {
        let shown = |entry: Option<&Compared>| {
            entry.map_or_else(
                || "nothing".to_owned(),
                |(status, body, _)| format!("`{status}` with `{body}`"),
            )
        };
        let (here, there) = (new.get(key), old.get(key));
        let answer =
            |entry: Option<&Compared>| entry.map(|(status, body, _)| (*status, body.clone()));
        if answer(here) != answer(there) {
            refusals.refuse(
                "ESS-HTTP-012",
                format!("{prefix}.{key}"),
                format!(
                    "`{prefix}.{key}` answers {} here, and the refined binding fixes {}; a \
                     refinement restates the refined binding's refusals exactly",
                    shown(here),
                    shown(there)
                ),
            );
        }
        // A refusal added or dropped is refused above, as a whole.
        let (Some((_, _, here)), Some((_, _, there))) = (here, there) else {
            continue;
        };
        if here != there {
            let shown = |headers: &Option<serde_json::Value>| {
                headers.as_ref().map_or_else(
                    || "no `headers`".to_owned(),
                    |headers| format!("`{headers}`"),
                )
            };
            refusals.refuse(
                "ESS-HTTP-012",
                format!("{prefix}.{key}.headers"),
                format!(
                    "`{prefix}.{key}` declares {} here, and the refined binding fixes {}; a \
                     refinement restates the refined binding's refusals exactly",
                    shown(here),
                    shown(there)
                ),
            );
        }
    }
}

/// Refuses a refined route whose `alike` groups differ from the refined binding's
/// (`ESS-HTTP-012`), at `routes[i].alike`. Groups are compared as sets of sets: the order they are
/// written in is not a change.
fn alike_changes(
    route: &RouteSpec,
    fixed: &RouteSpec,
    operation: &Operation,
    [index, at]: [usize; 2],
    refusals: &mut Refusals,
) {
    let groups = |route: &RouteSpec| -> Vec<Vec<String>> {
        let mut groups: Vec<Vec<String>> = route
            .alike
            .iter()
            .flatten()
            .map(|group| {
                let mut group = group.clone();
                group.sort();
                group
            })
            .collect();
        groups.sort();
        groups
    };
    let (new_groups, old_groups) = (groups(route), groups(fixed));
    if new_groups != old_groups {
        let shown = |groups: &[Vec<String>]| {
            if groups.is_empty() {
                "no `alike`".to_owned()
            } else {
                let written: Vec<String> = groups
                    .iter()
                    .map(|group| format!("[{}]", group.join(", ")))
                    .collect();
                format!("`alike: [{}]`", written.join(", "))
            }
        };
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].alike"),
            format!(
                "{operation} declares {} here, and at `routes[{at}]` the refined binding fixes \
                 {}; a refinement restates the answers of every route it restates",
                shown(&new_groups),
                shown(&old_groups)
            ),
        );
    }
}

/// Refuses a refined route whose answers are restated with another value (`ESS-HTTP-012`): each
/// listed outcome at `routes[i].outcomes.<name>` (its encoding included), the groups of `alike` at
/// `routes[i].alike`, the view body at `routes[i].body` and the empty
/// answer at `routes[i].empty`. A default written out or left out counts as that default, so the
/// comparison is over what compiles. Bodies are compared as written values, never as text, so that
/// a word cannot spell another form.
fn answer_changes(
    route: &RouteSpec,
    fixed: &RouteSpec,
    operation: &Operation,
    index: usize,
    at: usize,
    refusals: &mut Refusals,
) {
    let listed = |route: &RouteSpec| -> BTreeMap<String, (i64, BodySpec, String)> {
        let mut listed = BTreeMap::new();
        for (name, outcome) in route.outcomes.iter().flat_map(|entries| &entries.0) {
            listed.entry(name.clone()).or_insert_with(|| {
                let body = outcome
                    .body
                    .clone()
                    .unwrap_or_else(|| Body::Envelope.spec());
                let encoding = outcome
                    .encoding
                    .clone()
                    .unwrap_or_else(|| ResponseEncoding::default().as_str().to_owned());
                (outcome.status, body, encoding)
            });
        }
        listed
    };
    let (new, old) = (listed(route), listed(fixed));
    let names: std::collections::BTreeSet<&String> = new.keys().chain(old.keys()).collect();
    let shown = |entry: Option<&(i64, BodySpec, String)>| {
        entry.map_or_else(
            || "is not listed".to_owned(),
            |(status, body, encoding)| {
                format!("answers `{status}` with `{body}`, encoded `{encoding}`")
            },
        )
    };
    let (new_extras, old_extras) = (outcome_extras(route), outcome_extras(fixed));
    for name in names {
        let (here, there) = (new.get(name), old.get(name));
        if here != there {
            refusals.refuse(
                "ESS-HTTP-012",
                format!("routes[{index}].outcomes.{name}"),
                format!(
                    "outcome `{name}` of {operation} {} here, and at `routes[{at}]` of the \
                     refined binding it {}; a refinement restates the answers of every route it \
                     restates",
                    shown(here),
                    shown(there)
                ),
            );
        }
        // An outcome added or dropped is refused above, as a whole.
        let (Some(here), Some(there)) = (new_extras.get(name), old_extras.get(name)) else {
            continue;
        };
        extras_changes(name, here, there, operation, [index, at], refusals);
    }
    errors_changes(route, fixed, operation, [index, at], refusals);
    alike_changes(route, fixed, operation, [index, at], refusals);
    let body = |route: &RouteSpec| {
        route
            .body
            .clone()
            .unwrap_or_else(|| BodySpec::Word("rows".to_owned()))
    };
    let (new_body, old_body) = (body(route), body(fixed));
    if new_body != old_body {
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].body"),
            format!(
                "{operation} answers `{new_body}` here, and the refined binding fixes \
                 `{old_body}` at `routes[{at}]`"
            ),
        );
    }
    let shown_empty = |empty: Option<(i64, BodySpec)>| {
        empty.map_or_else(
            || "no empty answer".to_owned(),
            |(status, body)| format!("`{status}` with `{body}`"),
        )
    };
    // An empty answer's body left out is `none`.
    let empty = |route: &RouteSpec| {
        route.empty.as_ref().map(|empty| {
            (
                empty.status,
                empty
                    .body
                    .clone()
                    .unwrap_or_else(|| BodySpec::Word("none".to_owned())),
            )
        })
    };
    let (new_empty, old_empty) = (empty(route), empty(fixed));
    if new_empty != old_empty {
        let (new_empty, old_empty) = (shown_empty(new_empty), shown_empty(old_empty));
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].empty"),
            format!(
                "{operation} has {new_empty} for a view that holds no row here, and the refined \
                 binding fixes {old_empty} at `routes[{at}]`"
            ),
        );
    }
}

/// Refuses a refined route whose `representations` are added, dropped or changed
/// (`ESS-HTTP-012`), compared as a whole at `routes[i].representations`: in order, each by its
/// selector, its encoding with a default written out or left out counting as that default, and
/// its headers by name; an empty list counts as none.
fn representation_changes(
    route: &RouteSpec,
    fixed: &RouteSpec,
    operation: &Operation,
    [index, at]: [usize; 2],
    refusals: &mut Refusals,
) {
    let compared = |route: &RouteSpec| {
        route
            .representations
            .as_ref()
            .filter(|written| !written.is_empty())
            .map(|written| {
                written
                    .iter()
                    .map(RepresentationSpec::compared)
                    .collect::<Vec<_>>()
            })
    };
    if compared(route) != compared(fixed) {
        let shown = |route: &RouteSpec| match &route.representations {
            Some(written) if !written.is_empty() => format!("{} representations", written.len()),
            _ => "no representation".to_owned(),
        };
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].representations"),
            format!(
                "{operation} declares {} here, and the refined binding fixes {} at \
                 `routes[{at}]`, or the same number differing; a refinement restates the \
                 representations of every route it restates",
                shown(route),
                shown(fixed)
            ),
        );
    }
}

/// A route's `errors` as a refinement compares it: status, body (its default counted), headers and
/// redirect.
type ComparedErrors = (
    i64,
    BodySpec,
    Option<serde_json::Value>,
    Option<serde_json::Value>,
);

/// Refuses a refined route whose `errors` is restated with another value, or added or dropped
/// (`ESS-HTTP-012`), compared as a whole at `routes[i].errors`.
fn errors_changes(
    route: &RouteSpec,
    fixed: &RouteSpec,
    operation: &Operation,
    [index, at]: [usize; 2],
    refusals: &mut Refusals,
) {
    use std::fmt::Write as _;
    let errors = |route: &RouteSpec| -> Option<ComparedErrors> {
        route.errors.as_ref().map(|errors| {
            let body = errors.body.clone().unwrap_or_else(|| Body::Envelope.spec());
            let [headers, redirect] = [
                compared_entries(errors.headers.as_ref(), HeaderSpec::compared),
                errors.redirect.as_ref().map(RedirectSpec::compared),
            ];
            (errors.status, body, headers, redirect)
        })
    };
    let (new_errors, old_errors) = (errors(route), errors(fixed));
    if new_errors == old_errors {
        return;
    }
    let shown = |errors: &Option<ComparedErrors>| {
        errors.as_ref().map_or_else(
            || "declares no `errors`".to_owned(),
            |(status, body, headers, redirect)| {
                let mut shown = format!("answers its unlisted errors `{status}` with `{body}`");
                if let Some(headers) = headers {
                    let _ = write!(shown, ", headers `{headers}`");
                }
                if let Some(redirect) = redirect {
                    let _ = write!(shown, ", redirect `{redirect}`");
                }
                shown
            },
        )
    };
    refusals.refuse(
        "ESS-HTTP-012",
        format!("routes[{index}].errors"),
        format!(
            "{operation} {} here, and at `routes[{at}]` of the refined binding it {}; a \
             refinement restates the answers of every route it restates",
            shown(&new_errors),
            shown(&old_errors)
        ),
    );
}

/// Each listed outcome's headers and redirect as a refinement compares them (`ESS-HTTP-012`), the
/// first listing of a name counted.
fn outcome_extras(route: &RouteSpec) -> BTreeMap<String, [Option<serde_json::Value>; 2]> {
    let mut extras = BTreeMap::new();
    for (name, outcome) in route.outcomes.iter().flat_map(|entries| &entries.0) {
        extras.entry(name.clone()).or_insert([
            compared_entries(outcome.headers.as_ref(), HeaderSpec::compared),
            outcome.redirect.as_ref().map(RedirectSpec::compared),
        ]);
    }
    extras
}

/// Refuses outcome `name`, listed by a refined route and its refinement, whose headers or redirect
/// (`here` and `there`, as [`outcome_extras`] has them) differ (`ESS-HTTP-012`).
fn extras_changes(
    name: &str,
    here: &[Option<serde_json::Value>; 2],
    there: &[Option<serde_json::Value>; 2],
    operation: &Operation,
    [index, at]: [usize; 2],
    refusals: &mut Refusals,
) {
    for (key, (new, old)) in ["headers", "redirect"]
        .into_iter()
        .zip(here.iter().zip(there))
    {
        if new == old {
            continue;
        }
        let shown = |value: &Option<serde_json::Value>| {
            value
                .as_ref()
                .map_or_else(|| format!("no `{key}`"), |value| format!("`{value}`"))
        };
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].outcomes.{name}.{key}"),
            format!(
                "outcome `{name}` of {operation} declares {} here, and at `routes[{at}]` of the \
                 refined binding {}; a refinement restates the `{key}` of every outcome it \
                 restates",
                shown(new),
                shown(old)
            ),
        );
    }
}

/// Refuses (`ESS-HTTP-012`) a `request` key of `route`, at `routes[index]`, written otherwise than
/// the refined binding's `fixed` route at `routes[at]` writes it: a default written out or left out
/// counts as that default, `parts` are compared attribute by attribute and `query` in the order written.
fn request_changes(
    route: &RouteSpec,
    fixed: &RouteSpec,
    operation: &Operation,
    [index, at]: [usize; 2],
    refusals: &mut Refusals,
) {
    let fixed_request = written_request(fixed);
    for ((key, new), (_, old)) in written_request(route).into_iter().zip(fixed_request) {
        if new != old {
            refusals.refuse(
                "ESS-HTTP-012",
                format!("routes[{index}].request.{key}"),
                format!(
                    "{operation} has `request.{key}` `{new}` here, and the refined binding fixes \
                     `{old}` at `routes[{at}]`; a refinement restates the request of every route \
                     it restates"
                ),
            );
        }
    }
    let parts = |route: &RouteSpec| {
        route
            .request
            .as_ref()
            .and_then(|request| request.parts.clone())
            .unwrap_or_default()
    };
    if parts(route) != parts(fixed) {
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].request.parts"),
            format!(
                "{operation} states other `request.parts` here than the refined binding fixes at \
                 `routes[{at}]`; a refinement restates the parts of every route it restates, each \
                 attribute as written"
            ),
        );
    }
    let query = |route: &RouteSpec| {
        route
            .request
            .as_ref()
            .and_then(|request| request.query.clone())
            .unwrap_or_default()
    };
    if query(route) != query(fixed) {
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].request.query"),
            format!(
                "{operation} lists other `request.query` fields here than the refined binding \
                 fixes at `routes[{at}]`; a refinement restates the query of every route it \
                 restates, in the order written"
            ),
        );
    }
    if route.select != fixed.select {
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].select"),
            format!(
                "{operation} has another `select` here than the refined binding fixes at \
                 `routes[{at}]`; a refinement restates the select of every route it restates"
            ),
        );
    }
    let lists = |route: &RouteSpec| {
        let mut lists: BTreeMap<String, String> = BTreeMap::new();
        for (name, form) in route.lists.iter().flat_map(|lists| &lists.0) {
            lists.entry(name.clone()).or_insert_with(|| form.clone());
        }
        lists
    };
    if lists(route) != lists(fixed) {
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].lists"),
            format!(
                "{operation} writes other `lists` here than the refined binding fixes at \
                 `routes[{at}]`; a refinement restates the lists of every route it restates"
            ),
        );
    }
    let members = |route: &RouteSpec| {
        route
            .request
            .as_ref()
            .and_then(|request| request.members.clone())
            .unwrap_or_default()
    };
    if members(route) != members(fixed) {
        refusals.refuse(
            "ESS-HTTP-012",
            format!("routes[{index}].request.members"),
            format!(
                "{operation} maps other `request.members` here than the refined binding fixes at \
                 `routes[{at}]`; a refinement restates the members of every route it restates"
            ),
        );
    }
}

/// A route's `request` keys as written, each absent one as its default, so that writing a default
/// out is not a change.
fn written_request(route: &RouteSpec) -> [(&'static str, &str); 3] {
    let request = route.request.as_ref();
    [
        (
            "body",
            request
                .and_then(|request| request.body.as_deref())
                .unwrap_or("absent"),
        ),
        (
            Encoding::KEY,
            request
                .and_then(|request| request.encoding.as_deref())
                .unwrap_or(Encoding::default().as_str()),
        ),
        (
            Unknown::KEY,
            request
                .and_then(|request| request.unknown.as_deref())
                .unwrap_or(Unknown::default().as_str()),
        ),
    ]
}

fn specification(identity: &SpecificationIdentity, ir: &EssIr, refusals: &mut Refusals) {
    let digest = format!("sha256:{}", ir.source_digest());
    for (key, found, expected) in [
        ("system", identity.system.as_str(), ir.system().to_string()),
        (
            "version",
            identity.version.as_str(),
            ir.version().to_string(),
        ),
        ("source_digest", identity.source_digest.as_str(), digest),
    ] {
        if found != expected {
            refusals.refuse(
                "ESS-HTTP-002",
                format!("specification.{key}"),
                format!("names `{found}`, and the specification is `{expected}`"),
            );
        }
    }
}

/// A route that names an operation, whatever else it was refused for: it names that operation, so
/// the operation's derived route leaves the surface.
struct Bound {
    index: usize,
    operation: Operation,
    /// The method, path and segments it claims on the surface, when its method is admitted and its
    /// path parsed, whatever else it was refused for; `None` for an open route or either refusal.
    claim: Option<(Method, String, Vec<Segment>)>,
    /// The resolved route, when its operation, method, path, sources, request and `get` input
    /// resolved; it reaches the IR only when the binding earned no refusal at all.
    route: Option<Route>,
    /// Its `select`, as far as it resolved.
    select: Selecting,
    /// Whether it serves a command whose `json` body is the input object, off `get`: the only
    /// body `select` tells routes apart by.
    json_body: bool,
}

/// A route's `select`, as far as it resolved.
enum Selecting {
    /// None written: on a template it shares, the route is the catch-all.
    Unwritten,
    /// The alternatives, each held to its rules, no two meeting.
    Selects(Vec<Select>),
    /// Refused (`ESS-HTTP-020`), or left unchecked beside a refused encoding.
    Refused,
}

impl Selecting {
    /// The route's compiled `select`.
    fn compiled(&self) -> Option<Selection> {
        match self {
            Self::Selects(selects) => Some(Selection(selects.clone())),
            Self::Unwritten | Self::Refused => None,
        }
    }
}

/// Whether a route serves a command on a method other than `get` whose `json` body, as written,
/// is the input object: no bare `body`, and `json` written or left out.
fn members_body(spec: &RouteSpec, operation: &Operation, method: Option<Method>) -> bool {
    let request = spec.request.as_ref();
    matches!(operation, Operation::Command(_))
        && method.is_some_and(|method| method != Method::Get)
        && request.is_none_or(|request| {
            request.body.is_none()
                && request
                    .encoding
                    .as_deref()
                    .is_none_or(|text| text == Encoding::Json.as_str())
        })
}

/// A route's `select` (`ESS-HTTP-020`, at `routes[i].select`), resolved: only on a command whose
/// `json` body is the input object, off `get`; a map of member paths to literals, or a list of at
/// least one such map, each map an alternative testing at least one member, its literals strings,
/// Booleans or integers ([`select_of`]); and no two alternatives meeting.
fn selection(
    spec: &RouteSpec,
    operation: &Operation,
    target: Option<&Target<'_>>,
    method: Option<Method>,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> Selecting {
    let Some(written) = &spec.select else {
        return Selecting::Unwritten;
    };
    let at = format!("{path}.select");
    let request = spec.request.as_ref();
    let encoding = request
        .and_then(|request| request.encoding.as_deref())
        .map_or(Some(Encoding::Json), Encoding::parse);
    let rule = match (operation, encoding, method) {
        (Operation::View(_), _, _) => Some(format!(
            "{operation} takes no body: `select` tests members of a command's `json` body"
        )),
        (_, Some(encoding @ (Encoding::Form | Encoding::Query | Encoding::Multipart)), _) => {
            Some(format!(
                "`select` tests members of a `json` body, and this request is encoded `{encoding}`"
            ))
        }
        _ if request.is_some_and(|request| request.body.is_some()) => Some(
            "`select` tests members of the input object, and a bare `body` is one input field's \
             value, which holds no member of the input to test"
                .to_owned(),
        ),
        (_, _, Some(Method::Get)) => {
            Some("`select` tests members of a `json` body, and `get` carries none".to_owned())
        }
        _ => None,
    };
    if let Some(rule) = rule {
        refusals.refuse("ESS-HTTP-020", at, rule);
        return Selecting::Refused;
    }
    let maps: Vec<&serde_json::Map<String, serde_json::Value>> = match written {
        serde_json::Value::Object(map) => vec![map],
        serde_json::Value::Array(items) if !items.is_empty() => items
            .iter()
            .filter_map(serde_json::Value::as_object)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let listed = match written {
        serde_json::Value::Array(items) => items.len(),
        _ => 1,
    };
    if maps.len() != listed || maps.is_empty() {
        refusals.refuse(
            "ESS-HTTP-020",
            at,
            match written {
                serde_json::Value::Array(items) if items.is_empty() => {
                    "`select` is an empty list: it holds at least one map".to_owned()
                }
                _ => format!(
                    "`select` is `{written}`: it is a map of member paths to literals, or a list \
                     of such maps"
                ),
            },
        );
        return Selecting::Refused;
    }
    let (Some(target), Some(_)) = (target, encoding) else {
        return Selecting::Refused;
    };
    let mut selects = Vec::new();
    let mut refused = false;
    for map in maps {
        match select_of(map, target, request, mapped) {
            Ok(select) => selects.push(select),
            Err(rule) => {
                refusals.refuse("ESS-HTTP-020", at.as_str(), rule);
                refused = true;
            }
        }
    }
    if let Some((earlier, later)) = Selection::meeting(&selects) {
        refusals.refuse(
            "ESS-HTTP-020",
            at.as_str(),
            format!(
                "alternative `{}` and alternative `{}` both match `{}`; the alternatives of one \
                 route are disjoint",
                selects[earlier],
                selects[later],
                selects[earlier].common_body(&selects[later])
            ),
        );
        refused = true;
    }
    if refused {
        Selecting::Refused
    } else {
        Selecting::Selects(selects)
    }
}

/// One alternative of a `select` (`map`), or why it is refused: at least one member, each a member
/// path, none an object above another, each literal a string, a Boolean or an integer within 64
/// bits, and on a member an input field is read from a literal its type admits; a member above or
/// inside the member an input field is read from is refused.
fn select_of(
    map: &serde_json::Map<String, serde_json::Value>,
    target: &Target<'_>,
    request: Option<&RequestSpec>,
    mapped: &[String],
) -> Result<Select, String> {
    if map.is_empty() {
        return Err("`select` holds an empty map: an alternative tests at least one member".into());
    }
    // Where the body carries each input field the path and the query string do not.
    let queried: Vec<String> = request
        .and_then(|request| request.query.as_ref())
        .into_iter()
        .flatten()
        .filter_map(|text| Source::parse(text).map(|source| source.field().to_owned()))
        .collect();
    let members = request.and_then(|request| request.members.as_ref());
    let carried: Vec<(&ResolvedField, MemberPath)> = target
        .fields
        .iter()
        .filter(|field| {
            !mapped.contains(&format!("input.{}", field.name)) && !queried.contains(&field.name)
        })
        .filter_map(|field| {
            let text = members
                .and_then(|members| members.get(&field.name))
                .cloned()
                .unwrap_or_else(|| {
                    field
                        .naming
                        .wire
                        .clone()
                        .unwrap_or_else(|| field.name.clone())
                });
            MemberPath::parse(&text).map(|member| (field, member))
        })
        .collect();
    let mut tests = Vec::new();
    for (text, value) in map {
        let member = MemberPath::parse(text).ok_or_else(|| {
            format!(
                "`{text}` is not a member path: one or more member names joined by `.`, and a \
                 name is never empty"
            )
        })?;
        let literal = MemberLiteral::of(value).ok_or_else(|| {
            format!(
                "`{text}: {value}` is not a literal a `select` tests for: a string, a Boolean or \
                 an integer"
            )
        })?;
        for (field, read) in &carried {
            if *read == member {
                if !admits(target.ir, &field.type_ref, &literal) {
                    return Err(format!(
                        "`{text}: {value}` tests the member input field `{}` is read from, of type \
                         `{}`, which does not admit that literal",
                        field.name, field.type_ref
                    ));
                }
            } else if read.covers(&member) || member.covers(read) {
                return Err(format!(
                    "`{text}` is above or inside `{read}`, the member input field `{}` is read \
                     from: a tested member is the member of a field, or of none",
                    field.name
                ));
            }
        }
        tests.push((member, literal));
    }
    tests.sort();
    Select::checked(tests)
}

/// Whether `type_ref` admits `literal`: a Boolean a `Boolean`, an integer an `Integer`, a string a
/// `String` or an enum naming it among its variants' wire names, each through newtypes and
/// `Optional`.
fn admits(ir: &EssIr, type_ref: &ResolvedTypeRef, literal: &MemberLiteral) -> bool {
    match literal {
        MemberLiteral::Boolean(_) => of_primitive(type_ref, Primitive::Boolean, ir),
        MemberLiteral::Integer(_) => of_primitive(type_ref, Primitive::Integer, ir),
        MemberLiteral::String(text) => {
            of_primitive(type_ref, Primitive::String, ir) || names_variant(ir, type_ref, text)
        }
    }
}

/// Whether `type_ref`, through `Optional` and newtypes, is an enum one of whose variants is
/// written `text`.
fn names_variant(ir: &EssIr, type_ref: &ResolvedTypeRef, text: &str) -> bool {
    let mut current = type_ref;
    for _ in 0..=ir.types().len() {
        match current {
            ResolvedTypeRef::Optional { of } => current = of,
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => current = of,
                ResolvedBody::Enum { variants } => {
                    return variants.iter().any(|variant| {
                        variant.naming.wire.as_deref().unwrap_or(&variant.name) == text
                    });
                }
                _ => return false,
            },
            _ => return false,
        }
    }
    false
}

/// The fields a route's sources are read from: a command's input or a view's parameters.
struct Target<'a> {
    ir: &'a EssIr,
    fields: &'a [ResolvedField],
    served: Served<'a>,
}

/// The operation a route serves, resolved.
#[derive(Clone, Copy)]
enum Served<'a> {
    Command(&'a ResolvedCommand),
    View(&'a ResolvedView),
}

impl Target<'_> {
    fn is_command(&self) -> bool {
        matches!(self.served, Served::Command(_))
    }
}

/// Every route that names an operation, in document order. A route that earned a refusal is still
/// checked in full, so its other refusals are reported too, and it still takes part in the
/// collision checks whenever its method is admitted and its path parsed.
///
/// Within one route the refusals come in this order: the operation's shape (`004`), the method
/// (`006`), the path's grammar (`007`), the operation's existence and surface (`004`, `013`), the
/// mapping of variables (`008`), the sources (`009`), the request (`020`), the spelling of each
/// field it carries (`021`), the unmapped `get` input (`014`), the `lists` (`021`), the answers
/// (each listed outcome's
/// name `022`, status `023`, body `024`, headers `026` and `027` and redirect `028` in document
/// order, then `025`; a view's `outcomes`
/// `022`, `body` `024`, `empty` `030` and identity `031`), the representations (`035`, their headers
/// `026`), and the operation routed twice (`005`).
fn routes(
    specs: &[RouteSpec],
    ir: &EssIr,
    standard: &BTreeMap<RefusalKey, Refusal>,
    refusals: &mut Refusals,
) -> Vec<Bound> {
    let mut first_route: BTreeMap<Operation, usize> = BTreeMap::new();
    let mut routes = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let path = format!("routes[{index}]");
        let operation = served_operation(spec, &path, refusals);
        let method = method(spec, operation.as_ref(), &path, refusals);
        // `None` when the path breaks the grammar; `Some(None)` for an open route.
        let segments = match spec.path.as_deref().map(template) {
            None => Some(None),
            Some(Ok(segments)) => Some(Some(segments)),
            Some(Err(reasons)) => {
                for reason in reasons {
                    refusals.refuse("ESS-HTTP-007", format!("{path}.path"), reason);
                }
                None
            }
        };
        let Some(operation) = operation else {
            continue;
        };
        let target = match &operation {
            Operation::Command(name) => command(name, ir, &path, refusals),
            Operation::View(name) => view(name, ir, &path, refusals),
        };
        let mapped = variables(spec, segments.as_ref(), &path, refusals);
        let sources = target
            .as_ref()
            .and_then(|target| sources(spec, target, &path, refusals));
        let request = carried(
            spec,
            &operation,
            target.as_ref(),
            method,
            &mapped,
            &path,
            refusals,
        );
        let select = selection(
            spec,
            &operation,
            target.as_ref(),
            method,
            &mapped,
            &path,
            refusals,
        );
        let sources = sources.zip(lists(spec, target.as_ref(), &mapped, &path, refusals));
        let route_limits = route_limits(spec, &operation, method, &request, &path, refusals);
        let cors = route_cors(spec, &path, refusals);
        let own = self::standard(
            spec.refusals.as_ref(),
            &format!("{path}.refusals"),
            refusals,
        );
        let standing = standing(standard, &own, &path);
        let answers = answers(
            spec,
            &operation,
            target.as_ref(),
            &path,
            &path_variables(segments.as_ref()),
            &standing,
            refusals,
        );
        routed_once(&mut first_route, &operation, index, refusals);
        let claim = claim_of(method, segments.as_ref(), spec.path.as_ref());
        let route = match (target, method, segments, sources, request, answers) {
            (Some(_), Some(method), Some(_), Some((sources, lists)), Ok(request), Ok(answers)) => {
                Some(Route {
                    operation: operation.clone(),
                    operation_id: spec.operation_id.clone(),
                    authentication: spec.authentication.clone(),
                    method,
                    path: spec.path.clone(),
                    parameters: sources,
                    request,
                    select: select.compiled(),
                    lists,
                    outcomes: answers.outcomes,
                    errors: answers.errors,
                    alike: answers.alike,
                    body: answers.body,
                    empty: answers.empty,
                    refusals: own,
                    limits: route_limits,
                    cors,
                    representations: answers.representations,
                })
            }
            _ => None,
        };
        routes.push(Bound {
            index,
            json_body: members_body(spec, &operation, method),
            operation,
            claim,
            select,
            route,
        });
    }
    routes
}
/// Refuses `routes[index]` when an earlier route routes `operation` already (`ESS-HTTP-005`), and
/// records it as the operation's route otherwise.
fn routed_once(
    first_route: &mut BTreeMap<Operation, usize>,
    operation: &Operation,
    index: usize,
    refusals: &mut Refusals,
) {
    if let Some(first) = first_route.get(operation) {
        refusals.refuse(
            "ESS-HTTP-005",
            format!("routes[{index}]"),
            format!(
                "{operation} is already routed by `routes[{first}]`; an operation has one route"
            ),
        );
    } else {
        first_route.insert(operation.clone(), index);
    }
}

/// The method, path and segments a route claims on the surface: `None` for an open route, or when
/// its method was refused or its path broke the grammar.
fn claim_of(
    method: Option<Method>,
    segments: Option<&Option<Vec<Segment>>>,
    path: Option<&String>,
) -> Option<(Method, String, Vec<Segment>)> {
    match (method, segments, path) {
        (Some(method), Some(Some(segments)), Some(path)) => {
            Some((method, path.clone(), segments.clone()))
        }
        _ => None,
    }
}

/// The operation a route serves, or `None` when it names neither or both (`ESS-HTTP-004`).
fn served_operation(spec: &RouteSpec, path: &str, refusals: &mut Refusals) -> Option<Operation> {
    match (&spec.command, &spec.view) {
        (Some(name), None) => Some(Operation::Command(name.clone())),
        (None, Some(name)) => Some(Operation::View(name.clone())),
        (None, None) => {
            refusals.refuse(
                "ESS-HTTP-004",
                path,
                "a route names the command or the view it serves, with `command:` or `view:`",
            );
            None
        }
        (Some(command), Some(view)) => {
            refusals.refuse(
                "ESS-HTTP-004",
                path,
                format!(
                    "a route serves one operation, and this one names both command \
                     `{command}` and view `{view}`"
                ),
            );
            None
        }
    }
}

/// The route's method, when it is one of the closed set and admitted for what the route serves.
fn method(
    spec: &RouteSpec,
    operation: Option<&Operation>,
    path: &str,
    refusals: &mut Refusals,
) -> Option<Method> {
    let at = format!("{path}.method");
    let Some(method) = Method::parse(&spec.method) else {
        let set: Vec<String> = Method::ALL
            .iter()
            .map(|method| format!("`{method}`"))
            .collect();
        refusals.refuse(
            "ESS-HTTP-006",
            at,
            format!(
                "`{}` is not a method; a route names one of {}, in lower case",
                spec.method,
                set.join(", ")
            ),
        );
        return None;
    };
    let admitted = match operation {
        None => true,
        Some(Operation::View(_)) => method == Method::Get,
        Some(Operation::Command(_)) => matches!(
            method,
            Method::Post | Method::Put | Method::Patch | Method::Delete | Method::Get
        ),
    };
    if !admitted {
        let rule = match operation {
            Some(Operation::View(_)) => "a view is bound to `get` only",
            _ => {
                "a command is bound to `post`, `put`, `patch` or `delete`, or to `get` when every \
                 input field is mapped to the path or its request is encoded `query`"
            }
        };
        let kind = operation.map_or("operation", Operation::kind);
        refusals.refuse(
            "ESS-HTTP-006",
            at,
            format!("`{method}` is not admitted for a {kind}: {rule}"),
        );
        return None;
    }
    Some(method)
}

/// The command's input, when it exists and has an HTTP surface.
fn command<'a>(
    name: &str,
    ir: &'a EssIr,
    path: &str,
    refusals: &mut Refusals,
) -> Option<Target<'a>> {
    let at = format!("{path}.command");
    let Some((qualified, command)) = ir
        .commands()
        .iter()
        .find(|(qualified, _)| qualified.to_string() == name)
    else {
        refusals.refuse(
            "ESS-HTTP-004",
            at,
            format!("the specification declares no command `{name}`"),
        );
        return None;
    };
    let accepted = ir.components().values().any(|component| {
        component
            .accepts
            .iter()
            .any(|handle| handle.name() == qualified)
    });
    if !accepted {
        refusals.refuse(
            "ESS-HTTP-013",
            at,
            format!("no component accepts command `{name}`, so it has no HTTP surface to route"),
        );
        return None;
    }
    Some(Target {
        ir,
        fields: &command.input,
        served: Served::Command(command),
    })
}

/// The view's parameters, when it exists and a `reached_by: network` component serves it.
fn view<'a>(name: &str, ir: &'a EssIr, path: &str, refusals: &mut Refusals) -> Option<Target<'a>> {
    let at = format!("{path}.view");
    let Some(view) = ir
        .views()
        .iter()
        .find_map(|(qualified, view)| (qualified.to_string() == name).then_some(view))
    else {
        refusals.refuse(
            "ESS-HTTP-004",
            at,
            format!("the specification declares no view `{name}`"),
        );
        return None;
    };
    let served = ir.components().values().any(|component| {
        component.reached_by == Reach::Network && component.owns.contains(&view.domain)
    });
    if !served {
        refusals.refuse(
            "ESS-HTTP-013",
            at,
            format!(
                "no `reached_by: network` component owns the domain of view `{name}`, so it has \
                 no HTTP surface to route"
            ),
        );
        return None;
    }
    Some(Target {
        ir,
        fields: &view.params,
        served: Served::View(view),
    })
}

/// Holds each variable of the path to exactly one mapping, and each source to one variable
/// (`ESS-HTTP-008`), and returns the source text of every mapping that maps a variable of the
/// path. When the path breaks the grammar, its
/// variables are unknown: nothing here is refused, and every mapping is taken as mapped, so that
/// the one mistake is reported once, as `ESS-HTTP-007`.
fn variables(
    spec: &RouteSpec,
    segments: Option<&Option<Vec<Segment>>>,
    path: &str,
    refusals: &mut Refusals,
) -> Vec<String> {
    let Some(segments) = segments else {
        return spec
            .parameters
            .0
            .iter()
            .map(|(_, source)| source.clone())
            .collect();
    };
    let names: Vec<&str> = segments
        .iter()
        .flatten()
        .filter_map(|segment| match segment {
            Segment::Variable(name) => Some(name.as_str()),
            Segment::Literal(_) => None,
        })
        .collect();
    let written = spec.path.as_deref().unwrap_or_default();
    let mut reported = std::collections::BTreeSet::new();
    // Each source read so far, normalised, with the variable that reads it.
    let mut read: Vec<(String, &str)> = Vec::new();
    for (position, name) in names.iter().enumerate() {
        if names[..position].contains(name) {
            if reported.insert(*name) {
                refusals.refuse(
                    "ESS-HTTP-008",
                    format!("{path}.parameters.{name}"),
                    format!(
                        "variable `{name}` names two segments of `{written}`, which maps it \
                         twice; a variable names one segment"
                    ),
                );
            }
        } else if let Some(text) = spec.parameters.get(name) {
            let source = Source::parse(text).map_or_else(|| text.to_owned(), |s| s.to_string());
            if let Some((_, earlier)) = read.iter().find(|(seen, _)| *seen == source) {
                refusals.refuse(
                    "ESS-HTTP-008",
                    format!("{path}.parameters.{name}"),
                    format!(
                        "variable `{name}` of `{written}` reads `{text}`, which variable \
                         `{earlier}` already reads; a source is read by one variable, so that a \
                         request path gives it one value"
                    ),
                );
            } else {
                read.push((source, name));
            }
        } else {
            refusals.refuse(
                "ESS-HTTP-008",
                format!("{path}.parameters.{name}"),
                format!(
                    "variable `{name}` of `{written}` is not mapped; each variable of the path \
                     reads one source under `parameters`"
                ),
            );
        }
    }
    let mut mapped = Vec::new();
    for (position, (name, source)) in spec.parameters.0.iter().enumerate() {
        let at = format!("{path}.parameters.{name}");
        if spec.parameters.0[..position]
            .iter()
            .any(|(earlier, _)| earlier == name)
        {
            refusals.refuse(
                "ESS-HTTP-008",
                at,
                format!("variable `{name}` is mapped twice; each variable is mapped once"),
            );
        } else if !names.contains(&name.as_str()) {
            let message = match &spec.path {
                None => format!(
                    "the route has no path, so `{name}` maps no variable; a mapping names a \
                     variable of the path"
                ),
                Some(written) => format!(
                    "`{written}` has no variable `{name}`; a mapping names a variable of the path"
                ),
            };
            refusals.refuse("ESS-HTTP-008", at, message);
        } else {
            mapped.push(source.clone());
        }
    }
    mapped
}

/// Every mapping resolved to its typed source, or `None` when one is refused (`ESS-HTTP-009`).
fn sources(
    spec: &RouteSpec,
    target: &Target<'_>,
    path: &str,
    refusals: &mut Refusals,
) -> Option<BTreeMap<String, Source>> {
    let (namespace, kind, what) = if target.is_command() {
        ("input", "command", "input field")
    } else {
        ("param", "view", "parameter")
    };
    let mut resolved = BTreeMap::new();
    let mut refused = false;
    for (name, text) in &spec.parameters.0 {
        let at = format!("{path}.parameters.{name}");
        let source = Source::parse(text).filter(|source| source.namespace() == namespace);
        let Some(source) = source else {
            refusals.refuse(
                "ESS-HTTP-009",
                at,
                format!(
                    "`{text}` is not `{namespace}.<name>`: a {kind}'s path variable reads one \
                     {what} by name"
                ),
            );
            refused = true;
            continue;
        };
        let Some(field) = target
            .fields
            .iter()
            .find(|field| field.name == source.field())
        else {
            refusals.refuse(
                "ESS-HTTP-009",
                at,
                format!(
                    "`{text}` names no {what}: the {kind} declares no {what} `{}`",
                    source.field()
                ),
            );
            refused = true;
            continue;
        };
        if !segment_type(&field.type_ref, target.ir) {
            refusals.refuse(
                "ESS-HTTP-009",
                at,
                format!(
                    "`{text}` has type `{}`, which a path segment cannot carry: a segment is \
                     always present and holds one `String`, `Integer`, `Boolean`, `Decimal`, \
                     `Timestamp`, `Duration`, `Uuid`, enum, or unconstrained newtype of one of \
                     them",
                    field.type_ref
                ),
            );
            refused = true;
            continue;
        }
        resolved.insert(name.clone(), source);
    }
    (!refused).then_some(resolved)
}

/// A route's `lists`, compiled by name, or `None` when one is refused (`ESS-HTTP-021`, at
/// `routes[i].lists.<name>`, or at `routes[i].lists` when written empty): each name an input field
/// of the command or a parameter of the view, named once, that the path does not carry (`mapped`),
/// of a `List` of scalars, `Optional` or not, and carried in the query string or a `form` body (a
/// view's parameter, or a command's field on a route encoded `form` or `query` or listed in
/// `request.query`); and its form `comma`. Without a `target`, which is refused already, `None`.
fn lists(
    spec: &RouteSpec,
    target: Option<&Target<'_>>,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> Option<BTreeMap<String, ListForm>> {
    let Some(written) = &spec.lists else {
        return Some(BTreeMap::new());
    };
    if written.0.is_empty() {
        refusals.refuse(
            "ESS-HTTP-021",
            format!("{path}.lists"),
            "`lists` is empty: it names at least one list written comma-joined, and is left out \
             to repeat the key of every list",
        );
        return None;
    }
    let target = target?;
    let (namespace, kind, what) = if target.is_command() {
        ("input", "command", "input field")
    } else {
        ("param", "view", "parameter")
    };
    let encoding = match spec
        .request
        .as_ref()
        .and_then(|request| request.encoding.as_deref())
    {
        None => Some(Encoding::default()),
        Some(text) => Encoding::parse(text),
    };
    let queried = |name: &str| {
        spec.request
            .iter()
            .flat_map(|request| request.query.iter().flatten())
            .filter_map(|text| Source::parse(text))
            .any(|source| matches!(source, Source::Input(_)) && source.field() == name)
    };
    let mut compiled = BTreeMap::new();
    let mut refused = false;
    for (position, (name, word)) in written.0.iter().enumerate() {
        let field = target.fields.iter().find(|field| field.name == *name);
        let rule = match field {
            _ if written.0[..position]
                .iter()
                .any(|(earlier, _)| earlier == name) =>
            {
                format!("`{name}` is named twice, and a list is written in one form")
            }
            None => format!("`{name}` names no {what}: the {kind} declares no {what} `{name}`"),
            Some(_) if mapped.contains(&format!("{namespace}.{name}")) => format!(
                "`{name}` is carried by the path, which holds one value and no list; `lists` \
                 names a list carried in the query string or a `form` body"
            ),
            Some(field) if !scalar_list(&field.type_ref, target.ir) => format!(
                "{what} `{name}` has type `{}`, which is not a `List` of scalars: `lists` names a \
                 `List` or an `Optional` `List` of a primitive other than `Json`, an enum, or a \
                 newtype of a scalar",
                field.type_ref
            ),
            Some(_)
                if target.is_command()
                    && !queried(name)
                    && matches!(encoding, Some(Encoding::Json | Encoding::Multipart)) =>
            {
                format!(
                    "input field `{name}` travels in a `{}` body, which carries a list in its own \
                     form; `lists` names a list carried in the query string or a `form` body: \
                     encode the request `form` or `query`, or list `input.{name}` in \
                     `request.query`",
                    encoding.map_or("json", Encoding::as_str)
                )
            }
            Some(_) => match ListForm::parse(word) {
                Some(form) => {
                    compiled.insert(name.clone(), form);
                    continue;
                }
                None => format!(
                    "`{word}` is not a list form: `{name}` is written `comma`, one key whose value \
                     joins the elements with `,`"
                ),
            },
        };
        refusals.refuse("ESS-HTTP-021", format!("{path}.lists.{name}"), rule);
        refused = true;
    }
    (!refused).then_some(compiled)
}

/// Whether `type_ref` is a `List` of scalars, `Optional` or not.
fn scalar_list(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Optional { of } => {
            matches!(&**of, ResolvedTypeRef::List { of } if scalar(of, ir))
        }
        ResolvedTypeRef::List { of } => scalar(of, ir),
        _ => false,
    }
}

/// A route's request or the input it carries earned a refusal, already reported.
struct Refused;

/// The route's compiled request, `None` when it is all defaults, once the request (`ESS-HTTP-020`),
/// the spelling of each field a `form` or `query` request carries (`ESS-HTTP-021`) and a `get`
/// command's unmapped input (`ESS-HTTP-014`) are checked.
fn carried(
    spec: &RouteSpec,
    operation: &Operation,
    target: Option<&Target<'_>>,
    method: Option<Method>,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> Result<Option<Request>, Refused> {
    let checked = request(spec, operation, method, path, refusals);
    // What `request.query` lists, by local name, refused or not: neither the bare body nor the
    // flat spelling of the encoding holds it.
    let listed: Vec<String> = checked
        .query
        .iter()
        .flatten()
        .filter_map(|text| Source::parse(text))
        .map(|source| source.field().to_owned())
        .collect();
    let mut beside = mapped.to_vec();
    beside.extend(listed.iter().map(|field| format!("input.{field}")));
    let query = match (target, &checked.query) {
        (Some(target), Some(written)) => query_fields(
            written,
            target,
            spec.request.as_ref(),
            mapped,
            path,
            refusals,
        ),
        (None, Some(_)) => Err(Refused),
        (_, None) => Ok(Vec::new()),
    };
    let members = match (target, &checked.members) {
        (Some(target), Some(written)) => {
            nested_members(target, written, [mapped, &listed], path, refusals)
        }
        (None, Some(_)) => Err(Refused),
        (_, None) => Ok(Members::default()),
    };
    let bare = match (target, &checked.body) {
        (Some(target), Some(text)) => bare_body(
            target,
            text,
            [mapped, &listed],
            spec.path.is_none(),
            path,
            refusals,
        ),
        _ => Ok(None),
    };
    let flat = match (target, checked.encoding) {
        (
            Some(target),
            Some(encoding @ (Encoding::Form | Encoding::Query | Encoding::Multipart)),
        ) => flat_input(target, encoding, &beside, path, refusals),
        _ => true,
    };
    let unmapped = match (target, method, checked.encoding) {
        (Some(target), Some(Method::Get), Some(Encoding::Json)) if target.is_command() => {
            get_input(operation, target, mapped, path, refusals)
        }
        _ => false,
    };
    let parts = match (
        target,
        spec.request
            .as_ref()
            .and_then(|written| written.parts.as_ref()),
    ) {
        (Some(target), Some(written)) => {
            file_parts(written, target, checked.encoding, mapped, path, refusals)
        }
        (None, Some(_)) => Err(Refused),
        (_, None) => Ok(BTreeMap::new()),
    };
    match (bare, parts, query, members) {
        (Ok(_), Ok(_), Ok(_), Ok(_)) if checked.refused || !flat || unmapped => Err(Refused),
        (Err(Refused), _, _, _)
        | (_, Err(Refused), _, _)
        | (_, _, Err(Refused), _)
        | (_, _, _, Err(Refused)) => Err(Refused),
        (Ok(None), Ok(parts), Ok(query), Ok(members))
            if parts.is_empty() && query.is_empty() && members.is_empty() =>
        {
            Ok(checked.request)
        }
        (Ok(body), Ok(parts), Ok(query), Ok(members)) => Ok(Some(Request {
            body,
            members,
            parts,
            query,
            ..checked.request.unwrap_or_default()
        })),
    }
}

/// A `json` request's `members` (`ESS-HTTP-020`, at `routes[i].request.members.<field>`), compiled
/// in the command's field order: each key an input field of the command that neither the path
/// (`mapped`) nor `request.query` (`listed`, by local name) carries; each value a member path, no
/// two the same and none an object above another; and none starting at the top-level member an
/// input field left unmapped is carried under, its wire name.
fn nested_members(
    target: &Target<'_>,
    written: &BTreeMap<String, String>,
    [mapped, listed]: [&[String]; 2],
    path: &str,
    refusals: &mut Refusals,
) -> Result<Members, Refused> {
    let at = |field: &str| format!("{path}.request.members.{field}");
    let mut refused = false;
    for key in written.keys() {
        if !target.fields.iter().any(|field| field.name == *key) {
            refusals.refuse(
                "ESS-HTTP-020",
                at(key),
                format!(
                    "`{key}` names no input field: `members` maps an input field of the command to \
                     the member of the body it is read from"
                ),
            );
            refused = true;
        }
    }
    let carried = |name: &str| {
        mapped.contains(&format!("input.{name}")) || listed.iter().any(|field| field == name)
    };
    // The top-level members the input fields left unmapped are carried under.
    let unmapped: Vec<(&str, &str)> = target
        .fields
        .iter()
        .filter(|field| !written.contains_key(&field.name) && !carried(&field.name))
        .map(|field| {
            (
                field.name.as_str(),
                field.naming.wire.as_deref().unwrap_or(&field.name),
            )
        })
        .collect();
    let mut members: Vec<(String, MemberPath)> = Vec::new();
    for field in target.fields {
        let Some(text) = written.get(&field.name) else {
            continue;
        };
        let parsed = MemberPath::parse(text);
        let rule = if mapped.contains(&format!("input.{}", field.name)) {
            Some(format!(
                "`{}` is carried by the path already, and a field travels in one place",
                field.name
            ))
        } else if listed.contains(&field.name) {
            Some(format!(
                "`{}` is listed in `request.query` already, and a field travels in one place",
                field.name
            ))
        } else if let Some(parsed) = &parsed {
            members
                .iter()
                .find(|(_, other)| other.covers(parsed) || parsed.covers(other))
                .map(|(other, path)| {
                    if path == parsed {
                        format!(
                            "`{text}` is the member `{other}` is read from already: two fields \
                             never share one member"
                        )
                    } else {
                        format!(
                            "`{text}` and `{path}`, the member `{other}` is read from, are one an \
                             object above the other: a member holds one input field's value or \
                             members, never both"
                        )
                    }
                })
                .or_else(|| {
                    unmapped
                        .iter()
                        .find(|(_, wire)| parsed.top() == *wire)
                        .map(|(other, wire)| {
                            format!(
                                "`{text}` starts at the top-level member `{wire}`, which carries \
                                 the input field `{other}` under its wire name"
                            )
                        })
                })
        } else {
            Some(format!(
                "`{text}` is not a member path: one or more member names joined by `.`, and a name \
                 is never empty"
            ))
        };
        match (rule, parsed) {
            (Some(rule), _) => {
                refusals.refuse("ESS-HTTP-020", at(&field.name), rule);
                refused = true;
            }
            (None, Some(parsed)) => members.push((field.name.clone(), parsed)),
            (None, None) => refused = true,
        }
    }
    if refused {
        Err(Refused)
    } else {
        Ok(Members(members))
    }
}

/// A `multipart` request's `parts` (`ESS-HTTP-036`), compiled: only beside `multipart`; each key an
/// input field of type `Bytes` (a newtype of it, or an `Optional` of either) stating at least one
/// attribute; `filename` and `media_type` each `input.<field>` naming a `String` input field (a
/// newtype of it, or an `Optional` of either) that neither the path, a part nor another attribute
/// carries; `media_types` distinct RFC 6838 `type/subtype` names, at least one.
fn file_parts(
    written: &BTreeMap<String, PartSpec>,
    target: &Target<'_>,
    encoding: Option<Encoding>,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> Result<BTreeMap<String, Part>, Refused> {
    match encoding {
        Some(Encoding::Multipart) => {}
        Some(other) => {
            refusals.refuse(
                "ESS-HTTP-036",
                format!("{path}.request.parts"),
                format!(
                    "`parts` states the attributes of a `multipart` request's file parts, and this \
                     request is encoded `{other}`"
                ),
            );
            return Err(Refused);
        }
        // The encoding was refused by its own key, which leaves the parts unchecked.
        None => return Err(Refused),
    }
    let mut compiled = BTreeMap::new();
    let mut read: Vec<&str> = Vec::new();
    let mut refused = false;
    for (key, spec) in written {
        let at = format!("{path}.request.parts.{key}");
        let field = target.fields.iter().find(|field| field.name == *key);
        let rule = match field {
            None => Some(format!(
                "`{key}` names no input field: a part's attributes are keyed by the `Bytes` input \
                 field the part carries"
            )),
            Some(field) if !of_primitive(&field.type_ref, Primitive::Bytes, target.ir) => {
                Some(format!(
                    "input field `{key}` has type `{}`, and a file part carries a `Bytes` input \
                     field (a newtype of it, or an `Optional` of either)",
                    field.type_ref
                ))
            }
            Some(_) if spec == &PartSpec::default() => Some(format!(
                "`{key}` states nothing: a part's entry states a `filename`, a `media_type` or \
                 `media_types`, and a file part without one needs no entry"
            )),
            Some(_) => None,
        };
        if let Some(rule) = rule {
            refusals.refuse("ESS-HTTP-036", at, rule);
            refused = true;
            continue;
        }
        let mut part = Part::default();
        for (attribute, text, slot) in [
            ("filename", &spec.filename, &mut part.filename),
            ("media_type", &spec.media_type, &mut part.media_type),
        ] {
            let Some(text) = text else {
                continue;
            };
            match attribute_source(text, target, written, mapped, &read) {
                Ok((source, field)) => {
                    read.push(field);
                    *slot = Some(source);
                }
                Err(rule) => {
                    refusals.refuse(
                        "ESS-HTTP-036",
                        format!("{at}.{attribute}"),
                        format!("a part's `{attribute}` {rule}"),
                    );
                    refused = true;
                }
            }
        }
        if let Some(types) = &spec.media_types {
            match media_types_rule(types) {
                Some(rule) => {
                    refusals.refuse("ESS-HTTP-036", format!("{at}.media_types"), rule);
                    refused = true;
                }
                None => part.media_types.clone_from(types),
            }
        }
        compiled.insert(key.clone(), part);
    }
    if refused {
        Err(Refused)
    } else {
        Ok(compiled)
    }
}

/// The input field a part attribute's source `text` reads into, with its name, or why it may not:
/// it is `input.<field>` naming a `String` input field that neither the path, a part nor an
/// attribute already `read` carries.
fn attribute_source<'a>(
    text: &str,
    target: &Target<'a>,
    parts: &BTreeMap<String, PartSpec>,
    mapped: &[String],
    read: &[&str],
) -> Result<(Source, &'a str), String> {
    let source = Source::parse(text)
        .filter(|source| matches!(source, Source::Input(_)))
        .ok_or_else(|| {
            format!(
                "is `{text}`, which is not `input.<field>`: it is read into one input field of \
                 the command"
            )
        })?;
    let field = target
        .fields
        .iter()
        .find(|field| field.name == source.field())
        .ok_or_else(|| format!("is `{text}`, which names no input field of the command"))?;
    if !of_primitive(&field.type_ref, Primitive::String, target.ir) {
        return Err(format!(
            "is `{text}`, of type `{}`: it is read into a `String` input field (a newtype of it, \
             or an `Optional` of either)",
            field.type_ref
        ));
    }
    let carrier = if mapped.contains(&format!("input.{}", field.name)) {
        Some("the path carries it")
    } else if parts.contains_key(&field.name) {
        Some("it is a part")
    } else if read.contains(&field.name.as_str()) {
        Some("another attribute reads it, so it would be read twice")
    } else {
        None
    };
    match carrier {
        Some(carrier) => Err(format!(
            "is `{text}`, and {carrier}: an input field travels in one place"
        )),
        None => Ok((source, field.name.as_str())),
    }
}

/// Why a part's `media_types` is refused, if it is: empty, an entry that is not an RFC 6838
/// `type/subtype`, or one entry twice.
fn media_types_rule(types: &[String]) -> Option<String> {
    if types.is_empty() {
        return Some(
            "`media_types` is empty: it lists the media types the part may carry, at least one, \
             and is left out to admit any"
                .to_owned(),
        );
    }
    if let Some(bad) = types.iter().find(|text| !is_media_type(text)) {
        return Some(format!(
            "`{bad}` is not an RFC 6838 media type `type/subtype`, such as `text/csv`"
        ));
    }
    types
        .iter()
        .enumerate()
        .find(|(index, text)| types[..*index].contains(text))
        .map(|(_, text)| format!("`media_types` names `{text}` twice"))
}

/// Whether `type_ref` is the primitive `wanted`, a newtype of it, or an `Optional` of either.
fn of_primitive(type_ref: &ResolvedTypeRef, wanted: Primitive, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Optional { of } => of_primitive_value(of, wanted, ir),
        _ => of_primitive_value(type_ref, wanted, ir),
    }
}

/// Whether `type_ref` is the primitive `wanted` or a newtype of it, however deep.
fn of_primitive_value(type_ref: &ResolvedTypeRef, wanted: Primitive, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => *name == wanted,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => of_primitive_value(of, wanted, ir),
            _ => false,
        },
        _ => false,
    }
}

/// The input field a route's `request.body` names (`text`), once it is held to its rules
/// (`ESS-HTTP-020`, at `routes[i].request.body`): it is `input.<field>` naming an input field of
/// the command that is not `Optional` and that the path does not carry, and the path (`mapped`) or
/// `request.query` (`listed`, by local name) carries every other input field. A body that names no
/// such field is the only refusal reported for it.
///
/// On an `open` route, one without a path yet, whether the path carries every other input field
/// is the document's to answer that fills the path in: a protocol binding may fix the body of a
/// route whose path each server binding chooses, and the server binding, restating the body
/// (`ESS-HTTP-012`), is held to it.
fn bare_body(
    target: &Target<'_>,
    text: &str,
    [mapped, listed]: [&[String]; 2],
    open: bool,
    path: &str,
    refusals: &mut Refusals,
) -> Result<Option<Source>, Refused> {
    let at = format!("{path}.request.body");
    let source = Source::parse(text).filter(|source| matches!(source, Source::Input(_)));
    let field = source.as_ref().and_then(|source| {
        target
            .fields
            .iter()
            .find(|field| field.name == source.field())
    });
    let rule = match (&source, field) {
        (None, _) => Some(format!(
            "`{text}` is not `input.<field>`: a bare request body is one input field of the \
             command, named by its namespace and its name"
        )),
        (Some(source), None) => Some(format!(
            "`{text}` names no input field: the command declares no input field `{}`",
            source.field()
        )),
        (Some(_), Some(field)) if matches!(field.type_ref, ResolvedTypeRef::Optional { .. }) => {
            Some(format!(
                "`{text}` has type `{}`, and a bare body is always present: it is one input \
                 field that is not `Optional`",
                field.type_ref
            ))
        }
        (Some(_), Some(field)) if mapped.contains(&format!("input.{}", field.name)) => {
            Some(format!(
                "`{text}` is carried by the path already, and a field travels in one place: the \
                 body is an input field `{}` does not map to a variable of the path",
                field.name
            ))
        }
        _ => None,
    };
    if let Some(rule) = rule {
        refusals.refuse("ESS-HTTP-020", at, rule);
        return Err(Refused);
    }
    let Some(source) = source else {
        return Err(Refused);
    };
    let mut refused = false;
    for other in target.fields.iter().filter(|_| !open) {
        if other.name == source.field()
            || mapped.contains(&format!("input.{}", other.name))
            || listed.contains(&other.name)
        {
            continue;
        }
        refusals.refuse(
            "ESS-HTTP-020",
            at.as_str(),
            format!(
                "input field `{}` is in neither the path, the query nor the body: the path or \
                 `request.query` carries every input field but the body `{text}`, so map `{}` to a \
                 variable of the path or list it in `request.query`",
                other.name, other.name
            ),
        );
        refused = true;
    }
    if refused {
        Err(Refused)
    } else {
        Ok(Some(source))
    }
}

/// What a route's `request` resolved to.
struct Checked {
    /// Whether it earned a refusal (`ESS-HTTP-020`).
    refused: bool,
    /// The compiled request when it earned none, without its `body`; `None` when it is all
    /// defaults.
    request: Option<Request>,
    /// The encoding, when it is one of the set and admitted for the route's method; `None` when it
    /// was refused, which leaves the input it would carry unchecked.
    encoding: Option<Encoding>,
    /// `request.body` as written, when the encoding and the method admit one; the input field it
    /// names is [`bare_body`]'s to check.
    body: Option<String>,
    /// `request.query` as written, when the encoding and the method admit one; the input fields it
    /// lists are [`query_fields`]'s to check.
    query: Option<Vec<String>>,
    /// `request.members` as written, when a `json` body off `get` without a bare `body` carries
    /// them; the fields and paths are [`nested_members`]'s to check.
    members: Option<BTreeMap<String, String>>,
}

/// Why `encoding` is refused on a command route on `method` (`ESS-HTTP-020`), if it is: a body
/// encoding `written` on `get`, or `query` on a method other than `get` and `delete`.
fn encoding_rule(method: Option<Method>, encoding: Encoding, written: bool) -> Option<String> {
    match (method, encoding) {
        (Some(Method::Get), Encoding::Json | Encoding::Form | Encoding::Multipart) if written => {
            Some(format!(
                "`{encoding}` carries the input in a body, and `get` carries none; a command on \
                 `get` is encoded `query`, or leaves `encoding` out and maps every input field to \
                 the path"
            ))
        }
        (Some(method), Encoding::Query) if !matches!(method, Method::Get | Method::Delete) => {
            Some(format!(
                "`query` carries the input in the query string, the encoding of a command on \
                 `get` or `delete` only; a command on `{method}` is encoded `json`, `form` or \
                 `multipart`"
            ))
        }
        _ => None,
    }
}

/// Holds a route's `request` to its rules (`ESS-HTTP-020`): no `request` on a view, an encoding of
/// the set that the method admits when one is written (`query` on `get` or `delete` only, and on
/// `get` nothing else written), an `unknown` of the set, `ignore` beside no bare `body`,
/// and a `body` only beside `json` off `get` ([`view_request`], [`admitted_body`]). An absent
/// `request` is a `json` body refusing an unknown key, which a `get` command is in stage 1.
fn request(
    spec: &RouteSpec,
    operation: &Operation,
    method: Option<Method>,
    path: &str,
    refusals: &mut Refusals,
) -> Checked {
    let Some(written) = &spec.request else {
        return Checked {
            refused: false,
            request: None,
            encoding: Some(Encoding::Json),
            body: None,
            query: None,
            members: None,
        };
    };
    if let Operation::View(_) = operation {
        return view_request(written, operation, path, refusals);
    }
    let mut refused = false;
    let parsed = match written.encoding.as_deref() {
        None => Some(Encoding::default()),
        Some(text) => {
            let parsed = Encoding::parse(text);
            if parsed.is_none() {
                refusals.refuse(
                    "ESS-HTTP-020",
                    format!("{path}.request.encoding"),
                    format!(
                        "`{text}` is not an encoding; a request is encoded `json`, `form`, `query` \
                         or `multipart`"
                    ),
                );
                refused = true;
            }
            parsed
        }
    };
    let encoding = parsed.filter(|&encoding| {
        let Some(rule) = encoding_rule(method, encoding, written.encoding.is_some()) else {
            return true;
        };
        refusals.refuse("ESS-HTTP-020", format!("{path}.request.encoding"), rule);
        refused = true;
        false
    });
    let unknown = written_unknown(written, path, refusals);
    refused |= unknown.is_none();
    if parsed == Some(Encoding::Json) && written.body.is_some() && unknown == Some(Unknown::Ignore)
    {
        refusals.refuse(
            "ESS-HTTP-020",
            format!("{path}.request.unknown"),
            "`ignore` admits a member no input field names, and a bare body is one input field's \
             value, which holds no member of the input to ignore; `unknown` is `refuse` beside \
             `body`",
        );
        refused = true;
    }
    let body = written
        .body
        .as_deref()
        .and_then(|text| admitted_body(text, method, parsed, path, refusals));
    refused |= written.body.is_some() && body.is_none();
    let query = written
        .query
        .as_ref()
        .and_then(|listed| admitted_query(listed, method, parsed, path, refusals));
    refused |= written.query.is_some() && query.is_none();
    let members = written.members.as_ref().and_then(|members| {
        admitted_members(
            members,
            method,
            parsed,
            written.body.is_some(),
            path,
            refusals,
        )
    });
    refused |= written.members.is_some() && members.is_none();
    let request = match (refused, encoding, unknown) {
        (false, Some(encoding), Some(unknown)) => Some(Request {
            body: None,
            encoding,
            members: Members::default(),
            parts: BTreeMap::new(),
            query: Vec::new(),
            unknown,
        })
        .filter(|request| *request != Request::default()),
        _ => None,
    };
    Checked {
        refused,
        request,
        // A refused body leaves the input it would carry unchecked, as a refused encoding does.
        encoding: encoding.filter(|_| body.is_some() || written.body.is_none()),
        body,
        query,
        members,
    }
}

/// A request's `unknown`, its default when left out; `None` when it is outside the set, refused at
/// `routes[i].request.unknown` (`ESS-HTTP-020`).
fn written_unknown(written: &RequestSpec, path: &str, refusals: &mut Refusals) -> Option<Unknown> {
    let Some(text) = written.unknown.as_deref() else {
        return Some(Unknown::default());
    };
    let unknown = Unknown::parse(text);
    if unknown.is_none() {
        refusals.refuse(
            "ESS-HTTP-020",
            format!("{path}.request.unknown"),
            format!(
                "`{text}` is not what a request does with an unknown parameter; it is `refuse` or \
                 `ignore`"
            ),
        );
    }
    unknown
}

/// `request.members` as written (`members`), when a `json` body off `get` without a bare `body`
/// carries them; otherwise refused at `routes[i].request.members.<field>`, or at
/// `routes[i].request.members` when written empty (`ESS-HTTP-020`), and `None`. An encoding outside
/// the set (`parsed` `None`) is refused by its own key, and leaves the members to
/// [`nested_members`].
fn admitted_members(
    members: &BTreeMap<String, String>,
    method: Option<Method>,
    parsed: Option<Encoding>,
    bare: bool,
    path: &str,
    refusals: &mut Refusals,
) -> Option<BTreeMap<String, String>> {
    if members.is_empty() {
        refusals.refuse(
            "ESS-HTTP-020",
            format!("{path}.request.members"),
            "`members` is empty: it maps at least one input field to a member of the body, and is \
             left out to read every field from a top-level member",
        );
        return None;
    }
    let rule = match (method, parsed) {
        (_, Some(encoding @ (Encoding::Form | Encoding::Query | Encoding::Multipart))) => format!(
            "`members` reads input fields from members of a `json` body, and a `{encoding}` request \
             carries every field under its own key; a route with `members` is encoded `json`"
        ),
        _ if bare => "`members` reads input fields from members of the input object, and a bare \
                      `body` is one input field's value, which holds no member of the input"
            .to_owned(),
        (Some(Method::Get), _) => "`members` reads input fields from members of a `json` body, \
                                   and `get` carries none; a command on `get` is encoded `query`"
            .to_owned(),
        _ => return Some(members.clone()),
    };
    for field in members.keys() {
        refusals.refuse(
            "ESS-HTTP-020",
            format!("{path}.request.members.{field}"),
            rule.as_str(),
        );
    }
    None
}

/// `request.query` as written (`listed`), when a body encoding off `get` sits beside it; otherwise
/// refused at `routes[i].request.query` (`ESS-HTTP-020`) and `None`. An encoding outside the set
/// (`parsed` `None`) is refused by its own key, and leaves the list to [`query_fields`].
fn admitted_query(
    listed: &[String],
    method: Option<Method>,
    parsed: Option<Encoding>,
    path: &str,
    refusals: &mut Refusals,
) -> Option<Vec<String>> {
    let rule = match (method, parsed) {
        (_, Some(Encoding::Query)) => {
            "`query` lists the input fields read from the query string beside a body, and a \
             request encoded `query` carries every field there already; list none, or encode the \
             body `json`, `form` or `multipart`"
                .to_owned()
        }
        (Some(Method::Get), _) => {
            "`query` lists the input fields read from the query string beside a body, and `get` \
             carries none; a command on `get` is encoded `query`"
                .to_owned()
        }
        _ if listed.is_empty() => {
            "`query` is empty: it lists at least one input field, and is left out to read none \
             from the query string"
                .to_owned()
        }
        _ => return Some(listed.to_vec()),
    };
    refusals.refuse("ESS-HTTP-020", format!("{path}.request.query"), rule);
    None
}

/// The input fields a route's `request.query` lists (`listed`), once each is held to its rules
/// (`ESS-HTTP-020` and `ESS-HTTP-021`, at `routes[i].request.query`): `input.<field>` naming an
/// input field of the command, listed once, that neither the path, the bare `body` nor a part of a
/// `multipart` request carries, of a type with a flat spelling.
fn query_fields(
    listed: &[String],
    target: &Target<'_>,
    written: Option<&RequestSpec>,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> Result<Vec<Source>, Refused> {
    let at = format!("{path}.request.query");
    let field_of = |text: &str| {
        Source::parse(text)
            .filter(|source| matches!(source, Source::Input(_)))
            .map(|source| source.field().to_owned())
    };
    let body = written
        .and_then(|request| request.body.as_deref())
        .and_then(field_of);
    let parts = written
        .and_then(|request| request.parts.as_ref())
        .map(|parts| {
            let mut read: Vec<String> = parts.keys().cloned().collect();
            read.extend(
                parts
                    .values()
                    .flat_map(|part| [&part.filename, &part.media_type])
                    .flatten()
                    .filter_map(|text| field_of(text)),
            );
            read
        })
        .unwrap_or_default();
    let mut sources: Vec<Source> = Vec::new();
    let mut refused = false;
    for text in listed {
        let source = Source::parse(text).filter(|source| matches!(source, Source::Input(_)));
        let field = source.as_ref().and_then(|source| {
            target
                .fields
                .iter()
                .find(|field| field.name == source.field())
        });
        let rule = match (&source, field) {
            (None, _) => Some((
                "ESS-HTTP-020",
                format!(
                    "`{text}` is not `input.<field>`: `query` lists input fields of the command, \
                     each named by its namespace and its name"
                ),
            )),
            (Some(source), None) => Some((
                "ESS-HTTP-020",
                format!(
                    "`{text}` names no input field: the command declares no input field `{}`",
                    source.field()
                ),
            )),
            (Some(source), Some(_)) if sources.contains(source) => Some((
                "ESS-HTTP-020",
                format!("`{text}` is listed twice, and a field is read from the query string once"),
            )),
            (Some(_), Some(field)) if mapped.contains(&format!("input.{}", field.name)) => Some((
                "ESS-HTTP-020",
                format!(
                    "`{text}` is carried by the path already, and a field travels in one place"
                ),
            )),
            (Some(_), Some(field)) if body.as_deref() == Some(field.name.as_str()) => Some((
                "ESS-HTTP-020",
                format!("`{text}` is the bare `body` already, and a field travels in one place"),
            )),
            (Some(_), Some(field)) if parts.contains(&field.name) => Some((
                "ESS-HTTP-020",
                format!(
                    "`{text}` is read from a part of the `multipart` request already, and a field \
                 travels in one place"
                ),
            )),
            (Some(_), Some(field)) if !flat_type(&field.type_ref, target.ir) => Some((
                "ESS-HTTP-021",
                format!(
                    "input field `{}` has type `{}`, which has no flat spelling in the query \
                     string: a field `query` lists is a scalar, an `Optional` scalar or a `List` \
                     of scalars, a scalar being a primitive other than `Json`, an enum, or a \
                     newtype of a scalar",
                    field.name, field.type_ref
                ),
            )),
            _ => None,
        };
        if let Some((code, rule)) = rule {
            refusals.refuse(code, at.as_str(), rule);
            refused = true;
            continue;
        }
        if let Some(source) = source {
            sources.push(source);
        }
    }
    if refused {
        Err(Refused)
    } else {
        Ok(sources)
    }
}

/// A view's `request`, always refused (`ESS-HTTP-020`): at `routes[i].request.body` when it
/// writes a body, at `routes[i].request` otherwise.
fn view_request(
    written: &RequestSpec,
    operation: &Operation,
    path: &str,
    refusals: &mut Refusals,
) -> Checked {
    let refused = Checked {
        refused: true,
        request: None,
        encoding: None,
        body: None,
        query: None,
        members: None,
    };
    if let (None, Some(members)) = (&written.body, &written.members) {
        if !members.is_empty() {
            for field in members.keys() {
                refusals.refuse(
                    "ESS-HTTP-020",
                    format!("{path}.request.members.{field}"),
                    format!(
                        "{operation} takes no request body (`members`): a view reads its \
                         parameters from the path and the query string"
                    ),
                );
            }
            return refused;
        }
    }
    let (at, rule) = match &written.body {
        Some(body) => (
            format!("{path}.request.body"),
            format!(
                "{operation} takes no request body (`{body}`): a view reads its parameters from \
                 the path and the query string"
            ),
        ),
        None => (
            format!("{path}.request"),
            format!(
                "{operation} takes no `request`: a view reads its parameters from the path and \
                 the query string"
            ),
        ),
    };
    refusals.refuse("ESS-HTTP-020", at, rule);
    refused
}

/// `request.body` as written (`text`), when a `json` body off `get` carries it; otherwise refused
/// at `routes[i].request.body` (`ESS-HTTP-020`) and `None`. An encoding outside the set (`parsed`
/// `None`) is refused by its own key, and leaves the body to [`bare_body`].
fn admitted_body(
    text: &str,
    method: Option<Method>,
    parsed: Option<Encoding>,
    path: &str,
    refusals: &mut Refusals,
) -> Option<String> {
    let rule = match (method, parsed) {
        (_, Some(encoding @ (Encoding::Form | Encoding::Query))) => format!(
            "`body: {text}` carries one input field as a bare `json` body, and a `{encoding}` \
             request carries every field under its own key; a route with a bare body is encoded \
             `json`"
        ),
        (Some(Method::Get), Some(Encoding::Json)) => format!(
            "`body: {text}` carries one input field as the request body, and `get` carries none; \
             a command with a bare body is bound to `post`, `put`, `patch` or `delete`"
        ),
        _ => return Some(text.to_owned()),
    };
    refusals.refuse("ESS-HTTP-020", format!("{path}.request.body"), rule);
    None
}

/// Refuses each input field a `form` or `query` request carries, every one the path does not map,
/// whose type has no flat spelling (`ESS-HTTP-021`), and says whether every field has one.
fn flat_input(
    target: &Target<'_>,
    encoding: Encoding,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> bool {
    let mut flat = true;
    for field in target.fields {
        if mapped.contains(&format!("input.{}", field.name))
            || flat_type(&field.type_ref, target.ir)
        {
            continue;
        }
        refusals.refuse(
            "ESS-HTTP-021",
            format!("{path}.request.encoding"),
            format!(
                "input field `{}` has type `{}`, which has no flat spelling in a `{encoding}` \
                 request: a `form`, `query` or `multipart` field is a scalar, an `Optional` \
                 scalar or a `List` of scalars, a scalar being a primitive other than `Json`, an enum, or a newtype \
                 of a scalar",
                field.name, field.type_ref
            ),
        );
        flat = false;
    }
    flat
}

/// Whether a value of `type_ref` is spelled as `form` and `query` parameters: one key per value, a
/// `List` repeating its key, an absent `Optional` leaving it out. Never a struct, a union, a map,
/// `Json`, an `Optional` or `List` of anything but a scalar.
fn flat_type(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => scalar(of, ir),
        _ => scalar(type_ref, ir),
    }
}

/// Whether `type_ref` is one scalar value: a primitive other than `Json`, an enum, or a newtype of
/// a scalar, constrained or not.
fn scalar(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => *name != Primitive::Json,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Enum { .. } => true,
            ResolvedBody::Newtype { of, .. } => scalar(of, ir),
            ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => false,
        },
        ResolvedTypeRef::Optional { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => false,
    }
}

/// Refuses each input field of a command on `get` that no variable of the path maps
/// (`ESS-HTTP-014`), and says whether any was. A `get` command whose request is encoded `query`
/// carries its other fields in the query string and is not checked here.
fn get_input(
    operation: &Operation,
    target: &Target<'_>,
    mapped: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> bool {
    let mut unmapped = false;
    for field in target.fields {
        let source = format!("input.{}", field.name);
        if !mapped.contains(&source) {
            refusals.refuse(
                "ESS-HTTP-014",
                path,
                format!(
                    "command `{}` is bound to `get`, which carries no body, and its input field \
                     `{}` is not mapped to the path; map it, or encode the request `query`",
                    operation.name(),
                    field.name
                ),
            );
            unmapped = true;
        }
    }
    unmapped
}

/// What a route answers, compiled: a command's listed outcomes, a view's body and empty answer.
struct Answers {
    outcomes: BTreeMap<String, Outcome>,
    errors: Option<Outcome>,
    alike: Vec<Vec<String>>,
    body: ViewAnswer,
    empty: Option<Empty>,
    representations: Vec<Representation>,
}

/// The route's answers once every key of them is checked: a command's `outcomes` (`ESS-HTTP-022`
/// to `-025`) and the view keys it may not carry (`ESS-HTTP-024`, `-030`), or a view's
/// `outcomes`, `body` and `empty` (`ESS-HTTP-022`, `-024`, `-030`, `-031`), and the
/// route's `representations` against the path's `variables` (`ESS-HTTP-035`, `-026`).
fn answers(
    spec: &RouteSpec,
    operation: &Operation,
    target: Option<&Target<'_>>,
    path: &str,
    variables: &[String],
    standard: &Standing,
    refusals: &mut Refusals,
) -> Result<Answers, Refused> {
    let before = refusals.0.len();
    let mut answers = match operation {
        Operation::Command(_) => {
            let command = target.and_then(|target| match target.served {
                Served::Command(command) => Some((target.ir, command)),
                Served::View(_) => None,
            });
            let (outcomes, errors, alike) =
                outcomes(spec, operation, command, path, standard, refusals);
            if let Some(body) = &spec.body {
                refusals.refuse(
                    "ESS-HTTP-024",
                    format!("{path}.body"),
                    format!(
                        "`body: {body}` is a view route's key, and {operation} answers per \
                         outcome, under `outcomes`"
                    ),
                );
            }
            if spec.empty.is_some() {
                refusals.refuse(
                    "ESS-HTTP-030",
                    path,
                    format!(
                        "`empty` answers a `single` view that holds no row, and {operation} has \
                         no `single` body"
                    ),
                );
            }
            Answers {
                outcomes,
                errors,
                alike,
                body: ViewAnswer::default(),
                empty: None,
                representations: Vec::new(),
            }
        }
        Operation::View(_) => {
            let view = target.and_then(|target| match target.served {
                Served::View(view) => Some((target.ir, view)),
                Served::Command(_) => None,
            });
            view_answers(spec, operation, view, path, refusals)
        }
    };
    if let Ok(representations) = representations(spec, operation, target, variables, path, refusals)
    {
        answers.representations = representations;
    }
    if refusals.0.len() == before {
        Ok(answers)
    } else {
        Err(Refused)
    }
}

/// A command's listed outcomes and its `errors`, each checked: a key naming an outcome of the
/// command, once (`ESS-HTTP-022`), a status it may answer (`ESS-HTTP-023`) and a body it admits
/// (`ESS-HTTP-024`); `errors` held to the same checks for every outcome it covers, and refused
/// where it covers none (`ESS-HTTP-022`); then the outcomes no body tells apart on one status
/// (`ESS-HTTP-025`). Without the command, only the shape of each key is known, and nothing is
/// checked against it.
fn outcomes(
    spec: &RouteSpec,
    operation: &Operation,
    command: Option<(&EssIr, &ResolvedCommand)>,
    path: &str,
    standard: &Standing,
    refusals: &mut Refusals,
) -> (BTreeMap<String, Outcome>, Option<Outcome>, Vec<Vec<String>>) {
    let mut compiled = BTreeMap::new();
    let Some((ir, command)) = command else {
        return (compiled, None, Vec::new());
    };
    // No `outcomes` lists none: every outcome keeps its derived answer, which a `refusals:`
    // override is still held apart from.
    let none = Entries::default();
    let written = spec.outcomes.as_ref().unwrap_or(&none);
    // Each listed outcome that compiled, in document order.
    let mut listed: Vec<&str> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (name, outcome_spec) in &written.0 {
        let at = format!("{path}.outcomes.{name}");
        if !seen.insert(name.as_str()) {
            refusals.refuse(
                "ESS-HTTP-022",
                at,
                format!("outcome `{name}` is listed twice; each outcome is listed once"),
            );
            continue;
        }
        let Some(outcome) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == name)
        else {
            let declared: Vec<String> = command
                .outcomes
                .iter()
                .map(|outcome| format!("`{}`", outcome.name))
                .collect();
            refusals.refuse(
                "ESS-HTTP-022",
                at,
                format!(
                    "`{name}` is not an outcome of {operation}, whose outcomes are {}",
                    declared.join(", ")
                ),
            );
            continue;
        };
        if let Some(answer) = outcome_answer(outcome_spec, ir, command, outcome, &at, refusals) {
            compiled.insert(name.clone(), answer);
            listed.push(name);
        }
    }
    let (errors, covered) = errors(spec, operation, ir, command, &seen, path, refusals);
    let groups = spec.alike.as_deref().unwrap_or_default();
    let declared = Declared {
        compiled: &compiled,
        listed: &listed,
        written,
        errors: errors.as_ref(),
        covered: &covered,
        alike: groups,
    };
    let alike = alike(operation, ir, command, &declared, path, refusals);
    distinguishable(operation, ir, command, &declared, path, standard, refusals);
    (compiled, errors, alike)
}

/// A route's `alike`, checked: each group two or more outcomes of the command (`ESS-HTTP-022`),
/// none in two groups or twice in one, each with a declared answer, listed under `outcomes` or
/// covered by `errors` (`ESS-HTTP-022`); and in each group one answer the wire cannot tell apart
/// (`ESS-HTTP-025`): the same status, body, encoding, headers and redirect, never `envelope`,
/// whose `outcome` member names the outcome, never `{<member>: error}` under two error wire names
/// or over errors whose fields differ (one always writing a field the other never writes, or
/// differing otherwise, [`fields_of`]), never a header reading `error.wire` under two error wire
/// names, and never redirects whose parameters tell them apart. A member whose declared answer was
/// refused is refused for that alone. Every refusal is at `routes[i].alike`, one per name or per
/// group. Returns the groups as compiled IR writes them: names in name order, groups ordered by
/// their first name.
fn alike(
    operation: &Operation,
    ir: &EssIr,
    command: &ResolvedCommand,
    declared: &Declared<'_>,
    path: &str,
    refusals: &mut Refusals,
) -> Vec<Vec<String>> {
    let at = format!("{path}.alike");
    let written: std::collections::BTreeSet<&str> = declared
        .written
        .0
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let mut placed: BTreeMap<&str, usize> = BTreeMap::new();
    let mut compiled: Vec<Vec<String>> = Vec::new();
    for (index, group) in declared.alike.iter().enumerate() {
        let shown = format!("`alike[{index}]` (`[{}]`)", group.join(", "));
        if group.len() < 2 {
            refusals.refuse(
                "ESS-HTTP-022",
                at.as_str(),
                format!(
                    "{shown} names {} outcome; a group of outcomes answered alike names two or \
                     more",
                    if group.is_empty() { "no" } else { "one" }
                ),
            );
        }
        let mut members: Vec<(&ResolvedOutcome, &Outcome)> = Vec::new();
        let mut whole = group.len() >= 2;
        for name in group {
            let Some(outcome) = command
                .outcomes
                .iter()
                .find(|outcome| outcome.name.as_str() == name)
            else {
                let outcomes: Vec<String> = command
                    .outcomes
                    .iter()
                    .map(|outcome| format!("`{}`", outcome.name))
                    .collect();
                refusals.refuse(
                    "ESS-HTTP-022",
                    at.as_str(),
                    format!(
                        "{shown} names `{name}`, which is not an outcome of {operation}, whose \
                         outcomes are {}",
                        outcomes.join(", ")
                    ),
                );
                whole = false;
                continue;
            };
            if let Some(first) = placed.insert(name.as_str(), index) {
                let why = if first == index {
                    format!("{shown} names `{name}` twice; a group names each outcome once")
                } else {
                    format!(
                        "`{name}` is in two groups, `alike[{first}]` and {shown}; an outcome is \
                         answered alike with one group, so two groups sharing one are one group"
                    )
                };
                refusals.refuse("ESS-HTTP-022", at.as_str(), why);
                whole = false;
                continue;
            }
            match declared.answer(name) {
                Some(answer) => members.push((outcome, answer)),
                // Its declared answer was refused, for itself.
                None if written.contains(name.as_str())
                    || declared.covered.contains(&name.as_str()) =>
                {
                    whole = false;
                }
                None => {
                    let unless = if outcome.error.is_some() {
                        ", and the route declares no `errors` to cover it"
                    } else {
                        ", and it reports no error for `errors` to cover"
                    };
                    refusals.refuse(
                        "ESS-HTTP-022",
                        at.as_str(),
                        format!(
                            "{shown} names `{name}`, which has no declared answer: it is not \
                             listed under `outcomes`{unless}; outcomes are alike in the answer \
                             the binding declares for each"
                        ),
                    );
                    whole = false;
                }
            }
        }
        if whole {
            if let Some(reason) = alike_reason(ir, command, &members) {
                refusals.refuse("ESS-HTTP-025", at.as_str(), format!("{shown}: {reason}"));
            }
        }
        let mut names = group.clone();
        names.sort();
        compiled.push(names);
    }
    compiled.sort();
    compiled
}

/// What `{<member>: error}` writes beside its member for one outcome's error: `written`, each of
/// the error's fields by wire name with its type, in declaration order (empty for an outcome
/// without an error), `always`, the names among them every answer of the outcome writes, and
/// `kinds`, the JSON type of each always-written field whose values are all of one.
#[derive(Debug)]
struct ErrorFields<'a> {
    written: Vec<(&'a str, &'a ResolvedTypeRef)>,
    always: Vec<&'a str>,
    kinds: BTreeMap<&'a str, JsonKind>,
    /// Whether an answer of the outcome can leave every field out: the outcome is the
    /// unknown-instance answer of its command.
    unknown_instance: bool,
}

/// The one JSON type every value of an ESS type is written as, where there is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonKind {
    String,
    Boolean,
    Number,
    Array,
    Object,
}

/// The JSON type every value of `type_ref` is written as, followed through newtypes: `None` where
/// a value can be of more than one (`Json`, a union, whose unit variant is `null`) or the type
/// is `Optional`, which a field that is always written is not.
fn json_kind(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<JsonKind> {
    let mut current = type_ref;
    for _ in 0..=ir.types().len() {
        match current {
            ResolvedTypeRef::Primitive { name } => {
                return match name {
                    Primitive::Boolean => Some(JsonKind::Boolean),
                    Primitive::Integer | Primitive::Binary64 => Some(JsonKind::Number),
                    Primitive::Json => None,
                    Primitive::String
                    | Primitive::Decimal
                    | Primitive::Timestamp
                    | Primitive::Duration
                    | Primitive::Uuid
                    | Primitive::Bytes => Some(JsonKind::String),
                };
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => current = of,
                ResolvedBody::Struct { .. } => return Some(JsonKind::Object),
                ResolvedBody::Enum { .. } => return Some(JsonKind::String),
                ResolvedBody::Union { .. } => return None,
            },
            ResolvedTypeRef::Optional { .. } => return None,
            ResolvedTypeRef::List { .. } => return Some(JsonKind::Array),
            ResolvedTypeRef::Map { .. } => return Some(JsonKind::Object),
        }
    }
    None
}

/// What `{<member>: error}` writes beside its member for `outcome` of `command`.
///
/// A field is always written when its type, followed through every newtype, is not `Optional`
/// (a newtype over `Optional<…>` admits absence and every writer omits an absent value), and when
/// no answer of the outcome can be written without it. The unknown-instance answer of a command
/// (`ess_compiler::unknown_instance::unknown_instance_answer`, `docs/design/unknown-instance-seams.md`)
/// is the one that writes none of its error's fields, for an instance no record carries; the
/// function is the one `ess-gen` asks.
fn error_fields<'a>(
    ir: &'a EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> ErrorFields<'a> {
    let written: Vec<(&'a str, &'a ResolvedTypeRef)> =
        outcome.error.as_ref().map_or_else(Vec::new, |handle| {
            ir.error(handle)
                .fields
                .iter()
                .map(|field| {
                    (
                        field.naming.wire.as_deref().unwrap_or(&field.name),
                        &field.type_ref,
                    )
                })
                .collect()
        });
    let unknown_instance =
        unknown_instance_answer(ir, command).is_some_and(|answer| answer.name == outcome.name);
    let always: Vec<&'a str> = if unknown_instance {
        Vec::new()
    } else {
        written
            .iter()
            .filter(|(_, type_ref)| !admits_absence(ir, type_ref))
            .map(|(name, _)| *name)
            .collect()
    };
    let kinds = written
        .iter()
        .filter(|(name, _)| always.contains(name))
        .filter_map(|(name, type_ref)| Some((*name, json_kind(ir, type_ref)?)))
        .collect();
    ErrorFields {
        written,
        always,
        kinds,
        unknown_instance,
    }
}

/// Whether a value of `type_ref` can be absent: `Optional`, or a newtype, through any number of
/// newtypes, over one. The model has no other named type that wraps another.
fn admits_absence(ir: &EssIr, type_ref: &ResolvedTypeRef) -> bool {
    let mut current = type_ref;
    for _ in 0..=ir.types().len() {
        match current {
            ResolvedTypeRef::Optional { .. } => return true,
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => current = of,
                _ => return false,
            },
            _ => return false,
        }
    }
    false
}

/// What the fields two errors of one wire name write beside the member of `{<member>: error}` say
/// of the two bodies (`ESS-HTTP-025`).
#[derive(Debug, PartialEq, Eq)]
enum Fields<'a> {
    /// The same fields, in the same order.
    Same,
    /// One error always writes this field, by wire name and with a type that is not `Optional`,
    /// that the other never writes: the wire tells the two bodies apart.
    Deciding(&'a str),
    /// Both errors always write this field, with values of different JSON types: no body is an
    /// answer of both, and the binding still tells errors apart by which fields they write, not by
    /// a field's type.
    Typed(&'a str),
    /// The fields differ, and no always-written field decides, and no field both always write is
    /// of JSON types with no value in common. The refusal states that, and says nothing of what a
    /// body can hold.
    Undecided,
}

/// The one rule that decides whether the wire tells two `{<member>: error}` bodies of one error
/// wire name apart by the fields of their errors (`error_fields`): one error always writes a
/// field that the other never writes. The status check ([`distinguishable`]) and the `alike` check
/// ([`errors_apart`]) both call it.
fn fields_of<'a>(one: &ErrorFields<'a>, two: &ErrorFields<'a>) -> Fields<'a> {
    if one.written == two.written {
        return Fields::Same;
    }
    let deciding = |writes: &ErrorFields<'a>, other: &ErrorFields<'_>| {
        writes
            .always
            .iter()
            .find(|name| !other.written.iter().any(|(written, _)| written == *name))
            .copied()
    };
    if let Some(field) = deciding(one, two).or_else(|| deciding(two, one)) {
        return Fields::Deciding(field);
    }
    one.kinds
        .iter()
        .find(|(name, kind)| two.kinds.get(*name).is_some_and(|other| other != *kind))
        .map_or(Fields::Undecided, |(name, _)| Fields::Typed(name))
}

/// The forms that validate as written for two errors of one wire name whose fields differ and no
/// always-written field decides, as both refusals of such a pair name them. Under `alike:` only
/// the first does: a group of such a pair is refused, so the others apply with the group dropped.
const FIELD_FORMS: &str = "one error reported by both outcomes, under `alike:`; or, with \
                           the `alike:` group dropped, a field one error \
                           always writes that the other never writes, distinct error wire names, \
                           or `envelope`";

/// What both refusals state of that pair.
const NO_DECIDING_FIELD: &str =
    "their fields differ and no field one error always writes is one the other never writes";

/// What both refusals state of a pair that both write one field, with values of different JSON
/// types.
fn same_field_other_type(field: &str) -> String {
    format!(
        "both write `{field}`, with values of different JSON types, so no body is an answer of \
         both; the binding tells errors apart by which fields they write, not by a field's type"
    )
}

/// The forms that validate as written for such a pair: with `alike:` only the first does.
fn type_forms(field: &str) -> String {
    format!(
        "one error whose `{field}` has the type of the other's, reported by both outcomes, under \
         `alike:`; or, with the `alike:` group dropped, a field one error always writes that the \
         other never writes, distinct error wire names, or `envelope`"
    )
}

/// A clause for a refusal of a pair where an outcome is the unknown-instance answer of its
/// command (`error_fields`), which writes none of its error's fields; empty otherwise.
fn unknown_instance_clause(ones: &[(&str, &ErrorFields<'_>)]) -> String {
    let names: Vec<String> = ones
        .iter()
        .filter(|(_, fields)| fields.unknown_instance)
        .map(|(name, _)| format!("`{name}`"))
        .collect();
    if names.is_empty() {
        String::new()
    } else {
        format!(
            " (a `wrong_state` outcome of a command that acts on an instance its input names, \
             {}, answers an instance no record carries without its error's fields)",
            names.join(" and ")
        )
    }
}

/// Why two members of an `alike` group sharing `answer` are told apart by their errors, if they
/// are, or are not one answer: `{<member>: error}` under two error wire names or over errors one
/// of which always writes a field the other never writes, or whose fields differ otherwise; or a
/// header reading `error.wire` under two error wire names.
fn errors_apart(
    ir: &EssIr,
    command: &ResolvedCommand,
    answer: &Outcome,
    [(one, one_wire), (two, two_wire)]: [(&ResolvedOutcome, Option<&str>); 2],
) -> Option<String> {
    let error_object = matches!(answer.body, Body::Error { .. });
    if error_object && one_wire != two_wire {
        return Some(format!(
            "`{}` and `{}` answer `{}` with `{}`, whose member holds the error wire name, `{}` \
             for one and `{}` for the other, so the wire tells them apart",
            one.name,
            two.name,
            answer.status,
            answer.body,
            one_wire.unwrap_or_default(),
            two_wire.unwrap_or_default()
        ));
    }
    if error_object {
        let (ones, twos) = (
            error_fields(ir, command, one),
            error_fields(ir, command, two),
        );
        match fields_of(&ones, &twos) {
            Fields::Same => {}
            Fields::Deciding(field) => {
                return Some(format!(
                    "`{}` and `{}` answer `{}` with `{}`, which writes each error's own fields \
                     beside the member, and one error always writes `{field}`, which the other \
                     never writes, so the wire tells them apart",
                    one.name, two.name, answer.status, answer.body,
                ));
            }
            Fields::Typed(field) => {
                return Some(format!(
                    "`{}` and `{}` answer `{}` with `{}`, which writes each error's own fields \
                     beside the member, and {}, so they are not one answer; the forms that \
                     validate are {}",
                    one.name,
                    two.name,
                    answer.status,
                    answer.body,
                    same_field_other_type(field),
                    type_forms(field),
                ));
            }
            Fields::Undecided => {
                return Some(format!(
                    "`{}` and `{}` answer `{}` with `{}`, which writes each error's own fields \
                     beside the member, and {NO_DECIDING_FIELD}{}; the forms that validate are \
                     {FIELD_FORMS}",
                    one.name,
                    two.name,
                    answer.status,
                    answer.body,
                    unknown_instance_clause(&[
                        (one.name.as_str(), &ones),
                        (two.name.as_str(), &twos)
                    ]),
                ));
            }
        }
    }
    if one_wire != two_wire && reads_error_wire(&answer.headers) {
        return Some(format!(
            "`{}` and `{}` answer `{}` with a header that reads `error.wire`, `{}` for one and \
             `{}` for the other, so the wire tells them apart",
            one.name,
            two.name,
            answer.status,
            one_wire.unwrap_or_default(),
            two_wire.unwrap_or_default()
        ));
    }
    None
}

/// Whether a header, or a challenge param under one, reads `error.wire`.
fn reads_error_wire(headers: &BTreeMap<String, Header>) -> bool {
    headers.values().any(|header| match header {
        Header::Literal(_) => false,
        Header::Source(source) => *source == AnswerSource::ErrorWire,
        Header::Challenge(challenge) => challenge
            .params
            .values()
            .any(|param| *param == Param::Source(AnswerSource::ErrorWire)),
    })
}

/// Why the declared answers of one group of outcomes are not one answer the wire cannot tell
/// apart, if they are not: the first key in which an answer differs from the first member's
/// (`status`, `body`, `encoding`, `headers`, `redirect`), `envelope`, error objects under two error
/// wire names or over errors whose fields differ, a header reading `error.wire` over two error wire
/// names, or redirects whose parameters tell two members apart.
fn alike_reason(
    ir: &EssIr,
    command: &ResolvedCommand,
    members: &[(&ResolvedOutcome, &Outcome)],
) -> Option<String> {
    let (first, answer) = *members.first()?;
    for (other, theirs) in &members[1..] {
        let key = [
            ("status", answer.status != theirs.status),
            ("body", answer.body != theirs.body),
            ("encoding", answer.encoding != theirs.encoding),
            ("headers", answer.headers != theirs.headers),
            ("redirect", answer.redirect != theirs.redirect),
        ]
        .into_iter()
        .find_map(|(key, differs)| differs.then_some(key));
        if let Some(key) = key {
            return Some(format!(
                "outcomes `{}` and `{}` of `{}` are answered alike, and their declared answers \
                 differ in `{key}`: `{}` answers `{}` with `{}`, `{}` answers `{}` with `{}`; \
                 outcomes answered alike declare one answer",
                first.name,
                other.name,
                command.name,
                first.name,
                answer.status,
                answer.body,
                other.name,
                theirs.status,
                theirs.body
            ));
        }
    }
    if answer.body == Body::Envelope {
        return Some(format!(
            "every member answers `{}` with `envelope`, whose `outcome` member names each \
             outcome, so the wire tells them apart; outcomes answered alike answer a body that \
             does not",
            answer.status
        ));
    }
    let wire = |outcome: &ResolvedOutcome| {
        outcome.error.as_ref().map(|handle| {
            let error = ir.error(handle);
            error.naming.wire_or(&error.name)
        })
    };
    for (index, (one, one_answer)) in members.iter().enumerate() {
        for (two, two_answer) in &members[index + 1..] {
            let (one_wire, two_wire) = (wire(one), wire(two));
            if let Some(reason) =
                errors_apart(ir, command, answer, [(one, one_wire), (two, two_wire)])
            {
                return Some(reason);
            }
            if let (Some(one_redirect), Some(two_redirect)) =
                (&one_answer.redirect, &two_answer.redirect)
            {
                let one_written = Written::of(
                    one_redirect,
                    &Reader {
                        ir,
                        command,
                        outcome: one,
                    },
                );
                let two_written = Written::of(
                    two_redirect,
                    &Reader {
                        ir,
                        command,
                        outcome: two,
                    },
                );
                if one_written.apart(one_wire, &two_written, two_wire) {
                    return Some(format!(
                        "`{}` and `{}` redirect with parameters that tell them apart: one always \
                         writes a parameter the other never writes, or both write one from \
                         `error.wire` and their error wire names differ",
                        one.name, two.name,
                    ));
                }
            }
        }
    }
    None
}

/// One outcome's answer as `written` declares it, at `at`, unless a key of it is refused: its
/// status (`ESS-HTTP-023`), body (`ESS-HTTP-024`), headers (`ESS-HTTP-026`, `-027`) and redirect
/// (`ESS-HTTP-028`).
fn outcome_answer(
    written: &OutcomeSpec,
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    at: &str,
    refusals: &mut Refusals,
) -> Option<Outcome> {
    let status = outcome_status_checked(written.status, outcome, at, refusals);
    // A body beside a redirect is refused only where the redirect itself is not, on a redirect
    // status, so that one mistake is one refusal.
    let redirects =
        written.redirect.is_some() && status.is_some_and(|status| REDIRECTS.contains(&status));
    let body = outcome_body(
        written.body.as_ref(),
        status,
        redirects,
        &Reader {
            ir,
            command,
            outcome,
        },
        at,
        refusals,
    );
    let reader = Reader {
        ir,
        command,
        outcome,
    };
    let encoding = outcome_encoding(
        written.encoding.as_deref(),
        body.as_ref(),
        &reader,
        at,
        refusals,
    );
    let headers = answer_headers(
        written.headers.as_ref(),
        Some(&reader),
        status,
        at,
        refusals,
    );
    let redirect = outcome_redirect(written.redirect.as_ref(), status, &reader, at, refusals);
    match (status, body, encoding, headers, redirect) {
        (Some(status), Some(body), Some(encoding), Ok(headers), Ok(redirect)) => Some(Outcome {
            status,
            body,
            encoding,
            headers,
            redirect,
        }),
        _ => None,
    }
}

/// The outcome's encoding, unless it is refused (`ESS-HTTP-024`): a word outside `json` and
/// `text`, or `text` beside any body but `response.<field>` whose field is a `String` or a newtype
/// of one. `body` is the outcome's compiled body; one refused holds `text` to nothing more, so
/// that one mistake is one refusal. Under `errors` the body is never a response field, so `text`
/// is refused there for every outcome it covers.
fn outcome_encoding(
    written: Option<&str>,
    body: Option<&Body>,
    reader: &Reader<'_>,
    at: &str,
    refusals: &mut Refusals,
) -> Option<ResponseEncoding> {
    let Some(text) = written else {
        return Some(ResponseEncoding::Json);
    };
    let key = format!("{at}.encoding");
    let Some(encoding) = ResponseEncoding::parse(text) else {
        refusals.refuse(
            "ESS-HTTP-024",
            key,
            format!(
                "`{text}` is not an encoding; an outcome's body is encoded `json` or `text`, in \
                 lower case"
            ),
        );
        return None;
    };
    let reason = match (encoding, body) {
        (ResponseEncoding::Json, _) | (_, None) => None,
        (ResponseEncoding::Text, Some(Body::ResponseField(field))) => {
            match declared_type(&reader.command.response, field) {
                Some(type_ref) if !string_type(type_ref, reader.ir) => Some(format!(
                    "`encoding: text` writes the characters of a `String` response field, and \
                     `response.{field}` has type `{type_ref}`"
                )),
                _ => None,
            }
        }
        (ResponseEncoding::Text, Some(body)) => Some(format!(
            "`encoding: text` writes the characters of one `String` response field, \
             `response.<field>`, and outcome `{}` answers `{body}`",
            reader.outcome.name
        )),
    };
    match reason {
        Some(reason) => {
            refusals.refuse("ESS-HTTP-024", key, reason);
            None
        }
        None => Some(encoding),
    }
}

/// A route's `errors`, checked as if listed under every outcome of the command that reports an
/// error and is not under `outcomes` (`listed`), each refusal at `routes[i].errors…` naming the
/// outcome it was checked for; refused where it covers none (`ESS-HTTP-022`). Returns the compiled
/// answer, when every covered outcome admits it, and the names it covers, in declaration order.
fn errors<'c>(
    spec: &RouteSpec,
    operation: &Operation,
    ir: &EssIr,
    command: &'c ResolvedCommand,
    listed: &std::collections::BTreeSet<&str>,
    path: &str,
    refusals: &mut Refusals,
) -> (Option<Outcome>, Vec<&'c str>) {
    use std::fmt::Write as _;
    let Some(written) = &spec.errors else {
        return (None, Vec::new());
    };
    let at = format!("{path}.errors");
    let covering: Vec<&ResolvedOutcome> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_some() && !listed.contains(outcome.name.as_str()))
        .collect();
    if covering.is_empty() {
        let why = if command
            .outcomes
            .iter()
            .any(|outcome| outcome.error.is_some())
        {
            "every outcome of it that reports an error is listed under `outcomes`"
        } else {
            "no outcome of it reports an error"
        };
        refusals.refuse(
            "ESS-HTTP-022",
            at,
            format!(
                "`errors` answers the outcomes of {operation} that report an error and are not \
                 listed under `outcomes`, and there is none: {why}"
            ),
        );
        return (None, Vec::new());
    }
    let mut compiled: Option<Outcome> = None;
    let mut refused = false;
    for outcome in &covering {
        let mut checked = Refusals(Vec::new());
        let answer = outcome_answer(written, ir, command, outcome, &at, &mut checked);
        let name = &outcome.name;
        for mut diagnostic in checked.0 {
            if !diagnostic.message.contains(&format!("`{name}`")) {
                let _ = write!(
                    diagnostic.message,
                    " (checked for outcome `{name}`, which `errors` answers)"
                );
            }
            refusals.0.push(diagnostic);
        }
        match answer {
            Some(answer) if !refused => {
                compiled.get_or_insert(answer);
            }
            _ => refused = true,
        }
    }
    // A refused `errors` covers its outcomes all the same: none of them is counted at a derived
    // answer, as a refused listed outcome is not.
    let names = covering
        .iter()
        .map(|outcome| outcome.name.as_str())
        .collect();
    (if refused { None } else { compiled }, names)
}

/// The outcome's status, unless it is refused (`ESS-HTTP-023`): no status at all, a `1xx`, a `3xx`
/// other than the [`REDIRECTS`], or one of the class the outcome may not answer.
fn outcome_status_checked(
    status: i64,
    outcome: &ResolvedOutcome,
    at: &str,
    refusals: &mut Refusals,
) -> Option<u16> {
    let reason = status_reason(status).or_else(|| class_reason(status, outcome.error.is_some()));
    if let Some(reason) = reason {
        refusals.refuse(
            "ESS-HTTP-023",
            format!("{at}.status"),
            format!("outcome `{}` answers {reason}", outcome.name),
        );
        return None;
    }
    u16::try_from(status).ok()
}

/// The outcome's body, unless it is refused (`ESS-HTTP-024`): a word no outcome body spells, a
/// view's body, a literal, `response` or `response.<field>` where the outcome returns none, a
/// `response.<field>` naming no field of the response or an `Optional` one, a source other than
/// `response.<field>`, an error object where it reports no error or under a member it cannot hold,
/// or any body but `none` on `204` or beside a redirect. `status` is the outcome's compiled status:
/// one refused (`ESS-HTTP-023`) holds the body to no `204`; `redirects` says the outcome declares a
/// `redirect` on a redirect status.
/// What `response` or `response.<field>` answers, under one named member (beyond10x/ess#755):
/// `bare` resolves the form as the outcome's bare body, and a member holding any other form, an
/// empty member, or a form the outcome cannot answer bare is refused.
fn wrapped_body(
    wrapped: &BodySpec,
    member: &str,
    bare: impl FnOnce() -> Result<Body, String>,
    forms: &str,
) -> Result<Body, String> {
    if member.is_empty() {
        return Err(format!(
            "the member of `{wrapped}` is empty; a wrapped body holds its value under a named \
             member"
        ));
    }
    let member = member.to_owned();
    match bare() {
        Ok(Body::Response) => Ok(Body::Wrapped {
            member,
            field: None,
        }),
        Ok(Body::ResponseField(field)) => Ok(Body::Wrapped {
            member,
            field: Some(field),
        }),
        Ok(_) => Err(format!(
            "`{wrapped}` wraps another form; a member holds `response` or `response.<field>`, \
             and {forms}"
        )),
        Err(reason) => Err(format!("in `{wrapped}`, {reason}")),
    }
}

/// `body` unless the outcome answers `204` or declares a `redirect`, which answer `none` only.
fn no_content(
    body: Body,
    name: &impl fmt::Display,
    status: Option<u16>,
    redirects: bool,
) -> Result<Body, String> {
    if status == Some(204) && body != Body::None {
        Err(format!(
            "outcome `{name}` answers `204`, which carries no content, and its body is \
             `{body}`; a `204` answers `body: none`"
        ))
    } else if redirects && body != Body::None {
        Err(format!(
            "outcome `{name}` declares a `redirect`, and its body is `{body}`; a redirect \
             answers `body: none`"
        ))
    } else {
        Ok(body)
    }
}

fn outcome_body(
    written: Option<&BodySpec>,
    status: Option<u16>,
    redirects: bool,
    reader: &Reader<'_>,
    at: &str,
    refusals: &mut Refusals,
) -> Option<Body> {
    let (ir, command, outcome) = (reader.ir, reader.command, reader.outcome);
    let name = &outcome.name;
    let error = outcome.error.as_ref().map(|handle| ir.error(handle));
    // Why the outcome answers no response, if it does not: `response` and `response.<field>` alike.
    let answers_none = |text: &str| match (outcome.returns, error) {
        (_, Some(error)) => Some(format!(
            "`{text}` reads the command's response, and outcome `{name}` reports error `{}`, \
             which answers no response",
            error.name
        )),
        (false, None) => Some(format!(
            "`{text}` reads the command's response, and outcome `{name}` does not declare \
             `returns: true`, so it answers none"
        )),
        (true, None) => None,
    };
    let forms = "an outcome answers `envelope`, `response`, `response.<field>`, `none`, \
                 `{<member>: error}`, `{<member>: response}` or `{<member>: response.<field>}`";
    let word_body = |word: &String| match word.as_str() {
        "envelope" => Ok(Body::Envelope),
        "none" => Ok(Body::None),
        "response" => answers_none(word).map_or(Ok(Body::Response), Err),
        "rows" | "array" | "single" => Err(format!("`{word}` is a view's body; {forms}")),
        _ => match AnswerSource::parse(word) {
            Some(AnswerSource::Response(field)) => match answers_none(word) {
                Some(reason) => Err(reason),
                None => match declared_type(&command.response, &field) {
                    None => Err(format!(
                        "`{word}` names no field: command `{}` declares no response field \
                             `{field}`",
                        command.name
                    )),
                    Some(type_ref @ ResolvedTypeRef::Optional { .. }) => Err(format!(
                        "`{word}` has type `{type_ref}`, and a body is always present: a \
                             `response.<field>` body names a field that is not `Optional`"
                    )),
                    Some(_) => Ok(Body::ResponseField(field)),
                },
            },
            Some(source) => Err(format!(
                "`{source}` is a source a header or a redirect reads, not a body: of the \
                     sources, a body answers only `response.<field>`, and an error answers its \
                     object with `{{<member>: error}}`"
            )),
            None => Err(format!("`{word}` is not a body; {forms}")),
        },
    };
    let resolved = match written {
        None => Ok(Body::Envelope),
        Some(BodySpec::Word(word)) => word_body(word),
        Some(wrapped @ BodySpec::Wrapped { member, form }) => {
            wrapped_body(wrapped, member, || word_body(form), forms)
        }
        Some(BodySpec::Error(member)) => match error {
            None => Err(format!(
                "outcome `{name}` reports no error, so it has no error object for \
                 `{{{member}: error}}` to answer"
            )),
            Some(_) if member.is_empty() => Err(
                "the member of `{'': error}` is empty; the error object carries the error's \
                 wire name under a named member"
                    .to_owned(),
            ),
            Some(error) => match error
                .fields
                .iter()
                .find(|field| field.naming.wire.as_deref().unwrap_or(&field.name) == member)
            {
                Some(field) => Err(format!(
                    "error `{}` carries field `{}` under the wire name `{member}`, so \
                     `{{{member}: error}}` would hold `{member}` twice",
                    error.name, field.name
                )),
                None => Ok(Body::Error {
                    member: member.clone(),
                }),
            },
        },
        Some(literal @ BodySpec::Literal { .. }) => Err(format!(
            "`{literal}` is a literal body, which only a standard refusal under `refusals:` \
             answers"
        )),
    };
    match resolved.and_then(|body| no_content(body, name, status, redirects)) {
        Ok(body) => Some(body),
        Err(reason) => {
            refusals.refuse("ESS-HTTP-024", format!("{at}.body"), reason);
            None
        }
    }
}

/// What an outcome's headers and redirect read from: the command, the outcome and its error.
struct Reader<'a> {
    ir: &'a EssIr,
    command: &'a ResolvedCommand,
    outcome: &'a ResolvedOutcome,
}

impl<'a> Reader<'a> {
    /// The wire name of the outcome's error, which `error.wire` writes, known when the binding
    /// compiles; `None` when the outcome reports no error.
    fn wire(&self) -> Option<&'a str> {
        self.outcome.error.as_ref().map(|handle| {
            let error = self.ir.error(handle);
            error.naming.wire_or(&error.name)
        })
    }

    /// The source `text` names and its type, `None` for `error.wire`, which is always a `String`;
    /// or why the outcome cannot supply it: a source outside the four forms, `response.` on an
    /// outcome without `returns: true`, `error.` on one without an error, `error.wire` where the
    /// error declares a field named `wire`, or a field the command, its response or the error does
    /// not declare.
    fn read(&self, text: &str) -> Result<(AnswerSource, Option<&ResolvedTypeRef>), String> {
        let Some(source) = AnswerSource::parse(text) else {
            return Err(format!(
                "`{text}` is not a source an answer reads: `input.<field>`, `response.<field>`, \
                 `error.<field>` or `error.wire`"
            ));
        };
        let outcome = &self.outcome.name;
        let command = &self.command.name;
        let error = self
            .outcome
            .error
            .as_ref()
            .map(|handle| self.ir.error(handle));
        let found = match (&source, error) {
            (AnswerSource::Input(name), _) => declared_type(&self.command.input, name).ok_or_else(|| {
                format!("`{text}` names no field: command `{command}` declares no input field `{name}`")
            })?,
            (AnswerSource::Response(_), _) if !self.outcome.returns => {
                return Err(format!(
                    "`{text}` reads the response, and outcome `{outcome}` does not declare \
                     `returns: true`, so it answers none"
                ));
            }
            (AnswerSource::Response(name), _) => declared_type(&self.command.response, name)
                .ok_or_else(|| {
                    format!(
                        "`{text}` names no field: command `{command}` declares no response field \
                         `{name}`"
                    )
                })?,
            (AnswerSource::Error(_) | AnswerSource::ErrorWire, None) => {
                return Err(format!(
                    "`{text}` reads the error, and outcome `{outcome}` reports no error"
                ));
            }
            (AnswerSource::ErrorWire, Some(error)) => {
                if error.fields.iter().any(|declared| declared.name == "wire") {
                    return Err(format!(
                        "`error.wire` is ambiguous on outcome `{outcome}`: error `{}` declares a \
                         field named `wire`, and `error.wire` names the error's wire name",
                        error.name
                    ));
                }
                return Ok((source, None));
            }
            (AnswerSource::Error(name), Some(error)) => declared_type(&error.fields, name)
                .ok_or_else(|| {
                    format!(
                        "`{text}` names no field: error `{}` declares no field `{name}`",
                        error.name
                    )
                })?,
        };
        Ok((source, Some(found)))
    }

    /// [`Reader::read`], held to what a redirect's `location` reads: a `String` or a newtype of
    /// one, never `Optional`, and never `error.wire`, a wire name and no absolute URI.
    fn location(&self, text: &str) -> Result<AnswerSource, String> {
        match self.read(text) {
            Err(reason) => Err(format!("`location`: {reason}")),
            Ok((AnswerSource::ErrorWire, _)) => Err(
                "`location` reads `error.wire`, the error's wire name, and a redirect's \
                 `location` holds the absolute URI the caller is sent to, which a wire name never \
                 is"
                .to_owned(),
            ),
            Ok((_, Some(type_ref @ ResolvedTypeRef::Optional { .. }))) => Err(format!(
                "`location` reads `{text}`, of type `{type_ref}`, and a redirect's `location` is \
                 always present: a `String` or a newtype of `String`, never `Optional`"
            )),
            Ok((_, Some(type_ref))) if !string_type(type_ref, self.ir) => Err(format!(
                "`location` reads `{text}`, of type `{type_ref}`, and a redirect's `location` is \
                 a `String` or a newtype of `String`, holding an absolute URI"
            )),
            Ok((source, _)) => Ok(source),
        }
    }

    /// [`Reader::read`], held to a scalar or an `Optional` scalar, which is written as text and
    /// omitted when absent.
    fn read_text(&self, text: &str) -> Result<AnswerSource, String> {
        let (source, type_ref) = self.read(text)?;
        match type_ref {
            Some(type_ref) if !optional_scalar(type_ref, self.ir) => Err(format!(
                "`{text}` has type `{type_ref}`, which is not written as text: a header value, a \
                 challenge param and a redirect parameter read a scalar or an `Optional` scalar, a \
                 scalar being a primitive other than `Json`, an enum, or a newtype of a scalar"
            )),
            _ => Ok(source),
        }
    }
}

/// The type of the field of `fields` named `name`, if one is.
fn declared_type<'f>(fields: &'f [ResolvedField], name: &str) -> Option<&'f ResolvedTypeRef> {
    fields
        .iter()
        .find(|declared| declared.name == name)
        .map(|declared| &declared.type_ref)
}

/// A redirect's `mode`, or why it is none (`ESS-HTTP-028`): absent, or outside `query` and
/// `fragment`.
fn redirect_mode(written: Option<&str>) -> Result<RedirectMode, String> {
    let Some(text) = written else {
        return Err(
            "a redirect declares its `mode`, `query` or `fragment`: where its parameters go is \
             the author's to choose"
                .to_owned(),
        );
    };
    RedirectMode::parse(text).ok_or_else(|| {
        format!(
            "`{text}` is not a redirect mode: `query` appends the parameters to the location's \
             query, `fragment` writes them as its fragment"
        )
    })
}

/// Whether `type_ref` is a scalar or an `Optional` scalar ([`scalar`]).
fn optional_scalar(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Optional { of } => scalar(of, ir),
        _ => scalar(type_ref, ir),
    }
}

/// Whether `type_ref` is `String` or a newtype of one, at any depth.
fn string_type(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => *name == Primitive::String,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => string_type(of, ir),
            _ => false,
        },
        _ => false,
    }
}

/// An answer's headers under `at` (an outcome, or `refusals.<key>`), each checked in document
/// order (`ESS-HTTP-026`, `-027`), and a `401` held to a `WWW-Authenticate` challenge
/// (`ESS-HTTP-027`). `reader` is the outcome a source reads from; a standard refusal has none, so
/// every source in its headers is refused. `status` is the answer's compiled status, `None` when it
/// was refused, which holds it to no `401`.
fn answer_headers(
    written: Option<&Entries<HeaderSpec>>,
    reader: Option<&Reader<'_>>,
    status: Option<u16>,
    at: &str,
    refusals: &mut Refusals,
) -> Result<BTreeMap<String, Header>, Refused> {
    let before = refusals.0.len();
    let mut compiled = BTreeMap::new();
    let mut declared: Vec<&str> = Vec::new();
    // A challenge under `WWW-Authenticate`, whatever else it was refused for: a challenge refused
    // for its scheme is not refused again as missing.
    let mut challenged = false;
    for (name, header) in written.map_or(&[][..], |entries| &entries.0[..]) {
        let key = format!("{at}.headers.{name}");
        if matches!(header, HeaderSpec::Challenge(_)) && name.eq_ignore_ascii_case(CHALLENGE_HEADER)
        {
            challenged = true;
        }
        if !is_token(name) {
            refusals.refuse(
                "ESS-HTTP-026",
                key,
                format!(
                    "`{name}` is not a header name: a header name is an RFC 9110 token, one or \
                     more letters, digits and ``!#$%&'*+-.^_`|~``"
                ),
            );
            continue;
        }
        if let Some((owned, owner)) = RESERVED_HEADERS
            .iter()
            .find(|(owned, _)| owned.eq_ignore_ascii_case(name))
        {
            refusals.refuse(
                "ESS-HTTP-026",
                key,
                format!(
                    "header `{name}` is `{owned}`, which {owner} writes; a binding declares no \
                     header the server, the encoding or `redirect` writes"
                ),
            );
            continue;
        }
        if let Some(first) = same_header(name, declared.iter().copied()) {
            refusals.refuse(
                "ESS-HTTP-026",
                key,
                format!(
                    "header `{name}` is `{first}` again: header names are compared ignoring case, \
                     and each header is declared once"
                ),
            );
            continue;
        }
        declared.push(name);
        if let Some(value) = header_value(name, header, reader, &key, refusals) {
            compiled.insert(name.clone(), value);
        }
    }
    if status == Some(401) && !challenged {
        refusals.refuse(
            "ESS-HTTP-027",
            format!("{at}.headers"),
            format!(
                "`{at}` answers `401`, and declares no `{CHALLENGE_HEADER}` challenge; every `401` \
                 carries one (RFC 9110 §15.5.2), as `{CHALLENGE_HEADER}: {{challenge: {{scheme, \
                 params}}}}`"
            ),
        );
    }
    if refusals.0.len() == before {
        Ok(compiled)
    } else {
        Err(Refused)
    }
}

/// One header's value under `key`, compiled, or `None` once it is refused: a literal that is not a
/// field value, a source the reader cannot supply (`ESS-HTTP-026`, every source when there is no
/// reader) or `error.wire` whose wire name is not a field value (`ESS-HTTP-026`), a challenge
/// under any header but `WWW-Authenticate` or one [`challenge`] refuses (`ESS-HTTP-027`).
fn header_value(
    name: &str,
    header: &HeaderSpec,
    reader: Option<&Reader<'_>>,
    key: &str,
    refusals: &mut Refusals,
) -> Option<Header> {
    let value = match header {
        HeaderSpec::Literal(text) => match field_value_reason(text) {
            Some(reason) => Err(("ESS-HTTP-026", format!("header `{name}`: {reason}"))),
            None => Ok(Header::Literal(text.clone())),
        },
        HeaderSpec::Source(text) => match reader {
            None => Err((
                "ESS-HTTP-026",
                format!(
                    "header `{name}` reads `{text}`, and a standard refusal has no outcome to \
                     read a source from; its headers are literals and challenges whose params \
                     are literals"
                ),
            )),
            Some(reader) => reader
                .read_text(text)
                .and_then(|source| match (&source, reader.wire()) {
                    // The wire name is known now, and held to what a literal is.
                    (AnswerSource::ErrorWire, Some(wire)) => match field_value_reason(wire) {
                        Some(reason) => Err(format!(
                            "`error.wire` reads the error's wire name, known when the binding \
                             compiles, and {reason}"
                        )),
                        None => Ok(source),
                    },
                    _ => Ok(source),
                })
                .map(Header::Source)
                .map_err(|reason| ("ESS-HTTP-026", format!("header `{name}`: {reason}"))),
        },
        HeaderSpec::Challenge(_) if !name.eq_ignore_ascii_case(CHALLENGE_HEADER) => Err((
            "ESS-HTTP-027",
            format!(
                "header `{name}` holds a challenge, and a challenge is declared under \
                 `{CHALLENGE_HEADER}` only (RFC 9110 §11.6.1)"
            ),
        )),
        HeaderSpec::Challenge(written) => {
            return challenge(written, reader, key, refusals).map(Header::Challenge);
        }
    };
    value
        .map_err(|(code, message)| refusals.refuse(code, key, message))
        .ok()
}

/// One challenge under `key`, checked (`ESS-HTTP-027`): a token scheme, token param names each
/// once ignoring case, and literal params a quoted string carries. A sourced param is read as a
/// header source is (`ESS-HTTP-026`), and one reading `error.wire` is held to a quoted string as a
/// literal is (`ESS-HTTP-027`). `None` when anything was refused.
fn challenge(
    written: &ChallengeSpec,
    reader: Option<&Reader<'_>>,
    key: &str,
    refusals: &mut Refusals,
) -> Option<Challenge> {
    let before = refusals.0.len();
    if !is_token(&written.scheme) {
        refusals.refuse(
            "ESS-HTTP-027",
            format!("{key}.challenge.scheme"),
            format!(
                "`{}` is not a token: an auth scheme is an RFC 9110 token, one or more letters, \
                 digits and ``!#$%&'*+-.^_`|~``",
                written.scheme
            ),
        );
    }
    let mut params = BTreeMap::new();
    let mut declared: Vec<&str> = Vec::new();
    for (name, param) in written
        .params
        .as_ref()
        .map_or(&[][..], |entries| &entries.0[..])
    {
        let at = format!("{key}.challenge.params.{name}");
        if !is_token(name) {
            refusals.refuse(
                "ESS-HTTP-027",
                at,
                format!("`{name}` is not a token: an auth param's name is an RFC 9110 token"),
            );
            continue;
        }
        if let Some(first) = same_header(name, declared.iter().copied()) {
            refusals.refuse(
                "ESS-HTTP-027",
                at,
                format!(
                    "param `{name}` is `{first}` again: param names are compared ignoring case, \
                     and a challenge names each once (RFC 9110 §11.2)"
                ),
            );
            continue;
        }
        declared.push(name);
        let value = match param {
            ParamSpec::Literal(text) if !quotable(text) => Err((
                "ESS-HTTP-027",
                format!(
                    "param `{name}`: {text:?} is not text a quoted string carries, which holds \
                     visible US-ASCII, spaces and horizontal tabs"
                ),
            )),
            ParamSpec::Literal(text) => Ok(Param::Literal(text.clone())),
            ParamSpec::Source(text) => match reader {
                None => Err((
                    "ESS-HTTP-026",
                    format!(
                        "param `{name}` reads `{text}`, and a standard refusal has no outcome to \
                         read a source from; its challenge params are literals"
                    ),
                )),
                Some(reader) => match reader.read_text(text) {
                    Err(reason) => Err(("ESS-HTTP-026", format!("param `{name}`: {reason}"))),
                    // The wire name is known now, and held to what a literal param is.
                    Ok(AnswerSource::ErrorWire) => match reader.wire() {
                        Some(wire) if !quotable(wire) => Err((
                            "ESS-HTTP-027",
                            format!(
                                "param `{name}`: `error.wire` reads the error's wire name, known \
                                 when the binding compiles, and {wire:?} is not text a quoted \
                                 string carries, which holds visible US-ASCII, spaces and \
                                 horizontal tabs"
                            ),
                        )),
                        _ => Ok(Param::Source(AnswerSource::ErrorWire)),
                    },
                    Ok(source) => Ok(Param::Source(source)),
                },
            },
        };
        match value {
            Ok(value) => {
                params.insert(name.clone(), value);
            }
            Err((code, message)) => refusals.refuse(code, at, message),
        }
    }
    (refusals.0.len() == before).then(|| Challenge {
        scheme: written.scheme.clone(),
        params,
    })
}

/// The outcome's redirect under `at`, checked (`ESS-HTTP-028`): on a redirect status and on every
/// one, a `location` reading a `String` or a newtype of one that the outcome supplies and that is
/// never `Optional` and never `error.wire`, a `mode` of the set, and parameters each named in RFC
/// 3986 `unreserved` characters, written once, reading a scalar the outcome supplies. `status`
/// is the compiled status: one refused (`ESS-HTTP-023`) is not judged against the redirect again.
fn outcome_redirect(
    written: Option<&RedirectSpec>,
    status: Option<u16>,
    reader: &Reader<'_>,
    at: &str,
    refusals: &mut Refusals,
) -> Result<Option<Redirect>, Refused> {
    let before = refusals.0.len();
    let outcome = &reader.outcome.name;
    let redirects = status.map(|status| REDIRECTS.contains(&status));
    let Some(written) = written else {
        if let (Some(true), Some(status)) = (redirects, status) {
            refusals.refuse(
                "ESS-HTTP-028",
                format!("{at}.redirect"),
                format!(
                    "outcome `{outcome}` answers `{status}`, a redirect, and declares no \
                     `redirect`; each of `301`, `302`, `303`, `307` and `308` names the \
                     `location` it sends the caller to"
                ),
            );
            return Err(Refused);
        }
        return Ok(None);
    };
    if let (Some(false), Some(status)) = (redirects, status) {
        refusals.refuse(
            "ESS-HTTP-028",
            format!("{at}.redirect"),
            format!(
                "outcome `{outcome}` answers `{status}`, which is not a redirect, and declares a \
                 `redirect`; a `redirect` belongs to `301`, `302`, `303`, `307` and `308`"
            ),
        );
    }
    let location = reader
        .location(&written.location)
        .map_err(|reason| {
            refusals.refuse("ESS-HTTP-028", format!("{at}.redirect.location"), reason);
        })
        .ok();
    let mode = redirect_mode(written.mode.as_deref())
        .map_err(|reason| refusals.refuse("ESS-HTTP-028", format!("{at}.redirect.mode"), reason))
        .ok();
    let mut parameters = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for (name, text) in written
        .parameters
        .as_ref()
        .map_or(&[][..], |entries| &entries.0[..])
    {
        let key = format!("{at}.redirect.parameters.{name}");
        if !unreserved(name) {
            refusals.refuse("ESS-HTTP-028", key, unreserved_reason(name));
            continue;
        }
        if !seen.insert(name.as_str()) {
            refusals.refuse(
                "ESS-HTTP-028",
                key,
                format!(
                    "parameter `{name}` is declared twice; a redirect names each parameter once"
                ),
            );
            continue;
        }
        match reader.read_text(text) {
            Ok(source) => {
                parameters.insert(name.clone(), source);
            }
            Err(reason) => {
                refusals.refuse("ESS-HTTP-028", key, format!("parameter `{name}`: {reason}"));
            }
        }
    }
    match (location, mode) {
        (Some(location), Some(mode)) if refusals.0.len() == before => Ok(Some(Redirect {
            location,
            mode,
            parameters,
        })),
        _ => Err(Refused),
    }
}

/// An outcome as `ESS-HTTP-025` counts it: its name, status, body, error wire name, the parameter
/// names its redirect writes, if it declares one, the fields its error writes beside the
/// member of an error object (`error_fields`), and how its body is written.
type Counted<'a> = (
    &'a str,
    u16,
    Body,
    Option<&'a str>,
    Option<Written<'a>>,
    ErrorFields<'a>,
    ResponseEncoding,
);

/// The parameter names a redirect writes (`ESS-HTTP-025`): `always` those whose source is never
/// absent, `all` every one, an `Optional` source being omitted when absent, and `wired` those
/// written from `error.wire`, which are always written.
struct Written<'a> {
    always: std::collections::BTreeSet<&'a str>,
    all: std::collections::BTreeSet<&'a str>,
    wired: std::collections::BTreeSet<&'a str>,
}

impl<'a> Written<'a> {
    /// The names `redirect` writes for `outcome`. A source whose type is `Optional` may be absent;
    /// `error.wire` never is. A source that no longer reads is counted as possibly absent, which
    /// can only refuse more.
    fn of(redirect: &'a Redirect, reader: &Reader<'_>) -> Self {
        let mut always = std::collections::BTreeSet::new();
        let mut wired = std::collections::BTreeSet::new();
        for (name, source) in &redirect.parameters {
            let present = match reader.read(&source.to_string()) {
                Ok((AnswerSource::ErrorWire, _)) => {
                    wired.insert(name.as_str());
                    true
                }
                Ok((_, type_ref)) => !matches!(type_ref, Some(ResolvedTypeRef::Optional { .. })),
                Err(_) => false,
            };
            if present {
                always.insert(name.as_str());
            }
        }
        Self {
            always,
            all: redirect.parameters.keys().map(String::as_str).collect(),
            wired,
        }
    }

    /// Whether a reader of the parameters can tell these, written for an outcome whose error wire
    /// name is `wire`, from `other`'s, written for one whose is `other_wire`: one side always
    /// writes a name the other never writes, or both write one name from `error.wire` and the wire
    /// names differ (RFC 6749 §4.1.2.1: each authorization error redirects with its own `error`).
    /// Otherwise some answer carries a set of names, and values, both could write.
    fn apart(&self, wire: Option<&str>, other: &Self, other_wire: Option<&str>) -> bool {
        !self.always.is_subset(&other.all)
            || !other.always.is_subset(&self.all)
            || matches!((wire, other_wire), (Some(one), Some(two)) if one != two)
                && !self.wired.is_disjoint(&other.wired)
    }
}

/// What a command route declares of its outcomes' answers, as `ESS-HTTP-025` counts them: each
/// listed outcome that compiled (`compiled`), in document order (`listed`), every key written under
/// `outcomes` (`written`), `errors`, when it compiled, with the outcomes it covers (`covered`,
/// in declaration order, listed even when `errors` was refused), and the groups of `alike` as
/// written (`alike`).
struct Declared<'d> {
    compiled: &'d BTreeMap<String, Outcome>,
    listed: &'d [&'d str],
    written: &'d Entries<OutcomeSpec>,
    errors: Option<&'d Outcome>,
    covered: &'d [&'d str],
    alike: &'d [Vec<String>],
}

impl Declared<'_> {
    /// The declared answer of outcome `name`, if it has one that compiled.
    fn answer(&self, name: &str) -> Option<&Outcome> {
        self.compiled
            .get(name)
            .or_else(|| self.errors.filter(|_| self.covered.contains(&name)))
    }

    /// How a refusal names the answer of a counted outcome.
    fn how(&self, counted: &Counted<'_>) -> String {
        if self.compiled.contains_key(counted.0) {
            format!("`{}`", counted.2)
        } else if self.covered.contains(&counted.0) {
            format!("`{}`, under `errors`", counted.2)
        } else {
            "`envelope`, at its derived status, as it is not listed".to_owned()
        }
    }

    /// Whether one group of `alike` names both outcomes: the binding declares them answered alike,
    /// so `ESS-HTTP-025` does not ask the wire to tell them apart ([`alike`] holds the group to
    /// one answer).
    fn alike_pair(&self, one: &str, other: &str) -> bool {
        self.alike.iter().any(|group| {
            group.iter().any(|name| name == one) && group.iter().any(|name| name == other)
        })
    }
}

/// Refuses outcomes of one command on one status that the bodies cannot tell apart
/// (`ESS-HTTP-025`): every outcome counts, a listed one at its status and body, one `errors`
/// covers at the status and body of `errors`, any other at its derived status
/// ([`status::outcome_status`]) with `envelope`. Two bodies tell two outcomes apart when both are
/// `envelope`, whose `outcome` member names the branch, or both `{<member>: error}` on one member
/// with distinct error wire names or with one wire name and errors one of which always writes a
/// field the other never writes ([`fields_of`], which the `alike` check asks too). Two redirects on one status, both `body: none`, are told apart
/// by their parameters when one always writes a name the other never writes (RFC 6749 §4.1.2 and
/// §4.1.2.1: `code` and `error` on one `302`), or when both always write one name from
/// `error.wire` and their error wire names differ (§4.1.2.1: `access_denied` and `invalid_scope`
/// on one `302`); a redirect beside an outcome without one is not. A listed outcome that was
/// refused is not counted, nor one a refused `errors` covers. Each listed outcome, in document
/// order, then each outcome `errors` covers, in declaration order, is refused naming every outcome
/// it cannot be told apart from, but a pair an earlier refusal already named together; one with
/// nothing left to name is not refused, so outcomes no body tells apart are one refusal. A listed
/// outcome is refused at `routes[i].outcomes.<name>`, a covered one at `routes[i].errors`. Two
/// outcomes one group of `alike` names are not held apart: [`alike`] holds them to one answer.
///
/// Each standard refusal the binding overrides (`standard`) is held to the same rule against every
/// counted outcome, and refused at `refusals.<key>`, once per route, naming the route and the
/// outcomes on its status.
fn distinguishable(
    operation: &Operation,
    ir: &EssIr,
    command: &ResolvedCommand,
    declared: &Declared<'_>,
    path: &str,
    standard: &Standing,
    refusals: &mut Refusals,
) {
    let counted = counted(ir, command, declared);
    let apart = |left: &Counted<'_>, right: &Counted<'_>| {
        left.1 != right.1
            || match (&left.2, &right.2) {
                (Body::Envelope, Body::Envelope) => true,
                (Body::Error { member: one }, Body::Error { member: other }) => {
                    one == other
                        && (left.3 != right.3
                            || matches!(fields_of(&left.5, &right.5), Fields::Deciding(_)))
                }
                _ => false,
            }
            || match (&left.4, &right.4) {
                (Some(one), Some(other)) => one.apart(left.3, other, right.3),
                _ => false,
            }
    };
    // Each outcome the route declares an answer for, with where a refusal of it is reported.
    let errors_at = format!("{path}.errors");
    let mut declaring: Vec<(&str, String)> = declared
        .listed
        .iter()
        .map(|name| (*name, format!("{path}.outcomes.{name}")))
        .collect();
    if declared.errors.is_some() {
        declaring.extend(
            declared
                .covered
                .iter()
                .map(|name| (*name, errors_at.clone())),
        );
    }
    // The outcomes each earlier refusal named, the refused one included: a pair named together in
    // one refusal is not reported again.
    let mut reported: Vec<Vec<&str>> = Vec::new();
    for (name, at) in declaring {
        let Some(this) = counted.iter().find(|counted| counted.0 == name) else {
            continue;
        };
        let others: Vec<&Counted<'_>> = counted
            .iter()
            .filter(|other| {
                other.0 != name && !apart(this, other) && !declared.alike_pair(name, other.0)
            })
            .filter(|other| {
                !reported
                    .iter()
                    .any(|group| group.contains(&name) && group.contains(&other.0))
            })
            .collect();
        let mut group = vec![name];
        group.extend(others.iter().map(|other| other.0));
        reported.push(group);
        if others.is_empty() {
            continue;
        }
        let (clashing, fields) = clash_text(declared, this, &others);
        let under = if at == errors_at {
            ", under `errors`,"
        } else {
            ""
        };
        refusals.refuse(
            "ESS-HTTP-025",
            at,
            format!(
                "outcome `{name}` of {operation}{under} answers `{}` with `{}`, and so does \
                 {clashing}; two outcomes on one status are told apart by two `envelope` bodies, \
                 by `{{<member>: error}}` on one member with distinct error wire names or with \
                 one error always writing a field the other never writes, or by two redirects, \
                 one always writing a parameter the other never writes or both writing one \
                 parameter from `error.wire` with distinct error wire names{fields}",
                this.1, this.2,
            ),
        );
    }
    overridden(operation, declared, &counted, standard, path, refusals);
}

/// What the status refusal of `this` says of the `others` it clashes with: who they are, and the
/// fact about those that clash only by their error fields (`field_clash`) with the forms that
/// validate. The sentence that a reader cannot tell the bodies apart is said only of the others
/// whose answer is the same as `this` one's ([`same_answer`]), where it is true.
fn clash_text(
    declared: &Declared<'_>,
    this: &Counted<'_>,
    others: &[&Counted<'_>],
) -> (String, String) {
    let (by_fields, rest): (Vec<&&Counted<'_>>, Vec<&&Counted<'_>>) = others
        .iter()
        .partition(|other| field_clash(this, other).is_some());
    let (same, other): (Vec<&&Counted<'_>>, Vec<&&Counted<'_>>) =
        rest.into_iter().partition(|other| same_answer(this, other));
    let who = |list: &[&&Counted<'_>]| {
        list.iter()
            .map(|other| format!("`{}` ({})", other.0, declared.how(other)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut parts = Vec::new();
    if !same.is_empty() {
        parts.push(format!(
            "{}, with a body a reader cannot tell apart",
            who(&same)
        ));
    }
    if !other.is_empty() {
        parts.push(who(&other));
    }
    if !by_fields.is_empty() {
        parts.push(who(&by_fields));
    }
    (parts.join(", and so does "), field_facts(this, &by_fields))
}

/// Whether `other` answers as `this` does, so that no reader can tell a body of one from a body
/// of the other: the same body form, the same `encoding`, and either both a redirect or neither.
/// An answer that differs in body form (an `envelope` beside `{<member>: error}`, `none` beside a
/// member), in `encoding` (`text` beside `json` of one field: `n1` as `text/plain` against `"n1"`
/// as `application/json`) or that is a redirect beside one that is not is a clash the rule
/// refuses, which does not say no reader can tell.
fn same_answer(this: &Counted<'_>, other: &Counted<'_>) -> bool {
    this.2 == other.2 && this.6 == other.6 && this.4.is_some() == other.4.is_some()
}

/// How `other` clashes with `this` by error fields alone: both are `{<member>: error}` on one
/// member of one error wire name, and [`fields_of`] finds the fields differ without one error
/// always writing a field the other never writes ([`Fields::Undecided`] and [`Fields::Typed`]).
fn field_clash<'a>(this: &Counted<'a>, other: &Counted<'a>) -> Option<Fields<'a>> {
    match (&this.2, &other.2) {
        (Body::Error { member: one }, Body::Error { member: two })
            if one == two && this.3 == other.3 =>
        {
            Some(fields_of(&this.5, &other.5))
                .filter(|fields| matches!(fields, Fields::Undecided | Fields::Typed(_)))
        }
        _ => None,
    }
}

/// What a refusal of `this` adds for the `others` it clashes with by error fields alone: the fact
/// and the forms that validate, or nothing.
fn field_facts(this: &Counted<'_>, others: &[&&Counted<'_>]) -> String {
    let undecided: Vec<&&&Counted<'_>> = others
        .iter()
        .filter(|other| field_clash(this, other) == Some(Fields::Undecided))
        .collect();
    let mut facts: Vec<String> = Vec::new();
    if !undecided.is_empty() {
        let names: Vec<String> = undecided
            .iter()
            .map(|other| format!("`{}`", other.0))
            .collect();
        let mut fields = vec![(this.0, &this.5)];
        fields.extend(undecided.iter().map(|other| (other.0, &other.5)));
        facts.push(format!(
            "; the errors of `{}` and {} write one wire name, and {NO_DECIDING_FIELD}{}; the \
             forms that validate are {FIELD_FORMS}",
            this.0,
            names.join(", "),
            unknown_instance_clause(&fields)
        ));
    }
    for other in others {
        if let Some(Fields::Typed(field)) = field_clash(this, other) {
            facts.push(format!(
                "; the errors of `{}` and `{}` write one wire name, and {}; the forms that \
                 validate are {}",
                this.0,
                other.0,
                same_field_other_type(field),
                type_forms(field)
            ));
        }
    }
    facts.concat()
}

/// Every outcome [`distinguishable`] counts: its name, status, body, error wire name and redirect
/// parameters. An outcome with a declared answer that compiled counts with it, an outcome neither
/// listed nor covered by `errors` at its derived status with `envelope`, and a refused one not at
/// all.
fn counted<'a>(
    ir: &'a EssIr,
    command: &'a ResolvedCommand,
    declared: &'a Declared<'a>,
) -> Vec<Counted<'a>> {
    let listed_names: std::collections::BTreeSet<&str> = declared
        .written
        .0
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let mut counted: Vec<Counted<'a>> = Vec::new();
    for outcome in &command.outcomes {
        let name = outcome.name.as_str();
        let wire = outcome.error.as_ref().map(|handle| {
            let error = ir.error(handle);
            error.naming.wire_or(&error.name)
        });
        if let Some(answer) = declared.answer(name) {
            let reader = Reader {
                ir,
                command,
                outcome,
            };
            let redirect = answer
                .redirect
                .as_ref()
                .map(|redirect| Written::of(redirect, &reader));
            counted.push((
                name,
                answer.status,
                answer.body.clone(),
                wire,
                redirect,
                error_fields(ir, command, outcome),
                answer.encoding,
            ));
        } else if !listed_names.contains(name) && !declared.covered.contains(&name) {
            let derived = status::outcome_status(ir, outcome)
                .parse::<u16>()
                .unwrap_or_else(|error| panic!("a derived status is a number: {error}"));
            counted.push((
                name,
                derived,
                Body::Envelope,
                wire,
                None,
                error_fields(ir, command, outcome),
                ResponseEncoding::default(),
            ));
        }
    }
    counted
}

/// A standard refusal the binding overrides answers on this route too (`ESS-HTTP-025`): on the
/// status of a counted outcome, its body must tell the two apart, which only a literal beside an
/// error object on one member, the error's wire name differing from the text, does. Refused where
/// the refusal is declared, `routes[i].refusals.<key>` or `refusals.<key>`, naming the route and
/// every outcome on its status.
fn overridden(
    operation: &Operation,
    declared: &Declared<'_>,
    counted: &[Counted<'_>],
    standard: &Standing,
    path: &str,
    refusals: &mut Refusals,
) {
    for (key, (refusal, at)) in standard {
        let body = refusal.body.spec();
        let clashing: Vec<String> = counted
            .iter()
            .filter(|outcome| outcome.1 == refusal.status)
            .filter(|outcome| match (&outcome.2, &refusal.body) {
                (
                    Body::Error { member },
                    RefusalBody::Literal {
                        member: other,
                        text,
                    },
                ) => member != other || outcome.3 == Some(text.as_str()),
                _ => true,
            })
            .map(|outcome| format!("`{}` ({})", outcome.0, declared.how(outcome)))
            .collect();
        if clashing.is_empty() {
            continue;
        }
        refusals.refuse(
            "ESS-HTTP-025",
            at.as_str(),
            format!(
                "standard refusal `{key}` answers `{}` with `{body}` on `{path}`, and so does \
                 outcome {} of {operation}, with a body a reader cannot tell apart; a refusal and \
                 an outcome on one status are told apart by `{{<member>: {{literal: <text>}}}}` \
                 beside `{{<member>: error}}` on one member, the error's wire name differing \
                 from the text",
                refusal.status,
                clashing.join(", ")
            ),
        );
    }
}

/// A view route lists no `outcomes`, declares no `errors` and no `alike` (`ESS-HTTP-022`): a view
/// has no outcome to answer.
fn no_outcomes_on_a_view(
    spec: &RouteSpec,
    operation: &Operation,
    path: &str,
    refusals: &mut Refusals,
) {
    if spec.outcomes.is_some() {
        refusals.refuse(
            "ESS-HTTP-022",
            format!("{path}.outcomes"),
            format!(
                "{operation} has no outcomes to list: a view answers its rows, or one row with \
                 `body: single`"
            ),
        );
    }
    if spec.errors.is_some() {
        refusals.refuse(
            "ESS-HTTP-022",
            format!("{path}.errors"),
            format!(
                "{operation} has no error outcomes to answer: a view answers its rows, or one \
                 row with `body: single`"
            ),
        );
    }
    if spec.alike.is_some() {
        refusals.refuse(
            "ESS-HTTP-022",
            format!("{path}.alike"),
            format!(
                "{operation} has no outcomes to answer alike: a view answers its rows, or one row \
                 with `body: single`"
            ),
        );
    }
}

/// The body of a `single` view's empty answer (`ESS-HTTP-030` at `<path>.empty.body`): absent or
/// `none`, no body; `{<member>: {literal: <text>}}` under a member with a name, that one member.
/// `None` when refused, or left unchecked (`held` false) because the route answers rows.
fn empty_body(
    written: Option<&BodySpec>,
    held: bool,
    path: &str,
    refusals: &mut Refusals,
) -> Option<RefusalBody> {
    match written {
        None => Some(RefusalBody::None),
        Some(BodySpec::Word(word)) if word == "none" => Some(RefusalBody::None),
        Some(BodySpec::Literal { member, text }) if !member.is_empty() => {
            Some(RefusalBody::Literal {
                member: member.clone(),
                text: text.clone(),
            })
        }
        Some(written) => {
            if held {
                refusals.refuse(
                    "ESS-HTTP-030",
                    format!("{path}.empty.body"),
                    format!(
                        "`{written}` is not an empty answer's body: it is `none`, or \
                         `{{<member>: {{literal: <text>}}}}` under a member with a name"
                    ),
                );
            }
            None
        }
    }
}

/// A view route's body and empty answer, checked: no `outcomes` (`ESS-HTTP-022`), `rows`,
/// `array` or `single`, never `array` or `single` on a paged view (`ESS-HTTP-024`), `single`
/// exactly with an `empty` holding a `4xx` and a body that is none or one literal member
/// (`ESS-HTTP-030`), and `single` only where the filter selects by identity (`ESS-HTTP-031`).
fn view_answers(
    spec: &RouteSpec,
    operation: &Operation,
    view: Option<(&EssIr, &ResolvedView)>,
    path: &str,
    refusals: &mut Refusals,
) -> Answers {
    no_outcomes_on_a_view(spec, operation, path, refusals);
    // `None` when the body is refused, which leaves `empty` unchecked against it.
    let body = view_body(spec, operation, view, path, refusals);
    let form = body.as_ref().map(|body| body.form);
    match (form, &spec.empty) {
        (Some(ViewBody::Single), None) => refusals.refuse(
            "ESS-HTTP-030",
            path,
            "`body: single` answers one row, and needs `empty` (`{status: <4xx>}`), the answer \
             for a view that holds none",
        ),
        (Some(written @ (ViewBody::Rows | ViewBody::Array)), Some(_)) => refusals.refuse(
            "ESS-HTTP-030",
            path,
            format!(
                "`empty` answers a `single` view that holds no row, and this route answers `{}`",
                written.word()
            ),
        ),
        _ => {}
    }
    let held = !matches!(form, Some(ViewBody::Rows | ViewBody::Array));
    // An `empty` beside `rows` or `array` is refused above as a whole, so its status and body are
    // not refused again.
    let empty = spec.empty.as_ref().and_then(|empty| {
        let status = u16::try_from(empty.status)
            .ok()
            .filter(|status| (400..500).contains(status));
        if status.is_none() && held {
            refusals.refuse(
                "ESS-HTTP-030",
                path,
                format!(
                    "`empty` answers `{}`; the answer of a `single` view that holds no row is a \
                     `4xx` status",
                    empty.status
                ),
            );
        }
        let body = empty_body(empty.body.as_ref(), held, path, refusals);
        status
            .zip(body)
            .map(|(status, body)| Empty { status, body })
    });
    if form == Some(ViewBody::Single) {
        if let Some((ir, view)) = view {
            let entity = ir.entity(&view.source);
            let identity = entity.identity.name.as_str();
            if !view
                .filter
                .as_ref()
                .is_some_and(|filter| selects_identity(filter, identity))
            {
                refusals.refuse(
                    "ESS-HTTP-031",
                    format!("{path}.body"),
                    format!(
                        "`body: single` answers one row, and view `{}` does not select its source \
                         entity `{}` by its identity field `{identity}`: neither its filter nor a \
                         member of a top-level `all` is `{identity} == param.<name>`, so it can \
                         hold more than one row",
                        view.name, entity.name
                    ),
                );
            }
        }
    }
    Answers {
        outcomes: BTreeMap::new(),
        errors: None,
        alike: Vec::new(),
        body: body.unwrap_or_default(),
        empty,
        representations: Vec::new(),
    }
}

/// A view route's `body`: `rows`, `array` or `single`, never `array` or `single` on a paged view
/// (`ESS-HTTP-024`); `None` when it is refused.
fn view_body(
    spec: &RouteSpec,
    operation: &Operation,
    view: Option<(&EssIr, &ResolvedView)>,
    path: &str,
    refusals: &mut Refusals,
) -> Option<ViewAnswer> {
    let form = |word: &str| match word {
        "rows" => Some(ViewBody::Rows),
        "single" => Some(ViewBody::Single),
        "array" => Some(ViewBody::Array),
        _ => None,
    };
    let body = match &spec.body {
        None => Some(ViewAnswer::default()),
        Some(written) => match written {
            BodySpec::Word(word) => form(word).map(ViewAnswer::bare),
            // `{rows: array}` is `rows`, ESS's own answer; `rows` is an object already, so it is
            // never wrapped (beyond10x/ess#755).
            BodySpec::Wrapped { member, form: word } if !member.is_empty() => match form(word) {
                Some(ViewBody::Array)
                    if member == "rows" && view.is_none_or(|(_, view)| view.paging.is_none()) =>
                {
                    Some(ViewAnswer::default())
                }
                Some(form @ (ViewBody::Single | ViewBody::Array)) => Some(ViewAnswer {
                    form,
                    member: Some(member.clone()),
                }),
                _ => None,
            },
            _ => None,
        },
    };
    if body.is_none() {
        let written = spec
            .body
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        refusals.refuse(
            "ESS-HTTP-024",
            format!("{path}.body"),
            format!(
                "{operation} answers `rows`, `array`, `single`, `{{<member>: array}}` or \
                 `{{<member>: single}}` under a named member, and `{written}` is not a view's body"
            ),
        );
    }
    let form = body.as_ref().map(|body| body.form);
    if matches!(form, Some(ViewBody::Single | ViewBody::Array)) {
        if let Some((_, view)) = view.filter(|(_, view)| view.paging.is_some()) {
            let answer = if form == Some(ViewBody::Single) {
                "one bare row"
            } else {
                "a bare array of rows"
            };
            refusals.refuse(
                "ESS-HTTP-024",
                format!("{path}.body"),
                format!(
                    "view `{}` is paged: its answer carries a page, which {answer} cannot",
                    view.name
                ),
            );
        }
    }
    body
}

/// A view route's `representations`, checked and compiled in the order written
/// (beyond10x/ess#753): refused (`ESS-HTTP-035`) on a command, a `single` view or a paged view,
/// empty, with a `query` of other than one parameter, on a parameter other than the first
/// representation's, a view parameter's wire name or a path variable, on a name or value that is
/// not `unreserved` characters or a value given twice, with a `separator` outside `,`, `;` and a
/// tab, or over a view field without a flat spelling; its headers are literals, each checked as an
/// answer's header is (`ESS-HTTP-026`). Empty when the route declares none.
fn representations(
    spec: &RouteSpec,
    operation: &Operation,
    target: Option<&Target<'_>>,
    variables: &[String],
    path: &str,
    refusals: &mut Refusals,
) -> Result<Vec<Representation>, Refused> {
    let Some(written) = &spec.representations else {
        return Ok(Vec::new());
    };
    let before = refusals.0.len();
    let key = format!("{path}.representations");
    let view = target.and_then(|target| match target.served {
        Served::View(view) => Some((target.ir, view)),
        Served::Command(_) => None,
    });
    if let Some(reason) = misplaced(spec, operation, view) {
        refusals.refuse("ESS-HTTP-035", key.as_str(), reason);
    } else if written.is_empty() {
        refusals.refuse(
            "ESS-HTTP-035",
            key.as_str(),
            "`representations` is empty; a route that answers its `body` only leaves it out",
        );
    }
    let mut selectors: Vec<Selector> = Vec::new();
    let mut compiled = Vec::new();
    for (index, representation) in written.iter().enumerate() {
        let at = format!("{key}[{index}]");
        let query = selector(representation, view, variables, &selectors, &at, refusals);
        if let Some(query) = &query {
            selectors.push(query.clone());
        }
        let csv = &representation.encoding.csv;
        let separator = match csv.separator.as_deref() {
            None => Some(Separator::default()),
            Some(text) => Separator::parse(text).or_else(|| {
                refusals.refuse(
                    "ESS-HTTP-035",
                    format!("{at}.encoding.csv.separator"),
                    format!(
                        "{text:?} is not a CSV separator: it is `\",\"`, `\";\"` or `\"\\t\"`, \
                         a tab"
                    ),
                );
                None
            }),
        };
        if let Some((ir, view)) = view {
            columns(ir, view, &at, refusals);
        }
        let headers = representation_headers(representation.headers.as_ref(), &at, refusals);
        if let (Some(query), Some(separator), Ok(headers)) = (query, separator, headers) {
            compiled.push(Representation {
                query,
                encoding: RepresentationEncoding {
                    csv: Csv {
                        separator,
                        header: csv.header.unwrap_or(true),
                    },
                },
                headers,
            });
        }
    }
    if refusals.0.len() == before {
        Ok(compiled)
    } else {
        Err(Refused)
    }
}

/// Why a route may not declare `representations`, or `None` where it may: a command route, a
/// `single` view route and a paged view's route are refused (`ESS-HTTP-035`).
fn misplaced(
    spec: &RouteSpec,
    operation: &Operation,
    view: Option<(&EssIr, &ResolvedView)>,
) -> Option<String> {
    let single = match &spec.body {
        Some(BodySpec::Word(form) | BodySpec::Wrapped { form, .. }) => form == "single",
        _ => false,
    };
    match (operation, view) {
        (Operation::Command(_), _) => Some(format!(
            "{operation} answers per outcome, under `outcomes`; only a view route has \
             representations of its rows"
        )),
        _ if single => Some(format!(
            "{operation} answers `single`, one row, and a representation answers a list of rows"
        )),
        (_, Some((_, view))) if view.paging.is_some() => Some(format!(
            "view `{}` is paged: its answer carries a page, which a representation of its rows \
             cannot",
            view.name
        )),
        _ => None,
    }
}

/// Refuses each field of `view` that no CSV column spells (`ESS-HTTP-035`), at `{at}.encoding`:
/// a column is a scalar or an `Optional` scalar ([`optional_scalar`]).
fn columns(ir: &EssIr, view: &ResolvedView, at: &str, refusals: &mut Refusals) {
    for field in &view.fields {
        if !optional_scalar(&field.type_ref, ir) {
            refusals.refuse(
                "ESS-HTTP-035",
                format!("{at}.encoding"),
                format!(
                    "view field `{}` has type `{}`, which has no flat spelling in a CSV column: a \
                     column is a scalar or an `Optional` scalar, a scalar being a primitive other \
                     than `Json`, an enum, or a newtype of a scalar",
                    field.name, field.type_ref
                ),
            );
        }
    }
}

/// The variables of a route's path, in path order; none for an open path or one that did not
/// parse.
fn path_variables(segments: Option<&Option<Vec<Segment>>>) -> Vec<String> {
    segments
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|segment| match segment {
            Segment::Variable(name) => Some(name.clone()),
            Segment::Literal(_) => None,
        })
        .collect()
}

/// A representation's `query`, checked (`ESS-HTTP-035`) at `{at}.query`: exactly one parameter,
/// the one the first of `earlier` selects by, never a view parameter's wire name or a path
/// variable, its name and value `unreserved` characters, and a value no earlier representation
/// has. `None` once refused.
fn selector(
    representation: &RepresentationSpec,
    view: Option<(&EssIr, &ResolvedView)>,
    variables: &[String],
    earlier: &[Selector],
    at: &str,
    refusals: &mut Refusals,
) -> Option<Selector> {
    let at = format!("{at}.query");
    let [(parameter, value)] = &representation.query.0[..] else {
        refusals.refuse(
            "ESS-HTTP-035",
            at,
            format!(
                "`query` holds {} parameters; a representation is selected by exactly one \
                 parameter and one value of it, `{{<parameter>: <value>}}`",
                representation.query.0.len()
            ),
        );
        return None;
    };
    let before = refusals.0.len();
    let mut refuse = |reason: String| refusals.refuse("ESS-HTTP-035", at.as_str(), reason);
    if !unreserved(parameter) {
        refuse(format!(
            "{parameter:?} is not a selector parameter: a parameter name is one or more RFC 3986 \
             `unreserved` characters, letters, digits and `-._~`"
        ));
    } else if let Some(first) = earlier
        .first()
        .filter(|first| first.parameter != *parameter)
    {
        refuse(format!(
            "this representation is selected by `{parameter}`, and the first by `{}`; every \
             representation of a route is selected by one parameter",
            first.parameter
        ));
    } else if variables.contains(parameter) {
        refuse(format!(
            "`{parameter}` is a variable of the route's path; a representation is selected by a \
             query parameter the route does not read otherwise"
        ));
    } else if let Some((_, view)) = view.filter(|(_, view)| {
        view.params
            .iter()
            .any(|param| param.naming.wire.as_deref().unwrap_or(&param.name) == parameter)
    }) {
        refuse(format!(
            "`{parameter}` is the wire name of a parameter of view `{}`; a representation is \
             selected by a query parameter the route does not read otherwise",
            view.name
        ));
    }
    if !unreserved(value) {
        refuse(format!(
            "{value:?} is not a selector value: a value is one or more RFC 3986 `unreserved` \
             characters, letters, digits and `-._~`"
        ));
    } else if earlier
        .iter()
        .any(|earlier| earlier.parameter == *parameter && earlier.value == *value)
    {
        refuse(format!(
            "`{parameter}={value}` selects an earlier representation already; each value \
             selects one"
        ));
    }
    (refusals.0.len() == before).then(|| Selector {
        parameter: parameter.clone(),
        value: value.clone(),
    })
}

/// A representation's `headers`, compiled: each a literal, checked as an answer's header is
/// (`ESS-HTTP-026`); a source or a challenge is refused (`ESS-HTTP-026`), since a representation
/// adds the same text to every answer.
fn representation_headers(
    written: Option<&Entries<HeaderSpec>>,
    at: &str,
    refusals: &mut Refusals,
) -> Result<BTreeMap<String, Header>, Refused> {
    let before = refusals.0.len();
    let mut literals = Entries::default();
    for (name, header) in written.map_or(&[][..], |entries| &entries.0[..]) {
        match header {
            HeaderSpec::Literal(_) => literals.0.push((name.clone(), header.clone())),
            HeaderSpec::Source(_) | HeaderSpec::Challenge(_) => refusals.refuse(
                "ESS-HTTP-026",
                format!("{at}.headers.{name}"),
                format!(
                    "header `{name}` is not a literal: a representation's headers are \
                     `{{literal: <text>}}`, the same text on every answer"
                ),
            ),
        }
    }
    let headers = answer_headers(Some(&literals), None, None, at, refusals);
    if refusals.0.len() == before {
        headers
    } else {
        Err(Refused)
    }
}

/// Whether `filter` holds only of the row whose `identity` equals a view parameter: it is
/// `<identity> == param.<name>`, compared by value, in either operand order, or an `all` one of
/// whose members does, `all` nested in `all` included.
fn selects_identity(filter: &Predicate, identity: &str) -> bool {
    match filter {
        Predicate::All(members) => members
            .iter()
            .any(|member| selects_identity(member, identity)),
        Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq,
            right: Operand::Fact(right),
            kind: CompareKind::Value,
        } => {
            let is_identity = |path: &FactPath| path.segments() == [identity.to_owned()];
            let is_param =
                |path: &FactPath| matches!(path.segments(), [namespace, _] if namespace == "param");
            (is_identity(left) && is_param(right)) || (is_param(left) && is_identity(right))
        }
        _ => false,
    }
}

/// Whether a value of `type_ref` fits one path segment: always present and one value with one
/// text spelling, so `String`, `Integer`, `Boolean`, `Decimal`, `Timestamp`, `Duration`, `Uuid`, an
/// enum, or an unconstrained newtype of one of them (beyond10x/ess#755), and never `Optional`,
/// `List`, `Map`, a struct, a union, `Bytes` (its base64 text holds `/`), `Binary64` (no one text
/// spelling of a JSON number) or `Json`.
fn segment_type(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => matches!(
            name,
            Primitive::String
                | Primitive::Integer
                | Primitive::Boolean
                | Primitive::Decimal
                | Primitive::Timestamp
                | Primitive::Duration
                | Primitive::Uuid
        ),
        ResolvedTypeRef::Declared { name } => {
            let declared = ir.named_type(name);
            match &declared.body {
                ResolvedBody::Enum { .. } => true,
                ResolvedBody::Newtype { of, .. } => {
                    !declared.body.is_constrained() && segment_type(of, ir)
                }
                ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => false,
            }
        }
        ResolvedTypeRef::Optional { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => false,
    }
}

/// One route on one component's surface, declared or derived.
struct Claim<'a> {
    operation: &'a Operation,
    method: Method,
    path: &'a str,
    segments: Vec<Segment>,
    /// Where the binding declares it; `None` for a derived route.
    declared: Option<usize>,
    /// A declared route's `select`; `None` for a derived route.
    select: Option<&'a Selecting>,
    /// Whether its body is a `json` input object, the only body `select` tells apart.
    json_body: bool,
}

/// Refuses two declared routes of one component on one method and template, `this` at
/// `routes[at]`, unless their selects tell them apart (`ESS-HTTP-010` for the same tests,
/// `ESS-HTTP-011` for crossed ones); of nested selects the more specific answers.
fn shared(
    this: &Claim<'_>,
    other: &Claim<'_>,
    component: &str,
    at: usize,
    refusals: &mut Refusals,
) {
    match told(this, other) {
        Told::Apart | Told::Unknown => {}
        Told::Same => refusals.refuse(
            "ESS-HTTP-010",
            format!("routes[{at}]"),
            format!(
                "{COLLISION}: {} and {} are one route on component `{component}`, since variable \
                 names do not tell two templates apart; tell them apart with `select:`, literal \
                 values of members of a `json` body that is the input object, at most one of them \
                 without it",
                this.describe(),
                other.describe()
            ),
        ),
        Told::Crossed(mine, theirs, body) => refusals.refuse(
            "ESS-HTTP-011",
            format!("routes[{at}]"),
            format!(
                "{} and {} cross on component `{component}`: both select the body `{body}`, and \
                 neither is more specific, since the one tests `{mine}` where the other leaves it \
                 open and the other tests `{theirs}` where the one leaves it open; a select takes \
                 precedence only where it tests every member the other tests, with the same \
                 literals",
                this.describe(),
                other.describe()
            ),
        ),
    }
}

/// How two declared routes on one method and template are told apart by their selects.
enum Told {
    /// Every two alternatives are disjoint or nested: the more specific answers.
    Apart,
    /// Two alternatives test the same members for the same literals, two catch-alls among them, or
    /// a catch-all that is not a `json` body beside a select (`ESS-HTTP-010`).
    Same,
    /// Two alternatives cross: the first's member the second leaves open, the second's the first
    /// leaves open, and a body that matches both (`ESS-HTTP-011`).
    Crossed(MemberPath, MemberPath, String),
    /// A select was refused already.
    Unknown,
}

/// How the selects of two routes on one template tell them apart; a route without `select` is
/// the catch-all, one alternative testing nothing.
fn told(this: &Claim<'_>, other: &Claim<'_>) -> Told {
    const CATCH_ALL: &[Select] = &[Select(Vec::new())];
    fn alternatives<'a>(claim: &Claim<'a>) -> Option<&'a [Select]> {
        match claim.select {
            Some(Selecting::Selects(selects)) => Some(selects.as_slice()),
            Some(Selecting::Unwritten) => Some(CATCH_ALL),
            Some(Selecting::Refused) | None => None,
        }
    }
    let (Some(mine), Some(theirs)) = (alternatives(this), alternatives(other)) else {
        return Told::Unknown;
    };
    let selected = |claim: &Claim<'_>| matches!(claim.select, Some(Selecting::Selects(_)));
    if (selected(this) || selected(other)) && !(this.json_body && other.json_body) {
        return Told::Same;
    }
    for one in mine {
        for two in theirs {
            match one.meet(two) {
                Some(Meeting::Same) => return Told::Same,
                Some(Meeting::Crossed(left, right)) => {
                    return Told::Crossed(left, right, one.common_body(two));
                }
                Some(Meeting::Nested) | None => {}
            }
        }
    }
    Told::Apart
}

impl Claim<'_> {
    fn describe(&self) -> String {
        match self.declared {
            Some(index) => format!(
                "{} (`routes[{index}]`, `{} {}`)",
                self.operation, self.method, self.path
            ),
            None => format!(
                "{} (its derived route, `{} {}`)",
                self.operation, self.method, self.path
            ),
        }
    }
}

/// How two templates on one method meet.
#[derive(Debug, PartialEq, Eq)]
enum Clash {
    /// The same template once variable names are ignored (`ESS-HTTP-010`).
    Same,
    /// Different templates one request path matches both of, nested: at every position they
    /// differ, one and the same template holds the literal and the other a variable, so its paths
    /// are a strict subset of the other's and it answers them
    /// (`story:a-literal-path-segment-takes-precedence-over-a-variable-one`).
    Nested,
    /// Different templates one request path matches both of, crossed: each holds a literal at some
    /// position the other holds a variable, `left` the first such position of the first template
    /// and `right` of the second, zero-based. Neither is more specific (`ESS-HTTP-011`).
    Crossed {
        /// Where the first template first holds the literal.
        left: usize,
        /// Where the second template first holds the literal.
        right: usize,
    },
}

/// Whether two templates meet. They can only when they have as many segments and no position
/// holds two different literals; then they are the same template but for variable names, nested
/// (one holds the literal at every position they differ), or crossed.
fn clash(left: &[Segment], right: &[Segment]) -> Option<Clash> {
    if left.len() != right.len() {
        return None;
    }
    let (mut left_literal, mut right_literal) = (None, None);
    for (position, pair) in left.iter().zip(right).enumerate() {
        match pair {
            (Segment::Literal(left), Segment::Literal(right)) if left != right => return None,
            (Segment::Literal(_), Segment::Variable(_)) => {
                left_literal.get_or_insert(position);
            }
            (Segment::Variable(_), Segment::Literal(_)) => {
                right_literal.get_or_insert(position);
            }
            _ => {}
        }
    }
    Some(match (left_literal, right_literal) {
        (None, None) => Clash::Same,
        (Some(left), Some(right)) => Clash::Crossed { left, right },
        _ => Clash::Nested,
    })
}

/// A request path two templates that meet both match: each position's literal where either holds
/// one, the first template's variable where both hold variables.
fn common_path(left: &[Segment], right: &[Segment]) -> String {
    let mut path = String::new();
    for pair in left.iter().zip(right) {
        path.push('/');
        let segment = match pair {
            (literal @ Segment::Literal(_), _) | (_, literal @ Segment::Literal(_)) => literal,
            (variable, _) => variable,
        };
        path.push_str(&segment.to_string());
    }
    if path.is_empty() {
        path.push('/');
    }
    path
}

/// A derived path's segments: literals only, since ESS derives no variable.
fn literal_segments(path: &str) -> Vec<Segment> {
    let rest = path.strip_prefix('/').unwrap_or(path);
    if rest.is_empty() {
        return Vec::new();
    }
    rest.split('/')
        .map(|text| Segment::Literal(text.to_owned()))
        .collect()
}

/// Checks every component's surface, declared and derived routes together, for two operations
/// on one method and template (`ESS-HTTP-010`) and for templates that cross (`ESS-HTTP-011`); of
/// nested templates the more specific answers ([`Clash::Nested`]). Returns the derived routes that
/// stay: those of every operation the binding does not name, in IR order.
///
/// A declared route is on the surface of every component the table derives a route for its
/// operation on, and replaces that route there. An operation is named by every route that names
/// it, whatever else that route was refused for, so a refused route still replaces the derived
/// one; and every route whose method is admitted and whose path parsed is checked, so that its
/// collisions are reported beside its other refusals. An open route claims no path. Two derived routes
/// never collide with each other here: ESS derives them apart, and a refusal of the binding names
/// a route of the binding.
fn surface(
    bound: &[Bound],
    derived: &[DerivedRoute],
    refusals: &mut Refusals,
) -> Vec<DerivedRoute> {
    let mut named: BTreeMap<&Operation, Vec<&Bound>> = BTreeMap::new();
    for route in bound {
        named.entry(&route.operation).or_default().push(route);
    }
    let mut table: Vec<&DerivedRoute> = derived.iter().collect();
    table.sort_by(|left, right| derived_order(left, right));
    table.dedup();
    let mut kept = Vec::new();
    let mut components: BTreeMap<&str, Vec<Claim<'_>>> = BTreeMap::new();
    for route in table {
        let claims = components.entry(route.component.as_str()).or_default();
        if let Some(routes) = named.get(&route.operation) {
            for bound in routes {
                if let Some((method, path, segments)) = &bound.claim {
                    claims.push(Claim {
                        operation: &bound.operation,
                        method: *method,
                        path,
                        segments: segments.clone(),
                        declared: Some(bound.index),
                        select: Some(&bound.select),
                        json_body: bound.json_body,
                    });
                }
            }
        } else {
            kept.push(route.clone());
            claims.push(Claim {
                operation: &route.operation,
                method: route.method,
                path: &route.path,
                segments: literal_segments(&route.path),
                declared: None,
                select: None,
                json_body: false,
            });
        }
    }
    for (component, claims) in &components {
        for (later, right) in claims.iter().enumerate() {
            for left in &claims[..later] {
                if left.method != right.method || left.operation == right.operation {
                    continue;
                }
                let Some(at) = left.declared.max(right.declared) else {
                    continue;
                };
                let (this, other) = if right.declared == Some(at) {
                    (right, left)
                } else {
                    (left, right)
                };
                let both = this.declared.is_some() && other.declared.is_some();
                match clash(&this.segments, &other.segments) {
                    Some(Clash::Same) if both => {
                        shared(this, other, component, at, refusals);
                    }
                    Some(Clash::Same) => refusals.refuse(
                        "ESS-HTTP-010",
                        format!("routes[{at}]"),
                        format!(
                            "{COLLISION}: {} and {} are one route on component `{component}`, \
                             since variable names do not tell two templates apart",
                            this.describe(),
                            other.describe()
                        ),
                    ),
                    // story:a-literal-path-segment-takes-precedence-over-a-variable-one: crossed
                    // templates stay refused, and of nested ones the more specific answers.
                    Some(Clash::Crossed { left, right }) => refusals.refuse(
                        "ESS-HTTP-011",
                        format!("routes[{at}]"),
                        format!(
                            "{} and {} cross on component `{component}`: both match `{}`, and \
                             neither is more specific, since segment {} is `{}` in one where the \
                             other has `{}`, and segment {} is `{}` in the other where the one has \
                             `{}`; a literal takes precedence over a variable only where one \
                             template holds the literal at every segment the two differ",
                            this.describe(),
                            other.describe(),
                            common_path(&this.segments, &other.segments),
                            left + 1,
                            this.segments[left],
                            other.segments[left],
                            right + 1,
                            other.segments[right],
                            this.segments[right]
                        ),
                    ),
                    Some(Clash::Nested) | None => {}
                }
            }
        }
    }
    kept
}

/// The order derived routes are recorded in: component, then path, then method, then operation.
fn derived_order(left: &DerivedRoute, right: &DerivedRoute) -> std::cmp::Ordering {
    (&left.component, &left.path, left.method, &left.operation).cmp(&(
        &right.component,
        &right.path,
        right.method,
        &right.operation,
    ))
}

#[cfg(test)]
mod tests {
    use super::{clash, common_path, segment, template, Clash, Segment, Source};

    fn literal(text: &str) -> Segment {
        Segment::Literal(text.to_owned())
    }

    fn variable(name: &str) -> Segment {
        Segment::Variable(name.to_owned())
    }

    #[test]
    fn a_template_parses_into_literal_and_variable_segments() {
        assert_eq!(template("/"), Ok(Vec::new()));
        assert_eq!(
            template("/items/{item_id}/touch"),
            Ok(vec![
                literal("items"),
                variable("item_id"),
                literal("touch")
            ])
        );
        assert_eq!(
            template("/.well-known/oauth-authorization-server"),
            Ok(vec![
                literal(".well-known"),
                literal("oauth-authorization-server")
            ])
        );
    }

    #[test]
    fn each_reason_a_template_breaks_names_its_segment() {
        assert_eq!(
            template("items").unwrap_err(),
            ["`items` does not start with `/`; a path template is absolute"]
        );
        let reasons = template("/a//./{x}.json/%41/{_ok}/").unwrap_err();
        assert_eq!(reasons.len(), 5, "{reasons:?}");
        for (reason, named) in reasons.iter().zip([
            "segment 2 ",
            "segment 3 ",
            "`{x}.json`",
            "`%41`",
            "segment 7 ",
        ]) {
            assert!(reason.contains(named), "{named}: {reason}");
        }
        for text in [
            "{}", "{9}", "{a-b}", "{a", "a}", "{{a}}", "a{b}", "é", " ", "[", "\"",
        ] {
            assert!(segment(text).is_err(), "{text:?}");
        }
        for text in ["{a}", "{_}", "{A_9}"] {
            assert!(
                matches!(segment(text), Ok(Segment::Variable(_))),
                "{text:?}"
            );
        }
        for text in ["...", "a.b", "~", "!$&'()*+,;=:@", "-_"] {
            assert!(matches!(segment(text), Ok(Segment::Literal(_))), "{text:?}");
        }
    }

    #[test]
    fn templates_clash_when_one_request_path_matches_both() {
        let same = clash(
            &[literal("items"), variable("a")],
            &[literal("items"), variable("b")],
        );
        assert_eq!(same, Some(Clash::Same));
        let overlap = clash(
            &[literal("items"), literal("current")],
            &[literal("items"), variable("id")],
        );
        // story:a-literal-path-segment-takes-precedence-over-a-variable-one: one template holds the
        // literal at every position they differ, nested; each holds one the other does not,
        // crossed, naming the first position of each.
        assert_eq!(overlap, Some(Clash::Nested));
        let later = clash(
            &[variable("a"), literal("b"), variable("c")],
            &[variable("x"), variable("y"), literal("z")],
        );
        assert_eq!(later, Some(Clash::Crossed { left: 1, right: 2 }));
        assert_eq!(
            common_path(
                &[variable("a"), literal("b"), variable("c")],
                &[variable("x"), variable("y"), literal("z")],
            ),
            "/{a}/b/z"
        );
        assert_eq!(clash(&[literal("a")], &[literal("b")]), None);
        assert_eq!(
            clash(&[variable("a")], &[variable("a"), literal("b")]),
            None
        );
        assert_eq!(
            clash(
                &[literal("a"), variable("x"), literal("b")],
                &[literal("a"), literal("c"), literal("d")]
            ),
            None
        );
        assert_eq!(clash(&[], &[]), Some(Clash::Same));
    }

    #[test]
    fn a_source_is_one_namespace_and_one_name() {
        assert_eq!(
            Source::parse("input.item_id"),
            Some(Source::Input("item_id".into()))
        );
        assert_eq!(
            Source::parse("param.item_id"),
            Some(Source::Param("item_id".into()))
        );
        for text in [
            "item_id",
            "input.",
            "input.a.b",
            "query.a",
            "event.a",
            ".a",
            "",
        ] {
            assert_eq!(Source::parse(text), None, "{text:?}");
        }
        assert_eq!(Source::Input("a".into()).to_string(), "input.a");
    }
}
