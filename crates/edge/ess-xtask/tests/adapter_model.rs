//! The `ess-history-adapter/1` Rust type and JSON Schema, held to `models/recorded-log-adapter/`.
//!
//! The model declares the adapter document; `crates/verify/ess-conformance/src/recorded.rs` is the
//! hand-written reader and `schemas/ess-history-adapter.schema.json` the schema an operator's
//! tooling validates an adapter against. No generator links the three, so this file does: each
//! fails when a field, a field type or an enum value the model declares is missing, or when it
//! carries one the model does not declare.
//!
//! How the model maps onto a document, derived from the model rather than listed here:
//!
//! - a struct `recorded.adapter.T` is `struct T` with `#[serde(deny_unknown_fields)]` and a schema
//!   object with `additionalProperties: false`; `Adapter` is the document itself;
//! - an enum `recorded.adapter.T` is `enum T` of unit variants, compared by wire name, and a schema
//!   string `enum`;
//! - a newtype of `String` is a Rust `String`; its `prefix` is the reader's `starts_with` check and
//!   the schema's `pattern`;
//! - `String` is `String`; `Map<String, T>` is `Words<T>`, the log's words each mapped to a `T`,
//!   and a schema object whose `additionalProperties` is `T`. The model's key type is `String`
//!   because a JSON key is one; the reader also takes a YAML integer key as an integer word, which
//!   has no JSON spelling and so no place in the model or the schema;
//! - `Optional<T>` is the adapter's own spelling of a source that may be missing: the word
//!   `absent` (the model's enum `Absent`) or a `T`. ESS offers no untagged union
//!   (`crates/specify/ess-domain/src/types.rs`, "Unions are tagged"), so the choice is modelled as
//!   an `Optional` whose none is written `absent` rather than `null`. In Rust it is an enum of
//!   exactly two one-field variants, one holding `Absent` and the other `T`, read by a hand-written
//!   `Deserialize` rather than a derived one: a derived enum is externally tagged, and
//!   `#[serde(untagged)]` reads through serde's buffer, which also admits `{absent: null}` and a
//!   struct written as a list of its fields. In the schema it is a `oneOf` of the two. Every such
//!   field declares a `presence`: `null_when_absent` is a key that must be written,
//!   `omitted_when_absent` one that may be left out, which in Rust is `#[serde(default = "absent")]`
//!   and in the schema a key not `required`. What the hand-written reader admits is held by the
//!   adapter-document table (`tests/fixtures/recorded/adapter-documents.json` in ess-conformance).
//!
//! `recorded.adapter.Completion` is `concurrent.history.Completion`, redeclared because a domain
//! cannot name a type of another system; the reader takes it from `history.rs`, and a case below
//! holds the two models' copies to the same values.
//!
//! Every other serde key on a scanned type changes the wire in a way this scan does not model, and
//! is refused outright; [`ALLOWED_KEYS`] says what is admitted where. The mutant cases at the end
//! run the scan over altered copies of the real source to show it notices.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use serde_json::Value as Json;
use serde_yaml::Value as Yaml;

const MODEL: &str = "models/recorded-log-adapter/domains/adapter.yaml";
const HISTORY_MODEL: &str = "models/concurrent-history/domains/history.yaml";
const SOURCE: &str = "crates/verify/ess-conformance/src/recorded.rs";
/// Where the reader takes the types it imports from `crate::history`.
const HISTORY_SOURCE: &str = "crates/verify/ess-conformance/src/history.rs";
const SCHEMA: &str = "schemas/ess-history-adapter.schema.json";
const PREFIX: &str = "recorded.adapter.";
/// The struct the document's top level is.
const ROOT: &str = "Adapter";
/// The enum whose one value is how an adapter writes a source the log does not carry.
const ABSENT: &str = "Absent";
/// The function `#[serde(default = …)]` names for a source that may be left out.
const ABSENT_DEFAULT: &str = "absent";

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

fn text<'a>(value: &'a Yaml, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Yaml::as_str)
}

fn sequence<'a>(value: &'a Yaml, key: &str) -> &'a [Yaml] {
    value
        .get(key)
        .and_then(Yaml::as_sequence)
        .map_or(&[], Vec::as_slice)
}

fn generic<'a>(value: &'a str, constructor: &str) -> Option<&'a str> {
    value
        .strip_prefix(constructor)
        .and_then(|rest| rest.strip_prefix('<'))
        .and_then(|rest| rest.strip_suffix('>'))
        .map(str::trim)
}

/// One field of a struct: its type, and whether the key may be left out of a document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Shape {
    ty: String,
    omissible: bool,
}

/// Wire name → shape.
type Fields = BTreeMap<String, Shape>;

/// One struct field in the model: its name, and its model type and presence.
type ModelField = (String, (String, Option<String>));

/// What the model declares.
struct Model {
    /// Struct → its fields in declaration order, each with its model type and presence.
    structs: BTreeMap<String, Vec<ModelField>>,
    enums: BTreeMap<String, BTreeSet<String>>,
    /// Newtype → (what it wraps, its declared prefix).
    newtypes: BTreeMap<String, (String, Option<String>)>,
}

fn local(name: &str) -> String {
    name.strip_prefix(PREFIX)
        .unwrap_or_else(|| panic!("`{name}` is declared under `{PREFIX}`"))
        .to_owned()
}

