//! Adversary pass 2 against the mutation audit's `ess-mutation-report/2` (issues #203, #210),
//! attacking the correction: the subject in refusal keys, per-key counts, and the
//! `inconclusive` rule for excluded scenarios.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, MutantClass, MutationReport, RefusalKey, Verdict, BASELINE_DIR, MANIFEST_FILE,
    REPORT_FILE, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

mod support_versions;

fn load(paths: Vec<std::path::PathBuf>) -> (Vec<Document>, SourceMap) {
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in paths {
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
    load(found)
}

fn shop() -> (Vec<Document>, SourceMap) {
    load(vec![
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-gained-refusal.yaml")
    ])
}

/// The key is the refusal's identity for trade detection: a mutant that loses one refusal and
/// gains a *different* one with the same key is not seen. So within one suite, two refusals that
/// share a key must be the same refusal (same cause), for every code, not only `ESS-SYNTH-011`.
#[test]
fn two_different_refusals_never_share_a_key_in_any_audited_suite() {
    let mut collisions = Vec::new();
    for (name, (files, texts)) in [
        ("billing", example("billing")),
        ("oracle-fixture", example("oracle-fixture")),
        ("gatepass", example("gatepass")),
        ("shop", shop()),
    ] {
        let mut suites = vec![(
            "baseline".to_owned(),
            mutate::compile(files.clone(), &texts).expect("compiles"),
        )];
        for mutant in mutate::mutants(&files, MutantClass::ALL) {
            let mutated = mutate::apply(&files, &mutant.mutation).expect("applies");
            if let Ok(ir) = mutate::compile(mutated, &texts) {
                suites.push((mutant.id.clone(), ir));
            }
        }
        for (id, ir) in suites {
            let mut by_key: BTreeMap<RefusalKey, Vec<String>> = BTreeMap::new();
            for refusal in &ess_conformance::synthesize(&ir).refusals {
                by_key
                    .entry(RefusalKey::of(refusal))
                    .or_default()
                    .push(format!("{:?}", refusal.cause));
            }
            for (key, causes) in by_key {
                let mut distinct = causes.clone();
                distinct.sort();
                distinct.dedup();
                if distinct.len() > 1 {
                    collisions.push(format!("{name} {id}: {key} holds {distinct:#?}"));
                }
            }
        }
    }
    assert!(collisions.is_empty(), "{collisions:#?}");
}

/// A scenario the baseline did not execute, and that the mutant's suite holds byte for byte as the
/// baseline's, runs against the same target as the baseline's did: it cannot kill the mutant.
/// Excluding it hides nothing, so it must not turn a mutant that every other scenario let
/// through from `survived` into `inconclusive` (and exit 1 into exit 3). Only an excluded
/// scenario the mutant changed can hide a kill — which is the pass-1 order-flip case.
#[test]
fn an_excluded_scenario_the_mutant_did_not_change_does_not_make_it_inconclusive() {
    let reference = {
        let (files, texts) = example("billing");
        mutate::audit(&files, &texts, MutantClass::ALL, Billing::new)
            .unwrap_or_else(|refusal| panic!("{refusal}"))
    };
    let on_reference: BTreeMap<String, Verdict> = reference
        .mutants
        .iter()
        .map(|entry| (format!("billing {}", entry.id), entry.verdict))
        .collect();
    let mut hidden = Vec::new();
    let mut inconclusive = 0usize;
    for (name, (files, texts)) in [
        ("billing", example("billing")),
        ("oracle-fixture", example("oracle-fixture")),
        ("gatepass", example("gatepass")),
        ("shop", shop()),
    ] {
        let ir = mutate::compile(files.clone(), &texts).unwrap();
        let Ok(interpreted) = mutate::audit(&files, &texts, MutantClass::ALL, || {
            Interpreted::for_model(ir.clone())
        }) else {
            continue;
        };
        let baseline_suite = ess_conformance::synthesize(&ir).suite;
        let scenario_of = |suite: &ess_conformance::ConformanceSuite, id: &str| {
            suite
                .scenarios
                .iter()
                .find(|(key, _)| key.to_string() == id)
                .map(|(_, scenario)| scenario.clone())
        };
        let all = mutate::mutants(&files, MutantClass::ALL);
        for entry in &interpreted.mutants {
            let Some(excluded) = &entry.excluded else {
                continue;
            };
            if entry.verdict != Verdict::Inconclusive || entry.unscored.is_some() {
                continue;
            }
            inconclusive += 1;
            let mutant = all.iter().find(|it| it.id == entry.id).expect("listed");
            let mutated = mutate::apply(&files, &mutant.mutation).unwrap();
            let mutant_suite =
                ess_conformance::synthesize(&mutate::compile(mutated, &texts).unwrap()).suite;
            let unchanged = excluded.iter().all(|id| {
                scenario_of(&mutant_suite, id).is_some()
                    && scenario_of(&mutant_suite, id) == scenario_of(&baseline_suite, id)
            });
            if unchanged {
                hidden.push(format!(
                    "{name} {} is inconclusive only for {} excluded scenario(s) identical to the \
                 baseline's; on the billing reference: {:?}",
                    entry.id,
                    excluded.len(),
                    on_reference.get(&format!("{name} {}", entry.id))
                ));
            }
        }
    }
    assert!(inconclusive > 0, "the check reached no inconclusive mutant");
    assert!(
        hidden.is_empty(),
        "{} of {inconclusive} excluded-inconclusive mutants hidden:\n{hidden:#?}",
        hidden.len()
    );
}

/// The shop emission with a passing report beside every suite.
fn shop_emission() -> BTreeMap<String, String> {
    let (files, texts) = shop();
    let ir = mutate::compile(files.clone(), &texts).unwrap();
    let emission = mutate::emit(&files, &texts, &[MutantClass::GuardConnective]).unwrap();
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
        let admitted =
            AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")]).unwrap();
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
        let text = CountReport::from_run(&run, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        let mut report: serde_json::Value = serde_json::from_str(&text).unwrap();
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
        let text = serde_json::to_string(&report).unwrap();
        CountReport::from_json(&text, &admitted).expect("coherent report/2 collector input");
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    written
}

fn key(scenario: &str, subject: &str) -> serde_json::Value {
    serde_json::json!({"code": "ESS-SYNTH-005", "scenario": scenario, "subject": subject})
}

/// Collection still reads a genuine legacy report beside reports of current suites.
#[test]
fn legacy_report_one_and_current_report_two_collect_the_same_mutation_results() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).unwrap();
    let mut written = emission.files.clone();
    let dirs = std::iter::once(BASELINE_DIR.to_owned()).chain(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|entry| entry.dir.clone()),
    );
    for dir in dirs {
        let admitted = AdmittedSuite::from_json(&written[&format!("{dir}/{SUITE_FILE}")]).unwrap();
        let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Billing::new());
        let report = CountReport::from_run(&run, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        written.insert(format!("{dir}/{REPORT_FILE}"), report);
    }
    let current = mutate::collect(|path| written.get(path).cloned()).unwrap();
    assert!(
        current.counts.killed > 0,
        "the real reference kills an error swap"
    );
    let suite_path = format!("{BASELINE_DIR}/{SUITE_FILE}");
    let original = AdmittedSuite::from_json(&written[&suite_path]).unwrap();
    let legacy_text = support_versions::legacy_json(&written[&suite_path], 4);
    let legacy = AdmittedSuite::from_json(&legacy_text).expect("genuine suite/4 vocabulary");
    assert_eq!(legacy.suite().scenarios, original.suite().scenarios);
    assert_eq!(
        legacy.suite().provenance.spec_digest,
        original.suite().provenance.spec_digest
    );
    assert_ne!(legacy.digest(), original.digest());
    let run = Runner::for_suite(legacy.suite()).run_admitted(&legacy, &Billing::new());
    assert!(run
        .scenarios
        .iter()
        .all(|result| result.status == ess_conformance::Status::Passed));
    let text = run.standalone().to_canonical_json();
    let report: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(report["format"], "ess-conformance-report/1");
    assert_eq!(report["suite_version"], "ess-conformance/4");
    written.insert(suite_path, legacy_text);
    written.insert(format!("{BASELINE_DIR}/{REPORT_FILE}"), text);
    let mixed = mutate::collect(|path| written.get(path).cloned()).unwrap();
    assert_eq!(mixed, current);
}

