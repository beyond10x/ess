//! Every refusal a state only a set move reaches causes names that state.
//!
//! A set move is no driver of an arrangement (`docs/design/set-effects-over-filtered-instances.md`),
//! so no row is arranged in a state that only set moves reach. The `instances:` refusal and the
//! per-row trigger refusal name such a state and the set move reaching it. Two more refusals meet
//! the same cause:
//!
//! - `ChangedRowUnarranged`: `resume-on-poked` invokes `Resume/resumed`, which publishes an event
//!   per changed row and moves rows only from `Parked`, which only the set move `Park/parked`
//!   reaches (`set-move-only-changed-row.yaml`);
//! - the `affects:` entry refusal: `Clear/cleared` releases every child of its group from
//!   `Parked`, reached the same way (`set-move-only-affects.yaml`).
//!
//! Each names the state and the set move, with the help of the `instances:` refusal, rather than
//! asking for the filtered field to be written from an input, which the creator already does. A
//! refusal whose filter selects no row in any state keeps its old text.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// `Resume/resumed` publishes per changed row and starts only in `Parked`; a binding invokes it.
const CHANGED_ROW: &str = include_str!("fixtures/set-move-only-changed-row.yaml");
/// `Clear/cleared` releases, through an `affects:` entry, children resting only in `Parked`.
const AFFECTS: &str = include_str!("fixtures/set-move-only-affects.yaml");

const RESUMED: &str = "demo.z.Resume/outcome/resumed";
const DELIVERY: &str = "resume-on-poked/binding/delivery";
const FLOW: &str = "resume-on-poked/binding/flow";
const CLEARED: &str = "demo.g.Clear/outcome/cleared";

/// The help of the `instances:` refusal naming a state only a set move reaches.
const SET_MOVE_ONLY_HELP: &str = "a set move arranges no row for another scenario; declare a \
     single-row transition into the state (a branch moving the one instance its `instance:` names \
     there), or cover the branch with an authored scenario (ess-scenario/1)";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesize(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// The one refusal of `wanted`, as its code, its rendered cause and its help.
fn refusal_of(synthesis: &Synthesis, wanted: &str) -> (String, String, String) {
    let found: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(wanted)
        })
        .collect();
    let [refusal] = found.as_slice() else {
        panic!("{wanted} is refused once: {found:#?}");
    };
    (
        refusal.code().to_string(),
        refusal.cause.to_string(),
        refusal.hint().to_string(),
    )
}

/// `text` with `part`, which it holds once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

/// Every scenario of `synthesis` passes the interpreter of the model it was synthesized from.
fn passes(text: &str, synthesis: &Synthesis) {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    assert_eq!(
        results.len(),
        synthesis.suite.scenarios.len(),
        "{results:#?}"
    );
    assert!(
        results.values().all(|status| *status == Status::Passed),
        "every scenario passes on the interpreted target: {results:#?}"
    );
}

#[test]
fn a_changed_row_refusal_names_the_state_only_a_set_move_reaches() {
    let synthesis = synthesize(CHANGED_ROW);
    assert_eq!(
        synthesis.refusals.len(),
        4,
        "one refusal occurrence each for the branch, the delivery, the flow and the silent \
         failure policy: {:#?}",
        synthesis.refusals
    );
    let (_, _, instances_help) = refusal_of(&synthesis, RESUMED);
    assert_eq!(instances_help, SET_MOVE_ONLY_HELP);
    for wanted in [DELIVERY, FLOW] {
        let (code, cause, help) = refusal_of(&synthesis, wanted);
        assert_eq!(code, "ESS-SYNTH-010", "{cause}");
        assert_eq!(
            cause,
            "`resume-on-poked` invokes `demo.z.Resume/resumed`, which publishes an event once per \
             `demo.z.Child` it changes, and no row its filter selects can be arranged before the \
             trigger: `demo.z.Child`, whose move starts only in `Parked`, a state reached only by \
             the set move `demo.z.Park/parked`, which synthesis does not take to arrange a row \
             for another scenario",
            "{wanted} names the state and the set move reaching it"
        );
        assert_eq!(
            help, instances_help,
            "{wanted} gives the help of the instances: refusal"
        );
    }
    assert_eq!(
        ids(&synthesis),
        [
            "demo.z.AddChild/outcome/added",
            "demo.z.Park/outcome/parked",
            "demo.z.Poke/outcome/poked",
            "resume-on-poked/binding/mapping"
        ],
        "the suite is unchanged"
    );
    passes(CHANGED_ROW, &synthesis);
}

