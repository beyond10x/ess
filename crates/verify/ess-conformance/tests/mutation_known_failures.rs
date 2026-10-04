//! The mutation audit with explicitly declared known baseline failures (beyond10x/ess#294).
//!
//! `docs/design/mutation-scope-and-known-failures.md`, "Mutation scoring and persistence". The
//! target is the billing reference with one planted defect (`Fault::ExtraEvent`: cancelling an
//! invoice also announces it paid), which fails the same scenarios in the baseline and in every
//! mutant's suite. Without a declaration that baseline is refused with `ESS-MUTATE-001`, as it
//! always was. With an `ess-known-failures/1` declaration of exactly those scenarios, bound to the
//! suite bytes, the specification, the implementation label and the public build, the audit scores
//! every mutant on the baseline-passing scenarios only: a declared scenario, or one the baseline's
//! suite does not hold, never kills.
//!
//! Every report scored here comes from an actual run of a real target. Where a case needs exact
//! categories no target produces on demand, an actual report is rewritten and read back through
//! `CountReport`, as `tests/mutation_skipped_baseline.rs` does.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault, Faulty};
use ess_conformance::known_failures::{self, ExecutionContext, RefusalKind, EXECUTION_FILE};
use ess_conformance::mutate::{
    self, AuditRefusal, BoundDeclaration, Document, Emission, Exclusion, ExclusionReason,
    KnownFailing, Manifest, MutantClass, MutationReport, Verdict, BASELINE_DIR,
    KNOWN_FAILURES_FILE, MANIFEST_FILE, MANIFEST_FORMAT, MANIFEST_FORMAT_4, REPORT_FILE,
    REPORT_FORMAT_4, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::{json, Value};

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut found = Vec::new();
    let mut pending = vec![base];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).expect("well formed"),
        ));
        texts.insert(label, text);
    }
    (parsed, texts)
}

const CLASSES: &[MutantClass] = &[
    MutantClass::ErrorSwap,
    MutantClass::FromDrop,
    MutantClass::TransitionTo,
    MutantClass::GuardNegate,
];

fn build(digit: char) -> String {
    format!("sha256:{}", digit.to_string().repeat(64))
}

fn defective() -> Faulty<Billing> {
    faulty::billing(Fault::ExtraEvent)
}

/// The report/2 the Rust runner writes for `target` over the suite text `suite`.
fn run(suite: &str, target: &impl ConformanceTarget) -> String {
    let admitted = AdmittedSuite::from_json(suite).expect("an emitted suite is admitted");
    let executed = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    CountReport::from_run(&executed, &admitted)
        .expect("a complete run")
        .to_canonical_json()
        .expect("serializes")
}

fn statuses(report: &str, suite: &str) -> BTreeMap<String, &'static str> {
    CountReport::from_json(report, &AdmittedSuite::from_json(suite).unwrap())
        .expect("report/2")
        .statuses()
}

fn with_status(report: &str, suite: &str, status: &str) -> Vec<String> {
    statuses(report, suite)
        .into_iter()
        .filter(|(_, it)| *it == status)
        .map(|(id, _)| id)
        .collect()
}

/// The baseline suite every audit of `classes` runs, its defective report, and that report's label.
struct Baseline {
    suite: String,
    report: String,
    implementation: String,
}

fn baseline(classes: &[MutantClass]) -> Baseline {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, classes).expect("emits");
    let suite = emission.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")].clone();
    let report = run(&suite, &defective());
    let implementation =
        CountReport::from_json(&report, &AdmittedSuite::from_json(&suite).unwrap())
            .unwrap()
            .implementation()
            .to_owned();
    Baseline {
        suite,
        report,
        implementation,
    }
}

