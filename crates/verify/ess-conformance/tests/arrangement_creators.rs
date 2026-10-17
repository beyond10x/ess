//! An existing row is arranged through every declared creation, for every scenario that needs one.
//!
//! * beyond10x/ess#198: a stored-row search seeds every creating command, tried in command-name
//!   order, so a branch only a later creation's row selects is witnessed, whichever of the two
//!   carries the value.
//! * beyond10x/ess#209: an input-guarded refusal on a command addressing an existing record keeps
//!   its plain send (the input refusal is answered before existence) and gains an arranged half: the
//!   record created through a declared creation, the refused input sent for it.
//! * beyond10x/ess#199: a further source of a multi-source transition is arranged through a branch
//!   selected by a stored fact, as that branch's own scenarios already are.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::Synthesis, ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const TWO_CREATORS: &str = include_str!("fixtures/arrangement-two-creators.yaml");
const PAID_ROUTE: &str = include_str!("fixtures/arrangement-paid-route.yaml");
const INPUT_REFUSAL: &str = include_str!("fixtures/arrangement-input-refusal.yaml");

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
    ess_conformance::synthesize::synthesize(&ir)
}

fn ids(suite: &ConformanceSuite) -> Vec<String> {
    suite.scenarios.keys().map(ToString::to_string).collect()
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("no scenario {id} in {:#?}", ids(suite)),
            |(_, scenario)| scenario,
        )
}

/// Every refusal as `<code> <scenario>: <cause>`.
fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

/// The commands a scenario sends, in order, by local name.
fn sent(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => {
                Some(command.to_string().rsplit('.').next().unwrap().to_owned())
            }
            _ => None,
        })
        .collect()
}

/// `needle` appears in `haystack` as consecutive entries.
fn runs_through(haystack: &[String], needle: &[&str]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window.iter().zip(needle).all(|(held, want)| held == want))
}

/// The fixture with the two creations' `paid:` literals exchanged.
fn swapped() -> String {
    TWO_CREATORS
        .replace("sets: {paid: \"false\"}", "sets: {paid: SWAP}")
        .replace("sets: {paid: \"true\"}", "sets: {paid: \"false\"}")
        .replace("sets: {paid: SWAP}", "sets: {paid: \"true\"}")
}

const SHIPPED: &str = "shop.order.ShipOrder/outcome/shipped";
const NOT_PAID: &str = "shop.order.ShipOrder/outcome/not-paid";
const SHIP: &str = "shop.order.Order/transition/ship/by/shop.order.ShipOrder/shipped";
const CLOSE: &str = "shop.order.Order/transition/close/by/shop.order.CloseOrder/closed";
const SHIPPED_REFUSES: &str = "shop.order.Order/state/Shipped/refuses/shop.order.ShipOrder";

#[test]
fn both_branches_of_a_stored_guard_are_witnessed_whichever_creation_is_declared_first() {
    for (order, text) in [
        ("paid second", TWO_CREATORS.to_owned()),
        ("paid first", swapped()),
    ] {
        let result = synthesis(&text);
        assert!(
            refusals(&result).is_empty(),
            "{order}: {:#?}",
            refusals(&result)
        );
        for id in [SHIPPED, NOT_PAID, SHIP, CLOSE, SHIPPED_REFUSES] {
            scenario(&result.suite, id);
        }
    }
}

#[test]
fn each_branch_is_arranged_through_the_creation_whose_row_selects_it() {
    // `paid: false` is PlaceOrderA's in the fixture as written, PlaceOrderB's once swapped.
    for (text, unpaid, paid) in [
        (TWO_CREATORS.to_owned(), "PlaceOrderA", "PlaceOrderB"),
        (swapped(), "PlaceOrderB", "PlaceOrderA"),
    ] {
        let result = synthesis(&text);
        assert_eq!(
            sent(scenario(&result.suite, SHIPPED)),
            [paid, "ShipOrder"],
            "{:#?}",
            refusals(&result)
        );
        // A refusal naming no subject of its own is opened by the absent-subject send, and then
        // sent for the row the arrangement made.
        assert_eq!(
            sent(scenario(&result.suite, NOT_PAID)),
            ["ShipOrder", unpaid, "ShipOrder"],
            "{:#?}",
            refusals(&result)
        );
    }
}

