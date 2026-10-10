//! Adversary pass 1, wave 5 unit U3 (`story:synthesis-reads-selection-plan`): the stored-row
//! searches of `synthesize/subject_fact.rs` under the precedence plan.
//!
//! Each case synthesizes a model and runs it on the model interpreter inside one exchanged phase
//! order (`with_phase_order`). Synthesis and the interpreter read the same
//! plan, so every scenario synthesis writes should pass.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::AdmittedSuite;
use ess_conformance::Runner;
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn ir(text: &str) -> Option<EssIr> {
    let raw = RawSpecFile::parse(text).ok()?;
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)]).ok()?;
    compile(&spec, &texts).ok()
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let why: Vec<String> = result.diagnostics().map(|d| format!("{d:?}")).collect();
            format!(
                "{}: {:?} {}",
                result.scenario,
                result.status,
                why.join(" | ")
            )
        })
        .collect()
}

fn swap(a: Phase, b: Phase) -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| {
        if phase == a {
            b
        } else if phase == b {
            a
        } else {
            phase
        }
    })
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

fn fixture(name: &str) -> EssIr {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    ir(&text).unwrap_or_else(|| panic!("{name} compiles standalone"))
}

/// Synthesizes `model` and runs the suite on the model interpreter, both inside `order`; every
/// scenario not passed whose id starts with `prefix`.
fn failed_under(model: &EssIr, order: [Phase; 8], prefix: &str) -> Vec<String> {
    with_phase_order(order, || {
        let result = ess_conformance::synthesize::synthesize(model);
        not_passed(model.clone(), &result.suite)
    })
    .into_iter()
    .filter(|line| line.starts_with(prefix))
    .collect()
}

/// The implementor's claim (U3 report): with the input refusals and the held state plainly
/// exchanged, `ROTATE`'s `too-short` is sent as a plain invocation naming no arranged row
/// (`synthesize::is_state_input_refusal`: "answered before existence and before the held state"),
/// and the interpreter, reading `Existence` before `InputRefusal`, answers `unknown_instance`'s
/// question first. Synthesis and the interpreter read one exchanged plan; the suite should pass.
#[test]
#[ignore = "test seam only; story:existence-family-sends-read-selection-plan"]
fn input_and_held_exchanged_the_state_input_refusal_witness_passes_the_interpreter() {
    let model = ir(ROTATE).expect("ROTATE compiles");
    let failed = failed_under(
        &model,
        swap(Phase::InputRefusal, Phase::HeldState),
        "demo.secrets.RotateSecret/outcome/too-short",
    );
    assert_eq!(failed, Vec::<String>::new());
}

/// The same family on a repository fixture: `now-stored-rows.yaml` `RenewLease/blank-note`
/// (`when: note == ""` beside two `when_subject:` predicates), sent naming a lease nobody arranged.
#[test]
#[ignore = "test seam only; story:existence-family-sends-read-selection-plan"]
fn input_and_held_exchanged_blank_note_passes_the_interpreter() {
    let model = fixture("now-stored-rows.yaml");
    let failed = failed_under(
        &model,
        swap(Phase::InputRefusal, Phase::HeldState),
        "demo.leases.RenewLease/outcome/blank-note",
    );
    assert_eq!(failed, Vec::<String>::new());
}

/// The creation family, which `subject_fact` does not route (`Bind` reads no stored field):
/// `explore-stored-rows.yaml` `Bind/too-short` is sent with a key already bound, assuming the input
/// refusal answers before `existing_instance:`. With `InputRefusal` read after `Existence` the
/// interpreter answers `already-bound`.
#[test]
#[ignore = "test seam only; story:existence-family-sends-read-selection-plan"]
fn input_and_held_exchanged_bind_too_short_passes_the_interpreter() {
    let model = fixture("explore-stored-rows.yaml");
    let failed = failed_under(
        &model,
        swap(Phase::InputRefusal, Phase::HeldState),
        "exploredraw.keys.Bind/outcome/too-short",
    );
    assert_eq!(failed, Vec::<String>::new());
}