#[test]
fn report_two_stand_ins_keep_exact_suite_and_status_validation() {
    let written = shop_emission();
    let suite =
        AdmittedSuite::from_json(&written[&format!("{BASELINE_DIR}/{SUITE_FILE}")]).unwrap();
    let path = format!("{BASELINE_DIR}/{REPORT_FILE}");
    let original: serde_json::Value = serde_json::from_str(&written[&path]).unwrap();
    for change in ["digest", "count", "status"] {
        let mut altered = original.clone();
        match change {
            "digest" => altered["suite"]["digest"] = "0".repeat(64).into(),
            "count" => altered["counts"]["passed"] = (suite.suite().len() + 1).into(),
            "status" => altered["execution_status"] = "failed".into(),
            _ => unreachable!(),
        }
        let text = serde_json::to_string(&altered).unwrap();
        assert!(CountReport::from_json(&text, &suite).is_err(), "{change}");
        let mut invalid = written.clone();
        invalid.insert(path.clone(), text);
        assert!(
            mutate::collect(|path| invalid.get(path).cloned()).is_err(),
            "{change}"
        );
    }
}

fn rewrite(
    written: &mut BTreeMap<String, String>,
    baseline: Vec<serde_json::Value>,
    mutant: Vec<serde_json::Value>,
) {
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["baseline"]["refusals"] = baseline.len().into();
    manifest["baseline"]["refused"] = baseline.into();
    let entry = &mut manifest["mutants"][0];
    entry["refusals"] = mutant.len().into();
    entry["refused"] = mutant.into();
    // These manifests judge the mutant by its refusal delta alone: the guard its emitter found
    // dead (#218) would make it `equivalent` whatever the delta, so it is left undecided here.
    entry.as_object_mut().unwrap().remove("unsatisfiable_guard");
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
}

