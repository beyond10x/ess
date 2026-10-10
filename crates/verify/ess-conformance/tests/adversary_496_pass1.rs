//! Adversary pass 1 on beyond10x/ess#496: a stored-field guard no view observes.
//!
//! The unit's CHANGELOG entry and design page both promise that a model whose views cover the
//! guarded fields keeps its suite byte for byte ("A suite whose model declares a covering view
//! keeps its bytes", `docs/design/cross-record-and-stored-field-guards.md`). Since
//! beyond10x/ess#172 an `eventual` view with the identity, `state` and every guarded field is such
//! a covering view. The case below holds the promise to that model.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::synthesize;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const PARCELS: &str = include_str!("fixtures/stored-field-guards.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("parcels.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The parcel model with its one view of the parcel made `eventual`: that view still projects the
/// identity, `state`, `service` and `weight_kg`, so it covers every guarded field. Before
/// beyond10x/ess#496 (and in the released `ess` 0.57.0) its suite held exactly these four
/// scenarios, the `refused-overweight` refusal being refused for want of an immediate view for the
/// absent-subject send. A suite kept byte for byte keeps that scenario set.
#[test]
fn a_model_whose_eventual_view_covers_the_guarded_fields_keeps_its_suite() {
    let eventual = PARCELS.replace(
        "    consistency: read_your_writes\n",
        "    consistency: eventual\n",
    );
    assert_ne!(eventual, PARCELS, "the view is made eventual");
    let ids: Vec<String> = synthesize(&ir(&eventual))
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        ids,
        [
            "shipping.parcel.Create/outcome/created",
            "shipping.parcel.Dispatch/outcome/dispatched",
            "shipping.parcel.Parcel/state/Dispatched/refuses/shipping.parcel.Dispatch",
            "shipping.parcel.Parcel/transition/dispatch/by/shipping.parcel.Dispatch/dispatched",
        ],
        "a model with a covering (eventual) view gained a scenario, so its suite did not keep its \
         bytes as the CHANGELOG and the design page say"
    );
}
