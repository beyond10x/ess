use std::collections::{BTreeMap, BTreeSet};
/// Persisted format written by the `OpenAPI` importer.
pub const INTERFACE_FORMAT: &str = "ess-service-interface/1";

/// A typed service and its interface operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceInterface {
    /// Persisted envelope version.
    pub format: String,
    /// `OpenAPI` language version read from the source.
    pub source_openapi: String,
    /// Service identity and documentation.
    pub service: Service,
    /// Operations keyed by stable operation id.
    pub operations: BTreeMap<String, Operation>,
    /// Named interface types keyed by component-schema name.
    pub types: BTreeMap<String, InterfaceSchema>,
}

impl ServiceInterface {
    /// Serializes the interface as deterministic, newline-terminated JSON.
    pub fn to_canonical_json(&self) -> String {
        let mut output = serde_json::to_string_pretty(self)
            .expect("the typed service-interface representation serializes");
        output.push('\n');
        output
    }

    /// Validates invariants that deserialization alone cannot express.
    pub fn validate(&self) -> Result<(), Vec<Refusal>> {
        let mut refusals = Vec::new();
        if self.format != INTERFACE_FORMAT {
            refusals.push(Refusal::new(
                "/format",
                format!("expected `{INTERFACE_FORMAT}`, found `{}`", self.format),
            ));
        }
        if self.service.name.trim().is_empty() {
            refusals.push(Refusal::new("/service/name", "service name is empty"));
        }
        if self.service.version.trim().is_empty() {
            refusals.push(Refusal::new("/service/version", "service version is empty"));
        }
        let mut locations = BTreeSet::new();
        for (id, operation) in &self.operations {
            if id != &operation.id {
                refusals.push(Refusal::new(
                    format!("/operations/{id}/id"),
                    "operation map key and operation id differ",
                ));
            }
            if !operation.path.starts_with('/') {
                refusals.push(Refusal::new(
                    format!("/operations/{id}/path"),
                    "operation path must start with `/`",
                ));
            }
            if !locations.insert((operation.path.clone(), operation.method)) {
                refusals.push(Refusal::new(
                    format!("/operations/{id}"),
                    "two operations claim the same method and path",
                ));
            }
        }
        if refusals.is_empty() {
            Ok(())
        } else {
            Err(refusals)
        }
    }
}

/// Stable service metadata carried independently of one protocol rendering.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    /// Human-meaningful service name.
    pub name: String,
    /// Contract version declared by the service.
    pub version: String,
    /// Optional one-line summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Optional longer description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// One callable interface operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    /// Stable operation identity.
    pub id: String,
    /// Concrete protocol path.
    pub path: String,
    /// Concrete HTTP method.
    pub method: HttpMethod,
    /// Optional one-line summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Ordered, normalized tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// JSON request message, when the operation accepts one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Message>,
    /// Responses keyed by `OpenAPI` status selector (`202`, `default`).
    pub responses: BTreeMap<String, Message>,
}

/// Supported HTTP methods.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    /// GET.
    Get,
    /// PUT.
    Put,
    /// POST.
    Post,
    /// DELETE.
    Delete,
    /// OPTIONS.
    Options,
    /// HEAD.
    Head,
    /// PATCH.
    Patch,
    /// TRACE.
    Trace,
}

impl HttpMethod {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "get" => Some(Self::Get),
            "put" => Some(Self::Put),
            "post" => Some(Self::Post),
            "delete" => Some(Self::Delete),
            "options" => Some(Self::Options),
            "head" => Some(Self::Head),
            "patch" => Some(Self::Patch),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Put => "put",
            Self::Post => "post",
            Self::Delete => "delete",
            Self::Options => "options",
            Self::Head => "head",
            Self::Patch => "patch",
            Self::Trace => "trace",
        }
    }
}

/// A JSON request or response message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    /// Whether the message is required. Responses are always written as required.
    pub required: bool,
    /// Typed JSON shape, or no body when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<InterfaceSchema>,
}

/// Structural JSON Schema subset preserved by this adapter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InterfaceSchema {
    /// Reference to a named component schema.
    Reference {
        /// Component-schema name.
        name: String,
    },
    /// String value, optionally refined without changing its structural type.
    String {
        /// `OpenAPI` format annotation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        /// Regular-expression refinement.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        /// Closed string enumeration.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        values: Vec<String>,
        /// One required literal value.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        constant: Option<String>,
    },
    /// Integer value.
    Integer,
    /// General JSON number.
    Number,
    /// Boolean value.
    Boolean,
    /// Ordered array.
    Array {
        /// Element shape.
        items: Box<InterfaceSchema>,
    },
    /// Closed object with explicitly typed properties.
    Object {
        /// Properties keyed deterministically by wire name.
        properties: BTreeMap<String, InterfaceSchema>,
        /// Required property names.
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        required: BTreeSet<String>,
    },
}

/// A source defect that prevents a truthful typed import or projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Refusal {
    /// JSON Pointer to the offending source location.
    pub pointer: String,
    /// Reason the adapter refused it.
    pub message: String,
}

impl Refusal {
    fn new(pointer: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            pointer: pointer.into(),
            message: message.into(),
        }
    }
}
/// Reads an importer-produced service-interface document from JSON or YAML.
pub fn read_interface(text: &str) -> Result<ServiceInterface, Vec<Refusal>> {
    let interface: ServiceInterface = match serde_yaml::from_str(text) {
        Ok(interface) => interface,
        Err(error) => {
            return Err(vec![Refusal::new(
                "/",
                format!("malformed interface: {error}"),
            )])
        }
    };
    interface.validate()?;
    Ok(interface)
}
