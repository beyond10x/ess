//! In-memory implementations of the unchanged generated storage ports.
use super::{behaviour, context, layout::Layout, name};
use crate::plan::{CapabilityKind, SynthesisPlan, REGENERATE};
use ess_compiler::ir::EssIr;
use ess_gen::Artifact;
use std::fmt::Write as _;
mod identity;

pub(super) fn module(ir: &EssIr, plan: &SynthesisPlan, layout: &Layout) -> Artifact {
    let types = Layout::crate_ident(layout.package());
    let uses = behaviour::requirements(ir, plan, layout, None);
    let names = behaviour::storage_names(ir, layout);
    let mut out = plan.provenance.commented_for("//", REGENERATE);
    out.push_str("\n//! Ephemeral stores for the generated network entry points; no durability.\n");
    let mut fields = String::new();
    let mut keys = identity::Keys::new(ir, layout);
    for entity in &uses.storages {
        let declared = &ir.entities()[entity];
        let port = &names[entity];
        let store = format!("Memory{port}");
        let field = name::value_ident(port);
        let snapshot = format!(
            "{types}::{}::{}",
            layout.module(layout.owner(entity)),
            layout.entity_snapshot(entity)
        );
        let identity = layout
            .absolute_type(&declared.identity.type_ref)
            .replace("crate::", &format!("{types}::"));
        let id = name::value_ident(&declared.identity.name);
        let key = keys.expression(&declared.identity.type_ref, &format!("&row.data.{id}"));
        let key_other = keys.expression(&declared.identity.type_ref, &format!("&other.data.{id}"));
        let input_key = keys.expression(&declared.identity.type_ref, "identity");
        let _ = writeln!(out, "\n/// Ephemeral storage of `{entity}`, shared by clones.\n#[derive(Clone, Default)]\npub struct {store}(std::rc::Rc<std::cell::RefCell<Vec<{snapshot}>>>);\nimpl {types}::behaviour::{port} for {store} {{\nfn get(&self, identity: &{identity}) -> Option<{snapshot}> {{ let key = {input_key}; self.0.borrow().iter().find(|row| {key} == key).cloned() }}\nfn put(&mut self, snapshot: {snapshot}) {{ self.delete(&snapshot.data.{id}); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| {key}.cmp(&{key_other})); }}\nfn delete(&mut self, identity: &{identity}) {{ let key = {input_key}; self.0.borrow_mut().retain(|row| {key} != key); }}");
        if uses.listed.contains(entity) {
            let _ = writeln!(
                out,
                "fn list(&self) -> Vec<{snapshot}> {{ self.0.borrow().clone() }}"
            );
        }
        out.push_str("}\n");
        let _ = writeln!(fields, "/// Storage of `{entity}`.\npub {field}: {store},");
        let _ = writeln!(out, "impl {types}::behaviour::{port} for MemoryPorts {{\nfn get(&self, identity: &{identity}) -> Option<{snapshot}> {{ {types}::behaviour::{port}::get(&self.{field}, identity) }}\nfn put(&mut self, snapshot: {snapshot}) {{ {types}::behaviour::{port}::put(&mut self.{field}, snapshot); }}\nfn delete(&mut self, identity: &{identity}) {{ {types}::behaviour::{port}::delete(&mut self.{field}, identity); }}");
        if uses.listed.contains(entity) {
            let _ = writeln!(out, "fn list(&self) -> Vec<{snapshot}> {{ {types}::behaviour::{port}::list(&self.{field}) }}");
        }
        out.push_str("}\n");
    }
    let _ = writeln!(out, "\n/// Ports shared by generated components. Values disappear when the process exits.\n#[derive(Clone, Default)]\npub struct MemoryPorts {{\n{fields}}}");
    out.push_str(&context::implementation(ir, layout, &uses));
    if !uses.storages.is_empty() {
        out.push_str(&keys.helpers());
    }
    // Unreachable owed methods retain their typed refusal. Startup checks the reachable subset.
    for command in ir.commands().values() {
        if plan
            .obligation_of(CapabilityKind::CommandBehavior, &command.name.to_string())
            .is_none()
        {
            continue;
        }
        let ty = layout.type_name(&command.name);
        let module = layout.module(layout.owner(&command.name));
        let method = name::value_ident(&ty);
        let _ = writeln!(out, "impl {types}::{module}::obligations::{ty}Behavior for MemoryPorts {{\nfn {method}(&mut self, input: {types}::{module}::{ty}) -> Result<{types}::{module}::{ty}Outcome, {types}::obligation::UnmetObligation> {{ {types}::{module}::obligations::{ty}Behavior::{method}(&mut {types}::{module}::obligations::Unimplemented, input) }}\n}}");
    }
    for view in ir.views().values() {
        if plan
            .obligation_of(CapabilityKind::ViewQuery, &view.name.to_string())
            .is_none()
        {
            continue;
        }
        let ty = layout.type_name(&view.name);
        let module = layout.module(layout.owner(&view.name));
        let method = name::value_ident(&ty);
        let params = super::port::view_params(layout, &types, view);
        let signature = params
            .iter()
            .fold(String::new(), |mut signature, (ident, ty)| {
                let _ = write!(signature, ", {ident}: {ty}");
                signature
            });
        let arguments = params
            .iter()
            .fold(String::new(), |mut arguments, (ident, _)| {
                let _ = write!(arguments, ", {ident}");
                arguments
            });
        let _ = writeln!(out, "impl {types}::{module}::obligations::{ty}Query for MemoryPorts {{\nfn {method}(&self{signature}) -> Result<Vec<{types}::{module}::{ty}>, {types}::obligation::UnmetObligation> {{ {types}::{module}::obligations::{ty}Query::{method}(&{types}::{module}::obligations::Unimplemented{arguments}) }}\n}}");
    }
    Artifact::new(
        format!("crates/{}/src/memory.rs", layout.server_package()),
        out,
    )
}
