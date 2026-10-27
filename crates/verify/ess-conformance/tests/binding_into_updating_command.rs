//! A binding into a command that updates an existing row and branches on a related row (downstream
//! report, 2026-10-26).
//!
//! `TicketMoved` sets off `Refresh` on the ticket `MoveTicket` moved: `Refresh` `updates:` the row
//! its `ticket_id` names, and its branches are `when_related:` guards over the grant its
//! `grant_id` names — `exists: false`, a predicate, then the default. `MoveTicket` copies its own
//! `grant_id` input into the event, so a scenario can define the grant first and send its identity:
//! the grant decides the branch, as it does where `Refresh` acts on no row of its own. Where the
//! grant cannot be named before the trigger, the refusal names the guard and the row.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/binding-into-updating-command/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/binding-into-updating-command/components.yaml");
const DOMAIN: &str = include_str!("fixtures/binding-into-updating-command/b.yaml");

const BINDING: &str = "refresh-on-ticket-moved";

fn model(components: &str) -> EssIr {
    let parsed = [
        ("system.yaml", SYSTEM),
        ("components.yaml", components),
        ("b.yaml", DOMAIN),
    ]
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .clone()
}

/// The grant `MoveTicket` is sent naming in `id`, and whether the scenario captured it first.
fn named_grant(synthesis: &Synthesis, id: &str) -> (String, bool) {
    let steps = steps_of(synthesis, id);
    let trigger = steps
        .iter()
        .rev()
        .find(|step| step["command"] == "demo.b.MoveTicket")
        .unwrap_or_else(|| panic!("{id}: the trigger"));
    let named = trigger["input"]["grant_id"]["instance"]
        .as_str()
        .unwrap_or_else(|| panic!("{id}: `MoveTicket` names an arranged grant: {trigger}"))
        .to_owned();
    let captured = steps.iter().any(|step| {
        step["step"] == "capture_instance"
            && step["entity"] == "demo.b.Grant"
            && step["instance"] == named.as_str()
    });
    (named, captured)
}

#[test]
fn a_binding_into_an_updating_command_follows_its_related_guards() {
    let ir = model(COMPONENTS);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    assert_eq!(
        synthesis.suite.scenarios.len(),
        11,
        "{:#?}",
        synthesis.suite.scenarios.keys()
    );

    // The grant is defined and captured before the trigger, and `MoveTicket` names it: in the
    // binding's own scenarios, and in the trigger's, where the ticket comes to rest through it.
    for id in [
        format!("{BINDING}/binding/flow"),
        format!("{BINDING}/binding/mapping"),
        format!("{BINDING}/binding/delivery"),
        "demo.b.MoveTicket/outcome/moved".to_owned(),
    ] {
        let (named, captured) = named_grant(&synthesis, &id);
        assert!(captured, "{id}: {named} is arranged");
    }
    let flow = steps_of(&synthesis, &format!("{BINDING}/binding/flow"));
    let eventually: Vec<&str> = flow
        .iter()
        .filter(|step| step["step"] == "eventually_event")
        .filter_map(|step| step["event"].as_str())
        .collect();
    assert_eq!(eventually, ["demo.b.Refreshed"]);

    // The suite passes the interpreted target, the dropped refusal included.
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let not_passed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(not_passed, Vec::<String>::new());
    for id in [
        format!("{BINDING}/binding/flow"),
        format!("{BINDING}/binding/delivery"),
        format!("{BINDING}/binding/refusal/grant-missing"),
        "demo.b.MoveTicket/outcome/moved".to_owned(),
    ] {
        assert!(
            report
                .scenarios
                .iter()
                .any(|scenario| scenario.scenario.to_string() == id),
            "`{id}` ran"
        );
    }
}

#[test]
fn a_related_guard_still_unfollowed_names_the_guard_and_the_row() {
    // The binding fills `grant_id` with a constant now: no grant a scenario defines carries it, so
    // the guard cannot be followed, and the refusal says which guard reads which row.
    let from = "grant_id: event.grant_id";
    assert_eq!(COMPONENTS.matches(from).count(), 1);
    let components = COMPONENTS.replacen(from, "grant_id: some-grant", 1);
    let synthesis = synthesize(&model(&components));
    let refused = refusals(&synthesis);
    let unfollowed: Vec<&String> = refused
        .iter()
        .filter(|refusal| {
            refusal.contains(&format!("`{BINDING}/binding/flow`"))
                || refusal.contains(&format!("`{BINDING}/binding/delivery`"))
                || refusal.contains("`demo.b.MoveTicket/outcome/moved`")
        })
        .collect();
    assert_eq!(unfollowed.len(), 3, "{refused:#?}");
    for refusal in unfollowed {
        assert!(
            !refusal.contains("an input the event fills"),
            "the event is not blamed: {refusal}"
        );
        for named in [
            "`when_related:`",
            "`demo.b.Grant`",
            "`grant_id`",
            "grant-missing",
        ] {
            assert!(refusal.contains(named), "names {named}: {refusal}");
        }
    }
}
