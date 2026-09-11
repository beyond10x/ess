---
format: aep.planning-md/1
id: story:periodic-binding-trigger-contract
kind: story
status: active
title: Define periodic binding causes with observable timing and input authority
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
- depends_on: story:elapsed-time-claims
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-cli-project/tests/consumer_cli_bindings.rs
- confidence: cited
  path: crates/generate/ess-cli-project/tests/consumer_cli_model_ingress.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/graph.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/accessor.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/refusal.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/bridge.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/catalog.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/page.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web/refusal.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/graph.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/periodic_selection_integration.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/lib.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/time.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/periodic.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/delta.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/graph.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/periodic.rs
- confidence: inferred
  path: docs/design/periodic-binding-triggers.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 5
---
## Outcome and priority

High-priority design followed by implementation for gap 6: describe the measured two-second reconciliation cause honestly, including where its required command inputs come from. Source/IR/suite syntax is not selected by this planning record.

## Established boundaries

RawTrigger and BindingSpec are event-only (ess-domain/src/binding.rs:179-186,646-654); lowering resolves an event (ess-compiler/src/resolve.rs:2540-2560); synthesis requires a publisher (ess-conformance/src/synthesize.rs:3463-3485). Existing mark_instant/observe_elapsed target capabilities measure elapsed windows and watched-event counts (target.rs:173-219,576-605), not timer-bound invocation causation. The runner clock bounds polling and cannot prove a system timer fired. Existing Elapsed admits zero; a period must decide its own valid range.

## Design and acceptance

This story is complete when the measured two-second poll has a validated, projected declaration of its actual cause and input authority, and the reviewed observation contract is implemented with passing controlled-time witnesses for supported execution plus exact refusals for unsupported targets. A design-only record or a refusal on every target cannot close the story; unavailable input authority keeps it open. Remove the consumer UNMAPPED only after its declaration matches the actual code.

Required verification:

- Inspect the actual poll's first-fire anchor, lifetime/cancellation, rate versus completion-relative delay, overlap/missed-period behavior, and required-input authority before writing the binding design.
- Decide whether the period is an executable obligation or descriptive intent; define occurrence identity and delivery/failure scope. At-most-once delivery alone does not establish periodic liveness.
- Required inputs must come from an explicit typed host/context authority if no event supplies them. UNMAPPED: that authority is not yet admitted; no invented event or optionalized input may conceal it.
- Admit a finite positive period with explicit invalid/zero/overflow and event/period exclusivity diagnostics. Descriptions name the cause.
- Synthesis and Rust/Go execution establish the agreed invocation/timing observation, or retain an exact named capability refusal. Use controlled/reported target time; no mandatory wall-clock sleep or scheduler service.
- Distinguish correct, early/late/missing and misattributed occurrences as required by the chosen contract. Preserve legacy event-trigger bytes and use exact previous-reader evidence for any new vocabulary.
- Remove the actual consumer poll's UNMAPPED only once the declared cause and input source match its code.

Cron/calendars, backoff/jitter, production scheduling and clock synchronization are not implied. Reuse implemented elapsed-time claims; it is a prerequisite measurement capability, not completed periodic semantics.

## Historical source

Archived argument: story:a-binding-may-be-triggered-by-a-period. Its original acceptance is evidence to assess, not a legal lifecycle path to reactivate.


## Provenance and delivery

Decomposes task:ess-gaps-measured-in-a-consumer-specification under initiative:ess-evolution. The operator prioritized these gaps on 2026-09-11. Source-only scoping at dcdc3343 is retained in local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/time-size.md; original archived argument bytes and hashes are in gap-scoping/source/manifest.json. Original archived stories remain terminal and untouched; this is an explicitly distinct implementation/design unit, not an illegal unarchive. Planning does not claim implementation or a rerun of consumer measurements. Integrate overlapping CLI/compiler/conformance changes after the active accessor candidate; source overlap does not depend on finishing its separate downstream adoption proof. No full local workspace/ownership gate or unchanged reruns. Consumer-only PR pipeline failures remain accepted, with core feature checks retained.

## Scope

- Cited: crates/specify/ess-domain/src/binding.rs.
- Cited: crates/specify/ess-compiler/src/resolve.rs.
- Cited: crates/specify/ess-compiler/src/ir.rs.
- Cited: crates/verify/ess-conformance/src/synthesize.rs.
- Cited: crates/verify/ess-conformance/src/scenario.rs.
- Cited: crates/verify/ess-conformance/src/target.rs.
- Inferred: crates/generate/ess-gen/src/docs.rs.
- Inferred: crates/generate/ess-gen/src/asyncapi.rs.
- Inferred: crates/generate/ess-gen/src/graph.rs.
- Inferred: crates/verify/ess-conformance/src/runner.rs.
- Inferred: crates/verify/ess-conformance/src/go/runtime.go.
- Inferred: docs/design/periodic-binding-triggers.md.


## Approved implementation and concurrency checkpoint

The operator explicitly requested concurrent implementation of all eight gaps. The concrete periodic proposal at local-evidence:ess-evolution-20260910/priority-wave/periodic/design-proposal.md is admitted: typed lifetime-context and per-occurrence-read host authority, session-local fixed-rate first-after-period serial poll, one pending occurrence/drop excess, receipt-time eligibility, acknowledged closure, and controlled-time complete-window causal evidence. Actual required read results remain required; session identity alone does not fill the command. No fabricated event, lossless tick promise, production lateness SLA or scheduler service.

Implementor design_adversary owns wt-36e6402a4826, branch impl/gap-periodic-trigger, base9267dd3d, lease ess-periodic-01a089ee. Shared scope is split by periodic-only cause/host mapping/step branches; list selection owns its separate mapping branch, enum owns witness candidate filtering, root owns aggregate imports, source/suite selectors, public docs, generated artifacts and publication. Exact symbol ranges and returned hunk headers are retained in the implementation brief before edits. Root will inspect merge-tree output and resolve shared matches before integration.

All new vocabulary in this single unreleased PR shares conditional source3, ordinary suite6/coverage7 and target-failure3; report2 remains required. Existing event/source bytes remain unchanged when no new feature is used. Positive controlled Rust/Go execution and mutation verifiers are mandatory; an all-Unsupported result is not implementation completion. Build slot limited to two concurrent jobs under the assigned unique disk-backed target; no full local or unchanged repeated gate.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 65463c5b7b7bbfb52a2b38cb06c64b75fc96f5c7a92b98388c89a793364de8f5, retained as local-evidence:runtime-gaps/publication-replay/snapshots/65463c5b7b7bbfb52a2b38cb06c64b75fc96f5c7a92b98388c89a793364de8f5.md. Source creation recorded at 2026-09-11T00:24:38Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
