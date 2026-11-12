//! Adversary pass 1 on story:overlap-witnesses-per-phase (beyond10x/ess#472): two `when_subject`
//! refusals of one command whose stored predicates one row satisfies together.
//!
//! Each case asserts what the commit, its changelog sentence or its moved pins claim, against the
//! behaviour of synthesis and the mutation audit.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Note;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REVOKE: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/external-before-row-revoke.yaml");

const SHIP_EMPTY: &str = "demo.order.Ship/outcome/empty";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn run(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The `(scenario, first, other)` of every `Note::UnwitnessedOverlap`.
/// Whether every synthesis refusal is of `shipped`, which two refusals covering every row leave
/// unreachable.
fn refuses_only_shipped(result: &ess_conformance::synthesize::Synthesis) -> bool {
    result
        .refusals
        .iter()
        .all(|refusal| format!("{refusal:?}").contains("OutcomeName(shipped)"))
}

fn overlap_notes(result: &ess_conformance::synthesize::Synthesis) -> Vec<(String, String, String)> {
    result
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnwitnessedOverlap {
                scenario,
                first,
                other,
                ..
            } => Some((scenario.to_string(), first.to_string(), other.to_string())),
            _ => None,
        })
        .collect()
}

fn audit(text: &str) -> mutate::MutationReport {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    let files = vec![(Source::new("fixture.yaml"), raw)];
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(
        &files,
        &texts,
        &[mutate::MutantClass::PrecedenceSwap],
        || Interpreted::for_model(ir.clone()),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// The exit status `ess verify conform mutate` gives `report` (as `refusal_pair_overlap.rs`
/// computes it): 1 with a survivor, 3 with an unwitnessed or inconclusive mutant or none run.
fn exit_status(report: &mutate::MutationReport) -> u8 {
    let ran = report
        .mutants
        .iter()
        .filter(|entry| {
            !matches!(
                entry.verdict,
                mutate::Verdict::Stillborn | mutate::Verdict::Equivalent
            )
        })
        .count();
    if report.counts.survived > 0 {
        1
    } else if report.counts.unwitnessed > 0
        || report.counts.inconclusive > 0
        || ran == 0
        || report.unaudited_sites() > 0
    {
        3
    } else {
        0
    }
}

fn verdict_of(report: &mutate::MutationReport, id: &str) -> mutate::Verdict {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no site {id}:\n{}", report.render_text()))
        .verdict
}

/// A `demo.order` model, as `refusal_pair_overlap.rs` builds model C: `Place` creates the row
/// (refusing `quantity <= 0` and `discount < 0` on input where `place_refuses`), and `Ship`
/// declares `refusals` — `(name, when_subject predicate, error)` — in order, then `shipped`.
fn order_model(format: &str, place_refuses: bool, refusals: &[(&str, &str, &str)]) -> String {
    let place_refusals = if place_refuses {
        "      - {name: bad-quantity, when: quantity <= 0, error: demo.order.BadQuantity}\n      - {name: bad-discount, when: discount < 0, error: demo.order.BadDiscount}\n"
    } else {
        ""
    };
    let mut ship = String::new();
    for (name, predicate, error) in refusals {
        write!(
            ship,
            "      - name: {name}\n        when_subject: {{predicate: {predicate}}}\n        error: demo.order.{error}\n"
        )
        .unwrap();
    }
    format!(
        r"format: {format}
system: demo
version: v1
domain: demo.order
summary: Orders refused for two conditions their stored fields can hold together.
types:
  - {{name: demo.order.OrderId, kind: newtype, of: Uuid}}
entities:
  - name: demo.order.Order
    identity: {{name: order_id, type: demo.order.OrderId}}
    fields:
      - {{name: quantity, type: Integer}}
      - {{name: discount, type: Integer}}
      - {{name: locked, type: Boolean}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - name: demo.order.Agent
    may: [demo.order.Place, demo.order.Ship]
errors:
  - {{name: demo.order.BadQuantity, fields: []}}
  - {{name: demo.order.BadDiscount, fields: []}}
  - {{name: demo.order.Locked, fields: []}}
  - {{name: demo.order.Empty, fields: []}}
events:
  - name: demo.order.Placed
    fields: [{{name: order_id, type: demo.order.OrderId}}]
  - name: demo.order.Changed
    fields: [{{name: order_id, type: demo.order.OrderId}}]
commands:
  - name: demo.order.Place
    input:
      - {{name: quantity, type: Integer}}
      - {{name: discount, type: Integer}}
      - {{name: locked, type: Boolean}}
    outcomes:
{place_refusals}      - name: placed
        creates: demo.order.Order
        instance: order_id
        sets: {{quantity: input.quantity, discount: input.discount, locked: input.locked}}
        emits: [demo.order.Placed]
        payload: {{demo.order.Placed: {{order_id: {{generated: true}}}}}}
  - name: demo.order.Ship
    input:
      - {{name: order_id, type: demo.order.OrderId}}
    outcomes:
{ship}      - name: shipped
        updates: demo.order.Order
        instance: order_id
        sets: {{locked: true}}
        emits: [demo.order.Changed]
        payload: {{demo.order.Changed: {{order_id: input.order_id}}}}
components:
  - component: orders
    owns: {{domains: [demo.order]}}
    accepts: {{commands: [demo.order.Place, demo.order.Ship]}}
    reached_by: network
"
    )
}

/// The literal `field` is sent as, in JSON numbers as `f64` text; empty where it is not a literal.
fn literal_json(input: &BTreeMap<String, ScenarioValue>, field: &str) -> String {
    input
        .get(field)
        .and_then(ScenarioValue::as_literal)
        .map_or_else(String::new, |node| {
            match serde_json::to_value(node).expect("a literal is JSON") {
                serde_json::Value::Number(number) => number
                    .as_f64()
                    .map_or_else(|| number.to_string(), |value| value.to_string()),
                other => other.to_string(),
            }
        })
}

/// How many `Ship` sends of `scenario` the next step requires to answer `outcome`.
fn ship_requiring(scenario: &ConformanceScenario, outcome: &str) -> usize {
    scenario
        .steps
        .iter()
        .enumerate()
        .filter(|(at, step)| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.name().to_string() == "demo.order.Ship")
                && matches!(scenario.steps.get(at + 1), Some(ScenarioStep::ExpectOutcome { outcome: required })
                    if required.outcome.to_string() == outcome)
        })
        .count()
}

/// Whether `scenario` creates a row through `Place` sending `quantity` and `discount` and `locked`
/// as these literals (`None` matches any).
fn places(
    scenario: &ConformanceScenario,
    quantity: Option<&str>,
    discount: Option<&str>,
    locked: Option<&str>,
) -> bool {
    scenario.steps.iter().any(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. } => {
            command.name().to_string() == "demo.order.Place"
                && quantity.is_none_or(|it| literal_json(input, "quantity") == it)
                && discount.is_none_or(|it| literal_json(input, "discount") == it)
                && locked.is_none_or(|it| literal_json(input, "locked") == it)
        }
        _ => false,
    })
}

