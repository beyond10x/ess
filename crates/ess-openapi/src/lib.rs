//! Typed import and deterministic projection for the supported `OpenAPI` service-interface subset.
//!
//! This adapter deliberately models only semantics it can preserve. Source annotations are
//! normalized away, unsupported behavior is reported as a coverage gap, and external references
//! are refused rather than fetched or guessed.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

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

/// Successful import plus the limitations and unresolved local references it observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    /// Validated typed interface.
    pub interface: ServiceInterface,
    /// Source constructs intentionally normalized away or unsupported.
    pub coverage_gaps: Vec<String>,
    /// Local component references whose declarations were absent.
    pub unresolved_references: Vec<String>,
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

/// Imports one `OpenAPI` 3.1 document into typed service-interface IR.
pub fn import(text: &str) -> Result<ImportReport, Vec<Refusal>> {
    let value: Value = match serde_yaml::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            return Err(vec![Refusal::new(
                "/",
                format!("malformed document: {error}"),
            )])
        }
    };
    Importer::new().document(&value)
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

/// Projects typed service-interface IR into deterministic normalized `OpenAPI` 3.1 YAML.
pub fn project(interface: &ServiceInterface) -> Result<String, Vec<Refusal>> {
    interface.validate()?;
    let mut root = Map::new();
    root.insert("openapi".to_owned(), Value::String("3.1.0".to_owned()));
    root.insert("info".to_owned(), info_value(&interface.service));

    let mut paths: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for operation in interface.operations.values() {
        paths.entry(operation.path.clone()).or_default().insert(
            operation.method.as_str().to_owned(),
            operation_value(operation),
        );
    }
    root.insert(
        "paths".to_owned(),
        serde_json::to_value(paths).expect("typed paths serialize"),
    );
    let schemas: BTreeMap<&String, Value> = interface
        .types
        .iter()
        .map(|(name, schema)| (name, schema_value(schema)))
        .collect();
    root.insert(
        "components".to_owned(),
        serde_json::json!({ "schemas": schemas }),
    );
    serde_yaml::to_string(&Value::Object(root))
        .map_err(|error| vec![Refusal::new("/", format!("projection failed: {error}"))])
}

fn info_value(service: &Service) -> Value {
    let mut info = Map::new();
    info.insert("title".to_owned(), Value::String(service.name.clone()));
    info.insert("version".to_owned(), Value::String(service.version.clone()));
    if let Some(summary) = &service.summary {
        info.insert("summary".to_owned(), Value::String(summary.clone()));
    }
    if let Some(description) = &service.description {
        info.insert("description".to_owned(), Value::String(description.clone()));
    }
    Value::Object(info)
}

fn operation_value(operation: &Operation) -> Value {
    let mut value = Map::new();
    value.insert(
        "operationId".to_owned(),
        Value::String(operation.id.clone()),
    );
    if let Some(summary) = &operation.summary {
        value.insert("summary".to_owned(), Value::String(summary.clone()));
    }
    if !operation.tags.is_empty() {
        value.insert(
            "tags".to_owned(),
            serde_json::to_value(&operation.tags).expect("tags serialize"),
        );
    }
    if let Some(request) = &operation.request {
        let mut body = Map::new();
        body.insert("required".to_owned(), Value::Bool(request.required));
        if let Some(schema) = &request.schema {
            body.insert(
                "content".to_owned(),
                serde_json::json!({ "application/json": { "schema": schema_value(schema) } }),
            );
        }
        value.insert("requestBody".to_owned(), Value::Object(body));
    }
    let responses: BTreeMap<&String, Value> = operation
        .responses
        .iter()
        .map(|(status, message)| {
            let mut response = Map::new();
            response.insert(
                "description".to_owned(),
                Value::String(format!("Response {status}")),
            );
            if let Some(schema) = &message.schema {
                response.insert(
                    "content".to_owned(),
                    serde_json::json!({ "application/json": { "schema": schema_value(schema) } }),
                );
            }
            (status, Value::Object(response))
        })
        .collect();
    value.insert(
        "responses".to_owned(),
        serde_json::to_value(responses).expect("responses serialize"),
    );
    Value::Object(value)
}

