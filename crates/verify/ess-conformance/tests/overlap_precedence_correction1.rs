//! The declared precedence among overlapping guards (beyond10x/ess#217,
//! `docs/design/input-guard-overlap-precedence.md`), for the cases the first adversary pass found
//! silent or wrong: an overlap only a value between two literals reaches, the same for a refusal,
//! an overlap no scenario sends, an external branch declared after an accepting one, and a later
//! guard the interpreter cannot decide.
#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::execute::{execute, Externals, Store},
    synthesize::{synthesize, Note, Synthesis},
    ConformanceSuite, ScenarioStep, ScenarioValue,
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
  - name: demo.orders.TooMuch
    summary: The amount is refused.
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
commands:
  - name: demo.orders.PlaceOrder
";

fn branch(name: &str, when: &str, event: &str) -> String {
    format!(
        "      - name: {name}\n        when: {when}\n        creates: demo.orders.Order\n        instance: order_id\n        emits: [demo.orders.{event}]\n        payload: {{demo.orders.{event}: {{order_id: {{generated: true}}}}}}\n"
    )
}

fn refusal(name: &str, when: &str) -> String {
    format!("      - name: {name}\n        when: {when}\n        error: demo.orders.TooMuch\n")
}

fn external(name: &str) -> String {
    format!("      - name: {name}\n        external: the provider declines\n        error: demo.orders.Declined\n")
}

const DEFAULT: &str = "      - name: refused\n        error: demo.orders.NotPlaced\n";

fn model(inputs: &str, branches: &[String]) -> String {
    format!(
        "{HEAD}    input:\n{inputs}    outcomes:\n{}{DEFAULT}",
        branches.concat()
    )
}

const DECIMAL: &str = "      - {name: amount, type: Decimal}\n";
const INTEGER: &str = "      - {name: amount, type: Integer}\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

/// Every invocation of `command` one scenario sends, by field, with the branch it requires.
fn sent(
    suite: &ConformanceSuite,
    id: &str,
    command: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, String)> {
    let Some((_, scenario)) = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand {
            command: invoked,
            input,
            ..
        } = step
        else {
            continue;
        };
        if invoked.to_string() != command {
            continue;
        }
        if let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() {
            out.push((input.clone(), outcome.outcome.to_string()));
        }
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

fn between(value: Number, low: f64, high: f64) -> bool {
    value.get() > low && value.get() < high
}

fn overlap_notes(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnwitnessedOverlap { .. }))
        .map(ToString::to_string)
        .collect()
}

/// An input-guarded refusal overlapping an accepting branch only over `11 < amount < 12`, a Decimal
/// interval no ladder value lies in. The refusal is taken first there, so its scenario sends an
/// amount inside and requires it (beyond10x/ess#178, the same silent drop as #217's).
#[test]
fn a_refusal_overlap_between_two_literals_is_sent_requiring_the_refusal() {
    let text = model(
        DECIMAL,
        &[
            branch("flagged", "{all: [amount > 11, amount < 13]}", "Flagged"),
            refusal("too-much", "{all: [amount > 10, amount < 12]}"),
        ],
    );
    let synthesis = synthesize(&ir(&text));
    let invocations = sent(
        &synthesis.suite,
        "demo.orders.PlaceOrder/outcome/too-much",
        "demo.orders.PlaceOrder",
    );
    assert!(
        invocations
            .iter()
            .any(|(input, required)| required == "too-much"
                && number(input, "amount").is_some_and(|amount| between(amount, 11.0, 12.0))),
        "the overlap is not sent: {invocations:#?}\nnotes: {:#?}",
        overlap_notes(&synthesis)
    );
    assert_eq!(overlap_notes(&synthesis).len(), 0);
}

