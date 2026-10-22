//! A view filter ordering a field against `now` (`ess/30`, beyond10x/ess#560,
//! `story:expression-arithmetic-and-time`, `now_relative_view_filter`) in the generated Rust and Go
//! targets. A generated view query takes its parameters and the store, and no clock: it cannot read
//! the instant of the read, so the query stays an obligation named by the bound it would compare,
//! rather than a query that compares it with an instant of its own choosing. The command that
//! creates the rows is still generated.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-view-filter.yaml");

const COMPONENT: &str = "components:
  - component: ledger-service
    owns: {domains: [demo.ledger]}
    accepts: {commands: [demo.ledger.RecordIncident]}
    publishes: {events: [demo.ledger.IncidentRecorded]}
    reached_by: network
";

fn model() -> EssIr {
    let text = format!("{MODEL}{COMPONENT}");
    let spec = Specification::assemble([(
        Source::new("ledger.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn a_view_filter_reading_now_keeps_its_query_an_obligation_named_by_the_bound() {
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&model(), target).expect("a realizable target");
        match synthesis
            .plan
            .disposition_of(CapabilityKind::ViewQuery, "demo.ledger.RecentIncidents")
        {
            Some(SynthesisDisposition::Obligation(owed)) => {
                let reason = owed.reason.describes();
                assert!(
                    reason.contains("started_at >= now - 720h")
                        && reason.contains("the current time")
                        && reason.contains("in `filter:`"),
                    "{target:?}: {reason}"
                );
            }
            other => panic!("{target:?}: the query is owed, not {other:?}"),
        }
        assert_eq!(
            synthesis.plan.disposition_of(
                CapabilityKind::CommandBehavior,
                "demo.ledger.RecordIncident"
            ),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}
