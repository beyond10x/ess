//! Concrete finite-number use inventory for adapters that cannot execute its wire contract.

use crate::ir::{EssIr, ResolvedBody, ResolvedField, ResolvedTypeRef};
use crate::refs::{
    CommandRef, DeclaredTypeRef, EntityRef, ErrorRef, EssSemanticRef, EventRef, ViewRef,
};
use ess_domain::Primitive;
use std::collections::{BTreeMap, BTreeSet};

/// Every directly declared Binary64 position in the checked model, in stable source order.
pub fn locations(ir: &EssIr) -> BTreeSet<String> {
    uses(ir).into_keys().collect()
}

/// Each concrete Binary64 position and its compiler-owned semantic owner.
/// Named references need no expansion: their checked declarations occur exactly once.
pub fn uses(ir: &EssIr) -> BTreeMap<String, EssSemanticRef> {
    let mut found = BTreeMap::new();
    for declared in ir.types().values() {
        let at = format!("types.{}", declared.name);
        let owner = DeclaredTypeRef::new(declared.name.clone()).into();
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => {
                reference(of, &format!("{at}.of"), &owner, &mut found);
            }
            ResolvedBody::Struct {
                fields: members, ..
            } => fields(members, &format!("{at}.fields"), &owner, &mut found),
            ResolvedBody::Union { variants, .. } => {
                for (name, ty) in variants {
                    reference(ty, &format!("{at}.variants.{name}"), &owner, &mut found);
                }
            }
            ResolvedBody::Enum { .. } => {}
        }
    }
    for entity in ir.entities().values() {
        let owner = EntityRef::new(entity.name.clone()).into();
        reference(
            &entity.identity.type_ref,
            &format!("entity {}.identity", entity.name),
            &owner,
            &mut found,
        );
        fields(
            &entity.fields,
            &format!("entity {}.fields", entity.name),
            &owner,
            &mut found,
        );
    }
    for command in ir.commands().values() {
        fields(
            &command.response,
            &format!("command.{}.response", command.name),
            &CommandRef::new(command.name.clone()).into(),
            &mut found,
        );
        fields(
            &command.input,
            &format!("command.{}.input", command.name),
            &CommandRef::new(command.name.clone()).into(),
            &mut found,
        );
    }
    for event in ir.events().values() {
        fields(
            &event.fields,
            &format!("event.{}.fields", event.name),
            &EventRef::new(event.name.clone()).into(),
            &mut found,
        );
    }
    for error in ir.errors().values() {
        fields(
            &error.fields,
            &format!("error.{}.fields", error.name),
            &ErrorRef::new(error.name.clone()).into(),
            &mut found,
        );
    }
    for view in ir.views().values() {
        let owner = ViewRef::new(view.name.clone()).into();
        fields(
            &view.fields,
            &format!("view.{}.fields", view.name),
            &owner,
            &mut found,
        );
        fields(
            &view.params,
            &format!("view.{}.params", view.name),
            &owner,
            &mut found,
        );
    }
    found
}

fn fields(
    values: &[ResolvedField],
    at: &str,
    owner: &EssSemanticRef,
    found: &mut BTreeMap<String, EssSemanticRef>,
) {
    for field in values {
        reference(
            &field.type_ref,
            &format!("{at}.{}", field.name),
            owner,
            found,
        );
    }
}

fn reference(
    ty: &ResolvedTypeRef,
    at: &str,
    owner: &EssSemanticRef,
    found: &mut BTreeMap<String, EssSemanticRef>,
) {
    match ty {
        ResolvedTypeRef::Primitive {
            name: Primitive::Binary64,
        } => {
            found.insert(at.to_owned(), owner.clone());
        }
        ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => {
            reference(of, &format!("{at}.of"), owner, found);
        }
        ResolvedTypeRef::Map { key, value } => {
            if *key == Primitive::Binary64 {
                found.insert(format!("{at}.key"), owner.clone());
            }
            reference(value, &format!("{at}.value"), owner, found);
        }
        ResolvedTypeRef::Declared { .. } | ResolvedTypeRef::Primitive { .. } => {}
    }
}
