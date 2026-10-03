//! A mutant whose suite gained synthesis refusals the baseline does not have is `unwitnessed`.
//!
//! Issue #203. A mutant can make one of its own outcomes unsatisfiable; synthesis then refuses
//! that outcome's scenario and the mutant's suite is the baseline's minus it. Scoring what is left
//! as a complete suite called such a mutant `survived`. The verdict now keys on the refusal delta:
//! a mutant that added a refusal and was not killed is `unwitnessed` (`ESS-MUTATE-004`), and every
//! mutant names the refusals it added by code and scenario, on the built-in `--target` path and on
//! `--emit`/`--collect` alike. A failing scored scenario still kills it.
//!
//! The fixture is the issue's shape. Its mutant flips `any` to `all` over two disjoint equalities,
//! so the `settled` outcome has no witness in the mutant's suite. The stand-in runner of the
//! issue, which passes whatever is left, is fabricated here by writing a passing report beside
//! each emitted suite.
//!
//! Issue #218 then scores this very mutant `equivalent`: the guard it leaves is satisfied by no
//! input, which outranks the refusal it gained. The refusal is still named on it. A manifest whose
//! emitter did not decide the guard (`/1`, or a guard the domain does not decide) keeps the
//! refusal-delta verdict, `unwitnessed`.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, Emission, MutantClass, MutateCode, MutationReport, RefusalKey, Verdict,
    BASELINE_DIR, MANIFEST_FILE, MANIFEST_FORMAT, MANIFEST_FORMAT_1, MUTANT_FILE, REPORT_FILE,
    SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::report::Status;
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const MUTANT: &str = "guard-connective/shop.order.ReportStatus/settled/0";
const LOST_SCENARIO: &str = "shop.order.ReportStatus/outcome/settled";
const LOST_SUBJECT: &str = "outcome shop.order.ReportStatus/settled";

/// The issue's specification, with its enum variants in `variants` order.
fn shop(variants: &str) -> (Vec<Document>, SourceMap) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-gained-refusal.yaml");
    let text = std::fs::read_to_string(&path)
        .expect("the fixture is readable")
        .replace("variants: [Lost, New, Paid, Shipped]", variants);
    let label = path.display().to_string();
    let raw = RawSpecFile::parse(&text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text);
    (vec![(Source::new(label), raw)], texts)
}

fn lost_first() -> (Vec<Document>, SourceMap) {
    shop("variants: [Lost, New, Paid, Shipped]")
}

fn paid_first() -> (Vec<Document>, SourceMap) {
    shop("variants: [New, Paid, Shipped, Lost]")
}

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