fn enum_variants(declaration: &Yaml) -> BTreeSet<String> {
    sequence(declaration, "variants")
        .iter()
        .map(|variant| match variant {
            Yaml::String(name) => name.clone(),
            mapping => text(mapping, "wire")
                .or_else(|| text(mapping, "name"))
                .expect("a variant has a name")
                .to_owned(),
        })
        .collect()
}

fn read_model(text_of_model: &str) -> Model {
    let document: Yaml = serde_yaml::from_str(text_of_model).expect("the domain is YAML");
    let mut model = Model {
        structs: BTreeMap::new(),
        enums: BTreeMap::new(),
        newtypes: BTreeMap::new(),
    };
    for declaration in sequence(&document, "types") {
        let name = local(text(declaration, "name").expect("a type has a name"));
        match text(declaration, "kind") {
            Some("enum") => {
                model.enums.insert(name, enum_variants(declaration));
            }
            Some("newtype") => {
                model.newtypes.insert(
                    name,
                    (
                        text(declaration, "of").expect("a newtype wraps").to_owned(),
                        text(declaration, "prefix").map(str::to_owned),
                    ),
                );
            }
            Some("struct") => {
                let fields = sequence(declaration, "fields")
                    .iter()
                    .map(|field| {
                        (
                            text(field, "name").expect("a field has a name").to_owned(),
                            (
                                text(field, "type").expect("a field has a type").to_owned(),
                                text(field, "presence").map(str::to_owned),
                            ),
                        )
                    })
                    .collect();
                model.structs.insert(name, fields);
            }
            other => panic!("`{name}` has a kind this test does not map: {other:?}"),
        }
    }
    for kind in ["entities", "commands", "events"] {
        assert!(
            sequence(&document, kind).is_empty(),
            "{MODEL} declares {kind}, which this test does not map"
        );
    }
    model
}

fn model() -> Model {
    read_model(&source(MODEL))
}

impl Model {
    /// A model type in the spelling the Rust source carries after carriers are resolved.
    fn rust_spelling(&self, model_type: &str) -> String {
        if let Some(inner) = generic(model_type, "Optional") {
            return format!("Optional<{}>", self.rust_spelling(inner));
        }
        if let Some(inner) = generic(model_type, "Map") {
            let (key, value) = inner.split_once(',').expect("a map has a key and a value");
            assert_eq!(key.trim(), "String", "a map is keyed by a log's words");
            return format!("Words<{}>", self.rust_spelling(value.trim()));
        }
        match model_type {
            "String" => "String".to_owned(),
            other if other.starts_with(PREFIX) => {
                let name = local(other);
                match self.newtypes.get(&name) {
                    Some((of, _)) => self.rust_spelling(of),
                    None => name,
                }
            }
            other => panic!("the model type `{other}` has no mapping in this test"),
        }
    }

    /// Each modelled struct's fields in the shape the reader must give them, and every problem
    /// with a field's declaration itself.
    fn struct_shapes(&self, problems: &mut Vec<String>) -> BTreeMap<String, Fields> {
        self.structs
            .iter()
            .map(|(name, fields)| {
                let shapes = fields
                    .iter()
                    .map(|(field, (ty, presence))| {
                        let optional = generic(ty, "Optional").is_some();
                        let omissible = match (optional, presence.as_deref()) {
                            (true, Some("omitted_when_absent")) => true,
                            (true, Some("null_when_absent")) | (false, _) => false,
                            (true, other) => {
                                problems.push(format!(
                                    "{MODEL}: `{name}.{field}` is Optional and declares the \
                                     presence {other:?}; an adapter source says which of \
                                     `null_when_absent` (must be written) and \
                                     `omitted_when_absent` (may be left out) it is"
                                ));
                                false
                            }
                        };
                        (
                            field.clone(),
                            Shape {
                                ty: self.rust_spelling(ty),
                                omissible,
                            },
                        )
                    })
                    .collect();
                (name.clone(), shapes)
            })
            .collect()
    }
}

/// The serde keys admitted on a scanned item. Everything else — `alias`, `rename_all`, `skip`,
/// `other`, `flatten`, `tag`, `untagged`, `with`, a `default` naming another function, and any
/// `cfg_attr` carrying serde — changes what the reader admits without the model saying so.
/// `untagged` in particular reads through serde's buffer, which admits spellings the model does
/// not declare.
///
/// `default = "absent"` is admitted on a field only, where it is `presence: omitted_when_absent`.
const ALLOWED_KEYS: &[&str] = &["rename", "deny_unknown_fields", "default"];

/// What one set of serde attributes says.
#[derive(Default)]
struct Serde {
    rename: Option<String>,
    deny_unknown_fields: bool,
    default_absent: bool,
    refused: Vec<String>,
}

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

fn serde_attributes(attributes: &[syn::Attribute]) -> Serde {
    let mut found = Serde::default();
    for attribute in attributes {
        if attribute.path().is_ident("cfg_attr") {
            if attribute
                .meta
                .to_token_stream()
                .into_iter()
                .any(|token| contains_ident(&token, "serde"))
            {
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
                let admitted = ALLOWED_KEYS.contains(&key.as_str());
                match (admitted, key.as_str(), literal.as_deref()) {
                    (true, "rename", Some(_)) => found.rename = literal,
                    (true, "deny_unknown_fields", None) => found.deny_unknown_fields = true,
                    (true, "default", Some(ABSENT_DEFAULT)) => found.default_absent = true,
                    (_, key, Some(value)) => found.refused.push(format!("{key} = \"{value}\"")),
                    (_, key, None) => found.refused.push(key.to_owned()),
                }
                Ok(())
            })
            .expect("a `serde` attribute parses");
    }
    found
}

