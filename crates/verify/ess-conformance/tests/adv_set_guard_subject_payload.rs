//! Adversary cases for story:a-set-guard-fed-by-a-subject-payload-is-arranged (beyond10x/ess#705):
//! variants of the `binding-set-guard/subject` fixture around the creator steering
//! (`set_guard::steer_creator`). Every scenario synthesis emits must pass on the interpreter of
//! the same model; where the story's shape is kept, the binding is arranged, not refused.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/binding-set-guard/subject/system.yaml.txt");
const COMPONENTS: &str = include_str!("fixtures/binding-set-guard/subject/components.yaml.txt");
const GRP: &str = include_str!("fixtures/binding-set-guard/subject/grp.yaml.txt");
const RES: &str = include_str!("fixtures/binding-set-guard/subject/res.yaml.txt");

const FLOW: &str = "reconcile-on-leave/binding/flow";
const LEAVE_REMOVED: &str = "example.grp.Leave/outcome/removed";

fn edit(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

fn model(components: &str, grp: &str, res: &str) -> EssIr {
    let files = [
        ("system.yaml", SYSTEM),
        ("components.yaml", components),
        ("grp.yaml", grp),
        ("res.yaml", res),
    ];
    let mut sources = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.push((Source::new(label), raw));
    }
    let spec = Specification::assemble(sources).unwrap_or_else(|error| panic!("{error}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn document(synthesis: &Synthesis) -> Value {
    serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises")).expect("JSON")
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    document(synthesis)["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .clone()
}

fn sends_of<'a>(steps: &'a [Value], command: &str) -> Vec<&'a Value> {
    steps
        .iter()
        .filter(|step| step["step"] == "execute_command" && step["command"] == command)
        .collect()
}

fn failing(model: &EssIr, synthesis: &Synthesis) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<String> = result.diagnostics().map(|d| format!("{d:?}")).collect();
            (
                result.scenario.to_string(),
                format!("{:?}: {diagnostics:#?}", result.status),
            )
        })
        .collect()
}

/// Every scenario emitted passes on the interpreter; the flow, when emitted, is printed on failure.
fn all_pass(ir: &EssIr) -> Synthesis {
    let synthesis = synthesize(ir);
    let failed = failing(ir, &synthesis);
    let flow = document(&synthesis)["scenarios"][FLOW]["steps"].clone();
    assert_eq!(
        failed,
        BTreeMap::new(),
        "refusals: {:#?}\nflow: {flow:#}",
        refusals(&synthesis)
    );
    synthesis
}

fn arranged(synthesis: &Synthesis) {
    let refused: Vec<String> = refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("reconcile-on-leave"))
        .collect();
    assert_eq!(refused, Vec::<String>::new());
}

// The fixture as committed: the step creating the trigger's row is sent a value other than the
// one it is sent where nothing is steered (the trigger's own `Leave/outcome/removed` scenario), or
// `steer_creator` is never exercised and a mutant dropping its `input.insert` survives.
#[test]
fn the_fixture_steers_the_creator_to_a_value_it_would_not_send_unsteered() {
    let ir = model(COMPONENTS, GRP, RES);
    let synthesis = synthesize(&ir);
    let flow = steps_of(&synthesis, FLOW);
    let leave = steps_of(&synthesis, LEAVE_REMOVED);
    let steered = sends_of(&flow, "example.grp.Join")[0]["input"].clone();
    let unsteered = sends_of(&leave, "example.grp.Join")[0]["input"].clone();
    assert_ne!(
        steered["who"], unsteered["who"],
        "flow: {flow:#?}\nLeave/removed: {leave:#?}"
    );
}

// The default branch's guard negated: `kept` when no eligible active membership is left.
#[test]
fn exists_false_is_arranged_and_passes() {
    let res = edit(RES, "exists: true", "exists: false");
    let ir = model(COMPONENTS, GRP, &res);
    let synthesis = all_pass(&ir);
    arranged(&synthesis);
}

// Without `state == Active` the trigger's own row, removed but still a row, can be selected.
#[test]
fn exists_false_over_removed_rows_passes() {
    let res = edit(
        RES,
        "where: {all: [member_id == input.member_id, state == Active, eligible == true]}, exists: true",
        "where: {all: [member_id == input.member_id, eligible == true]}, exists: false",
    );
    all_pass(&model(COMPONENTS, GRP, &res));
}

