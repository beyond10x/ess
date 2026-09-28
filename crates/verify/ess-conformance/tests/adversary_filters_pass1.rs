//! Adversary cases for story:view-filters-witnessed-on-matching-rows, pass 1: a filtered view is
//! asserted `Contains` over a row its filter matches, beyond the one equality and one
//! `equals_ignore_case` shape the unit's own tests cover.

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

/// An order with `fields`, created by `demo.orders.Place` through `outcomes`, closable by
/// `demo.orders.Close`, read by `view`.
fn model(types: &str, fields: &str, input: &str, outcomes: &str, view: &str) -> String {
    format!(
        r"format: ess/15
system: demo
version: v1
domain: demo.orders
types:
{types}
entities:
  - name: demo.orders.Order
    identity: {{name: order_id, type: Uuid}}
    fields:
{fields}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
events:
  - name: demo.orders.Placed
    fields: [{{name: order_id, type: Uuid}}]
  - name: demo.orders.Closed
    fields: []
commands:
  - name: demo.orders.Place
    input:
{input}
    outcomes:
{outcomes}
  - name: demo.orders.Close
    input:
      - {{name: order_id, type: Uuid}}
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: order_id
        emits: [demo.orders.Closed]
views:
{view}
"
    )
}

fn placed(sets: &str) -> String {
    format!(
        r"      - name: placed
        creates: demo.orders.Order
        instance: order_id
        sets: {sets}
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            order_id: {{generated: true}}"
    )
}

fn view(filter: &str, fields: &str) -> String {
    format!(
        r"  - name: demo.orders.Web
    source: demo.orders.Order
    consistency: read_your_writes
    filter: {filter}
    fields:
{fields}"
    )
}

const ID_AND_SOURCE: &str =
    "      - {name: order_id, type: Uuid}\n      - {name: source, type: String}";

/// Per `Contains` asserted in `demo.orders.Web` over a named instance, the input the
/// `demo.orders.Place` that produced that instance was sent.
fn contained_inputs(synthesis: &Synthesis) -> Vec<BTreeMap<String, ScenarioValue>> {
    let mut found = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        for step in &scenario.steps {
            let expectation = match step {
                ScenarioStep::ExpectView {
                    view: name,
                    expectation,
                }
                | ScenarioStep::EventuallyView {
                    view: name,
                    expectation,
                    ..
                } if name.to_string() == "demo.orders.Web" => expectation,
                _ => continue,
            };
            let ViewExpectation::Contains { fields } = expectation else {
                continue;
            };
            let Some(ScenarioValue::Instance { instance }) = fields.get("order_id") else {
                continue;
            };
            let captured = scenario.steps.iter().position(|step| {
                matches!(step, ScenarioStep::CaptureInstance { instance: named, .. } if named == instance)
            });
            let sent = captured.and_then(|at| {
                scenario.steps[..at]
                    .iter()
                    .rev()
                    .find_map(|step| match step {
                        ScenarioStep::ExecuteCommand { command, input, .. }
                            if command.to_string() == "demo.orders.Place" =>
                        {
                            Some(input.clone())
                        }
                        _ => None,
                    })
            });
            if let Some(input) = sent {
                found.push(input);
            }
        }
    }
    found
}

