//! The two context answers a generated demonstration server can supply.
use super::{behaviour::Uses, layout::Layout};
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedTypeRef};
use ess_domain::types::Primitive;
use std::collections::BTreeSet;
use std::fmt::Write as _;

pub(super) fn unmet(ir: &EssIr, uses: &Uses) -> BTreeSet<String> {
    let mut unmet: BTreeSet<_> = uses
        .callers
        .values()
        .map(|(_, attribute)| format!("caller attribute: {attribute}"))
        .collect();
    for reference in uses.assigned.values() {
        if !crate::served::answered(ir, reference) {
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

pub(super) fn implementation(ir: &EssIr, layout: &Layout, uses: &Uses) -> String {
    if uses.callers.is_empty() && uses.assigned.is_empty() && !uses.external && !uses.clock {
        return String::new();
    }
    let types = Layout::crate_ident(layout.package());
    let mut out = format!("\nimpl {types}::behaviour::TryContext for MemoryPorts {{\n");
    for (method, (ty, attribute)) in &uses.callers {
        let ty = ty.replace("crate::", &format!("{types}::"));
        let message = format!("caller attribute: {attribute}");
        let _ = writeln!(
            out,
            "fn try_{method}(&self) -> Result<Option<{ty}>, {types}::obligation::UnmetObligation> {{ Err({types}::behaviour::unmet_context({message:?})) }}"
        );
    }
    for (method, reference) in &uses.assigned {
        let ty = layout
            .absolute_type(reference)
            .replace("crate::", &format!("{types}::"));
        let value = if reference.is_optional() {
            "Ok(None)".to_owned()
        } else if crate::served::supported(ir, reference) {
            format!("Ok({})", value(ir, layout, reference))
        } else {
            format!(
                "Err({types}::behaviour::unmet_context({:?}))",
                format!("assigned value: {reference}")
            )
        };
        let _ = writeln!(out, "fn try_{method}(&mut self) -> Result<{ty}, {types}::obligation::UnmetObligation> {{ {value} }}");
    }
    if uses.external {
        let _ = writeln!(out, "fn try_external(&mut self, _command: {types}::behaviour::ExternalCommand<'_>, _outcome: &'static str) -> Result<bool, {types}::obligation::UnmetObligation> {{ Err({types}::behaviour::unmet_context(\"external branch answer\")) }}");
    }
    if uses.clock {
        // The network entry is the deployment's host, and this is its explicit command clock: the
        // host's UTC clock, read once per decision by the behaviour (ess/22, family F A3).
        let _ = writeln!(out, "/// The host's UTC clock, the command clock this network entry supplies: read once per decision.\nfn try_command_clock(&mut self) -> Result<Option<{types}::primitives::Timestamp>, {types}::obligation::UnmetObligation> {{ Ok(Some({types}::primitives::Timestamp(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).expect(\"UTC time formats\")))) }}");
    }
    out.push_str("}\n");
    out
}

fn value(ir: &EssIr, layout: &Layout, reference: &ResolvedTypeRef) -> String {
    let types = Layout::crate_ident(layout.package());
    let ty = layout
        .absolute_type(reference)
        .replace("crate::", &format!("{types}::"));
    match reference {
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => format!("{ty}({})", value(ir, layout, of)),
            _ => unreachable!("supported context values are transparent"),
        },
        ResolvedTypeRef::Primitive { name: Primitive::Uuid } => format!("{ty}(uuid::Uuid::new_v4().to_string())"),
        ResolvedTypeRef::Primitive { name: Primitive::Timestamp } => format!("{ty}(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).expect(\"UTC time formats\"))"),
        _ => unreachable!("only UUID and timestamp are supplied"),
    }
}
