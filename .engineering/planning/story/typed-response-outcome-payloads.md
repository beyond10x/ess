---
format: aep.planning-md/1
id: story:typed-response-outcome-payloads
kind: story
status: implemented
title: Map typed command responses into complete emitted event payloads
tags:
- priority-high
relations:
- decomposes: task:runtime-adopter-gaps-9-13
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/src/schema.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/refusal.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/fixtures/response-native.go
- confidence: cited
  path: crates/generate/ess-synth/tests/fixtures/response-native.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/response_payload.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/binary64.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/graph.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/response_payload.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/response.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/response.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/selection.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_response_union.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/response-payload.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/response-runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/response-union.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_payload.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_union.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/response_payload.rs
- confidence: cited
  path: docs/design/typed-response-outcome-payloads.md
revision: 5
---
## Outcome

Measured:21 emitted events have response fields no outcome can declare,15 shape failures; upcoming36-operation retrofit needs this before its first domain area lands. Implement a bounded typed command response declaration/source, observation of actual returned response and comparison of mapped event values to it. Add explicit refusal of incomplete emitted payloads with a declared implementation-generated ownership escape; external events with zero emitters remain valid. Preserve legacy meanings including existing response.* literals via coordinated format admission. No arbitrary JSON bags or invented input fields. Design must specify response typing, native realization, persisted suite and old-reader refusal before code. Scoper read PayloadSource command.rs:624-655, validate_payloads:1711, SemanticCommandResult target.rs:456-488. Shared files split by CommandSpec/Outcome/PayloadSource/response lowering only, no ErrorSpec/ResolvedError edits. Root owns format selectors/generated artifacts. Other required test/native surfaces must be confirmed by implementor before editing.

## Acceptance

Address gap 13 of task:runtime-adopter-gaps-9-13 with executable evidence and actual adoption instructions. Use generic public examples; private consumer names stay in private evidence. No full local gate or unchanged passing test reruns.

## Scope

Confidence: high for cited existing symbols; new test paths are inferred.

- crates/specify/ess-domain/src/command.rs (inferred implementation surface from read-only intake).
- crates/specify/ess-compiler/src/ir.rs (inferred implementation surface from read-only intake).
- crates/specify/ess-compiler/src/resolve.rs (inferred implementation surface from read-only intake).
- crates/verify/ess-conformance/src/target.rs (inferred implementation surface from read-only intake).
- crates/verify/ess-conformance/src/go/runtime.go (inferred implementation surface from read-only intake).

## Coordinator design decision

Accepted design:typed-response-outcome-payloads on2026-09-11. Source4 introduces typed command response fields, closed {response: field}/{generated: true} payload spellings and complete emitted payload ownership. Old response.* strings remain literal. Suite8 ordinary/suite9 coverage carry actual response observations; report2 remains. Explicit generated ownership still validates payload shape; zero-emitter events remain allowed. Native response-bearing outcome variants include command-specific typed response only where response sources require it. Bounded maps required because measured adopter responses use them; unsupported constrained response semantics must be explicit rather than claimed checked.

## Final implementation and measured adoption

Source-only candidate 47d4241057014c370be09f0ca751ad085b22035a and the response-only tagged-union correction are integrated. The first adversary found an executable closed-union false acceptance; its unchanged semantic assertion now passes, new Rust/Go union boundaries pass, and the second/final review is green. Native helpers compiled and executed valid and mutated responses. The unit report records40 distinct focused cases before the bounded union correction; no unchanged native cases were rerun.

Coordinator integration review separately found direct typed suite writers omitted response admission. A new direct-construction case failed for old suite4, then passed after the normal writer/runner admission path enforced suite8 and validated every response observation. Malformed direct mappings also refuse; no assertion was weakened. Integrated error/response delta tests5/5 and strict scoped Clippy passed. Generated document schema and website validation passed.

Actual isolated adopter source4 validates all10 source files and44 authored scenarios synthesize. Source-grounded response records fill26 missing fields across21 event declarations. ClearWrapUp's item is Optional because the real backend returns null when no active wrap-up exists. Two actual HTTP adapter tests passed in0.006s. The first Go subtest-only run executed the case successfully but correctly refused report2 because its embedded full suite was not completely run. The correction used the existing CLI selection mechanism over a coverage9 suite, retaining complete parent bytes; the selected actual scenario then passed in0.406s. Report2 records1total,1passed,0failed,0skipped and overall inconclusive, with340outside scenarios and2 known synthesis refusals. This is bounded successful adoption, not full target conformance.

Exact report SHA25616b2c17e0e703d821cbfacf845d73a9590e67e6746c872fc07e8ced40b32a48b; selected input SHA25647b62d01008d8b6df3f62fea04d4c250e72feced547e6ca019f15b145bf9aa5b. Private migration/source witnesses remain outside public source. The currently installed AEP report reader refuses the new coverage9 input with UnsupportedSuiteVersion, coverage requires suite/5. Retain these bytes and generic test evidence; do not claim automated report import or advance downstream release pins. This reader compatibility remains a consumer follow-up, not an ESS dependency.

Location correction for the first immutable review: the implicated Rust union branch in that checkpoint is selection.rs:597, rather than the approximate481 in its typed finding. The report's symbol-level description and deciding regression identify the unchanged defect; its original evidence remains immutable.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 f0d8a0fc9482f1570da4fa501b791e3de84ce8bff509d943553f59c744459e62, retained as local-evidence:runtime-gaps/publication-replay/snapshots/f0d8a0fc9482f1570da4fa501b791e3de84ce8bff509d943553f59c744459e62.md. Source creation recorded at 2026-09-11T04:58:07Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
