//! Root-selected, accounted data-type realization from a qualified structural import.

mod diagnostic;
mod go;
mod native;
pub mod normalize;
mod rust;
mod ts;

use std::collections::{BTreeMap, BTreeSet};

use fluent_uri::IriRef;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::bundle::Bundle;
use diagnostic::{finding, path};
pub use diagnostic::{Finding, Refused};

/// Provenance and explicit target boundaries, not a claim of decoder conformance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    format: &'static str,
    source_digest: String,
    input: InputIdentity,
    generator_version: &'static str,
    target: &'static str,
    configuration: TargetConfiguration,
    roots: BTreeSet<String>,
    declarations: BTreeMap<String, String>,
    annotations: BTreeSet<Finding>,
    obligations: BTreeSet<Finding>,
}

/// Distinct provenance for qualified source imports and resolved model projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum InputIdentity {
    Bundle {
        bundle_digest: String,
    },
    Model {
        system: String,
        specification_version: String,
        contract_digest: String,
        projection_digest: String,
    },
}

/// Typed target identity and options recorded beside source provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "language", rename_all = "snake_case")]
pub enum TargetConfiguration {
    /// A declaration module consumed with strict optional-property semantics.
    Typescript,
    /// A standalone Cargo data library.
    Rust {
        /// Explicit Cargo package identity, independent of source type names.
        package: String,
    },
    /// A standalone Go module using only standard-library wire codecs.
    Go {
        /// Go package name.
        package: String,
        /// Explicit Go module identity.
        module: String,
    },
}

impl TargetConfiguration {
    fn name(&self) -> &'static str {
        match self {
            Self::Typescript => "typescript",
            Self::Rust { .. } => "rust",
            Self::Go { .. } => "go",
        }
    }
}

/// A complete target result; the source bundle remains the validation authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Realization {
    /// Native declarations, without a generated application or transport layer.
    pub declarations: String,
    /// Exact source identity, selected closure and source-located target accounting.
    pub report: Report,
    /// Native build metadata and helper files; never an application or deployment.
    pub supporting: BTreeMap<String, String>,
}

/// Sealed, language-neutral shape plan from a checked bundle or resolved model selection.
#[derive(Debug, Clone)]
pub struct Plan {
    definitions: BTreeMap<String, Node>,
    names: BTreeMap<String, String>,
    roots: BTreeSet<String>,
    source_digest: String,
    input: InputIdentity,
    newtypes: BTreeSet<String>,
    annotations: BTreeSet<Finding>,
    obligations: BTreeSet<Finding>,
}

#[derive(Debug, Clone)]
struct Node {
    pointer: String,
    shape: Shape,
}

#[derive(Debug, Clone)]
enum Shape {
    Json,
    Never,
    Null,
    Boolean,
    String,
    Number,
    Integer,
    Literal(Value),
    Ref(String),
    Object {
        fields: BTreeMap<String, Field>,
        additional: Box<Node>,
    },
    Array {
        prefix: Vec<Node>,
        items: Box<Node>,
        minimum: u64,
        maximum: Option<u64>,
    },
    Union {
        mode: UnionMode,
        variants: Vec<Node>,
    },
    Intersection(Vec<Node>),
}

#[derive(Debug, Clone, Copy)]
enum UnionMode {
    AnyOf,
    OneOf,
    TypeSet,
    Enum,
}

#[derive(Debug, Clone)]
struct Field {
    required: bool,
    value: Node,
}

#[derive(Default)]
struct Builder {
    model: bool,
    source_pointers: BTreeMap<String, String>,
    errors: BTreeSet<Finding>,
    annotations: BTreeSet<Finding>,
    obligations: BTreeSet<Finding>,
}

