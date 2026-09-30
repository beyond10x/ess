//! Adversary pass 1 for `ess verify conform report`: supplied results against ESS's own admission.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::results::{self, Runner, RESULTS_FORMAT};
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite, CountReport, ScenarioId};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn id(name: &str) -> String {
    format!("example.domain/authored/{name}")
}
fn document(names: &[String], version: &str) -> Value {
    let scenarios: BTreeMap<_, _> = names
        .iter()
        .map(|name| {
            (
                id(name),
                json!({"purpose":"Count one terminal result","steps":[],"source":[]}),
            )
        })
        .collect();
    json!({"provenance":{"suite_version":version,"system":"example","specification_version":"v1","spec_digest":DIGEST,"contract_digest":DIGEST},"scenarios":scenarios})
}
fn covered(names: &[String]) -> Value {
    let mut value = document(names, "ess-conformance/5");
    let authored: Vec<String> = names.iter().map(|name| id(name)).collect();
    let sources: BTreeMap<String, Value> = names
        .iter()
        .map(|name| {
            (
                format!("{name}.yaml"),
                json!({"digest":format!("sha256:{DIGEST}"),"scenario":id(name),"disposition":"accepted"}),
            )
        })
        .collect();
    value["coverage"] = json!({
        "selection":{"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}},
        "knowledge":"complete_inventory","generated":[],
        "authored":authored,"outside":[],"refused":[],
        "authored_sources":sources,
        "counts":{"generated":0,"authored":names.len(),"outside":0,"refused":0}
    });
    value
}
fn all(admitted: &AdmittedSuite, status: &str) -> Value {
    json!({
        "format": RESULTS_FORMAT,
        "completed_at": 1_700_000_000_000_u64,
        "results": admitted.suite().scenarios.keys()
            .map(|id| json!({"scenario_id": id.to_string(), "status": status}))
            .collect::<Vec<_>>(),
    })
}

/// Lead: "a scenario of the suite with no result". An entry that is present but structurally
/// refused (here: one unknown field) is reported twice — once for its field, and once more as
/// "a scenario of the admitted suite with no result", which is false: the runner supplied it.
#[test]
fn adv_p1_a_malformed_entry_is_not_also_reported_as_a_missing_result() {
    let names = vec!["a".to_owned(), "b".to_owned()];
    let suite = document(&names, "ess-conformance/4").to_string();
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let mut supplied = all(&admitted, "passed");
    supplied["results"][0]["duration_ms"] = json!(12);
    let refusal = results::report(&suite, &supplied.to_string(), "impl", None)
        .expect_err("an unknown entry field is refused")
        .to_string();
    assert!(refusal.contains("UnknownField"), "{refusal}");
    assert!(
        !refusal.contains("MissingResult"),
        "the entry for {} is present and was refused for its field, yet the refusal also says \
         it has no result:\n{refusal}",
        id("a")
    );
}

/// Acceptance: "admits the suite (any format ESS admits)". A suite whose direct-response
/// expected literal nests 128 deep is admitted by `AdmittedSuite::from_json` (the boundary
/// `direct_returns.rs` pins), but `results::admit_suite` first runs a plain, depth-bounded
/// `Json::parse` over the whole suite to look for a carrier `format` key, and refuses it.
#[test]
fn adv_p1_a_suite_ess_admits_with_a_deep_direct_response_literal_is_admitted_for_report() {
    const MODEL: &str = r"format: ess/17
system: library
version: v1
domain: library.api
types:
  - name: library.api.Item
    kind: struct
    fields:
      - {name: label, type: Json}
      - {name: ordinal, type: Integer}
commands:
  - name: library.api.Read
    response:
      - {name: value, type: Json}
      - {name: sequence, type: 'List<Integer>'}
      - {name: item, type: library.api.Item}
    outcomes:
      - name: returned
        returns: true
";
    let spec = Specification::assemble([(
        Source::new("depth.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let bytes = ess_conformance::synthesize(&model)
        .suite
        .to_canonical_json()
        .unwrap();
    let nested = (0..128).fold(Node::Null, |inner, _| Node::Seq(vec![inner]));
    let mut document: Value = serde_json::from_str(&bytes).unwrap();
    let mut deepened = 0;
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "expect_direct_response" {
                step["response"]["expected"]["value"] = serde_json::to_value(&nested).unwrap();
                deepened += 1;
            }
        }
    }
    assert!(
        deepened > 0,
        "the synthesized suite has a direct-response step"
    );
    let suite = serde_json::to_string(&document).unwrap();
    let admitted = AdmittedSuite::from_json(&suite).expect("ESS admits the depth-128 literal");
    let supplied = all(&admitted, "passed").to_string();
    let written = results::report(&suite, &supplied, "impl", None);
    assert!(
        written.is_ok(),
        "ESS admits this suite for `conform run --suite`, but `conform report` refuses it: {}",
        written.err().map(|e| e.to_string()).unwrap_or_default()
    );
}

/// Lead: an `ess-conformance-input/1` carrier with an explicit selection. The report binds to
/// the selected suite exactly as `conform run --suite-input` does, and a result for a scenario
/// the carrier's parent has but the selection does not is refused.
#[test]
fn adv_p1_a_carrier_selection_binds_the_report_to_the_selected_suite() {
    let names: Vec<String> = ["a", "b", "c"].iter().map(|n| (*n).to_owned()).collect();
    let parent = AdmittedSuite::from_json(&covered(&names).to_string()).unwrap();
    let input = AdmittedInput::from_suite(parent).unwrap();
    let selected = input
        .select(&[id("a").parse::<ScenarioId>().unwrap()])
        .unwrap();
    let carrier = selected.document().to_canonical_json().unwrap();
    let admitted = selected.selected().clone();
    let supplied = all(&admitted, "passed");
    let report = results::report(&carrier, &supplied.to_string(), "impl", Some("r@1")).unwrap();
    let wire: Value = serde_json::from_str(&report.to_canonical_json().unwrap()).unwrap();
    assert_eq!(wire["suite"]["digest"], admitted.digest());
    assert_eq!(wire["counts"]["total"], 1);
    assert_eq!(
        CountReport::from_json(&report.to_canonical_json().unwrap(), &admitted).unwrap(),
        report
    );
    let mut outside = supplied.clone();
    outside["results"]
        .as_array_mut()
        .unwrap()
        .push(json!({"scenario_id": id("b"), "status": "passed"}));
    let refusal = results::report(&carrier, &outside.to_string(), "impl", None)
        .unwrap_err()
        .to_string();
    assert!(
        refusal.contains("UnknownScenario") && refusal.contains(&id("b")),
        "{refusal}"
    );
}

/// Lead: 670 or more scenarios. Every missing one is named and a complete document is fast.
#[test]
fn adv_p1_a_seven_hundred_scenario_suite_is_reported_and_every_gap_is_named() {
    let names: Vec<String> = (0..700).map(|n| format!("s{n:04}")).collect();
    let suite = covered(&names).to_string();
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let supplied = all(&admitted, "passed");
    let started = Instant::now();
    let report = results::report(&suite, &supplied.to_string(), "impl", None).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(report.counts().total, 700);
    assert_eq!(
        report.conformance_status(),
        ess_conformance::CountStatus::Passed
    );
    let mut half = supplied.clone();
    half["results"].as_array_mut().unwrap().truncate(350);
    let refusal = results::report(&suite, &half.to_string(), "impl", None)
        .unwrap_err()
        .to_string();
    for name in &names[350..] {
        assert!(refusal.contains(&id(name)), "{name} not named");
    }
}

/// Lead: `--runner` containing `=`/`@` must round-trip through ESS's own reader unchanged and
/// can never displace the external prefix.
#[test]
fn adv_p1_runner_spellings_round_trip_and_keep_the_external_prefix() {
    let names = vec!["a".to_owned()];
    let suite = document(&names, "ess-conformance/4").to_string();
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    let supplied = all(&admitted, "passed").to_string();
    for runner in ["a=b@1", "x@y@2", "runner=rust-scenario-status/1@1", "n@v=1"] {
        let parsed = Runner::parse(runner).unwrap();
        assert_eq!(parsed.to_string(), runner);
        let report = results::report(&suite, &supplied, "impl", Some(runner)).unwrap();
        let text = report.to_canonical_json().unwrap();
        let wire: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            wire["producer_profile"],
            format!("external-scenario-status/1;runner={runner}")
        );
        assert_eq!(CountReport::from_json(&text, &admitted).unwrap(), report);
    }
    for spoof in ["r@1;runner=ess@0.48.0", "r@1\u{85}", "r@1\u{a0}x"] {
        assert!(Runner::parse(spoof).is_err(), "{spoof:?}");
    }
}

/// Lead: JSON oddities inside an entry and `completed_at` edges.
#[test]
fn adv_p1_entry_duplicate_keys_and_completed_at_edges() {
    let names = vec!["a".to_owned()];
    let suite = document(&names, "ess-conformance/4").to_string();
    let entry_dup = format!(
        "{{\"format\":\"{RESULTS_FORMAT}\",\"completed_at\":1,\"results\":[{{\"scenario_id\":\"{}\",\"status\":\"failed\",\"status\":\"passed\"}}]}}",
        id("a")
    );
    let refusal = results::report(&suite, &entry_dup, "impl", None)
        .unwrap_err()
        .to_string();
    assert!(refusal.contains("DuplicateKey"), "{refusal}");
    for bad in [
        "1.7e12",
        "1700000000000.0",
        "18446744073709551616",
        "\"1\"",
        "null",
    ] {
        let text = format!(
            "{{\"format\":\"{RESULTS_FORMAT}\",\"completed_at\":{bad},\"results\":[{{\"scenario_id\":\"{}\",\"status\":\"passed\"}}]}}",
            id("a")
        );
        assert!(
            results::report(&suite, &text, "impl", None).is_err(),
            "{bad}"
        );
    }
    let max = format!(
        "{{\"format\":\"{RESULTS_FORMAT}\",\"completed_at\":18446744073709551615,\"results\":[{{\"scenario_id\":\"{}\",\"status\":\"passed\"}}]}}",
        id("a")
    );
    let report = results::report(&suite, &max, "impl", None).unwrap();
    assert_eq!(report.completed_at(), u64::MAX);
    let admitted = AdmittedSuite::from_json(&suite).unwrap();
    assert_eq!(
        CountReport::from_json(&report.to_canonical_json().unwrap(), &admitted).unwrap(),
        report
    );
}