// ---- moved pins -----------------------------------------------------------------------------------

/// `external-before-row-revoke.yaml` gained an `Unreached` note (notes 1 -> 2). Its pair
/// `issued-to-another-client: client_id != input.client_id` before `already-revoked: state ==
/// Revoked` is held by one row: a token issued to one client and revoked, sent `Revoke` by
/// another — the `already-revoked` scenario arranges a revoked token already. The changelog says
/// such a pair is witnessed on every format, so a target checking `already-revoked` first fails.
#[test]
fn revoke_pair_one_row_holds_is_witnessed_not_noted_unreached() {
    let first = "      - name: issued-to-another-client\n        when_subject: {predicate: client_id != input.client_id}\n        error: demo.revoke.TokenNotIssuedToClient\n";
    let second = "      - name: already-revoked\n        when_subject: {predicate: state == Revoked}\n        error: demo.revoke.AlreadyRevoked\n";
    let declared = format!("{first}{second}");
    assert!(REVOKE.contains(&declared), "the fixture declares the pair");
    let swapped_text = REVOKE.replace(&declared, &format!("{second}{first}"));

    let result = synthesis(REVOKE);
    let honest = Interpreted::for_model(ir(REVOKE));
    assert_eq!(
        not_passed(&run(&result.suite, &honest)),
        Vec::<&str>::new(),
        "the model's own interpreter passes its suite"
    );
    let swapped = Interpreted::for_model(ir(&swapped_text));
    assert!(
        !not_passed(&run(&result.suite, &swapped)).is_empty(),
        "a target answering `already-revoked` before `issued-to-another-client` passes every \
         scenario, though a revoked token sent by another client holds both; notes: {:#?}",
        overlap_notes(&result)
    );
}

