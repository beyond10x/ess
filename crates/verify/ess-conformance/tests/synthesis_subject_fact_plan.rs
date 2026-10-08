//! The stored-row searches ask the precedence plan which branches answer first
//! (`story:synthesis-reads-selection-plan`, `docs/design/selection-plan.md`).
//!
//! `tests/fixtures/external-beside-held-guard.yaml`: `CheckPick` refuses a pick whose stored
//! `revision` differs from the input's (`stale`, a `when_subject:` guard, phase `HeldState`) and
//! declares the external `unlisted` (phase `Accepting`) after it. Under the precedence order the
//! held state answers first, so `unlisted` is sent the revision its row holds (beyond10x/ess#464).
//! With the two phases exchanged through the plan's one test seam (`with_phase_order`), `unlisted`
//! answers before `stale` and its plain witness is kept: a revision the row does not hold. The model
//! is compiled once, outside the exchange; only synthesis and the model interpreter run inside it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceSuite, InstanceName};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

const UNLISTED: &str = "demo.desk.CheckPick/outcome/unlisted";
const REFUSED_BY_UNLISTED: &str =
    "demo.desk.Pick/transition/refuse/by/demo.desk.CheckPick/unlisted";

/// The precedence order with the held-state and accepting/external phases exchanged.
fn exchanged() -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| match phase {
        Phase::HeldState => Phase::Accepting,
        Phase::Accepting => Phase::HeldState,
        other => other,
    })
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("no scenario {id}; refusals: {:#?}", refusals(result)),
            |(_, scenario)| scenario,
        )
}

/// The input of the first send of `command` after its provider is forced for `outcome`.
fn forced_send<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
    outcome: &str,
) -> &'a BTreeMap<String, ScenarioValue> {
    let at = scenario
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ConfigureExternalOutcome { force, .. }
                if force.command.name().to_string() == command
                    && force.outcome.to_string() == outcome)
        })
        .unwrap_or_else(|| panic!("no forced provider: {scenario:#?}"));
    scenario.steps[at + 1..]
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.name().to_string() == command => Some(input),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the forced provider is not followed by a send: {scenario:#?}"))
}

/// The `field` each instance a send of `command` captured was created with.
fn created_with(
    scenario: &ConformanceScenario,
    command: &str,
    field: &str,
) -> BTreeMap<InstanceName, ScenarioValue> {
    let mut found = BTreeMap::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand {
            command: sent,
            input,
            ..
        } = step
        else {
            continue;
        };
        if sent.name().to_string() != command {
            continue;
        }
        let captured = scenario.steps[at + 1..]
            .iter()
            .take_while(|step| !matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .find_map(|step| match step {
                ScenarioStep::CaptureInstance { instance, .. } => Some(instance.clone()),
                _ => None,
            });
        if let (Some(instance), Some(value)) = (captured, input.get(field)) {
            found.insert(instance, value.clone());
        }
    }
    found
}

/// The `revision` the `Pick` the forced `unlisted` send names was created with, and the one sent.
fn sent_against_row(scenario: &ConformanceScenario) -> (ScenarioValue, ScenarioValue) {
    let sent = forced_send(scenario, "demo.desk.CheckPick", "unlisted");
    let ScenarioValue::Instance { instance } = &sent["pick_id"] else {
        panic!("the pick is an arranged one: {sent:#?}");
    };
    let rows = created_with(scenario, "demo.desk.MakePick", "revision");
    (rows[instance].clone(), sent["revision"].clone())
}

/// Every scenario of `suite` run on the model interpreter of `ir`, by id, where it did not pass.
fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    statuses
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

#[test]
fn exchanging_held_state_and_accepting_moves_the_stored_row_external_witness() {
    let model = ir(MODEL);
    let declared = ess_conformance::synthesize::synthesize(&model);
    let swapped = with_phase_order(exchanged(), || {
        ess_conformance::synthesize::synthesize(&model)
    });
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        let (held, sent) = sent_against_row(scenario(&declared, id));
        assert_eq!(
            held, sent,
            "{id}: under the precedence order `stale` answers first, so `unlisted` is sent the \
             revision its row holds"
        );
        let (held, sent) = sent_against_row(scenario(&swapped, id));
        assert_ne!(
            held, sent,
            "{id}: with the phases exchanged `unlisted` answers before `stale`, so its plain \
             witness is kept, a revision the row does not hold"
        );
    }
    let failed = with_phase_order(exchanged(), || not_passed(model.clone(), &swapped.suite));
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the interpreter, reading the same exchanged order, passes the suite"
    );
}

