//! Steered creations keep distinct identities (beyond10x/ess#480, found by the adversary review
//! of #474).
//!
//! A zone is sealed only through a gate; a gate is installed reading a project and a permit (two
//! rows, so the permit is arranged beside the project); a permit is issued only for a zone whose
//! priority is not `Low`. Archiving needs a sealed zone, so the zone's arrangement drives
//! `SealZone`, which arranges a gate, which arranges a permit beside, which needs a zone.
//!
//! Synthesis witnesses the archive outcome (`adv_c3_beside_chain_regression.rs`), but on 0.55.0
//! and after #474 8 scenarios of this model end in `Error`, "a supplied identity is already held
//! by site.ops.Zone": a steered creation takes its identity from the plain witness, so the row and
//! both decoys carry the same literal. Every synthesized scenario has to pass.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
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

fn statuses(model: &EssIr, result: &Synthesis) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

#[test]
fn every_scenario_of_a_zone_sealed_through_a_gate_beside_a_permit_passes() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let ids = scenario_ids(&result);
    assert!(
        ids.iter().any(|id| id == ARCHIVED),
        "{ARCHIVED} has a witness (open a High zone for the permit, issue it, install the gate, \
         seal, archive); scenarios: {ids:#?}\nrefusals: {:#?}",
        refusals(&result)
    );
    let failing: Vec<_> = statuses(&model, &result)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}
