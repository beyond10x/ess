//! Checked type-root selection using the existing model-to-JSON wire mapping.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    ResolvedBody, ResolvedCommand, ResolvedEvent, ResolvedField, ResolvedType, ResolvedTypeRef,
    TypeHandle,
};
use ess_compiler::refs::{CommandRef, DeclaredTypeRef, EssSemanticRef, EventRef, ViewRef};
use ess_compiler::EssIr;
use ess_domain::name::QualifiedName;
use serde_json::Value;

use super::types::{self, Message};
use crate::http::Binding;
use crate::provenance::{Provenance, ProvenanceMint};

/// The member a root appends to a command's name to select its input.
const INPUT_ROOT: &str = "input";
/// The member a root appends to a command's name to select its response.
const RESPONSE_ROOT: &str = "response";
/// The member a root appends to a view's name to select one of its rows.
const ROW_ROOT: &str = "row";

/// A selection or wire-identity error, before any schema properties are collapsed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelTypeError {
    /// Qualified model type, or the root selection when no type was provided.
    pub name: String,
    /// Stable refusal category.
    pub rule: &'static str,
    /// What prevents the selected model from being realized faithfully.
    pub detail: String,
}

/// A sealed projection of selected types from a resolved model, not an imported service.
#[derive(Debug, Clone)]
pub struct ModelTypes {
    roots: BTreeSet<String>,
    definitions: BTreeMap<String, Value>,
    newtypes: BTreeSet<String>,
    binary64: BTreeSet<String>,
    /// Each object's wire names in the order the specification declares them, by the object's
    /// schema pointer (beyond10x/ess#661).
    field_order: BTreeMap<String, Vec<String>>,
    provenance: Provenance,
}

impl ModelTypes {
    /// Close explicitly selected roots through checked type handles and preserve wire mapping.
    ///
    /// A root names a declared type or an event (beyond10x/ess#393). An event's payload is selected
    /// as a struct of its fields under the event's own name and wire names, annotated
    /// `x-ess-kind: event-payload`, so a producer library holds the exact object a published event
    /// carries without the model declaring those fields twice.
    ///
    /// A root also names a message body the HTTP projection publishes (beyond10x/ess#656):
    /// `<command>.input`, `<command>.response` or `<view>.row`. Its definition is the schema that
    /// projection publishes for the body, from the same type mapping, under the root itself; a
    /// view with a declared `shape:` answers rows of that type, so its row is a reference to it.
    /// That projection is the unbound one; [`Self::select_bound`] selects under a binding.
    pub fn select(ir: &EssIr, roots: &BTreeSet<String>) -> Result<Self, Vec<ModelTypeError>> {
        Self::select_bound(ir, roots, None)
    }

    /// [`Self::select`], with each body the `OpenAPI` document projected under `binding`
    /// publishes (beyond10x/ess#656): `ess generate types --http`.
    ///
    /// A command's `<command>.input` is its `{command}.Input`, the input less every field its
    /// route carries in the path. A route that publishes no body, because it is encoded `query`,
    /// its path carries every input field or the command has none, has no `.input` root: selecting
    /// one is refused as `unpublished_body`, naming the command and why. Responses and view rows
    /// are published unchanged under a binding, so their roots are too. With `None`, it is
    /// [`Self::select`].
    pub fn select_bound(
        ir: &EssIr,
        roots: &BTreeSet<String>,
        binding: Option<&Binding>,
    ) -> Result<Self, Vec<ModelTypeError>> {
        let mut errors = BTreeSet::new();
        let mut selected: BTreeMap<_, ResolvedType> = BTreeMap::new();
        let mut events = BTreeSet::new();
        let mut bodies: BTreeMap<String, BodyRoot<'_>> = BTreeMap::new();
        if roots.is_empty() {
            errors.insert(ModelTypeError {
                name: String::new(),
                rule: "empty_roots",
                detail: "select at least one model type".to_owned(),
            });
        }
        for root in roots {
            if let Err(error) =
                select_root(ir, binding, root, &mut selected, &mut events, &mut bodies)
            {
                errors.insert(error);
            }
        }
        for declared in selected.values() {
            if let ResolvedBody::Struct { fields, .. } = &declared.body {
                wire_collisions(&declared.name.to_string(), fields, &mut errors);
            }
        }
        for (root, body) in &bodies {
            wire_collisions(root, &body.fields, &mut errors);
        }
        if !errors.is_empty() {
            return Err(errors.into_iter().collect());
        }
        let provenance = ProvenanceMint::new(ir)
            .of_seeds(
                selected
                    .keys()
                    .map(|name| {
                        if events.contains(name) {
                            EventRef::new(name.clone()).into()
                        } else {
                            DeclaredTypeRef::new(name.clone()).into()
                        }
                    })
                    .chain(bodies.values().map(|body| body.seed.clone())),
            )
            .provenance;
        let mut binary64 = BTreeSet::new();
        for item in selected.values() {
            body_numeric_locations(item, &mut binary64);
        }
        for (root, body) in &bodies {
            fields_numeric_locations(&pointer("/$defs", root), &body.fields, &mut binary64);
        }
        // Minted from the typed fields: the definitions below are sorted JSON maps.
        let mut field_order = BTreeMap::new();
        for item in selected.values() {
            if let ResolvedBody::Struct { fields, .. } = &item.body {
                field_order.insert(
                    pointer("/$defs", &item.name.to_string()),
                    declared_order(fields),
                );
            }
        }
        for (root, body) in &bodies {
            // A row of a view with a `shape:` is a reference, not an object of its own.
            if !body.fields.is_empty() {
                field_order.insert(pointer("/$defs", root), declared_order(&body.fields));
            }
        }
        Ok(Self {
            binary64,
            field_order,
            roots: roots.clone(),
            definitions: selected
                .values()
                .map(|item| {
                    let mut definition = serde_json::to_value(types::body(ir, item))
                        .expect("typed schema nodes serialize");
                    if events.contains(&item.name) {
                        definition["x-ess-kind"] = Value::from("event-payload");
                    }
                    (item.name.to_string(), definition)
                })
                .chain(
                    bodies
                        .into_iter()
                        .map(|(root, body)| (root, body.definition)),
                )
                .collect(),
            newtypes: selected
                .values()
                .filter(|item| matches!(item.body, ResolvedBody::Newtype { .. }))
                .map(|item| item.name.to_string())
                .collect(),
            provenance,
        })
    }

