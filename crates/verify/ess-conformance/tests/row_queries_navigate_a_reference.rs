//! A declared reference navigated as a value (`ess/28`, beyond10x/ess#557, beyond10x/ess#564,
//! `docs/design/generic-language-features.md` § G2, navigation): the related row's lifecycle state
//! read by a payload, and a view reading the referenced row's current field and filtering on it.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The reproduction of the downstream finding (2026-10-19, finding 9): a ticket is opened only on
/// an open desk, and the refusal names the state the desk is in; two views read the desk a ticket
/// names as it is when the view is read.
const DESK: &str = include_str!("fixtures/navigate-a-reference.yaml");

/// A maintenance window that may belong to an incident: what it reads of the incident is absent
/// where it belongs to none.
const WINDOW: &str = include_str!("fixtures/navigate-absent-reference.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("navigate.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(model: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id} among {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every payload the scenario's expectations of `error` compare.
fn error_fields(scenario: &ConformanceScenario, error: &str) -> Vec<BTreeMap<String, Node>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectError {
                error: seen,
                fields,
            } if seen.to_string() == error => Some(fields.clone()),
            _ => None,
        })
        .collect()
}

/// Runs `suite` against the reference interpreter of `model`, every scenario passing.
fn passes_on_the_interpreter(model: EssIr, suite: &ConformanceSuite) {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert_ne!(run.scenarios.len(), 0);
    let failed: Vec<_> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn related_state_reads_as_a_payload_value() {
    let model = ir(DESK);
    let suite = suite(&model);
    let refused = scenario(&suite, "relstate.desk.OpenTicket/outcome/desk-not-open");
    let asserted = error_fields(refused, "relstate.desk.DeskNotOpen");
    assert_eq!(
        asserted,
        vec![BTreeMap::from([(
            "current".to_owned(),
            Node::Text("Closed".to_owned())
        )])],
        "the refusal asserts the state that blocked it"
    );
    passes_on_the_interpreter(model, &suite);
}

/// One read of `view` in `scenario`, in step order: what it claims, and the commands sent before it.
#[derive(Debug)]
struct Claim {
    sent_before: Vec<String>,
    expectation: ViewExpectation,
}

/// Every claim `scenario` makes of `view`, each with the commands sent before it.
fn claims(scenario: &ConformanceScenario, view: &str) -> Vec<Claim> {
    let mut sent = Vec::new();
    let mut out = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, .. } => sent.push(command.to_string()),
            ScenarioStep::ExpectView {
                view: read,
                expectation,
            }
            | ScenarioStep::EventuallyView {
                view: read,
                expectation,
                ..
            } if read.to_string() == view => out.push(Claim {
                sent_before: sent.clone(),
                expectation: expectation.clone(),
            }),
            _ => {}
        }
    }
    out
}

