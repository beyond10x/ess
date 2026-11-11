//! A row-set guard over a field a creating branch copies from another row with
//! `{related: {via, field}}` (downstream report, 2026-10-18).
//!
//! `tests/fixtures/row-set-related-copy.yaml`: `Issue` stores on a seat whether its tier
//! qualifies, read from the tier `input.tier_id` names; `Check` answers `qualified` when a seat of
//! the holder qualifies. The seat's `qualifies` is nothing the issuing input carries, so synthesis
//! arranges it through the tier: a tier defined with the value the row needs, then the seat issued
//! under it. Before this the walk over the steps left the copied value unknown, and both branches
//! of `Check` were refused with `ESS-SYNTH-001` ("its own row set is Unknown").

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
    ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/row-set-related-copy.yaml");
const COPY: &str = "qualifies: {related: {via: input.tier_id, field: qualifies}}";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("row-set-related-copy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&model(MODEL));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn run(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, target)
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

#[test]
fn both_branches_of_a_row_set_over_a_copied_field_are_synthesized() {
    let ids: Vec<String> = suite().scenarios.keys().map(ToString::to_string).collect();
    for required in [
        "demo.seats.Check/outcome/qualified",
        "demo.seats.Check/outcome/not-qualified",
    ] {
        assert!(ids.iter().any(|id| id == required), "{required}: {ids:#?}");
    }
}

#[test]
fn the_copied_value_is_steered_through_the_row_it_is_copied_from() {
    let suite = suite();
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.seats.Check/outcome/qualified")
        .map(|(_, scenario)| scenario)
        .expect("synthesized");
    // Every tier the scenario defines, by the value it qualifies with.
    let mut defined: Vec<bool> = Vec::new();
    let mut issued_under_arranged_tier = 0;
    for step in &scenario.steps {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        match command.to_string().as_str() {
            "demo.seats.DefineTier" => match input.get("qualifies") {
                Some(ScenarioValue::Literal {
                    value: Node::Bool(flag),
                }) => defined.push(*flag),
                other => panic!("a tier is defined with a literal: {other:?}"),
            },
            "demo.seats.Issue" => {
                assert!(
                    matches!(input.get("tier_id"), Some(ScenarioValue::Instance { .. })),
                    "a seat is issued under a tier the scenario defined: {input:?}"
                );
                issued_under_arranged_tier += 1;
            }
            _ => {}
        }
    }
    assert!(
        defined.contains(&true) && defined.contains(&false),
        "the selected seat's tier qualifies and the decoy's does not: {defined:?}"
    );
    assert!(
        issued_under_arranged_tier >= 3,
        "{issued_under_arranged_tier}"
    );
}

/// [`MODEL`] with `Requalify` copying a tier's value onto an existing seat: the copy is written by
/// an update.
fn copied_on_update() -> String {
    MODEL
        .replace(
            "    may: [demo.seats.DefineTier, demo.seats.Issue, demo.seats.Check]",
            "    may: [demo.seats.DefineTier, demo.seats.Issue, demo.seats.Requalify, demo.seats.Check]",
        )
        .replace(
            "  - name: demo.seats.Check\n",
            "  - name: demo.seats.Requalify
    input:
      - {name: seat_id, type: demo.seats.SeatId}
      - {name: tier_id, type: demo.seats.TierId}
    outcomes:
      - name: requalified
        updates: demo.seats.Seat
        instance: seat_id
        sets: {qualifies: {related: {via: input.tier_id, field: qualifies}}}
        emits: [demo.seats.Issued]
        payload:
          demo.seats.Issued: {seat_id: input.seat_id}
  - name: demo.seats.Check\n",
        )
}

/// `text` synthesized with nothing refused, and its verdicts against the interpreter of `text` and
/// of each of `faulty`.
fn verdicts(
    label: &str,
    text: &str,
    faulty: &[String],
) -> (BTreeMap<String, Status>, Vec<BTreeMap<String, Status>>) {
    assert_ne!(text, MODEL, "{label}: the variant is written");
    let synthesis = synthesize(&model(text));
    assert!(
        synthesis.refusals.is_empty(),
        "{label}: nothing is refused: {:#?}",
        synthesis.refusals
    );
    let suite = synthesis.suite;
    let healthy = run(&suite, &Interpreted::for_model(model(text)));
    assert!(
        healthy.values().all(|status| *status == Status::Passed),
        "{label}: {healthy:#?}"
    );
    let faulty = faulty
        .iter()
        .map(|faulty| run(&suite, &Interpreted::for_model(model(faulty))))
        .collect();
    (healthy, faulty)
}

/// A copy written by an update is observed: the update's own scenario passes the interpreter and
/// fails a target storing either constant instead of the tier's value (review 2026-10-19, T3).
#[test]
fn a_copy_written_by_an_update_is_read_through_its_row() {
    let text = copied_on_update();
    let copy = "sets: {qualifies: {related: {via: input.tier_id, field: qualifies}}}";
    assert!(text.contains(copy));
    let faulty: Vec<String> = ["true", "false"]
        .into_iter()
        .map(|constant| text.replace(copy, &format!("sets: {{qualifies: {constant}}}")))
        .collect();
    let (healthy, faulty) = verdicts("update", &text, &faulty);
    let id = "demo.seats.Requalify/outcome/requalified";
    assert!(healthy.contains_key(id), "{healthy:#?}");
    assert!(
        faulty.iter().any(|statuses| statuses[id] != Status::Passed),
        "{faulty:#?}"
    );
}

#[test]
fn the_interpreter_passes_and_a_target_copying_no_tier_fails() {
    let suite = suite();
    let healthy = run(&suite, &Interpreted::for_model(model(MODEL)));
    assert!(
        healthy.values().all(|status| *status == Status::Passed),
        "{healthy:#?}"
    );
    // A target storing `qualifies: true` on every seat, whatever its tier holds.
    assert!(MODEL.contains(COPY));
    let faulty = MODEL.replace(COPY, "qualifies: true");
    let statuses = run(&suite, &Interpreted::for_model(model(&faulty)));
    assert_ne!(
        statuses.get("demo.seats.Check/outcome/not-qualified"),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
}

const IDENTITY: &str = include_str!("fixtures/row-set-related-copy-identity.yaml");

/// A guard over a field the creating branch fills with the related row's identity is witnessed:
/// the tier `Check` is sent is the one an earlier step defined and an earlier `Issue` was sent
/// (review 2026-10-19, T3).
#[test]
fn a_copied_related_identity_is_witnessed() {
    let identity = model(IDENTITY);
    let synthesis = synthesize(&identity);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    let suite = synthesis.suite;
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.seats.Check/outcome/qualified")
        .map(|(_, scenario)| scenario)
        .expect("demo.seats.Check/outcome/qualified exists");
    let check = scenario
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.seats.Check")
        })
        .expect("the scenario sends Check");
    let ScenarioStep::ExecuteCommand { input, .. } = &scenario.steps[check] else {
        unreachable!()
    };
    let Some(ScenarioValue::Instance { instance }) = input.get("tier_id") else {
        panic!("Check is sent a tier instance: {input:?}");
    };
    let before = &scenario.steps[..check];
    assert!(
        before.iter().any(|step| matches!(step,
            ScenarioStep::CaptureInstance { instance: name, entity, .. }
                if name == instance && entity.to_string() == "demo.seats.Tier")),
        "an earlier step created the tier: {before:#?}"
    );
    assert!(
        before.iter().any(|step| matches!(step,
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.seats.Issue"
                    && matches!(input.get("tier_id"),
                        Some(ScenarioValue::Instance { instance: name }) if name == instance))),
        "an earlier Issue was sent that tier: {before:#?}"
    );
    let statuses = run(&suite, &Interpreted::for_model(model(IDENTITY)));
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
    assert_eq!(
        statuses.get("demo.seats.Check/outcome/qualified"),
        Some(&Status::Passed)
    );
}

