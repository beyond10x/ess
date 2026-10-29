//! The HTTP surface one component's specification determines: one route table, read by everything.
//!
//! # Why this is a module and not a paragraph in `openapi.rs`
//!
//! Two artifacts have to agree about what `POST /invoices/commands/create-invoice` means: the
//! `OpenAPI` document that publishes the contract, and the synthesised server that answers it. When
//! each computed its own paths they would agree on the day they were written and drift the first
//! time a wire name moved — and the drift is invisible, because a server serving a path no document
//! declares looks exactly like a server that works. So the mapping lives here, once, and both read
//! it. It is the same argument `src/types.rs` settles for schemas, and this crate has already paid
//! for learning it: `openapi.rs` and `asyncapi.rs` each carried a copy of the type mapping, both
//! drifted, and the drift was published as two contradictory contracts for one event.
//!
//! # A component is only served when the specification says it is
//!
//! [`routes`] answers for every component, because every component has an `OpenAPI` document. What
//! changes with [`Reach::Network`] is the **view** half: a
//! view has no path at all until the specification says something outside the process reads it.
//! Until then, exposing one would invent a query surface — which is the row `openapi.rs`'s "what
//! this refuses to guess" table has always carried, and this is that row being closed by a
//! declaration rather than by a generator's opinion.
//!
//! # What is still not invented
//!
//! No pagination, no cursor, no ordering, no filter parameter: a view's filter is declared in the
//! model and its rows are what the projection holds. No `servers`, because the model still has no
//! URL; only an `ess-http/1` binding states them, per `reached_by: network` component. No path
//! version, because `info.version` is the only version the model has. The two segments a route
//! is built from — the domain's wire name and the construct's — are both declared, and the
//! `commands`/`views` segment between them is what stops a path from reading as a resource.

use ess_compiler::ir::{
    CommandHandle, EssIr, ResolvedComponent, ResolvedCondition, ResolvedOutcome, ViewHandle,
};
use ess_domain::component::Reach;

// The status of each declared outcome lives in `ess-http`, which checks a binding against it
// (`ESS-HTTP-025`); every item is re-exported here, so the document and the servers keep reading it
// from this module.
pub use ess_http::status::{
    answers_with_response, outcome_status, status, ANSWERED, CONFLICT, DIRECT_ANSWER_FORMAT,
    FORBIDDEN, NOT_FOUND, NO_INPUT, REFUSED, TAKEN, UPSTREAM,
};

/// Where the served component publishes the contract it answers.
///
/// A fixed path rather than a derived one: it names no construct of any specification, so there is
/// nothing to derive it from, and a served document a caller cannot find is a document that is not
/// published. JSON rather than the committed YAML because this is the machine's copy — the same
/// document, in the dialect every HTTP client already parses.
pub const OPENAPI: &str = "/openapi.json";

/// Where it publishes the prose the same model produced.
pub const DOCS: &str = "/docs";

/// The two paths a served surface answers about itself, which no bound route may match
/// ([`Binding::servable`]).
pub const RESERVED: [&str; 2] = [OPENAPI, DOCS];

/// The `Content-Type` a served answer writes for an outcome an `ess-http/1` binding answers
/// `encoding: text`: the string field's value alone, UTF-8.
pub const TEXT_CONTENT_TYPE: &str = "text/plain; charset=utf-8";

/// Whether a request for the concrete `path` would match a route whose template is `segments`: as
/// many segments, each literal equal and each variable taking whatever is there. The overlap rule
/// of `ESS-HTTP-011`, with one side a path rather than a template.
pub fn template_matches(segments: &[ess_http::Segment], path: &str) -> bool {
    let Some(rest) = path.strip_prefix('/') else {
        return false;
    };
    let given: Vec<&str> = if rest.is_empty() {
        Vec::new()
    } else {
        rest.split('/').collect()
    };
    given.len() == segments.len()
        && segments
            .iter()
            .zip(given)
            .all(|(segment, text)| match segment {
                ess_http::Segment::Literal(literal) => literal == text,
                ess_http::Segment::Variable(_) => true,
            })
}

