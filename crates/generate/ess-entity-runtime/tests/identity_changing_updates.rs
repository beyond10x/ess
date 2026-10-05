//! An `updates:` whose `sets:` writes the identity re-keys the record (ess/23, beyond10x/ess#429,
//! `docs/design/identity-changing-updates.md`) at Entity Runtime lowering.
//!
//! An entity-core operation acts on the instance its request names and has no move of an instance
//! to another identity, so the re-key is refused by name (`IdentityChangeUnsupported`) rather than
//! lowered as an update that writes a field. The same command updating another field earns no such
//! refusal.
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringCode, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/identity-changing-updates.yaml");

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("vault.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn lowered(ir: &EssIr) -> Vec<(LoweringCode, String, String)> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("vault-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("demo.vault.Secret").unwrap(),
            NonZeroU32::new(1).unwrap(),
        )]
        .into_iter()
        .collect(),
        scales: BTreeMap::new(),
    };
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .into_vec()
            .into_iter()
            .map(|d| (d.code, d.path.clone(), d.message.clone()))
            .collect(),
    }
}

#[test]
fn a_rename_is_refused_by_name() {
    let refused = lowered(&ir_of(MODEL));
    assert!(
        refused.iter().any(|(code, path, message)| {
            *code == LoweringCode::IdentityChangeUnsupported
                && path == "demo.vault.RenameSecret.renamed.sets.name"
                && message.contains("re-key")
        }),
        "{refused:#?}"
    );
}

#[test]
fn an_update_of_another_field_earns_no_such_refusal() {
    let plain = MODEL.replace(
        "        sets: {name: input.new_name}\n",
        "        sets: {value: input.new_name}\n",
    );
    assert_ne!(plain, MODEL);
    // `value` is a `String` and `new_name` a `SecretName`: read it through the same newtype.
    let plain = plain.replace(
        "      - {name: value, type: String}\n    lifecycle:",
        "      - {name: value, type: demo.vault.SecretName}\n    lifecycle:",
    );
    let plain = plain.replace(
        "      - {name: value, type: String}\ncomponents:",
        "      - {name: value, type: demo.vault.SecretName}\ncomponents:",
    );
    let plain = plain.replace(
        "      - {name: value, type: String}\n    outcomes:",
        "      - {name: value, type: demo.vault.SecretName}\n    outcomes:",
    );
    let refused = lowered(&ir_of(&plain));
    assert!(
        refused
            .iter()
            .all(|(code, _, _)| *code != LoweringCode::IdentityChangeUnsupported),
        "{refused:#?}"
    );
}
