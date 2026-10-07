//! `mutate --emit/--collect --component`: a repository that implements one component scores the
//! mutants its own suite can answer (beyond10x/ess#236).
//!
//! The suites are the component's, as `synthesize --component` writes them. A mutant is kept where
//! the site it mutates — a command's outcome, a view, a transition a command performs — belongs to
//! the component by the membership `synthesize --component` uses, and every other one is listed as
//! out of scope rather than scored. A survivor on the component's own site is scored and counted. The manifest names the component (`ess-mutation-manifest/4`),
//! and so does the report (`ess-mutation-report/4`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{
    self, AuditRefusal, Document, Emission, MutantClass, MutationReport, Verdict, BASELINE_DIR,
    MANIFEST_FILE, MANIFEST_FORMAT, MANIFEST_FORMAT_4, REPORT_FILE, REPORT_FORMAT, REPORT_FORMAT_4,
    SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const INVOICES: &str = "invoice-service";
const EMAIL: &str = "email-service";

fn billing() -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("billing exists");
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
        let raw = RawSpecFile::parse(&text).expect("well formed");
        texts.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    (parsed, texts)
}

fn emit(component: &str) -> Emission {
    let (files, texts) = billing();
    mutate::emit_for(&files, &texts, MutantClass::ALL, Some(component))
        .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// The emission, with the reference's report beside every suite it holds.
fn reported(emission: &Emission) -> BTreeMap<String, String> {
    let mut files = emission.files.clone();
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|mutant| mutant.dir.clone()),
    );
    for dir in dirs {
        let admitted = AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
            .expect("an emitted suite is admitted");
        let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Billing::new());
        files.insert(
            format!("{dir}/{REPORT_FILE}"),
            CountReport::from_run(&run, &admitted)
                .expect("a complete run has a report/2")
                .to_canonical_json()
                .expect("report/2 serializes"),
        );
    }
    files
}

fn collect(
    files: &BTreeMap<String, String>,
    component: Option<&str>,
) -> Result<MutationReport, AuditRefusal> {
    mutate::collect_for(|path| files.get(path).cloned(), component)
}

/// The suite `synthesize --component` writes for `ir`, as the audit emits it: without the
/// `…/grant/denied` scenarios no class alters.
fn component_suite(ir: &ess_compiler::EssIr, component: &str) -> ess_conformance::ConformanceSuite {
    let mut suite = ess_conformance::synthesize::synthesize_for(ir, component)
        .expect("billing declares the component")
        .suite;
    suite
        .scenarios
        .retain(|id, _| !matches!(id, ess_conformance::ScenarioId::Grant { .. }));
    suite.select_fresh_format_for(ir);
    suite
}

#[test]
fn a_component_emission_writes_the_components_suites_and_names_it() {
    let (files, texts) = billing();
    let ir = mutate::compile(files.clone(), &texts).expect("billing compiles");
    let emission = emit(EMAIL);
    assert_eq!(emission.manifest.format, MANIFEST_FORMAT_4);
    assert_eq!(emission.manifest.component.as_deref(), Some(EMAIL));
    let manifest: serde_json::Value =
        serde_json::from_str(&emission.files[MANIFEST_FILE]).expect("JSON");
    assert_eq!(manifest["component"], EMAIL);
    assert_eq!(
        emission.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")],
        component_suite(&ir, EMAIL)
            .to_canonical_json()
            .expect("serializes"),
        "the baseline is the component's suite, as `synthesize --component` writes it"
    );
    let whole = mutate::emit(&files, &texts, MutantClass::ALL).expect("billing emits");
    assert_ne!(
        emission.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")],
        whole.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")],
        "the email component's suite is a strict part of the system's"
    );
}

#[test]
fn a_mutant_is_in_scope_exactly_where_its_site_belongs_to_the_component() {
    let (files, _) = billing();
    // `invoice-service` owns `billing.invoice` and accepts its four commands; `email-service` owns
    // `billing.email`. Every billing mutant's site — a command, a view or an entity's transition —
    // is named inside exactly one of the two domains.
    for (component, domain) in [(INVOICES, "billing.invoice."), (EMAIL, "billing.email.")] {
        let emission = emit(component);
        let expected_out: BTreeSet<String> = mutate::mutants(&files, MutantClass::ALL)
            .into_iter()
            .map(|mutant| mutant.id)
            .filter(|id| !id.contains(domain))
            .collect();
        let out: BTreeSet<String> = emission
            .manifest
            .mutants
            .iter()
            .filter(|mutant| mutant.out_of_scope)
            .map(|mutant| mutant.id.clone())
            .collect();
        assert_eq!(out, expected_out, "{component}");
        for mutant in &emission.manifest.mutants {
            if mutant.out_of_scope {
                assert!(mutant.dir.is_none(), "{} has no suite to run", mutant.id);
                assert!(
                    mutant.stillborn.is_none(),
                    "{} is not compiled for this component",
                    mutant.id
                );
                assert!(
                    !emission
                        .files
                        .keys()
                        .any(|path| path.starts_with(&format!("{}/", mutant.id))),
                    "nothing is written for {}",
                    mutant.id
                );
            }
        }
    }
    // The email component leaves the invoice mutants out; the invoice component keeps its own.
    let email = emit(EMAIL);
    assert!(email.manifest.mutants.iter().any(|it| it.out_of_scope));
    let invoices = emit(INVOICES);
    assert!(invoices
        .manifest
        .mutants
        .iter()
        .any(|it| !it.out_of_scope && it.dir.is_some()));
}

