---
format: aep.planning-md/1
id: test-plan:ess-evolution
kind: test-plan
status: draft
title: ESS preservation and independent application acceptance
relations:
- verifies: initiative:ess-evolution
revision: 1
---
## Required evidence
See docs/design/ess-evolution/acceptance.md. Keep the current full ESS gate, source coverage/profile authority and every existing CLI behavior test. Counts are observations, not semantic acceptance. Preserve explicit unknown/refused coverage.

## Storage and execution
Shared file/SQLite/PostgreSQL tests cover atomic groups, request ordering, same-stream transaction-local expectations, group retry/content conflict, guards/projection rollback, concurrent writers, crash/reopen, blobs, snapshot invalidation and divergence refusal. ER tests cover complete records, zero-event decisions, non-state-changing observations, replay, record-ID conflict, multi-entity atomicity and tampering refusal.

## Applications
AEP migrations and receipt-bearing projection failures; generated billing/gatepass boot/HTTP/restart/auth/query/effects; Connectors locked/MSRV and local runtime/keyring/protected-input/revocation/uncertainty scenarios; Babelconnect descriptor/wire/cache/downstream/auth/reconnect/media compatibility; Linux/web/Android Flutter execution, both themes, existing Playwright and independent generated journeys with the real Go server.

## Independence
Expected behavior comes from specification scenarios independent of generated target plans. Demonstrate sensitivity to wrong fields, dropped patches, incorrect transitions, bypassed revocation and misbound UI actions. Synthetic media cannot prove audio. Desired declarations cannot prove observed infrastructure.

## Status
Required gates have not yet been executed for the new architecture. No missing lane may be counted as passing.