/// A Rust type as its last path segment and type arguments: `BTreeMap<String,Completion>`.
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

struct RustField {
    ty: String,
    default_absent: bool,
    refused: Vec<String>,
}

struct RustStruct {
    deny_unknown_fields: bool,
    refused: Vec<String>,
    fields: BTreeMap<String, RustField>,
}

struct RustVariant {
    /// The one field a tuple variant holds; `None` for a unit variant, and `Some("…")` spelled out
    /// for any other shape.
    payload: Option<String>,
    refused: Vec<String>,
}

struct RustEnum {
    /// Whether `#[derive(…)]` names `Deserialize`: the wire then is serde's externally tagged
    /// form, a variant name per unit variant.
    derives_deserialize: bool,
    refused: Vec<String>,
    variants: BTreeMap<String, RustVariant>,
}

#[derive(Default)]
struct Rust {
    structs: BTreeMap<String, RustStruct>,
    enums: BTreeMap<String, RustEnum>,
    functions: BTreeMap<String, String>,
    /// Types with a hand-written `impl Deserialize`.
    hand_deserialize: BTreeSet<String>,
    /// Names a `use crate::history::…` brings in.
    from_history: BTreeSet<String>,
}

fn imported(tree: &syn::UseTree, path: &mut Vec<String>, into: &mut BTreeSet<String>) {
    match tree {
        syn::UseTree::Path(segment) => {
            path.push(segment.ident.to_string());
            imported(&segment.tree, path, into);
            path.pop();
        }
        syn::UseTree::Name(name) if path.as_slice() == ["crate", "history"] => {
            into.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) if path.as_slice() == ["crate", "history"] => {
            into.insert(rename.rename.to_string());
        }
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .for_each(|item| imported(item, path, into)),
        _ => {}
    }
}

fn scan(text: &str) -> Rust {
    let file = syn::parse_file(text).expect("the source parses");
    let mut rust = Rust::default();
    for item in &file.items {
        match item {
            syn::Item::Struct(declaration) => {
                let container = serde_attributes(&declaration.attrs);
                let mut refused = container.refused;
                if container.default_absent {
                    refused.push(format!("default = \"{ABSENT_DEFAULT}\""));
                }
                let mut fields = BTreeMap::new();
                if let syn::Fields::Named(named) = &declaration.fields {
                    for field in &named.named {
                        let ident = field.ident.as_ref().expect("a named field").to_string();
                        let serde = serde_attributes(&field.attrs);
                        let mut field_refused = serde.refused;
                        if serde.deny_unknown_fields {
                            field_refused.push("a container key on a field".to_owned());
                        }
                        let wire = serde.rename.unwrap_or(ident);
                        fields.insert(
                            wire,
                            RustField {
                                ty: render(&field.ty),
                                default_absent: serde.default_absent,
                                refused: field_refused,
                            },
                        );
                    }
                }
                rust.structs.insert(
                    declaration.ident.to_string(),
                    RustStruct {
                        deny_unknown_fields: container.deny_unknown_fields,
                        refused,
                        fields,
                    },
                );
            }
            syn::Item::Enum(declaration) => {
                let container = serde_attributes(&declaration.attrs);
                let mut refused = container.refused;
                if container.deny_unknown_fields || container.default_absent {
                    refused.push("a key that is not an enum's".to_owned());
                }
                let mut variants = BTreeMap::new();
                for variant in &declaration.variants {
                    let serde = serde_attributes(&variant.attrs);
                    let mut variant_refused = serde.refused;
                    if serde.deny_unknown_fields || serde.default_absent {
                        variant_refused.push("a key that is not a variant's".to_owned());
                    }
                    let payload = match &variant.fields {
                        syn::Fields::Unit => None,
                        syn::Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                            Some(render(&unnamed.unnamed[0].ty))
                        }
                        other => Some(other.to_token_stream().to_string()),
                    };
                    variants.insert(
                        serde.rename.unwrap_or_else(|| variant.ident.to_string()),
                        RustVariant {
                            payload,
                            refused: variant_refused,
                        },
                    );
                }
                rust.enums.insert(
                    declaration.ident.to_string(),
                    RustEnum {
                        derives_deserialize: derives(&declaration.attrs, "Deserialize"),
                        refused,
                        variants,
                    },
                );
            }
            syn::Item::Fn(function) => {
                rust.functions.insert(
                    function.sig.ident.to_string(),
                    function.block.to_token_stream().to_string(),
                );
            }
            syn::Item::Use(declaration) => {
                imported(&declaration.tree, &mut Vec::new(), &mut rust.from_history);
            }
            syn::Item::Impl(block)
                if block.trait_.as_ref().is_some_and(|(_, path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Deserialize")
                }) =>
            {
                rust.hand_deserialize.insert(render(&block.self_ty));
            }
            _ => {}
        }
    }
    rust
}

/// Whether a `#[derive(…)]` on the item names `name`.
fn derives(attributes: &[syn::Attribute], name: &str) -> bool {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("derive"))
        .any(|attribute| {
            attribute
                .meta
                .to_token_stream()
                .into_iter()
                .any(|token| contains_ident(&token, name))
        })
}

/// The reader's view: `recorded.rs`, with the enums it imports from `crate::history` read from
/// `history.rs`.
struct Reader {
    recorded: Rust,
    history: Rust,
}