/// The accepting overlap over `11 < amount < 12` is sent at the exact midpoint of the two literals,
/// a value the type's own precision writes, and the swapped target fails.
#[test]
fn an_accepting_overlap_between_two_literals_is_sent_at_their_midpoint() {
    let text = model(
        DECIMAL,
        &[
            branch("small", "{all: [amount > 10, amount < 12]}", "Placed"),
            branch("flagged", "{all: [amount > 11, amount < 13]}", "Flagged"),
        ],
    );
    let synthesis = synthesize(&ir(&text));
    let invocations = sent(
        &synthesis.suite,
        "demo.orders.PlaceOrder/outcome/small",
        "demo.orders.PlaceOrder",
    );
    let inside: Vec<Number> = invocations
        .iter()
        .filter(|(_, required)| required == "small")
        .filter_map(|(input, _)| number(input, "amount"))
        .filter(|amount| between(*amount, 11.0, 12.0))
        .collect();
    assert_eq!(inside.len(), 1, "{invocations:#?}");
    assert_eq!(inside[0].exact_text(), "11.5");
    assert_eq!(overlap_notes(&synthesis).len(), 0);
}

/// Two branches whose guards cannot both hold are not an overlap, and no note says they are:
/// the candidates cover every region the literals divide `amount` into.
#[test]
fn disjoint_guards_are_not_noted() {
    for inputs in [DECIMAL, INTEGER] {
        let text = model(
            inputs,
            &[
                branch("small", "amount < 10", "Placed"),
                branch("big", "amount > 20", "Flagged"),
                refusal("odd", "{all: [amount > 12, amount < 15]}"),
            ],
        );
        let synthesis = synthesize(&ir(&text));
        assert!(
            overlap_notes(&synthesis).is_empty(),
            "{inputs}: {:#?}",
            overlap_notes(&synthesis)
        );
    }
}

/// Two overlapping plain accepting branches in a command whose subject is read by a
/// `when_subject:` sibling. Its scenarios are arranged over a stored row, and the overlap is either
/// sent requiring `low` or recorded as a note naming both branches — never neither.
#[test]
fn an_overlap_in_a_command_reading_the_stored_row_is_sent_or_noted() {
    let base = include_str!("fixtures/subject-guard-input.yaml");
    let text = base
        .replace(
            "      - {name: call_id, type: Uuid}\n      - {name: recording_id, type: String}\n    outcomes:\n",
            "      - {name: call_id, type: Uuid}\n      - {name: recording_id, type: String}\n      - {name: level, type: Integer}\n    outcomes:\n",
        )
        .replace(
            "      - name: stopped\n",
            "      - name: low\n        when: {all: [level >= 0, level < 10]}\n        preserves: calls.rec.Call\n        instance: call_id\n      - name: high\n        when: level > 5\n        preserves: calls.rec.Call\n        instance: call_id\n      - name: stopped\n",
        );
    assert!(text.contains("name: level") && text.contains("name: high"));
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"));
    let synthesis = synthesize(&ir);
    let witnessed = synthesis.suite.scenarios.keys().any(|id| {
        sent(&synthesis.suite, &id.to_string(), "calls.rec.Stop")
            .iter()
            .any(|(input, required)| {
                required.ends_with("low")
                    && number(input, "level").is_some_and(|level| between(level, 5.0, 10.0))
            })
    });
    let noted = overlap_notes(&synthesis)
        .iter()
        .any(|note| note.contains("`low`") && note.contains("`high`"));
    assert!(
        witnessed || noted,
        "the overlap of `low` and `high` is neither sent nor noted.\nrefusals: {:#?}\nnotes: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        overlap_notes(&synthesis)
    );
}

