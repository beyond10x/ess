---
format: aep.planning-md/3
id: review-result:consumer-one-time-wasm-resources
kind: review-result
status: active
title: Independent WASM resource-control extension review
relations:
- reviews: story:feature-request-389
revision: 1
---
approve

Independent Go-port-owner follow-up source review of crates/generate/ess-synth/tests/one_time_wasm.rs, SHA256 519bef005ccc633cdc90c38dc5e4c9b83a257314b85460020dd7a3c359f11943, diff against 827e16e63. Reviewer own executions: 0. Findings: none.

The nine additions instantiate real ResourceService inside WASM and run the native Runner through the generated browser transport. Manifest values are comparison expectations only. Actual callback traces and returned plaintexts are gathered after execution. Original nineteen controls are retained; all cases check exact statuses, all five counts, total, scenario identities, required codes and redaction. There is no precomputed report standing in for callbacks.

Coordinator evidence: corrected resource-host test builds and drives all28 suite requests but remains deliberately red0/1 at healthy Unsupported versus Passed, because the native observer implementation is not integrated. This is not a claim of28 passing target callback traces. Scoped strict Clippy passes. Logs: one-time-wasm-resources-red-2.log and one-time-wasm-resources-clippy.log. The first resource-host probe failed only because an unused shared expected-status helper needed the same test-module dead-code annotation as the existing shared Service; it is not product failure evidence. Exact feature green remains required after native integration.

```findings
[]
```
