//! The names a page uses inside itself, resolved (beyond10x/ess#322).
//!
//! With a model: `type_in_model` — a page parameter's named type is a type of the document, a
//! construct of the schema, an ESS primitive, or a type, entity or view of the model; and
//! `field_in_model` — a form's fields and an action's `bind` keys are inputs of the command they
//! run, and a collection's columns, a record's fields and the `row.<field>` paths of a row
//! action's `visible` are fields of the view the composite reads.
//!
//! Without one: `overlay_params` — every key of an overlay's `params` is read inside the overlay
//! as `params.<key>`.

use std::collections::BTreeSet;

use ess_domain::types::Primitive;
use ess_ui::{
    Action, Body, Columns, Composite, Document, Field, Located, NodePath, NodeRef, Overlay,
    TypeExpr,
};

use crate::model::Model;
use crate::schema::{self, is_unmapped_marker};
use crate::walk::{body_of, in_declaration};
use crate::Sink;

pub(crate) fn run(model: &Model, document: &Document, sink: &mut Sink) {
    for (name, page) in &document.pages {
        let at = NodePath::root().child("pages").child(name).child("params");
        for (param, ty) in &page.params {
            let mut unresolved = BTreeSet::new();
            unresolved_types(model, document, ty, &mut unresolved);
            for missing in unresolved {
                sink.push(
                    "type_in_model",
                    &at.child(param),
                    format!(
                        "`{missing}` names no type of the document, no construct of the schema \
                         and no type, entity or view of the model"
                    ),
                );
            }
        }
    }
    for node in document.nodes() {
        if in_declaration(&node.path) {
            continue;
        }
        if let NodeRef::Action(action) = node.node {
            bind_keys(model, sink, &node.path, action);
        }
        if let Some(Body::Composite(composite)) = body_of(node.node) {
            composite_fields(model, sink, &node, composite);
        }
    }
}

fn unresolved_types(model: &Model, document: &Document, ty: &TypeExpr, out: &mut BTreeSet<String>) {
    match ty {
        TypeExpr::Named(name) => {
            let known = is_unmapped_marker(name)
                || schema::primitive_types().contains(name)
                || schema::construct_names().contains(name)
                || document.types.contains_key(name)
                || Primitive::parse(name).is_some()
                || model.has_type(name);
            if !known {
                out.insert(name.clone());
            }
        }
        TypeExpr::List(list) => unresolved_types(model, document, &list.list, out),
        TypeExpr::Map(map) => {
            unresolved_types(model, document, &map.map.key, out);
            unresolved_types(model, document, &map.map.value, out);
        }
        TypeExpr::Optional(optional) => {
            unresolved_types(model, document, &optional.optional, out);
        }
        TypeExpr::OneOf(one_of) => {
            for alternative in &one_of.one_of {
                unresolved_types(model, document, alternative, out);
            }
        }
        TypeExpr::Record(record) => {
            for field in record.record.values() {
                unresolved_types(model, document, field, out);
            }
        }
        TypeExpr::Enum(_) | TypeExpr::Ref(_) | TypeExpr::Const(_) => {}
    }
}

/// The input fields of the command `does` names, with its qualified name.
fn inputs<'m>(model: &'m Model, does: &str) -> Option<(String, &'m BTreeSet<String>)> {
    let qualified = model.command(does)?;
    let inputs = model.inputs.get(&qualified)?;
    Some((qualified, inputs))
}

fn listing(names: &BTreeSet<String>) -> String {
    if names.is_empty() {
        return "none".to_owned();
    }
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn bind_keys(model: &Model, sink: &mut Sink, path: &NodePath, action: &Action) {
    let Some((qualified, inputs)) = action.does.as_deref().and_then(|does| inputs(model, does))
    else {
        return;
    };
    for key in action.bind.keys() {
        if !inputs.contains(key) {
            sink.push(
                "field_in_model",
                path,
                format!(
                    "`bind` key `{key}` is no input of command `{qualified}`, whose inputs are {}",
                    listing(inputs)
                ),
            );
        }
    }
}

/// The field a column or input names: its first segment, unless it is a marker.
fn field_head(field: &Field) -> Option<&str> {
    if is_unmapped_marker(&field.field) {
        return None;
    }
    field
        .field
        .split('.')
        .next()
        .filter(|head| !head.is_empty())
}

/// The names a composite's fields must be among: a view's fields or a command's inputs.
struct Known<'m> {
    /// The view or command, qualified.
    owner: String,
    names: &'m BTreeSet<String>,
    /// `field` or `input`.
    what: &'static str,
}

impl<'m> Known<'m> {
    fn view(model: &'m Model, reads: Option<&ess_ui::Reads>) -> Option<Self> {
        let (owner, view) = model.view(reads?.view.as_deref()?)?;
        Some(Self {
            owner,
            names: &view.fields,
            what: "field",
        })
    }