/// The projection was read.
///
/// The one `2xx` here that is not a `202`: a `GET` on a view returns the rows, so the response
/// *is* the answer rather than a receipt for a branch that was taken.
pub const READ: &str = "200";

/// The `refused` member of the standard refusal for an actor no grant admits (beyond10x/ess#265).
///
/// One refusal for every command, answered with [`FORBIDDEN`] before the command runs: the caller
/// the realization authenticated the request as is none, or is an actor the specification does not
/// grant the command. Its body is `{"refused": "not granted", "actor": <the actor's qualified name,
/// or null>}`. The contract states which actors may invoke a command (`x-ess-may-invoke`) and never
/// how a request proves it is one of them: the served surface is handed the authenticated caller,
/// and derives nothing about it from the request.
pub const NOT_GRANTED: &str = "not granted";

/// Whether any served surface checks grants: the model declares an actor and serves a component
/// (`reached_by: network`).
///
/// A model that declares no actor says nothing about who may invoke what, and one that serves
/// nothing has no surface to check a grant on — enforcement is then the caller's, against the
/// generated grant table — so neither checks anything and its contract keeps its bytes.
pub fn checks_grants(ir: &EssIr) -> bool {
    ir.components()
        .values()
        .any(|component| grants_checked_on(ir, component))
}

/// Whether `component`'s surface checks grants: the model declares an actor and the component is
/// served (`reached_by: network`).
pub fn grants_checked_on(ir: &EssIr, component: &ResolvedComponent) -> bool {
    !ir.actors().is_empty() && component.reached_by == Reach::Network
}

/// Whether any served surface checks a read grant (beyond10x/ess#286): it checks grants, and some
/// actor's `may:` names a view. A model naming no view keeps every view open and its bytes.
pub fn checks_read_grants(ir: &EssIr) -> bool {
    checks_grants(ir) && ir.grants_reads()
}

/// Whether reading `view` on a served surface checks its read grant (beyond10x/ess#286): the
/// surface checks grants and some actor's `may:` names the view. A view no actor names is open.
pub fn read_checked(ir: &EssIr, view: &ess_domain::name::QualifiedName) -> bool {
    checks_grants(ir) && ir.read_granted(view)
}

/// The command could not be carried through because the realization is unfinished: a port it runs
/// reported an unmet obligation, or the command's effect was committed and delivering what it
/// published to a binding failed.
///
/// Not a declared outcome, so no branch maps to it; every served command can answer it, so the
/// contract declares it on every command. A client must not retry it: after a failed delivery the
/// effect and its events stand, and a retry would perform the command twice.
pub const UNFINISHED: &str = "501";

/// Whether a request for `command` must carry a body.
///
/// The one answer for the document that declares the `requestBody` and the servers that read it.
/// A command with no input declares no body at all, and one whose every field is optional declares
/// a body that is not required, so a `POST` with an empty body is that command's input, `{}`. An
/// `input_absent:` branch (ess/16) declares the answer for a request with no body, so the body is
/// not required even where its fields are.
pub fn body_required(command: &ess_compiler::ir::ResolvedCommand) -> bool {
    body_required_without(command, &std::collections::BTreeSet::new())
}

/// [`body_required`], for a body that does not carry the input fields an `ess-http/1` route
/// `carried` in its path: whether some other input field is not optional.
pub fn body_required_without(
    command: &ess_compiler::ir::ResolvedCommand,
    carried: &std::collections::BTreeSet<&str>,
) -> bool {
    command
        .input
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .any(|field| !field.type_ref.is_optional())
        && !command
            .outcomes
            .iter()
            .any(|outcome| outcome.condition == ResolvedCondition::InputAbsent)
}

