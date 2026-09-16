---
format: aep.planning-md/1
id: story:callable-live-evaluator
kind: story
status: active
title: Expose context-aware native Go execution with truthful observation evidence
relations:
- decomposes: epic:compact-live-scenarios
- depends_on: story:compact-live-authoring
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance
revision: 5
---
## Acceptance
Expose a context-aware per-scenario execution session with fixed admitted inventory and native report finalization; retain Run(testing.T, factory) as a wrapper using the same evaluator. Compare event payloads and shapes, not names alone. Windows observe every occurrence and retain completeness/error information while commands run. Cancellation reaches target I/O. Clean up partial setup and retain cleanup errors. No unavailable selected scenario or missing receipt can qualify. Rust/Go share fixture expectations and report meanings.

## Scope
Cited: go/runtime.go Run and eventuallyEvent, target.rs, runner.rs run_scenario, report.rs, coverage.rs. Inferred: context-capable target adapter/session API and typed observed-trace window requests. Existing Go targets remain source-compatible through the wrapper.

## Tests
Wrong-resource matching, delayed match, malformed payload, duplicate/reordered occurrences, truncated/empty windows, one transient violation followed by recovery, cancellation, partial setup, cleanup failure, incomplete selection, unknown capability and report delivery failure; unchanged legacy projections and full task check.

## Metrics continuation

Extend both native evaluators for the versioned metrics vocabulary: event capture with required presence, exact signed-64-bit bounds/checked offsets, and a quiet baseline beginning at a fresh native completeness fence after setup. Every scoped frame remains visible during stable windows and blocking sync stimuli. Counterexamples cover slow first status, absent/malformed integers, nonzero baselines, quiet resets, overflow, transient zero, source gaps, cancellation and cleanup errors. Complete consumer coverage without rewriting sealed baseline eligibility.
