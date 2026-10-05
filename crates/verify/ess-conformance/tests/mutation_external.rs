//! The mutation audit against an external target: `--emit` the suites, `--collect` the reports.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, "Auditing an external target". The built-in
//! audit runs every suite in process; an adopter's implementation runs in its own language, so the
//! audit is split in two. [`mutate::emit`] writes the baseline suite and every mutant's suite and
//! runs nothing. The project runs its own runner over each and writes the conformance report it
//! already writes. [`mutate::collect`] scores those reports into the same `ess-mutation-report/3`.
//!
//! The deciding check is the first one: the reference target run over every emitted suite, its
//! reports collected, comes to exactly the report the built-in audit writes for the same tree.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::mutate::{
    self, AuditRefusal, Document, Emission, MutantClass, Verdict, BASELINE_DIR, MANIFEST_FILE,
    MANIFEST_FORMAT_4, MUTANT_FILE, REPORT_FILE, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
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

/// Runs `target` over the suite emitted at `dir` and returns the report a project runner would
/// write there.
fn run_emitted<T: ConformanceTarget>(emission: &Emission, dir: &str, target: &T) -> String {
    let text = &emission.files[&format!("{dir}/{SUITE_FILE}")];
    let admitted = AdmittedSuite::from_json(text).expect("an emitted suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    CountReport::from_run(&run, &admitted)
        .expect("a complete run has a report/2")
        .to_canonical_json()
        .expect("report/2 serializes")
}

/// The emission, plus a report beside every suite: the baseline's from `baseline`, every mutant's
/// from a fresh Billing.
fn reports<T: ConformanceTarget>(
    emission: &Emission,
    baseline: impl Fn() -> T,
) -> BTreeMap<String, String> {
    let mut files = emission.files.clone();
    files.insert(
        format!("{BASELINE_DIR}/{REPORT_FILE}"),
        run_emitted(emission, BASELINE_DIR, &baseline()),
    );
    for mutant in &emission.manifest.mutants {
        let Some(dir) = &mutant.dir else { continue };
        files.insert(
            format!("{dir}/{REPORT_FILE}"),
            run_emitted(emission, dir, &Billing::new()),
        );
    }
    files
}

fn collect(files: &BTreeMap<String, String>) -> Result<mutate::MutationReport, AuditRefusal> {
    mutate::collect(|path| files.get(path).cloned())
}

#[test]
fn collecting_the_reference_runs_of_every_emitted_suite_is_the_built_in_audit() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, MutantClass::ALL).expect("billing emits");
    let mut collected = collect(&reports(&emission, Billing::new)).expect("the reports collect");
    let audited = mutate::audit(&files, &texts, MutantClass::ALL, Billing::new).expect("audits");
    // A report names the implementation as `<name> <version>`; the built-in audit names it by
    // name alone. The one field the two may spell differently, and only in that way.
    let identity = Billing::new().identity().expect("billing names itself");
    assert_eq!(
        collected.implementation,
        format!("{} {}", identity.name, identity.version)
    );
    collected.implementation = audited.implementation.clone();
    assert_eq!(collected.to_canonical_json(), audited.to_canonical_json());
    assert!(collected.counts.killed > 0 && collected.counts.stillborn > 0);
}

#[test]
fn the_emission_is_a_manifest_a_baseline_and_one_directory_per_mutant() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, MutantClass::ALL).expect("billing emits");
    let manifest: serde_json::Value =
        serde_json::from_str(&emission.files[MANIFEST_FILE]).expect("the manifest is JSON");
    // `/4`: every class includes `emit-swap`, a class only `/4` readers know, and billing has
    // emit-swap mutants and unavailable sites (beyond10x/ess#295).
    assert_eq!(manifest["format"], MANIFEST_FORMAT_4);
    assert_eq!(emission.manifest.baseline.dir, BASELINE_DIR);
    assert!(emission
        .files
        .contains_key(&format!("{BASELINE_DIR}/{SUITE_FILE}")));
    assert!(
        !emission
            .files
            .keys()
            .any(|path| path.ends_with(REPORT_FILE)),
        "emit runs nothing, so it writes no report"
    );
    let audited = mutate::audit(&files, &texts, MutantClass::ALL, Billing::new).expect("audits");
    assert_eq!(emission.manifest.mutants.len(), audited.counts.mutants);
    for mutant in &emission.manifest.mutants {
        assert!(mutant.id.starts_with(&format!("{}/", mutant.class)));
        assert_eq!(format!("{}/{}", mutant.class, mutant.site), mutant.id);
        let identity: serde_json::Value =
            serde_json::from_str(&emission.files[&format!("{}/{MUTANT_FILE}", mutant.id)])
                .expect("the mutant's identity is JSON");
        assert_eq!(identity["id"], mutant.id.as_str());
        assert_eq!(identity["site"], mutant.site.as_str());
        assert_eq!(identity["change"], mutant.change.as_str());
        let suite = format!("{}/{SUITE_FILE}", mutant.id);
        if mutant.stillborn.is_some() {
            assert!(mutant.dir.is_none(), "{}", mutant.id);
            assert!(!emission.files.contains_key(&suite), "{}", mutant.id);
        } else {
            assert_eq!(mutant.dir.as_deref(), Some(mutant.id.as_str()));
            assert!(emission.files.contains_key(&suite), "{}", mutant.id);
        }
    }
}

