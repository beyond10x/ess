//! Native reading helpers share the domain's exact integer normalization and evidence checks.
use super::Emit;
use ess_compiler::ir::{EssIr, ResolvedType};
use ess_domain::reading::ReadingEncoding;
use std::fmt::Write as _;

pub(super) fn helpers(ir: &EssIr) -> String {
    if !ir
        .types()
        .values()
        .any(|declared| declared.reading.is_some())
    {
        return String::new();
    }
    format!(
        "\n/// Observed clock coordinates; no live clock or timezone database.\npub mod clock_reading {{\nmod normalize {{\n{}\n}}\npub use normalize::{{ReadingError, normalize_text, normalize_unix}};\nmod coordinate {{\n{}\n}}\npub use coordinate::{{ClockReadingEvidence, ClockCoordinate, resolve_clock_reading, compare_clock_readings}};\n}}\n",
        include_str!("../../../../specify/ess-domain/src/reading/normalize.rs"),
        include_str!("../../../../specify/ess-domain/src/reading/coordinate.rs")
    )
}

pub(super) fn wrapper(out: &mut String, emit: &Emit<'_>, declared: &ResolvedType) {
    let Some(contract) = &declared.reading else {
        return;
    };
    let encoding = serde_json::to_value(contract.encoding)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let origins = contract
        .origins
        .iter()
        .map(|origin| {
            let role = serde_json::to_value(origin.role).unwrap();
            let offset = serde_json::to_value(origin.offset).unwrap();
            format!("({role},{offset})")
        })
        .collect::<Vec<_>>()
        .join(",");
    let value = if contract.encoding == ReadingEncoding::UnixSeconds {
        "&self.0.to_string()"
    } else {
        "&self.0"
    };
    let name = emit.layout.type_name(&declared.name);
    let _ = writeln!(
        out,
        "impl {name} {{\n/// Resolve this nominal reading with observed occurrence facts.\npub fn resolve_reading(&self, evidence: &crate::primitives::clock_reading::ClockReadingEvidence, correlation: &str, occurrence: &str) -> Result<crate::primitives::clock_reading::ClockCoordinate, crate::primitives::clock_reading::ReadingError> {{ crate::primitives::clock_reading::resolve_clock_reading({encoding:?}, &[{origins}], {value}, evidence, correlation, occurrence) }}\n}}"
    );
}
