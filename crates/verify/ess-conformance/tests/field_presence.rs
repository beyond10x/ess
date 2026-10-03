//! A field's presence policy in a suite (beyond10x/ess#139).
//!
//! `null_when_absent` and `omitted_when_absent` are carried on a payload leaf as `presence`, and a
//! runner holding only the suite decides by it: an implementation that swaps the two fails. A
//! suite that carries one is written as `ess-conformance/24` (ordinary) or `/25` (coverage),
//! because a reader that predates the key drops it and passes the swap.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::SuiteFormat;
use ess_conformance::{Holds, LeafShape, ScenarioStep};
use ess_domain::types::{Presence, Primitive};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn text() -> Holds {
    Holds::Primitive {
        kind: Primitive::String,
    }
}

#[test]
fn issue_139_a_leaf_decides_null_and_absence_by_its_policy() {
    let null_when_absent = LeafShape::required(text())
        .optional()
        .with_presence(Some(Presence::NullWhenAbsent));
    let omitted_when_absent = LeafShape::required(text())
        .optional()
        .with_presence(Some(Presence::OmittedWhenAbsent));
    let either = LeafShape::required(text()).optional();
    let value = Node::Text("x".to_owned());
    for (leaf, absent, null, which) in [
        (&null_when_absent, false, true, "null_when_absent"),
        (&omitted_when_absent, true, false, "omitted_when_absent"),
        (&either, true, true, "no policy"),
    ] {
        assert_eq!(leaf.admits(None), absent, "{which}: an absent key");
        assert_eq!(
            leaf.admits(Some(&Node::Null)),
            null,
            "{which}: an explicit null"
        );
        assert!(leaf.admits(Some(&value)), "{which}: a value");
        assert!(
            !leaf.admits(Some(&Node::Bool(true))),
            "{which}: a wrong value"
        );
    }
}

#[test]
fn an_implementation_that_swaps_the_two_policies_fails_both_leaves() {
    // What a runner sees from the swapped implementation: the `null_when_absent` field left out,
    // and the `omitted_when_absent` field sent as `null`.
    let partner_ref = LeafShape::required(text())
        .optional()
        .with_presence(Some(Presence::NullWhenAbsent));
    let discount_code = LeafShape::required(text())
        .optional()
        .with_presence(Some(Presence::OmittedWhenAbsent));
    assert!(!partner_ref.admits(None));
    assert!(!discount_code.admits(Some(&Node::Null)));
    // And the correct one passes both.
    assert!(partner_ref.admits(Some(&Node::Null)));
    assert!(discount_code.admits(None));
}

#[test]
fn presence_is_written_only_where_declared_and_round_trips() {
    let leaf = LeafShape::required(text())
        .optional()
        .with_presence(Some(Presence::NullWhenAbsent));
    let written = serde_json::to_string(&leaf).expect("serialises");
    assert!(
        written.contains(r#""presence":"null_when_absent""#),
        "{written}"
    );
    assert_eq!(
        serde_json::from_str::<LeafShape>(&written).expect("reads back"),
        leaf
    );
    let plain = serde_json::to_string(&LeafShape::required(text()).optional()).expect("serialises");
    assert!(!plain.contains("presence"), "{plain}");
}

#[test]
fn suite_formats_24_and_25_remain_supported_and_future_versions_refuse() {
    for version in ["ess-conformance/24", "ess-conformance/25"] {
        assert!(SuiteFormat::parse(version)
            .expect("well formed")
            .is_supported());
    }
    assert!(!SuiteFormat::parse("ess-conformance/36")
        .expect("well formed")
        .is_supported());
}

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.Placed
    fields:
      - {name: note, type: Optional<String>}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: note, type: Optional<String>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed: {note: input.note}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn a_suite_with_a_presence_leaf_is_written_as_suite_24() {
    let mut suite = ess_conformance::synthesize::synthesize(&ir(MODEL)).suite;
    let before = suite.provenance.suite_version;
    assert_ne!(before.major(), 24, "no leaf carries a policy yet");
    let mut marked = false;
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectEvent { shape, .. } = step {
                if let Some(leaf) = shape.leaves().get("note").cloned() {
                    shape.insert("note", leaf.with_presence(Some(Presence::NullWhenAbsent)));
                    marked = true;
                }
            }
        }
    }
    assert!(marked, "the suite expects the event and describes `note`");
    suite.select_fresh_format();
    assert_eq!(
        suite.provenance.suite_version,
        SuiteFormat::parse("ess-conformance/24").unwrap()
    );
    assert!(ess_conformance::presence::used_by(&suite));
}

#[test]
fn a_suite_without_one_keeps_its_format() {
    let mut suite = ess_conformance::synthesize::synthesize(&ir(MODEL)).suite;
    let before = suite.provenance.suite_version;
    suite.select_fresh_format();
    assert_eq!(suite.provenance.suite_version, before);
    assert!(!ess_conformance::presence::used_by(&suite));
}
