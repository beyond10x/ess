//! Native Go reading evidence and checked coordinate normalization.
use super::{layout::Layout, Emit};
use ess_compiler::ir::{EssIr, ResolvedType};
use ess_domain::reading::ReadingEncoding;
use ess_gen::{Artifact, Provenance};
use std::fmt::Write as _;

pub(super) fn helper(ir: &EssIr, layout: &Layout, provenance: &Provenance) -> Option<Artifact> {
    if !ir
        .types()
        .values()
        .any(|declared| declared.reading.is_some())
    {
        return None;
    }
    let package = layout.primitives();
    let mut out = provenance.commented_for("//", super::REGENERATE);
    let _ = writeln!(
        out,
        "\npackage {}\n\nimport (\"fmt\"; \"strconv\"; \"time\")\n",
        package.name
    );
    out.push_str(include_str!(
        "../../../../specify/ess-domain/src/reading/coordinate.go"
    ));
    let path = package.file().replace(".go", "_clock_reading.go");
    Some(Artifact::new(path, out))
}

pub(super) fn wrapper(out: &mut String, emit: &Emit<'_>, declared: &ResolvedType) {
    let Some(contract) = &declared.reading else {
        return;
    };
    let package = emit.layout.primitives();
    let evidence = emit.qualify(package, "ClockReadingEvidence");
    let coordinate = emit.qualify(package, "ClockCoordinate");
    let resolve = emit.qualify(package, "ResolveClockReading");
    let encoding = serde_json::to_value(contract.encoding).unwrap();
    let origins = contract
        .origins
        .iter()
        .map(|origin| {
            format!(
                "{{{},{}}}",
                serde_json::to_value(origin.role).unwrap(),
                serde_json::to_value(origin.offset).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let value = if contract.encoding == ReadingEncoding::UnixSeconds {
        emit.import("strconv");
        "strconv.FormatInt(v.value, 10)"
    } else {
        "v.value"
    };
    let name = emit.layout.declared(&declared.name);
    let _ = writeln!(
        out,
        "\n// ResolveReading resolves this nominal reading with observed occurrence facts.\nfunc (v {name}) ResolveReading(evidence {evidence}, correlation, occurrence string) ({coordinate}, error) {{ return {resolve}({encoding}, [][2]string{{{origins}}}, {value}, evidence, correlation, occurrence) }}"
    );
}