// ---- disjointness -------------------------------------------------------------------------------

/// `quantity < 0` and `quantity > 10` hold on no row together: nothing to witness, and nothing to
/// note (the commit body: "a pair no row can satisfy ... is skipped").
#[test]
fn ordering_pair_no_row_can_satisfy_is_not_noted() {
    let model = order_model(
        "ess/20",
        false,
        &[
            ("empty", "quantity < 0", "Empty"),
            ("locked", "quantity > 10", "Locked"),
        ],
    );
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result),
        Vec::<(String, String, String)>::new(),
        "`quantity < 0` and `quantity > 10` overlap nowhere, so no unwitnessed overlap is noted"
    );
}

/// `quantity <= 0` and `quantity >= 0` meet only at `0`: the overlap is sent there, and a target
/// answering the second first fails `empty`'s scenario alone.
#[test]
fn ordering_pair_meeting_at_zero_is_witnessed() {
    let refusals = [
        ("empty", "quantity <= 0", "Empty"),
        ("locked", "quantity >= 0", "Locked"),
    ];
    let model = order_model("ess/20", false, &refusals);
    let result = synthesis(&model);
    // The two cover every row, so `shipped` is refused; nothing else is.
    assert!(refuses_only_shipped(&result), "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result),
        Vec::<(String, String, String)>::new()
    );
    let empty = scenario(&result.suite, SHIP_EMPTY);
    assert!(places(empty, Some("0"), None, None), "{empty:#?}");
    assert!(ship_requiring(empty, "empty") >= 2, "{empty:#?}");
    let swapped = order_model("ess/20", false, &[refusals[1], refusals[0]]);
    assert_eq!(
        not_passed(&run(&result.suite, &Interpreted::for_model(ir(&swapped)))),
        vec![SHIP_EMPTY]
    );
}

/// `discount == 5` and `discount != 5` hold on no row together: skipped, not noted, sent once.
#[test]
fn equality_and_inequality_of_one_literal_is_skipped() {
    let model = order_model(
        "ess/20",
        false,
        &[
            ("empty", "discount == 5", "Empty"),
            ("locked", "discount != 5", "Locked"),
        ],
    );
    let result = synthesis(&model);
    // The two cover every row, so `shipped` is refused; nothing else is.
    assert!(refuses_only_shipped(&result), "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result),
        Vec::<(String, String, String)>::new()
    );
    assert_eq!(
        ship_requiring(scenario(&result.suite, SHIP_EMPTY), "empty"),
        1
    );
}

/// A predicate over two fields beside one over a third: the overlap is a row holding all three,
/// and the swapped target fails `empty`'s scenario alone.
#[test]
fn predicate_over_two_fields_is_witnessed() {
    let refusals = [
        ("empty", "quantity <= 0", "Empty"),
        ("locked", "{all: [locked == true, discount < 0]}", "Locked"),
    ];
    let model = order_model("ess/20", false, &refusals);
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result),
        Vec::<(String, String, String)>::new()
    );
    let swapped = order_model("ess/20", false, &[refusals[1], refusals[0]]);
    assert_eq!(
        not_passed(&run(&result.suite, &Interpreted::for_model(ir(&swapped)))),
        vec![SHIP_EMPTY]
    );
}