fn schema_value(schema: &InterfaceSchema) -> Value {
    let mut value = Map::new();
    match schema {
        InterfaceSchema::Reference { name } => {
            value.insert(
                "$ref".to_owned(),
                Value::String(format!("#/components/schemas/{}", pointer_escape(name))),
            );
        }
        InterfaceSchema::String {
            format,
            pattern,
            values,
            constant,
        } => {
            value.insert("type".to_owned(), Value::String("string".to_owned()));
            if let Some(format) = format {
                value.insert("format".to_owned(), Value::String(format.clone()));
            }
            if let Some(pattern) = pattern {
                value.insert("pattern".to_owned(), Value::String(pattern.clone()));
            }
            if !values.is_empty() {
                value.insert(
                    "enum".to_owned(),
                    serde_json::to_value(values).expect("string values serialize"),
                );
            }
            if let Some(constant) = constant {
                value.insert("const".to_owned(), Value::String(constant.clone()));
            }
        }
        InterfaceSchema::Integer => {
            value.insert("type".to_owned(), Value::String("integer".to_owned()));
        }
        InterfaceSchema::Number => {
            value.insert("type".to_owned(), Value::String("number".to_owned()));
        }
        InterfaceSchema::Boolean => {
            value.insert("type".to_owned(), Value::String("boolean".to_owned()));
        }
        InterfaceSchema::Array { items } => {
            value.insert("type".to_owned(), Value::String("array".to_owned()));
            value.insert("items".to_owned(), schema_value(items));
        }
        InterfaceSchema::Object {
            properties,
            required,
        } => {
            value.insert("type".to_owned(), Value::String("object".to_owned()));
            value.insert(
                "properties".to_owned(),
                Value::Object(
                    properties
                        .iter()
                        .map(|(name, property)| (name.clone(), schema_value(property)))
                        .collect(),
                ),
            );
            value.insert(
                "required".to_owned(),
                serde_json::to_value(required).expect("required set serializes"),
            );
            value.insert("additionalProperties".to_owned(), Value::Bool(false));
        }
    }
    Value::Object(value)
}

struct Importer {
    gaps: BTreeSet<String>,
    refusals: Vec<Refusal>,
    references: BTreeSet<String>,
}

impl Importer {
    fn new() -> Self {
        Self {
            gaps: BTreeSet::new(),
            refusals: Vec::new(),
            references: BTreeSet::new(),
        }
    }

    fn document(mut self, value: &Value) -> Result<ImportReport, Vec<Refusal>> {
        let Some(root) = value.as_object() else {
            return Err(vec![Refusal::new("/", "document root must be an object")]);
        };
        let version = self.required_string(root, "openapi", "/openapi");
        if let Some(version) = &version {
            if !version.starts_with("3.1.") {
                self.refusals.push(Refusal::new(
                    "/openapi",
                    format!("only OpenAPI 3.1 is supported, found `{version}`"),
                ));
            }
        }
        let service = self.service(root.get("info"));
        let types = self.types(root.get("components"));
        let operations = self.operations(root.get("paths"));
        self.note_root_gaps(root);

        if !self.refusals.is_empty() {
            return Err(self.refusals);
        }
        let interface = ServiceInterface {
            format: INTERFACE_FORMAT.to_owned(),
            source_openapi: version.expect("required version checked"),
            service: service.expect("required info checked"),
            operations,
            types,
        };
        interface.validate()?;
        let unresolved_references = self
            .references
            .difference(&interface.types.keys().cloned().collect())
            .cloned()
            .collect();
        Ok(ImportReport {
            interface,
            coverage_gaps: self.gaps.into_iter().collect(),
            unresolved_references,
        })
    }

    fn service(&mut self, value: Option<&Value>) -> Option<Service> {
        let Some(info) = value.and_then(Value::as_object) else {
            self.refusals
                .push(Refusal::new("/info", "info must be an object"));
            return None;
        };
        let name = self.required_string(info, "title", "/info/title");
        let version = self.required_string(info, "version", "/info/version");
        for key in info.keys() {
            if !matches!(
                key.as_str(),
                "title" | "version" | "summary" | "description" | "x-ess-provenance"
            ) && !key.starts_with("x-")
            {
                self.gaps
                    .insert(format!("/info/{key}: annotation normalized"));
            }
        }
        match (name, version) {
            (Some(name), Some(version)) => Some(Service {
                name,
                version,
                summary: optional_string(info, "summary"),
                description: optional_string(info, "description"),
            }),
            _ => None,
        }
    }