/// An overlap the candidates neither reach nor show empty is noted: two guards over one text, one
/// ordered and one matched, which [`ess_conformance::witness::exhausts`] does not cover.
#[test]
fn an_overlap_the_candidates_do_not_decide_is_noted_or_sent() {
    let text = model(
        "      - {name: note, type: String}\n",
        &[
            branch("early", "note < \"m\"", "Placed"),
            branch("tagged", "{note: {ends_with: \"zz\"}}", "Flagged"),
        ],
    );
    let synthesis = synthesize(&ir(&text));
    let witnessed = sent(
        &synthesis.suite,
        "demo.orders.PlaceOrder/outcome/early",
        "demo.orders.PlaceOrder",
    )
    .iter()
    .any(|(input, required)| {
        required == "early"
            && matches!(input.get("note"), Some(ScenarioValue::Literal { value: Node::Text(t) }) if t.as_str() < "m" && t.ends_with("zz"))
    });
    let noted = overlap_notes(&synthesis)
        .iter()
        .any(|note| note.contains("`early`") && note.contains("`tagged`"));
    assert!(witnessed || noted, "{:#?}", synthesis.notes);
}

/// An external branch declared after an accepting guard is witnessed outside that guard, including
/// an external branch with a guard of its own, and the interpreter forced to it answers the
/// accepting branch where that guard holds (the Entity Runtime order,
/// `crates/generate/ess-entity-runtime/tests/input_guard_overlap.rs`).
#[test]
fn an_external_branch_follows_the_declaration_order() {
    let guarded = "      - name: declined\n        when: amount > 50\n        external: the provider declines\n        error: demo.orders.Declined\n".to_owned();
    for declined in [external("declined"), guarded] {
        let text = model(
            INTEGER,
            &[
                branch("small", "{all: [amount >= 0, amount < 100]}", "Placed"),
                declined.clone(),
            ],
        );
        let synthesis = synthesize(&ir(&text));
        let mut required = 0;
        for id in synthesis.suite.scenarios.keys() {
            for (input, branch) in sent(&synthesis.suite, &id.to_string(), "demo.orders.PlaceOrder")
            {
                if branch == "declined" {
                    required += 1;
                    let amount = number(&input, "amount").expect("an amount is sent");
                    assert!(
                        amount.get() < 0.0 || amount.get() >= 100.0,
                        "{declined}: `declined` required at {amount:?}, which `small` claims"
                    );
                }
            }
        }
        assert!(required > 0, "{declined}: {:#?}", synthesis.refusals);
    }

    let text = model(
        INTEGER,
        &[
            branch("small", "{all: [amount >= 0, amount < 100]}", "Placed"),
            external("declined"),
        ],
    );
    let model = ir(&text);
    let answer = |amount: i64, externals: &Externals| -> BTreeSet<String> {
        execute(
            &model,
            &Store::default(),
            &QualifiedName::new("demo.orders.PlaceOrder").unwrap(),
            &BTreeMap::from([("amount".to_owned(), Node::Number(Number::from(amount)))]),
            externals,
        )
        .expect("determined")
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), |o| o.outcome.to_string())
        })
        .collect()
    };
    let forced = Externals::Forced("declined".parse().unwrap());
    let set = |names: &[&str]| {
        names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(answer(5, &forced), set(&["small"]));
    assert_eq!(answer(-5, &forced), set(&["declined"]));
    assert_eq!(answer(5, &Externals::Open), set(&["small"]));
    assert_eq!(answer(-5, &Externals::Open), set(&["declined", "refused"]));
}

/// An external branch declared before an accepting one is taken first where its provider takes it,
/// and is one open answer beside the accepting branch.
#[test]
fn an_external_branch_declared_first_is_taken_first() {
    let text = model(
        INTEGER,
        &[
            external("declined"),
            branch("small", "{all: [amount >= 0, amount < 100]}", "Placed"),
        ],
    );
    let model = ir(&text);
    let answer = |externals: &Externals| -> BTreeSet<String> {
        execute(
            &model,
            &Store::default(),
            &QualifiedName::new("demo.orders.PlaceOrder").unwrap(),
            &BTreeMap::from([("amount".to_owned(), Node::Number(Number::from(5_i64)))]),
            externals,
        )
        .expect("determined")
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), |o| o.outcome.to_string())
        })
        .collect()
    };
    let set = |names: &[&str]| {
        names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(
        answer(&Externals::Forced("declined".parse().unwrap())),
        set(&["declined"])
    );
    assert_eq!(answer(&Externals::Open), set(&["declined", "small"]));
    assert_eq!(answer(&Externals::Withheld), set(&["small"]));
}