fn declaration(baseline: &Baseline, implementation: &str, scenarios: &[String]) -> String {
    let admitted = AdmittedSuite::from_json(&baseline.suite).unwrap();
    let mut scenarios = scenarios.to_vec();
    scenarios.sort();
    let value = json!({
        "format": "ess-known-failures/1",
        "spec_digest": admitted.suite().provenance.spec_digest.to_string(),
        "suite_digest": admitted.digest(),
        "implementation": implementation,
        "implementation_build": build('a'),
        "failures": scenarios.iter().map(|id| json!({
            "scenario": id,
            "reason": "cancelling also announces the invoice paid",
            "tracking": "ORDERS-412",
        })).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&value).unwrap() + "\n"
}

fn audit(
    classes: &[MutantClass],
    target: impl Fn() -> Faulty<Billing>,
    known: Option<KnownFailing<'_>>,
) -> Result<MutationReport, AuditRefusal> {
    let (files, texts) = example("billing");
    mutate::audit_with(&files, &texts, classes, target, known)
}

fn bound<'a>(declaration: &'a str, build: &'a str) -> KnownFailing<'a> {
    KnownFailing { declaration, build }
}

fn known_failures(refusal: AuditRefusal) -> known_failures::Refusal {
    match refusal {
        AuditRefusal::KnownFailures(refusal) => refusal,
        other => panic!("a known-failure refusal, not {other}"),
    }
}

/// An emission of `classes` binding `declaration`, with the defective target's actual report and a
/// host execution context of build `a` beside every suite.
fn collected_emission(
    classes: &[MutantClass],
    declaration: Option<&str>,
) -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = example("billing");
    let emission = mutate::emit_with(&files, &texts, classes, None, declaration).expect("emits");
    let mut written = emission.files.clone();
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|it| it.dir.clone()),
    );
    for dir in dirs {
        let suite = emission.files[&format!("{dir}/{SUITE_FILE}")].clone();
        let report = run(&suite, &defective());
        put_report(&mut written, &dir, &report, &build('a'));
    }
    (emission, written)
}

/// Writes `report` beside the suite in `dir`, with the execution context of build `build`.
fn put_report(written: &mut BTreeMap<String, String>, dir: &str, report: &str, build: &str) {
    let admitted = AdmittedSuite::from_json(&written[&format!("{dir}/{SUITE_FILE}")]).unwrap();
    let implementation = CountReport::from_json(report, &admitted)
        .expect("a report of this suite")
        .implementation()
        .to_owned();
    let context = ExecutionContext::new(report, &admitted, &implementation, build).unwrap();
    written.insert(format!("{dir}/{REPORT_FILE}"), report.to_owned());
    written.insert(
        format!("{dir}/{EXECUTION_FILE}"),
        context.to_canonical_json(),
    );
}

fn collect(
    written: &BTreeMap<String, String>,
    supplied: Option<&str>,
) -> Result<MutationReport, AuditRefusal> {
    mutate::collect_with(|path| written.get(path).cloned(), None, supplied)
}

/// An actual report of the suite in `suite`, rewritten so that exactly `failed` failed and every
/// other scenario passed, under the generated runners' profile; read back through `CountReport`.
fn exactly_failing(report: &str, suite: &str, failed: &BTreeSet<String>) -> String {
    let admitted = AdmittedSuite::from_json(suite).unwrap();
    let mut value: Value = serde_json::from_str(report).unwrap();
    let ids: Vec<String> = admitted
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let (red, green): (Vec<String>, Vec<String>) =
        ids.into_iter().partition(|id| failed.contains(id));
    assert_eq!(red.len(), failed.len(), "every failure names the suite");
    value["producer_profile"] = "go-scenario-status/2".into();
    value["counts"] = json!({"total": admitted.suite().len(), "passed": green.len(),
        "failed": red.len(), "error": 0, "unsupported": 0, "skipped": 0});
    value["outcomes"] =
        json!({"passed": green, "failed": red, "error": [], "unsupported": [], "skipped": []});
    let status = if red.is_empty() { "passed" } else { "failed" };
    value["execution_status"] = status.into();
    value["conformance_status"] = if red.is_empty() {
        "inconclusive"
    } else {
        "failed"
    }
    .into();
    let text = serde_json::to_string_pretty(&value).unwrap() + "\n";
    CountReport::from_json(&text, &admitted).expect("a coherent rewritten report/2");
    text
}

fn ids(suite: &str) -> BTreeSet<String> {
    AdmittedSuite::from_json(suite)
        .unwrap()
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn body(suite: &str, id: &str) -> Value {
    let value: Value = serde_json::from_str(suite).unwrap();
    value["scenarios"][id].clone()
}

// ---- the direct audit ----------------------------------------------------------------------------

#[test]
fn without_a_declaration_a_failing_baseline_is_still_refused_with_mutate_001() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    assert!(!failed.is_empty(), "the planted defect fails the baseline");
    let refusal = audit(CLASSES, defective, None).expect_err("a red baseline is refused");
    let AuditRefusal::BaselineFailed { not_passed, .. } = &refusal else {
        panic!("ESS-MUTATE-001, not {refusal}");
    };
    let mut not_passed = not_passed.clone();
    not_passed.sort();
    assert_eq!(not_passed, failed);
}

