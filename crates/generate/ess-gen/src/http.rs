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
//! URL. No path version, because `info.version` is the only version the model has. The two segments
//! a route is built from — the domain's wire name and the construct's — are both declared, and the
//! `commands`/`views` segment between them is what stops a path from reading as a resource.

use ess_compiler::ir::{
    CommandHandle, EssIr, ResolvedComponent, ResolvedCondition, ResolvedOutcome, ViewHandle,
};
use ess_domain::component::Reach;

/// Where the served component publishes the contract it answers.
///
/// A fixed path rather than a derived one: it names no construct of any specification, so there is
/// nothing to derive it from, and a served document a caller cannot find is a document that is not
/// published. JSON rather than the committed YAML because this is the machine's copy — the same
/// document, in the dialect every HTTP client already parses.
pub const OPENAPI: &str = "/openapi.json";

/// Where it publishes the prose the same model produced.
pub const DOCS: &str = "/docs";

/// The branch was taken; its events reach consumers elsewhere.
pub const TAKEN: &str = "202";

/// The projection was read.
///
/// The one `2xx` here that is not a `202`: a `GET` on a view returns the rows, so the response
/// *is* the answer rather than a receipt for a branch that was taken.
pub const READ: &str = "200";

/// The input was understood and refused on domain grounds.
///
/// A `422` and not a `400`: `400` is for a request the server could not parse, which is decided by
/// the schema and would be true of any endpoint. A refusal on domain grounds is a request the
/// server understood, and a client can act on the difference — one means fix the value, the other
/// means fix the serialiser.
pub const REFUSED: &str = "422";

/// Something outside the request refused; the input was acceptable.
///
/// A `5xx` because `external` names a branch the input cannot decide. Reporting it as a `4xx` sends
/// the caller to fix the one thing it cannot fix and tells every retry layer between that retrying
/// is pointless. `502` rather than `500`, which would claim a fault in this component, or `503`,
/// which would claim the whole component is unavailable when one provider refused one request.
pub const UPSTREAM: &str = "502";

/// The input was acceptable and the subject was in a state the command does not act from.
///
/// A `409` and not a `422`, and the difference is what a caller does next. `422` says the request
/// was wrong and resending it unchanged is pointless; `409` says the request was fine and the world
/// was not — the same visit, admitted before it was signed out, would have been accepted.
pub const CONFLICT: &str = "409";

/// The request named an identity no record carries, and the command declares that answer
/// (`unknown_instance:`, ess/15).
pub const NOT_FOUND: &str = "404";

/// The request carried no input at all, and the command declares that answer (`input_absent:`,
/// ess/16). A `400` and not a `422`: nothing was there to understand, which is the request the
/// server could not read rather than one it refused on domain grounds.
pub const NO_INPUT: &str = "400";

/// The caller is not one the branch admits: its guard compares the authenticated caller with the
/// input or the record (`caller.<attribute>`, ess/16). A `403` and not a `409` or a `422`: the
/// request and the record are fine, and the same request from the record's own agent is accepted.
///
/// It is also the status of the standard refusal for an actor no grant admits (beyond10x/ess#265,
/// [`NOT_GRANTED`]). One status for both, because both answer the same question — *this caller
/// may not do this* — and a client acts on both the same way: send it as someone else. `401`
/// would claim the credential is missing or wrong, and how a caller proves who it is belongs to
/// the realization, not to this contract. The two are told apart by the body: a declared branch
/// carries `outcome` and the declared `error`, the standard refusal carries `refused` and `actor`
/// and neither of those.
pub const FORBIDDEN: &str = "403";

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

/// Which status one declared outcome is.
///
/// The whole mapping, in one place, so that "which HTTP status does this refusal get" has exactly
/// one answer for the document that publishes it and the server that answers it. A server whose
/// statuses were computed separately would agree on the day it was written.
pub fn status(outcome: &ResolvedOutcome) -> &'static str {
    match (&outcome.condition, outcome.error.is_some()) {
        // First: who sent it decided the refusal, whatever else the guard reads.
        (_, true) if outcome.decided_by_caller => FORBIDDEN,
        (ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }, true) => {
            UPSTREAM
        }
        // All three are decided by the state the subject is resting in rather than by the
        // request, so a refusal from one of them is a conflict with that state and not a bad
        // request: `StateChange` is `SubjectState` with the states derived from the move.
        (
            ResolvedCondition::WrongState
            | ResolvedCondition::SubjectState { .. }
            | ResolvedCondition::SubjectField { .. }
            | ResolvedCondition::SubjectPredicate { .. }
            | ResolvedCondition::StateChange { .. }
            // And so is one decided by a stored row of another entity (ess/18, `when_related:`).
            | ResolvedCondition::Related { .. }
            // And by the rows a selector selects (ess/22, `when_related: {entity, where, …}`).
            | ResolvedCondition::RelatedSet { .. }
            // A duplicate of a record that exists conflicts with that record (ess/16).
            | ResolvedCondition::ExistingInstance,
            true,
        ) => CONFLICT,
        (ResolvedCondition::UnknownInstance, true) => NOT_FOUND,
        (ResolvedCondition::InputAbsent, true) => NO_INPUT,
        (ResolvedCondition::When { .. } | ResolvedCondition::Otherwise, true) => REFUSED,
        // An external branch that emits rather than errors is still a branch that was taken; what
        // decided it does not change what happened.
        (_, false) => TAKEN,
    }
}

/// The branch was taken and its answer is the command's response (`returns: true`, from
/// [`DIRECT_ANSWER_FORMAT`]).
///
/// A `200` and not a `202`: `202` tells a client the request was queued for later processing, and
/// a branch that returns its result in the same response was carried out, not queued
/// (beyond10x/ess#424).
pub const ANSWERED: &str = "200";

/// The first source format whose `returns: true` outcome is answered [`ANSWERED`], with the
/// command's declared response under `response` (beyond10x/ess#423, beyond10x/ess#424).
///
/// Below it the branch keeps the [`TAKEN`] status and the body it was published with, so a client
/// of an `ess/17` to `ess/21` contract keeps the answer it was told about.
pub const DIRECT_ANSWER_FORMAT: u32 = 22;

/// Whether `outcome` answers its caller with the command's response, under [`ANSWERED`].
///
/// The one answer for the document that declares the body and the servers that write it.
pub fn answers_with_response(ir: &EssIr, outcome: &ResolvedOutcome) -> bool {
    outcome.returns && outcome.error.is_none() && ir.format().major() >= DIRECT_ANSWER_FORMAT
}

/// Which status one declared outcome of `ir` is answered with: [`status`], except that a branch
/// that [answers with the response](answers_with_response) is [`ANSWERED`].
///
/// Every surface that writes or declares a command's status reads this, so the document and the
/// served Rust and Go applications cannot disagree about it.
pub fn outcome_status(ir: &EssIr, outcome: &ResolvedOutcome) -> &'static str {
    if answers_with_response(ir, outcome) {
        ANSWERED
    } else {
        status(outcome)
    }
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