    /// Every command body root: each command's `<command>.input`, and its `<command>.response`
    /// when it declares a response, as the JSON Schema projection publishes them.
    pub fn command_roots(ir: &EssIr) -> BTreeSet<String> {
        Self::bound_command_roots(ir, None)
    }

    /// [`Self::command_roots`] under `binding`, as [`Self::select_bound`] selects them: a command
    /// whose route publishes no request body has no `<command>.input`. With `None`, it is
    /// [`Self::command_roots`].
    pub fn bound_command_roots(ir: &EssIr, binding: Option<&Binding>) -> BTreeSet<String> {
        ir.commands()
            .values()
            .flat_map(|command| {
                let input = binding
                    .is_none_or(|binding| bound_input(command, binding).is_ok())
                    .then(|| format!("{}.{INPUT_ROOT}", command.name));
                let response = (!command.response.is_empty())
                    .then(|| format!("{}.{RESPONSE_ROOT}", command.name));
                input.into_iter().chain(response)
            })
            .collect()
    }

    /// Every view's `<view>.row` root.
    pub fn view_roots(ir: &EssIr) -> BTreeSet<String> {
        ir.views()
            .values()
            .map(|view| format!("{}.{ROW_ROOT}", view.name))
            .collect()
    }

    /// Exact selected roots, distinct from their transitive closure.
    pub fn roots(&self) -> &BTreeSet<String> {
        &self.roots
    }

    /// The complete reference closure, using the existing JSON Schema wire mapping.
    pub fn definitions(&self) -> &BTreeMap<String, Value> {
        &self.definitions
    }

    /// Nominal model identities that structural host aliases alone cannot enforce.
    pub fn newtypes(&self) -> &BTreeSet<String> {
        &self.newtypes
    }

    /// Exact schema-node locations whose finite Binary64 identity was minted from checked types.
    /// These are private projection metadata, never recovered from serialized annotations.
    pub fn binary64_locations(&self) -> &BTreeSet<String> {
        &self.binary64
    }

    /// Each selected object's wire names in the order the specification declares them, by the
    /// object's schema pointer: a struct's and an event payload's under its definition, a
    /// command's input and response and a view's row under its root (beyond10x/ess#661). A
    /// union variant's object has no declared order and no entry. Private projection metadata
    /// minted from the typed fields, since [`Self::definitions`] lists properties by name.
    pub fn declared_field_order(&self) -> &BTreeMap<String, Vec<String>> {
        &self.field_order
    }

    /// Existing model and contract provenance, minted from the resolved input.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Retain the projected definitions and model provenance as a JSON Schema document.
    pub fn to_json(&self) -> String {
        let document = serde_json::json!({
            "$schema": super::DIALECT,
            "$defs": self.definitions,
            "x-ess-provenance": self.provenance,
        });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&document).expect("typed selection serializes")
        )
    }
}

