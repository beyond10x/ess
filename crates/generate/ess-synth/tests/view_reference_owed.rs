//! A view read through a reference, and a value reading the referenced row's state, stay owed by
//! generated Rust and Go, named by the plan (`ess/28`, beyond10x/ess#564, beyond10x/ess#557,
//! `docs/design/generic-language-features.md` § G2): a generated query reads its source's rows
//! alone, and generated behaviour reads no `{related:}` value.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const DESK: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/navigate-a-reference.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("desk.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn owed(synthesis: &ess_synth::Synthesis, kind: CapabilityKind, name: &str) -> Option<String> {
    match synthesis.plan.disposition_of(kind, name) {
        Some(SynthesisDisposition::Obligation(owed)) => Some(owed.reason.describes()),
        _ => None,
    }
}

#[test]
fn generated_view_queries_name_the_read_through_a_reference_owed() {
    let ir = model(DESK);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        for view in ["relstate.desk.Tickets", "relstate.desk.TicketsAtOpenDesks"] {
            let reason = owed(&synthesis, CapabilityKind::ViewQuery, view)
                .unwrap_or_else(|| panic!("{target:?}: `{view}` is owed"));
            assert!(
                reason.contains("reference `desk_id`") && reason.contains("relstate.desk.Desk"),
                "{target:?}: `{view}` names the reference: {reason}"
            );
        }
        // The view over the referenced entity's own rows is still generated.
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::ViewQuery, "relstate.desk.Desks"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}

#[test]
fn generated_behaviour_names_the_referenced_rows_state_owed() {
    let ir = model(DESK);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let reason = owed(
            &synthesis,
            CapabilityKind::CommandBehavior,
            "relstate.desk.OpenTicket",
        )
        .unwrap_or_else(|| panic!("{target:?}: `OpenTicket` is owed"));
        assert!(
            reason.contains("field: state") && reason.contains("current"),
            "{target:?}: the state read is named: {reason}"
        );
    }
}

const WIRING: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/two-rows-one-branch.yaml");

/// A branch requiring two related rows at once (`when_related: {all: […]}`, ess/28, downstream
/// report 2026-10-19, finding 11) keeps its command owed, named for the rows it reads.
#[test]
fn generated_behaviour_names_two_rows_in_one_branch_owed() {
    let ir = model(WIRING);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let reason = owed(
            &synthesis,
            CapabilityKind::CommandBehavior,
            "wiring.board.Connect",
        )
        .unwrap_or_else(|| panic!("{target:?}: `Connect` is owed"));
        assert!(
            reason.contains("related row"),
            "{target:?}: the related rows are named: {reason}"
        );
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "wiring.board.AddPort"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}
