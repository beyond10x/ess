//! The `ess-history/1` Rust type and JSON Schema, held to `models/concurrent-history/`.
//!
//! The model declares the history document; `crates/verify/ess-conformance/src/history.rs` is the
//! hand-written strict reader and `schemas/ess-history.schema.json` the schema the Go and
//! TypeScript writers validate against. No generator links the three, so this file does: each
//! fails when a field, a field type or an enum value the model declares is missing, or when it
//! carries one the model does not declare.
//!
//! How the model maps onto a document, derived from the model rather than listed here:
//!
//! - an entity `concurrent.history.E` is `struct E` (what a caller holds and a writer serializes),
//!   the private `struct EWire` (what the reader deserializes) and a schema object: its identity,
//!   its fields and one field per relation, named for the relation;
//! - an `owns` relation with `via: X` nests the target inside its owner, so the target's field `X`
//!   is carried by the nesting and is not a field of the target;
//! - `Integer` is `u64`, `String` is `String`, `Uuid` is `Uuid`, `concurrent.history.T` is `T`,
//!   `Optional<T>` is `Option<T>`, `List<T>` is `Vec<T>` and a `many` relation is `Vec<Target>`; an
//!   optional field is not `required` in the schema; in the schema `Integer` is `type: integer`,
//!   `String` is `type: string`, a declared type or `Uuid` is a `$ref` to its definition,
//!   `Optional<T>` adds `null`, and a `List<T>` or a relation is an `array` of its items;
//! - `Timestamp` is `DecisionInstant`, a `$ref` to its definition; `Optional<Timestamp>` (an
//!   operation's `decision_time`, `ess-history/2`) is absent or an instant and never `null`, so its
//!   schema property adds no `null`;
//! - an enum `concurrent.history.T` is `enum T`, compared by wire name;
//! - a newtype `concurrent.history.T` is a `struct T` or a `pub use …::T`.
//!
//! A wire name is the field's identifier, or its `#[serde(rename = "…")]`, and every
//! `#[serde(alias = "…")]` is one more wire name the reader admits — so an alias the model does not
//! declare is drift. Every other serde key on a scanned type (`rename_all`, split renaming, `skip`,
//! `default`, `other`, `flatten`, …) and any `cfg_attr` carrying serde changes the wire in a way
//! this scan does not model, so it is refused outright; [`ALLOWED_KEYS`] lists what is admitted.
//!
//! The `SpecDigest` newtype declares its alphabet and length in the model; those are held to
//! `ess_primitives::evidence::SpecDigest` and to the schema's pattern. Every integer is bounded by
//! the reader's `MAX_INTEGER`, and the schema's `maximum` is held to it.
//!
//! Comparison is by set: order is not a defect in a closed set of names; a name appearing or
//! disappearing is. The scan is a function from source text to a list of problems, and the
//! mutant cases at the end run it over altered copies of the real source to show it notices.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use serde_yaml::Value as Yaml;

const MODEL: &str = "models/concurrent-history/domains/history.yaml";
const SOURCE: &str = "crates/verify/ess-conformance/src/history.rs";
const SCHEMA: &str = "schemas/ess-history.schema.json";
const PREFIX: &str = "concurrent.history.";
/// The entity the document's top level is.
const ROOT: &str = "History";
/// The largest integer the reader admits (2^53 − 1, exact in every JSON reader), and the schema's
/// `maximum` for every integer. Held to `MAX_INTEGER` in [`SOURCE`] by a case below.
const MAX_INTEGER: u64 = 9_007_199_254_740_991;
/// Where `SpecDigest` and its length bounds are declared.
const DIGEST_SOURCE: &str = "crates/specify/ess-primitives/src/evidence.rs";

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.contains("[workspace]"))
        })
        .expect("a workspace manifest stands above this crate")
        .to_path_buf()
}

fn source(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

fn local(name: &str) -> String {
    name.strip_prefix(PREFIX)
        .unwrap_or_else(|| panic!("`{name}` is declared under `{PREFIX}`"))
        .to_owned()
}

fn text<'a>(value: &'a Yaml, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Yaml::as_str)
}

fn sequence<'a>(value: &'a Yaml, key: &str) -> &'a [Yaml] {
    value
        .get(key)
        .and_then(Yaml::as_sequence)
        .map_or(&[], Vec::as_slice)
}

/// One field as the document carries it: its Rust type spelling, and whether it may be absent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Field {
    rust: String,
    optional: bool,
}

/// Wire name → field.
type Fields = BTreeMap<String, Field>;