impl Plan {
    /// Select admitted roots and their complete closure before any target is emitted.
    pub fn from_bundle(bundle: &Bundle, roots: &BTreeSet<String>) -> Result<Self, Refused> {
        let mut builder = Builder {
            source_pointers: bundle
                .definitions()
                .keys()
                .map(|name| (name.clone(), bundle.source_pointer(name)))
                .collect(),
            ..Builder::default()
        };
        if roots.is_empty() {
            builder
                .errors
                .insert(finding("/roots", "empty_roots", "select at least one root"));
        }
        let mut sources = BTreeMap::new();
        for root in roots {
            match bundle.root_definitions(root) {
                Ok(closure) => sources.extend(closure),
                Err(error) => {
                    builder
                        .errors
                        .insert(finding("/roots", "unselected_root", &error.to_string()));
                }
            }
        }
        let bytes = bundle
            .to_json()
            .map_err(|error| Refused(vec![finding("/", "bundle_encoding", &error.to_string())]))?;
        Self::build(
            builder,
            sources,
            roots,
            bundle.source_digest().to_owned(),
            InputIdentity::Bundle {
                bundle_digest: crate::bundle::source_digest(&bytes),
            },
            BTreeSet::new(),
        )
    }

    /// Use the sealed selection of a checked ESS model and its existing schema wire mapping.
    pub fn from_model(model: &ess_gen::schema::ModelTypes) -> Result<Self, Refused> {
        let provenance = model.provenance();
        Self::build(
            Builder {
                model: true,
                ..Builder::default()
            },
            model
                .definitions()
                .iter()
                .map(|(name, value)| (name.as_str(), value))
                .collect(),
            model.roots(),
            provenance.source_digest.clone(),
            InputIdentity::Model {
                system: provenance.system.clone(),
                specification_version: provenance.specification_version.clone(),
                contract_digest: provenance.contract_digest.clone(),
                projection_digest: crate::bundle::source_digest(&model.to_json()),
            },
            model.newtypes().clone(),
        )
    }

    fn build(
        mut builder: Builder,
        sources: BTreeMap<&str, &Value>,
        roots: &BTreeSet<String>,
        source_digest: String,
        input: InputIdentity,
        newtypes: BTreeSet<String>,
    ) -> Result<Self, Refused> {
        let prefix = if builder.model {
            "/$defs"
        } else {
            "/components/schemas"
        };
        let mut names = BTreeMap::new();
        let mut reverse = BTreeMap::new();
        let mut definitions = BTreeMap::new();
        for (name, schema) in sources {
            let pointer = builder
                .source_pointers
                .get(name)
                .cloned()
                .unwrap_or_else(|| path(prefix, name));
            let projected = declaration_name(name);
            if projected.is_empty()
                || projected.as_bytes()[0].is_ascii_digit()
                || ["Self", "EssJsonValue", "EssPresence", "EssNullable"]
                    .contains(&projected.as_str())
            {
                builder.errors.insert(finding(
                    &pointer,
                    "invalid_name",
                    "component cannot become a shared declaration name",
                ));
            }
            if let Some(previous) = reverse.insert(projected.clone(), name.to_owned()) {
                builder.errors.insert(finding(
                    &pointer,
                    "name_collision",
                    &format!("{previous:?} and {name:?} both map to {projected}"),
                ));
            }
            names.insert(name.to_owned(), projected);
            definitions.insert(name.to_owned(), builder.node(schema, &pointer));
        }
        for node in definitions.values() {
            check_references(node, &names, &mut builder.errors);
        }
        let graph = definitions
            .iter()
            .map(|(name, node)| {
                let mut references = BTreeSet::new();
                unguarded_references(node, &mut references);
                (name.clone(), references)
            })
            .collect::<BTreeMap<_, _>>();
        for name in graph.keys() {
            if reaches(name, name, &graph, &mut BTreeSet::new()) {
                builder.errors.insert(finding(
                    &definitions[name].pointer,
                    "unguarded_recursion",
                    "recursive aliases need an object or array boundary",
                ));
            }
        }
        if !builder.errors.is_empty() {
            return Err(Refused(builder.errors.into_iter().collect()));
        }
        Ok(Self {
            definitions,
            names,
            roots: roots.clone(),
            source_digest,
            input,
            newtypes,
            annotations: builder.annotations,
            obligations: builder.obligations,
        })
    }

