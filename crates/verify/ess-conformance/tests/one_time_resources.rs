//! Exact, shared disclosure payload resource boundaries with live command/read callbacks.
#[path = "support_one_time/resources.rs"]
mod resources;
#[allow(dead_code)]
mod support_one_time;
// Resource fixture uses the same shared live service through these parent imports.
use ess_conformance::{report::Status, Runner};
use resources::{ResourceMode, ResourceService};
use support_one_time::{Mode, Service};

fn check(mode: ResourceMode) {
    let input = support_one_time::admitted(Mode::View);
    let target = ResourceService::new(mode);
    let report = Runner::for_suite(input.suite()).run_admitted(&input, &target);
    assert_eq!(report.scenarios.len(), 1);
    assert_eq!(report.scenarios[0].status, mode.expected(), "{mode:?}");
    assert!(target.inner.trace().contains(&"query_view"));
    let counts = ess_conformance::counts::CountReport::from_run(&report, &input).unwrap();
    let serialized = serde_json::to_string(&report.scenarios).unwrap();
    let count_bytes = counts.to_canonical_json().unwrap();
    for secret in target.inner.returned_plaintexts() {
        assert!(!serialized.contains(&secret));
        assert!(!count_bytes.contains(&secret));
    }
    assert!(serialized.contains(if mode.expected() == Status::Passed {
        "ESS-CF-DISCLOSURE"
    } else {
        "ESS-CF-TARGET"
    }));
}
#[test]
fn compact_json_bytes_exact() {
    check(ResourceMode::BytesExact);
}
#[test]
fn compact_json_bytes_over() {
    check(ResourceMode::BytesOver);
}
#[test]
fn escaped_json_bytes_exact() {
    check(ResourceMode::EscapesExact);
}
#[test]
fn escaped_json_bytes_over() {
    check(ResourceMode::EscapesOver);
}
#[test]
fn numeric_json_bytes_over() {
    check(ResourceMode::NumbersOver);
}
#[test]
fn members_exact() {
    check(ResourceMode::MembersExact);
}
#[test]
fn members_over() {
    check(ResourceMode::MembersOver);
}
#[test]
fn value_depth_exact() {
    check(ResourceMode::DepthExact);
}
#[test]
fn value_depth_over() {
    check(ResourceMode::DepthOver);
}

#[test]
fn shared_resource_manifest_matches_actual_counts_and_encoding() {
    let mut manifest = Vec::new();
    for mode in ResourceMode::ALL {
        let input = support_one_time::admitted(Mode::View);
        let target = ResourceService::new(mode);
        let report = Runner::for_suite(input.suite()).run_admitted(&input, &target);
        assert_eq!(report.scenarios[0].status, mode.expected());
        let counts = ess_conformance::counts::CountReport::from_run(&report, &input).unwrap();
        let actual_counts = serde_json::to_value(counts.counts()).unwrap();
        assert_eq!(
            actual_counts,
            serde_json::json!({"total":1,"passed":usize::from(mode.expected()==Status::Passed),"failed":0,"skipped":0,"unsupported":usize::from(mode.expected()==Status::Unsupported),"error":0})
        );
        manifest.push(serde_json::json!({"case":mode,"status":mode.expected(),"counts":actual_counts,"canonical_payload_bytes":serde_json::to_vec(&mode.rows()).unwrap().len(),"callback_trace":target.inner.trace()}));
    }
    let bytes = format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap());
    if let Some(path) = std::env::var_os("ESS_ONE_TIME_RESOURCE_MANIFEST_OUT") {
        std::fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(
            std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/one-time-resources.json")
            )
            .unwrap(),
            bytes
        );
    }
}
