//! A view filter ordering a field against `now` (`ess/30`, beyond10x/ess#560,
//! `story:expression-arithmetic-and-time`, `now_relative_view_filter`) at Entity Runtime lowering.
//!
//! entity-core has no clock operand, so the filter is refused by name, as a command guard reading
//! `now` is (`tests/current_time_guard.rs`), rather than lowered to an ordering against the text
//! `now - 720h`. The same view filtered on a fixed instant lowers.
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
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-view-filter.yaml");

const COMPONENT: &str = "components:
  - component: ledger-service
    owns: {domains: [demo.ledger]}
    accepts: {commands: [demo.ledger.RecordIncident]}
    publishes: {events: [demo.ledger.IncidentRecorded]}
    reached_by: network
";

const NOW_FILTER: &str = "filter: \"started_at >= now - 720h\"";

fn ir(filter: &str) -> EssIr {
    let text = format!("{}{COMPONENT}", MODEL.replacen(NOW_FILTER, filter, 1));
    let spec = Specification::assemble([(
        Source::new("ledger.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn lowered(ir: &EssIr) -> Result<(), Vec<(LoweringCode, String, String)>> {
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
    lower(&service, &options)
        .map(|_| ())
        .map_err(|diagnostics| {
            diagnostics
                .into_vec()
                .into_iter()
                .map(|d| (d.code, d.path.clone(), d.message.clone()))
                .collect()
        })
}

#[test]
fn a_view_filter_reading_now_is_refused_by_name() {
    let refused = lowered(&ir(NOW_FILTER))
        .expect_err("entity-core has no clock operand, so the view filter must be refused");
    assert!(
        refused.iter().any(|(code, path, message)| {
            *code == LoweringCode::CurrentTimeUnsupported
                && path.contains("RecentIncidents")
                && message.contains("started_at")
                && message.contains("now - 720h")
        }),
        "{refused:?}"
    );
}

#[test]
fn the_same_view_filtered_on_a_fixed_instant_lowers() {
    lowered(&ir("filter: \"started_at >= '2019-12-01T00:00:00Z'\""))
        .expect("a fixed instant has an entity-core ordering");
}
