//! `ExecutedRun` cloning and discarded raw diagnostics retain exact source separation.
use ess_conformance::{AdmittedSuite, ConformanceStatus, CountReport, CountRun, Runner};
use serde_json::json;

#[test]
fn cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics() {
    let original = serde_json::to_string_pretty(&json!({
        "provenance":{"suite_version":"ess-conformance/4", "system":"review",
            "specification_version":"v1", "spec_digest":"a".repeat(64), "contract_digest":"a".repeat(64)},
        "scenarios":{"review.count.Type/invariant/at/review.count.Rows/e\u{301}":{
            "purpose":"Clone exact execution capability", "steps":[], "source":[]}}
    })).unwrap().replace('\n', "\r\n") + "\r\n";
    let admitted = AdmittedSuite::from_json(&original).unwrap();
    let escaped_original = original.replace('e', "\\u0065");
    let escaped = AdmittedSuite::from_json(&escaped_original).unwrap();
    assert_eq!(admitted.suite(), escaped.suite());
    assert_ne!(admitted.digest(), escaped.digest());
    let run = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &ess_conformance::reference::Billing::new());
    let retained = run.clone();
    let summary = CountReport::from_run(&retained, &admitted)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let detailed = CountRun::from_run(&retained, &admitted)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let mut legacy = run.into_report();
    legacy.status = ConformanceStatus::Failed;
    legacy.scenarios.clear();
    assert!(!legacy.is_conformant());
    assert!(retained.is_conformant());
    assert_eq!(retained.suite_digest(), admitted.digest());
    assert_eq!(
        CountReport::from_run(&retained, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
        summary
    );
    assert_eq!(
        CountRun::from_run(&retained, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
        detailed
    );
    assert!(CountReport::from_run(&retained, &escaped).is_err());
    assert!(CountRun::from_run(&retained, &escaped).is_err());
    assert!(CountReport::from_json(&summary, &escaped).is_err());
    assert!(CountRun::from_json(&detailed, &escaped).is_err());
    println!(
        "original digest: {}; escaped digest: {}; immutable retained run remains passed",
        admitted.digest(),
        escaped.digest()
    );
}