/// A guard the interpreter cannot decide is Undecidable only where it is read before the branch
/// that answers: an accepting guard before the holding one, or a refusal guard before any other.
#[test]
fn an_undecided_guard_is_undecidable_only_before_the_answer() {
    let inputs =
        "      - {name: amount, type: Integer}\n      - {name: note, type: Optional<String>}\n";
    let decide = |branches: &[String]| {
        execute(
            &ir(&model(inputs, branches)),
            &Store::default(),
            &QualifiedName::new("demo.orders.PlaceOrder").unwrap(),
            &BTreeMap::from([("amount".to_owned(), Node::Number(Number::from(5_i64)))]),
            &Externals::Withheld,
        )
        .map(|steps| {
            steps
                .iter()
                .map(|step| {
                    step.outcome
                        .as_ref()
                        .map_or("none".to_owned(), |o| o.outcome.to_string())
                })
                .collect::<BTreeSet<_>>()
        })
    };
    let rush = || branch("rush", "note == \"rush\"", "Flagged");
    let placed = || branch("placed", "amount > 0", "Placed");
    assert!(
        decide(&[rush(), placed()]).is_err(),
        "an undecided guard read first"
    );
    assert_eq!(
        decide(&[placed(), rush()]).unwrap(),
        BTreeSet::from(["placed".to_owned()])
    );
    assert!(
        decide(&[placed(), refusal("noted", "note == \"x\"")]).is_err(),
        "a refusal is read before any accepting branch"
    );
    assert_eq!(
        decide(&[
            placed(),
            refusal("positive", "amount > 0"),
            refusal("noted", "note == \"x\"")
        ])
        .unwrap(),
        BTreeSet::from(["positive".to_owned()])
    );
}

/// `tests/guarded_external.rs`'s finite partition with the guarded external branch declared before
/// the partition, which is where a branch the provider decides can still be taken: declared after
/// `sent-b: mode == B` and `sent: mode == A`, which cover every input, it answers nothing
/// (beyond10x/ess#217). Declared first, it is witnessed and nothing is refused.
#[test]
fn an_external_branch_declared_before_a_finite_partition_is_witnessed() {
    let text = r#"
format: ess/6
system: delivery
version: v1
domain: delivery.mail
types:
  - name: delivery.mail.Mode
    kind: enum
    variants: [A, B]
events:
  - name: delivery.mail.Sent
    fields: []
errors:
  - name: delivery.mail.Rejected
    fields: []
commands:
  - name: delivery.mail.Send
    input:
      - {name: mode, type: delivery.mail.Mode}
      - {name: retry, type: Boolean}
      - {name: recipient, type: String}
    outcomes:
      - name: rejected
        when:
          all: [retry == false, recipient != ""]
        external: the provider rejects the initial delivery
        error: delivery.mail.Rejected
      - name: sent-b
        when: mode == B
        emits: [delivery.mail.Sent]
      - name: sent
        when: mode == A
        emits: [delivery.mail.Sent]
"#;
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("guarded.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let result = synthesize(&ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let after = text.replace(
        "      - name: rejected\n        when:\n          all: [retry == false, recipient != \"\"]\n        external: the provider rejects the initial delivery\n        error: delivery.mail.Rejected\n",
        "",
    ) + "      - name: rejected\n        when:\n          all: [retry == false, recipient != \"\"]\n        external: the provider rejects the initial delivery\n        error: delivery.mail.Rejected\n";
    let raw = RawSpecFile::parse(&after).unwrap();
    let spec = Specification::assemble([(Source::new("guarded.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let refused: Vec<String> = synthesize(&ir)
        .refusals
        .iter()
        .map(|refusal| format!("{} {refusal}", refusal.code()))
        .collect();
    assert!(
        refused.len() == 1 && refused[0].contains("the accepting branch declared first"),
        "{refused:#?}"
    );
}
