//! Adversary pass 1, ESS 0.70.0 unit A (beyond10x/ess#554): from `ess/32` a command may guard on
//! rows its input names beside a row set. The story keeps Entity Runtime's `RelatedGuardUnsupported`
//! for such a command, and the format history says lowering refuses it by name. The reproduction
//! declares no component, so nothing lowered it; here it is lowered inside one.

use std::collections::BTreeMap;
use std::num::NonZeroU32;

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

const REPRO: &str =
    include_str!("../../../specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml");

const COMPONENT: &str = "components:
  - component: wiring-service
    owns: {domains: [wiring.board]}
    accepts: {commands: [wiring.board.AddPort, wiring.board.Connect]}
    publishes: {events: [wiring.board.PortAdded, wiring.board.Connected]}
    reached_by: network
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

fn lowered(ir: &EssIr) -> Vec<LoweringDiagnostic> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("wiring-service").unwrap())
        .unwrap_or_else(|error| panic!("extracts: {error:?}"));
    let options = LoweringOptions {
        definition_versions: ["wiring.board.Port", "wiring.board.Wire"]
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

/// Every identity-addressed branch of `Connect` is refused as `RelatedGuardUnsupported`, the row
/// set as `RowSetUnsupported`, each at its own branch; nothing of `AddPort` is refused.
#[test]
fn adversary_a_entity_runtime_refuses_identity_beside_row_set_by_name() {
    let text = format!("{REPRO}{COMPONENT}");
    let diagnostics = lowered(&compiled(&text));
    let at = |code: LoweringCode| -> Vec<String> {
        let mut paths: Vec<String> = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == code)
            .map(|diagnostic| diagnostic.path.clone())
            .collect();
        paths.sort();
        paths
    };
    assert_eq!(
        at(LoweringCode::RelatedGuardUnsupported),
        [
            "wiring.board.Connect.no-sink.when_related",
            "wiring.board.Connect.no-source.when_related",
            "wiring.board.Connect.sink-not-input.when_related",
            "wiring.board.Connect.source-not-output.when_related",
        ],
        "{diagnostics:#?}"
    );
    assert_eq!(
        at(LoweringCode::RowSetUnsupported),
        ["wiring.board.Connect.source-taken.when_related"],
        "{diagnostics:#?}"
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.path.starts_with("wiring.board.AddPort")),
        "{diagnostics:#?}"
    );
}
