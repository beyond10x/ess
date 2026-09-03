//! Generated composition client for `devcenter`.
//!
//! Endpoints, verified authority, and transport are injected. Operation inputs never contain
//! authentication coordinates.

#![forbid(unsafe_code)]

/// Stable composition identity.
pub const COMPOSITION: &str = "devcenter";

/// A selected ESS component and its exact semantic dependency closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Service {
    key: &'static str,
    system: &'static str,
    version: &'static str,
    source_digest: &'static str,
    component: &'static str,
    types: &'static [&'static str],
    events: &'static [&'static str],
    errors: &'static [&'static str],
}

impl Service {
    /// Composition-local service key.
    pub const fn key(self) -> &'static str { self.key }
    /// Exact ESS system identity.
    pub const fn system(self) -> &'static str { self.system }
    /// Exact ESS specification version.
    pub const fn version(self) -> &'static str { self.version }
    /// Exact compiler-owned semantic source digest.
    pub const fn source_digest(self) -> &'static str { self.source_digest }
    /// Selected ESS component.
    pub const fn component(self) -> &'static str { self.component }
    /// Recursive named-type closure required by the client surface.
    pub const fn types(self) -> &'static [&'static str] { self.types }
    /// Event contracts required by the client surface.
    pub const fn events(self) -> &'static [&'static str] { self.events }
    /// Error contracts required by the client surface.
    pub const fn errors(self) -> &'static [&'static str] { self.errors }
}

/// The two callable ESS surface kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    /// An intent represented by an ESS command.
    Command,
    /// A query represented by an ESS view.
    Query,
}

/// One unforgeable generated operation descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation {
    service_key: &'static str,
    semantic: &'static str,
    kind: OperationKind,
}

impl Operation {
    const fn new(service_key: &'static str, semantic: &'static str, kind: OperationKind) -> Self {
        Self { service_key, semantic, kind }
    }
    /// Composition-local service key.
    pub const fn service_key(self) -> &'static str { self.service_key }
    /// Fully qualified ESS command or view name.
    pub const fn semantic(self) -> &'static str { self.semantic }
    /// Whether this is a command or query.
    pub const fn kind(self) -> OperationKind { self.kind }
}

/// Supplies an environment endpoint for one exact selected service.
pub trait EndpointProvider {
    /// Returns the endpoint or `None` when this environment has no binding.
    fn endpoint(&self, service: &Service) -> Option<&str>;
}

/// Supplies verified authentication authority at execution time.
pub trait AuthorityProvider {
    /// Application-owned verified authority type.
    type Authority;
    /// Current verified authority, including any optional realm internally.
    fn authority(&self) -> &Self::Authority;
}

/// Executes encoded operation payloads over an application-selected protocol.
pub trait Transport<Authority> {
    /// Transport or remote-service failure.
    type Error;
    /// Executes one generated operation.
    fn execute(
        &self,
        endpoint: &str,
        authority: &Authority,
        operation: Operation,
        payload: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;
}

/// Failure before or during generated client execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError<E> {
    /// The environment did not bind the selected service.
    MissingEndpoint(&'static str),
    /// The injected transport failed.
    Transport(E),
}

/// Composition client with all environmental decisions injected.
pub struct Client<Endpoints, Authority, Wire> {
    endpoints: Endpoints,
    authority: Authority,
    wire: Wire,
}

impl<Endpoints, Authority, Wire> Client<Endpoints, Authority, Wire> {
    /// Binds providers without performing I/O.
    pub const fn new(endpoints: Endpoints, authority: Authority, wire: Wire) -> Self {
        Self { endpoints, authority, wire }
    }
}

impl<Endpoints, Authority, Wire> Client<Endpoints, Authority, Wire>
where
    Endpoints: EndpointProvider,
    Authority: AuthorityProvider,
    Wire: Transport<Authority::Authority>,
{
    /// Executes one generated command or query with an encoded domain payload.
    pub fn execute(
        &self,
        operation: Operation,
        payload: &[u8],
    ) -> Result<Vec<u8>, ClientError<Wire::Error>> {
        let service = service(operation.service_key)
            .expect("generated operations always name a generated service");
        let endpoint = self.endpoints.endpoint(service)
            .ok_or(ClientError::MissingEndpoint(operation.service_key))?;
        self.wire
            .execute(endpoint, self.authority.authority(), operation, payload)
            .map_err(ClientError::Transport)
    }
}

/// Client surface for `todo` / component `todo-component`.
pub mod service_todo {
    use super::{Operation, OperationKind, Service};
    /// Exact selected surface TYPES.
    pub const TYPES: &[&str] = &[
        "workbench.todo.ListDetails",
        "workbench.todo.ListId",
        "workbench.todo.ListRow",
        "workbench.todo.Title",
    ];
    /// Exact selected surface EVENTS.
    pub const EVENTS: &[&str] = &[
        "workbench.todo.ListCreated",
    ];
    /// Exact selected surface ERRORS.
    pub const ERRORS: &[&str] = &[
    ];
    /// Exact selected service metadata.
    pub const SERVICE: Service = Service {
        key: "todo",
        system: "workbench",
        version: "v1",
        source_digest: "29fd772e3abb87d8042f8155577c3df9549219ace80c9e595d7da9783f7b7dda",
        component: "todo-component",
        types: TYPES,
        events: EVENTS,
        errors: ERRORS,
    };
    /// ESS command `workbench.todo.CreateList`.
    pub const COMMAND_CREATE_LIST: Operation = Operation::new("todo", "workbench.todo.CreateList", OperationKind::Command);
    /// ESS view query `workbench.todo.ListById`.
    pub const QUERY_LIST_BY_ID: Operation = Operation::new("todo", "workbench.todo.ListById", OperationKind::Query);
}

/// Client surface for `usage` / component `usage-component`.
pub mod service_usage {
    use super::{Operation, OperationKind, Service};
    /// Exact selected surface TYPES.
    pub const TYPES: &[&str] = &[
        "workbench.usage.UsageCount",
        "workbench.usage.UsageDetails",
        "workbench.usage.UsageId",
        "workbench.usage.UsageRow",
    ];
    /// Exact selected surface EVENTS.
    pub const EVENTS: &[&str] = &[
        "workbench.usage.UsageRecorded",
    ];
    /// Exact selected surface ERRORS.
    pub const ERRORS: &[&str] = &[
    ];
    /// Exact selected service metadata.
    pub const SERVICE: Service = Service {
        key: "usage",
        system: "workbench",
        version: "v1",
        source_digest: "29fd772e3abb87d8042f8155577c3df9549219ace80c9e595d7da9783f7b7dda",
        component: "usage-component",
        types: TYPES,
        events: EVENTS,
        errors: ERRORS,
    };
    /// ESS command `workbench.usage.RecordUsage`.
    pub const COMMAND_RECORD_USAGE: Operation = Operation::new("usage", "workbench.usage.RecordUsage", OperationKind::Command);
    /// ESS view query `workbench.usage.UsageById`.
    pub const QUERY_USAGE_BY_ID: Operation = Operation::new("usage", "workbench.usage.UsageById", OperationKind::Query);
}

/// Looks up generated service metadata by composition-local key.
pub fn service(key: &str) -> Option<&'static Service> {
    match key {
        "todo" => Some(&service_todo::SERVICE),
        "usage" => Some(&service_usage::SERVICE),
        _ => None,
    }
}
