//! A refusal-selected failure policy (ess/22, beyond10x/ess#269) in the documentation, `AsyncAPI`
//! and graph projections: each says, per declared refusal, which policy answers it, and the
//! explicit fallback for a failure that carries no declared outcome. None of them prints the
//! fallback's word as the binding's one policy. A universal policy keeps its projection bytes.

use std::collections::BTreeMap;

use ess_compiler::{compile, EssIr};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::{run, Generator as _};
use ess_gen::asyncapi::AsyncApi;
use ess_gen::docs::Docs;
use ess_gen::graph::SystemGraph;
use serde_yaml::Value;

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";
const COMPONENTS: &str = "components:
  - component: ledger
    owns:
      domains: [demo.ledger]
    accepts:
      commands: [demo.ledger.Place, demo.ledger.Record]
    publishes:
      events: [demo.ledger.OrderPlaced, demo.ledger.Recorded, demo.ledger.RecordEscalated]
";

fn ir(model: &str) -> EssIr {
    let spec = Specification::assemble([
        (
            Source::new("ledger.yaml"),
            RawSpecFile::parse(model).unwrap(),
        ),
        (
            Source::new("components.yaml"),
            RawSpecFile::parse(COMPONENTS).unwrap(),
        ),
    ])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap()
}

fn universal() -> String {
    let model = MODEL.replace(
        POLICY,
        "    on_failure:\n      escalate:\n        emits: demo.ledger.RecordEscalated\n",
    );
    assert_ne!(model, MODEL);
    model
}

fn reaction(ir: &EssIr) -> Value {
    let mint = ess_gen::provenance::ProvenanceMint::new(ir);
    let documents: BTreeMap<String, String> = AsyncApi
        .generate(ir, &mint)
        .into_iter()
        .map(|artifact| (artifact.path, artifact.contents))
        .collect();
    let document: Value = serde_yaml::from_str(&documents["ledger.yaml"]).unwrap();
    document["operations"]["receive.demo.ledger.OrderPlaced"]["x-ess-reactions"][0].clone()
}

fn docs(ir: &EssIr) -> String {
    run(&Docs, ir)
        .expect("no two pages claim one path")
        .values()
        .map(|artifact| artifact.contents.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_asyncapi_reaction_names_every_refusals_policy_and_no_single_word() {
    let reaction = reaction(&ir(MODEL));
    assert!(
        reaction.get("on_failure").is_none(),
        "the fallback's word is not the binding's policy: {reaction:?}"
    );
    assert_eq!(
        reaction["escalates_with"].as_str(),
        Some("demo.ledger.RecordEscalated")
    );
    let refusals = reaction["on_refusal"]["refusals"].as_sequence().unwrap();
    let written: Vec<(String, String)> = refusals
        .iter()
        .map(|rule| {
            (
                rule["outcome"].as_str().unwrap().to_owned(),
                rule["policy"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        written,
        [
            ("unavailable", "retry"),
            ("busy", "retry"),
            ("rejected", "retry"),
            ("at-limit", "escalate"),
            ("wrong-state", "drop"),
        ]
        .map(|(outcome, policy)| (outcome.to_owned(), policy.to_owned()))
        .to_vec()
    );
    assert_eq!(
        reaction["on_refusal"]["fallback"]["policy"].as_str(),
        Some("escalate")
    );
    let means = reaction["on_failure_means"].as_str().unwrap();
    for phrase in [
        "`wrong-state`",
        "`at-limit`",
        "no declared outcome",
        "3 attempts",
    ] {
        assert!(means.contains(phrase), "{phrase}: {means}");
    }
}

#[test]
fn a_universal_policy_keeps_its_asyncapi_reaction() {
    let reaction = reaction(&ir(&universal()));
    assert_eq!(reaction["on_failure"].as_str(), Some("escalate"));
    assert!(reaction.get("on_refusal").is_none(), "{reaction:?}");
}

#[test]
fn the_documentation_states_each_refusals_policy() {
    let text = docs(&ir(MODEL));
    for phrase in [
        "selected per refusal",
        "`wrong-state`",
        "`at-limit`",
        "`demo.ledger.RecordEscalated`",
        "no declared outcome",
    ] {
        assert!(text.contains(phrase), "{phrase}");
    }
    assert!(!docs(&ir(&universal())).contains("selected per refusal"));
}

#[test]
fn the_graph_edge_carries_no_single_failure_word() {
    let ir = ir(MODEL);
    let graph = SystemGraph::of(&ir);
    let edge = graph
        .edges
        .iter()
        .find(|edge| edge.label.starts_with("notify-ledger"))
        .expect("the binding's edge");
    assert_eq!(edge.on_failure, None, "{edge:?}");
    assert!(edge.label.contains("per refusal"), "{}", edge.label);
    let universal = self::ir(&universal());
    let graph = SystemGraph::of(&universal);
    let edge = graph
        .edges
        .iter()
        .find(|edge| edge.label.starts_with("notify-ledger"))
        .expect("the binding's edge");
    assert!(edge.on_failure.is_some());
    assert!(!edge.label.contains("per refusal"));
}
