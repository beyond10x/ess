//! Typed import and deterministic projection for the supported `OpenAPI` service-interface subset.
//!
//! This adapter deliberately models only semantics it can preserve. Source annotations are
//! normalized away, unsupported behavior is reported as a coverage gap, and external references
//! are refused rather than fetched or guessed.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

mod accounting;

pub use accounting::{
    project_import, read_import, AccountingCode, AccountingEntry, ImportAccounting, ImportReport,
    ImportSource, UnresolvedReference, IMPORT_FORMAT, NORMALIZATION_PROFILE, SCHEMA_DIALECT,
};

/// Structural interface format retained inside the `OpenAPI` import envelope.
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

/// Imports one `OpenAPI` 3.1 document into typed service-interface IR.
pub fn import(text: &str) -> Result<ImportReport, Vec<Refusal>> {
    let value: Value = match serde_yaml::from_value(accounting::strict_value(text)?) {
        Ok(value) => value,
        Err(error) => {
            return Err(vec![Refusal::new(
                "/",
                format!("malformed document: {error}"),
            )])
        }
    };
    Importer::new().document(&value, text)
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
    gaps: BTreeSet<AccountingEntry>,
    normalizations: BTreeSet<AccountingEntry>,
    refusals: Vec<Refusal>,
    references: BTreeSet<UnresolvedReference>,
}

#[derive(Clone, Copy)]
enum KeywordContext {
    Schema,
    Interface,
    Operation,
    Message,
    Media,
    Path,
}

impl KeywordContext {
    fn annotation(self, key: &str) -> bool {
        match self {
            Self::Schema => matches!(
                key,
                "title" | "description" | "example" | "examples" | "default" | "deprecated"
            ),
            Self::Operation => matches!(key, "description" | "deprecated"),
            Self::Message | Self::Path => key == "description",
            Self::Media => matches!(key, "example" | "examples"),
            Self::Interface => false,
        }
    }
}

impl Importer {
    fn new() -> Self {
        Self {
            gaps: BTreeSet::new(),
            normalizations: BTreeSet::new(),
            refusals: Vec::new(),
            references: BTreeSet::new(),
        }
    }