/// What the model declares, in the shape the document takes.
struct Model {
    entities: BTreeMap<String, Fields>,
    enums: BTreeMap<String, BTreeSet<String>>,
    newtypes: BTreeSet<String>,
    /// Wire names of every field the model types `Integer` (directly or optionally), per entity.
    integers: BTreeMap<String, BTreeSet<String>>,
}

/// A model type in the spelling the Rust type carries.
fn rust_spelling(model_type: &str) -> String {
    if let Some(inner) = model_type
        .strip_prefix("Optional<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return format!("Option<{}>", rust_spelling(inner));
    }
    if let Some(inner) = model_type
        .strip_prefix("List<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return format!("Vec<{}>", rust_spelling(inner));
    }
    match model_type {
        "Integer" => "u64".to_owned(),
        "String" => "String".to_owned(),
        "Uuid" => "Uuid".to_owned(),
        "Timestamp" => "DecisionInstant".to_owned(),
        other if other.starts_with(PREFIX) => local(other),
        other => panic!("the model type `{other}` has no mapping in this test"),
    }
}

fn model() -> Model {
    let document: Yaml = serde_yaml::from_str(&source(MODEL)).expect("the domain is YAML");
    let mut enums = BTreeMap::new();
    let mut newtypes = BTreeSet::new();
    for declaration in sequence(&document, "types") {
        let name = local(text(declaration, "name").expect("a type has a name"));
        match text(declaration, "kind") {
            Some("enum") => {
                let variants = sequence(declaration, "variants")
                    .iter()
                    .map(|variant| variant.as_str().expect("a variant is a name").to_owned())
                    .collect();
                enums.insert(name, variants);
            }
            Some("newtype") => {
                newtypes.insert(name);
            }
            other => panic!("`{name}` has a kind this test does not map: {other:?}"),
        }
    }

    let mut entities = BTreeMap::new();
    let mut integers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut nested: Vec<(String, String)> = Vec::new();
    for entity in sequence(&document, "entities") {
        let name = local(text(entity, "name").expect("an entity has a name"));
        let mut fields = Fields::new();
        let identity = entity.get("identity").expect("an entity has an identity");
        fields.insert(
            text(identity, "name")
                .expect("an identity has a name")
                .to_owned(),
            Field {
                rust: rust_spelling(text(identity, "type").expect("an identity has a type")),
                optional: false,
            },
        );
        for field in sequence(entity, "fields") {
            let field_name = text(field, "name").expect("a field has a name").to_owned();
            let field_type = text(field, "type").expect("a field has a type");
            if field_type == "Integer" || field_type == "Optional<Integer>" {
                integers
                    .entry(name.clone())
                    .or_default()
                    .insert(field_name.clone());
            }
            fields.insert(
                field_name,
                Field {
                    rust: rust_spelling(field_type),
                    optional: field_type.starts_with("Optional<"),
                },
            );
        }
        for relation in sequence(entity, "relations") {
            let target = local(text(relation, "target").expect("a relation has a target"));
            assert_eq!(
                text(relation, "cardinality"),
                Some("many"),
                "only `many` relations are mapped by this test"
            );
            fields.insert(
                text(relation, "name")
                    .expect("a relation has a name")
                    .to_owned(),
                Field {
                    rust: format!("Vec<{target}>"),
                    optional: false,
                },
            );
            if text(relation, "kind") == Some("owns") {
                if let Some(via) = text(relation, "via") {
                    nested.push((target, via.to_owned()));
                }
            }
        }
        entities.insert(name, fields);
    }
    for (target, via) in nested {
        let fields = entities
            .get_mut(&target)
            .unwrap_or_else(|| panic!("the owned `{target}` is declared"));
        assert!(
            fields.remove(&via).is_some(),
            "`{target}` declares the `via` field `{via}`"
        );
        if let Some(names) = integers.get_mut(&target) {
            names.remove(&via);
        }
    }
    Model {
        entities,
        enums,
        newtypes,
        integers,
    }
}

/// The wire names one set of serde attributes gives an item, and the attributes that change the
/// wire in a way this scan does not model.
struct Serde {
    rename: Option<String>,
    aliases: Vec<String>,
    refused: Vec<String>,
}

