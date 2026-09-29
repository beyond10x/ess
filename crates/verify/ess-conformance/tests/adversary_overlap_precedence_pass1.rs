//! Adversary pass 1 against story:overlapping-accepting-guards-have-declared-precedence
//! (beyond10x/ess#217): the declared precedence among accepting guarded branches, attacked with
//! three overlapping branches, an overlap the witness ladder may not reach, an undecided later
//! guard, and an external branch declared after an accepting guard.
#![allow(clippy::too_many_lines, clippy::for_kv_map)]

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::{
        execute::{execute, Externals, Store},
        Interpreted,
    },
    report::Status,
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const HEAD: &str = r"
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
  - name: demo.orders.Declined
    summary: The provider declined.
    fields: []
events:
  - name: demo.orders.Placed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.Flagged
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.Bulk
    fields:
      - {name: order_id, type: demo.orders.OrderId}
commands:
  - name: demo.orders.PlaceOrder
";

fn branch(name: &str, when: &str, event: &str) -> String {
    format!(
        "      - name: {name}\n        when: {when}\n        creates: demo.orders.Order\n        instance: order_id\n        emits: [demo.orders.{event}]\n        payload: {{demo.orders.{event}: {{order_id: {{generated: true}}}}}}\n"
    )
}

const DEFAULT: &str = "      - name: refused\n        error: demo.orders.NotPlaced\n";

fn model(inputs: &str, branches: &[String]) -> String {
    format!(
        "{HEAD}    input:\n{inputs}    outcomes:\n{}{DEFAULT}",
        branches.concat()
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn failing(suite: &ConformanceSuite, target_model: &str) -> BTreeSet<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir(target_model)))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect()
}

/// Every `PlaceOrder` invocation one scenario sends, with the branch it requires.
fn invocations(
    suite: &ConformanceSuite,
    id: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, String)> {
    let Some(scenario) = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "demo.orders.PlaceOrder" {
            continue;
        }
        let required = match steps.peek() {
            Some(ScenarioStep::ExpectOutcome { outcome }) => outcome.outcome.to_string(),
            _ => "(none)".to_owned(),
        };
        out.push((
            input
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            required,
        ));
    }
    out
}

