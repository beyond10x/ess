//! A literal after `else:` (`{input: f, else: <literal>}`, source format `ess/16`, beyond10x/ess#163):
//! a scenario that omits the optional input asserts the literal, on the event and on the row, so an
//! implementation that stores a different default fails the suite.
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/16
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.Tier, kind: enum, variants: [Standard, Express]}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: tier, type: demo.orders.Tier}
      - {name: rank, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.Opened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: tier, type: demo.orders.Tier}
      - {name: rank, type: Integer}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Open]}
commands:
  - name: demo.orders.Open
    input:
      - {name: tier, type: Optional<demo.orders.Tier>}
      - {name: rank, type: Optional<Integer>}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {generated: true}
            tier: {input: tier, else: Express}
            rank: {input: rank, else: 3}
        sets:
          tier: {input: tier, else: Express}
          rank: {input: rank, else: 3}
views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: tier, type: demo.orders.Tier}
      - {name: rank, type: Integer}
";

fn suite(body: &str) -> Suite {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir).suite
}

/// What each scenario that invokes `Open` without `field` asserts about `field`: on the `Opened`
/// event it emits, and on the last row it requires the view to show. Keyed by scenario id.
fn asserted_when_omitted(
    suite: &Suite,
    field: &str,
) -> BTreeMap<String, (Option<Node>, Option<ScenarioValue>)> {
    let mut out = BTreeMap::new();
    for (id, scenario) in &suite.scenarios {
        let steps = &scenario.steps;
        let Some(at) = steps.iter().rposition(|step| {
            matches!(
                step,
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.orders.Open"
                        && matches!(
                            input.get(field),
                            None | Some(ScenarioValue::Literal { value: Node::Null })
                        )
            )
        }) else {
            continue;
        };
        let after = &steps[at..];
        let event = after.iter().find_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.orders.Opened" =>
            {
                Some(payload.get(field).cloned())
            }
            _ => None,
        });
        let row = after.iter().rev().find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.get(field).cloned()),
            _ => None,
        });
        out.insert(id.to_string(), (event.flatten(), row.flatten()));
    }
    out
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

