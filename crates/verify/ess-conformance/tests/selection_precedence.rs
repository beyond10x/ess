//! A branch the held state selects is declared before every accepting and external branch of its
//! command (beyond10x/ess#486).
//!
//! The held state selects at step 4 of the precedence order and the accepting and external
//! branches at step 6, whatever their declaration order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order"). The model
//! interpreter read the three in one declaration-order pass, so a command declaring an accepting or
//! external branch first answered differently here than in the Rust and Go targets. Validation now
//! refuses that order, and on every order it admits the interpreter answers as the design does.
//!
//! `tests/fixtures/external-beside-held-guard.yaml`: `CheckPick` refuses a pick whose stored
//! `revision` differs from the input's (`stale`, a `when_subject` guard) and declares the external
//! `unlisted` after it.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_domain::{
    command::OutcomeName,
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

const STALE_BRANCH: &str = "      - name: stale\n";
const UNLISTED_BRANCH: &str = "      - name: unlisted\n";
const ACCEPTED_BRANCH: &str = "      - name: accepted\n";

/// `CheckPick`'s input with a Boolean `rush`, and `rushed`, an accepting branch taking it.
const INPUT: &str = "      - {name: pick_id, type: demo.desk.PickId}\n      - {name: revision, type: Integer}\n    outcomes:\n";
const RUSH_INPUT: &str = "      - {name: pick_id, type: demo.desk.PickId}\n      - {name: revision, type: Integer}\n      - {name: rush, type: Boolean}\n    outcomes:\n";
const RUSHED_BRANCH: &str = "      - name: rushed\n        when: rush == true\n        moves: demo.desk.Pick.accept\n        instance: pick_id\n        emits: [demo.desk.PickAccepted]\n        payload:\n          demo.desk.PickAccepted: {pick_id: input.pick_id}\n";

