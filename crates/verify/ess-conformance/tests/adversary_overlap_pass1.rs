//! Adversary pass 1 against story:input-guard-overlap-precedence (beyond10x/ess#178).
//!
//! The unit's own documents say an input-guarded refusal "is taken before any accepting branch
//! whose guard it overlaps, whatever order they are written in", and that "each such refusal is sent
//! again at every overlap point" (`website/docs/guides/write-a-specification.md`,
//! `website/docs/reference/predicates.md`). These cases drive synthesis from those sentences.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Synthesis},
    ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const TICKETS: &str = include_str!("fixtures/input-guard-overlap.yaml");
const ID_REQUIRED: &str = "demo.tickets.SetTicketOpen/outcome/id-required";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tickets.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

/// Every `SetTicketOpen` invocation of one scenario, with the branch it requires.
fn invocations(
    suite: &ConformanceSuite,
    id: &str,
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
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "demo.tickets.SetTicketOpen" {
            continue;
        }
        if let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() {
            out.push((input.clone(), outcome.outcome.to_string()));
        }
    }
    out
}

fn lit(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<Node> {
    match input.get(field) {
        Some(ScenarioValue::Literal { value }) => Some(value.clone()),
        _ => None,
    }
}

/// #178 with the accepting branch also reading a stored field (`when_subject: {predicate}` beside
/// its `when: open == false`).
///
/// `closed` still reads `open == false` over the input and `id-required` still overlaps it at
/// `{ticket_id: "", open: false}` for a ticket whose `level` is positive. Entity Runtime now orders
/// the refusal first for this pair too (category 0 beats every non-refusal), and the guide says the
/// precedence holds "whatever order they are written in". `overlap_inputs` only pairs a refusal
/// with `ConstructInput` siblings, and `closed` is `ObserveSubjectFact`, so the overlap point is
/// never sent and a target that reads `open` first passes.
fn with_subject_fact() -> String {
    let text = TICKETS
        .replace(
            "    fields: []\n    lifecycle:",
            "    fields:\n      - {name: level, type: Integer}\n    lifecycle:",
        )
        .replace(
            "  - name: demo.tickets.OpenTicket\n    input: []\n",
            "  - name: demo.tickets.OpenTicket\n    input:\n      - {name: level, type: Integer}\n",
        )
        .replace(
            "        instance: ticket_id\n        emits: [demo.tickets.TicketOpened]\n",
            "        instance: ticket_id\n        sets: {level: input.level}\n        emits: [demo.tickets.TicketOpened]\n",
        )
        .replace(
            "      - name: closed\n        when: open == false\n",
            "      - name: closed\n        when_subject: {predicate: 'level > 0'}\n        when: open == false\n",
        )
        .replace(
            "      - {name: state, type: demo.tickets.Ticket.State}\n",
            "      - {name: state, type: demo.tickets.Ticket.State}\n      - {name: level, type: Integer}\n",
        );
    assert_ne!(text, TICKETS, "the mutation site is present");
    text
}

#[test]
fn adversary_overlap_a_subject_fact_accepting_branch_still_gets_its_overlap_point() {
    let text = with_subject_fact();
    let synthesis = synthesis(&text);
    let sent = invocations(&synthesis.suite, ID_REQUIRED);
    let overlap = sent.iter().any(|(input, branch)| {
        branch == "id-required"
            && lit(input, "ticket_id") == Some(Node::Text(String::new()))
            && lit(input, "open") == Some(Node::Bool(false))
    });
    assert!(
        overlap,
        "`id-required` is never sent the overlap point it shares with the subject-fact `closed`: \
         {sent:#?}\nrefusals: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// An accepting branch every input of which a sibling refusal also claims, beside a default.
///
/// `zero: count == 0` and `too-few: count <= 0`. `validate` accepts it (a default exists) and the
/// base synthesized `zero` at `count: 0`. Under the stated precedence `zero` is unreachable, so a
/// refusal is right — but the refusal renders only `zero`'s own guard, which `count: 0` satisfies,
/// so it tells the author a satisfiable guard has no witness and never names `too-few`.
#[test]
fn adversary_overlap_a_shadowed_accepting_branch_is_refused_naming_the_refusal_that_shadows_it() {
    let text = TICKETS
        .replace(
            "      - {name: open, type: Boolean}\n",
            "      - {name: open, type: Boolean}\n      - {name: count, type: Integer}\n",
        )
        .replace(
            "        when: open == false\n",
            "        when: count == 0\n",
        )
        .replace(
            "        error: demo.tickets.TicketIdRequired\n",
            "        error: demo.tickets.TicketIdRequired\n      - name: too-few\n        when: count <= 0\n        error: demo.tickets.TooFew\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.tickets.TooFew\n    summary: The count is not positive.\n    fields: []\n",
        );
    assert_ne!(text, TICKETS);
    let synthesis = synthesis(&text);
    let about_closed: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|text| text.contains("closed"))
        .collect();
    assert!(
        !about_closed.is_empty(),
        "the shadowed `closed` is refused: {:#?}",
        synthesis.refusals
    );
    assert!(
        about_closed.iter().any(|text| text.contains("too-few")),
        "the refusal for the shadowed `closed` does not name `too-few`, the refusal that claims \
         every input it accepts: {about_closed:#?}"
    );
}

/// A Decimal overlap: `too-large: amount > 1000` beside `closed: amount > 0`, with a default.
/// The overlap point (above 1000) is sent to `too-large` and requires it.
#[test]
fn adversary_overlap_a_decimal_refusal_is_sent_its_overlap_point() {
    let text = TICKETS
        .replace(
            "      - {name: open, type: Boolean}\n",
            "      - {name: open, type: Boolean}\n      - {name: amount, type: Decimal}\n",
        )
        .replace("        when: open == false\n", "        when: amount > 0\n")
        .replace(
            "        error: demo.tickets.TicketIdRequired\n",
            "        error: demo.tickets.TicketIdRequired\n      - name: too-large\n        when: amount > 1000\n        error: demo.tickets.TooLarge\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.tickets.TooLarge\n    summary: The amount is above the limit.\n    fields: []\n",
        );
    assert_ne!(text, TICKETS);
    let synthesis = synthesis(&text);
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    let sent = invocations(
        &synthesis.suite,
        "demo.tickets.SetTicketOpen/outcome/too-large",
    );
    assert!(!sent.is_empty(), "too-large is sent");
    for (input, branch) in &sent {
        assert_eq!(branch, "too-large", "{input:?}");
        assert_ne!(
            lit(input, "ticket_id"),
            Some(Node::Text(String::new())),
            "the overlap point refutes the other refusal: {input:?}"
        );
    }
    let closed = invocations(
        &synthesis.suite,
        "demo.tickets.SetTicketOpen/outcome/closed",
    );
    for (input, branch) in &closed {
        if branch != "closed" {
            continue;
        }
        let Some(Node::Number(amount)) = lit(input, "amount") else {
            continue;
        };
        assert!(
            amount <= ess_primitives::facts::Number::from(1000_i64),
            "`closed` is sent an amount `too-large` claims: {input:?}"
        );
    }
}

/// The same subject-fact command: `id-required` reads `ticket_id == ""`, yet its scenario sends the
/// arranged ticket's identity (`ScenarioValue::Instance`, a generated non-empty id) with `open:
/// true` and requires `id-required`. A target that implements the specification takes `reopened`
/// for that input and fails a scenario it should pass.
#[test]
fn adversary_overlap_an_identity_guarded_refusal_beside_a_subject_fact_is_sent_the_empty_id() {
    let synthesis = synthesis(&with_subject_fact());
    let sent = invocations(&synthesis.suite, ID_REQUIRED);
    assert!(!sent.is_empty(), "id-required is sent");
    for (input, branch) in &sent {
        if branch != "id-required" {
            continue;
        }
        assert_eq!(
            lit(input, "ticket_id"),
            Some(Node::Text(String::new())),
            "`id-required` (`ticket_id == \"\"`) is required for an input whose ticket_id is not \
             the empty text: {input:#?}"
        );
    }
}

/// `closed: count >= 5` and `too-few: 0 < count <= 5` overlap at one value only, `5`. The refusal is sent
/// that value (with the other refusal refuted) and required there.
#[test]
fn adversary_overlap_an_overlap_on_one_boundary_value_is_sent() {
    let text = TICKETS
        .replace(
            "      - {name: open, type: Boolean}\n",
            "      - {name: open, type: Boolean}\n      - {name: count, type: Integer}\n",
        )
        .replace("        when: open == false\n", "        when: count >= 5\n")
        .replace(
            "        error: demo.tickets.TicketIdRequired\n",
            "        error: demo.tickets.TicketIdRequired\n      - name: too-few\n        when: {all: ['count <= 5', 'count > 0']}\n        error: demo.tickets.TooFew\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.tickets.TooFew\n    summary: The count is five or less.\n    fields: []\n",
        );
    assert_ne!(text, TICKETS);
    let synthesis = synthesis(&text);
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let sent = invocations(
        &synthesis.suite,
        "demo.tickets.SetTicketOpen/outcome/too-few",
    );
    assert!(
        sent.iter().any(|(input, branch)| branch == "too-few"
            && lit(input, "count") == Some(Node::Number(5_i64.into()))),
        "`too-few` is not sent the single overlap value 5: {sent:#?}"
    );
    for (input, branch) in invocations(
        &synthesis.suite,
        "demo.tickets.SetTicketOpen/outcome/closed",
    ) {
        if branch == "closed" {
            assert_ne!(
                lit(&input, "count"),
                Some(Node::Number(5_i64.into())),
                "`closed` is required at 5, which `too-few` claims"
            );
        }
    }
}

/// An enum overlap: `closed: priority == High` and `urgent-closed-refused: priority != Low`,
/// beside a default. They overlap at `High`, which the refusal is sent and required at.
#[test]
fn adversary_overlap_an_enum_overlap_is_sent() {
    let text = TICKETS
        .replace(
            "types:\n",
            "types:\n  - name: demo.tickets.Priority\n    kind: enum\n    variants: [Low, Normal, High]\n",
        )
        .replace(
            "      - {name: open, type: Boolean}\n",
            "      - {name: open, type: Boolean}\n      - {name: priority, type: demo.tickets.Priority}\n",
        )
        .replace("        when: open == false\n", "        when: priority == High\n")
        .replace(
            "        error: demo.tickets.TicketIdRequired\n",
            "        error: demo.tickets.TicketIdRequired\n      - name: not-low\n        when: priority != Low\n        error: demo.tickets.NotLow\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.tickets.NotLow\n    summary: Only a low priority ticket is changed here.\n    fields: []\n",
        );
    assert_ne!(text, TICKETS);
    let synthesis = synthesis(&text);
    let sent = invocations(
        &synthesis.suite,
        "demo.tickets.SetTicketOpen/outcome/not-low",
    );
    assert!(
        sent.iter().any(|(input, branch)| branch == "not-low"
            && lit(input, "priority") == Some(Node::Text("High".into()))),
        "`not-low` is not sent `High`, where it overlaps `closed`: {sent:#?}\nrefusals: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}
