---
format: aep.planning-md/1
id: story:error-wire-codes
kind: story
status: implemented
title: Declare error wire codes without losing semantic identity
tags:
- priority-high
relations:
- decomposes: task:runtime-adopter-gaps-9-13
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/error_wire.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/error_wire.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/error_wire.rs
- confidence: cited
  path: docs/design/error-wire-codes.md
revision: 5
---
## Outcome

Measured: 117 expect_error steps need declared codes; aliases include eleven semantic refusals sharing one code. Add optional naming.wire through raw, admitted and resolved errors; native HTTP uses explicit code with the existing qualified-name fallback. Preserve distinct semantic declarations when codes alias; do not guess message discriminators. Include source/compiler/native/diff witnesses, legacy-byte preservation and explicit format admission. Scoper read command.rs:2242,2044, ir.rs:862, resolve.rs:1463 and HTTP emitters. Scope split with response payload work is by ErrorSpec/RawErrorSpec/ResolvedError/error lowering symbols only; no CommandSpec or Outcome payload edits. Root owns shared format selectors and generated outputs.

## Acceptance

Address gap 9 of task:runtime-adopter-gaps-9-13 with executable evidence and actual adoption instructions. Use generic public examples; private consumer names stay in private evidence. No full local gate or unchanged passing test reruns.

## Scope

Confidence: high for cited existing symbols; new test paths are inferred.

- crates/specify/ess-domain/src/command.rs (inferred implementation surface from read-only intake).
- crates/specify/ess-compiler/src/ir.rs (inferred implementation surface from read-only intake).
- crates/specify/ess-compiler/src/resolve.rs (inferred implementation surface from read-only intake).
- crates/generate/ess-synth/src/rust/http.rs (inferred implementation surface from read-only intake).
- crates/generate/ess-synth/src/go/http.rs (inferred implementation surface from read-only intake).
- crates/verify/ess-diff/src/change.rs (inferred implementation surface from read-only intake).
- crates/verify/ess-diff/src/diff.rs (inferred implementation surface from read-only intake).

## Reviewed implementation

Source-only candidate f12afa8a8ce475c14d1c8e7f7c5222ba71da920f passed the first independent adversary review. It preserves semantic error identity while native HTTP emits the declared wire code or qualified-name fallback. The focused domain/compiler/native/diff witnesses and strict scoped lint are retained with that candidate; native emission witnesses inspect generated source and do not claim a generated service execution.

Root integrated the reviewed source after a clean merge-tree preflight. Central coordination assigns source ess/4 and delta ess-diff/4, alongside typed response contracts. Updated delta refusal coverage includes prior versions 1 through 3. Final combined routing checks and remote publication remain pending; this story stays active until those results are attached.

## Final central evidence

After source integration, three focused error delta tests passed with minimum ess-diff/4 and explicit refusal under versions1–3. The two response delta tests passed beside them. Strict scoped Clippy passed after combining equal format-selection arms without changing their meaning. Source4 admission is reachable through the normal specification validator; generated document schema has been regenerated through cargo xtask schema. The website build passed. The earlier native/compiler/legacy-preservation witnesses and first independent green review remain attached; no unchanged full suite was rerun.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 3dba46a2bd16a33d2fa7fa32283dd2d2f72284751aa221acbb4e7cc6fc7c94a2, retained as local-evidence:runtime-gaps/publication-replay/snapshots/3dba46a2bd16a33d2fa7fa32283dd2d2f72284751aa221acbb4e7cc6fc7c94a2.md. Source creation recorded at 2026-09-11T04:57:59Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