fn specification(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = specification(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `model` with the branch starting at `moved` declared before the branch starting at `anchor`.
fn declared_before(model: &str, moved: &str, anchor: &str, next: &str) -> String {
    let at_anchor = model.find(anchor).expect("the anchor is declared");
    let at_moved = model.find(moved).expect("the moved branch is declared");
    let at_next = model.find(next).expect("the branch after it is declared");
    assert!(at_anchor < at_moved && at_moved < at_next);
    format!(
        "{}{}{}{}",
        &model[..at_anchor],
        &model[at_moved..at_next],
        &model[at_anchor..at_moved],
        &model[at_next..]
    )
}

/// The fixture with `rush` declared, and `rushed` inserted before the branch starting at `before`.
fn with_rushed_before(before: &str) -> String {
    let model = MODEL.replacen(INPUT, RUSH_INPUT, 1);
    assert_ne!(model, MODEL, "`CheckPick`'s input is declared");
    let rushed = model.replacen(before, &format!("{RUSHED_BRANCH}{before}"), 1);
    assert_ne!(rushed, model, "the anchor is declared");
    rushed
}

fn refusal(model: &str) -> String {
    specification(model).map_or_else(|errors| errors, |_| panic!("the model validates:\n{model}"))
}

fn name(text: &str) -> QualifiedName {
    QualifiedName::new(text).unwrap()
}

fn integer(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// A store holding one `Picked` pick of `revision`, made by `MakePick`, and its identity.
fn picked(ir: &EssIr, revision: i64) -> (Store, Node) {
    let steps = execute(
        ir,
        &Store::default(),
        &name("demo.desk.MakePick"),
        &BTreeMap::from([("revision".to_owned(), integer(revision))]),
        &Externals::Withheld,
    )
    .expect("the model determines MakePick");
    let [step] = steps.as_slice() else {
        panic!("MakePick answers once: {steps:#?}");
    };
    let (_, identity, _) = step
        .next
        .instances()
        .next()
        .expect("MakePick creates a pick");
    (step.next.clone(), identity.clone())
}

/// The branches `CheckPick` answers on a pick holding revision 1, sent `input` beside its identity.
fn answered(model: &str, input: &[(&str, Node)], externals: &Externals) -> BTreeSet<String> {
    let ir = ir(model);
    let (store, pick) = picked(&ir, 1);
    let mut sent = BTreeMap::from([("pick_id".to_owned(), pick)]);
    sent.extend(
        input
            .iter()
            .map(|(field, value)| ((*field).to_owned(), value.clone())),
    );
    execute(&ir, &store, &name("demo.desk.CheckPick"), &sent, externals)
        .unwrap_or_else(|why| panic!("the model determines CheckPick: {why}"))
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect()
}

fn one(branch: &str) -> BTreeSet<String> {
    BTreeSet::from([format!("demo.desk.CheckPick/{branch}")])
}

fn forced(branch: &str) -> Externals {
    Externals::Forced(OutcomeName::new(branch).unwrap())
}

#[test]
fn an_accepting_branch_declared_before_a_held_state_branch_is_refused() {
    let errors = refusal(&with_rushed_before(STALE_BRANCH));
    assert!(
        errors.contains(
            "[conflicting_declaration] command.demo.desk.CheckPick.outcomes.stale: `stale` is \
             selected by the held state, which answers before the accepting branch `rushed` \
             declared above it"
        ),
        "{errors}"
    );
    assert!(
        errors.contains("declare `stale` before `rushed`"),
        "{errors}"
    );
}

#[test]
fn an_external_branch_declared_before_a_held_state_branch_is_refused() {
    let errors = refusal(&declared_before(
        MODEL,
        UNLISTED_BRANCH,
        STALE_BRANCH,
        ACCEPTED_BRANCH,
    ));
    assert!(
        errors.contains(
            "[conflicting_declaration] command.demo.desk.CheckPick.outcomes.stale: `stale` is \
             selected by the held state, which answers before the external branch `unlisted` \
             declared above it"
        ),
        "{errors}"
    );
    assert!(
        errors.contains("declare `stale` before `unlisted`"),
        "{errors}"
    );
}

/// Declared before the external branch, the held-state branch answers wherever its guard holds,
/// under every provider.
#[test]
fn declared_first_the_held_state_branch_answers_before_the_external_one() {
    let stale = [("revision", integer(2))];
    assert_eq!(answered(MODEL, &stale, &Externals::Withheld), one("stale"));
    assert_eq!(answered(MODEL, &stale, &forced("unlisted")), one("stale"));
    assert_eq!(answered(MODEL, &stale, &Externals::Open), one("stale"));
    let current = [("revision", integer(1))];
    assert_eq!(
        answered(MODEL, &current, &forced("unlisted")),
        one("unlisted")
    );
    assert_eq!(
        answered(MODEL, &current, &Externals::Withheld),
        one("accepted")
    );
}

/// Declared before the accepting branch, the held-state branch answers where both guards hold, and
/// the accepting branch where only its own does.
#[test]
fn declared_first_the_held_state_branch_answers_before_the_accepting_one() {
    let model = with_rushed_before(UNLISTED_BRANCH);
    let rush =
        |revision: i64, rush: bool| [("revision", integer(revision)), ("rush", Node::Bool(rush))];
    assert_eq!(
        answered(&model, &rush(2, true), &Externals::Withheld),
        one("stale")
    );
    assert_eq!(
        answered(&model, &rush(1, true), &Externals::Withheld),
        one("rushed")
    );
    assert_eq!(
        answered(&model, &rush(1, false), &Externals::Withheld),
        one("accepted")
    );
}

// ---- the one precedence rule (G4) ----------------------------------------------------------

const RULE: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/precedence-rule-format-a.yaml");

/// The suite `precedence-rule-format-a.yaml` synthesizes under the one rule witnesses every branch
/// of `ChangeTier`, refuses none, and the interpreted target passes every scenario; the generated
/// Rust and Go targets run the same suite in `ess-synth`'s
/// `related_guard_unknown_instance_served.rs`.
#[test]
fn format_a_fixture_suite_passes_on_the_interpreted_target() {
    use ess_conformance::interpret::Interpreted;
    use ess_conformance::report::Status;
    use ess_conformance::synthesize::synthesize;
    use ess_conformance::{AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};

    let model = ir(RULE);
    let synthesis = synthesize(&model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for branch in [
        "blank-reason",
        "unknown-seat",
        "unknown-tier",
        "tier-closed",
        "changed",
    ] {
        let id = format!("demo.rule.ChangeTier/outcome/{branch}");
        assert!(ids.contains(&id), "{id}: {ids:#?}");
    }
    // `wrong_state:` is witnessed in each state no move of the command starts from.
    for state in ["Moved", "Released"] {
        let id = format!("demo.rule.Seat/state/{state}/refuses/demo.rule.ChangeTier");
        assert!(ids.contains(&id), "{id}: {ids:#?}");
    }
    // The missing tier's scenario first sends the blank reason for it, answered by the input
    // refusal at step 1: a target reading the tier first answers `unknown-tier` there and fails.
    let missing = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.rule.ChangeTier/outcome/unknown-tier")
        .map(|(_, scenario)| scenario)
        .expect("synthesized");
    let expected: Vec<String> = missing
        .steps
        .iter()
        .filter_map(|step| match step {
            ess_conformance::ScenarioStep::ExpectOutcome { outcome } => Some(outcome.to_string()),
            _ => None,
        })
        .filter(|outcome| outcome.starts_with("demo.rule.ChangeTier/"))
        .collect();
    assert_eq!(
        expected,
        [
            "demo.rule.ChangeTier/blank-reason",
            "demo.rule.ChangeTier/unknown-tier"
        ],
        "{:#?}",
        missing.steps
    );
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(&synthesis.suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(model))
    .into_report();
    let failed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
