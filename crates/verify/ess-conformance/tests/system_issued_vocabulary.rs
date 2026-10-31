//! The suite vocabulary of a command the system issues itself (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Suite persistence*): the steps `await_outcome` and
//! `expect_not_issued`, the value `{kind: awaited, field}`, and the majors `/52` (ordinary) and
//! `/53` (coverage) that carry them.
//!
//! What is held here: the Rust reader admits the vocabulary at `/52` and `/53` and refuses it below;
//! a fresh suite carrying it is selected as `/52`, and one without it keeps its number; and the Go
//! and TypeScript emitters, which do not run it yet (stage 2), refuse a `/52` suite by version.

use std::collections::BTreeMap;

use ess_conformance::scenario::{
    CommandRef, ConformanceScenario, EventRef, InstanceName, ScenarioId, ScenarioPurpose,
    ScenarioStep, ScenarioValue, SuiteFormat, SuiteProvenance,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite};
use ess_domain::QualifiedName;
use ess_primitives::evidence::SpecDigest;
use serde_json::{json, Value};

const DIGEST: &str = "abababababababababababababababababababababababababababababababab";

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a valid name")
}

/// One scenario: arrange, await the system's step, and require it not to happen a second time.
fn steps() -> Value {
    json!([
        {"step": "execute_command", "command": "queueing.entry.Enqueue"},
        {"step": "capture_instance", "instance": "entry", "entity": "queueing.entry.Entry",
         "event": "queueing.entry.Enqueued", "field": "entry_id"},
        {"step": "await_outcome", "command": "queueing.entry.Assign",
         "instance": {"kind": "instance", "instance": "entry"},
         "identity": "entry_id",
         "within": "PT5S",
         "input": {"worker": {"kind": "literal", "value": "w-1"}},
         "shape": {"worker": {"holds": "primitive", "kind": "string"}}},
        {"step": "expect_outcome",
         "outcome": {"command": "queueing.entry.Assign", "outcome": "assigned"}},
        {"step": "expect_event_values", "event": "queueing.entry.Assigned",
         "payload": {"worker": {"kind": "awaited", "field": "worker"}}},
        {"step": "expect_not_issued", "command": "queueing.entry.Assign",
         "instance": {"kind": "instance", "instance": "entry"},
         "within": "PT5S",
         "unpublished": ["queueing.entry.Assigned"]}
    ])
}

fn document(major: u32, steps: &Value) -> Value {
    let mut provenance = json!({
        "suite_version": format!("ess-conformance/{major}"),
        "system": "queueing",
        "specification_version": "v1",
        "spec_digest": DIGEST,
        "contract_digest": DIGEST,
    });
    if major >= 34 {
        provenance["scenario_initial_state"] = json!("empty");
    }
    json!({
        "provenance": provenance,
        "scenarios": {
            "queueing.entry/authored/the-system-assigns-once": {
                "purpose": "The system assigns a waiting entry once",
                "steps": steps,
                "source": []
            }
        }
    })
}

fn admit(document: &Value) -> Result<AdmittedSuite, String> {
    AdmittedSuite::from_json(&(serde_json::to_string_pretty(document).unwrap() + "\n"))
        .map_err(|error| error.to_string())
}

#[test]
fn suite_52_admits_the_await_steps_and_the_awaited_value() {
    let admitted = admit(&document(52, &steps())).unwrap_or_else(|error| panic!("{error}"));
    let scenario = admitted
        .suite()
        .scenarios
        .values()
        .next()
        .expect("one scenario");
    let ScenarioStep::AwaitOutcome {
        command,
        instance,
        identity,
        within,
        input,
        shape,
    } = &scenario.steps[2]
    else {
        panic!("{:?}", scenario.steps[2]);
    };
    assert_eq!(command.to_string(), "queueing.entry.Assign");
    assert_eq!(identity, "entry_id");
    assert_eq!(shape.leaves().len(), 1);
    assert_eq!(
        instance,
        &ScenarioValue::instance(InstanceName::new("entry").unwrap())
    );
    assert_eq!(
        within.map(|period| period.to_string()).as_deref(),
        Some("PT5S")
    );
    assert_eq!(
        input.get("worker").and_then(ScenarioValue::as_literal),
        Some(&ess_primitives::node::Node::Text("w-1".to_owned()))
    );
    let ScenarioStep::ExpectNotIssued { unpublished, .. } = &scenario.steps[5] else {
        panic!("{:?}", scenario.steps[5]);
    };
    assert_eq!(unpublished.len(), 1);
}

