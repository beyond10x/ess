//! A runner's own per-scenario results become report/2 over ESS's admission of the suite.
use ess_conformance::results::{self, ExternalResults, Runner, RESULTS_FORMAT};
use ess_conformance::{AdmittedSuite, CountReport, CountStatus};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn document(names: &[&str]) -> Value {
    let scenarios: BTreeMap<_, _> = names
        .iter()
        .map(|name| {
            (
                format!("example.domain/authored/{name}"),
                json!({"purpose":"Count one terminal result","steps":[],"source":[]}),
            )
        })
        .collect();
    json!({"provenance":{"suite_version":"ess-conformance/4","system":"example","specification_version":"v1","spec_digest":DIGEST,"contract_digest":DIGEST},"scenarios":scenarios})
}
fn original(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap() + "\n"
}
fn id(name: &str) -> String {
    format!("example.domain/authored/{name}")
}
fn results(entries: &[(&str, &str)]) -> Value {
    json!({
        "format": RESULTS_FORMAT,
        "completed_at": 1_700_000_000_000_u64,
        "results": entries
            .iter()
            .map(|(name, status)| json!({"scenario_id": id(name), "status": status}))
            .collect::<Vec<_>>(),
    })
}
fn report(suite: &str, results: &Value, runner: Option<&str>) -> Result<CountReport, String> {
    results::report(suite, &results.to_string(), "downstream-impl 2.0", runner)
        .map_err(|error| error.to_string())
}
fn refusal(suite: &str, results: &Value) -> String {
    report(suite, results, None).expect_err("refused")
}

#[test]
fn supplied_results_become_a_report_two_bound_to_the_admitted_suite() {
    let suite = original(&document(&["a", "b", "c", "d"]));
    let supplied = results(&[
        ("d", "unsupported"),
        ("a", "passed"),
        ("c", "error"),
        ("b", "failed"),
    ]);
    let report = report(&suite, &supplied, Some("acme-runner@1.4.0")).unwrap();
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let encoded = report.to_canonical_json().unwrap();
    let wire: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(wire["format"], "ess-conformance-report/2");
    assert_eq!(
        wire["producer_profile"],
        "external-scenario-status/1;runner=acme-runner@1.4.0"
    );
    assert_eq!(wire["implementation"], "downstream-impl 2.0");
    assert_eq!(wire["specification"], "example/v1");
    assert_eq!(wire["spec_digest"], DIGEST);
    assert_eq!(
        wire["suite"],
        json!({"version":"ess-conformance/4","digest_profile":"sha256-json-bytes/1","digest":admitted.digest()})
    );
    assert_eq!(wire["policy"], "complete-selection/1");
    assert_eq!(wire["coverage"], json!({"knowledge":"unknown"}));
    assert_eq!(wire["completed_at"], 1_700_000_000_000_u64);
    assert_eq!(
        wire["counts"],
        json!({"total":4,"passed":1,"failed":1,"error":1,"unsupported":1,"skipped":0})
    );
    assert_eq!(
        wire["outcomes"],
        json!({"passed":[id("a")],"failed":[id("b")],"error":[id("c")],"unsupported":[id("d")],"skipped":[]})
    );
    assert_eq!(report.execution_status(), CountStatus::Failed);
    assert_eq!(report.conformance_status(), CountStatus::Failed);
    assert_eq!(CountReport::from_json(&encoded, &admitted).unwrap(), report);
}

#[test]
fn without_a_runner_the_profile_still_says_the_results_were_supplied() {
    let suite = original(&document(&["a"]));
    let report = report(&suite, &results(&[("a", "passed")]), None).unwrap();
    let wire: Value = serde_json::from_str(&report.to_canonical_json().unwrap()).unwrap();
    assert_eq!(wire["producer_profile"], "external-scenario-status/1");
    assert_eq!(report.execution_status(), CountStatus::Passed);
    assert_eq!(report.conformance_status(), CountStatus::Inconclusive);
}

