//! Adversary, wave 2026-10-07c unit c3 (beyond10x/ess#474): the arranging chain now passed into
//! `beside()` narrows an arrangement that terminated before the fix.
//!
//! A zone is sealed only through a gate; a gate is installed reading a project and a permit (two
//! rows, so the permit is arranged beside the project); a permit is issued only for a zone whose
//! priority is not `Low`. Archiving needs a sealed zone, so the zone's arrangement drives
//! `SealZone`, which arranges a gate, which arranges a permit beside, which needs a zone.
//!
//! Before the fix the permit beside was searched from an empty chain: its zone was a fresh row
//! steered to `priority: High`, and the cycle ended there (no zone row needs a gate to be *open*).
//! With the chain passed in, `Zone` is already being arranged, so the permit's zone is arranged
//! one level deep, in the plain witness, and refused where that plain zone is `Low`.
//!
//! The row beside is searched from an empty chain again, and only a search that re-enters itself
//! is cut, so the witness found before the fix is found again. Whether every synthesized scenario
//! of this model then passes is held by `steered_rows_keep_distinct_identities.rs`.
//!
//! In a binary of its own because an overflow aborts every test beside it.

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::synthesize::Synthesis;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = r"format: ess/22
system: site
version: v1
domain: site.ops
types:
  - {name: site.ops.ProjectId, kind: newtype, of: String}
  - {name: site.ops.ZoneId, kind: newtype, of: String}
  - {name: site.ops.PermitId, kind: newtype, of: String}
  - {name: site.ops.GateId, kind: newtype, of: String}
  - name: site.ops.Priority
    kind: enum
    variants: [Low, High]
entities:
  - name: site.ops.Project
    identity: {name: project_id, type: site.ops.ProjectId}
    fields: [{name: title, type: String}]
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: site.ops.Zone
    identity: {name: zone_id, type: site.ops.ZoneId}
    fields: [{name: priority, type: site.ops.Priority}]
    lifecycle:
      initial: Open
      states: [Open, Sealed, Archived]
      terminal: [Archived]
      transitions:
        - {name: seal, from: [Open], to: Sealed}
        - {name: archive, from: [Sealed], to: Archived}
  - name: site.ops.Permit
    identity: {name: permit_id, type: site.ops.PermitId}
    fields: [{name: zone, type: site.ops.ZoneId}]
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
  - name: site.ops.Gate
    identity: {name: gate_id, type: site.ops.GateId}
    fields:
      - {name: project_id, type: site.ops.ProjectId}
      - {name: permit, type: site.ops.PermitId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - name: site.ops.Operator
    may:
      - site.ops.OpenProject
      - site.ops.OpenZone
      - site.ops.IssuePermit
      - site.ops.InstallGate
      - site.ops.SealZone
      - site.ops.ArchiveZone
errors:
  - {name: site.ops.NotFound, fields: []}
  - {name: site.ops.ZoneTooLow, fields: []}
commands:
  - name: site.ops.OpenProject
    input: [{name: project_id, type: site.ops.ProjectId}, {name: title, type: String}]
    outcomes:
      - name: opened
        creates: site.ops.Project
        instance: project_id
        sets: {title: input.title}
        emits: [site.ops.ProjectOpened]
        payload: {site.ops.ProjectOpened: {project_id: input.project_id}}
  - name: site.ops.OpenZone
    input: [{name: zone_id, type: site.ops.ZoneId}, {name: priority, type: site.ops.Priority}]
    outcomes:
      - name: opened
        creates: site.ops.Zone
        instance: zone_id
        sets: {priority: input.priority}
        emits: [site.ops.ZoneOpened]
        payload: {site.ops.ZoneOpened: {zone_id: input.zone_id}}
  - name: site.ops.IssuePermit
    input: [{name: permit_id, type: site.ops.PermitId}, {name: zone, type: site.ops.ZoneId}]
    outcomes:
      - name: no-such-zone
        when_related: {via: input.zone, exists: false}
        error: site.ops.NotFound
      - name: low-zone
        when_related: {via: input.zone, predicate: priority == Low}
        error: site.ops.ZoneTooLow
      - name: issued
        creates: site.ops.Permit
        instance: permit_id
        sets: {zone: input.zone}
        emits: [site.ops.PermitIssued]
        payload: {site.ops.PermitIssued: {permit_id: input.permit_id}}
  - name: site.ops.InstallGate
    input:
      - {name: gate_id, type: site.ops.GateId}
      - {name: project_id, type: site.ops.ProjectId}
      - {name: permit, type: site.ops.PermitId}
    outcomes:
      - name: no-such-project
        when_related: {via: input.project_id, exists: false}
        error: site.ops.NotFound
      - name: no-such-permit
        when_related: {via: input.permit, exists: false}
        error: site.ops.NotFound
      - name: installed
        creates: site.ops.Gate
        instance: gate_id
        sets: {project_id: input.project_id, permit: input.permit}
        emits: [site.ops.GateInstalled]
        payload: {site.ops.GateInstalled: {gate_id: input.gate_id}}
  - name: site.ops.SealZone
    input: [{name: zone_id, type: site.ops.ZoneId}, {name: gate, type: site.ops.GateId}]
    outcomes:
      - name: no-such-gate
        when_related: {via: input.gate, exists: false}
        error: site.ops.NotFound
      - name: sealed
        moves: site.ops.Zone.seal
        instance: zone_id
        emits: [site.ops.ZoneSealed]
        payload: {site.ops.ZoneSealed: {zone_id: input.zone_id}}
  - name: site.ops.ArchiveZone
    input: [{name: zone_id, type: site.ops.ZoneId}]
    outcomes:
      - name: archived
        moves: site.ops.Zone.archive
        instance: zone_id
        emits: [site.ops.ZoneArchived]
        payload: {site.ops.ZoneArchived: {zone_id: input.zone_id}}
events:
  - name: site.ops.ProjectOpened
    fields: [{name: project_id, type: site.ops.ProjectId}]
  - name: site.ops.ZoneOpened
    fields: [{name: zone_id, type: site.ops.ZoneId}]
  - name: site.ops.PermitIssued
    fields: [{name: permit_id, type: site.ops.PermitId}]
  - name: site.ops.GateInstalled
    fields: [{name: gate_id, type: site.ops.GateId}]
  - name: site.ops.ZoneSealed
    fields: [{name: zone_id, type: site.ops.ZoneId}]
  - name: site.ops.ZoneArchived
    fields: [{name: zone_id, type: site.ops.ZoneId}]
";

const ARCHIVED: &str = "site.ops.ArchiveZone/outcome/archived";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ops.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario_ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} [{}]: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(|| format!("{:?}", refusal.subject), ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

#[test]
fn adv_c3_archiving_a_zone_sealed_through_a_gate_beside_a_permit_is_witnessed() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let ids = scenario_ids(&result);
    assert!(
        ids.iter().any(|id| id == ARCHIVED),
        "{ARCHIVED} has a witness (open a High zone for the permit, issue it, install the gate, \
         seal, archive); scenarios: {ids:#?}\nrefusals: {:#?}",
        refusals(&result)
    );
}
