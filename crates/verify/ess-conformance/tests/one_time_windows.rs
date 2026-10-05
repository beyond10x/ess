//! Shared native callback evidence for multiple independent finite windows per operation.
#[allow(dead_code)]
mod support_one_time;
use ess_conformance::{
    one_time_response::{EventAuthority, EventWindow},
    report::Status,
    AdmittedSuite, Runner,
};
use support_one_time::{Mode, Service};

#[test]
fn multiple_event_windows_share_operation_start_and_complete_each_hold() {
    let mut suite = support_one_time::admitted(Mode::WindowHealthy)
        .suite()
        .clone();
    for scenario in suite.scenarios.values_mut() {
        let policy = scenario.one_time_response.as_mut().unwrap();
        policy.events[0].within_ms = 350;
        let second = "credentials.api.Second".parse().unwrap();
        policy.events.push(EventAuthority {
            event: second,
            within_ms: 650,
        });
        policy.event_windows.clear();
        for event in &policy.events {
            scenario.source.insert(event.event.clone().into());
            for within_ms in [0, event.within_ms] {
                policy.event_windows.push(EventWindow {
                    event: event.event.clone(),
                    after_step: 0,
                    within_ms,
                });
            }
        }
    }
    let input = AdmittedSuite::from_suite(&suite).unwrap();
    let mut manifest = Vec::new();
    for mode in [Mode::WindowHealthy, Mode::DelayedEvent] {
        let target = Service::new(mode);
        let report = Runner::for_suite(input.suite()).run_admitted(&input, &target);
        let (status, code) = mode.expected();
        assert_eq!(report.scenarios[0].status, status);
        let counts = ess_conformance::counts::CountReport::from_run(&report, &input).unwrap();
        let actual = serde_json::to_value(counts.counts()).unwrap();
        assert_eq!(
            actual,
            serde_json::json!({"total":1,"passed":usize::from(status==Status::Passed),"failed":usize::from(status==Status::Failed),"skipped":0,"unsupported":0,"error":0})
        );
        let evidence = serde_json::to_string(&report.scenarios).unwrap();
        assert!(evidence.contains(code));
        let count_bytes = counts.to_canonical_json().unwrap();
        for secret in target
            .returned_plaintexts()
            .into_iter()
            .chain([support_one_time::FIRST.into()])
        {
            assert!(!evidence.contains(&secret));
            assert!(!count_bytes.contains(&secret));
        }
        manifest.push(serde_json::json!({"case":mode,"status":status,"required_code":code,"counts":actual,"callback_trace":target.trace()}));
    }
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-windows");
    let output = std::env::var_os("ESS_ONE_TIME_WINDOWS_OUT").map(std::path::PathBuf::from);
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
