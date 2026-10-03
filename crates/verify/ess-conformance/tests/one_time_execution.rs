//! Native execution of the shared stateful one-time response controls.
mod support_one_time;
use ess_conformance::{report::Status, Runner};
use support_one_time::{admitted, Mode, Service, FIRST};

#[test]
fn shared_live_observer_vectors_match_stateful_callbacks_and_counts() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-execution");
    let output =
        std::env::var_os("ESS_ONE_TIME_EXECUTION_VECTOR_OUT").map(std::path::PathBuf::from);
    let mut manifest = Vec::new();
    for mode in Mode::ALL {
        let admitted = admitted(mode);
        let target = Service::new(mode);
        let (expected, required_code) = mode.expected();
        let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        assert_eq!(report.scenarios.len(), 1);
        assert_eq!(report.scenarios[0].status, expected, "{mode:?}");
        let persisted = serde_json::to_string(&report.scenarios).unwrap();
        assert!(persisted.contains(required_code));
        let count_report =
            ess_conformance::counts::CountReport::from_run(&report, &admitted).unwrap();
        let serialized_counts = count_report.to_canonical_json().unwrap();
        redacted(&target, &[&persisted, &serialized_counts]);
        let expected_counts = serde_json::json!({"passed": usize::from(expected == Status::Passed), "failed": usize::from(expected == Status::Failed), "skipped": 0, "unsupported": usize::from(expected == Status::Unsupported), "error": usize::from(expected == Status::Error)});
        let mut actual_counts = serde_json::to_value(count_report.counts()).unwrap();
        assert_eq!(
            actual_counts.as_object_mut().unwrap().remove("total"),
            Some(serde_json::json!(1))
        );
        assert_eq!(
            actual_counts, expected_counts,
            "{mode:?} actual producer counts"
        );
        let name = serde_json::to_value(mode)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        let suite = format!(
            "{}\n",
            serde_json::to_string_pretty(admitted.suite()).unwrap()
        );
        let item = serde_json::json!({
            "case": name, "suite": format!("{name}.json"), "status": expected,
            "required_code": required_code,
            "counts": actual_counts,
            "callback_trace": target.trace(),
        });
        manifest.push(item);
        vector_bytes(output.as_deref(), &root, &format!("{name}.json"), &suite);
    }
    vector_bytes(
        output.as_deref(),
        &root,
        "manifest.json",
        &format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    );
}

fn vector_bytes(output: Option<&std::path::Path>, root: &std::path::Path, name: &str, bytes: &str) {
    if let Some(output) = output {
        std::fs::create_dir_all(output).unwrap();
        std::fs::write(output.join(name), bytes).unwrap();
    } else {
        assert_eq!(
            std::fs::read_to_string(root.join(name)).unwrap(),
            bytes,
            "{name} shared execution vector drift"
        );
    }
}

fn redacted(target: &Service, persisted: &[&str]) {
    for value in target
        .returned_plaintexts()
        .into_iter()
        .chain([FIRST.to_owned()])
    {
        for bytes in persisted {
            assert!(
                !bytes.contains(&value),
                "observed plaintext entered persisted evidence"
            );
        }
    }
}
fn check(mode: Mode, status: Status, code: Option<&str>) {
    let target = Service::new(mode);
    let admitted = admitted(mode);
    let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(report.scenarios.len(), 1);
    assert_eq!(report.scenarios[0].status, status);
    let counts = ess_conformance::counts::CountReport::from_run(&report, &admitted)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let bytes = serde_json::to_string(&report.scenarios).unwrap();
    redacted(&target, &[&counts, &bytes]);
    if let Some(code) = code {
        assert!(bytes.contains(code), "{bytes}");
    }
    assert!(
        target.calls.get() > 0,
        "the implementation must actually execute"
    );
}

#[test]
fn one_time_fresh_rotation_executes_and_passes() {
    check(Mode::Healthy, Status::Passed, Some("ESS-CF-DISCLOSURE"));
}

#[test]
fn one_time_constrained_origins_retain_their_contract() {
    check(
        Mode::ConstrainedHealthy,
        Status::Passed,
        Some("ESS-CF-DISCLOSURE"),
    );
}

#[test]
fn one_time_identity_error_never_enters_persisted_counts() {
    check(
        Mode::IdentityError,
        Status::Passed,
        Some("ESS-CF-DISCLOSURE"),
    );
}
#[test]
fn one_time_dynamic_keys_are_scanned() {
    check(Mode::KeyLeak, Status::Failed, Some("ESS-CF-DISCLOSURE"));
}
#[test]
fn one_time_declared_error_is_not_a_disclosure_surface() {
    check(
        Mode::DeclaredError,
        Status::Failed,
        Some("ESS-CF-DISCLOSURE"),
    );
}
#[test]
fn one_time_rotation_retains_all_prior_values() {
    check(Mode::LaterReuse, Status::Failed, Some("ESS-CF-DISCLOSURE"));
}
#[test]
fn one_time_capture_bound_is_unsupported_not_eviction() {
    check(
        Mode::CaptureBound,
        Status::Unsupported,
        Some("ESS-CF-TARGET"),
    );
}

#[test]
fn one_time_unobserved_deadline_is_not_a_pass() {
    check(
        Mode::WindowUnsupported,
        Status::Unsupported,
        Some("ESS-CF-TARGET"),
    );
}
#[test]
fn one_time_short_window_is_not_complete_observation() {
    check(
        Mode::WindowShort,
        Status::Unsupported,
        Some("ESS-CF-TARGET"),
    );
}
#[test]
fn one_time_clean_repeated_log_snapshots_are_healthy() {
    check(
        Mode::WindowHealthy,
        Status::Passed,
        Some("ESS-CF-DISCLOSURE"),
    );
}
#[test]
fn one_time_constraint_violation_is_not_an_authorized_origin() {
    check(
        Mode::ConstrainedInvalid,
        Status::Failed,
        Some("ESS-CF-PAYLOAD"),
    );
}

#[test]
fn one_time_view_substrings_are_disclosures() {
    check(Mode::View, Status::Failed, Some("ESS-CF-DISCLOSURE"));
}
#[test]
fn one_time_independent_event_log_is_scanned_through_the_deadline() {
    check(
        Mode::DelayedEvent,
        Status::Failed,
        Some("ESS-CF-DISCLOSURE"),
    );
}
#[test]
fn one_time_retry_cannot_redisclose_the_first_value() {
    check(Mode::Retry, Status::Failed, Some("ESS-CF-DISCLOSURE"));
}
#[test]
fn one_time_direct_event_is_not_an_origin_exemption() {
    check(Mode::DirectEvent, Status::Failed, Some("ESS-CF-DISCLOSURE"));
}
#[test]
fn one_time_required_origin_cannot_pass_vacuously() {
    check(
        Mode::MissingOrigin,
        Status::Failed,
        Some("ESS-CF-DISCLOSURE"),
    );
}
#[test]
fn one_time_empty_origin_is_not_a_capture() {
    check(Mode::Empty, Status::Failed, Some("ESS-CF-PAYLOAD"));
}
#[test]
fn one_time_target_error_is_value_free_before_capture() {
    check(Mode::Error, Status::Error, Some("ESS-CF-TARGET"));
}
#[test]
fn one_time_unsupported_is_value_free_and_not_a_skip() {
    check(
        Mode::Unsupported,
        Status::Unsupported,
        Some("ESS-CF-TARGET"),
    );
}

#[test]
fn one_time_successful_identity_never_enters_persisted_counts() {
    check(
        Mode::IdentitySuccess,
        Status::Passed,
        Some("ESS-CF-DISCLOSURE"),
    );
}
