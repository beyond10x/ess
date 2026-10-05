//! Adversary pass 1 on unit E-U1: below `ess/22` the synthesis plan of a model keeps its bytes.
//!
//! The unit gives a `Timestamp` path its own kind in `determined::resolve`, which also changes the
//! obligation reason an older model's guard or view filter comparing a `Timestamp` with a literal
//! receives. The expected reasons were read from base `4e6180546` with this same model.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str = "format: ess/21
system: lease
version: v1
domain: lease.core
entities:
  - name: lease.core.Lease
    identity: {name: lease_id, type: Uuid}
    fields:
      - {name: starts_at, type: Timestamp}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - {name: lease.core.Late, fields: []}
  - {name: lease.core.Opened, fields: [{name: lease_id, type: Uuid}]}
commands:
  - name: lease.core.Open
    input:
      - {name: starts_at, type: Timestamp}
    outcomes:
      - name: late
        when: starts_at <= now
        emits: [lease.core.Late]
      - name: opened
        creates: lease.core.Lease
        instance: lease_id
        sets: {starts_at: input.starts_at}
        emits: [lease.core.Opened]
        payload: {lease.core.Opened: {lease_id: {generated: true}}}
views:
  - name: lease.core.Recent
    source: lease.core.Lease
    consistency: read_your_writes
    filter: 'starts_at >= \"2024-01-01T00:00:00Z\"'
    fields:
      - {name: lease_id, type: Uuid}
      - {name: starts_at, type: Timestamp}
components:
  - component: lease-service
    owns: {domains: [lease.core]}
    accepts: {commands: [lease.core.Open]}
    publishes: {events: [lease.core.Late, lease.core.Opened]}
";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("lease.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn reason(kind: CapabilityKind, source: &str) -> String {
    let synthesis = synthesize_for(&ir(), Target::Rust).unwrap_or_else(|_| panic!("synthesizes"));
    match synthesis.plan.disposition_of(kind, source) {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            serde_json::to_string(&obligation.reason).unwrap()
        }
        other => format!("{other:?}"),
    }
}

/// An `ess/21` current-time guard over a `Timestamp` input stays owed with the reason base gave it.
#[test]
fn adversary_ess21_timestamp_guard_obligation_reason_matches_base() {
    assert_eq!(
        reason(CapabilityKind::CommandBehavior, "lease.core.Open"),
        r#"{"class":"undetermined","construct":"a literal of another kind than the value it is compared with, in `late`"}"#
    );
}

/// An `ess/21` view filter comparing a `Timestamp` with an instant literal likewise.
#[test]
fn adversary_ess21_timestamp_view_filter_obligation_reason_matches_base() {
    assert_eq!(
        reason(CapabilityKind::ViewQuery, "lease.core.Recent"),
        r#"{"class":"undetermined","construct":"a literal of another kind than the value it is compared with, in `filter:`"}"#
    );
}
