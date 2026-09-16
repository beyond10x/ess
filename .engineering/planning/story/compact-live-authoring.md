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
revision: 5
---
## Acceptance
Admit a closed next-version compact scenario surface with given/when/then, acyclic typed recipes, captured real response/event identities, event payload matchers, observed anchors, ordering and bounded windows. Resolve through original pinned component EssIrs, including transitive types, rather than name-only composition exports. Retain every contract/source/recipe digest and source location in the admitted suite. Old readers refuse new semantics before target invocation; legacy bytes remain stable. No runtime snippets or inferred model declarations.

## Scope
Cited: authored.rs, scenario.rs, composition/lib.rs, edge CLI input discovery. Inferred: concrete typed compact authoring and window modules plus fixtures. One owner edits the shared suite admission/coverage interfaces before evaluator work consumes them.

## Tests
Unknown/mistyped references, cycles, missing/duplicate captures, stale anchors, pins, namespace ambiguity, interval boundaries, deterministic expansion, old-reader refusal and exact-input selection provenance.