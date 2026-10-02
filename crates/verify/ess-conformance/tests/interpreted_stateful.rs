//! Source-owned subject guards select against the actual stored instance.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node};
use std::collections::BTreeMap;
fn invoke(
    target: &Interpreted,
    command: &str,
    input: BTreeMap<String, Node>,
) -> SemanticCommandResult {
    target
        .execute_command(SemanticCommandRequest {
            command: command.parse().unwrap(),
            actor: None,
            caller: None,
            input,
            correlation: CorrelationId::new("held-state").unwrap(),
        })
        .unwrap()
}
#[test]
fn post_bridge_report_selects_default_only_after_all_held_state_guards_are_false() {
    let text = include_str!("fixtures/subject-state.yaml");
    let spec =
        Specification::assemble([(Source::new("model.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let target = Interpreted::for_model(ir);
    let opened = invoke(
        &target,
        "calls.core.Open",
        BTreeMap::from([("note".into(), Node::Text("before".into()))]),
    );
    let identity = opened.direct_events[0].payload["call_id"].clone();
    let bridged = invoke(
        &target,
        "calls.core.Bridge",
        BTreeMap::from([("call_id".into(), identity.clone())]),
    );
    assert_eq!(bridged.outcome.unwrap().outcome.to_string(), "bridged");
    let reported = invoke(
        &target,
        "calls.core.Report",
        BTreeMap::from([
            ("call_id".into(), identity),
            ("incoming".into(), Node::Text("Unspecified".into())),
            ("note".into(), Node::Text("after".into())),
        ]),
    );
    assert_eq!(reported.outcome.unwrap().outcome.to_string(), "enriched");
}
