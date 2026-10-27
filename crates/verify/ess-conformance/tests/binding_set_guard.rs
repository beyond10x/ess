//! A binding whose invoked command chooses its branch by a set guard,
//! `when_related: {entity:, where: <field> == input.<x>, exists: true}`, with the default branch
//! after it (a downstream adopter's report of 2026-10-26).
//!
//! The input the selector reads is filled from an input the trigger's caller supplies, so a
//! scenario can arrange the rows it selects before the trigger, as it does for a `via:` guard: the
//! flow, delivery and failure scenarios create a `Seat` naming the holder first, then send
//! `OpenSeat` for that holder, and the invocation reaches `kept`. Where no such row can be
//! arranged the refusal names the set guard, and its help is one the specification can follow: it
//! never asks to declare the default branch `external:`, which fails validation.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/binding-set-guard/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/binding-set-guard/components.yaml");
const DOMAIN: &str = include_str!("fixtures/binding-set-guard/a.yaml");

const BINDING: &str = "check-on-seat-opened";
const TRIGGER: &str = "demo.a.OpenSeat";

fn parsed(domain: &str) -> Result<EssIr, String> {
    let files = [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("a.yaml", domain),
    ];
    let mut sources = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).map_err(|error| format!("{label}: {error}"))?;
        sources.push((Source::new(label), raw));
    }
    let spec = Specification::assemble(sources).map_err(|errors| errors.to_string())?;
    compile(&spec, &SourceMap::new()).map_err(|error| format!("{error:?}"))
}

fn model(domain: &str) -> EssIr {
    parsed(domain).unwrap_or_else(|error| panic!("the fixture validates:\n{error}"))
}

/// `DOMAIN` with `part`, which it holds exactly once, replaced by `with`.
fn replaced(part: &str, with: &str) -> String {
    assert_eq!(DOMAIN.matches(part).count(), 1, "written once: {part}");
    DOMAIN.replacen(part, with, 1)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{id}`")))
        .collect()
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

/// Every scenario the interpreter of the same model does not pass, with its diagnostics.
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

#[test]
fn a_binding_arranges_the_row_a_set_guard_selects() {
    let ir = model(DOMAIN);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    assert_eq!(synthesis.suite.scenarios.len(), 8, "8 selected, 0 refused");

    for aspect in ["flow", "delivery", "on-failure"] {
        let id = format!("{BINDING}/binding/{aspect}");
        let steps = steps_of(&synthesis, &id);
        let sends: Vec<&Value> = steps
            .iter()
            .filter(|step| step["step"] == "execute_command" && step["command"] == TRIGGER)
            .collect();
        let (trigger, arranged) = sends.split_last().expect("the trigger is sent");
        let holder = &trigger["input"]["holder_id"];
        assert!(
            arranged
                .iter()
                .any(|send| send["input"]["holder_id"] == *holder),
            "{id}: a seat the set guard selects, held by {holder}, is arranged before the trigger: \
             {steps:#?}"
        );
    }
    // The arranged seat selects `kept`: the flow waits for it, never for the default's event.
    let flow = steps_of(&synthesis, &format!("{BINDING}/binding/flow"));
    let eventually: Vec<&str> = flow
        .iter()
        .filter(|step| step["step"] == "eventually_event")
        .filter_map(|step| step["event"].as_str())
        .collect();
    assert_eq!(eventually, ["demo.a.Kept"]);

    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

#[test]
fn a_binding_refusal_help_is_one_the_specification_can_follow() {
    // The trigger publishes a holder the implementation mints: no seat can be arranged for it, so
    // the refusal remains.
    const PUBLISHED: &str =
        "demo.a.SeatOpened: {seat_id: {generated: true}, holder_id: input.holder_id}";
    let minted = replaced(
        PUBLISHED,
        "demo.a.SeatOpened: {seat_id: {generated: true}, holder_id: {generated: true}}",
    );
    let synthesis = synthesize(&model(&minted));
    let flow = refused_as(&synthesis, &format!("{BINDING}/binding/flow"));
    assert_eq!(flow.len(), 1, "{flow:#?}");
    let refusal = &flow[0];
    // It names the set guard as the reason, not an input the event fills.
    assert!(
        refusal.contains("set guard") && refusal.contains("`demo.a.Seat` where"),
        "{refusal}"
    );
    // Its help never asks to declare a branch `external:`.
    let help = refusal
        .lines()
        .find_map(|line| line.trim().strip_prefix("help: "))
        .unwrap_or_else(|| panic!("a help line: {refusal}"));
    assert!(!help.contains("external:"), "{help}");

    // Declaring the default branch `external:` instead fails validation, as
    // `non_exhaustive_branches` (ESS-COMMAND-005 on the command line): a help naming it could not
    // be followed.
    let external_default = replaced(
        "      - name: dropped\n",
        "      - name: dropped\n        external: no seat is held\n",
    );
    let refused = parsed(&external_default).expect_err("an external default does not validate");
    assert!(refused.contains("non_exhaustive_branches"), "{refused}");

    // What the help asks for, the trigger copying the input the guard reads from an input of its
    // own, validates, and the binding then gets its scenarios.
    assert!(
        help.contains("from an input of its own"),
        "the help offers the trigger copying the input: {help}"
    );
    let followed = model(DOMAIN);
    let synthesis = synthesize(&followed);
    assert_eq!(
        refused_as(&synthesis, &format!("{BINDING}/binding/flow")),
        Vec::<String>::new()
    );
}
