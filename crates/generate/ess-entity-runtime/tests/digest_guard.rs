//! A guard reading a fact through a digest or encoding function (`ess/27`, beyond10x/ess#584) at
//! Entity Runtime lowering: entity-core has no digest or encoding operand, so the guard is refused
//! by name (`FunctionCallUnsupported`) rather than lowered as a read of the fact. The same command
//! comparing the stored challenge with the verifier itself, PKCE `plain`, earns no such refusal.
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
    include_str!("../../../verify/ess-conformance/tests/fixtures/digest-guard.yaml");

const COMPONENT: &str = "components:
  - component: pkce-service
    owns: {domains: [demo.pkce]}
    accepts: {commands: [demo.pkce.IssueCode, demo.pkce.Exchange]}
    publishes: {events: [demo.pkce.CodeIssued, demo.pkce.CodeRedeemed]}
    reached_by: network
";

fn ir_of(text: &str) -> EssIr {
    let text = format!("{text}{COMPONENT}");
    let spec =
        Specification::assemble([(Source::new("pkce.yaml"), RawSpecFile::parse(&text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn lowered(ir: &EssIr) -> Vec<(LoweringCode, String, String)> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("pkce-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("demo.pkce.Code").unwrap(),
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
fn a_guard_reading_a_fact_through_a_function_is_refused_by_name() {
    let refused = lowered(&ir_of(MODEL));
    assert!(
        refused.iter().any(|(code, path, message)| {
            *code == LoweringCode::FunctionCallUnsupported
                && path.contains("demo.pkce.Exchange")
                && message.contains("through a function")
        }),
        "{refused:#?}"
    );
}

#[test]
fn the_plain_method_earns_no_such_refusal() {
    let plain = MODEL.replace(
        "challenge != base64url(sha256(ascii(input.verifier)))",
        "challenge != input.verifier",
    );
    let refused = lowered(&ir_of(&plain));
    assert!(
        refused
            .iter()
            .all(|(code, _, _)| *code != LoweringCode::FunctionCallUnsupported),
        "{refused:#?}"
    );
}
