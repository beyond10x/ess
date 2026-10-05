//! A wrong-state witness may miss a sibling branch that needs both an input guard and a subject
//! guard by falsifying either (beyond10x/ess#192).
//!
//! `Confirm` moves an `Order` from `Open` and declares three siblings of that move:
//! `already-confirmed` (`when_subject: history == Confirmed` beside `when: token != ""`),
//! `token-required` (`when: token == ""`, an input-guarded refusal) and `gone` (`wrong_state`).
//! The two input guards are complementary, so no input refutes both; the witness for
//! `state/Closed/refuses/Confirm` sends `token != ""`, which misses `token-required` through its
//! input, and arranges a row whose `history` is not `Confirmed`, which misses `already-confirmed`
//! through its subject guard.
//!
//! A branch guarded by an input alone is still missed only through its input, and a mixed branch
//! whose subject guard the arranged row cannot falsify is missed only through its input as well:
//! where neither is possible the scenario is refused, because a target could then take either
//! `already-confirmed` or `gone` and the specification does not say which.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Synthesis},
    ConformanceScenario, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// The shape of #192: the row's `history` is chosen by the creating command's input, so the
/// arrangement can leave it at either variant.
const ORDERS: &str = r#"format: ess/16
system: demo
version: v1
domain: demo.orders
summary: An order confirmed with a token, refused without one, and gone once it has left Open.
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
  - {name: demo.orders.History, kind: enum, variants: [Pending, Confirmed]}
entities:
  - name: demo.orders.Order
    identity: {name: id, type: demo.orders.OrderId}
    fields:
      - {name: history, type: demo.orders.History}
    lifecycle:
      initial: Open
      states: [Open, Done, Closed]
      terminal: [Closed]
      transitions:
        - {name: confirm, from: [Open], to: Done}
        - {name: close, from: [Open, Done], to: Closed}
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.Place, demo.orders.Confirm, demo.orders.Close]
errors:
  - {name: demo.orders.TokenRequired, summary: The token is empty., fields: []}
  - {name: demo.orders.NoSuchOrder, summary: The order has left Open., fields: []}
events:
  - name: demo.orders.Placed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Confirmed
    fields: [{name: id, type: demo.orders.OrderId}]
  - name: demo.orders.Closed
    fields: [{name: id, type: demo.orders.OrderId}]
commands:
  - name: demo.orders.Place
    input: [{name: history, type: demo.orders.History}]
    outcomes:
      - name: placed
        creates: demo.orders.Order
        instance: id
        sets: {history: input.history}
        emits: [demo.orders.Placed]
        payload: {demo.orders.Placed: {id: {generated: true}}}
  - name: demo.orders.Confirm
    input:
      - {name: id, type: demo.orders.OrderId}
      - {name: token, type: String}
    outcomes:
      - name: confirmed
        moves: demo.orders.Order.confirm
        instance: id
        emits: [demo.orders.Confirmed]
        payload: {demo.orders.Confirmed: {id: input.id}}
      - name: already-confirmed
        when_subject: {field: history, equals: Confirmed}
        when: token != ""
        preserves: demo.orders.Order
        instance: id
      - name: token-required
        when: token == ""
        error: demo.orders.TokenRequired
      - name: gone
        wrong_state: true
        error: demo.orders.NoSuchOrder
  - name: demo.orders.Close
    input: [{name: id, type: demo.orders.OrderId}]
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: id
        emits: [demo.orders.Closed]
        payload: {demo.orders.Closed: {id: input.id}}
views:
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.orders.OrderId}
      - {name: history, type: demo.orders.History}
      - {name: state, type: demo.orders.Order.State}
"#;

const CLOSED: &str = "demo.orders.Order/state/Closed/refuses/demo.orders.Confirm";
const DONE: &str = "demo.orders.Order/state/Done/refuses/demo.orders.Confirm";
const GONE: &str = "demo.orders.Confirm/gone";

/// Every order is placed already confirmed: no arrangement can make `already-confirmed`'s subject
/// guard false.
fn forced(text: &str) -> String {
    let forced = text
        .replace(
            "    input: [{name: history, type: demo.orders.History}]\n",
            "    input: []\n",
        )
        .replace(
            "        sets: {history: input.history}\n",
            "        sets: {history: Confirmed}\n",
        );
    assert_ne!(forced, text);
    forced
}

