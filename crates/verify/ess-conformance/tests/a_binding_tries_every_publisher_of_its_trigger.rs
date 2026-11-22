//! A binding whose trigger event two commands publish (a downstream adopter's report of
//! 2026-10-26).
//!
//! `demo.e.Touch` fills the event's `grant_id` from its input, so a scenario chooses the `Grant`
//! row the bound `demo.e.Give` reads and decides its branch; `demo.e.SyncAll` fills it once per
//! `Ticket` row it changes (`{changed: grant_id}`), so the row is the upstream implementation's.
//! Synthesis tried only the lower-named publisher, `SyncAll`, and refused the binding's flow,
//! delivery and `refusal/missing` scenarios although `Touch` witnesses them, so renaming `Touch`
//! to `Bump` changed the suite. It now tries every publishing branch in the model's order and
//! uses the first that witnesses the binding.
//!
//! The mapping scenario it wrote over `SyncAll` sent it with no `Ticket` row, so the honest
//! system published no event and the scenario failed the interpreted target: a trigger that
//! publishes its event once per changed row now has a row its filter selects arranged first.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{binding_triggers_tried, synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const SYSTEM: &str = include_str!("fixtures/binding-every-publisher/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/binding-every-publisher/components.yaml");
const DOMAIN: &str = include_str!("fixtures/binding-every-publisher/domains/e.yaml");

const RENAMED_SYSTEM: &str = include_str!("fixtures/binding-every-publisher-renamed/system.yaml");
const RENAMED_COMPONENTS: &str =
    include_str!("fixtures/binding-every-publisher-renamed/components.yaml");
const RENAMED_DOMAIN: &str =
    include_str!("fixtures/binding-every-publisher-renamed/domains/e.yaml");

const BINDING: &str = "give-on-changed";

fn model(system: &str, components: &str, domain: &str) -> EssIr {
    let parsed = [
        ("system.yaml", system),
        ("components.yaml", components),
        ("domains/e.yaml", domain),
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

fn binding_scenarios(synthesis: &Synthesis) -> BTreeSet<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with(&format!("{BINDING}/binding/")))
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

/// Every scenario of `synthesis` that does not pass the interpreted target of `ir`.
fn not_passed(ir: EssIr, synthesis: &Synthesis) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect()
}

#[test]
fn a_binding_tries_every_publisher_of_its_trigger() {
    let expected: BTreeSet<String> = ["delivery", "flow", "mapping", "refusal/missing"]
        .map(|aspect| format!("{BINDING}/binding/{aspect}"))
        .into_iter()
        .collect();
    let mut sets = Vec::new();
    for (label, system, components, domain) in [
        ("as committed", SYSTEM, COMPONENTS, DOMAIN),
        (
            "renamed",
            RENAMED_SYSTEM,
            RENAMED_COMPONENTS,
            RENAMED_DOMAIN,
        ),
    ] {
        let ir = model(system, components, domain);
        let synthesis = synthesize(&ir);
        let binding_refusals: Vec<String> = refusals(&synthesis)
            .into_iter()
            .filter(|refusal| refusal.contains("ESS-SYNTH-010"))
            .collect();
        assert_eq!(binding_refusals, Vec::<String>::new(), "{label}");
        let set = binding_scenarios(&synthesis);
        assert_eq!(set, expected, "{label}");
        // The trigger is the publisher that fills the field from its input, whatever its name.
        for id in &expected {
            let steps = steps_of(&synthesis, id);
            let triggers: Vec<&str> = steps
                .iter()
                .filter(|step| step["step"] == "execute_command")
                .filter_map(|step| step["command"].as_str())
                .filter(|command| *command != "demo.e.DefineGrant")
                .collect();
            assert!(
                !triggers.contains(&"demo.e.SyncAll"),
                "{label}: {id} is set off by the input-filling publisher: {triggers:?}"
            );
        }
        assert_eq!(not_passed(ir, &synthesis), Vec::<String>::new(), "{label}");
        sets.push(set);
    }
    assert_eq!(sets[0], sets[1]);
}

