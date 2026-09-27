//! Adversary, pass 2, for `{input: f, else: <literal>}` (source format `ess/16`, beyond10x/ess#163).
//!
//! `{input: f, else: <literal>}` means two things: the input when the caller sent it, the literal
//! when it did not. Pass 1 attacked the second half; these cases attack the first half, what the
//! omission rule in `run` leaves asserted, and the refusal texts of a `sets:` fallback.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// One creating command. `{payload}` is the source of the event's `tier`, `{sets}` the entity's
/// fields.
fn model(payload: &str, sets: &str) -> String {
    format!(
        "format: ess/16
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
      - {{name: label, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.orders.Opened
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: tier, type: demo.orders.Tier}}
actors:
  - {{name: demo.orders.Clerk, may: [demo.orders.Open]}}
commands:
  - name: demo.orders.Open
    input:
      - {{name: tier, type: Optional<demo.orders.Tier>}}
      - {{name: rank, type: Optional<Integer>}}
      - {{name: label, type: Optional<String>}}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {{generated: true}}
            tier: {payload}
        sets:
{sets}
views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: tier, type: demo.orders.Tier}}
      - {{name: rank, type: Integer}}
      - {{name: label, type: String}}
"
    )
}

const SETS: &str = "          tier: {input: tier, else: Express}
          rank: {input: rank, else: 3}
          label: {input: label, else: 'none yet'}";

fn spec(body: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(body).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("orders.yaml"), raw)]).map_err(|error| error.to_string())
}

fn suite(body: &str) -> Suite {
    let spec = spec(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir).suite
}

/// For every `Open` invocation that sends `field` as a value, whether some expectation after it
/// (the `Opened` event or a view row) asserts that same value for `field`.
fn sent_and_asserted(suite: &Suite, field: &str) -> (usize, usize) {
    let (mut sent, mut asserted) = (0, 0);
    for scenario in suite.scenarios.values() {
        let steps = &scenario.steps;
        for (at, step) in steps.iter().enumerate() {
            let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                continue;
            };
            if command.to_string() != "demo.orders.Open" {
                continue;
            }
            let Some(ScenarioValue::Literal { value }) = input.get(field) else {
                continue;
            };
            if *value == Node::Null {
                continue;
            }
            sent += 1;
            let seen = steps[at..].iter().any(|step| match step {
                ScenarioStep::ExpectEvent { event, payload, .. } => {
                    event.to_string() == "demo.orders.Opened" && payload.get(field) == Some(value)
                }
                ScenarioStep::ExpectView {
                    expectation: ViewExpectation::Contains { fields },
                    ..
                } => {
                    fields.get(field)
                        == Some(&ScenarioValue::Literal {
                            value: value.clone(),
                        })
                }
                _ => false,
            });
            if seen {
                asserted += 1;
            }
        }
    }
    (sent, asserted)
}