/// The caller attributes a command reads (ess/16, beyond10x/ess#168), each once, in name order:
/// every `{caller: <attribute>}` of its values and every `caller.<attribute>` of its guards.
///
/// What a request has to be authenticated as for the command to mean what the specification says,
/// which the input does not carry and a reader of the contract cannot otherwise see.
pub fn caller_attributes(ir: &EssIr, command: &ess_compiler::ir::ResolvedCommand) -> Vec<String> {
    use ess_compiler::ir::ResolvedPayloadValue;
    use ess_domain::command::caller_value::CALLER_NAMESPACE;

    fn values(value: &ResolvedPayloadValue, found: &mut std::collections::BTreeSet<String>) {
        match value {
            ResolvedPayloadValue::CallerAttribute { attribute, .. } => {
                found.insert(attribute.clone());
            }
            ResolvedPayloadValue::Struct { fields } => {
                for leaf in fields {
                    values(&leaf.value, found);
                }
            }
            _ => {}
        }
    }
    let mut found = std::collections::BTreeSet::new();
    let mut guard = |predicate: &ess_primitives::predicate::Predicate,
                     roots: &[ess_compiler::ir::ResolvedField]| {
        if roots.iter().any(|field| field.name == CALLER_NAMESPACE) {
            return;
        }
        for path in predicate.fact_paths() {
            if path.namespace() == CALLER_NAMESPACE && path.segments().len() == 2 {
                found.insert(path.segments()[1].clone());
            }
        }
    };
    for outcome in &command.outcomes {
        match &outcome.condition {
            ResolvedCondition::When { predicate }
            | ResolvedCondition::ExternalWhen { predicate, .. } => {
                guard(predicate, &command.input);
            }
            ResolvedCondition::SubjectField {
                predicate: Some(predicate),
                ..
            }
            | ResolvedCondition::SubjectState {
                predicate: Some(predicate),
                ..
            }
            | ResolvedCondition::StateChange {
                predicate: Some(predicate),
                ..
            } => guard(predicate, &command.input),
            ResolvedCondition::SubjectPredicate { predicate, input } => {
                if let Some(input) = input {
                    guard(input, &command.input);
                }
                let stored = command
                    .selection_subject(outcome)
                    .map(|subject| ir.entity(&subject.entity).fields.clone())
                    .unwrap_or_default();
                guard(predicate, &stored);
            }
            _ => {}
        }
    }
    let mut read = found;
    for outcome in &command.outcomes {
        for field in outcome
            .payload
            .iter()
            .flat_map(|payload| &payload.fields)
            .chain(&outcome.sets)
        {
            values(&field.value, &mut read);
        }
    }
    read.into_iter().collect()
}

/// The methods a route is served on.
///
/// Without a binding there are two. A command changes state, so it is a `POST`; a view is a
/// projection a caller reads, so it is a `GET`. Nothing derived is a resource, so no derived route
/// is a `PUT`, a `PATCH` or a `DELETE` — the model describes no addressable thing to replace or
/// remove, and inventing one is exactly what the command-endpoint convention exists to avoid.
///
/// An `ess-http/1` binding ([`Binding`]) is where a protocol says otherwise, and the other three are
/// the methods it may bind a command to. Declared in name order, the order `ess-http-ir/1` orders
/// routes on one path in; `Get` before `Post` either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    /// Removes what a bound command names; only from a binding.
    Delete,
    /// Reads a projection, or a bound command whose every input field is in its path.
    Get,
    /// Changes part of what a bound command names; only from a binding.
    Patch,
    /// Issues a command.
    Post,
    /// Replaces what a bound command names; only from a binding.
    Put,
}

impl Method {
    /// The verb as it appears on the wire and in the document.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "DELETE",
            Self::Get => "GET",
            Self::Patch => "PATCH",
            Self::Post => "POST",
            Self::Put => "PUT",
        }
    }

    /// The `OpenAPI` path-item key, which is the verb in lower case.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Get => "get",
            Self::Patch => "patch",
            Self::Post => "post",
            Self::Put => "put",
        }
    }

    /// The same method as an `ess-http/1` binding spells it.
    pub fn binding(self) -> ess_http::Method {
        match self {
            Self::Delete => ess_http::Method::Delete,
            Self::Get => ess_http::Method::Get,
            Self::Patch => ess_http::Method::Patch,
            Self::Post => ess_http::Method::Post,
            Self::Put => ess_http::Method::Put,
        }
    }

    /// The method a binding's `method` serves on, if a route may be served on it: `head`, `options`
    /// and `trace` are refused for every operation in `ess-http/1` (`ESS-HTTP-006`).
    pub fn of(method: ess_http::Method) -> Option<Self> {
        match method {
            ess_http::Method::Delete => Some(Self::Delete),
            ess_http::Method::Get => Some(Self::Get),
            ess_http::Method::Patch => Some(Self::Patch),
            ess_http::Method::Post => Some(Self::Post),
            ess_http::Method::Put => Some(Self::Put),
            ess_http::Method::Head | ess_http::Method::Options | ess_http::Method::Trace => None,
        }
    }
}

