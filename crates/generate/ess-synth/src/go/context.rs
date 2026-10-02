//! Bounded demonstration context, using only cryptographic UUIDs and the UTC clock.
use super::{behaviour::Uses, Emit};
use ess_compiler::ir::ResolvedBody;
use std::fmt::Write as _;

fn assigned(emit: &Emit<'_>, of: &str) -> Option<String> {
    match of {
        "Uuid" => {
            emit.import("crypto/rand");
            emit.import("fmt");
            Some(format!(
                "{}(memoryUUID())",
                emit.qualify(emit.layout.primitives(), "NewUuid")
            ))
        }
        "Timestamp" => {
            emit.import("time");
            Some(format!(
                "{}(time.Now().UTC().Format(time.RFC3339Nano))",
                emit.qualify(emit.layout.primitives(), "NewTimestamp")
            ))
        }
        _ => {
            let (name, ty) = emit
                .ir
                .types()
                .iter()
                .find(|(name, _)| name.to_string() == of)?;
            let ResolvedBody::Newtype { of, .. } = &ty.body else {
                return None;
            };
            let value = assigned(emit, &of.to_string())?;
            Some(format!("{}({value})", emit.reference_ctor(name)))
        }
    }
}

pub(super) fn refusals(emit: &Emit<'_>, uses: &Uses) -> Vec<String> {
    let mut out: Vec<_> = uses
        .callers
        .values()
        .map(|(_, attribute)| format!("caller attribute `{attribute}`"))
        .collect();
    for (_, of) in uses.generates.values() {
        if assigned(emit, of).is_none() {
            out.push(format!("assigned value `{of}`"));
        }
    }
    if uses.external {
        for command in emit.ir.commands().values() {
            for outcome in &command.outcomes {
                if matches!(
                    outcome.condition,
                    ess_compiler::ir::ResolvedCondition::External { .. }
                        | ess_compiler::ir::ResolvedCondition::ExternalWhen { .. }
                ) {
                    out.push(format!(
                        "external outcome `{}.{}`",
                        command.name, outcome.name
                    ));
                }
            }
        }
    }
    out
}

pub(super) fn implementation(emit: &Emit<'_>, uses: &Uses) -> String {
    let mut out = "\n// InMemoryContext supplies UUID v4 identities and system-clock timestamps only.\ntype InMemoryContext struct{}\n".to_owned();
    for (method, (ty, attribute)) in &uses.callers {
        let _ = writeln!(out, "\nfunc (*InMemoryContext) {method}() ({ty}, bool) {{\n\tpanic(\"unsupported caller attribute: {attribute}\")\n}}");
    }
    let mut uuid = false;
    for (method, (ty, of)) in &uses.generates {
        let value = assigned(emit, of)
            .unwrap_or_else(|| format!("panic(\"unsupported assigned value: {of}\")"));
        uuid |= value.contains("memoryUUID()");
        let prefix = if value.starts_with("panic(") {
            ""
        } else {
            "return "
        };
        let _ = writeln!(
            out,
            "\nfunc (*InMemoryContext) {method}() {ty} {{\n\t{prefix}{value}\n}}"
        );
    }
    if uses.external {
        out.push_str("\nfunc (*InMemoryContext) External(command string, outcome string) bool {\n\tpanic(\"unsupported external outcome: \" + command + \".\" + outcome)\n}\n");
    }
    if uuid {
        out.push_str("\nfunc memoryUUID() string {\n\tvar bytes [16]byte\n\tif _, err := rand.Read(bytes[:]); err != nil {\n\t\tpanic(err)\n\t}\n\tbytes[6] = (bytes[6] & 0x0f) | 0x40\n\tbytes[8] = (bytes[8] & 0x3f) | 0x80\n\treturn fmt.Sprintf(\"%x-%x-%x-%x-%x\", bytes[0:4], bytes[4:6], bytes[6:8], bytes[8:10], bytes[10:16])\n}\n");
    }
    out
}
