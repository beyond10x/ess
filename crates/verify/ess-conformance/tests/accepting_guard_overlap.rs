//! Among the accepting guarded branches of one command, the first declared whose guard holds answers
//! (beyond10x/ess#217, `docs/design/input-guard-overlap-precedence.md`).
//!
//! `small: amount < 100` and `flagged: amount > 50` both hold of `amount: 75`. `validate` accepts the
//! command, so the answer for the overlap is declared rather than refused, and synthesis holds a
//! target to it:
//!
//! 1. the first-declared branch is sent again at an input in the overlap and required there;
//! 2. a later-declared branch's witness refutes every earlier accepting guarded sibling, so a target
//!    that honours the precedence is never asked to take the later branch for an input the earlier
//!    one claims.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::{
        execute::{execute, Externals, Store},
        Interpreted,
    },
    report::Status,
    synthesize::synthesize,
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

/// The #217 shape: two accepting guarded branches overlapping over `50 < amount < 100`, beside a
/// default, and an input-guarded refusal declared last that overlaps `flagged` above 1000. `small`
/// is bounded below by `amount >= 0` so the default `validate` requires is reached by a negative
/// amount; over `Integer` the issue's two guards alone cover every value.
const ORDERS: &str = r"
format: ess/16
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields: []
    lifecycle:
      initial: Placed
      states: [Placed]
      terminal: [Placed]
      transitions: []
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.PlaceOrder]
errors:
  - name: demo.orders.NotPlaced
    summary: No order was placed.
    fields: []
  - name: demo.orders.TooLarge
    summary: The amount is above the limit.
    fields: []
events:
  - name: demo.orders.Placed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.Flagged
    fields:
      - {name: order_id, type: demo.orders.OrderId}
commands:
  - name: demo.orders.PlaceOrder
    input:
      - {name: amount, type: Integer}
    outcomes:
      - name: small
        when: {all: [amount >= 0, amount < 100]}
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Placed]
        payload: {demo.orders.Placed: {order_id: {generated: true}}}
      - name: flagged
        when: amount > 50
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Flagged]
        payload: {demo.orders.Flagged: {order_id: {generated: true}}}
      - name: refused
        error: demo.orders.NotPlaced
      - name: too-large
        when: amount > 1000
        error: demo.orders.TooLarge
";

const SMALL: &str = "demo.orders.PlaceOrder/outcome/small";
const FLAGGED: &str = "demo.orders.PlaceOrder/outcome/flagged";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// The model with `flagged` declared before `small`: its interpreter answers `flagged` in the
/// overlap, which is the target the issue asks to fail.
fn swapped(model: &str) -> String {
    let small = model
        .find("      - name: small\n")
        .expect("`small` is declared");
    let flagged = model
        .find("      - name: flagged\n")
        .expect("`flagged` is declared");
    let refused = model
        .find("      - name: refused\n")
        .expect("the default is declared");
    assert!(small < flagged && flagged < refused);
    format!(
        "{}{}{}{}",
        &model[..small],
        &model[flagged..refused],
        &model[small..flagged],
        &model[refused..]
    )
}

/// The model with a Boolean `rush` beside `amount`, and `flagged: rush == false`: no witness or
/// boundary of `small` sends `rush: false`, so only the overlap witness reaches the overlap.
fn rushed() -> String {
    let text = ORDERS
        .replace(
            "      - {name: amount, type: Integer}\n",
            "      - {name: amount, type: Integer}\n      - {name: rush, type: Boolean}\n",
        )
        .replace("when: amount > 50", "when: rush == false");
    assert_ne!(text, ORDERS);
    text
}

/// Every `PlaceOrder` invocation one scenario sends with `rush: false`, by the amount and the branch
/// it requires.
fn rushed_invocations(suite: &ConformanceSuite, id: &str) -> Vec<(Node, String)> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1;
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { input, .. } = step else {
            continue;
        };
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        if input.get("rush") != Some(&ScenarioValue::literal(Node::Bool(false))) {
            continue;
        }
        let Some(ScenarioValue::Literal { value }) = input.get("amount") else {
            panic!("`amount` is sent as a literal: {input:?}");
        };
        out.push((value.clone(), outcome.outcome.to_string()));
    }
    out
}

/// Every `PlaceOrder` amount one scenario sends, with the branch it requires.
fn invocations(suite: &ConformanceSuite, id: &str) -> Vec<(Number, String)> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1;
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "demo.orders.PlaceOrder" {
            continue;
        }
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        let Some(ScenarioValue::Literal {
            value: Node::Number(amount),
        }) = input.get("amount")
        else {
            panic!("`amount` is sent as a literal number: {input:?}");
        };
        out.push((*amount, outcome.outcome.to_string()));
    }
    out
}

fn in_overlap(amount: &Number) -> bool {
    *amount > Number::from(50_i64) && *amount < Number::from(100_i64)
}