/// What one route serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Served<'a> {
    /// One command the component accepts.
    Command(&'a CommandHandle),
    /// One view a domain the component owns declares.
    View(&'a ViewHandle),
}

/// One route of a component's HTTP surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route<'a> {
    /// The verb.
    pub method: Method,
    /// The path, from the declared wire names.
    pub path: String,
    /// The construct it serves.
    pub serves: Served<'a>,
    /// The binding's route, where an `ess-http/1` binding declares this one: its method and path
    /// are then [`Route::method`] and [`Route::path`], and its `parameters` say which input field
    /// or view parameter each path variable carries. `None` for a derived route.
    pub declared: Option<&'a ess_http::Route>,
}

impl Served<'_> {
    /// The operation an `ess-http/1` binding names this construct by.
    pub fn operation(self) -> ess_http::Operation {
        match self {
            Self::Command(handle) => ess_http::Operation::Command(handle.name().to_string()),
            Self::View(handle) => ess_http::Operation::View(handle.name().to_string()),
        }
    }
}

/// An `ess-http/1` binding compiled against this IR ([`ess_http::compile`]), held to what a
/// projection that publishes routes needs: a path on every route, and a method a route is served
/// on (`docs/design/http-presentation-binding.md`, "What generated artifacts change (stage 1)").
///
/// [`bound_routes`] reads it; nothing else here does, so every artifact generated without one keeps
/// its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding(ess_http::HttpIr);

impl Binding {
    /// Holds `http` to a path on every route, refusing every route without one by its operation.
    ///
    /// An open route is valid `ess-http/1`: a protocol binding leaves a path to each server. It is
    /// not a route anything can be sent to, so a document publishing it would have to invent a
    /// path; the server binding that refines it fills the path in (decided for stage 1, answer 1).
    /// A method a route is never served on (`head`, `options`, `trace`) is refused the same way,
    /// though compiled IR cannot hold one (`ESS-HTTP-006`).
    pub fn new(http: ess_http::HttpIr) -> Result<Self, Unprojectable> {
        let refused: Vec<(ess_http::Operation, Reason)> = http
            .routes()
            .iter()
            .filter_map(|route| {
                if route.path.is_none() {
                    Some((route.operation.clone(), Reason::Open))
                } else if Method::of(route.method).is_none() {
                    Some((route.operation.clone(), Reason::Method(route.method)))
                } else {
                    None
                }
            })
            .collect();
        if refused.is_empty() {
            Ok(Self(http))
        } else {
            Err(Unprojectable(refused))
        }
    }

    /// The compiled binding.
    pub fn ir(&self) -> &ess_http::HttpIr {
        &self.0
    }

    /// Every route a served surface cannot answer, with its operation, in route order: one whose
    /// template a request for [`OPENAPI`] or [`DOCS`] would match. Each server answers those two
    /// paths about itself (`docs/design/http-presentation-binding.md`, "What generated artifacts
    /// change (stage 1)"), so a route claiming either is refused at synthesis rather than shadowed
    /// or shadowing. `ess specify http validate` accepts such a route: the document is valid, and
    /// only the server cannot honour it.
    ///
    /// # Errors
    ///
    /// [`Unprojectable`] naming each such route, by [`Reason::Reserved`].
    pub fn servable(&self) -> Result<(), Unprojectable> {
        let refused: Vec<(ess_http::Operation, Reason)> = self
            .0
            .routes()
            .iter()
            .filter_map(|route| {
                let segments = route.segments()?;
                RESERVED
                    .into_iter()
                    .find(|reserved| template_matches(&segments, reserved))
                    .map(|reserved| (route.operation.clone(), Reason::Reserved(reserved)))
            })
            .collect();
        if refused.is_empty() {
            Ok(())
        } else {
            Err(Unprojectable(refused))
        }
    }