    fn command(model: &'m Model, does: &str) -> Option<Self> {
        let (owner, names) = inputs(model, does)?;
        Some(Self {
            owner,
            names,
            what: "input",
        })
    }

    fn fields(&self, sink: &mut Sink, under: &NodePath, fields: &[Field]) {
        let Self { owner, names, what } = self;
        for field in fields {
            let Some(head) = field_head(field) else {
                continue;
            };
            if !names.contains(head) {
                sink.push(
                    "field_in_model",
                    &under.child(&field.name),
                    format!(
                        "`{head}` is no {what} of `{owner}`, whose {what}s are {}",
                        listing(names)
                    ),
                );
            }
        }
    }

    fn row_paths(&self, sink: &mut Sink, at: &NodePath, actions: &[Action]) {
        for action in actions {
            let Some(visible) = &action.visible else {
                continue;
            };
            for field in row_paths(&visible.0) {
                if !self.names.contains(field) {
                    sink.push(
                        "field_in_model",
                        &at.child(&action.name),
                        format!(
                            "`row.{field}` in `visible` is no field of `{}`, whose fields are {}",
                            self.owner,
                            listing(self.names)
                        ),
                    );
                }
            }
        }
    }
}

fn composite_fields(model: &Model, sink: &mut Sink, node: &Located<'_>, composite: &Composite) {
    let at = &node.path;
    match composite {
        Composite::Collection(collection) => {
            let Some(known) = Known::view(model, collection.reads.as_ref()) else {
                return;
            };
            match &collection.columns {
                Some(Columns::Fixed(columns)) => known.fields(sink, &at.child("columns"), columns),
                Some(Columns::Selectable(selectable)) => {
                    known.fields(sink, &at.child("columns").child("all"), &selectable.all);
                }
                _ => {}
            }
            known.row_paths(sink, &at.child("row_actions"), &collection.row_actions);
        }
        Composite::Record(record) => {
            if let Some(known) = Known::view(model, record.reads.as_ref()) {
                known.fields(sink, &at.child("fields"), &record.fields);
            }
        }
        Composite::Form(form) => {
            if let Some(known) = Known::command(model, &form.does) {
                known.fields(sink, &at.child("fields"), &form.fields);
            }
            for group in &form.groups {
                let does = group.does.as_deref().unwrap_or(&form.does);
                if let Some(known) = Known::command(model, does) {
                    let under = at.child("groups").child(&group.name).child("fields");
                    known.fields(sink, &under, &group.fields);
                }
            }
        }
        _ => {}
    }
}

fn is_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

/// The first segment of each `<root>.<segment>` path in an expression, where `<root>` stands
/// alone (not the tail of a longer name or path).
fn root_paths<'t>(text: &'t str, root: &str) -> Vec<&'t str> {
    let prefix = format!("{root}.");
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find(&prefix) {
        let start = from + offset;
        let name_start = start + prefix.len();
        let name_end = text[name_start..]
            .find(|character: char| !is_name_char(character))
            .map_or(text.len(), |end| name_start + end);
        let alone = text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !is_name_char(before) && before != '.');
        if alone && name_end > name_start {
            found.push(&text[name_start..name_end]);
        }
        from = name_start;
    }
    found
}

fn row_paths(text: &str) -> Vec<&str> {
    root_paths(text, "row")
}

/// `overlay_params`: every key of each overlay's `params` is read inside it as `params.<key>`.
pub(crate) fn overlay_params(located: &[Located<'_>], sink: &mut Sink) {
    for node in located {
        let NodeRef::Overlay(overlay) = node.node else {
            continue;
        };
        if overlay.params.is_empty() || in_declaration(&node.path) {
            continue;
        }
        let read = read_params(overlay);
        if read.is_none() {
            continue; // `matches(params)` reads them all
        }
        let read = read.unwrap_or_default();
        for key in overlay.params.keys() {
            if !read.contains(key.as_str()) {
                sink.push(
                    "overlay_params",
                    &node.path,
                    format!(
                        "`params.{key}` is passed to this overlay and read nowhere inside it; \
                         it reads {}",
                        if read.is_empty() {
                            "no parameter".to_owned()
                        } else {
                            read.iter()
                                .map(|name| format!("`params.{name}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ),
                );
            }
        }
    }
}

/// The parameters an overlay's body and frame read, or `None` when it reads `matches(params)`.
///
/// The typed overlay keeps no list of its expressions, and an expression may stand in almost
/// any string field, so the strings are read from the overlay's `Debug` rendering, which holds
/// every one of them verbatim. Its own `params` map is left out: a value there is evaluated by
/// the opener, not inside the overlay.
fn read_params(overlay: &Overlay) -> Option<BTreeSet<String>> {
    let rendered = format!(
        "{:?} {:?} {:?}",
        overlay.body, overlay.common, overlay.title
    );
    if rendered.contains("matches(params)") {
        return None;
    }
    Some(
        root_paths(&rendered, "params")
            .into_iter()
            .map(str::to_owned)
            .collect(),
    )
}
