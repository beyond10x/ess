//! Qualified, offline extraction of component schemas from a retained source document.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

use fluent_uri::pct_enc::{encoder::Fragment, EString};
use fluent_uri::{IriRef, Uri};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// The new import-result envelope; it does not change the service-interface format.
pub const FORMAT: &str = "ess-schema-bundle/1";
/// Explicit document-root identity requires a new envelope; v1 stays byte-compatible.
pub const DOCUMENT_FORMAT: &str = "ess-schema-bundle/2";
/// The explicitly supported structural dialect, independently of the source envelope.
pub const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// The caller's selected interpretation, never inferred from inconsistent source metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dialect {
    /// JSON Schema 2020-12, with format annotations rather than format assertions.
    #[serde(rename = "json-schema-2020-12")]
    Draft202012,
}

/// Strict adapter accounting; successful bundles have no unaccounted or unresolved semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accounting {
    /// The concrete adapter, not a service-interface importer.
    pub adapter: String,
    /// Only import is claimed by this accounting record.
    pub direction: String,
    /// Declared supported structural semantics.
    pub supported: BTreeSet<String>,
    /// No partial result is admitted when a constraint cannot be represented.
    pub coverage_gaps: Vec<Diagnostic>,
    /// No decoder or application behavior is claimed by structural import.
    pub obligations: Vec<String>,
    /// Local references that could not be resolved into the selected closure.
    pub unresolved_references: usize,
    /// Every refusal observed during import.
    pub refusals: Vec<Diagnostic>,
}

impl Accounting {
    fn complete() -> Self {
        Self {
            adapter: "ess.schema-component-bundle".to_owned(),
            direction: "import".to_owned(),
            supported: [
                "json-schema-2020-12-structural-subset",
                "component-reference-closure",
                "source-retention",
                "qualified-reload",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            coverage_gaps: Vec::new(),
            obligations: Vec::new(),
            unresolved_references: 0,
            refusals: Vec::new(),
        }
    }
}

/// A source-located import refusal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    /// Whether a reference failed to resolve; other refusals still carry their source pointer.
    pub unresolved_reference: bool,
    /// JSON Pointer in the original source, or the persisted envelope on reload failure.
    pub pointer: String,
    /// Why no complete import can be admitted.
    pub message: String,
}

/// All refusals observed before any result is admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError(pub Vec<Diagnostic>);