#[test]
fn complete_declared_coverage_from_the_suite_qualifies_supplied_passes() {
    let mut value = document(&["passed"]);
    value["provenance"]["suite_version"] = json!("ess-conformance/5");
    value["coverage"] = json!({
        "selection":{"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}},
        "knowledge":"complete_inventory","generated":[],
        "authored":[id("passed")],"outside":[],"refused":[],
        "authored_sources":{"passed.yaml":{"digest":format!("sha256:{DIGEST}"),
          "scenario":id("passed"),"disposition":"accepted"}},
        "counts":{"generated":0,"authored":1,"outside":0,"refused":0}
    });
    let suite = value.to_string();
    let report = report(&suite, &results(&[("passed", "passed")]), None).unwrap();
    let wire: Value = serde_json::from_str(&report.to_canonical_json().unwrap()).unwrap();
    for key in ["knowledge", "selection", "counts", "refused"] {
        assert_eq!(wire["coverage"][key], value["coverage"][key], "{key}");
    }
    assert_eq!(report.conformance_status(), CountStatus::Passed);
}

#[test]
fn a_message_is_admitted_and_a_matching_suite_digest_is_accepted() {
    let suite = original(&document(&["a"]));
    let digest = AdmittedSuite::from_json(&suite)
        .unwrap()
        .digest()
        .to_owned();
    let mut supplied = results(&[("a", "failed")]);
    supplied["results"][0]["message"] = json!("expected accepted, got rejected");
    supplied["suite_digest"] = json!(digest);
    assert!(report(&suite, &supplied, None).is_ok());
}

#[test]
fn a_result_for_a_scenario_the_suite_does_not_contain_is_refused() {
    let suite = original(&document(&["a"]));
    let message = refusal(&suite, &results(&[("a", "passed"), ("ghost", "passed")]));
    assert!(message.contains("UnknownScenario"), "{message}");
    assert!(message.contains(&id("ghost")), "{message}");
}

#[test]
fn a_scenario_of_the_suite_with_no_result_is_refused() {
    let suite = original(&document(&["a", "b"]));
    let message = refusal(&suite, &results(&[("a", "passed")]));
    assert!(message.contains("MissingResult"), "{message}");
    assert!(message.contains(&id("b")), "{message}");
}

#[test]
fn duplicate_results_are_refused_even_when_they_agree() {
    let suite = original(&document(&["a"]));
    let message = refusal(&suite, &results(&[("a", "passed"), ("a", "passed")]));
    assert!(message.contains("DuplicateResult"), "{message}");
    assert!(message.contains(&id("a")), "{message}");
}

#[test]
fn an_unknown_status_is_refused() {
    let suite = original(&document(&["a"]));
    for status in ["skipped", "PASSED", "inconclusive", ""] {
        let message = refusal(&suite, &results(&[("a", status)]));
        assert!(message.contains("UnknownStatus"), "{status}: {message}");
    }
}

#[test]
fn results_for_another_suite_digest_are_refused() {
    let suite = original(&document(&["a"]));
    let mut supplied = results(&[("a", "passed")]);
    supplied["suite_digest"] = json!(format!("sha256:{DIGEST}"));
    let message = refusal(&suite, &supplied);
    assert!(message.contains("SuiteDigestMismatch"), "{message}");
    assert!(message.contains(&format!("sha256:{DIGEST}")), "{message}");
}

#[test]
fn the_results_document_is_closed_and_versioned() {
    let suite = original(&document(&["a"]));
    let mut other = results(&[("a", "passed")]);
    other["format"] = json!("ess-conformance-results/2");
    assert!(refusal(&suite, &other).contains("UnsupportedResultsFormat"));
    let mut extra = results(&[("a", "passed")]);
    extra["verdict"] = json!("passed");
    assert!(refusal(&suite, &extra).contains("UnknownField"));
    let mut entry = results(&[("a", "passed")]);
    entry["results"][0]["duration_ms"] = json!(1);
    assert!(refusal(&suite, &entry).contains("UnknownField"));
    let mut missing = results(&[("a", "passed")]);
    missing.as_object_mut().unwrap().remove("completed_at");
    assert!(refusal(&suite, &missing).contains("MissingField"));
    let mut negative = results(&[("a", "passed")]);
    negative["completed_at"] = json!(-1);
    assert!(report(&suite, &negative, None).is_err());
    let duplicate_key = format!(
        "{{\"format\":\"{RESULTS_FORMAT}\",\"completed_at\":0,\"completed_at\":1,\"results\":[]}}"
    );
    assert!(results::report(&suite, &duplicate_key, "impl", None).is_err());
}

#[test]
fn every_refused_result_is_named_in_one_refusal() {
    let suite = original(&document(&["a", "b", "c"]));
    let message = refusal(
        &suite,
        &results(&[
            ("a", "passed"),
            ("a", "failed"),
            ("x", "passed"),
            ("b", "maybe"),
        ]),
    );
    for reason in [
        "DuplicateResult",
        "UnknownScenario",
        "UnknownStatus",
        "MissingResult",
    ] {
        assert!(message.contains(reason), "{reason}: {message}");
    }
}

#[test]
fn the_runner_and_implementation_names_are_checked() {
    for bad in [
        "",
        "runner",
        "@1.0",
        "runner@",
        "run ner@1",
        "r@1;x",
        "r@1\n",
    ] {
        assert!(Runner::parse(bad).is_err(), "{bad:?}");
    }
    let scoped = Runner::parse("@scope/runner@1.0.0-rc.1+build").unwrap();
    assert_eq!(scoped.name(), "@scope/runner");
    assert_eq!(scoped.version(), "1.0.0-rc.1+build");
    let suite = original(&document(&["a"]));
    let supplied = results(&[("a", "passed")]).to_string();
    for bad in ["", "   ", "impl\u{1}"] {
        assert!(
            results::report(&suite, &supplied, bad, None).is_err(),
            "{bad:?}"
        );
    }
    assert!(results::report(&suite, &supplied, "impl", Some("no-version")).is_err());
}

#[test]
fn parsed_results_expose_their_completion_and_entries() {
    let suite = original(&document(&["a"]));
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let parsed =
        ExternalResults::from_json(&results(&[("a", "passed")]).to_string(), &admitted).unwrap();
    assert_eq!(parsed.completed_at(), 1_700_000_000_000);
    assert_eq!(parsed.results().len(), 1);
}

#[test]
fn ess_own_reader_refuses_a_malformed_external_profile() {
    let suite = original(&document(&["a"]));
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let good = report(&suite, &results(&[("a", "passed")]), Some("r@1"))
        .unwrap()
        .to_canonical_json()
        .unwrap();
    for bad in [
        "external-scenario-status/1;runner=",
        "external-scenario-status/1;runner=r",
        "external-scenario-status/1;version=1",
        "external-scenario-status/2",
    ] {
        let mut value: Value = serde_json::from_str(&good).unwrap();
        value["producer_profile"] = json!(bad);
        assert!(
            CountReport::from_json(&value.to_string(), &admitted).is_err(),
            "{bad}"
        );
    }
    let mut skipped: Value = serde_json::from_str(&good).unwrap();
    skipped["counts"] =
        json!({"total":1,"passed":0,"failed":0,"error":0,"unsupported":0,"skipped":1});
    skipped["outcomes"] =
        json!({"passed":[],"failed":[],"error":[],"unsupported":[],"skipped":[id("a")]});
    skipped["execution_status"] = json!("inconclusive");
    skipped["conformance_status"] = json!("inconclusive");
    assert!(CountReport::from_json(&skipped.to_string(), &admitted).is_err());
}

#[test]
fn a_released_suite_twenty_six_is_admitted_and_reported() {
    let suite = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/released-0.38/suite26.json"
    ))
    .unwrap();
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let supplied = json!({
        "format": RESULTS_FORMAT,
        "completed_at": 5,
        "suite_digest": admitted.digest(),
        "results": admitted.suite().scenarios.keys()
            .map(|id| json!({"scenario_id": id.to_string(), "status": "passed"}))
            .collect::<Vec<_>>(),
    });
    let report = report(&suite, &supplied, Some("r@1")).unwrap();
    let wire: Value = serde_json::from_str(&report.to_canonical_json().unwrap()).unwrap();
    assert_eq!(wire["suite"]["version"], "ess-conformance/26");
    assert_eq!(wire["counts"]["passed"], 3);
    assert_eq!(report.execution_status(), CountStatus::Passed);
}

