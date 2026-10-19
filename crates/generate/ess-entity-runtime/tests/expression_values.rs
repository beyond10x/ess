//! An expression value, `{expr: …}` (`ess/27`, beyond10x/ess#560,
//! `docs/design/generic-language-features.md` § G1), at Entity Runtime lowering.
//!
//! Entity-core computes no expression, so a branch storing or publishing one is refused by name
//! (`ValueExpressionUnsupported`, construct "an expression value") rather than lowered as a value
//! the implementation supplies. The same command copying its input earns no such refusal.
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

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/expression-arithmetic-and-time.yaml"
);

const COMPONENT: &str = "components:
  - component: ledger-service
    owns: {domains: [demo.ledger]}
    accepts: {commands: [demo.ledger.RecordPastIncident]}
    publishes: {events: [demo.ledger.IncidentRecorded]}
    reached_by: network
";

fn ir_of(text: &str) -> EssIr {
    let text = format!("{text}{COMPONENT}");
    let spec = Specification::assemble([(
        Source::new("ledger.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn lowered(ir: &EssIr) -> Vec<(LoweringCode, String, String)> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("ledger-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("demo.ledger.Incident").unwrap(),
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
            .map(|d| (d.code, d.construct.to_owned(), d.message.clone()))
            .collect(),
    }
}

#[test]
fn an_expression_value_is_refused_by_name() {
    let refused = lowered(&ir_of(MODEL));
    assert!(
        refused.iter().any(|(code, construct, message)| {
            *code == LoweringCode::ValueExpressionUnsupported
                && construct == "an expression value (`{expr: …}`, ess/27)"
                && message.contains("input.ended_at - input.started_at")
        }),
        "{refused:#?}"
    );
}

#[test]
fn the_same_command_copying_its_input_earns_no_such_refusal() {
    let plain = MODEL
        .replace(
            "{expr: \"input.ended_at - input.started_at\"}",
            "{generated: true}",
        )
        .replace(
            "{expr: \"coalesce(round(input.affected / input.total, 2, half_even), 0)\"}",
            "{generated: true}",
        )
        .replace(", expr: \"ended_at - started_at\"}", "}")
        .replace("      - {name: elapsed, type: Duration}\n", "");
    let refused = lowered(&ir_of(&plain));
    assert!(
        refused
            .iter()
            .all(|(code, _, _)| *code != LoweringCode::ValueExpressionUnsupported),
        "{refused:#?}"
    );
}
