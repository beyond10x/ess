//! A one-time suite states its event catalog and response authority once (beyond10x/ess#570).
//!
//! The minimal model of beyond10x/ess#550 and the same model declaring ten events synthesize
//! suites whose scenarios are the same size byte for byte: the catalog, the response authority and
//! the scan after each operation are written once at the suite root from suite/52, and every
//! reader expands them back into the complete trace the writer held.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{report::Status, AdmittedSuite, ConformanceSuite};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::fmt::Write as _;

const WITH_CALLER: &str = include_str!("fixtures/one-time-caller/with-caller.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The minimal model with `count - 1` further declared events, none of which any outcome emits.
fn with_events(count: usize) -> String {
    let mut text = WITH_CALLER.to_owned();
    for index in 1..count {
        write!(
            text,
            "  - name: keys.api.Audited{index:02}\n    fields:\n      - {{name: key_id, type: Uuid}}\n"
        )
        .unwrap();
    }
    text
}

fn ten_events() -> String {
    with_events(10)
}

fn synthesized(text: &str) -> (EssIr, ConformanceSuite) {
    let ir = model(text);
    let mut synthesis = ess_conformance::synthesize(&ir);
    synthesis.suite.select_fresh_format_for(&ir);
    (ir, synthesis.suite)
}

fn scenario_sizes(json: &str) -> std::collections::BTreeMap<String, usize> {
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    value["scenarios"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(id, scenario)| (id.clone(), serde_json::to_string(scenario).unwrap().len()))
        .collect()
}

#[test]
fn suite_states_event_catalog_once() {
    let suites: Vec<_> = [1, 10, 40]
        .into_iter()
        .map(|count| synthesized(&with_events(count)).1)
        .collect();
    for suite in &suites {
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/52"
        );
        assert!(suite
            .scenarios
            .values()
            .any(|scenario| scenario.one_time_response.is_some()));
    }
    let json: Vec<_> = suites
        .iter()
        .map(|suite| suite.to_canonical_json().unwrap())
        .collect();
    let sizes: Vec<_> = json.iter().map(|json| scenario_sizes(json)).collect();
    let (minimal, ten, forty) = (&sizes[0], &sizes[1], &sizes[2]);
    assert_eq!(
        minimal.keys().collect::<Vec<_>>(),
        ten.keys().collect::<Vec<_>>(),
        "declaring events nothing emits adds no scenario"
    );
    assert_eq!(
        ten, forty,
        "a scenario grows with the number of declared events"
    );
    for (id, size) in minimal {
        // Beside the minimal model's scenario, one with events it does not publish names, once, the
        // event it leaves out of each run of absent events.
        assert!(
            ten[id] <= size + 4 * 80,
            "{id}: {size} bytes with one event, {} with ten",
            ten[id]
        );
    }
    for (json, count) in json.iter().zip([1, 10, 40]) {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(
            value["events"].as_array().unwrap().len(),
            count,
            "the suite names every declared event once"
        );
        assert_eq!(json.matches("\"event_windows\"").count(), 0);
        assert_eq!(json.matches("\"expect_no_event\"").count(), 0);
    }
}

#[test]
fn every_reader_gets_back_the_trace_the_writer_held() {
    let (_, suite) = synthesized(&ten_events());
    for json in [
        suite.to_canonical_json().unwrap(),
        suite.to_compact_json().unwrap(),
    ] {
        assert_eq!(ConformanceSuite::from_json(&json).unwrap(), suite);
        assert_eq!(AdmittedSuite::from_json(&json).unwrap().suite(), &suite);
    }
}

#[test]
fn the_compact_suite_passes_and_fails_as_the_complete_one_did() {
    let (ir, suite) = synthesized(&ten_events());
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let run = |target: ess_conformance::interpret::Interpreted| {
        ess_conformance::Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &target)
            .into_report()
    };
    let honest = run(ess_conformance::interpret::Interpreted::for_model(ir));
    assert!(
        honest
            .scenarios
            .iter()
            .all(|scenario| scenario.status == Status::Passed),
        "{:?}",
        honest
            .scenarios
            .iter()
            .filter(|scenario| scenario.status != Status::Passed)
            .map(|scenario| (scenario.scenario.to_string(), scenario.status))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_catalog_below_suite_52_and_a_trace_without_one_are_refused() {
    let (_, suite) = synthesized(WITH_CALLER);
    let json = suite.to_canonical_json().unwrap();
    let older = json.replace("\"ess-conformance/52\"", "\"ess-conformance/50\"");
    AdmittedSuite::from_json(&older).expect_err("a catalog requires suite/52");
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value.as_object_mut().unwrap().remove("one_time_response");
    AdmittedSuite::from_json(&value.to_string()).expect_err("a compact trace requires the catalog");
}
