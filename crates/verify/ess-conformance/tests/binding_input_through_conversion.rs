//! A binding whose invoked command reads a related row through an input the binding fills through
//! a declared conversion (downstream report, 2026-10-26).
//!
//! `MemberCreated` carries a `GroupRef`, the binding maps it onto `Enrol`'s `group_id: GroupId`
//! through the declared conversion `GroupRef -> GroupId`, and `Enrol` reads the group through
//! `when_related: {via: input.group_id, exists: false}`. Both types wrap the same representation
//! and hold it to nothing more, so the conversion hands the value on as it is: a scenario can define
//! a group and send its identity as `CreateMember`'s `group_ref`, as it does where the event carries
//! the `GroupId` itself. A conversion that changes the representation computes a value no scenario
//! knows, and the refusal names it.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/binding-input-through-conversion/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/binding-input-through-conversion/components.yaml");
const DOMAIN: &str = include_str!("fixtures/binding-input-through-conversion/x.yaml");

const BINDING: &str = "enrol-on-member-created";

fn model(domain: &str) -> EssIr {
    let parsed = [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("x.yaml", domain),
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
fn a_converted_binding_input_arranges_the_related_row() {
    let ir = model(DOMAIN);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    assert_eq!(
        synthesis.suite.scenarios.len(),
        8,
        "{:#?}",
        synthesis.suite.scenarios.keys()
    );

    // The group is defined and captured before the trigger, and `CreateMember` names it.
    for aspect in ["flow", "mapping", "delivery"] {
        let id = format!("{BINDING}/binding/{aspect}");
        let steps = steps_of(&synthesis, &id);
        let captured: Vec<&str> = steps
            .iter()
            .filter(|step| step["step"] == "capture_instance" && step["entity"] == "demo.x.Group")
            .filter_map(|step| step["instance"].as_str())
            .collect();
        let trigger = steps
            .iter()
            .find(|step| step["command"] == "demo.x.CreateMember")
            .unwrap_or_else(|| panic!("{id}: the trigger"));
        let named = trigger["input"]["group_ref"]["instance"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: `CreateMember` names an arranged group: {trigger}"));
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
    assert_eq!(eventually, ["demo.x.Enrolled"]);

    // The suite passes the interpreted target, the refusal scenario included.
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
    for aspect in ["flow", "delivery", "refusal/group-missing"] {
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

#[test]
fn a_conversion_that_changes_the_representation_is_named_in_the_refusal() {
    // The reference is a number now: what `GroupRef -> GroupId` makes of it is the
    // implementation's, so no arranged group can be named through it.
    let from = "{name: demo.x.GroupRef, kind: newtype, of: String}";
    assert_eq!(DOMAIN.matches(from).count(), 1);
    let domain = DOMAIN.replacen(
        from,
        "{name: demo.x.GroupRef, kind: newtype, of: Integer}",
        1,
    );
    let synthesis = synthesize(&model(&domain));
    let flow: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{BINDING}/binding/flow`")))
        .collect();
    assert!(
        flow.iter().any(|refusal| {
            refusal.contains("whose branch is decided by an input the event fills")
                && refusal.contains("`demo.x.GroupRef` -> `demo.x.GroupId`")
        }),
        "{flow:#?}"
    );
}