#[test]
fn a_scenario_that_omits_the_optional_input_asserts_the_literal() {
    let suite = suite(MODEL);
    let tier = asserted_when_omitted(&suite, "tier");
    assert!(
        !tier.is_empty(),
        "some scenario must invoke `Open` without `tier`; have: {:?}",
        suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    for (id, (event, row)) in &tier {
        assert_eq!(event.as_ref(), Some(&text("Express")), "{id}: event");
        assert_eq!(
            row.as_ref(),
            Some(&ScenarioValue::Literal {
                value: text("Express")
            }),
            "{id}: row"
        );
    }
    let rank = asserted_when_omitted(&suite, "rank");
    assert!(!rank.is_empty());
    for (id, (event, row)) in &rank {
        assert_eq!(
            event.as_ref(),
            Some(&Node::Number(3_i64.into())),
            "{id}: event"
        );
        assert_eq!(
            row.as_ref(),
            Some(&ScenarioValue::Literal {
                value: Node::Number(3_i64.into())
            }),
            "{id}: row"
        );
    }
}

#[test]
fn a_mutant_storing_a_different_default_is_killed() {
    // The mutant stores `Standard` where the specification says `Express`. Its suite and the
    // original's ask the same scenario for different rows, so an implementation of either fails
    // the other's suite: the mutant is killed.
    let original = asserted_when_omitted(&suite(MODEL), "tier");
    let mutated_model = MODEL.replacen(
        "sets:\n          tier: {input: tier, else: Express}",
        "sets:\n          tier: {input: tier, else: Standard}",
        1,
    );
    assert_ne!(mutated_model, MODEL, "the mutation applies");
    let mutant = asserted_when_omitted(&suite(&mutated_model), "tier");
    assert!(!original.is_empty());
    let mut killed = 0;
    for (id, (_, row)) in &original {
        let Some((_, mutant_row)) = mutant.get(id) else {
            continue;
        };
        assert_eq!(
            row.as_ref(),
            Some(&ScenarioValue::Literal {
                value: text("Express")
            }),
            "{id}"
        );
        assert_eq!(
            mutant_row.as_ref(),
            Some(&ScenarioValue::Literal {
                value: text("Standard")
            }),
            "{id}"
        );
        killed += 1;
    }
    assert!(killed > 0, "no shared scenario decides the default");
}

#[test]
fn an_input_read_elsewhere_is_still_sent_and_the_fallback_asserts_what_was_sent() {
    // `requested: input.tier` reads the input plainly, so leaving it out would change what that
    // field asserts: the input stays sent, and `tier` is asserted as the value sent, not the literal.
    let model = MODEL
        .replacen(
            "      - {name: rank, type: Integer}\nactors:",
            "      - {name: rank, type: Integer}\n      - {name: requested, type: Optional<demo.orders.Tier>}\nactors:",
            1,
        )
        .replacen(
            "            rank: {input: rank, else: 3}\n        sets:",
            "            rank: {input: rank, else: 3}\n            requested: input.tier\n        sets:",
            1,
        );
    assert_eq!(model.matches("requested").count(), 2, "{model}");
    let suite = suite(&model);
    let mut seen = 0;
    for (id, scenario) in &suite.scenarios {
        let steps = &scenario.steps;
        let Some(at) = steps.iter().rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.orders.Open")
        }) else {
            continue;
        };
        let ScenarioStep::ExecuteCommand { input, .. } = &steps[at] else {
            unreachable!()
        };
        let Some(ScenarioValue::Literal { value: sent }) = input.get("tier") else {
            panic!("{id}: `tier` is read by `requested` and must be sent: {input:?}");
        };
        let event = steps[at..].iter().find_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.orders.Opened" =>
            {
                Some(payload.clone())
            }
            _ => None,
        });
        let event = event.unwrap_or_else(|| panic!("{id}: no `Opened` expectation"));
        assert_eq!(event.get("tier"), Some(sent), "{id}");
        // `rank` is read only by its fallback, so it is still left out and asserted as the literal.
        assert_eq!(input.get("rank"), None, "{id}");
        assert_eq!(event.get("rank"), Some(&Node::Number(3_i64.into())), "{id}");
        seen += 1;
    }
    assert!(seen > 0, "no scenario invokes `Open`");
}

#[test]
fn an_input_a_guard_reads_is_still_sent() {
    // `opened` is selected by `tier == Standard`, so leaving `tier` out would take the other branch:
    // the omission is dropped, and `tier` is asserted as the value the guard needed.
    let model = MODEL
        .replacen(
            "      - name: opened\n",
            "      - name: opened\n        when: tier == Standard\n",
            1,
        )
        .replacen(
            "views:\n",
            "      - name: refused\n        error: demo.orders.Refused\nviews:\n",
            1,
        )
        .replacen(
            "events:\n",
            "errors:\n  - name: demo.orders.Refused\n    fields: []\nevents:\n",
            1,
        );
    let suite = suite(&model);
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.orders.Open/outcome/opened")
        .unwrap_or_else(|| panic!("no `opened` scenario"))
        .1
        .steps;
    let at = steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .expect("the command is executed");
    let ScenarioStep::ExecuteCommand { input, .. } = &steps[at] else {
        unreachable!()
    };
    assert_eq!(
        input.get("tier"),
        Some(&ScenarioValue::Literal {
            value: text("Standard")
        }),
        "{input:?}"
    );
    let event = steps[at..]
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.orders.Opened" =>
            {
                Some(payload.clone())
            }
            _ => None,
        })
        .expect("an `Opened` expectation");
    assert_eq!(event.get("tier"), Some(&text("Standard")));
    assert_eq!(event.get("rank"), Some(&Node::Number(3_i64.into())));
}