#[test]
fn a_further_transition_source_is_arranged_through_a_branch_a_stored_fact_selects() {
    let result = synthesis(PAID_ROUTE);
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    let close = sent(scenario(&result.suite, CLOSE));
    assert!(
        runs_through(
            &close,
            &["PlaceUnpaidOrder", "PayOrder", "ShipOrder", "CloseOrder"]
        ),
        "the Shipped source of `close` is not arranged: {close:?}"
    );
    // The branch's own scenarios keep the arrangement they had.
    assert_eq!(
        sent(scenario(&result.suite, SHIPPED)),
        ["PlaceUnpaidOrder", "PayOrder", "ShipOrder"]
    );
}

const SECRET: &str = "vault.acct.Configure/outcome/secret-too-short";
const MISSING: &str = "vault.acct.Configure/outcome/missing-configuration";
const ID_REQUIRED: &str = "vault.acct.Configure/outcome/id-required";

/// Whether a `Configure` in the scenario is sent for an instance an earlier step bound.
fn sent_for_a_bound_record(scenario: &ConformanceScenario) -> bool {
    scenario.steps.iter().any(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. } => {
            command.to_string() == "vault.acct.Configure"
                && matches!(input.get("id"), Some(ScenarioValue::Instance { .. }))
        }
        _ => false,
    })
}

#[test]
fn an_input_refusal_on_an_existing_record_is_also_sent_for_a_record_a_creation_made() {
    let result = synthesis(INPUT_REFUSAL);
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    for id in [SECRET, MISSING] {
        let refused = scenario(&result.suite, id);
        // The plain sends stay first (the input refusal is answered before existence, and a
        // boundary witness may send it again); the arranged half comes last.
        let commands = sent(refused);
        assert_eq!(
            commands.first().map(String::as_str),
            Some("Configure"),
            "{id}"
        );
        assert!(
            commands.ends_with(&["Onboard".to_owned(), "Configure".to_owned()]),
            "{id}: {commands:?}"
        );
        assert_eq!(
            commands.iter().filter(|name| *name == "Onboard").count(),
            1,
            "{id}: {commands:?}"
        );
        assert!(
            sent_for_a_bound_record(refused),
            "{id}: {:#?}",
            refused.steps
        );
        let errors = refused
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ExpectError { .. }))
            .count();
        let sends = commands.iter().filter(|name| *name == "Configure").count();
        assert_eq!(errors, sends, "{id}: every send requires the refusal");
    }
}

/// Never sent for a bound record. Besides its own witness, `id-required` is sent at its overlap with
/// each input refusal declared after it, which it answers first (beyond10x/ess#455). In the fixture
/// two follow it and both overlap it: `secret-too-short: secret.count < 12` and
/// `missing-configuration: issuer == ""` each hold beside `id == ""`. So it sends exactly
/// 1 + 2 = 3 plain `Configure` invocations, each requiring it.
#[test]
fn a_refusal_reading_the_identity_stays_a_plain_send() {
    let result = synthesis(INPUT_REFUSAL);
    let refused = scenario(&result.suite, ID_REQUIRED);
    let commands = sent(refused);
    assert_eq!(commands, ["Configure"; 3], "{:#?}", refused.steps);
    assert!(!sent_for_a_bound_record(refused), "{:#?}", refused.steps);
    let errors = refused
        .steps
        .iter()
        .filter(|step| matches!(step, ScenarioStep::ExpectError { .. }))
        .count();
    assert_eq!(errors, commands.len(), "every send requires the refusal");
}

