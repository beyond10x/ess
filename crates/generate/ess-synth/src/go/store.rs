//! Generated ephemeral stores behind the existing interfaces.
use super::{
    behaviour::{self, Seams, Uses},
    context, name, Emit,
};
use crate::plan::CapabilityKind;
use std::fmt::Write as _;
mod identity;

pub(super) fn implementation(emit: &Emit<'_>, uses: &Uses, seams: &Seams) -> String {
    let names = behaviour::storage_names(emit.ir, emit.layout);
    let mut out = context::implementation(emit, uses);
    let mut fields = Vec::new();
    let mut keys = identity::Keys::new(emit);
    for entity in &uses.storages {
        emit.import("sort");
        let declared = &emit.ir.entities()[entity];
        let port = &names[entity];
        let store = format!("Memory{port}");
        let snapshot = emit.qualify(emit.layout.package_of(entity), emit.layout.snapshot(entity));
        let identity = emit.go_type(&declared.identity.type_ref);
        let id = name::exported(&declared.identity.name);
        let left = keys.expression(
            &declared.identity.type_ref,
            &format!("memoryStore.rows[i].Data.{id}"),
        );
        let right = keys.expression(
            &declared.identity.type_ref,
            &format!("memoryStore.rows[j].Data.{id}"),
        );
        let row_key = keys.expression(&declared.identity.type_ref, &format!("row.Data.{id}"));
        let input_key = keys.expression(&declared.identity.type_ref, "memoryIdentity");
        let _ = writeln!(
            out,
            r"
// {store} is ephemeral storage of `{entity}`, in identity order.
type {store} struct {{
	rows []{snapshot}
}}

func (memoryStore *{store}) Get(memoryIdentity {identity}) ({snapshot}, bool) {{
	key := {input_key}
	for _, row := range memoryStore.rows {{
		if memoryCompare({row_key}, key) == 0 {{
			return row, true
		}}
	}}
	return {snapshot}{{}}, false
}}

func (memoryStore *{store}) Put(memorySnapshot {snapshot}) {{
	memoryStore.Delete(memorySnapshot.Data.{id})
	memoryStore.rows = append(memoryStore.rows, memorySnapshot)
	sort.SliceStable(memoryStore.rows, func(i, j int) bool {{
		return memoryCompare({left}, {right}) < 0
	}})
}}

func (memoryStore *{store}) Delete(memoryIdentity {identity}) {{
	key := {input_key}
	for i, row := range memoryStore.rows {{
		if memoryCompare({row_key}, key) == 0 {{
			memoryStore.rows = append(memoryStore.rows[:i], memoryStore.rows[i+1:]...)
			return
		}}
	}}
}}"
        );
        if uses.listed.contains(entity) {
            let _ = writeln!(out, "\nfunc (memoryStore *{store}) List() []{snapshot} {{\n\treturn append([]{snapshot}(nil), memoryStore.rows...)\n}}");
        }
        fields.push((port.clone(), format!("&{store}{{}}")));
    }
    let owed = owed_methods(&mut out, emit, seams);
    if owed {
        out.push_str("\ntype memoryOwed struct{}\n");
        fields.push(("Owed".into(), "&memoryOwed{}".into()));
    }
    out.push_str(&constructor(emit, fields));
    if !uses.storages.is_empty() {
        out.push_str(&keys.helpers());
    }
    out
}

fn constructor(emit: &Emit<'_>, fields: Vec<(String, String)>) -> String {
    let mut out = String::new();
    let ports = emit.qualify(emit.layout.behaviour(), "Ports");
    let _ = writeln!(out, "\n// NewMemoryPorts supplies ephemeral storage; clones of this value share its stores.\nfunc NewMemoryPorts() {ports} {{");
    if fields.is_empty() {
        let _ = writeln!(out, "\treturn {ports}{{}}\n}}");
    } else {
        let width = fields.iter().map(|(name, _)| name.len()).max().unwrap() + 1;
        let _ = writeln!(out, "\treturn {ports}{{");
        for (name, value) in fields {
            let _ = writeln!(out, "\t\t{:<width$} {value},", format!("{name}:"));
        }
        out.push_str("\t}\n}\n");
    }
    out
}

fn owed_methods(out: &mut String, emit: &Emit<'_>, seams: &Seams) -> bool {
    let mut owed = false;
    for command in emit.ir.commands().values() {
        if !seams.forwards(CapabilityKind::CommandBehavior, &command.name.to_string()) {
            continue;
        }
        owed = true;
        let method = emit.layout.declared(&command.name);
        let input = emit.reference(&command.name);
        let outcome = emit.reference_outcome(&command.name);
        let unmet = emit.unmet();
        let stub = emit.qualify(emit.layout.package_of(&command.name), "Unimplemented");
        let _ = writeln!(out, "\nfunc (*memoryOwed) {method}(input {input}) ({outcome}, {unmet}) {{\n\treturn ({stub}{{}}).{method}(input)\n}}");
    }
    for view in emit.ir.views().values() {
        if !seams.forwards(CapabilityKind::ViewQuery, &view.name.to_string()) {
            continue;
        }
        owed = true;
        let method = emit.layout.declared(&view.name);
        let parameters = super::port::view_params(emit, view, &[]);
        let params = super::port::signature(&parameters);
        let arguments = parameters
            .iter()
            .map(|(ident, _)| ident.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let row = emit.reference(&view.name);
        let unmet = emit.unmet();
        let stub = emit.qualify(emit.layout.package_of(&view.name), "Unimplemented");
        let _ = writeln!(
            out,
            "\nfunc (*memoryOwed) {method}({params}) ([]{row}, {unmet}) {{\n\treturn ({stub}{{}}).{method}({arguments})\n}}"
        );
    }
    owed
}
