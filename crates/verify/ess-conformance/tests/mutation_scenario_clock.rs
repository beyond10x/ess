//! The mutation audit runs its synthesized scenarios on the scenario clock
//! (story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock).
//!
//! `ess verify conform run --target interpreted` hands the interpreter the runner's command clock,
//! so a `now` guard is decided at the instant of the step being executed. The audit's authored half
//! already ran that way; its synthesized half ran unclocked, so every scenario that reaches a `now`
//! guard came out `unsupported`, was not scored, and a guard mutant on that guard survived although
//! nothing that ran tested it.
#![allow(clippy::missing_panics_doc)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_conformance::reference::Untraced;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

/// `Present` is `expired` while `exp < now - 10m`, else `accepted`.
const TOKENS: &str = include_str!("fixtures/now-guard-interpreted.yaml");

fn documents(text: &str, label: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn audit(classes: &[MutantClass]) -> MutationReport {
    let (documents, texts) = documents(TOKENS, "now-guard-interpreted.yaml");
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    mutate::audit(&documents, &texts, classes, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// Every synthesized scenario `run --target interpreted` passes executes in the audit's baseline:
/// none is reported `unsupported` and left unscored.
#[test]
fn the_baseline_executes_every_scenario_on_a_now_guarded_model() {
    let report = audit(MutantClass::ALL);
    assert!(
        report.baseline.not_scored.is_empty(),
        "story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock: {}",
        report.render_text()
    );
    assert!(report.baseline.scenarios >= 2, "{}", report.render_text());
}

/// The guard-boundary mutant on `exp < now - 10m` is killed by the scenario that pins the guard,
/// and no mutant on the guard is left `survived` by a suite that never decided it.
#[test]
fn a_guard_mutant_on_a_now_guard_is_killed() {
    let report = audit(&[MutantClass::GuardBoundary, MutantClass::GuardNegate]);
    let on_guard: Vec<_> = report
        .mutants
        .iter()
        .filter(|entry| entry.id.contains("demo.tok.Present/expired"))
        .collect();
    assert!(!on_guard.is_empty(), "{}", report.render_text());
    for entry in on_guard {
        assert_eq!(
            entry.verdict,
            Verdict::Killed,
            "story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock: {}",
            report.render_text()
        );
    }
    assert_eq!(report.counts.survived, 0, "{}", report.render_text());
}

/// On a model with no `now` guard the clock changes nothing: the interpreter handed it writes the
/// report the same interpreter writes unclocked, byte for byte. [`Untraced`] forwards every method
/// but the ones it does not name, so it takes no clock.
#[test]
fn a_model_without_a_now_guard_keeps_its_report_bytes() {
    for (label, text) in [
        (
            "mutation-authored.yaml",
            include_str!("fixtures/mutation-authored.yaml"),
        ),
        (
            "mutation-stored-guards.yaml",
            include_str!("fixtures/mutation-stored-guards.yaml"),
        ),
    ] {
        let (documents, texts) = documents(text, label);
        let ir =
            mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
        let clocked = mutate::audit(&documents, &texts, MutantClass::ALL, || {
            Interpreted::for_model(ir.clone())
        })
        .unwrap_or_else(|refusal| panic!("{label}: {refusal}"));
        let unclocked = mutate::audit(&documents, &texts, MutantClass::ALL, || {
            Untraced(Interpreted::for_model(ir.clone()))
        })
        .unwrap_or_else(|refusal| panic!("{label}: {refusal}"));
        assert_eq!(
            clocked.to_canonical_json(),
            unclocked.to_canonical_json(),
            "{label}"
        );
    }
}
