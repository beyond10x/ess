//! An ungrouped aggregate view with no parameter is witnessed by the change its scenario's rows
//! make (story `ungrouped-aggregate-views-are-witnessed`, beyond10x/ess#148).
//!
//! `docs/design/aggregate-views.md`, "Scoping". The view's one row is over every row of the source,
//! including rows another user of a shared target made, so its absolute value is not the
//! scenario's to assert. It is read and snapshotted before the rows are created, and only the
//! change in each `count` and `sum` is asserted after them. A view with neither, and a grouped view
//! nothing scopes, keep `ESS-SYNTH-016`.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    aggregate_delta,
    coverage::{Origins, Scope},
    coverage_build,
    scenario::{SuiteFormat, ViewExpectation},
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

const ORDERS: &str = include_str!("fixtures/aggregate-optional-fields.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

/// The fixture with its `views:` replaced.
fn with_views(views: &str) -> String {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    format!("{head}views:\n{views}")
}

fn n(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn scenario<'a>(suite: &'a ConformanceSuite, view: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == format!("{view}/aggregate"))
        .map_or_else(
            || panic!("no scenario for {view}"),
            |(_, scenario)| scenario,
        )
}

fn refused(result: &Synthesis, code: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == code)
        .filter_map(|refusal| refusal.scenario.as_ref().map(ToString::to_string))
        .collect()
}

const COUNTED: &str = "  - name: demo.orders.Counted\n    source: demo.orders.Order\n    consistency: read_your_writes\n    fields:\n      - {name: orders, type: Integer, aggregate: {count: {}}}\n      - {name: longest, type: Optional<Integer>, aggregate: {max: duration, skip_absent: true}}\n";

const LONGEST: &str = "  - name: demo.orders.Longest\n    source: demo.orders.Order\n    fields:\n      - {name: longest, type: Optional<Integer>, aggregate: {max: duration, skip_absent: true}}\n";

const PER_CHANNEL: &str = "  - name: demo.orders.PerChannel\n    source: demo.orders.Order\n    group_by: [channel]\n    fields:\n      - {name: channel, type: Optional<demo.orders.Channel>}\n      - {name: orders, type: Integer, aggregate: {count: {}}}\n";

#[test]
fn a_count_beside_a_maximum_asserts_the_count_change_and_says_the_maximum_is_not_asserted() {
    let result = synthesis(&with_views(COUNTED));
    let counted = scenario(&result.suite, "demo.orders.Counted");
    // `read_your_writes`: one read after the rows, and the change and the one row asserted on it.
    let first_place = counted
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .unwrap();
    let after: Vec<&ScenarioStep> = counted.steps[first_place..]
        .iter()
        .filter(|step| {
            matches!(
                step,
                ScenarioStep::QueryView { .. }
                    | ScenarioStep::ExpectView { .. }
                    | ScenarioStep::EventuallyView { .. }
            )
        })
        .collect();
    assert_eq!(after.len(), 3, "{after:#?}");
    assert!(matches!(after[0], ScenarioStep::QueryView { .. }));
    // Three pattern rows and the one that lacks `duration`.
    assert_eq!(
        after[1],
        &ScenarioStep::ExpectView {
            view: "demo.orders.Counted".parse().unwrap(),
            expectation: ViewExpectation::ChangedBy {
                fields: BTreeMap::from([("orders".to_owned(), n("4"))]),
                absent_is_zero: BTreeSet::new(),
            },
        }
    );
    let purpose = counted.purpose.to_string();
    assert!(purpose.contains("`longest`"), "{purpose}");
    assert!(refused(&result, "ESS-SYNTH-016").is_empty());
}

#[test]
fn an_ungrouped_view_with_no_count_or_sum_keeps_its_unscoped_refusal() {
    let result = synthesis(&with_views(LONGEST));
    assert_eq!(
        refused(&result, "ESS-SYNTH-016"),
        ["demo.orders.Longest/aggregate"]
    );
    assert!(!aggregate_delta::used_by(&result.suite));
}

#[test]
fn a_view_grouped_by_an_enum_alone_keeps_its_unscoped_refusal() {
    let result = synthesis(&with_views(PER_CHANNEL));
    assert_eq!(
        refused(&result, "ESS-SYNTH-016"),
        ["demo.orders.PerChannel/aggregate"]
    );
    assert!(!aggregate_delta::used_by(&result.suite));
}

#[test]
fn a_suite_with_a_change_is_written_at_26_and_its_coverage_at_27() {
    let suite = synthesis(ORDERS).suite;
    assert!(aggregate_delta::used_by(&suite));
    assert_eq!(
        suite.provenance.suite_version.major(),
        aggregate_delta::ORDINARY
    );
    let input = coverage_build::build(&ir(ORDERS), &[], Scope::System, Origins::Generated)
        .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(
        input.selected().suite().provenance.suite_version.major(),
        aggregate_delta::COVERAGE
    );
    // Without the ungrouped total, nothing needs the round-3 pair and the suite keeps /16.
    let without = with_views(
        ORDERS
            .split_once("  - name: demo.orders.PerGroup\n")
            .map_or_else(
                || panic!("the fixture has PerGroup"),
                |(_, rest)| format!("  - name: demo.orders.PerGroup\n{rest}"),
            )
            .as_str(),
    );
    let older = synthesis(&without).suite;
    assert!(!aggregate_delta::used_by(&older));
    assert_eq!(older.provenance.suite_version.major(), 16);
}

