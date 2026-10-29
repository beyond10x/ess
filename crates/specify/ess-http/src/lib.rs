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
];

/// The wording of a route collision, the same as the `OpenAPI` import's refusal of two operations
/// on one location, so the two read alike.
pub const COLLISION: &str = "two operations claim the same method and path";

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
}

impl Encoding {
    /// Every encoding, in the order the format lists them.
    pub const ALL: [Self; 3] = [Self::Json, Self::Form, Self::Query];

    /// The encoding as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Form => "form",
            Self::Query => "query",
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
    /// A parameter no field names makes the request malformed.
    #[default]
    Refuse,
    /// A parameter no field names is ignored (RFC 6749 §3.1).
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

/// A route's `request`, as written. Each value is read as text so that one outside its set is
/// refused with a code and a path (`ESS-HTTP-020`) rather than as a parse error.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestSpec {
    /// `json`, `form` or `query`; absent, `json`.
    #[serde(default)]
    pub encoding: Option<String>,
    /// `refuse` or `ignore`; absent, `refuse`.
    #[serde(default)]
    pub unknown: Option<String>,
}

/// A compiled route's `request`. Each key whose value is its default is omitted from the IR, and
/// a route whose request is all defaults carries none, so two documents that mean the same compile
/// to the same bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// How the request carries the input.
    #[serde(
        default,
        skip_serializing_if = "Encoding::is_default",
        deserialize_with = "omitted_default"
    )]
    pub encoding: Encoding,
    /// What a request parameter no field names does.
    #[serde(
        default,
        skip_serializing_if = "Unknown::is_default",
        deserialize_with = "omitted_default"
    )]
    pub unknown: Unknown,
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
}

impl BodySpec {
    /// The word, when it is one.
    fn word(&self) -> Option<&str> {
        match self {
            Self::Word(word) => Some(word),
            Self::Error(_) | Self::Literal { .. } => None,
        }
    }
}

impl fmt::Display for BodySpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Word(word) => formatter.write_str(word),
            Self::Error(member) => write!(formatter, "{{{member}: error}}"),
            Self::Literal { member, text } => {
                write!(formatter, "{{{member}: {{literal: {text}}}}}")
            }
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
        }
    }
}

/// The value of a body object's one member: `error` or `{literal: <text>}`.
enum MemberValue {
    Error,
    Literal(String),
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
                formatter.write_str("`error` or `{literal: <text>}`")
            }

            fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<Self::Value, E> {
                if text == "error" {
                    Ok(MemberValue::Error)
                } else {
                    Err(E::custom(format!(
                        "`{text}` is not what a body member holds: `error`, or \
                         `{{literal: <text>}}`"
                    )))
                }
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
        }
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
}

impl ViewBody {
    // `skip_serializing_if` passes the field by reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// Reads a view route's `body` of compiled IR, refusing `rows` written out: compiled IR omits it.
fn omitted_rows<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<ViewBody, D::Error> {
    let body = <ViewBody as serde::Deserialize>::deserialize(deserializer)?;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptySpec {
    /// A `4xx` status; required, and the only key.
    pub status: i64,
}

/// A `single` view route's answer for a view that holds no row, compiled: a `4xx` status and no
/// body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Empty {
    /// The status.
    pub status: u16,
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
    /// An unfinished realization; today `501`. Answered with a `5xx`.
    Unfinished,
}

impl RefusalKey {
    /// Every key, in name order.
    pub const ALL: [Self; 3] = [Self::Malformed, Self::NotGranted, Self::Unfinished];

    /// The key as a binding spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::NotGranted => "not_granted",
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
            Self::Malformed | Self::NotGranted => 4,
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
    /// A view route's body, `rows` or `single`; absent, `rows`.
    #[serde(default)]
    pub body: Option<BodySpec>,
    /// A `single` view route's answer for a view that holds no row.
    #[serde(default)]
    pub empty: Option<EmptySpec>,
}

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
    /// The standard refusals this binding answers its own way, keyed `malformed`, `not_granted`
    /// or `unfinished`; one not listed keeps today's answer.
    #[serde(default)]
    pub refusals: Option<Entries<RefusalSpec>>,
    /// The base URLs a `reached_by: network` component's paths resolve against, keyed by the
    /// component, each list in the order written: the first is the default.
    #[serde(default)]
    pub servers: Option<Entries<Vec<ServerSpec>>>,
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
        skip_serializing_if = "ViewBody::is_default",
        deserialize_with = "omitted_rows"
    )]
    pub body: ViewBody,
    /// A `single` view route's answer for a view that holds no row; present exactly with `single`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty: Option<Empty>,
}

impl Route {
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

