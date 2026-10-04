//! `{related: …}` through an Optional reference or across two references (ess/22,
//! beyond10x/ess#285) stays an obligation of the generated behaviour, as every `{related:}` value
//! does, and the plan names which form kept the command owed.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{CapabilityKind, SynthesisDisposition, SynthesisPlan};

const CHAINED: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-values-chained.yaml");
const TWO_HOPS: &str = "{related: {via: [objective_id, initiative_id], field: outcome_id}}";
const ONE_OPTIONAL_HOP: &str = "{related: {via: initiative_id, field: outcome_id}}";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("costs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

fn owed_because(text: &str) -> String {
    let plan = SynthesisPlan::of(&ir(text));
    match plan.disposition_of(CapabilityKind::CommandBehavior, "demo.costs.Book") {
        Some(SynthesisDisposition::Obligation(obligation)) => obligation.reason.describes(),
        other => panic!("`Book` stays an obligation, not {other:?}"),
    }
}

#[test]
fn issue_285_a_chained_read_keeps_its_command_owed_naming_the_chain() {
    let why = owed_because(CHAINED);
    assert!(why.contains("`{related:}` across two references"), "{why}");
}

#[test]
fn issue_285_an_optional_reference_keeps_its_command_owed_naming_the_absence() {
    let why = owed_because(&CHAINED.replace(TWO_HOPS, ONE_OPTIONAL_HOP));
    assert!(
        why.contains("`{related:}` through an Optional reference"),
        "{why}"
    );
}