/// The scenarios that do not pass against the interpreter of `model`.
fn failing(suite: &ConformanceSuite, model: &str) -> BTreeSet<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir(model)))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .inspect(|scenario| eprintln!("{scenario:#?}"))
        .map(|scenario| scenario.scenario.to_string())
        .collect()
}

/// #217's acceptance: an input in the overlap is sent, and `small`, the first declared, is required.
#[test]
fn issue_217_an_input_in_the_overlap_requires_the_first_declared_branch() {
    let sent = invocations(&suite_of(ORDERS), SMALL);
    assert!(
        sent.iter()
            .any(|(amount, branch)| in_overlap(amount) && branch == "small"),
        "no input in the overlap is required to take `small`: {sent:#?}"
    );
}

/// The later-declared branch is never required for an input the earlier one claims.
#[test]
fn issue_217_the_later_branch_is_witnessed_outside_the_earlier_one() {
    let sent = invocations(&suite_of(ORDERS), FLAGGED);
    assert!(sent.iter().any(|(_, branch)| branch == "flagged"));
    for (amount, branch) in &sent {
        assert!(
            branch != "flagged" || *amount >= Number::from(100_i64),
            "`flagged` is required at {amount}, which `small` claims: {sent:#?}"
        );
    }
}

/// The interpreter answers the overlap with the first-declared branch, so the model passes its own
/// suite, and the target answering with the later branch fails the first branch's scenario.
#[test]
fn issue_217_a_target_answering_with_the_later_branch_fails() {
    let suite = suite_of(ORDERS);
    assert_eq!(failing(&suite, ORDERS), BTreeSet::new());
    assert_eq!(
        failing(&suite, &swapped(ORDERS)),
        BTreeSet::from([SMALL.to_owned()])
    );
}

fn answered(model: &str, amount: i64) -> BTreeSet<String> {
    let steps = execute(
        &ir(model),
        &Store::default(),
        &QualifiedName::new("demo.orders.PlaceOrder").unwrap(),
        &BTreeMap::from([("amount".to_owned(), Node::Number(Number::from(amount)))]),
        &Externals::Withheld,
    )
    .expect("the model determines PlaceOrder");
    steps
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect()
}

/// The interpreter selects the first-declared accepting branch whose guard holds, and an
/// input-guarded refusal before any accepting branch it overlaps (beyond10x/ess#178).
#[test]
fn the_interpreter_selects_the_first_declared_accepting_branch() {
    let one = |name: &str| BTreeSet::from([format!("demo.orders.PlaceOrder/{name}")]);
    assert_eq!(answered(ORDERS, 75), one("small"));
    assert_eq!(answered(&swapped(ORDERS), 75), one("flagged"));
    assert_eq!(answered(ORDERS, 10), one("small"));
    assert_eq!(answered(ORDERS, 500), one("flagged"));
    assert_eq!(answered(ORDERS, 2000), one("too-large"));
}

/// A later-declared branch every input of which an earlier accepting branch claims is refused naming
/// that branch, rather than witnessed at an input the earlier branch answers.
#[test]
fn a_branch_the_first_declared_claims_entirely_is_refused_naming_it() {
    let text = ORDERS.replace(
        "when: amount > 50",
        "when: {all: [amount > 10, amount < 20]}",
    );
    assert_ne!(text, ORDERS);
    let synthesis = synthesize(&ir(&text));
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == FLAGGED)
        })
        .unwrap_or_else(|| panic!("`flagged` is not refused: {:#?}", synthesis.refusals));
    let rendered = format!("{} {refusal}", refusal.code());
    assert!(
        rendered.contains("ESS-SYNTH-003") && rendered.contains("small ("),
        "{rendered}"
    );
}

/// An overlap no witness or boundary of the first-declared branch reaches is still sent: `small`'s
/// scenario sends `rush: false` with an amount `small` admits and requires `small`, and the target
/// answering `flagged` fails exactly that scenario.
#[test]
fn an_overlap_over_another_field_is_sent_and_requires_the_first_declared_branch() {
    let model = rushed();
    let suite = suite_of(&model);
    let sent = rushed_invocations(&suite, SMALL);
    assert!(
        sent.iter().any(|(amount, branch)| branch == "small"
            && matches!(amount, Node::Number(n) if *n >= Number::from(0_i64) && *n < Number::from(100_i64))),
        "`small` is not sent its overlap with `flagged`: {sent:#?}"
    );
    for (amount, branch) in rushed_invocations(&suite, FLAGGED) {
        assert!(
            branch != "flagged"
                || matches!(&amount, Node::Number(n) if *n < Number::from(0_i64) || *n >= Number::from(100_i64)),
            "`flagged` is required at {amount:?}, which `small` claims"
        );
    }
    assert_eq!(failing(&suite, &model), BTreeSet::new());
    assert_eq!(
        failing(&suite, &swapped(&model)),
        BTreeSet::from([SMALL.to_owned()])
    );
}