    /// Component identities and their shared declaration names, in deterministic order.
    pub fn declarations(&self) -> &BTreeMap<String, String> {
        &self.names
    }

    /// Emit accounted structural TypeScript, with no implicit validation or coercion.
    pub fn typescript(&self) -> Realization {
        ts::emit(self)
    }

    /// Emit a standalone Rust data library with explicit presence and wire serialization.
    pub fn rust(&self, package: &str) -> Result<Realization, Refused> {
        rust::emit(self, package)
    }

    /// Emit a standalone Go data library preserving requiredness and nullable values.
    pub fn go(&self, package: &str, module: &str) -> Result<Realization, Refused> {
        go::emit(self, package, module)
    }

    fn report(&self, configuration: TargetConfiguration) -> Report {
        let mut obligations = self.obligations.clone();
        if matches!(configuration, TargetConfiguration::Typescript) {
            for name in &self.newtypes {
                obligations.insert(finding(&path("/$defs", name), "typescript_nominal_identity", "structural TypeScript aliases do not enforce distinct model newtype identities"));
            }
        }
        Report {
            format: "ess-types-report/3",
            source_digest: self.source_digest.clone(),
            input: self.input.clone(),
            generator_version: env!("CARGO_PKG_VERSION"),
            target: configuration.name(),
            configuration,
            roots: self.roots.clone(),
            declarations: self.names.clone(),
            annotations: self.annotations.clone(),
            obligations,
        }
    }

    fn input_comment(&self) -> String {
        match &self.input {
            InputIdentity::Bundle { bundle_digest } => {
                format!("Qualified bundle SHA-256: {bundle_digest}")
            }
            InputIdentity::Model {
                projection_digest, ..
            } => format!("Model schema projection SHA-256: {projection_digest}"),
        }
    }
}

impl Builder {
    fn node(&mut self, schema: &Value, pointer: &str) -> Node {
        let shape = match schema {
            Value::Bool(true) => Shape::Json,
            Value::Bool(false) => Shape::Never,
            Value::Object(object) => self.object_schema(object, pointer),
            _ => {
                self.errors.insert(finding(
                    pointer,
                    "invalid_schema",
                    "expected a checked schema",
                ));
                Shape::Never
            }
        };
        Node {
            pointer: pointer.to_owned(),
            shape,
        }
    }

    fn classify_keywords(&mut self, object: &Map<String, Value>, pointer: &str) {
        if !object.contains_key("type") {
            for keyword in [
                "properties",
                "required",
                "additionalProperties",
                "items",
                "prefixItems",
            ] {
                if object.contains_key(keyword) {
                    self.errors.insert(finding(&path(pointer, keyword), "conditional_shape", "without explicit type this keyword is conditional; no type may be inferred"));
                }
            }
        }
        for keyword in object.keys() {
            let location = path(pointer, keyword);
            match keyword.as_str() {
                "x-ess-name" | "x-ess-kind" | "x-ess-field" | "x-ess-map-key"
                | "x-ess-union-tag"
                    if self.model =>
                {
                    self.annotations.insert(finding(
                        &location,
                        keyword,
                        "model identity and wire mapping retained in source.schema.json",
                    ));
                }
                "x-ess-invariants" if self.model => {
                    self.obligations.insert(finding(&location, "model_invariants", "enforce the retained model invariant statements; structural codecs do not evaluate predicates"));
                }
                "propertyNames" if self.model => {
                    self.obligations.insert(finding(
                        &location,
                        "model_map_keys",
                        "validate JSON object key spellings against the retained model key schema",
                    ));
                }
                "$ref"
                | "type"
                | "const"
                | "enum"
                | "anyOf"
                | "oneOf"
                | "allOf"
                | "properties"
                | "required"
                | "additionalProperties"
                | "items"
                | "prefixItems" => {}
                "$schema" | "$comment" | "title" | "description" | "default" | "examples"
                | "deprecated" | "readOnly" | "writeOnly" | "format" | "contentEncoding"
                | "contentMediaType" => {
                    self.annotations.insert(finding(
                        &location,
                        keyword,
                        if self.model {
                            "retained in source.schema.json; no decoder behavior inferred"
                        } else {
                            "retained in the source bundle; no decoder behavior inferred"
                        },
                    ));
                }
                "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "multipleOf"
                | "minLength" | "maxLength" | "pattern" | "minItems" | "maxItems"
                | "uniqueItems" | "minProperties" | "maxProperties" => {
                    self.obligations.insert(finding(
                        &location,
                        keyword,
                        "validate this constraint against the source schema at runtime",
                    ));
                }
                _ => {
                    self.errors
                        .insert(finding(&location, "unsupported_keyword", keyword));
                }
            }
        }
    }