    fn types(&mut self, value: Option<&Value>) -> BTreeMap<String, InterfaceSchema> {
        let Some(components) = value.and_then(Value::as_object) else {
            return BTreeMap::new();
        };
        for key in components.keys() {
            if key != "schemas" {
                self.gaps
                    .insert(format!("/components/{key}: component kind unsupported"));
            }
        }
        let Some(schemas) = components.get("schemas").and_then(Value::as_object) else {
            return BTreeMap::new();
        };
        schemas
            .iter()
            .filter_map(|(name, value)| {
                self.schema(
                    value,
                    &format!("/components/schemas/{}", pointer_escape(name)),
                )
                .map(|schema| (name.clone(), schema))
            })
            .collect()
    }

    fn operations(&mut self, value: Option<&Value>) -> BTreeMap<String, Operation> {
        let Some(paths) = value.and_then(Value::as_object) else {
            self.refusals
                .push(Refusal::new("/paths", "paths must be an object"));
            return BTreeMap::new();
        };
        let mut operations = BTreeMap::new();
        for (path, item) in paths {
            if !path.starts_with('/') {
                self.refusals.push(Refusal::new(
                    format!("/paths/{}", pointer_escape(path)),
                    "path must start with `/`",
                ));
            }
            let Some(item) = item.as_object() else {
                self.refusals.push(Refusal::new(
                    format!("/paths/{}", pointer_escape(path)),
                    "path item must be an object",
                ));
                continue;
            };
            for (key, value) in item {
                let Some(method) = HttpMethod::parse(key) else {
                    if key == "$ref" {
                        self.refusals.push(Refusal::new(
                            format!("/paths/{}/$ref", pointer_escape(path)),
                            "path-item references are unsupported",
                        ));
                    } else {
                        self.gaps.insert(format!(
                            "/paths/{}/{}: path-item feature normalized",
                            pointer_escape(path),
                            pointer_escape(key)
                        ));
                    }
                    continue;
                };
                if let Some(operation) = self.operation(path, method, value) {
                    if operations.insert(operation.id.clone(), operation).is_some() {
                        self.refusals.push(Refusal::new(
                            format!("/paths/{}/{}", pointer_escape(path), key),
                            "duplicate operationId",
                        ));
                    }
                }
            }
        }
        operations
    }

    fn operation(&mut self, path: &str, method: HttpMethod, value: &Value) -> Option<Operation> {
        let pointer = format!("/paths/{}/{}", pointer_escape(path), method.as_str());
        let Some(object) = value.as_object() else {
            self.refusals
                .push(Refusal::new(&pointer, "operation must be an object"));
            return None;
        };
        let id = self.required_string(object, "operationId", &format!("{pointer}/operationId"));
        for key in object.keys() {
            if !matches!(
                key.as_str(),
                "operationId" | "summary" | "tags" | "requestBody" | "responses"
            ) {
                self.gaps.insert(format!(
                    "{pointer}/{}: operation feature unsupported",
                    pointer_escape(key)
                ));
            }
        }
        let responses = self.responses(object.get("responses"), &format!("{pointer}/responses"));
        id.map(|id| Operation {
            id,
            path: path.to_owned(),
            method,
            summary: optional_string(object, "summary"),
            tags: string_array(
                object.get("tags"),
                &format!("{pointer}/tags"),
                &mut self.refusals,
            ),
            request: object
                .get("requestBody")
                .and_then(|body| self.message(body, &format!("{pointer}/requestBody"), true)),
            responses,
        })
    }

    fn responses(&mut self, value: Option<&Value>, pointer: &str) -> BTreeMap<String, Message> {
        let Some(responses) = value.and_then(Value::as_object) else {
            self.refusals
                .push(Refusal::new(pointer, "responses must be an object"));
            return BTreeMap::new();
        };
        responses
            .iter()
            .filter_map(|(status, response)| {
                self.message(
                    response,
                    &format!("{pointer}/{}", pointer_escape(status)),
                    false,
                )
                .map(|message| (status.clone(), message))
            })
            .collect()
    }

    fn message(&mut self, value: &Value, pointer: &str, request: bool) -> Option<Message> {
        let Some(object) = value.as_object() else {
            self.refusals
                .push(Refusal::new(pointer, "message must be an object"));
            return None;
        };
        if object.contains_key("$ref") {
            self.refusals.push(Refusal::new(
                format!("{pointer}/$ref"),
                "message references are unsupported",
            ));
            return None;
        }
        for key in object.keys() {
            let supported = if request {
                matches!(key.as_str(), "description" | "required" | "content")
            } else {
                matches!(key.as_str(), "description" | "content")
            };
            if !supported {
                self.gaps.insert(format!(
                    "{pointer}/{}: message feature unsupported",
                    pointer_escape(key)
                ));
            }
        }
        let schema = self.json_schema(object.get("content"), &format!("{pointer}/content"));
        Some(Message {
            required: request
                && object
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            schema,
        })
    }