/// The literal `field` of a claim, where it names one.
fn literal(fields: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<Node> {
    match fields.get(field)? {
        ScenarioValue::Literal { value } => Some(value.clone()),
        _ => None,
    }
}

/// The instance `field` of a claim names, where it names one.
fn instance(fields: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<String> {
    match fields.get(field)? {
        ScenarioValue::Instance { instance } => Some(instance.to_string()),
        _ => None,
    }
}

#[test]
fn view_projects_the_referenced_rows_current_field() {
    let model = ir(DESK);
    let suite = suite(&model);
    let witness = scenario(&suite, "relstate.desk.Tickets/reference/desk_id");
    let claims = claims(witness, "relstate.desk.Tickets");
    // The named ticket's desk, read before and after that desk is closed: the view answers the
    // state the desk holds when it is read, not the one it held when the ticket was written.
    let named: Vec<(&Claim, Node)> = claims
        .iter()
        .filter_map(|claim| match &claim.expectation {
            ViewExpectation::Contains { fields } => {
                literal(fields, "desk_state").map(|state| (claim, state))
            }
            _ => None,
        })
        .collect();
    let open = named
        .iter()
        .find(|(_, state)| *state == Node::Text("Open".to_owned()))
        .expect("a read answers a ticket's open desk");
    let closed = named
        .iter()
        .rfind(|(_, state)| *state == Node::Text("Closed".to_owned()))
        .expect("a read answers a ticket's closed desk");
    let ticket = |claim: &Claim| match &claim.expectation {
        ViewExpectation::Contains { fields } => instance(fields, "ticket_id"),
        _ => None,
    };
    assert!(ticket(open.0).is_some(), "the row is named by its ticket");
    let after_close = named
        .iter()
        .find(|(claim, state)| {
            *state == Node::Text("Closed".to_owned()) && ticket(claim) == ticket(open.0)
        })
        .expect("the same ticket is read again with its desk closed");
    let desk_closings = |claim: &Claim| {
        claim
            .sent_before
            .iter()
            .filter(|command| *command == "relstate.desk.CloseDesk")
            .count()
    };
    assert!(
        desk_closings(after_close.0) > desk_closings(open.0),
        "the desk is closed after the ticket's row was read open: {claims:#?}"
    );
    // Its label is read too, and a decoy desk's ticket reads its own desk.
    assert!(
        named.iter().any(|(claim, _)| matches!(&claim.expectation,
            ViewExpectation::Contains { fields } if literal(fields, "desk_label").is_some())),
        "the referenced row's field is asserted: {claims:#?}"
    );
    assert_ne!(ticket(closed.0), None);
    passes_on_the_interpreter(model, &suite);
}

#[test]
fn view_filters_on_the_referenced_rows_current_field() {
    let model = ir(DESK);
    let suite = suite(&model);
    let witness = scenario(&suite, "relstate.desk.TicketsAtOpenDesks/reference/desk_id");
    let claims = claims(witness, "relstate.desk.TicketsAtOpenDesks");
    let named = |claim: &Claim| match &claim.expectation {
        ViewExpectation::Contains { fields } | ViewExpectation::Excludes { fields } => {
            instance(fields, "ticket_id")
        }
        _ => None,
    };
    let shown: Vec<&Claim> = claims
        .iter()
        .filter(|claim| matches!(claim.expectation, ViewExpectation::Contains { .. }))
        .collect();
    let hidden: Vec<&Claim> = claims
        .iter()
        .filter(|claim| matches!(claim.expectation, ViewExpectation::Excludes { .. }))
        .collect();
    let first = shown.first().expect("a ticket at an open desk is shown");
    // A decoy ticket, whose desk is closed, is not shown in the same read.
    assert!(
        hidden
            .iter()
            .any(|claim| claim.sent_before.len() == first.sent_before.len()
                && named(claim) != named(first)),
        "a decoy at a closed desk is excluded beside it: {claims:#?}"
    );
    // Once its desk closes, the shown ticket leaves the view.
    assert!(
        hidden.iter().any(|claim| named(claim) == named(first)
            && claim.sent_before.len() > first.sent_before.len()
            && claim.sent_before.last().map(String::as_str) == Some("relstate.desk.CloseDesk")),
        "the ticket is excluded once its desk is closed: {claims:#?}"
    );
    passes_on_the_interpreter(model, &suite);
}

#[test]
fn absent_reference_reads_absent_everywhere() {
    let model = ir(WINDOW);
    let suite = suite(&model);
    // A payload: the window planned with no incident publishes no incident state.
    let absent_runs: Vec<&ConformanceScenario> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| id.to_string() == "ledger.incident.PlanWindow/outcome/planned")
        .map(|(_, scenario)| scenario)
        .collect();
    assert_eq!(absent_runs.len(), 1);
    let sent_without = absent_runs[0].steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "ledger.incident.PlanWindow" && !input.contains_key("incident_id"))
    });
    assert!(sent_without, "a run plans a window with no incident");
    // Its event's `incident_state` admits no present value: no state, left out or `null`.
    let absent_claim = absent_runs[0]
        .steps
        .iter()
        .skip_while(|step| {
            !matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "ledger.incident.PlanWindow"
                    && !input.contains_key("incident_id"))
        })
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event,
                payload,
                shape,
            } if event.to_string() == "ledger.incident.WindowPlanned" => Some((payload, shape)),
            _ => None,
        })
        .expect("the window planned with no incident publishes its event");
    assert!(!absent_claim.0.contains_key("incident_state"));
    let leaf = serde_json::to_value(absent_claim.1).expect("a shape serialises");
    assert_eq!(
        leaf["incident_state"]["variants"],
        serde_json::json!([]),
        "no state is admitted: {leaf}"
    );
    // A view field: the window with no incident shows no incident state.
    let projected = claims(
        scenario(&suite, "ledger.incident.Windows/reference/incident_id"),
        "ledger.incident.Windows",
    );
    // Shown, with no state claimed, and holding none of the incident's states — left out or `null`,
    // whichever spelling the target publishes.
    let unpointed: Vec<Option<String>> = projected
        .iter()
        .filter_map(|claim| match &claim.expectation {
            ViewExpectation::Contains { fields } if !fields.contains_key("incident_state") => {
                Some(instance(fields, "window_id"))
            }
            _ => None,
        })
        .collect();
    assert_eq!(unpointed.len(), 1, "{projected:#?}");
    for state in ["Open", "Resolved"] {
        assert!(
            projected.iter().any(|claim| matches!(&claim.expectation,
                ViewExpectation::Excludes { fields }
                    if instance(fields, "window_id") == unpointed[0]
                        && literal(fields, "incident_state") == Some(Node::Text(state.to_owned())))),
            "the window with no incident holds no `{state}`: {projected:#?}"
        );
    }
    // A filter: the window with no incident is not among the windows of resolved incidents.
    let filtered = claims(
        scenario(
            &suite,
            "ledger.incident.WindowsOfResolvedIncidents/reference/incident_id",
        ),
        "ledger.incident.WindowsOfResolvedIncidents",
    );
    let excluded: BTreeSet<Option<String>> = filtered
        .iter()
        .filter_map(|claim| match &claim.expectation {
            ViewExpectation::Excludes { fields } => Some(instance(fields, "window_id")),
            _ => None,
        })
        .collect();
    let shown: BTreeSet<Option<String>> = filtered
        .iter()
        .filter_map(|claim| match &claim.expectation {
            ViewExpectation::Contains { fields } => Some(instance(fields, "window_id")),
            _ => None,
        })
        .collect();
    assert!(
        excluded.iter().any(|window| !shown.contains(window)),
        "a window never shown — the one with no incident — is excluded: {filtered:#?}"
    );
    passes_on_the_interpreter(model, &suite);
}

