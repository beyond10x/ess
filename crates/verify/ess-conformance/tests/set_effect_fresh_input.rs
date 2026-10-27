//! A follow-up whose set effect selects rows by a value only the follow-up sends composes after a
//! one-time response's origin.
//!
//! The disclosure cell `command/<follow-up>/<branch>` appends the follow-up's own run after the
//! origin. A set effect (`instances: {where: …}`) in that run selects every stored row its filter
//! holds for, the origin's included, so the run is appended only where its filter pins a stored
//! field to an input and every send of the command carries a value of that input the origin's run
//! never sent: no row the origin left holds it, so the run selects its own rows and no other. A
//! filter that pins nothing so, or an input whose value the origin sent too, would select the
//! origin's rows as well, and the cell stays refused with a message naming the filter.
use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioId, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SYSTEM: &str = include_str!("fixtures/set-effect-fresh-input/system.yaml");
const KEYS: &str = include_str!("fixtures/set-effect-fresh-input/domains/keys.yaml");
const SEATS: &str = include_str!("fixtures/set-effect-fresh-input/domains/seats.yaml");

const CELL: &str =
    "demo.w.keys.Issue/disclosure/issued/secret/command/demo.w.seats.Park/parked/as/anonymous";
const SELECTOR: &str = "instances: {where: owner == input.owner}";
const ORIGIN: &str = "demo.w.keys.Issue";
const PARK: &str = "demo.w.seats.Park";

fn model(seats: &str) -> EssIr {
    let documents: Vec<_> = [
        ("system.yaml", SYSTEM),
        ("domains/keys.yaml", KEYS),
        ("domains/seats.yaml", seats),
    ]
    .into_iter()
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    })
    .collect();
    let spec = Specification::assemble(documents).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// The one refusal of the scenario `cell`.
fn refusal_of(synthesis: &Synthesis, cell: &str) -> String {
    let refused: Vec<String> = refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{cell}`")))
        .collect();
    assert_eq!(refused.len(), 1, "{:#?}", refusals(synthesis));
    refused.into_iter().next().unwrap()
}

/// Every scenario the suite holds that does not pass on the model's own interpreted target.
fn failing(ir: &EssIr, synthesis: &Synthesis) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = Interpreted::for_model(ir.clone());
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The literal `field` each send of `command` in `steps` carries, in order.
fn sent(steps: &[ScenarioStep], command: &str, field: &str) -> Vec<serde_json::Value> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: named,
                input,
                ..
            } if named.to_string() == command => match input.get(field) {
                Some(ScenarioValue::Literal { value }) => serde_json::to_value(value).ok(),
                other => panic!("`{command}` sends `{field}` as {other:?}, not a literal"),
            },
            _ => None,
        })
        .collect()
}

/// The repro: the disclosure cell of the follow-up whose set effect selects by `input.owner` is
/// synthesized, 10 scenarios and no refusal (the nine it had and the cell; the `instance: owner`
/// form has an eleventh, the single-row move's transition scenario, which a set move does not
/// have). It passes the interpreted target, and the appended run's set effect selects only its own
/// rows: no step of the origin's run carries an owner the appended run parks by, so no row the
/// origin left is selected.
#[test]
fn a_set_effect_bound_to_a_fresh_input_composes_after_a_disclosure() {
    assert!(SEATS.contains(SELECTOR), "the fixture selects by input");
    let ir = model(SEATS);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    assert_eq!(synthesis.suite.scenarios.len(), 10);
    let id: ScenarioId = CELL.parse().unwrap();
    let cell = synthesis
        .suite
        .scenario(&id)
        .unwrap_or_else(|| panic!("no cell {id}"));
    let appended = cell
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() != ORIGIN)
        })
        .expect("the cell appends the follow-up's run after the origin");
    let (origin, followup) = cell.steps.split_at(appended);
    let parked = sent(followup, PARK, "owner");
    assert!(!parked.is_empty(), "the appended run parks: {followup:#?}");
    let origin = serde_json::to_string(origin).unwrap();
    for owner in &parked {
        assert!(
            !origin.contains(&owner.to_string()),
            "the origin's run carries the owner {owner} the appended run parks by"
        );
    }
    assert_eq!(failing(&ir, &synthesis), Vec::new());
}

/// A set effect whose filter pins no stored field to an input would select the origin's rows too:
/// the cell stays refused, and the refusal names the filter rather than the origin state.
#[test]
fn a_set_effect_selector_binding_no_per_run_value_stays_refused() {
    let seats = SEATS.replace(SELECTOR, "instances: {where: owner != input.owner}");
    assert_ne!(seats, SEATS);
    let refusal = refusal_of(&synthesize(&model(&seats)), CELL);
    assert!(
        refusal.contains("`demo.w.seats.Park/parked`")
            && refusal.contains("instances: {where: owner != input.owner}"),
        "the refusal names the selector: {refusal}"
    );
    assert!(
        !refusal.contains("retained origin state"),
        "the refusal blames the origin state: {refusal}"
    );
}

/// The origin itself writes a seat's owner from its input and the follow-up parks by the same
/// owner value: the filter pins `owner` to `input.owner`, but that value is not the follow-up's
/// alone, so the follow-up would select the origin's seat. The cell stays refused, naming the
/// filter.
#[test]
fn a_set_effect_bound_to_a_value_the_origin_sent_stays_refused() {
    let seats = SEATS
        .replace(
            "  - name: demo.w.seats.Grant\n    input:\n      - {name: owner, type: Uuid}\n",
            "  - name: demo.w.seats.Grant\n    input:\n      - {name: owner, type: Uuid}\n    \
             response:\n      - {name: secret, type: String}\n",
        )
        .replace(
            "        sets: {owner: input.owner}\n",
            "        sets: {owner: input.owner}\n        returns: true\n        \
             one_time_response: [secret]\n",
        );
    assert_eq!(seats.matches("one_time_response").count(), 1, "{seats}");
    let cell =
        "demo.w.seats.Grant/disclosure/granted/secret/command/demo.w.seats.Park/parked/as/anonymous";
    let refusal = refusal_of(&synthesize(&model(&seats)), cell);
    assert!(
        refusal.contains("`demo.w.seats.Park/parked`") && refusal.contains(SELECTOR),
        "the refusal names the selector: {refusal}"
    );
    assert!(
        !refusal.contains("retained origin state"),
        "the refusal blames the origin state: {refusal}"
    );
}
