//! The held lifecycle state as a value source, `{subject: state}` (ess/23, beyond10x/ess#458), at
//! Entity Runtime lowering.
//!
//! Lowering refuses every `{subject: …}` value by name, and the held state is one: a `wrong_state:`
//! payload reading it reports `ValueExpressionUnsupported` naming the `` `{subject: …}` `` construct
//! at `<command>.<outcome>`, and lowers nothing for that outcome. No `$from_state` payload lowering
//! exists to reuse; `$from_state` is a condition operand only.
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
    include_str!("../../../verify/ess-conformance/tests/fixtures/subject-state-source.yaml");

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("docs.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn lowered(ir: &EssIr) -> Vec<(LoweringCode, String, String, String)> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("docs-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("demo.docs.Doc").unwrap(),
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
            .map(|d| {
                (
                    d.code,
                    d.path.clone(),
                    d.construct.to_owned(),
                    d.message.clone(),
                )
            })
            .collect(),
    }
}

/// The model with every other `{subject: state}` read replaced by a literal, so the publishing
/// conflict is the one reading it.
fn only_the_conflict() -> String {
    MODEL
        .replace(
            "          demo.docs.DocPublished: {doc_id: input.doc_id, from: {subject: state}}\n",
            "          demo.docs.DocPublished: {doc_id: input.doc_id, from: Draft}\n",
        )
        .replace(
            "        sets: {previous: {subject: state}}\n",
            "        sets: {previous: Published}\n",
        )
        .replace(
            "{doc_id: input.doc_id, current: {subject: state}, requested: Archived}",
            "{doc_id: input.doc_id, current: Published, requested: Archived}",
        )
}

#[test]
fn entity_runtime_refuses_subject_state_by_name() {
    let text = only_the_conflict();
    assert_eq!(text.matches("{subject: state}").count(), 1, "{text}");
    let refused = lowered(&ir_of(&text));
    let named: Vec<_> = refused
        .iter()
        .filter(|(code, _, _, _)| *code == LoweringCode::ValueExpressionUnsupported)
        .collect();
    assert!(
        named.iter().any(
            |(_, path, construct, _)| path == "demo.docs.PublishDoc.conflict"
                && construct == "`{subject: …}`"
        ),
        "{refused:#?}"
    );
    // Nothing else about the outcome is lowered or refused under another code.
    assert!(
        refused
            .iter()
            .filter(|(_, path, _, _)| path.starts_with("demo.docs.PublishDoc.conflict"))
            .all(|(code, _, _, _)| *code == LoweringCode::ValueExpressionUnsupported),
        "{refused:#?}"
    );
}
