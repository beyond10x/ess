//! A set move whose rows rest only in a state another set move reaches is refused by name.
//!
//! `Resume/resumed` moves every `Child` under a key from `Parked` to `Held`, and `Parked` is
//! reached only through `Park/parked`, itself a set move (`instances:`). A set move is no driver
//! of an arrangement (`docs/design/set-effects-over-filtered-instances.md`), so no row is arranged
//! in `Parked` and `resumed` is refused as ESS-SYNTH-001. The refusal names the state and the set
//! move that alone reaches it, and its help names the remedy, rather than asking for the filtered
//! field to be written from an input, which the creator already does. A "no three rows" refusal
//! with another cause keeps its text.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// `Parked` is reached only through the set move `Park/parked`.
const SET_MOVE_ONLY: &str = include_str!("fixtures/set-move-only-state.yaml");

const RESUMED: &str = "demo.z.Resume/outcome/resumed";
const PARKED: &str = "demo.z.Park/outcome/parked";
const ADDED: &str = "demo.z.AddChild/outcome/added";

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

#[test]
fn a_state_only_a_set_move_reaches_is_named_in_the_refusal() {
    let synthesis = synthesize(SET_MOVE_ONLY);
    assert_eq!(
        synthesis.refusals.len(),
        1,
        "one refusal occurrence: {:#?}",
        synthesis.refusals
    );
    let (code, cause, help) = refusal_of(&synthesis, RESUMED);
    assert_eq!(code, "ESS-SYNTH-001", "{cause}");
    assert!(
        cause.contains("`Parked`, a state reached only by the set move `demo.z.Park/parked`"),
        "the refusal names the state and the set move reaching it: {cause}"
    );
    assert!(
        cause.contains("a state only a set move reaches is not arranged"),
        "the refusal says such a state is not arranged: {cause}"
    );
    assert!(
        help.contains("single-row transition into the state")
            && help.contains("authored scenario (ess-scenario/1)"),
        "the help names the remedy: {help}"
    );
    assert!(
        !help.contains("rather than from a literal or a generated value"),
        "the help does not ask for the filtered field to be written from an input: {help}"
    );

    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(ids, [ADDED, PARKED], "the suite is unchanged");
    let model = ir(SET_MOVE_ONLY);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    assert_eq!(results.len(), 2, "{results:#?}");
    assert!(
        results.values().all(|status| *status == Status::Passed),
        "every scenario passes on the interpreted target: {results:#?}"
    );
}

/// The creator writes the filtered key from a generated value, so no three rows share one: the
/// refusal of `Park/parked`, whose rows rest in the initial state, keeps its text and its help.
#[test]
fn a_no_three_rows_refusal_with_another_cause_keeps_its_text() {
    let generated = replaced(
        SET_MOVE_ONLY,
        "sets: {key: input.key}",
        "sets: {key: {generated: true}}",
    );
    let (code, cause, help) = refusal_of(&synthesize(&generated), PARKED);
    assert_eq!(code, "ESS-SYNTH-001", "{cause}");
    assert_eq!(
        cause,
        "no witness: `parked.instances` is `demo.z.Child`, which has no three rows the filter \
         selects that the arranging commands can produce"
    );
    assert_eq!(
        help,
        "let a command that creates the entity write each field the filter reads from an input or \
         from a caller attribute (`{caller: <attribute>}`) rather than from a literal or a \
         generated value, so the arranged rows can hold values on both sides of the filter"
    );
}