#[test]
fn exists_true_over_removed_rows_passes() {
    let res = edit(
        RES,
        "where: {all: [member_id == input.member_id, state == Active, eligible == true]}",
        "where: {all: [member_id == input.member_id, eligible == true]}",
    );
    all_pass(&model(COMPONENTS, GRP, &res));
}

fn two_conjuncts() -> (String, String, String) {
    let grp = edit(
        GRP,
        "{name: example.grp.Left, fields: [{name: member_id, type: example.grp.MemberRef}]}",
        "{name: example.grp.Left, fields: [{name: member_id, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]}",
    );
    let grp = edit(
        &grp,
        "payload: {example.grp.Left: {member_id: {subject: member_id}}}",
        "payload: {example.grp.Left: {member_id: {subject: member_id}, eligible: {subject: eligible}}}",
    );
    let res = edit(
        RES,
        "input: [{name: member_id, type: example.grp.MemberRef}]\n    outcomes:\n      - {name: unavailable",
        "input: [{name: member_id, type: example.grp.MemberRef}, {name: flag, type: Boolean}]\n    outcomes:\n      - {name: unavailable",
    );
    let res = edit(
        &res,
        "where: {all: [member_id == input.member_id, state == Active, eligible == true]}",
        "where: {all: [member_id == input.member_id, state == Active, eligible == input.flag]}",
    );
    let components = edit(
        COMPONENTS,
        "mapping: {member_id: event.member_id}",
        "mapping: {member_id: event.member_id, flag: event.eligible}",
    );
    (components, grp, res)
}

// Two inputs the selector reads, both filled from stored fields of the trigger's row.
#[test]
fn two_subject_conjuncts_are_arranged_and_pass() {
    let (components, grp, res) = two_conjuncts();
    let synthesis = all_pass(&model(&components, &grp, &res));
    arranged(&synthesis);
}

// The creator keeps its own branch only for one value of the stored field it is steered on.
#[test]
fn a_creator_guarding_the_steered_field_passes() {
    let (components, grp, res) = two_conjuncts();
    let grp = edit(
        &grp,
        "      - name: joined\n        creates: example.grp.Membership",
        "      - {name: ineligible, when: input.eligible == false, error: example.grp.Conflict}\n      - name: joined\n        creates: example.grp.Membership",
    );
    all_pass(&model(&components, &grp, &res));
}

// The creating command stores the field from an input of another name.
#[test]
fn a_creator_storing_under_another_input_name_is_arranged_and_passes() {
    // The fixture creates the trigger's row from `input.who` already.
    let synthesis = all_pass(&model(COMPONENTS, GRP, RES));
    arranged(&synthesis);
}

// Two commands create the trigger's row.
#[test]
fn two_creators_of_the_trigger_row_pass() {
    let grp = edit(
        GRP,
        "may: [example.grp.Join, example.grp.Leave]",
        "may: [example.grp.Join, example.grp.Enroll, example.grp.Leave]",
    );
    let grp = edit(
        &grp,
        "  - name: example.grp.Leave\n",
        "  - name: example.grp.Enroll\n    input: [{name: member_id, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]\n    outcomes:\n      - name: enrolled\n        creates: example.grp.Membership\n        instance: membership_id\n        emits: [example.grp.Joined]\n        payload: {example.grp.Joined: {membership_id: {generated: true}}}\n        sets: {member_id: input.member_id, eligible: input.eligible}\n  - name: example.grp.Leave\n",
    );
    let components = edit(
        COMPONENTS,
        "commands: [example.grp.Join, example.grp.Leave,",
        "commands: [example.grp.Join, example.grp.Enroll, example.grp.Leave,",
    );
    let synthesis = all_pass(&model(&components, &grp, RES));
    arranged(&synthesis);
}

// The creator's own event carries the field it is steered on.
#[test]
fn a_creator_publishing_the_steered_field_passes() {
    let grp = edit(
        GRP,
        "{name: example.grp.Joined, fields: [{name: membership_id, type: example.grp.MembershipId}]}",
        "{name: example.grp.Joined, fields: [{name: membership_id, type: example.grp.MembershipId}, {name: member_id, type: example.grp.MemberRef}]}",
    );
    let grp = edit(
        &grp,
        "payload: {example.grp.Joined: {membership_id: {generated: true}}}",
        "payload: {example.grp.Joined: {membership_id: {generated: true}, member_id: input.who}}",
    );
    let synthesis = all_pass(&model(COMPONENTS, &grp, RES));
    arranged(&synthesis);
}