#[test]
fn the_vocabulary_round_trips_byte_for_byte() {
    let text = serde_json::to_string_pretty(&document(52, &steps())).unwrap() + "\n";
    let suite = ConformanceSuite::from_json(&text).expect("parses");
    let canonical = suite.to_canonical_json().expect("admitted");
    let written: Value = serde_json::from_str(&canonical).unwrap();
    assert_eq!(written, document(52, &steps()), "nothing is lost or added");
    let again = ConformanceSuite::from_json(&canonical).expect("parses again");
    assert_eq!(again.to_canonical_json().expect("admitted"), canonical);
}

#[test]
fn a_major_below_52_refuses_each_construct() {
    for major in [36, 44, 48, 50] {
        let refused = admit(&document(major, &steps())).expect_err("refused below /52");
        assert!(
            refused.contains("UnsupportedVocabulary"),
            "{major}: {refused}"
        );
    }
    // The value alone, in a step older majors know.
    let value_only = json!([
        {"step": "execute_command", "command": "queueing.entry.Enqueue",
         "input": {"worker": {"kind": "awaited", "field": "worker"}}}
    ]);
    let refused = admit(&document(48, &value_only)).expect_err("refused below /52");
    assert!(refused.contains("awaited"), "{refused}");
}

#[test]
fn malformed_await_steps_are_refused() {
    let base = || steps().as_array().unwrap().clone();
    let mut cases: Vec<(&str, Value)> = Vec::new();
    let mut zero = base();
    zero[2]["within"] = json!("PT0S");
    cases.push(("a zero bound", Value::Array(zero)));
    let mut spelled = base();
    spelled[2]["within"] = json!(5);
    cases.push(("a bound that is not PT<seconds>S", Value::Array(spelled)));
    let mut absent = base();
    absent[2].as_object_mut().unwrap().remove("instance");
    cases.push(("no instance", Value::Array(absent)));
    let mut no_identity = base();
    no_identity[2].as_object_mut().unwrap().remove("identity");
    cases.push(("no identity", Value::Array(no_identity)));
    let mut observed = base();
    observed[2]["instance"] = json!({"kind": "awaited", "field": "entry_id"});
    cases.push((
        "an instance that is not captured or literal",
        Value::Array(observed),
    ));
    let mut unknown = base();
    unknown[5]["extra"] = json!(true);
    cases.push(("an unknown key", Value::Array(unknown)));
    let mut no_field = base();
    no_field[4]["payload"]["worker"] = json!({"kind": "awaited"});
    cases.push(("an awaited value with no field", Value::Array(no_field)));
    for (what, steps) in cases {
        assert!(admit(&document(52, &steps)).is_err(), "{what} is admitted");
    }
}

#[test]
fn suite_53_requires_coverage_and_62_is_unsupported() {
    let refused = admit(&document(53, &steps())).expect_err("coverage is required at /53");
    assert!(refused.contains("through /61"), "{refused}");
    for major in [52, 53] {
        assert!(SuiteFormat::parse(&format!("ess-conformance/{major}"))
            .unwrap()
            .is_supported());
    }
    assert!(!SuiteFormat::parse("ess-conformance/62")
        .unwrap()
        .is_supported());
    let refused = admit(&document(62, &json!([]))).expect_err("/62 is unsupported");
    assert!(refused.contains("UnsupportedSuiteVersion"), "{refused}");
}

