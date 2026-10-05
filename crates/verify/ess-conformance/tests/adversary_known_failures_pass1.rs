//! Adversary pass 1 against beyond10x/ess#294: a mutant with every witness excluded by a
//! known-failure declaration, whose suite also gained a synthesis refusal.
//!
//! The implementation decision recorded in `docs/design/mutation-scope-and-known-failures.md`
//! ("Implementation decisions for #294 and #296") says: "A gained or baseline synthesis refusal
//! still makes an otherwise unkilled mutant `unwitnessed` ahead of `inconclusive`, as before". A
//! mutant none of whose scenarios is a baseline-passing control is unkilled by construction, so a
//! gained or baseline refusal makes it `unwitnessed`, as it does without a declaration.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault, Faulty};
use ess_conformance::known_failures::{ExecutionContext, EXECUTION_FILE};
use ess_conformance::mutate::{
    self, Document, MutantClass, Verdict, BASELINE_DIR, REPORT_FILE, SUITE_FILE,
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

fn build() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn defective() -> Faulty<Billing> {
    faulty::billing(Fault::ExtraEvent)
}

fn run(suite: &str, target: &impl ConformanceTarget) -> String {
    let admitted = AdmittedSuite::from_json(suite).expect("an emitted suite is admitted");
    let executed = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    CountReport::from_run(&executed, &admitted)
        .expect("a complete run")
        .to_canonical_json()
        .expect("serializes")
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

/// An actual report of `suite`, rewritten so that exactly `failed` failed and every other scenario
/// passed, under the generated runners' profile; read back through `CountReport`.
fn exactly_failing(report: &str, suite: &str, failed: &BTreeSet<String>) -> String {
    let admitted = AdmittedSuite::from_json(suite).unwrap();
    let mut value: Value = serde_json::from_str(report).unwrap();
    let all: Vec<String> = admitted
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let (red, green): (Vec<String>, Vec<String>) =
        all.into_iter().partition(|id| failed.contains(id));
    value["producer_profile"] = "go-scenario-status/2".into();
    value["counts"] = json!({"total": admitted.suite().len(), "passed": green.len(),
        "failed": red.len(), "error": 0, "unsupported": 0, "skipped": 0});
    value["outcomes"] =
        json!({"passed": green, "failed": red, "error": [], "unsupported": [], "skipped": []});
    value["execution_status"] = if red.is_empty() { "passed" } else { "failed" }.into();
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

fn put_report(written: &mut BTreeMap<String, String>, dir: &str, report: &str) {
    let admitted = AdmittedSuite::from_json(&written[&format!("{dir}/{SUITE_FILE}")]).unwrap();
    let implementation = CountReport::from_json(report, &admitted)
        .expect("a report of this suite")
        .implementation()
        .to_owned();
    let context = ExecutionContext::new(report, &admitted, &implementation, &build()).unwrap();
    written.insert(format!("{dir}/{REPORT_FILE}"), report.to_owned());
    written.insert(
        format!("{dir}/{EXECUTION_FILE}"),
        context.to_canonical_json(),
    );
}

fn declaration(suite: &str, implementation: &str, failing: &BTreeSet<String>) -> String {
    let admitted = AdmittedSuite::from_json(suite).unwrap();
    let value = json!({
        "format": "ess-known-failures/1",
        "spec_digest": admitted.suite().provenance.spec_digest.to_string(),
        "suite_digest": admitted.digest(),
        "implementation": implementation,
        "implementation_build": build(),
        "failures": failing.iter().map(|id| json!({
            "scenario": id,
            "reason": "not built yet",
            "tracking": "ORDERS-412",
        })).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&value).unwrap() + "\n"
}

#[test]
fn a_witnessless_mutant_with_a_gained_or_baseline_refusal_is_unwitnessed_not_inconclusive() {
    let (files, texts) = example("billing");
    let plain = mutate::emit(&files, &texts, MutantClass::ALL).expect("emits");
    let baseline_suite = plain.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")].clone();
    let baseline_report = run(&baseline_suite, &defective());
    let implementation = CountReport::from_json(
        &baseline_report,
        &AdmittedSuite::from_json(&baseline_suite).unwrap(),
    )
    .unwrap()
    .implementation()
    .to_owned();
    let baseline_ids = ids(&baseline_suite);

    // Each mutant's actual report, run once; the suites do not depend on the declaration.
    let mut reports: BTreeMap<String, String> = BTreeMap::new();
    let mut kept_candidates: BTreeSet<String> = BTreeSet::new();
    for dir in plain
        .manifest
        .mutants
        .iter()
        .filter_map(|it| it.dir.clone())
    {
        let suite = &plain.files[&format!("{dir}/{SUITE_FILE}")];
        reports.insert(dir.clone(), run(suite, &defective()));
        if let Some(missing) = baseline_ids.difference(&ids(suite)).next() {
            kept_candidates.insert(missing.clone());
        }
    }
    assert!(
        !kept_candidates.is_empty(),
        "some billing mutant's suite lacks a baseline scenario"
    );

    let mut witnessless_with_refusal = 0;
    let mut misjudged: Vec<String> = Vec::new();
    for kept in kept_candidates.iter().take(6) {
        // The baseline passes `kept` alone and fails every other scenario, each declared: a mutant
        // whose suite lacks `kept` has no baseline-passing control at all.
        let failing: BTreeSet<String> = baseline_ids
            .iter()
            .filter(|id| *id != kept)
            .cloned()
            .collect();
        let red = exactly_failing(&baseline_report, &baseline_suite, &failing);
        let declared = declaration(&baseline_suite, &implementation, &failing);
        let emission = mutate::emit_with(&files, &texts, MutantClass::ALL, None, Some(&declared))
            .expect("emits under the declaration");
        let mut written = emission.files.clone();
        put_report(&mut written, BASELINE_DIR, &red);
        for dir in emission
            .manifest
            .mutants
            .iter()
            .filter_map(|it| it.dir.clone())
        {
            put_report(&mut written, &dir, &reports[&dir]);
        }
        let report = mutate::collect_with(|path| written.get(path).cloned(), None, None)
            .unwrap_or_else(|refusal| panic!("collects: {refusal}"));
        for entry in &report.mutants {
            let Some(scenarios) = entry.scenarios else {
                continue;
            };
            let excluded = entry.exclusions.as_ref().map_or(0, Vec::len);
            let refused = entry.added_refusals.is_some() || entry.baseline_refusals.is_some();
            if excluded == scenarios && refused && entry.unsatisfiable_guard.is_none() {
                witnessless_with_refusal += 1;
                if entry.verdict != Verdict::Unwitnessed {
                    misjudged.push(format!(
                        "{} (kept {kept}): {} with added_refusals {:?}, baseline_refusals {:?}",
                        entry.id,
                        entry.verdict.as_str(),
                        entry.added_refusals.as_ref().map(Vec::len),
                        entry.baseline_refusals,
                    ));
                }
            }
        }
        if witnessless_with_refusal > 0 {
            break;
        }
    }
    assert!(
        witnessless_with_refusal > 0,
        "some witnessless billing mutant gained a refusal"
    );
    assert_eq!(
        misjudged,
        Vec::<String>::new(),
        "a gained or baseline refusal makes an unkilled mutant unwitnessed ahead of inconclusive"
    );
}