    /// The standard refusal `key` the binding answers its own way, if it overrides it.
    pub fn refusal(&self, key: ess_http::RefusalKey) -> Option<&ess_http::Refusal> {
        self.0.refusals().get(&key)
    }

    /// The binding's route of `served`, if it declares one.
    pub fn route(&self, served: Served<'_>) -> Option<&ess_http::Route> {
        self.0.route(&served.operation())
    }
}

/// Why a binding's route cannot be projected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// It has no path.
    Open,
    /// It names a method no route is served on.
    Method(ess_http::Method),
    /// Its template matches this path, which a served surface answers about itself.
    Reserved(&'static str),
}

/// The routes of a binding [`Binding::new`] refuses, each with its operation, in route order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unprojectable(pub Vec<(ess_http::Operation, Reason)>);

impl std::fmt::Display for Unprojectable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (operation, reason)) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            match reason {
                Reason::Open => write!(
                    formatter,
                    "{operation} has an open route: the binding leaves its path to each server, \
                     and a published document needs one; fill it in with a binding that refines \
                     this one"
                )?,
                Reason::Method(method) => write!(
                    formatter,
                    "{operation} is bound to `{method}`, which no route is served on"
                )?,
                Reason::Reserved(path) => write!(
                    formatter,
                    "{operation} has a path a request for `{path}` would match, and a served \
                     surface answers `{path}` itself; route it elsewhere"
                )?,
            }
        }
        Ok(())
    }
}

impl std::error::Error for Unprojectable {}

/// Every route one component's surface has, with `binding`'s routes in place of the derived ones
/// of the operations it names, in path order, then method.
///
/// Without a binding it is [`routes`], exactly. With one, an operation the binding names is served
/// at the method and path it declares, and [`Route::declared`] holds the binding's route; every
/// other operation keeps the route [`routes`] derives for it. Compiling the binding already checked
/// that the declared and derived routes of each component do not collide (`ESS-HTTP-010`,
/// `ESS-HTTP-011`).
pub fn bound_routes<'a>(
    ir: &'a EssIr,
    component: &'a ResolvedComponent,
    binding: Option<&'a Binding>,
) -> Vec<Route<'a>> {
    let mut out = routes(ir, component);
    let Some(binding) = binding else {
        return out;
    };
    for route in &mut out {
        let Some(declared) = binding.route(route.serves) else {
            continue;
        };
        // `Binding::new` admits only routes with a path on a method a route is served on.
        if let (Some(path), Some(method)) = (&declared.path, Method::of(declared.method)) {
            route.method = method;
            route.path.clone_from(path);
            route.declared = Some(declared);
        }
    }
    out.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.method.cmp(&right.method))
    });
    out
}

/// The routes ESS derives for every component of `ir` without a binding, as `ess-http` reads them:
/// one entry per route of each component's [`routes`]. A binding is compiled against these
/// ([`ess_http::compile`]), which takes them as an input so that `ess-http` never depends on this
/// crate.
pub fn derived(ir: &EssIr) -> Vec<ess_http::DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for route in routes(ir, component) {
            table.push(ess_http::DerivedRoute {
                component: component.name.to_string(),
                operation: route.serves.operation(),
                method: route.method.binding(),
                path: route.path,
            });
        }
    }
    table
}

