//! Adversary pass 1 against `story:selection-plan-design-and-type` (wave 2, unit U1): a mutant of
//! `Composition::of` the unit's suite does not see.
//!
//! `Composition::of` counts the distinct rows a command reads through its input (`sort_unstable`
//! then `dedup`). Its only callers are `tests/precedence_classification.rs`, on
//! `related-guard-multiple.yaml` (two rows, four branches: "more than one" either way) and on a
//! model reading no row. A `Composition::of` that counted branches instead of rows would keep them
//! green, and would order the present-related refusal of every `ess/22` command reading one row
//! through two branches at step 5 — the validation side the next story moves onto it.
use ess_domain::command::precedence::{order, BranchShape, Composition, Phase};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;

const LEASES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml");

#[test]
fn adv_u1_one_row_read_by_two_branches_is_one_row_to_composition_of() {
    let raw = RawSpecFile::parse(LEASES).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let join = &spec.commands()[&"demo.leases.Join".parse().unwrap()];
    let format = spec.system().format;
    assert_eq!(
        format.major(),
        ess_domain::system::FormatVersion::V22.major()
    );
    let composition = Composition::of(join, format);
    assert!(
        !composition.orders_present_related_refusals(),
        "`Join` reads one member row (`exists: false` and a predicate over it) and declares no \
         `wrong_state`: its refusal keeps declaration order"
    );
    let shapes: Vec<BranchShape> = join.outcomes.iter().map(BranchShape::of).collect();
    let banned = join
        .outcomes
        .iter()
        .position(|outcome| outcome.name.as_str() == "banned-from-joining")
        .expect("the fixture declares `banned-from-joining`");
    let phase = order(&shapes, &composition)
        .into_iter()
        .find_map(|(phase, held)| held.contains(&banned).then_some(phase));
    assert_eq!(phase, Some(Phase::Accepting));
    assert!(
        Composition::new(&shapes, 2, format).orders_present_related_refusals(),
        "the row count decides it: counting `Join`'s two related branches as two rows orders it"
    );
}