impl Reader {
    fn read(recorded: &str, history: &str) -> Self {
        Self {
            recorded: scan(recorded),
            history: scan(history),
        }
    }

    fn enumeration(&self, name: &str) -> Option<(&RustEnum, &'static str)> {
        match self.recorded.enums.get(name) {
            Some(declared) => Some((declared, SOURCE)),
            None if self.recorded.from_history.contains(name) => self
                .history
                .enums
                .get(name)
                .map(|found| (found, HISTORY_SOURCE)),
            None => None,
        }
    }

    /// Whether `name` is an `Optional` carrier: an enum of this module read by a hand-written
    /// `Deserialize` and not a derived one.
    fn is_carrier(&self, name: &str) -> bool {
        self.recorded
            .enums
            .get(name)
            .is_some_and(|found| !found.derives_deserialize)
            && self.recorded.hand_deserialize.contains(name)
    }

    /// A field's Rust type with an `Optional` carrier resolved: a carrier of exactly one `Absent`
    /// variant and one other one-field variant is `Optional<Other>`; any other carrier is spelled
    /// out, and so matches nothing the model declares.
    fn resolve(&self, ty: &str) -> String {
        let Some(carrier) = self.recorded.enums.get(ty).filter(|_| self.is_carrier(ty)) else {
            return ty.to_owned();
        };
        let payloads: Vec<Option<&str>> = carrier
            .variants
            .values()
            .map(|variant| variant.payload.as_deref())
            .collect();
        let absent = payloads
            .iter()
            .filter(|payload| **payload == Some(ABSENT))
            .count();
        let others: Vec<&str> = payloads
            .iter()
            .filter_map(|payload| payload.filter(|inner| *inner != ABSENT))
            .collect();
        if payloads.len() == 2 && absent == 1 && others.len() == 1 {
            format!("Optional<{}>", others[0])
        } else {
            format!("Carrier<{payloads:?}>")
        }
    }

    /// The carriers the modelled structs' fields use.
    fn carriers<'a>(&'a self, model: &Model) -> BTreeSet<&'a str> {
        model
            .structs
            .keys()
            .filter_map(|name| self.recorded.structs.get(name))
            .flat_map(|declared| declared.fields.values())
            .filter_map(|field| {
                self.recorded
                    .enums
                    .get_key_value(field.ty.as_str())
                    .filter(|(name, _)| self.is_carrier(name))
                    .map(|(name, _)| name.as_str())
            })
            .collect()
    }
}

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

fn shape_set(fields: &Fields) -> BTreeSet<(String, String, bool)> {
    fields
        .iter()
        .map(|(name, shape)| (name.clone(), shape.ty.clone(), shape.omissible))
        .collect()
}

/// Every way the reader's source disagrees with the model. Empty is agreement.
fn rust_drift(model: &Model, recorded: &str, history: &str) -> Vec<String> {
    let reader = Reader::read(recorded, history);
    let mut problems = Vec::new();
    let shapes = model.struct_shapes(&mut problems);
    struct_drift(model, &reader, &shapes, &mut problems);
    enum_drift(model, &reader, &mut problems);

    let omissible = shapes
        .values()
        .flat_map(Fields::values)
        .any(|shape| shape.omissible);
    if omissible {
        let body = reader.recorded.functions.get(ABSENT_DEFAULT);
        if !body.is_some_and(|body| body.contains(&format!("{ABSENT} :: {}", model_absent(model))))
        {
            problems.push(format!(
                "{SOURCE}: `fn {ABSENT_DEFAULT}` does not return the `{ABSENT}` value, so a source \
                 left out is not read as `absent`"
            ));
        }
    }

    for (name, (of, prefix)) in &model.newtypes {
        assert_eq!(
            of, "String",
            "`{name}` wraps `String`, the one newtype this test maps"
        );
        if let Some(prefix) = prefix {
            let check = match prefix.chars().collect::<Vec<_>>().as_slice() {
                [single] => format!(".starts_with('{single}')"),
                _ => format!(".starts_with(\"{prefix}\")"),
            };
            prefix_drift(model, &reader, name, &check, &mut problems);
        }
    }
    problems
}

/// The reader checks `check` on every source that carries the newtype `name`.
///
/// The check sits in one function of the reader, over a list naming each source that carries a
/// pointer (`("client", &fields.client)`). A check that exists but leaves a source out of that
/// list admits that source's pointer unchecked, so every such source the model declares must be
/// named in the function that makes the check.
fn prefix_drift(
    model: &Model,
    reader: &Reader,
    name: &str,
    check: &str,
    problems: &mut Vec<String>,
) {
    let spelled = check.replace(".starts_with(", "starts_with (");
    let checking: Vec<(&String, &String)> = reader
        .recorded
        .functions
        .iter()
        .filter(|(_, body)| body.contains(&spelled))
        .collect();
    let [(function, body)] = checking.as_slice() else {
        problems.push(format!(
            "{SOURCE}: {} functions check `{check}`, the prefix {MODEL} declares for `{name}`; \
             the reader checks it in exactly one",
            checking.len()
        ));
        return;
    };
    let named: BTreeSet<&str> = body.split('"').skip(1).step_by(2).collect();
    // The structs with a field of the newtype, and then every field that holds one of them.
    let carrying: BTreeSet<String> = model
        .structs
        .iter()
        .filter(|(_, fields)| {
            fields
                .iter()
                .any(|(_, (ty, _))| *ty == format!("{PREFIX}{name}"))
        })
        .map(|(carrier, _)| format!("{PREFIX}{carrier}"))
        .collect();
    for (owner, fields) in &model.structs {
        for (field, (ty, _)) in fields {
            let held = generic(ty, "Optional").unwrap_or(ty);
            if carrying.contains(held) && !named.contains(field.as_str()) {
                problems.push(format!(
                    "{SOURCE}: `fn {function}` checks `{check}` but does not name `{owner}.{field}`, \
                     which carries a `{name}`: its pointer is admitted unchecked"
                ));
            }
        }
    }
}

