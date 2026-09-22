---
format: aep.planning-md/1
id: test-plan:ess-evolution
kind: test-plan
status: draft
title: ESS preservation, migration and runtime acceptance
relations:
- verifies: initiative:ess-evolution
revision: 3
---
## Required evidence

Follow docs/design/ess-evolution/acceptance.md and approved revision ess-evolution-20260915/1. Preserve the complete ESS gate, consumer/profile coverage authority, canonical formats and every existing CLI behavior. Counts are observations, not semantic acceptance. Unknown/refused coverage remains explicit.

## Storage and execution

Shared file/SQLite/PostgreSQL acceptance covers ordered atomic groups, transaction-local repeated-stream expectations, exact retry and content conflict, guard/projector rollback, concurrent writers, crash/reopen, blobs, snapshot invalidation and corruption/divergence refusal. ER acceptance covers complete recorded decisions, zero-event decisions, ordered non-state-changing observations, global record-ID conflict, unified history, atomic multi-entity execution, replay and tampering refusal.

## Migration

Test every legacy configuration and the explicit history boundary. Exercise divergent inputs, staged verification and source-fence recheck, interruption before/after cutover, Markdown drift/deletion, projection rebuild and committed mutation with failed projection. Verify all six actual stores in the approved order with retained recovery copies and mutation/restart/query evidence.

## Services and Connectors

Generated billing/gatepass fixtures build, boot, serve HTTP, persist through ER/Eventlog, restart and preserve authorization, query and effect behavior. Connectors executes its locked/MSRV gate and real disposable provider/keyring scenarios covering restart, repair/revoke races, protected input, uncertainty, redaction and exact metadata migration.

## Independence and gates

Expected behavior comes from upstream specifications and independent fixtures. Wrong fields, dropped state/event updates, incorrect transitions and bypassed revocation must fail their corresponding tests. Desired declarations do not prove observed infrastructure; retain independent observations and credential-redaction mutation tests.

Required commands: ESS task check with consumers enabled and task site-build; ER/AEP/Service SDK task check; Eventlog bash scripts/gate.sh; Connectors cargo run --locked -p connectors-build -- gate --msrv. Execute required disposable PostgreSQL lanes and affected minimum-compiler checks. No selected-zero or skipped required lane counts as success.

## Status and boundary

This document records required future acceptance, not a completed runtime migration. Generic protobuf/UI/Flutter acceptance is deferred to task:deferred-protocol-ui-bindings. Local acceptance does not authorize publication, release or deployment.