#[test]
fn a_declared_failure_is_excluded_and_independent_eligible_scenarios_still_kill() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let report = audit(CLASSES, defective, Some(bound(&declaration, &build('a'))))
        .unwrap_or_else(|refusal| panic!("scored, not {refusal}"));
    assert_eq!(report.format, REPORT_FORMAT_4);
    let summary = report
        .known_failures
        .as_ref()
        .expect("the declaration is named");
    assert_eq!(
        summary.declaration_digest,
        known_failures::sha256(declaration.as_bytes())
    );
    assert_eq!(summary.implementation_build, build('a'));
    assert_eq!(
        summary
            .failures
            .iter()
            .map(|it| it.scenario.clone())
            .collect::<Vec<_>>(),
        failed
    );
    assert!(report.counts.killed > 0, "{}", report.render_text());
    // A declared failure is excluded by the declaration, never listed as not executed.
    assert_eq!(
        report.baseline.not_scored.len(),
        0,
        "{:?}",
        report.baseline.not_scored
    );
    let declared: BTreeSet<&String> = failed.iter().collect();
    for entry in &report.mutants {
        assert_eq!(entry.excluded, None, "{}", entry.id);
        for killer in entry.killers.iter().flatten() {
            assert!(
                !declared.contains(killer),
                "{} killed by declared {killer}",
                entry.id
            );
        }
        if entry.scenarios.is_some() {
            let exclusions = entry.exclusions.as_deref().unwrap_or_default();
            for id in &failed {
                // Every mutant here keeps the declared scenarios' IDs.
                assert!(
                    exclusions.contains(&Exclusion {
                        reason: ExclusionReason::KnownFailed,
                        scenario: id.clone(),
                    }),
                    "{}: {exclusions:?}",
                    entry.id
                );
            }
        }
    }
    let text = report.render_text();
    for id in &failed {
        assert!(
            text.lines()
                .take(1 + failed.len())
                .any(|line| line.starts_with(&format!("known failure {id}"))),
            "known failures lead the text:\n{text}"
        );
    }
    assert!(text.contains("ORDERS-412"), "{text}");
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(
        json["known_failures"]["failures"][0]["tracking"],
        "ORDERS-412"
    );
}

#[test]
fn an_unlisted_failure_still_refuses_with_mutate_001_naming_only_it() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    assert!(failed.len() >= 2, "{failed:?}");
    let declaration = declaration(&baseline, &baseline.implementation, &failed[1..]);
    let refusal = audit(CLASSES, defective, Some(bound(&declaration, &build('a'))))
        .expect_err("an unlisted failure is refused");
    let AuditRefusal::BaselineFailed { not_passed, .. } = &refusal else {
        panic!("ESS-MUTATE-001, not {refusal}");
    };
    assert_eq!(not_passed, &[failed[0].clone()]);
}

#[test]
fn a_stale_declaration_is_refused_before_any_mutant_is_scored() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let passing = with_status(&baseline.report, &baseline.suite, "passed");
    let mut listed = failed.clone();
    listed.push(passing[0].clone());
    let stale = declaration(&baseline, &baseline.implementation, &listed);
    let refusal = known_failures(
        audit(CLASSES, defective, Some(bound(&stale, &build('a')))).expect_err("stale"),
    );
    assert_eq!(refusal.kind, RefusalKind::Stale, "{refusal}");
    // The defect repaired: every declared scenario passes, so the whole declaration is stale.
    let repaired = run(&baseline.suite, &Billing::new());
    let label = CountReport::from_json(
        &repaired,
        &AdmittedSuite::from_json(&baseline.suite).unwrap(),
    )
    .unwrap()
    .implementation()
    .to_owned();
    let declaration = declaration(&baseline, &label, &failed);
    let (files, texts) = example("billing");
    let refusal = known_failures(
        mutate::audit_with(
            &files,
            &texts,
            CLASSES,
            Billing::new,
            Some(bound(&declaration, &build('a'))),
        )
        .expect_err("stale after repair"),
    );
    assert_eq!(refusal.kind, RefusalKind::Stale, "{refusal}");
}