/// Every route one component's surface has, in path order.
///
/// The command routes are the ones `openapi.rs` has always published, unchanged and for every
/// component. The view routes exist only where the component declares that something outside the
/// process reaches it, because that is the declaration that turns "how is a view read" from a
/// question this generator would have to answer into one the specification has answered.
///
/// # Collisions
///
/// Two commands can derive one path — two domains may share a wire name, and a wire name is free
/// text. When that happens *both* move to their qualified names rather than one keeping the short
/// path, because a path whose meaning depends on which other commands exist is a path that changes
/// when an unrelated command is added. Views collide by the same rule and move the same way. A
/// command and a view cannot collide with each other: the segment between the domain and the name
/// is `commands` for one and `views` for the other.
pub fn routes<'a>(ir: &'a EssIr, component: &'a ResolvedComponent) -> Vec<Route<'a>> {
    let mut out: Vec<Route<'a>> = Vec::new();

    let mut claimed: std::collections::BTreeMap<String, Vec<&'a CommandHandle>> =
        std::collections::BTreeMap::new();
    for handle in &component.accepts {
        claimed
            .entry(command_path(ir, handle))
            .or_default()
            .push(handle);
    }
    for (path, handles) in claimed {
        let contested = handles.len() > 1;
        for handle in handles {
            out.push(Route {
                method: Method::Post,
                path: if contested {
                    format!("/commands/{}", ir.command(handle).name)
                } else {
                    path.clone()
                },
                serves: Served::Command(handle),
                declared: None,
            });
        }
    }

    if component.reached_by == Reach::Network {
        let mut claimed: std::collections::BTreeMap<String, Vec<&'a ViewHandle>> =
            std::collections::BTreeMap::new();
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                claimed
                    .entry(view_path(ir, handle))
                    .or_default()
                    .push(handle);
            }
        }
        for (path, handles) in claimed {
            let contested = handles.len() > 1;
            for handle in handles {
                out.push(Route {
                    method: Method::Get,
                    path: if contested {
                        format!("/views/{}", ir.view(handle).name)
                    } else {
                        path.clone()
                    },
                    serves: Served::View(handle),
                    declared: None,
                });
            }
        }
    }

    out.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.method.cmp(&right.method))
    });
    out
}

/// `/{domain wire name}/commands/{command wire name}`.
fn command_path(ir: &EssIr, handle: &CommandHandle) -> String {
    let command = ir.command(handle);
    let domain = ir.domain(&command.domain);
    format!(
        "/{}/commands/{}",
        domain.naming.wire_or(&domain.name),
        command.naming.wire_or(&command.name)
    )
}

/// `/{domain wire name}/views/{view wire name}`.
///
/// The `views` segment does the job `commands` does: it keeps the path from reading as a collection
/// resource, which is the claim this generator has no grounds for. A view *is* a collection of rows
/// — but it is a projection of entities the model never gives an address, and `/invoices/outstanding`
/// would invite a caller to expect `/invoices/{id}` beside it.
fn view_path(ir: &EssIr, handle: &ViewHandle) -> String {
    let view = ir.view(handle);
    let domain = ir.domain(&view.domain);
    format!(
        "/{}/views/{}",
        domain.naming.wire_or(&domain.name),
        view.naming.wire_or(&view.name)
    )
}

/// How a server reads one scalar off the wire as text — a path segment, a `form` or `query`
/// value — before the input's decoder reads it as the value it declares
/// (`docs/design/http-presentation-binding.md`, "What the servers write (k5, k6)").
///
/// Target-neutral: the Rust and Go servers read the same table, so a request one refuses the
/// other refuses too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flat {
    /// `String`, or a newtype of it: any text, the empty text included.
    String,
    /// Any other scalar a decoder reads from JSON text — a decimal, an instant, a duration, a
    /// UUID, base64 bytes, an enum's wire spelling. Empty text is malformed.
    Text,
    /// `Integer`: exact decimal text, read as a JSON number; anything else stays text for the
    /// decoder to refuse by name. Empty text is malformed.
    Integer,
    /// `Boolean`: `true` or `false`; anything else stays text for the decoder to refuse by name.
    /// Empty text is malformed.
    Boolean,
}

impl Flat {
    /// The variant's name, as an emitted table spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Text => "Text",
            Self::Integer => "Integer",
            Self::Boolean => "Boolean",
        }
    }
}

/// How many values one `form` or `query` key carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    /// Exactly one: the key given twice is malformed (RFC 6749 §3.1), absent is for the decoder
    /// to refuse.
    One,
    /// At most one: absent is `None`.
    Optional,
    /// Any number: the key repeats once per element.
    List,
}

impl Shape {
    /// The variant's name, as an emitted table spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "One",
            Self::Optional => "Optional",
            Self::List => "List",
        }
    }
}

