//! `unknown_instance:` beside a `when_related:` guard (source format `ess/27`, downstream report,
//! 2026-10-18).
//!
//! `tests/fixtures/related-guard-unknown-instance.yaml`: `ChangeTier` addresses a seat by the
//! identity its input carries and reads the tier its input names. The one precedence rule
//! (`docs/design/generic-language-features.md` § G4) answers an unknown seat first (step 2), then
//! an unknown tier and then a closed tier (step 4), then the change. The `unknown-seat` scenario
//! sends an identity no seat carries beside a tier that is present and closed, so a target reading
//! the tier before the seat's existence answers `tier-closed` and fails it;
//! `ess-synth`'s `related_guard_unknown_instance_served.rs` holds a generated target edited to read
//! the tier first to that verdict.
//!
//! `tests/fixtures/create-or-update-beside-related-refusal.yaml` (beyond10x/ess#641): `Bind`
//! creates a seat no row carries (`bound`, `unknown_instance:`), keeps one bound to the agent sent
//! (`already-bound`), refuses an agent no row carries (`no-agent`, `exists: false` over the input)
//! and otherwise rebinds (`rebound`). Under the one rule the unknown seat answers at step 2 and the
//! missing agent at step 4, so `bound` is witnessed for an unknown seat and an agent no row
//! carries, which only a target reading the seat's existence first answers `bound`; `no-agent` is
//! witnessed on a seat `bound` stored, naming another agent no row carries.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::ir::PrecedencePlan;
use ess_compiler::resolve::{compile, diagnose_locating};
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
    ScenarioValue,
};
use ess_domain::command::precedence::Phase;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-guard-unknown-instance.yaml");
const UNKNOWN_SEAT: &str = "demo.seats.ChangeTier/outcome/unknown-seat";

const BIND_MODEL: &str = include_str!("fixtures/create-or-update-beside-related-refusal.yaml");
const BIND: &str = "demo.desk.Bind";
const ADD_AGENT: &str = "demo.desk.AddAgent";