#[test]
fn a_declaration_of_another_build_suite_specification_or_scenario_refuses_before_any_target_runs() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let good: Value =
        serde_json::from_str(&declaration(&baseline, &baseline.implementation, &failed)).unwrap();
    let text = |value: &Value| serde_json::to_string_pretty(value).unwrap() + "\n";
    let mut cases: Vec<(&str, String, String, RefusalKind)> = vec![(
        "another build",
        text(&good),
        build('b'),
        RefusalKind::Identity,
    )];
    let mut suite = good.clone();
    suite["suite_digest"] = json!(build('c'));
    cases.push((
        "other suite bytes",
        text(&suite),
        build('a'),
        RefusalKind::Identity,
    ));
    let mut spec = good.clone();
    spec["spec_digest"] = json!("c".repeat(64));
    cases.push((
        "another specification",
        text(&spec),
        build('a'),
        RefusalKind::Identity,
    ));
    let mut unknown = good.clone();
    unknown["failures"][0]["scenario"] = json!("billing.invoice.CancelInvoice/outcome/vanished");
    cases.push((
        "an unknown scenario",
        text(&unknown),
        build('a'),
        RefusalKind::UnknownScenario,
    ));
    cases.push((
        "not a declaration",
        "{}".to_owned(),
        build('a'),
        RefusalKind::Malformed,
    ));
    for (case, declaration, build, kind) in cases {
        let made = Cell::new(0_usize);
        let refusal = known_failures(
            audit(
                CLASSES,
                || {
                    made.set(made.get() + 1);
                    defective()
                },
                Some(bound(&declaration, &build)),
            )
            .expect_err(case),
        );
        assert_eq!(refusal.kind, kind, "{case}: {refusal}");
        assert_eq!(made.get(), 0, "{case}: refused before any target was made");
    }
    // The label is the run's own, so it is checked once the baseline named itself.
    let other = declaration(&baseline, "billing-reference 9.9", &failed);
    let refusal = known_failures(
        audit(CLASSES, defective, Some(bound(&other, &build('a'))))
            .expect_err("another implementation"),
    );
    assert_eq!(refusal.kind, RefusalKind::Identity, "{refusal}");
}

// ---- emit and collect ----------------------------------------------------------------------------

#[test]
fn emit_and_collect_under_the_same_declaration_is_the_direct_audit() {
    let baseline = baseline(MutantClass::ALL);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let (emission, written) = collected_emission(MutantClass::ALL, Some(&declaration));
    assert_eq!(emission.manifest.format, MANIFEST_FORMAT_4);
    assert_eq!(
        emission.manifest.known_failures,
        Some(BoundDeclaration {
            digest: known_failures::sha256(declaration.as_bytes()),
            file: KNOWN_FAILURES_FILE.to_owned(),
            implementation: baseline.implementation.clone(),
            implementation_build: build('a'),
        })
    );
    assert_eq!(
        emission.files[KNOWN_FAILURES_FILE], declaration,
        "copied byte for byte"
    );
    assert!(emission.manifest.baseline.suite_digest.is_some());
    for mutant in emission
        .manifest
        .mutants
        .iter()
        .filter(|it| it.dir.is_some())
    {
        assert!(mutant.suite_digest.is_some(), "{}", mutant.id);
    }
    let mut collected =
        collect(&written, None).unwrap_or_else(|refusal| panic!("collects, not {refusal}"));
    let audited = audit(
        MutantClass::ALL,
        defective,
        Some(bound(&declaration, &build('a'))),
    )
    .unwrap();
    assert_eq!(collected.implementation, baseline.implementation);
    collected.implementation = audited.implementation.clone();
    assert_eq!(collected.to_canonical_json(), audited.to_canonical_json());
    let again = collect(&written, Some(&declaration)).expect("the bound bytes, supplied again");
    assert_eq!(again.counts, audited.counts);
}

#[test]
fn collect_refuses_a_late_changed_or_tampered_declaration() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    // Late: an emission that bound none cannot be scored under one supplied afterwards.
    let (unbound, written) = collected_emission(CLASSES, None);
    assert_eq!(unbound.manifest.format, MANIFEST_FORMAT);
    let refusal = known_failures(collect(&written, Some(&declaration)).expect_err("late"));
    assert_eq!(refusal.kind, RefusalKind::Unbound, "{refusal}");
    assert!(
        refusal.to_string().contains(MANIFEST_FORMAT_4),
        "re-emission is the migration: {refusal}"
    );

    let (_, written) = collected_emission(CLASSES, Some(&declaration));
    collect(&written, None).expect("the bound declaration collects");
    // Changed: the same entries in other bytes is another declaration.
    let changed = declaration.replace('\n', "\r\n");
    let refusal = known_failures(collect(&written, Some(&changed)).expect_err("changed"));
    assert_eq!(refusal.kind, RefusalKind::Unbound, "{refusal}");
    // Tampered in the emission after it was written.
    let mut tampered = written.clone();
    tampered.insert(
        KNOWN_FAILURES_FILE.to_owned(),
        declaration.replace("ORDERS-412", "ORDERS-413"),
    );
    let refusal = known_failures(collect(&tampered, None).expect_err("tampered"));
    assert_eq!(refusal.kind, RefusalKind::Unbound, "{refusal}");
    // The manifest's build edited to another one.
    let mut manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["known_failures"]["implementation_build"] = json!(build('b'));
    let mut edited = written.clone();
    edited.insert(MANIFEST_FILE.to_owned(), manifest.to_string());
    assert!(
        collect(&edited, None).is_err(),
        "the manifest's build is the declaration's"
    );
}

