//! The mutation audit with authored scenarios (<https://github.com/beyond10x/ess/issues/515>):
//! each `ess-scenario/1` scenario, compiled once against the unchanged specification, runs against
//! the interpreter of that specification for the baseline and against the interpreter of each
//! mutant, beside the synthesized suite. A mutant only an authored scenario pins is killed by it.
//!
//! The fixture's `at-limit` branch refuses once a counter reaches 18, beyond the 16 steps
//! synthesis follows a counter, so the baseline refuses its scenario (`ESS-SYNTH-003`) and the
//! mutants on that branch that its synthesized suite cannot see are unwitnessed. The authored
//! scenario raises the counter eighteen times and requires the refusal on the nineteenth.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, AuditRefusal, Document, MutantClass, MutationReport, Verdict};
use ess_conformance::{ConformanceScenario, ScenarioId};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/mutation-authored.yaml");
const AUTHORED: &str = include_str!("fixtures/mutation-authored/retries-reach-the-limit.yaml");
const AUTHORED_ID: &str = "work.tasks/authored/retries-reach-the-limit";

const ERROR_SWAP: &str = "error-swap/work.tasks.RetryTask/at-limit";
const STRICTER: &str = "guard-boundary/work.tasks.RetryTask/at-limit/when_subject/0";

const CLASSES: &[MutantClass] = &[
    MutantClass::ErrorSwap,
    MutantClass::GuardBoundary,
    MutantClass::GuardNegate,
];

fn documents() -> (Vec<Document>, SourceMap) {
    let label = "mutation-authored.yaml".to_owned();
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), MODEL.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn model(documents: &[Document], texts: &SourceMap) -> EssIr {
    mutate::compile(documents.to_vec(), texts).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `text` compiled as `run --scenarios` compiles it, every file accepted.
fn authored(ir: &EssIr, text: &str) -> BTreeMap<ScenarioId, ConformanceScenario> {
    let authoring = ess_conformance::authored::compile(
        ir,
        &[ess_conformance::authored::Source::new(
            "retries-reach-the-limit.yaml",
            text,
        )],
    );
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    authoring.scenarios
}

fn audit(
    authored: &BTreeMap<ScenarioId, ConformanceScenario>,
) -> Result<MutationReport, AuditRefusal> {
    let (documents, texts) = documents();
    let ir = model(&documents, &texts);
    mutate::audit_with_authored(
        &documents,
        &texts,
        CLASSES,
        || Interpreted::for_model(ir.clone()),
        None,
        authored,
    )
}

fn entry<'a>(report: &'a MutationReport, id: &str) -> &'a mutate::MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("{id}: {}", report.render_text()))
}

/// Without an authored scenario, the two mutants the limit hides from synthesis are unwitnessed:
/// the baseline refuses the outcome's scenario, and the mutant's suite cannot see the change.
#[test]
fn without_authored_scenarios_the_limit_mutants_are_unwitnessed() {
    let report = audit(&BTreeMap::new()).unwrap_or_else(|refusal| panic!("{refusal}"));
    for id in [ERROR_SWAP, STRICTER] {
        let entry = entry(&report, id);
        assert_eq!(entry.verdict, Verdict::Unwitnessed, "{entry:#?}");
        assert!(entry.baseline_refusals.is_some(), "{entry:#?}");
    }
    assert_eq!(report.baseline.scenarios, 2, "{}", report.render_text());
}

/// With the authored scenario, the interpreter of each mutant answers it otherwise than the
/// specification does — `Blocked` for `Exhausted`, a nineteenth raise accepted — so it fails, and
/// it is the mutant's killer, named as a synthesized killer is.
#[test]
fn the_authored_scenario_kills_the_mutants_only_it_pins() {
    let (documents, texts) = documents();
    let ir = model(&documents, &texts);
    let report = audit(&authored(&ir, AUTHORED)).unwrap_or_else(|refusal| panic!("{refusal}"));
    for id in [ERROR_SWAP, STRICTER] {
        let entry = entry(&report, id);
        assert_eq!(entry.verdict, Verdict::Killed, "{entry:#?}");
        assert_eq!(
            entry.killers.as_deref(),
            Some([AUTHORED_ID.to_owned()].as_slice()),
            "{entry:#?}"
        );
        // The outcome is witnessed by the authored scenario, so the baseline's refusal of its
        // generated scenario no longer stands in for a witness.
        assert_eq!(entry.baseline_refusals, None, "{entry:#?}");
    }
    assert_eq!(report.counts.unwitnessed, 0, "{}", report.render_text());
    assert_eq!(report.baseline.scenarios, 3, "{}", report.render_text());
    let text = report.render_text();
    assert!(
        text.lines().any(|line| line
            == format!(
                "killed {ERROR_SWAP}: `error: work.tasks.Exhausted` becomes `error: \
                 work.tasks.Blocked` — by {AUTHORED_ID} (1 in total)"
            )),
        "{text}"
    );
}

/// An authored scenario the specification's own interpreter fails is a failed baseline scenario:
/// the audit refuses with `ESS-MUTATE-001`, naming it, as it refuses a synthesized one.
#[test]
fn an_authored_scenario_the_baseline_fails_refuses_the_audit() {
    let (documents, texts) = documents();
    let ir = model(&documents, &texts);
    // The eighteenth raise is claimed to be refused already: the specification accepts it.
    let wrong = AUTHORED.replacen(
        "09:00:18Z, command: work.tasks.RetryTask, input: {task_id: {$instance: task}}, outcome: \
         retried}",
        "09:00:18Z, command: work.tasks.RetryTask, input: {task_id: {$instance: task}}, outcome: \
         at-limit}",
        1,
    );
    assert_ne!(wrong, AUTHORED, "the edit applies");
    match audit(&authored(&ir, &wrong)) {
        Err(AuditRefusal::BaselineFailed { not_passed, .. }) => {
            assert_eq!(not_passed, [AUTHORED_ID], "{not_passed:?}");
        }
        other => panic!("expected ESS-MUTATE-001: {other:?}"),
    }
}

/// No authored scenario is the audit as it was: the same report, byte for byte.
#[test]
fn an_empty_authored_set_keeps_the_report_bytes() {
    let (documents, texts) = documents();
    let ir = model(&documents, &texts);
    let before = mutate::audit(&documents, &texts, CLASSES, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let after = audit(&BTreeMap::new()).unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(before.to_canonical_json(), after.to_canonical_json());
    assert_eq!(before.render_text(), after.render_text());
}
