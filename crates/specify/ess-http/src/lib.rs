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

use ess_compiler::EssIr;
use ess_domain::component::Reach;

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
    "ESS-HTTP-013",
    "ESS-HTTP-014",
];

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
    pub parameters: BTreeMap<String, String>,
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
    /// Path template variable to the source it reads, ordered by variable. Absent when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}

/// A compiled `ess-http-ir/1`: every route resolved, ordered by path, then method, then operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpIr {
    format: String,
    specification: SpecificationIdentity,
    routes: Vec<Route>,
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

    /// The canonical JSON bytes, with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("HTTP IR serializes: {error}"));
        json.push('\n');
        json
    }

    /// Reads compiled IR back, refusing any format but [`HTTP_IR_FORMAT`] and any unknown key.
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
        Ok(ir)
    }
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

/// Resolves an HTTP binding against the specification it names.
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
/// | `ESS-HTTP-013` | an operation with no HTTP surface: a command no component accepts, a view no `reached_by: network` component serves |
/// | `ESS-HTTP-014` | a command on `get` with an input field the path does not map |
///
/// A route without a path is open, and valid: the artifacts that need a path refuse it.
pub fn compile(spec: &HttpSpec, ir: &EssIr) -> Result<HttpIr, HttpDiagnostics> {
    let mut refusals = Refusals(Vec::new());
    if spec.format != HTTP_FORMAT {
        refusals.refuse(
            "ESS-HTTP-001",
            "type",
            format!("expected `{HTTP_FORMAT}`, found `{}`", spec.format),
        );
    }
    specification(&spec.specification, ir, &mut refusals);
    let mut routes = routes(&spec.routes, ir, &mut refusals);
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

/// Every route that resolved, in document order. A route that earned a refusal is still checked
/// in full, so its other refusals are reported too.
fn routes(specs: &[RouteSpec], ir: &EssIr, refusals: &mut Refusals) -> Vec<Route> {
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
        let Some(operation) = operation else {
            continue;
        };
        let resolved = match &operation {
            Operation::Command(name) => command(spec, name, method, ir, &path, refusals),
            Operation::View(name) => view(name, ir, &path, refusals),
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
        if let (true, Some(method)) = (resolved, method) {
            routes.push(Route {
                operation,
                method,
                path: spec.path.clone(),
                parameters: spec.parameters.clone(),
            });
        }
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

/// Whether the command exists and has an HTTP surface; a `get` route also maps every input field.
fn command(
    spec: &RouteSpec,
    name: &str,
    method: Option<Method>,
    ir: &EssIr,
    path: &str,
    refusals: &mut Refusals,
) -> bool {
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
        return false;
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
        return false;
    }
    if method != Some(Method::Get) {
        return true;
    }
    let mut mapped = true;
    for field in &command.input {
        let source = format!("input.{}", field.name);
        if !spec.parameters.values().any(|value| *value == source) {
            refusals.refuse(
                "ESS-HTTP-014",
                path,
                format!(
                    "command `{name}` is bound to `get`, which carries no body, and its input \
                     field `{}` is not mapped to the path",
                    field.name
                ),
            );
            mapped = false;
        }
    }
    mapped
}

/// Whether the view exists and a `reached_by: network` component serves it.
fn view(name: &str, ir: &EssIr, path: &str, refusals: &mut Refusals) -> bool {
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
        return false;
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
        return false;
    }
    true
}