#[test]
fn every_collected_report_needs_its_execution_context_of_the_bound_build() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let (emission, written) = collected_emission(CLASSES, Some(&declaration));
    let full = collect(&written, None).unwrap();
    let context = format!("{BASELINE_DIR}/{EXECUTION_FILE}");

    let mut missing = written.clone();
    missing.remove(&context);
    let refusal = collect(&missing, None).expect_err("no baseline context");
    assert!(refusal.to_string().contains(EXECUTION_FILE), "{refusal}");

    let mut rebuilt = written.clone();
    let report = written[&format!("{BASELINE_DIR}/{REPORT_FILE}")].clone();
    put_report(&mut rebuilt, BASELINE_DIR, &report, &build('b'));
    let refusal = known_failures(collect(&rebuilt, None).expect_err("another build"));
    assert_eq!(refusal.kind, RefusalKind::Identity, "{refusal}");

    let killed = full
        .mutants
        .iter()
        .find(|it| it.verdict == Verdict::Killed)
        .expect("a killed mutant")
        .id
        .clone();
    for case in ["missing", "another build"] {
        let mut changed = written.clone();
        if case == "missing" {
            changed.remove(&format!("{killed}/{EXECUTION_FILE}"));
        } else {
            let report = written[&format!("{killed}/{REPORT_FILE}")].clone();
            put_report(&mut changed, &killed, &report, &build('b'));
        }
        let report = collect(&changed, None).expect("still collects");
        let entry = report.mutants.iter().find(|it| it.id == killed).unwrap();
        assert_eq!(entry.verdict, Verdict::Inconclusive, "{case}");
        assert_eq!(entry.killers, None, "{case}");
        assert!(
            entry
                .unscored
                .as_deref()
                .unwrap_or_default()
                .contains(EXECUTION_FILE),
            "{case}: {entry:?}"
        );
    }

    // A version-4 emission collects report/2 only: report/1 has no exact suite binding (and cannot
    // name these suites' major, so the document here is report/1's envelope alone).
    let _ = &emission;
    let mut legacy = written.clone();
    legacy.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        json!({"format": "ess-conformance-report/1"}).to_string(),
    );
    let refusal = collect(&legacy, None).expect_err("report/1 under manifest/4");
    assert!(
        refusal.to_string().contains("ess-conformance-report/2"),
        "{refusal}"
    );
}

#[test]
fn a_report_of_other_suite_bodies_with_the_same_ids_and_counts_is_not_scored() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let (_, written) = collected_emission(CLASSES, Some(&declaration));
    let full = collect(&written, None).unwrap();
    let killed = full
        .mutants
        .iter()
        .find(|it| it.verdict == Verdict::Killed)
        .expect("a killed mutant")
        .id
        .clone();
    let path = format!("{killed}/{SUITE_FILE}");
    let mut suite: Value = serde_json::from_str(&written[&path]).unwrap();
    let first = suite["scenarios"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    suite["scenarios"][&first]["purpose"] = json!("A stale copy of this scenario");
    let stale = serde_json::to_string_pretty(&suite).unwrap() + "\n";
    let admitted = AdmittedSuite::from_json(&stale).expect("still a suite");
    let original = AdmittedSuite::from_json(&written[&path]).unwrap();
    assert_eq!(
        admitted.suite().provenance.spec_digest,
        original.suite().provenance.spec_digest
    );
    assert_eq!(ids(&stale), ids(&written[&path]));
    let mut changed = written.clone();
    changed.insert(path, stale.clone());
    let report = run(&stale, &defective());
    put_report(&mut changed, &killed, &report, &build('a'));
    let collected = collect(&changed, None).expect("collects");
    let entry = collected.mutants.iter().find(|it| it.id == killed).unwrap();
    assert_eq!(entry.verdict, Verdict::Inconclusive, "{entry:?}");
    assert!(
        entry
            .unscored
            .as_deref()
            .unwrap_or_default()
            .contains("suite"),
        "{entry:?}"
    );
}

// ---- eligibility --------------------------------------------------------------------------------

