//! A `precedence-swap` of two branches whose observable answers are identical — the same error,
//! the same or no payload, and no state change, sets or events — is `equivalent`
//! (`ESS-MUTATE-005`), with the shared answer recorded apart from `unsatisfiable_guard`
//! (<https://github.com/beyond10x/ess/issues/517>). Two refusals that differ in error or payload
//! keep their scoring
//! (<https://github.com/beyond10x/ess/issues/472>).
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, Emission, MutantClass, MutationReport, BASELINE_DIR, MANIFEST_FILE,
    REPORT_FILE, SUITE_FILE,
};
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const OPEN: &str = "precedence-swap/desk.entry.Open/leading-hold/trailing-hold";
const CHECK: &str = "precedence-swap/desk.entry.Check/leading-hold/trailing-hold";
const GRADE: &str = "precedence-swap/desk.entry.Grade/too-low/low";

fn fixture() -> (Vec<Document>, SourceMap) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-identical-answer.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    let label = "mutation-identical-answer.yaml".to_owned();
    let raw = RawSpecFile::parse(&text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text);
    (vec![(Source::new(label), raw)], texts)
}

fn audit(classes: &[MutantClass]) -> MutationReport {
    let (files, texts) = fixture();
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(&files, &texts, classes, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn mutant<'a>(json: &'a Value, id: &str) -> &'a Value {
    json["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == id)
        .unwrap_or_else(|| panic!("no {id} in {json:#}"))
}

/// The emission of `classes`, with a report beside every suite: the interpreter's, where every
/// scenario it ran is then recorded as passed. Deliberate collector input, not evidence that this
/// target passed.
fn emitted(classes: &[MutantClass]) -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = fixture();
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    let emission = mutate::emit(&files, &texts, classes).expect("the fixture emits");
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
        let mut report: Value = serde_json::from_str(&text).unwrap();
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
    (emission, written)
}

const SHARED: &str =
    "refuse with `desk.entry.Invalid` and no payload; neither changes state, sets or emits anything";

#[test]
fn a_swap_of_two_refusals_answering_alike_is_equivalent_naming_the_shared_answer() {
    let report = audit(&[MutantClass::PrecedenceSwap]);
    let text = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let open = mutant(&json, OPEN);
    assert_eq!(open["verdict"], "equivalent", "{text}");
    assert_eq!(open["identical_answer"], SHARED, "{text}");
    assert!(
        open.get("unsatisfiable_guard").is_none(),
        "the guards overlap on `hold hold`; what makes the order moot is the answer, not a dead \
         overlap: {open:#}"
    );
    assert_eq!(
        json["format"], "ess-mutation-report/4",
        "`ess-mutation-report/3` has no `identical_answer`"
    );
    assert!(
        text.lines()
            .any(|line| line.starts_with(&format!("equivalent {OPEN}: "))
                && line.ends_with(&format!(" — ESS-MUTATE-005: both branches {SHARED}"))),
        "{text}"
    );
}

#[test]
fn refusals_differing_in_error_or_payload_keep_their_scoring() {
    let report = audit(&[MutantClass::PrecedenceSwap]);
    let text = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    for (id, verdict) in [
        // The same error, a payload read from different inputs.
        (CHECK, "survived"),
        // Different errors, overlapping below 10 (https://github.com/beyond10x/ess/issues/472).
        (GRADE, "killed"),
    ] {
        let entry = mutant(&json, id);
        assert_eq!(entry["verdict"], verdict, "{id}\n{text}");
        assert!(entry.get("identical_answer").is_none(), "{id}: {entry:#}");
    }
}

#[test]
fn the_manifest_and_the_collected_report_name_the_shared_answer_apart_from_a_dead_guard() {
    let (emission, written) = emitted(&[MutantClass::PrecedenceSwap]);
    let manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    assert_eq!(manifest["format"], "ess-mutation-manifest/4");
    let open = mutant(&manifest, OPEN);
    assert_eq!(open["identical_answer"], SHARED, "{open:#}");
    assert!(open.get("unsatisfiable_guard").is_none(), "{open:#}");
    for id in [CHECK, GRADE] {
        assert!(
            mutant(&manifest, id).get("identical_answer").is_none(),
            "{id}"
        );
    }
    let mutant_file: Value = serde_json::from_str(&emission.files[&format!("{OPEN}/mutant.json")])
        .expect("the mutant's own entry");
    assert_eq!(mutant_file["identical_answer"], SHARED);

    let report =
        mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-mutation-report/4");
    let open = mutant(&json, OPEN);
    assert_eq!(open["verdict"], "equivalent", "{}", report.render_text());
    assert_eq!(open["identical_answer"], SHARED);
    let check = mutant(&json, CHECK);
    assert_eq!(check["verdict"], "survived", "{}", report.render_text());
}

#[test]
fn a_version_3_manifest_carrying_identical_answer_is_refused() {
    let (emission, mut written) = emitted(&[MutantClass::GuardNegate]);
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/3");
    let mut manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    let carrier = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it.get("dir").is_some())
        .expect("a mutant with a suite");
    carrier["identical_answer"] = SHARED.into();
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("`ess-mutation-manifest/3` has no `identical_answer`");
    let said = refusal.to_string();
    assert!(
        said.contains("`identical_answer`") && said.contains("ess-mutation-manifest/4"),
        "{said}"
    );
}

#[test]
fn identical_answer_on_a_mutant_that_swaps_nothing_is_refused() {
    let (emission, mut written) = emitted(&[MutantClass::GuardNegate, MutantClass::PrecedenceSwap]);
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/4");
    let mut manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    let carrier = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it["class"] == "guard-negate" && it.get("dir").is_some())
        .expect("a guard-negate mutant with a suite");
    carrier["identical_answer"] = SHARED.into();
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("only a `precedence-swap` has two answers to compare");
    let said = refusal.to_string();
    assert!(
        said.contains("`identical_answer`") && said.contains("precedence-swap"),
        "{said}"
    );
}