fn collected(baseline: Vec<serde_json::Value>, mutant: Vec<serde_json::Value>) -> MutationReport {
    let mut written = shop_emission();
    rewrite(&mut written, baseline, mutant);
    mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"))
}

const S: &str = "shop.order.ReportStatus/outcome/settled";
const P: &str = "shop.order.ReportStatus/outcome/pending";

/// Losing N of one key never offsets gaining M of another; gaining M over a baseline holding N of
/// the same key adds exactly M − N.
#[test]
fn per_key_counts_add_the_surplus_of_each_key_and_nothing_else() {
    let a = key(S, "a");
    let b = key(P, "b");
    // Loses two of `a`, gains one more `b`: one added, `b`.
    let report = collected(
        vec![a.clone(), a.clone(), a.clone(), b.clone()],
        vec![a.clone(), b.clone(), b.clone()],
    );
    let entry = &report.mutants[0];
    assert_eq!(entry.verdict, Verdict::Unwitnessed, "{entry:?}");
    let added: Vec<String> = entry
        .added_refusals
        .iter()
        .flatten()
        .map(ToString::to_string)
        .collect();
    assert_eq!(added, vec![format!("ESS-SYNTH-005 `{P}` (b)")], "{entry:?}");
    // Gains three `a` over one: two added.
    let report = collected(vec![a.clone()], vec![a.clone(), a.clone(), a.clone()]);
    assert_eq!(
        report.mutants[0].added_refusals.as_ref().map(Vec::len),
        Some(2)
    );
    // Only losses: nothing gained, not unwitnessed.
    let report = collected(vec![a.clone(), a.clone(), b.clone()], vec![a]);
    assert_ne!(report.mutants[0].verdict, Verdict::Unwitnessed);
    assert!(report.mutants[0].added_refusals.is_none());
}

/// Same scenario, same code, different subject: two keys, so trading one for the other is gained.
#[test]
fn a_trade_between_subjects_at_one_scenario_is_a_gained_refusal() {
    let report = collected(vec![key(S, "view A")], vec![key(S, "view B")]);
    assert_eq!(report.mutants[0].verdict, Verdict::Unwitnessed);
}

