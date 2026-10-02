//! Expansion of a raw document into the form [`crate::Document`] reads.
//!
//! The passes run in the one order in which every shorthand sees its final tree:
//!
//! 1. page kind inheritance and `same_as` — structure only; named lists are matched by the name
//!    each entry has or will derive, so no shorthand needs to have run;
//! 2. widget instances — the body copied and every `args.<param>` substituted;
//! 3. every local shorthand, at the positions the schema types for it, over the merged and
//!    substituted tree;
//! 4. the page-contextual shorthands (`from_page`, nav label, layout, shell).
//!
//! Every shorthand whose expansion is a template takes that template from the schema's
//! `shorthands.index` and instantiates it; the operators (`first_present`,
//! `each_value_of_enum_type`, `remove_inherited`, `merge_under`) are interpreted here.

use std::collections::{BTreeMap, BTreeSet};

use serde_yaml::{Mapping, Value};

use crate::model::{Widget, COMPOSITE_KINDS};
use crate::schema::{Schema, Shape};
use crate::{LoadError, NodePath, FORMAT};

type Result<T> = std::result::Result<T, LoadError>;

/// Keys whose values are data, types or expressions rather than document structure.
const OPAQUE: &[&str] = &[
    "types",
    "type",
    "default",
    "props",
    "fixtures",
    "channels",
    "args",
    "params",
    "bind",
    "set",
    "sets",
    "place",
    "grid",
    "tone_by",
    "variant_by",
];

pub(crate) fn expand(raw: Value, schema: &Schema) -> Result<Value> {
    let root = NodePath::root();
    let Value::Mapping(mut document) = raw else {
        return Err(LoadError::new(root, "a document is a map"));
    };
    match document.get("format") {
        Some(Value::String(format)) if format == FORMAT => {}
        other => {
            return Err(LoadError::new(
                root.child("format"),
                format!("the format is `{FORMAT}`, not {other:?}"),
            ))
        }
    }
    refuse_member_named_widgets(&document)?;

    // 1. page kinds and same_as
    let kinds = Kinds {
        declared: document
            .get("page_kinds")
            .and_then(Value::as_mapping)
            .cloned()
            .unwrap_or_default(),
        builtins: schema.builtin_kinds(),
        inherited: schema.inherited_fields(),
    };
    if let Some(Value::Mapping(declared)) = document.get_mut("page_kinds") {
        for (name, kind) in declared.iter_mut() {
            let name = key_text(name);
            *kind = Value::Mapping(kinds.resolve(&name, &mut Vec::new(), &root)?);
        }
    }
    if let Some(Value::Mapping(pages)) = document.get_mut("pages") {
        for (name, page) in pages.iter_mut() {
            let name = key_text(name);
            let at = root.child("pages").child(&name);
            let Value::Mapping(body) = page else {
                return Err(LoadError::new(at, "a page is a map"));
            };
            let kind = match body.get("kind") {
                Some(Value::String(kind)) => kind.clone(),
                _ => return Err(LoadError::new(at, "a page names its `kind`")),
            };
            let merged = kinds.merge(
                &kinds.resolve(&kind, &mut Vec::new(), &at.child("kind"))?,
                body,
                &at,
            )?;
            *page = Value::Mapping(merged);
        }
    }
    resolve_same_as(&mut document)?;

    // 2. the Composite shorthand, then widget instances
    let types = document
        .get("types")
        .and_then(Value::as_mapping)
        .cloned()
        .unwrap_or_default();
    let composites = Local {
        schema,
        types: &types,
        scope: Scope::Composites,
    };
    for key in ["shells", "pages", "widgets"] {
        if let Some(value) = document.get_mut(key) {
            composites.value(value, &root.child(key), Ctx::Plain)?;
        }
    }
    expand_widgets(&mut document)?;

    // 3. local shorthands
    let local = Local {
        schema,
        types: &types,
        scope: Scope::All,
    };
    for key in ["shells", "pages", "widgets"] {
        if let Some(value) = document.get_mut(key) {
            local.value(value, &root.child(key), Ctx::Plain)?;
        }
    }
    if let Some(Value::Mapping(declared)) = document.get_mut("widgets") {
        declaration_view(declared);
    }

    // 4. page context
    let shells = shell_context(&document);
    if let Some(Value::Mapping(pages)) = document.get_mut("pages") {
        for (_, page) in pages.iter_mut() {
            if let Value::Mapping(page) = page {
                page_context(schema, page, &shells);
            }
        }
    }
    Ok(Value::Mapping(document))
}

/// `Composite.union.other_tag_values`: a `component` names a widget only when it names no member,
/// so a widget declared under a member's name could never be used.
fn refuse_member_named_widgets(document: &Mapping) -> Result<()> {
    let Some(Value::Mapping(widgets)) = document.get("widgets") else {
        return Ok(());
    };
    for name in widgets.keys() {
        let name = key_text(name);
        if COMPOSITE_KINDS.contains(&name.as_str()) {
            return Err(LoadError::new(
                NodePath::root().child("widgets").child(&name),
                format!(
                    "`{name}` is a member of the composite union, so a widget of that name could \
                     never be used"
                ),
            ));
        }
    }
    Ok(())
}

// ── placeholders and templates ───────────────────────────────────────────────────────────────

/// What a template's placeholders resolve to.
#[derive(Default)]
struct Bindings<'a> {
    value: Option<&'a Value>,
    key: Option<&'a str>,
    page_title: Option<&'a Value>,
    action: Option<&'a Mapping>,
    item: Option<&'a Value>,
}