    fn json_schema(&mut self, value: Option<&Value>, pointer: &str) -> Option<InterfaceSchema> {
        let content = value.and_then(Value::as_object)?;
        for media_type in content.keys() {
            if media_type != "application/json" {
                self.gaps.insert(format!(
                    "{pointer}/{}: media type unsupported",
                    pointer_escape(media_type)
                ));
            }
        }
        let media = content.get("application/json").and_then(Value::as_object)?;
        self.schema(
            media.get("schema")?,
            &format!("{pointer}/application~1json/schema"),
        )
    }

    fn schema(&mut self, value: &Value, pointer: &str) -> Option<InterfaceSchema> {
        let Some(object) = value.as_object() else {
            self.refusals
                .push(Refusal::new(pointer, "schema must be an object"));
            return None;
        };
        if let Some(reference) = object.get("$ref") {
            let Some(reference) = reference.as_str() else {
                self.refusals.push(Refusal::new(
                    format!("{pointer}/$ref"),
                    "$ref must be a string",
                ));
                return None;
            };
            let prefix = "#/components/schemas/";
            let Some(name) = reference.strip_prefix(prefix).and_then(pointer_unescape) else {
                self.refusals.push(Refusal::new(
                    format!("{pointer}/$ref"),
                    "only local component-schema references are supported",
                ));
                return None;
            };
            self.references.insert(name.clone());
            return Some(InterfaceSchema::Reference { name });
        }
        let kind = object.get("type").and_then(Value::as_str);
        let inferred_string = kind == Some("string")
            || (kind.is_none() && (object.contains_key("const") || object.contains_key("enum")));
        let schema = if inferred_string {
            let values = string_array(
                object.get("enum"),
                &format!("{pointer}/enum"),
                &mut self.refusals,
            );
            let constant = object
                .get("const")
                .map(|value| {
                    value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                        Refusal::new(format!("{pointer}/const"), "string const must be a string")
                    })
                })
                .transpose();
            match constant {
                Ok(constant) => Some(InterfaceSchema::String {
                    format: optional_string(object, "format"),
                    pattern: optional_string(object, "pattern"),
                    values,
                    constant,
                }),
                Err(refusal) => {
                    self.refusals.push(refusal);
                    None
                }
            }
        } else {
            match kind {
                Some("integer") => Some(InterfaceSchema::Integer),
                Some("number") => Some(InterfaceSchema::Number),
                Some("boolean") => Some(InterfaceSchema::Boolean),
                Some("array") => object
                    .get("items")
                    .and_then(|items| self.schema(items, &format!("{pointer}/items")))
                    .map(|items| InterfaceSchema::Array {
                        items: Box::new(items),
                    }),
                Some("object") => self.object_schema(object, pointer),
                Some(other) => {
                    self.refusals.push(Refusal::new(
                        format!("{pointer}/type"),
                        format!("schema type `{other}` is unsupported"),
                    ));
                    None
                }
                None => {
                    self.refusals.push(Refusal::new(
                        pointer,
                        "schema must declare `$ref` or one supported `type`",
                    ));
                    None
                }
            }
        };
        self.schema_gaps(object, pointer);
        schema
    }

    fn object_schema(
        &mut self,
        object: &Map<String, Value>,
        pointer: &str,
    ) -> Option<InterfaceSchema> {
        if object.get("additionalProperties") != Some(&Value::Bool(false)) {
            self.refusals.push(Refusal::new(
                format!("{pointer}/additionalProperties"),
                "supported objects must be closed with `additionalProperties: false`",
            ));
        }
        let Some(properties) = object.get("properties").and_then(Value::as_object) else {
            self.refusals.push(Refusal::new(
                format!("{pointer}/properties"),
                "object properties must be an object",
            ));
            return None;
        };
        let properties: BTreeMap<String, InterfaceSchema> = properties
            .iter()
            .filter_map(|(name, value)| {
                self.schema(
                    value,
                    &format!("{pointer}/properties/{}", pointer_escape(name)),
                )
                .map(|schema| (name.clone(), schema))
            })
            .collect();
        let required: BTreeSet<String> = string_array(
            object.get("required"),
            &format!("{pointer}/required"),
            &mut self.refusals,
        )
        .into_iter()
        .collect();
        for name in &required {
            if !properties.contains_key(name) {
                self.refusals.push(Refusal::new(
                    format!("{pointer}/required"),
                    format!("required property `{name}` is not declared"),
                ));
            }
        }
        Some(InterfaceSchema::Object {
            properties,
            required,
        })
    }

    fn schema_gaps(&mut self, object: &Map<String, Value>, pointer: &str) {
        for key in object.keys() {
            if !matches!(
                key.as_str(),
                "$ref"
                    | "type"
                    | "format"
                    | "pattern"
                    | "enum"
                    | "const"
                    | "items"
                    | "properties"
                    | "required"
                    | "additionalProperties"
                    | "title"
                    | "description"
                    | "x-ess-name"
                    | "x-ess-kind"
                    | "x-ess-invariants"
            ) && !key.starts_with("x-")
            {
                self.gaps.insert(format!(
                    "{pointer}/{}: schema keyword unsupported",
                    pointer_escape(key)
                ));
            }
        }
    }

    fn note_root_gaps(&mut self, root: &Map<String, Value>) {
        for key in root.keys() {
            if !matches!(
                key.as_str(),
                "openapi" | "info" | "paths" | "components" | "tags"
            ) && !key.starts_with("x-")
            {
                self.gaps.insert(format!(
                    "/{}: document feature unsupported",
                    pointer_escape(key)
                ));
            }
        }
        if root.contains_key("tags") {
            self.gaps
                .insert("/tags: top-level tag descriptions normalized".to_owned());
        }
    }

    fn required_string(
        &mut self,
        object: &Map<String, Value>,
        key: &str,
        pointer: &str,
    ) -> Option<String> {
        match object.get(key).and_then(Value::as_str) {
            Some(value) if !value.trim().is_empty() => Some(value.to_owned()),
            _ => {
                self.refusals.push(Refusal::new(
                    pointer,
                    "required non-empty string is missing",
                ));
                None
            }
        }
    }
}

