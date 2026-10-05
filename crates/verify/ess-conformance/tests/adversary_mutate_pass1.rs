//! Adversary pass 1 against the mutation audit's `ess-mutation-report/2` (issues #203, #210).
//!
//! Each case asserts a claim the unit's own documents make, driven from the implementation.

use std::collections::{BTreeMap, BTreeSet};
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

fn keys(ir: &ess_compiler::EssIr) -> Vec<RefusalKey> {
    let mut keys: Vec<RefusalKey> = ess_conformance::synthesize(ir)
        .refusals
        .iter()
        .map(RefusalKey::of)
        .collect();
    keys.sort();
    keys
}

/// `added_refusals` is a set difference over `RefusalKey`, while `refusals` and a `/1` manifest
/// count occurrences. The two agree only if no suite holds two refusals with one key.
#[test]
fn no_suite_of_the_repositorys_audits_holds_two_refusals_with_one_key() {
    let mut duplicated = Vec::new();
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
            let all = keys(&ir);
            let unique: BTreeSet<&RefusalKey> = all.iter().collect();
            if unique.len() != all.len() {
                duplicated.push(format!("{name} {id}: {all:?}"));
            }
        }
    }
    assert!(duplicated.is_empty(), "{duplicated:#?}");
}

/// The guide and the support page: "a survivor is a rule synthesis does not pin, answered by the
/// model or a synthesis gap". A survivor on a target that leaves scenarios unexecuted must then
/// also survive on a target that executes every scenario of the same suite.
#[test]
fn a_survivor_against_the_interpreter_is_a_survivor_against_the_billing_reference() {
    let (files, texts) = example("billing");
    let ir = mutate::compile(files.clone(), &texts).unwrap();
    let interpreted = mutate::audit(&files, &texts, MutantClass::ALL, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let reference = mutate::audit(&files, &texts, MutantClass::ALL, Billing::new)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let verdicts = |report: &MutationReport| -> BTreeMap<String, Verdict> {
        report
            .mutants
            .iter()
            .map(|entry| (entry.id.clone(), entry.verdict))
            .collect()
    };
    let full = verdicts(&reference);
    let false_survivors: Vec<String> = interpreted
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Survived)
        .filter(|entry| full[&entry.id] == Verdict::Killed)
        .map(|entry| {
            let killers = reference
                .mutants
                .iter()
                .find(|it| it.id == entry.id)
                .and_then(|it| it.killers.clone())
                .unwrap_or_default();
            format!(
                "{} survived against the interpreter with {:?} excluded; the reference kills it \
                 with {killers:?}",
                entry.id, entry.excluded
            )
        })
        .collect();
    assert!(
        false_survivors.is_empty(),
        "{false_survivors:#?}\n{}",
        interpreted.render_text()
    );
    // Coordinator decision F2: a mutant that no executed scenario kills while one of its own
    // scenarios was excluded is `inconclusive`, listing the excluded ones, not `survived`.
    let excluded_but_not_inconclusive: Vec<String> = interpreted
        .mutants
        .iter()
        .filter(|entry| entry.excluded.is_some())
        .filter(|entry| !matches!(entry.verdict, Verdict::Killed | Verdict::Unwitnessed))
        .filter(|entry| entry.verdict != Verdict::Inconclusive)
        .map(|entry| format!("{} is {:?}", entry.id, entry.verdict))
        .collect();
    assert!(
        excluded_but_not_inconclusive.is_empty(),
        "decision F2: an unkilled mutant with excluded scenarios is inconclusive: \
         {excluded_but_not_inconclusive:#?}"
    );
    let order_flip: Vec<&mutate::MutantEntry> = interpreted
        .mutants
        .iter()
        .filter(|entry| entry.class == MutantClass::OrderFlip && full[&entry.id] == Verdict::Killed)
        .collect();
    assert!(!order_flip.is_empty(), "billing has an order-flip mutant");
    for entry in order_flip {
        assert_eq!(entry.verdict, Verdict::Killed, "{}", entry.id);
        assert_eq!(entry.excluded, None);
        let expected = reference
            .mutants
            .iter()
            .find(|it| it.id == entry.id)
            .unwrap();
        assert_eq!(entry.killers, expected.killers, "{}", entry.id);
        assert!(entry
            .killers
            .as_ref()
            .unwrap()
            .iter()
            .any(|id| id == "billing.invoice.IssueInvoice/outcome/issued"));
    }
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

fn key(scenario: &str) -> serde_json::Value {
    serde_json::json!({"code": "ESS-SYNTH-003", "scenario": scenario})
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

/// Refusal identity is (code, scenario): losing one refusal and gaining another of the same code
/// is a gained refusal.
#[test]
fn a_mutant_that_trades_one_refusal_for_another_of_the_same_code_is_unwitnessed() {
    let mut written = shop_emission();
    rewrite(
        &mut written,
        vec![key("shop.order.ReportStatus/outcome/pending")],
        vec![key("shop.order.ReportStatus/outcome/settled")],
    );
    let report = mutate::collect(|path| written.get(path).cloned()).unwrap();
    let entry = &report.mutants[0];
    assert_eq!(entry.verdict, Verdict::Unwitnessed, "{entry:?}");
    assert_eq!(
        entry.added_refusals.as_ref().map(Vec::len),
        Some(1),
        "{entry:?}"
    );
}

/// `/2` counts `refusals` and lists `refused` of the same length; a mutant whose suite holds one
/// more refusal than the baseline's gained one, as `/1` (judged by count) says.
#[test]
fn a_second_refusal_with_a_key_the_baseline_has_is_a_gained_refusal() {
    let settled = key("shop.order.ReportStatus/outcome/settled");
    let mut written = shop_emission();
    rewrite(
        &mut written,
        vec![settled.clone()],
        vec![settled.clone(), settled],
    );
    let report = mutate::collect(|path| written.get(path).cloned()).unwrap();
    let entry = &report.mutants[0];
    assert_eq!(entry.verdict, Verdict::Unwitnessed, "{entry:?}");
}