#[test]
fn a_change_under_an_older_suite_label_is_refused_as_vocabulary_it_does_not_have() {
    let mut suite = synthesis(ORDERS).suite;
    let original = suite.to_canonical_json().unwrap();
    assert!(
        original.contains("\"expect\": \"changed_by\""),
        "{original}"
    );
    // The document relabelled with the ordinary major below the round-3 pair.
    let older = original.replace("\"ess-conformance/26\"", "\"ess-conformance/24\"");
    assert_ne!(older, original);
    let error = AdmittedSuite::from_json(&older).expect_err("a change needs suite/26");
    assert_eq!(error.issues[0].reason, "UnsupportedVocabulary", "{error}");
    assert!(error.to_string().contains("suite/26"), "{error}");
    // The typed suite pinned there is refused before serialization.
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/24").unwrap();
    let error = ess_conformance::admission::suite(&suite).expect_err("refused");
    assert_eq!(error.issues[0].reason, "UnsupportedVocabulary");
    assert!(error.to_string().contains("suite/26"), "{error}");
}

/// The suite with every `changed_by` expectation's fields replaced.
fn with_change(fields: &BTreeMap<String, Node>) -> ConformanceSuite {
    let mut suite = synthesis(ORDERS).suite;
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. } = step
            {
                if let ViewExpectation::ChangedBy { fields: held, .. } = expectation {
                    held.clone_from(fields);
                }
            }
        }
    }
    suite
}

#[test]
fn a_change_that_names_no_field_or_is_not_a_number_is_refused() {
    for fields in [
        BTreeMap::new(),
        BTreeMap::from([("total".to_owned(), Node::Text("5".to_owned()))]),
        BTreeMap::from([("total".to_owned(), Node::Null)]),
    ] {
        let error =
            AdmittedSuite::from_suite(&with_change(&fields)).expect_err("no claim is refused");
        assert_eq!(
            error.issues[0].reason, "InvalidSuite",
            "{fields:?}: {error}"
        );
    }
    // The same defects written into a document are refused by the byte-level reader too.
    let original = synthesis(ORDERS).suite.to_canonical_json().unwrap();
    for (amount, reason) in [
        (serde_json::json!("5"), "InvalidSuite"),
        (serde_json::json!(null), "InvalidSuite"),
        (
            serde_json::json!({"kind": "literal", "value": 5}),
            "InvalidSuite",
        ),
    ] {
        let mut document: serde_json::Value = serde_json::from_str(&original).unwrap();
        let steps = document["scenarios"]["demo.orders.DurationTotal/aggregate"]["steps"]
            .as_array_mut()
            .unwrap();
        let change = steps
            .iter_mut()
            .find(|step| step["expectation"]["expect"] == "changed_by")
            .expect("the scenario asserts a change");
        change["expectation"]["fields"]["total"] = amount.clone();
        let text = serde_json::to_string_pretty(&document).unwrap();
        let error = AdmittedSuite::from_json(&text).expect_err("a change is a number");
        assert_eq!(error.issues[0].reason, reason, "{amount}: {error}");
        assert!(error.to_string().contains("total"), "{amount}: {error}");
    }
}

#[test]
fn an_absent_is_zero_entry_that_names_no_field_of_the_change_is_refused() {
    // Typed: the change of `DurationTotal` lists `total`; a second, unknown name is no claim.
    let mut suite = synthesis(ORDERS).suite;
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. } = step
            {
                if let ViewExpectation::ChangedBy { absent_is_zero, .. } = expectation {
                    assert_eq!(*absent_is_zero, BTreeSet::from(["total".to_owned()]));
                    absent_is_zero.insert("orders".to_owned());
                }
            }
        }
    }
    let error = AdmittedSuite::from_suite(&suite).expect_err("an unknown name is refused");
    assert_eq!(error.issues[0].reason, "InvalidSuite", "{error}");
    assert!(error.to_string().contains("orders"), "{error}");
    // Bytes: the same entry, and an entry that is not a name.
    let original = synthesis(ORDERS).suite.to_canonical_json().unwrap();
    for listed in [
        serde_json::json!(["total", "orders"]),
        serde_json::json!([5]),
    ] {
        let mut document: serde_json::Value = serde_json::from_str(&original).unwrap();
        let steps = document["scenarios"]["demo.orders.DurationTotal/aggregate"]["steps"]
            .as_array_mut()
            .unwrap();
        let change = steps
            .iter_mut()
            .find(|step| step["expectation"]["expect"] == "changed_by")
            .expect("the scenario asserts a change");
        assert_eq!(
            change["expectation"]["absent_is_zero"],
            serde_json::json!(["total"])
        );
        change["expectation"]["absent_is_zero"] = listed.clone();
        let text = serde_json::to_string_pretty(&document).unwrap();
        assert!(
            AdmittedSuite::from_json(&text).is_err(),
            "{listed} is admitted"
        );
    }
}