#[test]
fn a_missing_report_is_inconclusive_and_says_which() {
    let (files, texts) = example("billing");
    let classes = [MutantClass::ErrorSwap];
    let emission = mutate::emit(&files, &texts, &classes).expect("billing emits");
    let mut written = reports(&emission, Billing::new);
    let missing = emission.manifest.mutants[0].id.clone();
    written.remove(&format!("{missing}/{REPORT_FILE}"));
    let report = collect(&written).expect("the rest collects");
    assert_eq!(report.counts.inconclusive, 1);
    assert_eq!(report.counts.killed, report.counts.mutants - 1);
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.id == missing)
        .expect("the unreported mutant is listed");
    assert_eq!(entry.verdict, Verdict::Inconclusive);
    let why = entry
        .unscored
        .as_deref()
        .expect("says why it is not scored");
    assert!(why.contains(&format!("{missing}/{REPORT_FILE}")), "{why}");
}

#[test]
fn a_report_of_another_suite_is_not_scored() {
    let (files, texts) = example("billing");
    let classes = [MutantClass::ErrorSwap];
    let emission = mutate::emit(&files, &texts, &classes).expect("billing emits");
    let mut written = reports(&emission, Billing::new);
    let baseline = written[&format!("{BASELINE_DIR}/{REPORT_FILE}")].clone();
    let target = emission.manifest.mutants[1].id.clone();
    written.insert(format!("{target}/{REPORT_FILE}"), baseline);
    let report = collect(&written).expect("the rest collects");
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.id == target)
        .expect("listed");
    assert_eq!(entry.verdict, Verdict::Inconclusive);
    assert!(entry.unscored.is_some());
}

#[test]
fn a_baseline_report_that_did_not_pass_is_refused_with_mutate_001() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, MutantClass::ALL).expect("billing emits");
    let written = reports(&emission, || faulty::billing(Fault::AllowIllegalTransition));
    let refusal = collect(&written).expect_err("a red baseline is refused");
    let AuditRefusal::BaselineFailed { not_passed, .. } = &refusal else {
        panic!("expected ESS-MUTATE-001, got {refusal}");
    };
    assert_eq!(
        refusal.code().map(|code| code.to_string()).as_deref(),
        Some("ESS-MUTATE-001")
    );
    let built_in = mutate::audit(&files, &texts, MutantClass::ALL, || {
        faulty::billing(Fault::AllowIllegalTransition)
    })
    .expect_err("the built-in audit refuses it too");
    let AuditRefusal::BaselineFailed {
        not_passed: expected,
        ..
    } = &built_in
    else {
        panic!("{built_in}");
    };
    let mut expected = expected.clone();
    expected.sort();
    let mut not_passed = not_passed.clone();
    not_passed.sort();
    assert_eq!(not_passed, expected);
}

#[test]
fn a_missing_baseline_report_is_refused() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    let mut written = reports(&emission, Billing::new);
    written.remove(&format!("{BASELINE_DIR}/{REPORT_FILE}"));
    let refusal = collect(&written).expect_err("nothing is scored without a baseline");
    assert!(refusal.to_string().contains(BASELINE_DIR), "{refusal}");
}

#[test]
fn a_manifest_directory_outside_the_emission_is_refused() {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    let mut written = reports(&emission, Billing::new);
    let manifest = written[MANIFEST_FILE].replace(
        &format!("\"dir\": \"{}\"", emission.manifest.mutants[0].id),
        "\"dir\": \"../../elsewhere\"",
    );
    assert_ne!(manifest, written[MANIFEST_FILE]);
    written.insert(MANIFEST_FILE.to_owned(), manifest);
    collect(&written).expect_err("a directory that leaves the emission is refused");
}

#[test]
fn a_class_with_no_site_emits_nothing_with_mutate_003() {
    let (files, texts) = example("oracle-fixture");
    let refusal = mutate::emit(&files, &texts, &[MutantClass::OrderFlip])
        .expect_err("no site, nothing to emit");
    assert_eq!(
        refusal.code().map(|code| code.to_string()).as_deref(),
        Some("ESS-MUTATE-003")
    );
}
