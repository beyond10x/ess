//! In-memory implementations of the existing ports, only for network-reached systems.
use super::{
    behaviour::{self, Uses},
    context, http,
    layout::Layout,
    name,
};
use crate::plan::{SynthesisPlan, REGENERATE};
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedTypeRef};
use ess_gen::Artifact;
use std::fmt::Write as _;

pub(super) fn module(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    uses: &Uses,
) -> Option<Artifact> {
    if http::served(ir).is_empty() || !behaviour::used(ir) {
        return None;
    }
    let names = behaviour::storage_names(ir, layout);
    let mut out = plan.provenance.commented_for("//", REGENERATE);
    out.push_str("\n//! Ephemeral storage for generated servers. Restarting loses every row.\n");
    out.push_str("\n/// All storage and the bounded demonstration context, shared by the system's components.\n#[derive(Clone, Default)]\npub struct InMemoryPorts {\n");
    for entity in &uses.storages {
        let field = name::value_ident(&names[entity]);
        let _ = writeln!(
            out,
            "    /// The rows of `{entity}`.\n    pub {field}: std::rc::Rc<std::cell::RefCell<{}>> ,",
            names[entity]
        );
    }
    out.push_str("}\n");
    for entity in &uses.storages {
        let storage = &names[entity];
        let field = name::value_ident(storage);
        let snapshot = format!(
            "crate::{}::{}",
            layout.module(layout.owner(entity)),
            layout.entity_snapshot(entity)
        );
        let identity = layout.absolute_type(&ir.entities()[entity].identity.type_ref);
        let id = name::value_ident(ir.entities()[entity].identity.name.as_str());
        let (_, key) = storage_key(
            ir,
            layout,
            &ir.entities()[entity].identity.type_ref,
            &format!("left.data.{id}"),
        );
        let (_, stored_key) = storage_key(
            ir,
            layout,
            &ir.entities()[entity].identity.type_ref,
            &format!("right.data.{id}"),
        );
        let _ = writeln!(out, "\n/// In-memory rows of `{entity}`, ordered by identity.\n#[derive(Default)]\npub struct {storage} {{ rows: Vec<{snapshot}> }}\nimpl crate::behaviour::{storage} for {storage} {{\n    fn get(&self, identity: &{identity}) -> Option<{snapshot}> {{ self.rows.iter().find(|row| row.data.{id} == *identity).cloned() }}\n    fn put(&mut self, snapshot: {snapshot}) {{\n        if let Some(row) = self.rows.iter_mut().find(|row| row.data.{id} == snapshot.data.{id}) {{ *row = snapshot; }} else {{ self.rows.push(snapshot); }}\n    }}\n    fn delete(&mut self, identity: &{identity}) {{ self.rows.retain(|row| row.data.{id} != *identity); }}");
        if uses.listed.contains(entity) {
            let _ = writeln!(
                out,
                "    fn list(&self) -> Vec<{snapshot}> {{ let mut rows = self.rows.clone(); rows.sort_by(|left, right| {key}.cmp(&{stored_key})); rows }}"
            );
        }
        out.push_str("}\n");
        let _ = writeln!(out, "impl crate::behaviour::{storage} for InMemoryPorts {{\n    fn get(&self, identity: &{identity}) -> Option<{snapshot}> {{ crate::behaviour::{storage}::get(&*self.{field}.borrow(), identity) }}\n    fn put(&mut self, snapshot: {snapshot}) {{ crate::behaviour::{storage}::put(&mut *self.{field}.borrow_mut(), snapshot); }}\n    fn delete(&mut self, identity: &{identity}) {{ crate::behaviour::{storage}::delete(&mut *self.{field}.borrow_mut(), identity); }}");
        if uses.listed.contains(entity) {
            let _ = writeln!(out, "    fn list(&self) -> Vec<{snapshot}> {{ crate::behaviour::{storage}::list(&*self.{field}.borrow()) }}");
        }
        out.push_str("}\n");
    }
    out.push_str(&context::implementation(ir, layout, uses));
    Some(Artifact::new(
        format!("crates/{}/src/memory.rs", layout.package()),
        out,
    ))
}

fn storage_key(
    ir: &EssIr,
    layout: &Layout,
    ty: &ResolvedTypeRef,
    expression: &str,
) -> (String, String) {
    if let ResolvedTypeRef::Declared { name } = ty {
        if let ResolvedBody::Newtype { of, .. } = &ir.types()[name.name()].body {
            return storage_key(ir, layout, of, &format!("{expression}.0"));
        }
        return (
            "String".to_owned(),
            format!("format!(\"{{:?}}\", {expression})"),
        );
    }
    match ty {
        ResolvedTypeRef::Primitive { name } if *name != ess_domain::types::Primitive::Json => {
            (layout.absolute_type(ty), expression.to_owned())
        }
        _ => (
            "String".to_owned(),
            format!("format!(\"{{:?}}\", {expression})"),
        ),
    }
}