/// The declared emission of every class, with `rewrite` choosing, per mutant, the scenarios its
/// actual report is rewritten to fail (`None` keeps the actual report).
fn rescored(
    rewrite: impl Fn(&str, &str, &str) -> Option<BTreeSet<String>>,
    declared: bool,
) -> (MutationReport, Baseline, Vec<String>) {
    let baseline = baseline(MutantClass::ALL);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let (emission, mut written) =
        collected_emission(MutantClass::ALL, declared.then_some(declaration.as_str()));
    if !declared {
        // The control without a declaration: the baseline rewritten green, so it is scored at all.
        let green = exactly_failing(&baseline.report, &baseline.suite, &BTreeSet::new());
        put_report(&mut written, BASELINE_DIR, &green, &build('a'));
    }
    for mutant in &emission.manifest.mutants {
        let Some(dir) = &mutant.dir else { continue };
        let suite = written[&format!("{dir}/{SUITE_FILE}")].clone();
        let report = written[&format!("{dir}/{REPORT_FILE}")].clone();
        if let Some(failing) = rewrite(&mutant.id, &suite, &report) {
            let text = exactly_failing(&report, &suite, &failing);
            put_report(&mut written, dir, &text, &build('a'));
        }
    }
    (collect(&written, None).expect("collects"), baseline, failed)
}

#[test]
fn a_scenario_new_to_the_mutant_has_no_baseline_control_and_cannot_kill_alone() {
    let baseline_ids = ids(&baseline(MutantClass::ALL).suite);
    let only_new = |_: &str, suite: &str, _: &str| -> Option<BTreeSet<String>> {
        let new: BTreeSet<String> = ids(suite).difference(&baseline_ids).cloned().collect();
        (!new.is_empty()).then_some(new)
    };
    let (declared, _, _) = rescored(only_new, true);
    let (control, _, _) = rescored(only_new, false);
    let mut seen = 0;
    for entry in &declared.mutants {
        let Some(exclusions) = &entry.exclusions else {
            continue;
        };
        let new: Vec<&Exclusion> = exclusions
            .iter()
            .filter(|it| it.reason == ExclusionReason::NoBaselineControl)
            .collect();
        // A gained or baseline refusal makes it unwitnessed whatever else holds.
        if new.is_empty() || entry.added_refusals.is_some() || entry.baseline_refusals.is_some() {
            continue;
        }
        seen += 1;
        assert_eq!(entry.verdict, Verdict::Inconclusive, "{entry:?}");
        assert_eq!(entry.killers, None);
        // The planted control: scored without the rule, the same reports kill it.
        let before = control.mutants.iter().find(|it| it.id == entry.id).unwrap();
        assert_eq!(before.verdict, Verdict::Killed, "{before:?}");
    }
    assert!(
        seen > 0,
        "some billing mutant adds a scenario the baseline does not hold"
    );
}

#[test]
fn a_declared_scenario_kills_nothing_changed_or_not() {
    let baseline = baseline(MutantClass::ALL);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let target = failed[0].clone();
    let only_declared = |_: &str, suite: &str, _: &str| -> Option<BTreeSet<String>> {
        ids(suite)
            .contains(&target)
            .then(|| BTreeSet::from([target.clone()]))
    };
    let (report, _, _) = rescored(only_declared, true);
    let (mut changed, mut unchanged) = (0, 0);
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, MutantClass::ALL).unwrap();
    for entry in report.mutants.iter().filter(|it| it.scenarios.is_some()) {
        let mutant = emission
            .manifest
            .mutants
            .iter()
            .find(|it| it.id == entry.id)
            .unwrap();
        let suite = &emission.files[&format!("{}/{SUITE_FILE}", mutant.dir.as_ref().unwrap())];
        // Only mutants whose verdict nothing else decides: no scenario new to them (which hides a
        // kill), no gained or baseline refusal (unwitnessed) and no dead guard (equivalent).
        if !ids(suite).contains(&target)
            || entry.exclusions.as_ref().is_some_and(|it| {
                it.iter()
                    .any(|it| it.reason == ExclusionReason::NoBaselineControl)
            })
            || entry.added_refusals.is_some()
            || entry.baseline_refusals.is_some()
            || entry.unsatisfiable_guard.is_some()
        {
            continue;
        }
        assert_eq!(entry.killers, None, "{entry:?}");
        // Whether the mutant changed any declared scenario: each such copy could have killed it,
        // had the baseline passed it.
        let changes_one = failed
            .iter()
            .any(|id| ids(suite).contains(id) && body(suite, id) != body(&baseline.suite, id));
        if changes_one {
            changed += 1;
            // Its changed copy might have killed it, had the baseline passed it.
            assert_eq!(entry.verdict, Verdict::Inconclusive, "{entry:?}");
        } else {
            unchanged += 1;
            // Every declared copy asks what the baseline asked: nothing contradicted the mutant.
            assert_eq!(entry.verdict, Verdict::Survived, "{entry:?}");
        }
    }
    assert!(
        unchanged > 0,
        "some mutant leaves every declared scenario as the baseline has it"
    );
    assert!(changed > 0, "some mutant changes a declared scenario");
}

