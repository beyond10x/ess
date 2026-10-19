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

mod support_go;

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
    let honest = run(ess_conformance::interpret::Interpreted::for_model(
        ir.clone(),
    ));
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
    // A target publishing an event the expanded runs say is absent fails the scenarios carrying
    // them (review 2026-10-19, T4): the compact runs are read as checks, not dropped.
    let statuses = |target: &Publishing<'_>| {
        ess_conformance::Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, target)
            .into_report()
            .scenarios
            .into_iter()
            .map(|scenario| (scenario.scenario.to_string(), scenario.status))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let inner = ess_conformance::interpret::Interpreted::for_model(ir);
    let wrapped = statuses(&Publishing {
        inner: &inner,
        extra: None,
    });
    let faulty = statuses(&Publishing {
        inner: &inner,
        extra: Some("keys.api.Audited01"),
    });
    let json = suite.to_canonical_json().unwrap();
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let caught: Vec<&String> = faulty
        .iter()
        .filter(|(id, status)| **status == Status::Failed && wrapped[*id] == Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(!caught.is_empty(), "{faulty:#?}");
    for id in caught {
        assert!(
            document["scenarios"][id.as_str()]
                .get("absent_events")
                .is_some(),
            "{id} fails only through an absent-event run"
        );
    }
}

/// The interpreter, also publishing one occurrence of `extra` beside every command it executes.
struct Publishing<'a> {
    inner: &'a ess_conformance::interpret::Interpreted,
    extra: Option<&'static str>,
}

impl ess_conformance::target::ConformanceTarget for Publishing<'_> {
    fn identity(
        &self,
    ) -> Result<ess_conformance::target::ImplementationIdentity, ess_conformance::target::TargetError>
    {
        self.inner.identity()
    }
    fn begin_scenario(
        &self,
        scenario: &ess_conformance::target::ScenarioContext,
    ) -> Result<(), ess_conformance::target::TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(
        &self,
        scenario: &ess_conformance::target::ScenarioContext,
    ) -> Result<(), ess_conformance::target::TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: ess_conformance::target::SemanticCommandRequest,
    ) -> Result<ess_conformance::target::SemanticCommandResult, ess_conformance::target::TargetError>
    {
        let mut result = self.inner.execute_command(request)?;
        if let Some(extra) = self.extra {
            result.direct_events.push(
                ess_conformance::target::ObservedEvent::new(
                    ess_conformance::scenario::EventRef::new(
                        ess_domain::name::QualifiedName::new(extra).expect("an event name"),
                    ),
                )
                .with(
                    "key_id",
                    ess_primitives::node::Node::Text("00000000-0000-4000-8000-0000000000aa".into()),
                ),
            );
        }
        Ok(result)
    }
    fn query_view(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
    ) -> Result<ess_conformance::target::SemanticViewResult, ess_conformance::target::TargetError>
    {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: ess_conformance::target::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::target::ObservedEvent>, ess_conformance::target::TargetError>
    {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ess_conformance::target::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::target::TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(
        &self,
        request: ess_conformance::target::RedeliveryRequest,
    ) -> Result<(), ess_conformance::target::TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// The ten-event suite as JSON, and the id of its first scenario carrying `key`.
fn carrying(key: &str) -> (serde_json::Value, String) {
    let (_, suite) = synthesized(&ten_events());
    let document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    let id = document["scenarios"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, scenario)| scenario.get(key).is_some())
        .map_or_else(
            || panic!("a scenario carrying {key}: {document:#}"),
            |(id, _)| id.clone(),
        );
    (document, id)
}

/// Every edit of `key` on scenario `id` of `document` that no reader may admit, by name.
fn malformed(
    document: &serde_json::Value,
    id: &str,
    key: &str,
) -> Vec<(&'static str, serde_json::Value)> {
    let events: Vec<serde_json::Value> = document["events"].as_array().unwrap().clone();
    let written = document["scenarios"][id][key].clone();
    let first = written[0].clone();
    let position = if key == "absent_events" {
        "before"
    } else {
        "step"
    };
    let steps = document["scenarios"][id]["steps"].as_array().unwrap().len();
    let edit = |change: &dyn Fn(&mut serde_json::Value)| {
        let mut value = document.clone();
        change(&mut value["scenarios"][id][key]);
        value
    };
    let mut reversed: Vec<serde_json::Value> = events.clone();
    reversed.reverse();
    vec![
        ("null", edit(&|value| *value = serde_json::Value::Null)),
        ("empty", edit(&|value| *value = serde_json::json!([]))),
        (
            "past the steps",
            edit(&|value| value[0][position] = (steps + 1).into()),
        ),
        (
            "keeping no event",
            edit(&|value| value[0]["except"] = events.clone().into()),
        ),
        (
            "out of catalog order",
            edit(&|value| value[0]["except"] = reversed[..2].to_vec().into()),
        ),
        (
            "an event outside the catalog",
            edit(&|value| value[0]["except"] = serde_json::json!(["keys.api.Unknown"])),
        ),
        (
            "descending positions",
            edit(&|value| {
                let mut later = first.clone();
                later[position] = 0.into();
                let mut earlier = first.clone();
                earlier[position] = 1.into();
                *value = serde_json::json!([earlier, later]);
            }),
        ),
    ]
}

/// The suite admission, the plain reader and the generated Go runtime each refuse every malformed
/// absent-event run and unpublished-event list (review 2026-10-19, T4, B13).
#[test]
fn a_malformed_absent_run_or_unpublished_list_is_refused_by_every_reader() {
    let (_, suite) = synthesized(&ten_events());
    let directory = support_go::package(
        "compact-malformed",
        &suite,
        &[support_go::TRANSCRIPT_TARGET],
    );
    let transcript = directory.join("transcript.json");
    std::fs::write(&transcript, "{}").unwrap();
    let mut admitted = Vec::new();
    for key in ["absent_events", "unpublished_events"] {
        let (document, id) = carrying(key);
        for (label, value) in malformed(&document, &id, key) {
            let text = value.to_string();
            if AdmittedSuite::from_json(&text).is_ok() {
                admitted.push(format!("{key} {label}: admitted"));
            }
            if ConformanceSuite::from_json(&text).is_ok() {
                admitted.push(format!("{key} {label}: read"));
            }
            support_go::rewrite_suite(&directory, |document| *document = value.clone());
            let go = support_go::go_test(
                &directory,
                "TestTranscriptReplay",
                &[
                    ("ESS_TRANSCRIPT", transcript.to_str().unwrap()),
                    (
                        "ESS_TRANSCRIPT_DIVERGENCE",
                        directory.join("divergence.txt").to_str().unwrap(),
                    ),
                ],
            );
            if go.success || !go.outcomes.is_empty() {
                admitted.push(format!("{key} {label}: admitted by Go\n{}", go.log));
            }
        }
    }
    assert!(admitted.is_empty(), "{admitted:#?}");
}

/// A suite-level catalog written `null` is not the catalog left out (review 2026-10-19, B13), as
/// the Go and TypeScript runtimes already read it.
#[test]
fn a_null_suite_catalog_is_refused() {
    let (document, _) = carrying("absent_events");
    for key in ["events", "one_time_response"] {
        let mut value = document.clone();
        value[key] = serde_json::Value::Null;
        let text = value.to_string();
        assert!(AdmittedSuite::from_json(&text).is_err(), "{key}: admitted");
        assert!(ConformanceSuite::from_json(&text).is_err(), "{key}: read");
    }
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
