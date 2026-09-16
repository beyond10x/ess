//! A host runner uses the native Go evaluator without manufacturing testing.T.
#[path = "support/live_metrics.rs"]
mod metrics_fixture;

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "Keep shared Rust/Go inputs, subprocess execution and cross-reader assertions together."
)]
fn native_go_session_and_event_counterexamples() {
    let directory = std::env::temp_dir().join(format!("ess-session-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let (coverage, parent) = selection_fixture();
    std::fs::write(directory.join("selection-parent.json"), &coverage).unwrap();
    std::fs::write(directory.join("go.mod"), "module sessionproof\n\ngo 1.24\n").unwrap();
    std::fs::write(
        directory.join("runtime.go"),
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            include_str!("../src/go/runtime.go"),
            include_str!("../src/go/reading.go"),
            include_str!("../src/go/response.go"),
            include_str!("../src/go/temporal.go"),
            include_str!("../src/go/live_trace.go"),
            include_str!("../../../specify/ess-domain/src/reading/coordinate.go")
        ),
    )
    .unwrap();
    std::fs::write(
        directory.join("predicate.go"),
        include_str!("../src/go/predicate.go"),
    )
    .unwrap();
    std::fs::write(
        directory.join("session_test.go"),
        include_str!("fixtures/session-runtime.go"),
    )
    .unwrap();
    std::fs::write(
        directory.join("temporal_test.go"),
        include_str!("fixtures/temporal-runtime.go"),
    )
    .unwrap();
    std::fs::write(
        directory.join("live_binding_test.go"),
        include_str!("fixtures/live-binding-runtime.go"),
    )
    .unwrap();
    std::fs::write(
        directory.join("live-suite.json"),
        include_str!("fixtures/live-binding-suite.json"),
    )
    .unwrap();
    std::fs::write(
        directory.join("live-trace-suite.json"),
        include_str!("fixtures/live-trace-suite.json"),
    )
    .unwrap();
    std::fs::write(
        directory.join("live_trace_test.go"),
        include_str!("fixtures/live-trace-runtime.go"),
    )
    .unwrap();
    let metrics = metrics_fixture::compilation(metrics_fixture::SOURCE).unwrap();
    std::fs::write(
        directory.join("live-metrics-suite.json"),
        metrics.input.selected().original_json(),
    )
    .unwrap();
    std::fs::write(
        directory.join("live-metrics-manifest.json"),
        &metrics.manifest,
    )
    .unwrap();
    std::fs::write(
        directory.join("live-metrics-cases.json"),
        metrics_fixture::cases().to_string(),
    )
    .unwrap();
    std::fs::write(
        directory.join("live_metrics_test.go"),
        include_str!("fixtures/live-metrics-runtime.go"),
    )
    .unwrap();
    let output = std::process::Command::new("go")
        .args(["test", "-race", "-count=1", "."])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let selected = ess_conformance::coverage::AdmittedInput::from_json(
        &std::fs::read_to_string(directory.join("selection-child.json")).unwrap(),
    )
    .unwrap();
    let expected = parent
        .select(&["example.domain/authored/second".parse().unwrap()])
        .unwrap();
    assert_eq!(selected.selected().suite(), expected.selected().suite());
    assert_eq!(
        selected.selected().coverage(),
        expected.selected().coverage()
    );
    assert_eq!(selected.parents()[0].original_json(), coverage);
    let nested = ess_conformance::coverage::AdmittedInput::from_json(
        &std::fs::read_to_string(directory.join("selection-empty.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(nested.parents().len(), 2);
    assert!(nested.selected().suite().is_empty());
    let selected_metrics = ess_conformance::coverage::AdmittedInput::from_json(
        &std::fs::read_to_string(directory.join("live-metrics-selected.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        selected_metrics
            .selected()
            .suite()
            .provenance
            .suite_version
            .major(),
        13
    );
    assert_eq!(
        selected_metrics.parents()[0].original_json(),
        metrics.input.selected().original_json()
    );
    std::fs::remove_dir_all(&directory).unwrap();
}

fn selection_fixture() -> (String, ess_conformance::coverage::AdmittedInput) {
    let mut coverage: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/live-binding-suite.json")).unwrap();
    coverage["provenance"]["suite_version"] = "ess-conformance/11".into();
    let first = coverage["scenarios"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let second = "example.domain/authored/second";
    // Selection must preserve general Node values; the report encoder only
    // handles unsigned counts and is not a suite serializer.
    coverage["scenarios"][&first]["steps"][0]["input"]["selection_probe"] =
        serde_json::json!({"kind":"literal", "value":{"enabled":true,"fraction":-0.5}});
    coverage["scenarios"][second] = coverage["scenarios"][&first].clone();
    let mut ids = vec![first.clone(), second.to_owned()];
    ids.sort();
    coverage["coverage"] = serde_json::json!({
        "selection":{"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}},
        "knowledge":"complete_inventory", "generated":[], "authored":ids,
        "outside":[], "refused":[],
        "authored_sources":{
            "first.yaml":{"digest":format!("sha256:{}", "a".repeat(64)),"scenario":first,"disposition":"accepted"},
            "second.yaml":{"digest":format!("sha256:{}", "b".repeat(64)),"scenario":second,"disposition":"accepted"}},
        "counts":{"generated":0,"authored":2,"outside":0,"refused":0}
    });
    let coverage = serde_json::to_string_pretty(&coverage).unwrap() + "\n";
    let parent = ess_conformance::coverage::AdmittedInput::from_suite(
        ess_conformance::AdmittedSuite::from_json(&coverage).unwrap(),
    )
    .unwrap();
    (coverage, parent)
}
