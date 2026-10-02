//! The explicitly bounded context supplied by a generated demonstration server.
use super::{behaviour::Uses, layout::Layout};
use ess_compiler::ir::{EssIr, ResolvedBody};
use std::fmt::Write as _;

pub(super) fn assigned(ir: &EssIr, layout: &Layout, of: &str) -> Option<String> {
    match of {
        "Uuid" => Some("crate::primitives::Uuid(uuid::Uuid::new_v4().to_string())".into()),
        "Timestamp" => Some("crate::primitives::Timestamp(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).expect(\"the system clock formats as RFC3339\"))".into()),
        _ => {
            let (name, ty) = ir.types().iter().find(|(name, _)| name.to_string() == of)?;
            let ResolvedBody::Newtype { of, .. } = &ty.body else { return None; };
            let value = assigned(ir, layout, &of.to_string())?;
            Some(format!("crate::{}::{}({value})", layout.module(layout.owner(name)), layout.type_name(name)))
        }
    }
}

pub(super) fn refusals(ir: &EssIr, layout: &Layout, uses: &Uses) -> Vec<String> {
    let mut out: Vec<_> = uses
        .callers
        .values()
        .map(|(_, attribute)| format!("caller attribute `{attribute}`"))
        .collect();
    for (_, of) in uses.generates.values() {
        if assigned(ir, layout, of).is_none() {
            out.push(format!("assigned value `{of}`"));
        }
    }
    if uses.external {
        for command in ir.commands().values() {
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

pub(super) fn implementation(ir: &EssIr, layout: &Layout, uses: &Uses) -> String {
    if uses.callers.is_empty() && uses.generates.is_empty() && !uses.external {
        return String::new();
    }
    let mut out = "\nimpl crate::behaviour::Context for InMemoryPorts {\n".to_owned();
    for (method, (ty, attribute)) in &uses.callers {
        let _ = writeln!(out, "    fn {method}(&self) -> Option<{ty}> {{ panic!(\"unsupported caller attribute: {attribute}\") }}");
    }
    for (method, (ty, of)) in &uses.generates {
        let value = assigned(ir, layout, of)
            .unwrap_or_else(|| format!("panic!(\"unsupported assigned value: {of}\")"));
        let _ = writeln!(out, "    fn {method}(&mut self) -> {ty} {{ {value} }}");
    }
    if uses.external {
        out.push_str("    fn external(&mut self, command: &'static str, outcome: &'static str) -> bool { panic!(\"unsupported external outcome: {command}.{outcome}\") }\n");
    }
    out.push_str("}\n");
    out
}
