//! Adversary cases for `mutate::emit` / `mutate::collect` (beyond10x/ess#153): the branches of
//! `collect` the unit's own `mutation_external.rs` does not reach.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::mutate::{
    self, AuditRefusal, Document, Emission, MutantClass, Verdict, BASELINE_DIR, MANIFEST_FILE,
    REPORT_FILE, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .expect("the example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
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
        let raw = RawSpecFile::parse(&text).expect("well formed");
        texts.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    (parsed, texts)
}

fn report2<T: ConformanceTarget>(emission: &Emission, dir: &str, target: &T) -> String {
    let admitted = AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
        .expect("admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    CountReport::from_run(&run, &admitted)
        .expect("report/2")
        .to_canonical_json()
        .expect("serializes")
}

/// Billing's error-swap emission with a current report from the reference beside every suite.
fn green() -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    let mut written = emission.files.clone();
    written.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        report2(&emission, BASELINE_DIR, &Billing::new()),
    );
    for mutant in &emission.manifest.mutants {
        let Some(dir) = &mutant.dir else { continue };
        written.insert(
            format!("{dir}/{REPORT_FILE}"),
            report2(&emission, dir, &Billing::new()),
        );
    }
    (emission, written)
}

fn collect(files: &BTreeMap<String, String>) -> Result<mutate::MutationReport, AuditRefusal> {
    mutate::collect(|path| files.get(path).cloned())
}

fn verdict_of(report: &mutate::MutationReport, id: &str) -> (Verdict, Option<String>) {
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .expect("listed");
    (entry.verdict, entry.unscored.clone())
}

#[test]
fn adversary_a_mutant_report_answered_by_another_implementation_is_inconclusive() {
    let (emission, mut written) = green();
    let id = emission.manifest.mutants[0].id.clone();
    let path = format!("{id}/{REPORT_FILE}");
    let mut value: Value = serde_json::from_str(&written[&path]).unwrap();
    value["implementation"] = Value::from("someone-else 9.9.9");
    written.insert(path, serde_json::to_string_pretty(&value).unwrap() + "\n");
    let report = collect(&written).expect("collects");
    let (verdict, why) = verdict_of(&report, &id);
    assert_eq!(verdict, Verdict::Inconclusive);
    let why = why.expect("unscored says why");
    assert!(why.contains("someone-else 9.9.9"), "{why}");
}

#[test]
fn adversary_a_red_report2_baseline_is_refused_with_mutate_001() {
    let (emission, mut written) = green();
    written.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        report2(
            &emission,
            BASELINE_DIR,
            &faulty::billing(Fault::AllowIllegalTransition),
        ),
    );
    let refusal = collect(&written).expect_err("a red report/2 baseline is refused");
    assert_eq!(
        refusal.code().map(|code| code.to_string()).as_deref(),
        Some("ESS-MUTATE-001"),
        "{refusal}"
    );
}

#[test]
fn adversary_a_report2_naming_a_scenario_the_suite_does_not_hold_is_not_scored() {
    let (emission, mut written) = green();
    let id = emission.manifest.mutants[0].id.clone();
    let path = format!("{id}/{REPORT_FILE}");
    let mut value: Value = serde_json::from_str(&written[&path]).unwrap();
    let unknown = "no.such.Command/outcome/nowhere";
    let failed = value["outcomes"]["failed"].as_array_mut().unwrap();
    failed.push(Value::from(unknown));
    failed.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    value["counts"]["failed"] = Value::from(failed.len());
    let total = value["counts"]["total"].as_u64().unwrap();
    value["counts"]["total"] = Value::from(total + 1);
    value["execution_status"] = Value::from("failed");
    value["conformance_status"] = Value::from("failed");
    written.insert(path, serde_json::to_string_pretty(&value).unwrap() + "\n");
    let report = collect(&written).expect("collects");
    let (verdict, why) = verdict_of(&report, &id);
    assert_eq!(verdict, Verdict::Inconclusive, "{why:?}");
    let why = why.expect("unscored says why");
    assert!(
        why.contains("total or exact selected outcome membership disagrees"),
        "{why}"
    );
}