fn provenance(major: u32) -> SuiteProvenance {
    SuiteProvenance {
        suite_version: SuiteFormat::parse(&format!("ess-conformance/{major}")).unwrap(),
        system: "queueing".to_owned(),
        specification_version: "v1".to_owned(),
        spec_digest: SpecDigest::new(DIGEST).unwrap(),
        contract_digest: SpecDigest::new(DIGEST).unwrap(),
        component: None,
        scenario_initial_state: None,
        synthesis_seeds: None,
    }
}

fn suite_with(steps: Vec<ScenarioStep>) -> ConformanceSuite {
    let mut suite = ConformanceSuite::new(provenance(4));
    suite
        .insert(
            ScenarioId::parse("queueing.entry/authored/the-system-assigns-once").unwrap(),
            ConformanceScenario {
                purpose: ScenarioPurpose::new("The system assigns a waiting entry once").unwrap(),
                steps,
                source: std::collections::BTreeSet::default(),
                one_time_response: None,
            },
        )
        .unwrap();
    suite
}

#[test]
fn a_fresh_suite_awaiting_a_command_is_selected_as_52() {
    let entry = ScenarioValue::instance(InstanceName::new("entry").unwrap());
    for step in [
        ScenarioStep::AwaitOutcome {
            command: CommandRef::new(name("queueing.entry.Assign")),
            instance: entry.clone(),
            identity: "entry_id".to_owned(),
            within: None,
            input: BTreeMap::new(),
            shape: ess_conformance::scenario::PayloadShape::default(),
        },
        ScenarioStep::ExpectNotIssued {
            command: CommandRef::new(name("queueing.entry.Assign")),
            instance: entry.clone(),
            within: None,
            unpublished: vec![EventRef::new(name("queueing.entry.Assigned"))],
        },
        ScenarioStep::ExpectEventValues {
            event: EventRef::new(name("queueing.entry.Assigned")),
            payload: BTreeMap::from([(
                "worker".to_owned(),
                ScenarioValue::Awaited {
                    field: "worker".to_owned(),
                },
            )]),
            shape: ess_conformance::scenario::PayloadShape::default(),
        },
    ] {
        let mut suite = suite_with(vec![step.clone()]);
        suite.select_fresh_format();
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/52",
            "{step:?}"
        );
        assert!(ess_conformance::system_issued::used_by(&suite));
        assert_eq!(
            ess_conformance::system_issued::coverage_floor(&suite),
            Some(53)
        );
    }

    let mut ordinary = suite_with(vec![ScenarioStep::ExpectNoError]);
    ordinary.select_fresh_format();
    assert_eq!(
        ordinary.provenance.suite_version.to_string(),
        "ess-conformance/34",
        "a suite without the vocabulary keeps its number"
    );
    assert!(!ess_conformance::system_issued::used_by(&ordinary));
}

/// Stage 2: the generated Go and TypeScript runners execute the await pair, so both emitters write
/// a package for it, with the run instructions every major from `/8` carries.
#[test]
fn the_go_and_typescript_emitters_admit_the_await_pair() {
    for major in [52, 53] {
        let suite = ConformanceSuite::new(provenance(major));
        let go = ess_conformance::go::emit(&suite)
            .unwrap_or_else(|error| panic!("Go emits /{major}: {error}"));
        let typescript = ess_conformance::ts::emit(&suite)
            .unwrap_or_else(|error| panic!("TypeScript emits /{major}: {error}"));
        for readme in [
            go.iter()
                .find(|artifact| artifact.path.ends_with("README.md"))
                .map(|artifact| &artifact.contents),
            typescript
                .iter()
                .find(|artifact| artifact.path.ends_with("README.md"))
                .map(|artifact| &artifact.contents),
        ] {
            let readme = readme.expect("a README");
            assert!(readme.contains("ESS_REPORT_FORMAT=2"), "{readme}");
        }
    }
}

#[test]
fn browser_coverage_replay_admits_no_await_major() {
    let admission = include_str!("../assets/coverage-admission.js");
    let line = admission
        .lines()
        .find(|line| line.contains("replay requires suite/"))
        .expect("the replay version table");
    for major in [52, 53] {
        assert!(
            !line.contains(&format!("'ess-conformance/{major}'")),
            "browser replay admits /{major}: {line}"
        );
    }
}