#[test]
fn a_mutant_with_every_witness_excluded_is_inconclusive_never_killed() {
    // The baseline passes one scenario, `kept`, and fails every other one, each declared; a mutant
    // whose suite lacks `kept` then has no baseline-passing control at all: every scenario of its
    // suite is declared or new to it.
    let baseline = baseline(MutantClass::ALL);
    let baseline_ids = ids(&baseline.suite);
    let (files, texts) = example("billing");
    let plain = mutate::emit(&files, &texts, MutantClass::ALL).unwrap();
    let (mutant, kept) = plain
        .manifest
        .mutants
        .iter()
        .filter_map(|it| it.dir.clone())
        .find_map(|dir| {
            let held = ids(&plain.files[&format!("{dir}/{SUITE_FILE}")]);
            let missing = baseline_ids.difference(&held).next()?.clone();
            Some((dir, missing))
        })
        .expect("some billing mutant's suite lacks a baseline scenario");
    let failing: BTreeSet<String> = baseline_ids
        .iter()
        .filter(|id| **id != kept)
        .cloned()
        .collect();
    let red = exactly_failing(&baseline.report, &baseline.suite, &failing);
    let declared = Baseline {
        suite: baseline.suite.clone(),
        report: red.clone(),
        implementation: baseline.implementation.clone(),
    };
    let declaration = declaration(
        &declared,
        &baseline.implementation,
        &failing.iter().cloned().collect::<Vec<_>>(),
    );
    let (_, mut written) = collected_emission(MutantClass::ALL, Some(&declaration));
    put_report(&mut written, BASELINE_DIR, &red, &build('a'));
    let mutant_report = written[&format!("{mutant}/{REPORT_FILE}")].clone();
    let suite = written[&format!("{mutant}/{SUITE_FILE}")].clone();
    assert!(
        !with_status(&mutant_report, &suite, "failed").is_empty(),
        "the mutant's own run fails scenarios, so a scorer that let them witness would kill it"
    );
    let report = collect(&written, None).unwrap_or_else(|refusal| panic!("{refusal}"));
    let entry = report.mutants.iter().find(|it| it.id == mutant).unwrap();
    // Never killed. This mutant also gained a synthesis refusal, which makes it unwitnessed ahead
    // of inconclusive (docs/design/mutation-scope-and-known-failures.md).
    let expected = if entry.added_refusals.is_some() || entry.baseline_refusals.is_some() {
        Verdict::Unwitnessed
    } else {
        Verdict::Inconclusive
    };
    assert_eq!(entry.verdict, expected, "{entry:?}");
    assert_eq!(entry.killers, None);
    assert_eq!(
        entry.exclusions.as_ref().map(Vec::len),
        entry.scenarios,
        "every scenario of its suite is excluded: {entry:?}"
    );
    for entry in &report.mutants {
        let Some(scenarios) = entry.scenarios else {
            continue;
        };
        if entry.exclusions.as_ref().map_or(0, Vec::len) == scenarios {
            // A gained or baseline refusal makes it unwitnessed ahead of inconclusive.
            let expected = if entry.added_refusals.is_some() || entry.baseline_refusals.is_some() {
                Verdict::Unwitnessed
            } else {
                Verdict::Inconclusive
            };
            assert_eq!(entry.verdict, expected, "{entry:?}");
        }
    }
}