/// An event's payload as a struct of its fields, under the event's name and naming.
/// Add one root and the named types it reaches to the selection; refused when it names nothing, or
/// a body `binding` does not publish.
fn select_root<'a>(
    ir: &'a EssIr,
    binding: Option<&Binding>,
    root: &str,
    selected: &mut BTreeMap<QualifiedName, ResolvedType>,
    events: &mut BTreeSet<QualifiedName>,
    bodies: &mut BTreeMap<String, BodyRoot<'a>>,
) -> Result<(), ModelTypeError> {
    let declared = if let Some(declared) = ir
        .types()
        .values()
        .find(|item| item.name.to_string() == root)
    {
        declared.clone()
    } else if let Some(event) = ir
        .events()
        .values()
        .find(|item| item.name.to_string() == root)
    {
        events.insert(event.name.clone());
        event_payload(event)
    } else if let Some(body) = body_root(ir, binding, root) {
        let body = body?;
        for handle in types::reachable(ir, body.leaves.iter().copied()) {
            let reached = ir.named_type(handle);
            selected.insert(reached.name.clone(), reached.clone());
        }
        bodies.insert(root.to_owned(), body);
        return Ok(());
    } else {
        return Err(ModelTypeError {
            name: root.to_owned(),
            rule: "unknown_root",
            detail: format!(
                "no resolved model type or event has this qualified name, and it is not a \
                 command's `.{INPUT_ROOT}` or `.{RESPONSE_ROOT}` or a view's `.{ROW_ROOT}`"
            ),
        });
    };
    for handle in types::reachable(ir, types::body_leaves(&declared.body)) {
        let reached = ir.named_type(handle);
        selected.insert(reached.name.clone(), reached.clone());
    }
    selected.insert(declared.name.clone(), declared);
    Ok(())
}

fn event_payload(event: &ResolvedEvent) -> ResolvedType {
    ResolvedType {
        name: event.name.clone(),
        body: ResolvedBody::Struct {
            fields: event.fields.clone(),
            invariants: Vec::new(),
        },
        naming: event.naming.clone(),
        reading: None,
    }
}

/// A message body a root names rather than a declared type: a command's input or response, or a
/// view's row (beyond10x/ess#656).
struct BodyRoot<'a> {
    /// The schema the HTTP projection publishes for the body.
    definition: Value,
    /// The fields the body carries; none for a row that is a reference to the view's shape. Owned
    /// for a bound input, which is the command's input less the fields its route's path carries.
    fields: Cow<'a, [ResolvedField]>,
    /// The named types the body mentions directly.
    leaves: Vec<&'a TypeHandle>,
    /// The construct the body belongs to, for provenance.
    seed: EssSemanticRef,
}

/// The body `root` names as `<command>.input`, `<command>.response` or `<view>.row`, if it names
/// one: refused when it names a command's input `binding` publishes no body for.
fn body_root<'a>(
    ir: &'a EssIr,
    binding: Option<&Binding>,
    root: &str,
) -> Option<Result<BodyRoot<'a>, ModelTypeError>> {
    let (owner, member) = root.rsplit_once('.')?;
    let command = || {
        ir.commands()
            .values()
            .find(|command| command.name.to_string() == owner)
    };
    let (message, seed): (Message<'a>, EssSemanticRef) = match member {
        INPUT_ROOT => {
            let command = command()?;
            if let Some(binding) = binding {
                return Some(
                    bound_input(command, binding).map_err(|reason| ModelTypeError {
                        name: root.to_owned(),
                        rule: "unpublished_body",
                        detail: format!(
                            "command `{}` has no request body under the HTTP binding: {reason}",
                            command.name
                        ),
                    }),
                );
            }
            (
                Message::of_command(command),
                CommandRef::new(command.name.clone()).into(),
            )
        }
        RESPONSE_ROOT => {
            let command = command()?;
            (
                Message::of_response(command),
                CommandRef::new(command.name.clone()).into(),
            )
        }
        ROW_ROOT => {
            let view = ir
                .views()
                .values()
                .find(|view| view.name.to_string() == owner)?;
            let seed = ViewRef::new(view.name.clone()).into();
            if let Some(shape) = &view.shape {
                return Some(Ok(BodyRoot {
                    definition: serde_json::json!({
                        "$ref": types::pointer(&ir.named_type(shape).name),
                    }),
                    fields: Cow::Borrowed(&[]),
                    leaves: vec![shape],
                    seed,
                }));
            }
            (Message::of_view(view), seed)
        }
        _ => return None,
    };
    Some(Ok(BodyRoot {
        definition: serde_json::to_value(types::message(&message))
            .expect("typed schema nodes serialize"),
        fields: Cow::Borrowed(message.fields),
        leaves: types::field_leaves(message.fields),
        seed,
    }))
}