/// How a value of `type_ref` is read from flat text, if it has a flat spelling: a scalar, an
/// `Optional` scalar or a `List` of scalars, a scalar being a primitive other than `Json` and
/// `Binary64`, an enum, or a newtype of a scalar. The same line `ESS-HTTP-021` draws, so every field
/// a compiled binding carries in a `form` or `query` request has one.
pub fn flat(ir: &EssIr, type_ref: &ess_compiler::ir::ResolvedTypeRef) -> Option<(Shape, Flat)> {
    use ess_compiler::ir::ResolvedTypeRef;
    match type_ref {
        ResolvedTypeRef::Optional { of } => scalar(ir, of).map(|flat| (Shape::Optional, flat)),
        ResolvedTypeRef::List { of } => scalar(ir, of).map(|flat| (Shape::List, flat)),
        _ => scalar(ir, type_ref).map(|flat| (Shape::One, flat)),
    }
}

/// How one scalar of `type_ref` is read from flat text; `None` for anything that is not one.
pub fn scalar(ir: &EssIr, type_ref: &ess_compiler::ir::ResolvedTypeRef) -> Option<Flat> {
    use ess_compiler::ir::{ResolvedBody, ResolvedTypeRef};
    use ess_domain::Primitive;
    match type_ref {
        ResolvedTypeRef::Primitive { name } => match name {
            Primitive::String => Some(Flat::String),
            Primitive::Integer => Some(Flat::Integer),
            Primitive::Boolean => Some(Flat::Boolean),
            Primitive::Json | Primitive::Binary64 => None,
            _ => Some(Flat::Text),
        },
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => scalar(ir, of),
            ResolvedBody::Enum { .. } => Some(Flat::Text),
            ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => None,
        },
        ResolvedTypeRef::Optional { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => None,
    }
}

/// The fields a bound route reads: a command's input fields, or a view's parameters.
fn route_fields<'a>(ir: &'a EssIr, route: &Route<'a>) -> &'a [ess_compiler::ir::ResolvedField] {
    match route.serves {
        Served::Command(handle) => &ir.command(handle).input,
        Served::View(handle) => &ir.view(handle).params,
    }
}

/// Each path variable of a bound route, in path order, with the input field or view parameter it
/// carries. Empty for a derived route, whose path has no variable.
pub fn path_fields<'a>(
    ir: &'a EssIr,
    route: &Route<'a>,
) -> Vec<(String, &'a ess_compiler::ir::ResolvedField)> {
    let Some(declared) = route.declared else {
        return Vec::new();
    };
    let fields = route_fields(ir, route);
    declared
        .segments()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|segment| match segment {
            ess_http::Segment::Variable(variable) => {
                let source = declared.parameters.get(&variable)?;
                let field = fields.iter().find(|field| field.name == source.field())?;
                Some((variable, field))
            }
            ess_http::Segment::Literal(_) => None,
        })
        .collect()
}

/// The names of the fields a bound route's path carries, which leave its body or query string.
pub fn carried<'a>(ir: &'a EssIr, route: &Route<'a>) -> std::collections::BTreeSet<&'a str> {
    path_fields(ir, route)
        .into_iter()
        .map(|(_, field)| field.name.as_str())
        .collect()
}

/// A bound route's request: its encoding and what an unknown parameter does. A derived route, and
/// a bound one that states none, read ESS's own `json` body.
pub fn request(route: &Route<'_>) -> ess_http::Request {
    route
        .declared
        .and_then(|declared| declared.request)
        .unwrap_or_default()
}

/// The answer a bound route declares for `outcome`: the one it lists under `outcomes`, else its
/// `errors` when the outcome reports an error. An outcome with neither keeps [`outcome_status`]
/// and the envelope.
pub fn listed<'a>(route: &Route<'a>, outcome: &ResolvedOutcome) -> Option<&'a ess_http::Outcome> {
    route
        .declared
        .and_then(|declared| declared.answer(outcome.name.as_str(), outcome.error.is_some()))
}

/// Whether a bound route declares the answer of any of its command's outcomes, under `outcomes`
/// or `errors`; an unbound route declares none.
pub fn declares_answers(route: &Route<'_>) -> bool {
    route
        .declared
        .is_some_and(ess_http::Route::declares_answers)
}