// ---- adversary pass 1 ---------------------------------------------------------------------------

/// `text` with `part`, which it holds exactly once, replaced by `with`.
fn adversary_replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

/// Every refusal of `text`'s synthesis that names `name`.
fn adversary_refusals_naming(text: &str, name: &str) -> (ConformanceSuite, Vec<String>) {
    let synthesis = synthesize(&model(text));
    let named = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(name))
        .collect();
    (synthesis.suite, named)
}

const ADVERSARY_SYSTEM: &str = "format: ess/25
system: demo
version: v1
domains:
  - demo.a
components:
  - component: svc
    owns:
      domains: [demo.a]
    accepts:
      commands: [demo.a.DefineTier, demo.a.OpenSeat, demo.a.Requalify, demo.a.Check]
bindings:
  - id: check-on-requalified
    when:
      event: demo.a.Requalified
    invoke:
      command: demo.a.Check
    mapping:
      holder_id: event.holder_id
    delivery: at_least_once
    on_failure: retry
";

/// Seats open not qualifying; only `Requalify`, an update, copies a tier's `qualifies` onto one.
/// A binding on `Requalified` invokes `Check`, whose first branch is a set guard over that field.
const ADVERSARY_DOMAIN: &str = "domain: demo.a
types:
  - {name: demo.a.SeatId, kind: newtype, of: Uuid}
  - {name: demo.a.HolderId, kind: newtype, of: Uuid}
  - {name: demo.a.TierId, kind: newtype, of: Uuid}
