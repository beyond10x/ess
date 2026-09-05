//! The dependency graph, read off the specifications this repository actually ships.
//!
//! Two claims are worth a test here and the rest is arithmetic. The first is that **every relation
//! in the vocabulary is minted by a walk over a real model** — a relation nothing produces is an
//! edge nobody can be reached by, which is the same defect class as a refusal that cannot fire, and
//! it is invisible from inside a unit test built to produce one. The second is that the walk records
//! the edges an author actually wrote, in the direction the author wrote them, which is the one
//! thing about a graph that can be wrong without failing to compile.

mod support;

use std::collections::BTreeSet;

use ess_conformance::scenario::{
    ActorRef, CommandRef, ComponentRef, DeclaredTypeRef, EntityRef, EssSemanticRef, EventRef,
};
use ess_diff::graph::{DependencyRelation, SemanticDependencyGraph};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use support::compiled;

/// A qualified name for an assertion.
fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a valid qualified name")
}

/// The graph of the normative example.
fn billing() -> SemanticDependencyGraph {
    SemanticDependencyGraph::of(&compiled("examples/billing"))
}

#[test]
fn every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships() {
    // The check the vocabulary is worth having. `DependencyRelation` is a closed set, so a variant
    // no walk produces is an edge that can never explain anything — and adding one is exactly the
    // kind of change that compiles, reads well and does nothing.
    //
    // Two specifications, because neither alone carries every construct: `billing` has the views,
    // bindings, unions and escalation, and `revision-pair` is the smaller one the delta tests use.
    let mut minted: BTreeSet<DependencyRelation> = BTreeSet::new();
    for example in [
        "examples/billing",
        "examples/oracle-fixture",
        "examples/revision-pair/before",
    ] {
        let graph = SemanticDependencyGraph::of(&compiled(example));
        minted.extend(graph.edges().map(|edge| edge.relation));
    }

    // The legacy examples retain all their production obligations; this valid compiled extension
    // exercises the two new CLI/parameter declarations that those examples do not yet publish.
    minted.extend(
        SemanticDependencyGraph::of(&review_cli_parameter_fixture(false))
            .edges()
            .map(|edge| edge.relation),
    );
    minted.extend(
        SemanticDependencyGraph::of(&correction2_row_fixture(true))
            .edges()
            .map(|edge| edge.relation),
    );

    let missing: Vec<DependencyRelation> = DependencyRelation::ALL
        .into_iter()
        .filter(|relation| !minted.contains(relation))
        .collect();
    assert!(
        missing.is_empty(),
        "no example specification produces {missing:?} — either a walk is missing, or the relation \
         names something the model cannot express and should not be declared"
    );
}

#[test]
fn the_graph_records_the_reference_an_author_wrote_and_not_its_reverse() {
    // The direction is the one property of this graph that is wrong silently: reversed, every
    // closure still runs, still terminates and reports a plausible, empty answer. So the assertion
    // is made on a pair where the two directions differ visibly — an actor references a command,
    // and a command references no actor.
    let graph = billing();
    let customer: EssSemanticRef = ActorRef::new(name("billing.invoice.Customer")).into();
    let create: EssSemanticRef = CommandRef::new(name("billing.invoice.CreateInvoice")).into();

    let grants: Vec<_> = graph
        .dependents_of(&create)
        .filter(|edge| edge.dependent == customer)
        .collect();
    assert_eq!(
        grants.len(),
        1,
        "the actor depends on the command it may invoke: {:?}",
        graph.dependents_of(&create).collect::<Vec<_>>()
    );
    assert_eq!(grants[0].relation, DependencyRelation::MayInvoke);

    assert_eq!(
        graph.dependents_of(&customer).count(),
        0,
        "nothing in a specification depends on an actor, and an edge here would mean the walk \
         recorded the reference backwards"
    );
}

#[test]
fn a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name() {
    // The transitive claim the whole wave rests on, on the model a person can audit. `Channel` is
    // held by the `Invoice` entity and by nothing else, so a scenario that never mentions `Channel`
    // is still reached — through the entity — and the path is what says why.
    let graph = billing();
    let channel: EssSemanticRef = DeclaredTypeRef::new(name("billing.invoice.Channel")).into();
    let invoice: EssSemanticRef = EntityRef::new(name("billing.invoice.Invoice")).into();

    let reach = graph.closure(&channel);

    assert!(
        reach.reaches(&invoice),
        "the entity holds a `Channel` field"
    );
    let path = reach.path(&invoice).expect("the entity was reached");
    assert_eq!(path.len(), 1, "one hop, and it is the field: {path:?}");
    assert_eq!(path[0].relation, DependencyRelation::FieldType);
    assert_eq!(path[0].dependency, channel);

    // And the view over that entity, which mentions no type at all in its own declaration of the
    // channel — two hops, which is the answer a text search for `Channel` could not have given.
    let views: Vec<_> = reach
        .constructs()
        .filter(|construct| matches!(construct, EssSemanticRef::View { .. }))
        .collect();
    assert!(
        !views.is_empty(),
        "a view projecting the invoice must be reachable from a type the invoice holds"
    );
}

