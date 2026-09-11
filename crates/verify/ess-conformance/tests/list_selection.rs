//! Observable first-match selection keeps original occurrence identity and fallback order.
use ess_compiler::{
    ir::{EssIr, ResolvedMappingValue},
    resolve::compile_locating,
    source::SourceMap,
};
use ess_conformance::{accessor::Expected, selection::Observation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;
const MODEL: &str = include_str!("fixtures/binding-selection.yaml");
fn compile() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("selection.yaml"), raw)]).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", MODEL);
    compile_locating(&spec, &sources, &["selection.yaml"]).unwrap()
}
fn observations(ir: &EssIr) -> Vec<Observation> {
    let binding = ir.bindings().values().next().unwrap();
    binding
        .mapping
        .iter()
        .map(|mapped| {
            let ResolvedMappingValue::Selection {
                selector,
                projection,
                ..
            } = &mapped.value
            else {
                panic!("selection mapping")
            };
            Observation::of(ir, binding, *selector, projection, &mapped.target_type).unwrap()
        })
        .collect()
}
fn leg(id: &str, domain: Option<&str>, source: Option<&str>) -> Node {
    let mut fields = BTreeMap::from([
        ("id".to_owned(), Node::Text(id.into())),
        ("from".to_owned(), Node::Text("remote".into())),
    ]);
    if let Some(domain) = domain {
        fields.insert("domain".into(), Node::Text(domain.into()));
    }
    if let Some(source) = source {
        fields.insert("source".into(), Node::Text(source.into()));
    }
    Node::Map(fields)
}
fn result(value: Option<&str>) -> Expected {
    value.map_or(Expected::Absent, |value| {
        Expected::Present(Node::Text(value.into()))
    })
}
#[test]
fn actual_ordered_mixed_leg_and_fallback_decision_table() {
    let observations = observations(&compile());
    let rows = [
        (vec![], None, None),
        (vec![Node::Null, leg("x", None, None)], Some("x"), Some("x")),
        (
            vec![
                leg("x", Some("External"), None),
                leg("y", Some("Internal"), None),
            ],
            Some("y"),
            Some("x"),
        ),
        (
            vec![leg("x", Some("External"), Some("Webrtc"))],
            Some("x"),
            Some("x"),
        ),
        (
            vec![
                leg("x", Some("External"), Some("Webrtc")),
                leg("y", Some("External"), None),
            ],
            Some("x"),
            Some("y"),
        ),
        (
            vec![
                leg("x", Some("Internal"), None),
                leg("y", Some("External"), Some("Webrtc")),
            ],
            Some("x"),
            Some("y"),
        ),
        (
            vec![
                leg("x", Some("External"), Some("Webrtc")),
                leg("y", Some("External"), Some("Webrtc")),
            ],
            Some("x"),
            Some("y"),
        ),
        (
            vec![
                leg("x", Some("External"), None),
                leg("y", Some("External"), None),
            ],
            Some("x"),
            Some("x"),
        ),
        (
            vec![Node::Null, leg("", None, None), leg("z", None, None)],
            Some("z"),
            Some("z"),
        ),
        (
            vec![
                leg("", Some("Internal"), None),
                leg("y", Some("External"), None),
            ],
            Some(""),
            Some("y"),
        ),
        (
            vec![leg("", Some("External"), None), leg("y", None, None)],
            Some("y"),
            Some(""),
        ),
        (
            vec![
                leg("x", Some("Internal"), None),
                leg("y", Some("Internal"), None),
                leg("z", Some("External"), None),
            ],
            Some("x"),
            Some("z"),
        ),
        (
            vec![
                leg("same", Some("External"), Some("Webrtc")),
                leg("same", Some("External"), None),
            ],
            Some("same"),
            Some("same"),
        ),
    ];
    for (index, (items, agent, external)) in rows.into_iter().enumerate() {
        let payload = BTreeMap::from([("data".into(), Node::Seq(items))]);
        assert_eq!(
            observations[0].evaluate(&payload).unwrap(),
            result(agent),
            "agent row {index}"
        );
        assert_eq!(
            observations[1].evaluate(&payload).unwrap(),
            result(external),
            "external row {index}"
        );
    }
}
#[test]
fn malformed_and_over_limit_tail_cannot_hide_behind_first_match() {
    let observations = observations(&compile());
    let first = leg("selected", Some("Internal"), None);
    for tail in [
        Node::Text("malformed".into()),
        leg("bad", Some("unknown"), None),
    ] {
        let payload = BTreeMap::from([("data".into(), Node::Seq(vec![first.clone(), tail]))]);
        assert!(observations[0]
            .evaluate(&payload)
            .unwrap_err()
            .contains("invalid_input"));
    }
    let payload = BTreeMap::from([("data".into(), Node::Seq(vec![first; 65]))]);
    assert!(observations[0]
        .evaluate(&payload)
        .unwrap_err()
        .contains("resource"));
}
#[test]
fn retained_indices_and_declared_predicate_facts_are_revalidated() {
    let observations = observations(&compile());
    let original = serde_json::to_value(&observations[0]).unwrap();
    assert!(serde_json::from_value::<Observation>(original.clone()).is_ok());
    let mut forward = original.clone();
    forward["plan"]["selectors"][1]["operation"]["excluding"] = serde_json::json!([4]);
    assert!(serde_json::from_value::<Observation>(forward).is_err());
    let mut changed = original;
    changed["plan"]["inputs"][0]["optional_items"] = serde_json::json!(false);
    assert!(serde_json::from_value::<Observation>(changed).is_err());
}

