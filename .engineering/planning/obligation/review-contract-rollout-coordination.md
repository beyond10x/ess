---
format: aep.planning-md/1
id: obligation:review-contract-rollout-coordination
kind: obligation
status: open
title: Coordinate relying readers before changed ESS formats become defaults
relations:
- decomposes: epic:review-boundary-remediation
revision: 2
---
## Outstanding outcome

Any remediation that changes persisted meaning or bytes verified by another repository needs the Atlas ADR and relying-party rollout order required by `AGENTS.md:47` and the organization authority. Atlas `ROADMAP.md:45` at `a8fb936ddcb35c8971311610e5c63cc86d612fab` records the ESS-to-AEP evidence seam.

## Discharge condition

Recorded compatibility and rollout evidence shows that every affected relying reader supports the selected contract before its default writer is enabled.

## Procedure and required evidence

For each selected migration, inventory exact reader/writer revisions, create the Atlas ADR and governed work in each affected repository, prove old/new compatibility using actual adapters, then enable the default writer only in the approved rollout order. Record the checked commits and release evidence. If a remediation preserves bytes and meaning, document that result explicitly instead of manufacturing a migration.

For each selected migration, inventory exact reader/writer revisions, create the Atlas ADR and governed work in each affected repository, prove old/new compatibility using actual adapters, then enable the default writer only in the approved rollout order. Record the checked commits and release evidence. If a remediation preserves bytes and meaning, document that result explicitly instead of manufacturing a migration.

## Known scope

F03 suite/report coverage and count semantics, F06 observation completeness and F08 exact numbers are migration candidates; F04 persisted import accounting and F11 evidence-kind meaning may also require coordination. This list is provisional until each design inventories consumers. The scoper's local AEP inspection is a lead, not proof of its current remote revision.

## Scheduling limit

This obligation is not a blocker on byte-preserving P0 fixes or drafting designs. It remains open through the affected release readiness checks and cannot be replaced by an ESS-only green test run.