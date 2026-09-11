---
format: aep.planning-md/1
id: story:refuse-misparsed-predicate-disjunctions
kind: story
status: implemented
title: Refuse trailing Boolean syntax in quoted predicate comparisons
tags:
- priority-high
relations:
- decomposes: task:runtime-adopter-gaps-9-13
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/adversary_quoted_roundtrip.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/quoted_predicates.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/quoted_predicate_format.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/adversary-quoted-specials.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/quoted-predicates.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/quoted-predicates.mjs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/quoted-roundtrip.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/quoted-roundtrip.mjs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/quoted-special-boundaries.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/quoted_predicate_versions.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/quoted_predicates.rs
revision: 6
---
## Outcome

Measured: to == "" or text == "" becomes a single wrong literal and synthesizes misleading inputs. Reject trailing tokens after a closed quoted operand with guidance to structured any/all/not. Preserve quoted Boolean words, single/double quotes, whitespace, escapes, structured predicates and supported prefix not. Rust source and persisted Go runtime admission must agree. No new infix expression language. Scoper read predicate.rs:1055,1097 and facts.rs:643,665; Go paths inferred from runtime.go:3732 and predicate.go:320,364.

## Acceptance

Address gap 10 of task:runtime-adopter-gaps-9-13 with executable evidence and actual adoption instructions. Use generic public examples; private consumer names stay in private evidence. No full local gate or unchanged passing test reruns.

## Scope

Confidence: high for cited existing symbols; new test paths are inferred.

- crates/specify/ess-primitives/src/predicate.rs (inferred implementation surface from read-only intake).
- crates/specify/ess-primitives/src/facts.rs (inferred implementation surface from read-only intake).
- crates/verify/ess-conformance/src/go/predicate.go (inferred implementation surface from read-only intake).
- crates/verify/ess-conformance/src/go/runtime.go (inferred implementation surface from read-only intake).

## Final correction and integration

Source-only candidate 09a16599b1e85532869fbcb837dd37cb9c5f9ad6 and its reviewed incremental special-number correction are integrated. The first attack found canonical quoted-text round-trip loss; the second found newly introduced Go structured-scalar NaN interpretation. The first is resolved, none carried, one new finding was corrected. Full typed findings comparison is retained with review2/findings-ledger.json.

The coordinator read the second correction, confirmed the original deciding assertion and prior wrapper assertions remain, and executed the updated actual Go emitter wrapper. The integrated predicate/version/selection command passed all eight outer tests; it includes the original second-adversary case, all 27 decimal/text boundaries, 13 exact cross-reader literals, fresh and old header behavior, and the primitive selection checks. Strict scoped Clippy passed. No third attack or full local gate was run. Source-only test evidence is distinct from final remote delivery, which remains the wave's publication step.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 8affe9a37ebe474155f7625146df1f2a53cf2cfb1ee103003c0c000717cdb319, retained as local-evidence:runtime-gaps/publication-replay/snapshots/8affe9a37ebe474155f7625146df1f2a53cf2cfb1ee103003c0c000717cdb319.md. Source creation recorded at 2026-09-11T04:58:01Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