    fn object_schema(&mut self, object: &Map<String, Value>, pointer: &str) -> Shape {
        self.classify_keywords(object, pointer);
        let mut terms = Vec::new();
        if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
            match local_definition(reference) {
                Some(name) => terms.push(Node {
                    pointer: path(pointer, "$ref"),
                    shape: Shape::Ref(name),
                }),
                None => {
                    self.errors.insert(finding(
                        &path(pointer, "$ref"),
                        "unsupported_reference",
                        "expected an exact component root reference, not a nested schema",
                    ));
                }
            }
        }
        if let Some(value) = object.get("const") {
            terms.push(Node {
                pointer: path(pointer, "const"),
                shape: Shape::Literal(value.clone()),
            });
        }
        if let Some(values) = object.get("enum").and_then(Value::as_array) {
            terms.push(Node {
                pointer: path(pointer, "enum"),
                shape: Shape::Union {
                    mode: UnionMode::Enum,
                    variants: values
                        .iter()
                        .enumerate()
                        .map(|(index, value)| Node {
                            pointer: path(&path(pointer, "enum"), &index.to_string()),
                            shape: Shape::Literal(value.clone()),
                        })
                        .collect(),
                },
            });
        }
        for (keyword, mode) in [
            ("anyOf", UnionMode::AnyOf),
            ("oneOf", UnionMode::OneOf),
            ("allOf", UnionMode::AnyOf),
        ] {
            if let Some(values) = object.get(keyword).and_then(Value::as_array) {
                let location = path(pointer, keyword);
                let variants = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| self.node(value, &path(&location, &index.to_string())))
                    .collect();
                let shape = if keyword == "allOf" {
                    Shape::Intersection(variants)
                } else {
                    Shape::Union { mode, variants }
                };
                terms.push(Node {
                    pointer: location,
                    shape,
                });
            }
        }
        if let Some(kind) = object.get("type") {
            let shape = if let Some(kind) = kind.as_str() {
                self.kind(kind, object, pointer)
            } else {
                Shape::Union {
                    mode: UnionMode::TypeSet,
                    variants: kind
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|kind| Node {
                            pointer: pointer.to_owned(),
                            shape: self.kind(kind, object, pointer),
                        })
                        .collect(),
                }
            };
            terms.push(Node {
                pointer: pointer.to_owned(),
                shape,
            });
        }
        match terms.len() {
            0 => Shape::Json,
            1 => terms.pop().expect("one term").shape,
            _ => Shape::Intersection(terms),
        }
    }

    fn kind(&mut self, kind: &str, object: &Map<String, Value>, pointer: &str) -> Shape {
        match kind {
            "null" => Shape::Null,
            "boolean" => Shape::Boolean,
            "string" => Shape::String,
            "number" => Shape::Number,
            "integer" => Shape::Integer,
            "object" => {
                let additional = self.keyword_node(object, "additionalProperties", pointer);
                let required = object
                    .get("required")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>();
                let mut fields = object
                    .get("properties")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flatten()
                    .map(|(name, value)| {
                        (
                            name.clone(),
                            Field {
                                required: required.contains(name.as_str()),
                                value: self.node(value, &path(&path(pointer, "properties"), name)),
                            },
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                for name in required {
                    fields.entry(name.to_owned()).or_insert_with(|| Field {
                        required: true,
                        value: additional.clone(),
                    });
                }
                Shape::Object {
                    fields,
                    additional: Box::new(additional),
                }
            }
            "array" => {
                let prefix = object
                    .get("prefixItems")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, value)| {
                        self.node(
                            value,
                            &path(&path(pointer, "prefixItems"), &index.to_string()),
                        )
                    })
                    .collect();
                let items = Box::new(self.keyword_node(object, "items", pointer));
                Shape::Array {
                    prefix,
                    items,
                    minimum: self.array_bound(object, "minItems", pointer).unwrap_or(0),
                    maximum: self.array_bound(object, "maxItems", pointer),
                }
            }
            _ => {
                self.errors.insert(finding(pointer, "invalid_type", kind));
                Shape::Never
            }
        }
    }

    fn keyword_node(&mut self, object: &Map<String, Value>, keyword: &str, pointer: &str) -> Node {
        object.get(keyword).map_or_else(
            || Node {
                pointer: pointer.to_owned(),
                shape: Shape::Json,
            },
            |value| self.node(value, &path(pointer, keyword)),
        )
    }

    fn array_bound(
        &mut self,
        object: &Map<String, Value>,
        keyword: &str,
        pointer: &str,
    ) -> Option<u64> {
        let value = object.get(keyword)?;
        let bound = exact_u64(value);
        if bound.is_none() {
            self.errors.insert(finding(
                &path(pointer, keyword),
                "array_bound_range",
                "array cardinality exceeds the supported unsigned 64-bit range",
            ));
        }
        bound
    }
}