impl ImportError {
    /// Complete adapter accounting for this refused import, without a partial successful bundle.
    pub fn accounting(&self) -> Accounting {
        Accounting {
            unresolved_references: self
                .0
                .iter()
                .filter(|diagnostic| diagnostic.unresolved_reference)
                .count(),
            refusals: self.0.clone(),
            ..Accounting::complete()
        }
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.0 {
            writeln!(formatter, "{}: {}", diagnostic.pointer, diagnostic.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ImportError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireBundle {
    format: String,
    source: String,
    source_digest: String,
    declared_dialect: Option<String>,
    selected_dialect: Dialect,
    accounting: Accounting,
    roots: BTreeSet<String>,
    definitions: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    document_root: Option<String>,
}

/// A checked import. Construction and persisted reload both validate the complete source closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    wire: WireBundle,
    dependencies: BTreeMap<String, BTreeSet<String>>,
}

impl Bundle {
    /// Revalidated strict adapter accounting, retained by the persisted envelope.
    pub fn accounting(&self) -> &Accounting {
        &self.wire.accounting
    }

    /// Exact digest of the retained source bytes.
    pub fn source_digest(&self) -> &str {
        &self.wire.source_digest
    }

    /// The source envelope's claim, which may differ from the explicitly selected dialect.
    pub fn declared_dialect(&self) -> Option<&str> {
        self.wire.declared_dialect.as_deref()
    }

    /// The explicitly selected component roots, not a fabricated union of their meanings.
    pub fn roots(&self) -> &BTreeSet<String> {
        &self.wire.roots
    }

    /// The complete, transformed transitive reference closure.
    pub fn definitions(&self) -> &BTreeMap<String, Value> {
        &self.wire.definitions
    }

    /// Original source location of a retained definition, including the document root.
    pub fn source_pointer(&self, name: &str) -> String {
        definition_pointer(self.wire.document_root.as_deref(), name)
    }

    /// The checked closure of one explicitly selected root, without unrelated components.
    pub fn root_definitions(&self, root: &str) -> Result<BTreeMap<&str, &Value>, ImportError> {
        if !self.wire.roots.contains(root) {
            return Err(failure("/roots", "root was not selected by the import"));
        }
        self.root_closure(root)
    }

    /// Deterministic persisted bytes, including source qualification and original source bytes.
    pub fn to_json(&self) -> Result<String, ImportError> {
        serde_json::to_string_pretty(&self.wire)
            .map(|text| format!("{text}\n"))
            .map_err(|error| failure("/", error.to_string()))
    }

    /// Reloads by reimporting the retained source and checking the whole persisted result.
    pub fn read(text: &str) -> Result<Self, ImportError> {
        let document: Value =
            serde_json::from_str(text).map_err(|error| failure("/", error.to_string()))?;
        if document.get("format").and_then(Value::as_str) == Some(FORMAT)
            && document.get("document_root").is_some()
        {
            return Err(failure(
                "/document_root",
                "v1 cannot carry document-root identity",
            ));
        }
        let wire: WireBundle =
            serde_json::from_value(document).map_err(|error| failure("/", error.to_string()))?;
        let expected = match (wire.format.as_str(), wire.document_root.as_deref()) {
            (FORMAT, None) => import(&wire.source, &wire.roots, wire.selected_dialect)?,
            (DOCUMENT_FORMAT, Some(root)) => {
                let mut definitions = wire.roots.clone();
                definitions.remove(root);
                import_document(&wire.source, root, &definitions, wire.selected_dialect)?
            }
            _ => {
                return Err(failure(
                    "/format",
                    "unsupported schema-bundle format or document-root identity",
                ))
            }
        };
        if wire != expected.wire {
            return Err(failure(
                "/",
                "persisted import does not match its retained source and selection",
            ));
        }
        Ok(expected)
    }

    /// Projects one selected root as a standalone schema with explicit source qualification.
    pub fn schema(&self, root: &str, id: &str) -> Result<Value, ImportError> {
        if !self.wire.roots.contains(root) {
            return Err(failure("/roots", "root was not selected by the import"));
        }
        let uri = Uri::parse(id).map_err(|error| failure("/$id", error.to_string()))?;
        if uri.fragment().is_some() {
            return Err(failure("/$id", "schema id must have no fragment"));
        }
        Ok(json!({
            "$schema": SCHEMA_DIALECT,
            "$id": id,
            "$ref": fragment(&format!("/$defs/{}", escape(root))),
            "$defs": self.root_closure(root)?,
            "x-ess-source": {
                "format": FORMAT,
                "source_digest": self.wire.source_digest,
                "declared_dialect": self.wire.declared_dialect,
                "selected_dialect": self.wire.selected_dialect,
                "accounting": self.wire.accounting,
                "root": root,
                "qualification": "explicit-structural-interpretation-not-service-validation"
            }
        }))
    }

    fn root_closure(&self, root: &str) -> Result<BTreeMap<&str, &Value>, ImportError> {
        let mut pending = BTreeSet::from([root.to_owned()]);
        let mut selected = BTreeMap::new();
        while let Some(name) = pending.pop_first() {
            if selected.contains_key(name.as_str()) {
                continue;
            }
            let (name, definition) = self
                .wire
                .definitions
                .get_key_value(&name)
                .ok_or_else(|| failure("/definitions", "checked component is absent"))?;
            selected.insert(name.as_str(), definition);
            let dependencies = self
                .dependencies
                .get(name)
                .ok_or_else(|| failure("/definitions", "checked dependency index is absent"))?;
            pending.extend(dependencies.iter().cloned());
        }
        Ok(selected)
    }

    /// Validates a value against one selected root without changing the instance or applying defaults.
    pub fn validate(&self, root: &str, instance: &Value) -> Result<Vec<Diagnostic>, ImportError> {
        let schema = self.schema(root, "urn:ess:bundle:validation")?;
        let validator = compile(&schema, self.wire.document_root.as_deref())?;
        Ok(validator
            .iter_errors(instance)
            .map(|error| Diagnostic {
                unresolved_reference: false,
                pointer: error.instance_path().to_string(),
                message: error.to_string(),
            })
            .collect())
    }
}

/// Imports a complete selected component closure under an explicitly supplied structural dialect.
pub fn import(
    source: &str,
    roots: &BTreeSet<String>,
    dialect: Dialect,
) -> Result<Bundle, ImportError> {
    import_source(source, roots, dialect, None)
}

/// Retains a JSON Schema document root and its selected local definition closure.
pub fn import_document(
    source: &str,
    root: &str,
    definitions: &BTreeSet<String>,
    dialect: Dialect,
) -> Result<Bundle, ImportError> {
    if root.is_empty() {
        return Err(failure("/root", "document root name must not be empty"));
    }
    let mut roots = definitions.clone();
    roots.insert(root.to_owned());
    import_source(source, &roots, dialect, Some(root))
}

fn import_source(
    source: &str,
    roots: &BTreeSet<String>,
    dialect: Dialect,
    document_root: Option<&str>,
) -> Result<Bundle, ImportError> {
    let document: Value =
        serde_json::from_str(source).map_err(|error| failure("/", error.to_string()))?;
    let references = ReferenceSource {
        document: &document,
        root: document_root,
    };
    let empty = serde_json::Map::new();
    let components = references.definitions()?.unwrap_or(&empty);
    if roots.is_empty() {
        return Err(failure("/roots", "select at least one component root"));
    }
    let dialect_key = if document_root.is_some() {
        "$schema"
    } else {
        "openapi"
    };
    let declared_dialect = declared_dialect(&document, dialect_key)?;
    let mut pending = roots.clone();
    let mut seen = BTreeSet::new();
    let mut definitions = BTreeMap::new();
    let mut dependencies = BTreeMap::new();
    let mut errors = Vec::new();
    while let Some(name) = pending.pop_first() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let path = references.pointer(&name);
        let schema = if document_root == Some(name.as_str()) {
            Some(&document)
        } else {
            components.get(&name)
        };
        let Some(schema) = schema else {
            errors.push(diagnostic(
                &path,
                "selected or referenced component is absent",
            ));
            continue;
        };
        let mut transformed = schema.clone();
        if document_root == Some(name.as_str()) {
            if let Some(object) = transformed.as_object_mut() {
                object.remove("$defs");
            }
        }
        let mut required = BTreeSet::new();
        transform(
            &mut transformed,
            &path,
            &references,
            &mut required,
            &mut errors,
        );
        pending.extend(required.iter().cloned());
        dependencies.insert(name.clone(), required);
        definitions.insert(name, transformed);
    }
    if !errors.is_empty() {
        return Err(ImportError(errors));
    }
    let mut accounting = Accounting::complete();
    if document_root.is_some() {
        accounting
            .supported
            .insert("document-root-retention".to_owned());
    }
    let result = Bundle {
        wire: WireBundle {
            format: if document_root.is_some() {
                DOCUMENT_FORMAT
            } else {
                FORMAT
            }
            .to_owned(),
            source: source.to_owned(),
            source_digest: source_digest(source),
            declared_dialect,
            selected_dialect: dialect,
            accounting,
            roots: roots.clone(),
            definitions,
            document_root: document_root.map(str::to_owned),
        },
        dependencies,
    };
    for root in roots {
        let schema = result.schema(root, "urn:ess:bundle:validation")?;
        if let Err(error) = compile(&schema, document_root) {
            errors.extend(error.0);
        }
    }
    if !errors.is_empty() {
        return Err(ImportError(errors));
    }
    Ok(result)
}

fn declared_dialect(document: &Value, key: &str) -> Result<Option<String>, ImportError> {
    match document.get(key) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        None => Ok(None),
        Some(_) => Err(failure(
            &format!("/{key}"),
            "declared envelope dialect must be a string",
        )),
    }
}

fn compile(
    schema: &Value,
    document_root: Option<&str>,
) -> Result<jsonschema::Validator, ImportError> {
    jsonschema::meta::validate(schema).map_err(|error| {
        failure(
            &source_error_pointer(&error.instance_path().to_string(), document_root),
            error.to_string(),
        )
    })?;
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(false)
        .build(schema)
        .map_err(|error| {
            failure(
                if document_root.is_some() {
                    "/$defs"
                } else {
                    "/components/schemas"
                },
                error.to_string(),
            )
        })
}

struct ReferenceSource<'a> {
    document: &'a Value,
    root: Option<&'a str>,
}
impl ReferenceSource<'_> {
    fn definitions(&self) -> Result<Option<&serde_json::Map<String, Value>>, ImportError> {
        match self.document.pointer(self.prefix()) {
            Some(Value::Object(entries)) => {
                if self.root.is_some_and(|root| entries.contains_key(root)) {
                    return Err(failure(
                        "/root",
                        "document root name collides with a source definition",
                    ));
                }
                Ok(Some(entries))
            }
            None if self.root.is_some() => Ok(None),
            _ => Err(failure(self.prefix(), "expected a named schema collection")),
        }
    }

    fn prefix(&self) -> &'static str {
        if self.root.is_some() {
            "/$defs"
        } else {
            "/components/schemas"
        }
    }
    fn pointer(&self, name: &str) -> String {
        definition_pointer(self.root, name)
    }
}