/// The modelled structs, and the carriers their fields use, against the reader's.
fn struct_drift(
    model: &Model,
    reader: &Reader,
    shapes: &BTreeMap<String, Fields>,
    problems: &mut Vec<String>,
) {
    for (name, expected) in shapes {
        let Some(declared) = reader.recorded.structs.get(name) else {
            problems.push(format!("`struct {name}` is not declared in {SOURCE}"));
            continue;
        };
        for refused in &declared.refused {
            problems.push(format!(
                "{SOURCE}: `{name}: {refused}` changes the wire in a way this scan does not \
                 model; declare the shape in {MODEL} instead"
            ));
        }
        if !declared.deny_unknown_fields {
            problems.push(format!(
                "{SOURCE}: `{name}` does not refuse unknown fields (`deny_unknown_fields`)"
            ));
        }
        let mut found = Fields::new();
        for (wire, field) in &declared.fields {
            for refused in &field.refused {
                problems.push(format!(
                    "{SOURCE}: `{name}.{wire}: {refused}` changes the wire in a way this scan \
                     does not model; declare the shape in {MODEL} instead"
                ));
            }
            found.insert(
                wire.clone(),
                Shape {
                    ty: reader.resolve(&field.ty),
                    omissible: field.default_absent,
                },
            );
        }
        compare(
            problems,
            &format!("`{name}` fields (wire name, type, may be left out)"),
            &shape_set(expected),
            &shape_set(&found),
            SOURCE,
        );
    }

    for carrier in reader.carriers(model) {
        let declared = &reader.recorded.enums[carrier];
        for refused in declared.refused.iter().chain(
            declared
                .variants
                .values()
                .flat_map(|variant| variant.refused.iter()),
        ) {
            problems.push(format!(
                "{SOURCE}: `{carrier}: {refused}` changes the wire in a way this scan does not \
                 model"
            ));
        }
    }
}

/// The modelled enums against the reader's, including those it imports from `crate::history`.
fn enum_drift(model: &Model, reader: &Reader, problems: &mut Vec<String>) {
    for (name, values) in &model.enums {
        let Some((declared, file)) = reader.enumeration(name) else {
            problems.push(format!(
                "`enum {name}` is neither declared in {SOURCE} nor imported from `crate::history`"
            ));
            continue;
        };
        for refused in &declared.refused {
            problems.push(format!(
                "{file}: `{name}: {refused}` changes the wire in a way this scan does not model"
            ));
        }
        if !declared.derives_deserialize {
            problems.push(format!(
                "{file}: `{name}` does not derive `Deserialize`, so its values are not read by the \
                 wire names this scan compares"
            ));
        }
        for (wire, variant) in &declared.variants {
            for refused in &variant.refused {
                problems.push(format!(
                    "{file}: `{name}::{wire}: {refused}` changes the wire in a way this scan \
                     does not model"
                ));
            }
            if variant.payload.is_some() {
                problems.push(format!(
                    "{file}: `{name}::{wire}` carries a value; the model declares a name"
                ));
            }
        }
        compare(
            problems,
            &format!("`{name}` values"),
            values,
            &declared.variants.keys().cloned().collect(),
            file,
        );
    }
}

/// The Rust identifier of the model's one `Absent` value.
fn model_absent(model: &Model) -> String {
    let values = model
        .enums
        .get(ABSENT)
        .unwrap_or_else(|| panic!("{MODEL} declares the enum `{ABSENT}`"));
    assert_eq!(values.len(), 1, "`{ABSENT}` has exactly one value");
    let wire = values.iter().next().expect("one value");
    let mut chars = wire.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .expect("a non-empty value")
}

/// A model type in the vocabulary [`schema_kind`] reads a schema property into.
fn expected_kind(model_type: &str) -> String {
    if let Some(inner) = generic(model_type, "Optional") {
        let mut branches = [format!("ref:{ABSENT}"), expected_kind(inner)];
        branches.sort();
        return format!("oneOf[{}]", branches.join(","));
    }
    if let Some(inner) = generic(model_type, "Map") {
        let (key, value) = inner.split_once(',').expect("a map has a key and a value");
        assert_eq!(key.trim(), "String", "a JSON object's keys are strings");
        return format!("map:{}", expected_kind(value.trim()));
    }
    match model_type {
        "String" => "string".to_owned(),
        other => format!("ref:{}", local(other)),
    }
}

