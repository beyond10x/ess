//! A stored-field guard beside a lifecycle-state guard in one command stays refused
//! (`conflicting_declaration`, ESS-COMMAND-004), and the refusal names the idiom that states the
//! same command: the held state read as `state` in a `when_subject` predicate (beyond10x/ess#461).
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationCode;

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/subject-fact-complete-refusal.yaml"
);

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the model");
    out
}

/// The command as first written: the stored-field refusal, an update guarded by
/// `when_subject_state: Active`, and a plain default refusal for every other state.
fn mixed(format: &str) -> String {
    let text = replaced(
        MODEL,
        "      - name: not-active\n        when_subject:\n          predicate: state != Active\n        error: demo.inst.InstanceNotActive\n",
        "",
    );
    let text = replaced(
        &text,
        "      - name: updated\n        updates: demo.inst.Instance\n",
        "      - name: updated\n        when_subject_state: Active\n        updates: demo.inst.Instance\n",
    );
    let text = replaced(
        &text,
        "        payload: {demo.inst.InstanceUpdated: {name: input.name}}\n",
        "        payload: {demo.inst.InstanceUpdated: {name: input.name}}\n      - {name: not-active, error: demo.inst.InstanceNotActive}\n",
    );
    let text = text.replace("format: ess/23\n", &format!("format: {format}\n"));
    assert!(text.starts_with(&format!("format: {format}\n")), "{format}");
    text
}

#[test]
fn lifecycle_mix_is_refused_with_the_state_predicate_hint() {
    for format in ["ess/23", "ess/22", "ess/18"] {
        let text = mixed(format);
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        let errors = Specification::assemble([(Source::new("instance.yaml"), raw)])
            .expect_err("the two strategies stay apart");
        let mix: Vec<_> = errors
            .as_slice()
            .iter()
            .filter(|error| {
                error.message
                    == "subject fact and lifecycle guards cannot be combined in one command"
            })
            .collect();
        assert!(!mix.is_empty(), "{format}: {errors}");
        for error in mix {
            assert_eq!(
                error.code,
                ValidationCode::ConflictingDeclaration,
                "{error:?}"
            );
            assert_eq!(
                error.location, "command.demo.inst.UpdateInstance.outcomes",
                "{error:?}"
            );
            let hint = error.hint.as_deref().unwrap_or_default();
            for phrase in ["`when_subject: {predicate: state", "`when_subject_state:`"] {
                assert!(
                    hint.contains(phrase),
                    "{format}: the hint names {phrase}: {error:?}"
                );
            }
        }
    }
}

#[test]
fn the_state_predicate_idiom_validates() {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    Specification::assemble([(Source::new("instance.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the idiom the hint names is admitted: {errors}"));
}