/// `frozen` (`when_subject: {predicate: state == Picked}`) holds of every row `unlisted` moves
/// from, so under the precedence order `unlisted` is refused naming it (`ESS-SYNTH-003`). With the
/// held state read after the external branch, nothing claims `unlisted` and it is written.
#[test]
fn exchanging_held_state_and_accepting_writes_the_external_the_held_state_claimed() {
    let frozen = MODEL.replace(
        "        when_subject:\n          predicate: revision != input.revision\n",
        "        when_subject:\n          predicate: state == Picked\n",
    );
    assert_ne!(frozen, MODEL);
    let model = ir(&frozen);
    let declared = ess_conformance::synthesize::synthesize(&model);
    let swapped = with_phase_order(exchanged(), || {
        ess_conformance::synthesize::synthesize(&model)
    });
    for id in [UNLISTED, REFUSED_BY_UNLISTED] {
        assert!(
            !ids(&declared).contains(&id.to_owned()),
            "{id}: under the precedence order `stale` claims every row: {:#?}",
            ids(&declared)
        );
        assert!(
            ids(&swapped).contains(&id.to_owned()),
            "{id}: with the phases exchanged nothing claims `unlisted`; refusals: {:#?}",
            refusals(&swapped)
        );
    }
    let failed = with_phase_order(exchanged(), || not_passed(model.clone(), &swapped.suite));
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the interpreter, reading the same exchanged order, passes the suite"
    );
}

const ROTATE: &str = r#"format: ess/16
system: demo
version: v1
domain: demo.secrets
summary: A configuration per tenant, rotated while on the basic tier; a short secret is refused.
types:
  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}
  - {name: demo.secrets.Tier, kind: enum, variants: [Basic, Gold]}
entities:
  - name: demo.secrets.Configuration
    identity: {name: tenant_id, type: demo.secrets.TenantId}
    fields:
      - {name: secret, type: String}
      - {name: tier, type: demo.secrets.Tier}
    lifecycle:
      initial: Configured
      states: [Configured]
      terminal: [Configured]
actors:
  - name: demo.secrets.Operator
    may: [demo.secrets.Register, demo.secrets.RotateSecret]
errors:
  - {name: demo.secrets.SecretTooShort, fields: []}
  - {name: demo.secrets.NotBasic, fields: []}
events:
  - name: demo.secrets.Registered
    fields: [{name: tenant_id, type: demo.secrets.TenantId}]
  - name: demo.secrets.SecretRotated
    fields: [{name: tenant_id, type: demo.secrets.TenantId}]
commands:
  - name: demo.secrets.Register
    input:
      - {name: tier, type: demo.secrets.Tier}
    outcomes:
      - name: registered
        creates: demo.secrets.Configuration
        instance: tenant_id
        sets: {secret: "initial-secret-value", tier: input.tier}
        emits: [demo.secrets.Registered]
        payload: {demo.secrets.Registered: {tenant_id: {generated: true}}}
  - name: demo.secrets.RotateSecret
    input:
      - {name: tenant_id, type: demo.secrets.TenantId}
      - {name: secret, type: String}
    outcomes:
      - name: too-short
        when: secret.count < 12
        error: demo.secrets.SecretTooShort
      - name: rotated
        when_subject: {field: tier, equals: Basic}
        updates: demo.secrets.Configuration
        instance: tenant_id
        sets: {secret: input.secret}
        emits: [demo.secrets.SecretRotated]
        payload: {demo.secrets.SecretRotated: {tenant_id: input.tenant_id}}
      - name: not-basic
        error: demo.secrets.NotBasic