// The range and fractional checks make the floating-token conversion exact, including `3.0`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn exact_u64(value: &Value) -> Option<u64> {
    value.as_u64().or_else(|| {
        let number = value.as_f64()?;
        if (0.0..18_446_744_073_709_551_616.0).contains(&number) && number.fract() == 0.0 {
            Some(number as u64)
        } else {
            None
        }
    })
}

fn check_references(node: &Node, names: &BTreeMap<String, String>, errors: &mut BTreeSet<Finding>) {
    if let Shape::Ref(name) = &node.shape {
        if !names.contains_key(name) {
            errors.insert(finding(&node.pointer, "unresolved_reference", name));
        }
    }
    for child in children(node) {
        check_references(child, names, errors);
    }
}

fn children(node: &Node) -> Vec<&Node> {
    match &node.shape {
        Shape::Object { fields, additional } => fields
            .values()
            .map(|field| &field.value)
            .chain(std::iter::once(additional.as_ref()))
            .collect(),
        Shape::Array { prefix, items, .. } => prefix
            .iter()
            .chain(std::iter::once(items.as_ref()))
            .collect(),
        Shape::Union { variants, .. } | Shape::Intersection(variants) => variants.iter().collect(),
        _ => Vec::new(),
    }
}

fn unguarded_references(node: &Node, references: &mut BTreeSet<String>) {
    match &node.shape {
        Shape::Ref(name) => {
            references.insert(name.clone());
        }
        Shape::Union { variants, .. } | Shape::Intersection(variants) => {
            for variant in variants {
                unguarded_references(variant, references);
            }
        }
        _ => {}
    }
}

fn reaches(
    start: &str,
    name: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    seen: &mut BTreeSet<String>,
) -> bool {
    if !seen.insert(name.to_owned()) {
        return false;
    }
    graph.get(name).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge == start || reaches(start, edge, graph, seen))
    })
}

fn local_definition(text: &str) -> Option<String> {
    let reference = IriRef::parse(text).ok()?;
    if reference.scheme().is_some()
        || reference.authority().is_some()
        || !reference.path().is_empty()
        || reference.query().is_some()
    {
        return None;
    }
    let pointer = reference.fragment()?.decode().to_string().ok()?;
    let tail = pointer.strip_prefix("/$defs/")?;
    if tail.contains('/') {
        return None;
    }
    Some(tail.replace("~1", "/").replace("~0", "~"))
}

fn declaration_name(source: &str) -> String {
    let mut result = String::new();
    for part in source
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
    {
        let mut chars = part.chars();
        result.push(chars.next().expect("nonempty part").to_ascii_uppercase());
        result.push_str(chars.as_str());
    }
    result
}
