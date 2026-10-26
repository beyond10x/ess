//! Adversary pass 1 on unit J (synthesis at ess/26 does not stop with `InvalidReplay`).
//!
//! The unit makes a synthesized scenario its own admission refuses a typed refusal instead of a
//! whole-suite stop. These cases ask what that net catches besides the disclosure cell it was
//! built for, and whether the neutral fixture holds on every runner, not only on admission.

mod support_go;
mod support_ts;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ScenarioId};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const FIXTURE: &str = include_str!("fixtures/one-time-followup/external-before-update.yaml");

/// The fixture's updater's external refusal, as the fixture declares it.
const REFUSAL_BRANCH: &str = "      - name: store-unavailable\n        external: the store cannot \
                              be written to\n        error: demo.shop.StoreUnavailable\n        \
                              summary: Nothing changed.\n";

const FORMATS: [&str; 3] = ["ess/25", "ess/26", "ess/32"];

/// The fixture's domain served on a network surface, so grant-denied scenarios are owed.
const SERVED: &str = "components:
  - component: shop-service
    owns: {domains: [demo.shop]}
    accepts: {commands: [demo.shop.CreateItem, demo.shop.Relabel]}
    publishes: {events: [demo.shop.ItemCreated, demo.shop.ItemRelabelled]}
    reached_by: network
";

/// A command with a declared replay of its creating branch, granted to one of two actors and
/// served on a network surface: `<command>/grant/denied` is owed, and the replay scenario reads
/// the row back before its retry.
const REPLAY_SERVED: &str = "format: ess/26
system: retained
version: v1
domain: retained.core
entities:
  - name: retained.core.Record
    identity: {name: record_id, type: Uuid}
    fields:
      - {name: value, type: String}
    lifecycle:
      initial: Committed
      states: [Committed]
      terminal: [Committed]
actors:
  - {name: retained.core.Writer, may: [retained.core.Seed]}
  - {name: retained.core.Reader, may: []}
commands:
  - name: retained.core.Seed
    input: [{name: document, type: String}]
    response:
      - {name: revision_id, type: Uuid}
    outcomes:
      - name: seeded
        creates: retained.core.Record
        instance: record_id
        sets: {value: input.document}
        emits: [retained.core.Seeded]
        payload:
          retained.core.Seeded:
            record_id: {generated: true}
      - name: replayed
        external: retained logical input and trusted context match
        replays: seeded
events:
  - name: retained.core.Seeded
    fields: [{name: record_id, type: Uuid}]
views:
  - name: retained.core.Records
    source: retained.core.Record
    consistency: read_your_writes
    fields:
      - {name: record_id, type: Uuid}
      - {name: value, type: String}
      - {name: state, type: retained.core.Record.State}
components:
  - component: retained-service
    owns: {domains: [retained.core]}
    accepts: {commands: [retained.core.Seed]}
    publishes: {events: [retained.core.Seeded]}
    reached_by: network
";

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the text no longer holds {from:?}");
    text.replace(from, to)
}

fn at_format(text: &str, format: &str) -> String {
    edited(text, "format: ess/26", &format!("format: {format}"))
}

fn self_refused(synthesis: &ess_conformance::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|text| text.contains("own admission refuses"))
        .collect()
}

fn failing(ir: &EssIr, synthesis: &ess_conformance::Synthesis) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let target = Interpreted::for_model(ir.clone());
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// RED (defect). `grant.rs` builds `<command>/grant/denied` from the shortest template that
/// snapshots before an accepted send, which for a declared replay is the replay scenario: the
/// prefix keeps its `capture_command_result` and the denied send drops `expect_replay_result`.
/// The unit's net withdraws the scenario as an ESS defect, so the model loses its grant check.
#[test]
fn adv_j_a_served_replay_keeps_its_grant_denied_scenario() {
    let ir = model(REPLAY_SERVED);
    let synthesis = ess_conformance::synthesize(&ir);
    let denied: ScenarioId = "retained.core.Seed/grant/denied".parse().unwrap();
    assert_eq!(
        self_refused(&synthesis),
        Vec::<String>::new(),
        "synthesis wrote a scenario its own admission refuses"
    );
    assert!(
        synthesis.suite.scenario(&denied).is_some(),
        "no {denied}: {:#?}",
        synthesis.refusals
    );
}

/// GREEN (guard). The withdrawal is the same on every run: refusals, their order and the suite.
#[test]
fn adv_j_a_withdrawal_is_deterministic() {
    let ir = model(REPLAY_SERVED);
    let first = ess_conformance::synthesize(&ir);
    let second = ess_conformance::synthesize(&ir);
    let lines = |s: &ess_conformance::Synthesis| -> Vec<String> {
        s.refusals.iter().map(ToString::to_string).collect()
    };
    assert_eq!(lines(&first), lines(&second));
    assert_eq!(
        first.suite.to_canonical_json().unwrap(),
        second.suite.to_canonical_json().unwrap()
    );
}