/// Two rows in one branch (downstream report 2026-10-19, finding 11): a wire is refused where both
/// ports face one way, each pair a branch over both rows (`when_related: {all: […]}`).
const WIRING: &str = include_str!("fixtures/two-rows-one-branch.yaml");

/// The direction each `AddPort` the scenario sends gives the port it captures, by instance.
fn ports(scenario: &ConformanceScenario) -> BTreeMap<String, Node> {
    let mut out = BTreeMap::new();
    let mut direction = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "wiring.board.AddPort" =>
            {
                direction = match input.get("direction") {
                    Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                    other => panic!("the direction is a literal: {other:?}"),
                };
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "wiring.board.Port" => {
                out.insert(
                    instance.to_string(),
                    direction.take().expect("a capture follows its AddPort"),
                );
            }
            _ => {}
        }
    }
    out
}

/// The directions of the two ports the scenario's last `Connect` names, source first.
fn connected(scenario: &ConformanceScenario) -> (Node, Node) {
    let ports = ports(scenario);
    let input = scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "wiring.board.Connect" =>
            {
                Some(input)
            }
            _ => None,
        })
        .expect("the scenario connects");
    let port = |field: &str| match input.get(field) {
        Some(ScenarioValue::Instance { instance }) => ports
            .get(&instance.to_string())
            .cloned()
            .unwrap_or_else(|| panic!("`{field}` names an arranged port: {ports:?}")),
        other => panic!("`{field}` names an arranged port: {other:?}"),
    };
    (port("from_port"), port("to_port"))
}

#[test]
fn two_rows_in_one_branch_refuse_only_together() {
    let model = ir(WIRING);
    let suite = suite(&model);
    let text = |value: &str| Node::Text(value.to_owned());
    for (outcome, expected) in [
        ("both-inbound", (text("inbound"), text("inbound"))),
        ("both-outbound", (text("outbound"), text("outbound"))),
    ] {
        let scenario = scenario(&suite, &format!("wiring.board.Connect/outcome/{outcome}"));
        assert_eq!(
            connected(scenario),
            expected,
            "`{outcome}` is sent two ports facing one way"
        );
    }
    let (from, to) = connected(scenario(&suite, "wiring.board.Connect/outcome/connected"));
    assert_ne!(from, to, "a wire is sent two ports facing opposite ways");
    passes_on_the_interpreter(model, &suite);
}