#[test]
fn independent_go_observation_witness_receives_closed_typed_contract() {
    let observations = observations(&compile());
    if let Some(directory) = std::env::var_os("ESS_SELECTION_ORACLE_OUT") {
        let base = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(
            base.join("observation.json"),
            serde_json::to_vec(&observations).unwrap(),
        )
        .unwrap();
        std::fs::write(
            base.join("runtime.go"),
            include_str!("../src/go/runtime.go"),
        )
        .unwrap();
        std::fs::write(
            base.join("predicate.go"),
            include_str!("../src/go/predicate.go"),
        )
        .unwrap();
    }
    assert!(observations
        .iter()
        .all(|observation| observation.validate().is_ok()));
}

#[test]
fn generated_binding_scenario_retains_observed_selection_and_version_six() {
    let model = MODEL.replace("commands:\n", "commands:\n  - name: selection.core.Publish\n    input: []\n    outcomes:\n      - name: published\n        emits: [selection.core.Arrived]\n");
    let spec = Specification::assemble([(
        Source::new("selection.yaml"),
        RawSpecFile::parse(&model).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", model);
    let ir = compile_locating(&spec, &sources, &["selection.yaml"]).unwrap();
    let result = ess_conformance::synthesize(&ir);
    let suite = result.suite.to_canonical_json().unwrap();
    assert!(suite.contains("observed_selection"), "{result:?}");
    assert!(suite.contains("ess-conformance/6"));
    if let Some(directory) = std::env::var_os("ESS_SELECTION_ORACLE_OUT") {
        std::fs::write(
            std::path::PathBuf::from(directory).join("suite.json"),
            suite,
        )
        .unwrap();
    }
}

#[test]
fn exclusions_compare_occurrence_indices_even_when_identifiers_are_equal() {
    let model = MODEL.replace(
        "external_id: {selection: external, path: [id]}",
        "external_id: {selection: external, path: [from]}",
    );
    let spec = Specification::assemble([(
        Source::new("selection.yaml"),
        RawSpecFile::parse(&model).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", model);
    let ir = compile_locating(&spec, &sources, &["selection.yaml"]).unwrap();
    let observations = observations(&ir);
    let mut first = leg("same", Some("External"), Some("Webrtc"));
    let mut second = leg("same", Some("External"), None);
    let Node::Map(first_fields) = &mut first else {
        unreachable!()
    };
    first_fields.insert("from".into(), Node::Text("first".into()));
    let Node::Map(second_fields) = &mut second else {
        unreachable!()
    };
    second_fields.insert("from".into(), Node::Text("second".into()));
    let payload = BTreeMap::from([("data".into(), Node::Seq(vec![first, second]))]);
    assert_eq!(
        observations[1].evaluate(&payload).unwrap(),
        result(Some("second"))
    );
}