/// One schema property as `string`, `ref:<Definition>`, `map:<kind>` or `oneOf[<kind>,…]`;
/// anything else is spelled out as its JSON and so matches nothing expected.
fn schema_kind(property: &Json) -> String {
    let only = |keys: &[&str]| {
        property
            .as_object()
            .is_some_and(|object| object.keys().all(|key| keys.contains(&key.as_str())))
    };
    if let Some(target) = property.get("$ref").and_then(Json::as_str) {
        if only(&["$ref", "description"]) {
            return target
                .strip_prefix("#/definitions/")
                .map_or_else(|| format!("ref?{target}"), |name| format!("ref:{name}"));
        }
    }
    if let Some(branches) = property.get("oneOf").and_then(Json::as_array) {
        if only(&["oneOf", "description"]) {
            let mut kinds: Vec<String> = branches.iter().map(schema_kind).collect();
            kinds.sort();
            return format!("oneOf[{}]", kinds.join(","));
        }
    }
    if property.get("type") == Some(&Json::from("object"))
        && only(&["type", "additionalProperties", "description"])
    {
        if let Some(values) = property
            .get("additionalProperties")
            .filter(|values| values.is_object())
        {
            return format!("map:{}", schema_kind(values));
        }
    }
    if property.get("type") == Some(&Json::from("string")) && only(&["type", "description"]) {
        return "string".to_owned();
    }
    property.to_string()
}

fn definition<'a>(schema: &'a Json, name: &str) -> Option<&'a Json> {
    if name == ROOT {
        Some(schema)
    } else {
        schema.pointer(&format!("/definitions/{name}"))
    }
}

/// Every way the schema disagrees with the model. Empty is agreement.
fn schema_drift(model: &Model, schema: &Json) -> Vec<String> {
    let mut problems = Vec::new();
    struct_schema_drift(model, schema, &mut problems);
    enum_schema_drift(model, schema, &mut problems);
    newtype_schema_drift(model, schema, &mut problems);
    problems
}