#[test]
fn an_affects_entry_refusal_names_the_state_only_a_set_move_reaches() {
    let synthesis = synthesize(AFFECTS);
    assert_eq!(
        synthesis.refusals.len(),
        1,
        "one refusal occurrence: {:#?}",
        synthesis.refusals
    );
    let (code, cause, help) = refusal_of(&synthesis, CLEARED);
    assert_eq!(code, "ESS-SYNTH-001", "{cause}");
    assert_eq!(
        cause,
        "no witness: `cleared.affects[0]` is `demo.g.Child`, whose move starts only in `Parked`, \
         a state reached only by the set move `demo.g.Park/parked`, which synthesis does not take \
         to arrange a row for another scenario: a state only a set move reaches is not arranged, \
         so there are no three rows the filter selects that the arranging commands can produce"
    );
    assert_eq!(help, SET_MOVE_ONLY_HELP);
    assert_eq!(
        ids(&synthesis),
        [
            "demo.g.AddChild/outcome/added",
            "demo.g.AddGroup/outcome/added",
            "demo.g.Clear/outcome/wrong-state",
            "demo.g.Group/state/Closed/refuses/demo.g.Clear",
            "demo.g.Group/transition/close/by/demo.g.Clear/cleared",
            "demo.g.Park/outcome/parked"
        ],
        "the suite is unchanged"
    );
    passes(AFFECTS, &synthesis);
}

/// An entry writing no field is refused where only the state the move arrives in is arranged;
/// here no state the move starts from is, and the refusal names the state a set move alone
/// reaches instead.
#[test]
fn an_affects_entry_writing_nothing_names_the_state_only_a_set_move_reaches() {
    let bare = replaced(AFFECTS, "            sets: {note: input.note}\n", "");
    let (code, cause, help) = refusal_of(&synthesize(&bare), CLEARED);
    assert_eq!(code, "ESS-SYNTH-001", "{cause}");
    assert!(
        cause.contains(
            "`demo.g.Child`, whose move starts only in `Parked`, a state reached only by the set \
             move `demo.g.Park/parked`"
        ),
        "the refusal names the state and the set move reaching it: {cause}"
    );
    assert_eq!(help, SET_MOVE_ONLY_HELP);
}

/// The creator writes the key from a generated value, so no row the filter selects rests in any
/// state: the per-row refusals keep naming the key, with their old help.
#[test]
fn a_changed_row_refusal_whose_filter_is_unmet_in_every_state_keeps_its_text() {
    let generated = replaced(
        CHANGED_ROW,
        "sets: {key: input.key}",
        "sets: {key: {generated: true}}",
    );
    let synthesis = synthesize(&generated);
    for wanted in [DELIVERY, FLOW] {
        let (code, cause, help) = refusal_of(&synthesis, wanted);
        assert_eq!(code, "ESS-SYNTH-010", "{cause}");
        assert_eq!(
            cause,
            "`resume-on-poked` invokes `demo.z.Resume/resumed`, which publishes an event once per \
             `demo.z.Child` it changes, and no row its filter selects can be arranged before the \
             trigger: no arranging command leaves a `demo.z.Child` it selects resting where the \
             event is published for it"
        );
        assert_eq!(
            help,
            "let a command that creates the entity write each field the filter reads from an \
             input; or cover the binding with an authored scenario (ess-scenario/1)"
        );
    }
}

/// The creator writes the group from a generated value, so no row the entry's filter selects rests
/// in any state: the refusal keeps naming the creating branch, with its old help.
#[test]
fn an_affects_entry_refusal_whose_filter_is_unmet_in_every_state_keeps_its_text() {
    let generated = replaced(
        AFFECTS,
        "sets: {group_id: input.group_id, note: input.note}",
        "sets: {group_id: {generated: true}, note: input.note}",
    );
    let (code, cause, help) = refusal_of(&synthesize(&generated), CLEARED);
    assert_eq!(code, "ESS-SYNTH-001", "{cause}");
    assert_eq!(
        cause,
        "no witness: `cleared.affects[0]` is `demo.g.Child` and created only by \
         `demo.g.AddChild/added`, which no arrangement takes for a row the filter selects with the \
         rows the scenario holds when it is sent, so no row is arranged for the entry"
    );
    assert_eq!(
        help,
        "the creating branch named is not taken when sent with the rows the scenario holds, in \
         the states it holds them; narrow the guard that answers before it there, or cover the \
         branch with an authored scenario (ess-scenario/1)"
    );
}