#[test]
fn collecting_a_component_emission_scores_its_mutants_and_lists_the_rest() {
    for component in [INVOICES, EMAIL] {
        collected_for(component);
    }
    // The email component accepts only `SendEmail`, whose one mutant is stillborn: every other
    // mutant is billing.invoice's, and its suite asks the email component nothing new.
    let invoices = collect(&reported(&emit(INVOICES)), None).expect("collects");
    assert!(invoices.counts.killed > 0, "{}", invoices.render_text());
    let email = collect(&reported(&emit(EMAIL)), None).expect("collects");
    assert_ne!(
        email.out_of_scope.as_deref().unwrap_or_default().len(),
        0,
        "another component's mutants are listed out of scope"
    );
}

fn collected_for(component: &str) {
    let emission = emit(component);
    let files = reported(&emission);
    let report = collect(&files, None).unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(report.format, REPORT_FORMAT_4);
    assert_eq!(report.component.as_deref(), Some(component));
    let out: Vec<String> = emission
        .manifest
        .mutants
        .iter()
        .filter(|mutant| mutant.out_of_scope)
        .map(|mutant| mutant.id.clone())
        .collect();
    let listed: Vec<String> = report
        .out_of_scope
        .as_ref()
        .expect("listed")
        .iter()
        .map(|it| it.id.clone())
        .collect();
    assert_eq!(listed, out);
    assert!(report.mutants.iter().all(|entry| !out.contains(&entry.id)));
    assert_eq!(
        report.counts.mutants,
        emission.manifest.mutants.len() - out.len()
    );
    assert_eq!(report.counts.inconclusive, 0, "{}", report.render_text());
    assert!(
        report
            .mutants
            .iter()
            .all(|entry| matches!(entry.verdict, Verdict::Killed | Verdict::Stillborn)),
        "{}",
        report.render_text()
    );
    let text = report.render_text();
    assert!(text.contains(&format!("component `{component}`")), "{text}");
    assert!(
        text.contains(&format!("{} out of scope", out.len())),
        "{text}"
    );
}

#[test]
fn collect_names_the_emitted_component_or_is_refused() {
    let emission = emit(EMAIL);
    let files = reported(&emission);
    let named = collect(&files, Some(EMAIL)).expect("the emitted component");
    let read = collect(&files, None).expect("the manifest's own component");
    assert_eq!(named.to_canonical_json(), read.to_canonical_json());

    let refusal = collect(&files, Some(INVOICES)).expect_err("another component");
    assert!(
        matches!(refusal, AuditRefusal::Uncollectable(_)),
        "{refusal}"
    );
    let text = refusal.to_string();
    assert!(text.contains(EMAIL) && text.contains(INVOICES), "{text}");

    let (system, texts) = billing();
    let whole = mutate::emit(&system, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    let whole_files = reported(&whole);
    let refusal = collect(&whole_files, Some(EMAIL)).expect_err("the emission names none");
    assert!(refusal.to_string().contains(EMAIL), "{refusal}");
}

#[test]
fn an_undeclared_component_is_refused_naming_the_declared_ones() {
    let (files, texts) = billing();
    let refusal = mutate::emit_for(&files, &texts, MutantClass::ALL, Some("ledger"))
        .expect_err("billing declares no ledger");
    assert!(
        matches!(refusal, AuditRefusal::UnknownComponent(_)),
        "{refusal}"
    );
    assert!(!refusal.is_inconclusive());
    let text = refusal.to_string();
    for name in ["ledger", INVOICES, EMAIL] {
        assert!(text.contains(name), "{text}");
    }
}

#[test]
fn a_whole_system_emission_keeps_its_formats() {
    let (files, texts) = billing();
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    assert_eq!(emission.manifest.format, MANIFEST_FORMAT);
    assert_eq!(emission.manifest.component, None);
    assert!(!emission.files[MANIFEST_FILE].contains("component"));
    assert!(!emission.files[MANIFEST_FILE].contains("out_of_scope"));
    let report = collect(&reported(&emission), None).expect("collects");
    assert_eq!(report.format, REPORT_FORMAT);
    let json = report.to_canonical_json();
    assert!(!json.contains("\"component\"") && !json.contains("\"out_of_scope\""));
}

#[test]
fn a_version_3_manifest_naming_a_component_is_refused_naming_version_4() {
    let emission = emit(EMAIL);
    let mut manifest: serde_json::Value =
        serde_json::from_str(&emission.files[MANIFEST_FILE]).expect("JSON");
    // Only the component and the mutants it leaves out are version 4 here.
    manifest["mutants"] = serde_json::Value::Array(
        manifest["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|it| {
                !matches!(it["class"].as_str(), Some("sets-drop" | "precedence-swap"))
                    && it.get("out_of_scope").is_none()
            })
            .cloned()
            .collect(),
    );
    manifest["format"] = MANIFEST_FORMAT.into();
    let refusal = mutate::Manifest::from_json(&manifest.to_string()).expect_err("component is /4");
    assert!(refusal.contains("component"), "{refusal}");
    assert!(refusal.contains(MANIFEST_FORMAT_4), "{refusal}");
}

#[test]
fn an_out_of_scope_mutant_without_a_component_is_refused() {
    let emission = emit(EMAIL);
    let mut manifest: serde_json::Value =
        serde_json::from_str(&emission.files[MANIFEST_FILE]).expect("JSON");
    manifest
        .as_object_mut()
        .unwrap()
        .remove("component")
        .expect("named");
    let refusal =
        mutate::Manifest::from_json(&manifest.to_string()).expect_err("out of scope of nothing");
    assert!(refusal.contains("out_of_scope"), "{refusal}");
}