/// `command`'s request body as the `OpenAPI` document projected under `binding` publishes it as
/// `{command}.Input` (`openapi.rs`, `command_schemas`): the input less every field the command's
/// route carries in its path, built from the same message. A command the binding does not route
/// keeps its derived route, which carries nothing in its path.
///
/// `Err` says why the document publishes no body: the route is encoded `query`, its path carries
/// every input field, or the command has no input field.
fn bound_input<'a>(
    command: &'a ResolvedCommand,
    binding: &Binding,
) -> Result<BodyRoot<'a>, String> {
    let route = binding
        .ir()
        .route(&ess_http::Operation::Command(command.name.to_string()));
    // The fields the route's parameters read, as `openapi.rs` `carried` reads them; a compiled
    // binding maps each to exactly one variable of its path (`ESS-HTTP-008`).
    let carried: BTreeSet<&str> = route
        .map(|route| {
            route
                .parameters
                .values()
                .map(ess_http::Source::field)
                .collect()
        })
        .unwrap_or_default();
    let encoding = route
        .and_then(|route| route.request)
        .unwrap_or_default()
        .encoding;
    if encoding == ess_http::Encoding::Query {
        return Err(
            "its route is encoded `query`, so every input field its path does not carry is in \
             the query string"
                .to_owned(),
        );
    }
    let body: Vec<&'a ResolvedField> = command
        .input
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .collect();
    if body.is_empty() {
        return Err(if command.input.is_empty() {
            "it declares no input field".to_owned()
        } else {
            "its route carries every input field in its path".to_owned()
        });
    }
    let leaves = body
        .iter()
        .flat_map(|&field| field.type_ref.named_leaves())
        .collect();
    let fields: Vec<ResolvedField> = body.into_iter().cloned().collect();
    let definition = serde_json::to_value(types::message(&Message {
        fields: &fields,
        ..Message::of_command(command)
    }))
    .expect("typed schema nodes serialize");
    Ok(BodyRoot {
        definition,
        fields: Cow::Owned(fields),
        leaves,
        seed: CommandRef::new(command.name.clone()).into(),
    })
}

/// Refuse two fields of one selected object that would share a wire key.
fn wire_collisions(name: &str, fields: &[ResolvedField], errors: &mut BTreeSet<ModelTypeError>) {
    let mut wires = BTreeMap::new();
    for field in fields {
        let wire = types::wire_name(field);
        if let Some(previous) = wires.insert(wire, &field.name) {
            errors.insert(ModelTypeError {
                name: name.to_owned(),
                rule: "wire_field_collision",
                detail: format!(
                    "fields {previous:?} and {:?} both use wire key {wire:?}",
                    field.name
                ),
            });
        }
    }
}

fn pointer(at: &str, key: &str) -> String {
    format!("{at}/{}", key.replace('~', "~0").replace('/', "~1"))
}

/// The wire names of `fields`, in the order they are declared.
fn declared_order(fields: &[ResolvedField]) -> Vec<String> {
    fields
        .iter()
        .map(|field| types::wire_name(field).to_owned())
        .collect()
}

fn numeric_locations(reference: &ResolvedTypeRef, at: &str, found: &mut BTreeSet<String>) {
    match reference {
        ResolvedTypeRef::Primitive {
            name: ess_domain::Primitive::Binary64,
        } => {
            found.insert(at.to_owned());
        }
        ResolvedTypeRef::Optional { of } => numeric_locations(of, &format!("{at}/anyOf/0"), found),
        ResolvedTypeRef::List { of } => numeric_locations(of, &format!("{at}/items"), found),
        ResolvedTypeRef::Map { value, .. } => {
            numeric_locations(value, &format!("{at}/additionalProperties"), found);
        }
        ResolvedTypeRef::Declared { .. } | ResolvedTypeRef::Primitive { .. } => {}
    }
}

fn fields_numeric_locations(at: &str, fields: &[ResolvedField], found: &mut BTreeSet<String>) {
    for field in fields {
        numeric_locations(
            field.type_ref.required(),
            &pointer(&format!("{at}/properties"), types::wire_name(field)),
            found,
        );
    }
}

fn body_numeric_locations(item: &ess_compiler::ir::ResolvedType, found: &mut BTreeSet<String>) {
    let at = pointer("/$defs", &item.name.to_string());
    match &item.body {
        ResolvedBody::Newtype { of, .. } => numeric_locations(of, &at, found),
        ResolvedBody::Struct { fields, .. } => fields_numeric_locations(&at, fields, found),
        ResolvedBody::Union { tag, variants } => {
            let content = types::content_key(tag);
            for (index, (_, payload)) in variants.iter().enumerate() {
                // A unit variant (ess/22) carries no number.
                let Some(payload) = payload else { continue };
                numeric_locations(
                    payload.required(),
                    &pointer(&format!("{at}/oneOf/{index}/properties"), content),
                    found,
                );
            }
        }
        ResolvedBody::Enum { .. } => {}
    }
}