fn optional_string(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn string_array(value: Option<&Value>, pointer: &str, refusals: &mut Vec<Refusal>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(values) = value.as_array() else {
        refusals.push(Refusal::new(pointer, "expected an array of strings"));
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|value| {
            if let Some(value) = value.as_str() {
                Some(value.to_owned())
            } else {
                refusals.push(Refusal::new(pointer, "array entry must be a string"));
                None
            }
        })
        .collect()
}

fn pointer_escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn pointer_unescape(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '~' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('0') => output.push('~'),
            Some('1') => output.push('/'),
            _ => return None,
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r"
openapi: 3.1.0
info:
  title: invoice-service
  version: v1
paths:
  /invoices:
    post:
      operationId: createInvoice
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateInvoice'
      responses:
        '202':
          description: accepted
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Invoice'
components:
  schemas:
    CreateInvoice:
      type: object
      properties:
        amount:
          type: number
      required: [amount]
      additionalProperties: false
    Invoice:
      type: object
      properties:
        id:
          type: string
          format: uuid
      required: [id]
      additionalProperties: false
";

    #[test]
    fn supported_ir_survives_projection_and_reimport_semantically() {
        let imported = import(SOURCE).expect("supported source imports");
        let projected = project(&imported.interface).expect("typed IR projects");
        let reimported = import(&projected).expect("projection imports");
        assert_eq!(reimported.interface, imported.interface);
        assert_eq!(project(&reimported.interface), Ok(projected));
    }

    #[test]
    fn unresolved_local_references_are_reported_without_being_guessed() {
        let source = SOURCE.replace(
            "#/components/schemas/Invoice'",
            "#/components/schemas/Missing'",
        );
        let imported = import(&source).expect("a dangling local reference remains representable");
        assert_eq!(imported.unresolved_references, vec!["Missing"]);
    }

    #[test]
    fn an_external_reference_is_refused_instead_of_fetched() {
        let source = SOURCE.replace(
            "#/components/schemas/CreateInvoice'",
            "https://example.invalid/schema.json'",
        );
        let refusals = import(&source).expect_err("external references are outside the adapter");
        assert!(refusals.iter().any(|refusal| {
            refusal.pointer.ends_with("/$ref")
                && refusal.message.contains("only local component-schema")
        }));
    }

    #[test]
    fn projection_is_byte_deterministic() {
        let interface = import(SOURCE).expect("source imports").interface;
        assert_eq!(project(&interface), project(&interface));
        assert_eq!(interface.to_canonical_json(), interface.to_canonical_json());
    }
}