entities:
  - name: demo.a.Tier
    identity: {name: tier_id, type: demo.a.TierId}
    fields:
      - {name: qualifies, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.a.Seat
    identity: {name: seat_id, type: demo.a.SeatId}
    fields:
      - {name: holder_id, type: demo.a.HolderId}
      - {name: qualifies, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
commands:
  - name: demo.a.DefineTier
    input:
      - {name: qualifies, type: Boolean}
    outcomes:
      - name: defined
        creates: demo.a.Tier
        instance: tier_id
        sets: {qualifies: input.qualifies}
        emits: [demo.a.TierDefined]
        payload:
          demo.a.TierDefined: {tier_id: {generated: true}}
  - name: demo.a.OpenSeat
    input:
      - {name: holder_id, type: demo.a.HolderId}
    outcomes:
      - name: opened
        creates: demo.a.Seat
        instance: seat_id
        sets: {holder_id: input.holder_id, qualifies: false}
        emits: [demo.a.SeatOpened]
        payload:
          demo.a.SeatOpened: {seat_id: {generated: true}}
  - name: demo.a.Requalify
    input:
      - {name: seat_id, type: demo.a.SeatId}
      - {name: tier_id, type: demo.a.TierId}
      - {name: holder_id, type: demo.a.HolderId}
    outcomes:
      - name: requalified
        updates: demo.a.Seat
        instance: seat_id
        sets: {qualifies: {related: {via: input.tier_id, field: qualifies}}}
        emits: [demo.a.Requalified]
        payload:
          demo.a.Requalified: {seat_id: input.seat_id, holder_id: input.holder_id}
  - name: demo.a.Check
    input:
      - {name: holder_id, type: demo.a.HolderId}
    outcomes:
      - name: unavailable
        external: the store cannot be written to
        error: demo.a.Unavailable
      - name: quiet
        when_related: {entity: demo.a.Seat, where: qualifies == true, exists: false}
        emits: [demo.a.Quiet]
        payload:
          demo.a.Quiet: {holder_id: input.holder_id}
      - name: noticed
        emits: [demo.a.Noticed]
        payload:
          demo.a.Noticed: {holder_id: input.holder_id}
errors:
  - name: demo.a.Unavailable
events:
  - name: demo.a.TierDefined
    fields:
      - {name: tier_id, type: demo.a.TierId}
  - name: demo.a.SeatOpened
    fields:
      - {name: seat_id, type: demo.a.SeatId}
  - name: demo.a.Requalified
    fields:
      - {name: seat_id, type: demo.a.SeatId}
      - {name: holder_id, type: demo.a.HolderId}
  - name: demo.a.Quiet
    fields:
      - {name: holder_id, type: demo.a.HolderId}
  - name: demo.a.Noticed
    fields:
      - {name: holder_id, type: demo.a.HolderId}
views:
  - name: demo.a.Seats
    source: demo.a.Seat
    consistency: read_your_writes
    fields:
      - {name: seat_id, type: demo.a.SeatId}
      - {name: holder_id, type: demo.a.HolderId}
      - {name: qualifies, type: Boolean}
";

fn adversary_bound_model() -> EssIr {
    let mut sources = Vec::new();
    for (label, text) in [
        ("system.yaml", ADVERSARY_SYSTEM),
        ("a.yaml", ADVERSARY_DOMAIN),
    ] {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.push((Source::new(label), raw));
    }
    let spec = Specification::assemble(sources).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// A copy written by an update that a later set guard reads: the binding `Requalified` sets off
/// decides `Check`'s branch over the rows the trigger leaves, the requalified seat among them, so
/// the walk over the steps has to carry the value the update copied (`before.extend(read)` in the
/// update arm of `apply`, `row_set.rs`). Without it the seat's `qualifies` is unknown, no branch
/// is decided, and the binding is refused.
#[test]
fn adversary_a_set_guard_after_an_update_copy_is_decided() {
    let ir = adversary_bound_model();
    let synthesis = synthesize(&ir);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("check-on-requalified"))
        .collect();
    assert_eq!(refused, Vec::<String>::new());
    let flow = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "check-on-requalified/binding/flow")
        .map_or_else(
            || {
                panic!(
                    "the binding's flow is synthesized: {:#?}",
                    synthesis.suite.scenarios.keys().collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        );
    assert!(
        flow.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.a.Requalify")),
        "the trigger is sent: {:#?}",
        flow.steps
    );
    let statuses = run(&synthesis.suite, &Interpreted::for_model(ir.clone()));
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
}

/// [`IDENTITY`] with `Issue` refusing a missing tier alone, the guard a creator copying a related
/// identity most often declares: the row-set guard is still witnessed.
#[test]
fn adversary_split_an_identity_copy_refusing_a_missing_tier_is_witnessed() {
    let text = adversary_replaced(
        IDENTITY,
        "\nevents:\n",
        "\nerrors:\n  - {name: demo.seats.NoTier, summary: No tier carries that identity., fields: []}\nevents:\n",
    );
    let text = adversary_replaced(
        &text,
        "      - {name: tier_id, type: demo.seats.TierId}\n    outcomes:\n      - name: issued\n",
        "      - {name: tier_id, type: demo.seats.TierId}\n    outcomes:\n      - name: no-tier\n        when_related: {via: input.tier_id, exists: false}\n        error: demo.seats.NoTier\n      - name: issued\n",
    );
    let (suite, refusals) = adversary_refusals_naming(&text, "demo.seats.Check");
    assert_eq!(refusals, Vec::<String>::new());
    let statuses = run(&suite, &Interpreted::for_model(model(&text)));
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
}
