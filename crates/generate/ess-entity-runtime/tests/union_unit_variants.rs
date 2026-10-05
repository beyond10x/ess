//! A union with a unit variant (ess/22, beyond10x/ess#418) has no exact Entity Runtime field
//! definition: an entity-core union variant always admits a payload member, and a unit variant is
//! the tag alone. Lowering refuses it by name — `UnitVariantUnsupported` — rather than lowering a
//! definition that admits `{"kind": "Open", "value": …}`.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringCode, LoweringDiagnostic, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

/// A union `Status` stored on `Child`, with `open` written as `variant`.
fn contract(open: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let changes = [
        ("system.yaml", "format: ess/4", "format: ess/22".to_owned()),
        (
            "domains/local.yaml",
            "types:\n",
            format!(
                "types:\n  - name: contract.local.Status\n    kind: union\n    tag: kind\n    \
                 variants:\n      {open}\n      closed: String\n\n"
            ),
        ),
        (
            "domains/local.yaml",
            "      - name: memo\n        type: Optional<String>\n",
            "      - name: memo\n        type: Optional<String>\n      - name: status\n        \
             type: Optional<contract.local.Status>\n"
                .to_owned(),
        ),
    ];
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let (mut parsed, mut labels, mut sources) = (Vec::new(), Vec::new(), SourceMap::new());
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{target} holds {before}");
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn diagnostics(ir: &EssIr) -> Vec<LoweringDiagnostic> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("local-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
            .into_iter()
            .map(|name| {
                (
                    QualifiedName::new(name).unwrap(),
                    NonZeroU32::new(1).unwrap(),
                )
            })
            .collect(),
        scales: BTreeMap::new(),
    };
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics.into_vec(),
    }
}

#[test]
fn a_stored_union_with_a_unit_variant_is_refused_by_name() {
    let diagnostics = diagnostics(&contract("open:"));
    let unit: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LoweringCode::UnitVariantUnsupported)
        .collect();
    // The stored field, and every operation field lowered from it: each is refused where it sits.
    assert!(
        unit.iter()
            .any(|diagnostic| diagnostic.path == "contract.local.Child.fields.status.open"),
        "{diagnostics:#?}"
    );
    for diagnostic in &unit {
        assert!(
            diagnostic.path.rsplit('.').next() == Some("open"),
            "{diagnostics:#?}"
        );
        assert!(
            diagnostic.message.contains("tag alone"),
            "{}",
            diagnostic.message
        );
    }
}

#[test]
fn the_same_union_with_a_payload_variant_is_not_refused_for_it() {
    let diagnostics = diagnostics(&contract("open: String"));
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != LoweringCode::UnitVariantUnsupported),
        "{diagnostics:#?}"
    );
}