// ---- mutation ---------------------------------------------------------------------------------

/// Two `when_subject` refusals naming two values of one field (`discount == 1`, `discount == 2`)
/// never answer the same request, so their swap decides nothing; synthesis already skips the pair
/// as disjoint. An input pair so disjoint is `equivalent` (beyond10x/ess#703). Beside `Place`'s
/// killed input pair, the audit exited 0 before this change listed the stored pair's site.
#[test]
fn disjoint_stored_pair_swap_keeps_the_audit_green() {
    let model = order_model(
        "ess/20",
        true,
        &[
            ("empty", "discount == 1", "Empty"),
            ("locked", "discount == 2", "Locked"),
        ],
    );
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result),
        Vec::<(String, String, String)>::new()
    );
    let report = audit(&model);
    assert_eq!(
        verdict_of(
            &report,
            "precedence-swap/demo.order.Place/bad-quantity/bad-discount"
        ),
        mutate::Verdict::Killed,
        "{}",
        report.render_text()
    );
    let swap = verdict_of(&report, "precedence-swap/demo.order.Ship/empty/locked");
    assert_eq!(
        (swap, exit_status(&report)),
        (mutate::Verdict::Equivalent, 0),
        "a swap of two refusals no row selects together is equivalent, and the audit stays green:\n{}",
        report.render_text()
    );
}

/// Three refusals of the stored row in one phase: both adjacent swaps are killed.
#[test]
fn three_stored_refusals_both_swaps_killed() {
    let model = order_model(
        "ess/20",
        false,
        &[
            ("empty", "quantity <= 0", "Empty"),
            ("locked", "locked == true", "Locked"),
            ("negative", "discount < 0", "BadDiscount"),
        ],
    );
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result),
        Vec::<(String, String, String)>::new()
    );
    let report = audit(&model);
    for site in [
        "precedence-swap/demo.order.Ship/empty/locked",
        "precedence-swap/demo.order.Ship/locked/negative",
    ] {
        assert_eq!(
            verdict_of(&report, site),
            mutate::Verdict::Killed,
            "{site}:\n{}",
            report.render_text()
        );
    }
    assert_eq!(exit_status(&report), 0, "{}", report.render_text());
}

// ---- changelog --------------------------------------------------------------------------------

/// "from `ess/23` it snapshots that row before the send and compares it after": model C at
/// `ess/23` observes the subject after the overlap send, and at `ess/20` does not.
#[test]
fn overlap_send_is_observed_from_ess23_only() {
    let refusals = [
        ("empty", "quantity <= 0", "Empty"),
        ("locked", "locked == true", "Locked"),
    ];
    let after_overlap = |format: &str| {
        let result = synthesis(&order_model(format, false, &refusals));
        assert_eq!(result.refusals.len(), 0, "{format}: {:#?}", result.refusals);
        let empty = scenario(&result.suite, SHIP_EMPTY).clone();
        let at = empty
            .steps
            .iter()
            .position(|step| match step {
                ScenarioStep::ExecuteCommand { command, input, .. } => {
                    command.name().to_string() == "demo.order.Place"
                        && literal_json(input, "quantity") == "0"
                        && literal_json(input, "locked") == "true"
                }
                _ => false,
            })
            .unwrap_or_else(|| panic!("{format}: the overlap row is arranged: {empty:#?}"));
        empty
            .steps
            .iter()
            .skip(at)
            .any(|step| matches!(step, ScenarioStep::ExpectNoEvents))
    };
    assert!(after_overlap("ess/23"), "ess/23 observes the overlap send");
    assert!(!after_overlap("ess/20"), "ess/20 does not");
}