/// `/1` has no `refused`; a mutant with more refusals than the baseline is unwitnessed by count and
/// names none, and one with fewer is not.
#[test]
fn a_version_one_manifest_is_judged_by_count() {
    for (baseline, mutant, unwitnessed) in [(1usize, 2usize, true), (2, 1, false), (1, 1, false)] {
        let mut written = shop_emission();
        let mut manifest: serde_json::Value =
            serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
        manifest["format"] = mutate::MANIFEST_FORMAT_1.into();
        manifest["baseline"]["refusals"] = baseline.into();
        manifest["baseline"]
            .as_object_mut()
            .unwrap()
            .remove("refused");
        for entry in manifest["mutants"].as_array_mut().unwrap() {
            entry.as_object_mut().unwrap().remove("refused");
            entry.as_object_mut().unwrap().remove("unsatisfiable_guard");
            if entry.get("refusals").is_some() {
                entry["refusals"] = mutant.into();
            }
        }
        written.insert(
            MANIFEST_FILE.to_owned(),
            serde_json::to_string_pretty(&manifest).unwrap(),
        );
        let report = mutate::collect(|path| written.get(path).cloned()).unwrap();
        for entry in report
            .mutants
            .iter()
            .filter(|it| it.verdict != Verdict::Stillborn)
        {
            assert_eq!(
                entry.verdict == Verdict::Unwitnessed,
                unwitnessed,
                "{baseline}->{mutant}: {entry:?}"
            );
            assert!(entry.added_refusals.is_none());
        }
    }
}

/// The report's bytes and text are a function of the tree and the target.
#[test]
fn the_billing_interpreted_report_is_byte_identical_across_runs() {
    let (files, texts) = example("billing");
    let ir = mutate::compile(files.clone(), &texts).unwrap();
    let run = || {
        mutate::audit(&files, &texts, MutantClass::ALL, || {
            Interpreted::for_model(ir.clone())
        })
        .unwrap()
    };
    let (first, second) = (run(), run());
    assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    assert_eq!(first.render_text(), second.render_text());
}

