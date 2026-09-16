---
format: aep.planning-md/1
id: story:compact-live-authoring
kind: story
status: active
title: Compile typed recipes, composed contracts and observed timelines
relations:
- decomposes: epic:compact-live-scenarios
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/specify/ess-composition
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-diff/src/impact.rs
- confidence: cited
  path: docs/design/compact-live-scenarios.md
revision: 7
---
## Acceptance
Admit a closed next-version compact scenario surface with given/when/then, acyclic typed recipes, captured real response/event identities, event payload matchers, observed anchors, ordering and bounded windows. Resolve through original pinned component EssIrs, including transitive types, rather than name-only composition exports. Retain every contract/source/recipe digest and source location in the admitted suite. Old readers refuse new semantics before target invocation; legacy bytes remain stable. No runtime snippets or inferred model declarations.

## Scope
Cited: authored.rs, scenario.rs, composition/lib.rs, edge CLI input discovery. Inferred: concrete typed compact authoring and window modules plus fixtures. One owner edits the shared suite admission/coverage interfaces before evaluator work consumes them.

## Tests
Unknown/mistyped references, cycles, missing/duplicate captures, stale anchors, pins, namespace ambiguity, interval boundaries, deterministic expansion, old-reader refusal and exact-input selection provenance.

## Metrics continuation

Implement the approved next surface ess-scenario/4 and ordinary/inventory suite formats 12/13. Preserve previous formats and canonical bytes. Add captures from named actual event occurrences with explicit optional-field presence, signed-64-bit integer bounds and checked baseline offsets, and native two-second quiet-baseline observations fenced after setup. Assert the first selected live status; bounds must not filter away an earlier violation. Keep original model owner, field shape and occurrence identity. Document constructs before code and qualify both Rust and generated Go. Existing consumer-coverage blocker remains until finite classifications and executed evidence pass.

## Metrics implementation evidence

Implemented source /4 with typed actual event captures, first-occurrence integer bounds, exact signed-64-bit offsets, fresh fenced quiet baselines and intermediate do:stable. Lowers to ordinary suite/12 and inventory suite/13; source /3 and older suites reject new vocabulary. Identical admitted inputs and cases run through Rust and emitted Go, including nonzero and >2^53 baselines, late quiet reset, slow-first replay, missing/null/fractional/out-of-range integers, overflow, transient zero, gap, truncation, cancellation and cleanup failure. All ess-conformance tests and shared Go-session cases passed; the current full task check is still running. Consumer coverage is not yet cleared and no real service qualification is claimed.
