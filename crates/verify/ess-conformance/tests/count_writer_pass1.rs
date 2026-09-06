//! First bounded count-writer review, using public execution and paired report APIs.
use ess_conformance::{AdmittedSuite, CountReport, CountRun, Runner};
use serde_json::{json, Value};

fn document() -> Value {
    json!({
        "provenance":{"suite_version":"ess-conformance/4", "system":"review",
            "specification_version":"v1", "spec_digest":"a".repeat(64), "contract_digest":"a".repeat(64)},
        "scenarios":{"review.count/authored/one":{"purpose":"An exact original suite", "steps":[], "source":[]}}
    })
}

fn execute(suite: &AdmittedSuite) -> ess_conformance::ConformanceReport {
    Runner::for_suite(suite.suite())
        .run_admitted(suite, &ess_conformance::reference::Billing::new())
}

#[test]
fn a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes() {
    let original = serde_json::to_string_pretty(&document()).unwrap() + "\n";
    let executed = AdmittedSuite::from_json(&original).unwrap();
    let run = execute(&executed);
    assert!(run.is_conformant());
    assert!(CountReport::from_run(&run, &executed).is_ok());
    assert!(CountRun::from_run(&run, &executed).is_ok());

    let mut changed_steps = document();
    changed_steps["scenarios"]["review.count/authored/one"]["steps"] =
        json!([{"step":"expect_event", "event":"review.count.Missing"}]);
    let mut accepted = Vec::new();
    for (label, other_bytes) in [
        ("different final newline", format!("{original}\n")),
        ("different executed assertion", changed_steps.to_string()),
    ] {
        let other = AdmittedSuite::from_json(&other_bytes).unwrap();
        assert_ne!(executed.digest(), other.digest());
        assert_eq!(executed.suite().provenance, other.suite().provenance);
        assert_eq!(
            executed.suite().scenarios.keys().collect::<Vec<_>>(),
            other.suite().scenarios.keys().collect::<Vec<_>>()
        );
        if label == "different executed assertion" {
            assert!(
                !execute(&other).is_conformant(),
                "the changed suite actually fails"
            );
        }
        if let Ok(report) = CountReport::from_run(&run, &other) {
            let text = report.to_canonical_json().unwrap();
            assert!(CountReport::from_json(&text, &other).is_ok());
            accepted.push(format!(
                "standalone {label}: executed={} claimed={}\n{text}",
                executed.digest(),
                other.digest()
            ));
        }
        if let Ok(detailed) = CountRun::from_run(&run, &other) {
            let text = detailed.to_canonical_json().unwrap();
            assert!(CountRun::from_json(&text, &other).is_ok());
            accepted.push(format!(
                "detailed {label}: executed={} claimed={}\n{text}",
                executed.digest(),
                other.digest()
            ));
        }
    }
    assert!(
        accepted.is_empty(),
        "a paired writer must retain the identity actually issued to Runner::run_admitted:\n{}",
        accepted.join("\n")
    );
}
