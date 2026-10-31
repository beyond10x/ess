//! Concrete finite-number use inventory for adapters that cannot execute its wire contract.

use crate::ir::{
    EssIr, ResolvedBody, ResolvedField, ResolvedMappingValue, ResolvedTypeRef, TypeHandle,
};
use crate::refs::{
    ActorRef, CommandRef, DeclaredTypeRef, EntityRef, ErrorRef, EssSemanticRef, EventRef, ViewRef,
};
use ess_domain::name::QualifiedName;
use ess_domain::Primitive;
use std::collections::{BTreeMap, BTreeSet};

/// Every directly declared Binary64 position in the checked model, in stable source order.
pub fn locations(ir: &EssIr) -> BTreeSet<String> {
    uses(ir).into_keys().collect()
}

/// Each concrete Binary64 position and its compiler-owned semantic owner.
/// Named references need no expansion: their checked declarations occur exactly once.
pub fn uses(ir: &EssIr) -> BTreeMap<String, EssSemanticRef> {
    uses_of(ir, Primitive::Binary64)
}

/// The Binary64 positions of [`uses`] a conformance run reaches, in stable source order.
///
/// A position whose owner is an entity, command, event, error, view or actor is reached. One in a declared
/// type is reached when that type is in the type closure of an entity (its identity and every
/// field, published or not), a command's input or response, an event, an error, a view's fields,
/// parameters or row shape, an actor's attributes, a binding's delivery context, mappings or
/// selection, or a declared conversion. Every other position is in a type nothing a scenario sends
/// or reads names; [`unreached_locations`] lists those. [`locations`] stays the structural
/// inventory a codec reads.
pub fn reached_uses(ir: &EssIr) -> BTreeMap<String, EssSemanticRef> {
    let reached = reached_types(ir);
    uses(ir)
        .into_iter()
        .filter(|(_, owner)| match owner {
            EssSemanticRef::Type { name } => reached.contains(name.name()),
            _ => true,
        })
        .collect()
}

/// Every Binary64 position of [`uses`] that [`reached_uses`] leaves out, in stable source order.
pub fn unreached_locations(ir: &EssIr) -> BTreeSet<String> {
    let reached = reached_uses(ir);
    locations(ir)
        .into_iter()
        .filter(|at| !reached.contains_key(at))
        .collect()
}

/// The Binary64 positions of [`reached_uses`] that the conformance suite contract does not carry,
/// in stable source order.
///
/// A suite carries a finite Binary64 in an entity's fields, a command's input and response, an
/// event's and an error's fields, and a view's fields and row shape. It does not carry one in an
/// entity's identity, a view's parameters, an actor's attributes, a binding's delivery context,
/// mappings or selection, or a declared conversion: a position reached through any of those is
/// listed here, even when a carried root reaches the same declared type.
pub fn uncarried_uses(ir: &EssIr) -> BTreeMap<String, EssSemanticRef> {
    let uncarried = types_reached_from(ir, Roots::Uncarried);
    let mut prefixes = Vec::new();
    for entity in ir.entities().values() {
        prefixes.push(format!("entity {}.identity", entity.name));
    }
    for view in ir.views().values() {
        prefixes.push(format!("view.{}.params.", view.name));
    }
    for actor in ir.actors().values() {
        prefixes.push(format!("actor.{}.attributes.", actor.name));
    }
    reached_uses(ir)
        .into_iter()
        .filter(|(at, owner)| match owner {
            EssSemanticRef::Type { name } => uncarried.contains(name.name()),
            _ => prefixes
                .iter()
                .any(|prefix| at.starts_with(prefix.as_str())),
        })
        .collect()
}

/// Which roots a type closure starts from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Roots {
    /// Every root a conformance run reaches.
    All,
    /// Only the roots the suite contract does not carry a Binary64 through.
    Uncarried,
}

/// The declared types a model's commands, events, errors, views, entities, actors, bindings and
/// conversions reach, through every wrapper and declaration.
fn reached_types(ir: &EssIr) -> BTreeSet<QualifiedName> {
    types_reached_from(ir, Roots::All)
}