fn definition_pointer(root: Option<&str>, name: &str) -> String {
    if root == Some(name) {
        String::new()
    } else {
        format!(
            "{}/{}",
            if root.is_some() {
                "/$defs"
            } else {
                "/components/schemas"
            },
            escape(name)
        )
    }
}

fn source_error_pointer(pointer: &str, root: Option<&str>) -> String {
    if let Some(root) = root {
        let prefix = format!("/$defs/{}", escape(root));
        if let Some(tail) = pointer.strip_prefix(&prefix) {
            if tail.is_empty() || tail.starts_with('/') {
                return tail.to_owned();
            }
        }
        pointer.to_owned()
    } else {
        pointer.replacen("/$defs/", "/components/schemas/", 1)
    }
}

fn transform(
    schema: &mut Value,
    path: &str,
    source: &ReferenceSource<'_>,
    pending: &mut BTreeSet<String>,
    errors: &mut Vec<Diagnostic>,
) {
    if schema.is_boolean() {
        return;
    }
    let Some(object) = schema.as_object_mut() else {
        errors.push(diagnostic(path, "schema must be an object or boolean"));
        return;
    };
    for (keyword, value) in object {
        let location = format!("{path}/{}", escape(keyword));
        match keyword.as_str() {
            "$ref" => match component_reference(value, source) {
                Ok((name, rewritten)) => {
                    pending.insert(name);
                    *value = Value::String(rewritten);
                }
                Err(message) => errors.push(Diagnostic {
                    unresolved_reference: true,
                    ..diagnostic(&location, message)
                }),
            },
            "$schema" => {
                if value.as_str() != Some(SCHEMA_DIALECT) {
                    errors.push(diagnostic(
                        &location,
                        "embedded dialect contradicts the selected structural dialect",
                    ));
                }
            }
            "$defs" | "definitions" | "properties" | "patternProperties" | "dependentSchemas" => {
                if let Some(entries) = value.as_object_mut() {
                    for (name, nested) in entries {
                        transform(
                            nested,
                            &format!("{location}/{}", escape(name)),
                            source,
                            pending,
                            errors,
                        );
                    }
                } else {
                    errors.push(diagnostic(&location, "expected a map of schemas"));
                }
            }
            "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                if let Some(entries) = value.as_array_mut() {
                    for (index, nested) in entries.iter_mut().enumerate() {
                        transform(
                            nested,
                            &format!("{location}/{index}"),
                            source,
                            pending,
                            errors,
                        );
                    }
                } else {
                    errors.push(diagnostic(&location, "expected an array of schemas"));
                }
            }
            "items"
            | "additionalProperties"
            | "contains"
            | "not"
            | "if"
            | "then"
            | "else"
            | "unevaluatedItems"
            | "unevaluatedProperties"
            | "propertyNames"
            | "contentSchema" => {
                transform(value, &location, source, pending, errors);
            }
            "$comment" | "title" | "description" | "default" | "examples" | "deprecated"
            | "readOnly" | "writeOnly" | "format" | "contentEncoding" | "contentMediaType"
            | "type" | "enum" | "const" | "required" | "dependentRequired" | "minimum"
            | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "multipleOf" | "minLength"
            | "maxLength" | "pattern" | "minItems" | "maxItems" | "uniqueItems" | "minContains"
            | "maxContains" | "minProperties" | "maxProperties" => {}
            other => errors.push(diagnostic(
                &location,
                format!("unsupported schema keyword `{other}`"),
            )),
        }
    }
}