#[test]
fn a_trigger_publishing_once_per_changed_row_is_sent_with_a_selected_row() {
    // `SyncAll` alone publishes the event: the mapping is witnessed through it, with a `Ticket`
    // row its filter selects arranged before it, so the honest system publishes the event.
    let command = "  - name: demo.e.Touch\n    input:\n      - {name: grant_id, type: demo.e.GrantId}\n    outcomes:\n      - name: touched\n        emits: [demo.e.Changed]\n        payload: {demo.e.Changed: {grant_id: input.grant_id}}\n";
    assert_eq!(DOMAIN.matches(command).count(), 1);
    let domain = DOMAIN.replacen(command, "", 1);
    let accepted = "demo.e.Touch, ";
    assert_eq!(COMPONENTS.matches(accepted).count(), 1);
    let components = COMPONENTS.replacen(accepted, "", 1);
    let ir = model(SYSTEM, &components, &domain);
    let synthesis = synthesize(&ir);
    let mapping = steps_of(&synthesis, &format!("{BINDING}/binding/mapping"));
    let commands: Vec<&str> = mapping
        .iter()
        .filter(|step| step["step"] == "execute_command")
        .filter_map(|step| step["command"].as_str())
        .collect();
    assert_eq!(
        commands,
        ["demo.e.OpenTicket", "demo.e.SyncAll"],
        "{mapping:#?}"
    );
    assert_eq!(not_passed(ir, &synthesis), Vec::<String>::new());
}

/// `publishers` commands publishing the trigger of six bindings, each invoking a command with no
/// branch a failure can be forced on.
fn unforced(publishers: usize) -> EssIr {
    let publishing: String = (0..publishers).fold(String::new(), |mut text, i| {
        let _ = writeln!(
            text,
            "  - name: shop.work.Publish{i:02}\n    outcomes: [{{name: published, emits: [shop.work.Created]}}]"
        );
        text
    });
    let recording: String = (0..6).fold(String::new(), |mut text, i| {
        let _ = writeln!(
            text,
            "  - name: shop.work.Record{i}\n    input: [{{name: category, type: String}}]\n    outcomes: [{{name: recorded, emits: [shop.work.Recorded]}}]"
        );
        text
    });
    let accepted = (0..publishers)
        .map(|i| format!("shop.work.Publish{i:02}"))
        .chain((0..6).map(|i| format!("shop.work.Record{i}")))
        .collect::<Vec<_>>()
        .join(", ");
    let bindings: String = (0..6).fold(String::new(), |mut text, i| {
        let _ = writeln!(
            text,
            "  - id: send-record-{i}
    when: {{event: shop.work.Created}}
    invoke: {{command: shop.work.Record{i}}}
    mapping: {{category: note}}
    delivery: at_least_once
    on_failure: retry"
        );
        text
    });
    let text = format!(
        "format: ess/25
system: shop
version: v1
domain: shop.work
commands:
{publishing}{recording}events: [{{name: shop.work.Created, fields: []}}, {{name: shop.work.Recorded, fields: []}}]
components:
  - component: svc
    owns: {{domains: [shop.work]}}
    accepts: {{commands: [{accepted}]}}
    publishes: {{events: [shop.work.Created, shop.work.Recorded]}}
bindings:
{bindings}"
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("work.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("{d}"))
}

/// A refusal that reads nothing of the publisher is not tried again per publisher: where the
/// invoked command has no branch a failure can be forced on, every publisher leaves the binding's
/// on-failure aspect refused, so once one leaves nothing else refused, the publishers after it are
/// not tried. Twelve eligible publishers built the scenarios of each binding twelve times, 72
/// builds where one publisher took 6, for the same scenarios and refusals. A work bound rather than
/// a clock.
#[test]
fn a_refusal_no_publisher_changes_is_not_tried_again_per_publisher() {
    let (one, tried_one) = binding_triggers_tried(|| synthesize(&unforced(1)));
    let (twelve, tried) = binding_triggers_tried(|| synthesize(&unforced(12)));
    let bindings = |synthesis: &Synthesis| -> (Vec<String>, Vec<String>) {
        (
            synthesis
                .suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .filter(|id| id.contains("/binding/"))
                .collect(),
            synthesis.refusals.iter().map(ToString::to_string).collect(),
        )
    };
    assert_eq!(bindings(&twelve), bindings(&one));
    assert_eq!(twelve.refusals.len(), 6, "{:#?}", twelve.refusals);
    assert_eq!(tried_one, 6);
    assert_eq!(tried, 6, "six bindings tried {tried} publishers");
}