fn types_reached_from(ir: &EssIr, which: Roots) -> BTreeSet<QualifiedName> {
    let all = which == Roots::All;
    let mut roots: Vec<&ResolvedTypeRef> = Vec::new();
    let mut named: Vec<&TypeHandle> = Vec::new();
    for entity in ir.entities().values() {
        roots.push(&entity.identity.type_ref);
        if all {
            roots.extend(entity.fields.iter().map(|field| &field.type_ref));
            named.push(&entity.state_type);
        }
    }
    if all {
        for command in ir.commands().values() {
            roots.extend(command.input.iter().map(|field| &field.type_ref));
            roots.extend(command.response.iter().map(|field| &field.type_ref));
        }
        for event in ir.events().values() {
            roots.extend(event.fields.iter().map(|field| &field.type_ref));
        }
        for error in ir.errors().values() {
            roots.extend(error.fields.iter().map(|field| &field.type_ref));
        }
    }
    for view in ir.views().values() {
        if all {
            roots.extend(view.fields.iter().map(|field| &field.type_ref));
            named.extend(view.shape.iter());
        }
        roots.extend(view.params.iter().map(|field| &field.type_ref));
    }
    for actor in ir.actors().values() {
        roots.extend(actor.attributes.iter().map(|field| &field.type_ref));
    }
    for binding in ir.bindings().values() {
        if let Some(context) = &binding.context {
            roots.extend(context.fields.iter().map(|field| &field.type_ref));
        }
        for mapping in &binding.mapping {
            roots.push(&mapping.target_type);
            match &mapping.value {
                ResolvedMappingValue::DeliveryContext { type_ref, .. }
                | ResolvedMappingValue::HostContext { type_ref, .. }
                | ResolvedMappingValue::HostRead { type_ref, .. }
                | ResolvedMappingValue::Selection { type_ref, .. }
                | ResolvedMappingValue::EventField { type_ref, .. } => roots.push(type_ref),
                ResolvedMappingValue::EventAccessor {
                    type_ref, types, ..
                } => {
                    roots.push(type_ref);
                    named.extend(types.values());
                }
                ResolvedMappingValue::Literal { .. } => {}
            }
        }
        if let Some(selection) = &binding.selection {
            named.extend(selection.types.values());
        }
    }
    for conversion in ir.conversions() {
        roots.push(&conversion.from);
        roots.push(&conversion.to);
    }
    let mut found = BTreeSet::new();
    let mut pending: Vec<&TypeHandle> = named;
    for root in roots {
        pending.extend(root.named_leaves());
    }
    while let Some(handle) = pending.pop() {
        if !found.insert(handle.name().clone()) {
            continue;
        }
        match &ir.named_type(handle).body {
            ResolvedBody::Newtype { of, .. } => pending.extend(of.named_leaves()),
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    pending.extend(field.type_ref.named_leaves());
                }
            }
            ResolvedBody::Union { variants, .. } => {
                for variant in variants.values().flatten() {
                    pending.extend(variant.named_leaves());
                }
            }
            ResolvedBody::Enum { .. } => {}
        }
    }
    found
}

/// Every directly declared position of `primitive` in the checked model, in stable source order:
/// the same inventory for another primitive an adapter may have no codec for, such as `Json`
/// (beyond10x/ess#138).
pub fn locations_of(ir: &EssIr, primitive: Primitive) -> BTreeSet<String> {
    uses_of(ir, primitive).into_keys().collect()
}

fn uses_of(ir: &EssIr, primitive: Primitive) -> BTreeMap<String, EssSemanticRef> {
    let mut found = BTreeMap::new();
    declared_type_uses(ir, primitive, &mut found);
    owner_uses(ir, primitive, &mut found);
    found
}

