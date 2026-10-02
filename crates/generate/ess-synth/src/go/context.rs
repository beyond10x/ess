//! Explicit generated context answers and named startup refusals.
use super::{behaviour::Uses, Emit};
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedTypeRef};
use ess_domain::types::Primitive;
use std::{collections::BTreeSet, fmt::Write as _};

pub(super) fn unmet(ir: &EssIr, uses: &Uses) -> BTreeSet<String> {
    let mut unmet: BTreeSet<_> = uses
        .callers
        .values()
        .map(|(_, attribute)| format!("caller attribute: {attribute}"))
        .collect();
    for reference in uses.assigned.values() {
        if !crate::served::supported(ir, reference) {
            unmet.insert(format!("assigned value: {reference}"));
        }
    }
    unmet.extend(
        uses.externals
            .iter()
            .map(|branch| format!("external branch answer: {branch}")),
    );
    unmet
}

pub(super) fn implementation(emit: &Emit<'_>, uses: &Uses) -> String {
    let mut out = String::from("\n// MemoryContext supplies UUIDs and clock timestamps only.\ntype MemoryContext struct{}\n");
    if uses.attributes.is_empty() && uses.assigned.is_empty() && !uses.external {
        return out;
    }
    let unmet = emit.unmet();
    let unavailable = emit.qualify(emit.layout.behaviour(), "UnmetContext");
    for (method, reference) in &uses.attributes {
        let ty = emit.go_type(reference);
        let message = format!("caller attribute: {}", uses.callers[method].1);
        let _ = writeln!(out, "\nfunc (*MemoryContext) Try{method}() ({ty}, bool, {unmet}) {{\n\tvar zero {ty}\n\treturn zero, false, {unavailable}({message:?})\n}}");
    }
    for (method, reference) in &uses.assigned {
        let ty = emit.go_type(reference);
        let body = if crate::served::supported(emit.ir, reference) {
            let value = value(emit, reference, "assigned");
            if crate::served::primitive(emit.ir, reference) == Some(Primitive::Uuid) {
                format!("assigned, err := memoryUUID()\n\tif err != nil {{\n\t\tvar zero {ty}\n\t\treturn zero, err\n\t}}\n\treturn {value}, nil")
            } else {
                format!("return {value}, nil")
            }
        } else {
            format!(
                "var zero {ty}\n\treturn zero, {unavailable}({:?})",
                format!("assigned value: {reference}")
            )
        };
        let _ = writeln!(
            out,
            "\nfunc (*MemoryContext) Try{method}() ({ty}, {unmet}) {{\n\t{body}\n}}"
        );
    }
    if uses.external {
        let _ = writeln!(out, "\nfunc (*MemoryContext) TryExternal(command string, outcome string) (bool, {unmet}) {{\n\treturn false, {unavailable}(\"external branch answer\")\n}}");
    }
    if uses
        .assigned
        .values()
        .any(|reference| crate::served::primitive(emit.ir, reference) == Some(Primitive::Uuid))
    {
        emit.import("crypto/rand");
        emit.import("fmt");
        let _ = writeln!(out, "\nfunc memoryUUID() (string, {unmet}) {{\n\tvar bytes [16]byte\n\tif _, err := rand.Read(bytes[:]); err != nil {{\n\t\treturn \"\", {unavailable}(\"UUID entropy\")\n\t}}\n\tbytes[6] = (bytes[6] & 15) | 64\n\tbytes[8] = (bytes[8] & 63) | 128\n\treturn fmt.Sprintf(\"%x-%x-%x-%x-%x\", bytes[0:4], bytes[4:6], bytes[6:8], bytes[8:10], bytes[10:16]), nil\n}}");
    }
    out
}

fn value(emit: &Emit<'_>, reference: &ResolvedTypeRef, uuid: &str) -> String {
    match reference {
        ResolvedTypeRef::Declared { name } => match &emit.ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => {
                format!(
                    "{}({})",
                    emit.reference_ctor(name.name()),
                    value(emit, of, uuid)
                )
            }
            _ => unreachable!("supported context values are transparent"),
        },
        ResolvedTypeRef::Primitive {
            name: Primitive::Uuid,
        } => format!("{}({uuid})", emit.primitive_ctor(Primitive::Uuid)),
        ResolvedTypeRef::Primitive {
            name: Primitive::Timestamp,
        } => {
            emit.import("time");
            format!(
                "{}(time.Now().UTC().Format(time.RFC3339Nano))",
                emit.primitive_ctor(Primitive::Timestamp)
            )
        }
        _ => unreachable!("only UUID and timestamp are supplied"),
    }
}