views:
  - name: demo.secrets.Configurations
    source: demo.secrets.Configuration
    consistency: read_your_writes
    fields:
      - {name: tenant_id, type: demo.secrets.TenantId}
      - {name: state, type: demo.secrets.Configuration.State}
      - {name: secret, type: String}
      - {name: tier, type: demo.secrets.Tier}
"#;

const TOO_SHORT: &str = "demo.secrets.RotateSecret/outcome/too-short";

/// The precedence order with the held state read before the input refusals, and existence still
/// after both: `rotated` (`when_subject:`, phase `HeldState`) then answers before `too-short`.
fn held_state_first() -> [Phase; 8] {
    [
        Phase::InputAbsent,
        Phase::RelatedRow,
        Phase::HeldState,
        Phase::InputRefusal,
        Phase::Existence,
        Phase::PresentRelated,
        Phase::Accepting,
        Phase::Default,
    ]
}

/// The `tier` of every row `scenario` registers.
fn registered_tiers(scenario: &ConformanceScenario) -> Vec<ScenarioValue> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.name().to_string() == "demo.secrets.Register" =>
            {
                Some(input["tier"].clone())
            }
            _ => None,
        })
        .collect()
}

fn variant(name: &str) -> ScenarioValue {
    ScenarioValue::literal(ess_primitives::node::Node::Text(name.into()))
}

/// `RotateSecret` refuses a short secret (`too-short`, an input refusal) and rotates a basic-tier
/// row's secret (`rotated`, `when_subject: {field: tier, equals: Basic}`). `too-short`'s scenario
/// also sends its input to an arranged row. Under the precedence order the input refusal answers
/// first, so the stored-row search (`selects`) keeps the first row it arranges, a basic one.
/// With the held state read first, a basic row selects `rotated` before `too-short`, so the search
/// goes on to a gold row, which `rotated` does not claim.
#[test]
fn reading_the_held_state_first_moves_the_refusal_stored_row_witness() {
    let model = ir(ROTATE);
    let declared = ess_conformance::synthesize::synthesize(&model);
    let swapped = with_phase_order(held_state_first(), || {
        ess_conformance::synthesize::synthesize(&model)
    });
    assert_eq!(
        registered_tiers(scenario(&declared, TOO_SHORT)),
        vec![variant("Basic")],
        "under the precedence order `too-short` answers before `rotated` on any row"
    );
    assert_eq!(
        registered_tiers(scenario(&swapped, TOO_SHORT)),
        vec![variant("Gold")],
        "with the held state read first `rotated` claims a basic row, so the row is a gold one"
    );
    assert_eq!(
        not_passed(model.clone(), &declared.suite),
        Vec::<String>::new()
    );
    let failed = with_phase_order(held_state_first(), || {
        not_passed(model.clone(), &swapped.suite)
    });
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the interpreter, reading the same order, passes the suite"
    );
}

/// The #278 shape of `tests/mixed_guard_overlap.rs`: `RecordResult` declares `held-for-promotion`
/// (`when: result == Healthy` beside `when_subject:` over `auto_promote`, phase `HeldState`) before
/// `promoted` (`when: result == Healthy` alone, phase `Accepting`).
const ROLLOUT: &str = r#"format: ess/19
system: demo
version: v1
domain: demo.rollout
summary: A deployment held for promotion unless it promotes itself.
types:
  - {name: demo.rollout.Result, kind: enum, variants: [Healthy, Regression]}
entities:
  - name: demo.rollout.Deployment
    identity: {name: deployment_id, type: Uuid}
    fields:
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
    lifecycle:
      initial: Canary
      states: [Canary, Held, Promoted, RolledBack]
      terminal: [Promoted, RolledBack]
      transitions:
        - {name: hold, from: [Canary], to: Held}
        - {name: promote, from: [Canary], to: Promoted}
        - {name: rollback, from: [Canary], to: RolledBack}
        - {name: approve, from: [Held], to: Promoted}
        - {name: reject, from: [Held], to: RolledBack}
errors:
  - name: demo.rollout.Conflict
    summary: wrong state
    fields:
      - {name: state, type: demo.rollout.Deployment.State}
events:
  - name: demo.rollout.Moved
    fields: [{name: deployment_id, type: Uuid}]