/// `token-required` removed: nothing claims `token == ""`.
fn without_token_required(text: &str) -> String {
    let without = text.replace(
        "      - name: token-required\n        when: token == \"\"\n        error: demo.orders.TokenRequired\n",
        "",
    );
    assert_ne!(without, text);
    without
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; refusals: {:#?}",
                    refusals_about(result, id)
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The input of every invocation of `command` in the scenario, in step order.
fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

fn text(value: &ScenarioValue) -> Option<String> {
    value
        .as_literal()
        .and_then(Node::as_text)
        .map(str::to_owned)
}

/// The token of the invocation under test: the last `Confirm`, after any the arrangement sent to
/// reach `Done`.
fn token(scenario: &ConformanceScenario) -> String {
    let sent = inputs(scenario, "demo.orders.Confirm");
    let Some(only) = sent.last() else {
        panic!("no Confirm invocation");
    };
    only.get("token")
        .and_then(text)
        .unwrap_or_else(|| panic!("Confirm sends a literal token: {only:#?}"))
}

fn placed_history(scenario: &ConformanceScenario) -> Vec<String> {
    inputs(scenario, "demo.orders.Place")
        .iter()
        .map(|input| {
            input
                .get("history")
                .and_then(text)
                .unwrap_or_else(|| panic!("Place sends a literal history: {input:#?}"))
        })
        .collect()
}

/// Whether a view step before the invocation under test requires the row's `history` at
/// `Pending`: the fact the witness relies on is observed, not assumed.
fn observes_pending_history(scenario: &ConformanceScenario) -> bool {
    let last = scenario
        .steps
        .iter()
        .rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.orders.Confirm")
        })
        .expect("a Confirm invocation");
    scenario.steps[..last].iter().any(|step| {
        matches!(
            step,
            ScenarioStep::EventuallyView { .. }
                | ScenarioStep::QueryView { .. }
                | ScenarioStep::ExpectView { .. }
        ) && {
            let shown = format!("{step:?}");
            shown.contains("\"history\"") && shown.contains("\"Pending\"")
        }
    })
}