    fn document(mut self, value: &Value, text: &str) -> Result<ImportReport, Vec<Refusal>> {
        let Some(root) = value.as_object() else {
            return Err(vec![Refusal::new("/", "document root must be an object")]);
        };
        let version = self.required_string(root, "openapi", "/openapi");
        if let Some(version) = &version {
            if !version.strip_prefix("3.1.").is_some_and(|patch| {
                !patch.is_empty()
                    && patch.bytes().all(|byte| byte.is_ascii_digit())
                    && (patch == "0" || !patch.starts_with('0'))
            }) {
                self.refusals.push(Refusal::new(
                    "/openapi",
                    format!("only OpenAPI 3.1 is supported, found `{version}`"),
                ));
            }
        }
        self.check_dialect(root.get("jsonSchemaDialect"), "/jsonSchemaDialect");
        if !self.refusals.is_empty() {
            return Err(self.refusals);
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
            .into_iter()
            .filter(|reference| !interface.types.contains_key(&reference.target))
            .collect();
        Ok(ImportReport::new(
            text,
            interface,
            ImportAccounting {
                normalizations: self.normalizations.into_iter().collect(),
                coverage_gaps: self.gaps.into_iter().collect(),
                unresolved_references,
            },
        ))
    }

    fn service(&mut self, value: Option<&Value>) -> Option<Service> {
        let Some(info) = value.and_then(Value::as_object) else {
            self.refusals
                .push(Refusal::new("/info", "info must be an object"));
            return None;
        };
        let name = self.required_string(info, "title", "/info/title");
        let version = self.required_string(info, "version", "/info/version");
        self.note_keywords(
            info,
            &["title", "version", "summary", "description"],
            "/info",
            KeywordContext::Interface,
        );
        match (name, version) {
            (Some(name), Some(version)) => Some(Service {
                name,
                version,
                summary: self.optional_string(info, "summary", "/info"),
                description: self.optional_string(info, "description", "/info"),
            }),
            _ => None,
        }
    }

    fn types(&mut self, value: Option<&Value>) -> BTreeMap<String, InterfaceSchema> {
        let Some(value) = value else {
            return BTreeMap::new();
        };
        let Some(components) = value.as_object() else {
            self.refusals
                .push(Refusal::new("/components", "components must be an object"));
            return BTreeMap::new();
        };
        self.note_keywords(
            components,
            &["schemas"],
            "/components",
            KeywordContext::Interface,
        );
        let Some(value) = components.get("schemas") else {
            return BTreeMap::new();
        };
        let Some(schemas) = value.as_object() else {
            self.refusals.push(Refusal::new(
                "/components/schemas",
                "schemas must be an object",
            ));
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
                        self.note_keyword(
                            key,
                            &format!("/paths/{}", pointer_escape(path)),
                            KeywordContext::Path,
                        );
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
        self.note_keywords(
            object,
            &["operationId", "summary", "tags", "requestBody", "responses"],
            &pointer,
            KeywordContext::Operation,
        );
        let responses = self.responses(object.get("responses"), &format!("{pointer}/responses"));
        id.map(|id| Operation {
            id,
            path: path.to_owned(),
            method,
            summary: self.optional_string(object, "summary", &pointer),
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
        self.note_keywords(
            object,
            if request {
                &["required", "content"]
            } else {
                &["content"]
            },
            pointer,
            KeywordContext::Message,
        );
        if request
            && object
                .get("required")
                .is_some_and(|value| !value.is_boolean())
        {
            self.refusals.push(Refusal::new(
                format!("{pointer}/required"),
                "message required must be a boolean",
            ));
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
        let value = value?;
        let Some(content) = value.as_object() else {
            self.refusals
                .push(Refusal::new(pointer, "content must be an object"));
            return None;
        };
        for media_type in content.keys() {
            if media_type != "application/json" {
                self.note_keyword(media_type, pointer, KeywordContext::Interface);
            }
        }
        let value = content.get("application/json")?;
        let media_pointer = format!("{pointer}/application~1json");
        let Some(media) = value.as_object() else {
            self.refusals
                .push(Refusal::new(&media_pointer, "media type must be an object"));
            return None;
        };
        self.note_keywords(media, &["schema"], &media_pointer, KeywordContext::Media);
        let Some(schema) = media.get("schema") else {
            self.refusals.push(Refusal::new(
                format!("{media_pointer}/schema"),
                "JSON content without a typed schema is unsupported",
            ));
            return None;
        };
        self.schema(schema, &format!("{pointer}/application~1json/schema"))
    }

    fn schema(&mut self, value: &Value, pointer: &str) -> Option<InterfaceSchema> {
        let Some(object) = value.as_object() else {
            self.refusals.push(Refusal::new(
                pointer,
                "only object schemas are supported; boolean schemas are valid but unsupported",
            ));
            return None;
        };
        if !self.schema_context(object, pointer) {
            return None;
        }
        if let Some(reference) = object.get("$ref") {
            let schema = self.local_reference(reference, pointer);
            self.note_keywords(
                object,
                &["$ref", "$schema"],
                pointer,
                KeywordContext::Schema,
            );
            return schema;
        }
        let kind = object.get("type").and_then(Value::as_str);
        let inferred_string = kind == Some("string")
            || (kind.is_none() && (object.contains_key("const") || object.contains_key("enum")));
        let schema = if inferred_string {
            self.string_schema(object, pointer)
        } else {
            match kind {
                Some("integer") => Some(InterfaceSchema::Integer),
                Some("number") => Some(InterfaceSchema::Number),
                Some("boolean") => Some(InterfaceSchema::Boolean),
                Some("array") => {
                    if let Some(items) = object.get("items") {
                        self.schema(items, &format!("{pointer}/items"))
                            .map(|items| InterfaceSchema::Array {
                                items: Box::new(items),
                            })
                    } else {
                        self.refusals.push(Refusal::new(
                            format!("{pointer}/items"),
                            "array without typed items is a valid but unsupported adapter shape",
                        ));
                        None
                    }
                }
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
        let consumed: &[&str] = match &schema {
            Some(InterfaceSchema::String { .. }) => {
                &["type", "$schema", "format", "pattern", "enum", "const"]
            }
            Some(InterfaceSchema::Array { .. }) => &["type", "$schema", "items"],
            Some(InterfaceSchema::Object { .. }) => &[
                "type",
                "$schema",
                "properties",
                "required",
                "additionalProperties",
            ],
            _ => &["type", "$schema"],
        };
        self.note_keywords(object, consumed, pointer, KeywordContext::Schema);
        schema
    }

    fn string_schema(
        &mut self,
        object: &Map<String, Value>,
        pointer: &str,
    ) -> Option<InterfaceSchema> {
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
                format: self.optional_string(object, "format", pointer),
                pattern: self.optional_string(object, "pattern", pointer),
                values,
                constant,
            }),
            Err(refusal) => {
                self.refusals.push(refusal);
                None
            }
        }
    }

    fn schema_context(&mut self, object: &Map<String, Value>, pointer: &str) -> bool {
        if !self.check_dialect(object.get("$schema"), &format!("{pointer}/$schema")) {
            return false;
        }
        for key in [
            "$id",
            "$anchor",
            "$dynamicAnchor",
            "$dynamicRef",
            "$recursiveAnchor",
            "$recursiveRef",
            "$vocabulary",
        ] {
            if object.contains_key(key) {
                self.refusals.push(Refusal::new(
                    format!("{pointer}/{key}"),
                    "schema resource, anchor or vocabulary interpretation is unsupported",
                ));
                return false;
            }
        }
        if let Some(values) = object.get("enum") {
            if values.as_array().is_none_or(Vec::is_empty) {
                self.refusals.push(Refusal::new(format!("{pointer}/enum"), "a nonempty enum array is required by this adapter; empty enumerations are unsupported"));
                return false;
            }
        }
        if object.get("type").is_some_and(|kind| !kind.is_string()) {
            self.refusals.push(Refusal::new(
                format!("{pointer}/type"),
                "only a single string schema type is supported; type arrays are unsupported",
            ));
            return false;
        }
        true
    }

    fn local_reference(&mut self, value: &Value, pointer: &str) -> Option<InterfaceSchema> {
        let Some(reference) = value.as_str() else {
            self.refusals.push(Refusal::new(
                format!("{pointer}/$ref"),
                "$ref must be a string",
            ));
            return None;
        };
        let Some(name) = reference
            .strip_prefix("#/components/schemas/")
            .filter(|name| {
                !name.is_empty()
                    && !name.chars().any(|character| {
                        matches!(character, '/' | '%' | '#')
                            || character.is_whitespace()
                            || character.is_control()
                    })
            })
            .and_then(pointer_unescape)
        else {
            self.refusals.push(Refusal::new(
                format!("{pointer}/$ref"),
                "only local component-schema references are supported",
            ));
            return None;
        };
        self.references.insert(UnresolvedReference {
            pointer: format!("{pointer}/$ref"),
            target: name.clone(),
        });
        Some(InterfaceSchema::Reference { name })
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

    fn note_keywords(
        &mut self,
        object: &Map<String, Value>,
        consumed: &[&str],
        pointer: &str,
        context: KeywordContext,
    ) {
        for key in object.keys() {
            if !consumed.contains(&key.as_str()) {
                self.note_keyword(key, pointer, context);
            }
        }
    }

    fn note_keyword(&mut self, key: &str, pointer: &str, context: KeywordContext) {
        let pointer = format!("{pointer}/{}", pointer_escape(key));
        if context.annotation(key) {
            self.normalizations.insert(AccountingEntry {
                pointer,
                code: AccountingCode::AnnotationOmitted,
                detail: "known annotation omitted without applying a default".to_owned(),
            });
        } else {
            let schema = matches!(context, KeywordContext::Schema);
            self.gaps.insert(AccountingEntry {
                pointer,
                code: if schema && !key.starts_with("x-") {
                    AccountingCode::ConstraintUnpreserved
                } else {
                    AccountingCode::FeatureUnpreserved
                },
                detail: if schema {
                    "schema keyword not preserved by this variant"
                } else {
                    "interface feature not preserved"
                }
                .to_owned(),
            });
        }
    }

    fn note_root_gaps(&mut self, root: &Map<String, Value>) {
        self.note_keywords(
            root,
            &[
                "openapi",
                "info",
                "paths",
                "components",
                "jsonSchemaDialect",
            ],
            "",
            KeywordContext::Interface,
        );
    }

    fn check_dialect(&mut self, value: Option<&Value>, pointer: &str) -> bool {
        if value.is_some_and(|value| value.as_str() != Some(SCHEMA_DIALECT)) {
            self.refusals.push(Refusal::new(
                pointer,
                format!("only schema dialect `{SCHEMA_DIALECT}` is supported"),
            ));
            false
        } else {
            true
        }
    }

    fn optional_string(
        &mut self,
        object: &Map<String, Value>,
        key: &str,
        pointer: &str,
    ) -> Option<String> {
        let value = object.get(key)?;
        if let Some(value) = value.as_str() {
            Some(value.to_owned())
        } else {
            self.refusals.push(Refusal::new(
                format!("{pointer}/{}", pointer_escape(key)),
                "expected a string",
            ));
            None
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
        let projected = project(imported.interface()).expect("typed IR projects");
        let reimported = import(&projected).expect("projection imports");
        assert_eq!(reimported.interface(), imported.interface());
        assert_eq!(project(reimported.interface()), Ok(projected));
    }

    #[test]
    fn unresolved_local_references_are_reported_without_being_guessed() {
        let source = SOURCE.replace(
            "#/components/schemas/Invoice'",
            "#/components/schemas/Missing'",
        );
        let imported = import(&source).expect("a dangling local reference remains representable");
        assert_eq!(
            imported
                .accounting()
                .unresolved_references
                .iter()
                .map(|reference| reference.target.as_str())
                .collect::<Vec<_>>(),
            vec!["Missing"]
        );
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
        let imported = import(SOURCE).expect("source imports");
        let interface = imported.interface();
        assert_eq!(project(interface), project(interface));
        assert_eq!(interface.to_canonical_json(), interface.to_canonical_json());
    }

    fn at_site(schema: Value, site: usize) -> (String, String) {
        let mut document = serde_json::json!({
            "openapi": "3.1.0", "info": {"title": "test", "version": "v1"},
            "paths": {"/items": {"post": {"operationId": "create", "responses": {"200": {"description": "ok"}}}}},
            "components": {"schemas": {}}
        });
        let pointer = match site {
            0 => {
                document["components"]["schemas"]["A~/"] = schema;
                "/components/schemas/A~0~1"
            }
            1 => {
                document["components"]["schemas"]["A~/"] = serde_json::json!({
                    "type": "object", "additionalProperties": false,
                    "properties": {"p~/": schema}
                });
                "/components/schemas/A~0~1/properties/p~0~1"
            }
            2 => {
                document["paths"]["/items"]["post"]["requestBody"] = serde_json::json!({
                    "content": {"application/json": {"schema": schema}}
                });
                "/paths/~1items/post/requestBody/content/application~1json/schema"
            }
            3 => {
                document["paths"]["/items"]["post"]["responses"]["200"]["content"] =
                    serde_json::json!({"application/json": {"schema": schema}});
                "/paths/~1items/post/responses/200/content/application~1json/schema"
            }
            _ => unreachable!(),
        };
        (document.to_string(), pointer.to_owned())
    }

    #[test]
    fn review_nonstring_enum_and_const_are_accounted_at_every_site() {
        for schema in [
            serde_json::json!({"type": "integer", "enum": [1, 2], "const": 1}),
            serde_json::json!({"type": "number", "enum": [1.5], "const": 1.5}),
            serde_json::json!({"type": "boolean", "enum": [true], "const": true}),
            serde_json::json!({"type": "array", "items": {"type": "string"}, "enum": [[]], "const": []}),
            serde_json::json!({"type": "object", "properties": {}, "additionalProperties": false, "enum": [{}], "const": {}}),
        ] {
            for site in 0..4 {
                let (source, pointer) = at_site(schema.clone(), site);
                let imported = import(&source).expect("partial typed shape remains representable");
                for keyword in ["enum", "const"] {
                    assert!(
                        imported
                            .accounting()
                            .coverage_gaps
                            .iter()
                            .any(|gap| gap.pointer == format!("{pointer}/{keyword}")),
                        "lost {pointer}/{keyword}: {imported:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn review_reference_sibling_constraints_are_accounted_at_every_site() {
        for site in 0..4 {
            let (source, pointer) = at_site(
                serde_json::json!({
                    "$ref": "#/components/schemas/Missing", "enum": [1], "minimum": 1
                }),
                site,
            );
            let imported = import(&source).expect("local reference remains representable");
            for keyword in ["enum", "minimum"] {
                assert!(
                    imported
                        .accounting()
                        .coverage_gaps
                        .iter()
                        .any(|gap| gap.pointer == format!("{pointer}/{keyword}")),
                    "lost reference sibling {pointer}/{keyword}"
                );
            }
        }
    }

    fn refused_at_every_site(schema: &Value, suffix: &str) {
        for site in 0..4 {
            let (source, pointer) = at_site(schema.clone(), site);
            let refusals = import(&source).expect_err("unsupported schema must not disappear");
            assert!(
                refusals
                    .iter()
                    .any(|refusal| refusal.pointer == format!("{pointer}{suffix}")),
                "{refusals:?}"
            );
        }
    }

    #[test]
    fn review_missing_array_items_refuses_at_every_site() {
        refused_at_every_site(&serde_json::json!({"type": "array"}), "/items");
    }

    #[test]
    fn review_empty_enum_refuses_at_every_site() {
        refused_at_every_site(&serde_json::json!({"type": "string", "enum": []}), "/enum");
    }

    #[test]
    fn review_type_array_is_not_an_absent_type() {
        refused_at_every_site(
            &serde_json::json!({"type": ["string", "null"], "enum": ["a"]}),
            "/type",
        );
    }

    #[test]
    fn review_schema_dialect_override_refuses_at_every_site() {
        refused_at_every_site(
            &serde_json::json!({"type": "string", "$schema": "https://json-schema.org/draft/2020-12/schema"}),
            "/$schema",
        );
    }

    #[test]
    fn review_document_dialect_is_checked_before_schema_interpretation() {
        let (source, _) = at_site(serde_json::json!({"type": "string"}), 0);
        let mut value: Value = serde_json::from_str(&source).unwrap();
        value["jsonSchemaDialect"] = Value::String("https://example.invalid/dialect".to_owned());
        let refusals = import(&value.to_string()).expect_err("unknown dialect refuses");
        assert!(refusals
            .iter()
            .any(|refusal| refusal.pointer == "/jsonSchemaDialect"));
    }

    #[test]
    fn review_version_prefix_is_not_a_valid_patch_version() {
        for version in ["3.1.", "3.1.wrong", "3.1.0.extra", "3.1.01"] {
            let refusals = import(&SOURCE.replace("3.1.0", version))
                .expect_err("invalid patch version refuses");
            assert!(refusals.iter().any(|refusal| refusal.pointer == "/openapi"));
        }
    }

    #[test]
    fn review_unsupported_reference_fragments_are_not_dangling_names() {
        for reference in [
            "#/components/schemas/A%20B",
            "#/components/schemas/A/properties/p",
        ] {
            refused_at_every_site(&serde_json::json!({"$ref": reference}), "/$ref");
        }
    }

    #[test]
    fn review_invalid_optional_string_constraints_refuse() {
        for keyword in ["pattern", "format"] {
            let mut schema = serde_json::json!({"type": "string"});
            schema[keyword] = Value::Bool(false);
            refused_at_every_site(&schema, &format!("/{keyword}"));
        }
    }

    #[test]
    fn review_string_enum_and_const_are_both_retained() {
        for site in 0..4 {
            let (source, _) = at_site(
                serde_json::json!({"type": "string", "enum": ["a", "b"], "const": "a"}),
                site,
            );
            let imported = import(&source).unwrap();
            let projected = project(imported.interface()).unwrap();
            let reimported = import(&projected).unwrap();
            assert_eq!(imported.interface(), reimported.interface());
            assert!(projected.contains("const: a"));
            assert!(projected.contains("enum:"));
        }
    }

    #[test]
    fn review_annotation_names_only_normalize_at_admitted_positions() {
        let (source, _) = at_site(serde_json::json!({"type": "string"}), 0);
        let mut value: Value = serde_json::from_str(&source).unwrap();
        value["default"] = serde_json::json!({"$schema": "literal, not a schema"});
        value["info"]["examples"] = serde_json::json!(["not an info annotation"]);
        let imported = import(&value.to_string()).unwrap();
        for pointer in ["/default", "/info/examples"] {
            assert!(
                imported
                    .accounting()
                    .coverage_gaps
                    .iter()
                    .any(|gap| gap.pointer == pointer),
                "unsupported-position feature lost: {pointer}"
            );
        }
    }

    #[test]
    fn review_schema_annotation_literals_are_not_walked_as_schemas() {
        let (source, _) = at_site(
            serde_json::json!({
                "type": "string", "description": "text", "default": {"$ref": "https://example.invalid/literal", "$schema": "literal"},
                "readOnly": true, "writeOnly": true, "x-ess-invariants": ["kept as a gap"], "x-other": true
            }),
            0,
        );
        let imported = import(&source).unwrap();
        assert_eq!(imported.accounting().coverage_gaps.len(), 4);
        assert!(imported
            .accounting()
            .normalizations
            .iter()
            .any(|entry| entry.pointer.ends_with("/default")));
        assert!(project_import(&imported).is_err());
    }
}