/// The serde keys a scanned type may carry. Everything else is refused outright: `skip`,
/// `skip_deserializing`, `default`, `other`, `flatten`, `rename_all`, `tag`, `untagged`, `with`,
/// `from` and the rest each change what the reader admits without the model saying so, and an
/// allow-list closes that class where a deny-list would wait for the next spelling.
///
/// `skip_serializing_if` changes only what a writer emits for an absent `Option`, which the reader
/// admits either way. `deserialize_with = "objects"` is the one reader hook: it refuses an
/// operation written as a JSON array and otherwise defers to the derived visitor.
const ALLOWED_KEYS: &[&str] = &[
    "rename",
    "alias",
    "deny_unknown_fields",
    "skip_serializing_if",
];
const ALLOWED_DESERIALIZE_WITH: &str = "objects";

fn serde_attributes(attributes: &[syn::Attribute]) -> Serde {
    let mut found = Serde {
        rename: None,
        aliases: Vec::new(),
        refused: Vec::new(),
    };
    for attribute in attributes {
        if attribute.path().is_ident("cfg_attr") {
            // serde honours `serde(…)` inside `cfg_attr`; whether the predicate holds is a build
            // fact this scan cannot decide, so the wrapper is refused whenever it carries serde.
            let carries_serde = attribute
                .meta
                .to_token_stream()
                .into_iter()
                .any(|token| contains_ident(&token, "serde"));
            if carries_serde {
                found.refused.push("cfg_attr(…, serde(…))".to_owned());
            }
            continue;
        }
        if !attribute.path().is_ident("serde") {
            continue;
        }
        attribute
            .parse_nested_meta(|meta| {
                let key = meta.path.to_token_stream().to_string();
                if meta.input.peek(syn::token::Paren) {
                    // `rename(serialize = …, deserialize = …)`, `rename_all(…)`: split names.
                    found.refused.push(format!("{key}(…)"));
                    let _ = meta.parse_nested_meta(|inner| {
                        if inner.input.peek(syn::Token![=]) {
                            let _: syn::Expr = inner.value()?.parse()?;
                        }
                        Ok(())
                    });
                    return Ok(());
                }
                let literal = if meta.input.peek(syn::Token![=]) {
                    let value: syn::Expr = meta.value()?.parse()?;
                    match &value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(text),
                            ..
                        }) => Some(text.value()),
                        _ => None,
                    }
                } else {
                    None
                };
                match key.as_str() {
                    "rename" => found.rename = literal,
                    "alias" => found.aliases.extend(literal),
                    "deserialize_with" if literal.as_deref() == Some(ALLOWED_DESERIALIZE_WITH) => {}
                    allowed if ALLOWED_KEYS.contains(&allowed) => {}
                    _ => found.refused.push(key),
                }
                Ok(())
            })
            .expect("a `serde` attribute parses");
    }
    found
}

/// Whether a token tree holds the identifier `name` anywhere inside it.
fn contains_ident(token: &proc_macro2::TokenTree, name: &str) -> bool {
    match token {
        proc_macro2::TokenTree::Ident(ident) => ident == name,
        proc_macro2::TokenTree::Group(group) => group
            .stream()
            .into_iter()
            .any(|inner| contains_ident(&inner, name)),
        _ => false,
    }
}

/// A Rust type in the spelling [`rust_spelling`] produces: last path segment and its type
/// arguments, `Option<u64>` rather than `std::option::Option < u64 >`.
fn render(ty: &syn::Type) -> String {
    let syn::Type::Path(path) = ty else {
        return ty.to_token_stream().to_string();
    };
    let Some(segment) = path.path.segments.last() else {
        return ty.to_token_stream().to_string();
    };
    let mut spelled = segment.ident.to_string();
    if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
        let inner: Vec<String> = arguments
            .args
            .iter()
            .map(|argument| match argument {
                syn::GenericArgument::Type(inner) => render(inner),
                other => other.to_token_stream().to_string(),
            })
            .collect();
        spelled = format!("{spelled}<{}>", inner.join(","));
    }
    spelled
}

/// What the Rust source declares, by type name.
struct Rust {
    structs: BTreeMap<String, Fields>,
    enums: BTreeMap<String, BTreeSet<String>>,
    reexports: BTreeSet<String>,
    /// Owning type, and `Type`, `Type.field` or `Type::Variant` with the refused attribute.
    refused: Vec<(String, String)>,
}

fn reexported(tree: &syn::UseTree, into: &mut BTreeSet<String>) {
    match tree {
        syn::UseTree::Path(path) => reexported(&path.tree, into),
        syn::UseTree::Name(name) => {
            into.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            into.insert(rename.rename.to_string());
        }
        syn::UseTree::Group(group) => group.items.iter().for_each(|item| reexported(item, into)),
        syn::UseTree::Glob(_) => {}
    }
}