/// The modelled structs against their schema objects.
fn struct_schema_drift(model: &Model, schema: &Json, problems: &mut Vec<String>) {
    for (name, fields) in &model.structs {
        let Some(definition) = definition(schema, name) else {
            problems.push(format!("{SCHEMA} does not define `definitions/{name}`"));
            continue;
        };
        let object = definition;
        if object.get("type") != Some(&Json::from("object")) {
            problems.push(format!("{SCHEMA}: `{name}` is not `type: object`"));
        }
        if object.get("additionalProperties") != Some(&Json::Bool(false)) {
            problems.push(format!(
                "{SCHEMA}: `{name}` does not refuse unknown properties"
            ));
        }
        let required: BTreeSet<&str> = object
            .get("required")
            .and_then(Json::as_array)
            .map(|names| names.iter().filter_map(Json::as_str).collect())
            .unwrap_or_default();
        let declared: BTreeSet<(String, String, bool)> = object
            .get("properties")
            .and_then(Json::as_object)
            .map(|properties| {
                properties
                    .iter()
                    .map(|(field, property)| {
                        (
                            field.clone(),
                            schema_kind(property),
                            !required.contains(field.as_str()),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let expected: BTreeSet<(String, String, bool)> = fields
            .iter()
            .map(|(field, (ty, presence))| {
                (
                    field.clone(),
                    expected_kind(ty),
                    presence.as_deref() == Some("omitted_when_absent"),
                )
            })
            .collect();
        compare(
            problems,
            &format!("`{name}` properties (name, JSON kind, may be left out)"),
            &expected,
            &declared,
            SCHEMA,
        );
        let unknown_required: Vec<&&str> = required
            .iter()
            .filter(|field| !fields.iter().any(|(declared, _)| declared == **field))
            .collect();
        if !unknown_required.is_empty() {
            problems.push(format!(
                "{SCHEMA}: `{name}` requires properties it does not declare: {unknown_required:?}"
            ));
        }
    }
}

/// The modelled enums against their schema enums.
fn enum_schema_drift(model: &Model, schema: &Json, problems: &mut Vec<String>) {
    for (name, values) in &model.enums {
        let Some(definition) = definition(schema, name) else {
            problems.push(format!("{SCHEMA} does not define `definitions/{name}`"));
            continue;
        };
        let object = definition;
        if object.get("type") != Some(&Json::from("string")) {
            problems.push(format!("{SCHEMA}: `{name}` is not `type: string`"));
        }
        match object.get("enum").and_then(Json::as_array) {
            None => problems.push(format!("{SCHEMA}: `{name}` is not an `enum`")),
            Some(found) => compare(
                problems,
                &format!("`{name}` values"),
                values,
                &found
                    .iter()
                    .filter_map(Json::as_str)
                    .map(str::to_owned)
                    .collect(),
                SCHEMA,
            ),
        }
    }
}

/// The modelled newtypes against their schema strings.
fn newtype_schema_drift(model: &Model, schema: &Json, problems: &mut Vec<String>) {
    for (name, (_, prefix)) in &model.newtypes {
        let expected = serde_json::json!({
            "type": "string",
            "pattern": format!("^{}", regex_escape(prefix.as_deref().unwrap_or(""))),
        });
        let found = definition(schema, name).map(|object| {
            let mut object = object.clone();
            if let Some(map) = object.as_object_mut() {
                map.remove("description");
            }
            object
        });
        if found.as_ref() != Some(&expected) {
            problems.push(format!(
                "{SCHEMA}: `definitions/{name}` is {found:?}; {MODEL} makes it {expected}"
            ));
        }
    }
}

fn regex_escape(literal: &str) -> String {
    literal
        .chars()
        .flat_map(|ch| {
            let escape = "\\^$.|?*+()[]{}".contains(ch);
            escape
                .then_some('\\')
                .into_iter()
                .chain(std::iter::once(ch))
        })
        .collect()
}

fn schema() -> Json {
    serde_json::from_str(&source(SCHEMA)).expect("the schema is JSON")
}

fn assert_no_drift(problems: &[String]) {
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_rust_type_carries_exactly_the_modelled_fields_types_and_values() {
    assert_no_drift(&rust_drift(
        &model(),
        &source(SOURCE),
        &source(HISTORY_SOURCE),
    ));
}

#[test]
fn the_schema_declares_exactly_the_modelled_fields_and_values() {
    assert_no_drift(&schema_drift(&model(), &schema()));
}

/// `recorded.adapter.Completion` is a copy of `concurrent.history.Completion`, and stays one.
#[test]
fn the_redeclared_completion_is_the_history_models() {
    let history: Yaml =
        serde_yaml::from_str(&source(HISTORY_MODEL)).expect("the history domain is YAML");
    let original = sequence(&history, "types")
        .iter()
        .find(|declaration| text(declaration, "name") == Some("concurrent.history.Completion"))
        .map(enum_variants)
        .expect("the history model declares Completion");
    assert_eq!(
        model().enums.get("Completion"),
        Some(&original),
        "{MODEL} redeclares `Completion` with the values {HISTORY_MODEL} declares"
    );
}

/// The scan reads the declarations it names, rather than comparing empty sets.
#[test]
fn every_declaration_this_scan_reads_is_non_empty() {
    let model = model();
    assert!(model.structs.contains_key(ROOT), "no `{ROOT}` was read");
    assert!(
        model.structs.len() >= 4,
        "fewer than four structs were read"
    );
    assert!(
        model
            .structs
            .get("Fields")
            .is_some_and(|fields| fields.len() >= 9),
        "`Fields` was read with fewer than nine sources"
    );
    assert!(model.enums.len() >= 3, "fewer than three enums were read");
    assert!(!model.newtypes.is_empty(), "no newtype was read");
    let reader = Reader::read(&source(SOURCE), &source(HISTORY_SOURCE));
    assert_eq!(
        reader.carriers(&model).len(),
        2,
        "the two `Optional` carriers (field and completion) were not both found"
    );
    assert!(
        reader.recorded.from_history.contains("Completion"),
        "the `use crate::history` import was not read"
    );
}

/// The real source with one exact edit applied, which must be present to apply.
fn mutant(relative: &str, from: &str, to: &str) -> String {
    let real = source(relative);
    assert_eq!(
        real.matches(from).count(),
        1,
        "the mutant's anchor `{from}` occurs exactly once in {relative}"
    );
    real.replacen(from, to, 1)
}

fn assert_caught(recorded: &str, history: &str, expected: &str) {
    let problems = rust_drift(&model(), recorded, history);
    assert!(
        problems.iter().any(|problem| problem.contains(expected)),
        "the drift scan missed a mutant; expected a problem mentioning `{expected}`, got \
         {problems:?}"
    );
}

fn assert_recorded_caught(from: &str, to: &str, expected: &str) {
    assert_caught(&mutant(SOURCE, from, to), &source(HISTORY_SOURCE), expected);
}

#[test]
fn a_lost_field_is_caught() {
    assert_recorded_caught("    pub outcome: FieldSource,\n", "", "\"outcome\"");
}

#[test]
fn a_gained_field_is_caught() {
    assert_recorded_caught(
        "    pub outcome: FieldSource,\n",
        "    pub outcome: FieldSource,\n    pub retry_of: FieldSource,\n",
        "\"retry_of\"",
    );
}

#[test]
fn a_gained_format_value_is_caught() {
    assert_recorded_caught(
        "    V1,\n}",
        "    V1,\n    #[serde(rename = \"ess-history-adapter/2\")]\n    V2,\n}",
        "ess-history-adapter/2",
    );
}

#[test]
fn a_gained_completion_value_is_caught() {
    assert_caught(
        &source(SOURCE),
        &mutant(
            HISTORY_SOURCE,
            "    Indeterminate,\n}",
            "    Indeterminate,\n    Cancelled,\n}",
        ),
        "Cancelled",
    );
}

#[test]
fn a_third_way_to_write_a_source_is_caught() {
    assert_recorded_caught(
        "    Pointer(Pointer),\n}",
        "    Pointer(Pointer),\n    Literal(String),\n}",
        "Carrier",
    );
}

/// A derived `Deserialize` on a source reads serde's externally tagged form (`{Pointer: {…}}`),
/// not `absent` or `{ pointer: /… }`.
#[test]
fn a_source_read_by_a_derived_deserialize_is_caught() {
    assert_recorded_caught(
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum FieldSource {",
        "#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]\npub enum FieldSource {",
        "\"FieldSource\"",
    );
}

/// `#[serde(untagged)]` reads through serde's buffer, which admits `{absent: null}` and a struct
/// written as a list of its fields.
#[test]
fn an_untagged_source_is_caught() {
    assert_recorded_caught(
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum FieldSource {",
        "#[derive(Debug, Clone, PartialEq, Eq)]\n#[serde(untagged)]\npub enum FieldSource {",
        "FieldSource: untagged",
    );
}

/// A source with no reader of its own is not a carrier.
#[test]
fn a_source_without_its_hand_written_reader_is_caught() {
    assert_recorded_caught(
        "impl<'de> Deserialize<'de> for CompletionSource {",
        "impl<'de> Deserialize<'de> for CompletionSourceUnused {",
        "\"CompletionSource\"",
    );
}

#[test]
fn a_required_source_that_may_be_left_out_is_caught() {
    assert_recorded_caught(
        "    pub client: FieldSource,",
        "    #[serde(default = \"absent\")]\n    pub client: FieldSource,",
        "\"client\"",
    );
}

#[test]
fn an_omissible_source_that_must_be_written_is_caught() {
    assert_recorded_caught(
        "    #[serde(default = \"absent\")]\n    pub rows: FieldSource,",
        "    pub rows: FieldSource,",
        "\"rows\"",
    );
}

#[test]
fn a_map_of_the_wrong_value_type_is_caught() {
    assert_recorded_caught(
        "pub values: Words<Completion>,",
        "pub values: Words<String>,",
        "Words<String>",
    );
}

#[test]
fn unknown_fields_admitted_are_caught() {
    assert_recorded_caught(
        "#[serde(deny_unknown_fields)]\npub struct Pointer {",
        "pub struct Pointer {",
        "`Pointer` does not refuse unknown fields",
    );
}

#[test]
fn an_unmodelled_serde_key_is_caught() {
    assert_recorded_caught(
        "#[serde(deny_unknown_fields)]\npub struct Fields {",
        "#[serde(deny_unknown_fields, rename_all = \"camelCase\")]\npub struct Fields {",
        "Fields: rename_all",
    );
    assert_recorded_caught(
        "    pub command: FieldSource,",
        "    #[serde(alias = \"cmd\")]\n    pub command: FieldSource,",
        "Fields.command: alias",
    );
    assert_recorded_caught(
        "#[serde(deny_unknown_fields)]\npub struct Fields {",
        "#[serde(deny_unknown_fields)]\n#[cfg_attr(all(), serde(rename_all = \"camelCase\"))]\npub struct Fields {",
        "cfg_attr",
    );
}

/// The check stands, but one source is left out of the list it runs over.
#[test]
fn a_source_left_out_of_the_prefix_check_is_caught() {
    assert_recorded_caught("        (\"rows\", &fields.rows),\n", "", "Fields.rows");
    assert_recorded_caught(
        "Some((\"completion\", &mapped.pointer))",
        "Some((\"completion_pointer\", &mapped.pointer))",
        "Fields.completion",
    );
}

/// A second copy of the check elsewhere is not the check.
#[test]
fn a_prefix_check_outside_the_reader_is_caught() {
    assert_recorded_caught(
        "fn absent() -> FieldSource {\n",
        "fn decoy(text: &str) -> bool {\n    text.starts_with('/')\n}\n\nfn absent() -> FieldSource {\n",
        "2 functions check",
    );
}

#[test]
fn a_dropped_prefix_check_is_caught() {
    assert_recorded_caught(
        "if !pointer.starts_with('/') {",
        "if pointer.is_empty() {",
        "starts_with('/')",
    );
}

/// Schema mutants: the pointer replaced, what replaces it, and what that says.
fn schema_mutants() -> Vec<(&'static str, Json, &'static str)> {
    vec![
        (
            "/definitions/Fields/properties/client",
            serde_json::json!({ "$ref": "#/definitions/Pointer" }),
            "Fields.client without `absent`",
        ),
        (
            "/definitions/MappedCompletion/properties/values",
            serde_json::json!({ "type": "object", "additionalProperties": { "type": "string" } }),
            "MappedCompletion.values of any string",
        ),
        (
            "/definitions/AdapterFormat/enum",
            serde_json::json!(["ess-history-adapter/1", "ess-history-adapter/2"]),
            "a second format",
        ),
        (
            "/definitions/JsonPointer/pattern",
            serde_json::json!("^"),
            "a pointer of any spelling",
        ),
        (
            "/definitions/Pointer/additionalProperties",
            serde_json::json!(true),
            "a pointer with unknown keys",
        ),
        (
            "/definitions/Pointer",
            serde_json::json!({
                "oneOf": [
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["pointer"],
                        "properties": { "pointer": { "$ref": "#/definitions/JsonPointer" } }
                    },
                    {
                        "type": "array",
                        "items": [{ "$ref": "#/definitions/JsonPointer" }],
                        "additionalItems": false,
                        "minItems": 1
                    }
                ]
            }),
            "a pointer also written as a list, which the reader refuses",
        ),
        (
            "/definitions/Absent",
            serde_json::json!({
                "oneOf": [
                    { "type": "string", "enum": ["absent"] },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["absent"],
                        "properties": { "absent": { "type": "null" } }
                    }
                ]
            }),
            "absent also written as a one-key object, which the reader refuses",
        ),
        (
            "/definitions/Completion/enum",
            serde_json::json!(["Returned", "Indeterminate", "Cancelled"]),
            "a completion the reader refuses",
        ),
    ]
}

#[test]
fn schema_drift_is_caught() {
    let mut missed = Vec::new();
    for (pointer, replacement, what) in schema_mutants() {
        let mut altered = schema();
        *altered
            .pointer_mut(pointer)
            .unwrap_or_else(|| panic!("the schema has {pointer}")) = replacement;
        if schema_drift(&model(), &altered).is_empty() {
            missed.push(what);
        }
    }
    let mut optional_rows = schema();
    let required = optional_rows
        .pointer_mut("/definitions/Fields/required")
        .and_then(Json::as_array_mut)
        .expect("Fields lists what it requires");
    required.push(Json::from("rows"));
    if schema_drift(&model(), &optional_rows).is_empty() {
        missed.push("rows required");
    }
    assert!(
        missed.is_empty(),
        "the schema drift scan passed: {missed:?}"
    );
}