    /// Each `reached_by: network` component's base URLs the binding declares, by component name,
    /// each list in the order written: the first is the default.
    pub fn servers(&self) -> &BTreeMap<String, Vec<Server>> {
        &self.servers
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
    /// beside `json`; `outcomes` on a view, or `body` or `empty` on a command; `outcomes` or `refusals` out of key
    /// order or empty; an outcome status outside `100..=599`, a `1xx` or a `3xx` other than the
    /// [`REDIRECTS`]; an outcome body that is `envelope` written out, a literal, a view's body, an
    /// error object under an empty member, or anything but `none` on `204`; an outcome `encoding`
    /// `json` written out, or `text` beside any body but `response.<field>`; `alike` on a view,
    /// written empty, a group of fewer than two names, a name in two groups or twice, names out of
    /// name order or groups out of the order of their first names, a name no listed outcome or
    /// `errors` answers, or a group whose answers differ or hold an `envelope`; a view body `rows`
    /// written out, `single` without `empty` or `empty` without `single`, or an `empty` status
    /// outside `4xx`; a refusal key outside `malformed`, `not_granted` and `unfinished`, a refusal
    /// status outside its class (`4xx`, `4xx`, `5xx`), or a refusal body other than `none` or one
    /// literal member with a name; `headers`, challenge `params` or redirect `parameters` out of
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
    /// component. Beyond these, it refuses, naming
    /// the key, any IR that would not write back what it was given: a key compiled IR omits,
    /// written as an empty map or list, a `null` or a default.
    ///
    /// What needs the specification is not checked: that each operation, field and outcome exists,
    /// its type, the `get` input, an outcome's status class and body against its error and
    /// `returns`, whether a `text` field is a `String`, whether an `alike` name is an outcome of
    /// the command and whether error objects alike share one wire name, the distinguishability of
    /// outcomes and overridden refusals on one status, a
    /// `single` view's identity selection and paging, whether the outcome supplies each header,
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
        for (key, refusal) in &self.refusals {
            refusal_invariants(*key, refusal)?;
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
        self.presentation_invariants()
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
        request_invariants(route, *request, at)?;
    }
    answer_invariants(route, at)
}

/// The invariants of one compiled route's answers (see [`HttpIr::from_json`]): `outcomes` on a
/// command, each with a status an outcome may answer and a body admitted beside it; `body` and
/// `empty` on a view, `single` exactly with an `empty` holding a `4xx`.
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
            match (route.body, route.empty) {
                (ViewBody::Single, None) => Err(format!(
                    "`{at}.body` is `single`, and `{at}.empty` is missing; compiled IR writes \
                     the empty answer of every `single` route"
                )),
                (ViewBody::Rows, Some(_)) => Err(format!(
                    "`{at}.empty` is on a route that answers `rows`; only a `single` route has \
                     an empty answer"
                )),
                (_, Some(empty)) if !(400..500).contains(&empty.status) => Err(format!(
                    "`{at}.empty.status` `{}` is not a `4xx` status",
                    empty.status
                )),
                _ => Ok(()),
            }
        }
        Operation::Command(_) => {
            if route.body != ViewBody::Rows {
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
                if matches!(errors.body, Body::Response | Body::ResponseField(_)) {
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
/// key's class, and a literal body under a named member when it has one.
fn refusal_invariants(key: RefusalKey, refusal: &Refusal) -> Result<(), String> {
    if refusal.status / 100 != key.class() {
        return Err(format!(
            "`refusals.{key}.status` `{}` is not a `{}xx` status, the class of `{key}`",
            refusal.status,
            key.class()
        ));
    }
    if let RefusalBody::Literal { member, .. } = &refusal.body {
        if member.is_empty() {
            return Err(format!(
                "`refusals.{key}.body` names an empty member; a literal body has one named member"
            ));
        }
    }
    headers_invariants(
        &refusal.headers,
        refusal.status,
        &format!("refusals.{key}"),
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
/// command, not all defaults, and admitted for the route's method, with `ignore` beside `form` or
/// `query` only.
fn request_invariants(route: &Route, request: Request, at: &str) -> Result<(), String> {
    if let Operation::View(_) = route.operation {
        return Err(format!(
            "`{at}.request` is on {}; a view takes no request",
            route.operation
        ));
    }
    if request == Request::default() {
        return Err(format!(
            "`{at}.request` holds only defaults; compiled IR omits a request that is a `json` body \
             refusing an unknown key"
        ));
    }
    let admitted = match request.encoding {
        Encoding::Query => matches!(route.method, Method::Get | Method::Delete),
        Encoding::Json | Encoding::Form => route.method != Method::Get,
    };
    if !admitted {
        return Err(format!(
            "`{at}.request.encoding` `{}` is not admitted on `{}`: `query` is the only encoding \
             of a command on `get`, and an encoding of one on `delete`, and of no other",
            request.encoding, route.method
        ));
    }
    if request.encoding == Encoding::Json && request.unknown == Unknown::Ignore {
        return Err(format!(
            "`{at}.request.unknown` is `ignore` on a `json` body, which stays closed"
        ));
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
/// | `ESS-HTTP-010` | two operations of one component on one method and template, variable names ignored, declared or derived |
/// | `ESS-HTTP-011` | two templates of one component and method that a request path can both match, with a literal beside a variable at the first position they differ: a literal takes no precedence |
/// | `ESS-HTTP-013` | an operation with no HTTP surface: a command no component accepts, a view no `reached_by: network` component serves |
/// | `ESS-HTTP-014` | a command on `get` with an input field no variable of the path maps, unless its request is encoded `query` |
/// | `ESS-HTTP-020` | at `routes[i].request.encoding`: an encoding outside `json`, `form` and `query`, `json` or `form` written on a `get` command, or `query` on a method other than `get` and `delete`; at `routes[i].request.unknown`: a value outside `refuse` and `ignore`, or `ignore` with a `json` body; at `routes[i].request`: `request` on a view |
/// | `ESS-HTTP-021` | at `routes[i].request.encoding`: a `form` or `query` request carrying an input field whose type has no flat spelling, naming the field and its type. A flat field is a scalar (a primitive other than `Json`, an enum, or a newtype of a scalar), an `Optional` scalar or a `List` of scalars; a field mapped to the path is not carried |
/// | `ESS-HTTP-022` | at `routes[i].outcomes.<name>`: a key that is not the local name of an outcome of the command, or one listed twice; at `routes[i].outcomes`: `outcomes` on a view; at `routes[i].errors`: `errors` on a view, or on a command none of whose outcomes reports an error without being listed under `outcomes`; at `routes[i].alike`: `alike` on a view, a group of fewer than two names, a name that is not the local name of an outcome of the command, one in two groups or twice in one, or one with no declared answer (neither listed under `outcomes` nor covered by `errors`) |
/// | `ESS-HTTP-023` | at `routes[i].outcomes.<name>.status`, or `routes[i].errors.status` once for each outcome `errors` covers: a status outside `100..=599`, a `1xx`, a `3xx` other than `301`, `302`, `303`, `307` and `308`, a `2xx` on an outcome with an error, or a `4xx` or `5xx` on one without |
/// | `ESS-HTTP-024` | at `routes[i].outcomes.<name>.body`, or `routes[i].errors.body` once for each outcome `errors` covers: a word no body form spells, `response` or `response.<field>` on an outcome without `returns: true` or with an error, `response.<field>` naming no field of the response or an `Optional` one, `input.<field>`, `error.<field>` or `error.wire` as a body, `{<member>: error}` on one without an error, under an empty `<member>` or one equal to the wire name of a field of the error, a literal body, `rows` or `single`, or any body but `none` on `204` when that status compiled, or beside a `redirect` on a redirect status; at `routes[i].body`: `body` on a command, a body other than `rows` and `single` on a view, or `single` on a paged view; at `routes[i].outcomes.<name>.encoding`, or `routes[i].errors.encoding` once for each outcome `errors` covers: a word other than `json` and `text`, or `text` beside any body but `response.<field>` or on a field that is not a `String` (a newtype of one counts) |
/// | `ESS-HTTP-025` | at `routes[i].outcomes.<name>`, or `routes[i].errors` for an outcome `errors` covers: two outcomes of the command on one status whose bodies do not tell them apart (both `envelope`, or both `{<member>: error}` on one member with distinct error wire names, do) and whose redirects do not either (both declare one and one always writes a parameter, its source not `Optional`, the other never writes, do), an outcome `errors` covers counted at the status and body of `errors`, any other the route does not list at its derived status ([`status::outcome_status`]) with `envelope`. Reported once, at the first listed outcome in document order, then the first covered one in declaration order, naming the others and the status; a pair already named together is not reported again, and an outcome left with none is not reported; at `refusals.<key>`: an override on a route the binding names, on the status of an outcome of its command, whose body does not tell the two apart (only `{<member>: {literal: <text>}}` beside `{<member>: error}` on one member, the text differing from the error's wire name, does), once per route, naming the route, the outcomes and the status; two outcomes in one `alike` group are not a pair; at `routes[i].alike`: a group whose answers differ (the first differing key among `status`, `body`, `encoding`, `headers` and `redirect` named), or that the wire tells apart: `envelope` bodies, `{<member>: error}` bodies under distinct error wire names, or redirects one of which always writes a parameter the other never does |
/// | `ESS-HTTP-026` | at `routes[i].outcomes.<name>.headers.<Name>`, `routes[i].errors.headers.<Name>` (once for each outcome `errors` covers) or `refusals.<key>.headers.<Name>`: a name that is not an RFC 9110 token; `Content-Type`, `Content-Length`, `Transfer-Encoding` or `Location` in any case, which the encoding and `redirect` write; a name declared again ignoring case; a literal that is not a field value (visible US-ASCII, spaces and tabs, no space or tab at either end); a source outside `input.<field>`, `response.<field>`, `error.<field>` and `error.wire`, naming no field, naming one of a type other than a scalar or an `Optional` scalar, or one the outcome cannot supply (`response.` without `returns: true`, `error.` without an error), or `error.wire` where the error declares a field named `wire`; any source in a standard refusal's header. At `….challenge.params.<param>`: the same source refusals for a sourced challenge param |
/// | `ESS-HTTP-027` | at `….headers.<Name>`: a challenge under any header but `WWW-Authenticate`; at `….challenge.scheme`: a scheme that is not a token; at `….challenge.params.<param>`: a param name that is not a token, one declared again ignoring case, or a literal a quoted string cannot carry; at `….headers`: an outcome or standard refusal on `401` without a `WWW-Authenticate` challenge (one refused for its scheme or params counts as declared) |
/// | `ESS-HTTP-028` | at `routes[i].outcomes.<name>.redirect` or `routes[i].errors.redirect` (once for each outcome `errors` covers): a `redirect` on a compiled status other than `301`, `302`, `303`, `307` and `308`, or one of those without a `redirect`; at `….redirect.location`: a source the outcome cannot supply (as for `ESS-HTTP-026`), or of a type other than `String` or a newtype of `String`, `Optional` included; at `….redirect.mode`: none, or one outside `query` and `fragment`; at `….redirect.parameters.<name>`: a parameter written twice, or a source refused as a header source is |
/// | `ESS-HTTP-029` | at `refusals.<key>`: a key outside `malformed`, `not_granted` and `unfinished`, or one written twice; at `refusals.<key>.status`: a status outside its class, `4xx` for `malformed` and `not_granted`, `5xx` for `unfinished`; at `refusals.<key>.body`: a body other than `none` or `{<member>: {literal: <text>}}` under a non-empty member |
/// | `ESS-HTTP-030` | at `routes[i]`: `body: single` without `empty`, `empty` without `single` (on a command too), or an `empty` status outside `4xx`, unless the route answers `rows`, where `empty` is refused once, for itself |
/// | `ESS-HTTP-031` | at `routes[i].body`: `body: single` on a view whose filter does not select by its source entity's identity: neither the filter nor a member of a top-level `all` (nested `all` included) is `<identity field> == param.<name>`, compared by value, in either operand order; naming the view, its entity and the identity field |
/// | `ESS-HTTP-032` | at `routes[i].operation_id`: an id outside `[A-Za-z_][A-Za-z0-9_.-]*`; one an earlier route already gives, naming both routes; or the qualified name of another command or view of the specification, which is that operation's derived `operationId` |
/// | `ESS-HTTP-033` | at `servers.<component>`: a component the specification does not declare, one not `reached_by: network`, one written twice, or an empty list; at `servers.<component>[j].url`: a URL that is neither absolute `http://` or `https://` with a host nor `/`-rooted, one holding whitespace, `{`, `}`, `?`, `#` or userinfo, or one given twice for the component |
///
/// A listed outcome, a `single` view and a standard refusal are written into the IR with every
/// default omitted: an outcome's `envelope`, a view's `rows`, an empty `outcomes`, `refusals`,
/// `headers`, challenge `params` or redirect `parameters`. Header, param and parameter maps are
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
/// | `ESS-HTTP-012` | at `routes[i].method` or `routes[i].path`: a method or path the refined binding fixed is changed, or a fixed path left open; at `routes[i].request.encoding` or `routes[i].request.unknown`: a refined route's request restated with another value, a default written out or left out counting as that default; at `routes[i].outcomes.<name>`, `routes[i].errors`, `routes[i].body` or `routes[i].empty`: a refined route's outcome, `errors` (its headers and redirect included), view body or empty answer added, dropped or restated with another status or body, by the same counting; at `routes[i].outcomes.<name>.headers` or `….redirect`: an outcome both list whose headers or redirect differ, compared in name order with an empty map counting as none; at `refusals.<key>`: a standard refusal added, dropped or changed; at `refusals.<key>.headers`: one both declare whose headers differ; at `routes`: a route of the refined binding is dropped; at `routes[i].operation_id`: an operation id the refined route fixes changed or left out (one may be added where the refined route has none); at `routes[i].outcomes.<name>`: an outcome both list restated with another `encoding`, `json` written out or left out counting as `json`; at `routes[i].alike`: the groups added, dropped or restated, each compared in name order |
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
    let compared = refinement(spec, refined, ir, derived, &mut refusals);
    let (routes, derived_routes) = body(spec, ir, derived, &standard, &mut refusals);
    let servers = presentation(spec, ir, &mut refusals);
    if let Some(refined) = compared {
        changes(spec, refined, &mut refusals);
        operation_id_changes(spec, refined, &mut refusals);
    }
    if refusals.0.is_empty() {
        Ok(HttpIr {
            format: HTTP_IR_FORMAT.to_owned(),
            specification: spec.specification.clone(),
            refines: spec.refines.clone(),
            refusals: standard,
            servers,
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
    standard(spec.refusals.as_ref(), refusals)
}

/// Holds each `refusals:` entry to its rules (`ESS-HTTP-029`): a key of the set, written once, a
/// status of the key's class and a body that is `none` or one literal member with a name.
fn standard(
    written: Option<&Entries<RefusalSpec>>,
    refusals: &mut Refusals,
) -> BTreeMap<RefusalKey, Refusal> {
    let mut compiled = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for (name, spec) in written.map_or(&[][..], |entries| &entries.0[..]) {
        let at = format!("refusals.{name}");
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
                     `not_granted` and `unfinished`"
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
                     `malformed` and `not_granted` refuse the request (`4xx`), `unfinished` is \
                     the server's (`5xx`)",
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
    body(&refined.spec, ir, derived, &standard, &mut own);
    presentation(&refined.spec, ir, &mut own);
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
    refusal_changes(spec, refined, refusals);
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
        let fixed_request = written_request(fixed);
        for ((key, new), (_, old)) in written_request(route).into_iter().zip(fixed_request) {
            if new != old {
                refusals.refuse(
                    "ESS-HTTP-012",
                    format!("routes[{index}].request.{key}"),
                    format!(
                        "{operation} has `request.{key}` `{new}` here, and the refined binding \
                         fixes `{old}` at `routes[{at}]`; a refinement restates the request of \
                         every route it restates"
                    ),
                );
            }
        }
        answer_changes(route, fixed, &operation, index, at, refusals);
    }
}

/// Refuses a refining document whose `refusals:` differs from the refined binding's
/// (`ESS-HTTP-012`), entry by entry, at `refusals.<key>`: an entry added, dropped, or restated with
/// another status or body. The first entry of each key is compared; a second is `ESS-HTTP-029`.
/// Bodies are compared as written values, never as text, so that one literal cannot spell another.
fn refusal_changes(spec: &HttpSpec, refined: &HttpSpec, refusals: &mut Refusals) {
    let entries = |document: &HttpSpec| -> BTreeMap<String, (i64, BodySpec)> {
        let mut entries = BTreeMap::new();
        for (key, refusal) in document.refusals.iter().flat_map(|entries| &entries.0) {
            entries
                .entry(key.clone())
                .or_insert_with(|| (refusal.status, refusal.body.clone()));
        }
        entries
    };
    let (new, old) = (entries(spec), entries(refined));
    let headers = |document: &HttpSpec| -> BTreeMap<String, Option<serde_json::Value>> {
        let mut headers = BTreeMap::new();
        for (key, refusal) in document.refusals.iter().flat_map(|entries| &entries.0) {
            headers.entry(key.clone()).or_insert_with(|| {
                compared_entries(refusal.headers.as_ref(), HeaderSpec::compared)
            });
        }
        headers
    };
    let (new_headers, old_headers) = (headers(spec), headers(refined));
    let keys: std::collections::BTreeSet<&String> = new.keys().chain(old.keys()).collect();
    for key in keys {
        let shown = |entry: Option<&(i64, BodySpec)>| {
            entry.map_or_else(
                || "nothing".to_owned(),
                |(status, body)| format!("`{status}` with `{body}`"),
            )
        };
        let (here, there) = (new.get(key), old.get(key));
        if here != there {
            refusals.refuse(
                "ESS-HTTP-012",
                format!("refusals.{key}"),
                format!(
                    "`refusals.{key}` answers {} here, and the refined binding fixes {}; a \
                     refinement restates the refined binding's `refusals:` exactly",
                    shown(here),
                    shown(there)
                ),
            );
        }
        // A refusal added or dropped is refused above, as a whole.
        let (Some(here), Some(there)) = (new_headers.get(key), old_headers.get(key)) else {
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
                format!("refusals.{key}.headers"),
                format!(
                    "`refusals.{key}` declares {} here, and the refined binding fixes {}; a \
                     refinement restates the refined binding's `refusals:` exactly",
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
    let shown_empty = |empty: Option<i64>| {
        empty.map_or_else(
            || "no empty answer".to_owned(),
            |status| format!("`{status}`"),
        )
    };
    let (new_empty, old_empty) = (
        route.empty.map(|empty| empty.status),
        fixed.empty.map(|empty| empty.status),
    );
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

/// A route's `request` keys as written, each absent one as its default, so that writing a default
/// out is not a change.
fn written_request(route: &RouteSpec) -> [(&'static str, &str); 2] {
    let request = route.request.as_ref();
    [
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
/// field it carries (`021`), the unmapped `get` input (`014`), the answers (each listed outcome's
/// name `022`, status `023`, body `024`, headers `026` and `027` and redirect `028` in document
/// order, then `025`; a view's `outcomes`
/// `022`, `body` `024`, `empty` `030` and identity `031`), and the operation routed twice (`005`).
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
        let operation = match (&spec.command, &spec.view) {
            (Some(name), None) => Some(Operation::Command(name.clone())),
            (None, Some(name)) => Some(Operation::View(name.clone())),
            (None, None) => {
                refusals.refuse(
                    "ESS-HTTP-004",
                    path.as_str(),
                    "a route names the command or the view it serves, with `command:` or `view:`",
                );
                None
            }
            (Some(command), Some(view)) => {
                refusals.refuse(
                    "ESS-HTTP-004",
                    path.as_str(),
                    format!(
                        "a route serves one operation, and this one names both command \
                         `{command}` and view `{view}`"
                    ),
                );
                None
            }
        };
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
        let answers = answers(spec, &operation, target.as_ref(), &path, standard, refusals);
        if let Some(first) = first_route.get(&operation) {
            refusals.refuse(
                "ESS-HTTP-005",
                path.as_str(),
                format!("{operation} is already routed by `routes[{first}]`; an operation has one route"),
            );
        } else {
            first_route.insert(operation.clone(), index);
        }
        let claim = match (method, &segments, &spec.path) {
            (Some(method), Some(Some(segments)), Some(path)) => {
                Some((method, path.clone(), segments.clone()))
            }
            _ => None,
        };
        let route = match (target, method, segments, sources, request, answers) {
            (Some(_), Some(method), Some(_), Some(sources), Ok(request), Ok(answers)) => {
                Some(Route {
                    operation: operation.clone(),
                    operation_id: spec.operation_id.clone(),
                    method,
                    path: spec.path.clone(),
                    parameters: sources,
                    request,
                    outcomes: answers.outcomes,
                    errors: answers.errors,
                    alike: answers.alike,
                    body: answers.body,
                    empty: answers.empty,
                })
            }
            _ => None,
        };
        routes.push(Bound {
            index,
            operation,
            claim,
            route,
        });
    }
    routes
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
                     always present and holds one `String`, `Integer`, enum, or unconstrained \
                     newtype of one of them",
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
    let flat = match (target, checked.encoding) {
        (Some(target), Some(encoding @ (Encoding::Form | Encoding::Query))) => {
            flat_input(target, encoding, mapped, path, refusals)
        }
        _ => true,
    };
    let unmapped = match (target, method, checked.encoding) {
        (Some(target), Some(Method::Get), Some(Encoding::Json)) if target.is_command() => {
            get_input(operation, target, mapped, path, refusals)
        }
        _ => false,
    };
    if checked.refused || !flat || unmapped {
        Err(Refused)
    } else {
        Ok(checked.request)
    }
}

/// What a route's `request` resolved to.
struct Checked {
    /// Whether it earned a refusal (`ESS-HTTP-020`).
    refused: bool,
    /// The compiled request when it earned none; `None` when it is all defaults.
    request: Option<Request>,
    /// The encoding, when it is one of the set and admitted for the route's method; `None` when it
    /// was refused, which leaves the input it would carry unchecked.
    encoding: Option<Encoding>,
}

/// Holds a route's `request` to its rules (`ESS-HTTP-020`): no `request` on a view, an encoding of
/// the set that the method admits when one is written (`query` on `get` or `delete` only, and on
/// `get` nothing else written), and an `unknown` of the set, `ignore` only beside `form` or `query`. An absent
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
        };
    };
    if let Operation::View(_) = operation {
        refusals.refuse(
            "ESS-HTTP-020",
            format!("{path}.request"),
            format!(
                "{operation} takes no `request`: a view reads its parameters from the path and the \
                 query string"
            ),
        );
        return Checked {
            refused: true,
            request: None,
            encoding: None,
        };
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
                        "`{text}` is not an encoding; a request is encoded `json`, `form` or \
                         `query`"
                    ),
                );
                refused = true;
            }
            parsed
        }
    };
    let encoding = parsed.filter(|&encoding| {
        let rule = match (method, encoding) {
            (Some(Method::Get), Encoding::Json | Encoding::Form) if written.encoding.is_some() => {
                format!(
                    "`{encoding}` carries the input in a body, and `get` carries none; a command \
                     on `get` is encoded `query`, or leaves `encoding` out and maps every input \
                     field to the path"
                )
            }
            (Some(method), Encoding::Query) if !matches!(method, Method::Get | Method::Delete) => {
                format!(
                    "`query` carries the input in the query string, the encoding of a command on \
                     `get` or `delete` only; a command on `{method}` is encoded `json` or `form`"
                )
            }
            _ => return true,
        };
        refusals.refuse("ESS-HTTP-020", format!("{path}.request.encoding"), rule);
        refused = true;
        false
    });
    let unknown = match written.unknown.as_deref() {
        None => Some(Unknown::default()),
        Some(text) => {
            let unknown = Unknown::parse(text);
            if unknown.is_none() {
                refusals.refuse(
                    "ESS-HTTP-020",
                    format!("{path}.request.unknown"),
                    format!(
                        "`{text}` is not what a request does with an unknown parameter; it is \
                         `refuse` or `ignore`"
                    ),
                );
                refused = true;
            }
            unknown
        }
    };
    if parsed == Some(Encoding::Json) && unknown == Some(Unknown::Ignore) {
        refusals.refuse(
            "ESS-HTTP-020",
            format!("{path}.request.unknown"),
            "`ignore` admits a request parameter no field names, which only a `form` or `query` \
             request has; a `json` body stays closed, so `unknown` is `refuse` beside `json`",
        );
        refused = true;
    }
    let request = match (refused, encoding, unknown) {
        (false, Some(encoding), Some(unknown)) => {
            Some(Request { encoding, unknown }).filter(|request| *request != Request::default())
        }
        _ => None,
    };
    Checked {
        refused,
        request,
        encoding,
    }
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
                 request: a `form` or `query` field is a scalar, an `Optional` scalar or a `List` \
                 of scalars, a scalar being a primitive other than `Json`, an enum, or a newtype \
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
    body: ViewBody,
    empty: Option<Empty>,
}

/// The route's answers once every key of them is checked: a command's `outcomes` (`ESS-HTTP-022`
/// to `-025`) and the view keys it may not carry (`ESS-HTTP-024`, `-030`), or a view's
/// `outcomes`, `body` and `empty` (`ESS-HTTP-022`, `-024`, `-030`, `-031`).
fn answers(
    spec: &RouteSpec,
    operation: &Operation,
    target: Option<&Target<'_>>,
    path: &str,
    standard: &BTreeMap<RefusalKey, Refusal>,
    refusals: &mut Refusals,
) -> Result<Answers, Refused> {
    let before = refusals.0.len();
    let answers = match operation {
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
                body: ViewBody::Rows,
                empty: None,
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
    standard: &BTreeMap<RefusalKey, Refusal>,
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
/// whose `outcome` member names the outcome, never `{<member>: error}` under two error wire names,
/// and never redirects whose parameters tell them apart. A member whose declared answer was
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

/// Why the declared answers of one group of outcomes are not one answer the wire cannot tell
/// apart, if they are not: the first key in which an answer differs from the first member's
/// (`status`, `body`, `encoding`, `headers`, `redirect`), `envelope`, error objects under two error
/// wire names, or redirects whose parameters tell two members apart.
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
            if matches!(answer.body, Body::Error { .. }) && one_wire != two_wire {
                return Some(format!(
                    "`{}` and `{}` answer `{}` with `{}`, whose member holds the error wire name, \
                     `{}` for one and `{}` for the other, so the wire tells them apart",
                    one.name,
                    two.name,
                    answer.status,
                    answer.body,
                    one_wire.unwrap_or_default(),
                    two_wire.unwrap_or_default()
                ));
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
    let forms = "an outcome answers `envelope`, `response`, `response.<field>`, `none` or \
                 `{<member>: error}`";
    let resolved = match written {
        None => Ok(Body::Envelope),
        Some(BodySpec::Word(word)) => match word.as_str() {
            "envelope" => Ok(Body::Envelope),
            "none" => Ok(Body::None),
            "response" => answers_none(word).map_or(Ok(Body::Response), Err),
            "rows" | "single" => Err(format!("`{word}` is a view's body; {forms}")),
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
        },
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
    let checked = resolved.and_then(|body| {
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
    });
    match checked {
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

/// An outcome as `ESS-HTTP-025` counts it: its name, status, body, error wire name, and the
/// parameter names its redirect writes, if it declares one.
type Counted<'a> = (&'a str, u16, Body, Option<&'a str>, Option<Written<'a>>);

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
/// with distinct error wire names. Two redirects on one status, both `body: none`, are told apart
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
    standard: &BTreeMap<RefusalKey, Refusal>,
    refusals: &mut Refusals,
) {
    let counted = counted(ir, command, declared);
    let apart = |left: &Counted<'_>, right: &Counted<'_>| {
        left.1 != right.1
            || match (&left.2, &right.2) {
                (Body::Envelope, Body::Envelope) => true,
                (Body::Error { member: one }, Body::Error { member: other }) => {
                    one == other && left.3 != right.3
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
        let clashing: Vec<String> = others
            .into_iter()
            .map(|other| format!("`{}` ({})", other.0, declared.how(other)))
            .collect();
        if clashing.is_empty() {
            continue;
        }
        let under = if at == errors_at {
            ", under `errors`,"
        } else {
            ""
        };
        refusals.refuse(
            "ESS-HTTP-025",
            at,
            format!(
                "outcome `{name}` of {operation}{under} answers `{}` with `{}`, and so does {}, \
                 with a body a reader cannot tell apart; two outcomes on one status are told \
                 apart by two `envelope` bodies, by `{{<member>: error}}` on one member with \
                 distinct error wire names, or by two redirects, one always writing a parameter \
                 the other never writes or both writing one parameter from `error.wire` with \
                 distinct error wire names",
                this.1,
                this.2,
                clashing.join(", ")
            ),
        );
    }
    overridden(operation, declared, &counted, standard, path, refusals);
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
            counted.push((name, answer.status, answer.body.clone(), wire, redirect));
        } else if !listed_names.contains(name) && !declared.covered.contains(&name) {
            let derived = status::outcome_status(ir, outcome)
                .parse::<u16>()
                .unwrap_or_else(|error| panic!("a derived status is a number: {error}"));
            counted.push((name, derived, Body::Envelope, wire, None));
        }
    }
    counted
}

/// A standard refusal the binding overrides answers on this route too (`ESS-HTTP-025`): on the
/// status of a counted outcome, its body must tell the two apart, which only a literal beside an
/// error object on one member, the error's wire name differing from the text, does. Refused at
/// `refusals.<key>`, naming the route and every outcome on its status.
fn overridden(
    operation: &Operation,
    declared: &Declared<'_>,
    counted: &[Counted<'_>],
    standard: &BTreeMap<RefusalKey, Refusal>,
    path: &str,
    refusals: &mut Refusals,
) {
    for (key, refusal) in standard {
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
            format!("refusals.{key}"),
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

/// A view route's body and empty answer, checked: no `outcomes` (`ESS-HTTP-022`), `rows` or
/// `single`, never `single` on a paged view (`ESS-HTTP-024`), `single` exactly with an `empty`
/// holding a `4xx` (`ESS-HTTP-030`), and `single` only where the filter selects by identity
/// (`ESS-HTTP-031`).
fn view_answers(
    spec: &RouteSpec,
    operation: &Operation,
    view: Option<(&EssIr, &ResolvedView)>,
    path: &str,
    refusals: &mut Refusals,
) -> Answers {
    no_outcomes_on_a_view(spec, operation, path, refusals);
    // `None` when the body is refused, which leaves `empty` unchecked against it.
    let body = match &spec.body {
        None => Some(ViewBody::Rows),
        Some(written) => match written.word() {
            Some("rows") => Some(ViewBody::Rows),
            Some("single") => Some(ViewBody::Single),
            _ => {
                refusals.refuse(
                    "ESS-HTTP-024",
                    format!("{path}.body"),
                    format!(
                        "{operation} answers `rows` or `single`, and `{written}` is not a view's \
                         body"
                    ),
                );
                None
            }
        },
    };
    if body == Some(ViewBody::Single) {
        if let Some((_, view)) = view.filter(|(_, view)| view.paging.is_some()) {
            refusals.refuse(
                "ESS-HTTP-024",
                format!("{path}.body"),
                format!(
                    "view `{}` is paged: its answer carries a page, which one bare row cannot",
                    view.name
                ),
            );
        }
    }
    match (body, &spec.empty) {
        (Some(ViewBody::Single), None) => refusals.refuse(
            "ESS-HTTP-030",
            path,
            "`body: single` answers one row, and needs `empty` (`{status: <4xx>}`), the answer \
             for a view that holds none",
        ),
        (Some(ViewBody::Rows), Some(_)) => refusals.refuse(
            "ESS-HTTP-030",
            path,
            "`empty` answers a `single` view that holds no row, and this route answers `rows`",
        ),
        _ => {}
    }
    // An `empty` beside `rows` is refused above as a whole, so its status is not refused again.
    let empty = spec.empty.and_then(|empty| {
        let status = u16::try_from(empty.status)
            .ok()
            .filter(|status| (400..500).contains(status));
        if status.is_none() && body != Some(ViewBody::Rows) {
            refusals.refuse(
                "ESS-HTTP-030",
                path,
                format!(
                    "`empty` answers `{}`; the answer of a `single` view that holds no row is a \
                     `4xx` status, without a body",
                    empty.status
                ),
            );
        }
        status.map(|status| Empty { status })
    });
    if body == Some(ViewBody::Single) {
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

/// Whether a value of `type_ref` fits one path segment: always present and one value, so `String`,
/// `Integer`, an enum, or an unconstrained newtype of one of them, and never `Optional`, `List`,
/// `Map`, a struct, a union or another primitive.
fn segment_type(type_ref: &ResolvedTypeRef, ir: &EssIr) -> bool {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => {
            matches!(name, Primitive::String | Primitive::Integer)
        }
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
    /// Different templates one request path matches both of, a literal beside a variable at the
    /// zero-based position they first differ (`ESS-HTTP-011`).
    Overlap(usize),
}

/// Whether two templates meet. They can only when they have as many segments and no position
/// holds two different literals; then they are the same template, or they overlap at the first
/// position one holds a literal and the other a variable, after a prefix equal but for variable
/// names.
fn clash(left: &[Segment], right: &[Segment]) -> Option<Clash> {
    if left.len() != right.len() {
        return None;
    }
    let mut first_mixed = None;
    for (position, pair) in left.iter().zip(right).enumerate() {
        match pair {
            (Segment::Literal(left), Segment::Literal(right)) if left != right => return None,
            (Segment::Literal(_), Segment::Variable(_))
            | (Segment::Variable(_), Segment::Literal(_)) => {
                first_mixed.get_or_insert(position);
            }
            _ => {}
        }
    }
    Some(first_mixed.map_or(Clash::Same, Clash::Overlap))
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
/// on one method and template (`ESS-HTTP-010`) and for templates that overlap (`ESS-HTTP-011`),
/// and returns the derived routes that stay: those of every operation the binding does not name,
/// in IR order.
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
                match clash(&this.segments, &other.segments) {
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
                    Some(Clash::Overlap(position)) => refusals.refuse(
                        "ESS-HTTP-011",
                        format!("routes[{at}]"),
                        format!(
                            "{} and {} overlap on component `{component}`: after an equal prefix, \
                             segment {} is `{}` in one and `{}` in the other, so one request path \
                             matches both, and a literal takes no precedence over a variable",
                            this.describe(),
                            other.describe(),
                            position + 1,
                            this.segments[position],
                            other.segments[position]
                        ),
                    ),
                    None => {}
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
    use super::{clash, segment, template, Clash, Segment, Source};

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
        assert_eq!(overlap, Some(Clash::Overlap(1)));
        // The first literal beside a variable is the one named, later ones are not.
        let later = clash(
            &[variable("a"), literal("b"), variable("c")],
            &[variable("x"), variable("y"), literal("z")],
        );
        assert_eq!(later, Some(Clash::Overlap(1)));
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
