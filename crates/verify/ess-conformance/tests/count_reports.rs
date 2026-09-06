//! Count-stage contracts, original bytes, exact scalars and actual Rust producer fixtures.
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    AdmittedSuite, Clock, CountReport, CountRun, CountStatus, Ids, Runner, RunnerConfig,
};
use ess_primitives::time::Timestamp;
use serde_json::{json, Value};
use sha2::Digest;
use std::{cell::Cell, collections::BTreeMap, fmt::Write, path::Path};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn document(names: &[&str]) -> Value {
    let mut scenarios = BTreeMap::new();
    for name in names {
        let steps = if *name == "failed" {
            json!([
                {"step":"execute_command","command":"example.domain.Execute"},
                {"step":"expect_outcome","outcome":{"command":"example.domain.Execute","outcome":"accepted"}}
            ])
        } else if *name == "mixed-checks" {
            json!([{"step":"mark_instant","instant":"start"}])
        } else {
            json!([])
        };
        scenarios.insert(
            format!("example.domain/authored/{name}"),
            json!({"purpose":"Count one terminal result","steps":steps,"source":[]}),
        );
    }
    json!({"provenance":{"suite_version":"ess-conformance/4","system":"example","specification_version":"v1","spec_digest":DIGEST,"contract_digest":DIGEST},"scenarios":scenarios})
}
fn admitted(names: &[&str]) -> AdmittedSuite {
    AdmittedSuite::from_json(&(serde_json::to_string_pretty(&document(names)).unwrap() + "\n"))
        .unwrap()
}
struct FixedClock(u64);
impl Clock for FixedClock {
    fn now(&mut self) -> Timestamp {
        Timestamp::from_epoch_millis(self.0)
    }
}
struct Target {
    identity_calls: Cell<usize>,
}
impl Target {
    fn new() -> Self {
        Self {
            identity_calls: Cell::new(0),
        }
    }
}
impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.identity_calls.set(self.identity_calls.get() + 1);
        Ok(ImplementationIdentity::new("count-fixture", "1"))
    }
    fn begin_scenario(&self, s: &ScenarioContext) -> Result<(), TargetError> {
        let id = s.scenario.to_string();
        if id.ends_with("/error") {
            Err(TargetError::unavailable("begin", "fixture error"))
        } else if id.ends_with("/unsupported") {
            Err(TargetError::unsupported("begin", "fixture unsupported"))
        } else {
            Ok(())
        }
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        if scenario.scenario.to_string().ends_with("/mixed-checks") {
            Err(TargetError::unavailable(
                "end",
                "error beside unsupported check",
            ))
        } else {
            Ok(())
        }
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Ok(SemanticCommandResult::undeclared())
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        unreachable!()
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        unreachable!()
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        unreachable!()
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        unreachable!()
    }
}
fn execute(suite: &AdmittedSuite, clock: u64) -> ess_conformance::ExecutedRun {
    Runner::new(
        RunnerConfig::default(),
        FixedClock(clock),
        Ids::for_suite(suite.suite()),
    )
    .run_admitted(suite, &Target::new())
}
fn write_fixture(
    name: &str,
    suite: &AdmittedSuite,
    run: &ess_conformance::ExecutedRun,
    expected: &Value,
    fixed_clock: u64,
) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/review-boundaries-8/producer-pairs/rust")
        .join(name);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("suite.json"), suite.original_json()).unwrap();
    std::fs::write(
        directory.join("report.json"),
        CountReport::from_run(run, suite)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("run.json"),
        CountRun::from_run(run, suite)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("legacy-report.json"),
        run.standalone().to_canonical_json(),
    )
    .unwrap();
    let manifest = json!({"producer":"actual Runner::run_admitted + CountReport::from_run","runtime":concat!("ess-conformance/",env!("CARGO_PKG_VERSION")),"command":"cargo test -p ess-conformance --locked --test count_reports actual_rust_producer_pairs -- --nocapture","suite":"suite.json","report":"report.json","detailed":"run.json","legacy":"legacy-report.json","expected_model_digest":DIGEST,"expected_selected_ids":suite.suite().scenarios.keys().map(ToString::to_string).collect::<Vec<_>>(),"expected":expected,"clock":fixed_clock});
    std::fs::write(
        directory.join("fixture.json"),
        serde_json::to_string_pretty(&manifest).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
fn actual_rust_producer_pairs_preserve_categories_precedence_empty_and_high_u64() {
    for (label, names, counts, status) in [
        ("passed", vec!["passed"], [1, 0, 0, 0], CountStatus::Passed),
        ("failed", vec!["failed"], [0, 1, 0, 0], CountStatus::Failed),
        (
            "error",
            vec!["error"],
            [0, 0, 1, 0],
            CountStatus::Inconclusive,
        ),
        (
            "unsupported",
            vec!["unsupported"],
            [0, 0, 0, 1],
            CountStatus::Failed,
        ),
        (
            "mixed",
            vec!["error", "unsupported"],
            [0, 0, 1, 1],
            CountStatus::Failed,
        ),
        (
            "mixed-checks",
            vec!["mixed-checks"],
            [0, 0, 1, 0],
            CountStatus::Inconclusive,
        ),
        ("empty", vec![], [0, 0, 0, 0], CountStatus::Passed),
    ] {
        let suite = admitted(&names);
        let run = execute(&suite, 1_788_680_000_000);
        let report = CountReport::from_run(&run, &suite).unwrap();
        assert_eq!(
            [
                report.counts().passed,
                report.counts().failed,
                report.counts().error,
                report.counts().unsupported
            ],
            counts
        );
        assert_eq!(report.counts().skipped, 0);
        assert_eq!(report.execution_status(), status);
        assert_ne!(report.conformance_status(), CountStatus::Passed);
        let bytes = report.to_canonical_json().unwrap();
        assert_eq!(CountReport::from_json(&bytes, &suite).unwrap(), report);
        let detailed = CountRun::from_run(&run, &suite)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        CountRun::from_json(&detailed, &suite).unwrap();
        write_fixture(
            label,
            &suite,
            &run,
            &json!({"passed":counts[0],"failed":counts[1],"error":counts[2],"unsupported":counts[3],"skipped":0,"total":names.len(),"execution_status":status,"conformance_status":if status==CountStatus::Failed{"failed"}else{"inconclusive"}}),
            1_788_680_000_000,
        );
    }
    for instant in [
        0,
        9_007_199_254_740_993,
        9_223_372_036_854_775_807,
        9_223_372_036_854_775_808,
        u64::MAX,
    ] {
        let suite = admitted(&["passed"]);
        let run = execute(&suite, instant);
        let report = CountReport::from_run(&run, &suite).unwrap();
        let bytes = report.to_canonical_json().unwrap();
        assert_eq!(
            CountReport::from_json(&bytes, &suite)
                .unwrap()
                .completed_at(),
            instant
        );
        let detailed = CountRun::from_run(&run, &suite)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        CountRun::from_json(&detailed, &suite).unwrap();
        write_fixture(
            &format!("clock-{instant}"),
            &suite,
            &run,
            &json!({"passed":1,"failed":0,"error":0,"unsupported":0,"skipped":0,"total":1,"execution_status":"passed","conformance_status":"inconclusive"}),
            instant,
        );
    }
}

#[test]
fn new_scalar_tokens_are_exact_unsigned_in_both_surfaces() {
    let suite = admitted(&["passed"]);
    let run = execute(&suite, 7);
    let report = CountReport::from_run(&run, &suite)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let detailed = CountRun::from_run(&run, &suite)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    for token in [
        "-1",
        "-0",
        "1.0",
        "1e0",
        "0.5",
        "\"1\"",
        "18446744073709551616",
        "+1",
        "01",
    ] {
        for (key, old) in [
            ("completed_at", "7"),
            ("total", "1"),
            ("passed", "1"),
            ("failed", "0"),
            ("error", "0"),
            ("unsupported", "0"),
            ("skipped", "0"),
        ] {
            let bad = report.replace(&format!("\"{key}\": {old}"), &format!("\"{key}\": {token}"));
            assert_ne!(bad, report);
            assert!(
                CountReport::from_json(&bad, &suite).is_err(),
                "{key}={token}"
            );
        }
        for (key, old) in [
            ("started_at", "7"),
            ("completed_at", "7"),
            ("duration_ms", "0"),
        ] {
            let bad =
                detailed.replace(&format!("\"{key}\": {old}"), &format!("\"{key}\": {token}"));
            assert_ne!(bad, detailed);
            assert!(CountRun::from_json(&bad, &suite).is_err(), "{key}={token}");
        }
    }
}

#[test]
fn exact_bytes_profiles_partition_and_identity_cannot_be_guessed() {
    let suite = admitted(&["passed", "error"]);
    let run = execute(&suite, 7);
    let report = CountReport::from_run(&run, &suite)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let value: Value = serde_json::from_str(&report).unwrap();
    for key in [
        "format",
        "producer_profile",
        "policy",
        "execution_status",
        "conformance_status",
    ] {
        let mut bad = value.clone();
        bad[key] = json!("unknown");
        assert!(CountReport::from_json(&bad.to_string(), &suite).is_err());
    }
    let mut mutations = Vec::new();
    let mut bad = value.clone();
    bad["counts"]["total"] = json!(u64::MAX);
    mutations.push(bad);
    let mut bad = value.clone();
    bad["counts"]["failed"] = json!(u64::MAX);
    bad["counts"]["passed"] = json!(u64::MAX);
    mutations.push(bad);
    let mut bad = value.clone();
    bad["outcomes"]["passed"] = json!([
        "example.domain/authored/passed",
        "example.domain/authored/passed"
    ]);
    mutations.push(bad);
    let mut bad = value.clone();
    bad["outcomes"]["error"] = bad["outcomes"]["passed"].clone();
    mutations.push(bad);
    let mut bad = value.clone();
    bad["outcomes"]["passed"] = json!(["example.domain/authored/extra"]);
    mutations.push(bad);
    let mut bad = value.clone();
    bad["outcomes"]["error"] = json!([]);
    mutations.push(bad);
    let mut bad = value.clone();
    bad["producer_profile"] = json!("go-scenario-status/1");
    mutations.push(bad);
    let mut bad = value.clone();
    bad["coverage"]["selection"] = json!({});
    mutations.push(bad);
    let mut bad = value.clone();
    bad["coverage"]["knowledge"] = json!("complete_inventory");
    mutations.push(bad);
    let mut bad = value.clone();
    bad["specification"] = json!("wrong/v1");
    mutations.push(bad);
    let mut bad = value.clone();
    bad["spec_digest"] = json!("b".repeat(64));
    mutations.push(bad);
    let mut bad = value.clone();
    bad["suite"]["version"] = json!("ess-conformance/5");
    mutations.push(bad);
    let mut bad = value.clone();
    bad["suite"]["digest_profile"] = json!("canonical/1");
    mutations.push(bad);
    let mut bad = value.clone();
    bad["alien"] = json!(true);
    mutations.push(bad);
    for bad in mutations {
        assert!(
            CountReport::from_json(&bad.to_string(), &suite).is_err(),
            "{bad}"
        );
    }
    let changed = AdmittedSuite::from_json(&format!("{}\n", suite.original_json())).unwrap();
    assert_ne!(changed.digest(), suite.digest());
    assert!(CountReport::from_json(&report, &changed).is_err());
    let duplicate = report.replacen(
        "\"completed_at\": 7,",
        "\"completed_at\": 7, \"completed_\\u0061t\": 7,",
        1,
    );
    assert!(CountReport::from_json(&duplicate, &suite).is_err());
    assert!(CountReport::from_json(&run.to_canonical_json(), &suite).is_err());
}

#[test]
fn suite_admission_closes_structural_variants_before_target_identity() {
    for version in [
        "ess-conformance/5",
        "ess-conformance/6",
        "ess-conformance/99",
    ] {
        let mut value = document(&["passed"]);
        value["provenance"]["suite_version"] = json!(version);
        assert!(AdmittedSuite::from_json(&value.to_string()).is_err());
        let mut suite = admitted(&["passed"]).suite().clone();
        suite.provenance.suite_version =
            ess_conformance::scenario::SuiteFormat::parse(version).unwrap();
        let target = Target::new();
        assert!(Runner::for_suite(&suite).try_run(&suite, &target).is_err());
        assert_eq!(target.identity_calls.get(), 0);
    }
    for (major, step) in [
        (
            1,
            json!({"step":"expect_view","view":"example.domain.View","expectation":{"expect":"counts","at_least":0}}),
        ),
        (2, json!({"step":"mark_instant","instant":"start"})),
        (
            3,
            json!({"step":"expect_halt","view":"example.domain.View","after":1}),
        ),
    ] {
        let mut value = document(&["passed"]);
        value["provenance"]["suite_version"] = json!(format!("ess-conformance/{major}"));
        value["scenarios"]["example.domain/authored/passed"]["steps"] = json!([step]);
        assert!(AdmittedSuite::from_json(&value.to_string()).is_err());
    }
    let mut variants = Vec::new();
    let mut v = document(&["passed"]);
    v["future"] = json!({});
    variants.push(v);
    let mut v = document(&["passed"]);
    v["provenance"]["future"] = json!(0);
    variants.push(v);
    let mut v = document(&["passed"]);
    v["scenarios"]["example.domain/authored/passed"]["future"] = json!(0);
    variants.push(v);
    for step in [
        json!({"step":"mark_instant","instant":"start","future":1}),
        json!({"step":"execute_command","command":"example.domain.Execute","input":{"x":{"kind":"literal","value":{},"future":1}}}),
        json!({"step":"expect_event","event":"example.domain.Event","shape":{"x":{"holds":"list","future":1}}}),
        json!({"step":"expect_view","view":"example.domain.View","expectation":{"expect":"at","order_by":[],"position":{"row":"first","future":1}}}),
    ] {
        let mut v = document(&["passed"]);
        v["scenarios"]["example.domain/authored/passed"]["steps"] = json!([step]);
        variants.push(v);
    }
    for v in variants {
        assert!(AdmittedSuite::from_json(&v.to_string()).is_err(), "{v}");
    }
    let raw = admitted(&["passed"]).original_json().to_owned();
    let dup = raw.replacen("\"steps\": []", "\"steps\": [], \"st\\u0065ps\": []", 1);
    assert!(AdmittedSuite::from_json(&dup).is_err());
}

#[test]
fn detailed_admission_checks_fields_outcome_order_and_checked_time() {
    let suite = admitted(&["passed", "error"]);
    let run = execute(&suite, 7);
    let text = CountRun::from_run(&run, &suite)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    for route in [
        "summary", "scenario", "check", "root", "time", "order", "duration", "overflow",
    ] {
        let mut bad = value.clone();
        match route {
            "summary" => bad["summary"]["outcomes"]["passed"] = json!([]),
            "scenario" => bad["scenarios"][0]["unknown"] = json!(1),
            "check" => bad["scenarios"][0]["checks"][0]["unknown"] = json!(1),
            "root" => bad["unknown"] = json!(true),
            "time" => bad["started_at"] = json!(8),
            "order" => bad["scenarios"].as_array_mut().unwrap().reverse(),
            "duration" => bad["scenarios"][0]["duration_ms"] = json!(u64::MAX),
            "overflow" => {
                bad["scenarios"][0]["duration_ms"] = json!(u64::MAX);
                bad["scenarios"][1]["duration_ms"] = json!(u64::MAX);
            }
            _ => unreachable!(),
        }
        assert!(
            CountRun::from_json(&bad.to_string(), &suite).is_err(),
            "{route}"
        );
    }
    assert!(CountReport::from_json(&text, &suite).is_err());
}

#[test]
fn legacy_dto_is_not_original_byte_admission_and_typed_execution_still_checks_versions() {
    let mut raw = document(&["passed"]);
    raw["provenance"]["compiler_version"] = json!("historical ignored metadata");
    let original = serde_json::to_string_pretty(&raw).unwrap() + "\n";
    assert!(AdmittedSuite::from_json(&original).is_err());
    let dto = ess_conformance::ConformanceSuite::from_json(&original).unwrap();
    let generic: ess_conformance::ConformanceSuite = serde_json::from_str(&original).unwrap();
    assert_eq!(dto, generic);
    let admitted = AdmittedSuite::from_suite(&dto).unwrap();
    assert_ne!(admitted.original_json(), original);
    assert!(!admitted.original_json().contains("compiler_version"));
    let run = execute(&admitted, 7);
    let report = CountReport::from_run(&run, &admitted).unwrap();
    let report_json: Value = serde_json::from_str(&report.to_canonical_json().unwrap()).unwrap();
    let digest = sha2::Sha256::digest(original.as_bytes()).iter().fold(
        "sha256:".to_owned(),
        |mut text, byte| {
            write!(text, "{byte:02x}").unwrap();
            text
        },
    );
    assert_ne!(report_json["suite"]["digest"], digest);
    raw["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("compiler_version");
    raw["provenance"]["suite_version"] = json!("ess-conformance/1");
    raw["scenarios"]["example.domain/authored/passed"]["steps"] =
        json!([{"step":"mark_instant","instant":"start"}]);
    let dto: ess_conformance::ConformanceSuite = serde_json::from_value(raw).unwrap();
    let target = Target::new();
    assert!(Runner::for_suite(&dto).try_run(&dto, &target).is_err());
    assert_eq!(target.identity_calls.get(), 0);
}