fn rust(text: &str) -> Rust {
    let file = syn::parse_file(text).expect("the history module parses");
    let mut rust = Rust {
        structs: BTreeMap::new(),
        enums: BTreeMap::new(),
        reexports: BTreeSet::new(),
        refused: Vec::new(),
    };
    for item in &file.items {
        match item {
            syn::Item::Struct(declaration) => {
                let owner = declaration.ident.to_string();
                for refused in serde_attributes(&declaration.attrs).refused {
                    rust.refused
                        .push((owner.clone(), format!("{owner}: {refused}")));
                }
                let mut fields = Fields::new();
                if let syn::Fields::Named(named) = &declaration.fields {
                    for field in &named.named {
                        let ident = field.ident.as_ref().expect("a named field").to_string();
                        let serde = serde_attributes(&field.attrs);
                        for refused in serde.refused {
                            rust.refused
                                .push((owner.clone(), format!("{owner}.{ident}: {refused}")));
                        }
                        let rendered = render(&field.ty);
                        let shape = Field {
                            optional: rendered.starts_with("Option<"),
                            rust: rendered,
                        };
                        let primary = serde.rename.unwrap_or(ident);
                        for wire in std::iter::once(primary).chain(serde.aliases) {
                            fields.insert(wire, shape.clone());
                        }
                    }
                }
                rust.structs.insert(owner, fields);
            }
            syn::Item::Enum(declaration) => {
                let owner = declaration.ident.to_string();
                for refused in serde_attributes(&declaration.attrs).refused {
                    rust.refused
                        .push((owner.clone(), format!("{owner}: {refused}")));
                }
                let mut variants = BTreeSet::new();
                for variant in &declaration.variants {
                    let ident = variant.ident.to_string();
                    let serde = serde_attributes(&variant.attrs);
                    for refused in serde.refused {
                        rust.refused
                            .push((owner.clone(), format!("{owner}::{ident}: {refused}")));
                    }
                    variants.insert(serde.rename.unwrap_or(ident));
                    variants.extend(serde.aliases);
                }
                rust.enums.insert(owner, variants);
            }
            syn::Item::Use(declaration)
                if matches!(declaration.vis, syn::Visibility::Public(_)) =>
            {
                reexported(&declaration.tree, &mut rust.reexports);
            }
            _ => {}
        }
    }
    rust
}

/// One problem per differing set, naming both directions.
fn compare<T: Ord + std::fmt::Debug>(
    problems: &mut Vec<String>,
    what: &str,
    model: &BTreeSet<T>,
    other: &BTreeSet<T>,
    other_name: &str,
) {
    if model != other {
        problems.push(format!(
            "{what}: {other_name} has drifted from {MODEL}. Declared by the model and missing \
             from {other_name}: {:?}. In {other_name} and not declared by the model: {:?}",
            model.difference(other).collect::<Vec<_>>(),
            other.difference(model).collect::<Vec<_>>(),
        ));
    }
}

fn field_set(fields: &Fields) -> BTreeSet<(String, String, bool)> {
    fields
        .iter()
        .map(|(name, field)| (name.clone(), field.rust.clone(), field.optional))
        .collect()
}

/// The private type [`read`] deserializes an entity through: `History` → `HistoryWire`.
///
/// The public type is what a caller holds and a writer serializes; the wire twin is what the
/// reader admits. Both are held to the model.
fn wire_twin(entity: &str) -> String {
    format!("{entity}Wire")
}

/// Every way `text` (the Rust source) disagrees with the model. Empty is agreement.
fn rust_drift(model: &Model, text: &str) -> Vec<String> {
    let rust = rust(text);
    let scanned: BTreeSet<String> = model
        .entities
        .keys()
        .flat_map(|entity| [entity.clone(), wire_twin(entity)])
        .chain(model.enums.keys().cloned())
        .collect();
    let mut problems: Vec<String> = rust
        .refused
        .iter()
        .filter(|(owner, _)| scanned.contains(owner))
        .map(|(_, refused)| {
            format!(
                "{SOURCE}: `{refused}` changes the wire in a way this scan does not model; \
                 declare the shape in {MODEL} instead"
            )
        })
        .collect();
    for (entity, fields) in &model.entities {
        for rust_name in [entity.clone(), wire_twin(entity)] {
            match rust.structs.get(&rust_name) {
                None => problems.push(format!("`struct {rust_name}` is not declared in {SOURCE}")),
                Some(declared) => compare(
                    &mut problems,
                    &format!("`{rust_name}` fields (wire name, type, optional)"),
                    &field_set(fields),
                    &field_set(declared),
                    SOURCE,
                ),
            }
        }
    }
    for (name, variants) in &model.enums {
        match rust.enums.get(name) {
            None => problems.push(format!("`enum {name}` is not declared in {SOURCE}")),
            Some(declared) => compare(
                &mut problems,
                &format!("`{name}` values"),
                variants,
                declared,
                SOURCE,
            ),
        }
    }
    for name in &model.newtypes {
        if !rust.structs.contains_key(name) && !rust.reexports.contains(name) {
            problems.push(format!(
                "the newtype `{name}` is neither a struct nor a `pub use` in {SOURCE}"
            ));
        }
    }
    problems
}