#[test]
fn a_declared_baseline_scenario_ending_error_unsupported_or_skipped_is_refused() {
    let baseline = baseline(CLASSES);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let (_, written) = collected_emission(CLASSES, Some(&declaration));
    for category in ["error", "unsupported", "skipped"] {
        let mut value: Value = serde_json::from_str(&baseline.report).unwrap();
        let outcomes = value["outcomes"].as_object_mut().unwrap();
        let moved = failed[0].clone();
        outcomes["failed"]
            .as_array_mut()
            .unwrap()
            .retain(|it| it != &json!(moved));
        outcomes[category]
            .as_array_mut()
            .unwrap()
            .push(json!(moved));
        let mut list: Vec<String> = serde_json::from_value(outcomes[category].clone()).unwrap();
        list.sort();
        outcomes[category] = json!(list);
        value["producer_profile"] = "go-scenario-status/2".into();
        value["counts"]["failed"] = json!(failed.len() - 1);
        value["counts"][category] = json!(1);
        if failed.len() == 1 && category != "unsupported" {
            value["execution_status"] = "inconclusive".into();
            value["conformance_status"] = "inconclusive".into();
        }
        let text = serde_json::to_string_pretty(&value).unwrap() + "\n";
        CountReport::from_json(&text, &AdmittedSuite::from_json(&baseline.suite).unwrap())
            .expect("a coherent rewritten report/2");
        let mut changed = written.clone();
        put_report(&mut changed, BASELINE_DIR, &text, &build('a'));
        let refusal = known_failures(collect(&changed, None).expect_err(category));
        assert_eq!(
            refusal.kind,
            RefusalKind::NotFailed,
            "{category}: {refusal}"
        );
    }
}

// ---- the manifest ---------------------------------------------------------------------------------

#[test]
fn only_version_4_binds_a_declaration_and_records_suite_digests() {
    let baseline = baseline(&[MutantClass::ErrorSwap]);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let declaration = declaration(&baseline, &baseline.implementation, &failed);
    let (files, texts) = example("billing");
    let declared = mutate::emit_with(
        &files,
        &texts,
        &[MutantClass::ErrorSwap],
        None,
        Some(&declaration),
    )
    .unwrap();
    let plain = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).unwrap();
    assert_eq!(declared.manifest.format, MANIFEST_FORMAT_4);
    assert_eq!(plain.manifest.format, MANIFEST_FORMAT);
    assert!(
        !plain.files[MANIFEST_FILE].contains("suite_digest"),
        "version 3 is unchanged"
    );
    Manifest::from_json(&declared.files[MANIFEST_FILE]).expect("reads");

    let value: Value = serde_json::from_str(&declared.files[MANIFEST_FILE]).unwrap();
    let mut downgraded = value.clone();
    downgraded["format"] = json!(MANIFEST_FORMAT);
    let refusal = Manifest::from_json(&downgraded.to_string()).expect_err("known_failures is /4");
    assert!(
        refusal.contains("known_failures") && refusal.contains(MANIFEST_FORMAT_4),
        "{refusal}"
    );
    downgraded.as_object_mut().unwrap().remove("known_failures");
    let refusal = Manifest::from_json(&downgraded.to_string()).expect_err("suite_digest is /4");
    assert!(
        refusal.contains("suite_digest") && refusal.contains(MANIFEST_FORMAT_4),
        "{refusal}"
    );

    let mut undigested = value.clone();
    undigested["baseline"]
        .as_object_mut()
        .unwrap()
        .remove("suite_digest");
    let refusal = Manifest::from_json(&undigested.to_string()).expect_err("/4 records digests");
    assert!(refusal.contains("suite_digest"), "{refusal}");

    let mut extra = value.clone();
    extra["known_failures"]["waiver"] = json!(true);
    assert!(Manifest::from_json(&extra.to_string()).is_err(), "closed");
    let mut outside = value.clone();
    outside["known_failures"]["file"] = json!("../known-failures.json");
    assert!(
        Manifest::from_json(&outside.to_string()).is_err(),
        "inside the emission"
    );
    let mut digest = value;
    digest["known_failures"]["digest"] = json!("sha256:XYZ");
    assert!(
        Manifest::from_json(&digest.to_string()).is_err(),
        "a SHA-256"
    );
}

#[test]
fn emit_checks_the_declaration_against_the_baseline_it_writes() {
    let baseline = baseline(&[MutantClass::ErrorSwap]);
    let failed = with_status(&baseline.report, &baseline.suite, "failed");
    let mut value: Value =
        serde_json::from_str(&declaration(&baseline, &baseline.implementation, &failed)).unwrap();
    value["suite_digest"] = json!(build('e'));
    let (files, texts) = example("billing");
    let refusal = known_failures(
        mutate::emit_with(
            &files,
            &texts,
            &[MutantClass::ErrorSwap],
            None,
            Some(&value.to_string()),
        )
        .expect_err("another suite"),
    );
    assert_eq!(refusal.kind, RefusalKind::Identity, "{refusal}");
    // The refusal names the digest the emission's baseline actually has.
    let admitted = AdmittedSuite::from_json(&baseline.suite).unwrap();
    assert!(refusal.to_string().contains(admitted.digest()), "{refusal}");
}
