//! A guard, a filter and an order over a `Binary64` are generated, membership stays owed, and the
//! Clap target keeps its earlier refusal at every `Binary64` location.

#[path = "binary64_served/support.rs"]
mod support;

use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, SynthesisPlan, Target};

#[test]
fn a_guard_a_filter_and_an_order_over_a_binary64_are_generated() {
    let plan = SynthesisPlan::of(&support::model());
    let shown = plan.to_canonical_json();
    assert!(
        plan.is_generated(CapabilityKind::CommandBehavior, support::ADD),
        "{shown}"
    );
    assert!(
        plan.is_generated(CapabilityKind::ViewQuery, support::DOCS),
        "{shown}"
    );
}

#[test]
fn membership_over_a_binary64_stays_owed() {
    let source =
        support::MODEL.replace("when: \"input.added >= 0.5\"", "when: {added: [0.5, 1.5]}");
    assert_ne!(source, support::MODEL);
    let plan = SynthesisPlan::of(&support::compile(&source));
    let Some(SynthesisDisposition::Obligation(owed)) =
        plan.disposition_of(CapabilityKind::CommandBehavior, support::ADD)
    else {
        panic!("{}", plan.to_canonical_json());
    };
    let owed = serde_json::to_string(owed).expect("serializes");
    assert!(owed.contains("membership over a Binary64"), "{owed}");
}

#[test]
fn clap_refuses_the_fixture_at_every_binary64_location_with_the_earlier_detail() {
    let Err(failure) = synthesize_for(&support::model(), Target::Clap) else {
        panic!("Clap carries no Binary64");
    };
    let mut sources: Vec<String> = failure
        .causes()
        .iter()
        .flat_map(|cause| cause.sources().to_vec())
        .collect();
    sources.sort();
    assert_eq!(
        sources,
        [
            "command.demo.search.AddDoc.input.added",
            "entity demo.search.Doc.fields.added",
            "error.demo.search.TooLow.fields.added",
            "event.demo.search.DocAdded.fields.added",
            "types.demo.search.Hit.fields.by_name.value",
            "types.demo.search.Hit.fields.many.of",
            "types.demo.search.Hit.fields.maybe.of",
            "types.demo.search.Hit.fields.score",
            "types.demo.search.Ratio.of",
            "view.demo.search.DocsByAdded.fields.added",
        ]
    );
    assert!(failure
        .causes()
        .iter()
        .all(|cause| cause.detail() == "this target has no qualified finite Binary64 codec"));
}
