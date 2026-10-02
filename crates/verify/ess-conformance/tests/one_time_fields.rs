//! Stateful multi-field origin exemption controls shared across all runtime ports.
#[allow(dead_code)]
mod support_one_time;
use support_one_time::{Mode, Service, FIRST};
#[path = "support_one_time/fields.rs"]
mod fields;
use ess_conformance::{report::Status, Runner};
use fields::{FieldMode, FieldService};

#[test]
fn multiple_field_exemption_controls_match_actual_reports() {
    let input = fields::admitted();
    let mut manifest = Vec::new();
    for mode in FieldMode::ALL {
        let target = FieldService::new(mode);
        let report = Runner::for_suite(input.suite()).run_admitted(&input, &target);
        assert_eq!(report.scenarios[0].status, mode.expected(), "{mode:?}");
        let counts = ess_conformance::counts::CountReport::from_run(&report, &input).unwrap();
        let actual = serde_json::to_value(counts.counts()).unwrap();
        assert_eq!(
            actual,
            serde_json::json!({"total":1,"passed":usize::from(mode.expected()==Status::Passed),"failed":usize::from(mode.expected()==Status::Failed),"skipped":0,"unsupported":0,"error":0})
        );
        let evidence = serde_json::to_string(&report.scenarios).unwrap();
        let count_bytes = counts.to_canonical_json().unwrap();
        assert!(evidence.contains("ESS-CF-DISCLOSURE"));
        for secret in target
            .returned_plaintexts()
            .into_iter()
            .chain([FIRST.into()])
        {
            assert!(!evidence.contains(&secret));
            assert!(!count_bytes.contains(&secret));
        }
        manifest.push(serde_json::json!({"case":mode,"status":mode.expected(),"required_code":"ESS-CF-DISCLOSURE","counts":actual,"callback_trace":target.inner.trace()}));
    }
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-fields");
    let output = std::env::var_os("ESS_ONE_TIME_FIELDS_OUT").map(std::path::PathBuf::from);
    for (name, bytes) in [
        (
            "suite.json",
            format!("{}\n", serde_json::to_string_pretty(input.suite()).unwrap()),
        ),
        (
            "manifest.json",
            format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
        ),
    ] {
        if let Some(output) = &output {
            std::fs::create_dir_all(output).unwrap();
            std::fs::write(output.join(name), bytes).unwrap();
        } else {
            assert_eq!(std::fs::read_to_string(root.join(name)).unwrap(), bytes);
        }
    }
}
