//! A binding whose trigger refuses the same missing related row the invoked command reads
//! (downstream report, 2026-10-26).
//!
//! `Enrol` answers `when_related: {via: input.group_id, exists: false}` with a refusal before it
//! publishes `Enrolled`, and the binding maps that `group_id` onto `Grant`, which reads the same
//! group through the same guard. The trigger publishes only while the group exists, so the flow and
//! delivery scenarios define a group and send `Enrol` naming it, and the invocation reaches
//! `granted`. The `group-missing` refusal is the one cell no trigger reaches, and its refusal names
//! the trigger's guard.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/binding-trigger-shares-row-guard/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/binding-trigger-shares-row-guard/components.yaml");
const DOMAIN: &str = include_str!("fixtures/binding-trigger-shares-row-guard/x.yaml");

const BINDING: &str = "grant-on-enrolled";

fn model() -> EssIr {
    let parsed = [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("x.yaml", DOMAIN),
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

#[test]
fn a_binding_whose_trigger_shares_the_row_guard_flows() {
    let ir = model();
    let synthesis = synthesize(&ir);

    // The one cell no trigger reaches stays refused, naming the trigger's guard.
    let refused = refusals(&synthesis);
    assert_eq!(refused.len(), 1, "{refused:#?}");
    let missing = format!("`{BINDING}/binding/refusal/group-missing`");
    assert!(
        refused[0].contains("ESS-SYNTH-010")
            && refused[0].contains(&missing)
            && refused[0].contains("`demo.x.Enrol` first, refuses that missing row itself"),
        "{refused:#?}"
    );
    assert_eq!(
        synthesis.suite.scenarios.len(),
        8,
        "{:#?}",
        synthesis.suite.scenarios.keys()
    );

    // The group is defined before the trigger, and `Enrol` is sent naming it.
    for aspect in ["flow", "delivery"] {
        let id = format!("{BINDING}/binding/{aspect}");
        let steps = steps_of(&synthesis, &id);
        let captured: Vec<&str> = steps
            .iter()
            .filter(|step| step["step"] == "capture_instance" && step["entity"] == "demo.x.Group")
            .filter_map(|step| step["instance"].as_str())
            .collect();
        let defined = steps
            .iter()
            .position(|step| step["command"] == "demo.x.DefineGroup")
            .unwrap_or_else(|| panic!("{id}: a group is defined"));
        let sent = steps
            .iter()
            .rposition(|step| step["command"] == "demo.x.Enrol")
            .unwrap_or_else(|| panic!("{id}: the trigger"));
        assert!(defined < sent, "{id}: the group exists before the trigger");
        let named = steps[sent]["input"]["group_id"]["instance"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: `Enrol` names an arranged group: {}", steps[sent]));
        assert!(
            captured.contains(&named),
            "{id}: {named} is arranged: {captured:?}"
        );
    }
    let flow = steps_of(&synthesis, &format!("{BINDING}/binding/flow"));
    let eventually: Vec<&str> = flow
        .iter()
        .filter(|step| step["step"] == "eventually_event")
        .filter_map(|step| step["event"].as_str())
        .collect();
    assert_eq!(eventually, ["demo.x.Granted"]);

    // The suite passes the interpreted target, flow and delivery included.
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
    for aspect in ["flow", "delivery"] {
        let id = format!("{BINDING}/binding/{aspect}");
        assert!(
            report
                .scenarios
                .iter()
                .any(|scenario| scenario.scenario.to_string() == id),
            "`{id}` ran"
        );
    }
}