#[test]
fn a_component_is_reached_through_what_it_accepts_and_publishes() {
    // Design §24's worked example, on this repository's own model: the point of an impact report is
    // that it can name the deployable unit a change lands in, and it must get there by a semantic
    // path rather than by a risk score.
    let graph = billing();
    let created: EssSemanticRef = EventRef::new(name("billing.invoice.InvoiceCreated")).into();
    let reach = graph.closure(&created);

    let components: Vec<&EssSemanticRef> = reach
        .constructs()
        .filter(|construct| matches!(construct, EssSemanticRef::Component { .. }))
        .collect();
    assert!(
        !components.is_empty(),
        "some component publishes or reacts to the event: {:?}",
        reach.constructs().collect::<Vec<_>>()
    );

    let invoice_service: EssSemanticRef =
        ComponentRef::new(ComponentName::new("invoice-service").expect("a component name")).into();
    let path = reach
        .path(&invoice_service)
        .expect("the component that publishes the event is reached");
    assert_eq!(
        path.last().expect("a non-empty path").relation,
        DependencyRelation::Publishes
    );
}

#[test]
fn building_the_same_graph_twice_produces_the_same_edges_in_the_same_order() {
    // Two independent compilations and two independent walks. An unordered map anywhere in the
    // build would show up here as a different edge order rather than as a rumour, which is the same
    // check `tests/canonical.rs` makes of the delta's bytes.
    let first: Vec<String> = billing().edges().map(ToString::to_string).collect();
    let second: Vec<String> = billing().edges().map(ToString::to_string).collect();

    assert_eq!(first, second);
    assert!(
        first.len() > 100,
        "the billing graph is not a toy: {} edge(s)",
        first.len()
    );
}

#[test]
fn a_closure_over_the_whole_model_terminates_and_stays_inside_it() {
    // A graph with a cycle in it — and the model permits one, because a binding's command can emit
    // the event another binding reacts to — would hang a naive walk. This runs the closure from
    // every node there is, so the fixture reaches the state the guard is load-bearing in rather than
    // relying on one hand-picked start.
    let graph = billing();
    let nodes = graph.nodes().clone();

    for node in &nodes {
        let reach = graph.closure(node);
        for reached in reach.constructs() {
            assert!(
                nodes.contains(reached),
                "the closure from {node} reported {reached}, which is not a node of the graph"
            );
        }
        assert!(reach.reaches(node), "every construct is in its own closure");
    }
}

#[test]
fn review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union() {
    let before = support::compiled("examples/billing");
    let after = support::compiled_with("examples/billing", |files| {
        for entity in files.iter_mut().flat_map(|(_, file)| &mut file.entities) {
            entity.relations.clear();
        }
    });
    let graph = SemanticDependencyGraph::of(&before);
    let account: EssSemanticRef = EntityRef::new(name("billing.invoice.Account")).into();
    let invoice: EssSemanticRef = EntityRef::new(name("billing.invoice.Invoice")).into();
    let edges: Vec<_> = graph
        .edges()
        .map(|edge| {
            (
                edge.dependent.clone(),
                serde_json::to_value(edge.relation).unwrap(),
                edge.dependency.clone(),
            )
        })
        .collect();
    assert!(edges.contains(&(
        account.clone(),
        serde_json::json!("relation-target"),
        invoice.clone()
    )));
    assert!(edges.contains(&(
        invoice.clone(),
        serde_json::json!("ownership-carrier"),
        account.clone()
    )));
    assert!(graph
        .slice(&[invoice.clone()].into_iter().collect())
        .contains_key(&account));
    let union = graph.merged(&SemanticDependencyGraph::of(&after));
    assert!(
        union.closure(&account).reaches(&invoice),
        "removed relation still affects its old carrier"
    );
}

fn review_cli_parameter_fixture(grouped: bool) -> ess_compiler::EssIr {
    support::compiled_with("examples/billing", |files| {
        for (_, file) in files {
            for view in &mut file.views {
                if view.name == name("billing.invoice.InvoiceById") {
                    view.params = vec![serde_json::from_value(
                        serde_json::json!({"name":"wanted", "type":"billing.invoice.InvoiceId"}),
                    )
                    .unwrap()];
                    view.filter = Some(
                        ess_primitives::predicate::Predicate::parse_expression(
                            "invoice_id == param.wanted",
                        )
                        .unwrap(),
                    );
                }
            }
            for component in &mut file.components {
                if component.name == "invoice-service" {
                    component.reached_by = ess_domain::component::Reach::CommandLine;
                    let views = vec![
                        name("billing.invoice.InvoiceById"),
                        name("billing.invoice.OutstandingInvoices"),
                    ];
                    component.cli = Some(ess_domain::component::RawCommandLineSurface {
                        binary: "invoices".to_owned(),
                        commands: component.accepts.commands.clone(),
                        views: if grouped { Vec::new() } else { views.clone() },
                        groups: if grouped {
                            vec![ess_domain::component::RawCommandGroup {
                                name: "read".to_owned(),
                                summary: None,
                                commands: Vec::new(),
                                views,
                            }]
                        } else {
                            Vec::new()
                        },
                    });
                }
            }
        }
    })
}