/// The #210 workflow: an adopter's runner cannot execute one scenario (`unsupported`) in the
/// baseline, and a mutant's suite holds that scenario unchanged, so its runner reports it
/// `unsupported` again; every other scenario of the mutant passed (the reports here are written
/// as a runner that let every mutant through would write them). The excluded scenario is the
/// baseline's own, run against the same implementation, so it cannot kill that mutant on this
/// target: nothing about the verdict is unknown, and the mutant is a survivor (exit 1). Today it
/// is `inconclusive` (exit 3), and the survivor is hidden.
#[test]
fn a_mutant_whose_only_excluded_scenario_is_unchanged_from_the_baseline_survives() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, MutantClass::ALL).expect("emits");
    let mut written = emission.files.clone();
    let suite = |dir: &str| {
        AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
            .unwrap()
            .suite()
            .clone()
    };
    let baseline = suite(BASELINE_DIR);
    let dirs: Vec<String> = emission
        .manifest
        .mutants
        .iter()
        .filter_map(|it| it.dir.clone())
        .collect();
    // The baseline scenario the most mutants hold unchanged.
    let (unchanged, holders) = baseline
        .scenarios
        .iter()
        .map(|(id, scenario)| {
            let holders: Vec<String> = dirs
                .iter()
                .filter(|dir| suite(dir).scenarios.get(id) == Some(scenario))
                .cloned()
                .collect();
            (id.to_string(), holders)
        })
        .max_by_key(|(_, holders)| holders.len())
        .expect("a baseline scenario");
    assert!(!holders.is_empty(), "no mutant holds a scenario unchanged");
    let mut all = vec![BASELINE_DIR.to_owned()];
    all.extend(dirs.iter().cloned());
    for dir in &all {
        let admitted =
            AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")]).unwrap();
        let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Billing::new());
        let text = CountReport::from_run(&run, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        let mut report: serde_json::Value = serde_json::from_str(&text).unwrap();
        // Deliberately exclude exactly the shared scenario in the baseline and its holders.
        let excluded = dir == BASELINE_DIR || holders.contains(dir);
        let (unsupported, passed): (Vec<_>, Vec<_>) = admitted
            .suite()
            .scenarios
            .keys()
            .partition(|id| excluded && id.to_string() == unchanged);
        assert_eq!(unsupported.len(), usize::from(excluded));
        report["outcomes"] = serde_json::json!({
            "passed": passed, "failed": [], "error": [],
            "unsupported": unsupported, "skipped": []
        });
        report["counts"] = serde_json::json!({
            "total": admitted.suite().len(), "passed": passed.len(), "failed": 0,
            "error": 0, "unsupported": unsupported.len(), "skipped": 0
        });
        report["execution_status"] = if excluded { "failed" } else { "passed" }.into();
        report["conformance_status"] = if excluded { "failed" } else { "inconclusive" }.into();
        let text = serde_json::to_string(&report).unwrap();
        CountReport::from_json(&text, &admitted).expect("coherent report/2 collector fault");
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    let report = mutate::collect(|path| written.get(path).cloned()).unwrap();
    let dir_of: BTreeMap<&str, &str> = emission
        .manifest
        .mutants
        .iter()
        .filter_map(|it| it.dir.as_deref().map(|dir| (it.id.as_str(), dir)))
        .collect();
    let mut checked = 0;
    let mut hidden = Vec::new();
    for entry in &report.mutants {
        let Some(dir) = dir_of.get(entry.id.as_str()) else {
            continue;
        };
        if !holders.iter().any(|it| it == dir) || entry.added_refusals.is_some() {
            continue;
        }
        if entry.excluded.as_deref() != Some(&[unchanged.clone()][..]) {
            continue;
        }
        checked += 1;
        if entry.verdict != Verdict::Survived {
            hidden.push(format!("{} is {:?}", entry.id, entry.verdict));
        }
    }
    assert!(checked > 0, "no mutant reached the case");
    assert!(
        hidden.is_empty(),
        "{} of {checked} mutants whose one excluded scenario `{unchanged}` is the baseline's, \
         byte for byte, and which passed every scored scenario, are not survivors \
         (counts {:?}):\n{hidden:#?}",
        hidden.len(),
        report.counts
    );
}

/// `ESS-SYNTH-005` is pushed once per undecidable view of one scenario (`synthesize.rs`, the
/// `for view in views` loop), with the scenario's subject, not the view's. Two such refusals are
/// two refusals the audit must tell apart — a mutant that makes one view decidable and another
/// undecidable trades one for the other — so their keys must differ, as the two
/// `ESS-SYNTH-011` invariants' now do.
#[test]
fn two_undecidable_views_at_one_scenario_are_two_keys() {
    use ess_conformance::scenario::ViewRef;
    let (files, texts) = example("billing");
    let ir = mutate::compile(files, &texts).unwrap();
    let names: Vec<_> = ir.views().keys().take(2).cloned().collect();
    assert_eq!(names.len(), 2, "billing declares two views");
    let (template, subject) = ess_conformance::synthesize(&ir)
        .suite
        .scenarios
        .keys()
        .find_map(|id| match id {
            ess_conformance::ScenarioId::Outcome { outcome } => Some((
                id.clone(),
                ess_conformance::EssSemanticRef::from(outcome.clone()),
            )),
            _ => None,
        })
        .expect("an outcome scenario");
    let undecidable = |view: &ess_domain::name::QualifiedName| ess_conformance::Refusal {
        subject: subject.clone(),
        scenario: Some(template.clone()),
        cause: ess_conformance::RefusalCause::ViewUndecidable {
            view: ViewRef::new(view.clone()),
            filter: "status == Issued".to_owned(),
            state: ess_domain::entity::StateName::new("Draft").unwrap(),
            unbound: Vec::new(),
        },
    };
    let (first, second) = (undecidable(&names[0]), undecidable(&names[1]));
    assert_ne!(first, second);
    assert_ne!(RefusalKey::of(&first), RefusalKey::of(&second));
}