fn audit((files, texts): &(Vec<Document>, SourceMap)) -> MutationReport {
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    mutate::audit(files, texts, &[MutantClass::GuardConnective], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn lost() -> RefusalKey {
    RefusalKey {
        code: "ESS-SYNTH-003".to_owned(),
        scenario: Some(LOST_SCENARIO.to_owned()),
        subject: Some(LOST_SUBJECT.to_owned()),
    }
}

/// The emission, with a report beside every suite from the unmutated model's interpreter; with
/// `stand_in`, every report says each scenario passed, as the issue's stand-in runner did.
fn emitted(
    (files, texts): &(Vec<Document>, SourceMap),
    stand_in: bool,
) -> (Emission, BTreeMap<String, String>) {
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    let emission =
        mutate::emit(files, texts, &[MutantClass::GuardConnective]).expect("the fixture emits");
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
        let admitted = AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
            .expect("an emitted suite is admitted");
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
        let text = CountReport::from_run(&run, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        let mut report: serde_json::Value = serde_json::from_str(&text).unwrap();
        if stand_in {
            // Deliberate collector input, not evidence that this target passed.
            let ids: Vec<_> = admitted.suite().scenarios.keys().collect();
            report["outcomes"] = serde_json::json!({
                "passed": ids, "failed": [], "error": [], "unsupported": [], "skipped": []
            });
            report["counts"] = serde_json::json!({
                "total": ids.len(), "passed": ids.len(), "failed": 0,
                "error": 0, "unsupported": 0, "skipped": 0
            });
            report["execution_status"] = "passed".into();
            report["conformance_status"] = "inconclusive".into();
        }
        let text = serde_json::to_string(&report).unwrap();
        CountReport::from_json(&text, &admitted).expect("coherent report/2 collector input");
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    (emission, written)
}

fn collect(written: &BTreeMap<String, String>) -> MutationReport {
    mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"))
}

fn entry(report: &MutationReport) -> &mutate::MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == MUTANT)
        .expect("the flipped connective is a mutant")
}

// ---- the issue's shape, collected ---------------------------------------------------------------

#[test]
fn a_mutant_whose_suite_lost_its_outcomes_scenario_is_not_survived_and_names_the_refusal() {
    let report = collect(&emitted(&lost_first(), true).1);
    let entry = entry(&report);
    assert_eq!(entry.verdict, Verdict::Equivalent, "{entry:?}");
    assert_eq!(entry.added_refusals.as_deref(), Some(&[lost()][..]));
    assert_eq!(entry.killers, None);
    assert_eq!(report.counts.equivalent, 1);
    assert_eq!(report.counts.survived, 0);
    assert_eq!(report.baseline.refusals, 0);
    let json: serde_json::Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-mutation-report/3");
    assert_eq!(json["counts"]["equivalent"], 1);
    let mutant = &json["mutants"][0];
    assert_eq!(mutant["verdict"], "equivalent");
    assert_eq!(
        mutant["added_refusals"],
        serde_json::json!([{"code": "ESS-SYNTH-003", "scenario": LOST_SCENARIO, "subject": LOST_SUBJECT}])
    );
}

#[test]
fn the_verdict_is_the_same_in_either_enum_order() {
    for spec in [lost_first(), paid_first()] {
        let report = collect(&emitted(&spec, true).1);
        assert_eq!(entry(&report).verdict, Verdict::Equivalent);
        assert_eq!(
            entry(&report).added_refusals.as_deref(),
            Some(&[lost()][..])
        );
    }
}

#[test]
fn the_text_names_the_added_refusal_beside_the_verdict() {
    let text = collect(&emitted(&lost_first(), true).1).render_text();
    let line = text
        .lines()
        .find(|line| line.starts_with(&format!("equivalent {MUTANT}:")))
        .unwrap_or_else(|| panic!("no equivalent line in:\n{text}"));
    assert!(line.contains("ESS-MUTATE-005"), "{line}");
    assert!(
        line.contains(&format!("adds ESS-SYNTH-003 `{LOST_SCENARIO}`")),
        "{line}"
    );
    assert!(
        text.lines().next().unwrap().contains("1 equivalent"),
        "{text}"
    );
}

// ---- a failure still kills ----------------------------------------------------------------------

#[test]
fn a_mutant_that_gained_a_refusal_and_failed_a_scenario_is_killed_and_names_the_refusal() {
    // The interpreter of the unmutated model settles the witnesses the mutant's fallback scenario
    // now probes, so the mutant's suite fails there: a real kill outranks the refusal delta.
    for spec in [lost_first(), paid_first()] {
        let audited = audit(&spec);
        let entry = entry(&audited);
        assert_eq!(entry.verdict, Verdict::Killed, "{entry:?}");
        assert_eq!(entry.added_refusals.as_deref(), Some(&[lost()][..]));

        let mut collected = collect(&emitted(&spec, false).1);
        collected.implementation = audited.implementation.clone();
        assert_eq!(collected.to_canonical_json(), audited.to_canonical_json());
    }
}

#[test]
fn the_built_in_audit_records_the_refusals_a_killed_mutant_added() {
    let (files, texts) = example("billing");
    let report = mutate::audit(&files, &texts, &[MutantClass::GuardBoundary], Billing::new)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let gained: Vec<&mutate::MutantEntry> = report
        .mutants
        .iter()
        .filter(|entry| entry.refusals > Some(report.baseline.refusals))
        .collect();
    assert!(
        !gained.is_empty(),
        "billing has a mutant that gains a refusal"
    );
    for entry in gained {
        let added = entry
            .added_refusals
            .as_ref()
            .unwrap_or_else(|| panic!("{} names what it added", entry.id));
        assert!(!added.is_empty());
        assert!(
            added.iter().all(|key| key.code.starts_with("ESS-SYNTH-")),
            "{added:?}"
        );
        assert_eq!(entry.verdict, Verdict::Killed, "{}", entry.id);
    }
}

#[test]
fn a_failure_outranks_a_gained_refusal_and_a_gained_refusal_outranks_the_rest() {
    use Status::{Error, Failed, Passed, Unsupported};
    assert_eq!(
        Verdict::judge(&[Passed, Failed], true, false),
        Verdict::Killed
    );
    assert_eq!(Verdict::judge(&[Passed], true, false), Verdict::Unwitnessed);
    assert_eq!(Verdict::judge(&[], true, false), Verdict::Unwitnessed);
    assert_eq!(Verdict::judge(&[Error], true, false), Verdict::Unwitnessed);
    assert_eq!(
        Verdict::judge(&[Unsupported], true, false),
        Verdict::Unwitnessed
    );
    assert_eq!(Verdict::judge(&[Passed], false, false), Verdict::Survived);
    assert_eq!(
        Verdict::judge(&[Error], false, false),
        Verdict::Inconclusive
    );
    assert_eq!(Verdict::judge(&[Failed], false, false), Verdict::Killed);
    assert_eq!(
        Verdict::judge(&[Passed], false, true),
        Verdict::Inconclusive
    );
    assert_eq!(Verdict::judge(&[], false, true), Verdict::Inconclusive);
    assert_eq!(Verdict::judge(&[Failed], false, true), Verdict::Killed);
    assert_eq!(Verdict::judge(&[Passed], true, true), Verdict::Unwitnessed);
}

#[test]
fn unwitnessed_has_its_own_code() {
    assert_eq!(MutateCode::Unwitnessed.code().to_string(), "ESS-MUTATE-004");
    assert_eq!(Verdict::Unwitnessed.as_str(), "unwitnessed");
}

// ---- the manifest -------------------------------------------------------------------------------

#[test]
fn emit_records_each_suites_refusals_by_code_and_scenario() {
    let (emission, _) = emitted(&lost_first(), true);
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/3");
    assert_eq!(emission.manifest.format, MANIFEST_FORMAT);
    assert_eq!(emission.manifest.baseline.refused.as_deref(), Some(&[][..]));
    let mutant = emission
        .manifest
        .mutants
        .iter()
        .find(|mutant| mutant.id == MUTANT)
        .expect("emitted");
    assert_eq!(mutant.refusals, Some(1));
    assert_eq!(mutant.refused.as_deref(), Some(&[lost()][..]));
    let expected = serde_json::json!([{"code": "ESS-SYNTH-003", "scenario": LOST_SCENARIO, "subject": LOST_SUBJECT}]);
    let manifest: serde_json::Value =
        serde_json::from_str(&emission.files[MANIFEST_FILE]).expect("JSON");
    assert_eq!(manifest["baseline"]["refused"], serde_json::json!([]));
    assert_eq!(manifest["mutants"][0]["refused"], expected);
    let identity: serde_json::Value =
        serde_json::from_str(&emission.files[&format!("{MUTANT}/{MUTANT_FILE}")]).expect("JSON");
    assert_eq!(identity["refused"], expected);
}

/// The emission as an earlier release wrote it: `/1`, and no `refused` anywhere.
fn as_version_1(written: &mut BTreeMap<String, String>) {
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["format"] = MANIFEST_FORMAT_1.into();
    manifest["baseline"]
        .as_object_mut()
        .unwrap()
        .remove("refused");
    for mutant in manifest["mutants"].as_array_mut().unwrap() {
        mutant.as_object_mut().unwrap().remove("refused");
        mutant
            .as_object_mut()
            .unwrap()
            .remove("unsatisfiable_guard");
    }
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
}

#[test]
fn a_version_1_manifest_still_collects_and_judges_by_the_refusal_count() {
    let (_, mut written) = emitted(&lost_first(), true);
    as_version_1(&mut written);
    let report = collect(&written);
    let entry = entry(&report);
    assert_eq!(entry.verdict, Verdict::Unwitnessed, "{entry:?}");
    assert_eq!(entry.added_refusals, None, "a /1 manifest names no refusal");
    let text = report.render_text();
    assert!(
        text.contains("ESS-MUTATE-004: 1 refusal(s) against the baseline's 0"),
        "{text}"
    );
}

#[test]
fn a_version_2_manifest_without_a_suites_refusals_is_refused() {
    let (_, mut written) = emitted(&lost_first(), true);
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["mutants"][0]
        .as_object_mut()
        .unwrap()
        .remove("refused");
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("a /2 suite names its refusals");
    assert!(refusal.to_string().contains("`refused`"), "{refusal}");
}

#[test]
fn a_manifest_whose_refusals_disagree_with_their_count_is_refused() {
    let (_, mut written) = emitted(&lost_first(), true);
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["mutants"][0]["refused"] = serde_json::json!([]);
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("a count and a list that disagree are refused");
    assert!(
        refusal.to_string().contains("counts 1 refusal(s)"),
        "{refusal}"
    );
}

#[test]
fn a_version_1_manifest_carrying_refusals_is_refused() {
    let (_, mut written) = emitted(&lost_first(), true);
    let manifest = written[MANIFEST_FILE].replace(MANIFEST_FORMAT, MANIFEST_FORMAT_1);
    written.insert(MANIFEST_FILE.to_owned(), manifest);
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("`refused` is not a /1 field");
    assert!(refusal.to_string().contains(MANIFEST_FORMAT_1), "{refusal}");
}