// ---- the creator stores `member_id` from `input.who` ------------------------------------------
//
// A UUID witness is hashed from its fact path (`witness::uuid`), so `Join.member_id` and
// `Reconcile.member_id` share one value and the committed fixture never steers anything. Renaming
// the creator's input gives it a value of its own, and `steer_creator` has to replace it.

fn renamed(grp: &str) -> String {
    if grp.contains("{name: who,") {
        return grp.to_owned();
    }
    let grp = edit(
        grp,
        "    input: [{name: member_id, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]\n    outcomes:\n      - name: joined",
        "    input: [{name: who, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]\n    outcomes:\n      - name: joined",
    );
    edit(
        &grp,
        "sets: {member_id: input.member_id,",
        "sets: {member_id: input.who,",
    )
}

fn failing_suite(model: &EssIr, suite: &ConformanceSuite) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| (result.scenario.to_string(), format!("{:?}", result.status)))
        .collect()
}

// The steered flow sends the creator the value the rows were arranged against, not its own, and
// the suite emitted with the creator left unsteered (what a mutant dropping `steer_creator`'s
// `input.insert` emits) fails on the interpreter: this variant catches that mutant.
#[test]
#[ignore = "backlog: story:a-binding-flow-awaits-the-event-its-trigger-causes"]
fn renamed_creator_is_steered_and_the_unsteered_mutant_is_caught() {
    let grp = renamed(GRP);
    let ir = model(COMPONENTS, &grp, RES);
    let synthesis = all_pass(&ir);
    arranged(&synthesis);

    let flow = steps_of(&synthesis, FLOW);
    let leave = steps_of(&synthesis, LEAVE_REMOVED);
    let steered = sends_of(&flow, "example.grp.Join")[0]["input"]["who"].clone();
    let unsteered = sends_of(&leave, "example.grp.Join")[0]["input"]["who"].clone();
    assert_ne!(steered, unsteered, "flow: {flow:#?}");
    let reconciled = sends_of(&flow, "example.grp.Join")
        .iter()
        .skip(1)
        .any(|send| send["input"]["who"] == steered);
    assert!(
        reconciled,
        "a row naming the steered value is arranged: {flow:#?}"
    );
    let eventually: Vec<String> = flow
        .iter()
        .filter(|step| step["step"] == "eventually_event")
        .filter_map(|step| step["event"].as_str().map(str::to_owned))
        .collect();

    let mut raw = document(&synthesis);
    let steps = raw["scenarios"][FLOW]["steps"]
        .as_array_mut()
        .expect("steps");
    let first = steps
        .iter_mut()
        .find(|step| step["step"] == "execute_command" && step["command"] == "example.grp.Join")
        .expect("the creator");
    first["input"]["who"] = unsteered.clone();
    let mutated = first.clone();
    let mutant = ConformanceSuite::from_json(&raw.to_string()).expect("a suite");
    let failed = failing_suite(&ir, &mutant);
    assert!(
        failed.contains_key(FLOW),
        "the unsteered flow fails: {failed:#?}\neventually: {eventually:?}\nsteered {steered} \
         unsteered {unsteered}\nmutated creator: {mutated:#}"
    );
}

// The renamed creator keeps the story's own result: the flow ends in `example.res.Kept`, as the
// committed fixture's does (acceptance, first bullet, over a creator whose input is named apart).
#[test]
fn renamed_creator_flow_ends_in_kept() {
    let synthesis = all_pass(&model(COMPONENTS, &renamed(GRP), RES));
    let flow = steps_of(&synthesis, FLOW);
    let eventually: Vec<&str> = flow
        .iter()
        .filter(|step| step["step"] == "eventually_event")
        .filter_map(|step| step["event"].as_str())
        .collect();
    assert_eq!(eventually, ["example.res.Kept"], "{flow:#?}");
}

