//! Implements the generated storage ports without changing them.
use super::{
    behaviour::{self, Uses},
    context,
    layout::Layout,
    name, Emit,
};
use ess_compiler::ir::EssIr;
use ess_gen::{Artifact, Provenance};
use std::fmt::Write as _;

pub(super) fn package(
    ir: &EssIr,
    layout: &Layout,
    provenance: &Provenance,
    uses: &Uses,
) -> Option<Artifact> {
    if uses.storages.is_empty()
        && uses.callers.is_empty()
        && uses.generates.is_empty()
        && !uses.external
    {
        return None;
    }
    let emit = Emit::new(ir, layout, layout.behaviour(), None);
    let names = behaviour::storage_names(ir, layout);
    let mut out = String::new();
    for entity in &uses.storages {
        emit.import("reflect");
        let storage = format!("InMemory{}", names[entity]);
        let snapshot = emit.qualify(layout.package_of(entity), layout.snapshot(entity));
        let identity = emit.go_type(&ir.entities()[entity].identity.type_ref);
        let id = name::exported(&ir.entities()[entity].identity.name);
        let _ = writeln!(
            out,
            r"
// {storage} holds ephemeral rows of `{entity}`.
type {storage} struct{{ rows []{snapshot} }}

func (s *{storage}) Get(identity {identity}) ({snapshot}, bool) {{
	for _, row := range s.rows {{
		if reflect.DeepEqual(row.Data.{id}, identity) {{
			return row, true
		}}
	}}
	return {snapshot}{{}}, false
}}
func (s *{storage}) Put(snapshot {snapshot}) {{
	for i, row := range s.rows {{
		if reflect.DeepEqual(row.Data.{id}, snapshot.Data.{id}) {{
			s.rows[i] = snapshot
			return
		}}
	}}
	s.rows = append(s.rows, snapshot)
}}
func (s *{storage}) Delete(identity {identity}) {{
	for i, row := range s.rows {{
		if reflect.DeepEqual(row.Data.{id}, identity) {{
			s.rows = append(s.rows[:i], s.rows[i+1:]...)
			return
		}}
	}}
}}"
        );
        if uses.listed.contains(entity) {
            emit.import("sort");
            let _ = writeln!(
                out,
                r"func (s *{storage}) List() []{snapshot} {{
	rows := append([]{snapshot}{{}}, s.rows...)
	sort.Slice(rows, func(i, j int) bool {{
		return memoryLess(reflect.ValueOf(rows[i].Data.{id}), reflect.ValueOf(rows[j].Data.{id}))
	}})
	return rows
}}"
            );
        }
    }
    if !uses.listed.is_empty() {
        out.push_str(LESS);
    }
    out.push_str(&context::implementation(&emit, uses));
    // Context signatures were collected in the behaviour package's import environment.
    for (_, (ty, _)) in uses.callers.iter().chain(uses.generates.iter()) {
        for (_, package) in layout.packages() {
            if ty.contains(&format!("{}.", package.name)) {
                emit.import(&package.import);
            }
        }
        if ty.contains("primitives.") {
            emit.import(&layout.primitives().import);
        }
    }
    Some(emit.file_at(
        "types/behaviour/memory.go",
        provenance,
        "// Ephemeral storage and context for generated demonstration servers.\n",
        &out,
    ))
}

const LESS: &str = r#"
// Compare identity representations, including nested newtypes and byte strings.
func memoryLess(a, b reflect.Value) bool {
	switch a.Kind() {
	case reflect.String:
		return a.String() < b.String()
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return a.Int() < b.Int()
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return a.Uint() < b.Uint()
	case reflect.Bool:
		return !a.Bool() && b.Bool()
	case reflect.Struct:
		for i := 0; i < a.NumField(); i++ {
			if memoryLess(a.Field(i), b.Field(i)) {
				return true
			}
			if memoryLess(b.Field(i), a.Field(i)) {
				return false
			}
		}
	case reflect.Array, reflect.Slice:
		for i := 0; i < a.Len() && i < b.Len(); i++ {
			if memoryLess(a.Index(i), b.Index(i)) {
				return true
			}
			if memoryLess(b.Index(i), a.Index(i)) {
				return false
			}
		}
		return a.Len() < b.Len()
	default:
		panic("unsupported storage identity representation")
	}
	return false
}
"#;
