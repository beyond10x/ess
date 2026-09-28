//! Adversary, pass 1, for `{input: f, else: <literal>}` (source format `ess/16`, beyond10x/ess#163).
//!
//! Each case drives the implementation from what the unit wrote about itself: E4 in
//! `docs/design/value-expressions.md` and the guide say the literal after `else:` "is checked against
//! the target exactly as a literal written there is", and that a scenario omitting the input asserts
//! the literal.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// One creating command; `{payload}` fills the event's `note` (String), `{sets}` the entity's
/// fields, `{views}` is appended verbatim.
fn model(format: &str, payload: &str, sets: &str, views: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.orders
types:
  - {{name: demo.orders.OrderId, kind: newtype, of: String}}
  - {{name: demo.orders.Tier, kind: enum, variants: [Standard, Express]}}
entities:
  - name: demo.orders.Order
    identity: {{name: order_id, type: demo.orders.OrderId}}
    fields:
      - {{name: tier, type: demo.orders.Tier}}
      - {{name: rank, type: Integer}}
      - {{name: score, type: Decimal}}
      - {{name: urgent, type: Boolean}}
      - {{name: maybe, type: Optional<demo.orders.Tier>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.orders.Opened
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: note, type: String}}
actors:
  - {{name: demo.orders.Clerk, may: [demo.orders.Open]}}
commands:
  - name: demo.orders.Open
    input:
      - {{name: tier, type: Optional<demo.orders.Tier>}}
      - {{name: rank, type: Optional<Integer>}}
      - {{name: score, type: Optional<Decimal>}}
      - {{name: urgent, type: Optional<Boolean>}}
      - {{name: maybe, type: Optional<demo.orders.Tier>}}
      - {{name: note, type: Optional<String>}}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {{generated: true}}
            note: {payload}
        sets:
{sets}
{views}"
    )
}

const SETS: &str = "          tier: {input: tier, else: Express}
          rank: {input: rank, else: 3}
          score: {input: score, else: 2.5}
          urgent: {input: urgent, else: true}
          maybe: {input: maybe, else: Standard}";

const ROW: &str = "views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: tier, type: demo.orders.Tier}
      - {name: rank, type: Integer}
      - {name: score, type: Decimal}
      - {name: urgent, type: Boolean}
      - {name: maybe, type: Optional<demo.orders.Tier>}
";

fn spec(body: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(body).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("orders.yaml"), raw)]).map_err(|error| error.to_string())
}

fn suite(body: &str) -> Suite {
    let spec = spec(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir).suite
}

/// Every `(scenario id, steps after the last `Open` that omitted `field`)`.
fn omitting<'s>(suite: &'s Suite, field: &str) -> Vec<(String, &'s [ScenarioStep])> {
    let mut out = Vec::new();
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
        out.push((id.to_string(), &steps[at..]));
    }
    out
}

fn row_field(steps: &[ScenarioStep], view: &str, field: &str) -> Option<ScenarioValue> {
    steps.iter().rev().find_map(|step| match step {
        ScenarioStep::ExpectView {
            view: name,
            expectation: ViewExpectation::Contains { fields },
        } if name.to_string() == view => Some(fields.get(field).cloned()),
        _ => None,
    })?
}

/// The E4 text: the literal after `else:` is "held to the rule a literal written in the target's
/// place is". In a payload, a literal that spells one of the command's inputs is refused as
/// `misspelled_reference` (`check_payload_literal`); after `else:` the same text compiles as the
/// literal `rank`.
#[test]
fn a_payload_fallback_literal_naming_an_input_is_refused_as_a_top_level_one_is() {
    let top = model("ess/16", "rank", SETS, "");
    let error = spec(&top)
        .err()
        .unwrap_or_else(|| panic!("control: a top-level `note: rank` must be refused"));
    assert!(error.contains("misspelled_reference"), "control: {error}");

    let fallback = model("ess/16", "{input: note, else: rank}", SETS, "");
    let error = spec(&fallback).err().unwrap_or_else(|| {
        panic!(
            "`note: {{input: note, else: rank}}` compiled; the top-level `note: rank` is refused \
             as misspelled_reference, and E4 says the literal is held to that rule"
        )
    });
    assert!(error.contains("misspelled_reference"), "{error}");
}

/// Same parity, for the near-miss `inptu.<field>` the top-level payload rule refuses.
#[test]
fn a_payload_fallback_literal_misspelling_input_dot_is_refused_as_a_top_level_one_is() {
    let top = model("ess/16", "inptu.rank", SETS, "");
    let error = spec(&top)
        .err()
        .unwrap_or_else(|| panic!("control: a top-level `note: inptu.rank` must be refused"));
    assert!(error.contains("misspelled_reference"), "control: {error}");

    let fallback = model("ess/16", "{input: note, else: inptu.rank}", SETS, "");
    let error = spec(&fallback)
        .err()
        .unwrap_or_else(|| panic!("`note: {{input: note, else: inptu.rank}}` compiled"));
    assert!(error.contains("misspelled_reference"), "{error}");
}

