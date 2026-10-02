---
format: aep.planning-md/3
id: review-result:consumer-one-time-wasm-harness
kind: review-result
status: active
title: Independent real WASM disclosure harness review
relations:
- reviews: story:feature-request-389
revision: 1
---
approve

Independent Go-port-owner source review of crates/generate/ess-synth/tests/one_time_wasm.rs, SHA256 d9a3aac6d9c038b60652e40dfebcf1e4ee311efb8994dc45fb5f12e03448a715. Reviewer own executions: 0. Findings: none.

The test compiles the actual native Runner and shared Service into wasm32, dispatches all 19 frozen suite files through the actual emitted bridge.js, and compares all five counts, total, exact scenario identities, callback traces, required diagnostic codes and plaintext redaction. Captures are checked inside WASM and never returned by the harness. If redaction fails, the module returns only a static false flag. The test accurately limits its claim to the WASM runner harness, rather than the billing lab or a synthesized marked application.

Coordinator author evidence: one-time-wasm-red-2.log reports 0 passed/1 failed because healthy status is Unsupported instead of Passed on the pre-observer implementation, after the actual WASM build and browser transport executed. Initial one-time-wasm-red.log hit the host redaction assertion during identity-error before returning aggregate results; it is retained as an initial probe, not the deciding status regression. Scoped strict Clippy passes after correcting path-literal formatting (one-time-wasm-clippy-2.log). No successful feature execution is claimed yet; the exact test must pass after native observer integration.

```findings
[]
```