#[test]
fn a_library_consumer_reads_the_producer_profile_and_the_runner() {
    use ess_conformance::counts::ProducerProfile;
    let suite = original(&document(&["a"]));
    let supplied = results(&[("a", "passed")]);
    let named = report(&suite, &supplied, Some("acme-runner@1.4.0")).unwrap();
    assert_eq!(named.producer_profile(), ProducerProfile::External);
    assert_eq!(
        named.runner().map(ToString::to_string).as_deref(),
        Some("acme-runner@1.4.0")
    );
    let unnamed = report(&suite, &supplied, None).unwrap();
    assert_eq!(unnamed.producer_profile(), ProducerProfile::External);
    assert!(unnamed.runner().is_none());
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let read = CountReport::from_json(&named.to_canonical_json().unwrap(), &admitted).unwrap();
    assert_eq!(read.producer_profile(), ProducerProfile::External);
    assert_eq!(read.runner().map(Runner::name), Some("acme-runner"));
    let mut own: Value = serde_json::from_str(&named.to_canonical_json().unwrap()).unwrap();
    own["producer_profile"] = json!("rust-scenario-status/1");
    let own = CountReport::from_json(&own.to_string(), &admitted).unwrap();
    assert_eq!(own.producer_profile(), ProducerProfile::Rust);
    assert!(own.runner().is_none());
}
