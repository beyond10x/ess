//! Adversary pass 1 on beyond10x/ess#496: a stored-field guard no view observes.
//!
//! A model whose views cover the guarded fields keeps its suite byte for byte only where one of
//! them is immediate. Since beyond10x/ess#172 an `eventual` view with the identity, `state` and
//! every guarded field also covers them, and since #496 the absent-subject send is written without
//! an immediate view (it claims no absence), so such a model gains that one scenario. The cases
//! below hold both halves.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::synthesize;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const PARCELS: &str = include_str!("fixtures/stored-field-guards.yaml");

const BEFORE: [&str; 4] = [
    "shipping.parcel.Create/outcome/created",
    "shipping.parcel.Dispatch/outcome/dispatched",
    "shipping.parcel.Parcel/state/Dispatched/refuses/shipping.parcel.Dispatch",
    "shipping.parcel.Parcel/transition/dispatch/by/shipping.parcel.Dispatch/dispatched",
];

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("parcels.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn ids(text: &str) -> Vec<String> {
    synthesize(&ir(text))
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

/// The parcel model with its one view of the parcel made `eventual`: that view still projects the
/// identity, `state`, `service` and `weight_kg`. Before #496 (and in the released `ess` 0.57.0)
/// its suite held the four scenarios of `BEFORE`, `refused-overweight` being refused for want of
/// an immediate view for the absent-subject send. It now gains that send and nothing else.
#[test]
fn a_model_whose_only_covering_view_is_eventual_gains_the_absent_subject_send() {
    let eventual = PARCELS.replace(
        "    consistency: read_your_writes\n",
        "    consistency: eventual\n",
    );
    assert_ne!(eventual, PARCELS, "the view is made eventual");
    let mut expected: Vec<String> = BEFORE.iter().map(ToString::to_string).collect();
    expected.insert(
        2,
        "shipping.parcel.Dispatch/outcome/refused-overweight".into(),
    );
    assert_eq!(ids(&eventual), expected);
}