fn field(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<String> {
    match input.get(field).and_then(ScenarioValue::as_literal) {
        Some(Node::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// `in_ignore_case` folds as `equals_ignore_case` does, and the reference page says so: some row
/// asserted in the view must carry one of the listed literals in another ASCII case.
#[test]
fn adv_filters_in_ignore_case_is_asserted_over_a_case_changed_matching_row() {
    let text = model(
        "  []",
        "      - {name: source, type: String}",
        "      - {name: source, type: String}",
        &placed("{source: input.source}"),
        &view("{source: {in_ignore_case: [web, app]}}", ID_AND_SOURCE),
    );
    let synthesis = synthesize(&compiled(&text));
    let sources: Vec<String> = contained_inputs(&synthesis)
        .iter()
        .filter_map(text_of)
        .collect();
    assert!(
        sources
            .iter()
            .any(|s| (s.eq_ignore_ascii_case("web") && s != "web")
                || (s.eq_ignore_ascii_case("app") && s != "app")),
        "no case-changed matching row asserted: {sources:?}, refusals {:?}",
        refusals(&synthesis)
    );
}

fn text_of(input: &BTreeMap<String, ScenarioValue>) -> Option<String> {
    field(input, "source")
}

/// A filter over two row fields: the matching row has to meet both.
#[test]
fn adv_filters_two_field_filter_is_asserted_over_a_row_meeting_both() {
    let text = model(
        "  []",
        "      - {name: source, type: String}\n      - {name: region, type: String}",
        "      - {name: source, type: String}\n      - {name: region, type: String}",
        &placed("{source: input.source, region: input.region}"),
        &view(
            "['source == \"web\"', 'region == \"eu\"']",
            "      - {name: order_id, type: Uuid}\n      - {name: source, type: String}\n      - {name: region, type: String}",
        ),
    );
    let synthesis = synthesize(&compiled(&text));
    let rows: Vec<(Option<String>, Option<String>)> = contained_inputs(&synthesis)
        .iter()
        .map(|input| (field(input, "source"), field(input, "region")))
        .collect();
    assert!(
        rows.iter()
            .any(|(s, r)| s.as_deref() == Some("web") && r.as_deref() == Some("eu")),
        "no row meeting both conjuncts asserted: {rows:?}, refusals {:?}",
        refusals(&synthesis)
    );
}

/// An enum filter on the variant the plain witness does not pick.
#[test]
fn adv_filters_enum_filter_is_asserted_over_a_matching_row() {
    let text = model(
        "  - name: demo.orders.Channel\n    kind: enum\n    variants: [Phone, Web]",
        "      - {name: channel, type: demo.orders.Channel}",
        "      - {name: channel, type: demo.orders.Channel}",
        &placed("{channel: input.channel}"),
        &view(
            "channel == Web",
            "      - {name: order_id, type: Uuid}\n      - {name: channel, type: demo.orders.Channel}",
        ),
    );
    let synthesis = synthesize(&compiled(&text));
    let channels: Vec<Option<String>> = contained_inputs(&synthesis)
        .iter()
        .map(|input| field(input, "channel"))
        .collect();
    assert!(
        channels.iter().any(|c| c.as_deref() == Some("Web")),
        "no Web row asserted: {channels:?}, refusals {:?}",
        refusals(&synthesis)
    );
}

/// An `Optional<String>` field the filter compares.
#[test]
fn adv_filters_optional_field_filter_is_asserted_over_a_matching_row() {
    let text = model(
        "  []",
        "      - {name: source, type: Optional<String>}",
        "      - {name: source, type: Optional<String>}",
        &placed("{source: input.source}"),
        &view(
            "'source == \"web\"'",
            "      - {name: order_id, type: Uuid}\n      - {name: source, type: Optional<String>}",
        ),
    );
    let synthesis = synthesize(&compiled(&text));
    let sources: Vec<Option<String>> = contained_inputs(&synthesis)
        .iter()
        .map(|input| field(input, "source"))
        .collect();
    assert!(
        sources.iter().any(|s| s.as_deref() == Some("web")),
        "no web row asserted: {sources:?}, refusals {:?}",
        refusals(&synthesis)
    );
}

/// The matching row has to be created toward the filter *and* moved to the state it names.
#[test]
fn adv_filters_filter_needing_a_state_change_is_asserted_over_a_matching_row() {
    let text = model(
        "  []",
        "      - {name: source, type: String}",
        "      - {name: source, type: String}",
        &placed("{source: input.source}"),
        &view("[state == Closed, 'source == \"web\"']", ID_AND_SOURCE),
    );
    let synthesis = synthesize(&compiled(&text));
    let sources: Vec<Option<String>> = contained_inputs(&synthesis)
        .iter()
        .map(|input| field(input, "source"))
        .collect();
    assert!(
        sources.iter().any(|s| s.as_deref() == Some("web")),
        "no closed web row asserted: {sources:?}, refusals {:?}",
        refusals(&synthesis)
    );
}

/// The acceptance's fold case where the creating command branches on the same literal: the branch
/// `web` is taken for `source == "web"` and the unguarded `other` otherwise, and both set `source`. The
/// plain witness of one scenario sends `web` byte for byte, so a row asserted over `web` alone
/// still passes a byte-wise target; `WEB` through `other` is the row that target drops, and it is
/// reachable.
#[test]
fn adv_filters_fold_beside_a_branch_on_the_literal_still_asserts_a_case_changed_row() {
    let outcomes = r#"      - name: web
        when: 'source == "web"'
        creates: demo.orders.Order
        instance: order_id
        sets: {source: input.source}
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            order_id: {generated: true}
      - name: other
        creates: demo.orders.Order
        instance: order_id
        sets: {source: input.source}
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            order_id: {generated: true}"#;
    let text = model(
        "  []",
        "      - {name: source, type: String}",
        "      - {name: source, type: String}",
        outcomes,
        &view("{source: {equals_ignore_case: web}}", ID_AND_SOURCE),
    );
    let synthesis = synthesize(&compiled(&text));
    let sources: Vec<String> = contained_inputs(&synthesis)
        .iter()
        .filter_map(text_of)
        .collect();
    assert!(
        sources
            .iter()
            .any(|s| s.eq_ignore_ascii_case("web") && s != "web"),
        "the folded view is asserted only over {sources:?}: a byte-wise filter passes, refusals {:?}",
        refusals(&synthesis)
    );
}

/// A filtered view that does not project the identity: the subject's `Excludes` has no field to
/// name a row by and reads "holds no rows", so a matching row arranged in the same scenario and
/// asserted `Contains {}` ("holds a row") makes the pair unsatisfiable by every implementation.
#[test]
fn adv_filters_no_identity_view_is_never_asserted_empty_and_non_empty_at_once() {
    let text = model(
        "  []",
        "      - {name: source, type: String}",
        "      - {name: source, type: String}",
        &placed("{source: input.source}"),
        &view(
            "'source == \"web\"'",
            "      - {name: source, type: String}",
        ),
    );
    let synthesis = synthesize(&compiled(&text));
    let mut contradictions = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut empty = false;
        let mut holds = false;
        for step in &scenario.steps {
            let expectation = match step {
                ScenarioStep::ExpectView {
                    view: name,
                    expectation,
                }
                | ScenarioStep::EventuallyView {
                    view: name,
                    expectation,
                    ..
                } if name.to_string() == "demo.orders.Web" => expectation,
                _ => continue,
            };
            match expectation {
                ViewExpectation::Excludes { fields } if fields.is_empty() => empty = true,
                ViewExpectation::Contains { .. } => holds = true,
                _ => {}
            }
        }
        if empty && holds {
            contradictions.push(id.to_string());
        }
    }
    assert!(
        contradictions.is_empty(),
        "scenarios require the view to hold no rows and a row at once: {contradictions:?}"
    );
}

/// A ranked view filtered on a row field was refused `OrderUnwitnessed` before this unit; it now
/// gets companions from the same search. Where the order is asserted, the count floor beside it
/// must be at least two rows that the filter admits.
#[test]
fn adv_filters_ranked_filtered_view_asserts_order_over_two_matching_rows() {
    let text = model(
        "  []",
        "      - {name: source, type: String}\n      - {name: rank, type: Integer}",
        "      - {name: source, type: String}\n      - {name: rank, type: Integer}",
        &placed("{source: input.source, rank: input.rank}"),
        &format!(
            "{}\n    order_by: [rank desc]",
            view(
                "'source == \"web\"'",
                "      - {name: order_id, type: Uuid}\n      - {name: source, type: String}\n      - {name: rank, type: Integer}",
            )
        ),
    );
    let synthesis = synthesize(&compiled(&text));
    let mut ranked = 0;
    for scenario in synthesis.suite.scenarios.values() {
        let mut floor = None;
        let mut orders = false;
        for step in &scenario.steps {
            let expectation = match step {
                ScenarioStep::ExpectView {
                    view: name,
                    expectation,
                }
                | ScenarioStep::EventuallyView {
                    view: name,
                    expectation,
                    ..
                } if name.to_string() == "demo.orders.Web" => expectation,
                _ => continue,
            };
            match expectation {
                ViewExpectation::Ranked { .. } => orders = true,
                ViewExpectation::Counts { at_least, .. } => floor = *at_least,
                _ => {}
            }
        }
        if orders {
            ranked += 1;
            assert!(
                floor.is_some_and(|n| n >= 2),
                "order asserted with floor {floor:?}"
            );
        }
    }
    let web: Vec<Option<String>> = contained_inputs(&synthesis).iter().map(text_of).collect();
    assert!(
        ranked > 0 && web.iter().filter(|s| s.as_deref() == Some("web")).count() >= 1,
        "ranked {ranked}, contained {web:?}, refusals {:?}",
        refusals(&synthesis)
    );
}
