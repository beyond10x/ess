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

use ess_compiler::ir::{ResolvedBody, ResolvedField, ResolvedTypeRef};
use ess_compiler::EssIr;
use ess_domain::component::Reach;
use ess_domain::types::Primitive;

/// The adopter-authored HTTP binding format.
pub const HTTP_FORMAT: &str = "ess-http/1";

/// The compiled HTTP binding format.
pub const HTTP_IR_FORMAT: &str = "ess-http-ir/1";

/// Every code [`compile`] can refuse with, in code order.
pub const CODES: &[&str] = &[
    "ESS-HTTP-001",
    "ESS-HTTP-002",
    "ESS-HTTP-004",
    "ESS-HTTP-005",
    "ESS-HTTP-006",
    "ESS-HTTP-007",
    "ESS-HTTP-008",
    "ESS-HTTP-009",
    "ESS-HTTP-010",
    "ESS-HTTP-011",
    "ESS-HTTP-013",
    "ESS-HTTP-014",
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

/// One route, before resolution.
///
/// Every key is closed: a key of a later stage of the format (`request`, `outcomes`, `headers`,
/// `redirect` and the rest) is refused by name as an unknown field, never read and dropped.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteSpec {
    /// The qualified name of the command it serves; exactly one of this and `view`.
    #[serde(default)]
    pub command: Option<String>,
    /// The qualified name of the view it serves; exactly one of this and `command`.
    #[serde(default)]
    pub view: Option<String>,
    /// The method, as written. Read as text so that one outside the closed set is refused with a
    /// code and a path rather than as a parse error.
    pub method: String,
    /// The path template. Absent, the route is open: each server chooses its path.
    #[serde(default)]
    pub path: Option<String>,
    /// Path template variable to the source it reads: `input.<field>` or `param.<name>`.
    #[serde(default)]
    pub parameters: Parameters,
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
    /// One route per bound command or view, in document order.
    pub routes: Vec<RouteSpec>,
}

impl HttpSpec {
    /// Reads strict YAML. Semantic validation happens in [`compile`].
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
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
}

impl Route {
    /// The path's segments; `None` for an open route.
    ///
    /// A compiled path always parses: [`compile`] refuses one that does not, and
    /// [`HttpIr::from_json`] refuses compiled IR holding one.
    pub fn segments(&self) -> Option<Vec<Segment>> {
        self.path.as_deref().and_then(|path| template(path).ok())
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

    /// Reads compiled IR back, refusing any format but [`HTTP_IR_FORMAT`], any unknown key, a path
    /// that breaks the template grammar, and `routes`, `derived_routes` or a route's `parameters`
    /// out of canonical order, naming the key: what reads back writes back the bytes it was given.
    ///
    /// The routes are taken as compiled; a reader that needs them checked against a specification
    /// compiles the authored binding instead.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let ir: Self = serde_json::from_str(text)?;
        if ir.format != HTTP_IR_FORMAT {
            return Err(serde::de::Error::custom(format!(
                "expected `{HTTP_IR_FORMAT}`, found `{}`",
                ir.format
            )));
        }
        for path in ir.routes.iter().filter_map(|route| route.path.as_deref()) {
            if let Err(reasons) = template(path) {
                return Err(serde::de::Error::custom(reasons.join("; ")));
            }
        }
        let route_key = |route: &Route| (route.path.clone(), route.method, route.operation.clone());
        if let Some(at) = out_of_order(&ir.routes, |left, right| {
            route_key(left).cmp(&route_key(right))
        }) {
            return Err(serde::de::Error::custom(format!(
                "`routes[{at}]` is out of canonical order: compiled IR orders `routes` by path, \
                 then method, then operation, each once"
            )));
        }
        if let Some(at) = out_of_order(&ir.derived_routes, derived_order) {
            return Err(serde::de::Error::custom(format!(
                "`derived_routes[{at}]` is out of canonical order: compiled IR orders \
                 `derived_routes` by component, then path, then method, then operation, each once"
            )));
        }
        Ok(ir)
    }
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
/// Every stage-1 check of `docs/design/http-presentation-binding.md` this build implements runs,
/// and every refusal is returned at once rather than the first:
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
/// | `ESS-HTTP-014` | a command on `get` with an input field no variable of the path maps |
///
/// A route without a path is open, and valid: the artifacts that need a path refuse it. It takes
/// no part in collision checks, because no path is claimed.
pub fn compile(
    spec: &HttpSpec,
    ir: &EssIr,
    derived: &[DerivedRoute],
) -> Result<HttpIr, HttpDiagnostics> {
    let mut refusals = Refusals(Vec::new());
    if spec.format != HTTP_FORMAT {
        refusals.refuse(
            "ESS-HTTP-001",
            "type",
            format!("expected `{HTTP_FORMAT}`, found `{}`", spec.format),
        );
    }
    specification(&spec.specification, ir, &mut refusals);
    let resolved = routes(&spec.routes, ir, &mut refusals);
    let derived_routes = surface(&resolved, derived, &mut refusals);
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
    if refusals.0.is_empty() {
        Ok(HttpIr {
            format: HTTP_IR_FORMAT.to_owned(),
            specification: spec.specification.clone(),
            routes,
            derived_routes,
        })
    } else {
        Err(HttpDiagnostics(refusals.0))
    }
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
    /// The resolved route, when its operation, method, path, sources and `get` input resolved; it
    /// reaches the IR only when the binding earned no refusal at all.
    route: Option<Route>,
}

/// The fields a route's sources are read from: a command's input or a view's parameters.
struct Target<'a> {
    ir: &'a EssIr,
    fields: &'a [ResolvedField],
    command: bool,
}

/// Every route that names an operation, in document order. A route that earned a refusal is still
/// checked in full, so its other refusals are reported too, and it still takes part in the
/// collision checks whenever its method is admitted and its path parsed.
///
/// Within one route the refusals come in this order: the operation's shape (`004`), the method
/// (`006`), the path's grammar (`007`), the operation's existence and surface (`004`, `013`), the
/// mapping of variables (`008`), the sources (`009`), the unmapped `get` input (`014`), and the
/// operation routed twice (`005`).
fn routes(specs: &[RouteSpec], ir: &EssIr, refusals: &mut Refusals) -> Vec<Bound> {
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
        let unmapped = match (&target, method) {
            (Some(target), Some(Method::Get)) if target.command => {
                get_input(&operation, target, &mapped, &path, refusals)
            }
            _ => false,
        };
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
        let route = match (target, method, segments, sources, unmapped) {
            (Some(_), Some(method), Some(_), Some(sources), false) => Some(Route {
                operation: operation.clone(),
                method,
                path: spec.path.clone(),
                parameters: sources,
            }),
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
                 input field is mapped to the path"
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
        command: true,
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
        command: false,
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
    let (namespace, kind, what) = if target.command {
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

/// Refuses each input field of a command on `get` that no variable of the path maps
/// (`ESS-HTTP-014`), and says whether any was.
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
                     `{}` is not mapped to the path",
                    operation.name(),
                    field.name
                ),
            );
            unmapped = true;
        }
    }
    unmapped
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
