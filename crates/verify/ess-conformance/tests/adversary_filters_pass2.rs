//! Adversary cases for story:view-filters-witnessed-on-matching-rows, pass 2: the matching row a
//! filtered view is given lands in every view of the entity and takes an instance number, and
//! neither is checked against what the rest of the scenario already says.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{synthesize, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|e| panic!("parses: {e}\n{text}"));
    let specification = Specification::assemble([(Source::new("adv.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new()).unwrap_or_else(|d| panic!("resolves:\n{d}"))
}

/// An order created with `source` (and `flag`), with the given extra commands and views.
fn model(commands: &str, views: &str) -> String {
    format!(
        r"format: ess/15
system: demo
version: v1
domain: demo.orders
entities:
  - name: demo.orders.Order
    identity: {{name: order_id, type: Uuid}}
    fields:
      - {{name: source, type: String}}
      - {{name: flag, type: Boolean}}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: demo.orders.Placed
    fields: [{{name: order_id, type: Uuid}}]
  - name: demo.orders.Flagged
    fields: []
commands:
  - name: demo.orders.Place
    input:
      - {{name: source, type: String}}
      - {{name: flag, type: Boolean}}
    outcomes:
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        sets: {{source: input.source, flag: input.flag}}
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            order_id: {{generated: true}}
{commands}views:
{views}"
    )
}

const WEB_BY_ID: &str = r#"  - name: demo.orders.Web
    source: demo.orders.Order
    consistency: read_your_writes
    filter: 'source == "web"'
    fields:
      - {name: order_id, type: Uuid}
      - {name: source, type: String}
"#;

/// Per scenario, per view, what it is asserted to hold.
fn expectations(synthesis: &Synthesis, view: &str) -> Vec<(String, Vec<ViewExpectation>)> {
    synthesis
        .suite
        .scenarios
        .iter()
        .map(|(id, scenario)| {
            let found = scenario
                .steps
                .iter()
                .filter_map(|step| match step {
                    ScenarioStep::ExpectView {
                        view: name,
                        expectation,
                    }
                    | ScenarioStep::EventuallyView {
                        view: name,
                        expectation,
                        ..
                    } if name.to_string() == view => Some(expectation.clone()),
                    _ => None,
                })
                .collect();
            (id.to_string(), found)
        })
        .collect()
}

/// Pass 1's correction keeps a view that does not project the identity out of `arrange_matching`,
/// so it gets `Excludes {}` ("holds no rows") alone. But the row arranged for a *sibling* view that
/// does project it lands in every view of the entity: when that row meets the unidentified view's
/// filter too, the scenario creates a `web` row and asserts the unidentified `web` view holds none.
/// No implementation passes that scenario.
#[test]
fn adv_filters_p2_a_sibling_views_matching_row_does_not_contradict_an_empty_view() {
    let views = format!(
        r#"{WEB_BY_ID}  - name: demo.orders.WebSources
    source: demo.orders.Order
    consistency: read_your_writes
    filter: 'source == "web"'
    fields:
      - {{name: source, type: String}}
"#
    );
    let synthesis = synthesize(&compiled(&model("", &views)));
    let web = expectations(&synthesis, "demo.orders.Web");
    let arranged: Vec<&String> = web
        .iter()
        .filter(|(_, found)| {
            found
                .iter()
                .any(|e| matches!(e, ViewExpectation::Contains { fields } if matches!(fields.get("order_id"), Some(ScenarioValue::Instance { .. }))))
        })
        .map(|(id, _)| id)
        .collect();
    assert!(
        !arranged.is_empty(),
        "precondition: the identified view is given an arranged matching row"
    );
    let contradicted: Vec<String> = expectations(&synthesis, "demo.orders.WebSources")
        .into_iter()
        .filter(|(id, found)| {
            arranged.contains(&id)
                && found
                    .iter()
                    .any(|e| matches!(e, ViewExpectation::Excludes { fields } if fields.is_empty()))
        })
        .map(|(id, _)| id)
        .collect();
    assert!(
        contradicted.is_empty(),
        "scenarios arrange a `web` row and assert `demo.orders.WebSources` holds no rows: {contradicted:?}"
    );
}

/// An `updates:` writing a literal the plain witness already holds arranges its subject under a
/// further distinction (beyond10x/ess#161) — `order-2`. The matching row numbers itself
/// `companions.len() + 1` from the subject's own numbering's blind side, so it is `order-2` too:
/// the scenario captures one name twice and then asserts the view both excludes and contains it.
#[test]
fn adv_filters_p2_the_matching_row_never_reuses_the_subjects_instance_name() {
    let commands = r"  - name: demo.orders.SetOn
    input:
      - {name: order_id, type: Uuid}
    outcomes:
      - name: on
        updates: demo.orders.Order
        instance: order_id
        sets: {flag: true}
        emits: [demo.orders.Flagged]
  - name: demo.orders.SetOff
    input:
      - {name: order_id, type: Uuid}
    outcomes:
      - name: off
        updates: demo.orders.Order
        instance: order_id
        sets: {flag: false}
        emits: [demo.orders.Flagged]
";
    let synthesis = synthesize(&compiled(&model(commands, WEB_BY_ID)));
    let mut reused = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for step in &scenario.steps {
            if let ScenarioStep::CaptureInstance { instance, .. } = step {
                *seen.entry(instance.to_string()).or_default() += 1;
            }
        }
        for (name, count) in seen {
            if count > 1 {
                reused.push(format!("{id}: `{name}` captured {count} times"));
            }
        }
    }
    assert!(reused.is_empty(), "{reused:#?}");
}

/// The acceptance's own rule — where the suite asserts a folded view at all, some row it asserts
/// carries the literal in another ASCII case — under a negated fold. The subject (`source`) is
/// admitted and asserted `Contains`; a target that compares bytes admits it too, and the row that
/// separates the two (`WEB`, which the fold refuses and a byte-wise `not` admits) is never
/// arranged. A byte-wise target passes every scenario.
#[test]
fn adv_filters_p2_a_negated_fold_is_witnessed_on_a_case_changed_row() {
    let views = r"  - name: demo.orders.Web
    source: demo.orders.Order
    consistency: read_your_writes
    filter: {not: {source: {equals_ignore_case: web}}}
    fields:
      - {name: order_id, type: Uuid}
      - {name: source, type: String}
";
    let synthesis = synthesize(&compiled(&model("", views)));
    let mut asserted = false;
    let mut texts = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        if !format!("{:?}", scenario.steps).contains("demo.orders.Web") {
            continue;
        }
        asserted = true;
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                if let Some(Node::Text(text)) =
                    input.get("source").and_then(ScenarioValue::as_literal)
                {
                    texts.push(text.clone());
                }
            }
        }
    }
    assert!(asserted, "precondition: the view is asserted");
    assert!(
        texts
            .iter()
            .any(|text| text.eq_ignore_ascii_case("web") && text != "web"),
        "the negated fold is asserted only over rows {texts:?}: a byte-wise filter passes"
    );
}
