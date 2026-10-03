//! Nested response contracts observe independently returned and emitted values.
mod support_go;
mod support_nested_response;
use ess_conformance::{report::Status, Runner};
use support_nested_response::{suite, Backend, Fault, MODEL};

#[test]
fn healthy_nested_response_and_independent_generated_sibling_are_observed() {
    let admitted = suite(MODEL);
    for (fault, expected) in [
        (Fault::None, Status::Passed),
        (Fault::GeneratedSibling, Status::Failed),
    ] {
        let target = Backend::new(fault);
        let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
        assert_eq!(run.scenarios[0].status, expected, "{fault:?}: {run:?}");
        assert_eq!(target.calls.get(), 1);
    }
}

#[test]
fn independent_wrong_nested_event_fails_the_actual_relationship() {
    let admitted = suite(MODEL);
    let target = Backend::new(Fault::Event);
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(run.scenarios[0].status, Status::Failed, "{run:?}");
    assert_eq!(target.calls.get(), 1);
}

#[test]
fn independent_wrong_response_fails_the_actual_nested_relationship() {
    let admitted = suite(MODEL);
    let target = Backend::new(Fault::Response);
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(run.scenarios[0].status, Status::Failed, "{run:?}");
    assert_eq!(target.calls.get(), 1);
}

#[test]
fn actual_go_observes_nested_relationship_and_generated_sibling() {
    let admitted = suite(MODEL);
    for fault in [
        Fault::None,
        Fault::Event,
        Fault::Response,
        Fault::GeneratedSibling,
    ] {
        let verdicts =
            support_go::assert_parity("nested-response", admitted.suite(), Backend::new(fault));
        assert_eq!(
            verdicts.values().all(|value| value == "passed"),
            matches!(fault, Fault::None)
        );
    }
}

#[test]
fn actual_typescript_observes_nested_relationship_and_generated_sibling() {
    let admitted = suite(MODEL);
    for (index, fault) in [
        Fault::None,
        Fault::Event,
        Fault::Response,
        Fault::GeneratedSibling,
    ]
    .into_iter()
    .enumerate()
    {
        let (output, report, target) = support_nested_response::foreign::typescript_run(
            &admitted,
            admitted.original_json(),
            Backend::new(fault),
            &format!("fault-{index}"),
        );
        assert_eq!(
            target.calls.get(),
            1,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.status.success(),
            matches!(fault, Fault::None),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report = report.unwrap();
        assert_eq!(
            report["counts"]["passed"],
            usize::from(matches!(fault, Fault::None))
        );
        assert_eq!(
            report["counts"]["failed"],
            usize::from(!matches!(fault, Fault::None))
        );
    }
}

#[test]
fn actual_foreign_readers_refuse_malformed_authority_before_any_target_callback() {
    let admitted = suite(MODEL);
    for (label, value) in support_nested_response::admission::malformed() {
        let document = value.to_string();
        let (run, target) = support_nested_response::foreign::go_run(
            &admitted,
            &document,
            Backend::new(Fault::None),
            label,
        );
        assert!(!run.success, "{label}: {}", run.log);
        assert_eq!(target.callbacks.get(), 0, "{label}: {}", run.log);
        let (output, _report, target) = support_nested_response::foreign::typescript_run(
            &admitted,
            &document,
            Backend::new(Fault::None),
            label,
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(target.callbacks.get(), 0, "{label}");
    }
}

#[test]
fn actual_frozen_reader_refuses_new_closed_field_in_both_held_suite_versions() {
    let Some(reader) = std::env::var_os("ESS_NESTED_OLD_READER") else {
        return;
    };
    let model = support_nested_response::model(MODEL);
    let covered = ess_conformance::coverage_build::build(
        &model,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap();
    let admitted = suite(MODEL);
    let directory = support_go::directory("nested-old-reader");
    for input in [&admitted, covered.selected()] {
        let major = input.suite().provenance.suite_version.major();
        assert!([34, 35].contains(&major));
        let path = directory.join(format!("nested-{major}.json"));
        std::fs::write(&path, input.original_json()).unwrap();
        let output = std::process::Command::new(&reader)
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let diagnostic = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(diagnostic.contains("nested"), "{diagnostic}");
        std::fs::write(directory.join(format!("nested-{major}.log")), diagnostic).unwrap();
    }
}
