//! A binding's event-payload condition (ess/22, beyond10x/ess#268) in every projection that
//! describes bindings: the documentation's prose and flow diagram, the system graph's binding
//! edge, and the `AsyncAPI` document's reaction and consumer. A binding without one renders as
//! it did.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/binding-condition.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";
const COMPONENT: &str = "components:
  - component: messages-service
    summary: Receives messages, and records them against their order.
    owns:
      domains: [demo.messages]
    accepts:
      commands: [demo.messages.ReceiveMessage, demo.messages.MessageEvent, demo.messages.LogMessage]
    publishes:
      events: [demo.messages.MessageReceived, demo.messages.OrderMessaged, demo.messages.MessageLogged]
    reached_by: network
";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("messages.yaml"),
        RawSpecFile::parse(&format!("{text}{COMPONENT}")).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The condition as `Predicate`'s own rendering writes it.
fn rendered() -> String {
    let ir = ir(MODEL);
    ir.bindings()
        .values()
        .find(|b| b.name.as_str() == "received")
        .unwrap()
        .condition
        .as_ref()
        .expect("the binding declares a condition")
        .plan
        .predicate
        .to_string()
}

fn joined(ir: &EssIr, prefix: &str) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .iter()
        .filter(|(path, _)| path.starts_with(prefix))
        .map(|(_, artifact)| artifact.contents.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_documentation_states_the_condition_and_draws_it_on_the_flow() {
    let docs = joined(&ir(MODEL), "docs/");
    let predicate = rendered();
    assert!(
        docs.contains(&format!(
            "It invokes only for an occurrence whose payload makes `{predicate}` hold"
        )),
        "{docs}"
    );
    assert!(
        docs.contains("received when "),
        "the flow edge names it: {docs}"
    );
}

#[test]
fn the_graph_edge_names_the_condition() {
    let model = ir(MODEL);
    let graph = ess_gen::SystemGraph::of(&model);
    let predicate = rendered();
    assert!(graph.dot().contains("when "), "{}", graph.dot());
    assert!(
        graph.mermaid().contains(&predicate) || graph.mermaid().contains("received when"),
        "{}",
        graph.mermaid()
    );
}

#[test]
fn the_asyncapi_reaction_and_consumer_carry_the_condition() {
    let asyncapi = joined(&ir(MODEL), "asyncapi/");
    let predicate = rendered();
    assert_eq!(
        asyncapi.matches(&format!("where: {predicate}")).count()
            + asyncapi.matches(&format!("where: '{predicate}'")).count()
            + asyncapi.matches(&format!("where: \"{predicate}\"")).count(),
        2,
        "once on the reaction, once on the consumer: {asyncapi}"
    );
}

#[test]
fn a_binding_without_a_condition_renders_no_condition() {
    let plain = MODEL.replace(WHERE, "").replace(
        "      order_id: event.order.id\n",
        "      order_id: event.message_id\n",
    );
    let ir = ir(&plain);
    for prefix in ["docs/", "asyncapi/"] {
        let content = joined(&ir, prefix);
        assert!(!content.contains("where:"), "{prefix}: {content}");
        assert!(
            !content.contains("It invokes only for an occurrence"),
            "{prefix}"
        );
    }
    assert!(!ess_gen::SystemGraph::of(&ir).dot().contains(" when "));
}