/// The acceptance's mutant is "a different default". The other mutant of the same construct is an
/// implementation that ignores the input and always stores the literal: `sets: {tier: Express}`.
/// Before the omission rule every witness sent `tier` and asserted it on the event and the row;
/// after it no generated scenario sends `tier` at all, so the input half of `{input: tier, else:
/// Express}` is asserted nowhere and the ignore-the-input mutant passes the whole suite.
#[test]
fn some_scenario_sends_the_input_a_literal_fallback_reads_and_asserts_it_wins() {
    let body = model("{input: tier, else: Express}", SETS);
    let suite = suite(&body);
    for field in ["tier", "rank", "label"] {
        let (sent, asserted) = sent_and_asserted(&suite, field);
        assert!(
            asserted > 0,
            "`{field}`: {sent} invocation(s) of `Open` send it and none asserts the sent value \
             wins over the `else:` literal, so an implementation that always stores the literal \
             passes; scenarios: {:?}",
            suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
    }
}

/// The same mutant, measured as a kill: the suite of `{input: tier, else: Express}` and the suite
/// of the ignore-the-input specification `tier: Express` must disagree somewhere on `tier`.
#[test]
fn the_always_the_literal_mutant_is_killed() {
    let original = suite(&model("{input: tier, else: Express}", SETS));
    let mutant = suite(&model(
        "Express",
        "          tier: Express
          rank: {input: rank, else: 3}
          label: {input: label, else: 'none yet'}",
    ));
    let asserted = |suite: &Suite| {
        let mut out = Vec::new();
        for (id, scenario) in &suite.scenarios {
            for step in &scenario.steps {
                match step {
                    ScenarioStep::ExecuteCommand { command, input, .. }
                        if command.to_string() == "demo.orders.Open" =>
                    {
                        out.push((id.to_string(), "sent", format!("{:?}", input.get("tier"))));
                    }
                    ScenarioStep::ExpectEvent { event, payload, .. }
                        if event.to_string() == "demo.orders.Opened" =>
                    {
                        out.push((
                            id.to_string(),
                            "event",
                            format!("{:?}", payload.get("tier")),
                        ));
                    }
                    ScenarioStep::ExpectView {
                        expectation: ViewExpectation::Contains { fields },
                        ..
                    } => out.push((id.to_string(), "row", format!("{:?}", fields.get("tier")))),
                    _ => {}
                }
            }
        }
        out
    };
    // Only what the original suite asks of an implementation matters: run against the mutant
    // implementation, does any original scenario send a `tier` and expect it back?
    let (sent, wins) = sent_and_asserted(&original, "tier");
    assert!(
        wins > 0,
        "no scenario of the original suite ({sent} sending `tier`) distinguishes it from the \
         always-`Express` mutant; original: {:?}\nmutant: {:?}",
        asserted(&original),
        asserted(&mutant)
    );
}

/// A `{generated: true}` fallback on the same input keeps it sent (it pushes `literal: false`), and
/// the literal fallback beside it then asserts the sent value, not the literal.
#[test]
fn a_generated_fallback_on_the_same_input_keeps_it_sent() {
    let suite = suite(&model("{input: tier, else: {generated: true}}", SETS));
    let (sent, asserted) = sent_and_asserted(&suite, "tier");
    assert!(
        sent > 0,
        "`tier` is read by a generated fallback and must be sent"
    );
    assert!(asserted > 0, "{sent} sent, none asserted");
}

/// A bare `sets:` literal is refused naming the entity's field (`demo.orders.Order.rank`); the same
/// literal after `else:` goes through `refuse_literal` with the command as owner and names the
/// command's input `demo.orders.Open.rank`, which is not what the literal fills.
#[test]
fn a_sets_fallback_refusal_names_the_entity_field_as_a_bare_sets_literal_does() {
    let bare = spec(&model(
        "{input: tier, else: Express}",
        "          tier: {input: tier, else: Express}
          rank: 2.5
          label: {input: label, else: 'none yet'}",
    ))
    .expect_err("2.5 is no Integer");
    let fallback = spec(&model(
        "{input: tier, else: Express}",
        "          tier: {input: tier, else: Express}
          rank: {input: rank, else: 2.5}
          label: {input: label, else: 'none yet'}",
    ))
    .expect_err("2.5 is no Integer");
    assert!(
        bare.contains("`demo.orders.Order.rank`"),
        "bare refusal names the entity field: {bare}"
    );
    assert!(
        fallback.contains("`demo.orders.Order.rank`")
            && !fallback.contains("demo.orders.Open.rank"),
        "the fallback refusal must name what the literal fills, as the bare one does\nbare:     \
         {bare}\nfallback: {fallback}"
    );
}

/// The repair a fallback refusal names must be one the language admits. `{input: tier, else:
/// rank}` is refused (pass 1's correction) with the bare-literal hint "write `input.rank` to read
/// the input", and `else: input.rank` is itself refused: `else:` admits a literal only.
#[test]
fn the_hint_on_a_payload_fallback_naming_an_input_is_a_spelling_that_compiles() {
    let error = spec(&model("{input: tier, else: rank}", SETS))
        .expect_err("`else: rank` names an input without its prefix");
    if error.contains("write `input.rank`") {
        let followed = spec(&model("{input: tier, else: input.rank}", SETS));
        assert!(
            followed.is_ok(),
            "the hint says write `input.rank`; doing so is refused:\nhint:     {error}\nfollowed: \
             {}",
            followed.err().unwrap_or_default()
        );
    }
}
