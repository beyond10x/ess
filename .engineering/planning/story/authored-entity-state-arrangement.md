---
format: aep.planning-md/1
id: story:authored-entity-state-arrangement
kind: story
status: active
title: Establish typed upstream entity state for non-vacuous authored view tests
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/tests/support/coverage_producer.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/authored.rs
- confidence: inferred
  path: docs/design/authored-entity-state-arrangement.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 5
---
## Outcome and priority

Immediate read-correctness priority for gap 5: an authored scenario can establish real upstream-owned entity data through an explicit target setup capability, then assert populated view results. This addresses setup, not a supposed absence of positive read assertions.

## Established boundaries and witness

Arrangement currently carries only instance/entity and emits no setup operation (ess-conformance/src/authored.rs:278,1619). The current source also rejects an empty timeline (:1645). Positive Contains/count assertions already fail empty results (authored.rs:2047-2057; runner.rs:1796,2057), correcting the archived claim that an always-empty implementation necessarily passes. ConformanceTarget and suite Step lack a setup seam (target.rs:87; scenario.rs:1657); generated Go needs the same capability. Existing typed input binding/Completeness in input.rs:92 is reusable shape authority, not complete entity invariant proof.

The concrete first witness is the consumer's backend-sourced CallRecord and descending-time CallHistory view. No local command creates the records. Seed two distinct records, query membership and ranking, and prove an always-empty target fails. The broader five-entity/nineteen-view count remains source-task evidence until independently audited.

## Acceptance

This story is complete when the previously unarrangeable no-creator CallRecord/CallHistory case executes with two typed rows established in the real target and verifies their values, membership and ordering, while empty, wrong and cross-scenario state fail and all setup/version checks below pass.

Required verification:

- A reviewed authored setup form specifies actual entity identity, required typed fields and declared lifecycle state. Reject duplicate instance/identity, unknown/missing/wrong-typed fields, undeclared state and invariant violations; distinguish Optional absence/null. Do not fabricate lifecycle history.
- A versioned suite operation establishes data in the real Rust/Go target within scenario isolation. Unsupported setup yields Unsupported, never pass. No fake command/event or runner-only cache may impersonate the system under test.
- Subsequent instance references and queries use the established identity/data, with explicit consistency/freshness rules and cleanup/isolation boundaries.
- Valid arrangement plus view assertions works without an unrelated dummy timeline command; a scenario that sets up and asserts nothing still refuses.
- Both working and adversarial targets prove membership, values and ordering. Empty/wrong data fails, unsupported setup does not pass, and scenarios cannot borrow another scenario's rows. Existing command-created setup and positive assertion semantics stay intact.
- Decide source/suite versions, old-reader refusals, deterministic bytes/provenance and native parity before adding fields/steps. Adopt consumer cases only with an adapter that truly establishes the backend read state.

Automatic seed generation for all synthesized scenarios is not required by this authored setup unit. No hard dependency on subject guards or enum coverage; order overlapping conformance runtime/schema edits at integration.

## Historical source

Archived argument: story:an-authored-scenario-can-arrange-a-view-row. Preserved original snapshot and the corrections above define the new unit.


## Provenance and delivery

Decomposes task:ess-gaps-measured-in-a-consumer-specification under initiative:ess-evolution. Prioritized by the operator on 2026-09-11. Source-only scoping at ESS dcdc3343 and observed consumer commit 2497faf27959b59b6eb0700829bb321f851adf1c is retained at local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/state-views.md. Original archived argument bytes/hashes remain under gap-scoping/source. Counts and historical test results are inherited evidence, not newly executed checks. Archived source entries stay terminal; this distinct unit records the maintainer's assessed implementation contract. Overlapping accessor source edits must be integrated or explicitly isolated before dispatch; that collision does not require completion of its downstream adoption acceptance. No full local/ownership gate or unchanged test reruns. Consumer-only PR pipeline failures are accepted; core feature evidence remains required.

## Scope

- Cited: crates/verify/ess-conformance/src/authored.rs.
- Cited: crates/verify/ess-conformance/tests/authored.rs.
- Cited: crates/verify/ess-conformance/src/input.rs.
- Cited: crates/verify/ess-conformance/src/scenario.rs.
- Cited: crates/verify/ess-conformance/src/target.rs.
- Cited: crates/verify/ess-conformance/src/runner.rs.
- Cited: crates/verify/ess-conformance/src/go/runtime.go.
- Cited: crates/verify/ess-conformance/src/go/mod.rs.
- Inferred: docs/design/authored-entity-state-arrangement.md.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 d0672c589cc9f94f2c591c60b14a1732d28403dfc550f44005d4c561ab207c64, retained as local-evidence:runtime-gaps/publication-replay/snapshots/d0672c589cc9f94f2c591c60b14a1732d28403dfc550f44005d4c561ab207c64.md. Source creation recorded at 2026-09-11T00:26:27Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