fn spec(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = spec(text).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn model() -> EssIr {
    ir(MODEL)
}

fn suite_of(model: &EssIr) -> ConformanceSuite {
    let synthesis = synthesize(model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn suite() -> ConformanceSuite {
    suite_of(&model())
}

fn run_on(suite: &ConformanceSuite, target: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(target))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

fn run(suite: &ConformanceSuite) -> BTreeMap<String, Status> {
    run_on(suite, model())
}

#[test]
fn every_branch_beside_unknown_instance_is_synthesized() {
    let ids: Vec<String> = suite().scenarios.keys().map(ToString::to_string).collect();
    for required in [
        UNKNOWN_SEAT,
        "demo.seats.ChangeTier/outcome/unknown-tier",
        "demo.seats.ChangeTier/outcome/tier-closed",
        "demo.seats.ChangeTier/outcome/changed",
    ] {
        assert!(ids.iter().any(|id| id == required), "{required}: {ids:#?}");
    }
}

#[test]
fn the_unknown_identity_is_sent_beside_a_present_closed_tier_and_a_missing_one() {
    let suite = suite();
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == UNKNOWN_SEAT)
        .map(|(_, scenario)| scenario)
        .expect("synthesized");
    let mut closed = None;
    let mut sends = Vec::new();
    for step in &scenario.steps {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        match command.to_string().as_str() {
            "demo.seats.DefineTier" => {
                closed = Some(input.get("open").cloned());
            }
            "demo.seats.ChangeTier" => sends.push(input.clone()),
            _ => {}
        }
    }
    assert_eq!(
        closed,
        Some(Some(ScenarioValue::Literal {
            value: Node::Bool(false)
        })),
        "the tier is defined closed: {:#?}",
        scenario.steps
    );
    let [sent, missing] = sends.as_slice() else {
        panic!("the command is sent twice: {sends:#?}");
    };
    assert!(
        matches!(sent.get("tier_id"), Some(ScenarioValue::Instance { .. })),
        "the tier the scenario defined is named: {sent:?}"
    );
    assert!(
        matches!(sent.get("seat_id"), Some(ScenarioValue::Literal { .. })),
        "the seat is an identity no row carries: {sent:?}"
    );
    // Under the one rule a missing tier answers after the seat's existence: the unknown seat is
    // sent once more naming a tier no row carries.
    assert_eq!(missing.get("seat_id"), sent.get("seat_id"), "{missing:?}");
    assert!(
        matches!(missing.get("tier_id"), Some(ScenarioValue::Literal { .. })),
        "the tier is an identity no row carries: {missing:?}"
    );
}

#[test]
fn the_interpreter_passes_every_scenario() {
    let statuses = run(&suite());
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
}

/// The order every consumer reads under the one rule: the unknown seat at step 2, the unknown tier
/// and then the closed tier at step 4 — before the change, the default.
#[test]
fn the_precedence_plan_answers_the_unknown_seat_before_the_closed_tier() {
    let ir = model();
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.seats.ChangeTier")
        .expect("declared");
    let plan = PrecedencePlan::new(command, ir.format());
    let phase_of = |name: &str| {
        plan.phases()
            .iter()
            .find(|planned| {
                planned
                    .branches
                    .iter()
                    .any(|branch| branch.name.as_str() == name)
            })
            .map(|planned| planned.phase)
    };
    assert_eq!(phase_of("unknown-tier"), Some(Phase::PresentRelated));
    assert_eq!(phase_of("unknown-seat"), Some(Phase::Existence));
    assert_eq!(phase_of("tier-closed"), Some(Phase::PresentRelated));
    assert_eq!(phase_of("changed"), Some(Phase::Default));
}

// ---- create-or-update beside a related-row refusal (beyond10x/ess#641) ----------------------

/// One send of a scenario: the command, its input, the branch the step after it expects, and the
/// instances captured from what it published.
struct Send {
    command: String,
    input: BTreeMap<String, ScenarioValue>,
    expects: Option<String>,
    captured: Vec<ScenarioValue>,
}

fn sends(suite: &ConformanceSuite, id: &str) -> Vec<Send> {
    let Some((_, scenario)) = suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
    else {
        panic!("{id} is synthesized");
    };
    let mut sends: Vec<Send> = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => sends.push(Send {
                command: command.to_string(),
                input: input.clone(),
                expects: None,
                captured: Vec::new(),
            }),
            ScenarioStep::ExpectOutcome { outcome } => {
                if let Some(last) = sends.last_mut() {
                    last.expects
                        .get_or_insert(outcome.outcome.as_str().to_owned());
                }
            }
            ScenarioStep::CaptureInstance { instance, .. } => {
                if let Some(last) = sends.last_mut() {
                    last.captured.push(ScenarioValue::Instance {
                        instance: instance.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    sends
}

/// The agent identities the `AddAgent` sends among `sends` create.
fn agents_created(sends: &[Send]) -> Vec<&ScenarioValue> {
    sends
        .iter()
        .filter(|send| send.command == ADD_AGENT)
        .filter_map(|send| send.input.get("agent_id"))
        .collect()
}

fn bind_scenario(branch: &str) -> String {
    format!("{BIND}/outcome/{branch}")
}

#[test]
fn a_create_or_update_beside_a_related_refusal_synthesizes_every_branch() {
    let ids: Vec<String> = suite_of(&ir(BIND_MODEL))
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for branch in ["bound", "no-agent", "rebound", "already-bound"] {
        let required = bind_scenario(branch);
        assert!(ids.contains(&required), "{required}: {ids:#?}");
    }
}

/// Under the one rule a seat no row carries answers `bound` at step 2, before the agent's
/// existence is read at step 4: the creator is sent naming an agent no row carries, which a target
/// reading the agent first refuses.
#[test]
fn the_creator_is_sent_with_an_agent_no_row_carries() {
    let suite = suite_of(&ir(BIND_MODEL));
    let sends = sends(&suite, &bind_scenario("bound"));
    let (last, before) = sends.split_last().expect("the scenario sends");
    assert_eq!(last.command, BIND, "the scenario ends on the creator");
    assert_eq!(last.expects.as_deref(), Some("bound"));
    let seat = last.input.get("seat_id").expect("a seat is sent");
    assert!(
        matches!(seat, ScenarioValue::Literal { .. }),
        "the seat is an identity no row carries: {seat:?}"
    );
    assert!(
        before
            .iter()
            .filter(|send| send.command == BIND)
            .all(|send| send.input.get("seat_id") != Some(seat)),
        "no earlier send stores the seat"
    );
    let agent = last.input.get("agent_id").expect("an agent is sent");
    assert!(
        matches!(agent, ScenarioValue::Literal { .. }),
        "the agent is an identity, not an arranged row: {agent:?}"
    );
    assert!(
        !agents_created(before).contains(&agent),
        "no step before the creator creates the agent {agent:?}"
    );
}

/// `no-agent` is sent on a seat `bound` stored, naming an agent no row carries and other than the
/// stored one, so the held state (`already-bound`) does not take it and step 4 answers.
#[test]
fn the_related_refusal_is_witnessed_on_a_seat_the_creator_stored() {
    let suite = suite_of(&ir(BIND_MODEL));
    let sends = sends(&suite, &bind_scenario("no-agent"));
    let (last, before) = sends.split_last().expect("the scenario sends");
    assert_eq!(last.command, BIND, "the scenario ends on the refusal");
    assert_eq!(last.expects.as_deref(), Some("no-agent"));
    let stored = before
        .iter()
        .rev()
        .find(|send| send.command == BIND && send.expects.as_deref() == Some("bound"))
        .expect("the seat is stored through the creator first");
    let seat = last.input.get("seat_id").expect("a seat is sent");
    assert!(
        stored.input.get("seat_id") == Some(seat) || stored.captured.contains(seat),
        "the refusal is sent for the seat the creator stored: {seat:?}, stored {:?} captured as \
         {:?}",
        stored.input.get("seat_id"),
        stored.captured
    );
    let agent = last.input.get("agent_id").expect("an agent is sent");
    assert_ne!(
        Some(agent),
        stored.input.get("agent_id"),
        "the agent differs from the stored one"
    );
    assert!(
        !agents_created(before).contains(&agent),
        "no step creates the agent {agent:?}"
    );
}

#[test]
fn every_create_or_update_branch_passes_on_the_interpreted_target() {
    let model = ir(BIND_MODEL);
    let suite = suite_of(&model);
    let statuses = run_on(&suite, model.clone());
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
    for branch in ["bound", "no-agent", "rebound", "already-bound"] {
        assert_eq!(
            statuses.get(&bind_scenario(branch)),
            Some(&Status::Passed),
            "{branch}: {statuses:#?}"
        );
    }

    // A target reading the agent's existence before the seat's is the generated one edited so
    // (`ess-synth`'s `related_guard_unknown_instance_served.rs`): the interpreter answers the
    // addressed row's existence before a present-related refusal whatever order
    // `with_phase_order` sets (beyond10x/ess#282), as synthesis under that exchange does.

    // A target skipping the related check rebinds where it should refuse.
    let skipping = BIND_MODEL.replace(
        "      - name: no-agent\n        when_related: {via: input.agent_id, exists: false}\n        error: demo.desk.NoAgent\n",
        "",
    );
    assert_ne!(skipping, BIND_MODEL, "the fixture moved");
    let skipped = run_on(&suite, ir(&skipping));
    assert_ne!(
        skipped.get(&bind_scenario("no-agent")),
        Some(&Status::Passed),
        "a target without the related check fails `no-agent`: {skipped:#?}"
    );
}

#[test]
fn the_precedence_plan_answers_the_unknown_seat_before_the_missing_agent() {
    let ir = ir(BIND_MODEL);
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == BIND)
        .expect("declared");
    let plan = PrecedencePlan::new(command, ir.format());
    let phase_of = |name: &str| {
        plan.phases()
            .iter()
            .find(|planned| {
                planned
                    .branches
                    .iter()
                    .any(|branch| branch.name.as_str() == name)
            })
            .map(|planned| planned.phase)
    };
    assert_eq!(phase_of("bound"), Some(Phase::Existence));
    assert_eq!(phase_of("already-bound"), Some(Phase::HeldState));
    assert_eq!(phase_of("no-agent"), Some(Phase::PresentRelated));
}

/// Below `ess/27` an identity guard beside a related-row guard does not validate (ESS-COMMAND-004).
#[test]
fn below_ess27_a_create_or_update_beside_a_related_refusal_does_not_validate() {
    let text = BIND_MODEL.replace("format: ess/31", "format: ess/26");
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let errors = Specification::assemble([(Source::new("model.yaml"), raw)])
        .expect_err("ess/26 refuses the composition");
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text);
    let diagnostics = diagnose_locating(&errors, &sources, &["model.yaml"]);
    let unordered: Vec<_> = diagnostics
        .as_slice()
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .message
                .contains("on an `unknown_instance` branch; which of the two answers first")
        })
        .collect();
    assert!(!unordered.is_empty(), "{diagnostics:#?}");
    for diagnostic in unordered {
        assert_eq!(
            diagnostic.code.to_string(),
            "ESS-COMMAND-004",
            "{diagnostic:#?}"
        );
    }
    spec(&BIND_MODEL.replace("format: ess/31", "format: ess/27")).expect("ess/27 admits it");
}