fn component_reference(
    value: &Value,
    source: &ReferenceSource<'_>,
) -> Result<(String, String), String> {
    let text = value.as_str().ok_or("reference must be a string")?;
    let reference = IriRef::parse(text).map_err(|error| error.to_string())?;
    if reference.scheme().is_some()
        || reference.authority().is_some()
        || !reference.path().is_empty()
        || reference.query().is_some()
    {
        return Err("only local component references are supported".to_owned());
    }
    let pointer = reference
        .fragment()
        .ok_or("expected a component JSON Pointer fragment")?
        .decode()
        .to_string()
        .map_err(|_| "reference fragment is not UTF-8")?;
    if pointer.is_empty() {
        if let Some(root) = source.root {
            return Ok((
                root.to_owned(),
                fragment(&format!("/$defs/{}", escape(root))),
            ));
        }
    }
    let tail = pointer
        .strip_prefix(&format!("{}/", source.prefix()))
        .ok_or("reference must be within the selected local definition collection")?;
    let target = source
        .document
        .pointer(&pointer)
        .ok_or("referenced source pointer is absent")?;
    if !target.is_object() && !target.is_boolean() {
        return Err("reference target is not a schema".to_owned());
    }
    let name = tail
        .split('/')
        .next()
        .ok_or("reference has no component name")?
        .replace("~1", "/")
        .replace("~0", "~");
    Ok((name, fragment(&format!("/$defs/{tail}"))))
}

fn fragment(pointer: &str) -> String {
    let mut encoded = EString::<Fragment>::new();
    encoded.encode_str::<Fragment>(pointer);
    format!("#{encoded}")
}

fn escape(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
fn diagnostic(pointer: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        unresolved_reference: false,
        pointer: pointer.to_owned(),
        message: message.into(),
    }
}
fn failure(pointer: &str, message: impl Into<String>) -> ImportError {
    ImportError(vec![diagnostic(pointer, message)])
}

pub(crate) fn source_digest(source: &str) -> String {
    let mut output = String::with_capacity(64);
    for byte in &Sha256::digest(source.as_bytes()) {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