fn number(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<Number> {
    match input.get(field) {
        Some(ScenarioValue::Literal {
            value: Node::Number(number),
        }) => Some(*number),
        _ => None,
    }
}

const TWO_INTEGERS: &str =
    "      - {name: amount, type: Integer}\n      - {name: qty, type: Integer}\n";

/// Three accepting branches that overlap pairwise, beside a default: `a` over `0 <= amount < 100`,
/// `b` over `amount > 50`, `c` over `qty > 5`. Every order of the three is a different target — the
/// last-declared-wins mutant is the reversed order, a specificity mutant is one of the others — and
/// every one of them answers some input differently from the model. The suite has to fail each.
#[test]
fn every_reordering_of_three_overlapping_accepting_branches_fails_the_suite() {
    let a = branch("a", "{all: [amount >= 0, amount < 100]}", "Placed");
    let b = branch("b", "amount > 50", "Flagged");
    let c = branch("c", "qty > 5", "Bulk");
    let declared = model(TWO_INTEGERS, &[a.clone(), b.clone(), c.clone()]);
    let result = synthesis(&declared);
    assert!(
        result.refusals.is_empty(),
        "every branch is reachable: {:#?}",
        result.refusals
    );
    assert_eq!(failing(&result.suite, &declared), BTreeSet::new());
    let orders: [(&str, [&String; 3]); 5] = [
        ("a c b", [&a, &c, &b]),
        ("b a c", [&b, &a, &c]),
        ("b c a", [&b, &c, &a]),
        ("c a b", [&c, &a, &b]),
        ("c b a (last declared wins)", [&c, &b, &a]),
    ];
    let mut survivors = Vec::new();
    for (label, order) in orders {
        let mutant = model(TWO_INTEGERS, &order.map(Clone::clone));
        if failing(&result.suite, &mutant).is_empty() {
            survivors.push(label);
        }
    }
    assert!(
        survivors.is_empty(),
        "reorderings the suite does not fail: {survivors:?}"
    );
}

/// Two Decimal branches overlapping over `11 < amount < 12` only, beside a default. The overlap is
/// decidable and nonempty. Either the first-declared branch is sent an input in it and required
/// there, or synthesis says, as a refusal or a note, that it was not — never nothing at all.
#[test]
fn a_narrow_decimal_overlap_is_witnessed_or_reported() {
    let text = model(
        "      - {name: amount, type: Decimal}\n",
        &[
            branch("small", "{all: [amount > 10, amount < 12]}", "Placed"),
            branch("flagged", "{all: [amount > 11, amount < 13]}", "Flagged"),
        ],
    );
    let result = synthesis(&text);
    let sent = invocations(&result.suite, "demo.orders.PlaceOrder/outcome/small");
    let witnessed = sent.iter().any(|(input, required)| {
        (required == "small" || required.ends_with("/small"))
            && number(input, "amount").is_some_and(|amount| {
                amount > Number::from(11_i64) && amount < Number::from(12_i64)
            })
    });
    let reported = result
        .refusals
        .iter()
        .map(ToString::to_string)
        .chain(result.notes.iter().map(ToString::to_string))
        .any(|text| text.contains("small") && text.contains("flagged"));
    assert!(
        witnessed || reported,
        "the overlap of `small` and `flagged` is neither sent nor reported.\nsent to small: \
         {sent:#?}\nrefusals: {:#?}\nnotes: {:#?}",
        result.refusals,
        result.notes
    );
    // And the swapped target, which answers `flagged` over the overlap, fails.
    let swapped = model(
        "      - {name: amount, type: Decimal}\n",
        &[
            branch("flagged", "{all: [amount > 11, amount < 13]}", "Flagged"),
            branch("small", "{all: [amount > 10, amount < 12]}", "Placed"),
        ],
    );
    if witnessed {
        assert!(!failing(&result.suite, &swapped).is_empty());
    }
}

/// The declared precedence answers `amount: 5` with `a` whatever `b` reads: `a` is declared first
/// and holds. The interpreter still evaluates `b` over an absent optional and answers nothing.
#[test]
fn the_interpreter_answers_the_first_declared_branch_when_a_later_guard_is_undecided() {
    let text = model(
        "      - {name: amount, type: Integer}\n      - {name: note, type: Optional<String>}\n",
        &[
            branch("a", "amount > 0", "Placed"),
            branch("b", "note == \"rush\"", "Flagged"),
        ],
    );
    let steps = execute(
        &ir(&text),
        &Store::default(),
        &QualifiedName::new("demo.orders.PlaceOrder").unwrap(),
        &BTreeMap::from([("amount".to_owned(), Node::Number(Number::from(5_i64)))]),
        &Externals::Withheld,
    );
    let answered: Result<BTreeSet<String>, String> = steps
        .map(|steps| {
            steps
                .iter()
                .map(|step| {
                    step.outcome
                        .as_ref()
                        .map_or("none".to_owned(), ToString::to_string)
                })
                .collect()
        })
        .map_err(|error| error.to_string());
    assert_eq!(
        answered,
        Ok(BTreeSet::from(["demo.orders.PlaceOrder/a".to_owned()]))
    );
}

/// An external refusal declared after an accepting guarded branch. Entity Runtime orders both in
/// the same category by source position, so an input `small` claims takes `small` there whatever
/// the provider says. The inject-fault scenario for `declined` must therefore send an input `small`
/// does not claim, or the target answering by the declared order fails the model's own suite.
#[test]
fn an_external_branch_after_an_accepting_guard_is_sent_an_input_the_guard_does_not_claim() {
    let text = model(
        "      - {name: amount, type: Integer}\n",
        &[
            branch("small", "{all: [amount >= 0, amount < 100]}", "Placed"),
            "      - name: declined\n        external: the provider declines\n        error: demo.orders.Declined\n".to_owned(),
        ],
    );
    let result = synthesis(&text);
    let mut claimed = Vec::new();
    let mut sent = 0;
    for (id, _) in &result.suite.scenarios {
        for (input, required) in invocations(&result.suite, &id.to_string()) {
            let declined = required == "declined" || required.ends_with("/declined");
            sent += usize::from(declined);
            if declined
                && number(&input, "amount").is_some_and(|amount| {
                    amount >= Number::from(0_i64) && amount < Number::from(100_i64)
                })
            {
                claimed.push((id.to_string(), input));
            }
        }
    }
    assert!(
        sent > 0,
        "`declined` is never required: {:#?}",
        result.refusals
    );
    assert!(
        claimed.is_empty(),
        "`declined` is required at an input `small`, declared first, claims: {claimed:#?}"
    );
}

/// `b: 11 < amount < 13` after `a: amount <= 11.5 or amount >= 11.6`. `b` is not entirely claimed
/// by `a`: `11.55` reaches it. A refusal saying every input of `b` is claimed by the branch declared
/// first states something false about the model.
#[test]
fn a_branch_reachable_between_ladder_values_is_not_refused_as_claimed_entirely() {
    let text = model(
        "      - {name: amount, type: Decimal}\n",
        &[
            branch("a", "{any: [amount <= 11.5, amount >= 11.6]}", "Placed"),
            branch("b", "{all: [amount > 11, amount < 13]}", "Flagged"),
        ],
    );
    let result = synthesis(&text);
    let shadowed: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{} {refusal}", refusal.code()))
        .filter(|text| text.contains("the accepting branch declared first"))
        .collect();
    assert!(
        shadowed.is_empty(),
        "`b` is refused as claimed entirely by `a`: {shadowed:#?}"
    );
}

/// The subject-state fixture's `Report`, with two plain accepting `when:` branches over `level`
/// that overlap over `5 < level < 10`, declared before the default. The command also has branches
/// guarded by the held state. The overlap is decidable over the input alone; the first-declared
/// branch's scenario sends an input in it and requires that branch, as it does for a command
/// without a state guard.
#[test]
fn a_plain_overlap_in_a_command_with_a_state_guarded_sibling_is_witnessed() {
    let base = include_str!("fixtures/subject-state.yaml");
    let text = base
        .replace(
            "      - {name: note, type: String}\n    outcomes:\n      - name: ringing\n",
            "      - {name: note, type: String}\n      - {name: level, type: Integer}\n    outcomes:\n      - name: ringing\n",
        )
        .replace(
            "      - name: enriched\n",
            "      - name: low\n        when: {all: [incoming == Unspecified, level >= 0, level < 10]}\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n        sets: {note: input.note}\n      - name: high\n        when: {all: [incoming == Unspecified, level > 5]}\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n        sets: {note: input.note}\n      - name: enriched\n",
        );
    assert!(text.contains("name: level") && text.contains("name: low"));
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"));
    let result = synthesize(&ir);
    let mut sent = Vec::new();
    for (id, scenario) in &result.suite.scenarios {
        let mut steps = scenario.steps.iter().peekable();
        while let Some(step) = steps.next() {
            let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                continue;
            };
            if command.to_string() != "calls.core.Report" {
                continue;
            }
            let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
                continue;
            };
            let level = match input.get("level") {
                Some(ScenarioValue::Literal {
                    value: Node::Number(number),
                }) => Some(*number),
                _ => None,
            };
            sent.push((id.to_string(), level, outcome.outcome.to_string()));
        }
    }
    let low_in_overlap = sent.iter().any(|(_, level, required)| {
        required.ends_with("low")
            && level
                .is_some_and(|level| level > Number::from(5_i64) && level < Number::from(10_i64))
    });
    let high_in_overlap: Vec<_> = sent
        .iter()
        .filter(|(_, level, required)| {
            required.ends_with("high")
                && level.is_some_and(|level| {
                    level > Number::from(5_i64) && level < Number::from(10_i64)
                })
        })
        .collect();
    assert!(
        high_in_overlap.is_empty(),
        "`high` is required in the overlap `low`, declared first, answers: {high_in_overlap:#?}"
    );
    assert!(
        low_in_overlap,
        "no input in the overlap of `low` and `high` is sent requiring `low`.\nsent: {sent:#?}\nrefusals: {:#?}\nnotes: {:#?}",
        result.refusals.iter().map(ToString::to_string).collect::<Vec<_>>(),
        result.notes.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
}
