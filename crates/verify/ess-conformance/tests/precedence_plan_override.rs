//! The phase-order seam reaches a consumer: a precedence plan built by `ess-compiler`'s
//! constructor inside `ess-domain`'s `with_phase_order` has the exchanged order
//! (`docs/design/selection-plan.md`, "One test seam"). Every consumer story's "phases exchanged"
//! test builds on this.
use ess_compiler::ir::{EssIr, PrecedencePlan};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::precedence::{phase_order, with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const PICK: &str = include_str!("fixtures/external-beside-held-guard.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(PICK).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// The branch names of a plan, in the order it reads them.
fn read_order(ir: &EssIr) -> Vec<(Phase, String)> {
    let command = &ir.commands()[&"demo.desk.CheckPick".parse().unwrap()];
    PrecedencePlan::new(command, ir.format())
        .iter()
        .map(|(phase, outcome)| (phase, outcome.name.to_string()))
        .collect()
}

fn named(order: &[(Phase, &str)]) -> Vec<(Phase, String)> {
    order
        .iter()
        .map(|(phase, name)| (*phase, (*name).to_owned()))
        .collect()
}

#[test]
fn a_plan_built_inside_the_override_has_the_exchanged_phase_order() {
    let ir = ir();
    assert_eq!(
        read_order(&ir),
        named(&[
            (Phase::HeldState, "stale"),
            (Phase::HeldState, "wrong-state"),
            (Phase::Accepting, "unlisted"),
            (Phase::Default, "accepted"),
        ])
    );
    let mut exchanged = Phase::PRECEDENCE;
    let held = exchanged
        .iter()
        .position(|phase| *phase == Phase::HeldState)
        .unwrap();
    let accepting = exchanged
        .iter()
        .position(|phase| *phase == Phase::Accepting)
        .unwrap();
    exchanged.swap(held, accepting);

    let (phases, order) = with_phase_order(exchanged, || {
        let command = &ir.commands()[&"demo.desk.CheckPick".parse().unwrap()];
        let phases: Vec<Phase> = PrecedencePlan::new(command, ir.format())
            .phases()
            .iter()
            .map(|planned| planned.phase)
            .collect();
        (phases, read_order(&ir))
    });
    assert_eq!(
        phases, exchanged,
        "the constructor reads the overridden order"
    );
    assert_eq!(
        order,
        named(&[
            (Phase::Accepting, "unlisted"),
            (Phase::HeldState, "stale"),
            (Phase::HeldState, "wrong-state"),
            (Phase::Default, "accepted"),
        ]),
        "the external branch is read before the held-state branches"
    );
    assert_eq!(
        phase_order(),
        Phase::PRECEDENCE,
        "restored after the closure"
    );
    assert_eq!(read_order(&ir)[0], (Phase::HeldState, "stale".to_owned()));
}