actors:
  - name: demo.rollout.Engine
    may: [demo.rollout.Deploy, demo.rollout.RecordResult, demo.rollout.Decide]
commands:
  - name: demo.rollout.Deploy
    input:
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
    outcomes:
      - name: deployed
        creates: demo.rollout.Deployment
        instance: deployment_id
        sets:
          auto_promote: input.auto_promote
          automatic_rollback: input.automatic_rollback
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: {generated: true}}
  - name: demo.rollout.RecordResult
    input:
      - {name: deployment_id, type: Uuid}
      - {name: result, type: demo.rollout.Result}
    outcomes:
      - name: held-for-promotion
        when: result == Healthy
        when_subject:
          predicate: {any: ["not defined(auto_promote)", auto_promote == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: promoted
        when: result == Healthy
        moves: demo.rollout.Deployment.promote
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: held-for-rollback
        when: result == Regression
        when_subject:
          predicate: {any: ["not defined(automatic_rollback)", automatic_rollback == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: rolled-back
        moves: demo.rollout.Deployment.rollback
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: wrong-state
        wrong_state: true
        error: demo.rollout.Conflict
  - name: demo.rollout.Decide
    input:
      - {name: deployment_id, type: Uuid}
      - {name: decision, type: demo.rollout.Result}
    outcomes:
      - name: promoted
        when: decision == Healthy
        moves: demo.rollout.Deployment.approve
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: rolled-back
        moves: demo.rollout.Deployment.reject
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: wrong-state
        wrong_state: true
        error: demo.rollout.Conflict
views:
  - name: demo.rollout.Deployments
    source: demo.rollout.Deployment
    consistency: read_your_writes
    fields:
      - {name: deployment_id, type: Uuid}
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
      - {name: state, type: demo.rollout.Deployment.State}
"#;

const HELD_FOR_PROMOTION: &str = "demo.rollout.RecordResult/outcome/held-for-promotion";
const HOLD: &str =
    "demo.rollout.Deployment/transition/hold/by/demo.rollout.RecordResult/held-for-promotion";

/// No row and input selects `held-for-promotion` alone: wherever its guard holds, `promoted`'s does
/// too. So its witness is searched for under `Order::FirstDeclared`, where the branch the
/// precedence plan reads first answers. Under the precedence order that is `held-for-promotion`
/// (held state before accepting). With the two phases exchanged, `promoted` answers wherever
/// `held-for-promotion`'s guard holds, so the stored-row search finds no witness and the scenarios
/// are refused for want of one (`ESS-SYNTH-003`). Picking the first declared there wrote a witness
/// `promoted` answers, which synthesis's own check then refused as a defect in ess
/// (`ESS-SYNTH-019`).
#[test]
fn exchanging_held_state_and_accepting_moves_the_first_declared_pick() {
    let model = ir(ROLLOUT);
    let declared = ess_conformance::synthesize::synthesize(&model);
    let swapped = with_phase_order(exchanged(), || {
        ess_conformance::synthesize::synthesize(&model)
    });
    for id in [HELD_FOR_PROMOTION, HOLD] {
        assert!(
            ids(&declared).contains(&id.to_owned()),
            "{id}: under the precedence order `held-for-promotion` answers first; refusals: {:#?}",
            refusals(&declared)
        );
        assert!(
            !ids(&swapped).contains(&id.to_owned()),
            "{id}: with the phases exchanged `promoted` answers first wherever \
             `held-for-promotion`'s guard holds: {:#?}",
            ids(&swapped)
        );
        let named: Vec<String> = refusals(&swapped)
            .into_iter()
            .filter(|text| text.contains(&format!("`{id}`")))
            .collect();
        assert!(
            !named.is_empty()
                && named
                    .iter()
                    .all(|text| text.starts_with("refusal[ESS-SYNTH-003]")),
            "{id}: refused for want of a witness, never written for a step `promoted` answers: \
             {named:#?}"
        );
    }
    let failed = with_phase_order(exchanged(), || not_passed(model.clone(), &swapped.suite));
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the interpreter, reading the same exchanged order, passes the suite"
    );
}
