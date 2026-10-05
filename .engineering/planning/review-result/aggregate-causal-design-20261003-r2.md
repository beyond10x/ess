---
format: aep.planning-md/3
id: review-result:aggregate-causal-design-20261003-r2
kind: review-result
status: active
title: Aggregate causal completion design final review
relations:
- reviews: component-design:aggregate-observation-integration-boundary
- reviews: dependency-blocker:aggregate-binding-cut-authority
revision: 1
---
approve

## Findings

No design findings remain in frozen candidate `3dfaba0eea3108103c6e46099a56f0851f91e190`.

```json
[]
```

## Review result

Both pass-1 blockers are resolved. Rust now exposes begin, execute, query, and end as default methods directly on `ConformanceTarget`, so the existing generic runner can invoke the optional capability without another bound. The lifecycle guarantees one observed execution, no fallback execution after Unsupported or receipt loss, aligned query rows, teardown on every post-begin exit, and no capability call for empty relevance.

The production boundary now owns every actual receipt seam: generated Rust/Go dispatch records occurrence, child, attempt, retry, result, and escalation facts at their producing calls; a shared command transaction coordinator stages writes/events and allocates one effect position; the causal adapter seals roots and captures the inventory and immutable typed rows under that coordinator; native execution records at its real transitions; and WASM/full-browser adapters forward the same installed generated session rather than a shadow service.

The full design remains conservative about source authority, transitive relevance, retry gaps, uncertain remote effects, immutable snapshots, correlation isolation, exact numbers, bounded resources, compatibility, and private observation state. New suite/38 and inventory/39 vocabulary is rejected by old readers, while unchanged old suites retain their format. The versioned observation envelope is separate from released command/event results and requires closed lossless admission.

The acceptance matrix is fault-sensitive across native, generated Rust/Go services, generated Go/TypeScript runtimes, WASM, and the full browser. It requires faults at actual producer and synchronization seams, including omitted descendants, retry gaps, early snapshots, wrong receipts, effect ordering, unknown commits, disclosure, log draining, queued retries, transaction races, and direct-store bypass.

The current integration copy is byte-identical to the frozen design. The dependency blocker remains open until actual independent adapter execution; this approval covers the design and feasibility only. This was the second and final read-only pass, with no compiler or repository mutation.