#[test]
fn adversary_a_suite_swapped_for_the_baselines_is_not_scored() {
    let (emission, mut written) = green();
    let id = emission.manifest.mutants[0].id.clone();
    let baseline_suite = written[&format!("{BASELINE_DIR}/{SUITE_FILE}")].clone();
    let baseline_report = written[&format!("{BASELINE_DIR}/{REPORT_FILE}")].clone();
    written.insert(format!("{id}/{SUITE_FILE}"), baseline_suite);
    written.insert(format!("{id}/{REPORT_FILE}"), baseline_report);
    let report = collect(&written).expect("collects");
    let (verdict, why) = verdict_of(&report, &id);
    assert_eq!(verdict, Verdict::Inconclusive);
    assert!(why.is_some_and(|why| why.contains("not the suite")));
}

/// A Go-profile report/2 that books the killers as `skipped`: nothing was contradicted.
#[test]
fn adversary_go_skipped_in_report2_is_inconclusive_not_killed_or_survived() {
    let (emission, mut written) = green();
    let (dir, text) = emission
        .manifest
        .mutants
        .iter()
        .filter_map(|mutant| mutant.dir.clone())
        .map(|dir| {
            let text = report2(&emission, &dir, &Billing::new());
            (dir, text)
        })
        .find(|(_, text)| {
            let value: Value = serde_json::from_str(text).unwrap();
            value["counts"]["failed"].as_u64().unwrap() > 0
                && value["counts"]["error"] == 0
                && value["counts"]["unsupported"] == 0
        })
        .expect("some error-swap mutant is killed by failures alone");
    let mut value: Value = serde_json::from_str(&text).unwrap();
    value["producer_profile"] = Value::from("go-scenario-status/1");
    value["outcomes"]["skipped"] = value["outcomes"]["failed"].take();
    value["outcomes"]["failed"] = Value::Array(Vec::new());
    value["counts"]["skipped"] = value["counts"]["failed"].take();
    value["counts"]["failed"] = Value::from(0);
    value["execution_status"] = Value::from("inconclusive");
    value["conformance_status"] = Value::from("inconclusive");
    let text = serde_json::to_string_pretty(&value).unwrap() + "\n";
    let admitted =
        AdmittedSuite::from_json(&written[&format!("{dir}/{SUITE_FILE}")]).expect("admitted");
    CountReport::from_json(&text, &admitted).expect("the rewritten report/2 is coherent");
    written.insert(format!("{dir}/{REPORT_FILE}"), text);
    let report = collect(&written).expect("collects");
    let (verdict, why) = verdict_of(&report, &dir);
    assert_eq!(verdict, Verdict::Inconclusive, "{why:?}");
}

#[test]
fn adversary_every_traversal_spelling_in_the_manifest_is_refused() {
    let (emission, written) = green();
    let original = format!("\"dir\": \"{}\"", emission.manifest.mutants[0].id);
    for bad in [
        "..",
        "../x",
        "a/../../x",
        "/etc",
        "//etc",
        "a//b",
        "./a",
        "a/.",
        "a/",
        "",
        "a\\\\..\\\\b",
        "C:/x",
        "~/x",
    ] {
        let manifest = written[MANIFEST_FILE].replace(&original, &format!("\"dir\": \"{bad}\""));
        assert_ne!(manifest, written[MANIFEST_FILE]);
        let mut files = written.clone();
        files.insert(MANIFEST_FILE.to_owned(), manifest);
        assert!(
            matches!(collect(&files), Err(AuditRefusal::Uncollectable(_))),
            "`{bad}` must be refused"
        );
        let baseline = written[MANIFEST_FILE].replace(
            &format!("\"dir\": \"{BASELINE_DIR}\""),
            &format!("\"dir\": \"{bad}\""),
        );
        let mut files = written.clone();
        files.insert(MANIFEST_FILE.to_owned(), baseline);
        assert!(
            matches!(collect(&files), Err(AuditRefusal::Uncollectable(_))),
            "baseline `{bad}` must be refused"
        );
    }
}

#[test]
fn adversary_a_report_of_another_format_is_not_scored() {
    let (emission, mut written) = green();
    let id = emission.manifest.mutants[0].id.clone();
    let path = format!("{id}/{REPORT_FILE}");
    let mut value: Value = serde_json::from_str(&written[&path]).unwrap();
    value["format"] = Value::from("ess-conformance-report/3");
    written.insert(path, serde_json::to_string_pretty(&value).unwrap() + "\n");
    let (verdict, _) = verdict_of(&collect(&written).expect("collects"), &id);
    assert_eq!(verdict, Verdict::Inconclusive);
}