// Acceptance, negative bullet: `Join` storing `member_id: {generated: true}` and no `member_id`
// input synthesizes 9 scenarios and 4 refusals, three of them ESS-SYNTH-010.
#[test]
fn the_acceptance_negative_counts() {
    let grp = edit(
        GRP,
        "    input: [{name: who, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]\n    outcomes:\n      - name: joined",
        "    input: [{name: eligible, type: Boolean}]\n    outcomes:\n      - name: joined",
    );
    let grp = edit(
        &grp,
        "sets: {member_id: input.who, eligible: input.eligible}",
        "sets: {member_id: {generated: true}, eligible: input.eligible}",
    );
    let synthesis = synthesize(&model(COMPONENTS, &grp, RES));
    let refused = refusals(&synthesis);
    assert_eq!(
        (
            synthesis.suite.scenarios.len(),
            refused.len(),
            refused
                .iter()
                .filter(|refusal| refusal.contains("ESS-SYNTH-010"))
                .count()
        ),
        (9, 4, 3),
        "{refused:#?}"
    );
}

#[test]
fn renamed_exists_false_passes() {
    let res = edit(RES, "exists: true", "exists: false");
    let synthesis = all_pass(&model(COMPONENTS, &renamed(GRP), &res));
    arranged(&synthesis);
}

#[test]
fn renamed_exists_false_over_removed_rows_passes() {
    let res = edit(
        RES,
        "where: {all: [member_id == input.member_id, state == Active, eligible == true]}, exists: true",
        "where: {all: [member_id == input.member_id, eligible == true]}, exists: false",
    );
    all_pass(&model(COMPONENTS, &renamed(GRP), &res));
}

#[test]
fn renamed_exists_true_over_removed_rows_passes() {
    let res = edit(
        RES,
        "where: {all: [member_id == input.member_id, state == Active, eligible == true]}",
        "where: {all: [member_id == input.member_id, eligible == true]}",
    );
    all_pass(&model(COMPONENTS, &renamed(GRP), &res));
}

#[test]
fn renamed_two_subject_conjuncts_pass() {
    let (components, grp, res) = two_conjuncts();
    let synthesis = all_pass(&model(&components, &renamed(&grp), &res));
    arranged(&synthesis);
}

#[test]
fn renamed_creator_publishing_the_steered_field_passes() {
    let grp = edit(
        &renamed(GRP),
        "{name: example.grp.Joined, fields: [{name: membership_id, type: example.grp.MembershipId}]}",
        "{name: example.grp.Joined, fields: [{name: membership_id, type: example.grp.MembershipId}, {name: member_id, type: example.grp.MemberRef}]}",
    );
    let grp = edit(
        &grp,
        "payload: {example.grp.Joined: {membership_id: {generated: true}}}",
        "payload: {example.grp.Joined: {membership_id: {generated: true}, member_id: input.who}}",
    );
    all_pass(&model(COMPONENTS, &grp, RES));
}

// The trigger compares the stored field with an input of its own: steering the creator alone
// sends the trigger down its refusal, and the binding never fires.
fn guarded_trigger(grp: &str) -> String {
    let grp = edit(
        grp,
        "input: [{name: membership_id, type: example.grp.MembershipId}]",
        "input: [{name: membership_id, type: example.grp.MembershipId}, {name: member_id, type: example.grp.MemberRef}]",
    );
    edit(
        &grp,
        "    outcomes:\n      - name: removed\n",
        "    outcomes:\n      - {name: not-theirs, when_subject: {predicate: member_id != input.member_id}, error: example.grp.Conflict}\n      - name: removed\n",
    )
}

#[test]
fn a_trigger_guarding_the_stored_field_passes() {
    all_pass(&model(COMPONENTS, &guarded_trigger(GRP), RES));
}

#[test]
fn renamed_trigger_guarding_the_stored_field_passes() {
    all_pass(&model(COMPONENTS, &guarded_trigger(&renamed(GRP)), RES));
}

// A second step of the trigger's arrangement sends the creator's value again: the trigger also
// stores it, from its own input, beside the stored field the event carries.
#[test]
fn renamed_trigger_resending_the_stored_value_passes() {
    let grp = edit(
        &renamed(GRP),
        "input: [{name: membership_id, type: example.grp.MembershipId}]",
        "input: [{name: membership_id, type: example.grp.MembershipId}, {name: member_id, type: example.grp.MemberRef}]",
    );
    all_pass(&model(COMPONENTS, &grp, RES));
}