impl Bindings<'_> {
    fn resolve(&self, placeholder: &str) -> Option<Value> {
        match placeholder {
            "$value" => self.value.cloned(),
            "$key" => self.key.map(Value::from),
            "$page.title" => self.page_title.cloned(),
            "$item" => self.item.cloned(),
            other => {
                if let Some(field) = other.strip_prefix("$value.") {
                    self.value.and_then(|value| value.get(field)).cloned()
                } else if let Some(field) = other.strip_prefix("$action.") {
                    self.action.and_then(|action| action.get(field)).cloned()
                } else {
                    None
                }
            }
        }
        .filter(|value| !value.is_null())
    }
}

/// Instantiates an `expands_to` template; an entry whose placeholder resolves to nothing is
/// dropped (`shorthands.absent_placeholder`).
fn instantiate(template: &Value, bindings: &Bindings<'_>) -> Option<Value> {
    match template {
        Value::String(text) if text.starts_with('$') => bindings.resolve(text),
        Value::Mapping(mapping) if mapping.len() == 1 && mapping.contains_key("expr") => {
            let text = mapping["expr"].as_str()?;
            Some(Value::from(
                text.replace("$key", bindings.key.unwrap_or_default()),
            ))
        }
        Value::Mapping(mapping) => {
            let mut out = Mapping::new();
            for (key, value) in mapping {
                if let Some(value) = instantiate(value, bindings) {
                    out.insert(key.clone(), value);
                }
            }
            Some(Value::Mapping(out))
        }
        Value::Sequence(entries) => Some(Value::Sequence(
            entries
                .iter()
                .filter_map(|entry| instantiate(entry, bindings))
                .collect(),
        )),
        other => Some(other.clone()),
    }
}

/// `{first_present: [source, …]}`: the first source with a value, as a name.
///
/// A source is a dotted path into `mapping`; its last segment may be one of the two transforms
/// the schema uses, `last_segment_snake_case` and `first_key`.
fn first_present(template: &Value, mapping: &Mapping) -> Option<String> {
    template["first_present"]
        .as_sequence()?
        .iter()
        .filter_map(Value::as_str)
        .find_map(|source| lookup(mapping, source))
}

fn lookup(mapping: &Mapping, source: &str) -> Option<String> {
    let mut current = mapping.get(source.split('.').next()?)?;
    for segment in source.split('.').skip(1) {
        match segment {
            "last_segment_snake_case" => {
                return current
                    .as_str()
                    .and_then(|text| text.rsplit('.').next())
                    .map(snake_case);
            }
            "first_key" => {
                return current
                    .as_mapping()
                    .and_then(|map| map.keys().next())
                    .map(key_text);
            }
            field => current = current.get(field)?,
        }
    }
    match current {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn snake_case(text: &str) -> String {
    let mut out = String::new();
    for (index, character) in text.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    out
}

fn key_text(key: &Value) -> String {
    match key {
        Value::String(text) => text.clone(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_owned(),
    }
}

/// The path segment of a list entry: its `name`, else its `field`, else its `view`, else the
/// scalar itself (`NodePath.syntax.containers`).
pub(crate) fn entry_segment(entry: &Value) -> String {
    match entry {
        Value::Mapping(mapping) => ["name", "field", "view"]
            .iter()
            .find_map(|key| mapping.get(*key).and_then(Value::as_str))
            .unwrap_or_default()
            .to_owned(),
        Value::String(text) => text.clone(),
        _ => String::new(),
    }
}

// ── local shorthands ─────────────────────────────────────────────────────────────────────────

/// What the value being walked is, where a key alone does not say.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    /// Document structure.
    Plain,
    /// An `Action`.
    Action,
    /// An action's `export` record, whose `reads` names a view rather than holding a `Reads`.
    Export,
}

/// Which shorthands a walk applies.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// Only `Composite`, before widget expansion: a bare name must be a member of the union,
    /// never a widget, and is checked while it is still what the author wrote.
    Composites,
    /// Every local shorthand, over the merged and substituted tree.
    All,
}

struct Local<'a> {
    schema: &'a Schema,
    types: &'a Mapping,
    scope: Scope,
}

/// The `UNMAPPED: <reason>` marker (`unmapped_marker.pattern`).
fn is_marker(value: &Value) -> bool {
    value
        .as_str()
        .and_then(|text| text.strip_prefix("UNMAPPED: "))
        .is_some_and(|reason| !reason.is_empty())
}

fn is_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.-".contains(character))
}

/// Whether `value` is what a shorthand's `accepts` says; an absent value is `Null`.
fn accepts(accepted: &Value, value: &Value) -> bool {
    match accepted {
        Value::String(kind) => match kind.as_str() {
            "absent" => value.is_null(),
            "string" => value.is_string() && !is_marker(value),
            "name" => value.as_str().is_some_and(is_name),
            "expr" => matches!(value, Value::String(_) | Value::Number(_) | Value::Bool(_)),
            _ => false,
        },
        Value::Mapping(constructor) => {
            if let Some(inner) = constructor.get("optional") {
                return value.is_null() || accepts(inner, value);
            }
            if let Some(kind) = constructor.get("ref").and_then(Value::as_str) {
                return match value.as_str() {
                    Some(_) if is_marker(value) => false,
                    Some(text) if kind == "composite_kind" => COMPOSITE_KINDS.contains(&text),
                    Some(text) => !text.is_empty(),
                    None => false,
                };
            }
            if let Some(constant) = constructor.get("const") {
                return value == constant;
            }
            if let Some(Value::Mapping(fields)) = constructor.get("record") {
                let Some(written) = value.as_mapping() else {
                    return false;
                };
                return written
                    .iter()
                    .all(|(key, field)| fields.get(key).is_some_and(|ty| accepts(ty, field)))
                    && fields.iter().all(|(key, ty)| {
                        ty.get("optional").is_some() || written.contains_key(key)
                    });
            }
            false
        }
        _ => false,
    }
}

