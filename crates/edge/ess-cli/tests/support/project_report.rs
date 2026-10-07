//! The `ess-conformance-report/2` a project's own runner writes beside an emitted mutation suite.
//!
//! The emitted suites are current formats (ess-conformance/34, #312), which report/1 cannot name,
//! so a project runner answers them with report/2 — the migration the library-side mutation tests
//! made (`crates/verify/ess-conformance/tests/mutation_skipped_baseline.rs`). The report is a
//! deliberate collector input with exact per-scenario categories, not target execution evidence,
//! and it is read back through `CountReport` against the exact suite bytes before it is written.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::json;

/// Writes `dir/report.json` for `dir/suite.json`: every scenario passed except each one
/// `not_passed` names as `"<status> <id>"`, with `<status>` one of `failed`, `error`,
/// `unsupported` or `skipped`.
pub fn fabricate(dir: &Path, not_passed: &[String]) {
    let text = std::fs::read_to_string(dir.join("suite.json")).expect("the emitted suite");
    let admitted = ess_conformance::AdmittedSuite::from_json(&text).expect("the suite is admitted");
    let mut assigned = BTreeMap::new();
    for entry in not_passed {
        let (status, id) = entry.split_once(' ').expect("`<status> <id>`");
        assert!(
            matches!(status, "failed" | "error" | "unsupported" | "skipped"),
            "{entry}"
        );
        assigned.insert(id.to_owned(), status);
    }
    let mut outcomes: BTreeMap<&str, Vec<String>> =
        ["passed", "failed", "error", "unsupported", "skipped"]
            .into_iter()
            .map(|status| (status, Vec::new()))
            .collect();
    for id in admitted.suite().scenarios.keys().map(ToString::to_string) {
        let status = assigned.remove(&id).unwrap_or("passed");
        outcomes.get_mut(status).expect("a known status").push(id);
    }
    assert!(
        assigned.is_empty(),
        "every entry names a scenario of the suite: {assigned:?}"
    );
    let count = |status: &str| outcomes[status].len();
    let execution = if count("failed") > 0 || count("unsupported") > 0 {
        "failed"
    } else if count("error") > 0 || count("skipped") > 0 {
        "inconclusive"
    } else {
        "passed"
    };
    let conformance = if execution == "failed" {
        "failed"
    } else {
        "inconclusive"
    };
    let provenance = &admitted.suite().provenance;
    let report = json!({
        "format": "ess-conformance-report/2",
        "specification": format!("{}/{}", provenance.system, provenance.specification_version),
        "spec_digest": provenance.spec_digest,
        "implementation": "project-runner 1.0.0",
        "policy": "complete-selection/1",
        "producer_profile": "go-scenario-status/2",
        "suite": {
            "digest": admitted.digest(),
            "digest_profile": "sha256-json-bytes/1",
            "version": provenance.suite_version.to_string(),
        },
        "coverage": {"knowledge": "unknown"},
        "counts": {
            "total": admitted.suite().scenarios.len(),
            "passed": count("passed"),
            "failed": count("failed"),
            "error": count("error"),
            "unsupported": count("unsupported"),
            "skipped": count("skipped"),
        },
        "outcomes": outcomes,
        "execution_status": execution,
        "conformance_status": conformance,
        "completed_at": 1_700_000_000_000_u64,
    });
    let written = serde_json::to_string_pretty(&report).expect("serializes") + "\n";
    ess_conformance::CountReport::from_json(&written, &admitted).unwrap_or_else(|error| {
        panic!("a coherent report/2 collector fixture: {error}\n{written}")
    });
    std::fs::write(dir.join("report.json"), written).expect("the report is written");
}