/// The positions of `primitive` inside declared type bodies, each owned by its type.
fn declared_type_uses(
    ir: &EssIr,
    primitive: Primitive,
    found: &mut BTreeMap<String, EssSemanticRef>,
) {
    for declared in ir.types().values() {
        let at = format!("types.{}", declared.name);
        let owner = DeclaredTypeRef::new(declared.name.clone()).into();
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => {
                reference(primitive, of, &format!("{at}.of"), &owner, found);
            }
            ResolvedBody::Struct {
                fields: members, ..
            } => fields(primitive, members, &format!("{at}.fields"), &owner, found),
            ResolvedBody::Union { variants, .. } => {
                for (name, ty) in variants.iter().filter_map(|(n, t)| Some((n, t.as_ref()?))) {
                    reference(
                        primitive,
                        ty,
                        &format!("{at}.variants.{name}"),
                        &owner,
                        found,
                    );
                }
            }
            ResolvedBody::Enum { .. } => {}
        }
    }
}

/// The positions of `primitive` an entity, command, event, error, view or actor declares itself.
fn owner_uses(ir: &EssIr, primitive: Primitive, found: &mut BTreeMap<String, EssSemanticRef>) {
    for entity in ir.entities().values() {
        let owner = EntityRef::new(entity.name.clone()).into();
        reference(
            primitive,
            &entity.identity.type_ref,
            &format!("entity {}.identity", entity.name),
            &owner,
            found,
        );
        fields(
            primitive,
            &entity.fields,
            &format!("entity {}.fields", entity.name),
            &owner,
            found,
        );
    }
    for command in ir.commands().values() {
        fields(
            primitive,
            &command.response,
            &format!("command.{}.response", command.name),
            &CommandRef::new(command.name.clone()).into(),
            found,
        );
        fields(
            primitive,
            &command.input,
            &format!("command.{}.input", command.name),
            &CommandRef::new(command.name.clone()).into(),
            found,
        );
    }
    for event in ir.events().values() {
        fields(
            primitive,
            &event.fields,
            &format!("event.{}.fields", event.name),
            &EventRef::new(event.name.clone()).into(),
            found,
        );
    }
    for error in ir.errors().values() {
        fields(
            primitive,
            &error.fields,
            &format!("error.{}.fields", error.name),
            &ErrorRef::new(error.name.clone()).into(),
            found,
        );
    }
    for view in ir.views().values() {
        let owner = ViewRef::new(view.name.clone()).into();
        fields(
            primitive,
            &view.fields,
            &format!("view.{}.fields", view.name),
            &owner,
            found,
        );
        fields(
            primitive,
            &view.params,
            &format!("view.{}.params", view.name),
            &owner,
            found,
        );
    }
    for actor in ir.actors().values() {
        fields(
            primitive,
            &actor.attributes,
            &format!("actor.{}.attributes", actor.name),
            &ActorRef::new(actor.name.clone()).into(),
            found,
        );
    }
}

fn fields(
    primitive: Primitive,
    values: &[ResolvedField],
    at: &str,
    owner: &EssSemanticRef,
    found: &mut BTreeMap<String, EssSemanticRef>,
) {
    for field in values {
        reference(
            primitive,
            &field.type_ref,
            &format!("{at}.{}", field.name),
            owner,
            found,
        );
    }
}

fn reference(
    primitive: Primitive,
    ty: &ResolvedTypeRef,
    at: &str,
    owner: &EssSemanticRef,
    found: &mut BTreeMap<String, EssSemanticRef>,
) {
    match ty {
        ResolvedTypeRef::Primitive { name } if *name == primitive => {
            found.insert(at.to_owned(), owner.clone());
        }
        ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => {
            reference(primitive, of, &format!("{at}.of"), owner, found);
        }
        ResolvedTypeRef::Map { key, value } => {
            if *key == primitive {
                found.insert(format!("{at}.key"), owner.clone());
            }
            reference(primitive, value, &format!("{at}.value"), owner, found);
        }
        ResolvedTypeRef::Declared { .. } | ResolvedTypeRef::Primitive { .. } => {}
    }
}