fn requires(scenario: &ConformanceScenario, outcome: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

#[test]
fn the_issue_shape_synthesizes_its_wrong_state_scenarios() {
    let result = synthesis(ORDERS);
    for id in [CLOSED, DONE] {
        assert_eq!(refusals_about(&result, id), Vec::<String>::new(), "{id}");
        let witness = scenario(&result, id);
        assert!(requires(witness, GONE), "{id} requires `gone`");
    }
}

#[test]
fn the_witness_misses_token_required_by_its_input_and_already_confirmed_by_its_subject() {
    let result = synthesis(ORDERS);
    for id in [CLOSED, DONE] {
        let witness = scenario(&result, id);
        assert_ne!(
            token(witness),
            "",
            "{id}: `token != \"\"` misses token-required"
        );
        let history = placed_history(witness);
        assert!(!history.is_empty(), "{id}: the row is placed");
        assert!(
            history.iter().all(|value| value != "Confirmed"),
            "{id}: the arranged row falsifies already-confirmed's subject guard: {history:?}"
        );
        assert!(
            observes_pending_history(witness),
            "{id}: history is observed before Confirm"
        );
    }
}

#[test]
fn the_row_is_arranged_away_from_the_subject_guard_whichever_variant_is_declared_first() {
    let reversed = ORDERS.replace(
        "variants: [Pending, Confirmed]",
        "variants: [Confirmed, Pending]",
    );
    assert_ne!(reversed, ORDERS);
    let result = synthesis(&reversed);
    for id in [CLOSED, DONE] {
        assert_eq!(refusals_about(&result, id), Vec::<String>::new(), "{id}");
        let witness = scenario(&result, id);
        assert_ne!(token(witness), "", "{id}");
        let history = placed_history(witness);
        assert!(
            !history.is_empty() && history.iter().all(|value| value != "Confirmed"),
            "{id}: {history:?}"
        );
        assert!(
            observes_pending_history(witness),
            "{id}: history is observed before Confirm"
        );
        assert!(requires(witness, GONE), "{id} requires `gone`");
    }
}

#[test]
fn a_mixed_sibling_whose_subject_guard_holds_is_missed_through_its_input() {
    // The reverse: every row is confirmed, so `already-confirmed` is missed only by `token == ""`,
    // and with no refusal claiming that input the witness sends it.
    let result = synthesis(&forced(&without_token_required(ORDERS)));
    for id in [CLOSED, DONE] {
        assert_eq!(refusals_about(&result, id), Vec::<String>::new(), "{id}");
        let witness = scenario(&result, id);
        assert_eq!(token(witness), "", "{id}");
        assert!(requires(witness, GONE), "{id} requires `gone`");
    }
}

#[test]
fn a_mixed_sibling_the_state_forces_leaves_the_scenario_refused_rather_than_order_dependent() {
    // Every row is confirmed and `token-required` claims `token == ""`: `token != ""` reaches
    // `already-confirmed` as much as `gone`, and `token == ""` reaches `token-required`. No
    // witness keeps the state the only thing the scenario varies, so it is refused and names both
    // guards the input could not escape.
    let result = synthesis(&forced(ORDERS));
    // `Done` is unreachable here by construction: `confirmed` needs an input neither sibling claims.
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == CLOSED),
        "{CLOSED} is not written"
    );
    let refused = refusals_about(&result, CLOSED);
    let [only] = refused.as_slice() else {
        panic!("{CLOSED}: one refusal, found {refused:#?}");
    };
    assert!(only.starts_with("ESS-SYNTH-003"), "{only}");
    assert!(only.contains(r#"token != """#), "{only}");
    assert!(only.contains(r#"token == """#), "{only}");
}

#[test]
fn an_input_alone_guard_is_still_missed_only_through_its_input() {
    // `token-required` has no subject half, so no arrangement stands in for refuting it: every
    // wrong-state witness sends a token it refutes, in the plain shape and the reversed one alike.
    for text in [ORDERS.to_owned(), forced(&without_token_required(ORDERS))] {
        let result = synthesis(&text);
        for id in [CLOSED, DONE] {
            let witness = scenario(&result, id);
            if text.contains("token-required") {
                assert_ne!(token(witness), "", "{id}");
            }
        }
    }
    // And a refusal guarded by its input alone that overlaps the moving branch leaves nothing: the
    // row cannot rescue `token-required`.
    let only_refusal = ORDERS.replace(
        "      - name: already-confirmed\n        when_subject: {field: history, equals: Confirmed}\n        when: token != \"\"\n        preserves: demo.orders.Order\n        instance: id\n",
        "      - name: already-confirmed\n        when: token != \"\"\n        error: demo.orders.NoSuchOrder\n",
    );
    assert_ne!(only_refusal, ORDERS);
    let result = synthesis(&only_refusal);
    for id in [CLOSED, DONE] {
        assert!(
            !result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id),
            "{id}: two complementary input-only guards leave no wrong-state witness"
        );
    }
}

// ---- siblings selected by the stored row alone ---------------------------------------------------
//
// A guarded branch is selected in any state before `wrong_state` applies (coordinator ruling,
// beyond10x/ess#192; Entity Runtime orders guarded branches before the wrong-state one), so a
// sibling guarded by the row alone is missed only through the row.

/// `held: when_subject: history == Pending`, a refusal selected by the stored row alone.
const HELD: &str = "      - name: held\n        when_subject: {predicate: 'history == Pending'}\n        error: demo.orders.Held\n";

fn with_held(text: &str) -> String {
    let held = text
        .replace(
            "      - name: gone\n        wrong_state: true\n",
            &format!("{HELD}      - name: gone\n        wrong_state: true\n"),
        )
        .replace(
            "errors:\n",
            "errors:\n  - {name: demo.orders.Held, summary: The order is held., fields: []}\n",
        );
    assert_ne!(held, text);
    held
}

/// Only `held` beside the move and `gone`: no input guard anywhere.
fn held_alone(variants: &str) -> String {
    let text = with_held(&without_token_required(ORDERS))
        .replace(
            "      - name: already-confirmed\n        when_subject: {field: history, equals: Confirmed}\n        when: token != \"\"\n        preserves: demo.orders.Order\n        instance: id\n",
            "",
        )
        .replace(
            "variants: [Pending, Confirmed]",
            &format!("variants: {variants}"),
        );
    assert!(!text.contains("already-confirmed"));
    text
}

#[test]
fn a_row_only_sibling_is_missed_through_the_row_whichever_variant_is_declared_first() {
    for variants in ["[Pending, Confirmed]", "[Confirmed, Pending]"] {
        let result = synthesis(&held_alone(variants));
        let refused = refusals_about(&result, CLOSED);
        assert_eq!(refused, Vec::<String>::new(), "{variants}");
        let witness = scenario(&result, CLOSED);
        let history = placed_history(witness);
        assert!(
            !history.is_empty() && history.iter().all(|value| value == "Confirmed"),
            "{variants}: the row refutes held: {history:?}"
        );
        assert!(requires(witness, GONE), "{variants}: requires `gone`");
        let observed = witness.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::QueryView { .. }
                    | ScenarioStep::ExpectView { .. }
                    | ScenarioStep::EventuallyView { .. }
            ) && format!("{step:?}").contains("\"Confirmed\"")
        });
        assert!(observed, "{variants}: history is observed before Confirm");
    }
}

#[test]
fn no_row_refuting_every_subject_guard_refuses_the_scenario_and_names_them() {
    // `held` claims every Pending row, `already-confirmed` every Confirmed row under
    // `token != ""`, and `token-required` claims `token == ""`: no row and input miss all three.
    let result = synthesis(&with_held(ORDERS));
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == CLOSED),
        "{CLOSED} is not written"
    );
    let refused = refusals_about(&result, CLOSED);
    let [only] = refused.as_slice() else {
        panic!("{CLOSED}: one refusal, found {refused:#?}");
    };
    assert!(only.starts_with("ESS-SYNTH-003"), "{only}");
    assert!(only.contains("history == Pending"), "{only}");
    assert!(only.contains(r#"token == """#), "{only}");
}

#[test]
fn interpreted_executes_the_original_mixed_guard_shape() {
    let model = ir(ORDERS);
    let suite = synthesize(&model).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    assert_ne!(report.scenarios.len(), 0);
    for run in report.scenarios {
        assert_eq!(
            run.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            run.scenario,
            run.checks
        );
    }
}