/// GREEN (guard). A withdrawn scenario is a refusal of the coverage inventory, counted once, and
/// the inventory's generated list is the written suite.
#[test]
fn adv_j_a_withdrawal_is_counted_in_the_coverage_inventory() {
    let ir = model(REPLAY_SERVED);
    let input = ess_conformance::coverage_build::build(
        &ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let selected = input.selected();
    let inventory = selected.coverage().expect("a coverage inventory");
    let denied: ScenarioId = "retained.core.Seed/grant/denied".parse().unwrap();
    let withdrawn: Vec<_> = inventory
        .refused
        .iter()
        .filter(|refusal| refusal.message.contains("own admission refuses"))
        .collect();
    assert_eq!(
        withdrawn.len(),
        self_refused(&ess_conformance::synthesize(&ir)).len(),
        "the inventory files each withdrawal synthesis made, once"
    );
    for refusal in &withdrawn {
        assert_eq!(refusal.code, "ESS-SYNTH-001");
        assert_eq!(refusal.scenario.as_ref(), Some(&denied));
        assert!(selected.suite().scenario(&denied).is_none());
    }
    assert_eq!(inventory.counts.refused, inventory.refused.len() as u64);
    assert_eq!(inventory.counts.generated, selected.suite().len() as u64);
}

/// GREEN (guard). The fixture with any one ingredient left out, in either branch order and at each
/// format, is admitted and passes on the model's own interpreted target, not only admitted.
#[test]
fn adv_j_every_ingredient_variant_passes_on_the_interpreter() {
    let variants: [(&str, &str, &str); 4] = [
        (
            "one-time response",
            "        one_time_response: [code]\n",
            "",
        ),
        (
            "an actor denied the updater",
            "may: [demo.shop.CreateItem]",
            "may: [demo.shop.CreateItem, demo.shop.Relabel]",
        ),
        ("the external refusal", REFUSAL_BRANCH, ""),
        (
            "a read_your_writes view",
            "consistency: read_your_writes",
            "consistency: eventual",
        ),
    ];
    let mut failures = Vec::new();
    for format in FORMATS {
        for (name, from, to) in variants {
            let text = edited(&at_format(FIXTURE, format), from, to);
            let ir = model(&text);
            let synthesis = ess_conformance::synthesize(&ir);
            let fails = failing(&ir, &synthesis);
            if !fails.is_empty() || !self_refused(&synthesis).is_empty() {
                failures.push(format!(
                    "{format}, no {name}: {fails:?} {:?}",
                    self_refused(&synthesis)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// GREEN (guard). Served on a network surface, the fixture owes `Relabel/grant/denied` too; at each
/// format the suite is admitted with nothing withdrawn and passes on the interpreter.
#[test]
fn adv_j_the_served_fixture_owes_and_passes_its_grant_scenarios() {
    let denied: ScenarioId = "demo.shop.Relabel/grant/denied".parse().unwrap();
    for format in FORMATS {
        let text = format!("{}{SERVED}", at_format(FIXTURE, format));
        let ir = model(&text);
        let synthesis = ess_conformance::synthesize(&ir);
        assert_eq!(self_refused(&synthesis), Vec::<String>::new(), "{format}");
        assert!(synthesis.suite.scenario(&denied).is_some(), "{format}");
        assert_eq!(failing(&ir, &synthesis), Vec::new(), "{format}");
    }
}

/// GREEN (guard). The Go runtime gives the reference verdict on every scenario of the fixture's
/// suite at ess/26, the denied cell's new comparison after a refused send included.
#[test]
fn adv_j_the_fixture_has_go_parity() {
    let ir = model(FIXTURE);
    let synthesis = ess_conformance::synthesize(&ir);
    let verdicts = support_go::assert_parity(
        "adv-j-fixture",
        &synthesis.suite,
        Interpreted::for_model(ir.clone()),
    );
    assert!(support_go::not_passed(&verdicts).is_empty(), "{verdicts:?}");
}

/// GREEN (guard). The same in the TypeScript runtime.
#[test]
fn adv_j_the_fixture_has_typescript_parity() {
    let ir = model(FIXTURE);
    let synthesis = ess_conformance::synthesize(&ir);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("admits");
    let package = support_ts::Package::new("adv-j-fixture", &synthesis.suite);
    let recorder = support_go::Recorder::new(Interpreted::for_model(ir.clone()));
    let reference = support_go::rust_outcomes_admitted(&admitted, &recorder);
    assert!(
        support_go::not_passed(&reference).is_empty(),
        "{reference:?}"
    );
    let replayed = package.replay(&admitted, &recorder.transcript(), false);
    assert_ne!(replayed.exit, Some(2), "{}", replayed.log);
    assert!(
        replayed.divergences.is_empty(),
        "{}\n{}",
        replayed.divergences.join("\n"),
        replayed.log
    );
    assert_eq!(replayed.outcomes, reference, "{}", replayed.log);
}