fn not_accepted(at: &NodePath, construct: &str, accepted: &Value, value: &Value) -> LoadError {
    let accepted = serde_yaml::to_string(accepted).unwrap_or_default();
    let value = serde_yaml::to_string(value).unwrap_or_default();
    LoadError::new(
        at.clone(),
        format!(
            "the {construct} shorthand accepts {}, not {}",
            accepted.trim(),
            value.trim()
        ),
    )
}

impl Local<'_> {
    fn value(&self, value: &mut Value, at: &NodePath, ctx: Ctx) -> Result<()> {
        match value {
            Value::Mapping(mapping) => self.mapping(mapping, at, ctx),
            Value::Sequence(entries) => {
                for entry in entries {
                    let here = at.child(&entry_segment(entry));
                    self.value(entry, &here, ctx)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Applies every local shorthand at the positions the schema types for it
    /// ([`Schema::positions`]): `Reads`, `Field`, `Node` (the `Composite` shorthand), `Action`
    /// (its `sets`, `confirm` and `name`) and `choice` `options`.
    fn mapping(&self, mapping: &mut Mapping, at: &NodePath, ctx: Ctx) -> Result<()> {
        let positions = self.schema.positions();
        let all = self.scope == Scope::All;
        let component = mapping
            .get("component")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let keys: Vec<String> = mapping.keys().map(key_text).collect();
        for key in keys {
            if OPAQUE.contains(&key.as_str()) {
                continue;
            }
            let here = at.child(&key);
            let is =
                |set: &BTreeSet<(String, Shape)>, shape: Shape| set.contains(&(key.clone(), shape));
            if all
                && ctx != Ctx::Export
                && is(&positions.reads, Shape::One)
                && mapping.get(key.as_str()).is_some_and(is_marker)
            {
                // `unmapped_marker.renderer: treat_as_absent`: no read, and the marker kept
                // where a node records its gaps.
                let marker = mapping.remove(key.as_str()).unwrap_or_default();
                if component.is_some() || mapping.contains_key("primitive") {
                    record_unmapped(mapping, &key, &marker);
                }
                continue;
            }
            let Some(child) = mapping.get_mut(key.as_str()) else {
                continue;
            };
            if all && ctx != Ctx::Export && is(&positions.reads, Shape::One) && child.is_string() {
                self.reads(child, &here)?;
            }
            if all && is(&positions.fields, Shape::List) {
                if let Value::Sequence(entries) = child {
                    for entry in entries.iter_mut() {
                        let entry_at = here.child(&entry_segment(entry));
                        self.field(entry, &entry_at)?;
                    }
                }
            }
            match child {
                Value::String(_) if is(&positions.nodes, Shape::One) => {
                    self.composite(child, &here)?;
                }
                Value::Sequence(entries) if is(&positions.nodes, Shape::List) => {
                    for entry in entries.iter_mut() {
                        let entry_at = here.child(&entry_segment(entry));
                        self.composite(entry, &entry_at)?;
                    }
                }
                Value::Mapping(entries) if is(&positions.nodes, Shape::Map) => {
                    for (name, entry) in entries.iter_mut() {
                        self.composite(entry, &here.child(&key_text(name)))?;
                    }
                }
                _ => {}
            }
            // A position typed `one_of: [Node, Action]` (`Tab.form`) holds an action when the
            // value names neither a component nor a primitive.
            let mut child_ctx = if ctx == Ctx::Action && key == "export" {
                Ctx::Export
            } else {
                Ctx::Plain
            };
            if is(&positions.actions, Shape::One) {
                if let Value::Mapping(action) = child {
                    if !action.contains_key("component") && !action.contains_key("primitive") {
                        if all {
                            self.action(action, &here)?;
                        }
                        child_ctx = Ctx::Action;
                    }
                }
            }
            if is(&positions.actions, Shape::List) {
                if let Value::Sequence(entries) = child {
                    if all {
                        for entry in entries.iter_mut() {
                            if let Value::Mapping(action) = entry {
                                self.action(action, &here)?;
                            }
                        }
                    }
                    child_ctx = Ctx::Action;
                }
            }
            if all && key == "options" && component.as_deref() == Some("choice") {
                self.options(child, &here)?;
            }
            self.value(child, &here, child_ctx)?;
        }
        Ok(())
    }

    /// `Reads`: a view name.
    fn reads(&self, reads: &mut Value, at: &NodePath) -> Result<()> {
        let accepted = self.schema.accepts("Reads", "");
        if !accepts(accepted, reads) {
            return Err(not_accepted(at, "Reads", accepted, reads));
        }
        let written = reads.clone();
        *reads = instantiate(
            self.schema.template("Reads", ""),
            &Bindings {
                value: Some(&written),
                ..Bindings::default()
            },
        )
        .unwrap_or_default();
        Ok(())
    }

    /// `Field`: a bare name, and an absent `name`.
    fn field(&self, entry: &mut Value, at: &NodePath) -> Result<()> {
        if !entry.is_mapping() {
            let accepted = self.schema.accepts("Field", "");
            if !accepts(accepted, entry) {
                return Err(not_accepted(at, "Field", accepted, entry));
            }
            let written = entry.clone();
            *entry = instantiate(
                self.schema.template("Field", ""),
                &Bindings {
                    value: Some(&written),
                    ..Bindings::default()
                },
            )
            .unwrap_or_default();
        }
        if let Value::Mapping(field) = entry {
            if field.contains_key("field") && !field.contains_key("name") {
                if let Some(name) = first_present(self.schema.template("Field", "name"), field) {
                    field.insert(Value::from("name"), Value::from(name));
                }
            }
        }
        Ok(())
    }

    /// `Composite`: a bare member name where a nested node is expected. A widget is never named
    /// this way; it is used as `{component: <widget>, args: …}`.
    fn composite(&self, node: &mut Value, at: &NodePath) -> Result<()> {
        if node.is_mapping() {
            return Ok(());
        }
        let accepted = self.schema.accepts("Composite", "");
        if !accepts(accepted, node) {
            return Err(not_accepted(at, "Composite", accepted, node)
                .with_hint("a widget is used as `{component: <widget>, args: …}`"));
        }
        let written = node.clone();
        *node = instantiate(
            self.schema.template("Composite", ""),
            &Bindings {
                value: Some(&written),
                ..Bindings::default()
            },
        )
        .unwrap_or_default();
        Ok(())
    }

    /// `choice` options: a named enum type, and bare values.
    fn options(&self, options: &mut Value, at: &NodePath) -> Result<()> {
        match options {
            Value::Sequence(entries) => {
                let accepted = self.schema.accepts("choice", "options[]");
                let template = self.schema.template("choice", "options[]");
                for entry in entries.iter_mut() {
                    if entry.is_mapping() {
                        continue;
                    }
                    if !accepts(accepted, entry) {
                        return Err(not_accepted(at, "choice option", accepted, entry));
                    }
                    let written = entry.clone();
                    *entry = instantiate(
                        template,
                        &Bindings {
                            value: Some(&written),
                            ..Bindings::default()
                        },
                    )
                    .unwrap_or_default();
                }
            }
            short => {
                let accepted = self.schema.accepts("choice", "options");
                if !accepts(accepted, short) {
                    return Err(not_accepted(at, "choice options", accepted, short));
                }
                let type_name = key_text(short);
                let values = self
                    .types
                    .get(type_name.as_str())
                    .and_then(|ty| ty["enum"].as_sequence())
                    .ok_or_else(|| {
                        LoadError::new(
                            at.clone(),
                            format!("`{type_name}` is not an enum type of this document"),
                        )
                        .with_hint(
                            "a renderer runs without the ESS model, so it cannot list a model \
                             enum's variants; list them, or declare them under `types`, and \
                             `ess ui check --model` holds a form field's options to its command \
                             input's variants",
                        )
                    })?;
                let shape = &self.schema.template("choice", "options")["as"];
                *short = Value::Sequence(
                    values
                        .iter()
                        .filter_map(|item| {
                            instantiate(
                                shape,
                                &Bindings {
                                    item: Some(item),
                                    ..Bindings::default()
                                },
                            )
                        })
                        .collect(),
                );
            }
        }
        Ok(())
    }

    /// `Action`: `sets` toggles, a `confirm` record, and an absent `name`.
    fn action(&self, action: &mut Mapping, at: &NodePath) -> Result<()> {
        if let Some(Value::Mapping(sets)) = action.get_mut("sets") {
            let accepted = self.schema.accepts("Action", "sets.*");
            let template = self.schema.template("Action", "sets.*");
            for (key, value) in sets.iter_mut() {
                if accepts(accepted, value) {
                    let key = key_text(key);
                    *value = instantiate(
                        template,
                        &Bindings {
                            key: Some(&key),
                            ..Bindings::default()
                        },
                    )
                    .unwrap_or_default();
                }
            }
        }
        // The long form is `{overlay: …}`; any other map is the short form.
        if let Some(confirm @ Value::Mapping(written)) = action.get("confirm") {
            if !written.contains_key("overlay") {
                let confirm = confirm.clone();
                let accepted = self.schema.accepts("Action", "confirm");
                if !accepts(accepted, &confirm) {
                    return Err(not_accepted(
                        &at.child("confirm"),
                        "Action confirm",
                        accepted,
                        &confirm,
                    ));
                }
                let expanded = instantiate(
                    self.schema.template("Action", "confirm"),
                    &Bindings {
                        value: Some(&confirm),
                        action: Some(&*action),
                        ..Bindings::default()
                    },
                )
                .unwrap_or_default();
                action.insert(Value::from("confirm"), expanded);
            }
        }
        if !action.contains_key("name") {
            let name =
                first_present(self.schema.template("Action", "name"), action).ok_or_else(|| {
                    LoadError::new(
                        at.clone(),
                        "an action without a `name` needs one of opens, does, navigate, export, \
                         upload or sets to derive it from; a `copy` action is named explicitly",
                    )
                })?;
            action.insert(Value::from("name"), Value::from(name));
        }
        Ok(())
    }
}

/// Appends `<key>: <reason>` to a node's `unmapped` list.
fn record_unmapped(mapping: &mut Mapping, key: &str, marker: &Value) {
    let note = format!("{key}: {}", marker.as_str().unwrap_or_default());
    match mapping.get_mut("unmapped") {
        Some(Value::Sequence(notes)) => notes.push(Value::from(note)),
        _ => {
            mapping.insert(
                Value::from("unmapped"),
                Value::Sequence(vec![Value::from(note)]),
            );
        }
    }
}

/// A widget declaration as the typed model reads it. A declaration that reads as written is
/// kept. One that does not, only because a body position holds exactly `args.<param>` for a
/// param typed as a list, map or record, is read with those positions left out: only a use
/// site, after substitution, has the value such a position needs, and every use is read in full.
/// Otherwise the declaration is kept as written and its own error is reported.
fn declaration_view(widgets: &mut Mapping) {
    for (_, widget) in widgets.iter_mut() {
        let structural: Vec<String> = widget
            .get("params")
            .and_then(Value::as_mapping)
            .map(|params| {
                params
                    .iter()
                    .filter(|(_, spec)| {
                        spec.get("type")
                            .and_then(Value::as_mapping)
                            .is_some_and(|ty| {
                                ["list", "map", "record"]
                                    .iter()
                                    .any(|constructor| ty.contains_key(*constructor))
                            })
                    })
                    .map(|(name, _)| format!("args.{}", key_text(name)))
                    .collect()
            })
            .unwrap_or_default();
        if structural.is_empty() || serde_yaml::from_value::<Widget>(widget.clone()).is_ok() {
            continue;
        }
        let mut stripped = widget.clone();
        if let Some(body) = stripped.get_mut("body") {
            strip_args(body, &structural);
        }
        if serde_yaml::from_value::<Widget>(stripped.clone()).is_ok() {
            *widget = stripped;
        }
    }
}

fn strip_args(value: &mut Value, structural: &[String]) {
    match value {
        Value::Mapping(mapping) => {
            mapping.retain(|_, child| {
                !child
                    .as_str()
                    .is_some_and(|text| structural.iter().any(|arg| arg == text))
            });
            for (key, child) in mapping.iter_mut() {
                if key.as_str() != Some("args") {
                    strip_args(child, structural);
                }
            }
        }
        Value::Sequence(entries) => {
            for entry in entries {
                strip_args(entry, structural);
            }
        }
        _ => {}
    }
}

// ── pass 2: page kinds ───────────────────────────────────────────────────────────────────────

struct Kinds {
    declared: Mapping,
    builtins: Mapping,
    inherited: Vec<String>,
}

impl Kinds {
    /// A kind merged over the chain of kinds it extends.
    fn resolve(&self, name: &str, stack: &mut Vec<String>, at: &NodePath) -> Result<Mapping> {
        if stack.iter().any(|seen| seen == name) {
            return Err(LoadError::new(
                at.clone(),
                format!("page kinds extend each other in a cycle through `{name}`"),
            ));
        }
        if let Some(Value::Mapping(body)) = self.declared.get(name) {
            let here = NodePath::root().child("page_kinds").child(name);
            stack.push(name.to_owned());
            let base = match body.get("extends") {
                Some(Value::String(parent)) => {
                    self.resolve(parent, stack, &here.child("extends"))?
                }
                Some(_) => {
                    return Err(LoadError::new(
                        here.child("extends"),
                        "`extends` names a page kind",
                    ))
                }
                None => Mapping::new(),
            };
            stack.pop();
            self.merge(&base, body, &here)
        } else if let Some(Value::Mapping(body)) = self.builtins.get(name) {
            Ok(body.clone())
        } else {
            Err(LoadError::new(
                at.clone(),
                format!("no page kind `{name}`, built in or declared"),
            ))
        }
    }

    /// `over` merged over the inherited fields of `base` (`shorthands.inheritance`).
    fn merge(&self, base: &Mapping, over: &Mapping, at: &NodePath) -> Result<Mapping> {
        let mut out = Mapping::new();
        for (key, value) in over {
            if !self.inherited.contains(&key_text(key)) {
                out.insert(key.clone(), value.clone());
            }
        }
        for field in &self.inherited {
            let merged = match (base.get(field.as_str()), over.get(field.as_str())) {
                (Some(_) | None, Some(Value::Null)) | (None, None) => None,
                (Some(inherited), Some(own)) => {
                    Some(merge_value(field, inherited, own, &at.child(field))?)
                }
                (Some(inherited), None) => Some(inherited.clone()),
                (None, Some(own)) => Some(own.clone()),
            };
            if let Some(merged) = merged {
                out.insert(Value::from(field.as_str()), merged);
            }
        }
        Ok(out)
    }
}

fn merge_value(key: &str, base: &Value, over: &Value, at: &NodePath) -> Result<Value> {
    if key == "layout" {
        return Ok(over.clone());
    }
    match (base, over) {
        (Value::Mapping(base), Value::Mapping(over)) => {
            Ok(Value::Mapping(merge_map(base, over, at)?))
        }
        (Value::Sequence(base), Value::Sequence(over))
            if named_list(key, base) && named_list(key, over) =>
        {
            Ok(Value::Sequence(merge_named(key, base, over, at)?))
        }
        _ => Ok(over.clone()),
    }
}

fn merge_map(base: &Mapping, over: &Mapping, at: &NodePath) -> Result<Mapping> {
    let mut out = Mapping::new();
    for (key, inherited) in base {
        let name = key_text(key);
        match over.get(key) {
            Some(Value::Null) => {}
            Some(own) => {
                out.insert(
                    key.clone(),
                    merge_value(&name, inherited, own, &at.child(&name))?,
                );
            }
            None => {
                out.insert(key.clone(), inherited.clone());
            }
        }
    }
    for (key, own) in over {
        if !base.contains_key(key) && !own.is_null() {
            out.insert(key.clone(), own.clone());
        }
    }
    Ok(out)
}

/// The name a list entry under `key` has or will derive: its `name`; for a `Field` list its
/// `field` or the bare name written; for an `Action` list the name `Action` at `name` derives.
/// `None` for an entry that has none, which makes the list merge by replacement.
fn entry_key(key: &str, entry: &Value) -> Option<String> {
    if let Some(name) = entry.get("name").and_then(Value::as_str) {
        return Some(name.to_owned());
    }
    let schema = Schema::embedded();
    let positions = schema.positions();
    let listed = |set: &BTreeSet<(String, Shape)>| set.contains(&(key.to_owned(), Shape::List));
    if listed(&positions.fields) {
        return match entry {
            Value::String(name) => Some(name.clone()),
            other => other
                .get("field")
                .and_then(Value::as_str)
                .map(str::to_owned),
        };
    }
    if listed(&positions.actions) {
        return entry
            .as_mapping()
            .and_then(|action| first_present(schema.template("Action", "name"), action));
    }
    None
}

fn named_list(key: &str, entries: &[Value]) -> bool {
    entries.iter().all(|entry| entry_key(key, entry).is_some())
}

fn removes(entry: &Value) -> bool {
    entry.get("remove").and_then(Value::as_bool) == Some(true)
}

/// Refuses two entries of one list that share a name (`names_unique`), before a merge could
/// fold them into one.
fn unique_keys(key: &str, entries: &[Value], at: &NodePath) -> Result<()> {
    let mut seen = BTreeSet::new();
    for entry in entries {
        if let Some(name) = entry_key(key, entry) {
            if !seen.insert(name.clone()) {
                return Err(LoadError::new(
                    at.child(&name),
                    "two siblings share this name (names_unique)",
                ));
            }
        }
    }
    Ok(())
}

/// Named lists merge by name; `{name: n, remove: true}` removes the inherited entry `n`.
fn merge_named(key: &str, base: &[Value], over: &[Value], at: &NodePath) -> Result<Vec<Value>> {
    unique_keys(key, base, at)?;
    unique_keys(key, over, at)?;
    // `{name: n, remove: true}` is the whole short form; anything beside it is refused.
    let (construct, position) = if key == "choices" {
        ("filter_bar", "choices[]")
    } else {
        ("Page", "sections[]")
    };
    let accepted = Schema::embedded().accepts(construct, position);
    for own in over.iter().filter(|own| removes(own)) {
        if !accepts(accepted, own) {
            return Err(not_accepted(
                &at.child(&entry_key(key, own).unwrap_or_default()),
                "removal",
                accepted,
                own,
            ));
        }
    }
    let name_of = |entry: &Value| entry_key(key, entry).unwrap_or_default();
    let mut out = Vec::new();
    for inherited in base {
        let name = name_of(inherited);
        match over.iter().find(|own| name_of(own) == name) {
            Some(own) if removes(own) => {}
            Some(own) => {
                let replaces = matches!(
                    (inherited.get("component"), own.get("component")),
                    (Some(a), Some(b)) if a != b
                );
                match (inherited, own) {
                    (Value::Mapping(inherited), Value::Mapping(own)) if !replaces => {
                        out.push(Value::Mapping(merge_map(inherited, own, &at.child(&name))?));
                    }
                    _ => out.push(own.clone()),
                }
            }
            None => out.push(inherited.clone()),
        }
    }
    for own in over {
        let name = name_of(own);
        if base.iter().any(|inherited| name_of(inherited) == name) {
            continue;
        }
        if removes(own) {
            return Err(LoadError::new(
                at.child(&name),
                "`remove: true` names no inherited entry",
            ));
        }
        out.push(own.clone());
    }
    Ok(out)
}

// ── pass 3: page context and same_as ─────────────────────────────────────────────────────────

/// The values `Page.shell`'s `first_present` sources read: `shells.<name>` and `only_shell`.
fn shell_context(document: &Mapping) -> Mapping {
    let names: Vec<String> = document
        .get("shells")
        .and_then(Value::as_mapping)
        .map(|shells| shells.keys().map(key_text).collect())
        .unwrap_or_default();
    let mut shells = Mapping::new();
    for name in &names {
        shells.insert(Value::from(name.as_str()), Value::from(name.as_str()));
    }
    let mut context = Mapping::new();
    context.insert(Value::from("shells"), Value::Mapping(shells));
    if let [only] = names.as_slice() {
        context.insert(Value::from("only_shell"), Value::from(only.as_str()));
    }
    context
}

fn page_context(schema: &Schema, page: &mut Mapping, shells: &Mapping) {
    let title = page.get("title").cloned();
    let bindings = Bindings {
        page_title: title.as_ref(),
        ..Bindings::default()
    };
    if let Some(Value::Mapping(header)) = page.get_mut("header") {
        if header.get("title").and_then(Value::as_str) == Some("from_page") {
            match instantiate(schema.template("header", "title"), &bindings) {
                Some(title) => header.insert(Value::from("title"), title),
                None => header.remove("title"),
            };
        }
    }
    if let Some(Value::Mapping(nav)) = page.get_mut("nav") {
        if !nav.contains_key("label") {
            if let Some(label) = instantiate(schema.template("NavEntry", "label"), &bindings) {
                nav.insert(Value::from("label"), label);
            }
        }
    }
    if !page.contains_key("layout") {
        let layout = instantiate(schema.template("PageLayout", ""), &bindings).unwrap_or_default();
        page.insert(Value::from("layout"), layout);
    }
    if !page.contains_key("shell") {
        if let Some(shell) = first_present(schema.template("Page", "shell"), shells) {
            page.insert(Value::from("shell"), Value::from(shell));
        }
    }
}

/// `same_as: <page>.<overlay>`: the named overlay copied, the local props merged over it.
fn resolve_same_as(document: &mut Mapping) -> Result<()> {
    let snapshot = document
        .get("pages")
        .and_then(Value::as_mapping)
        .cloned()
        .unwrap_or_default();
    for container in ["pages", "shells"] {
        let Some(Value::Mapping(owners)) = document.get_mut(container) else {
            continue;
        };
        for (owner, body) in owners.iter_mut() {
            let owner = key_text(owner);
            let Some(Value::Mapping(overlays)) = body.get_mut("overlays") else {
                continue;
            };
            for (name, overlay) in overlays.iter_mut() {
                let at = NodePath::root()
                    .child(container)
                    .child(&owner)
                    .child("overlays")
                    .child(&key_text(name));
                if let Value::Mapping(local) = overlay {
                    if local.contains_key("same_as") {
                        *local = same_as(local, &snapshot, &at, 0)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn same_as(local: &Mapping, pages: &Mapping, at: &NodePath, depth: usize) -> Result<Mapping> {
    let Some(Value::String(target)) = local.get("same_as") else {
        return Ok(local.clone());
    };
    if depth > 8 {
        return Err(LoadError::new(
            at.child("same_as"),
            "`same_as` chains loop or run deeper than 8",
        ));
    }
    let unresolved = || {
        LoadError::new(
            at.child("same_as"),
            format!("`{target}` names no overlay (`<page>.<overlay>`)"),
        )
    };
    let (page, overlay) = target.rsplit_once('.').ok_or_else(unresolved)?;
    let Some(Value::Mapping(inherited)) = pages
        .get(page)
        .and_then(|page| page.get("overlays"))
        .and_then(|overlays| overlays.get(overlay))
    else {
        return Err(unresolved());
    };
    let inherited = same_as(inherited, pages, at, depth + 1)?;
    let mut merged = merge_map(&inherited, local, at)?;
    merged.insert(Value::from("same_as"), Value::from(target.as_str()));
    Ok(merged)
}

// ── pass 4: widget instances ─────────────────────────────────────────────────────────────────

/// Gives every widget use under `shells` and `pages` its expanded body, within one
/// [`Budget`] for the document.
fn expand_widgets(document: &mut Mapping) -> Result<()> {
    let root = NodePath::root();
    let widgets = document
        .get("widgets")
        .and_then(Value::as_mapping)
        .cloned()
        .unwrap_or_default();
    let mut budget = Budget::default();
    for key in ["shells", "pages"] {
        if let Some(value) = document.get_mut(key) {
            widget_uses(
                value,
                &root.child(key),
                &widgets,
                &mut Vec::new(),
                &mut budget,
            )?;
        }
    }
    Ok(())
}

/// The most YAML values the widget bodies of one document may expand to, all uses together.
///
/// A widget may use another more than once, so expansion can grow exponentially with nesting
/// depth while every widget is still free of cycles (beyond10x/ess#300). The limit keeps loading
/// a document linear in its size: the use that would pass it is refused, after at most this many
/// values have been copied.
pub(crate) const EXPANSION_LIMIT: usize = 100_000;

/// What is left of [`EXPANSION_LIMIT`] for the document.
struct Budget {
    left: usize,
    exceeded: bool,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            left: EXPANSION_LIMIT,
            exceeded: false,
        }
    }
}

fn exceeded(widget: &str, at: &NodePath) -> LoadError {
    LoadError::new(
        at.clone(),
        format!(
            "widget `{widget}`: widget expansion exceeds {EXPANSION_LIMIT} values for the \
             document; a widget used more than once at each level of nesting grows exponentially"
        ),
    )
}

/// The number of YAML values in `value`, itself included.
fn values_in(value: &Value) -> usize {
    1 + match value {
        Value::Mapping(mapping) => mapping
            .iter()
            .map(|(key, child)| values_in(key) + values_in(child))
            .sum(),
        Value::Sequence(entries) => entries.iter().map(values_in).sum(),
        Value::Tagged(tagged) => values_in(&tagged.value),
        _ => 0,
    }
}

fn widget_uses(
    value: &mut Value,
    at: &NodePath,
    widgets: &Mapping,
    stack: &mut Vec<String>,
    budget: &mut Budget,
) -> Result<()> {
    match value {
        Value::Mapping(mapping) => {
            for (key, child) in mapping.iter_mut() {
                let key = key_text(key);
                if OPAQUE.contains(&key.as_str()) {
                    continue;
                }
                widget_uses(child, &at.child(&key), widgets, stack, budget)?;
            }
            let widget = mapping
                .get("component")
                .and_then(Value::as_str)
                .filter(|component| !COMPOSITE_KINDS.contains(component))
                .map(str::to_owned);
            if let Some(widget) = widget {
                let body = instance_body(&widget, mapping, at, widgets, stack, budget)?;
                mapping.insert(Value::from("body"), body);
            }
            Ok(())
        }
        Value::Sequence(entries) => {
            for entry in entries {
                let here = at.child(&entry_segment(entry));
                widget_uses(entry, &here, widgets, stack, budget)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn instance_body(
    widget: &str,
    instance: &Mapping,
    at: &NodePath,
    widgets: &Mapping,
    stack: &mut Vec<String>,
    budget: &mut Budget,
) -> Result<Value> {
    let Some(definition) = widgets.get(widget) else {
        return Err(LoadError::new(
            at.child("component"),
            format!("`{widget}` names neither a member of the composite union nor a widget"),
        ));
    };
    if stack.iter().any(|seen| seen == widget) {
        return Err(LoadError::new(
            at.clone(),
            format!("widget `{widget}` contains itself"),
        ));
    }
    if instance.contains_key("body") {
        return Err(LoadError::new(
            at.child("body"),
            "`body` is written by widget expansion, never by an author",
        ));
    }
    let empty = Mapping::new();
    let args = instance
        .get("args")
        .and_then(Value::as_mapping)
        .unwrap_or(&empty);
    let params = definition
        .get("params")
        .and_then(Value::as_mapping)
        .unwrap_or(&empty);
    for key in args.keys() {
        if !params.contains_key(key) {
            let key = key_text(key);
            return Err(LoadError::new(
                at.child("args").child(&key),
                format!("widget `{widget}` has no param `{key}`"),
            ));
        }
    }
    let mut bindings = BTreeMap::new();
    for (param, spec) in params {
        let param = key_text(param);
        let bound = match (args.get(param.as_str()), spec.get("default")) {
            (Some(arg), _) => arg.clone(),
            (None, Some(default)) => default.clone(),
            (None, None) if spec.get("required").and_then(Value::as_bool) == Some(true) => {
                return Err(LoadError::new(
                    at.clone(),
                    format!("widget `{widget}` needs the argument `{param}`"),
                ));
            }
            (None, None) => Value::Null,
        };
        bindings.insert(param, bound);
    }
    let mut body = definition.get("body").cloned().unwrap_or_default();
    substitute(&mut body, &bindings);
    let Some(left) = budget.left.checked_sub(values_in(&body)) else {
        budget.exceeded = true;
        return Err(exceeded(widget, at));
    };
    budget.left = left;
    if let Some(unbound) = unbound_arg(&body) {
        let message = if bindings.contains_key(unbound.as_str()) {
            format!(
                "`args.{unbound}` of widget `{widget}` is bound to a map or list, which cannot \
                 stand inside an expression"
            )
        } else {
            format!("`args.{unbound}` names no param of widget `{widget}`")
        };
        return Err(LoadError::new(at.child("body"), message));
    }
    stack.push(widget.to_owned());
    let expanded = widget_uses(&mut body, &at.child("body"), widgets, stack, budget);
    stack.pop();
    match expanded {
        // Reported at the outermost use, the one an author can see and change, rather than at
        // the nested copy where the count happened to run out.
        Err(_) if budget.exceeded && stack.is_empty() => Err(exceeded(widget, at)),
        Err(error) => Err(error),
        Ok(()) => Ok(body),
    }
}

fn is_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

/// Each `args.<param>` token in a string, as `(start, end, param)`.
fn arg_tokens(text: &str) -> Vec<(usize, usize, &str)> {
    let mut tokens = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find("args.") {
        let start = from + offset;
        let name_start = start + "args.".len();
        let name_end = text[name_start..]
            .find(|character: char| !is_name_char(character))
            .map_or(text.len(), |end| name_start + end);
        let boundary = text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !is_name_char(before) && before != '.');
        if boundary && name_end > name_start {
            tokens.push((start, name_end, &text[name_start..name_end]));
        }
        from = name_end.max(name_start);
    }
    tokens
}

fn substitute(value: &mut Value, bindings: &BTreeMap<String, Value>) {
    match value {
        Value::String(text) => {
            if let Some(param) = text.strip_prefix("args.") {
                if let Some(bound) = bindings.get(param) {
                    *value = bound.clone();
                    return;
                }
            }
            let mut out = String::new();
            let mut last = 0;
            for (start, end, param) in arg_tokens(text) {
                let replacement = match bindings.get(param) {
                    Some(Value::String(bound)) => bound.clone(),
                    Some(Value::Number(bound)) => bound.to_string(),
                    Some(Value::Bool(bound)) => bound.to_string(),
                    // An omitted optional param without a default is absent: `null` in an expression.
                    Some(Value::Null) => "null".to_owned(),
                    _ => continue,
                };
                out.push_str(&text[last..start]);
                out.push_str(&replacement);
                last = end;
            }
            if last > 0 {
                out.push_str(&text[last..]);
                *text = out;
            }
        }
        Value::Mapping(mapping) => {
            for (_, child) in mapping.iter_mut() {
                substitute(child, bindings);
            }
        }
        Value::Sequence(entries) => {
            for entry in entries {
                substitute(entry, bindings);
            }
        }
        _ => {}
    }
}

/// The first `args.<param>` left in an expanded body.
fn unbound_arg(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => arg_tokens(text)
            .first()
            .map(|(_, _, param)| (*param).to_owned()),
        Value::Mapping(mapping) => mapping.iter().find_map(|(_, child)| unbound_arg(child)),
        Value::Sequence(entries) => entries.iter().find_map(unbound_arg),
        _ => None,
    }
}