/// The `ess/16` gate is enforced for a literal fallback written in `sets:` alone; the unit's own
/// test writes one in the payload too, so a gate on one place only would pass it.
#[test]
fn a_sets_only_literal_fallback_is_refused_below_ess_16() {
    for format in ["ess/14", "ess/15"] {
        let body = model(
            format,
            "'fixed'",
            "          tier: {input: tier, else: Express}
          rank: 0
          score: 0.5
          urgent: false
          maybe: Standard",
            "",
        );
        let error = spec(&body)
            .err()
            .unwrap_or_else(|| panic!("{format}: a `sets:`-only literal fallback compiled"));
        assert!(
            error.contains("unsupported_format_version")
                && error.contains("a literal after `else:` requires specification format ess/16"),
            "{format}: {error}"
        );
    }
}

/// Decimal, Boolean and an `Optional` enum target: an omitting scenario asserts each literal read
/// as the target's type, not as text.
#[test]
fn decimal_boolean_and_optional_targets_assert_the_typed_literal() {
    let suite = suite(&model("ess/16", "'fixed'", SETS, ROW));
    let cases: [(&str, Node); 4] = [
        (
            "score",
            Node::Number(ess_primitives::facts::Number::decimal_literal("2.5").expect("2.5")),
        ),
        ("urgent", Node::Bool(true)),
        ("maybe", Node::Text("Standard".to_owned())),
        ("rank", Node::Number(3_i64.into())),
    ];
    for (field, expected) in cases {
        let seen = omitting(&suite, field);
        assert!(!seen.is_empty(), "no scenario omits `{field}`");
        for (id, steps) in seen {
            assert_eq!(
                row_field(steps, "demo.orders.OrderRow", field),
                Some(ScenarioValue::Literal {
                    value: expected.clone()
                }),
                "{id}: `{field}`"
            );
        }
    }
}

/// A view filtered on the fallback's field: a scenario that omits `tier` leaves a row holding
/// `Express`, so it must never require that row in a view that shows `Standard` rows only, nor
/// exclude it from one that shows `Express` rows.
#[test]
fn a_filtered_view_is_decided_by_the_literal_not_by_the_absent_input() {
    let views = format!(
        "{ROW}  - name: demo.orders.StandardRows
    source: demo.orders.Order
    consistency: read_your_writes
    filter: tier == Standard
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: tier, type: demo.orders.Tier}}
  - name: demo.orders.ExpressRows
    source: demo.orders.Order
    consistency: read_your_writes
    filter: tier == Express
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: tier, type: demo.orders.Tier}}
"
    );
    let suite = suite(&model("ess/16", "'fixed'", SETS, &views));
    let seen = omitting(&suite, "tier");
    assert!(!seen.is_empty(), "no scenario omits `tier`");
    for (id, steps) in seen {
        for step in steps {
            if let ScenarioStep::ExpectView { view, expectation } = step {
                let view = view.to_string();
                let wrong = match expectation {
                    ViewExpectation::Contains { .. } => view == "demo.orders.StandardRows",
                    ViewExpectation::Excludes { .. } => view == "demo.orders.ExpressRows",
                    _ => false,
                };
                assert!(!wrong, "{id}: {view} {expectation:?}");
            }
        }
    }
}

/// An `updates:` outcome on an arranged row: omitting the input asserts the literal, which differs
/// from what the arrangement wrote, so an implementation that keeps the old value is killed too.
#[test]
fn an_updating_outcome_that_omits_the_input_asserts_the_literal_over_the_arranged_value() {
    let body = model("ess/16", "'fixed'", SETS, ROW)
        .replacen(
            "  - {name: demo.orders.Clerk, may: [demo.orders.Open]}",
            "  - {name: demo.orders.Clerk, may: [demo.orders.Open, demo.orders.Retier]}",
            1,
        )
        .replacen(
            "views:\n",
            "  - name: demo.orders.Retier
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: tier, type: Optional<demo.orders.Tier>}
    outcomes:
      - name: retiered
        updates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Retiered]
        payload:
          demo.orders.Retiered:
            order_id: input.order_id
        sets:
          tier: {input: tier, else: Standard}
views:\n",
            1,
        )
        .replacen(
            "actors:\n",
            "  - name: demo.orders.Retiered
    fields:
      - {name: order_id, type: demo.orders.OrderId}
actors:\n",
            1,
        );
    let suite = suite(&body);
    let (id, scenario) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.orders.Retier/outcome/retiered")
        .unwrap_or_else(|| {
            panic!(
                "no retiered scenario: {:?}",
                suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        });
    let steps = &scenario.steps;
    let at = steps
        .iter()
        .rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.orders.Retier")
        })
        .expect("Retier is executed");
    let ScenarioStep::ExecuteCommand { input, .. } = &steps[at] else {
        unreachable!()
    };
    assert_eq!(
        input.get("tier"),
        None,
        "{id}: `tier` should be left out: {input:?}"
    );
    assert_eq!(
        row_field(&steps[at..], "demo.orders.OrderRow", "tier"),
        Some(ScenarioValue::Literal {
            value: Node::Text("Standard".to_owned())
        }),
        "{id}"
    );
}