/// A model field's Rust spelling in the vocabulary [`schema_kind`] reads a schema property into.
fn expected_kind(rust: &str) -> String {
    // A decision time is absent where no decision was observed and never `null`
    // (`ess-history/2`): its schema property is the bare reference, not the reference or null.
    if rust == "Option<DecisionInstant>" {
        return "ref:DecisionInstant".to_owned();
    }
    if let Some(inner) = rust
        .strip_prefix("Option<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return format!("opt:{}", expected_kind(inner));
    }
    if let Some(inner) = rust
        .strip_prefix("Vec<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return format!("array:{}", expected_kind(inner));
    }
    match rust {
        "u64" => "integer".to_owned(),
        "String" => "string".to_owned(),
        named => format!("ref:{named}"),
    }
}

/// One schema property as `integer`, `string`, `ref:<Definition>`, `array:<kind>` or
/// `opt:<kind>`; anything else is spelled out as its JSON and so matches nothing expected.
fn schema_kind(property: &serde_json::Value) -> String {
    if let Some(target) = property.get("$ref").and_then(serde_json::Value::as_str) {
        return target
            .strip_prefix("#/definitions/")
            .map_or_else(|| format!("ref?{target}"), |name| format!("ref:{name}"));
    }
    let null = serde_json::json!({ "type": "null" });
    if let Some(branches) = property.get("anyOf").and_then(serde_json::Value::as_array) {
        if branches.len() == 2 && branches.contains(&null) {
            let other = branches
                .iter()
                .find(|branch| **branch != null)
                .expect("two branches, one null");
            return format!("opt:{}", schema_kind(other));
        }
    }
    match property.get("type") {
        Some(serde_json::Value::String(kind)) if kind == "array" => format!(
            "array:{}",
            property
                .get("items")
                .map_or_else(|| "?".to_owned(), schema_kind)
        ),
        Some(serde_json::Value::String(kind)) if kind == "integer" || kind == "string" => {
            kind.clone()
        }
        Some(serde_json::Value::Array(kinds))
            if kinds.len() == 2 && kinds.contains(&serde_json::json!("null")) =>
        {
            let other = kinds
                .iter()
                .find(|kind| kind.as_str() != Some("null"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("?");
            format!("opt:{other}")
        }
        _ => property.to_string(),
    }
}

/// One schema object's properties: `rust` carries the property's [`schema_kind`], and `required`
/// is read as "not optional".
fn schema_fields(object: &serde_json::Value, what: &str, problems: &mut Vec<String>) -> Fields {
    if object.get("additionalProperties") != Some(&serde_json::Value::Bool(false)) {
        problems.push(format!(
            "{SCHEMA}: {what} does not refuse unknown properties"
        ));
    }
    let required: BTreeSet<&str> = object
        .get("required")
        .and_then(serde_json::Value::as_array)
        .map(|names| names.iter().filter_map(serde_json::Value::as_str).collect())
        .unwrap_or_default();
    object
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, property)| {
                    (
                        name.clone(),
                        Field {
                            rust: schema_kind(property),
                            optional: !required.contains(name.as_str()),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Every way the schema disagrees with the model. Empty is agreement.
fn schema_drift(model: &Model, schema: &serde_json::Value) -> Vec<String> {
    let mut problems = Vec::new();
    for (entity, fields) in &model.entities {
        let object = if entity == ROOT {
            Some(schema)
        } else {
            schema.pointer(&format!("/definitions/{entity}"))
        };
        let Some(object) = object else {
            problems.push(format!("{SCHEMA} does not define `definitions/{entity}`"));
            continue;
        };
        let declared = schema_fields(object, entity, &mut problems);
        let expected: Fields = fields
            .iter()
            .map(|(name, field)| {
                (
                    name.clone(),
                    Field {
                        rust: expected_kind(&field.rust),
                        optional: field.optional,
                    },
                )
            })
            .collect();
        compare(
            &mut problems,
            &format!("`{entity}` properties (name, JSON type, optional)"),
            &field_set(&expected),
            &field_set(&declared),
            SCHEMA,
        );
        for integer in model.integers.get(entity).into_iter().flatten() {
            let property = object.pointer(&format!("/properties/{integer}"));
            let bound = |key: &str| {
                property
                    .and_then(|value| value.get(key))
                    .and_then(serde_json::Value::as_u64)
            };
            // `minimum` may be above 0 where the reader refuses 0 by name (`clients`).
            if bound("minimum").is_none() || bound("maximum") != Some(MAX_INTEGER) {
                problems.push(format!(
                    "{SCHEMA}: `{entity}.{integer}` is not bounded within 0..={MAX_INTEGER}, the \
                     range the reader admits"
                ));
            }
        }
    }
    for (name, variants) in &model.enums {
        match schema
            .pointer(&format!("/definitions/{name}/enum"))
            .and_then(serde_json::Value::as_array)
        {
            None => problems.push(format!(
                "{SCHEMA} does not define `definitions/{name}` as an `enum`"
            )),
            Some(values) => {
                let values: BTreeSet<String> = values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .collect();
                compare(
                    &mut problems,
                    &format!("`{name}` values"),
                    variants,
                    &values,
                    SCHEMA,
                );
            }
        }
    }
    problems
}

fn schema() -> serde_json::Value {
    serde_json::from_str(&source(SCHEMA)).expect("the schema is JSON")
}

fn assert_no_drift(problems: &[String]) {
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_rust_type_carries_exactly_the_modelled_fields_types_and_values() {
    assert_no_drift(&rust_drift(&model(), &source(SOURCE)));
}

#[test]
fn the_schema_declares_exactly_the_modelled_fields_and_values() {
    assert_no_drift(&schema_drift(&model(), &schema()));
}

/// The scan reads the declarations it names, rather than comparing two empty sets.
#[test]
fn every_declaration_this_scan_reads_is_non_empty() {
    let model = model();
    let rust = rust(&source(SOURCE));
    assert!(
        model.entities.contains_key(ROOT),
        "no `{ROOT}` entity was read"
    );
    assert!(
        model.entities.len() >= 2,
        "fewer than two entities were read"
    );
    assert!(
        model.entities.values().all(|fields| fields.len() >= 2),
        "an entity was read with fewer than two fields"
    );
    assert!(!model.enums.is_empty(), "no enum was read");
    assert!(!model.newtypes.is_empty(), "no newtype was read");
    assert!(
        model.integers.values().map(BTreeSet::len).sum::<usize>() >= 3,
        "fewer than three integer fields were read"
    );
    assert!(!rust.structs.is_empty(), "no struct was read from {SOURCE}");
    assert!(!rust.enums.is_empty(), "no enum was read from {SOURCE}");
    assert!(
        model
            .entities
            .get("Operation")
            .is_some_and(|fields| fields.values().any(|field| field.optional)),
        "no optional field was read, so `Optional<…>` is not being recognised"
    );
}

/// The real source with one exact edit applied, which must be present to apply.
fn mutant(from: &str, to: &str) -> String {
    let real = source(SOURCE);
    assert_eq!(
        real.matches(from).count(),
        1,
        "the mutant's anchor `{from}` occurs exactly once in {SOURCE}"
    );
    real.replacen(from, to, 1)
}

/// The drift scan over a mutant reports a problem mentioning `expected`.
fn assert_caught(text: &str, expected: &str) {
    let problems = rust_drift(&model(), text);
    assert!(
        problems.iter().any(|problem| problem.contains(expected)),
        "the drift scan missed a mutant; expected a problem mentioning `{expected}`, got {problems:?}"
    );
}

#[test]
fn a_container_rename_all_is_caught() {
    let text = mutant(
        "#[serde(deny_unknown_fields)]\npub struct Operation {",
        "#[serde(deny_unknown_fields, rename_all = \"camelCase\")]\npub struct Operation {",
    );
    assert_caught(&text, "Operation: rename_all");
}

#[test]
fn an_undeclared_alias_is_caught() {
    let text = mutant(
        "    pub outcome: Option<QualifiedName>,",
        "    #[serde(alias = \"result\")]\n    pub outcome: Option<QualifiedName>,",
    );
    assert_caught(&text, "\"result\"");
}

#[test]
fn a_field_of_the_wrong_type_is_caught() {
    let text = mutant(
        "pub returned_at: Option<u64>,",
        "pub returned_at: Option<String>,",
    );
    assert_caught(&text, "Option<String>");
}

#[test]
fn a_split_rename_is_caught() {
    let text = mutant(
        "    pub subject_key: String,",
        "    #[serde(rename(deserialize = \"subject\"))]\n    pub subject_key: String,",
    );
    assert_caught(&text, "Operation.subject_key: rename(…)");
}

#[test]
fn a_missing_field_and_an_extra_value_are_caught() {
    assert_caught(&mutant("    pub seed: u64,\n", ""), "\"seed\"");
    assert_caught(
        &mutant(
            "    Indeterminate,\n}",
            "    Indeterminate,\n    Cancelled,\n}",
        ),
        "Cancelled",
    );
}

/// The integer literal of a `const` named `name`, at the top level or inside `impl <owner>`.
fn integer_const(text: &str, owner: Option<&str>, name: &str) -> u64 {
    let file = syn::parse_file(text).expect("the source parses");
    let literal = |expression: &syn::Expr| match expression {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(number),
            ..
        }) => number.base10_parse::<u64>().ok(),
        _ => None,
    };
    file.items
        .iter()
        .find_map(|item| match (item, owner) {
            (syn::Item::Const(declaration), None) if declaration.ident == name => {
                literal(&declaration.expr)
            }
            (syn::Item::Impl(block), Some(owner))
                if block.trait_.is_none() && render(&block.self_ty) == owner =>
            {
                block.items.iter().find_map(|inner| match inner {
                    syn::ImplItem::Const(declaration) if declaration.ident == name => {
                        literal(&declaration.expr)
                    }
                    _ => None,
                })
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("an integer `const {name}` is declared ({owner:?})"))
}

/// What the model declares about `SpecDigest`: its alphabet and its length bounds.
fn modelled_digest() -> (BTreeSet<char>, u64, u64) {
    let document: Yaml = serde_yaml::from_str(&source(MODEL)).expect("the domain is YAML");
    let declaration = sequence(&document, "types")
        .iter()
        .find(|declaration| text(declaration, "name") == Some("concurrent.history.SpecDigest"))
        .expect("the model declares SpecDigest");
    let alphabet = text(declaration, "alphabet")
        .expect("SpecDigest declares an alphabet")
        .chars()
        .collect();
    let mut minimum = None;
    let mut maximum = None;
    for invariant in sequence(declaration, "invariants") {
        let invariant = invariant.as_str().expect("an invariant is text");
        if let Some(bound) = invariant.strip_prefix("value.count >= ") {
            minimum = bound.parse().ok();
        } else if let Some(bound) = invariant.strip_prefix("value.count <= ") {
            maximum = bound.parse().ok();
        }
    }
    (
        alphabet,
        minimum.expect("SpecDigest declares `value.count >= N`"),
        maximum.expect("SpecDigest declares `value.count <= N`"),
    )
}

/// The model's `SpecDigest` is the reader's: the same alphabet, the same length bounds, and the
/// schema's pattern says the same.
#[test]
fn the_modelled_spec_digest_is_the_one_the_reader_admits() {
    let (alphabet, minimum, maximum) = modelled_digest();
    let digest_source = source(DIGEST_SOURCE);
    // `SpecDigest::new` admits `is_ascii_hexdigit` characters that are not `is_ascii_uppercase`.
    for predicate in [
        "ch.is_ascii_hexdigit()",
        "!value.chars().any(|ch| ch.is_ascii_uppercase())",
    ] {
        assert!(
            digest_source.contains(predicate),
            "{DIGEST_SOURCE} no longer spells the admission rule this case reads (`{predicate}`); \
             re-derive the model's alphabet from it"
        );
    }
    let admitted: BTreeSet<char> = (0_u8..128)
        .map(char::from)
        .filter(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        .collect();
    assert_eq!(alphabet, admitted, "the model's SpecDigest alphabet");
    assert_eq!(
        (minimum, maximum),
        (
            integer_const(&digest_source, Some("SpecDigest"), "MIN_LENGTH"),
            integer_const(&digest_source, Some("SpecDigest"), "MAX_LENGTH"),
        ),
        "the model's SpecDigest length bounds"
    );
    assert_eq!(
        schema()
            .pointer("/definitions/SpecDigest/pattern")
            .and_then(serde_json::Value::as_str),
        Some(format!("^[0-9a-f]{{{minimum},{maximum}}}$").as_str()),
        "the schema's SpecDigest pattern"
    );
}

/// The bound the drift scan holds the schema to is the reader's own.
#[test]
fn the_integer_bound_is_the_readers() {
    assert_eq!(
        integer_const(&source(SOURCE), None, "MAX_INTEGER"),
        MAX_INTEGER,
        "`MAX_INTEGER` in {SOURCE}"
    );
}

// Adversary pass 2 (story:concurrent-history-format). Each mutant below changes what the reader
// admits on the wire without touching a field identifier, a type spelling or a `rename`/`alias`,
// so the model and the Rust type disagree while the scan sees the same names.

/// The drift scan over a mutant reports at least one problem.
fn assert_any_problem(text: &str, mutation: &str) {
    let problems = rust_drift(&model(), text);
    assert!(
        !problems.is_empty(),
        "the drift scan missed a mutant ({mutation}): it reported no problem, so the reader's \
         wire shape can leave the model without this test failing"
    );
}

/// `#[serde(skip)]` removes `seed` from the wire: the reader then refuses a document carrying the
/// field the model declares (`deny_unknown_fields`), and admits one without it.
#[test]
fn adversary_a_skipped_field_is_caught() {
    let text = mutant(
        "    pub seed: u64,\n",
        "    #[serde(skip)]\n    pub seed: u64,\n",
    );
    assert_any_problem(&text, "#[serde(skip)] on History.seed");
}

/// `#[serde(default)]` makes the required `Integer` field `seed` optional on the wire; the scan
/// decides optionality from the `Option<…>` spelling alone.
#[test]
fn adversary_a_defaulted_required_field_is_caught() {
    let text = mutant(
        "    pub seed: u64,\n",
        "    #[serde(default)]\n    pub seed: u64,\n",
    );
    assert_any_problem(&text, "#[serde(default)] on History.seed");
}

/// `#[serde(skip_deserializing)]` on a variant removes a declared enum value from what the reader
/// admits.
#[test]
fn adversary_a_variant_the_reader_cannot_read_is_caught() {
    let text = mutant(
        "    Indeterminate,\n}",
        "    #[serde(skip_deserializing)]\n    Indeterminate,\n}",
    );
    assert_any_problem(
        &text,
        "#[serde(skip_deserializing)] on Completion::Indeterminate",
    );
}

/// `#[serde(other)]` makes every undeclared completion string read as `Indeterminate`: the enum
/// then admits values the model does not declare.
#[test]
fn adversary_a_catch_all_variant_is_caught() {
    let text = mutant(
        "    Indeterminate,\n}",
        "    #[serde(other)]\n    Indeterminate,\n}",
    );
    assert_any_problem(&text, "#[serde(other)] on Completion::Indeterminate");
}

/// serde honours a `serde(…)` attribute inside `cfg_attr`; the scan reads only `#[serde(…)]`, so
/// the same `rename_all` it refuses outright passes when wrapped.
#[test]
fn adversary_a_rename_all_inside_cfg_attr_is_caught() {
    let text = mutant(
        "#[serde(deny_unknown_fields)]\npub struct Operation {",
        "#[serde(deny_unknown_fields)]\n#[cfg_attr(all(), serde(rename_all = \"camelCase\"))]\npub struct Operation {",
    );
    assert_any_problem(&text, "cfg_attr(all(), serde(rename_all)) on Operation");
}

/// The module docs above say each check "fails when a field, a field type or an enum value the
/// model declares is missing"; the schema half compares names and optionality only, so a schema
/// property of the wrong JSON type passes it.
#[test]
fn adversary_a_schema_property_of_the_wrong_type_is_caught() {
    let mut missed = Vec::new();
    for (pointer, replacement, what) in [
        (
            "/definitions/Operation/properties/subject_key",
            serde_json::json!({ "type": "integer" }),
            "Operation.subject_key (String) as integer",
        ),
        (
            "/properties/seed/type",
            serde_json::json!("string"),
            "History.seed (Integer) as string",
        ),
        (
            "/definitions/Operation/properties/command",
            serde_json::json!({ "$ref": "#/definitions/Uuid" }),
            "Operation.command (QualifiedName) as Uuid",
        ),
    ] {
        let mut altered = schema();
        *altered
            .pointer_mut(pointer)
            .unwrap_or_else(|| panic!("the schema has {pointer}")) = replacement;
        if schema_drift(&model(), &altered).is_empty() {
            missed.push(what);
        }
    }
    assert!(
        missed.is_empty(),
        "the schema drift scan does not compare field types; it passed: {missed:?}"
    );
}