/// beyond10x/ess#553: a guarded stored value or related row's state is arranged through the
/// declared command that reaches it — a later command's `updates:`, a move of a related row — for
/// the subject and for owned rows alike. In the fixture an incident is locked once `Seal`, run
/// after a review it owns has been moved to `Approved`, stores that review in `approved_by`; no
/// creation writes the pointer.
mod through_writers {
    use super::{refusals, scenario, sent, synthesis, Synthesis};
    use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};

    const LOCK: &str = include_str!("fixtures/arrangement-through-writers.yaml");

    fn arranged() -> Synthesis {
        synthesis(LOCK)
    }

    /// The refusals naming `id`, so a failure says why the scenario is missing.
    fn refused(result: &Synthesis, id: &str) -> Vec<String> {
        refusals(result)
            .into_iter()
            .filter(|refusal| refusal.contains(id))
            .collect()
    }

    /// The scenario `id` exists and sends `Seal` after `Approve`, before its last command.
    fn sealed_before(result: &Synthesis, id: &str) {
        let commands = sent(scenario(&result.suite, id));
        let approve = commands.iter().position(|name| name == "Approve");
        let seal = commands.iter().rposition(|name| name == "Seal");
        assert!(
            matches!((approve, seal), (Some(approve), Some(seal)) if approve < seal && seal + 1 < commands.len()),
            "{id}: {commands:?}"
        );
    }

    #[test]
    fn stored_pointer_set_by_a_later_update_is_arranged() {
        let result = arranged();
        let id = "ledger.lock.Retitle/outcome/locked";
        assert!(
            result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id),
            "{:#?}",
            refused(&result, id)
        );
        sealed_before(&result, id);
    }

    #[test]
    fn related_state_reached_by_a_move_is_arranged() {
        let result = arranged();
        // The review the seal stores is owned by the incident and moved to `Approved` first.
        let commands = sent(scenario(&result.suite, "ledger.lock.Seal/outcome/sealed"));
        let approve = commands.iter().position(|name| name == "Approve");
        assert!(
            matches!((approve, commands.last()), (Some(_), Some(last)) if last == "Seal"),
            "{commands:?}"
        );
        // Commands on the owned review read the incident's pointer, which only the seal writes.
        for id in [
            "ledger.lock.Annotate/outcome/locked",
            "ledger.lock.Approve/outcome/locked",
        ] {
            assert!(
                result
                    .suite
                    .scenarios
                    .keys()
                    .any(|key| key.to_string() == id),
                "{id}: {:#?}",
                refused(&result, id)
            );
            sealed_before(&result, id);
        }
    }

    #[test]
    fn row_set_over_moved_rows_is_arranged() {
        let result = arranged();
        let id = "ledger.lock.Reopen/outcome/approved-review";
        assert!(
            result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id),
            "{:#?}",
            refused(&result, id)
        );
        let commands = sent(scenario(&result.suite, id));
        assert!(
            super::runs_through(&commands, &["Approve"])
                && commands.last().map(String::as_str) == Some("Reopen"),
            "{commands:?}"
        );
    }

    #[test]
    fn the_arranged_suite_passes_on_the_interpreted_target() {
        let raw = ess_domain::spec::RawSpecFile::parse(LOCK).unwrap();
        let spec = ess_domain::spec::Specification::assemble([(
            ess_domain::system::Source::new("model.yaml"),
            raw,
        )])
        .unwrap();
        let ir =
            ess_compiler::resolve::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap();
        let suite = ess_conformance::synthesize::synthesize(&ir).suite;
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let report = Runner::for_suite(&suite)
            .run_admitted(&admitted, &Interpreted::for_model(ir))
            .into_report();
        assert_ne!(report.scenarios.len(), 0);
        for scenario in report.scenarios {
            assert_eq!(
                scenario.status,
                Status::Passed,
                "{}: {:?}",
                scenario.scenario,
                scenario.checks
            );
        }
    }
}