#[test]
fn review_cli_views_and_parameter_types_are_forward_slice_dependencies() {
    for grouped in [false, true] {
        let graph = SemanticDependencyGraph::of(&review_cli_parameter_fixture(grouped));
        let component: EssSemanticRef =
            ComponentRef::new(ComponentName::new("invoice-service").unwrap()).into();
        let view: EssSemanticRef =
            ess_compiler::refs::ViewRef::new(name("billing.invoice.InvoiceById")).into();
        let id: EssSemanticRef = DeclaredTypeRef::new(name("billing.invoice.InvoiceId")).into();
        let edges: Vec<_> = graph
            .edges()
            .map(|edge| {
                (
                    edge.dependent.clone(),
                    serde_json::to_value(edge.relation).unwrap(),
                    edge.dependency.clone(),
                )
            })
            .collect();
        assert!(edges.contains(&(
            component.clone(),
            serde_json::json!("exposes-view"),
            view.clone()
        )));
        assert!(edges.contains(&(
            view.clone(),
            serde_json::json!("parameter-type"),
            id.clone()
        )));
        assert!(graph
            .slice(&[component.clone()].into_iter().collect())
            .contains_key(&view));
        assert!(graph.closure(&id).reaches(&component));
    }
}

#[test]
fn correction2_network_exposure_matches_actual_routes_and_owned_domains() {
    for reach in [
        ess_domain::component::Reach::InProcess,
        ess_domain::component::Reach::Network,
    ] {
        let ir = support::compiled_with("examples/billing", |files| {
            for component in files.iter_mut().flat_map(|(_, file)| &mut file.components) {
                component.reached_by = reach;
            }
        });
        let graph = SemanticDependencyGraph::of(&ir);
        let mut served = 0;
        for (name, component) in ir.components() {
            let subject: EssSemanticRef = ComponentRef::new(name.clone()).into();
            let expected: BTreeSet<EssSemanticRef> = ess_gen::http::routes(&ir, component)
                .iter()
                .filter_map(|route| match route.serves {
                    ess_gen::http::Served::View(view) => {
                        Some(ess_compiler::refs::ViewRef::from(view).into())
                    }
                    ess_gen::http::Served::Command(_) => None,
                })
                .collect();
            served += expected.len();
            let actual: BTreeSet<_> = graph
                .edges()
                .filter(|edge| {
                    edge.dependent == subject && edge.relation == DependencyRelation::ExposesView
                })
                .map(|edge| edge.dependency.clone())
                .collect();
            assert_eq!(
                actual, expected,
                "{name} with {reach:?}: only actual served views"
            );
            let slice = graph.slice(&[subject.clone()].into());
            for view in expected {
                assert!(slice.contains_key(&view));
                assert!(graph.closure(&view).reaches(&subject));
            }
        }
        assert_eq!(
            served,
            if reach == ess_domain::component::Reach::Network {
                2
            } else {
                0
            }
        );
    }
}

fn correction2_row_fixture(shaped: bool) -> ess_compiler::EssIr {
    let row = if shaped {
        "shape: probe.core.Row"
    } else {
        "fields: [{name: item_id, type: Uuid}, {name: amount, type: Integer}]"
    };
    let text = format!(
        r"
format: ess/1
system: probe
version: v1
domain: probe.core
types:
  - name: probe.core.Row
    kind: struct
    fields: [{{name: item_id, type: Uuid}}, {{name: amount, type: Integer}}]
    invariants: [amount >= 0]
entities:
  - name: probe.core.Item
    identity: {{name: item_id, type: Uuid}}
    fields: [{{name: amount, type: Integer}}]
    lifecycle:
      initial: Ready
      states: [Ready]
      terminal: [Ready]
views:
  - name: probe.core.Items
    source: probe.core.Item
    {row}
    consistency: read_your_writes
"
    );
    let raw = ess_domain::spec::RawSpecFile::parse(&text).unwrap();
    let spec = ess_domain::spec::Specification::assemble(vec![(
        ess_domain::system::Source::new("row.yaml"),
        raw,
    )])
    .unwrap();
    ess_compiler::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap()
}

#[test]
fn correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union() {
    let before = SemanticDependencyGraph::of(&correction2_row_fixture(true));
    let after = SemanticDependencyGraph::of(&correction2_row_fixture(false));
    let view: EssSemanticRef = ess_compiler::refs::ViewRef::new(name("probe.core.Items")).into();
    let row: EssSemanticRef = DeclaredTypeRef::new(name("probe.core.Row")).into();
    let edges: Vec<_> = before
        .edges()
        .filter(|edge| edge.dependent == view && edge.dependency == row)
        .collect();
    assert_eq!(edges.len(), 1, "the named row is its own dependency");
    assert_eq!(
        serde_json::to_value(edges[0].relation).unwrap(),
        "row-shape"
    );
    assert!(before.slice(&[view.clone()].into()).contains_key(&row));
    assert!(
        !after.slice(&[view.clone()].into()).contains_key(&row),
        "inline fields do not consume the reusable definition"
    );
    assert!(before.merged(&after).closure(&row).reaches(&view));
}
